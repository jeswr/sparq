// [OPUS-4.8] sq-xetf7: differential-correctness fixtures for the EL+ RBox classifier (Phase E2:
// CR10 role inclusion + CR11 role composition / transitive roles / property chains).
//
// `#![cfg(feature = "rbox")]` — these run ONLY with the `rbox` feature on. A feature-matrix.yml
// leg (`sparq-reason-el (rbox)`) builds + tests + clippies this file with the feature enabled, so
// it is NOT a compiled-empty-and-never-run suite (the gap #1171 flagged); the default build (and
// ci.yml's `--workspace --all-targets` archive, which passes no `--features`) compiles it empty.
//
// ORACLE NOTE (same discipline as tests/differential.rs, flagged in the PR + issue #1163): the
// expected closures are hand-derived from the Baader–Brandt–Lutz / ELK CR1–CR5 + CR10/CR11
// calculus (the SAME role rules ELK implements) and asserted EXHAUSTIVELY — every expected
// subsumption holds AND no spurious one does. Each fixture is small enough to verify the role
// automaton by hand. Making ELK (a JVM + jar fetched over the network) a CI dependency is against
// the lean-default discipline; where ELK is available locally it can cross-check the same Turtle
// offline (a developer step, not a gate). E3 (sq-s2nob) revisits ELK-on-CI at biomedical scale.

#![cfg(feature = "rbox")]

use oxrdf::{NamedNode, Term as OTerm};
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;
use sparq_reason_el::Classifier;

const PRE: &str = r#"
    @prefix : <http://ex/> .
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
"#;

fn iri(dict: &Dict, frag: &str) -> Id {
    dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(format!(
        "http://ex/{frag}"
    ))))
}

/// Asserts the classifier's named-class subsumption relation EXACTLY equals `expected` over the
/// class set `classes` (every pair in `expected` holds; no other proper named subsumption does).
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
fn cr10_role_inclusion_propagates_existential() {
    // r ⊑ s,  A ⊑ ∃r.B,  ∃s.B ⊑ C.  CR10 lifts the (A,B) ∈ R(r) link to R(s), so CR4 over
    // `∃s.B ⊑ C` derives A ⊑ C — a subsumption the E1 classifier (no role hierarchy) misses.
    let ttl = format!(
        "{PRE}
         :r rdfs:subPropertyOf :s .
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         [ owl:onProperty :s ; owl:someValuesFrom :B ] rdfs:subClassOf :C ."
    );
    assert_closure(&ttl, &["A", "B", "C"], &[("A", "C")]);
}

#[test]
fn cr10_does_not_lift_downward() {
    // r ⊑ s,  A ⊑ ∃s.B,  ∃r.B ⊑ C.  The link is on s (a SUPER-role of r); CR10 only lifts
    // UPWARD, so it must NOT reach `∃r.B ⊑ C`. A ⊑ C must NOT be derived (soundness witness).
    let ttl = format!(
        "{PRE}
         :r rdfs:subPropertyOf :s .
         :A rdfs:subClassOf [ owl:onProperty :s ; owl:someValuesFrom :B ] .
         [ owl:onProperty :r ; owl:someValuesFrom :B ] rdfs:subClassOf :C ."
    );
    assert_closure(&ttl, &["A", "B", "C"], &[]);
}

#[test]
fn cr11_transitive_role() {
    // r transitive (r ∘ r ⊑ r),  A ⊑ ∃r.B,  B ⊑ ∃r.D,  ∃r.D ⊑ E.
    // (A,B) ∈ R(r) and (B,D) ∈ R(r) compose to (A,D) ∈ R(r) (CR11), so CR4 over `∃r.D ⊑ E`
    // derives A ⊑ E. Also B ⊑ E directly via (B,D). Transitive-role reasoning is SNOMED-critical.
    let ttl = format!(
        "{PRE}
         :r rdf:type owl:TransitiveProperty .
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :D ] .
         [ owl:onProperty :r ; owl:someValuesFrom :D ] rdfs:subClassOf :E ."
    );
    assert_closure(&ttl, &["A", "B", "D", "E"], &[("A", "E"), ("B", "E")]);
}

#[test]
fn cr11_transitive_three_hops_recomposes_derived_link() {
    // Stress the worklist: r transitive, A→B→C→D all via r. (A,B)∘(B,C)⇒(A,C), then the
    // DERIVED (A,C) must compose with (C,D) ⇒ (A,D) — a composed link re-feeding CR11. With
    // `∃r.D ⊑ E`, A ⊑ E only holds if the derived link recomposes. B ⊑ E and C ⊑ E too.
    let ttl = format!(
        "{PRE}
         :r rdf:type owl:TransitiveProperty .
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :C ] .
         :C rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :D ] .
         [ owl:onProperty :r ; owl:someValuesFrom :D ] rdfs:subClassOf :E ."
    );
    assert_closure(
        &ttl,
        &["A", "B", "C", "D", "E"],
        &[("A", "E"), ("B", "E"), ("C", "E")],
    );
}

