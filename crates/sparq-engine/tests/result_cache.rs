//! [OPUS-4.8] (sq-a9cn) Materialised-view / query-result cache tests.
//!
//! Only built under the opt-in `result-cache` feature; the whole file is empty
//! otherwise so the default `cargo test` is unaffected.
#![cfg(feature = "result-cache")]

use sparq_core::Graph;
use sparq_engine::PreparedQuery;
use sparq_engine::{is_cacheable, query, QueryBudget, ResultCache};

const DATA: &str = r#"@prefix ex: <http://ex/> .
    ex:a ex:p 1 . ex:b ex:p 2 . ex:c ex:p 3 .
"#;

fn g() -> Graph {
    Graph::load_str(DATA, "turtle").unwrap()
}

fn parse(q: &str) -> spargebra::Query {
    PreparedQuery::parse(q).unwrap().into_query()
}

/// A hit returns the SAME result a fresh evaluation would, and is counted as a hit.
#[test]
fn hit_matches_fresh_eval_and_counts() {
    let graph = g();
    let cache = ResultCache::new(16);
    let q = parse("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:p ?o } ORDER BY ?o");
    let budget = QueryBudget::unlimited();

    let fresh = query(
        &graph,
        "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:p ?o } ORDER BY ?o",
    )
    .unwrap();

    let first = cache.get_or_eval(&graph, &q, 0, &budget).unwrap();
    let second = cache.get_or_eval(&graph, &q, 0, &budget).unwrap();

    // Same row count + same bound terms as the uncached path.
    assert_eq!(first.rows.len(), fresh.rows.len());
    assert_eq!(first.vars, fresh.vars);
    for (a, b) in first.rows.iter().zip(fresh.rows.iter()) {
        assert_eq!(a, b);
    }
    // The two cached reads are the SAME Arc (a genuine hit, not a re-eval).
    assert!(std::sync::Arc::ptr_eq(&first, &second));

    let stats = cache.stats();
    assert_eq!(stats.misses, 1, "first call is a miss");
    assert_eq!(stats.hits, 1, "second call is a hit");
    assert_eq!(stats.entries, 1);
}

/// Whitespace / comment / prefix-spelling differences collapse to one entry
/// (keyed on the parsed algebra, not the source string).
#[test]
fn textual_variants_share_one_entry() {
    let graph = g();
    let cache = ResultCache::new(16);
    let budget = QueryBudget::unlimited();
    let a = parse("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:p ?o }");
    let b = parse("PREFIX foo: <http://ex/>\n# a comment\nSELECT   ?s WHERE {?s foo:p ?o}");

    cache.get_or_eval(&graph, &a, 0, &budget).unwrap();
    cache.get_or_eval(&graph, &b, 0, &budget).unwrap();

    let stats = cache.stats();
    assert_eq!(stats.hits, 1, "the second spelling hits the first's entry");
    assert_eq!(stats.entries, 1);
}

/// A version bump invalidates: the new version misses, and the stale generation
/// is reclaimed (no unbounded growth across epochs).
#[test]
fn version_bump_invalidates() {
    let graph = g();
    let cache = ResultCache::new(16);
    let budget = QueryBudget::unlimited();
    let q = parse("PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:p ?o }");

    cache.get_or_eval(&graph, &q, 0, &budget).unwrap();
    assert_eq!(cache.stats().entries, 1);
    // Advance the epoch (as a writer would after a mutation).
    cache.get_or_eval(&graph, &q, 1, &budget).unwrap();
    let stats = cache.stats();
    assert_eq!(stats.misses, 2, "the new version is a miss");
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.entries, 1, "the stale v0 entry was reclaimed");
}

/// A version bump after a real mutation serves the UPDATED result, never the stale one.
#[test]
fn updated_graph_after_version_bump_is_fresh() {
    let mut graph = g();
    let cache = ResultCache::new(16);
    let budget = QueryBudget::unlimited();
    let q = parse("SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }");

    let before = cache.get_or_eval(&graph, &q, 0, &budget).unwrap();
    let n_before = before.rows[0][0].as_ref().unwrap().to_string();

    graph
        .apply_delta_nquads(
            "<http://ex/d> <http://ex/p> \"4\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
            "",
        )
        .unwrap();

    // Same version => STALE hit (caller bug we deliberately demonstrate is possible).
    let stale = cache.get_or_eval(&graph, &q, 0, &budget).unwrap();
    assert_eq!(stale.rows[0][0].as_ref().unwrap().to_string(), n_before);

    // Bumped version => fresh count.
    let fresh = cache.get_or_eval(&graph, &q, 1, &budget).unwrap();
    let n_fresh = fresh.rows[0][0].as_ref().unwrap().to_string();
    assert_ne!(
        n_fresh, n_before,
        "after a mutation + version bump the count changed"
    );
}

