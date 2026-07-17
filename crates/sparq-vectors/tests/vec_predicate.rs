//! End-to-end tests for the `vec:` magic predicates (sq-k6ex): real SPARQL
//! queries with `vec:nearest` / `vec:search` evaluated against a tiny in-memory
//! `.spqv` vector store, asserting the expected neighbours. [OPUS-4.8]
#![cfg(feature = "vec-predicate")]

use oxrdf::{NamedNode, Term, Triple};
use sparq_core::Graph;
use sparq_vectors::{query_vec, QueryResult, VectorStore};

fn tmp_path(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "sparq_vec_pred_{}_{}.spqv",
        std::process::id(),
        name
    ));
    p
}

/// Five entities on the unit circle; a-style vectors point along +x, b-style
/// along +y, plus a near-x entity. The store is built (not finalized) in memory
/// — `nearest_exact` scans the build-phase backing directly.
fn fixture(name: &str) -> (Graph, VectorStore) {
    let g = Graph::load_str(
        r#"
        <http://ex/a> <http://ex/label> "alpha" .
        <http://ex/b> <http://ex/label> "beta" .
        <http://ex/c> <http://ex/label> "gamma" .
        <http://ex/d> <http://ex/label> "delta" .
        <http://ex/e> <http://ex/label> "epsilon" .
        "#,
        "ntriples",
    )
    .unwrap();
    let id = |s: &str| {
        g.id_of(&Term::NamedNode(NamedNode::new(s).unwrap()))
            .unwrap()
    };
    let mut store = VectorStore::create(tmp_path(name), 2).unwrap();
    store.put(id("http://ex/a"), &[1.0, 0.0]).unwrap(); // +x
    store.put(id("http://ex/b"), &[0.0, 1.0]).unwrap(); // +y
    store.put(id("http://ex/c"), &[0.9, 0.1]).unwrap(); // near +x
    store.put(id("http://ex/d"), &[-1.0, 0.0]).unwrap(); // -x
    store.put(id("http://ex/e"), &[0.2, 0.98]).unwrap(); // near +y
    (g, store)
}

