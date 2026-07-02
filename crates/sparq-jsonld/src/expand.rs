//! Expansion Algorithm + Value Expansion (JSON-LD 1.1 API §5.1, §5.3); `frameExpansion` mode.
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.25`.
//! Expansion is the pipeline's central hinge: it turns a compact JSON-LD document into the
//! canonical **expanded** form every other output projects from (compaction, flattening,
//! framing, and `toRdf`). Scoped/typed contexts, `@nest`, and `@index` containers — the
//! structures an RDF-first writer cannot recover — are handled here (design record §1.1).
