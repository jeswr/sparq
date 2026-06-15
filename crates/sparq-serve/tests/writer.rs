//! Sequenced-writer behavior (§6.5) against an instrumented mock
//! [`ApplyUpdates`]: group-commit batching, FIFO ordering, sync ack generation
//! numbers, failed-update isolation (including the partial-mutation recovery
//! path — the mock deliberately poisons the working copy before failing),
//! window/max-batch closing, graceful drain on drop, and the
//! readers-never-stalled stress.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sparq_serve::{
    ApplyUpdates, GenerationRing, PodId, WriteError, Writer, WriterConfig,
};

/// Snapshot = the log of applied updates, in application order.
type Log = Vec<String>;

/// Mock applier: `fork` clones the base log (instrumented), `apply` appends the
/// update — or, for `"fail:<msg>"` updates, POISONS the working copy first and
/// then fails, exercising the writer's discard-and-replay recovery exactly the
/// way `update_in_place`'s prefix-application hazard would.
struct MockApplier {
    forks: Arc<AtomicUsize>,
    /// Artificial per-fork cost, for the reader-stall stress (simulates the
    /// O(graph) fork of the production applier).
    fork_delay: Duration,
    /// [OPUS-4.8] (sq-vpx4) Injected durable-write failures: when > 0, `seal`
    /// fails (returns `Err`) and decrements — modelling a transient `ENOSPC`/I/O
    /// error on a `--persist` mirror that later clears. Lets a test drive the
    /// "writer survives a seal failure, the batch is refused, a later batch after
    /// recovery succeeds" path against the mock.
    seal_fails: Arc<AtomicUsize>,
    /// Counts the seals the writer actually attempted (success or failure), so a
    /// test can assert the writer kept running and re-attempted after a refusal.
    seals: Arc<AtomicUsize>,
}

impl MockApplier {
    fn new(forks: &Arc<AtomicUsize>) -> Self {
        MockApplier {
            forks: forks.clone(),
            fork_delay: Duration::ZERO,
            seal_fails: Arc::new(AtomicUsize::new(0)),
            seals: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// [OPUS-4.8] (sq-vpx4) Build a mock sharing the seal-failure and seal-count
    /// handles with the test, so the test can arm transient durable-write failures
    /// and observe that the writer survived them and re-attempted later.
    fn with_seal_control(
        forks: &Arc<AtomicUsize>,
        seal_fails: &Arc<AtomicUsize>,
        seals: &Arc<AtomicUsize>,
    ) -> Self {
        MockApplier {
            forks: forks.clone(),
            fork_delay: Duration::ZERO,
            seal_fails: seal_fails.clone(),
            seals: seals.clone(),
        }
    }
}

impl ApplyUpdates for MockApplier {
    type Snapshot = Log;
    type Working = Log;
    type Update = String;

    fn fork(&mut self, base: &Log) -> Result<Log, String> {
        self.forks.fetch_add(1, Ordering::SeqCst);
        if !self.fork_delay.is_zero() {
            std::thread::sleep(self.fork_delay);
        }
        Ok(base.clone())
    }

    fn apply(&mut self, working: &mut Log, update: &String) -> Result<(), String> {
        if let Some(msg) = update.strip_prefix("fail:") {
            working.push("POISON".into()); // partial mutation before the error
            return Err(msg.to_string());
        }
        if update == "panic" {
            panic!("applier panic requested by test");
        }
        working.push(update.clone());
        Ok(())
    }

    fn seal(&mut self, working: Log) -> Result<Log, String> {
        self.seals.fetch_add(1, Ordering::SeqCst);
        // [OPUS-4.8] (sq-vpx4) Transient durable-write failure injection: consume one
        // armed failure (if any) and refuse the batch. The writer must NOT publish and
        // must NOT panic — it fails the submitters with `Unavailable` and keeps running.
        if self.seal_fails.load(Ordering::SeqCst) > 0 {
            self.seal_fails.fetch_sub(1, Ordering::SeqCst);
            return Err("injected transient durable-write error (ENOSPC)".into());
        }
        Ok(working)
    }
}

fn pods(ids: &[&str]) -> Vec<PodId> {
    ids.iter().map(|s| PodId::from(*s)).collect()
}

/// N submissions inside one window → ONE generation whose epochs are the union
/// of the touched pods, applied in submission order (FIFO).
#[test]
fn one_window_one_generation_epochs_union() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_millis(250), max_batch: 64, ..WriterConfig::default() },
    );

    // 7 fire-and-forget + 1 sync, all queued well inside the 250 ms window the
    // first one opens. The sync ack proves the whole batch published together.
    for i in 0..7 {
        writer
            .submit_detached(format!("u{i}"), pods(&[&format!("pod:{i}"), "pod:shared"]))
            .unwrap();
    }
    let generation = writer.submit("u7".to_string(), pods(&["pod:7", "pod:shared"])).unwrap();

    assert_eq!(generation, 1, "all 8 updates collapse into the single generation 1");
    let current = ring.current();
    assert_eq!(current.number(), 1);
    let expected: Log = (0..8).map(|i| format!("u{i}")).collect();
    assert_eq!(*current.snapshot(), expected, "strict FIFO submission order");
    // Union of touched pods, each bumped ONCE per publish (dedup across updates).
    for i in 0..8 {
        assert_eq!(current.epochs().epoch(&PodId::from(format!("pod:{i}"))), 1);
    }
    assert_eq!(current.epochs().epoch(&PodId::from("pod:shared")), 1);
    assert_eq!(current.epochs().len(), 9);
    assert_eq!(forks.load(Ordering::SeqCst), 1, "one working copy for the whole batch");
}

