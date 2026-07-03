//! TBox-capture accounting tests — sq-pbz04.3.3 [SONNET-4.6]
//!
//! Validates the total-accounting invariant: every `rdfs:`/`owl:` predicate triple is EITHER
//! captured (positive inclusion), counted in `skipped` (non-QL axiom), counted in
//! `consistency_relevant` (negative/disjointness axiom), or classified as a known restriction
//! sub-predicate / annotation (silently ignored). `unrecognised_schema` catches the remainder.
//! `fully_captured()` is a decidable predicate over `skipped` and `unrecognised_schema`.
//!
//! Tests run in BOTH feature states (default OFF and `--features experimental`): `TBox::extract`
//! is always compiled (not behind the `experimental` gate), so these tests are unconditional.

use oxrdf::Triple;
use sparq_reason_ql::{Basic, ConceptInclusion, Role, TBox};
use std::str::FromStr;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EX: &str = "http://example.org/";
const OWL: &str = "http://www.w3.org/2002/07/owl#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";

fn triple(s: &str) -> Triple {
    Triple::from_str(s).unwrap_or_else(|e| panic!("bad N-Triple {s:?}: {e}"))
}

fn triples(lines: &[&str]) -> Vec<Triple> {
    lines.iter().map(|l| triple(l)).collect()
}

fn iri(local: &str) -> String {
    format!("{EX}{local}")
}

// ---------------------------------------------------------------------------
// A1 — owl:equivalentClass rewrites BOTH directions [SONNET-4.6]
// ---------------------------------------------------------------------------

#[test]
fn equivalent_class_captures_both_directions() {
    // :A owl:equivalentClass :B  =>  A ⊑ B  AND  B ⊑ A
    let ts = triples(&[&format!(
        "<{EX}A> <{OWL}equivalentClass> <{EX}B> ."
    )]);
    let tbox = TBox::extract(&ts);

    let a_sub_b = ConceptInclusion {
        sub: Basic::Class(iri("A")),
        sup: iri("B"),
    };
    let b_sub_a = ConceptInclusion {
        sub: Basic::Class(iri("B")),
        sup: iri("A"),
    };

    assert!(
        tbox.concept_incl.contains(&a_sub_b),
        "expected A ⊑ B from equivalentClass; got {:?}",
        tbox.concept_incl
    );
    assert!(
        tbox.concept_incl.contains(&b_sub_a),
        "expected B ⊑ A from equivalentClass; got {:?}",
        tbox.concept_incl
    );
    assert_eq!(
        tbox.skipped, 0,
        "named-class equivalentClass must not count as skipped"
    );
    assert_eq!(
        tbox.unrecognised_schema, 0,
        "equivalentClass is a recognised predicate"
    );
}

#[test]
fn equivalent_class_complex_object_counts_skipped() {
    // :A owl:equivalentClass _:bn  (blank node = complex expression — outside QL rewriting)
    // The triple DOES still need to be accounted for (not silently ignored).
    let ts = triples(&[&format!(
        "<{EX}A> <{OWL}equivalentClass> _:complex ."
    )]);
    let tbox = TBox::extract(&ts);

    assert_eq!(
        tbox.concept_incl.len(),
        0,
        "complex equivalentClass must not produce an inclusion"
    );
    assert_eq!(
        tbox.skipped, 1,
        "complex equivalentClass counts as skipped (outside QL rewriting)"
    );
}

// ---------------------------------------------------------------------------
// A2 — owl:equivalentProperty rewrites BOTH directions [SONNET-4.6]
// ---------------------------------------------------------------------------

#[test]
fn equivalent_property_captures_both_directions() {
    // :manages owl:equivalentProperty :worksFor  =>  manages ⊑ worksFor  AND  worksFor ⊑ manages
    let ts = triples(&[&format!(
        "<{EX}manages> <{OWL}equivalentProperty> <{EX}worksFor> ."
    )]);
    let tbox = TBox::extract(&ts);

    let r_manages = Role::named(iri("manages"));
    let r_works_for = Role::named(iri("worksFor"));

    let manages_sub_works =
        tbox.role_incl.contains(&(r_manages.clone(), r_works_for.clone()));
    let works_sub_manages =
        tbox.role_incl.contains(&(r_works_for.clone(), r_manages.clone()));

    assert!(
        manages_sub_works,
        "expected manages ⊑ worksFor; got {:?}",
        tbox.role_incl
    );
    assert!(
        works_sub_manages,
        "expected worksFor ⊑ manages; got {:?}",
        tbox.role_incl
    );
    assert_eq!(tbox.skipped, 0, "named-role equivalentProperty must not count as skipped");
    assert_eq!(
        tbox.unrecognised_schema, 0,
        "equivalentProperty is a recognised predicate"
    );
}

