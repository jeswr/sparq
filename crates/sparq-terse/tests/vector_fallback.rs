//! [OPUS-4.8] sq-leg8n — integration tests for the `vectors`-feature vector fallback in
//! `V("phrase")` concept resolution (`research/llm-ergonomic-sparql-surface.md` §6.4/§6.5).
//!
//! 🤖 SPARQ agent. These exercise the REAL path: a real `VectorStore` keyed to the graph's
//! dictionary ids (built with `with_fingerprint` so the staleness guard has something to
//! check), a real `Embedder`, and the real `nearest_exact` search — no mock that bypasses
//! the logic. The load-bearing invariants asserted here:
//!   - **lexical-first** — the vector path is consulted ONLY when lexical returns nothing;
//!   - **staleness guard (mandatory)** — a store built against a DIFFERENT graph generation
//!     is a hard error, never silently-wrong neighbours.
#![cfg(feature = "vectors")]

use oxrdf::Term;
use sparq_core::Graph;
use sparq_terse::transpile::TerseError;
use sparq_terse::vector::VectorBackend;
use sparq_terse::{ResolveConfig, Resolver};
use sparq_vectors::{Embedder, VectorStore};

/// A deterministic, fixed-map test embedder. It maps a few KNOWN strings to chosen 3-d
/// vectors and everything else to a far-away constant. This lets the test drive a phrase
/// that is a genuine VECTOR neighbour of a concept WITHOUT any lexical overlap (so the
/// lexical-first path provably misses and the fallback provably fires). The embedder is
/// the injected dependency; the store, the `nearest_exact` search and the staleness guard
/// are all the REAL code under test.
struct FixedEmbedder;

impl Embedder for FixedEmbedder {
    fn dim(&self) -> usize {
        3
    }
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        Ok(texts
            .iter()
            .map(|t| match *t {
                // The concept labels (what the store is built from).
                "graph database systems" => vec![1.0, 0.0, 0.0],
                "join ordering heuristics" => vec![0.0, 1.0, 0.0],
                "approximate vector search" => vec![0.0, 0.0, 1.0],
                // A QUERY phrase with NO lexical overlap with any label, deliberately near
                // the graph-databases vector so the vector fallback resolves to it.
                "rdf triple store" => vec![0.96, 0.05, 0.05],
                _ => vec![0.1, 0.1, 0.9],
            })
            .collect())
    }
}

/// A small concept graph; each concept carries a `skos:prefLabel`.
fn concept_graph() -> Graph {
    let ttl = r#"
        @prefix skos: <http://www.w3.org/2004/02/skos/core#> .
        @prefix topic: <http://example.org/topic/> .
        topic:graph-databases a skos:Concept ; skos:prefLabel "graph database systems" .
        topic:join-ordering a skos:Concept ; skos:prefLabel "join ordering heuristics" .
        topic:vector-search a skos:Concept ; skos:prefLabel "approximate vector search" .
    "#;
    Graph::load_str(ttl, "turtle").expect("graph parses")
}

/// Builds an in-memory vector store keyed to `graph`'s term ids, where each concept's
/// vector is the embedding of its prefLabel — the realistic label-embedding setup. Bound
/// to `graph` via `with_fingerprint` so `check_graph` has a fingerprint to verify.
fn label_store(graph: &Graph, embedder: &FixedEmbedder) -> VectorStore {
    let mut store = VectorStore::create("", embedder.dim())
        .expect("create store")
        .with_fingerprint(graph);
    let labels: &[(&str, &str)] = &[
        (
            "http://example.org/topic/graph-databases",
            "graph database systems",
        ),
        (
            "http://example.org/topic/join-ordering",
            "join ordering heuristics",
        ),
        (
            "http://example.org/topic/vector-search",
            "approximate vector search",
        ),
    ];
    let texts: Vec<&str> = labels.iter().map(|&(_, l)| l).collect();
    let vecs = embedder.embed(&texts).expect("embed labels");
    for ((iri, _), v) in labels.iter().zip(&vecs) {
        let term = Term::NamedNode(oxrdf::NamedNode::new(*iri).unwrap());
        let id = graph.id_of(&term).expect("concept is in the dict");
        store.put(id, v).expect("put vector");
    }
    store
}

