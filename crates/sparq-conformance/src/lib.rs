//! Shared machinery for the W3C conformance harnesses: manifest walking, RDF
//! plumbing, result parsing/comparison, and per-test execution. Three binaries
//! sit on top:
//!
//! - `sparq-conformance` (src/main.rs) — the W3C SPARQL suites (query/update
//!   evaluation + syntax), gated in CI by a pass-count ratchet.
//! - `sparq-inference-conformance` (src/bin/inference.rs) — the reasoning
//!   suites (RDF Semantics rdf-mt, OWL 2 RL, N3, SPARQL entailment regimes)
//!   run against `sparq-reason`.
//! - `sparq-conformance-scoreboard` (src/bin/scoreboard.rs) — [OPUS-4.8]
//!   sq-ncvq.16: the CONSOLIDATED index of EVERY conformance ratchet across the
//!   workspace (the two binaries above PLUS the crate-local W3C SHACL and OGC
//!   GeoSPARQL ratchets), rendered from the central `scoreboard` registry.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod compare;
pub mod inference;
pub mod manifest;
pub mod rdf;
pub mod results;
pub mod run;
// [OPUS-4.8] sq-ncvq.16 — the CENTRAL conformance scoreboard registry: every
// ratchet (W3C SPARQL, inference, W3C SHACL, OGC GeoSPARQL) in one place. The
// `sparq-conformance-scoreboard` binary renders it; a guard test keeps the
// crate-local SHACL/geo floors in sync.
pub mod scoreboard;
// [OPUS-4.8] (B4) W3C rdf-turtle suite run THROUGH the sparq Turtle parser
// (`Graph::parse_to_triples`) — the rejection/acceptance oracle for the Turtle T1 spike,
// distinct from the oxttl-differential chunked-vs-serial test and the N3-parser TurtleTests.
pub mod turtle_suite;
