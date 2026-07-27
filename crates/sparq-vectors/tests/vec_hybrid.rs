//! [SONNET-4.6] sq-lhcot.4 — the `vec:hybrid` magic predicate end to end: hybrid retrieval
//! (dense + caller-supplied arms, fused by deterministic weighted RRF) and out-of-process
//! reranking, expressed INSIDE plain SPARQL and evaluated through the ordinary engine.
//!
//! What these tests pin, in order:
//!
//! 1. the rewrite binds `?node` / `?score` / `?rank` / `?prov`, and `?prov` reports which arm
//!    ranked each row and where;
//! 2. an arm genuinely changes the answer — a row NO dense neighbour search returns still
//!    surfaces when a sparse arm ranks it, and consensus reorders the result;
//! 3. the second stage reorders under `?rank`, and its fail-open / fail-closed policy behaves as
//!    documented (fail-open never marks a row `rerank=…`);
//! 4. the guard rails: `vec:hybrid` through the plain `query_vec` is a hard error, a text query
//!    without a query embedder is a hard error, and a `vec:` query with no hybrid pattern is
//!    unaffected by the config.
//!
//! No test here asserts that fusion or reranking IMPROVES retrieval — that is an empirical
//! question about a corpus, answered by `hybrid::ablate`, not by this crate.

#![cfg(feature = "vec-predicate")]

use oxrdf::{NamedNode, Term};
use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_vectors::hybrid::{
    ArmQuery, FusedHit, HybridConfig, RerankPolicy, Reranker, Rescored,
};
use sparq_vectors::{
    parse_provenance, query_vec, query_vec_hybrid, query_vec_hybrid_with_budget,
    rewrite_query_hybrid, QueryResult, VectorStore,
};

/// Four entities on the unit square. `a` is the x-axis; `c` is nearly x; `b` is the y-axis; `d`
/// is embedded but far from x. A dense query of `1,0` therefore ranks `a`, `c`, then `d`/`b`.
const TTL: &str = r#"
<http://ex/a> <http://ex/p> "a" .
<http://ex/b> <http://ex/p> "b" .
<http://ex/c> <http://ex/p> "c" .
<http://ex/d> <http://ex/p> "d" .
"#;

fn fixture(tag: &str) -> (Graph, VectorStore, std::path::PathBuf) {
    let g = Graph::load_str(TTL, "ntriples").unwrap();
    let path = std::env::temp_dir().join(format!(
        "sparq-vec-hybrid-{}-{}.spqv",
        tag,
        std::process::id()
    ));
    let mut store = VectorStore::create(&path, 2).unwrap();
    store.put(id(&g, "a"), &[1.0, 0.0]).unwrap();
    store.put(id(&g, "b"), &[0.0, 1.0]).unwrap();
    store.put(id(&g, "c"), &[0.95, 0.05]).unwrap();
    store.put(id(&g, "d"), &[0.6, 0.8]).unwrap();
    store.finalize().unwrap();
    (g, store, path)
}

fn iri(local: &str) -> Term {
    Term::NamedNode(NamedNode::new(format!("http://ex/{local}")).unwrap())
}

fn id(g: &Graph, local: &str) -> Id {
    g.id_of(&iri(local)).unwrap()
}

/// `(node local name, score, rank, provenance)` per row, ordered by the bound `?rank` so the
/// assertions read in result order (VALUES rows carry no order through joins).
fn rows(r: &QueryResult) -> Vec<(String, f64, i64, String)> {
    let mut out: Vec<(String, f64, i64, String)> = r
        .rows
        .iter()
        .map(|row| {
            let node = match row[0].as_ref().unwrap() {
                Term::NamedNode(n) => n.as_str().rsplit('/').next().unwrap().to_string(),
                other => panic!("expected an IRI, got {}", other),
            };
            let lit = |i: usize| match row[i].as_ref().unwrap() {
                Term::Literal(l) => l.value().to_string(),
                other => panic!("expected a literal, got {}", other),
            };
            (
                node,
                lit(1).parse().unwrap(),
                lit(2).parse().unwrap(),
                lit(3),
            )
        })
        .collect();
    out.sort_by_key(|r| r.2);
    out
}

const HYBRID_Q: &str = r#"
PREFIX vec: <http://sparq.dev/vec#>
SELECT ?node ?score ?rank ?prov WHERE {
  ( ?node ?score ?rank ?prov ) vec:hybrid ( "1,0" 3 )
}"#;

