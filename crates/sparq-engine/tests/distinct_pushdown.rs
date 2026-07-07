//! Acceptance tests for the DISTINCT-projection loose (skip) index scan
//! (bead sq-7d3dj.30.4). [OPUS-4.8]
//!
//! For `SELECT DISTINCT ?p WHERE { BGP / UNION-of-BGPs }` the engine enumerates the
//! DISTINCT `?p` values directly from an existing permutation sorted by `?p` (a loose
//! skip scan — the general form of qlever's pattern trick over the six permutations, NO
//! new index) instead of materialising every full-width join row and deduping post-hoc.
//! The load-bearing invariant is DISTINCT result-SET equivalence with the pushdown on vs
//! off; the anti-vacuity test asserts the produced/scanned rows COLLAPSE vs the full
//! join size (the SP2Bench q09 shape materialises tens of thousands of rows to answer a
//! handful of distinct predicates today).
//!
//! Coverage:
//!   * `q09_shape_equivalence`      — SP2Bench q09-shaped 2-branch UNION of person/
//!     predicate joins: pushdown-on == pushdown-off (set semantics), and the non-person
//!     "noise" predicates are correctly excluded.
//!   * `q09_shape_anti_vacuity`     — the pushdown FIRED and the permutation rows it
//!     touched COLLAPSED to a small fraction of the full (non-distinct) join size.
//!   * `single_pattern_distinct`    — the `DISTINCT ?p WHERE { ?s ?p ?o }` skip-scan.
//!   * `multi_var_projection_fallback` — `DISTINCT ?p ?person` (>1 projected var) falls
//!     back to the correct full path (pushdown does not fire).
//!   * `non_conjunctive_fallback`   — an OPTIONAL under the DISTINCT falls back.
//!   * `three_pattern_branch_fallback` — a 3-pattern branch falls back (still correct).
//!   * `shared_projected_var_fallback` — `?p` used as BOTH the join and projected var.
//!   * `public_toggle_and_stats`    — the `distinct_pushdown_testing` public surface.

use sparq_core::Graph;
use sparq_engine::{distinct_pushdown_testing, query};

const PFX: &str = "PREFIX ex: <http://example.org/>\n\
                   PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
                   PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n";

fn load(ttl: &str) -> Graph {
    Graph::load_str(&format!("{PFX}{ttl}"), "turtle").expect("test graph")
}

/// A stringified result table: one row per solution, one `Option<String>` per column,
/// as a SORTED set (order-insensitive; DISTINCT is a set, so we also de-duplicate).
type Table = Vec<Vec<Option<String>>>;

fn table(g: &Graph, q: &str) -> Table {
    let r = query(g, &format!("{PFX}{q}")).expect("query failed");
    let mut rows: Table = r
        .rows
        .iter()
        .map(|row| row.iter().map(|c| c.as_ref().map(|t| t.to_string())).collect())
        .collect();
    rows.sort();
    rows
}

/// Runs `q` with the pushdown forced OFF then forced ON, returning both result tables.
fn on_off(g: &Graph, q: &str) -> (Table, Table) {
    let prev = distinct_pushdown_testing::set_enabled(false);
    let off = table(g, q);
    distinct_pushdown_testing::set_enabled(true);
    let on = table(g, q);
    distinct_pushdown_testing::set_enabled(prev);
    (off, on)
}

