//! [OPUS-5] sq-nc3c6 — the BOUNDED, SHARDED masked-replica cache behind
//! [`crate::PodStore::scoped_dataset`] (design record
//! `research/odrl-pattern-scoped-targets-2026-07.md` §6).
//!
//! # Why this exists
//!
//! `scoped_dataset` materializes a masked replica of the session's whole accessible
//! dataset (decode → filter → rebuild, O(accessible dataset)) — the cost this design
//! deliberately pays *instead of* per-scan filtering, so masked triples are physically
//! absent and oracle equivalence is an identity rather than an audit obligation. Before
//! this module that rebuild happened on EVERY call, so a session issuing N scoped
//! queries paid it N times (the `scoped_build_ms` dimension of `bench/pattern-scope/`).
//! Caching the replica makes the build cost amortized over the queries that reuse it,
//! and lets every session in the same **scope class** (the common case: role-based
//! policies, few distinct scopes) share ONE replica instead of building its own.
//!
//! # Shape (mirrors `session_cache`, with two deliberate differences)
//!
//! - **`&self`-readable** — [`std::sync::RwLock`] per shard, so `scoped_dataset` keeps
//!   its `&self` signature and N concurrent `&PodStore` readers share replicas.
//! - **sharded** — [`SHARDS`] independent lock stripes keyed by the key hash (the
//!   design record's "scope fingerprint"), so two different scope classes almost never
//!   contend.
//! - **bounded** — at most [`SHARD_CAP`] replicas per shard, LRU-evicted. The cap is
//!   deliberately TINY compared to the session cache's: a session-cache entry is a
//!   small `Arc`'d list of graph NAMES, whereas a replica is a full masked COPY of the
//!   accessible dataset, so the honest memory bound (design record §6) is tens of
//!   replicas, not tens of thousands. Because the cap is that small, LRU is an
//!   `O(SHARD_CAP)` min-stamp scan rather than a recency queue — no queue to keep
//!   bounded, and hence none of the queue-growth failure mode `session_cache`'s
//!   `RECENCY_SLACK` compaction exists to prevent.
//!
//! # Invalidation (the soundness argument)
//!
//! A replica holds actual TRIPLES, so unlike the session cache it must be invalidated by
//! **data** writes as well as authorization changes. Every seam in this crate that takes
//! `&mut self` and mutates `PodStore::graph` calls `PodStore::bump_write_gen` first, and
//! [`ReplicaCache::get_or_build`] takes that generation and drops the WHOLE cache the
//! moment it differs from the one the cached replicas were built against. Two properties
//! make that airtight:
//!
//! 1. **No read can interleave a write.** Every write seam takes `&mut PodStore` and
//!    every read (`scoped_dataset`) takes `&PodStore`, so Rust's aliasing rules already
//!    guarantee the generation cannot change *during* a `get_or_build` call — checking it
//!    once on entry is sufficient, not merely likely.
//! 2. **Invalidation is whole-cache and conservative.** The generation is bumped BEFORE
//!    each mutation and unconditionally (even when the mutation turns out to be a no-op
//!    or fails part-way), so it can only ever over-invalidate. Over-invalidating costs a
//!    rebuild; under-invalidating would be a stale read, i.e. a masked-data correctness
//!    bug — so the bias is deliberate.
//!
//! Callers that mutate the public `PodStore::graph` field directly bypass every seam
//! (this already breaks the session index — see the field docs); they must call
//! [`crate::PodStore::invalidate_scoped_replicas`].
//!
//! # Key exactness (no fingerprint-collision leak)
//!
//! The design record describes the key as "a hashed normalized `GraphScope`". A raw hash
//! would make a 64-bit collision serve the WRONG mask — a silent LEAK. So the hash is
//! used ONLY to pick the shard: the map key is the FULL normalized decision set
//! ([`ReplicaKey`]), compared for exact equality by `HashMap` itself, so a hash collision
//! can at worst put two keys in one bucket, never confuse them. Normalization is
//! sort + dedup of each pattern list (the visibility predicate `any(allow) && !any(deny)`
//! is order- and duplicate-insensitive, so this conflates exactly the scopes that ARE
//! equal) over the canonical N-Triples form of every term (injective, so it conflates
//! nothing else).