/// An arm that returns a fixed ranking, ignoring the query — a stand-in for a lexical/BM25 or
/// structural retriever, whose only job here is to be a SECOND opinion the fusion must honour.
fn fixed_arm(ranked: Vec<(Id, f64)>) -> sparq_vectors::ArmFn<'static> {
    Box::new(move |_q: &ArmQuery<'_>, n: usize| {
        Ok(ranked.iter().copied().take(n).collect::<Vec<_>>())
    })
}

#[test]
fn hybrid_binds_node_score_rank_and_rank_provenance() {
    let (g, store, _p) = fixture("bind");
    // A sparse arm that ranks `b` first — `b` is the WORST dense match for the query "1,0".
    let cfg = HybridConfig::new().arm("text", 1.0, fixed_arm(vec![(id(&g, "b"), 9.0)]));

    let r = query_vec_hybrid(&g, HYBRID_Q, &store, &cfg).unwrap();
    let rows = rows(&r);

    // Ranks are a gap-free 1..n in fused order.
    assert_eq!(
        rows.iter().map(|r| r.2).collect::<Vec<_>>(),
        (1..=rows.len() as i64).collect::<Vec<_>>()
    );
    // `a` is the dense rank-1 hit and is unranked by the text arm; `b` is text rank-1 only.
    let a = rows.iter().find(|r| r.0 == "a").unwrap();
    assert_eq!(a.3, "vector=1", "the dense arm alone ranked a, at rank 1");
    let b = rows.iter().find(|r| r.0 == "b").unwrap();
    assert!(
        b.3.contains("text=1"),
        "b must carry its text-arm provenance, got {}",
        b.3
    );
    // The provenance parses back into (arm, rank) pairs.
    assert_eq!(parse_provenance(&a.3).unwrap(), vec![("vector".into(), 1)]);
    // Scores are the fused RRF scores: finite, positive, and non-increasing down the ranking.
    assert!(rows.iter().all(|r| r.1 > 0.0 && r.1.is_finite()));
    for w in rows.windows(2) {
        assert!(w[0].1 >= w[1].1, "fused scores must not increase with rank");
    }
}

#[test]
fn an_arm_changes_the_answer_consensus_outranks_a_single_arm() {
    let (g, store, _p) = fixture("consensus");

    // Dense-only (an arm with no ranking) — the baseline the fusion must move away from.
    let dense_only = HybridConfig::new();
    let baseline = rows(&query_vec_hybrid(&g, HYBRID_Q, &store, &dense_only).unwrap());
    assert_eq!(
        baseline.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        vec!["a", "c", "d"],
        "the dense arm alone ranks by cosine to (1,0)"
    );
    assert!(baseline.iter().all(|r| r.3 == format!("vector={}", r.2)));

    // Now a text arm that ranks `c` first and `b` second. `c` is dense rank 2 AND text rank 1 —
    // consensus — so it must overtake `a`, which only the dense arm ranks.
    let cfg = HybridConfig::new().arm(
        "text",
        1.0,
        fixed_arm(vec![(id(&g, "c"), 9.0), (id(&g, "b"), 8.0)]),
    );
    let fused = rows(&query_vec_hybrid(&g, HYBRID_Q, &store, &cfg).unwrap());
    assert_eq!(fused[0].0, "c", "consensus beats a single arm's top hit");
    assert_eq!(fused[0].3, "vector=2;text=1");
    assert_ne!(
        fused.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        baseline.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        "the arm must actually change the answer — otherwise this test proves nothing"
    );

    // A muted dense arm (weight 0) is a pure sparse ranking: ONLY the text arm's items appear.
    let sparse_only = HybridConfig::new()
        .vector_weight(0.0)
        .arm("text", 1.0, fixed_arm(vec![(id(&g, "b"), 8.0)]));
    let only = rows(&query_vec_hybrid(&g, HYBRID_Q, &store, &sparse_only).unwrap());
    assert_eq!(only.len(), 1);
    assert_eq!((only[0].0.as_str(), only[0].3.as_str()), ("b", "text=1"));
}

/// A reranker driven by a fixed script, so both policy paths are exercised deterministically.
struct Scripted(Result<Vec<Rescored>, String>);

impl Reranker for Scripted {
    fn rerank(
        &self,
        _query: &ArmQuery<'_>,
        _candidates: &[FusedHit],
    ) -> Result<Vec<Rescored>, String> {
        self.0.clone()
    }
}

