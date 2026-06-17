//! Time-travel retention + lookup tests (opt-in `RingConfig::time_travel`):
//! `at()` hit/miss/aged-out, `published_at`/`as_of` resolution under an injected
//! deterministic clock, and the retention composition contract — the concurrency
//! bound K is a floor, the time-travel extension is bounded by count AND age.
//!
//! Determinism note (the concern the design called out): every timestamp
//! assertion runs against a test-local injected clock (`RingConfig::clock` is a
//! plain `fn` pointer reading a test-local atomic) — no test reads wall time, so
//! nothing here can flake on scheduler stalls or clock steps. Each test owns its
//! own static, so parallel test threads never share a clock.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sparq_serve::{GenerationRing, RingConfig, TimeTravelConfig};

/// Snapshots are plain u64 payloads here: time travel is about *which* generation
/// is served, not about drop instrumentation (tests/ring.rs covers reclamation).
fn ring(retain: usize, tt: Option<TimeTravelConfig>) -> GenerationRing<u64> {
    GenerationRing::with_config(
        0,
        RingConfig {
            retain,
            time_travel: tt,
            ..RingConfig::default()
        },
    )
}

// ---------------------------------------------------------------------------
// at(): hit / miss / aged-out
// ---------------------------------------------------------------------------

#[test]
fn at_hits_every_retained_generation_and_misses_the_rest() {
    let ring = ring(
        2,
        Some(TimeTravelConfig {
            max_generations: 8,
            max_age: None,
        }),
    );
    for i in 1..=10u64 {
        ring.publish(i, std::iter::empty());
    }
    // Time travel extends retention past K=2: current (10) + 8 older.
    assert_eq!(ring.oldest_retained(), 2);

    // Hit: every retained generation, including current, served with its own data.
    for n in 2..=10u64 {
        let g = ring
            .at(n)
            .unwrap_or_else(|| panic!("generation {n} should be retained"));
        assert_eq!(g.number(), n);
        assert_eq!(*g.snapshot(), n);
    }
    // Aged out: evicted by the max_generations bound.
    assert!(ring.at(1).is_none(), "generation 1 aged out of the window");
    assert!(ring.at(0).is_none(), "generation 0 aged out of the window");
    // Miss: never published.
    assert!(
        ring.at(11).is_none(),
        "generation 11 has not been published"
    );
    assert!(ring.at(u64::MAX).is_none());
}

#[test]
fn at_without_time_travel_serves_exactly_the_k_window() {
    // Opt-in contract: time_travel: None changes nothing — at() still works, but
    // only over the K generations the ring keeps for concurrency anyway.
    let ring = ring(3, None);
    for i in 1..=10u64 {
        ring.publish(i, std::iter::empty());
    }
    assert_eq!(ring.oldest_retained(), 7);
    assert!(
        ring.at(6).is_none(),
        "beyond K with no time travel: aged out"
    );
    assert_eq!(*ring.at(7).expect("inside K").snapshot(), 7);
    assert_eq!(*ring.at(10).expect("current").snapshot(), 10);
}

#[test]
fn at_returns_none_for_a_generation_only_a_reader_still_pins() {
    // The ring serves only retention it owns: a reader's pin keeps the generation
    // alive, but once the ring forgot its reference, at() is honestly None.
    let ring = ring(1, None);
    let pinned = ring.current(); // generation 0
    for i in 1..=5u64 {
        ring.publish(i, std::iter::empty());
    }
    assert_eq!(pinned.number(), 0);
    assert_eq!(
        *pinned.snapshot(),
        0,
        "the pin itself still serves old data"
    );
    assert!(ring.at(0).is_none(), "ring no longer owns generation 0");
}

// ---------------------------------------------------------------------------
// Retention composition: K is the floor, count and age bound the extension
// ---------------------------------------------------------------------------

#[test]
fn k_floor_wins_when_time_travel_bound_is_smaller() {
    // time-travel max_generations < K: the concurrency floor wins — time travel
    // can extend retention, never shrink it below K.
    let ring = ring(
        4,
        Some(TimeTravelConfig {
            max_generations: 1,
            max_age: None,
        }),
    );
    for i in 1..=10u64 {
        ring.publish(i, std::iter::empty());
    }
    assert_eq!(
        ring.oldest_retained(),
        6,
        "K=4 older generations kept despite max_generations=1"
    );
    assert_eq!(ring.live_generations(), 5);
}