use crate::pattern_scope::GraphScope;
use oxrdf::Term;
use rustc_hash::{FxBuildHasher, FxHashMap};
use sparq_core::Graph;
use sparq_engine::FxHashSet;
use std::hash::BuildHasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Number of independent lock stripes. A power of two so shard selection is a mask.
pub(crate) const SHARDS: usize = 8;

/// Maximum live replicas PER SHARD (whole cache bounded by `SHARDS * SHARD_CAP`). Small
/// on purpose — see the module docs: each entry is a full masked copy of the accessible
/// dataset, not a list of names.
pub(crate) const SHARD_CAP: usize = 4;

/// One masked replica: the assembled dataset (`masked_dataset` output) plus the set of
/// names it contains, in the `Arc` shape [`sparq_engine::DatasetView`] takes.
pub(crate) struct Replica {
    pub(crate) graph: Graph,
    pub(crate) named: Arc<FxHashSet<Term>>,
}

/// One scope pattern in canonical form: each component is the term's N-Triples string or
/// `None` for a wildcard. `Ord` so a pattern LIST can be sorted into a canonical order.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NormPattern(Option<String>, Option<String>, Option<String>);

/// One graph's scope in canonical form: both pattern lists sorted and deduplicated.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct NormScope {
    allow: Vec<NormPattern>,
    deny: Vec<NormPattern>,
}

/// The cache key: the COMPLETE per-graph visibility decision set `scoped_dataset`
/// derived (accessible graphs only, each with its effective scope), canonicalized and
/// sorted by graph name. It is an exact description of the replica's contents, so two
/// calls share a replica iff they would have built byte-identical ones.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ReplicaKey(Vec<(String, NormScope)>);

impl ReplicaKey {
    /// Canonicalize the decision map `scoped_dataset` assembled.
    pub(crate) fn of(decisions: &FxHashMap<Term, GraphScope>) -> ReplicaKey {
        fn norm(patterns: &[crate::ScopePattern]) -> Vec<NormPattern> {
            let mut out: Vec<NormPattern> = patterns
                .iter()
                .map(|p| {
                    let [s, p, o] = p.components();
                    let text = |t: &Option<Term>| t.as_ref().map(ToString::to_string);
                    NormPattern(text(s), text(p), text(o))
                })
                .collect();
            out.sort();
            out.dedup();
            out
        }
        let mut entries: Vec<(String, NormScope)> = decisions
            .iter()
            .map(|(name, scope)| {
                let (allow, deny) = scope.rules();
                (name.to_string(), NormScope { allow: norm(allow), deny: norm(deny) })
            })
            .collect();
        // FxHashMap iteration order is unspecified; sort so the key is canonical.
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        ReplicaKey(entries)
    }
}

/// One cached replica plus its LRU stamp. The stamp is atomic so a cache HIT can record
/// its recency under the shard's READ lock (hits never need the write lock).
struct Slot {
    replica: Arc<Replica>,
    stamp: AtomicU64,
}

/// One lock stripe: a bounded LRU map of [`ReplicaKey`] → cached replica.
struct Shard {
    map: FxHashMap<ReplicaKey, Slot>,
}

impl Shard {
    /// Evict least-recently-used entries until the map is within [`SHARD_CAP`]. An
    /// `O(SHARD_CAP)` min-stamp scan; the cap is tiny, so this is cheaper (and far
    /// simpler to keep bounded) than a recency queue.
    fn evict_to_cap(&mut self) {
        while self.map.len() > SHARD_CAP {
            let victim = self
                .map
                .iter()
                .min_by_key(|(_, slot)| slot.stamp.load(Ordering::Relaxed))
                .map(|(key, _)| key.clone());
            match victim {
                Some(key) => {
                    self.map.remove(&key);
                }
                None => break,
            }
        }
    }
}

/// The bounded, sharded, interior-mutable masked-replica cache. One per [`crate::PodStore`].
pub(crate) struct ReplicaCache {
    shards: Vec<RwLock<Shard>>,
    /// The store write generation every live entry was built against.
    generation: AtomicU64,
    /// Monotonic LRU clock (shared across shards — only the ORDER matters).
    clock: AtomicU64,
}

impl ReplicaCache {
    pub(crate) fn new() -> ReplicaCache {
        ReplicaCache {
            shards: (0..SHARDS).map(|_| RwLock::new(Shard { map: FxHashMap::default() })).collect(),
            generation: AtomicU64::new(0),
            clock: AtomicU64::new(0),
        }
    }

