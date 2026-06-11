//! Shared machinery for the W3C conformance harnesses: manifest walking, RDF
//! plumbing, result parsing/comparison, and per-test execution. Two binaries
//! sit on top:
//!
//! - `sparq-conformance` (src/main.rs) — the W3C SPARQL suites (query/update
//!   evaluation + syntax), gated in CI by a pass-count ratchet.
//! - `sparq-inference-conformance` (src/bin/inference.rs) — the reasoning
//!   suites (RDF Semantics rdf-mt, OWL 2 RL, N3, SPARQL entailment regimes)
//!   run against `sparq-reason`.

pub mod compare;
pub mod inference;
pub mod manifest;
pub mod rdf;
pub mod results;
pub mod run;