/// Sequential sync submits land in consecutive generations and each ack carries
/// the generation that contains it.
#[test]
fn sync_submit_returns_its_generation_number() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_millis(1), max_batch: 256, ..WriterConfig::default() },
    );

    // Each submit blocks until publish, so the next opens a fresh window.
    assert_eq!(writer.submit("a".into(), pods(&["p"])).unwrap(), 1);
    assert_eq!(writer.submit("b".into(), pods(&["p"])).unwrap(), 2);
    assert_eq!(writer.submit("c".into(), pods(&["q"])).unwrap(), 3);

    let current = ring.current();
    assert_eq!(current.number(), 3);
    assert_eq!(*current.snapshot(), vec!["a".to_string(), "b".into(), "c".into()]);
    assert_eq!(current.epochs().epoch(&PodId::from("p")), 2);
    assert_eq!(current.epochs().epoch(&PodId::from("q")), 1);
}

/// A failing update is reported to its submitter and skipped; the rest of its
/// batch publishes (in order), and the failed update's pods are NOT bumped.
/// The mock poisons the working copy on failure, so this also proves the
/// re-fork-and-replay recovery discards the partial mutation.
#[test]
fn failed_update_is_isolated_and_recovered() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Arc::new(Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_millis(300), max_batch: 64, ..WriterConfig::default() },
    ));

    // Deterministic in-window order via staggered sync submitters: a (t≈0),
    // fail (t≈60 ms), c (t≈120 ms) — all inside the 300 ms window a opens.
    let spawn_submit = |update: &str, pod: &str, delay_ms: u64| {
        let writer = writer.clone();
        let update = update.to_string();
        let pod = PodId::from(pod);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(delay_ms));
            writer.submit(update, [pod])
        })
    };
    let a = spawn_submit("a", "pod:a", 0);
    let f = spawn_submit("fail:boom", "pod:bad", 60);
    let c = spawn_submit("c", "pod:c", 120);

    assert_eq!(a.join().unwrap().unwrap(), 1);
    assert_eq!(
        f.join().unwrap().unwrap_err(),
        WriteError::Rejected("boom".into()),
        "the failed update's submitter gets ITS error"
    );
    assert_eq!(c.join().unwrap().unwrap(), 1, "the batch survivor publishes in the same generation");

    let current = ring.current();
    assert_eq!(current.number(), 1, "one generation despite the mid-batch failure");
    assert_eq!(*current.snapshot(), vec!["a".to_string(), "c".into()], "no POISON, order kept");
    assert_eq!(current.epochs().epoch(&PodId::from("pod:a")), 1);
    assert_eq!(current.epochs().epoch(&PodId::from("pod:c")), 1);
    assert_eq!(current.epochs().epoch(&PodId::from("pod:bad")), 0, "skipped update bumps nothing");
    assert_eq!(forks.load(Ordering::SeqCst), 2, "initial fork + one recovery re-fork");
}

