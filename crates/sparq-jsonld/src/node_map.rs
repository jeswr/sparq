//! Node Map Generation (JSON-LD 1.1 API §6.1) + merge.
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.26`.
//! The node map indexes every node object by graph and `@id`, assigning blank-node
//! identifiers; it is the shared intermediate consumed by [`flatten`](crate::flatten) and
//! [`from_rdf`](crate::from_rdf).
