//! Deserialize JSON-LD to RDF — expanded document → RDF (JSON-LD 1.1 API §8.2).
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.30`.
//! The native `expand ∘ to_rdf` path completes the ingest cases `oxjsonld` cannot express
//! (`expandContext`, `rdfDirection`, strict error codes); `oxjsonld` remains the default
//! streaming fast path, with a differential lane cross-checking the two (design record §3.3).
