//! `JsonLdProcessor` facade — the public entry points (expand / compact / flatten / frame /
//! toRdf / fromRdf).
//!
//! **Scaffold (`sq-oy1f.23`).** Spec references only; the facade methods are wired as the
//! algorithm modules land (beads `sq-oy1f.25`+) and are honoured across every surface
//! (server profile conneg, CLI flags, the Python API) in bead `sq-oy1f.34`. Each method
//! threads a [`JsonLdOptions`](crate::options::JsonLdOptions) and a
//! [`DocumentLoader`](crate::loader::DocumentLoader), and returns
//! [`Result`]`<_, `[`JsonLdError`](crate::error::JsonLdError)`>`.
