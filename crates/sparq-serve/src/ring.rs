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
use std::time::{Duration, SystemTime};

use arc_swap::ArcSwap;
use rustc_hash::FxHashSet;

use crate::epoch::{PodEpochs, PodId};

/// Default retention bound K. §6.4 prescribes "config; e.g. 4–8"; we default to
/// the low end because the memory budget is `live + K × delta` (~50 MB/generation
/// measured at 1 M triples early in a divergence) and K is cheap to raise per
/// deployment via [`RingConfig`].
pub const DEFAULT_RETAIN: usize = 4;

/// Opt-in extended retention for time-travel queries: keep generations beyond the
/// concurrency bound K so [`GenerationRing::at`] / [`GenerationRing::as_of`] can
/// serve them as queryable historical snapshots.
///
/// **Memory cost, stated honestly:** with today's [`GraphApplier`](crate::GraphApplier)
/// every published generation is a FULL `Graph` (the per-batch O(graph) fork shares no
/// structure), so time-travel retention costs `M × full graph` — at the measured
/// ~780 MB/1 M-triple graph, `max_generations: 16` can pin ~12 GB of history. Budget
/// `max_generations` (and/or `max_age`) accordingly. When the recorded structural-fork
/// follow-up lands (persistent/COW indexes — see the applier module docs), retained
/// generations become delta chains and this cost collapses; the API here is
/// deliberately number/timestamp-based so that swap needs **no API change** (an
/// OSTRICH-style delta-chain archive is the named follow-up, out of scope for Wave A).
#[derive(Clone, Debug)]
pub struct TimeTravelConfig {
    /// Keep up to this many generations older than current available for time
    /// travel. Composes with the concurrency bound K as a **floor**: the ring
    /// retains `max(retain, max_generations)` older generations (time travel can
    /// extend retention, never shrink it below K).
    pub max_generations: usize,
    /// Additionally evict time-travel generations older than this age (measured
    /// against the ring's clock at publish time). The age bound applies only to
    /// the time-travel *extension* — the K newest older-than-current generations
    /// are never age-evicted (the concurrency floor wins). `None` = count-bounded
    /// only. Eviction is publish-driven (no background timer): an over-age
    /// generation stays retained — and `at()`-servable — until the next publish
    /// prunes it.
    pub max_age: Option<Duration>,
}

impl Default for TimeTravelConfig {
    fn default() -> Self {
        TimeTravelConfig { max_generations: 16, max_age: None }
    }
}

/// Configuration for a [`GenerationRing`].
#[derive(Clone, Debug)]
pub struct RingConfig {
    /// Retention bound K: how many generations *older than current* the ring keeps
    /// its own strong reference to. Beyond K the ring forgets its reference — the
    /// generation then lives exactly as long as its last reader `Arc`.
    pub retain: usize,
    /// Opt-in time-travel retention (default `None` — no extended retention, no
    /// behaviour change). See [`TimeTravelConfig`] for the memory-cost contract.
    pub time_travel: Option<TimeTravelConfig>,
    /// The clock that stamps [`Generation::published_at`] and drives `max_age`
    /// eviction. A plain `fn` pointer (not a closure) keeps `RingConfig` `Clone +
    /// Debug` and the default zero-cost; tests inject a deterministic clock (e.g.
    /// reading an atomic) so timestamp assertions never race wall time. Defaults
    /// to [`SystemTime::now`].
    pub clock: fn() -> SystemTime,
}

