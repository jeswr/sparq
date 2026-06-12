//! The sequenced writer over the PRODUCTION store: `GraphApplier` applying real
//! SPARQL Updates to `sparq_core::Graph` through the engine's rebuild (fork) +
//! delta-overlay (apply) paths. Proves the §6.5 batch semantics hold end-to-end
//! on the real snapshot type, including failed-SPARQL isolation, and carries the
//! `--ignored` fork-cost measurement quoted in `applier.rs`'s module docs.

use std::sync::Arc;
use std::time::{Duration, Instant};

use sparq_core::Graph;
use sparq_serve::{GenerationRing, GraphApplier, PodId, WriteError, Writer, WriterConfig};

fn graph_with(n: usize) -> Graph {
    let mut nt = String::new();
    for i in 0..n {
        nt.push_str(&format!("<http://ex/s{i}> <http://ex/p> \"v{i}\" .\n"));
    }
    Graph::load_str(&nt, "ntriples").expect("load test graph")
}

fn insert(i: usize) -> String {
    format!("INSERT DATA {{ <http://ex/new{i}> <http://ex/p> \"w{i}\" }}")
}

/// A batch of real SPARQL updates → one generation, one folded graph.
#[test]
fn sparql_batch_publishes_one_generation() {
    let ring = Arc::new(GenerationRing::new(graph_with(10)));
    let writer = Writer::spawn(
        ring.clone(),
        GraphApplier,
        WriterConfig { window: Duration::from_millis(200), max_batch: 64 },
    );

    let before = ring.current(); // pinned across the commit
    for i in 0..3 {
        writer.submit_detached(insert(i), [PodId::from("pod:alice")]).unwrap();
    }
    let generation = writer
        .submit(
            "DELETE DATA { <http://ex/s0> <http://ex/p> \"v0\" }".to_string(),
            [PodId::from("pod:alice")],
        )
        .unwrap();

    assert_eq!(generation, 1, "3 inserts + 1 delete in one window = one generation");
    let current = ring.current();
    assert_eq!(current.number(), 1);
    assert_eq!(current.snapshot().len(), 12, "10 + 3 inserts - 1 delete");
    assert_eq!(current.epochs().epoch(&PodId::from("pod:alice")), 1);
    // Snapshot isolation: the pinned pre-commit generation is untouched.
    assert_eq!(before.snapshot().len(), 10);
}

/// A non-SPARQL update fails alone: its submitter gets the parse error, the
/// rest of the batch lands, and a later window works on the published result.
#[test]
fn bad_sparql_is_isolated_from_its_batch() {
    let ring = Arc::new(GenerationRing::new(graph_with(5)));
    let writer = Arc::new(Writer::spawn(
        ring.clone(),
        GraphApplier,
        WriterConfig { window: Duration::from_millis(300), max_batch: 64 },
    ));

    writer.submit_detached(insert(0), [PodId::from("pod:a")]).unwrap();
    let bad = {
        let writer = writer.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            writer.submit("THIS IS NOT SPARQL".to_string(), [PodId::from("pod:bad")])
        })
    };
    std::thread::sleep(Duration::from_millis(120));
    let ok = writer.submit(insert(1), [PodId::from("pod:b")]).unwrap();

    match bad.join().unwrap() {
        Err(WriteError::Rejected(e)) => {
            assert!(!e.is_empty(), "parse error is forwarded to the submitter")
        }
        other => panic!("bad SPARQL must be rejected, got {other:?}"),
    }
    assert_eq!(ok, 1, "the surviving updates publish as one generation");
    let current = ring.current();
    assert_eq!(current.number(), 1);
    assert_eq!(current.snapshot().len(), 7, "5 + the 2 good inserts");
    assert_eq!(current.epochs().epoch(&PodId::from("pod:a")), 1);
    assert_eq!(current.epochs().epoch(&PodId::from("pod:b")), 1);
    assert_eq!(current.epochs().epoch(&PodId::from("pod:bad")), 0);

    // Next window builds on the committed state.
    assert_eq!(writer.submit(insert(2), [PodId::from("pod:a")]).unwrap(), 2);
    assert_eq!(ring.current().snapshot().len(), 8);
}

/// Consecutive windows produce consecutive generations over the real store.
#[test]
fn consecutive_windows_consecutive_generations() {
    let ring = Arc::new(GenerationRing::new(graph_with(2)));
    let writer = Writer::spawn(
        ring.clone(),
        GraphApplier,
        WriterConfig { window: Duration::from_millis(1), max_batch: 256 },
    );

    for expect in 1..=4u64 {
        let g = writer.submit(insert(expect as usize), [PodId::from("pod:a")]).unwrap();
        assert_eq!(g, expect);
    }
    assert_eq!(ring.current().snapshot().len(), 6);
    assert_eq!(ring.current().epochs().epoch(&PodId::from("pod:a")), 4);
}

/// The fork-cost measurement quoted in `applier.rs` (run manually):
/// `cargo test -p sparq-serve --release fork_cost_measurement -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not an assertion — run with --ignored --nocapture in release"]
fn fork_cost_measurement() {
    for &n in &[100_000usize, 1_000_000] {
        let g = graph_with(n);
        let mut applier = GraphApplier;
        // Warm + measure (3 forks, report best — steady-state estimate).
        let mut best = Duration::MAX;
        for _ in 0..3 {
            let t = Instant::now();
            let forked = sparq_serve::ApplyUpdates::fork(&mut applier, &g).unwrap();
            best = best.min(t.elapsed());
            assert_eq!(forked.len(), n);
        }
        println!("fork (engine rebuild) of {n} triples: {best:?} (best of 3)");
    }
}
