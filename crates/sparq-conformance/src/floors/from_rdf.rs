//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `fromRdf` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// fromRdf (RDF → JSON-LD via the native writer, round-trip) pass floor.
/// RATCHET: may only RISE. Measured 51/53 at the pinned revision (the two
/// failures are lists whose cells are shared across graphs, which the
/// writer's `@list` collapsing renames).
pub const FLOOR: usize = 51;
