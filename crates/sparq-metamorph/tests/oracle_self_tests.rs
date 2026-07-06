//! Oracle self-tests — the crate's non-vacuity + fail-closed invariant. [FABLE-5] sq-gum8.6
//!
//! * **Non-vacuity (mutation check):** a deliberately-injected wrong-result mutant
//!   (`FilterDropsRow`, which silently removes a row from any `FILTER` query's result)
//!   must be flagged by the TLP, NoREC, AND differential oracles. These tests run the
//!   REAL sparq engine (`InProcessSparq` over `sparq_engine::query_json`), not a mock
//!   evaluator.
//! * **Fail-closed:** an engine error (invalid query, failing driver) must yield
//!   `Verdict::EngineFailure` — never `Pass`, never `Violation` — keeping the
//!   wrong-result / engine-error classes strictly separated.
//! * **Three-branch coverage:** the fixed dataset exercises all three EBV outcomes —
//!   true, false, and error via BOTH error causes (a type error from an incomparable
//!   bound value, and an unbound OPTIONAL variable).

use sparq_difftest::QueryResults;
use sparq_metamorph::engine::{FilterDropsRow, InProcessSparq, SparqlEngine};
use sparq_metamorph::verdict::{FailureKind, Verdict};
use sparq_metamorph::{check_differential, check_norec, check_tlp, generate_case, tlp_queries};

/// Ages straddle the predicate `?age < 25` (true for s2, false for s1), s3's age is a
/// string (type error under `<`), and s4 has an `ex:name` but NO `ex:age` — so the
/// OPTIONAL pattern leaves `?age` unbound (unbound-evaluation error).
const DATA: &str = concat!(
    "<http://example.org/s1> <http://example.org/age> \"30\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
    "<http://example.org/s1> <http://example.org/name> \"alice\" .\n",
    "<http://example.org/s2> <http://example.org/age> \"20\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
    "<http://example.org/s2> <http://example.org/name> \"bob\" .\n",
    "<http://example.org/s3> <http://example.org/age> \"twenty\" .\n",
    "<http://example.org/s3> <http://example.org/name> \"carol\" .\n",
    "<http://example.org/s4> <http://example.org/name> \"dave\" .\n",
);

/// A pattern whose OPTIONAL leaves `?age` unbound for s4.
const PATTERN: &str =
    "?s <http://example.org/name> ?n . OPTIONAL { ?s <http://example.org/age> ?age }";
/// True for s2 (20), false for s1 (30), type error for s3 ("twenty"), unbound error for s4.
const PREDICATE: &str = "?age < 25";

fn pristine() -> InProcessSparq {
    InProcessSparq::from_ntriples("sparq", DATA).expect("test data loads")
}

fn rows(engine: &dyn SparqlEngine, query: &str) -> usize {
    match engine.select(query).expect("query evaluates") {
        QueryResults::Solutions { solutions, .. } => solutions.len(),
        QueryResults::Boolean(_) => panic!("SELECT returned a boolean"),
    }
}

// --- the three-branch semantics itself (the TLP case analysis, checked concretely) ---

#[test]
fn tlp_branches_partition_the_base_as_the_case_analysis_predicts() {
    let engine = pristine();
    let queries = tlp_queries(PATTERN, PREDICATE);
    assert_eq!(rows(&engine, &queries.base), 4, "s1..s4");
    assert_eq!(rows(&engine, &queries.branch_true), 1, "s2: 20 < 25");
    assert_eq!(rows(&engine, &queries.branch_false), 1, "s1: 30 < 25 is false");
    // s3: "twenty" < 25 is a type error; s4: ?age unbound (OPTIONAL) is an error.
    assert_eq!(rows(&engine, &queries.branch_error), 2, "s3 type error + s4 unbound");
}

// --- non-vacuity: each oracle passes on the pristine engine, flags the mutant ---

#[test]
fn tlp_passes_on_pristine_sparq() {
    let verdict = check_tlp(&pristine(), PATTERN, PREDICATE);
    assert!(verdict.is_pass(), "expected a pass, got {verdict:?}");
}

#[test]
fn tlp_flags_the_seeded_wrong_result_mutant() {
    let mutant = FilterDropsRow::new(pristine());
    let verdict = check_tlp(&mutant, PATTERN, PREDICATE);
    assert!(verdict.is_violation(), "expected a violation, got {verdict:?}");
}

