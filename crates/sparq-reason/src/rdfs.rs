//! RDFS forward-chaining materialization.
//!
//! Implements the *useful, non-explosive* RDFS entailment rules (RDF 1.1 Semantics §9.2.1)
//! as a fixpoint over dictionary-encoded triples:
//!
//! | rule | premise | conclusion |
//! |------|---------|------------|
//! | rdfs2  | `(p domain c)`, `(s p o)`         | `(s type c)` |
//! | rdfs3  | `(p range c)`, `(s p o)`          | `(o type c)` |
//! | rdfs5  | `(p subPropertyOf q)`, `(q subPropertyOf r)` | `(p subPropertyOf r)` |
//! | rdfs7  | `(p subPropertyOf q)`, `(s p o)`  | `(s q o)` |
//! | rdfs9  | `(c subClassOf d)`, `(s type c)`  | `(s type d)` |
//! | rdfs11 | `(c subClassOf d)`, `(d subClassOf e)` | `(c subClassOf e)` |
//!
//! Deliberately OMITTED: the axiomatic triples and the reflexive/`rdfs:Resource` rules
//! (rdfs4a/4b, rdfs6, rdfs8, rdfs10, rdfs13). They entail that every resource is an
//! `rdfs:Resource` and every class a subclass of itself/`Resource` — true but useless, and
//! they blow up the store by O(terms). This matches the "RDFS" rule set materialized by
//! production engines (GraphDB/RDF4J `rdfs` minus the axiomatic closure).
//!
//! The fixpoint is **naive** (re-derive from the full set each round until stable) — chosen
//! for obvious correctness; RDFS closures converge in a handful of rounds (≈ hierarchy
//! depth + 1). Semi-naive (delta-only) evaluation is a future optimization; materialization
//! is an opt-in build-time step, never on the query hot path.

use crate::Vocab;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

/// Incremental indexes for SEMI-NAIVE RDFS materialization: a new triple joins only against
/// the relevant index (not a full scan), and each rule is fired once per newly-derived fact
/// (the `delta`). Both directions of every join are covered so transitivity (rdfs5/11)
/// closes correctly. Without this the naive fixpoint is O(N³) on deep hierarchies (the O(N²)
/// subclass closure re-derived O(N) rounds).
#[derive(Default)]
pub(crate) struct RdfsIndex {
    sc_super: FxHashMap<Id, Vec<Id>>, // c -> super-classes d (c subClassOf d)
    sc_sub: FxHashMap<Id, Vec<Id>>,   // d -> sub-classes c
    sp_super: FxHashMap<Id, Vec<Id>>, // p -> super-properties q
    sp_sub: FxHashMap<Id, Vec<Id>>,   // q -> sub-properties p
    domain: FxHashMap<Id, Vec<Id>>,   // p -> domain classes
    range: FxHashMap<Id, Vec<Id>>,    // p -> range classes
    type_sub: FxHashMap<Id, Vec<Id>>, // c -> subjects typed c
    po: FxHashMap<Id, Vec<(Id, Id)>>, // predicate -> (subject, object) assertions
}

impl RdfsIndex {
    pub(crate) fn insert(&mut self, [s, p, o]: [Id; 3], v: &Vocab) {
        if p == v.sub_class {
            self.sc_super.entry(s).or_default().push(o);
            self.sc_sub.entry(o).or_default().push(s);
        } else if p == v.sub_prop {
            self.sp_super.entry(s).or_default().push(o);
            self.sp_sub.entry(o).or_default().push(s);
        } else if p == v.domain {
            self.domain.entry(s).or_default().push(o);
        } else if p == v.range {
            self.range.entry(s).or_default().push(o);
        } else if p == v.ty {
            self.type_sub.entry(o).or_default().push(s);
        }
        self.po.entry(p).or_default().push((s, o));
    }