#[test]
fn the_second_stage_reorders_and_marks_its_provenance() {
    let (g, store, _p) = fixture("rerank");
    // Promote the fused runner-up to first place and drop everything else.
    let reranker = Scripted(Ok(vec![
        Rescored {
            index: 1,
            score: 42.0,
        },
        Rescored {
            index: 0,
            score: 1.0,
        },
    ]));
    let cfg = HybridConfig::new().reranker(&reranker, RerankPolicy::FailClosed);
    let out = rows(&query_vec_hybrid(&g, HYBRID_Q, &store, &cfg).unwrap());

    assert_eq!(out.len(), 2, "the reranker dropped the remaining candidates");
    assert_eq!(out[0].0, "c", "the fused runner-up was promoted to rank 1");
    assert_eq!(out[0].1, 42.0, "?score is the SECOND-STAGE score once reranked");
    assert_eq!(out[0].3, "vector=2;rerank=1");
    assert_eq!(out[1].3, "vector=1;rerank=2");
    // The rank provenance survives the round trip through SPARQL.
    assert_eq!(
        parse_provenance(&out[0].3).unwrap(),
        vec![("vector".to_string(), 2), ("rerank".to_string(), 1)]
    );
}

#[test]
fn a_failing_second_stage_is_fail_open_or_fail_closed() {
    let (g, store, _p) = fixture("policy");
    let broken = Scripted(Err("reranker service timed out".to_string()));

    // Fail open: the query still answers, with the first-stage order, and NO row claims a
    // second stage that did not run.
    let open = HybridConfig::new().reranker(&broken, RerankPolicy::FailOpen);
    let out = rows(&query_vec_hybrid(&g, HYBRID_Q, &store, &open).unwrap());
    assert_eq!(
        out.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        vec!["a", "c", "d"]
    );
    assert!(
        out.iter().all(|r| !r.3.contains("rerank")),
        "fail-open must not mark rows as reranked: {:?}",
        out
    );

    // Fail closed: a hard query error naming the underlying failure.
    let closed = HybridConfig::new().reranker(&broken, RerankPolicy::FailClosed);
    let err = query_vec_hybrid(&g, HYBRID_Q, &store, &closed).unwrap_err();
    assert!(err.contains("fail-closed"), "got: {}", err);
    assert!(err.contains("timed out"), "got: {}", err);

    // A malformed response — an index no candidate has — is treated the same way: an
    // out-of-process stage must never be able to inject a result no arm retrieved.
    let liar = Scripted(Ok(vec![Rescored {
        index: 99,
        score: 1.0,
    }]));
    let closed = HybridConfig::new().reranker(&liar, RerankPolicy::FailClosed);
    assert!(query_vec_hybrid(&g, HYBRID_Q, &store, &closed)
        .unwrap_err()
        .contains("out of range"));
}

#[test]
fn a_natural_language_query_needs_a_query_embedder() {
    let (g, store, _p) = fixture("text");
    const TEXT_Q: &str = r#"
PREFIX vec: <http://sparq.dev/vec#>
SELECT ?node ?score ?rank ?prov WHERE {
  ( ?node ?score ?rank ?prov ) vec:hybrid ( "the x axis"@en 2 )
}"#;

    // Without an embedder the dense arm has no query: a hard error, never a silent degradation.
    let err = query_vec_hybrid(&g, TEXT_Q, &store, &HybridConfig::new()).unwrap_err();
    assert!(err.contains("needs a query embedder"), "got: {}", err);

    // With one, the text form searches exactly like the equivalent vector literal, and the arms
    // see the text (here: an arm that only fires for an "@en" query).
    let cfg = HybridConfig::new()
        .query_embedder(Box::new(|text: &str, _lang: &str| {
            assert_eq!(text, "the x axis");
            Ok(vec![1.0, 0.0])
        }))
        .arm(
            "text",
            1.0,
            Box::new(move |q: &ArmQuery<'_>, _n| {
                assert_eq!(q.text.unwrap().1, "en");
                assert_eq!(q.vector, Some(&[1.0f32, 0.0][..]));
                Ok(Vec::new())
            }),
        );
    let out = rows(&query_vec_hybrid(&g, TEXT_Q, &store, &cfg).unwrap());
    assert_eq!(out.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(), vec!["a", "c"]);

    // An embedder whose dimension disagrees with the store is rejected, not silently scored.
    let wrong = HybridConfig::new()
        .query_embedder(Box::new(|_t: &str, _l: &str| Ok(vec![1.0, 0.0, 0.0])));
    let err = query_vec_hybrid(&g, TEXT_Q, &store, &wrong).unwrap_err();
    assert!(err.contains("3 dims but the store has 2"), "got: {}", err);
}

