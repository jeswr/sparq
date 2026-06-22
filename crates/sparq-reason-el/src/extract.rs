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
use crate::Report;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

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
}

/// Extracts and normalizes the EL+⊥ TBox from `triples`, returning the normal-form axioms,
/// the name table, and a fresh [`Report`] tallying skipped (non-EL) axioms. Single-threaded.
pub fn extract(dict: &Dict, triples: &[[Id; 3]]) -> (Vec<Normal>, Names, Report) {
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
    report.named_classes = names.concept_count();
    (axioms, names, report)
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