    /// All immediate RDFS consequences of `[s,p,o]` joining against the current index, pushed
    /// into `out`. Each rule appears in BOTH delta directions (this triple as either premise).
    pub(crate) fn derive(&self, [s, p, o]: [Id; 3], v: &Vocab, out: &mut Vec<[Id; 3]>) {
        if p == v.sub_class {
            // rdfs11 (s sc o)(o sc x)⊢(s sc x) and (c sc s)(s sc o)⊢(c sc o)
            if let Some(xs) = self.sc_super.get(&o) {
                out.extend(xs.iter().map(|&x| [s, v.sub_class, x]));
            }
            if let Some(cs) = self.sc_sub.get(&s) {
                out.extend(cs.iter().map(|&c| [c, v.sub_class, o]));
            }
            // rdfs9 (y type s)(s sc o)⊢(y type o)
            if let Some(ys) = self.type_sub.get(&s) {
                out.extend(ys.iter().map(|&y| [y, v.ty, o]));
            }
        } else if p == v.sub_prop {
            // rdfs5 transitivity (both directions)
            if let Some(xs) = self.sp_super.get(&o) {
                out.extend(xs.iter().map(|&x| [s, v.sub_prop, x]));
            }
            if let Some(cs) = self.sp_sub.get(&s) {
                out.extend(cs.iter().map(|&c| [c, v.sub_prop, o]));
            }
            // rdfs7 (x s y)(s sp o)⊢(x o y)
            if let Some(pairs) = self.po.get(&s) {
                out.extend(pairs.iter().map(|&(x, y)| [x, o, y]));
            }
        } else if p == v.ty {
            // rdfs9 (s type o)(o sc d)⊢(s type d)
            if let Some(ds) = self.sc_super.get(&o) {
                out.extend(ds.iter().map(|&d| [s, v.ty, d]));
            }
        } else if p == v.domain {
            // rdfs2 (x s y)(s domain o)⊢(x type o)
            if let Some(pairs) = self.po.get(&s) {
                out.extend(pairs.iter().map(|&(x, _)| [x, v.ty, o]));
            }
        } else if p == v.range {
            // rdfs3 (x s y)(s range o)⊢(y type o)
            if let Some(pairs) = self.po.get(&s) {
                out.extend(pairs.iter().map(|&(_, y)| [y, v.ty, o]));
            }
        }
        // For the OTHER delta direction of rdfs7/2/3: this (s p o) as the data triple.
        if let Some(qs) = self.sp_super.get(&p) {
            out.extend(qs.iter().map(|&q| [s, q, o]));
        }
        if let Some(cs) = self.domain.get(&p) {
            out.extend(cs.iter().map(|&c| [s, v.ty, c]));
        }
        if let Some(cs) = self.range.get(&p) {
            out.extend(cs.iter().map(|&c| [o, v.ty, c]));
        }
    }
}

/// Below this many input triples the ABox sweep runs single-threaded (rayon fan-out is not
/// worth the overhead). Only referenced by the rayon-backed sweep, so it is gated with
/// `parallel` — without the feature (e.g. sparq-solid's `default-features = false`
/// dependency) it would otherwise be dead code and trip `clippy -D warnings`.
/// [OPUS-4.8] sq-xor3: discovered while running the bead's sparq-solid clippy gate.
#[cfg(feature = "parallel")]
const PAR_THRESHOLD: usize = 4096;

/// Transitive closure of a directed relation (`node -> direct successors`): returns
/// `node -> ALL reachable successors`. BFS per source — the schema graph (subClassOf /
/// subPropertyOf) is small relative to the data, even when deep.
pub(crate) fn transitive_closure(direct: &FxHashMap<Id, Vec<Id>>) -> FxHashMap<Id, Vec<Id>> {
    let mut closure: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    for (&start, succ0) in direct {
        let mut seen: FxHashSet<Id> = FxHashSet::default();
        let mut stack: Vec<Id> = succ0.clone();
        while let Some(n) = stack.pop() {
            if seen.insert(n) {
                if let Some(succ) = direct.get(&n) {
                    stack.extend(succ.iter().copied());
                }
            }
        }
        closure.insert(start, seen.into_iter().collect());
    }
    closure
}

