//! Concurrent stress test (§6.4): reader threads load and hold generations across
//! many publishes while the single writer publishes flat out. Asserts no torn
//! reads via two invariants every reader checks on every loaded generation:
//!
//! 1. `snapshot.value == generation number` (the writer stamps each snapshot with
//!    the number it will receive) — a torn current-pointer or a generation built
//!    from a stale predecessor would break it;
//! 2. the sum of all pod epochs equals the generation number (each publish bumps
//!    exactly one pod by exactly one) — a torn or mis-derived epoch vector would
//!    break it.
//!
//! Runtime is bounded by publish count (~50 k), well under the 5 s budget.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use sparq_serve::{Epoch, Generation, GenerationRing, PodId, RingConfig};

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

fn check(gen: &Generation<MockSnapshot>) {
    assert_eq!(
        gen.snapshot().value,
        gen.number(),
        "torn read: snapshot does not match its generation number"
    );
    let epoch_sum: Epoch = gen.epochs().iter().map(|(_, e)| e).sum();
    assert_eq!(
        epoch_sum,
        gen.number(),
        "torn read: epoch vector inconsistent with generation number"
    );
}

#[test]
fn concurrent_readers_pin_across_many_publishes_without_torn_reads() {
    const PUBLISHES: u64 = 50_000;
    const READERS: usize = 4;
    const POD_FANOUT: u64 = 8;

    let live = Arc::new(AtomicUsize::new(0));
    let ring = Arc::new(GenerationRing::with_config(
        MockSnapshot::new(0, &live),
        RingConfig { retain: 4, ..RingConfig::default() },
    ));
    let done = Arc::new(AtomicBool::new(false));

    let readers: Vec<_> = (0..READERS)
        .map(|r| {
            let ring = ring.clone();
            let done = done.clone();
            std::thread::spawn(move || {
                let mut held: Vec<Arc<Generation<MockSnapshot>>> = Vec::new();
                let mut max_seen = 0u64;
                let mut loads = 0u64;
                while !done.load(Ordering::Acquire) {
                    let gen = ring.current();
                    check(&gen);
                    assert!(
                        gen.number() >= max_seen,
                        "current generation went backwards: {} after {}",
                        gen.number(),
                        max_seen
                    );
                    max_seen = gen.number();
                    loads += 1;
                    // Pin some generations across many future publishes; stagger
                    // per reader so pins span different windows.
                    if loads % (1_000 + 137 * r as u64) == 0 {
                        held.push(gen);
                        if held.len() > 8 {
                            // Long-held pins must still be intact before release.
                            for old in held.drain(..4) {
                                check(&old);
                            }
                        }
                    }
                }
                // Everything still held at the end must be intact and readable.
                for old in &held {
                    check(old);
                }
                loads
            })
        })
        .collect();

    let mut published = 0u64;
    for i in 1..=PUBLISHES {
        let pod = PodId::new(format!("pod:{}", i % POD_FANOUT));
        let gen = ring.publish(MockSnapshot::new(i, &live), [pod]);
        assert_eq!(gen.number(), i, "single writer must observe dense numbering");
        published += 1;
    }
    done.store(true, Ordering::Release);

    let mut total_loads = 0u64;
    for r in readers {
        total_loads += r.join().expect("reader thread panicked (torn read?)");
    }
    assert_eq!(published, PUBLISHES);
    assert!(total_loads > 0, "readers must have observed the run");

    let final_gen = ring.current();
    assert_eq!(final_gen.number(), PUBLISHES);
    check(&final_gen);

    // With all reader pins released, only the ring's own retention keeps
    // generations alive: current + K, plus the `final_gen` pin (already counted —
    // gen PUBLISHES is retained by the ring too).
    drop(final_gen);
    assert_eq!(ring.live_generations(), ring.retain_bound() + 1);
    assert_eq!(live.load(Ordering::SeqCst), ring.retain_bound() + 1);
    assert_eq!(ring.oldest_retained(), PUBLISHES - ring.retain_bound() as u64);
}
