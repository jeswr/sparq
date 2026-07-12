// [FABLE-5] sq-pbz04.4.4 (epic sq-pbz04.4) — integration tests for the L4 fragment-dispatch
// Direct-Semantics checker + entailment-by-refutation (design record
// research/owl2-direct-semantics-scoping.md §4).
//
// 🤖 SPARQ agent. Every test goes end-to-end through the REAL pipeline: Turtle → Dict →
// fail-closed L1 extraction → L2 profile dispatch → the RL/EL/QL/ALCH branch under test.
// The bead's three named acceptance cases are here: the divergence-guarded RL case
// (`rl_divergence_guard_*`), the EL skipped-axioms case (`el_skipped_axioms_*`), and the
// fresh-class refutation case (`entailment_role_assertion_*`). Verdicts are asserted as
// exact enum variants (mutation-robust), and each public API item has a direct test.

#![cfg(feature = "dispatch")]

use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;
use sparq_reason_dl::check::{
    Branch, ConsistencyVerdict, DirectChecker, EntailmentVerdict, UnknownReason,
};
use sparq_reason_dl::tableau::Budget;

const PRE: &str = r#"
    @prefix : <http://ex/> .
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
"#;

/// Parse one Turtle document.
fn parse(body: &str) -> (Dict, Vec<[Id; 3]>) {
    Graph::parse_to_triples(&format!("{}{}", PRE, body), "turtle").expect("parse")
}

/// Parse a premise and a conclusion document into ONE shared dict (the conclusion's ids are
/// re-interned into the premise dict term-by-term). Re-interning shares the blank-node label
/// space across both documents: each distinct label (`_:foo`) maps to one dict id regardless
/// of which document introduced it. Tests that use blank nodes in BOTH the premise and the
/// conclusion must therefore use disjoint blank-node labels to avoid aliasing distinct
/// anonymous individuals. [OPUS-4.8] sq-pbz04.4.6 — stale "blank-node-free" claim removed.
fn parse_two(premise: &str, conclusion: &str) -> (Dict, Vec<[Id; 3]>, Vec<[Id; 3]>) {
    let (mut dict, prem) = parse(premise);
    let (cdict, ctriples) = parse(conclusion);
    let concl = ctriples
        .iter()
        .map(|t| t.map(|id| dict.intern(&cdict.term(id))))
        .collect();
    (dict, prem, concl)
}

fn check(body: &str) -> (ConsistencyVerdict, Branch) {
    let (mut dict, triples) = parse(body);
    let out = DirectChecker::new().consistency(&mut dict, &triples);
    (out.verdict, out.branch)
}

fn entail(premise: &str, conclusion: &str) -> (EntailmentVerdict, Branch) {
    let (mut dict, prem, concl) = parse_two(premise, conclusion);
    let out = DirectChecker::new().entailment(&mut dict, &prem, &concl);
    (out.verdict, out.branch)
}

// -------------------------------------------------------------------------------------------
// Constructor / helper API (direct coverage of every public item)
// -------------------------------------------------------------------------------------------

#[test]
fn checker_constructors() {
    assert_eq!(DirectChecker::new(), DirectChecker::default());
    let pinned = Budget {
        max_nodes: 7,
        max_rule_applications: 9,
    };
    // with_budget produces a distinct checker (the budget is load-bearing state).
    assert_ne!(DirectChecker::with_budget(pinned), DirectChecker::new());
    assert_eq!(
        DirectChecker::with_budget(Budget::default()),
        DirectChecker::new()
    );
}

#[test]
fn consistency_verdict_helpers() {
    assert!(ConsistencyVerdict::Consistent.is_consistent());
    assert!(!ConsistencyVerdict::Consistent.is_inconsistent());
    assert!(ConsistencyVerdict::Inconsistent.is_inconsistent());
    assert!(!ConsistencyVerdict::Inconsistent.is_unknown());
    let unknown = ConsistencyVerdict::Unknown(UnknownReason::QlConsistencyPending);
    assert!(unknown.is_unknown());
    assert!(!unknown.is_consistent());
}