/// For each property, the full set of typing classes implied by `dr` (its domain or range
/// triples), gathered over the property AND all its super-properties, each closed upward through
/// the subclass closure. So in one pass `(s p o)` yields `(s type c)` / `(o type c)` for every
/// `c` here — no fixpoint over rdfs2/3 ⋈ rdfs7 ⋈ rdfs9 needed.
pub(crate) fn close_dr(
    dr: &FxHashMap<Id, Vec<Id>>,
    sp_closure: &FxHashMap<Id, Vec<Id>>,
    sc_closure: &FxHashMap<Id, Vec<Id>>,
) -> FxHashMap<Id, Vec<Id>> {
    let props: FxHashSet<Id> = dr.keys().chain(sp_closure.keys()).copied().collect();
    let mut out: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    for p in props {
        let mut classes: FxHashSet<Id> = FxHashSet::default();
        let supers = sp_closure.get(&p).map(|v| v.as_slice()).unwrap_or(&[]);
        for &q in std::iter::once(&p).chain(supers) {
            if let Some(cs) = dr.get(&q) {
                for &c in cs {
                    classes.insert(c);
                    if let Some(up) = sc_closure.get(&c) {
                        classes.extend(up.iter().copied());
                    }
                }
            }
        }
        if !classes.is_empty() {
            out.insert(p, classes.into_iter().collect());
        }
    }
    out
}

/// The MONOTONE OWL-RL property layer that can be saturated in the single pass: for each
/// property `p`, the full set of `(r, swapped)` pairs reachable from `p` by composing
/// `rdfs:subPropertyOf` (rdfs7), `owl:inverseOf` (prp-inv), `owl:SymmetricProperty`
/// (prp-symp), and `owl:equivalentProperty` (= bidirectional subPropertyOf). `swapped=false`
/// keeps the subject/object orientation; `swapped=true` transposes it. Closed once over the
/// (small, TBox-only) property-orientation graph — the property analogue of `sc_closure`.
///
/// When present this REPLACES the plain `sp_closure` rewrite in [`emit_consequences`]: an
/// asserted `(x p y)` then emits `(x r y)` for every `(r,false)` and `(y r x)` for every
/// `(r,true)`, and the domain/range typing of `r` is applied with that orientation. Every
/// emitted edge is an OWL-RL entailment (each composition step is a valid rule), and the BFS
/// closure is complete, so the result is byte-identical to the fixpoint over this subset.
#[derive(Default)]
pub(crate) struct PropExpand {
    /// `p -> [(r, swapped)]` — derived predicate `r` and whether subject/object are transposed.
    map: FxHashMap<Id, Vec<(Id, bool)>>,
}

/// All RDFS consequences of one asserted triple against the (already fully-closed) schema,
/// optionally extended by the monotone-OWL property-orientation closure `px`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_consequences(
    [s, p, o]: [Id; 3],
    v: &Vocab,
    sc_closure: &FxHashMap<Id, Vec<Id>>,
    sp_closure: &FxHashMap<Id, Vec<Id>>,
    dom_full: &FxHashMap<Id, Vec<Id>>,
    rng_full: &FxHashMap<Id, Vec<Id>>,
    px: Option<&PropExpand>,
    out: &mut Vec<[Id; 3]>,
) {
    if p == v.ty {
        // rdfs9: (s type o), (o sc* d) ⊢ (s type d)
        if let Some(ds) = sc_closure.get(&o) {
            out.extend(ds.iter().map(|&d| [s, v.ty, d]));
        }
    } else if let Some(px) = px.filter(|px| px.map.contains_key(&p)) {
        // Monotone-OWL path: rewrite the predicate AND orient through inverse/symmetric, then
        // type via domain/range of the (oriented) derived predicate.
        for &(r, swapped) in &px.map[&p] {
            let (rs_, ro_) = if swapped { (o, s) } else { (s, o) };
            out.push([rs_, r, ro_]); // rdfs7 / prp-inv / prp-symp / prp-eqp
            if let Some(cs) = dom_full.get(&r) {
                out.extend(cs.iter().map(|&c| [rs_, v.ty, c])); // rdfs2 (+rdfs9)
            }
            if let Some(cs) = rng_full.get(&r) {
                out.extend(cs.iter().map(|&c| [ro_, v.ty, c])); // rdfs3 (+rdfs9)
            }
        }
    } else {
        // No PropExpand active, OR `p` never appears in the property-orientation TBox (no
        // subPropertyOf/inverseOf/symmetric/equivalent edge) — its expansion is the trivial
        // {(p, false)}, i.e. exactly the plain-RDFS emission below. Falling through here (not
        // skipping!) keeps domain/range typing for properties that have ONLY a domain/range
        // declaration: build_prop_expand's all_props never collects those, so an early
        // `px.map.get(&p) → None` return used to silently drop their rdfs2/rdfs3 typing while
        // the full fixpoint path emitted it.
        // rdfs7: (s p o), (p sp* q) ⊢ (s q o)
        if let Some(qs) = sp_closure.get(&p) {
            out.extend(qs.iter().map(|&q| [s, q, o]));
        }
        // rdfs2 + rdfs9: domain typing through all super-properties, closed up subclass
        if let Some(cs) = dom_full.get(&p) {
            out.extend(cs.iter().map(|&c| [s, v.ty, c]));
        }
        // rdfs3 + rdfs9: range typing
        if let Some(cs) = rng_full.get(&p) {
            out.extend(cs.iter().map(|&c| [o, v.ty, c]));
        }
    }
}

