// [OPUS-4.8] sq-evb1: extract an EL+⊥ TBox from dict-encoded RDF triples.
//
// Reads the OWL axioms an EL classifier needs out of the `(Dict, Vec<[Id;3]>)` substrate and
// produces a `Vec<Normal>` plus the `Names` table. Recognised constructs (EL+⊥ minus RBox):
//
//   rdfs:subClassOf            C ⊑ D
//   owl:equivalentClass        C ≡ D  (both directions)
//   owl:intersectionOf (list)  C ⊓ … ⊓ Cn  (as a class expression node)
//   owl:Restriction + owl:onProperty + owl:someValuesFrom   ∃r.C
//   owl:disjointWith           C ⊓ D ⊑ ⊥
//   owl:Thing / owl:Nothing    ⊤ / ⊥
//
// Anything NOT in this fragment (unionOf, complementOf, allValuesFrom, cardinality, hasValue,
// data-properties, nominals, property chains, …) is OUT OF EL+⊥-MVP and is recorded as a skip
// in the [`crate::Report`] rather than silently dropped — so a user running `el` over a
// non-EL ontology gets an honest "n axioms outside the EL fragment were ignored" count.

use crate::normal::{Expr, Names, Normal, Normalizer, BOTTOM, TOP};
#[cfg(feature = "rbox")]
use crate::normal::{Role, RoleAxiom};
use crate::Report;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

/// Everything the extractor reads out of the RDF substrate: the normalized concept axioms, the
/// name table, the [`Report`], and — only under the `rbox` feature — the normalized RBox role
/// axioms (E2). Keeping the role axioms in a feature-gated field means the default build's
/// extraction is byte-for-byte the E1 path.
pub struct Extracted {
    pub axioms: Vec<Normal>,
    pub names: Names,
    pub report: Report,
    /// Normalized RBox axioms (role inclusions + compositions). Empty/absent without `rbox`.
    #[cfg(feature = "rbox")]
    pub role_axioms: Vec<RoleAxiom>,
}

const OWL: &str = "http://www.w3.org/2002/07/owl#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";

/// OWL class-expression / restriction predicates that lie OUTSIDE the EL+⊥-MVP fragment. A
/// class node carrying any of these is non-EL: [`decode`] returns `None` so the enclosing
/// axiom is recorded as a skip rather than the node being mistaken for an opaque named class.
const NON_EL_MARKERS: &[&str] = &[
    "unionOf",       // disjunction
    "complementOf",  // negation
    "allValuesFrom", // universal restriction
    "hasValue",      // value restriction (nominal-flavoured)
    "oneOf",         // nominals / enumeration
    "minCardinality",
    "maxCardinality",
    "cardinality",
    "minQualifiedCardinality",
    "maxQualifiedCardinality",
    "qualifiedCardinality",
    "hasSelf",
];

/// The dict ids of the vocabulary terms the extractor matches on. A term absent from the
/// store gets `NO_ID` from `lookup` (matching nothing), exactly like the RL path.
struct Vocab {
    ty: Id,
    sub_class_of: Id,
    equivalent_class: Id,
    intersection_of: Id,
    on_property: Id,
    some_values_from: Id,
    disjoint_with: Id,
    restriction: Id,
    rdf_first: Id,
    rdf_rest: Id,
    rdf_nil: Id,
    thing: Id,
    nothing: Id,
    /// Predicate ids whose presence on a class node makes it non-EL (see [`NON_EL_MARKERS`]).
    non_el: FxHashSet<Id>,
    /// RBox vocabulary — only consulted under the `rbox` feature (E2).
    #[cfg(feature = "rbox")]
    sub_property_of: Id,
    #[cfg(feature = "rbox")]
    property_chain_axiom: Id,
    #[cfg(feature = "rbox")]
    transitive_property: Id,
}

