#![doc = include_str!("../README.md")]
// [OPUS-4.8] MS-G2 (sq-8wbn): make `// SAFETY:` mandatory on every first-party unsafe block.
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod ann;
// [OPUS-4.8] (sq-ip3a) Pluggable ANN backend seam (exact default vs approximate) + the iterative
// over-fetch FILTERED path — `filtered-ann` only (the approximate impl is additionally `approx-ann`).
#[cfg(feature = "filtered-ann")]
pub mod backend;
// [OPUS-4.8] (sq-7hx6) Filtered-ANN pre-filter vs post-filter cost model — `filtered-ann` only.
#[cfg(feature = "filtered-ann")]
pub mod cost;
// [OPUS-4.8] (sq-pi44) Incremental add/remove/update of vectors against a finalized store via an
// in-RAM delta sidecar (+ `compact`) — `delta` feature only. Additive and dependency-free; the
// default build carries no delta code.
#[cfg(feature = "delta")]
pub mod delta;
pub mod diskann;
pub mod embed;
#[cfg(feature = "filtered-ann")]
pub mod filter;
pub mod fingerprint;
pub mod fuse;
pub mod import;
pub mod labels;
pub mod quant;
#[cfg(feature = "vec-predicate")]
pub mod rewrite;
pub mod store;
// [OPUS-4.8] sq-0wo9e.1 (epic sq-0wo9e): the P0 structure-aware-vectorisation preprocessing +
// sampling-logic layer — closure-before-vectorise + type-constrained negative sampling. `structure`
// feature only; the only feature pulling sparq-reason + sparq-introspect, so the default build
// carries zero structure-prep code.
#[cfg(feature = "structure")]
pub mod structure;
// [OPUS-4.8] sq-0wo9e.8 (epic sq-0wo9e): the MEASUREMENT FOUNDATION — a thin DistMult KGE trainer
// (`train`) over the P0 closure + type-constrained negatives, and the filtered link-prediction eval
// harness (`eval`) with the {closure}×{type-neg} ablation matrix, long-tail breakdown, and synthetic
// gUFO slice. `kge` feature only (implies `structure`); the default build carries zero trainer/eval
// code and no new dependency.
#[cfg(feature = "kge")]
pub mod eval;
#[cfg(feature = "kge")]
pub mod train;
pub mod verbalize;

/// The `vec:` vocabulary — magic predicates recognised by `rewrite`
/// (`http://sparq.dev/vec#`, the sparq extension namespace). [OPUS-4.8] (sq-k6ex)
pub mod vocab {
    /// `vec:` — the sparq vector-search namespace.
    pub const VEC_NS: &str = "http://sparq.dev/vec#";
    /// `?node vec:nearest ( <query> <k> )` — binds `?node` to the `<k>` nearest
    /// neighbours (best first) of `<query>`, which is either a node IRI (whose
    /// stored vector is the query, the seed excluded) or a comma-separated
    /// vector literal.
    pub const NEAREST: &str = "http://sparq.dev/vec#nearest";
    /// `( ?node ?score ) vec:search ( <query> <k> )` — like [`NEAREST`] but also
    /// binds `?score` to each neighbour's cosine similarity (`xsd:double`).
    pub const SEARCH: &str = "http://sparq.dev/vec#search";
}