/// Run [`emit_consequences`] over every asserted triple. With the `parallel` feature the sweep
/// fans out over rayon (read-only on the schema closures) into per-thread buffers.
#[cfg(feature = "parallel")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn sweep(
    triples: &[[Id; 3]],
    v: &Vocab,
    sc: &FxHashMap<Id, Vec<Id>>,
    sp: &FxHashMap<Id, Vec<Id>>,
    dom: &FxHashMap<Id, Vec<Id>>,
    rng: &FxHashMap<Id, Vec<Id>>,
    px: Option<&PropExpand>,
) -> Vec<[Id; 3]> {
    use rayon::prelude::*;
    if triples.len() < PAR_THRESHOLD {
        let mut out = Vec::new();
        for &t in triples {
            emit_consequences(t, v, sc, sp, dom, rng, px, &mut out);
        }
        return out;
    }
    triples
        .par_iter()
        .fold(Vec::new, |mut acc, &t| {
            emit_consequences(t, v, sc, sp, dom, rng, px, &mut acc);
            acc
        })
        .reduce(Vec::new, |mut a, mut b| {
            a.append(&mut b);
            a
        })
}
#[cfg(not(feature = "parallel"))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn sweep(
    triples: &[[Id; 3]],
    v: &Vocab,
    sc: &FxHashMap<Id, Vec<Id>>,
    sp: &FxHashMap<Id, Vec<Id>>,
    dom: &FxHashMap<Id, Vec<Id>>,
    rng: &FxHashMap<Id, Vec<Id>>,
    px: Option<&PropExpand>,
) -> Vec<[Id; 3]> {
    let mut out = Vec::new();
    for &t in triples {
        emit_consequences(t, v, sc, sp, dom, rng, px, &mut out);
    }
    out
}

/// Drop the candidates already asserted, de-duplicate the rest, and return them sorted (so the
/// materialized output is deterministic). With the `parallel` feature the filter + sort run on
/// rayon — a one-shot dedup, unlike the per-round HashSet churn of the old fixpoint.
#[cfg(feature = "parallel")]
fn dedup_derived(emitted: Vec<[Id; 3]>, original: &FxHashSet<[Id; 3]>) -> Vec<[Id; 3]> {
    use rayon::prelude::*;
    if emitted.len() < PAR_THRESHOLD {
        let mut set: FxHashSet<[Id; 3]> = FxHashSet::default();
        for t in emitted {
            if !original.contains(&t) {
                set.insert(t);
            }
        }
        let mut d: Vec<[Id; 3]> = set.into_iter().collect();
        d.sort_unstable();
        return d;
    }
    let mut d: Vec<[Id; 3]> = emitted.into_par_iter().filter(|t| !original.contains(t)).collect();
    d.par_sort_unstable();
    d.dedup();
    d
}
#[cfg(not(feature = "parallel"))]
fn dedup_derived(emitted: Vec<[Id; 3]>, original: &FxHashSet<[Id; 3]>) -> Vec<[Id; 3]> {
    let mut set: FxHashSet<[Id; 3]> = FxHashSet::default();
    for t in emitted {
        if !original.contains(&t) {
            set.insert(t);
        }
    }
    let mut d: Vec<[Id; 3]> = set.into_iter().collect();
    d.sort_unstable();
    d
}