#[test]
fn entailment_verdict_helpers() {
    assert!(EntailmentVerdict::Entailed.is_entailed());
    assert!(!EntailmentVerdict::Entailed.is_not_entailed());
    assert!(EntailmentVerdict::NotEntailed.is_not_entailed());
    assert!(!EntailmentVerdict::NotEntailed.is_unknown());
    let unknown = EntailmentVerdict::Unknown(UnknownReason::ElTopGuard);
    assert!(unknown.is_unknown());
    assert!(!unknown.is_entailed());
}

// -------------------------------------------------------------------------------------------
// RL branch
// -------------------------------------------------------------------------------------------

#[test]
fn rl_consistent_past_clean_guard() {
    // Plain Horn subclass + assertion: in RL, no divergence-implicated construct, no
    // punning, no clash — the ONE shape where the RL branch may say Consistent.
    let (verdict, branch) = check(":A rdfs:subClassOf :B . :x a :A .");
    assert_eq!(verdict, ConsistencyVerdict::Consistent);
    assert_eq!(branch, Branch::RlMaterialization);
}

#[test]
fn rl_inconsistent_via_materialized_clash() {
    // x ∈ A ⊑ ⊥: the RL closure types x into owl:Nothing (cax-sco + cls-nothing) — an
    // Inconsistent verdict sound via checked-PR1.
    let (verdict, branch) = check(":A rdfs:subClassOf owl:Nothing . :x a :A .");
    assert_eq!(verdict, ConsistencyVerdict::Inconsistent);
    assert_eq!(branch, Branch::RlMaterialization);
}

#[test]
fn rl_divergence_guard_disjointness_blocks_consistent() {
    // BEAD ACCEPTANCE CASE: in-RL, clash-free, but touching owl:disjointWith — a construct
    // implicated in DOCUMENTED_DIVERGENCES — so "no clash" must NOT read as Consistent.
    let (verdict, branch) = check(":A owl:disjointWith :B . :x a :A .");
    assert!(
        matches!(
            verdict,
            ConsistencyVerdict::Unknown(UnknownReason::RlDivergenceGuard(_))
        ),
        "expected RlDivergenceGuard, got {:?}",
        verdict
    );
    assert_eq!(branch, Branch::RlMaterialization);
}

#[test]
fn rl_divergence_guard_complement_and_union() {
    // complementOf (RL super-CE position) and unionOf (RL sub-CE position) each trip the
    // guard on a clash-free in-RL ontology.
    let (v1, b1) = check(":A rdfs:subClassOf [ owl:complementOf :B ] . :x a :A .");
    assert!(
        matches!(
            v1,
            ConsistencyVerdict::Unknown(UnknownReason::RlDivergenceGuard(_))
        ),
        "complementOf: expected RlDivergenceGuard, got {:?}",
        v1
    );
    assert_eq!(b1, Branch::RlMaterialization);
    let (v2, _) = check("[ owl:unionOf (:A :B) ] rdfs:subClassOf :C .");
    assert!(
        matches!(
            v2,
            ConsistencyVerdict::Unknown(UnknownReason::RlDivergenceGuard(_))
        ),
        "unionOf: expected RlDivergenceGuard, got {:?}",
        v2
    );
}

#[test]
fn rl_divergence_guard_does_not_mask_inconsistency() {
    // Inconsistent stays sound EVEN with a guarded construct present: the guard narrows
    // Consistent only.
    let (verdict, branch) = check(":A owl:disjointWith :B . :x a :A . :x a :B .");
    assert_eq!(verdict, ConsistencyVerdict::Inconsistent);
    assert_eq!(branch, Branch::RlMaterialization);
}

#[test]
fn rl_pr1_punning_abstains() {
    // :A is used both as a class (:x a :A) and as an individual (:A a :B) — usage-level
    // punning violates the Theorem PR1 preconditions, so BOTH verdicts are withheld.
    let (verdict, branch) = check(":x a :A . :A a :B .");
    assert!(
        matches!(
            verdict,
            ConsistencyVerdict::Unknown(UnknownReason::RlPr1Preconditions(_))
        ),
        "expected RlPr1Preconditions, got {:?}",
        verdict
    );
    assert_eq!(branch, Branch::RlMaterialization);
}

// -------------------------------------------------------------------------------------------
// EL branch
// -------------------------------------------------------------------------------------------

