//! OWL 2 RL materialization (the forward-chainable OWL profile).
//!
//! Implements a correct, useful subset of the W3C OWL 2 RL/RDF rules
//! (<https://www.w3.org/TR/owl2-profiles/#Reasoning_in_OWL_2_RL_and_RDF_Graphs>) over the
//! same dictionary-encoded forward-chaining fixpoint as RDFS — RL rules are datalog-style
//! joins over integer ids. The RDFS rules (rdfs2/3/5/7/9/11) are included (RL subsumes them).
//!
//! | group | rule | premise ⊢ conclusion |
//! |---|---|---|
//! | equality | eq-sym   | `(x sameAs y)` ⊢ `(y sameAs x)` |
//! |          | eq-trans | `(x sameAs y),(y sameAs z)` ⊢ `(x sameAs z)` |
//! |          | eq-rep-s/p/o | `(s sameAs s'),(s p o)` ⊢ `(s' p o)` (and for p, o) |
//! | property | prp-inv1/2 | `(p inverseOf q),(x p y)` ⊢ `(y q x)` (and converse) |
//! |          | prp-symp | `(p a SymmetricProperty),(x p y)` ⊢ `(y p x)` |
//! |          | prp-trp  | `(p a TransitiveProperty),(x p y),(y p z)` ⊢ `(x p z)` |
//! |          | prp-eqp1/2 | `(p equivalentProperty q),(x p y)` ⊢ `(x q y)` (and converse) |
//! |          | prp-fp   | `(p a FunctionalProperty),(x p y1),(x p y2)` ⊢ `(y1 sameAs y2)` |
//! |          | prp-ifp  | `(p a InverseFunctionalProperty),(x1 p y),(x2 p y)` ⊢ `(x1 sameAs x2)` |
//! | class    | cax-eqc1/2 | `(c equivalentClass d),(x a c)` ⊢ `(x a d)` (and converse) |
//!
//! Coverage now includes the class-expression rules (cls-* for someValuesFrom/allValuesFrom/
//! hasValue/intersection/union via `owl:Restriction` + RDF-list decoding), `prp-spo2`
//! (propertyChainAxiom), cardinality/`hasKey`, the schema (scm-*) rules, and the consistency
//! clashes (see [`inconsistencies`]). `owl:sameAs` is handled by union-find ENTITY REWRITING
//! (reason over canonical representatives, expand at the end) rather than the quadratic eq-rep
//! substitution. The fixpoint is SEMI-NAIVE: the recursive rules (RDFS transitivity + prp-trp)
//! derive only from the previous round's new facts against incrementally-maintained indexes.

use crate::{RdfsIndex, Schema, Vocab};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

const OWL: &str = "http://www.w3.org/2002/07/owl#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

struct Owl {
    same_as: Id,
    inverse_of: Id,
    symmetric: Id,       // owl:SymmetricProperty
    transitive: Id,      // owl:TransitiveProperty
    equiv_prop: Id,      // owl:equivalentProperty
    equiv_class: Id,     // owl:equivalentClass
    functional: Id,      // owl:FunctionalProperty
    inv_functional: Id,  // owl:InverseFunctionalProperty
    property_chain: Id,  // owl:propertyChainAxiom
    on_property: Id,     // owl:onProperty
    some_values: Id,     // owl:someValuesFrom
    all_values: Id,      // owl:allValuesFrom
    has_value: Id,       // owl:hasValue
    intersection: Id,    // owl:intersectionOf
    union: Id,           // owl:unionOf
    thing: Id,           // owl:Thing
    has_key: Id,         // owl:hasKey
    max_cardinality: Id, // owl:maxCardinality
    max_qual_card: Id,   // owl:maxQualifiedCardinality
    on_class: Id,        // owl:onClass
    rdf_first: Id,
    rdf_rest: Id,
    rdf_nil: Id,
}

impl Owl {
    fn intern(dict: &mut Dict) -> Owl {
        let mut i = |frag: &str| dict.intern_iri(&format!("{OWL}{frag}"));
        Owl {
            same_as: i("sameAs"),
            inverse_of: i("inverseOf"),
            symmetric: i("SymmetricProperty"),
            transitive: i("TransitiveProperty"),
            equiv_prop: i("equivalentProperty"),
            equiv_class: i("equivalentClass"),
            functional: i("FunctionalProperty"),
            inv_functional: i("InverseFunctionalProperty"),
            property_chain: i("propertyChainAxiom"),
            on_property: i("onProperty"),
            some_values: i("someValuesFrom"),
            all_values: i("allValuesFrom"),
            has_value: i("hasValue"),
            intersection: i("intersectionOf"),
            union: i("unionOf"),
            thing: i("Thing"),
            has_key: i("hasKey"),
            max_cardinality: i("maxCardinality"),
            max_qual_card: i("maxQualifiedCardinality"),
            on_class: i("onClass"),
            rdf_first: dict.intern_iri(&format!("{RDF}first")),
            rdf_rest: dict.intern_iri(&format!("{RDF}rest")),
            rdf_nil: dict.intern_iri(&format!("{RDF}nil")),
        }
    }
}

/// The integer value of a literal id (inline ints are direct; else parse the lexical form).
/// Used for `owl:maxCardinality` thresholds.
fn lit_int(dict: &Dict, id: Id) -> Option<i64> {
    if sparq_core::dict::is_inline(id) {
        return Some((id - sparq_core::dict::INLINE_BASE) as i64);
    }
    match dict.term(id) {
        oxrdf::Term::Literal(l) => l.value().parse().ok(),
        _ => None,
    }
}

/// Decode every RDF list (`rdf:first`/`rdf:rest`/`rdf:nil`) reachable in `all` into
/// `head node -> [members]`. Used for `propertyChainAxiom`, `intersectionOf`, `unionOf`.
fn decode_lists(all: &FxHashSet<[Id; 3]>, o: &Owl) -> FxHashMap<Id, Vec<Id>> {
    let mut first: FxHashMap<Id, Id> = FxHashMap::default();
    let mut rest: FxHashMap<Id, Id> = FxHashMap::default();
    for &[s, p, obj] in all {
        if p == o.rdf_first {
            first.insert(s, obj);
        } else if p == o.rdf_rest {
            rest.insert(s, obj);
        }
    }
    let mut lists: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    for (&head, _) in first.iter() {
        let mut members = Vec::new();
        let mut cur = head;
        for _ in 0..first.len() + 1 {
            match first.get(&cur) {
                Some(&m) => members.push(m),
                None => break,
            }
            match rest.get(&cur) {
                Some(&n) if n != o.rdf_nil => cur = n,
                _ => break,
            }
        }
        lists.insert(head, members);
    }
    lists
}