impl Vocab {
    fn intern(dict: &Dict) -> Vocab {
        use oxrdf::{NamedNode, Term as OTerm};
        let look = |iri: String| dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(iri)));
        Vocab {
            ty: look(format!("{}type", RDF)),
            sub_class_of: look(format!("{}subClassOf", RDFS)),
            equivalent_class: look(format!("{}equivalentClass", OWL)),
            intersection_of: look(format!("{}intersectionOf", OWL)),
            on_property: look(format!("{}onProperty", OWL)),
            some_values_from: look(format!("{}someValuesFrom", OWL)),
            disjoint_with: look(format!("{}disjointWith", OWL)),
            restriction: look(format!("{}Restriction", OWL)),
            rdf_first: look(format!("{}first", RDF)),
            rdf_rest: look(format!("{}rest", RDF)),
            rdf_nil: look(format!("{}nil", RDF)),
            thing: look(format!("{}Thing", OWL)),
            nothing: look(format!("{}Nothing", OWL)),
            non_el: NON_EL_MARKERS
                .iter()
                .map(|m| look(format!("{}{}", OWL, m)))
                .collect(),
            #[cfg(feature = "rbox")]
            sub_property_of: look(format!("{}subPropertyOf", RDFS)),
            #[cfg(feature = "rbox")]
            property_chain_axiom: look(format!("{}propertyChainAxiom", OWL)),
            #[cfg(feature = "rbox")]
            transitive_property: look(format!("{}TransitiveProperty", OWL)),
        }
    }
}

/// The structural index the extractor builds in one pass over the triples: enough to decode
/// restrictions and intersection lists into [`Expr`] trees without re-scanning.
#[derive(Default)]
struct Idx {
    sub_class: Vec<(Id, Id)>,      // (C, D) from rdfs:subClassOf
    equiv_class: Vec<(Id, Id)>,    // (C, D) from owl:equivalentClass
    disjoint: Vec<(Id, Id)>,       // (C, D) from owl:disjointWith
    on_prop: FxHashMap<Id, Id>,    // restriction node -> property
    svf: FxHashMap<Id, Id>,        // restriction node -> someValuesFrom filler
    inter_head: FxHashMap<Id, Id>, // class node -> intersectionOf list head
    first: FxHashMap<Id, Id>,      // rdf:first edges
    rest: FxHashMap<Id, Id>,       // rdf:rest edges
    is_restriction: FxHashSet<Id>, // nodes typed owl:Restriction
    non_el_node: FxHashSet<Id>,    // nodes carrying a non-EL marker predicate
    #[cfg(feature = "rbox")]
    sub_property: Vec<(Id, Id)>, // (r, s) from rdfs:subPropertyOf — a role inclusion r ⊑ s
    #[cfg(feature = "rbox")]
    chain_head: FxHashMap<Id, Id>, // super-property -> owl:propertyChainAxiom list head
    #[cfg(feature = "rbox")]
    transitive: Vec<Id>, // properties typed owl:TransitiveProperty
}

/// Extracts and normalizes the EL+⊥ TBox from `triples`, returning the [`Extracted`] bundle
/// (normal-form concept axioms, the name table, the [`Report`], and — under `rbox` — the
/// normalized RBox role axioms). Single-threaded.
pub fn extract(dict: &Dict, triples: &[[Id; 3]]) -> Extracted {
    let v = Vocab::intern(dict);
    let mut idx = Idx::default();
    for &[s, p, o] in triples {
        if p == v.sub_class_of {
            idx.sub_class.push((s, o));
        } else if p == v.equivalent_class {
            idx.equiv_class.push((s, o));
        } else if p == v.disjoint_with {
            idx.disjoint.push((s, o));
        } else if p == v.on_property {
            idx.on_prop.insert(s, o);
        } else if p == v.some_values_from {
            idx.svf.insert(s, o);
        } else if p == v.intersection_of {
            idx.inter_head.insert(s, o);
        } else if p == v.rdf_first {
            idx.first.insert(s, o);
        } else if p == v.rdf_rest {
            idx.rest.insert(s, o);
        } else if p == v.ty && o == v.restriction {
            idx.is_restriction.insert(s);
        } else if v.non_el.contains(&p) {
            idx.non_el_node.insert(s);
        } else {
            #[cfg(feature = "rbox")]
            extract_rbox_triple(&mut idx, &v, s, p, o);
        }
    }

    let mut names = Names::new();
    // Pre-seed ⊤/⊥ so the recognisers route owl:Thing/owl:Nothing dict ids to TOP/BOTTOM.
    let mut report = Report::default();
    let mut norm = Normalizer::new(&mut names);

    // Decode every subClassOf / equivalentClass / disjointWith axiom into Expr -> Expr and
    // hand each to the normalizer. The `decode` closure resolves a class node into an Expr,
    // returning None (and bumping the skip count) when the node is a non-EL construct.
    let process = |a: Id, b: Id, equiv: bool, report: &mut Report, norm: &mut Normalizer| {
        let mut cache = FxHashMap::default();
        let lhs = decode(a, &idx, &v, norm.names, &mut cache, 0);
        let rhs = decode(b, &idx, &v, norm.names, &mut cache, 0);
        match (lhs, rhs) {
            (Some(l), Some(r)) => {
                norm.add_sub(&l, &r);
                if equiv {
                    norm.add_sub(&r, &l);
                }
            }
            _ => report.skipped_axioms += 1,
        }
    };

    for &(a, b) in &idx.sub_class {
        process(a, b, false, &mut report, &mut norm);
    }
    for &(a, b) in &idx.equiv_class {
        process(a, b, true, &mut report, &mut norm);
    }
    // disjointWith(C, D)  ⇒  C ⊓ D ⊑ ⊥.
    for &(a, b) in &idx.disjoint {
        let mut cache = FxHashMap::default();
        let lhs = decode(a, &idx, &v, norm.names, &mut cache, 0);
        let rhs = decode(b, &idx, &v, norm.names, &mut cache, 0);
        match (lhs, rhs) {
            (Some(l), Some(r)) => norm.add_sub(&Expr::And(vec![l, r]), &Expr::Atom(BOTTOM)),
            _ => report.skipped_axioms += 1,
        }
    }

    let axioms = norm.finish();

    // RBox normalization (E2): role inclusions + property chains + transitive roles into
    // [`RoleAxiom`] forms. Done here while `names` is still borrowable so role ids agree with
    // the existential links minted during concept decoding. No-op without the `rbox` feature.
    #[cfg(feature = "rbox")]
    let role_axioms = normalize_rbox(&idx, &v, &mut names);

    report.named_classes = names.concept_count();
    Extracted {
        axioms,
        names,
        report,
        #[cfg(feature = "rbox")]
        role_axioms,
    }
}

