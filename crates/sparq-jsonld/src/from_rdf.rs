//! Serialize RDF as JSON-LD — RDF → expanded document (JSON-LD 1.1 API §8.1).
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.28`.
//! Reconstructs an expanded JSON-LD document from an RDF dataset: `rdfDirection` (both
//! modes), `@json` literals, and `rdf:List` reconstruction. This subsumes `sparq-engine`'s
//! existing `fromRdf` writer core, which delegates here once this lands.