/// Union-find over term ids for owl:sameAs ENTITY REWRITING (RDFox's approach). Instead of the
/// quadratic eq-rep substitution rule (copy every triple between every pair of equal terms,
/// re-derived each round), equal individuals are merged to a canonical representative (the
/// smallest id) and the whole closure is computed over representatives. The full eq-rep
/// expansion + the sameAs relation are emitted ONCE at the end. Large speed/memory win on
/// equality-heavy data; ~free when there are no equalities. (RDFox: up to 7.8× memory / 31×
/// time / 45–85× fewer derivations.)
#[derive(Default)]
struct UnionFind {
    parent: FxHashMap<Id, Id>,
}
impl UnionFind {
    fn find(&mut self, x: Id) -> Id {
        let mut root = x;
        while let Some(&p) = self.parent.get(&root) {
            if p == root {
                break;
            }
            root = p;
        }
        // path compression
        let mut cur = x;
        while cur != root {
            let nxt = self.parent[&cur];
            self.parent.insert(cur, root);
            cur = nxt;
        }
        root
    }
    /// Merge `a` and `b` (canonical rep = the smaller id). Returns true if newly merged.
    fn union(&mut self, a: Id, b: Id) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        let (lo, hi) = (ra.min(rb), ra.max(rb));
        self.parent.insert(hi, lo);
        self.parent.entry(lo).or_insert(lo);
        true
    }
    /// rep -> all member ids, for every non-singleton equivalence class.
    fn classes(&mut self) -> FxHashMap<Id, Vec<Id>> {
        let ids: Vec<Id> = self.parent.keys().copied().collect();
        let mut m: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        for id in ids {
            let r = self.find(id);
            m.entry(r).or_default().push(id);
        }
        m
    }
}

/// Re-key every triple to its sameAs representative, dropping sameAs triples (the union-find
/// holds equality). Skips reflexive results.
fn canonicalize(all: &FxHashSet<[Id; 3]>, uf: &mut UnionFind, same_as: Id) -> FxHashSet<[Id; 3]> {
    let mut out = FxHashSet::default();
    for &[s, p, o] in all {
        if p == same_as {
            continue;
        }
        out.insert([uf.find(s), uf.find(p), uf.find(o)]);
    }
    out
}

/// (Re)build the transitive/functional/IFP adjacency over `all`: `out` maps p -> (subj -> [obj])
/// and `inc` maps p -> (obj -> [subj]), for the predicates in `need`. Used to seed the index
/// before the fixpoint and to rebuild it after a sameAs merge rewrites ids.
fn build_adjacency(
    all: &FxHashSet<[Id; 3]>,
    need: &FxHashSet<Id>,
    out: &mut FxHashMap<Id, FxHashMap<Id, Vec<Id>>>,
    inc: &mut FxHashMap<Id, FxHashMap<Id, Vec<Id>>>,
) {
    out.clear();
    inc.clear();
    if need.is_empty() {
        return;
    }
    for &[s, p, obj] in all {
        if need.contains(&p) {
            out.entry(p).or_default().entry(s).or_default().push(obj);
            inc.entry(p).or_default().entry(obj).or_default().push(s);
        }
    }
}

/// True if the ontology uses any OWL-specific feature (so it needs the OWL fixpoint). When false,
/// the OWL-RL closure is exactly RDFS + scm-dom/rng, computed by the fast single-pass path.
fn owl_uses_features(triples: &[[Id; 3]], v: &Vocab, o: &Owl) -> bool {
    let preds: FxHashSet<Id> = [
        o.same_as,
        o.inverse_of,
        o.equiv_prop,
        o.equiv_class,
        o.property_chain,
        o.on_property,
        o.some_values,
        o.all_values,
        o.has_value,
        o.intersection,
        o.union,
        o.has_key,
        o.max_cardinality,
        o.max_qual_card,
        o.on_class,
    ]
    .into_iter()
    .collect();
    let types: FxHashSet<Id> = [o.symmetric, o.transitive, o.functional, o.inv_functional].into_iter().collect();
    triples.iter().any(|&[_, p, ob]| preds.contains(&p) || (p == v.ty && types.contains(&ob)))
}

/// If the ontology uses ONLY the monotone / non-recursive OWL-RL subset (equivalentClass,
/// equivalentProperty, inverseOf, SymmetricProperty) — and NONE of the recursive features that
/// require the fixpoint (owl:sameAs, Transitive/Functional/InverseFunctionalProperty,
/// propertyChainAxiom, the restriction/cardinality/hasKey/intersection/union family) — return a
/// [`MonoOwl`](crate::rdfs::MonoOwl) descriptor so the closure runs in the single-pass sweep.
/// Returns `None` (→ fixpoint) the moment any recursive feature is present.
fn monotone_only(triples: &[[Id; 3]], v: &Vocab, o: &Owl) -> Option<crate::rdfs::MonoOwl> {
    // Predicates / type-objects whose presence forces the recursive fixpoint.
    let recursive_preds: FxHashSet<Id> = [
        o.same_as,
        o.property_chain,
        o.on_property,
        o.some_values,
        o.all_values,
        o.has_value,
        o.intersection,
        o.union,
        o.has_key,
        o.max_cardinality,
        o.max_qual_card,
        o.on_class,
    ]
    .into_iter()
    .collect();
    let recursive_types: FxHashSet<Id> =
        [o.transitive, o.functional, o.inv_functional].into_iter().collect();

    let mut mono = crate::rdfs::MonoOwl::default();
    for &[s, p, ob] in triples {
        if recursive_preds.contains(&p) {
            return None;
        }
        if p == o.inverse_of {
            mono.inverse.entry(s).or_default().push(ob);
            mono.inverse.entry(ob).or_default().push(s);
        } else if p == o.equiv_class {
            mono.equiv_class.push((s, ob));
        } else if p == o.equiv_prop {
            mono.equiv_prop.push((s, ob));
        } else if p == v.ty {
            if recursive_types.contains(&ob) {
                return None;
            }
            if ob == o.symmetric {
                mono.symmetric.insert(s);
            }
        }
    }
    Some(mono)
}

