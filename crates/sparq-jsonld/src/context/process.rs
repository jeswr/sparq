//! Context Processing Algorithm + Create Term Definition (JSON-LD 1.1 API §4).
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; implemented in bead `sq-oy1f.24`.
//! This is where a chain of local/remote `@context` definitions is folded into an
//! [`ActiveContext`](super::ActiveContext) — including keyword handling, `@protected` term
//! protection, `@propagate`, `@import`, and `@vocab`/`@base`/`@language`/`@direction`
//! defaults — with remote contexts fetched only through the
//! [`DocumentLoader`](crate::loader::DocumentLoader) (deny-by-default).
