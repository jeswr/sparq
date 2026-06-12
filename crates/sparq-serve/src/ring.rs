//! The generation ring (research/concurrent-serving.md §6.4).
//!
//! `ArcSwap<Generation>` current pointer + a bounded deque of the ring's own
//! strong references to recent generations. The writer publishes forward and
//! NEVER waits; readers pin generations for as long as they live; reclamation is
//! plain `Arc` drop. The retention bound K limits only how many *old* generations
//! the ring itself keeps alive for late-arriving pinners — it cannot invalidate a
//! generation a reader already holds.
//!
//! Deviation from §6.4, recorded honestly: the research text says "the ring counts
//! live *pinned* generations; at the bound new streams are refused
//! (`Shed(SnapshotPressure)`) and the oldest pinned stream past a wall-clock cap is
//! cancelled". Admission control and stream cancellation belong to the scheduler /
//! streaming waves (C/D) — A1 ships the substrate: K bounds the ring's own
//! references, and [`live_generations`](GenerationRing::live_generations) /
//! [`oldest_retained`](GenerationRing::oldest_retained) expose exactly the counts
//! those waves (and the metrics endpoint) will gate on.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Weak};

use arc_swap::ArcSwap;
use rustc_hash::FxHashSet;

use crate::epoch::{PodEpochs, PodId};

/// Default retention bound K. §6.4 prescribes "config; e.g. 4–8"; we default to
/// the low end because the memory budget is `live + K × delta` (~50 MB/generation
/// measured at 1 M triples early in a divergence) and K is cheap to raise per
/// deployment via [`RingConfig`].
pub const DEFAULT_RETAIN: usize = 4;

/// Configuration for a [`GenerationRing`].
#[derive(Clone, Debug)]
pub struct RingConfig {
    /// Retention bound K: how many generations *older than current* the ring keeps
    /// its own strong reference to. Beyond K the ring forgets its reference — the
    /// generation then lives exactly as long as its last reader `Arc`.
    pub retain: usize,
}

impl Default for RingConfig {
    fn default() -> Self {
        RingConfig { retain: DEFAULT_RETAIN }
    }
}

/// One immutable published state of the store: a snapshot handle, the generation
/// number, and the per-pod epoch vector at publish time.
///
/// Generic over the snapshot handle `S` (see the crate docs for why this is a type
/// parameter and not a trait): production uses the engine's real immutable store
/// (`sparq_core::Graph`, whose existing snapshot mechanism is a 27 ns `Arc` clone);
/// tests use instrumented mocks. A `Generation` is only ever handed out as
/// `Arc<Generation<S>>` — holding that `Arc` *is* pinning the snapshot.
#[derive(Debug)]
pub struct Generation<S> {
    number: u64,
    snapshot: S,
    epochs: PodEpochs,
}

impl<S> Generation<S> {
    /// The generation number: 0 for the ring's initial state, +1 per publish.
    pub fn number(&self) -> u64 {
        self.number
    }

    /// The immutable store snapshot this generation serves reads from.
    pub fn snapshot(&self) -> &S {
        &self.snapshot
    }

    /// The per-pod epoch vector as of this generation (Wave B's cache-invalidation
    /// hook, §6.3).
    pub fn epochs(&self) -> &PodEpochs {
        &self.epochs
    }
}

/// The generation ring: lock-free `current()`, wait-free-for-readers `publish()`,
/// bounded retention of the ring's own references.
///
/// Concurrency contract:
/// - [`current`](Self::current) is lock-free (`ArcSwap::load_full`) — the hot
///   read path never touches the mutex.
/// - [`publish`](Self::publish) is intended for the single sequenced writer
///   (§6.5); it takes the internal mutex, but never blocks on readers — the only
///   other holders of that mutex are the O(K) introspection accessors.
/// - Dropping the ring releases its references; pinned generations survive with
///   their readers (the ring owns retention, not lifetime).
pub struct GenerationRing<S> {
    /// The published current generation. Readers `load` here, lock-free.
    current: ArcSwap<Generation<S>>,
    /// Retention bound K (immutable after construction).
    retain: usize,
    /// Writer/introspection-side state; never touched by `current()`.
    inner: Mutex<RingInner<S>>,
}