/// A batch in which EVERY update fails publishes nothing — the generation
/// number does not move and no epochs are bumped.
#[test]
fn all_failed_batch_publishes_no_generation() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_millis(1), max_batch: 8, ..WriterConfig::default() },
    );

    let err = writer.submit("fail:nope".into(), pods(&["pod:x"])).unwrap_err();
    assert_eq!(err, WriteError::Rejected("nope".into()));
    assert_eq!(ring.current().number(), 0, "nothing changed, nothing published");
    assert!(ring.current().epochs().is_empty());

    // The writer is still healthy afterwards.
    assert_eq!(writer.submit("ok".into(), pods(&["pod:x"])).unwrap(), 1);
}

/// The window closes early at max_batch: with max_batch = 2 and a 10 s window,
/// two concurrent sync submits ack long before the window would elapse.
#[test]
fn window_closes_early_on_max_batch() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Arc::new(Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_secs(10), max_batch: 2, ..WriterConfig::default() },
    ));

    let started = Instant::now();
    let t = {
        let writer = writer.clone();
        std::thread::spawn(move || writer.submit("a".into(), pods(&["p"])))
    };
    let g2 = writer.submit("b".into(), pods(&["p"])).unwrap();
    let g1 = t.join().unwrap().unwrap();
    let elapsed = started.elapsed();

    assert_eq!((g1, g2), (1, 1), "both updates in the one max_batch-closed generation");
    assert!(
        elapsed < Duration::from_secs(5),
        "max_batch must close the 10 s window early (took {elapsed:?})"
    );
}

/// Window timing is honored approximately: a lone update's sync ack waits out
/// the window (the writer holds the batch open for late arrivals) but not much
/// longer.
#[test]
fn window_timing_approximately_honored() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_millis(80), max_batch: 256, ..WriterConfig::default() },
    );

    let started = Instant::now();
    assert_eq!(writer.submit("a".into(), pods(&["p"])).unwrap(), 1);
    let elapsed = started.elapsed();

    assert!(
        elapsed >= Duration::from_millis(50),
        "the window stays open for late arrivals (acked after {elapsed:?}, window 80 ms)"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "the ack follows the window without unbounded slack (took {elapsed:?})"
    );
}

/// Dropping the writer drains: a pending batch is committed (early, without
/// waiting out a long window) before the thread exits.
#[test]
fn drop_drains_pending_batch() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_secs(10), max_batch: 256, ..WriterConfig::default() },
    );

    writer.submit_detached("a".into(), pods(&["p"])).unwrap();
    writer.submit_detached("b".into(), pods(&["q"])).unwrap();
    let started = Instant::now();
    drop(writer); // closes the queue; the open window ends early; batch commits
    assert!(started.elapsed() < Duration::from_secs(5), "drop does not wait out the window");

    let current = ring.current();
    assert_eq!(current.number(), 1);
    assert_eq!(*current.snapshot(), vec!["a".to_string(), "b".into()]);
    assert_eq!(current.epochs().epoch(&PodId::from("q")), 1);
}

/// A panicking applier kills the writer thread; pending and subsequent
/// submitters get `Shutdown`, not a hang.
#[test]
fn applier_panic_reports_shutdown() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::new(&forks),
        WriterConfig { window: Duration::from_millis(1), max_batch: 8, ..WriterConfig::default() },
    );

    assert_eq!(writer.submit("panic".into(), pods(&["p"])).unwrap_err(), WriteError::Shutdown);
    // The thread is gone (the receiver dropped with it): later submissions fail
    // fast instead of queueing forever.
    assert_eq!(writer.submit("a".into(), pods(&["p"])).unwrap_err(), WriteError::Shutdown);
    assert_eq!(ring.current().number(), 0);
}

