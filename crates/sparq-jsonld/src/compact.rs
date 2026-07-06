//! Compaction Algorithm (JSON-LD 1.1 API §7) — document-level, over expanded input.
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.27`.
//! Running compaction over the *expanded document* with a real
//! [`ActiveContext`](crate::context::ActiveContext) — rather than as a bespoke inverse of
//! the RDF writer — is what structurally removes the self-reparse-invisible data-loss class
//! of bug (design record §3.2). `sparq-engine`'s existing compaction entry points delegate
//! here once this lands.