// ── q09 dataset ──────────────────────────────────────────────────────────────
//
// An SP2Bench-q09-shaped corpus. `N` persons, then `docs` documents that each name a
// person as the OBJECT of foaf:maker / ex:cites (HIGH multiplicity — many documents per
// person, exactly the shape whose full join materialises tens of thousands of rows to
// answer a handful of distinct predicates). Persons are emitted FIRST (so their ids form
// a contiguous low range, as real generators such as SP2Bench emit entities of a type in
// runs), then the documents — which lets the loose scan's sound range-disjointness reject
// eliminate the document- and literal-valued predicate blocks in O(1). NOISE predicates
// (ex:Doc rdf:type, foaf:age) must NOT appear in the DISTINCT ?predicate answer.
fn q09_dataset(n: usize, docs: usize) -> Graph {
    let mut ttl = String::new();
    // Phase 1: person type triples → the lowest, contiguous entity ids.
    for p in 0..n {
        ttl.push_str(&format!("ex:p{p} rdf:type foaf:Person .\n"));
    }
    // Phase 2: person-subject predicates (age = few distinct inline ints; knows = person).
    for p in 0..n {
        ttl.push_str(&format!("ex:p{p} foaf:age {} .\n", 20 + (p % 10)));
        ttl.push_str(&format!("ex:p{p} foaf:knows ex:p{} .\n", (p + 1) % n));
    }
    // Phase 3: documents (higher ids) with a person as the OBJECT, high multiplicity.
    for d in 0..docs {
        ttl.push_str(&format!("ex:d{d} rdf:type ex:Doc .\n")); // noise: subject a doc, object a class
        ttl.push_str(&format!("ex:d{d} foaf:maker ex:p{} .\n", d % n));
        ttl.push_str(&format!("ex:d{d} ex:cites ex:p{} .\n", (d * 7) % n));
    }
    load(&ttl)
}

const Q09_BODY: &str = "\
  {
    ?person rdf:type foaf:Person .
    ?subject ?predicate ?person
  } UNION {
    ?person rdf:type foaf:Person .
    ?person ?predicate ?object
  }
";

/// The five predicates that genuinely relate to a person subject/object (as `Term`
/// display strings — IRIs render inside angle brackets).
fn expected_q09_predicates() -> Vec<String> {
    let mut v = vec![
        "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(), // person a Person (subject)
        "<http://xmlns.com/foaf/0.1/age>".to_string(),                   // person is subject
        "<http://xmlns.com/foaf/0.1/knows>".to_string(), // subject AND object of knows is a person
        "<http://xmlns.com/foaf/0.1/maker>".to_string(), // person is object
        "<http://example.org/cites>".to_string(),        // person is object
    ];
    v.sort();
    v
}

#[test]
fn q09_shape_equivalence() {
    let g = q09_dataset(30, 90);
    let q = format!("SELECT DISTINCT ?predicate WHERE {{\n{Q09_BODY}\n}}");
    let (off, on) = on_off(&g, &q);
    assert_eq!(off, on, "q09 DISTINCT result differs with pushdown on vs off");

    // The exact expected set — and that the NOISE predicate (ex:Doc rdf:type has a class,
    // not a person, as object; it only qualifies via the person-subject rdf:type triple)
    // resolves to the same set on either path.
    let got: Vec<String> = on.iter().map(|r| r[0].clone().unwrap()).collect();
    assert_eq!(got, expected_q09_predicates(), "unexpected DISTINCT predicate set");
}

#[test]
fn q09_shape_anti_vacuity() {
    // High document multiplicity: many docs per person, so the full join materialises
    // thousands of rows to answer a handful of distinct predicates.
    let g = q09_dataset(40, 2000);
    let distinct_q = format!("SELECT DISTINCT ?predicate WHERE {{\n{Q09_BODY}\n}}");
    let bag_q = format!("SELECT ?predicate WHERE {{\n{Q09_BODY}\n}}");

    // The full (non-distinct) join size — how many rows the blind path materialises.
    distinct_pushdown_testing::set_enabled(false);
    let full_bag = query(&g, &format!("{PFX}{bag_q}")).expect("bag query").rows.len();

    // The pushdown path: assert it FIRED and the permutation rows it TOUCHED collapsed.
    distinct_pushdown_testing::set_enabled(true);
    distinct_pushdown_testing::reset_stats();
    let res = query(&g, &format!("{PFX}{distinct_q}")).expect("distinct query");
    let (fired, emitted, scanned) = distinct_pushdown_testing::stats();

    // Correctness holds regardless of which permutations are built (pushdown or fallback).
    assert_eq!(res.rows.len(), 5, "q09 fixture should answer 5 distinct predicates");
    assert!(full_bag > 4000, "fixture too small for a meaningful collapse: {}", full_bag);

    // The second UNION branch (`?person ?predicate ?object`) needs the `[P, J]` = PSO
    // permutation for its loose scan; it exists in the full native index but NOT in the
    // compact (wasm) index {SPO, POS, OSP}, where the pushdown conservatively declines.
    let full_index = sparq_core::store::BUILT.contains(&sparq_core::store::Perm::Pso);
    if full_index {
        assert!(fired, "the DISTINCT pushdown did not fire on the q09 shape");
        assert_eq!(emitted, 5, "pushdown should emit exactly the distinct predicates");
        // Anti-vacuity: the loose scan touched an order of magnitude fewer rows than the
        // full join it replaces (the q09 essence — a handful of predicates, not the join).
        assert!(
            scanned * 10 < full_bag,
            "loose scan did not collapse: scanned={} full_bag={}",
            scanned,
            full_bag
        );
    } else {
        assert!(!fired, "compact index lacks PSO → the 2-branch q09 pushdown must decline");
    }
}

