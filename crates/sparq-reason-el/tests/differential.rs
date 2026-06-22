// [OPUS-4.8] sq-evb1: differential-correctness fixtures for the EL+⊥ classifier.
//
// ORACLE NOTE (best-judgment decision, flagged for the maintainer in the PR + a GH issue):
// the bead asks for a differential check vs ELK (Java, Apache-2.0). Making ELK a CI/build
// dependency (a JVM + the ELK jar, fetched over the network) is fragile and pushes a heavy,
// non-Rust toolchain onto the gate — against the lean-default discipline. So the oracle here
// is a set of EL ontologies whose COMPLETE subsumption closure is hand-derived from the
// Baader–Brandt–Lutz / ELK CR1–CR5 calculus (the same rules ELK implements) and asserted
// exhaustively: every expected subsumption is present AND no spurious one is. Each fixture is
// small enough to verify the closure by hand. Where ELK is available locally it can be run
// offline against the same Turtle to cross-check; that is a developer step, not a gate.
//
// This is the standard oracle discipline for a from-scratch reasoner whose external reference
// is not a viable CI dependency. Phase E3 (bead sq-s2nob) revisits scale with ELK-on-CI cross
// checks if/when the larger biomedical fixtures are wired in.

use oxrdf::{NamedNode, Term as OTerm};
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;
use sparq_reason_el::Classifier;

const PRE: &str = r#"
    @prefix : <http://ex/> .
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
"#;

fn iri(dict: &Dict, frag: &str) -> Id {
    dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(format!(
        "http://ex/{frag}"
    ))))
}

/// Asserts the classifier's named-class subsumption relation EXACTLY equals `expected` over
/// the class set `classes` (every pair `(sub, sup)` in `expected` holds and NO other proper
/// named subsumption among `classes` does). This is the differential-equality oracle.
fn assert_closure(ttl: &str, classes: &[&str], expected: &[(&str, &str)]) {
    let (dict, triples) = Graph::parse_to_triples(ttl, "turtle").expect("parse");
    let h = Classifier::classify(&dict, &triples);
    let exp: std::collections::HashSet<(&str, &str)> = expected.iter().copied().collect();
    for &sub in classes {
        for &sup in classes {
            if sub == sup {
                continue;
            }
            let got = h.is_subclass_of(iri(&dict, sub), iri(&dict, sup));
            let want = exp.contains(&(sub, sup));
            assert_eq!(
                got, want,
                "subsumption {sub} ⊑ {sup}: got {got}, want {want}"
            );
        }
    }
}

#[test]
fn el_chain_with_existential_traversal() {
    // A ⊑ ∃r.B,  B ⊑ C,  ∃r.C ⊑ D,  D ⊑ E.
    // Closure (CR1–CR4): B⊑C; A⊑D (CR4 via the r-successor); A⊑E, D⊑E (CR1).
    let ttl = format!(
        "{PRE}
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf :C .
         [ owl:onProperty :r ; owl:someValuesFrom :C ] rdfs:subClassOf :D .
         :D rdfs:subClassOf :E ."
    );
    assert_closure(
        &ttl,
        &["A", "B", "C", "D", "E"],
        &[("B", "C"), ("A", "D"), ("A", "E"), ("D", "E")],
    );
}

#[test]
fn el_conjunction_and_equivalence() {
    // A ≡ B ⊓ C,  D ⊑ A.  Closure: A⊑B, A⊑C (from ≡ RHS split); D⊑A, D⊑B, D⊑C.
    // Note B⊓C ⊑ A means anything with both B and C is an A — but nothing here has both
    // except via A, so the only sub of A is D (and A itself).
    let ttl = format!(
        "{PRE}
         :A owl:equivalentClass [ owl:intersectionOf ( :B :C ) ] .
         :D rdfs:subClassOf :A ."
    );
    assert_closure(
        &ttl,
        &["A", "B", "C", "D"],
        &[("A", "B"), ("A", "C"), ("D", "A"), ("D", "B"), ("D", "C")],
    );
}

#[test]
fn el_conjunction_lhs_triggers_subsumption() {
    // B ⊓ C ⊑ A,  X ⊑ B,  X ⊑ C.  Closure: X⊑A (CR2), plus X⊑B, X⊑C asserted.
    let ttl = format!(
        "{PRE}
         [ owl:intersectionOf ( :B :C ) ] rdfs:subClassOf :A .
         :X rdfs:subClassOf :B . :X rdfs:subClassOf :C ."
    );
    assert_closure(
        &ttl,
        &["A", "B", "C", "X"],
        &[("X", "A"), ("X", "B"), ("X", "C")],
    );
}

#[test]
fn el_bottom_propagates_through_existential() {
    // CR5: A ⊑ ∃r.B,  B ⊑ ⊥ (via disjointness)  ⊨  A ⊑ ⊥.
    // Encode B ⊑ ⊥ as B ⊑ (P ⊓ Q) with P disjointWith Q.
    let ttl = format!(
        "{PRE}
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf :P . :B rdfs:subClassOf :Q .
         :P owl:disjointWith :Q ."
    );
    let (dict, triples) = Graph::parse_to_triples(&ttl, "turtle").expect("parse");
    let h = Classifier::classify(&dict, &triples);
    let unsat = h.unsatisfiable_classes();
    assert!(
        unsat.contains(&iri(&dict, "B")),
        "B is unsatisfiable (P ⊓ Q ⊑ ⊥)"
    );
    assert!(
        unsat.contains(&iri(&dict, "A")),
        "CR5: A ⊑ ∃r.B with B ⊑ ⊥ makes A ⊑ ⊥"
    );
}

#[test]
fn diamond_no_spurious_cross_subsumption() {
    // A ⊑ B, A ⊑ C, B ⊑ D, C ⊑ D. B and C must NOT subsume each other.
    let ttl = format!(
        "{PRE}
         :A rdfs:subClassOf :B . :A rdfs:subClassOf :C .
         :B rdfs:subClassOf :D . :C rdfs:subClassOf :D ."
    );
    assert_closure(
        &ttl,
        &["A", "B", "C", "D"],
        &[("A", "B"), ("A", "C"), ("A", "D"), ("B", "D"), ("C", "D")],
    );
}

#[test]
fn two_hop_existential_chain() {
    // A ⊑ ∃r.B,  ∃r.B ⊑ C,  C ⊑ ∃s.D,  ∃s.D ⊑ E.
    // Closure: A⊑C (CR4 hop 1), A⊑∃s.D via C, A⊑E (CR4 hop 2 through the s-successor), C⊑E.
    let ttl = format!(
        "{PRE}
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         [ owl:onProperty :r ; owl:someValuesFrom :B ] rdfs:subClassOf :C .
         :C rdfs:subClassOf [ owl:onProperty :s ; owl:someValuesFrom :D ] .
         [ owl:onProperty :s ; owl:someValuesFrom :D ] rdfs:subClassOf :E ."
    );
    assert_closure(
        &ttl,
        &["A", "B", "C", "D", "E"],
        &[("A", "C"), ("A", "E"), ("C", "E")],
    );
}

#[test]
fn rl_incompleteness_witness_is_complete_under_el() {
    // The exact spike §1.3 ontology that OWL 2 RL classifies INCOMPLETELY. Under EL the full
    // closure includes A ⊑ D — the subsumption RL silently omits.
    let ttl = format!(
        "{PRE}
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf :C .
         [ owl:onProperty :r ; owl:someValuesFrom :C ] rdfs:subClassOf :D ."
    );
    assert_closure(&ttl, &["A", "B", "C", "D"], &[("B", "C"), ("A", "D")]);
}
