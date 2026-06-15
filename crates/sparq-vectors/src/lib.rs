#![doc = include_str!("../README.md")]

pub mod ann;
pub mod diskann;
pub mod embed;
pub mod fingerprint;
pub mod fuse;
pub mod labels;
pub mod quant;
pub mod store;
pub mod verbalize;

pub use ann::{
    cosine, nearest_exact, nearest_term_exact, nearest_term_exact_checked, HnswConfig, VectorIndex,
};
pub use diskann::{sibling_graph_path, DiskAnnIndex, VamanaConfig, SPQG_MAGIC, SPQG_VERSION};
pub use embed::{Embedder, HashEmbedder};
pub use fingerprint::{check_against, CheckResult, Fingerprint, FINGERPRINT_LEN};
pub use fuse::{fuse_rrf, fuse_rrf_weighted, fuse_scores, RRF_K};
pub use quant::{
    cosine_from_sq_dist, DistanceTable, EncodedStore, PqConfig, ProductQuantizer, ScalarQuantizer,
};
pub use labels::{embed_labels, embed_labels_with, LabelConfig};
pub use store::{StreamingWriter, VectorStore, SPQV_MAGIC, SPQV_VERSION};
pub use verbalize::{
    description_predicates, embed_entities, label_predicates, verbalize, EntityTextConfig,
    ObjectKind, PropertyGroup,
};
