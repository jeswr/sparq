//! Integration with the PRODUCTION snapshot type (§6.4 / crate-docs decision):
//! the ring instantiated with `sparq_core::Graph` — the same immutable store whose
//! `Arc` clone is today's `AppState::snapshot()` (27 ns, §4.3). Proves the generic
//! `Generation<S>` wraps the real store with zero adaptation: snapshot-consistent
//! reads at a pinned generation while the writer publishes past it, including
//! cross-thread (Graph is Send + Sync, as the axum server already relies on).

use std::sync::Arc;

use sparq_core::Graph;
use sparq_serve::{GenerationRing, PodId, RingConfig};

fn graph_with(n: usize) -> Graph {
    let mut nt = String::new();
    for i in 0..n {
        nt.push_str(&format!("<http://ex/s{i}> <http://ex/p> \"v{i}\" .\n"));
    }
    Graph::load_str(&nt, "ntriples").expect("load test graph")
}

#[test]
fn ring_serves_real_graph_snapshots_consistently() {
    let ring = GenerationRing::with_config(graph_with(2), RingConfig { retain: 2 });

    let gen0 = ring.current();
    assert_eq!(gen0.snapshot().len(), 2);

    // Writer publishes new immutable graphs (in production: the sequenced writer's
    // working copy after a group-commit window, Wave A2).
    for n in 3..=8 {
        ring.publish(graph_with(n), [PodId::from("pod:alice")]);
    }

    // The pinned generation still answers at its own state — snapshot-consistent
    // streaming is free by construction (§6.6).
    assert_eq!(gen0.number(), 0);
    assert_eq!(gen0.snapshot().len(), 2, "pinned graph unchanged after 6 publishes");
    let current = ring.current();
    assert_eq!(current.number(), 6);
    assert_eq!(current.snapshot().len(), 8);
    assert_eq!(current.epochs().epoch(&PodId::from("pod:alice")), 6);
    assert_eq!(current.epochs().epoch(&PodId::from("pod:bob")), 0);

    // Ring retention moved past gen 0; the pin alone keeps it alive.
    assert_eq!(ring.oldest_retained(), 4);
    assert_eq!(ring.live_generations(), 4); // current + K (3) + pinned gen 0
}

#[test]
fn real_graph_generations_are_readable_across_threads() {
    let ring = Arc::new(GenerationRing::new(graph_with(5)));
    let pinned = ring.current();

    let writer = {
        let ring = ring.clone();
        std::thread::spawn(move || {
            for n in 6..=20 {
                ring.publish(graph_with(n), [PodId::from("pod:a")]);
            }
        })
    };
    let reader = std::thread::spawn(move || {
        // A reader on another thread querying its pinned generation while the
        // writer publishes: always sees exactly its snapshot.
        for _ in 0..100 {
            assert_eq!(pinned.snapshot().len(), 5);
        }
        pinned.number()
    });

    writer.join().expect("writer panicked");
    assert_eq!(reader.join().expect("reader panicked"), 0);
    assert_eq!(ring.current().snapshot().len(), 20);
}
