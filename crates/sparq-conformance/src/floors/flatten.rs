//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `flatten` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// [OPUS-4.8] sq-oy1f — `flatten` (RDF → flattened JSON-LD via the
/// ALREADY-SHIPPING writer `graph_to_jsonld(JsonLdForm::Flattened)`, the
/// `serialize-rdf` feature) pass floor over the `flatten` category of
/// `w3c/json-ld-api`. RATCHET: may only RISE. This is the MEASURED pass count
/// at the pinned suite revision — the number of `jld:FlattenTest` cases for
/// which flattening the input's RDF and re-parsing the produced flattened
/// document reconstructs the SAME RDF dataset as re-parsing the suite's
/// NORMATIVE expected flattened document (`reparse(flatten(D)) ≡ reparse(expected)`).
/// Same oracle, SKIP buckets, and caveat as the `expand` floor (flattening is the
/// node-merged normal form; the oracle anchors on the expected document, not the
/// input). MEASURED 50/58 at the pinned revision: flatten 50 pass / 0 fail / 8
/// skip — every flatten case the writer drives round-trips to the normative
/// expected document; the 8 SKIP are the documented buckets (1
/// NegativeEvaluationTest, JSON-LD-1.0-only positives, and empty-RDF inputs).
pub const FLOOR: usize = 50;
