//! [SONNET-4.6] (sq-l6zks, gh-3216) End-to-end invariants of the `ChangeSink` seam against a
//! REAL durable change log (`ChangeLog` over real `Graph` generations): the relay delivers
//! every recorded commit **in order**, its **durable cursor** resumes exactly after a restart,
//! a failing sink is **at-least-once and never gapping** (retryable re-delivers; permanent
//! fail-closes), a **re-base gap marker is delivered, never swallowed**, and the relay's ack
//! watermark is the retention safety bound (a trimmed-away offset **fails closed**).

#![cfg(feature = "change-sink")]

use std::fs;
use std::path::PathBuf;

use sparq_core::Graph;
use sparq_serve::change_sink::{ChangeRelay, ChangeSink, RelayConfig, RelayError, SinkError};
use sparq_serve::{
    ChangeLog, ChangeLogConfig, ChangeRecord, GenerationRing, JsonLinesSink, PodId,
    RetentionPolicy,
};

fn scratch(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "sparq-cdc-sink-it-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id()
    ))
}

fn graph_with(nq: &str) -> Graph {
    Graph::load_dataset(nq, "nquads").expect("seed parses")
}

/// Builds a ring plus a log holding `n` recorded commits, each adding one triple.
/// Returns the log directory and the ring's final generation number.
fn seeded_log(dir: &PathBuf, n: u64, config: ChangeLogConfig) -> (GenerationRing<Graph>, u64) {
    let pod = PodId::new("http://ex/g");
    let ring: GenerationRing<Graph> = GenerationRing::new(graph_with(""));
    let mut log = ChangeLog::open_with_config(dir, config).expect("open log");
    let mut prev = ring.current();
    let mut body = String::new();
    for i in 0..n {
        // [SONNET-4.6] positional format args (CodeQL rust/unused-variable false-positive).
        body.push_str(&format!(
            "<http://ex/s{}> <http://ex/p> \"v{}\" .\n",
            i, i
        ));
        let next = ring.publish(graph_with(&body), [pod.clone()]);
        log.record_commit(&prev, &next).expect("record commit");
        prev = next;
    }
    (ring, prev.number())
}

/// A test sink that records what it saw and can be scripted to fail on a chosen seq.
struct ScriptedSink {
    seen: Vec<u64>,
    flushes: usize,
    fail_on: Option<(u64, bool)>, // (seq, retryable)
}

impl ScriptedSink {
    fn new() -> Self {
        ScriptedSink {
            seen: Vec::new(),
            flushes: 0,
            fail_on: None,
        }
    }

    fn failing(seq: u64, retryable: bool) -> Self {
        ScriptedSink {
            seen: Vec::new(),
            flushes: 0,
            fail_on: Some((seq, retryable)),
        }
    }
}

impl ChangeSink for ScriptedSink {
    fn deliver(&mut self, record: &ChangeRecord) -> Result<(), SinkError> {
        if let Some((seq, retryable)) = self.fail_on {
            if record.seq == seq {
                return Err(if retryable {
                    SinkError::Retryable(format!("broker unreachable at {}", seq))
                } else {
                    SinkError::Permanent(format!("payload rejected at {}", seq))
                });
            }
        }
        self.seen.push(record.seq);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        self.flushes += 1;
        Ok(())
    }
}

/// THE load-bearing invariant: every recorded commit reaches the sink exactly once, in
/// ascending seq order, and the durable cursor tracks what the sink took. (Mutation witness:
/// make `pump` advance the cursor without delivering and `seen` comes back empty.)
#[test]
fn relay_delivers_every_record_in_order_once() {
    let dir = scratch("inorder");
    let _ = fs::remove_dir_all(&dir);
    let (_ring, _gen) = seeded_log(&dir, 4, ChangeLogConfig::default());

    let log = ChangeLog::open(&dir).expect("reader handle");
    let mut relay =
        ChangeRelay::open(dir.join("relay.cursor"), ScriptedSink::new()).expect("open relay");
    assert_eq!(relay.acked_through_seq(), None, "fresh cursor is empty");

    let report = relay.pump(&log).expect("pump");
    assert_eq!(report.delivered, 4);
    assert_eq!(report.acked_through_seq, Some(3));
    assert!(!report.has_more);
    assert_eq!(relay.sink().seen, vec![0, 1, 2, 3], "ascending, gapless");

    // A second pump with nothing new is a no-op — never a re-delivery.
    let again = relay.pump(&log).expect("idle pump");
    assert_eq!(again.delivered, 0);
    assert_eq!(relay.sink().seen, vec![0, 1, 2, 3]);

    let _ = fs::remove_dir_all(&dir);
}

