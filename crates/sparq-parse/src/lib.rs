//! sparq-parse: custom RDF parsing + compressed I/O for sparq.
//!
//! EMPTY SCAFFOLD (measure-first discipline). The crate exists so the workspace
//! wiring is settled, but contains no code yet: which parsers/codecs are worth
//! building — and their shape — is decided by the measured baseline in
//! `research/custom-parsers-baseline.md` (benchmarks under `bench/parse/`).
//!
//! Hard constraint: this crate must NOT enter the wasm dependency graph
//! (`sparq-wasm` must never depend on it, directly or transitively).