/// An EL ontology (∃-chain on the super side keeps it OUT of RL) nested deeper than the EL
/// classifier's decode depth, so the classifier reports a skipped axiom.
fn deep_el_body(depth: usize) -> String {
    let mut body = String::from(":A rdfs:subClassOf ");
    for _ in 0..depth {
        body.push_str("[ owl:onProperty :r ; owl:someValuesFrom ");
    }
    body.push_str(":B ");
    for _ in 0..depth {
        body.push_str("] ");
    }
    body.push('.');
    body
}

#[test]
fn el_skipped_axioms_abstains() {
    // BEAD ACCEPTANCE CASE: in-EL (L1 extracts it — nesting is under L1's 512 cap), but
    // the EL classifier skips the axiom (over its 256 decode depth): a skipped axiom could
    // BE the inconsistency, so the branch abstains rather than trusting the lattice.
    let (verdict, branch) = check(&deep_el_body(300));
    assert!(
        matches!(
            verdict,
            ConsistencyVerdict::Unknown(UnknownReason::ElSkippedAxioms(n)) if n >= 1
        ),
        "expected ElSkippedAxioms(>=1), got {:?}",
        verdict
    );
    assert_eq!(branch, Branch::ElClassification);
}

#[test]
fn el_unapplied_abox_abstains() {
    // In-EL with an ABox: the TBox classifier neither applies nor counts assertions, so a
    // Consistent verdict would be unfounded — abstain with the unapplied-kind reason.
    let (verdict, branch) =
        check(":A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] . :x a :A .");
    assert!(
        matches!(
            verdict,
            ConsistencyVerdict::Unknown(UnknownReason::ElUnappliedAxioms(_))
        ),
        "expected ElUnappliedAxioms, got {:?}",
        verdict
    );
    assert_eq!(branch, Branch::ElClassification);
}

#[test]
fn el_top_guard_never_calls_thing_bottom_consistent() {
    // SOUNDNESS PIN: `owl:Thing ⊑ owl:Nothing` is INCONSISTENT under the Direct Semantics
    // (domains are non-empty), but the EL classifier does not track ⊤'s satisfiability
    // (its unsatisfiable-class list stays empty here). The ⊤ guard must abstain — a
    // Consistent verdict would be UNSOUND.
    let (verdict, branch) = check("owl:Thing rdfs:subClassOf owl:Nothing .");
    assert_eq!(
        verdict,
        ConsistencyVerdict::Unknown(UnknownReason::ElTopGuard)
    );
    assert_eq!(branch, Branch::ElClassification);
}

#[test]
fn el_consistent_for_top_free_tbox() {
    // Pure ⊤-free EL+⊥ TBox (super-∃ keeps it out of RL): Consistent by the
    // empty-interpretation model construction — the one shape the EL branch certifies.
    let (verdict, branch) = check(
        ":A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] . \
         :B rdfs:subClassOf :C .",
    );
    assert_eq!(verdict, ConsistencyVerdict::Consistent);
    assert_eq!(branch, Branch::ElClassification);
}

// -------------------------------------------------------------------------------------------
// QL + ALCH branches, extraction, budget
// -------------------------------------------------------------------------------------------

#[test]
fn ql_always_pending() {
    // In QL (super-∃ with a named filler kills RL; the complement kills EL), so the QL
    // branch owns it — and QL consistency is wholly deferred (sq-pbz04.3.4), never decided
    // here.
    let (verdict, branch) = check(
        ":A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] . \
         :C rdfs:subClassOf [ owl:complementOf :D ] .",
    );
    assert_eq!(
        verdict,
        ConsistencyVerdict::Unknown(UnknownReason::QlConsistencyPending)
    );
    assert_eq!(branch, Branch::QlDeferred);
}

#[test]
fn alch_branch_consistent_and_inconsistent() {
    // ¬A ⊑ B is in NO profile (complement in sub position) — the tableau owns it and is
    // complete for the fragment, so BOTH definitive verdicts flow through.
    let (v1, b1) = check("[ owl:complementOf :A ] rdfs:subClassOf :B .");
    assert_eq!(v1, ConsistencyVerdict::Consistent);
    assert_eq!(b1, Branch::AlchTableau);
    // A ≡ ¬A has no model (also profile-free: complement as an equivalence operand).
    let (v2, b2) = check(":A owl:equivalentClass [ owl:complementOf :A ] .");
    assert_eq!(v2, ConsistencyVerdict::Inconsistent);
    assert_eq!(b2, Branch::AlchTableau);
}

