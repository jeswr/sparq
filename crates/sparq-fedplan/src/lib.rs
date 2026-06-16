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
//! ## Streaming execution (sq-vf7q)
//!
//! 3. [`StreamJoin`] — an **ANAPSID-style non-blocking streaming join with bounded
//!    operator spill** ([`stream`]): the execution-side operator the planner's
//!    [`JoinAlgo::Streaming`] choice corresponds to. It consumes two incrementally-arriving
//!    tuple streams (federated sub-results returning at different rates), builds *and*
//!    probes both sides, and emits matches without first materialising either full input.
//!    Memory is bounded by [`StreamJoinOptions::mem_budget_tuples`]: over-budget join-key
//!    partitions spill to a backing store (a temp file by default — std only) and are
//!    reconciled on probe. The load-bearing invariant — proven by tests — is that the
//!    streamed + spilled result is **multiset-equal** to the equivalent blocking hash join
//!    ([`blocking_hash_join`]) for any interleaving and any budget.
//!
//! ## Live adaptive re-planning (sq-7s4z) — opt-in
//!
//! 4. `AdaptiveExecutor` — **mid-execution plan switching** behind the separate
//!    `adaptive-replan` cargo feature (OFF by default; implies `fedplan`). It captures the
//!    *observed* runtime statistics (`RuntimeStats`: real per-pattern cardinality +
//!    per-source latency) as execution proceeds, and at **stage boundaries** — between two
//!    leaf joins of the left-deep plan — re-invokes the planner on the **remaining** (not-
//!    yet-executed) patterns when an observation diverges from its estimate past a policy
//!    factor (`ReplanPolicy::divergence_factor`), adopting the new suffix order only when
//!    it wins by a hysteresis margin (`ReplanPolicy::improvement_margin`) so stable-but-
//!    noisy stats never thrash. **Soundness boundary**: re-planning reorders only the
//!    not-yet-started suffix (never a mid-operator swap); because BGP join is
//!    commutative/associative, any order over the same patterns yields the *same* result
//!    multiset — proven by `adaptive::tests::replan_result_equals_static` against an
//!    order-driven reference executor. Mid-operator adaptivity + live source failover are
//!    deferred (beaded under epic sq-3183).
//!
//! [OPUS-4.8] sq-a35t / sq-vf7q / sq-7s4z — flagged for Fable re-review.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-a35t: crate has zero `unsafe`.
#![cfg_attr(not(feature = "fedplan"), allow(dead_code, unused_imports))]

// [OPUS-4.8] sq-7s4z: live adaptive mid-execution re-planning. Gated behind the
// `adaptive-replan` feature (which implies `fedplan`); compiled out entirely otherwise.
#[cfg(feature = "adaptive-replan")]
mod adaptive;
#[cfg(feature = "fedplan")]
mod descriptor;
#[cfg(feature = "fedplan")]
mod pattern;
#[cfg(feature = "fedplan")]
mod plan;
#[cfg(feature = "fedplan")]
mod selection;
// [OPUS-4.8] sq-vf7q: the execution-side non-blocking streaming join + spill operator.
#[cfg(feature = "fedplan")]
mod stream;

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
// [OPUS-4.8] sq-vf7q.
#[cfg(feature = "fedplan")]
pub use stream::{
    blocking_hash_join, run_streaming, SpillStore, StreamJoin, StreamJoinOptions, Tuple,
};
// [OPUS-4.8] sq-7s4z: live adaptive mid-execution re-planning surface.
#[cfg(feature = "adaptive-replan")]
pub use adaptive::{exec_oracle, AdaptiveExecutor, ReplanOutcome, ReplanPolicy, RuntimeStats};