/// Expand `triples` in place with the OWL 2 RL (+ RDFS) closure. Returns NEW triple count.
pub fn materialize_owl_rl(dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> usize {
    let v = Vocab::intern(dict);
    let o = Owl::intern(dict);

    // Fast path 1: an ontology with NO OWL-specific feature has an OWL-RL closure equal to the
    // RDFS closure plus the scm-dom/rng domain/range closure — so it runs through the single-pass
    // (parallel, no fixpoint) materializer instead of the OWL fixpoint. The common "OWL profile
    // but really just classes/properties/domains" case then reasons ~as fast as RDFS.
    if !owl_uses_features(triples, &v, &o) {
        return crate::rdfs::rdfs_closure(dict, triples, true, &crate::rdfs::MonoOwl::default());
    }

    // Fast path 2: the ontology uses ONLY the MONOTONE / non-recursive OWL-RL subset —
    // equivalentClass, equivalentProperty, inverseOf, SymmetricProperty — and NONE of the
    // recursive features (sameAs, Transitive/Functional/InverseFunctional, propertyChain,
    // restrictions, hasKey, cardinality, intersection/union). Then the OWL-RL closure is
    // saturable in the single ABox pass: equiv* fold into subClass/subProperty, and
    // inverse/symmetric are handled by the property-orientation closure (see `PropExpand`).
    // This is byte-identical to the fixpoint on this subset (asserted in tests) and skips the
    // semi-naive loop entirely.
    if let Some(mono) = monotone_only(triples, &v, &o) {
        return crate::rdfs::rdfs_closure(dict, triples, true, &mono);
    }

    let before = triples.len();
    let mut all: FxHashSet<[Id; 3]> = triples.iter().copied().collect();

    // owl:sameAs entity rewriting: seed the union-find from explicit sameAs, then reason over
    // canonical representatives (no quadratic eq-rep during the fixpoint).
    let mut uf = UnionFind::default();
    for &[s, p, obj] in &all {
        if p == o.same_as {
            uf.union(s, obj);
        }
    }
    all = canonicalize(&all, &mut uf, o.same_as);

    // SEMI-NAIVE driver for the RECURSIVE rules (RDFS transitivity rdfs5/9/11 + prp-trp): an
    // incremental index over `all` plus the previous round's newly-derived facts (`delta`).
    // The other (non-recursive) OWL rules below stay naive — they re-scan all of `all` each
    // round, which is correct because they saturate in O(1) rounds; only the recursive rules
    // need many rounds, and re-deriving the whole closure every round was the cost.
    let mut rdfs_idx = RdfsIndex::default();
    for &t in &all {
        rdfs_idx.insert(t, &v);
    }
    let mut delta: Vec<[Id; 3]> = all.iter().copied().collect();

    // The restriction / list / cardinality / key rules are defined entirely by TBox structural
    // predicates that no OWL-RL rule ever derives, so whether the ontology uses any of them is
    // fixed by the input. Detect it once; if absent we skip building those (expensive,
    // per-round, O(|all|)) indexes and running those rules entirely.
    let feature_preds: FxHashSet<Id> = [
        o.on_property,
        o.some_values,
        o.all_values,
        o.has_value,
        o.max_cardinality,
        o.max_qual_card,
        o.on_class,
        o.has_key,
        o.property_chain,
        o.intersection,
        o.union,
    ]
    .into_iter()
    .collect();
    let uses_class_features = all.iter().any(|[_, p, _]| feature_preds.contains(p));

    // The property axioms (inverse/symmetric/transitive/functional/IFP/equivalent) are TBox and
    // never derived, so build them ONCE (rebuilt only after a sameAs merge rewrites ids). The
    // transitive/functional/IFP adjacency (`out`/`inc`) is likewise maintained INCREMENTALLY —
    // delta edges are appended as they are derived — instead of rebuilt from `all` every round,
    // which was the dominant per-round cost on recursive (transitive-closure) workloads.
    let mut ax = Axioms::build(&all, &v, &o);
    let mut need: FxHashSet<Id> =
        ax.transitive.iter().chain(ax.functional.iter()).chain(ax.inv_functional.iter()).copied().collect();
    let mut out: FxHashMap<Id, FxHashMap<Id, Vec<Id>>> = FxHashMap::default();
    let mut inc: FxHashMap<Id, FxHashMap<Id, Vec<Id>>> = FxHashMap::default();
    build_adjacency(&all, &need, &mut out, &mut inc);

    loop {
        let schema = Schema::build(&all, &v);
        let mut cand: Vec<[Id; 3]> = Vec::new();

        // RDFS rules (RL includes them) — SEMI-NAIVE: derive only from `delta` against the
        // incremental index (RdfsIndex::derive fires each rule in both delta directions), never
        // re-scanning the whole closure.
        for &t in &delta {
            rdfs_idx.derive(t, &v, &mut cand);
        }

        // Property/class-equivalence rules are single-premise over assertions joined against the
        // fixed property axioms, so SEMI-NAIVE: drive from `delta` (axiom side never changes).
        for &[s, p, obj] in &delta {
            // (eq-sym / eq-rep are handled by the union-find, not as rules.)
            // --- property axioms on assertion (s p obj) ------------------------------
            if let Some(invs) = ax.inverse.get(&p) {
                cand.extend(invs.iter().map(|&q| [obj, q, s])); // prp-inv1/2
            }
            if ax.symmetric.contains(&p) {
                cand.push([obj, p, s]); // prp-symp
            }
            if let Some(eqp) = ax.equiv_prop.get(&p) {
                cand.extend(eqp.iter().map(|&q| [s, q, obj])); // prp-eqp1/2
            }
            // --- class equivalence on type assertion ---------------------------------
            if p == v.ty {
                if let Some(eqc) = ax.equiv_class.get(&obj) {
                    cand.extend(eqc.iter().map(|&d| [s, v.ty, d])); // cax-eqc1/2
                }
            }
            // --- scm-eqc/eqp: equivalence ⊢ subClassOf/subPropertyOf both ways ---------
            if p == o.equiv_class {
                cand.push([s, v.sub_class, obj]);
                cand.push([obj, v.sub_class, s]);
            } else if p == o.equiv_prop {
                cand.push([s, v.sub_prop, obj]);
                cand.push([obj, v.sub_prop, s]);
            }
        }

        // --- scm-dom1/2, scm-rng1/2: domain/range propagate UP subClassOf and DOWN
        // subPropertyOf (makes the schema-level domain/range closure explicit). ----------
        let mut subprop_inv: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        for (&p2, supers) in &schema.sub_prop {
            for &sup in supers {
                subprop_inv.entry(sup).or_default().push(p2);
            }
        }
        for (which, map) in [(v.domain, &schema.domain), (v.range, &schema.range)] {
            for (&p, classes) in map {
                for &c in classes {
                    // scm-dom1/rng1: (p domain/range c), (c subClassOf d) ⊢ (p domain/range d).
                    if let Some(ds) = schema.sub_class.get(&c) {
                        cand.extend(ds.iter().map(|&d| [p, which, d]));
                    }
                }
                // scm-dom2/rng2: (p2 subPropertyOf p), (p domain/range c) ⊢ (p2 domain/range c).
                if let Some(subs) = subprop_inv.get(&p) {
                    for &p2 in subs {
                        cand.extend(classes.iter().map(|&c| [p2, which, c]));
                    }
                }
            }
        }

        // --- joins that need an adjacency index (prp-trp / prp-fp / prp-ifp) ----------
        // `out`/`inc` are maintained incrementally (seeded before the loop, delta edges appended
        // at the bottom), so no per-round rebuild here.
        if !need.is_empty() {
            // prp-trp: (x p y),(y p z) ⊢ (x p z). SEMI-NAIVE — join only the NEW edges in
            // `delta` (each against the full adjacency), not the whole closure. The one-step
            // transitive rule has two delta directions for a new edge (x p y): as the FIRST
            // premise it extends forward through out[y]; as the SECOND premise it extends
            // backward through inc[x]. (Δ⋈full ∪ full⋈Δ; Δ⋈Δ double-counts but dedup absorbs.)
            for &[x, p, y] in &delta {
                if !ax.transitive.contains(&p) {
                    continue;
                }
                if let Some(zs) = out.get(&p).and_then(|m| m.get(&y)) {
                    cand.extend(zs.iter().map(|&z| [x, p, z]));
                }
                if let Some(ws) = inc.get(&p).and_then(|m| m.get(&x)) {
                    cand.extend(ws.iter().map(|&w| [w, p, y]));
                }
            }
            // prp-fp: functional ⊢ the two objects of one subject are sameAs.
            for &p in &ax.functional {
                if let Some(adj) = out.get(&p) {
                    for ys in adj.values() {
                        for i in 0..ys.len() {
                            for j in (i + 1)..ys.len() {
                                cand.push([ys[i], o.same_as, ys[j]]);
                            }
                        }
                    }
                }
            }
            // prp-ifp: inverse-functional ⊢ the two subjects of one object are sameAs.
            for &p in &ax.inv_functional {
                if let Some(adj) = inc.get(&p) {
                    for xs in adj.values() {
                        for i in 0..xs.len() {
                            for j in (i + 1)..xs.len() {
                                cand.push([xs[i], o.same_as, xs[j]]);
                            }
                        }
                    }
                }
            }
        }

        // --- list/restriction rules (prp-spo2, cls-svf/avf/hv, cls-int, scm-uni) ------
        // Decode RDF lists + restrictions + class lists once, plus the adjacency / type
        // indexes the rules join over. Skipped wholesale when the ontology uses none of these
        // features (the common case) — that is where the per-round O(|all|) cost was going.
        if uses_class_features {
            let lists = decode_lists(&all, &o);
            let mut on_prop: FxHashMap<Id, Id> = FxHashMap::default();
            let (mut svf, mut avf, mut hv) = (
                FxHashMap::<Id, Id>::default(),
                FxHashMap::<Id, Id>::default(),
                FxHashMap::<Id, Id>::default(),
            );
            let mut chains: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
            let (mut inters, mut unions) = (
                FxHashMap::<Id, Vec<Id>>::default(),
                FxHashMap::<Id, Vec<Id>>::default(),
            );
            let mut by_pred: FxHashMap<Id, FxHashMap<Id, Vec<Id>>> = FxHashMap::default();
            let mut type_subj: FxHashMap<Id, Vec<Id>> = FxHashMap::default(); // class -> subjects
            let mut subj_types: FxHashMap<Id, Vec<Id>> = FxHashMap::default(); // subject -> classes
            let mut max_card: FxHashMap<Id, i64> = FxHashMap::default(); // restriction -> maxCardinality
            let mut max_qcard: FxHashMap<Id, i64> = FxHashMap::default(); // restriction -> maxQualifiedCardinality
            let mut on_class: FxHashMap<Id, Id> = FxHashMap::default(); // restriction -> onClass
            let mut keys: FxHashMap<Id, Vec<Id>> = FxHashMap::default(); // class -> hasKey property list
            for &[s, p, obj] in &all {
                by_pred
                    .entry(p)
                    .or_default()
                    .entry(s)
                    .or_default()
                    .push(obj);
                if p == v.ty {
                    type_subj.entry(obj).or_default().push(s);
                    subj_types.entry(s).or_default().push(obj);
                } else if p == o.on_property {
                    on_prop.insert(s, obj);
                } else if p == o.some_values {
                    svf.insert(s, obj);
                } else if p == o.all_values {
                    avf.insert(s, obj);
                } else if p == o.has_value {
                    hv.insert(s, obj);
                } else if p == o.max_cardinality {
                    if let Some(n) = lit_int(dict, obj) {
                        max_card.insert(s, n);
                    }
                } else if p == o.max_qual_card {
                    if let Some(n) = lit_int(dict, obj) {
                        max_qcard.insert(s, n);
                    }
                } else if p == o.on_class {
                    on_class.insert(s, obj);
                } else if p == o.has_key {
                    if let Some(l) = lists.get(&obj) {
                        keys.insert(s, l.clone());
                    }
                } else if p == o.property_chain {
                    if let Some(l) = lists.get(&obj) {
                        chains.insert(s, l.clone());
                    }
                } else if p == o.intersection {
                    if let Some(l) = lists.get(&obj) {
                        inters.insert(s, l.clone());
                    }
                } else if p == o.union {
                    if let Some(l) = lists.get(&obj) {
                        unions.insert(s, l.clone());
                    }
                }
            }
            let has_type = |x: Id, c: Id| subj_types.get(&x).is_some_and(|cs| cs.contains(&c));

            // cls-maxc2 — maxCardinality 1: the (≤1) values of `p` on an x∈R are all sameAs.
            for (&r, &n) in &max_card {
                if n == 1 {
                    if let (Some(&p), Some(xs)) = (on_prop.get(&r), type_subj.get(&r)) {
                        if let Some(adj) = by_pred.get(&p) {
                            for &x in xs {
                                if let Some(ys) = adj.get(&x) {
                                    for i in 0..ys.len() {
                                        for j in (i + 1)..ys.len() {
                                            cand.push([ys[i], o.same_as, ys[j]]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // cls-maxqc3/4 — maxQualifiedCardinality 1 onClass c: the (≤1) c-typed values are sameAs.
            for (&r, &n) in &max_qcard {
                if n == 1 {
                    if let (Some(&p), Some(&c), Some(xs)) =
                        (on_prop.get(&r), on_class.get(&r), type_subj.get(&r))
                    {
                        if let Some(adj) = by_pred.get(&p) {
                            for &x in xs {
                                if let Some(ys) = adj.get(&x) {
                                    let q: Vec<Id> = ys
                                        .iter()
                                        .copied()
                                        .filter(|&y| c == o.thing || has_type(y, c))
                                        .collect();
                                    for i in 0..q.len() {
                                        for j in (i + 1)..q.len() {
                                            cand.push([q[i], o.same_as, q[j]]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // prp-key — owl:hasKey: individuals of a class agreeing on all key-property values are
            // the same. (Single-value-per-key common case: group by the key-value tuple.)
            for (&c, kprops) in &keys {
                if kprops.is_empty() {
                    continue;
                }
                if let Some(individuals) = type_subj.get(&c) {
                    let mut by_tuple: FxHashMap<Vec<Id>, Id> = FxHashMap::default();
                    for &x in individuals {
                        let mut tuple = Vec::with_capacity(kprops.len());
                        let mut complete = true;
                        for &kp in kprops {
                            match by_pred
                                .get(&kp)
                                .and_then(|adj| adj.get(&x))
                                .and_then(|vs| vs.first())
                            {
                                Some(&val) => tuple.push(val),
                                None => {
                                    complete = false;
                                    break;
                                }
                            }
                        }
                        if complete {
                            if let Some(&y) = by_tuple.get(&tuple) {
                                cand.push([x, o.same_as, y]);
                            } else {
                                by_tuple.insert(tuple, x);
                            }
                        }
                    }
                }
            }

            // prp-spo2 — property chain: (x p1 y1)…(y_{n-1} pn z) ⊢ (x p z).
            for (&p, chain) in &chains {
                if chain.is_empty() {
                    continue;
                }
                let mut paths: Vec<(Id, Id)> = Vec::new();
                if let Some(adj) = by_pred.get(&chain[0]) {
                    for (&s, os) in adj {
                        paths.extend(os.iter().map(|&z| (s, z)));
                    }
                }
                for &pi in &chain[1..] {
                    let mut next = Vec::new();
                    if let Some(adj) = by_pred.get(&pi) {
                        for &(start, mid) in &paths {
                            if let Some(os) = adj.get(&mid) {
                                next.extend(os.iter().map(|&z| (start, z)));
                            }
                        }
                    }
                    paths = next;
                    if paths.is_empty() {
                        break;
                    }
                }
                cand.extend(paths.into_iter().map(|(x, z)| [x, p, z]));
            }

            // cls-svf1 — someValuesFrom: (x p u),(u type c) ⊢ (x type R)   [c = owl:Thing ⇒ any u]
            for (&r, &c) in &svf {
                if let Some(&p) = on_prop.get(&r) {
                    if let Some(adj) = by_pred.get(&p) {
                        for (&x, us) in adj {
                            if us.iter().any(|&u| c == o.thing || has_type(u, c)) {
                                cand.push([x, v.ty, r]);
                            }
                        }
                    }
                }
            }
            // cls-avf1 — allValuesFrom: (x type R),(x p u) ⊢ (u type c)
            for (&r, &c) in &avf {
                if let (Some(&p), Some(xs)) = (on_prop.get(&r), type_subj.get(&r)) {
                    if let Some(adj) = by_pred.get(&p) {
                        for &x in xs {
                            if let Some(us) = adj.get(&x) {
                                cand.extend(us.iter().map(|&u| [u, v.ty, c]));
                            }
                        }
                    }
                }
            }
            // cls-hv1/hv2 — hasValue: (x type R) ⊢ (x p w) and (x p w) ⊢ (x type R)
            for (&r, &w) in &hv {
                if let Some(&p) = on_prop.get(&r) {
                    if let Some(xs) = type_subj.get(&r) {
                        cand.extend(xs.iter().map(|&x| [x, p, w])); // cls-hv1
                    }
                    if let Some(adj) = by_pred.get(&p) {
                        for (&x, os) in adj {
                            if os.contains(&w) {
                                cand.push([x, v.ty, r]); // cls-hv2
                            }
                        }
                    }
                }
            }
            // cls-int1/int2 — intersectionOf: x type all members ⇔ x type c.
            for (&c, members) in &inters {
                if members.is_empty() {
                    continue;
                }
                if let Some(xs) = type_subj.get(&c) {
                    for &x in xs {
                        cand.extend(members.iter().map(|&m| [x, v.ty, m])); // int2
                    }
                }
                if let Some(first) = type_subj.get(&members[0]) {
                    for &x in first {
                        if members.iter().all(|&m| has_type(x, m)) {
                            cand.push([x, v.ty, c]); // int1
                        }
                    }
                }
            }
            // scm-uni — unionOf: x type member ⊢ x type c.
            for (&c, members) in &unions {
                for &m in members {
                    if let Some(xs) = type_subj.get(&m) {
                        cand.extend(xs.iter().map(|&x| [x, v.ty, c]));
                    }
                }
            }
            // scm-int / scm-uni (schema level): intersectionOf c ⊢ c subClassOf each member;
            // unionOf c ⊢ each member subClassOf c. Makes the class hierarchy explicit (queryable),
            // complementing the type-level cls-int2/scm-uni rules above.
            for (&c, members) in &inters {
                cand.extend(members.iter().map(|&m| [c, v.sub_class, m]));
            }
            for (&c, members) in &unions {
                cand.extend(members.iter().map(|&m| [m, v.sub_class, c]));
            }
        } // uses_class_features

        let mut merged = false;
        let mut new_delta: Vec<[Id; 3]> = Vec::new();
        for t in cand {
            if t[1] == o.same_as {
                // A derived sameAs (prp-fp/ifp, cls-maxc2/maxqc, …) → merge, don't store.
                if uf.union(t[0], t[2]) {
                    merged = true;
                }
            } else {
                let c = [uf.find(t[0]), uf.find(t[1]), uf.find(t[2])];
                if all.insert(c) {
                    rdfs_idx.insert(c, &v);
                    if need.contains(&c[1]) {
                        out.entry(c[1]).or_default().entry(c[0]).or_default().push(c[2]);
                        inc.entry(c[1]).or_default().entry(c[2]).or_default().push(c[0]);
                    }
                    new_delta.push(c);
                }
            }
        }
        if merged {
            // A merge rewrites representatives across `all`; recanonicalize, rebuild the
            // incremental indexes, and run a full (naive) round next so nothing is missed. Merges
            // are bounded by the individual count, so this fallback cannot loop indefinitely.
            all = canonicalize(&all, &mut uf, o.same_as);
            ax = Axioms::build(&all, &v, &o);
            need =
                ax.transitive.iter().chain(ax.functional.iter()).chain(ax.inv_functional.iter()).copied().collect();
            build_adjacency(&all, &need, &mut out, &mut inc);
            rdfs_idx = RdfsIndex::default();
            for &t in &all {
                rdfs_idx.insert(t, &v);
            }
            delta = all.iter().copied().collect();
        } else if new_delta.is_empty() {
            break;
        } else {
            delta = new_delta;
        }
    }

    // EXPAND the owl:sameAs equivalence classes back into the full closure: every canonical
    // triple is re-emitted for each member combination, plus the full sameAs relation. (The
    // memory-efficient canonical form would need a sameAs-aware query engine; we expand so the
    // existing engine answers queries over any member correctly.)
    let classes = uf.classes();
    let mut closure: FxHashSet<[Id; 3]> = FxHashSet::default();
    for &[s, p, ob] in &all {
        let sm = classes
            .get(&s)
            .map_or(std::slice::from_ref(&s), |v| v.as_slice());
        let pm = classes
            .get(&p)
            .map_or(std::slice::from_ref(&p), |v| v.as_slice());
        let om = classes
            .get(&ob)
            .map_or(std::slice::from_ref(&ob), |v| v.as_slice());
        if sm.len() == 1 && pm.len() == 1 && om.len() == 1 {
            closure.insert([s, p, ob]); // common case: no equalities involved
        } else {
            for &s2 in sm {
                for &p2 in pm {
                    for &o2 in om {
                        closure.insert([s2, p2, o2]);
                    }
                }
            }
        }
    }
    // Emit the sameAs relation (all ordered pairs within each non-singleton class).
    for mem in classes.values() {
        for &a in mem {
            for &b in mem {
                if a != b {
                    closure.insert([a, o.same_as, b]);
                }
            }
        }
    }
    all = closure;

    let original: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
    let mut derived: Vec<[Id; 3]> = all
        .iter()
        .copied()
        .filter(|t| !original.contains(t))
        .collect();
    derived.sort_unstable();
    let mut base: Vec<[Id; 3]> = original.into_iter().collect();
    base.sort_unstable();
    triples.clear();
    triples.extend(base);
    triples.extend(derived);
    triples.len() - before
}

/// Detect OWL 2 RL inconsistencies (clashes) in a triple set — run it AFTER materialization
/// so entailed types are present. Returns human-readable clash descriptions (empty = no
/// detected inconsistency). Covers cax-dw (disjointWith), cls-com (complementOf), eq-diff
/// (sameAs ∩ differentFrom), and cls-nothing (owl:Nothing instances).
pub fn inconsistencies(dict: &Dict, triples: &[[Id; 3]]) -> Vec<String> {
    use oxrdf::{NamedNode, Term as OTerm};
    let look = |iri: String| dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(iri)));
    let ty = look(oxrdf::vocab::rdf::TYPE.as_str().to_string());
    let disjoint_with = look(format!("{OWL}disjointWith"));
    let complement_of = look(format!("{OWL}complementOf"));
    let same_as = look(format!("{OWL}sameAs"));
    let different_from = look(format!("{OWL}differentFrom"));
    let nothing = look(format!("{OWL}Nothing"));
    let on_property = look(format!("{OWL}onProperty"));
    let max_cardinality = look(format!("{OWL}maxCardinality"));
    let all: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
    let mut subj_types: FxHashMap<Id, FxHashSet<Id>> = FxHashMap::default();
    let mut same: FxHashSet<(Id, Id)> = FxHashSet::default();
    let (mut disjoint, mut complement, mut different) = (Vec::new(), Vec::new(), Vec::new());
    let mut on_prop: FxHashMap<Id, Id> = FxHashMap::default();
    let mut max_card0: FxHashSet<Id> = FxHashSet::default();
    let mut prop_subj: FxHashSet<(Id, Id)> = FxHashSet::default(); // (property, subject-with-a-value)
    for &[s, p, ob] in &all {
        if p == ty {
            subj_types.entry(s).or_default().insert(ob);
        } else if p == disjoint_with {
            disjoint.push((s, ob));
        } else if p == complement_of {
            complement.push((s, ob));
        } else if p == same_as {
            same.insert((s.min(ob), s.max(ob)));
        } else if p == different_from {
            different.push((s, ob));
        } else if p == on_property {
            on_prop.insert(s, ob);
        } else if p == max_cardinality && lit_int(dict, ob) == Some(0) {
            max_card0.insert(s);
        }
        prop_subj.insert((p, s));
    }
    let mut out = Vec::new();
    let term = |id: Id| dict.term(id).to_string();
    // cax-dw / cls-com: an individual typed by two disjoint/complementary classes.
    for (c1, c2) in disjoint.iter().chain(complement.iter()) {
        for (x, ts) in &subj_types {
            if ts.contains(c1) && ts.contains(c2) {
                out.push(format!(
                    "{} is both {} and disjoint/complement {}",
                    term(*x),
                    term(*c1),
                    term(*c2)
                ));
            }
        }
    }
    // eq-diff: x sameAs y but also differentFrom y.
    for (x, y) in &different {
        if same.contains(&((*x).min(*y), (*x).max(*y))) {
            out.push(format!(
                "{} is both sameAs and differentFrom {}",
                term(*x),
                term(*y)
            ));
        }
    }
    // cls-nothing: anything typed owl:Nothing.
    if nothing != 0 {
        for (x, ts) in &subj_types {
            if ts.contains(&nothing) {
                out.push(format!("{} is typed owl:Nothing", term(*x)));
            }
        }
    }
    // cls-maxc1: x typed a maxCardinality-0 restriction on p, yet x has a p-value.
    for &r in &max_card0 {
        if let Some(&p) = on_prop.get(&r) {
            for (x, ts) in &subj_types {
                if ts.contains(&r) && prop_subj.contains(&(p, *x)) {
                    out.push(format!(
                        "{} has a {} value but is maxCardinality 0 on it",
                        term(*x),
                        term(p)
                    ));
                }
            }
        }
    }
    out
}

/// OWL axiom maps (the TBox), rebuilt each fixpoint round.
#[derive(Default)]
struct Axioms {
    inverse: FxHashMap<Id, Vec<Id>>, // p -> inverse properties (both directions)
    equiv_prop: FxHashMap<Id, Vec<Id>>, // p -> equivalent properties (both directions)
    equiv_class: FxHashMap<Id, Vec<Id>>, // c -> equivalent classes (both directions)
    symmetric: FxHashSet<Id>,
    transitive: FxHashSet<Id>,
    functional: FxHashSet<Id>,
    inv_functional: FxHashSet<Id>,
}

impl Axioms {
    fn build(all: &FxHashSet<[Id; 3]>, v: &Vocab, o: &Owl) -> Axioms {
        let mut ax = Axioms::default();
        let bi = |m: &mut FxHashMap<Id, Vec<Id>>, a: Id, b: Id| {
            m.entry(a).or_default().push(b);
            m.entry(b).or_default().push(a);
        };
        for &[s, p, obj] in all {
            if p == o.inverse_of {
                bi(&mut ax.inverse, s, obj);
            } else if p == o.equiv_prop {
                bi(&mut ax.equiv_prop, s, obj);
            } else if p == o.equiv_class {
                bi(&mut ax.equiv_class, s, obj);
            } else if p == v.ty {
                if obj == o.symmetric {
                    ax.symmetric.insert(s);
                } else if obj == o.transitive {
                    ax.transitive.insert(s);
                } else if obj == o.functional {
                    ax.functional.insert(s);
                } else if obj == o.inv_functional {
                    ax.inv_functional.insert(s);
                }
            }
        }
        ax
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::vocab::rdf;
    use oxrdf::{NamedNode, Term};

    fn ex(dict: &mut Dict, local: &str) -> Id {
        dict.intern_iri(&format!("http://ex/{local}"))
    }
    fn owl(dict: &mut Dict, frag: &str) -> Id {
        dict.intern_iri(&format!("{OWL}{frag}"))
    }
    fn has(dict: &Dict, set: &FxHashSet<[Id; 3]>, s: &str, p: &str, o: &str) -> bool {
        let g =
            |iri: &str| dict.lookup(&Term::NamedNode(NamedNode::new_unchecked(iri.to_string())));
        let (si, pi, oi) = (g(s), g(p), g(o));
        si != 0 && pi != 0 && oi != 0 && set.contains(&[si, pi, oi])
    }

    #[test]
    fn inverse_symmetric_transitive() {
        let mut dict = Dict::new();
        let (parent, child, a, b, c) = (
            ex(&mut dict, "parentOf"),
            ex(&mut dict, "childOf"),
            ex(&mut dict, "a"),
            ex(&mut dict, "b"),
            ex(&mut dict, "c"),
        );
        let (anc, knows) = (ex(&mut dict, "ancestorOf"), ex(&mut dict, "knows"));
        let ty = dict.intern_iri(rdf::TYPE.as_str());
        let inv = owl(&mut dict, "inverseOf");
        let trans = owl(&mut dict, "TransitiveProperty");
        let sym = owl(&mut dict, "SymmetricProperty");
        let mut triples = vec![
            [parent, inv, child], // parentOf inverseOf childOf
            [anc, ty, trans],     // ancestorOf a TransitiveProperty
            [knows, ty, sym],     // knows a SymmetricProperty
            [a, parent, b],       // a parentOf b
            [a, anc, b],
            [b, anc, c],   // a anc b ; b anc c
            [a, knows, b], // a knows b
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            has(
                &dict,
                &set,
                "http://ex/b",
                "http://ex/childOf",
                "http://ex/a"
            ),
            "prp-inv"
        );
        assert!(
            has(
                &dict,
                &set,
                "http://ex/a",
                "http://ex/ancestorOf",
                "http://ex/c"
            ),
            "prp-trp"
        );
        assert!(
            has(&dict, &set, "http://ex/b", "http://ex/knows", "http://ex/a"),
            "prp-symp"
        );
    }

    #[test]
    fn sameas_and_functional() {
        // ex:hasSSN a FunctionalProperty ; a hasSSN s ; a hasSSN s2  ⊢  s sameAs s2.
        // Then eq-rep: (s p o) carries to s2.
        let mut dict = Dict::new();
        let (ssn, a, s1, s2, mark) = (
            ex(&mut dict, "hasSSN"),
            ex(&mut dict, "a"),
            ex(&mut dict, "s1"),
            ex(&mut dict, "s2"),
            ex(&mut dict, "marker"),
        );
        let ty = dict.intern_iri(rdf::TYPE.as_str());
        let func = owl(&mut dict, "FunctionalProperty");
        let p = ex(&mut dict, "p");
        let mut triples = vec![
            [ssn, ty, func],
            [a, ssn, s1],
            [a, ssn, s2],  // ⊢ s1 sameAs s2
            [s1, p, mark], // eq-rep ⊢ s2 p marker
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            has(
                &dict,
                &set,
                "http://ex/s1",
                &format!("{OWL}sameAs"),
                "http://ex/s2"
            ),
            "prp-fp ⊢ sameAs"
        );
        assert!(
            has(
                &dict,
                &set,
                "http://ex/s2",
                "http://ex/p",
                "http://ex/marker"
            ),
            "eq-rep substitution"
        );
    }

    fn rdf(dict: &mut Dict, frag: &str) -> Id {
        dict.intern_iri(&format!(
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#{frag}"
        ))
    }
    /// Build an RDF list of `items` into `triples`, returning the head node id.
    fn list(dict: &mut Dict, triples: &mut Vec<[Id; 3]>, items: &[Id]) -> Id {
        let (first, rest, nil) = (rdf(dict, "first"), rdf(dict, "rest"), rdf(dict, "nil"));
        let mut head = nil;
        for (k, &it) in items.iter().enumerate().rev() {
            let node = dict.intern_blank(&format!("_l{k}_{it}"));
            triples.push([node, first, it]);
            triples.push([node, rest, head]);
            head = node;
        }
        head
    }

    #[test]
    fn property_chain() {
        // :uncleOf owl:propertyChainAxiom ( :parentOf :brotherOf ) ; a parentOf b ; b brotherOf c
        // ⊢ a uncleOf c.
        let mut dict = Dict::new();
        let (uncle, parent, brother, a, b, c) = (
            ex(&mut dict, "uncleOf"),
            ex(&mut dict, "parentOf"),
            ex(&mut dict, "brotherOf"),
            ex(&mut dict, "a"),
            ex(&mut dict, "b"),
            ex(&mut dict, "c"),
        );
        let pca = owl(&mut dict, "propertyChainAxiom");
        let mut triples = vec![[a, parent, b], [b, brother, c]];
        let chain = list(&mut dict, &mut triples, &[parent, brother]);
        triples.push([uncle, pca, chain]);
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            has(
                &dict,
                &set,
                "http://ex/a",
                "http://ex/uncleOf",
                "http://ex/c"
            ),
            "prp-spo2 property chain"
        );
    }

    #[test]
    fn owl_no_features_routes_to_single_pass_with_scm_dom_rng() {
        // No OWL-specific feature → the OWL-RL closure is RDFS + scm-dom/rng, computed by the
        // single-pass fast path. Validates the routing produces rdfs7/2/3/9 + scm-dom1/2/rng1.
        let mut dict = Dict::new();
        let r = |d: &mut Dict, f: &str| d.intern_iri(&format!("http://www.w3.org/2000/01/rdf-schema#{f}"));
        let (sc, sp, dom, rng) =
            (r(&mut dict, "subClassOf"), r(&mut dict, "subPropertyOf"), r(&mut dict, "domain"), r(&mut dict, "range"));
        let ty = rdf(&mut dict, "type");
        let (c0, c1, d0, d1, p, q, x, y) = (
            ex(&mut dict, "c0"),
            ex(&mut dict, "c1"),
            ex(&mut dict, "d0"),
            ex(&mut dict, "d1"),
            ex(&mut dict, "p"),
            ex(&mut dict, "q"),
            ex(&mut dict, "x"),
            ex(&mut dict, "y"),
        );
        let mut triples =
            vec![[c0, sc, c1], [d0, sc, d1], [p, dom, c0], [p, rng, d0], [q, sp, p], [x, q, y]];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(set.contains(&[x, p, y]), "rdfs7 (q subPropertyOf p)");
        assert!(set.contains(&[x, ty, c0]) && set.contains(&[x, ty, c1]), "rdfs2 domain + rdfs9 up subclass");
        assert!(set.contains(&[y, ty, d0]) && set.contains(&[y, ty, d1]), "rdfs3 range + rdfs9 up subclass");
        assert!(set.contains(&[p, dom, c1]), "scm-dom1 (domain up subclass)");
        assert!(set.contains(&[q, dom, c0]), "scm-dom2 (domain down subproperty)");
        assert!(set.contains(&[p, rng, d1]), "scm-rng1 (range up subclass)");
    }

    #[test]
    fn restriction_has_value_and_some_values() {
        // hasValue: R1 = [onProperty :status; hasValue :active]; x a R1 ⊢ x :status :active.
        // someValuesFrom: R2 = [onProperty :hasPet; someValuesFrom :Dog]; x hasPet d, d a Dog ⊢ x a R2.
        let mut dict = Dict::new();
        let (status, active, haspet, dog, x, d) = (
            ex(&mut dict, "status"),
            ex(&mut dict, "active"),
            ex(&mut dict, "hasPet"),
            ex(&mut dict, "Dog"),
            ex(&mut dict, "x"),
            ex(&mut dict, "d"),
        );
        let (r1, r2) = (dict.intern_blank("_R1"), dict.intern_blank("_R2"));
        let ty = rdf(&mut dict, "type");
        let (onp, hv, svf) = (
            owl(&mut dict, "onProperty"),
            owl(&mut dict, "hasValue"),
            owl(&mut dict, "someValuesFrom"),
        );
        let mut triples = vec![
            [r1, onp, status],
            [r1, hv, active],
            [x, ty, r1],
            [r2, onp, haspet],
            [r2, svf, dog],
            [x, haspet, d],
            [d, ty, dog],
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            has(
                &dict,
                &set,
                "http://ex/x",
                "http://ex/status",
                "http://ex/active"
            ),
            "cls-hv1"
        );
        // x is in R2 because it has a Dog pet:
        assert!(set.contains(&[x, ty, r2]), "cls-svf1");
    }

    #[test]
    fn has_key_identifies() {
        // :Person owl:hasKey ( :ssn ) ; a,b both :Person with the same :ssn ⊢ a sameAs b.
        let mut dict = Dict::new();
        let (person, ssn, a, b) = (
            ex(&mut dict, "Person"),
            ex(&mut dict, "ssn"),
            ex(&mut dict, "a"),
            ex(&mut dict, "b"),
        );
        let ty = rdf(&mut dict, "type");
        let hk = owl(&mut dict, "hasKey");
        let v = dict.intern_lit("123", "http://www.w3.org/2001/XMLSchema#string", None);
        let mut triples = vec![[a, ty, person], [a, ssn, v], [b, ty, person], [b, ssn, v]];
        let keylist = list(&mut dict, &mut triples, &[ssn]);
        triples.push([person, hk, keylist]);
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        let sa = owl(&mut dict, "sameAs");
        assert!(
            set.contains(&[a, sa, b]) || set.contains(&[b, sa, a]),
            "prp-key ⊢ sameAs"
        );
    }

    #[test]
    fn max_cardinality_one_and_zero() {
        // maxCardinality 1: the two spouses of :a are sameAs.
        let mut dict = Dict::new();
        let (spouse, a, x, y) = (
            ex(&mut dict, "spouse"),
            ex(&mut dict, "a"),
            ex(&mut dict, "x"),
            ex(&mut dict, "y"),
        );
        let r = dict.intern_blank("_R1");
        let ty = rdf(&mut dict, "type");
        let (onp, mc) = (
            owl(&mut dict, "onProperty"),
            owl(&mut dict, "maxCardinality"),
        );
        let one = dict.intern_lit("1", "http://www.w3.org/2001/XMLSchema#integer", None);
        let mut triples = vec![
            [r, onp, spouse],
            [r, mc, one],
            [a, ty, r],
            [a, spouse, x],
            [a, spouse, y],
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        let sa = owl(&mut dict, "sameAs");
        assert!(
            set.contains(&[x, sa, y]) || set.contains(&[y, sa, x]),
            "cls-maxc2 ⊢ sameAs"
        );

        // maxCardinality 0: :b typed a no-child restriction yet has a child ⊢ inconsistent.
        let mut d2 = Dict::new();
        let (child, b) = (ex(&mut d2, "child"), ex(&mut d2, "b"));
        let r2 = d2.intern_blank("_R0");
        let ty = rdf(&mut d2, "type");
        let (onp, mc) = (owl(&mut d2, "onProperty"), owl(&mut d2, "maxCardinality"));
        let zero = d2.intern_lit("0", "http://www.w3.org/2001/XMLSchema#integer", None);
        let mut t2 = vec![
            [r2, onp, child],
            [r2, mc, zero],
            [b, ty, r2],
            [b, child, ex(&mut d2, "kid")],
        ];
        materialize_owl_rl(&mut d2, &mut t2);
        assert!(
            !inconsistencies(&d2, &t2).is_empty(),
            "cls-maxc1 clash detected"
        );
    }

    #[test]
    fn max_qualified_cardinality() {
        // R = [onProperty :parent; maxQualifiedCardinality 1; onClass :Mother];
        // :x a R, :x :parent :a, :x :parent :b, :a a :Mother, :b a :Mother ⊢ :a sameAs :b.
        let mut dict = Dict::new();
        let (parent, mother, x, a, b) = (
            ex(&mut dict, "parent"),
            ex(&mut dict, "Mother"),
            ex(&mut dict, "x"),
            ex(&mut dict, "a"),
            ex(&mut dict, "b"),
        );
        let r = dict.intern_blank("_RQ");
        let ty = rdf(&mut dict, "type");
        let (onp, mqc, onc) = (
            owl(&mut dict, "onProperty"),
            owl(&mut dict, "maxQualifiedCardinality"),
            owl(&mut dict, "onClass"),
        );
        let one = dict.intern_lit("1", "http://www.w3.org/2001/XMLSchema#integer", None);
        let mut triples = vec![
            [r, onp, parent],
            [r, mqc, one],
            [r, onc, mother],
            [x, ty, r],
            [x, parent, a],
            [x, parent, b],
            [a, ty, mother],
            [b, ty, mother],
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        let sa = owl(&mut dict, "sameAs");
        assert!(
            set.contains(&[a, sa, b]) || set.contains(&[b, sa, a]),
            "cls-maxqc ⊢ sameAs"
        );
    }

    #[test]
    fn consistency_disjoint_clash() {
        // :Cat owl:disjointWith :Dog ; :felix a :Cat ; :felix a :Dog  → inconsistent.
        let mut dict = Dict::new();
        let (cat, dog, felix) = (
            ex(&mut dict, "Cat"),
            ex(&mut dict, "Dog"),
            ex(&mut dict, "felix"),
        );
        let ty = rdf(&mut dict, "type");
        let dw = owl(&mut dict, "disjointWith");
        let mut triples = vec![[cat, dw, dog], [felix, ty, cat], [felix, ty, dog]];
        materialize_owl_rl(&mut dict, &mut triples);
        let clashes = inconsistencies(&dict, &triples);
        assert!(!clashes.is_empty(), "disjointWith clash should be detected");

        // A consistent variant (felix only a Cat) must report none.
        let mut dict2 = Dict::new();
        let (cat, dog, felix) = (
            ex(&mut dict2, "Cat"),
            ex(&mut dict2, "Dog"),
            ex(&mut dict2, "felix"),
        );
        let ty = rdf(&mut dict2, "type");
        let dw = owl(&mut dict2, "disjointWith");
        let mut t2 = vec![[cat, dw, dog], [felix, ty, cat]];
        materialize_owl_rl(&mut dict2, &mut t2);
        assert!(
            inconsistencies(&dict2, &t2).is_empty(),
            "consistent graph reports no clash"
        );
    }

    #[test]
    fn scm_domain_range_propagation() {
        // :p2 subPropertyOf :p ; :p domain :C ; :C subClassOf :D
        // ⊢ :p domain :D (scm-dom1, up subClassOf) and :p2 domain :C (scm-dom2, down subPropertyOf).
        let mut dict = Dict::new();
        let (p, p2, cc, dd) = (
            ex(&mut dict, "p"),
            ex(&mut dict, "p2"),
            ex(&mut dict, "C"),
            ex(&mut dict, "D"),
        );
        let rdfs = |d: &mut Dict, f: &str| {
            d.intern_iri(&format!("http://www.w3.org/2000/01/rdf-schema#{f}"))
        };
        let (sp, dom, sc) = (
            rdfs(&mut dict, "subPropertyOf"),
            rdfs(&mut dict, "domain"),
            rdfs(&mut dict, "subClassOf"),
        );
        let mut triples = vec![[p2, sp, p], [p, dom, cc], [cc, sc, dd]];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            set.contains(&[p, dom, dd]),
            "scm-dom1: domain up subClassOf"
        );
        assert!(
            set.contains(&[p2, dom, cc]),
            "scm-dom2: domain down subPropertyOf"
        );
    }

    #[test]
    fn scm_schema_subclass() {
        // equivalentClass ⊢ subClassOf both ways; intersectionOf ⊢ c subClassOf each member.
        let mut dict = Dict::new();
        let (human, person, mother, woman, parent) = (
            ex(&mut dict, "Human"),
            ex(&mut dict, "Person"),
            ex(&mut dict, "Mother"),
            ex(&mut dict, "Woman"),
            ex(&mut dict, "Parent"),
        );
        let (eqc, io) = (
            owl(&mut dict, "equivalentClass"),
            owl(&mut dict, "intersectionOf"),
        );
        let sco = dict.intern_iri("http://www.w3.org/2000/01/rdf-schema#subClassOf");
        let mut triples = vec![[human, eqc, person]];
        let l = list(&mut dict, &mut triples, &[woman, parent]);
        triples.push([mother, io, l]);
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(set.contains(&[human, sco, person]), "scm-eqc forward");
        assert!(set.contains(&[person, sco, human]), "scm-eqc backward");
        assert!(
            set.contains(&[mother, sco, woman]) && set.contains(&[mother, sco, parent]),
            "scm-int"
        );
    }

    #[test]
    fn intersection_of() {
        // :Mother owl:intersectionOf ( :Woman :Parent ) ; x a Woman ; x a Parent ⊢ x a Mother.
        let mut dict = Dict::new();
        let (mother, woman, parent, x) = (
            ex(&mut dict, "Mother"),
            ex(&mut dict, "Woman"),
            ex(&mut dict, "Parent"),
            ex(&mut dict, "x"),
        );
        let (ty, io) = (rdf(&mut dict, "type"), owl(&mut dict, "intersectionOf"));
        let mut triples = vec![[x, ty, woman], [x, ty, parent]];
        let l = list(&mut dict, &mut triples, &[woman, parent]);
        triples.push([mother, io, l]);
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            set.contains(&[x, ty, mother]),
            "cls-int1 intersection membership"
        );
    }

    #[test]
    fn equivalent_class_and_property() {
        let mut dict = Dict::new();
        let (human, person, x) = (
            ex(&mut dict, "Human"),
            ex(&mut dict, "Person"),
            ex(&mut dict, "x"),
        );
        let (likes, enjoys, y) = (
            ex(&mut dict, "likes"),
            ex(&mut dict, "enjoys"),
            ex(&mut dict, "y"),
        );
        let ty = dict.intern_iri(rdf::TYPE.as_str());
        let eqc = owl(&mut dict, "equivalentClass");
        let eqp = owl(&mut dict, "equivalentProperty");
        let mut triples = vec![
            [human, eqc, person],
            [x, ty, human],
            [likes, eqp, enjoys],
            [x, likes, y],
        ];
        materialize_owl_rl(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(
            has(
                &dict,
                &set,
                "http://ex/x",
                rdf::TYPE.as_str(),
                "http://ex/Person"
            ),
            "cax-eqc"
        );
        assert!(
            has(
                &dict,
                &set,
                "http://ex/x",
                "http://ex/enjoys",
                "http://ex/y"
            ),
            "prp-eqp"
        );
    }
}