#[test]
fn cr11_property_chain_right_identity() {
    // The SNOMED right-identity shape: partOf ∘ isa ⊑ partOf — here r ∘ s ⊑ r.
    // A ⊑ ∃r.B,  B ⊑ ∃s.D,  ∃r.D ⊑ E.  (A,B)∈R(r) ∘ (B,D)∈R(s) ⇒ (A,D)∈R(r) (CR11),
    // so CR4 over `∃r.D ⊑ E` derives A ⊑ E.
    let ttl = format!(
        "{PRE}
         :r owl:propertyChainAxiom ( :r :s ) .
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf [ owl:onProperty :s ; owl:someValuesFrom :D ] .
         [ owl:onProperty :r ; owl:someValuesFrom :D ] rdfs:subClassOf :E ."
    );
    assert_closure(&ttl, &["A", "B", "D", "E"], &[("A", "E")]);
}

#[test]
fn cr11_three_role_chain() {
    // Length-3 chain p ∘ q ∘ r ⊑ t (left-folded over a fresh role).
    // A ⊑ ∃p.B,  B ⊑ ∃q.C,  C ⊑ ∃r.D,  ∃t.D ⊑ G.  The three links compose to (A,D)∈R(t),
    // so CR4 over `∃t.D ⊑ G` derives A ⊑ G. No shorter prefix derives anything (t is the only
    // role with a `∃t.· ⊑ ·` axiom).
    let ttl = format!(
        "{PRE}
         :t owl:propertyChainAxiom ( :p :q :r ) .
         :A rdfs:subClassOf [ owl:onProperty :p ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf [ owl:onProperty :q ; owl:someValuesFrom :C ] .
         :C rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :D ] .
         [ owl:onProperty :t ; owl:someValuesFrom :D ] rdfs:subClassOf :G ."
    );
    assert_closure(&ttl, &["A", "B", "C", "D", "G"], &[("A", "G")]);
}

#[test]
fn cr10_transitive_inclusion_chain() {
    // r ⊑ s ⊑ t (a 2-level role hierarchy),  A ⊑ ∃r.B,  ∃t.B ⊑ C.
    // The reflexive-transitive closure lifts (A,B)∈R(r) all the way to R(t), so A ⊑ C.
    let ttl = format!(
        "{PRE}
         :r rdfs:subPropertyOf :s .  :s rdfs:subPropertyOf :t .
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         [ owl:onProperty :t ; owl:someValuesFrom :B ] rdfs:subClassOf :C ."
    );
    assert_closure(&ttl, &["A", "B", "C"], &[("A", "C")]);
}

#[test]
fn cr11_bottom_propagates_through_composed_link() {
    // Composition feeds CR5 too: r transitive,  A ⊑ ∃r.B,  B ⊑ ∃r.D,  D ⊑ ⊥ (via disjointness).
    // (A,D)∈R(r) by composition, and ⊥∈S(D) ⇒ ⊥∈S(A) (CR5) — A and B are both unsatisfiable.
    let ttl = format!(
        "{PRE}
         :r rdf:type owl:TransitiveProperty .
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :D ] .
         :D rdfs:subClassOf :P .  :D rdfs:subClassOf :Q .  :P owl:disjointWith :Q ."
    );
    let (dict, triples) = Graph::parse_to_triples(&ttl, "turtle").expect("parse");
    let h = Classifier::classify(&dict, &triples);
    let unsat = h.unsatisfiable_classes();
    assert!(unsat.contains(&iri(&dict, "D")), "D ⊑ ⊥ (P ⊓ Q ⊑ ⊥)");
    assert!(
        unsat.contains(&iri(&dict, "B")),
        "CR5: B ⊑ ∃r.D with D ⊑ ⊥ makes B ⊑ ⊥"
    );
    assert!(
        unsat.contains(&iri(&dict, "A")),
        "CR5 over the composed (A,D) link makes A ⊑ ⊥"
    );
}

#[test]
fn no_chain_no_spurious_composition() {
    // Sanity: WITHOUT any transitive/chain axiom, a two-hop existential structure must NOT
    // self-compose. r is a plain role; A ⊑ ∃r.B, B ⊑ ∃r.D, ∃r.D ⊑ E ⇒ only B ⊑ E (not A ⊑ E).
    let ttl = format!(
        "{PRE}
         :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
         :B rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :D ] .
         [ owl:onProperty :r ; owl:someValuesFrom :D ] rdfs:subClassOf :E ."
    );
    assert_closure(&ttl, &["A", "B", "D", "E"], &[("B", "E")]);
}
