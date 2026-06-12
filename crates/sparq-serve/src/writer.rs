//! The single sequenced writer with a group-commit window
//! (research/concurrent-serving.md §6.5) — Wave A deliverable 2, composing over
//! the A1 [`GenerationRing`].
//!
//! One writer thread owns the sole right to publish generations. Callers submit
//! updates (an opaque `Update` value plus the set of touched [`PodId`]s — for
//! prod-solid-server shapes statically the resource graph + parent graph, §6.5);
//! the writer batches every update that arrives within a **group-commit window**
//! (`min(T_window, N_max)`, default 3 ms / 256 updates) into ONE new snapshot and
//! publishes it as ONE generation, bumping the epochs of the union of the touched
//! pods. Readers are never involved: they keep loading
//! [`GenerationRing::current`] lock-free; the only writer/reader interaction is
//! the ring's arc-swap at publish.
//!
//! ## Sequencing and ACID (§6.5)
//!
//! Updates inside a batch are applied in **submission order** (strict FIFO — the
//! deterministic, replayable single-order log that replication will lean on;
//! per-pod conflict reordering is explicitly a later deliverable). Cross-batch
//! reordering: none — batches are committed in sequence by the one thread.
//! A = per-update atomicity via the failed-update policy below; C = single-order
//! application; I = snapshot isolation for readers + the serial writer;
//! D = out of scope for the in-memory server v1 (as §6.5 records).
//!
//! ## Failed-update policy (documented contract)
//!
//! A failing update must not poison its batch: updates are applied in submission
//! order against a writer-private working copy; an update that fails to apply is
//! **reported to its submitter and skipped — every other update in the batch
//! proceeds**. Rationale: batch membership is an artifact of arrival timing, not
//! of intent — two independent clients whose requests happen to share a 3 ms
//! window must not see each other's failures (atomicity stays per update, exactly
//! as today's `AppState::apply_update` provides). The epochs bumped at publish
//! are the union of the touched pods of the *successful* updates only — a skipped
//! update changed nothing, so bumping its pods would churn Wave B cache keys
//! spuriously. If every update in a batch fails, **no generation is published**
//! (an identical snapshot under a new number would invalidate nothing and help
//! nobody).
//!
//! Recovery detail: a failed [`ApplyUpdates::apply`] may have left the working
//! copy partially mutated (the engine's `update_in_place` documents exactly this
//! prefix-application hazard, and the existing server Writer discards its buffer
//! for the same reason). The writer therefore conservatively discards the working
//! copy, re-forks the base snapshot, and replays the batch's previously-successful
//! updates in order before continuing — one extra O(fork) per failed update,
//! acceptable because failures are the rare path (parse errors and constraint
//! violations, not steady state).
//!
//! ## Snapshot production (the §6.4 "working copy", decided honestly)
//!
//! §6.4 imagines a persistent writer-private working copy folded periodically.
//! That scheme worked for the double buffer because the writer could *reclaim*
//! the previously published graph (`Arc::try_unwrap` + poll) — which is both the
//! measured pathology A1 removed (5.4 s/32 s pinned-snapshot stall, §4.3/§4.4)
//! and *impossible* under the ring by design: the ring itself retains up to K
//! old generations, so a published graph never drains back to the writer.
//! `sparq_core::Graph` is deliberately not `Clone` and shares no internal
//! structure, so every publish must mint a fresh `Graph`. Today's cheapest public
//! path is one O(graph) fork per batch (engine rebuild) + O(batch) in-place
//! overlay updates — see [`GraphApplier`](crate::GraphApplier) for the concrete
//! numbers and the recorded limitation (a cheap structural fork is a later
//! deliverable; A2 does not build a new storage layer).

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::epoch::PodId;
use crate::ring::GenerationRing;

/// Default group-commit window. §6.5 prescribes 2–5 ms; the middle of the band
/// adds ≤3 ms p50 write latency (far inside the 45 s contract) while absorbing
/// ~hundreds of updates per window at the measured 17 µs/update in-place cost.
pub const DEFAULT_WINDOW: Duration = Duration::from_millis(3);

/// Default max batch size: the window closes early once this many updates have
/// been collected (§6.5's `min(T_window, N_max)`, e.g. "2–5 ms or 256 updates").
pub const DEFAULT_MAX_BATCH: usize = 256;

/// Configuration for the sequenced [`Writer`].
#[derive(Clone, Debug)]
pub struct WriterConfig {
    /// Group-commit window: the first update to arrive opens the window; every
    /// update arriving before it elapses joins the same batch → one generation.
    pub window: Duration,
    /// The window also closes as soon as this many updates are batched
    /// (back-pressure bound; values below 1 are treated as 1).
    pub max_batch: usize,
}

impl Default for WriterConfig {
    fn default() -> Self {
        WriterConfig { window: DEFAULT_WINDOW, max_batch: DEFAULT_MAX_BATCH }
    }
}

