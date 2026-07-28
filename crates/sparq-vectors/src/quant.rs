//! **Vector quantization** for large stores — SCALAR (`f32 → u8` per dimension) and PRODUCT
//! (`M` bytes per vector, ADC distance tables) encoders, plus the [`EncodedStore`] candidate
//! cache the DiskANN search loop ranks on.
//!
//! [OPUS-5] (issue #3699) **This module was EXTRACTED into the stand-alone
//! [`sparq_vamana::quant`] crate module** and is re-exported here unchanged, so
//! `sparq_vectors::quant::*` and `sparq_vectors::{ProductQuantizer, …}` keep working exactly as
//! before. The implementation was only ever coupled to this crate through `VectorStore` +
//! dictionary term ids, both of which are now the generic
//! [`VectorSource`](sparq_vamana::VectorSource) / [`VectorId`](sparq_vamana::VectorId) seam —
//! `VectorStore` implements `VectorSource` (see [`crate::store`]), so
//! [`ProductQuantizer::encode_store`] still takes a `&VectorStore` at every existing call site.
//!
//! There is exactly ONE copy of the encoders in the tree; nothing is duplicated or deprecated
//! here. See the extracted module's docs for the cosine convention, the k-means/ADC details and
//! the DiskANN "search on PQ, re-rank on disk" wiring.

pub use sparq_vamana::quant::{
    cosine_from_sq_dist, DistanceTable, EncodedStore, PqConfig, ProductQuantizer, ScalarQuantizer,
};
