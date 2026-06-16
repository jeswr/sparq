//! sparq-fedplan: **cost-based federated source selection + bind-vs-hash join
//! planning** over already-fetched source *descriptors* — the first opt-in slice of
//! cost-based federation (bead sq-a35t, epic sq-3183).
//!
//! This crate plans a federated SPARQL Basic Graph Pattern (BGP) across a set of
//! remote SPARQL endpoints whose **statistics are already in hand** — each source is
//! described by a [`SourceDescriptor`] carrying the W3C VoID property/class partitions
//! (`void:propertyPartition` / `void:classPartition`) that a `sparq-server` already
//! serves at `/.well-known/void`, *plus* the mined **characteristic sets** the server
//! also serves under the `scs:` vocab (`http://sparq.dev/ns/cs#`). The planner is:
//!
//! * **Pure** — it consumes descriptors, never the network. No I/O, no live `Graph`.
//!   A caller fetches descriptors once (or builds them programmatically /
//!   [`SourceDescriptor::from_void_nt`] from the served N-Triples) and hands them in.
//! * **Deterministic** — same descriptors + same BGP ⇒ same plan, every time. Every
//!   internal ordering breaks ties on a stable key (source id, then pattern index).
//!
//! ## What it produces (this slice)
//!
//! 1. [`select_sources`] — **source selection**: for each triple pattern, which
//!    sources *can* contribute, with a per-(pattern, source) cardinality estimate.
//!    Two complementary techniques:
//!    * **HiBISCuS-style authority/prefix pruning** ([`selection`]) — a source is
//!      pruned for a pattern only when its capability set (the IRI authorities /
//!      predicates / classes it is known to hold) makes a contribution *impossible*.
//!      This is **recall-safe**: a source is dropped only on positive evidence of
//!      non-contribution; *any* uncertainty keeps it. See [`selection`] for the
//!      invariant and its proof obligations (tested in `selection::tests`).
//!    * **CostFed-style skew-aware cardinality** — the per-pattern, per-source
//!      cardinality is estimated from the served per-predicate stats (triples, average
//!      multiplicity) carried in the characteristic sets, *not* a uniform-distribution
//!      guess; predicate skew is preserved.
//! 2. [`plan_bgp`] — a **bind-vs-hash join planner** ([`plan`]): a greedy join-order
//!    heuristic over the selected sources that, for each join, chooses a **bind join**
//!    (probe the right side with the left side's bindings — cheap when the left is
//!    small and the bound variable is selective) or a **hash/symmetric join** (scan
//!    both sides once — cheap when both sides are large), using characteristic-set
//!    star-join cardinality to estimate intermediate-result sizes. The output is a
//!    [`JoinTree`] with per-node estimated cardinality and cost.
//!
//! ## Opt-in (hard constraint)
//!
//! The whole planner is behind the **`fedplan` cargo feature, OFF by default**, and the
//! crate is a standalone workspace member with `publish = false` — `sparq-core` /
//! `sparq-engine` never depend on it, so the default engine and the WASM artifact are
//! byte-identical with or without it. A build that does not enable `fedplan` compiles
//! an empty crate.
//!
//! ## Deferred (NOT in this slice — tracked as a roadmap bead)
//!
//! **ANAPSID-style non-blocking streaming joins with operator spill** and **live
//! adaptive re-planning** are explicitly out of scope here. This slice is a *static*
//! cost-based plan computed up front from descriptors; it does not adapt mid-execution,
//! and it does not model memory-bounded streaming operators. Those are the natural next
//! slice (see the roadmap bead filed from sq-a35t under epic sq-3183).
//!
//! [OPUS-4.8] sq-a35t — flagged for Fable re-review.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-a35t: crate has zero `unsafe`.
#![cfg_attr(not(feature = "fedplan"), allow(dead_code, unused_imports))]

#[cfg(feature = "fedplan")]
mod descriptor;
#[cfg(feature = "fedplan")]
mod pattern;
#[cfg(feature = "fedplan")]
mod plan;
#[cfg(feature = "fedplan")]
mod selection;

#[cfg(feature = "fedplan")]
pub use descriptor::{
    CharSet, ClassPartition, PredPartition, SourceDescriptor, SourceId, CS_NS, VOID_NS,
};
#[cfg(feature = "fedplan")]
pub use pattern::{Bgp, Term, TriplePattern, Var};
#[cfg(feature = "fedplan")]
pub use plan::{plan_bgp, JoinAlgo, JoinNode, JoinTree, PlanOptions};
#[cfg(feature = "fedplan")]
pub use selection::{select_sources, PatternSources, SourceCandidate};