/// Non-deterministic queries are never cached: each call re-evaluates and the store
/// stays empty.
#[test]
fn non_deterministic_not_cached() {
    let graph = g();
    let cache = ResultCache::new(16);
    let budget = QueryBudget::unlimited();

    for src in [
        "SELECT (NOW() AS ?x) WHERE {}",
        "SELECT (RAND() AS ?x) WHERE {}",
        "SELECT (UUID() AS ?x) WHERE {}",
        "SELECT (STRUUID() AS ?x) WHERE {}",
        "SELECT (BNODE() AS ?x) WHERE {}",
    ] {
        let q = parse(src);
        assert!(!is_cacheable(&q), "{src} must be classified non-cacheable");
        cache.get_or_eval(&graph, &q, 0, &budget).unwrap();
    }
    assert_eq!(
        cache.stats().entries,
        0,
        "no non-deterministic query was stored"
    );
    assert_eq!(cache.stats().hits, 0);
}

/// A deterministic query with a non-deterministic sub-expression (e.g. inside a
/// FILTER) is also refused.
#[test]
fn non_determinism_inside_filter_refused() {
    let q = parse("SELECT ?s WHERE { ?s ?p ?o FILTER(?o > RAND()) }");
    assert!(!is_cacheable(&q));
    // A plain deterministic FILTER is fine.
    let ok = parse("SELECT ?s WHERE { ?s ?p ?o FILTER(?o > 1) }");
    assert!(is_cacheable(&ok));
}

/// Standard aggregates, ORDER BY, DISTINCT, UNION, OPTIONAL, VALUES, ASK are cacheable.
#[test]
fn standard_forms_are_cacheable() {
    for src in [
        "SELECT DISTINCT ?o WHERE { ?s ?p ?o }",
        "SELECT (SUM(?o) AS ?t) WHERE { ?s ?p ?o }",
        "SELECT ?s WHERE { { ?s ?p 1 } UNION { ?s ?p 2 } }",
        "SELECT ?s ?o2 WHERE { ?s ?p ?o OPTIONAL { ?s ?p2 ?o2 } }",
        "SELECT ?x WHERE { VALUES ?x { 1 2 3 } }",
        "ASK { ?s ?p ?o }",
    ] {
        assert!(is_cacheable(&parse(src)), "{src} should be cacheable");
    }
}

/// SERVICE (remote, time-varying) and CONSTRUCT/DESCRIBE are not cacheable.
#[test]
fn service_and_graph_forms_refused() {
    assert!(!is_cacheable(&parse(
        "SELECT * WHERE { SERVICE <http://example.org/sparql> { ?s ?p ?o } }"
    )));
    assert!(!is_cacheable(&parse(
        "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }"
    )));
    assert!(!is_cacheable(&parse("DESCRIBE ?s WHERE { ?s ?p ?o }")));
}

/// Capacity 0 disables storage; an over-capacity cache evicts the LRU entry.
#[test]
fn eviction_and_disabled() {
    let graph = g();
    let budget = QueryBudget::unlimited();

    let off = ResultCache::new(0);
    let q = parse("SELECT ?s WHERE { ?s ?p ?o }");
    off.get_or_eval(&graph, &q, 0, &budget).unwrap();
    off.get_or_eval(&graph, &q, 0, &budget).unwrap();
    assert_eq!(off.stats().entries, 0);
    assert_eq!(off.stats().hits, 0, "capacity-0 never hits");

    let cache = ResultCache::new(2);
    let q1 = parse("SELECT ?s WHERE { ?s ?p 1 }");
    let q2 = parse("SELECT ?s WHERE { ?s ?p 2 }");
    let q3 = parse("SELECT ?s WHERE { ?s ?p 3 }");
    cache.get_or_eval(&graph, &q1, 0, &budget).unwrap();
    cache.get_or_eval(&graph, &q2, 0, &budget).unwrap();
    // Touch q1 so q2 becomes the LRU.
    cache.get_or_eval(&graph, &q1, 0, &budget).unwrap();
    cache.get_or_eval(&graph, &q3, 0, &budget).unwrap(); // evicts q2
    assert_eq!(cache.stats().entries, 2);
    // q1 still resident (hit), q2 evicted (miss again).
    let h0 = cache.stats().hits;
    cache.get_or_eval(&graph, &q1, 0, &budget).unwrap();
    assert_eq!(cache.stats().hits, h0 + 1, "q1 survived eviction");
    let m0 = cache.stats().misses;
    cache.get_or_eval(&graph, &q2, 0, &budget).unwrap();
    assert_eq!(cache.stats().misses, m0 + 1, "q2 was evicted, so it misses");
}

/// `clear` empties the store but keeps capacity.
#[test]
fn clear_empties() {
    let graph = g();
    let budget = QueryBudget::unlimited();
    let cache = ResultCache::new(4);
    cache
        .get_or_eval(&graph, &parse("SELECT ?s WHERE { ?s ?p ?o }"), 0, &budget)
        .unwrap();
    assert_eq!(cache.stats().entries, 1);
    cache.clear();
    assert_eq!(cache.stats().entries, 0);
    assert_eq!(cache.capacity(), 4);
}