#[test]
fn vector_fallback_resolves_when_lexical_misses() {
    let g = concept_graph();
    let embedder = FixedEmbedder;
    let store = label_store(&g, &embedder);

    // A phrase with NO lexical overlap with any label ("rdf triple store" shares no word or
    // substring with "graph database systems"), but the injected embedder places it near
    // that concept's vector. So the lexical path provably MISSES and the FALLBACK fires.
    let backend = VectorBackend::new(&store, &embedder);
    let cfg = ResolveConfig {
        confidence_floor: 0.4,
        ambiguity_margin: 0.05,
        candidate_k: 8,
    };
    let resolver = Resolver::with_config(&g, cfg).with_vectors(backend);

    let q = r#"SELECT ?f WHERE { ?f <http://example.org/about> V("rdf triple store") }"#;
    let exp = resolver
        .transpile(q)
        .expect("vector fallback resolves the no-overlap phrase");
    assert_eq!(exp.resolutions.len(), 1);
    let res = &exp.resolutions[0];
    assert_eq!(
        res.method,
        sparq_terse::ResolutionMethod::Vector,
        "expected the VECTOR path (lexical has no overlap with the phrase)"
    );
    assert_eq!(res.iri, "http://example.org/topic/graph-databases");
    assert!(exp
        .canonical_sparql
        .contains("<http://example.org/topic/graph-databases>"));
}

#[test]
fn lexical_first_means_an_exact_label_never_uses_vectors() {
    let g = concept_graph();
    let embedder = FixedEmbedder;
    let store = label_store(&g, &embedder);
    let backend = VectorBackend::new(&store, &embedder);
    let resolver = Resolver::new(&g).with_vectors(backend);

    // An EXACT label is resolved by the deterministic lexical path; the vector backend is
    // attached but must NOT be the method that fires (the §6.4 lexical-first rule).
    let q = r#"SELECT * WHERE { ?s ?p V("graph database systems") }"#;
    let exp = resolver
        .transpile(q)
        .expect("exact label resolves lexically");
    assert_eq!(
        exp.resolutions[0].method,
        sparq_terse::ResolutionMethod::Lexical
    );
}

#[test]
fn staleness_guard_is_a_loud_error_never_stale_neighbours() {
    // Build the store against ONE graph generation, then query the backend against a
    // DIFFERENT graph (different term set → different fingerprint). The mandatory staleness
    // guard must turn this into a loud Unresolved error, never silently-wrong neighbours.
    let built_against = concept_graph();
    let embedder = FixedEmbedder;
    let store = label_store(&built_against, &embedder);

    // A different graph: extra triples shift the dictionary fingerprint.
    let other = Graph::load_str(
        r#"@prefix skos: <http://www.w3.org/2004/02/skos/core#> .
           @prefix topic: <http://example.org/topic/> .
           topic:graph-databases a skos:Concept ; skos:prefLabel "graph database systems" .
           topic:extra a skos:Concept ; skos:prefLabel "an extra concept that shifts the dict" ."#,
        "turtle",
    )
    .expect("other graph parses");

    let backend = VectorBackend::new(&store, &embedder);
    // Force the vector path (a phrase with no lexical overlap), against the WRONG graph.
    let cfg = ResolveConfig {
        confidence_floor: 0.0,
        ambiguity_margin: 0.0,
        candidate_k: 8,
    };
    let resolver = Resolver::with_config(&other, cfg).with_vectors(backend);
    let q = r#"SELECT * WHERE { ?s ?p V("rdf triple store") }"#;
    let err = resolver.transpile(q).unwrap_err();
    match err {
        TerseError::Unresolved { detail, .. } => {
            assert!(
                detail.contains("stale"),
                "expected a staleness refusal, got: {detail}"
            );
        }
        other => panic!("expected a loud Unresolved staleness error, got {other:?}"),
    }
}