    /// The shard index for `key` — the design record's "scope fingerprint", used ONLY to
    /// pick a stripe (exact equality is `HashMap`'s job; see the module docs).
    fn shard_of(&self, key: &ReplicaKey) -> usize {
        (FxBuildHasher.hash_one(key) as usize) & (SHARDS - 1)
    }

    fn tick(&self) -> u64 {
        self.clock.fetch_add(1, Ordering::Relaxed)
    }

    /// Drop every cached replica across every shard.
    pub(crate) fn clear(&self) {
        for shard in &self.shards {
            shard.write().expect("replica-cache shard poisoned").map.clear();
        }
    }

    /// Drop everything if the store has been written since the live entries were built.
    /// Racing readers may both clear (idempotent) or one may clear just after the other
    /// filled — which only costs a rebuild. Clearing can never surface a stale replica.
    fn sync_generation(&self, generation: u64) {
        if self.generation.load(Ordering::Acquire) == generation {
            return;
        }
        self.clear();
        self.generation.store(generation, Ordering::Release);
    }

    /// The replica for `key` at store write-generation `generation`, building it with
    /// `build` on a miss.
    ///
    /// `build` runs OUTSIDE every lock: it is the O(accessible dataset) decode → filter →
    /// rebuild, and holding a shard's write lock across it would serialize unrelated
    /// misses that happened to hash to the same stripe. Two threads racing on the same
    /// key may therefore both build; the first to reach the write lock wins and the other
    /// drops its (by-construction identical) copy.
    pub(crate) fn get_or_build<F>(&self, generation: u64, key: ReplicaKey, build: F) -> Arc<Replica>
    where
        F: FnOnce() -> Replica,
    {
        self.sync_generation(generation);
        let idx = self.shard_of(&key);
        // Fast path: shared read lock, clone the `Arc` if warm.
        {
            let shard = self.shards[idx].read().expect("replica-cache shard poisoned");
            if let Some(slot) = shard.map.get(&key) {
                slot.stamp.store(self.tick(), Ordering::Relaxed);
                return Arc::clone(&slot.replica);
            }
        }
        let replica = Arc::new(build());
        let mut shard = self.shards[idx].write().expect("replica-cache shard poisoned");
        if let Some(slot) = shard.map.get(&key) {
            // A concurrent builder won the race — keep its replica so both callers share.
            slot.stamp.store(self.tick(), Ordering::Relaxed);
            return Arc::clone(&slot.replica);
        }
        let stamp = AtomicU64::new(self.tick());
        shard.map.insert(key, Slot { replica: Arc::clone(&replica), stamp });
        shard.evict_to_cap();
        replica
    }

    /// Total live replicas across all shards (test/inspection only).
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().expect("poisoned").map.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    //! [OPUS-5] sq-nc3c6 — DIRECT unit tests for the cache mechanics, independent of the
    //! WAC/ODRL path: build-once/hit, generation invalidation, bounded eviction, and the
    //! key-normalization equalities (and, load-bearing, the NON-equalities).
    use super::*;
    use crate::{GraphScope, ScopePattern};
    use oxrdf::NamedNode;
    use sparq_core::dict::Dict;
    use std::sync::atomic::AtomicUsize;

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    fn empty_replica() -> Replica {
        Replica {
            graph: Graph::from_parts(Dict::new(), Vec::new()),
            named: Arc::new(FxHashSet::default()),
        }
    }

    fn key_for(pairs: &[(&str, GraphScope)]) -> ReplicaKey {
        let mut decisions = FxHashMap::default();
        for (name, scope) in pairs {
            decisions.insert(iri(name), scope.clone());
        }
        ReplicaKey::of(&decisions)
    }