#[test]
fn hybrid_without_a_config_is_a_hard_error_and_other_predicates_are_untouched() {
    let (g, store, _p) = fixture("guard");

    // Plain `query_vec` carries no arms: answering with the dense arm alone would be a
    // well-formed ranking that is NOT the hybrid ranking the query asked for.
    let err = query_vec(&g, HYBRID_Q, &store).unwrap_err();
    assert!(err.contains("HybridConfig"), "got: {}", err);

    // `vec:nearest` / `vec:search` are unaffected by a hybrid config, and an unknown `vec:` term
    // still fails loudly.
    let cfg = HybridConfig::new().arm("text", 1.0, fixed_arm(vec![(id(&g, "b"), 1.0)]));
    let plain = query_vec_hybrid(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:nearest ( \"1,0\" 2 ) }",
        &store,
        &cfg,
    )
    .unwrap();
    assert_eq!(plain.rows.len(), 2, "vec:nearest keeps its own semantics");

    let err = query_vec_hybrid(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:teleport ( \"1,0\" 2 ) }",
        &store,
        &cfg,
    )
    .unwrap_err();
    assert!(err.contains("unknown magic predicate"), "got: {}", err);
}

#[test]
fn the_subject_list_is_prefix_optional_and_validated() {
    let (g, store, _p) = fixture("shape");
    let cfg = HybridConfig::new();

    // A bare `?node` subject is the minimal form.
    let bare = query_vec_hybrid(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node WHERE { ?node vec:hybrid ( \"1,0\" 2 ) }",
        &store,
        &cfg,
    )
    .unwrap();
    assert_eq!(bare.rows.len(), 2);

    // `( ?node ?score )` binds two columns.
    let two = query_vec_hybrid(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node ?score WHERE { ( ?node ?score ) vec:hybrid ( \"1,0\" 2 ) }",
        &store,
        &cfg,
    )
    .unwrap();
    assert_eq!(two.rows.len(), 2);
    assert!(two.rows.iter().all(|r| r[1].is_some()));

    // A non-variable position, a too-long list, and a bad query argument are hard errors.
    for (q, want) in [
        (
            "SELECT ?n WHERE { ( ?n <http://ex/a> ) vec:hybrid ( \"1,0\" 2 ) }",
            "must be a variable",
        ),
        (
            "SELECT ?n WHERE { ( ?n ?s ?r ?p ?x ) vec:hybrid ( \"1,0\" 2 ) }",
            "1- to 4-element list",
        ),
        (
            "SELECT ?n WHERE { ( ?n ?s ) vec:hybrid ( \"1,0\" ) }",
            "exactly two elements",
        ),
        (
            "SELECT ?n WHERE { ( ?n ?s ) vec:hybrid ( _:b 2 ) }",
            "must be a node IRI",
        ),
    ] {
        let sparql = format!("PREFIX vec: <http://sparq.dev/vec#>\n{}", q);
        let err = query_vec_hybrid(&g, &sparql, &store, &cfg).unwrap_err();
        assert!(err.contains(want), "expected {:?} in: {}", want, err);
    }
}

#[test]
fn the_budget_and_algebra_entry_points_agree_with_the_query_one() {
    let (g, store, _p) = fixture("entrypoints");
    let cfg = HybridConfig::new().arm("text", 1.0, fixed_arm(vec![(id(&g, "b"), 1.0)]));
    let want = rows(&query_vec_hybrid(&g, HYBRID_Q, &store, &cfg).unwrap());

    // Under an unlimited budget the answer is identical.
    let budgeted = query_vec_hybrid_with_budget(
        &g,
        HYBRID_Q,
        &store,
        &cfg,
        &sparq_vectors::QueryBudget::unlimited(),
    )
    .unwrap();
    assert_eq!(rows(&budgeted), want);

    // And so is the algebra-level rewrite evaluated through the engine directly: the rewrite is
    // the whole mechanism — no engine change is involved.
    let parsed = spargebra::SparqlParser::new().parse_query(HYBRID_Q).unwrap();
    let rewritten = rewrite_query_hybrid(parsed, &g, &store, &cfg).unwrap();
    let out = sparq_vectors::query_prepared(&g, &rewritten.into()).unwrap();
    assert_eq!(rows(&out), want);
}

#[test]
fn a_hybrid_pattern_joins_the_surrounding_bgp() {
    let (g, store, _p) = fixture("join");
    let cfg = HybridConfig::new();
    // The hit table joins to ordinary triples through the store's permutation indexes, exactly
    // as the vec:nearest/vec:search tables do.
    let r = query_vec_hybrid(
        &g,
        "PREFIX vec: <http://sparq.dev/vec#>
         SELECT ?node ?o WHERE {
           ( ?node ?score ?rank ) vec:hybrid ( \"1,0\" 4 ) .
           ?node <http://ex/p> ?o .
         }",
        &store,
        &cfg,
    )
    .unwrap();
    assert_eq!(r.rows.len(), 4, "every neighbour has one <http://ex/p> object");
    assert!(r.rows.iter().all(|row| row[1].is_some()));
}
