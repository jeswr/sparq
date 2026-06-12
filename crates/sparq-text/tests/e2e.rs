//! Engine-level integration: `text:` magic predicates inside full SPARQL,
//! joining text hits to triples through the ordinary engine (`engine`
//! feature — the default).
#![cfg(feature = "engine")]

use oxrdf::Term;
use sparq_core::Graph;
use sparq_text::{prepare_text, query_text, TextIndex};

const DATA: &str = r#"
    <http://ex/post1> <http://ex/title> "Autonomous driving milestones" .
    <http://ex/post1> <http://ex/author> <http://ex/alice> .
    <http://ex/post2> <http://ex/title> "The quick brown fox" .
    <http://ex/post2> <http://ex/author> <http://ex/bob> .
    <http://ex/post3> <http://ex/title> "Fox hunting banned" .
    <http://ex/post3> <http://ex/author> <http://ex/alice> .
    <http://ex/post4> <http://ex/title> "Café culture in ΣΟΦΊΑ"@en .
    <http://ex/post4> <http://ex/author> <http://ex/eve> .
"#;

fn setup() -> (Graph, TextIndex) {
    let g = Graph::load_str(DATA, "ntriples").unwrap();
    let idx = TextIndex::build(&g);
    (g, idx)
}

fn iri(t: &Option<Term>) -> String {
    match t {
        Some(Term::NamedNode(n)) => n.as_str().to_string(),
        other => panic!("expected an IRI, got {other:?}"),
    }
}

#[test]
fn join_text_hits_to_triples() {
    let (g, idx) = setup();
    let r = query_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           SELECT ?post ?author WHERE {
             ?post <http://ex/title> ?title .
             ?post <http://ex/author> ?author .
             ?title text:matches "fox" .
           }"#,
        &idx,
    )
    .unwrap();
    let mut posts: Vec<String> = r.rows.iter().map(|row| iri(&row[0])).collect();
    posts.sort();
    assert_eq!(posts, ["http://ex/post2", "http://ex/post3"]);
}

#[test]
fn score_variable_and_order_by() {
    let (g, idx) = setup();
    let r = query_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           SELECT ?post ?s WHERE {
             ?post <http://ex/title> ?title .
             ?title text:matches "fox" .
             ?title text:score ?s .
           } ORDER BY DESC(?s)"#,
        &idx,
    )
    .unwrap();
    assert_eq!(r.rows.len(), 2);
    // Scores are bound xsd:doubles, descending.
    let scores: Vec<f64> = r
        .rows
        .iter()
        .map(|row| match &row[1] {
            Some(Term::Literal(l)) => {
                assert_eq!(l.datatype().as_str(), "http://www.w3.org/2001/XMLSchema#double");
                l.value().parse().unwrap()
            }
            other => panic!("expected a score literal, got {other:?}"),
        })
        .collect();
    assert!(scores[0] >= scores[1]);
    assert!(scores.iter().all(|s| *s > 0.0));
}

#[test]
fn matches_any_and_prefix() {
    let (g, idx) = setup();
    let r = query_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           SELECT ?post WHERE {
             ?post <http://ex/title> ?title .
             ?title text:matchesAny "fox auto*" .
           }"#,
        &idx,
    )
    .unwrap();
    let mut posts: Vec<String> = r.rows.iter().map(|row| iri(&row[0])).collect();
    posts.sort();
    assert_eq!(posts, ["http://ex/post1", "http://ex/post2", "http://ex/post3"]);
}

#[test]
fn unicode_and_language_tagged_join() {
    let (g, idx) = setup();
    let r = query_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           SELECT ?post WHERE {
             ?post <http://ex/title> ?title .
             ?title text:matches "σοφία café" .
           }"#,
        &idx,
    )
    .unwrap();
    assert_eq!(r.rows.len(), 1);
    assert_eq!(iri(&r.rows[0][0]), "http://ex/post4");
}

#[test]
fn magic_pattern_inside_optional_and_no_match() {
    let (g, idx) = setup();
    // OPTIONAL { ... text:matches } — rewrite recurses into sub-patterns.
    let r = query_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           SELECT ?post ?hit WHERE {
             ?post <http://ex/title> ?title .
             OPTIONAL { ?title text:matches "banned" . BIND(?title AS ?hit) }
           }"#,
        &idx,
    )
    .unwrap();
    assert_eq!(r.rows.len(), 4);
    assert_eq!(r.rows.iter().filter(|row| row[1].is_some()).count(), 1);

    // No token matches -> empty VALUES -> zero rows, not an error.
    let r = query_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           SELECT ?post WHERE {
             ?post <http://ex/title> ?title . ?title text:matches "zzz" .
           }"#,
        &idx,
    )
    .unwrap();
    assert!(r.rows.is_empty());
}

#[test]
fn passthrough_without_text_patterns() {
    let (g, idx) = setup();
    let q = "SELECT ?s WHERE { ?s <http://ex/author> <http://ex/alice> }";
    let through_text = query_text(&g, q, &idx).unwrap();
    let plain = sparq_engine::query(&g, q).unwrap();
    assert_eq!(through_text.rows.len(), plain.rows.len());
    assert_eq!(through_text.rows.len(), 2);
}

#[test]
fn prepared_composition_ask() {
    let (g, idx) = setup();
    let prepared = prepare_text(
        &g,
        r#"PREFIX text: <http://sparq.dev/text#>
           ASK { ?post <http://ex/title> ?t . ?t text:matches "milestones" }"#,
        &idx,
    )
    .unwrap();
    assert!(sparq_engine::ask_prepared(&g, &prepared).unwrap());
}

#[test]
fn rewrite_errors() {
    let (g, idx) = setup();
    let cases = [
        // Query string must be a constant literal.
        (
            "SELECT ?t WHERE { ?t <http://sparq.dev/text#matches> ?q }",
            "constant query-string literal",
        ),
        // Unknown text: predicate is a typo guard.
        (
            r#"SELECT ?t WHERE { ?t <http://sparq.dev/text#fuzzy> "fox" }"#,
            "unknown magic predicate",
        ),
        // text:score needs a match pattern on the same variable.
        (
            "SELECT ?t WHERE { ?t <http://sparq.dev/text#score> ?s }",
            "no text:matches",
        ),
        // text:score object must be a variable.
        (
            r#"SELECT ?t WHERE { ?t <http://sparq.dev/text#matches> "fox" . ?t <http://sparq.dev/text#score> "1.0" }"#,
            "must be a variable",
        ),
        // Subject must be a variable.
        (
            r#"SELECT ?t WHERE { <http://ex/post1> <http://sparq.dev/text#matches> "fox" }"#,
            "must be a variable",
        ),
    ];
    for (q, needle) in cases {
        let err = query_text(&g, q, &idx).unwrap_err();
        assert!(err.contains(needle), "query {q:?}: error {err:?} should contain {needle:?}");
    }
}
