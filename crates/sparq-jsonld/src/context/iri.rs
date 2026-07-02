//! IRI Expansion, IRI Compaction, and Term Selection (JSON-LD 1.1 API §5.2, §7.1–7.2).
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.24`.
//! Expansion resolves a term/CURIE/relative-IRI against an
//! [`ActiveContext`](super::ActiveContext) (honouring `@vocab`, `@base`, and keyword
//! aliases); compaction selects the shortest term via the
//! [`InverseContext`](super::InverseContext).