impl Default for RingConfig {
    fn default() -> Self {
        RingConfig { retain: DEFAULT_RETAIN, time_travel: None, clock: SystemTime::now }
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
    published_at: SystemTime,
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

    /// When this generation was published, per the ring's configured clock
    /// ([`RingConfig::clock`]; generation 0 is stamped at ring construction).
    /// Lets callers resolve "as of T" to a generation number
    /// ([`GenerationRing::as_of`]). Only as trustworthy as the clock: the default
    /// `SystemTime::now` is wall time (non-monotonic under clock steps); inject a
    /// deterministic clock in tests.
    pub fn published_at(&self) -> SystemTime {
        self.published_at
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
    /// Opt-in time-travel retention (immutable after construction).
    time_travel: Option<TimeTravelConfig>,
    /// Stamps `published_at` and drives `max_age` eviction.
    clock: fn() -> SystemTime,
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
            published_at: (config.clock)(),
        });
        let inner = RingInner {
            retained: VecDeque::from([gen0.clone()]),
            registry: VecDeque::from([Arc::downgrade(&gen0)]),
        };
        GenerationRing {
            current: ArcSwap::from(gen0),
            retain: config.retain,
            time_travel: config.time_travel,
            clock: config.clock,
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
        let now = (self.clock)();
        let next = Arc::new(Generation {
            number: prev.number + 1,
            snapshot,
            epochs,
            published_at: now,
        });
        drop(prev);
        // Swap the read pointer first: readers see the new generation immediately;
        // retention bookkeeping below is invisible to them.
        self.current.store(next.clone());
        inner.retained.push_back(next.clone());
        // Forget the ring's own reference beyond the retention bound. If a reader
        // still pins a forgotten generation it stays alive through that reader's
        // Arc. Retention composition (documented on [`TimeTravelConfig`]): the
        // concurrency bound K is a FLOOR — the K newest older-than-current
        // generations are always kept; the time-travel extension keeps a
        // generation only while it satisfies BOTH its count bound
        // (`max_generations`) and its age bound (`max_age`). The deque is
        // oldest-first, so once the front survives, everything behind it does too.
        while let Some(front) = inner.retained.front() {
            let older_than_current = inner.retained.len() - 1;
            if older_than_current <= self.retain {
                break; // the K floor: never evicted, time travel or not
            }
            let keep_for_time_travel = self.time_travel.as_ref().is_some_and(|tt| {
                older_than_current <= tt.max_generations.max(self.retain)
                    && tt.max_age.is_none_or(|max| {
                        // A clock that stepped backwards makes the front look
                        // future-published; keep it (conservative, never lies
                        // about retention it still holds).
                        now.duration_since(front.published_at).map_or(true, |age| age <= max)
                    })
            });
            if keep_for_time_travel {
                break;
            }
            inner.retained.pop_front();
        }
        inner.registry.retain(|w| w.strong_count() > 0);
        inner.registry.push_back(Arc::downgrade(&next));
        next
    }

    /// The retained generation numbered `number`, pinned — time travel's lookup
    /// (a retained generation IS a queryable snapshot). Returns `None` when the
    /// generation is not retained: never published yet (`number` beyond current),
    /// or aged out of the retention window (evicted by the K/`max_generations`/
    /// `max_age` bounds — see [`TimeTravelConfig`]). A generation a *reader*
    /// still pins but the ring has forgotten is honestly `None` too: the ring
    /// only serves retention it owns.
    ///
    /// Takes the writer-side mutex briefly (O(1) — retained numbers are
    /// contiguous); not for the per-request hot path unless time travel was
    /// requested.
    pub fn at(&self, number: u64) -> Option<Arc<Generation<S>>> {
        let inner = self.inner.lock().expect("generation ring poisoned");
        let front = inner.retained.front()?.number();
        let idx = number.checked_sub(front)?;
        inner.retained.get(usize::try_from(idx).ok()?).cloned()
    }

    /// Resolves "as of `t`" to the generation that was current at `t`: the
    /// newest retained generation with `published_at <= t`, pinned. `None` when
    /// that generation is no longer retained (aged out) or `t` predates the
    /// ring's first retained generation — never silently substitutes a different
    /// point in time. Timestamps come from the configured clock; with the
    /// default wall clock, ties/steps resolve toward the newest qualifying
    /// generation (O(retained) scan — robust to non-monotonic clocks).
    pub fn as_of(&self, t: SystemTime) -> Option<Arc<Generation<S>>> {
        let inner = self.inner.lock().expect("generation ring poisoned");
        inner.retained.iter().rev().find(|g| g.published_at <= t).cloned()
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
