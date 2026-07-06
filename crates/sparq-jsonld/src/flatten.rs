//! Flattening Algorithm (JSON-LD 1.1 API §6.2).
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.26`.
//! Flattening collects every node object from the [`node_map`](crate::node_map) into a
//! single flat `@graph` array (optionally re-compacted against a supplied context).
