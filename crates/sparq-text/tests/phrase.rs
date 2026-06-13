//! Phrase queries (the opt-in positional index): adjacency + order. [OPUS-4.8]
//!
//! `phrase("foo bar")` matches a document only where `foo` is immediately
//! followed by `bar` (consecutive positions, in order). These tests pin: a
//! matching adjacent phrase, a near-miss (same tokens, non-adjacent), order
//! sensitivity ("foo bar" != "bar foo"), the analyzer (casefolding) agreeing
//! with indexing, and — load-bearing — that the cheap non-positional default
//! path is byte-for-byte unchanged.

use oxrdf::Term;
use sparq_core::Graph;
use sparq_text::TextIndex;

/// A graph whose `ex:comment` objects are the given literals (one subject each).
fn graph_of(literals: &[&str]) -> Graph {
    let nt: String = literals
        .iter()
        .enumerate()
        .map(|(i, l)| format!("<http://ex/s{i}> <http://ex/comment> {l} .\n"))
        .collect();
    Graph::load_str(&nt, "ntriples").unwrap()
}

/// The lexical values of the phrase-hit ids, in the returned (ascending-id) order.
fn values(graph: &Graph, ids: &[sparq_core::dict::Id]) -> Vec<String> {
    ids.iter()
        .map(|&id| match graph.dict.term(id) {
            Term::Literal(l) => l.value().to_string(),
            other => panic!("hit {id} is not a literal: {other}"),
        })
        .collect()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[test]
fn adjacent_phrase_matches() {
    let g = graph_of(&[
        r#""the quick brown fox""#,
        r#""a quick brown dog""#,
        r#""nothing relevant here""#,
    ]);
    let idx = TextIndex::build_with_positions(&g);
    assert!(idx.has_positions());
    assert_eq!(
        sorted(values(&g, &idx.phrase("quick brown"))),
        ["a quick brown dog", "the quick brown fox"]
    );
    // The full phrase narrows to the one document containing it adjacently.
    assert_eq!(values(&g, &idx.phrase("quick brown fox")), ["the quick brown fox"]);
}

#[test]
fn non_adjacent_is_a_near_miss() {
    // All three tokens are present, but never as a consecutive run.
    let g = graph_of(&[r#""quick and brown fox""#]);
    let idx = TextIndex::build_with_positions(&g);
    // The two tokens are separated by "and" — no adjacency, no match.
    assert!(idx.phrase("quick brown").is_empty());
    // …yet the same tokens DO match under the (positionless) AND search,
    // which only requires co-occurrence.
    assert_eq!(idx.search("quick brown").len(), 1);
    // Adjacent sub-phrases that DO occur still match.
    assert_eq!(values(&g, &idx.phrase("brown fox")), ["quick and brown fox"]);
}

#[test]
fn order_is_significant() {
    let g = graph_of(&[r#""foo bar baz""#]);
    let idx = TextIndex::build_with_positions(&g);
    assert_eq!(values(&g, &idx.phrase("foo bar")), ["foo bar baz"]);
    // Reversed order is a different phrase and does not match.
    assert!(idx.phrase("bar foo").is_empty());
}

#[test]
fn repeated_token_phrase() {
    // A token repeated in the phrase must align with repeats in the doc.
    let g = graph_of(&[r#""ha ha ha""#, r#""ha ho ha""#]);
    let idx = TextIndex::build_with_positions(&g);
    assert_eq!(values(&g, &idx.phrase("ha ha")), ["ha ha ha"]);
    // "ho ha" is adjacent only in the second doc.
    assert_eq!(values(&g, &idx.phrase("ho ha")), ["ha ho ha"]);
}

#[test]
fn single_token_phrase_is_presence() {
    let g = graph_of(&[r#""alpha beta""#, r#""beta gamma""#, r#""delta""#]);
    let idx = TextIndex::build_with_positions(&g);
    assert_eq!(
        sorted(values(&g, &idx.phrase("beta"))),
        ["alpha beta", "beta gamma"]
    );
    assert!(idx.phrase("missing").is_empty());
}

#[test]
fn phrase_honours_the_analyzer() {
    // Same UAX #29 segmentation + Unicode casefolding as indexing.
    let g = graph_of(&[r#""The Quick Brown Fox""#, r#""ΣΟΦΊΑ City""#]);
    let idx = TextIndex::build_with_positions(&g);
    // Casefolding: the query case is irrelevant.
    assert_eq!(values(&g, &idx.phrase("QUICK brown")), ["The Quick Brown Fox"]);
    // Unicode casefolding agrees with the document tokenization.
    assert_eq!(values(&g, &idx.phrase("σοφία city")), ["ΣΟΦΊΑ City"]);
    // Punctuation between words is segmentation, not a token: the phrase
    // "quick, brown" tokenizes to ["quick", "brown"] and still matches.
    assert_eq!(values(&g, &idx.phrase("quick, brown")), ["The Quick Brown Fox"]);
}

#[test]
fn empty_phrase_matches_nothing() {
    let g = graph_of(&[r#""anything at all""#]);
    let idx = TextIndex::build_with_positions(&g);
    assert!(idx.phrase("").is_empty());
    assert!(idx.phrase("  ,.;! ").is_empty());
}

#[test]
fn positions_off_by_default() {
    // The cheap default records NO positions and is byte-for-byte the index
    // that existed before phrase support.
    let g = graph_of(&[r#""the quick brown fox""#, r#""lazy dog""#]);
    let plain = TextIndex::build(&g);
    assert!(!plain.has_positions());
    // The default and positional builds agree on every BM25-scored search:
    // positions are a pure add-on, never altering the existing query paths.
    let positional = TextIndex::build_with_positions(&g);
    for q in ["quick fox", "the", "br*", "dog", "missing"] {
        assert_eq!(plain.search(q), positional.search(q), "search({q:?}) diverged");
        assert_eq!(plain.search_any(q), positional.search_any(q));
    }
    // …but they are NOT equal as values (one carries positions, one does not).
    assert_ne!(plain, positional);
}

#[test]
#[should_panic(expected = "requires a positional index")]
fn phrase_without_positions_panics() {
    let g = graph_of(&[r#""the quick brown fox""#]);
    let plain = TextIndex::build(&g);
    let _ = plain.phrase("quick brown");
}

/// The parallel build (>= MIN_PARALLEL_TERMS docs) merges per-shard position
/// maps in `append_shard`; this exercises that path and pins it equal to the
/// serial build. [OPUS-4.8]
#[cfg(feature = "parallel")]
#[test]
fn parallel_positional_build_merges_shards() {
    // 5000 distinct literals (> the 4096 parallel threshold). Most are noise;
    // a handful carry the target phrase, each in a DISTINCT literal (a unique
    // suffix keeps them separate documents) at scattered doc ids, so the
    // matches must be assembled from several shards.
    let mut lits: Vec<String> = (0..5000).map(|i| format!(r#""filler token number {i}""#)).collect();
    let seeded: Vec<usize> = (0..5000).step_by(900).collect();
    for &i in &seeded {
        lits[i] = format!(r#""the quick brown fox doc{i}""#);
    }
    let refs: Vec<&str> = lits.iter().map(String::as_str).collect();
    let g = graph_of(&refs);

    let idx = TextIndex::build_with_positions(&g);
    assert!(idx.has_positions());
    // Adjacency holds for every seeded doc (matches span multiple shards); the
    // near-miss (reversed order) never matches.
    let hits = idx.phrase("quick brown fox");
    assert_eq!(hits.len(), seeded.len(), "one match per seeded literal");
    assert!(idx.phrase("fox brown").is_empty());
    // Every hit carries the adjacent phrase — the shard merge did not drop or
    // scramble offsets.
    let vals = values(&g, &hits);
    assert!(
        vals.iter().all(|v| v.starts_with("the quick brown fox doc")),
        "got {vals:?}"
    );
}

#[test]
fn positions_via_apply_delta() {
    use oxrdf::{Literal, NamedNode};

    // with_positions() seeds an empty positional index that delta-fed docs
    // populate — phrase search must then work over the inserted literals.
    let mut graph = Graph::load_str("", "ntriples").unwrap();
    let mut idx = TextIndex::with_positions();
    assert!(idx.has_positions());

    let inserts = [[
        Term::NamedNode(NamedNode::new_unchecked("http://ex/s")),
        Term::NamedNode(NamedNode::new_unchecked("http://ex/p")),
        Term::Literal(Literal::new_simple_literal("the quick brown fox")),
    ]];
    graph.apply_delta(&inserts, &[]).unwrap();
    idx.apply_delta(&graph, &inserts, &[]);

    assert_eq!(values(&graph, &idx.phrase("quick brown")), ["the quick brown fox"]);
    assert!(idx.phrase("brown quick").is_empty());
    // Equal to a from-scratch positional rebuild (the delta differential
    // property extends to positions).
    assert_eq!(idx, TextIndex::build_with_positions(&graph));
}