    #[test]
    fn build_runs_once_then_hits_are_warm() {
        let cache = ReplicaCache::new();
        let key = key_for(&[("https://pod.ex/g", GraphScope::deny_within(Vec::new()))]);
        let builds = AtomicUsize::new(0);
        let build = || {
            builds.fetch_add(1, Ordering::SeqCst);
            empty_replica()
        };
        let a = cache.get_or_build(0, key.clone(), build);
        let b = cache.get_or_build(0, key, build);
        assert_eq!(builds.load(Ordering::SeqCst), 1, "build ran exactly once");
        assert!(Arc::ptr_eq(&a, &b), "the same replica is shared, not rebuilt");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn a_generation_bump_invalidates_everything() {
        let cache = ReplicaCache::new();
        let key = key_for(&[("https://pod.ex/g", GraphScope::deny_within(Vec::new()))]);
        let a = cache.get_or_build(0, key.clone(), empty_replica);
        // Same generation -> hit.
        let b = cache.get_or_build(0, key.clone(), empty_replica);
        assert!(Arc::ptr_eq(&a, &b));
        // Bumped generation -> the whole cache is dropped and the replica rebuilt.
        let c = cache.get_or_build(1, key, empty_replica);
        assert!(!Arc::ptr_eq(&a, &c), "a write generation bump must force a rebuild");
        assert_eq!(cache.len(), 1, "the cache holds only the fresh generation's replica");
    }

    #[test]
    fn bounded_eviction_caps_each_shard() {
        let cache = ReplicaCache::new();
        // Insert far more distinct scope classes than the whole cache can hold.
        for n in 0..(SHARDS * SHARD_CAP * 4) {
            let scope = GraphScope::allow_only(vec![ScopePattern::new(
                None,
                Some(iri(&format!("https://ex.dev/ns#p{n}"))),
                None,
            )]);
            cache.get_or_build(0, key_for(&[("https://pod.ex/g", scope)]), empty_replica);
        }
        assert!(cache.len() <= SHARDS * SHARD_CAP, "whole cache bounded ({})", cache.len());
        for shard in &cache.shards {
            let len = shard.read().unwrap().map.len();
            assert!(len <= SHARD_CAP, "shard bounded ({len} <= {SHARD_CAP})");
        }
    }

    #[test]
    fn key_normalization_conflates_reordering_and_duplicates_only() {
        let p = ScopePattern::new(None, Some(iri("https://ex.dev/ns#phone")), None);
        let q = ScopePattern::new(None, Some(iri("https://ex.dev/ns#name")), None);
        let g = "https://pod.ex/g";
        let a = key_for(&[(g, GraphScope::deny_within(vec![p.clone(), q.clone()]))]);
        let b = key_for(&[(g, GraphScope::deny_within(vec![q.clone(), p.clone()]))]);
        let c = key_for(&[(
            "https://pod.ex/g",
            GraphScope::deny_within(vec![p.clone(), q.clone(), p.clone()]),
        )]);
        assert_eq!(a, b, "pattern ORDER is not semantically significant");
        assert_eq!(a, c, "DUPLICATE patterns are not semantically significant");

        // …and nothing else is conflated: a different pattern set, the allow/deny side,
        // the graph name, and the accessible-graph set all key distinctly.
        assert_ne!(a, key_for(&[("https://pod.ex/g", GraphScope::deny_within(vec![p.clone()]))]));
        assert_ne!(a, key_for(&[("https://pod.ex/g", GraphScope::allow_only(vec![p.clone(), q]))]));
        assert_ne!(a, key_for(&[("https://pod.ex/other", GraphScope::deny_within(vec![p]))]));
        assert_ne!(
            a,
            key_for(&[
                ("https://pod.ex/g", GraphScope::deny_within(Vec::new())),
                ("https://pod.ex/h", GraphScope::deny_within(Vec::new())),
            ])
        );
    }

    #[test]
    fn key_distinguishes_a_wildcard_from_a_concrete_term() {
        // Regression guard for the canonical-form encoding: `None` (wildcard) and a
        // concrete term must never normalize to the same component.
        let g = "https://pod.ex/g";
        let wild = key_for(&[(g, GraphScope::deny_within(vec![ScopePattern::any()]))]);
        let x = ScopePattern::new(None, None, Some(iri("https://ex.dev/ns#x")));
        let concrete = key_for(&[(g, GraphScope::deny_within(vec![x]))]);
        assert_ne!(wild, concrete);
    }

    #[test]
    fn shard_selection_is_stable_and_in_range() {
        let cache = ReplicaCache::new();
        let key = key_for(&[("https://pod.ex/g", GraphScope::deny_within(Vec::new()))]);
        assert_eq!(cache.shard_of(&key), cache.shard_of(&key), "deterministic");
        assert!(cache.shard_of(&key) < SHARDS, "in range");
    }
}