/// READERS ARE NEVER STALLED BY COMMITS (§6.4/§6.5 invariant): while the writer
/// commits continuously — with an artificially slow O(fork) like the production
/// applier's O(graph) rebuild — reader threads hammer `ring.current()` and
/// their p99 latency stays microseconds: no read ever blocks on the write path.
#[test]
fn readers_never_stall_while_writer_commits() {
    let forks = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Arc::new(Writer::spawn(
        ring.clone(),
        MockApplier {
            forks: forks.clone(),
            fork_delay: Duration::from_millis(2),
            seal_fails: Arc::new(AtomicUsize::new(0)),
            seals: Arc::new(AtomicUsize::new(0)),
        },
        WriterConfig { window: Duration::from_millis(1), max_batch: 64, ..WriterConfig::default() },
    ));

    let stop = Arc::new(AtomicBool::new(false));

    // Feeder: keeps the writer committing back-to-back for the whole run.
    let feeder = {
        let writer = writer.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            let mut i = 0u64;
            while !stop.load(Ordering::Relaxed) {
                if writer.submit_detached(format!("u{i}"), [PodId::from("pod:w")]).is_err() {
                    break;
                }
                i += 1;
                std::thread::sleep(Duration::from_micros(200));
            }
        })
    };

    // Readers: measure every `current()` call for ~1.2 s.
    let readers: Vec<_> = (0..2)
        .map(|_| {
            let ring = ring.clone();
            let stop = stop.clone();
            std::thread::spawn(move || {
                let mut lat_ns: Vec<u64> = Vec::with_capacity(1 << 20);
                while !stop.load(Ordering::Relaxed) {
                    let t = Instant::now();
                    let g = ring.current();
                    lat_ns.push(t.elapsed().as_nanos() as u64);
                    std::hint::black_box(g.number());
                }
                lat_ns
            })
        })
        .collect();

    std::thread::sleep(Duration::from_millis(1200));
    stop.store(true, Ordering::Relaxed);
    feeder.join().unwrap();

    let committed = ring.current().number();
    assert!(committed > 50, "writer must have committed continuously (got {committed})");

    for r in readers {
        let mut lat = r.join().unwrap();
        assert!(lat.len() > 10_000, "reader must have run hot ({} reads)", lat.len());
        lat.sort_unstable();
        let p99 = lat[lat.len() * 99 / 100];
        // Lock-free load is ~tens of ns; 1 ms is two orders of magnitude of
        // scheduler-noise headroom. A reader blocked behind a 2 ms fork or the
        // writer's batch application would blow straight past this.
        assert!(p99 < 1_000_000, "reader p99 = {p99} ns — a read blocked on the write path");
    }
}

/// [OPUS-4.8] (sq-vpx4) A TRANSIENT durable-write error (one armed seal failure that
/// then clears) must REFUSE the in-flight write with `WriteError::Unavailable` (a
/// retryable 503-class outcome) WITHOUT killing the writer thread, WITHOUT publishing,
/// and WITHOUT acking 2xx — and a SUBSEQUENT write after recovery must succeed and
/// publish. This is the core graceful-degradation contract.
#[test]
fn transient_seal_failure_refuses_then_recovers() {
    let forks = Arc::new(AtomicUsize::new(0));
    let seal_fails = Arc::new(AtomicUsize::new(0));
    let seals = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::with_seal_control(&forks, &seal_fails, &seals),
        // One update per window so each is its own batch / generation attempt.
        WriterConfig { window: Duration::from_millis(5), max_batch: 1, ..WriterConfig::default() },
    );

    // Baseline: a normal write publishes generation 1.
    let g1 = writer.submit("ok-1".to_string(), pods(&["pod:a"])).unwrap();
    assert_eq!(g1, 1, "happy-path write must publish");
    assert_eq!(ring.current().number(), 1);

    // Arm ONE transient durable-write failure, then submit. The seal fails once.
    seal_fails.store(1, Ordering::SeqCst);
    let refused = writer.submit("will-be-refused".to_string(), pods(&["pod:a"]));
    assert!(
        matches!(refused, Err(WriteError::Unavailable(_))),
        "transient durable-write error must be a retryable Unavailable refusal, got {refused:?}",
    );

    // FAIL-CLOSED: nothing was published — the ring is byte-identical to before
    // (still generation 1, the refused update's effect absent).
    assert_eq!(
        ring.current().number(),
        1,
        "a refused (non-durable) write must NOT publish a new generation",
    );
    assert_eq!(
        ring.current().snapshot(),
        &vec!["ok-1".to_string()],
        "the refused write must leave the published snapshot byte-identical",
    );

    // The writer thread SURVIVED: a subsequent write after recovery succeeds + publishes.
    let g2 = writer.submit("ok-2".to_string(), pods(&["pod:a"])).unwrap();
    assert_eq!(g2, 2, "writer must keep running and publish after a transient failure");
    assert_eq!(ring.current().snapshot(), &vec!["ok-1".to_string(), "ok-2".to_string()]);

    // Sanity: the writer actually re-attempted the seal after the refusal.
    assert!(seals.load(Ordering::SeqCst) >= 3, "writer must have re-attempted seal after refusal");
}