/// How the writer turns the current snapshot plus a batch of updates into the
/// next snapshot. Implemented by [`GraphApplier`](crate::GraphApplier) for the
/// production store; tests use instrumented mocks.
///
/// Contract the writer relies on:
/// - [`fork`](Self::fork) produces a private working copy of `base`; the writer
///   mutates only that copy, so readers of `base` are never affected.
/// - [`apply`](Self::apply) on `Err` may leave the working copy in ANY state
///   (partial prefix application is allowed — the writer recovers by re-forking
///   and replaying, see the module docs). It must, however, be deterministic
///   enough that an update that succeeded once succeeds again on replay against
///   the same prefix; a replay failure is handled (the update is demoted to
///   failed) but is reported as an anomaly in its error message.
/// - [`seal`](Self::seal) finalizes the working copy into the published snapshot
///   type (fold/compact hooks live here).
pub trait ApplyUpdates: Send + 'static {
    /// The published snapshot type — the `S` of the [`GenerationRing`].
    type Snapshot: Send + Sync + 'static;
    /// The writer-private mutable working copy.
    type Working;
    /// One submitted update (e.g. a SPARQL Update string).
    type Update: Send + 'static;

    /// Produces a writer-private working copy of `base`.
    fn fork(&mut self, base: &Self::Snapshot) -> Result<Self::Working, String>;

    /// Applies one update to the working copy. On `Err` the copy may be
    /// partially mutated; the writer discards and re-forks (module docs).
    fn apply(&mut self, working: &mut Self::Working, update: &Self::Update) -> Result<(), String>;

    /// Finalizes the working copy into a publishable snapshot.
    fn seal(&mut self, working: Self::Working) -> Self::Snapshot;
}

/// Why a submitted update did not land in a published generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// The update failed to apply and was skipped; the rest of its batch
    /// proceeded (per-update atomicity — module docs). Carries the application
    /// error (e.g. the SPARQL parse error).
    Rejected(String),
    /// The writer has shut down (or its thread panicked); the update was not
    /// applied.
    Shutdown,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::Rejected(e) => write!(f, "update rejected: {e}"),
            WriteError::Shutdown => f.write_str("writer has shut down"),
        }
    }
}

impl std::error::Error for WriteError {}

/// One queued submission.
struct Msg<U> {
    update: U,
    touched: Vec<PodId>,
    /// `Some` for sync submissions: receives `Ok(generation number)` once the
    /// generation containing the update is published, or the `WriteError`.
    /// `None` for fire-and-forget.
    ack: Option<SyncSender<Result<u64, WriteError>>>,
}

/// The single sequenced writer (§6.5): owns the one thread allowed to call
/// [`GenerationRing::publish`], batching submissions in a group-commit window.
///
/// `Writer` is `Send + Sync`; share it as `Arc<Writer<_>>` (or by reference) —
/// both submission methods take `&self`. Dropping the writer closes the queue,
/// commits any in-flight batch (graceful drain — pending sync submitters still
/// get their generation number), and joins the thread.
pub struct Writer<U: Send + 'static> {
    /// `Some` until drop; dropping the sender is what tells the thread to drain.
    tx: Option<mpsc::Sender<Msg<U>>>,
    thread: Option<JoinHandle<()>>,
}

impl<U: Send + 'static> Writer<U> {
    /// Spawns the writer thread over `ring`, owning `applier` as its snapshot
    /// production strategy.
    pub fn spawn<A>(ring: Arc<GenerationRing<A::Snapshot>>, applier: A, config: WriterConfig) -> Self
    where
        A: ApplyUpdates<Update = U>,
    {
        let (tx, rx) = mpsc::channel::<Msg<U>>();
        let thread = thread::Builder::new()
            .name("sparq-serve-writer".into())
            .spawn(move || run(ring, applier, config, rx))
            .expect("spawn writer thread");
        Writer { tx: Some(tx), thread: Some(thread) }
    }

    /// Submits an update and **blocks until the generation containing it is
    /// published**, returning that generation's number (the group-commit ack:
    /// at most one window + one batch application of latency). `touched` is the
    /// update's conflict tag — the pods (named graphs) it writes, whose epochs
    /// the publish bumps.
    ///
    /// `Err(Rejected)` means *this* update failed and was skipped while its
    /// batch proceeded; `Err(Shutdown)` means it was never applied.
    pub fn submit(
        &self,
        update: U,
        touched: impl IntoIterator<Item = PodId>,
    ) -> Result<u64, WriteError> {
        let (ack_tx, ack_rx) = mpsc::sync_channel(1);
        self.send(Msg { update, touched: touched.into_iter().collect(), ack: Some(ack_tx) })?;
        // A dropped ack sender means the writer thread died mid-batch.
        ack_rx.recv().map_err(|_| WriteError::Shutdown)?
    }

    /// Fire-and-forget submission: queues the update and returns immediately.
    /// `Err(Shutdown)` if the writer is gone; otherwise the outcome (including a
    /// possible [`WriteError::Rejected`]) is not reported — callers that need
    /// the result use [`submit`](Self::submit).
    pub fn submit_detached(
        &self,
        update: U,
        touched: impl IntoIterator<Item = PodId>,
    ) -> Result<(), WriteError> {
        self.send(Msg { update, touched: touched.into_iter().collect(), ack: None })
    }

    fn send(&self, msg: Msg<U>) -> Result<(), WriteError> {
        self.tx
            .as_ref()
            .expect("sender present until drop")
            .send(msg)
            .map_err(|_| WriteError::Shutdown)
    }
}

