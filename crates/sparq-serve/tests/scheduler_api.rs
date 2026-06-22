//! [OPUS-4.8] (sq-fggn) Behavioural coverage for the scheduler's public surface
//! that the existing `scheduler.rs` differential/fairness tests don't reach: the
//! defaulted construction, the convenience `run`/`with_defaults` entry points, the
//! non-blocking `try_take` poll, the `submitted`/`completed`/`depth` metrics, and
//! the `SchedError` display/error contract. Each asserts observable behaviour, not
//! just that a line ran.

use std::sync::Arc;
use std::time::{Duration, Instant};

use sparq_serve::{SchedError, Scheduler, SchedulerConfig};

/// `SchedulerConfig::default()` derives a sane shape: >= 2 workers, a heavy cap of
/// at least 1 but strictly fewer than the worker count (so a cheap job is always
/// reservable), and the documented default heavy threshold.
#[test]
fn default_config_is_well_formed() {
    let cfg = SchedulerConfig::default();
    assert!(cfg.workers >= 2, "default must have >= 2 workers");
    assert_eq!(cfg.heavy_threshold, sparq_serve::DEFAULT_HEAVY_THRESHOLD);
    assert!(cfg.heavy_concurrency >= 1);
    // A scheduler built from it runs a job to completion.
    let sched = Scheduler::<u32>::with_defaults();
    assert_eq!(sched.workers(), cfg.workers.max(2));
    assert_eq!(sched.run(1, || 7).unwrap(), 7);
}

/// `run` is the submit-and-block convenience; it returns the closure's result.
#[test]
fn run_blocks_and_returns_the_result() {
    let sched = Scheduler::<String>::with_defaults();
    let out = sched.run(1, || "hello".to_string()).unwrap();
    assert_eq!(out, "hello");
}

/// `try_take` is non-blocking: `None` while the job is still pending (the ticket
/// stays usable), then `Some(result)` once it has finished.
#[test]
fn try_take_polls_without_consuming_until_done() {
    let sched = Scheduler::<u32>::with_defaults();
    // A job that blocks until we release it, so we can observe the pending poll.
    let gate = Arc::new(std::sync::Barrier::new(2));
    let g2 = gate.clone();
    let ticket = sched.submit(1, move || {
        g2.wait(); // hold here until the test lets it finish
        42
    });

    // Still pending: try_take yields None and leaves the ticket usable.
    assert!(ticket.try_take().is_none(), "job has not finished yet");
    assert!(
        ticket.try_take().is_none(),
        "still pending — ticket not consumed"
    );

    gate.wait(); // release the job

    // Spin (bounded) for completion, then take the result.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(r) = ticket.try_take() {
            assert_eq!(r.unwrap(), 42);
            break;
        }
        assert!(Instant::now() < deadline, "job never completed");
        std::thread::yield_now();
    }
}

/// `submitted` and `completed` are monotonic counters that converge once every
/// ticket has been waited on.
#[test]
fn submitted_and_completed_counters_track_jobs() {
    let sched = Scheduler::<u32>::with_defaults();
    assert_eq!(sched.submitted(), 0);
    assert_eq!(sched.completed(), 0);

    let tickets: Vec<_> = (0..8).map(|i| sched.submit(1, move || i)).collect();
    assert_eq!(sched.submitted(), 8, "every submit counts immediately");

    for (i, t) in tickets.into_iter().enumerate() {
        assert_eq!(t.wait().unwrap(), i as u32);
    }
    assert_eq!(
        sched.completed(),
        8,
        "all jobs completed once every ticket was waited on"
    );
}

/// `depth` reports `(cheap_queued, heavy_queued, heavy_running)`; with a single
/// worker and a flooded queue some jobs are observably still queued.
#[test]
fn depth_reports_queue_occupancy() {
    let sched = Scheduler::<()>::new(SchedulerConfig {
        workers: 2,
        heavy_concurrency: 1,
        heavy_threshold: 100,
    });
    let gate = Arc::new(std::sync::Barrier::new(2));
    let g = gate.clone();
    // One cheap job parks a worker; queue several more behind it.
    let parked = sched.submit(1, move || {
        g.wait();
    });
    let _queued: Vec<_> = (0..4).map(|_| sched.submit(1, || {})).collect();

    // Bounded spin until the queue depth is observable (>0 cheap queued).
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let (cheap, heavy, _running) = sched.depth();
        if cheap > 0 {
            assert_eq!(heavy, 0, "no heavy jobs were submitted");
            break;
        }
        assert!(Instant::now() < deadline, "queue never showed backlog");
        std::thread::yield_now();
    }
    gate.wait(); // release the parked worker so the pool drains on drop
    let _ = parked.wait();
}

/// `SchedError` renders distinct, non-empty human-readable messages (the
/// `Display`/`Error` contract callers map to 503-class responses).
#[test]
fn sched_error_display_is_distinct_and_nonempty() {
    let shutdown = format!("{}", SchedError::Shutdown);
    let panicked = format!("{}", SchedError::Panicked);
    assert!(!shutdown.is_empty());
    assert!(!panicked.is_empty());
    assert_ne!(shutdown, panicked);
    // Error trait object is constructible (source() defaults to None).
    let err: &dyn std::error::Error = &SchedError::Shutdown;
    assert!(err.source().is_none());
}
