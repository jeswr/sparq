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
//! - [`verbalize`] / [`embed_entities`]: the **entity verbalization layer** — renders
//!   each entity's text passage from its literal properties (label + type +
//!   description + extra prefixed literals, multilingual-aware, char-budgeted) per
//!   [`EntityTextConfig`], the way production KG/vector systems do (see
//!   `research/genai-text-embedding-practices.md`), and embeds it.
//!   [`embed_labels`] (configurable via [`LabelConfig`]) is the label-only
//!   back-compat wrapper.
//! - [`fuse_rrf`] / [`fuse_scores`]: rank/score **fusion for hybrid retrieval** —
//!   combine the text-vector ranking with another ranked signal (e.g. `sparq-sim`'s
//!   structural similarity) without a dependency between the crates.
//!
//! Everything reads `sparq-core` through its public API only; the crate is opt-in and
//! nothing in the workspace depends on it — the default engine build does not even
//! compile it.

pub mod ann;
pub mod embed;
pub mod fuse;
pub mod labels;
pub mod store;
pub mod verbalize;

pub use ann::{cosine, nearest_exact, nearest_term_exact, HnswConfig, VectorIndex};
pub use embed::{Embedder, HashEmbedder};
pub use fuse::{fuse_rrf, fuse_scores, RRF_K};
pub use labels::{embed_labels, embed_labels_with, LabelConfig};
pub use store::{VectorStore, SPQV_MAGIC, SPQV_VERSION};
pub use verbalize::{
    description_predicates, embed_entities, label_predicates, verbalize, EntityTextConfig,
    ObjectKind, PropertyGroup,
};