/// The durable cursor IS the restart contract: drop the relay (process restart), re-open it
/// against the same cursor file, and it resumes at the first UNDELIVERED record — it neither
/// replays what the sink already took nor skips what it did not.
#[test]
fn durable_cursor_resumes_exactly_after_a_restart() {
    let dir = scratch("resume");
    let _ = fs::remove_dir_all(&dir);
    let (ring, _gen) = seeded_log(&dir, 2, ChangeLogConfig::default());
    let cursor = dir.join("relay.cursor");

    {
        let log = ChangeLog::open(&dir).expect("reader");
        let mut relay = ChangeRelay::open(&cursor, ScriptedSink::new()).expect("open relay");
        assert_eq!(relay.pump(&log).expect("pump").delivered, 2);
        assert_eq!(relay.sink().seen, vec![0, 1]);
    } // relay dropped == process restart

    // Two more commits land while the relay is down.
    {
        let mut log = ChangeLog::open(&dir).expect("append handle");
        let pod = PodId::new("http://ex/g");
        let mut prev = ring.current();
        for i in 2..4u64 {
            let next = ring.publish(
                graph_with(&format!(
                    "<http://ex/s0> <http://ex/p> \"v0\" .\n\
                     <http://ex/s1> <http://ex/p> \"v1\" .\n\
                     <http://ex/late{}> <http://ex/p> \"x\" .\n",
                    i
                )),
                [pod.clone()],
            );
            log.record_commit(&prev, &next).expect("record");
            prev = next;
        }
    }

    let log = ChangeLog::open(&dir).expect("reader");
    let mut relay = ChangeRelay::open(&cursor, ScriptedSink::new()).expect("reopen relay");
    assert_eq!(
        relay.acked_through_seq(),
        Some(1),
        "cursor recovered from disk"
    );
    let report = relay.pump(&log).expect("pump after restart");
    assert_eq!(report.delivered, 2);
    assert_eq!(
        relay.sink().seen,
        vec![2, 3],
        "resumes at the first undelivered record — no replay, no skip"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// A RETRYABLE sink failure is at-least-once and never gapping: the pass stops AT the failing
/// record (later records are not reordered ahead of it), progress up to it is durable, the
/// relay stays armed, and the next pump re-delivers from exactly that record.
#[test]
fn retryable_failure_stops_at_the_record_and_re_delivers() {
    let dir = scratch("retry");
    let _ = fs::remove_dir_all(&dir);
    let (_ring, _gen) = seeded_log(&dir, 4, ChangeLogConfig::default());
    let cursor = dir.join("relay.cursor");
    let log = ChangeLog::open(&dir).expect("reader");

    let mut relay =
        ChangeRelay::open(&cursor, ScriptedSink::failing(2, true)).expect("open relay");
    let err = relay.pump(&log).expect_err("sink refuses seq 2");
    match &err {
        RelayError::Sink { seq, source } => {
            assert_eq!(*seq, 2);
            assert!(source.is_retryable());
        }
        other => panic!("expected a sink error, got {}", other),
    }
    assert_eq!(relay.sink().seen, vec![0, 1], "stops AT the failing record");
    assert_eq!(
        relay.acked_through_seq(),
        Some(1),
        "cursor advanced only through what the sink confirmed"
    );
    assert!(!relay.is_poisoned(), "retryable never poisons");
    drop(relay);

    // The broker recovers; a fresh relay on the same cursor re-delivers from seq 2 onward.
    let mut relay = ChangeRelay::open(&cursor, ScriptedSink::new()).expect("reopen");
    let report = relay.pump(&log).expect("pump after recovery");
    assert_eq!(report.delivered, 2);
    assert_eq!(relay.sink().seen, vec![2, 3], "no hole in the feed");

    let _ = fs::remove_dir_all(&dir);
}

/// A PERMANENT sink failure fail-closes the relay: it is poisoned, every later pump refuses
/// rather than skipping the record, and the durable cursor still points AT the failing record
/// so a relay re-opened after the sink is fixed resumes exactly there.
#[test]
fn permanent_failure_poisons_the_relay_and_never_skips() {
    let dir = scratch("poison");
    let _ = fs::remove_dir_all(&dir);
    let (_ring, _gen) = seeded_log(&dir, 3, ChangeLogConfig::default());
    let cursor = dir.join("relay.cursor");
    let log = ChangeLog::open(&dir).expect("reader");

    let mut relay =
        ChangeRelay::open(&cursor, ScriptedSink::failing(1, false)).expect("open relay");
    let err = relay.pump(&log).expect_err("permanent refusal");
    assert!(matches!(err, RelayError::Sink { seq: 1, .. }), "{}", err);
    assert!(relay.is_poisoned());

    let again = relay.pump(&log).expect_err("poisoned relay refuses");
    assert!(
        matches!(again, RelayError::Poisoned(_)),
        "expected fail-closed, got {}",
        again
    );
    assert_eq!(relay.sink().seen, vec![0], "seq 1 was never skipped");
    drop(relay);

    let mut fixed = ChangeRelay::open(&cursor, ScriptedSink::new()).expect("reopen after fix");
    assert_eq!(fixed.pump(&log).expect("pump").delivered, 2);
    assert_eq!(fixed.sink().seen, vec![1, 2], "resumes AT the failing record");

    let _ = fs::remove_dir_all(&dir);
}

/// A re-base GAP marker is a first-class record on the feed: the relay hands it to the sink
/// (with `rebase = true` and no changes) instead of flattening it away, so a downstream
/// consumer learns the span was never captured.
#[test]
fn rebase_gap_marker_is_delivered_not_swallowed() {
    let dir = scratch("rebase");
    let _ = fs::remove_dir_all(&dir);
    let (ring, last_gen) = seeded_log(&dir, 2, ChangeLogConfig::default());

    // A commit the recorder MISSES (as after a dropped record), then an explicit resync.
    let pod = PodId::new("http://ex/g");
    let skipped = ring.publish(
        graph_with("<http://ex/s0> <http://ex/p> \"v0\" .\n"),
        [pod.clone()],
    );
    assert!(skipped.number() > last_gen);
    {
        let mut log = ChangeLog::open(&dir).expect("append handle");
        log.rebase_to(skipped.number()).expect("rebase");
    }

    let log = ChangeLog::open(&dir).expect("reader");
    let sink = JsonLinesSink::new(Vec::<u8>::new());
    let mut relay = ChangeRelay::open(dir.join("relay.cursor"), sink).expect("open relay");
    assert_eq!(relay.pump(&log).expect("pump").delivered, 3);

    let text = String::from_utf8(relay.sink().get_ref().clone()).expect("utf-8");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "the gap marker is its own message: {}", text);
    assert!(lines[2].contains("\"rebase\":true"), "{}", lines[2]);
    assert!(lines[2].contains("\"changes\":[]"), "{}", lines[2]);
    assert!(
        !lines[0].contains("\"rebase\":true") && !lines[1].contains("\"rebase\":true"),
        "only the gap record is marked"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The relay's ack watermark is the retention SAFETY BOUND, and a trimmed-away offset fails
/// closed: retention driven by `relay.retention_policy()` never drops a record the sink has
/// not taken, while trimming past the cursor (an operator overriding the watermark) makes the
/// next pump fail closed rather than silently resuming at a later offset.
#[test]
fn ack_watermark_gates_retention_and_a_trimmed_offset_fails_closed() {
    let dir = scratch("retention");
    let _ = fs::remove_dir_all(&dir);
    // segment_target_bytes = 1 => every record rolls its own segment, so retention has
    // whole segments to drop.
    let (_ring, _gen) = seeded_log(
        &dir,
        4,
        ChangeLogConfig {
            segment_target_bytes: 1,
            fsync: false,
        },
    );
    let log = ChangeLog::open(&dir).expect("reader");

    // Deliver only the first two records (a bounded pump), then hand the watermark to
    // retention: nothing beyond seq 1 may be dropped.
    let mut relay = ChangeRelay::open_with_config(
        dir.join("relay.cursor"),
        ScriptedSink::new(),
        RelayConfig {
            max_records_per_pump: 2,
            fsync: false,
        },
    )
    .expect("open relay");
    let report = relay.pump(&log).expect("bounded pump");
    assert_eq!(report.delivered, 2);
    assert!(report.has_more, "the log still holds records");
    assert_eq!(relay.acked_through_seq(), Some(1));

    let policy = relay.retention_policy();
    assert_eq!(policy.acked_through_seq, Some(1));
    let mut appender = ChangeLog::open(&dir).expect("append handle");
    let report = appender
        .apply_retention(&RetentionPolicy {
            max_total_bytes: Some(0), // maximal size pressure — ack must still hold it back
            ..policy
        })
        .expect("retention");
    assert_eq!(
        report.first_retained_seq, 2,
        "the ack watermark must hold back seq 2 (unacked) under maximal size pressure"
    );

    // Everything the relay still needs is retained, so it drains cleanly.
    let log = ChangeLog::open(&dir).expect("reader after retention");
    let drained = relay.pump(&log).expect("pump after retention");
    assert_eq!(drained.delivered, 2);
    assert_eq!(relay.sink().seen, vec![0, 1, 2, 3]);

    let _ = fs::remove_dir_all(&dir);
}

/// If retention drops records the relay's cursor still needs (an operator trimming past the
/// watermark), the next pump **fails closed** — it never silently resumes at a later offset,
/// which would hand the broker a feed with an invisible hole.
#[test]
fn a_cursor_trimmed_away_fails_closed_rather_than_skipping() {
    let dir = scratch("trimmed");
    let _ = fs::remove_dir_all(&dir);
    let (_ring, _gen) = seeded_log(
        &dir,
        4,
        ChangeLogConfig {
            segment_target_bytes: 1,
            fsync: false,
        },
    );

    let log = ChangeLog::open(&dir).expect("reader");
    let mut relay = ChangeRelay::open_with_config(
        dir.join("relay.cursor"),
        ScriptedSink::new(),
        RelayConfig {
            max_records_per_pump: 2,
            fsync: false,
        },
    )
    .expect("open relay");
    assert_eq!(relay.pump(&log).expect("bounded pump").delivered, 2);
    assert_eq!(relay.acked_through_seq(), Some(1));

    // The operator trims past the relay's watermark — seq 2 is gone.
    let mut appender = ChangeLog::open(&dir).expect("append handle");
    let report = appender
        .apply_retention(&RetentionPolicy {
            acked_through_seq: Some(3),
            max_total_bytes: Some(0),
            ..RetentionPolicy::default()
        })
        .expect("aggressive retention");
    assert!(
        report.first_retained_seq > 2,
        "this test needs seq 2 trimmed away (first_retained_seq {})",
        report.first_retained_seq
    );

    let log = ChangeLog::open(&dir).expect("reader after trim");
    let err = relay.pump(&log).expect_err("the cursor's next record is gone");
    assert!(
        matches!(err, RelayError::Log(_)),
        "expected a fail-closed log error, got {}",
        err
    );
    assert!(
        err.to_string().contains("precedes the earliest retained"),
        "{}",
        err
    );
    assert_eq!(
        relay.sink().seen,
        vec![0, 1],
        "nothing past the hole was delivered"
    );

    let _ = fs::remove_dir_all(&dir);
}