impl<U: Send + 'static> Drop for Writer<U> {
    fn drop(&mut self) {
        // Closing the channel ends the thread's loop after it drains + commits
        // whatever batch is in flight (the disconnect also closes an open window
        // early — no point waiting for arrivals that can no longer happen).
        drop(self.tx.take());
        if let Some(t) = self.thread.take() {
            // A panicked writer thread already failed its submitters via their
            // dropped ack channels; never panic in drop ourselves.
            let _ = t.join();
        }
    }
}

/// The writer thread: collect a batch per group-commit window, commit, repeat.
fn run<A: ApplyUpdates>(
    ring: Arc<GenerationRing<A::Snapshot>>,
    mut applier: A,
    config: WriterConfig,
    rx: Receiver<Msg<A::Update>>,
) {
    let max_batch = config.max_batch.max(1);
    loop {
        // Block until the first update arrives — it opens the window.
        let first = match rx.recv() {
            Ok(m) => m,
            Err(_) => return, // all senders gone, nothing queued: done
        };
        let deadline = Instant::now() + config.window;
        let mut batch = vec![first];
        let mut disconnected = false;
        while batch.len() < max_batch {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            match rx.recv_timeout(deadline - now) {
                Ok(m) => batch.push(m),
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    disconnected = true; // drain: commit what we have, then exit
                    break;
                }
            }
        }
        commit(&ring, &mut applier, batch);
        if disconnected {
            return;
        }
    }
}

/// Applies one batch to a fresh working copy and publishes ONE generation
/// (module docs: FIFO order, failed updates skipped + re-fork recovery, epochs =
/// union of successful updates' pods, no publish if nothing succeeded).
fn commit<A: ApplyUpdates>(
    ring: &GenerationRing<A::Snapshot>,
    applier: &mut A,
    batch: Vec<Msg<A::Update>>,
) {
    let base = ring.current();
    let mut errs: Vec<Option<String>> = (0..batch.len()).map(|_| None).collect();

    let mut working = match applier.fork(base.snapshot()) {
        Ok(w) => w,
        Err(e) => return fail_all(batch, &format!("snapshot fork failed: {e}")),
    };
    // Indexes (submission order) of updates successfully applied to `working`.
    let mut applied: Vec<usize> = Vec::with_capacity(batch.len());

    for i in 0..batch.len() {
        match applier.apply(&mut working, &batch[i].update) {
            Ok(()) => applied.push(i),
            Err(e) => {
                errs[i] = Some(e);
                // `working` may be partially mutated: rebuild and replay.
                match replay(applier, base.snapshot(), &batch, &mut applied, &mut errs) {
                    Ok(w) => working = w,
                    Err(e) => return fail_all(batch, &format!("snapshot fork failed: {e}")),
                }
            }
        }
    }

    if applied.is_empty() {
        // Every update failed: nothing changed, publish nothing (module docs).
        for (i, msg) in batch.into_iter().enumerate() {
            if let Some(ack) = msg.ack {
                let e = errs[i].take().expect("unapplied update carries its error");
                let _ = ack.send(Err(WriteError::Rejected(e)));
            }
        }
        return;
    }

    let snapshot = applier.seal(working);
    let touched = applied.iter().flat_map(|&i| batch[i].touched.iter().cloned());
    let number = ring.publish(snapshot, touched).number();
    for (i, msg) in batch.into_iter().enumerate() {
        if let Some(ack) = msg.ack {
            let _ = ack.send(match errs[i].take() {
                Some(e) => Err(WriteError::Rejected(e)),
                None => Ok(number),
            });
        }
    }
}

/// Failure recovery: re-fork the base and replay the already-successful updates
/// in submission order. An update that fails on replay (anomalous: it succeeded
/// once against the same prefix) is demoted to failed with an explanatory error
/// and the replay restarts — `applied` strictly shrinks, so this terminates.
fn replay<A: ApplyUpdates>(
    applier: &mut A,
    base: &A::Snapshot,
    batch: &[Msg<A::Update>],
    applied: &mut Vec<usize>,
    errs: &mut [Option<String>],
) -> Result<A::Working, String> {
    'rebuild: loop {
        let mut working = applier.fork(base)?;
        for k in 0..applied.len() {
            let i = applied[k];
            if let Err(e) = applier.apply(&mut working, &batch[i].update) {
                errs[i] = Some(format!(
                    "update applied cleanly but failed on deterministic replay \
                     (non-deterministic ApplyUpdates::apply?): {e}"
                ));
                applied.remove(k);
                continue 'rebuild;
            }
        }
        return Ok(working);
    }
}

/// Fails every pending submitter in the batch with `Rejected(reason)` — used
/// when no working copy can be produced at all (fork failure).
fn fail_all<U>(batch: Vec<Msg<U>>, reason: &str) {
    for msg in batch {
        if let Some(ack) = msg.ack {
            let _ = ack.send(Err(WriteError::Rejected(reason.to_string())));
        }
    }
}