// ---------------------------------------------------------------------------
// A3 — owl:disjointWith reports consistency_relevant > 0 [SONNET-4.6]
// ---------------------------------------------------------------------------

#[test]
fn disjoint_with_increments_consistency_relevant() {
    let ts = triples(&[&format!(
        "<{EX}A> <{OWL}disjointWith> <{EX}B> ."
    )]);
    let tbox = TBox::extract(&ts);

    assert!(
        tbox.consistency_relevant > 0,
        "owl:disjointWith must increment consistency_relevant"
    );
    assert_eq!(
        tbox.concept_incl.len(),
        0,
        "owl:disjointWith must NOT produce a positive inclusion (unsound)"
    );
    assert_eq!(tbox.skipped, 0, "owl:disjointWith is not 'outside QL' — it is QL-legal");
    assert_eq!(tbox.unrecognised_schema, 0, "owl:disjointWith is a recognised predicate");
}

#[test]
fn property_disjoint_with_increments_consistency_relevant() {
    let ts = triples(&[&format!(
        "<{EX}p> <{OWL}propertyDisjointWith> <{EX}q> ."
    )]);
    let tbox = TBox::extract(&ts);

    assert!(
        tbox.consistency_relevant > 0,
        "owl:propertyDisjointWith must increment consistency_relevant"
    );
    assert_eq!(tbox.role_incl.len(), 0, "owl:propertyDisjointWith must NOT produce a role inclusion");
    assert_eq!(tbox.skipped, 0, "owl:propertyDisjointWith is QL-legal");
}

#[test]
fn complement_of_increments_consistency_relevant() {
    let ts = triples(&[&format!(
        "<{EX}A> <{OWL}complementOf> <{EX}B> ."
    )]);
    let tbox = TBox::extract(&ts);

    assert!(
        tbox.consistency_relevant > 0,
        "owl:complementOf must increment consistency_relevant"
    );
    assert_eq!(
        tbox.concept_incl.len(),
        0,
        "owl:complementOf must NOT produce a positive inclusion"
    );
}

// ---------------------------------------------------------------------------
// A4 — allValuesFrom still counted as skipped (existing behaviour preserved)
// ---------------------------------------------------------------------------

#[test]
fn all_values_from_still_counted_skipped() {
    // :A rdfs:subClassOf [ owl:onProperty :p ; owl:allValuesFrom :B ]
    // The restriction blank node is poisoned (allValuesFrom in NON_QL_RESTRICTION); add_subclass
    // cannot resolve it → skipped += 1. This test guards the existing semantics against regression.
    let ts = triples(&[
        &format!("<{EX}A> <{RDFS}subClassOf> _:r ."),
        &format!("_:r <{OWL}onProperty> <{EX}p> ."),
        &format!("_:r <{OWL}allValuesFrom> <{EX}B> ."),
    ]);
    let tbox = TBox::extract(&ts);

    assert_eq!(
        tbox.skipped, 1,
        "allValuesFrom restriction must count as skipped (non-QL rewriting construct)"
    );
    assert_eq!(
        tbox.concept_incl.len(),
        0,
        "allValuesFrom must NOT produce a positive inclusion"
    );
    // The restriction sub-predicate triples themselves must NOT count as unrecognised_schema.
    assert_eq!(
        tbox.unrecognised_schema, 0,
        "owl:onProperty and owl:allValuesFrom are known restriction sub-predicates"
    );
}

// ---------------------------------------------------------------------------
// A5 — fully_captured() correctness [SONNET-4.6]
// ---------------------------------------------------------------------------

#[test]
fn fully_captured_false_when_skipped_nonzero() {
    // allValuesFrom-based restriction → skipped = 1 → not fully captured.
    let ts = triples(&[
        &format!("<{EX}A> <{RDFS}subClassOf> _:r ."),
        &format!("_:r <{OWL}onProperty> <{EX}p> ."),
        &format!("_:r <{OWL}allValuesFrom> <{EX}B> ."),
    ]);
    let tbox = TBox::extract(&ts);
    assert!(!tbox.fully_captured(), "skipped > 0 => fully_captured() must return false");
}

#[test]
fn fully_captured_false_when_unrecognised_schema_nonzero() {
    // owl:unionOf is in the owl: namespace but NOT a recognised TBox predicate or restriction
    // sub-predicate or annotation → unrecognised_schema += 1.
    let ts = triples(&[&format!(
        "<{EX}A> <{OWL}unionOf> _:list ."
    )]);
    let tbox = TBox::extract(&ts);
    assert!(
        tbox.unrecognised_schema > 0,
        "owl:unionOf as predicate is unrecognised schema vocabulary; got unrecognised_schema={}",
        tbox.unrecognised_schema
    );
    assert!(
        !tbox.fully_captured(),
        "unrecognised_schema > 0 => fully_captured() must return false"
    );
}

