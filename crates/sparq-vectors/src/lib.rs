#![doc = include_str!("../README.md")]

pub mod ann;
pub mod diskann;
pub mod embed;
pub mod fingerprint;
pub mod fuse;
pub mod import;
pub mod labels;
pub mod quant;
#[cfg(feature = "vec-predicate")]
pub mod rewrite;
pub mod store;
pub mod verbalize;

/// The `vec:` vocabulary — magic predicates recognised by [`rewrite`]
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

pub use ann::{
    cosine, nearest_exact, nearest_term_exact, nearest_term_exact_checked, HnswConfig, VectorIndex,
};
pub use diskann::{sibling_graph_path, DiskAnnIndex, VamanaConfig, SPQG_MAGIC, SPQG_VERSION};
pub use embed::{Embedder, HashEmbedder};
pub use fingerprint::{check_against, Artifact, CheckResult, Fingerprint, FINGERPRINT_LEN};
pub use fuse::{fuse_rrf, fuse_rrf_weighted, fuse_scores, hybrid_search, Retriever, RRF_K};
pub use import::{ImportBinding, ImportSpec, MAX_NPY_HEADER_LEN};
pub use labels::{embed_labels, embed_labels_with, LabelConfig};
pub use quant::{
    cosine_from_sq_dist, DistanceTable, EncodedStore, PqConfig, ProductQuantizer, ScalarQuantizer,
};
#[cfg(feature = "vec-predicate")]
pub use rewrite::{prepare_vec, query_vec, query_vec_with_budget, rewrite_query};
// Re-export the engine result/budget types the `vec:` entry points return/take, so
// callers (and tests) need not also declare a direct `sparq-engine` dependency.
// [OPUS-4.8] (sq-k6ex)
#[cfg(feature = "vec-predicate")]
pub use sparq_engine::{PreparedQuery, QueryBudget, QueryResult};
pub use store::{StreamingWriter, VectorStore, SPQV_MAGIC, SPQV_VERSION};
pub use verbalize::{
    description_predicates, embed_entities, label_predicates, verbalize, EntityTextConfig,
    ObjectKind, PropertyGroup,
};