/// Routes one RBox triple into the structural index: `rdfs:subPropertyOf` (role inclusion),
/// `owl:propertyChainAxiom` (the super-property is the SUBJECT, the chain list is the object),
/// or an `owl:TransitiveProperty` type assertion. Bare datatype-property `subPropertyOf`s are
/// captured too but never fire — they cannot appear in an `∃r.C` link, so a spurious inclusion
/// among data properties is inert (kept simple rather than requiring property-type declarations).
#[cfg(feature = "rbox")]
fn extract_rbox_triple(idx: &mut Idx, v: &Vocab, s: Id, p: Id, o: Id) {
    if p == v.sub_property_of {
        idx.sub_property.push((s, o));
    } else if p == v.property_chain_axiom {
        idx.chain_head.insert(s, o);
    } else if p == v.ty && o == v.transitive_property {
        idx.transitive.push(s);
    }
}

/// Builds the normalized [`RoleAxiom`] list from the RBox structural index, minting role ids
/// through the shared [`Names`] table. A property chain of length `n` is left-folded into `n-1`
/// binary compositions over fresh intermediate roles; a transitive property `r` becomes the
/// composition `r ∘ r ⊑ r`. A degenerate chain (length 0/1) is treated as a plain inclusion.
#[cfg(feature = "rbox")]
fn normalize_rbox(idx: &Idx, v: &Vocab, names: &mut Names) -> Vec<RoleAxiom> {
    let mut out: Vec<RoleAxiom> = Vec::new();
    // r ⊑ s.
    for &(r, s) in &idx.sub_property {
        let (ri, si) = (names.role(r), names.role(s));
        out.push(RoleAxiom::Sub(ri, si));
    }
    // owl:propertyChainAxiom: r1 ∘ … ∘ rn ⊑ super.
    for (&super_prop, &head) in &idx.chain_head {
        let members = decode_list(head, idx, v);
        let chain: Vec<Role> = members.iter().map(|&m| names.role(m)).collect();
        let sup = names.role(super_prop);
        fold_chain(&chain, sup, names, &mut out);
    }
    // owl:TransitiveProperty(r) ≡ r ∘ r ⊑ r.
    for &r in &idx.transitive {
        let ri = names.role(r);
        out.push(RoleAxiom::Chain(ri, ri, ri));
    }
    out
}