#[test]
fn norec_passes_on_pristine_sparq() {
    let verdict = check_norec(&pristine(), PATTERN, PREDICATE);
    assert!(verdict.is_pass(), "expected a pass, got {verdict:?}");
}

#[test]
fn norec_flags_the_seeded_wrong_result_mutant() {
    let mutant = FilterDropsRow::new(pristine());
    let verdict = check_norec(&mutant, PATTERN, PREDICATE);
    assert!(verdict.is_violation(), "expected a violation, got {verdict:?}");
}

#[test]
fn differential_passes_on_two_pristine_sparq_instances() {
    let a = pristine();
    let b = InProcessSparq::from_ntriples("sparq-b", DATA).unwrap();
    let query = format!("SELECT * WHERE {{ {PATTERN} FILTER( {PREDICATE} ) }}");
    let verdict = check_differential(&[&a, &b], &query);
    assert!(verdict.is_pass(), "expected a pass, got {verdict:?}");
}

#[test]
fn differential_flags_the_seeded_wrong_result_mutant() {
    let reference = pristine();
    let mutant = FilterDropsRow::new(pristine());
    let query = format!("SELECT * WHERE {{ {PATTERN} FILTER( {PREDICATE} ) }}");
    let verdict = check_differential(&[&reference, &mutant], &query);
    assert!(verdict.is_violation(), "expected a violation, got {verdict:?}");
}

// --- fail-closed: engine errors are engine failures, never passes or violations ---

/// A driver whose every evaluation fails, standing in for a crashed/unreachable engine.
struct AlwaysFails;
impl SparqlEngine for AlwaysFails {
    fn name(&self) -> &str {
        "always-fails"
    }
    fn select(&self, sparql: &str) -> Result<QueryResults, sparq_metamorph::EngineFailure> {
        Err(sparq_metamorph::EngineFailure {
            engine: "always-fails".into(),
            query: sparql.to_string(),
            kind: FailureKind::Evaluation,
            message: "synthetic failure".into(),
        })
    }
}

#[test]
fn all_three_oracles_fail_closed_on_an_erroring_engine() {
    let tlp = check_tlp(&AlwaysFails, PATTERN, PREDICATE);
    let norec = check_norec(&AlwaysFails, PATTERN, PREDICATE);
    let differential = check_differential(&[&AlwaysFails, &AlwaysFails], "ASK { ?s ?p ?o }");
    for verdict in [&tlp, &norec, &differential] {
        assert!(
            verdict.is_engine_failure(),
            "an engine error must be an EngineFailure verdict, got {verdict:?}"
        );
        assert!(!verdict.is_pass() && !verdict.is_violation());
    }
}

#[test]
fn a_syntactically_invalid_query_is_an_engine_failure_on_the_real_engine() {
    let engine = pristine();
    let verdict = check_differential(&[&engine, &engine], "SELECT * WHERE { broken");
    match verdict {
        Verdict::EngineFailure(f) => assert_eq!(f.kind, FailureKind::Evaluation),
        other => panic!("expected an engine failure, got {other:?}"),
    }
}

// --- generated cases: the oracles hold on the real engine across seeds ---

/// TLP + NoREC over the seeded generator against real sparq: every verdict must be a
/// pass (the relations hold) or — never — a violation. Engine failures would surface
/// generator/grammar bugs, so they are also rejected here: sparq accepts the full
/// generated grammar today, and a grammar extension that sparq rejects should be
/// caught at generation time, not silently skipped.
#[test]
fn generated_cases_hold_on_pristine_sparq() {
    for seed in 0..50 {
        let case = generate_case(seed);
        let engine = InProcessSparq::from_ntriples("sparq", &case.data_ntriples)
            .unwrap_or_else(|e| panic!("seed {seed}: generated data must load: {e}"));
        let tlp = check_tlp(&engine, &case.pattern, &case.predicate);
        assert!(
            tlp.is_pass(),
            "seed {seed}: TLP must hold on sparq (predicate: {}): {tlp:?}",
            case.predicate
        );
        let norec = check_norec(&engine, &case.pattern, &case.predicate);
        assert!(
            norec.is_pass(),
            "seed {seed}: NoREC must hold on sparq (predicate: {}): {norec:?}",
            case.predicate
        );
    }
}
