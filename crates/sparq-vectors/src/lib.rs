//! sparq-vectors: **opt-in embedding storage + nearest-neighbour search** for the sparq
//! RDF engine (GenAI phase 4 — see `research/genai-design.md`).
//!
//! - [`VectorStore`]: one f32 embedding per dictionary term id in a flat, memory-mapped
//!   `.spqv` file, designed for **sparse coverage** (only entities get embeddings, not
//!   every literal) — see [`store`] for the format.
//! - [`nearest_exact`] / [`VectorIndex`]: exact brute-force cosine top-`k` (the
//!   baseline and the recall ground truth) and an HNSW approximate index
//!   (`instant-distance`, pure Rust), both returning `(Id, cosine)` pairs;
//!   [`VectorIndex::nearest_term`] / [`nearest_term_exact`] query by [`oxrdf::Term`],
//!   resolving through a [`Graph`](sparq_core::Graph)'s dictionary.
//! - [`Embedder`]: provider-agnostic embedding trait. Embeddings are produced
//!   **outside** the engine (design decision: out-of-process); [`HashEmbedder`] is the
//!   deterministic, offline, **test-only** implementation, and the non-default
//!   `provider` feature carries the API shape for a live OpenAI-compatible endpoint
//!   (caller-supplied HTTP transport — this crate never opens a socket).
//! - [`embed_labels`]: embeds each entity's `rdfs:label`/`skos:prefLabel`/name-ish
//!   literal (configurable via [`LabelConfig`]) so a graph gains vector lookup.
//!
//! Everything reads `sparq-core` through its public API only; the crate is opt-in and
//! nothing in the workspace depends on it — the default engine build does not even
//! compile it.

pub mod ann;
pub mod embed;
pub mod labels;
pub mod store;

pub use ann::{cosine, nearest_exact, nearest_term_exact, HnswConfig, VectorIndex};
pub use embed::{Embedder, HashEmbedder};
pub use labels::{embed_labels, embed_labels_with, LabelConfig};
pub use store::{VectorStore, SPQV_MAGIC, SPQV_VERSION};
