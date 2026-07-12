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
use sparq_metamorph::{
    check_differential, check_norec, check_tlp, generate_case, norec_queries, tlp_queries,
};

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

// [GPT-5.6] sq-aglhv: independently guard the row-preserving Extend/Project step on
// which NoREC's relative optimized-count versus true-flag-count comparison relies.
fn norec_rewrite_preserves_cardinality(
    engine: &dyn SparqlEngine,
    pattern: &str,
    predicate: &str,
) -> bool {
    let base = format!("SELECT * WHERE {{ {pattern} }}");
    let rewritten = norec_queries(pattern, predicate).rewritten;
    rows(engine, &rewritten) == rows(engine, &base)
}

/// A test-only mutant that removes one row only from NoREC's FILTER-free rewrite.
struct RewriteDropsRow<E> {
    inner: E,
}

impl<E> RewriteDropsRow<E> {
    fn new(inner: E) -> Self {
        Self { inner }
    }
}

impl<E: SparqlEngine> SparqlEngine for RewriteDropsRow<E> {
    fn name(&self) -> &str {
        "rewrite-drops-row"
    }

    fn select(&self, sparql: &str) -> Result<QueryResults, sparq_metamorph::EngineFailure> {
        let mut results = self.inner.select(sparql)?;
        if sparql.starts_with("SELECT ( IF(") {
            if let QueryResults::Solutions { solutions, .. } = &mut results {
                solutions.pop();
            }
        }
        Ok(results)
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

// --- the sq-996jn term-inspection atoms: LANG + isLiteral over the real engine ---

/// One object per EBV outcome of `LANG(?v) = "fr"`: a language-tagged literal (true),
/// a plain literal (`LANG` returns `""` — false), and an IRI (`LANG` on a non-literal
/// is a type error, SPARQL 1.1 §17.3 operand rules).
const LANG_DATA: &str = concat!(
    "<http://example.org/s1> <http://example.org/v> \"mot\"@fr .\n",
    "<http://example.org/s2> <http://example.org/v> \"str\" .\n",
    "<http://example.org/s3> <http://example.org/v> <http://example.org/o3> .\n",
);
const LANG_PATTERN: &str = "?s <http://example.org/v> ?v";

/// The `LANG(?v) = "fr"` atom must land one row in EACH TLP branch on the mixed
/// dataset — the concrete witness that the atom is genuine three-outcome fuel (the
/// whole point of adding it, sq-996jn), not just a true/false predicate.
#[test]
fn lang_atom_exercises_the_full_ebv_trichotomy() {
    let engine = InProcessSparq::from_ntriples("sparq", LANG_DATA).unwrap();
    let predicate = "LANG(?v) = \"fr\"";
    let queries = tlp_queries(LANG_PATTERN, predicate);
    assert_eq!(rows(&engine, &queries.base), 3, "s1..s3");
    assert_eq!(rows(&engine, &queries.branch_true), 1, "s1: LANG(\"mot\"@fr) = \"fr\"");
    assert_eq!(
        rows(&engine, &queries.branch_false),
        1,
        "s2: LANG on a plain literal returns \"\", which differs from \"fr\""
    );
    assert_eq!(
        rows(&engine, &queries.branch_error),
        1,
        "s3: LANG on an IRI is a type error"
    );
    assert!(check_tlp(&engine, LANG_PATTERN, predicate).is_pass());
    assert!(check_norec(&engine, LANG_PATTERN, predicate).is_pass());
}

/// `isLiteral(?v)` over a bound variable splits literals (true) from IRIs (false)
/// and never errors — the two-outcome companion atom (mirrors `isIRI`).
#[test]
fn is_literal_atom_splits_literals_from_iris() {
    let engine = InProcessSparq::from_ntriples("sparq", LANG_DATA).unwrap();
    let predicate = "isLiteral(?v)";
    let queries = tlp_queries(LANG_PATTERN, predicate);
    assert_eq!(rows(&engine, &queries.branch_true), 2, "s1 lang-tagged + s2 plain");
    assert_eq!(rows(&engine, &queries.branch_false), 1, "s3: an IRI is not a literal");
    assert_eq!(rows(&engine, &queries.branch_error), 0, "?v is always bound here");
    assert!(check_tlp(&engine, LANG_PATTERN, predicate).is_pass());
    assert!(check_norec(&engine, LANG_PATTERN, predicate).is_pass());
}

/// The oracles still catch the seeded wrong-result mutant on a LANG predicate —
/// the non-vacuity anchor extended to the new atom family.
#[test]
fn mutant_is_flagged_on_a_lang_predicate() {
    let mutant =
        FilterDropsRow::new(InProcessSparq::from_ntriples("sparq", LANG_DATA).unwrap());
    let verdict = check_tlp(&mutant, LANG_PATTERN, "LANG(?v) = \"fr\"");
    assert!(verdict.is_violation(), "expected a violation, got {verdict:?}");
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

#[test]
fn norec_rewrite_preserves_absolute_cardinality_across_generated_cases() {
    for seed in 0..10 {
        let case = generate_case(seed);
        let engine = InProcessSparq::from_ntriples("sparq", &case.data_ntriples)
            .unwrap_or_else(|e| panic!("seed {seed}: generated data must load: {e}"));
        assert!(
            norec_rewrite_preserves_cardinality(&engine, &case.pattern, &case.predicate),
            "seed {seed}: the NoREC rewrite must preserve the base row cardinality"
        );
        let verdict = check_norec(&engine, &case.pattern, &case.predicate);
        assert!(
            verdict.is_pass(),
            "seed {seed}: NoREC sanity check must pass: {verdict:?}"
        );
    }
}

#[test]
fn norec_rewrite_cardinality_check_witnesses_a_dropped_rewrite_row() {
    for seed in 0..10 {
        let case = generate_case(seed);
        let engine = InProcessSparq::from_ntriples("sparq", &case.data_ntriples)
            .unwrap_or_else(|e| panic!("seed {seed}: generated data must load: {e}"));
        let mutant = RewriteDropsRow::new(engine);
        assert!(
            !norec_rewrite_preserves_cardinality(&mutant, &case.pattern, &case.predicate),
            "seed {seed}: mutation witness must make the cardinality check fail"
        );
    }
}

/// Generated cases whose predicates carry the sq-996jn atoms hold on real sparq:
/// hunt the deterministic seed stream for a fixed number of LANG- and
/// isLiteral-bearing cases and require both oracles to pass on each. Fail-closed on
/// reachability too: if the hunt cannot find enough such cases, the test fails
/// (a grammar regression, not a skip).
#[test]
fn generated_cases_with_the_new_atoms_hold_on_pristine_sparq() {
    const WANT: u32 = 10;
    let (mut lang_checked, mut is_literal_checked) = (0u32, 0u32);
    for seed in 0..2000u64 {
        if lang_checked >= WANT && is_literal_checked >= WANT {
            break;
        }
        let case = generate_case(seed);
        let has_lang = case.predicate.contains("LANG(?v)");
        let has_is_literal = case.predicate.contains("isLiteral(?v)");
        if !(has_lang && lang_checked < WANT) && !(has_is_literal && is_literal_checked < WANT)
        {
            continue;
        }
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
        lang_checked += u32::from(has_lang);
        is_literal_checked += u32::from(has_is_literal);
    }
    assert!(
        lang_checked >= WANT && is_literal_checked >= WANT,
        "seeds 0..2000 must reach {WANT} LANG and {WANT} isLiteral cases \
         (got lang={lang_checked}, isLiteral={is_literal_checked})"
    );
}
