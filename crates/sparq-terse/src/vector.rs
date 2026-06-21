//! The vector fallback for `V("phrase")` — **`vectors` feature only**
//! (`research/llm-ergonomic-sparql-surface.md` §3.3, §6.4, §6.5).
//!
//! 🤖 SPARQ agent. This path is consulted by [`crate::resolve::Resolver`] **only when the
//! deterministic lexical match returns nothing** (the §6.4 lexical-first rule). It pays
//! the embedding dependency surface and the silent-drift risk that lexical avoids, so it
//! is opt-in and additionally guarded:
//!
//! - **Injected embedder** — the live phrase-embedding model is the *caller's* concern;
//!   this crate never opens a socket. The caller supplies any `sparq_vectors::Embedder`
//!   (the same trait `vec:nearest` uses), embeds `"phrase"` to a query vector, and we
//!   search the store with it.
//! - **Staleness guard (mandatory, §6.5)** — every search first calls
//!   [`sparq_vectors::VectorStore::check_graph`]; a store built against a different graph
//!   generation is a hard `Err`, never silently-wrong neighbours
//!   (`sq-32i5` provides this).
//!
//! The result is still subject to the resolver's confidence gate — a vector "nearest"
//! that is not confident enough does NOT bind.

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_vectors::{ann::nearest_exact, Embedder, VectorStore};

use crate::resolve::Candidate;
use crate::transpile::TerseError;

/// A vector-search fallback bound to a vector store + an embedder. Construct it and attach
/// it to a [`crate::resolve::Resolver`] via `Resolver::with_vectors`.
pub struct VectorBackend<'g> {
    store: &'g VectorStore,
    embedder: &'g dyn Embedder,
}

impl<'g> VectorBackend<'g> {
    /// Builds a fallback over `store` using `embedder` to vectorise query phrases. The
    /// store must have been built against the same graph the [`crate::resolve::Resolver`]
    /// is querying — the staleness guard enforces this at search time.
    pub fn new(store: &'g VectorStore, embedder: &'g dyn Embedder) -> Self {
        VectorBackend { store, embedder }
    }

    /// Returns up to `k` nearest-concept candidates for `phrase`. Runs the staleness guard
    /// FIRST (a mismatch is a loud `Err`, never stale neighbours), then embeds the phrase
    /// and searches the store exactly. Only `NamedNode` (concept IRI) neighbours are
    /// returned; literal/blank neighbours are skipped. Scores are cosine similarities
    /// clamped to `[0, 1]` (cosine can be negative; a negative-similarity neighbour is not
    /// a meaningful concept match, so it is dropped).
    pub(crate) fn candidates(
        &self,
        graph: &Graph,
        phrase: &str,
        k: usize,
    ) -> Result<Vec<Candidate>, TerseError> {
        // §6.5: the mandatory staleness guard, BEFORE any neighbour is read.
        self.store
            .check_graph(graph)
            .map_err(|e| TerseError::Unresolved {
                phrase: phrase.to_string(),
                detail: format!("vector store is stale against the graph: {}", e),
            })?;

        let query = self
            .embedder
            .embed(&[phrase])
            .map_err(|e| TerseError::Unresolved {
                phrase: phrase.to_string(),
                detail: format!("embedding the phrase failed: {}", e),
            })?;
        let Some(qvec) = query.into_iter().next() else {
            return Ok(Vec::new());
        };

        let mut out = Vec::new();
        for (id, score) in nearest_exact(self.store, &qvec, k) {
            if score <= 0.0 {
                continue;
            }
            if let Term::NamedNode(iri) = graph.dict.term(id) {
                out.push(Candidate::vector(
                    NamedNode::new_unchecked(iri.as_str()),
                    score as f64,
                ));
            }
        }
        Ok(out)
    }
}