#[test]
fn fully_captured_true_in_total_case() {
    // A TBox that uses ONLY recognised QL predicates with valid shapes: subClassOf, subPropertyOf,
    // equivalentClass, equivalentProperty, inverseOf — all named-node sides.
    let ts = triples(&[
        &format!("<{EX}Manager> <{RDFS}subClassOf> <{EX}Employee> ."),
        &format!("<{EX}manages> <{RDFS}subPropertyOf> <{EX}employs> ."),
        &format!("<{EX}Admin> <{OWL}equivalentClass> <{EX}Manager> ."),
        &format!("<{EX}worksFor> <{OWL}equivalentProperty> <{EX}employedBy> ."),
        &format!("<{EX}employs> <{OWL}inverseOf> <{EX}worksFor> ."),
        // RDFS annotation — silently ignored, not unrecognised_schema.
        &format!("<{EX}Employee> <{RDFS}comment> \"An employee\" ."),
    ]);
    let tbox = TBox::extract(&ts);

    assert_eq!(tbox.skipped, 0, "all axioms in this TBox are QL-capturable");
    assert_eq!(tbox.unrecognised_schema, 0, "no unrecognised schema predicates");
    assert!(tbox.fully_captured(), "total case: fully_captured() must return true");
}

#[test]
fn fully_captured_consistency_relevant_does_not_block() {
    // A TBox with only disjointWith — consistency-relevant but NOT blocking fully_captured().
    // (The crate documents it does not check consistency; its absence from the rewriting is
    // already accounted for by the caller knowing the crate's boundary.)
    let ts = triples(&[
        &format!("<{EX}A> <{RDFS}subClassOf> <{EX}B> ."),
        &format!("<{EX}A> <{OWL}disjointWith> <{EX}C> ."),
    ]);
    let tbox = TBox::extract(&ts);

    assert!(
        tbox.consistency_relevant > 0,
        "disjointWith is consistency-relevant"
    );
    assert_eq!(tbox.skipped, 0);
    assert_eq!(tbox.unrecognised_schema, 0);
    assert!(
        tbox.fully_captured(),
        "consistency_relevant does NOT block fully_captured()"
    );
}

// ---------------------------------------------------------------------------
// A6 — annotation predicates not counted as unrecognised_schema [SONNET-4.6]
// ---------------------------------------------------------------------------

#[test]
fn rdfs_annotation_predicates_not_counted_as_unrecognised() {
    // rdfs:label, rdfs:comment, rdfs:isDefinedBy, rdfs:seeAlso are annotations.
    let ts = triples(&[
        &format!("<{EX}A> <{RDFS}label> \"Class A\" ."),
        &format!("<{EX}A> <{RDFS}comment> \"A comment.\" ."),
        &format!("<{EX}A> <{RDFS}isDefinedBy> <{EX}onto> ."),
        &format!("<{EX}A> <{RDFS}seeAlso> <{EX}docs> ."),
    ]);
    let tbox = TBox::extract(&ts);

    assert_eq!(tbox.unrecognised_schema, 0, "rdfs annotation predicates must not be unrecognised");
    assert_eq!(tbox.skipped, 0);
}

#[test]
fn owl_annotation_predicates_not_counted_as_unrecognised() {
    // owl:deprecated, owl:versionInfo — annotation predicates.
    let ts = triples(&[
        &format!("<{EX}onto> <{OWL}deprecated> \"true\"^^<http://www.w3.org/2001/XMLSchema#boolean> ."),
        &format!("<{EX}onto> <{OWL}versionInfo> \"1.0\" ."),
    ]);
    let tbox = TBox::extract(&ts);

    assert_eq!(tbox.unrecognised_schema, 0, "owl annotation predicates must not be unrecognised");
}

// ---------------------------------------------------------------------------
// A7 — ABox triples (non-rdfs/owl predicates) are silently ignored [SONNET-4.6]
// ---------------------------------------------------------------------------

#[test]
fn abox_triples_do_not_affect_any_tally() {
    // A plain data triple with a custom predicate: not in rdfs:/owl: namespace.
    let ts = triples(&[&format!("<{EX}alice> <{EX}name> \"Alice\" .")]);
    let tbox = TBox::extract(&ts);

    assert_eq!(tbox.concept_incl.len(), 0);
    assert_eq!(tbox.role_incl.len(), 0);
    assert_eq!(tbox.skipped, 0);
    assert_eq!(tbox.consistency_relevant, 0);
    assert_eq!(tbox.unrecognised_schema, 0);
    assert!(tbox.fully_captured(), "no schema at all = fully captured");
}