/// Left-folds a property chain `r1 ∘ … ∘ rn ⊑ sup` into binary [`RoleAxiom::Chain`]s:
/// `r1 ∘ r2 ⊑ f1`, `f1 ∘ r3 ⊑ f2`, …, `f_{n-2} ∘ rn ⊑ sup`. Length-1 chains degenerate to an
/// inclusion `r1 ⊑ sup`; a length-0 chain (malformed) is dropped.
#[cfg(feature = "rbox")]
fn fold_chain(chain: &[Role], sup: Role, names: &mut Names, out: &mut Vec<RoleAxiom>) {
    match chain {
        [] => {}
        [r1] => out.push(RoleAxiom::Sub(*r1, sup)),
        [r1, r2] => out.push(RoleAxiom::Chain(*r1, *r2, sup)),
        [first, rest @ ..] => {
            let mut acc = *first;
            // rest has len ≥ 2 here; compose all but the last over fresh roles, last lands on sup.
            for &next in &rest[..rest.len() - 1] {
                let f = names.fresh_role();
                out.push(RoleAxiom::Chain(acc, next, f));
                acc = f;
            }
            let last = rest[rest.len() - 1];
            out.push(RoleAxiom::Chain(acc, last, sup));
        }
    }
}

/// Resolves a class node (named class, ⊤, ⊥, an `owl:Restriction` node, or an
/// `owl:intersectionOf` node) into an [`Expr`]. Returns `None` for any node that uses a
/// construct outside EL+⊥ (so the caller can record a skip). `depth` guards against cyclic
/// blank-node structures (a malformed list pointing at itself).
fn decode(
    node: Id,
    idx: &Idx,
    v: &Vocab,
    names: &mut Names,
    cache: &mut FxHashMap<Id, Option<Expr>>,
    depth: u32,
) -> Option<Expr> {
    if depth > 256 {
        return None; // pathological nesting / cycle: treat as non-EL.
    }
    if node == v.thing {
        return Some(Expr::Atom(TOP));
    }
    if node == v.nothing {
        return Some(Expr::Atom(BOTTOM));
    }
    if let Some(hit) = cache.get(&node) {
        return hit.clone();
    }

    // A node carrying a non-EL class-expression marker (unionOf / complementOf / cardinality /
    // allValuesFrom / oneOf / hasValue / …) is outside EL+⊥: report it as a skip, do NOT
    // mistake it for an opaque named class (that would silently drop its real semantics).
    if idx.non_el_node.contains(&node) {
        cache.insert(node, None);
        return None;
    }

    // A restriction node: must carry exactly onProperty + someValuesFrom for EL+⊥.
    if idx.is_restriction.contains(&node)
        || idx.svf.contains_key(&node)
        || idx.on_prop.contains_key(&node)
    {
        let result = match (idx.on_prop.get(&node), idx.svf.get(&node)) {
            (Some(&p), Some(&filler)) => {
                let role = names.role(p);
                decode(filler, idx, v, names, cache, depth + 1)
                    .map(|f| Expr::Exists(role, Box::new(f)))
            }
            // A restriction node with onProperty but a non-someValuesFrom body (allValuesFrom,
            // cardinality, hasValue, …) is outside EL+⊥-MVP.
            _ => None,
        };
        cache.insert(node, result.clone());
        return result;
    }

    // An intersection node: decode the RDF list, recursively decoding each member.
    if let Some(&head) = idx.inter_head.get(&node) {
        let members_ids = decode_list(head, idx, v);
        let mut parts = Vec::with_capacity(members_ids.len());
        for m in members_ids {
            match decode(m, idx, v, names, cache, depth + 1) {
                Some(e) => parts.push(e),
                None => {
                    cache.insert(node, None);
                    return None;
                }
            }
        }
        let result = if parts.len() >= 2 {
            Some(Expr::And(parts))
        } else if parts.len() == 1 {
            parts.into_iter().next()
        } else {
            None
        };
        cache.insert(node, result.clone());
        return result;
    }

    // Otherwise it is a plain named class.
    let c = names.class(node);
    let result = Some(Expr::Atom(c));
    cache.insert(node, result.clone());
    result
}

/// Walks an `rdf:first`/`rdf:rest` list from `head`, returning member node ids. Bounded by
/// the number of list cells seen (guards a malformed self-referential `rdf:rest`).
fn decode_list(head: Id, idx: &Idx, v: &Vocab) -> Vec<Id> {
    let mut out = Vec::new();
    let mut cur = head;
    let mut seen = FxHashSet::default();
    while cur != v.rdf_nil && seen.insert(cur) {
        match idx.first.get(&cur) {
            Some(&m) => out.push(m),
            None => break,
        }
        match idx.rest.get(&cur) {
            Some(&n) => cur = n,
            None => break,
        }
    }
    out
}