/// Collects the single-projection IRI column of a result, in row order.
fn iris(r: &QueryResult, col: usize) -> Vec<String> {
    r.rows
        .iter()
        .filter_map(|row| match &row[col] {
            Some(Term::NamedNode(n)) => Some(n.as_str().to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn nearest_by_query_vector() {
    let (g, store) = fixture("nearest_vec");
    // Query "1,0" → the two most +x-aligned: a (exact) then c.
    let r = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:nearest ( \"1,0\" 2 ) }",
        &store,
    )
    .unwrap();
    let got = iris(&r, 0);
    assert_eq!(
        got,
        vec!["http://ex/a".to_string(), "http://ex/c".to_string()],
        "{r:?}"
    );
}

#[test]
fn nearest_by_seed_iri_excludes_self() {
    let (g, store) = fixture("nearest_seed");
    // Neighbours of <a> (which is +x); a itself is excluded → c is nearest.
    let r = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:nearest ( <http://ex/a> 1 ) }",
        &store,
    )
    .unwrap();
    let got = iris(&r, 0);
    assert_eq!(
        got,
        vec!["http://ex/c".to_string()],
        "seed must be excluded: {r:?}"
    );
}

#[test]
fn nearest_joins_to_surrounding_bgp() {
    let (g, store) = fixture("nearest_join");
    // The neighbours' labels, joined through the ordinary triple pattern.
    let r = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?label WHERE {
           ?node vec:nearest ( \"0,1\" 2 ) .
           ?node <http://ex/label> ?label .
         }",
        &store,
    )
    .unwrap();
    let mut labels: Vec<String> = r
        .rows
        .iter()
        .filter_map(|row| match &row[0] {
            Some(Term::Literal(l)) => Some(l.value().to_string()),
            _ => None,
        })
        .collect();
    labels.sort();
    // "0,1" → b (+y) and e (near +y): labels beta, epsilon.
    assert_eq!(
        labels,
        vec!["beta".to_string(), "epsilon".to_string()],
        "{r:?}"
    );
}

/// [GPT-5.6] sq-1ijoc: RDF 1.2 ground triple terms returned by vector search
/// must survive the rewrite's inline VALUES table, including nested terms.
#[test]
fn nearest_returns_nested_ground_triple_term() {
    let g = Graph::load_str(
        r#"
        <http://ex/r> <http://ex/reifies> <<( <http://ex/s> <http://ex/p> <<( <http://ex/a> <http://ex/q> <http://ex/o> )>> )>> .
        "#,
        "ntriples",
    )
    .unwrap();
    let named = |iri: &str| NamedNode::new(iri).unwrap();
    let inner = Term::Triple(Box::new(Triple::new(
        named("http://ex/a"),
        named("http://ex/q"),
        named("http://ex/o"),
    )));
    let expected = Term::Triple(Box::new(Triple::new(
        named("http://ex/s"),
        named("http://ex/p"),
        inner,
    )));
    let triple_id = g
        .id_of(&expected)
        .expect("fixture triple term must be dictionary encoded");
    let mut store = VectorStore::create(tmp_path("triple_term"), 2).unwrap();
    store.put(triple_id, &[1.0, 0.0]).unwrap();

    let r = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:nearest ( \"1,0\" 1 ) }",
        &store,
    )
    .unwrap();

    assert_eq!(r.rows, vec![vec![Some(expected)]], "{r:?}");
}

#[test]
fn search_binds_score_and_orders() {
    let (g, store) = fixture("search_score");
    // vec:search binds the cosine score; ORDER BY DESC recovers best-first.
    let r = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node ?score WHERE {
           ( ?node ?score ) vec:search ( \"1,0\" 3 )
         } ORDER BY DESC(?score)",
        &store,
    )
    .unwrap();
    let order = iris(&r, 0);
    // a (cos 1.0) > c (cos ~0.994) > b/e/d further; top-3 best-first.
    assert_eq!(
        order,
        vec![
            "http://ex/a".to_string(),
            "http://ex/c".to_string(),
            "http://ex/e".to_string()
        ],
        "{r:?}"
    );
    // a's score is ~1.0 (exact alignment).
    let a_score = match &r.rows[0][1] {
        Some(Term::Literal(l)) => l.value().parse::<f64>().unwrap(),
        other => panic!("expected score literal, got {other:?}"),
    };
    assert!(
        (a_score - 1.0).abs() < 1e-5,
        "a should be cosine 1.0, got {a_score}"
    );
}

#[test]
fn unembedded_seed_yields_no_rows() {
    let (g, store) = fixture("missing_seed");
    let r = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:nearest ( <http://ex/nope> 5 ) }",
        &store,
    )
    .unwrap();
    assert!(
        r.is_empty(),
        "an unembedded/absent seed has no neighbours: {r:?}"
    );
}

#[test]
fn dimension_mismatch_is_an_error() {
    let (g, store) = fixture("dim_mismatch");
    let err = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:nearest ( \"1,0,0\" 2 ) }",
        &store,
    )
    .unwrap_err();
    assert!(
        err.contains("dims"),
        "expected a dimension error, got: {err}"
    );
}

#[test]
fn unknown_vec_predicate_is_rejected() {
    let (g, store) = fixture("unknown_pred");
    let err = query_vec(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:teleport ( \"1,0\" 2 ) }",
        &store,
    )
    .unwrap_err();
    assert!(err.contains("unknown magic predicate"), "got: {err}");
}

#[test]
fn no_vec_predicate_passes_through() {
    let (g, store) = fixture("passthrough");
    // A query with no vec: predicate must evaluate exactly as the plain engine would.
    let r = query_vec(
        &g,
        "SELECT ?node WHERE { ?node <http://ex/label> \"alpha\" }",
        &store,
    )
    .unwrap();
    assert_eq!(iris(&r, 0), vec!["http://ex/a".to_string()], "{r:?}");
}

/// [OPUS-4.8] Regression: a query that legitimately uses the RDF collection
/// vocabulary (`rdf:first`/`rdf:rest`) but NO `vec:` predicate must pass through
/// unchanged — the rewrite must not strip those triples (it previously did,
/// unconditionally). Here we query an actual `rdf:first` cell and expect the
/// element back.
#[test]
fn rdf_collection_without_vec_predicate_is_preserved() {
    let g = Graph::load_str(
        r#"
        <http://ex/list> <http://www.w3.org/1999/02/22-rdf-syntax-ns#first> <http://ex/elem> .
        <http://ex/list> <http://www.w3.org/1999/02/22-rdf-syntax-ns#rest> <http://www.w3.org/1999/02/22-rdf-syntax-ns#nil> .
        "#,
        "ntriples",
    )
    .unwrap();
    // An empty store (no fingerprint) — exercises the unbound/legacy path too.
    let store = VectorStore::create(tmp_path("collection_passthrough"), 2).unwrap();
    let r = query_vec(
        &g,
        "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         SELECT ?elem WHERE { <http://ex/list> rdf:first ?elem }",
        &store,
    )
    .unwrap();
    assert_eq!(iris(&r, 0), vec!["http://ex/elem".to_string()], "{r:?}");
}