struct RingInner<S> {
    /// The ring's own strong references: the current generation plus up to
    /// `retain` older ones (`len ≤ retain + 1`).
    retained: VecDeque<Arc<Generation<S>>>,
    /// Weak references to every generation that may still be alive, oldest first;
    /// pruned opportunistically on publish and on introspection. Powers
    /// `live_generations` without giving the ring any lifetime influence.
    registry: VecDeque<Weak<Generation<S>>>,
}

impl<S> GenerationRing<S> {
    /// Creates a ring whose generation 0 wraps `initial` with an empty epoch
    /// vector, using the default retention bound ([`DEFAULT_RETAIN`]).
    pub fn new(initial: S) -> Self {
        Self::with_config(initial, RingConfig::default())
    }

    /// Creates a ring with an explicit [`RingConfig`].
    pub fn with_config(initial: S, config: RingConfig) -> Self {
        let gen0 = Arc::new(Generation {
            number: 0,
            snapshot: initial,
            epochs: PodEpochs::default(),
        });
        let inner = RingInner {
            retained: VecDeque::from([gen0.clone()]),
            registry: VecDeque::from([Arc::downgrade(&gen0)]),
        };
        GenerationRing {
            current: ArcSwap::from(gen0),
            retain: config.retain,
            inner: Mutex::new(inner),
        }
    }

    /// The current generation, pinned: hold the returned `Arc` and the snapshot
    /// stays readable no matter how far the writer publishes past it. Lock-free.
    pub fn current(&self) -> Arc<Generation<S>> {
        self.current.load_full()
    }

    /// Publishes the next generation wrapping `snapshot`, bumping the epoch of
    /// every pod in `touched` (duplicates are deduplicated: one bump per pod per
    /// publish). Returns the published generation.
    ///
    /// Single sequenced writer intended (§6.5); concurrent calls are nevertheless
    /// safe (the mutex serialises them) so misuse degrades to serialisation, not
    /// corruption. This never waits on readers and never frees anything a reader
    /// pins: it only drops the *ring's* reference to generations older than K.
    pub fn publish<I>(&self, snapshot: S, touched: I) -> Arc<Generation<S>>
    where
        I: IntoIterator<Item = PodId>,
    {
        let mut inner = self.inner.lock().expect("generation ring poisoned");
        let prev = self.current.load();
        let mut epochs = prev.epochs.clone();
        let mut seen: FxHashSet<PodId> = FxHashSet::default();
        for pod in touched {
            if seen.insert(pod.clone()) {
                epochs.bump(pod);
            }
        }
        let next = Arc::new(Generation {
            number: prev.number + 1,
            snapshot,
            epochs,
        });
        drop(prev);
        // Swap the read pointer first: readers see the new generation immediately;
        // retention bookkeeping below is invisible to them.
        self.current.store(next.clone());
        inner.retained.push_back(next.clone());
        // Forget the ring's own reference beyond current + K. If a reader still
        // pins the forgotten generation it stays alive through that reader's Arc.
        while inner.retained.len() > self.retain + 1 {
            inner.retained.pop_front();
        }
        inner.registry.retain(|w| w.strong_count() > 0);
        inner.registry.push_back(Arc::downgrade(&next));
        next
    }

    /// How many generations are still alive anywhere — retained by the ring *or*
    /// pinned by readers (includes the current generation). The §6.4 pressure
    /// signal for later waves' stream admission and for metrics.
    pub fn live_generations(&self) -> usize {
        let mut inner = self.inner.lock().expect("generation ring poisoned");
        inner.registry.retain(|w| w.strong_count() > 0);
        inner.registry.len()
    }

    /// The number of the oldest generation the ring itself still holds a strong
    /// reference to. Older generations may still be alive, but only through
    /// readers' own pins.
    pub fn oldest_retained(&self) -> u64 {
        let inner = self.inner.lock().expect("generation ring poisoned");
        inner
            .retained
            .front()
            .map(|g| g.number)
            .expect("ring always retains the current generation")
    }

    /// The configured retention bound K.
    pub fn retain_bound(&self) -> usize {
        self.retain
    }
}