/// The MONOTONE OWL-RL TBox that the single-pass sweep can saturate alongside RDFS, supplied
/// by the OWL materializer when an ontology uses ONLY these (non-recursive) features. Each is
/// fixed by the input (no OWL-RL rule derives `inverseOf` / `SymmetricProperty` / `equivalent*`
/// / `subPropertyOf`-from-nothing), so saturating them once is complete.
#[derive(Default)]
pub(crate) struct MonoOwl {
    /// `owl:equivalentClass` pairs (folded into subClassOf both ways — scm-eqc + cax-eqc).
    pub equiv_class: Vec<(Id, Id)>,
    /// `owl:equivalentProperty` pairs (folded into subPropertyOf both ways — scm-eqp + prp-eqp).
    pub equiv_prop: Vec<(Id, Id)>,
    /// `owl:inverseOf` direct edges, both directions (prp-inv1/2).
    pub inverse: FxHashMap<Id, Vec<Id>>,
    /// `owl:SymmetricProperty` (prp-symp).
    pub symmetric: FxHashSet<Id>,
}

impl MonoOwl {
    fn needs_prop_expand(&self) -> bool {
        !self.inverse.is_empty() || !self.symmetric.is_empty()
    }
}

/// Build the property-orientation closure (see [`PropExpand`]) from the saturated subPropertyOf
/// closure plus the monotone-OWL inverse/symmetric axioms. BFS over `(property, orientation)`
/// nodes; small (TBox-only). The start node `(p,false)` is INCLUDED so the asserted edge's own
/// predicate is rewritten uniformly (and its domain/range typed); the asserted triple itself is
/// dropped later by the dedup against `original`.
fn build_prop_expand(sp_closure: &FxHashMap<Id, Vec<Id>>, m: &MonoOwl) -> PropExpand {
    // Every property mentioned anywhere in the property TBox/ABox flows through here; we expand
    // lazily per start property encountered as a key of the union of relevant maps.
    let starts: FxHashSet<Id> = sp_closure
        .keys()
        .chain(m.inverse.keys())
        .chain(m.symmetric.iter())
        .copied()
        .collect();
    // We must also expand properties that only appear as subPropertyOf SUPERS or inverse targets;
    // collect those too so a bare asserted `(x p y)` whose p is e.g. only an inverse target still
    // resolves. (Properties never mentioned in the TBox have the trivial {(p,false)} expansion,
    // handled by the `None` fall-through in emit_consequences via sp_closure absence — but with
    // PropExpand active we need an explicit entry, so include all reachable property ids.)
    let mut all_props: FxHashSet<Id> = starts.clone();
    for sup in sp_closure.values() {
        all_props.extend(sup.iter().copied());
    }
    for inv in m.inverse.values() {
        all_props.extend(inv.iter().copied());
    }
    let mut map: FxHashMap<Id, Vec<(Id, bool)>> = FxHashMap::default();
    for &p in &all_props {
        let mut seen: FxHashSet<(Id, bool)> = FxHashSet::default();
        let mut stack: Vec<(Id, bool)> = vec![(p, false)];
        while let Some((q, or)) = stack.pop() {
            if !seen.insert((q, or)) {
                continue;
            }
            // subPropertyOf (incl. equivalentProperty folded in earlier): same orientation.
            if let Some(sups) = sp_closure.get(&q) {
                for &r in sups {
                    if !seen.contains(&(r, or)) {
                        stack.push((r, or));
                    }
                }
            }
            // inverseOf: flip orientation.
            if let Some(invs) = m.inverse.get(&q) {
                for &r in invs {
                    if !seen.contains(&(r, !or)) {
                        stack.push((r, !or));
                    }
                }
            }
            // SymmetricProperty: flip orientation, same predicate.
            if m.symmetric.contains(&q) && !seen.contains(&(q, !or)) {
                stack.push((q, !or));
            }
        }
        map.insert(p, seen.into_iter().collect());
    }
    PropExpand { map }
}

