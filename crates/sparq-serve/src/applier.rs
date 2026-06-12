//! [`ApplyUpdates`] for the production store: `sparq_core::Graph` snapshots,
//! SPARQL Update strings as the update type.
//!
//! ## Snapshot-production decision (A2, recorded honestly)
//!
//! `Graph` is deliberately not `Clone` (a deep copy of the dictionary arena +
//! six permutation indexes is O(n)) and shares no internal structure between
//! instances, so producing the next published snapshot must mint a fresh
//! `Graph`. The candidate paths in today's codebase:
//!
//! 1. **Double-buffer reclaim** (today's `sparq-server` Writer): get the old
//!    published graph back via `Arc::try_unwrap` + poll, mutate in place
//!    (O(batch)). NOT AVAILABLE under the generation ring — the ring *retains*
//!    up to K old generations by design, so a published graph never drains back
//!    to refcount 1; and the reclaim wait is precisely the measured §4.3/§4.4
//!    pathology (5.4 s/32 s stall) the ring exists to remove.
//! 2. **Engine rebuild** (`sparq_engine::update(&Graph, sparql) -> Graph`,
//!    today's `apply_by_rebuild` slow path): decode the dictionary-encoded
//!    store to terms, apply, rebuild dict + indexes — O(graph), works from a
//!    shared `&Graph`. This is the only public owned-`Graph`-from-`&Graph`
//!    path in the codebase today.
//!
//! A2 therefore uses path 2 once per *batch*: [`fork`](ApplyUpdates::fork) is
//! `sparq_engine::update(base, "")` (the empty update is valid SPARQL — zero
//! operations, pure decode + rebuild), then each update in the batch is the
//! O(batch) `update_in_place` delta-overlay path (measured 17 µs/update, §4.3).
//!
//! **Known limitation (deliberate, deferred):** batch commit cost is dominated
//! by the O(graph) fork — measured at 56 ms/100 k triples and 1.15 s/1 M
//! (release, Apple Silicon, best of 3; run the `fork_cost_measurement` test
//! with `--ignored --nocapture` to reproduce). Today's double buffer avoids
//! this steady-state only via the reclaim that the ring makes impossible (and
//! that was the measured stall). The group-commit window therefore bounds
//! *queueing* delay, not end-to-end ack latency, once graphs are large; at the
//! Solid write rate (≤ hundreds/s, ~1 update/window measured envelope §6.5)
//! the throughput is still 10–100× over requirement, but a cheap structural
//! fork (persistent/COW indexes) is the recorded follow-up deliverable — A2
//! does not build a new storage layer.
//!
//! A free side effect: because `fork` rebuilds from decoded terms, every
//! published generation has **no pending delta-overlay** — readers always scan
//! the folded fast path, and the server's `compact_every` amortization is
//! unnecessary on this path ([`seal`](ApplyUpdates::seal) is the identity).
//!
//! Extension functions: this applier runs the plain engine update entry points.
//! A server that registers SPARQL extension functions (sparq-server's
//! `with_extensions`) should wrap its own `ApplyUpdates` around these calls —
//! the trait is the seam for exactly that.

use sparq_core::Graph;

use crate::writer::ApplyUpdates;

/// The production [`ApplyUpdates`]: `Graph` snapshots, SPARQL Update strings.
/// See the module docs for the fork-cost decision and limitation.
#[derive(Debug, Default)]
pub struct GraphApplier;

impl ApplyUpdates for GraphApplier {
    type Snapshot = Graph;
    type Working = Graph;
    type Update = String;

    /// O(graph): engine rebuild of `base` (decode + re-intern + re-index),
    /// named graphs preserved. The measured cost and why there is no cheaper
    /// public path today are in the module docs.
    fn fork(&mut self, base: &Graph) -> Result<Graph, String> {
        sparq_engine::update(base, "")
    }

    /// O(batch): the T17 delta-overlay path. On `Err` the graph may hold a
    /// partially applied prefix of the update's operations — exactly the hazard
    /// the writer's re-fork-and-replay recovery handles.
    fn apply(&mut self, working: &mut Graph, update: &String) -> Result<(), String> {
        sparq_engine::update_in_place(working, update)
    }

    /// Identity: the fork already produced a fully folded base (no overlay to
    /// compact for the batch's own updates beyond the in-place deltas, which
    /// the next batch's fork folds).
    fn seal(&mut self, working: Graph) -> Graph {
        working
    }
}
