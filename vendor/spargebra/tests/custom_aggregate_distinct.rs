// [OPUS-4.8] The vendored crate's `[lints.clippy]` table is pedantic and aimed
// at library code; this is an integration-test crate, so a couple of those lints
// are inappropriate here and are relaxed file-wide:
//   - tests_outside_test_module: an integration test crate IS the test module;
//     there is no library code to separate the tests from.
//   - expect_used: panicking on the unexpected is exactly what a test wants.
//   - assertions_on_result_states: `assert!(... .is_err())` is the clearest way
//     to state "this query must be rejected" and is the whole point of the test.
// (None of these gate in the real build: the project lints spargebra as a
//  `[patch.crates-io]` dependency, and clippy does not lint dependency targets.)
#![allow(
    clippy::tests_outside_test_module,
    clippy::expect_used,
    clippy::assertions_on_result_states
)]

// [OPUS-4.8] Regression tests for the custom-aggregate DISTINCT parse fix
// (bead sq-fldo, SPARQ-PATCHES.md §9).
//
// spargebra's `Aggregate` grammar already had a `<agg>(DISTINCT expr)`
// alternative for a registered custom-aggregate IRI, but `iriOrFunction` was
// tried first in `PrimaryExpression` and greedily accepted the bare IRI when its
// optional `ArgList` failed to parse the `(DISTINCT ?x)` argument list (DISTINCT
// is not a valid `Expression`). PEG therefore never backtracked into the
// `Aggregate` rule, so `<agg>(DISTINCT ?x)` failed to parse with a misleading
// "expected ENCODE_FOR_URI" error. The fix makes `iriOrFunction` reject a bare
// registered-aggregate IRI so the parser falls through to the `Aggregate` rule.

use oxrdf::NamedNode;
use spargebra::SparqlParser;

fn parser() -> SparqlParser {
    SparqlParser::new().with_custom_aggregate_function(
        NamedNode::new("http://example.com/agg#x").expect("valid IRI"),
    )
}

const PREFIXED_NO_DISTINCT: &str = "PREFIX agg: <http://example.com/agg#> \
     SELECT (agg:x(?v) AS ?r) WHERE { ?s ?p ?v } GROUP BY ?s";
const PREFIXED_DISTINCT: &str = "PREFIX agg: <http://example.com/agg#> \
     SELECT (agg:x(DISTINCT ?v) AS ?r) WHERE { ?s ?p ?v } GROUP BY ?s";
const FULL_IRI_DISTINCT: &str = "SELECT (<http://example.com/agg#x>(DISTINCT ?v) AS ?r) \
     WHERE { ?s ?p ?v } GROUP BY ?s";

/// The pre-fix regression: a registered custom aggregate called WITH DISTINCT
/// (both the prefixed-name and the full-IRI spelling) must parse, not error.
#[test]
fn custom_aggregate_with_distinct_parses() {
    assert!(
        parser().parse_query(PREFIXED_DISTINCT).is_ok(),
        "prefixed custom aggregate with DISTINCT must parse"
    );
    assert!(
        parser().parse_query(FULL_IRI_DISTINCT).is_ok(),
        "full-IRI custom aggregate with DISTINCT must parse"
    );
}

/// The fix must not regress the already-working DISTINCT-free form.
#[test]
fn custom_aggregate_without_distinct_still_parses() {
    assert!(
        parser().parse_query(PREFIXED_NO_DISTINCT).is_ok(),
        "custom aggregate without DISTINCT must still parse"
    );
}

/// The parsed DISTINCT and non-DISTINCT forms must carry the distinct flag
/// through to the algebra so the evaluator can act on it: the displayed query
/// retains `DISTINCT` for the DISTINCT form and omits it otherwise.
#[test]
fn distinct_flag_survives_to_algebra() {
    let distinct = parser().parse_query(PREFIXED_DISTINCT).expect("parses").to_string();
    assert!(distinct.contains("DISTINCT"), "DISTINCT must survive into the algebra: {distinct}");

    let plain = parser().parse_query(PREFIXED_NO_DISTINCT).expect("parses").to_string();
    assert!(!plain.contains("DISTINCT"), "no DISTINCT expected: {plain}");
}

/// An IRI that was NOT declared a custom aggregate is still a regular function
/// call, and `f(DISTINCT ?x)` for such an IRI remains a parse error — the fix
/// only changes behaviour for DECLARED aggregate IRIs.
#[test]
fn undeclared_iri_with_distinct_is_still_an_error() {
    let p = SparqlParser::new(); // no custom aggregate declared
    let q = "SELECT (<http://example.com/agg#x>(DISTINCT ?v) AS ?r) WHERE { ?s ?p ?v } GROUP BY ?s";
    assert!(p.parse_query(q).is_err(), "DISTINCT in a regular function call is not legal SPARQL");
}