#[test]
fn time_travel_bound_wins_when_larger_than_k() {
    let ring = ring(
        2,
        Some(TimeTravelConfig {
            max_generations: 6,
            max_age: None,
        }),
    );
    for i in 1..=10u64 {
        ring.publish(i, std::iter::empty());
    }
    assert_eq!(
        ring.oldest_retained(),
        4,
        "max_generations=6 older generations kept, K=2 floor inactive"
    );
    assert_eq!(ring.live_generations(), 7);
}

#[test]
fn max_age_evicts_only_beyond_the_k_floor_and_only_on_publish() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CLOCK_SECS: AtomicU64 = AtomicU64::new(1_000);
    fn clock() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(CLOCK_SECS.load(Ordering::SeqCst))
    }

    let ring = GenerationRing::with_config(
        0u64,
        RingConfig {
            retain: 1,
            time_travel: Some(TimeTravelConfig {
                max_generations: 100,
                max_age: Some(Duration::from_secs(60)),
            }),
            clock,
        },
    );
    // Publish generations 1..=4 at t=1010, 1020, 1030, 1040.
    for i in 1..=4u64 {
        CLOCK_SECS.store(1_000 + i * 10, Ordering::SeqCst);
        ring.publish(i, std::iter::empty());
    }
    // All within 60 s of t=1040: everything retained.
    assert_eq!(ring.oldest_retained(), 0);

    // Move the clock to t=1095: generations 0 (t=1000) and 1 (t=1010) are now
    // over-age — but eviction is publish-driven, so they are STILL servable.
    CLOCK_SECS.store(1_095, Ordering::SeqCst);
    assert!(ring.at(0).is_some(), "age eviction only happens at publish");

    // The next publish (gen 5 at t=1095) prunes them; 2 (t=1020) is within 75 s…
    // no — within max_age=60 of 1095 means published_at >= 1035: gens 2 (1020)
    // and 3 (1030) are over-age too. But gen 4 sits inside the K=1 floor, which
    // age NEVER evicts.
    ring.publish(5, std::iter::empty());
    assert_eq!(
        ring.oldest_retained(),
        4,
        "over-age extension evicted, K floor kept"
    );
    assert!(ring.at(3).is_none(), "aged out by max_age");
    assert_eq!(*ring.at(4).expect("K floor survives any age").snapshot(), 4);
    assert_eq!(*ring.at(5).expect("current").snapshot(), 5);
}

// ---------------------------------------------------------------------------
// published_at + as_of: timestamp → generation resolution
// ---------------------------------------------------------------------------

#[test]
fn as_of_resolves_timestamps_to_the_generation_current_at_that_time() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CLOCK_SECS: AtomicU64 = AtomicU64::new(100);
    fn clock() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(CLOCK_SECS.load(Ordering::SeqCst))
    }
    let at = |secs: u64| UNIX_EPOCH + Duration::from_secs(secs);

    let ring = GenerationRing::with_config(
        0u64,
        RingConfig {
            retain: 0,
            time_travel: Some(TimeTravelConfig {
                max_generations: 2,
                max_age: None,
            }),
            clock,
        },
    );
    // gen 0 @ t=100 (construction), gen 1 @ t=200, gen 2 @ t=300.
    CLOCK_SECS.store(200, Ordering::SeqCst);
    ring.publish(1, std::iter::empty());
    CLOCK_SECS.store(300, Ordering::SeqCst);
    ring.publish(2, std::iter::empty());

    assert_eq!(
        ring.at(1).unwrap().published_at(),
        at(200),
        "publish stamps the injected clock"
    );

    // Exact hits and in-between times resolve to the generation current THEN.
    assert_eq!(ring.as_of(at(100)).unwrap().number(), 0);
    assert_eq!(ring.as_of(at(199)).unwrap().number(), 0);
    assert_eq!(ring.as_of(at(200)).unwrap().number(), 1);
    assert_eq!(ring.as_of(at(250)).unwrap().number(), 1);
    assert_eq!(ring.as_of(at(300)).unwrap().number(), 2);
    // After the newest publish: the current generation was current then.
    assert_eq!(ring.as_of(at(10_000)).unwrap().number(), 2);
    // Before the ring existed: no generation covers that time.
    assert!(ring.as_of(at(99)).is_none());

    // Age gen 0 out (max_generations=2 older): as_of for its window is an honest
    // None — never silently substituted with a different point in time.
    CLOCK_SECS.store(400, Ordering::SeqCst);
    ring.publish(3, std::iter::empty());
    assert!(ring.at(0).is_none());
    assert!(
        ring.as_of(at(150)).is_none(),
        "the generation covering t=150 aged out"
    );
    assert_eq!(
        ring.as_of(at(200)).unwrap().number(),
        1,
        "still-retained history resolves"
    );
}