#[test]
fn alch_budget_exhaustion_abstains() {
    let (mut dict, triples) = parse("[ owl:complementOf :A ] rdfs:subClassOf :B .");
    let starved = DirectChecker::with_budget(Budget {
        max_nodes: 1,
        max_rule_applications: 0,
    });
    let out = starved.consistency(&mut dict, &triples);
    assert!(
        matches!(
            out.verdict,
            ConsistencyVerdict::Unknown(UnknownReason::ResourceBudget(_))
        ),
        "expected ResourceBudget, got {:?}",
        out.verdict
    );
    assert_eq!(out.branch, Branch::AlchTableau);
}

#[test]
fn extraction_failure_fails_closed() {
    // owl:inverseOf is outside the L1 fragment: refused BEFORE any branch runs.
    let (verdict, branch) = check(":p owl:inverseOf :q .");
    assert!(
        matches!(
            verdict,
            ConsistencyVerdict::Unknown(UnknownReason::OutOfFragment(_))
        ),
        "expected OutOfFragment, got {:?}",
        verdict
    );
    assert_eq!(branch, Branch::Extraction);
}

// -------------------------------------------------------------------------------------------
// Entailment by refutation
// -------------------------------------------------------------------------------------------

#[test]
fn entailment_subclass_transitivity() {
    let premise = ":A rdfs:subClassOf :B . :B rdfs:subClassOf :C .";
    let (v, b) = entail(premise, ":A rdfs:subClassOf :C .");
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
    // The converse is NOT entailed — certified by the complete tableau (a definitive
    // NegativeEntailment-style verdict, sound per the record's complete-branch rule).
    let (v, b) = entail(premise, ":C rdfs:subClassOf :A .");
    assert_eq!(v, EntailmentVerdict::NotEntailed);
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_class_assertion() {
    let premise = ":x a :A . :A rdfs:subClassOf :B .";
    let (v, _) = entail(premise, ":x a :B .");
    assert_eq!(v, EntailmentVerdict::Entailed);
    let (v, _) = entail(premise, ":y a :B .");
    assert_eq!(v, EntailmentVerdict::NotEntailed);
}

#[test]
fn entailment_role_assertion_fresh_class_trick() {
    // BEAD ACCEPTANCE CASE: ObjectPropertyAssertion via the fresh-class encoding
    // {B(b), (∀R.¬B)(a)} — r ⊑ s and r(x,y) entail s(x,y); the ∀ on the SUPER-role must
    // reach the sub-role edge (the L3 hierarchy machinery) to close the refutation.
    let premise = ":r a owl:ObjectProperty . :s a owl:ObjectProperty . \
                   :r rdfs:subPropertyOf :s . :x :r :y .";
    let conclusion = ":s a owl:ObjectProperty . :x :s :y .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
    // Completeness half: the REVERSED assertion is not entailed, and the fresh-class
    // encoding must certify that (a model interpreting the fresh class as {target} exists).
    let (v, _) = entail(premise, ":s a owl:ObjectProperty . :y :s :x .");
    assert_eq!(v, EntailmentVerdict::NotEntailed);
}

#[test]
fn entailment_equivalence_needs_both_inclusions() {
    let (v, _) = entail(
        ":A rdfs:subClassOf :B . :B rdfs:subClassOf :A .",
        ":A owl:equivalentClass :B .",
    );
    assert_eq!(v, EntailmentVerdict::Entailed);
    // One inclusion alone does not entail the equivalence: the second refutation
    // component is satisfiable.
    let (v, _) = entail(":A rdfs:subClassOf :B .", ":A owl:equivalentClass :B .");
    assert_eq!(v, EntailmentVerdict::NotEntailed);
}

#[test]
fn entailment_disjointness_domain_range_desugarings() {
    // DisjointClasses via A ⊑ ¬B.
    let (v, _) = entail(
        ":A rdfs:subClassOf [ owl:complementOf :B ] .",
        ":A owl:disjointWith :B .",
    );
    assert_eq!(v, EntailmentVerdict::Entailed);
    // Domain widens along subclassing (∃r.⊤ ⊑ A ⊑ B).
    let (v, _) = entail(
        ":r a owl:ObjectProperty . :r rdfs:domain :A . :A rdfs:subClassOf :B .",
        ":r a owl:ObjectProperty . :r rdfs:domain :B .",
    );
    assert_eq!(v, EntailmentVerdict::Entailed);
    // Range widens along subclassing (⊤ ⊑ ∀r.A, A ⊑ B).
    let (v, _) = entail(
        ":r a owl:ObjectProperty . :r rdfs:range :A . :A rdfs:subClassOf :B .",
        ":r a owl:ObjectProperty . :r rdfs:range :B .",
    );
    assert_eq!(v, EntailmentVerdict::Entailed);
}

// -------------------------------------------------------------------------------------------
// SubObjectPropertyOf conclusions — the fresh-individual-pair encoding ([FABLE-5] sq-pbz04.4.9)
// -------------------------------------------------------------------------------------------

#[test]
fn entailment_subproperty_direct_and_converse() {
    // BEAD ACCEPTANCE CASE (sq-pbz04.4.9): a SubObjectPropertyOf CONCLUSION now gets a
    // definitive verdict via {R(a,b), B(b), (∀S.¬B)(a)} with fresh a/b/B — previously an
    // UnencodedConclusion abstention. r ⊑ s entails r ⊑ s...
    let premise = ":r a owl:ObjectProperty . :s a owl:ObjectProperty . :r rdfs:subPropertyOf :s .";
    let conclusion =
        ":r a owl:ObjectProperty . :s a owl:ObjectProperty . :r rdfs:subPropertyOf :s .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
    // ...and the CONVERSE s ⊑ r is definitively NOT entailed (completeness half: the model
    // interpreting B = {b} with an s-edge outside r certifies satisfiability). A mutation
    // that quantifies ∀ over the SUB role instead of the SUPER role makes this refutation
    // unsatisfiable and flips it to a WRONG Entailed — this assertion is the witness.
    let (v, b) = entail(
        premise,
        ":r a owl:ObjectProperty . :s a owl:ObjectProperty . :s rdfs:subPropertyOf :r .",
    );
    assert_eq!(v, EntailmentVerdict::NotEntailed);
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_subproperty_transitivity() {
    // Role-hierarchy transitivity flows through the tableau's reflexive-transitive
    // `is_subrole` closure: r ⊑ s ⊑ t entails r ⊑ t.
    let premise = ":r a owl:ObjectProperty . :s a owl:ObjectProperty . :t a owl:ObjectProperty . \
                   :r rdfs:subPropertyOf :s . :s rdfs:subPropertyOf :t .";
    let (v, b) = entail(
        premise,
        ":r a owl:ObjectProperty . :t a owl:ObjectProperty . :r rdfs:subPropertyOf :t .",
    );
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
    // t ⊑ r is not entailed by the chain.
    let (v, _) = entail(
        premise,
        ":r a owl:ObjectProperty . :t a owl:ObjectProperty . :t rdfs:subPropertyOf :r .",
    );
    assert_eq!(v, EntailmentVerdict::NotEntailed);
}

#[test]
fn entailment_subproperty_reflexivity_from_empty_premise() {
    // R ⊑ R holds in EVERY interpretation, so even an (axiom-)empty premise entails it —
    // the refutation {r(a,b), B(b), (∀r.¬B)(a)} clashes on the r-edge itself (the
    // reflexive base of `is_subrole`). A mutation that drops B(b) or the role assertion
    // from the encoding leaves the refutation satisfiable and flips this to NotEntailed.
    let (v, b) = entail(
        ":x a :A .",
        ":r a owl:ObjectProperty . :r rdfs:subPropertyOf :r .",
    );
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
    // Two UNRELATED roles from the same premise: definitively not entailed (freshness: the
    // fresh B/a/b must not capture anything in the premise).
    let (v, _) = entail(
        ":x a :A .",
        ":r a owl:ObjectProperty . :s a owl:ObjectProperty . :r rdfs:subPropertyOf :s .",
    );
    assert_eq!(v, EntailmentVerdict::NotEntailed);
}

#[test]
fn entailment_subproperty_mixed_conclusion_aggregates() {
    // A conclusion mixing the NEW RBox encoding with an ABox one: r ⊑ s AND s(x,y) both
    // follow from {r ⊑ s, r(x,y)} — the per-axiom conjunction aggregates to Entailed.
    let premise = ":r a owl:ObjectProperty . :s a owl:ObjectProperty . \
                   :r rdfs:subPropertyOf :s . :x :r :y .";
    let conclusion = ":r a owl:ObjectProperty . :s a owl:ObjectProperty . \
                      :r rdfs:subPropertyOf :s . :x :s :y .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
    // Flip ONE conjunct (t ⊑ r not entailed): the whole conclusion set fails definitively.
    let conclusion = ":r a owl:ObjectProperty . :s a owl:ObjectProperty . \
                      :t a owl:ObjectProperty . :t rdfs:subPropertyOf :r . :x :s :y .";
    let (v, _) = entail(premise, conclusion);
    assert_eq!(v, EntailmentVerdict::NotEntailed);
}

#[test]
fn entailment_out_of_fragment_inputs_fail_closed() {
    // Premise outside the fragment.
    let (v, b) = entail(":p owl:inverseOf :q .", ":A rdfs:subClassOf :B .");
    assert!(
        matches!(
            v,
            EntailmentVerdict::Unknown(UnknownReason::OutOfFragment(_))
        ),
        "premise: expected OutOfFragment, got {:?}",
        v
    );
    assert_eq!(b, Branch::Extraction);
    // Conclusion outside the fragment.
    let (v, b) = entail(":A rdfs:subClassOf :B .", ":p owl:inverseOf :q .");
    assert!(
        matches!(
            v,
            EntailmentVerdict::Unknown(UnknownReason::OutOfFragment(_))
        ),
        "conclusion: expected OutOfFragment, got {:?}",
        v
    );
    assert_eq!(b, Branch::Extraction);
}

#[test]
fn entailment_empty_conclusion_is_trivially_entailed() {
    let (v, b) = entail(":A rdfs:subClassOf :B .", "");
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
}

// -------------------------------------------------------------------------------------------
// Conclusion anonymous individuals — existential reading + rolling-up ([OPUS-4.8] sq-pbz04.4.13)
// -------------------------------------------------------------------------------------------

#[test]
fn entailment_conclusion_bnode_somevaluesfrom2bnode() {
    // Faithful reconstruction of the W3C DIRECT PositiveEntailmentTest `somevaluesfrom2bnode`
    // ("Shows that a BNode is an existential variable"): the premise types `a` into ∃p.⊤, the
    // conclusion asserts `a p _:x`. Under the official existential reading this IS entailed;
    // rolling `a p _:x` into `a : ∃p.owl:Thing` lets the complete tableau certify it. Before
    // the fix, `_:x` was read as a skolem CONSTANT and the refutation stayed satisfiable → a
    // WRONG NotEntailed (the pinned M2 divergence).
    let premise = ":p a owl:ObjectProperty . \
                   :a rdf:type [ owl:onProperty :p ; owl:someValuesFrom owl:Thing ] .";
    let conclusion = ":p a owl:ObjectProperty . :a :p _:x .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(
        v,
        EntailmentVerdict::Entailed,
        "somevaluesfrom2bnode must be Entailed"
    );
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_conclusion_bnode_webont_somevaluesfrom_003() {
    // Faithful reconstruction of W3C DIRECT PositiveEntailmentTest `WebOnt-someValuesFrom-003`
    // ("A simple infinite loop for implementors to avoid"): premise `person ≡ ∃parent.person`
    // with `fred : person`; conclusion an anonymous parent-CHAIN `fred parent _:b1 parent _:b2`
    // (each an owl:Thing). The chain rolls into `fred : ∃parent.(⊤ ⊓ ∃parent.⊤)`, decided by
    // the tableau (termination via blocking) — genuinely Entailed, not the M2 skolem NotEntailed.
    let premise = ":parent a owl:ObjectProperty . \
                   :person owl:equivalentClass [ owl:onProperty :parent ; owl:someValuesFrom :person ] . \
                   :fred a :person .";
    let conclusion = ":parent a owl:ObjectProperty . \
                      :fred a owl:Thing . \
                      :fred :parent _:b1 . _:b1 a owl:Thing . \
                      _:b1 :parent _:b2 . _:b2 a owl:Thing .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(
        v,
        EntailmentVerdict::Entailed,
        "WebOnt-someValuesFrom-003 must be Entailed"
    );
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_conclusion_bnode_rollable_notentailed_is_sound() {
    // A rollable conclusion bnode whose existential is genuinely NOT entailed: premise only
    // says `a : A` (a need not have any p-successor), conclusion asserts `a p _:x`. Rolling to
    // `a : ∃p.⊤` and refuting on the COMPLETE tableau yields a SOUND NotEntailed — rolling is
    // sound in BOTH directions, so a definitive negative verdict here is legitimate (it is NOT
    // the unsound skolem-constant NotEntailed, which the non-rollable shapes below abstain on).
    let premise = ":p a owl:ObjectProperty . :a a :A .";
    let conclusion = ":p a owl:ObjectProperty . :a :p _:x .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(v, EntailmentVerdict::NotEntailed);
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_conclusion_bnode_shared_abstains_never_notentailed() {
    // NEGATIVE PROBE: `_:x` is SHARED between two property assertions (`a p _:x` and `b p _:x`)
    // — a non-tree shape. The rolling-up is not applicable, so the checker must ABSTAIN
    // fail-closed, NEVER emit a skolem-constant NotEntailed.
    let premise = ":p a owl:ObjectProperty . :a a :A . :b a :A .";
    let conclusion = ":p a owl:ObjectProperty . :a :p _:x . :b :p _:x .";
    let (v, b) = entail(premise, conclusion);
    assert!(
        matches!(
            v,
            EntailmentVerdict::Unknown(UnknownReason::ConclusionAnonymousIndividual(_))
        ),
        "shared conclusion bnode must abstain (ConclusionAnonymousIndividual), got {:?}",
        v
    );
    assert!(!v.is_not_entailed(), "must NOT be a skolem NotEntailed");
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_conclusion_bnode_cyclic_abstains_never_notentailed() {
    // NEGATIVE PROBE: a cyclic anonymous shape `_:x p _:y . _:y p _:x .` with no named anchor —
    // not tree-shaped, so abstain fail-closed, never a skolem NotEntailed.
    let premise = ":p a owl:ObjectProperty . :a a :A .";
    let conclusion = ":p a owl:ObjectProperty . _:x :p _:y . _:y :p _:x .";
    let (v, b) = entail(premise, conclusion);
    assert!(
        matches!(
            v,
            EntailmentVerdict::Unknown(UnknownReason::ConclusionAnonymousIndividual(_))
        ),
        "cyclic conclusion bnode must abstain (ConclusionAnonymousIndividual), got {:?}",
        v
    );
    assert!(!v.is_not_entailed());
    assert_eq!(b, Branch::AlchTableau);
}

#[test]
fn entailment_conclusion_bnode_named_successor_abstains() {
    // NEGATIVE PROBE: an anonymous individual with a NAMED successor (`a p _:x . _:x q :c .`)
    // would roll up only through a nominal `{c}`, which is out of the ALCH fragment — abstain.
    let premise = ":p a owl:ObjectProperty . :q a owl:ObjectProperty . :a a :A .";
    let conclusion = ":p a owl:ObjectProperty . :q a owl:ObjectProperty . \
                      :a :p _:x . _:x :q :c .";
    let (v, _) = entail(premise, conclusion);
    assert!(
        matches!(
            v,
            EntailmentVerdict::Unknown(UnknownReason::ConclusionAnonymousIndividual(_))
        ),
        "named-successor conclusion bnode must abstain, got {:?}",
        v
    );
    assert!(!v.is_not_entailed());
}

#[test]
fn entailment_premise_side_bnode_unaffected_skolemisation_stays() {
    // Premise-side blank nodes are UNAFFECTED — skolemisation is entailment-preserving on the
    // premise. The premise `a p _:pb`, `_:pb : C`, `C ⊑ D` (a has an anonymous p-successor that
    // is a C, hence a D) entails the bnode-FREE conclusion `a : ∃p.D` — a definitive verdict,
    // proving the abstention is CONCLUSION-triggered only and premise bnodes still reason.
    let premise = ":p a owl:ObjectProperty . :a :p _:pb . _:pb a :C . :C rdfs:subClassOf :D .";
    let conclusion = ":a rdf:type [ owl:onProperty :p ; owl:someValuesFrom :D ] .";
    let (v, b) = entail(premise, conclusion);
    assert_eq!(v, EntailmentVerdict::Entailed);
    assert_eq!(b, Branch::AlchTableau);
}
