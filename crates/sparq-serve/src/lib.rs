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
//! Library-first rule (§6.1): sync, runtime-agnostic, no HTTP and no async-runtime
//! types anywhere in the API. The sequenced writer, group-commit window, scheduler,
//! result cache, and stream admission (`Shed(SnapshotPressure)` at the bound) are
//! LATER waves; this crate ships the foundation they compose over and exposes the
//! introspection (live-generation count, oldest retained number) they will need.
//!
//! Snapshot integration decision: §6.4 sketches `Generation { id, graph: Arc<Graph>,
//! epochs }`. The production snapshot mechanism is exactly an `Arc<sparq_core::Graph>`
//! clone (`AppState::snapshot()`, measured 27 ns in §4.3), which fits behind a plain
//! type parameter at zero cost — so [`Generation<S>`] is generic over the snapshot
//! handle instead of introducing a `StoreSnapshot` trait that would carry no methods.
//! The integration test `tests/real_store.rs` instantiates the ring with the real
//! `Graph`; unit tests use an instrumented mock to observe drops.

mod epoch;
mod ring;

pub use epoch::{Epoch, PodEpochs, PodId};
pub use ring::{Generation, GenerationRing, RingConfig, DEFAULT_RETAIN};
