//! Generation-ring unit tests (research/concurrent-serving.md §6.4 invariants):
//! readers pin past the writer, retention forgets ring references without
//! invalidating reader pins, epochs bump only for touched pods.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sparq_serve::{GenerationRing, PodId, RingConfig};

/// Instrumented snapshot: carries a payload and decrements a shared live-counter
/// on drop, so tests can observe exactly when a generation is reclaimed.
struct MockSnapshot {
    value: u64,
    live: Arc<AtomicUsize>,
}

impl MockSnapshot {
    fn new(value: u64, live: &Arc<AtomicUsize>) -> Self {
        live.fetch_add(1, Ordering::SeqCst);
        MockSnapshot { value, live: live.clone() }
    }
}

impl Drop for MockSnapshot {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::SeqCst);
    }
}

fn pods(ids: &[&str]) -> Vec<PodId> {
    ids.iter().map(|s| PodId::from(*s)).collect()
}

#[test]
fn initial_state_is_generation_zero() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::new(MockSnapshot::new(0, &live));
    let gen0 = ring.current();
    assert_eq!(gen0.number(), 0);
    assert_eq!(gen0.snapshot().value, 0);
    assert!(gen0.epochs().is_empty());
    assert_eq!(ring.live_generations(), 1);
    assert_eq!(ring.oldest_retained(), 0);
}

#[test]
fn readers_pin_old_generations_while_writer_publishes_past() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::with_config(MockSnapshot::new(0, &live), RingConfig { retain: 2, ..RingConfig::default() });

    // A reader pins generation 0 (e.g. a long-lived stream).
    let pinned = ring.current();

    // The writer publishes far past the retention bound — it must never wait and
    // the pinned generation must remain readable, byte-identical.
    for i in 1..=50u64 {
        let published = ring.publish(MockSnapshot::new(i, &live), pods(&["pod:a"]));
        assert_eq!(published.number(), i);
    }
    assert_eq!(ring.current().number(), 50);
    assert_eq!(ring.current().snapshot().value, 50);

    // Old data still readable through the pin, untouched by 50 publishes.
    assert_eq!(pinned.number(), 0);
    assert_eq!(pinned.snapshot().value, 0);
    assert!(pinned.epochs().is_empty());

    // Alive: ring's K+1 (= 3) plus the reader-pinned gen 0.
    assert_eq!(ring.live_generations(), 4);
    assert_eq!(ring.oldest_retained(), 48);

    drop(pinned);
    assert_eq!(ring.live_generations(), 3);
}

#[test]
fn retention_bound_drops_ring_references_but_not_reader_pins() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::with_config(MockSnapshot::new(0, &live), RingConfig { retain: 3, ..RingConfig::default() });

    // No readers: beyond current + K, forgotten generations are freed promptly.
    for i in 1..=10u64 {
        ring.publish(MockSnapshot::new(i, &live), std::iter::empty());
    }
    assert_eq!(live.load(Ordering::SeqCst), 4, "ring retains current + K = 4");
    assert_eq!(ring.live_generations(), 4);
    assert_eq!(ring.oldest_retained(), 7);

    // A reader pins generation 10; the ring then forgets its own reference to it.
    let pinned = ring.current();
    assert_eq!(pinned.number(), 10);
    for i in 11..=20u64 {
        ring.publish(MockSnapshot::new(i, &live), std::iter::empty());
    }
    assert_eq!(ring.oldest_retained(), 17, "ring's own refs moved past the pin");
    // current+K from the ring, plus the externally pinned gen 10.
    assert_eq!(live.load(Ordering::SeqCst), 5);
    assert_eq!(ring.live_generations(), 5);
    // The pin is NOT invalidated by retention: data still readable.
    assert_eq!(pinned.snapshot().value, 10);

    // Reclamation is the reader's Arc drop — immediate, no polling, no grace timer.
    drop(pinned);
    assert_eq!(live.load(Ordering::SeqCst), 4);
    assert_eq!(ring.live_generations(), 4);
}

#[test]
fn epoch_bumps_only_for_touched_pods() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::new(MockSnapshot::new(0, &live));
    let (a, b, c) = (PodId::from("pod:a"), PodId::from("pod:b"), PodId::from("pod:c"));

    let g1 = ring.publish(MockSnapshot::new(1, &live), pods(&["pod:a", "pod:b"]));
    assert_eq!(g1.epochs().epoch(&a), 1);
    assert_eq!(g1.epochs().epoch(&b), 1);
    assert_eq!(g1.epochs().epoch(&c), 0, "untouched pod stays at 0");
    assert_eq!(g1.epochs().len(), 2);

    let g2 = ring.publish(MockSnapshot::new(2, &live), pods(&["pod:b"]));
    assert_eq!(g2.epochs().epoch(&a), 1, "untouched pod's epoch carried unchanged");
    assert_eq!(g2.epochs().epoch(&b), 2);
    assert_eq!(g2.epochs().epoch(&c), 0);

    // Published vectors are immutable: g1's view is unaffected by g2's publish.
    assert_eq!(g1.epochs().epoch(&b), 1);

    // Duplicate mentions within one publish bump once.
    let g3 = ring.publish(MockSnapshot::new(3, &live), pods(&["pod:c", "pod:c", "pod:c"]));
    assert_eq!(g3.epochs().epoch(&c), 1);
    assert_eq!(g3.epochs().epoch(&a), 1);
    assert_eq!(g3.epochs().epoch(&b), 2);
}

#[test]
fn publish_with_no_touched_pods_keeps_epochs() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::new(MockSnapshot::new(0, &live));
    let a = PodId::from("pod:a");

    let g1 = ring.publish(MockSnapshot::new(1, &live), pods(&["pod:a"]));
    let g2 = ring.publish(MockSnapshot::new(2, &live), std::iter::empty());
    assert_eq!(g2.number(), 2);
    assert_eq!(g2.epochs().epoch(&a), 1);
    assert_eq!(g1.epochs(), g2.epochs());
}

#[test]
fn retain_zero_keeps_only_current() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::with_config(MockSnapshot::new(0, &live), RingConfig { retain: 0, ..RingConfig::default() });
    for i in 1..=5u64 {
        ring.publish(MockSnapshot::new(i, &live), std::iter::empty());
    }
    assert_eq!(live.load(Ordering::SeqCst), 1);
    assert_eq!(ring.oldest_retained(), 5);
    assert_eq!(ring.live_generations(), 1);
}

#[test]
fn dropping_the_ring_releases_its_references_but_not_pins() {
    let live = Arc::new(AtomicUsize::new(0));
    let ring = GenerationRing::new(MockSnapshot::new(0, &live));
    for i in 1..=3u64 {
        ring.publish(MockSnapshot::new(i, &live), std::iter::empty());
    }
    let pinned = ring.current();
    drop(ring);
    // Everything the ring held is gone; the reader's pin survives.
    assert_eq!(live.load(Ordering::SeqCst), 1);
    assert_eq!(pinned.snapshot().value, 3);
}