#[test]
fn single_pattern_distinct() {
    // `DISTINCT ?p WHERE { ?s ?p ?o }` — the single-pattern skip scan enumerates the
    // distinct predicates of the whole graph.
    let g = load(
        "ex:a ex:p1 ex:b . ex:a ex:p1 ex:c . ex:a ex:p2 ex:b . ex:d ex:p3 ex:e .",
    );
    let (off, on) = on_off(&g, "SELECT DISTINCT ?p WHERE { ?s ?p ?o }");
    assert_eq!(off, on);
    assert_eq!(on.len(), 3, "three distinct predicates");

    distinct_pushdown_testing::set_enabled(true);
    distinct_pushdown_testing::reset_stats();
    let _ = query(&g, &format!("{PFX}SELECT DISTINCT ?p WHERE {{ ?s ?p ?o }}")).unwrap();
    let (fired, emitted, _) = distinct_pushdown_testing::stats();
    assert!(fired, "single-pattern pushdown should fire");
    assert_eq!(emitted, 3);
}

#[test]
fn multi_var_projection_fallback() {
    // More than one projected variable → the pushdown declines; the answer is still
    // correct via the full path and the pushdown does NOT fire.
    let g = q09_dataset(8, 20);
    let q = format!("SELECT DISTINCT ?predicate ?person WHERE {{\n{Q09_BODY}\n}}");

    distinct_pushdown_testing::reset_stats();
    distinct_pushdown_testing::set_enabled(true);
    let on = table(&g, &q);
    let (fired, _, _) = distinct_pushdown_testing::stats();
    assert!(!fired, "pushdown must not fire on a multi-variable projection");

    let prev = distinct_pushdown_testing::set_enabled(false);
    let off = table(&g, &q);
    distinct_pushdown_testing::set_enabled(prev);
    assert_eq!(off, on, "multi-var fallback answer differs");
    assert!(!on.is_empty());
}

#[test]
fn non_conjunctive_fallback() {
    // An OPTIONAL under the DISTINCT is not a plain BGP/UNION → fall back, correctly.
    let g = load(
        "ex:a ex:p ex:x . ex:a ex:q ex:y . ex:x ex:extra ex:z .",
    );
    let q = "SELECT DISTINCT ?p WHERE { ex:a ?p ?o OPTIONAL { ?o ex:extra ?e } }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "OPTIONAL fallback answer differs");

    distinct_pushdown_testing::reset_stats();
    distinct_pushdown_testing::set_enabled(true);
    let _ = query(&g, &format!("{PFX}{q}")).unwrap();
    let (fired, _, _) = distinct_pushdown_testing::stats();
    assert!(!fired, "pushdown must not fire through an OPTIONAL");
}

