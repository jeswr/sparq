//! sparq-serve: the concurrent-serving core (research/concurrent-serving.md §6).
//!
//! Wave A deliverable 1 (§8): the **generation ring** — an arc-swapped chain of
//! immutable store snapshots with a bounded retention ring and per-pod epoch
//! vectors. This replaces the double-buffered `AppState` snapshot scheme whose two
//! measured pathologies (§4.3/§4.4: the 5.4 s/32 s pinned-snapshot writer stall and
//! reclaim-poll degradation under reader churn) motivated the redesign.
//!
//! Invariants (load-bearing, tested):
//! - **Readers never block the writer; the writer never waits for readers** and
//!   never reclaims in place. Old generations are freed by ordinary `Arc` drop
//!   when their last holder lets go — there is no reclaim poll, no grace timer.
//! - **The ring only forgets its own references** beyond the retention bound K;
//!   it can never invalidate a generation a reader still pins.
//! - `current()` is lock-free (`ArcSwap::load`, ~10–20 ns); only `publish()` and
//!   the introspection accessors take the (writer-side) mutex.
//!
//! Wave A deliverable 2 (§6.5): the **single sequenced writer** with a
//! group-commit window — [`Writer`] owns the sole right to publish generations;
//! updates arriving within a window (default 3 ms / 256 updates) are applied to
//! one working copy and published as ONE generation, epochs bumped for the union
//! of touched pods. Sync ([`Writer::submit`], returns the generation number) and
//! fire-and-forget ([`Writer::submit_detached`]) submission; failed updates are
//! reported to their submitter and skipped, never poisoning the batch (policy
//! documented in the `writer` module). [`GraphApplier`] is the production
//! [`ApplyUpdates`] strategy — the STRUCTURAL FORK (`Graph::fork`, Arc-shared
//! immutable storage + per-generation delta) makes its fork O(pending delta),
//! with a threshold compaction policy in `seal` bounding the delta (its module
//! docs record the design and the measured before/after numbers).
//!
//! Opt-in time travel: a retained generation IS a queryable snapshot, so
//! [`RingConfig::time_travel`] ([`TimeTravelConfig`]) extends retention beyond the
//! concurrency bound K (count and/or age bounded; K stays the floor) and
//! [`GenerationRing::at`] / [`GenerationRing::as_of`] resolve a generation number /
//! "as of T" timestamp to a pinned historical generation ([`Generation::published_at`]
//! is stamped by the ring's injectable clock). Memory cost is honest and documented on
//! [`TimeTravelConfig`]: each retained generation is a FULL `Graph` until the
//! structural-fork follow-up lands; the API is shaped so delta-chain retention can
//! replace full graphs later without an API change.
//!
//! Wave B deliverable (§6.2): the read-side **query [`Scheduler`]** — goal #4
//! requirements 3 + 4 (cheap-query prioritisation + no head-of-line blocking). A
//! cost-aware bounded thread pool with two lanes: cheap jobs are prioritised and
//! always keep a reserved worker (so an expensive-query flood can never block a
//! cheap query — the structural no-HoL guarantee), while heavy jobs run under a
//! bounded concurrency cap with SRPT-approx + unbounded-aging ordering
//! (shortest-estimated-first, starvation-free). Readers still pin a generation on
//! entry (A1) inside the submitted closure, so snapshot consistency is preserved
//! and scheduling never changes results. See the `scheduler` module docs for the
//! Umbra-derived design (litreview-C Topic A) and the empirical-honesty test plan.
//!
//! Library-first rule (§6.1): sync, runtime-agnostic, no HTTP and no async-runtime
//! types anywhere in the API (the writer and scheduler use `std::thread` +
//! channels/condvars internally). The result cache and stream admission
//! (`Shed(SnapshotPressure)` at the bound) are LATER waves; this crate ships the
//! foundation they compose over and exposes the introspection (live-generation
//! count, oldest retained number, scheduler queue depths) they will need.
//!
//! Snapshot integration decision: §6.4 sketches `Generation { id, graph: Arc<Graph>,
//! epochs }`. The production snapshot mechanism is exactly an `Arc<sparq_core::Graph>`
//! clone (`AppState::snapshot()`, measured 27 ns in §4.3), which fits behind a plain
//! type parameter at zero cost — so [`Generation<S>`] is generic over the snapshot
//! handle instead of introducing a `StoreSnapshot` trait that would carry no methods.
//! The integration test `tests/real_store.rs` instantiates the ring with the real
//! `Graph`; unit tests use an instrumented mock to observe drops.

mod applier;
mod epoch;
mod footprint;
mod ring;
mod scheduler;
mod writer;

pub use applier::{GraphApplier, DEFAULT_COMPACT_THRESHOLD};
pub use epoch::{Epoch, PodEpochs, PodId};
pub use footprint::{Footprint, TargetGraph};
pub use ring::{Generation, GenerationRing, RingConfig, TimeTravelConfig, DEFAULT_RETAIN};
pub use scheduler::{
    Cost, Lane, SchedError, Scheduler, SchedulerConfig, Ticket, DEFAULT_HEAVY_THRESHOLD, P0,
};
pub use writer::{
    ApplyUpdates, CommitGranularity, WriteError, Writer, WriterConfig, DEFAULT_MAX_BATCH,
    DEFAULT_WINDOW,
};
