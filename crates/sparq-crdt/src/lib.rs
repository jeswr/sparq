//! Executable formal model and convergence verification harness for the
//! **SPARQL-CRDT** proposal (`site/specs/sparql-crdt.typ`, bead `sq-tag1q.4`),
//! bead `sq-tag1q.7.4`.
//!
//! The proposal specifies a replicated RDF dataset as an add-wins observed-remove
//! quad set: each concrete quad addition carries a globally unique *dot*
//! `(replica, counter)`; a removal carries exactly the dots observed at the
//! update's origin; and replicas merge a per-quad dot store plus a causal
//! context with an associative, commutative, idempotent join. This crate
//! transcribes that normative algebra — the *exact* join equations of
//! `CRDT-JOIN-1`, the clock-plus-cloud causal context of `CRDT-CTX-1`, the
//! primitive mutators of `CRDT-MUT-1..3`, and the evaluate-at-origin update
//! compilation of `CRDT-UPD-*` — into small, direct Rust so that it can be
//! model-checked and property-tested, and so the future production
//! implementation (the rest of epic `sq-tag1q.7`) can be differentially tested
//! against it. Each item's documentation cites the proposal identifiers it
//! implements.
//!
//! The model abstracts RDF terms as small opaque identifiers: `CRDT-DATA-1`
//! requires only *term equality* for quad identity, so nothing in the merge
//! algebra depends on term structure, lexical forms, or skolemisation
//! mechanics. Those belong to the production crate and its conformance
//! fixtures, not to this algebraic model.
//!
//! # What verification this crate provides — and what it does not
//!
//! Three distinct kinds of evidence must not be conflated:
//!
//! 1. **Bounded, exhaustive model checking** (`checker`): every reachable
//!    configuration of a *bounded* multi-replica system — all interleavings of
//!    origin operations and delta deliveries — is explored; state invariants
//!    (`CRDT-STATE-1`) are checked at every configuration, and strong eventual
//!    consistency (`CRDT-SEC-2`: equal dot stores, equal causal-context
//!    denotations, equal visible quad sets) at every terminal configuration.
//!    The verdict holds **only within the explored bounds**.
//! 2. **Generated-schedule property tests** (`schedule`, `tests/`): randomized
//!    schedules that permute, duplicate, batch, snapshot, compact, and replay
//!    deltas across replicas. These are *sampled evidence* and regression
//!    protection, not exhaustive.
//! 3. **A formal convergence argument**: the proposal's `CRDT-SEC-2` proof
//!    obligation (the dotted-set join is a join-semilattice least upper bound,
//!    hence order/grouping/duplication independent). **This crate does not
//!    provide that proof.** The semilattice argument in the proposal is an
//!    informative sketch; no proof is claimed until a mechanized or
//!    peer-reviewed proof artifact exists and has been independently reviewed.
//!
//! In short: this crate can *falsify* the design within its bounds and supply
//! strong empirical evidence, but a green run is not a convergence theorem.

#![forbid(unsafe_code)]

pub mod checker;
pub mod context;
pub mod origin;
pub mod schedule;
pub mod state;

pub use checker::{check_convergence, CheckReport, LawReport, Scenario};
pub use context::{CausalContext, Counter, Dot, ReplicaId};
pub use origin::{Envelope, Op, Replica};
pub use schedule::{assert_converged, deliver_all, run_schedule, Step};
pub use state::{Delta, GraphKey, Quad, State};
