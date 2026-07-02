//! Framing Algorithm (JSON-LD 1.1 Framing §3) — matching, value patterns, `@embed`.
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.29`.
//! Framing matches a frame document against the expanded input (node patterns, value
//! patterns over `@value` alternatives, `@explicit`/`@default`/`@requireAll`, named graphs,
//! `@list`/`@set`, and blank-node `@embed`) — matching that happens on the expanded
//! document, not on RDF terms (design record §1.1).
//!
//! Spec: <https://www.w3.org/TR/json-ld11-framing/>.