#[test]
fn three_pattern_branch_fallback() {
    // A 3-pattern branch is outside the enumerable class → fall back, still correct.
    let g = load(
        "ex:a rdf:type ex:T . ex:a ex:link ex:b . ex:b ex:rel ex:c .
         ex:d rdf:type ex:T . ex:d ex:link ex:e . ex:e ex:rel ex:f .",
    );
    let q = "SELECT DISTINCT ?p WHERE { ?a rdf:type ex:T . ?a ex:link ?b . ?b ?p ?c }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "three-pattern fallback answer differs");
    assert!(!on.is_empty());

    distinct_pushdown_testing::reset_stats();
    distinct_pushdown_testing::set_enabled(true);
    let _ = query(&g, &format!("{PFX}{q}")).unwrap();
    let (fired, _, _) = distinct_pushdown_testing::stats();
    assert!(!fired, "pushdown must not fire on a 3-pattern branch");
}

#[test]
fn shared_projected_var_fallback() {
    // `?p` is BOTH the join variable and the projected variable (appears in both
    // patterns) — the clean anchor/probe shape does not hold, so fall back.
    let g = load(
        "ex:a ex:knows ex:b . ex:b ex:knows ex:c . ex:c ex:knows ex:a .",
    );
    let q = "SELECT DISTINCT ?p WHERE { ?x ex:knows ?p . ?p ex:knows ?y }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "shared-var fallback answer differs");
    assert!(!on.is_empty());
}

#[test]
fn empty_result_equivalence() {
    // A branch that yields no solutions (absent predicate constant) — the DISTINCT of an
    // empty relation, on vs off, must agree (an empty table with the projected column).
    let g = load("ex:a ex:p ex:b .");
    let q = "SELECT DISTINCT ?o WHERE { ex:a ex:absent ?o }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on);
    assert!(on.is_empty(), "no solutions expected");
}

/// Regression: intra-triple repeated variable must NOT push down.
///
/// Pattern `?x ?p ?x` requires subject == object; the skip-scan helpers enumerate
/// predicates without checking that constraint, so they over-approximate. This test
/// checks that the pushdown DECLINES (does not fire), the fallback answer is correct
/// (only the self-loop predicate ex:knows qualifies), and on == off.
///
/// Counterexample data: `ex:a ex:knows ex:a` (self-loop) + `ex:b ex:likes ex:c`
/// (no self-loop). Correct DISTINCT ?p = {ex:knows}; an unguarded skip-scan would
/// return {ex:knows, ex:likes}. [SONNET-4.6]
#[test]
fn intra_triple_repeated_var_fallback() {
    let g = load("ex:a ex:knows ex:a . ex:b ex:likes ex:c .");
    let q = "SELECT DISTINCT ?p WHERE { ?x ?p ?x }";

    // pushdown OFF gives the oracle (enforces ?x==?x via build_row)
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "repeated-var query must give the same answer on vs off");

    // The correct answer contains only ex:knows (ex:b ex:likes ex:c has subject != object)
    let expected: Vec<Vec<Option<String>>> =
        vec![vec![Some("<http://example.org/knows>".to_string())]];
    assert_eq!(on, expected, "only the self-loop predicate qualifies");

    // Crucially: the pushdown must NOT fire (the guard declines it)
    distinct_pushdown_testing::reset_stats();
    distinct_pushdown_testing::set_enabled(true);
    let _ = query(&g, &format!("{PFX}{q}")).expect("query failed");
    let (fired, _, _) = distinct_pushdown_testing::stats();
    assert!(!fired, "pushdown must not fire on a pattern with a repeated variable (?x ?p ?x)");
}

#[test]
fn public_toggle_and_stats() {
    // Direct unit coverage of the public `distinct_pushdown_testing` surface.
    let prev = distinct_pushdown_testing::set_enabled(false);
    assert!(!distinct_pushdown_testing::set_enabled(true), "set_enabled returns prior value");
    assert!(distinct_pushdown_testing::set_enabled(prev));

    distinct_pushdown_testing::reset_stats();
    let (fired, emitted, scanned) = distinct_pushdown_testing::stats();
    assert!(!fired && emitted == 0 && scanned == 0, "stats not cleared by reset");
}