/// [OPUS-4.8] (sq-vpx4) Per-submitter failure semantics when `seal` fails on a MIXED
/// batch: one update is rejected during `apply` (a deterministic 400-class error) and one
/// applies cleanly (pending durable commit). The seal then fails. The rejected submitter
/// must keep its ORIGINAL `Rejected` reason (NOT laundered into a retryable 503), while the
/// clean-but-undurable submitter gets `Unavailable`. Fail-closed holds: nothing publishes.
#[test]
fn seal_failure_keeps_apply_rejection_distinct_from_unavailable() {
    let forks = Arc::new(AtomicUsize::new(0));
    let seal_fails = Arc::new(AtomicUsize::new(0));
    let seals = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    // One wide window so both staggered submitters land in the SAME batch.
    let writer = Arc::new(Writer::spawn(
        ring.clone(),
        MockApplier::with_seal_control(&forks, &seal_fails, &seals),
        WriterConfig { window: Duration::from_millis(300), max_batch: 64, ..WriterConfig::default() },
    ));

    // Arm ONE seal failure: it fires when the writer seals this batch.
    seal_fails.store(1, Ordering::SeqCst);

    let spawn_submit = |update: &str, pod: &str, delay_ms: u64| {
        let writer = writer.clone();
        let update = update.to_string();
        let pod = PodId::from(pod);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(delay_ms));
            writer.submit(update, [pod])
        })
    };
    // A rejected-during-apply update + a valid one, both inside the open window.
    let rejected = spawn_submit("fail:bad-request", "pod:bad", 0);
    let valid = spawn_submit("ok", "pod:ok", 60);

    // The to-be-rejected update keeps ITS 400-class reason, not a 503 Unavailable.
    assert_eq!(
        rejected.join().unwrap().unwrap_err(),
        WriteError::Rejected("bad-request".into()),
        "an update rejected during apply must keep its original Rejected reason after a seal failure",
    );
    // The valid-but-undurable update is refused with the retryable Unavailable.
    assert!(
        matches!(valid.join().unwrap(), Err(WriteError::Unavailable(_))),
        "a clean update that was pending durable commit must get Unavailable when seal fails",
    );

    // FAIL-CLOSED: nothing published; the writer survived and recovers on the next write.
    assert_eq!(ring.current().number(), 0, "a seal-failed batch must publish no generation");
    assert!(ring.current().snapshot().is_empty());
    let g = writer.submit("ok-after".to_string(), pods(&["pod:ok"])).unwrap();
    assert_eq!(g, 1, "writer must keep running and publish once durability recovers");
}

/// [OPUS-4.8] (sq-vpx4) A PERSISTENT durable-write error → every write is refused with
/// `Unavailable` (no panic, no publish), the writer stays alive across all of them, and
/// reads keep being served from the last published snapshot the whole time (degraded
/// read-only mode). When durability finally recovers, writes resume.
#[test]
fn persistent_seal_failure_refuses_repeatedly_reads_survive() {
    let forks = Arc::new(AtomicUsize::new(0));
    let seal_fails = Arc::new(AtomicUsize::new(0));
    let seals = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::new(Log::new()));
    let writer = Writer::spawn(
        ring.clone(),
        MockApplier::with_seal_control(&forks, &seal_fails, &seals),
        WriterConfig { window: Duration::from_millis(5), max_batch: 1, ..WriterConfig::default() },
    );

    // Establish a published snapshot, then jam durability "persistently".
    writer.submit("ok-1".to_string(), pods(&["pod:a"])).unwrap();
    seal_fails.store(5, Ordering::SeqCst);

    for n in 0..5 {
        let r = writer.submit(format!("refused-{n}"), pods(&["pod:a"]));
        assert!(
            matches!(r, Err(WriteError::Unavailable(_))),
            "write #{n} under persistent durability failure must be Unavailable, got {r:?}",
        );
        // Reads still served from the last good snapshot throughout the outage.
        assert_eq!(ring.current().number(), 1, "reads must keep serving generation 1 during the outage");
        assert_eq!(ring.current().snapshot(), &vec!["ok-1".to_string()]);
    }

    // Durability recovered (all armed failures consumed): writes resume.
    let g = writer.submit("ok-2".to_string(), pods(&["pod:a"])).unwrap();
    assert_eq!(g, 2, "writes must resume once durability recovers");
}
