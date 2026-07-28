#![doc = include_str!("../README.md")]
// [OPUS-5] Inherited from `sparq-vectors` (MS-G2 / sq-8wbn): `// SAFETY:` is mandatory on every
// first-party unsafe block. Every `unsafe` in this crate is a bounds-validated slice cast into a
// read backing whose alignment is guaranteed by construction — see `backing.rs`.
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod backing;
pub mod graph;
pub mod quant;
pub mod source;

pub use backing::{open_backing, Bytes};
pub use graph::{
    sibling_graph_path, StalenessToken, VamanaConfig, VamanaIndex, SPQG_HEADER_LEN,
    SPQG_HEADER_LEN_V1, SPQG_MAGIC, SPQG_VERSION, STALENESS_TOKEN_LEN,
};
pub use quant::{
    cosine_from_sq_dist, DistanceTable, EncodedStore, PqConfig, ProductQuantizer, ScalarQuantizer,
};
pub use source::{SliceVectors, VectorId, VectorSource};
