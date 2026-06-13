//! sparq-zk-compose circuit core (plan S4.E module (ii), v1).
//!
//! Generic implementations of the per-property proof circuits; the tiny bin
//! packages (`scan_k{K}_n{N}_r{R}`, `filter_int_d{D}`, `filter_f64`) are the
//! compiled circuit-family members -- each `main` is one monomorphisation of a
//! function in this library (Q5: a circuit FAMILY over a (k, n) lattice, not
//! one dynamic circuit).
//!
//! Bit-compatibility contract: every hash here is
//! `poseidon::poseidon2::Poseidon2::hash` (noir-lang/poseidon v0.3.0), the
//! exact function `crates/sparq-zk/src/poseidon2.rs` ports and cross-tests
//! against `nargo` (`tests/poseidon2_noir_cross.rs`). `tests.nr` re-asserts
//! the same fixture vectors from the Noir side.

pub mod hashes;
pub mod scan;
pub mod filter_int;
pub mod filter_float;

mod tests;