/// Expand `triples` in place with the RDFS closure. Returns the number of NEW triples added.
pub fn materialize_rdfs(dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> usize {
    rdfs_closure(dict, triples, false, &MonoOwl::default())
}

/// The RDFS closure, plus — when `emit_dr_closure` — the OWL-RL `scm-dom1/2` / `scm-rng1/2`
/// domain/range closure triples.
///
/// Strategy: SATURATE THE SCHEMA (TBox) once — subClassOf / subPropertyOf transitive closures
/// and the domain/range typing sets closed through them — then do a SINGLE data-parallel sweep
/// of the assertions (ABox). Because the schema is fully closed, every assertion's consequences
/// are independent and need no fixpoint, so the sweep is embarrassingly parallel. Used directly
/// for RDFS, and by the OWL-RL materializer when the ontology uses no OWL-specific features (the
/// OWL closure is then exactly RDFS + scm-dom/rng).
pub(crate) fn rdfs_closure(
    dict: &mut Dict,
    triples: &mut Vec<[Id; 3]>,
    emit_dr_closure: bool,
    mono: &MonoOwl,
) -> usize {
    let v = Vocab::intern(dict);
    let original: FxHashSet<[Id; 3]> = triples.iter().copied().collect();

    // 1. Raw schema maps from the input.
    let mut sc: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    let mut sp: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    let mut dom: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    let mut rng: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
    for &[s, p, o] in &original {
        if p == v.sub_class {
            sc.entry(s).or_default().push(o);
        } else if p == v.sub_prop {
            sp.entry(s).or_default().push(o);
        } else if p == v.domain {
            dom.entry(s).or_default().push(o);
        } else if p == v.range {
            rng.entry(s).or_default().push(o);
        }
    }
    // Monotone OWL: fold equivalentClass/equivalentProperty into subClassOf/subPropertyOf both
    // ways BEFORE saturation (scm-eqc/scm-eqp). cax-eqc1/2 and prp-eqp1/2 then follow for free
    // from rdfs9/rdfs7 over the added edges, and the closure emits the subClassOf/subPropertyOf
    // triples exactly as the fixpoint's scm-eqc/eqp do.
    for &(a, b) in &mono.equiv_class {
        sc.entry(a).or_default().push(b);
        sc.entry(b).or_default().push(a);
    }
    for &(a, b) in &mono.equiv_prop {
        sp.entry(a).or_default().push(b);
        sp.entry(b).or_default().push(a);
    }

    // 2. Saturate the schema (small, serial).
    let sc_closure = transitive_closure(&sc);
    let sp_closure = transitive_closure(&sp);
    let dom_full = close_dr(&dom, &sp_closure, &sc_closure);
    let rng_full = close_dr(&rng, &sp_closure, &sc_closure);

    // Property-orientation closure for inverseOf/SymmetricProperty (only when needed).
    let prop_expand = if mono.needs_prop_expand() {
        Some(build_prop_expand(&sp_closure, mono))
    } else {
        None
    };

    // 3. Single parallel ABox sweep + the schema-closure triples (rdfs11 / rdfs5).
    let asserted: Vec<[Id; 3]> = original.iter().copied().collect();
    let mut emitted = sweep(
        &asserted,
        &v,
        &sc_closure,
        &sp_closure,
        &dom_full,
        &rng_full,
        prop_expand.as_ref(),
    );
    for (&c, ds) in &sc_closure {
        emitted.extend(ds.iter().map(|&d| [c, v.sub_class, d]));
    }
    for (&p, qs) in &sp_closure {
        emitted.extend(qs.iter().map(|&q| [p, v.sub_prop, q]));
    }
    // OWL-RL scm-dom1/2 + scm-rng1/2: the saturated domain/range sets ARE their closure.
    if emit_dr_closure {
        for (&p, cs) in &dom_full {
            emitted.extend(cs.iter().map(|&c| [p, v.domain, c]));
        }
        for (&p, cs) in &rng_full {
            emitted.extend(cs.iter().map(|&c| [p, v.range, c]));
        }
    }

    // 4. De-duplicate the derived facts, drop those already asserted, sort for determinism.
    let derived = dedup_derived(emitted, &original);

    let added = derived.len();
    triples.clear();
    triples.extend(original_in_order(&original));
    triples.extend(derived);
    // NOT `triples.len() - before`: a caller may pass DUPLICATE input triples
    // (e.g. RDF/XML loaders emit repeated declarations), which the rebuild
    // dedups — the subtraction then underflows. The new-triple count is the
    // derived set's size by construction.
    added
}


/// The original triples, deduplicated but in a deterministic (sorted) order. (We do not have
/// the caller's original Vec here after clearing; sorting gives a stable, reproducible base.)
fn original_in_order(original: &FxHashSet<[Id; 3]>) -> Vec<[Id; 3]> {
    let mut v: Vec<[Id; 3]> = original.iter().copied().collect();
    v.sort_unstable();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::vocab::{rdf, rdfs};
    use oxrdf::{NamedNode, Term};

    fn iri(dict: &mut Dict, s: &str) -> Id {
        dict.intern_iri(s)
    }
    fn ex(dict: &mut Dict, local: &str) -> Id {
        dict.intern_iri(&format!("http://ex/{local}"))
    }
    /// Is the triple (by IRI strings) in the materialized set?
    fn has(dict: &Dict, set: &FxHashSet<[Id; 3]>, s: &str, p: &str, o: &str) -> bool {
        let g = |iri: &str| dict.lookup(&Term::NamedNode(NamedNode::new_unchecked(iri.to_string())));
        let (si, pi, oi) = (g(s), g(p), g(o));
        si != 0 && pi != 0 && oi != 0 && set.contains(&[si, pi, oi])
    }

    #[test]
    fn subclass_transitivity_and_type_propagation() {
        // ex:Dog sc ex:Mammal sc ex:Animal ; ex:rex a ex:Dog.  Expect ex:rex a Mammal, Animal.
        let mut dict = Dict::new();
        let (dog, mammal, animal, rex) =
            (ex(&mut dict, "Dog"), ex(&mut dict, "Mammal"), ex(&mut dict, "Animal"), ex(&mut dict, "rex"));
        let (sc, ty) = (iri(&mut dict, rdfs::SUB_CLASS_OF.as_str()), iri(&mut dict, rdf::TYPE.as_str()));
        let mut triples = vec![[dog, sc, mammal], [mammal, sc, animal], [rex, ty, dog]];
        let added = materialize_rdfs(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/rex", rdf::TYPE.as_str(), "http://ex/Mammal"), "rdfs9 one hop");
        assert!(has(&dict, &set, "http://ex/rex", rdf::TYPE.as_str(), "http://ex/Animal"), "rdfs9 transitive");
        assert!(has(&dict, &set, "http://ex/Dog", rdfs::SUB_CLASS_OF.as_str(), "http://ex/Animal"), "rdfs11");
        assert!(added >= 3);
    }

    #[test]
    fn domain_range_and_subproperty() {
        // ex:hasParent sp ex:relatedTo ; ex:hasParent domain ex:Person ; ex:hasParent range ex:Person.
        // ex:a ex:hasParent ex:b.  Expect: a relatedTo b; a type Person; b type Person.
        let mut dict = Dict::new();
        let (hp, rel, person, a, b) = (
            ex(&mut dict, "hasParent"), ex(&mut dict, "relatedTo"), ex(&mut dict, "Person"),
            ex(&mut dict, "a"), ex(&mut dict, "b"),
        );
        let (sp, dom, rng) = (
            iri(&mut dict, rdfs::SUB_PROPERTY_OF.as_str()),
            iri(&mut dict, rdfs::DOMAIN.as_str()),
            iri(&mut dict, rdfs::RANGE.as_str()),
        );
        let mut triples = vec![[hp, sp, rel], [hp, dom, person], [hp, rng, person], [a, hp, b]];
        materialize_rdfs(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/a", "http://ex/relatedTo", "http://ex/b"), "rdfs7");
        assert!(has(&dict, &set, "http://ex/a", rdf::TYPE.as_str(), "http://ex/Person"), "rdfs2 domain");
        assert!(has(&dict, &set, "http://ex/b", rdf::TYPE.as_str(), "http://ex/Person"), "rdfs3 range");
    }

    #[test]
    fn subproperty_domain_interaction() {
        // ex:p sp ex:q ; ex:q domain ex:C ; ex:s ex:p ex:o.  rdfs7 gives (s q o), then rdfs2
        // on q's domain gives (s type C). Tests the rule-interaction the fixpoint must catch.
        let mut dict = Dict::new();
        let (p, q, c, s, o) = (
            ex(&mut dict, "p"), ex(&mut dict, "q"), ex(&mut dict, "C"), ex(&mut dict, "s"), ex(&mut dict, "o"),
        );
        let (sp, dom) = (iri(&mut dict, rdfs::SUB_PROPERTY_OF.as_str()), iri(&mut dict, rdfs::DOMAIN.as_str()));
        let mut triples = vec![[p, sp, q], [q, dom, c], [s, p, o]];
        materialize_rdfs(&mut dict, &mut triples);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(has(&dict, &set, "http://ex/s", "http://ex/q", "http://ex/o"), "rdfs7");
        assert!(has(&dict, &set, "http://ex/s", rdf::TYPE.as_str(), "http://ex/C"), "rdfs7->rdfs2 interaction");
    }

    #[test]
    fn prop_expand_keeps_domain_range_only_properties() {
        // REGRESSION: with PropExpand active (any inverseOf/symmetric axiom in the ontology), a
        // property that has a domain/range but NO property-orientation edge (no subPropertyOf /
        // inverseOf / symmetric / equivalent) was absent from the px map, and emit_consequences
        // dropped its rdfs2/rdfs3 typing entirely — diverging from the full fixpoint path.
        let mut dict = Dict::new();
        let (p, q, r, c, d, a, b) = (
            ex(&mut dict, "p"), ex(&mut dict, "q"), ex(&mut dict, "r"), ex(&mut dict, "C"),
            ex(&mut dict, "D"), ex(&mut dict, "a"), ex(&mut dict, "b"),
        );
        let (dom, rng, ty) = (
            iri(&mut dict, rdfs::DOMAIN.as_str()),
            iri(&mut dict, rdfs::RANGE.as_str()),
            iri(&mut dict, rdf::TYPE.as_str()),
        );
        // q owl:inverseOf r activates PropExpand; p has ONLY domain/range declarations.
        let mono = MonoOwl {
            inverse: [(q, vec![r]), (r, vec![q])].into_iter().collect(),
            ..MonoOwl::default()
        };
        let mut triples = vec![[p, dom, c], [p, rng, d], [a, p, b]];
        rdfs_closure(&mut dict, &mut triples, false, &mono);
        let set: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
        assert!(set.contains(&[a, ty, c]), "rdfs2 domain typing must survive active PropExpand");
        assert!(set.contains(&[b, ty, d]), "rdfs3 range typing must survive active PropExpand");
    }

    #[test]
    fn idempotent() {
        let mut dict = Dict::new();
        let (dog, animal, rex) = (ex(&mut dict, "Dog"), ex(&mut dict, "Animal"), ex(&mut dict, "rex"));
        let (sc, ty) = (iri(&mut dict, rdfs::SUB_CLASS_OF.as_str()), iri(&mut dict, rdf::TYPE.as_str()));
        let mut triples = vec![[dog, sc, animal], [rex, ty, dog]];
        materialize_rdfs(&mut dict, &mut triples);
        let n = triples.len();
        let added2 = materialize_rdfs(&mut dict, &mut triples);
        assert_eq!(added2, 0, "second materialization adds nothing");
        assert_eq!(triples.len(), n, "idempotent");
    }
}