pub use ann::{cosine, nearest_exact, nearest_term_exact, nearest_term_exact_checked};
// [OPUS-4.8] (sq-ip3a) The in-RAM HNSW index is the approximate backend — `approx-ann` only
// (the only feature pulling `instant-distance`). The default build re-exports just the exact
// searchers above.
#[cfg(feature = "approx-ann")]
pub use ann::{HnswConfig, VectorIndex};
pub use diskann::{sibling_graph_path, DiskAnnIndex, VamanaConfig, SPQG_MAGIC, SPQG_VERSION};
pub use embed::{Embedder, HashEmbedder};
// [OPUS-4.8] (sq-1wc1) Predicate-constrained (filtered) ANN — the `filtered-ann` feature only.
#[cfg(feature = "filtered-ann")]
pub use filter::{nearest_exact_filtered, FilterConfig, IdMask};
// [OPUS-4.8] (sq-7hx6) Pre-filter vs post-filter cost model — the `filtered-ann` feature only.
#[cfg(feature = "filtered-ann")]
pub use cost::{
    nearest_filtered_costed, overfetch_target, postfilter_exact, CostEstimate, CostModel, Strategy,
};
// [OPUS-4.8] (sq-ip3a) Pluggable ANN backend + iterative over-fetch filtered path — `filtered-ann`
// only. `ApproxBackend` is additionally `approx-ann` (re-exported below).
#[cfg(feature = "filtered-ann")]
pub use backend::{
    nearest_filtered_overfetch, nearest_filtered_overfetch_default, AnnBackend, ExactBackend,
    DEFAULT_MAX_ROUNDS,
};
#[cfg(all(feature = "filtered-ann", feature = "approx-ann"))]
pub use backend::ApproxBackend;
pub use fingerprint::{check_against, Artifact, CheckResult, Fingerprint, FINGERPRINT_LEN};
pub use fuse::{fuse_rrf, fuse_rrf_weighted, fuse_scores, hybrid_search, Retriever, RRF_K};
pub use import::{ImportBinding, ImportSpec, MAX_NPY_HEADER_LEN};
pub use labels::{embed_labels, embed_labels_with, LabelConfig};
pub use quant::{
    cosine_from_sq_dist, DistanceTable, EncodedStore, PqConfig, ProductQuantizer, ScalarQuantizer,
};
#[cfg(feature = "vec-predicate")]
pub use rewrite::{prepare_vec, query_vec, query_vec_with_budget, rewrite_query};
// [OPUS-4.8] (sq-z589, epic sq-3183) The APPROXIMATE `vec:` entry points — the unfiltered k-NN runs
// through an on-disk `DiskAnnIndex` (Vamana) instead of the exact full scan, for large `.spqv`
// stores. APPROXIMATE (recall < 1.0); gated on `approx-ann` (the only feature that compiles an
// approximate index) on top of `vec-predicate`.
#[cfg(all(feature = "vec-predicate", feature = "approx-ann"))]
pub use rewrite::{prepare_vec_approx, query_vec_approx, query_vec_approx_with_budget};
// Re-export the engine result/budget types the `vec:` entry points return/take (and `query_prepared`
// so callers can evaluate a `prepare_vec*` `PreparedQuery`), so callers (and tests) need not also
// declare a direct `sparq-engine` dependency. [OPUS-4.8] (sq-k6ex; query_prepared added sq-z589)
#[cfg(feature = "vec-predicate")]
pub use sparq_engine::{query_prepared, PreparedQuery, QueryBudget, QueryResult};
// [OPUS-4.8] (sq-pi44) The incremental delta sidecar value type — `delta` feature only. The
// add/remove/update/compact APIs live on `VectorStore` (also `delta`-gated).
// [OPUS-4.8] (sq-7e50) The persisted `.spqd` sidecar magic/version — the save/open APIs
// (`save_delta`/`open_with_delta`/`sibling_delta_path`) live on `VectorStore` (also `delta`-gated).
#[cfg(feature = "delta")]
pub use delta::{VectorDelta, SPQD_MAGIC, SPQD_VERSION};
pub use store::{StreamingWriter, VectorStore, SPQV_MAGIC, SPQV_VERSION};
// [OPUS-4.8] sq-0wo9e.1 (epic sq-0wo9e): the structure-aware-vectorisation P0 surface — the
// closure-before-vectorise step, the type-constraint extractor, and the type-constrained negative
// sampler with its on/off ablation switch. `structure` feature only.
#[cfg(feature = "structure")]
pub use structure::{
    close_for_vectorise, materialise_closure, ClosedGraph, Corrupt, NegativeSampler, SamplingMode,
    TypeConstraints,
};
// [OPUS-4.8] sq-0wo9e.8 (epic sq-0wo9e): the DistMult trainer + filtered link-prediction eval
// harness surface — `kge` feature only.
#[cfg(feature = "kge")]
pub use eval::{
    run_ablation, synthetic_gufo_ttl, synthetic_relational_ttl, AblationCell, EvalConfig, LongTail,
    Metrics, Splits, SCHEMA_PREDICATES,
};
#[cfg(feature = "kge")]
pub use train::{train, TrainConfig, TrainReport, TrainedModel};
pub use verbalize::{
    description_predicates, embed_entities, label_predicates, verbalize, EntityTextConfig,
    ObjectKind, PropertyGroup,
};
