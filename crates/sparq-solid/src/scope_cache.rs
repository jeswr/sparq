//! [SONNET-4.6] sq-nc3c6 (design record `research/odrl-pattern-scoped-targets-2026-07.md`
//! §6) — the BOUNDED, SHARDED masked-replica cache behind the opt-in `pattern-scope`
//! feature.
//!
//! # Why this exists
//!
//! [`crate::PodStore::scoped_dataset`] materializes a masked replica of every scoped
//! accessible graph. That build is the cost the design pays INSTEAD of per-scan
//! filtering (the oracle-equivalence argument in [`crate::pattern_scope`]) — but paying
//! it on EVERY call means a session that issues N queries against one scope pays it N
//! times, which is what `bench/pattern-scope/` measures. A masked replica is a **pure
//! function of (source graph contents, scope)**, so it can be built once and reused by
//! every session whose policy resolves to the same scope class — the common case, since
//! role-based policies produce few distinct scopes.
//!
//! The cache is shaped after the `session_cache` precedent (it does NOT reuse that
//! module: a session entry carries per-origin surgery + memo state this one has no use
//! for, and this one's values are immutable once built, so the eviction policies differ
//! — see `Shard::insert`):
//!
//! - **`&self`-readable** — interior mutability ([`RwLock`] per shard), because
//!   `scoped_dataset` takes `&self` and `&PodStore: Sync` (concurrent readers).
//! - **sharded** — [`SHARDS`] lock stripes keyed by the hashed key, so two requests for
//!   different (graph, scope) pairs almost never contend.
//! - **bounded** — at most [`SHARD_CAP`] replicas per shard, evicted in insertion order
//!   (see `Shard::insert`), so a caller that keeps minting fresh scope classes cannot
//!   grow the cache without bound.
//!
//! # Exact-semantics preservation
//!
//! - **No hash-collision fail-open.** The map key carries the FULL normalized
//!   [`GraphScope`], not just its hash — the design record's "scope fingerprint" is used
//!   only to pick the shard. A 64-bit collision therefore lands two DIFFERENT scopes in
//!   the same stripe, where `Eq` keeps them apart; it can never serve one scope's replica
//!   for another's mask.
//! - **Invalidation is the write-path seam.** A replica depends on the source graph's
//!   CONTENT, so it is dropped wholesale by [`ReplicaCache::clear`] at the two seams that
//!   can change content: `reindex_with` (every `materialize_*` / `put_acl` / `delete_acl`
//!   / bridge / trust path routes through it) and the in-place data write in
//!   `update_inner`. Staleness here is a FRESHNESS bug, not a mask bypass — a stale
//!   replica holds only triples that were in the source and passed the same scope — but a
//!   redaction performed by DELETING data would be defeated by it, so it is invalidated
//!   unconditionally rather than diffed.
//! - Mutating [`crate::PodStore::graph`] directly bypasses both seams and is therefore
//!   unsupported, exactly as it already is for the session cache and the ACL index.

use crate::pattern_scope::GraphScope;
use oxrdf::Term;
use rustc_hash::{FxBuildHasher, FxHashMap};
use sparq_core::Graph;
use std::collections::VecDeque;
use std::hash::BuildHasher;
use std::sync::RwLock;

/// Number of independent lock stripes. A power of two so shard selection is a mask.
pub(crate) const SHARDS: usize = 16;

/// Maximum live replicas PER SHARD, so the whole cache holds at most
/// `SHARDS * SHARD_CAP` masked replicas — the honest memory bound of design record §6
/// ("one replica per (graph × scope class)") made finite. A deployment with more live
/// (graph × scope class) pairs than the cap still answers correctly; it just rebuilds
/// the evicted ones, exactly as if the cache were absent.
///
/// Sized against the working set, not plucked: one `scoped_dataset` call touches every
/// SCOPED accessible graph in one pass, so a cap below that count makes each pass evict
/// what the next pass needs and the cache degenerates to a pure miss stream. Measured
/// on the `wac_fixture_sized` bench fixture (`bench/pattern-scope/`, work-box and
/// therefore NON-canonical), a single session's accessible set is in the high hundreds
/// of graphs, and at `SHARDS * SHARD_CAP` = 4096 the warm build is a small fraction of
/// the cold one where an under-sized cap showed no amortization at all. Only genuinely
/// masked graphs land here — an unmasked one is forked, never replicated (see
/// `GraphScope::masks_nothing`) — so this bound is not a bound on the whole store.
pub(crate) const SHARD_CAP: usize = 256;

/// The cache key: the source graph's name plus the **normalized** scope
/// ([`GraphScope::normalized`]). Carrying the whole scope — not a fingerprint — is what
/// makes a hash collision harmless (see the module docs).
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ReplicaKey {
    graph: Term,
    scope: GraphScope,
}

/// One lock stripe: a bounded map of key → masked replica.
struct Shard {
    map: FxHashMap<ReplicaKey, Graph>,
    /// Insertion order, front = oldest. A key is pushed exactly once (an entry is only
    /// ever inserted on a verified miss, and never re-touched), so this holds exactly one
    /// record per live entry and is bounded by `map.len()` — no compaction pass needed.
    order: VecDeque<ReplicaKey>,
}

impl Shard {
    fn new() -> Shard {
        Shard { map: FxHashMap::default(), order: VecDeque::new() }
    }

    /// Insert a freshly-built replica and evict oldest-first until within [`SHARD_CAP`].
    ///
    /// Eviction is INSERTION-order, not true LRU: a cache hit takes only the shard's READ
    /// lock (that is the whole point — concurrent `&PodStore` readers must not serialise),
    /// so a hit cannot record recency without giving that up. Evicting a still-hot replica
    /// costs one rebuild and can never affect correctness — a replica is a pure function of
    /// (source graph, scope) and is re-derived on the next call.
    fn insert(&mut self, key: ReplicaKey, replica: Graph) {
        self.order.push_back(key.clone());
        self.map.insert(key, replica);
        while self.map.len() > SHARD_CAP {
            let Some(oldest) = self.order.pop_front() else { break };
            self.map.remove(&oldest);
        }
    }
}

/// The bounded, sharded, interior-mutable masked-replica cache. One per
/// [`crate::PodStore`], present only under the `pattern-scope` feature.
pub(crate) struct ReplicaCache {
    shards: Vec<RwLock<Shard>>,
}

impl ReplicaCache {
    pub(crate) fn new() -> ReplicaCache {
        ReplicaCache { shards: (0..SHARDS).map(|_| RwLock::new(Shard::new())).collect() }
    }

    /// The shard index for `key` — the design record's "scope fingerprint", used ONLY to
    /// pick a stripe (mask, not modulo: [`SHARDS`] is a power of two).
    fn shard_of(&self, key: &ReplicaKey) -> usize {
        (FxBuildHasher.hash_one(key) as usize) & (SHARDS - 1)
    }

    /// The masked replica of `graph` under `scope`, built via `build` on a miss.
    ///
    /// The returned [`Graph`] is a [`Graph::fork`] of the cached replica — a logically
    /// independent copy that `Arc`-shares the replica's immutable storage, so a HIT costs
    /// a shard read-lock plus a handful of refcount bumps instead of the O(source graph)
    /// decode → filter → rebuild that `build` performs.
    ///
    /// `build` runs OUTSIDE the lock, so a slow rebuild never blocks readers of other
    /// keys in the same stripe; a racing builder is resolved by re-checking under the
    /// write lock (the loser drops its own build and takes the winner's replica, so the
    /// cache never holds two replicas for one key).
    pub(crate) fn get_or_build(
        &self,
        graph: &Term,
        scope: &GraphScope,
        build: impl FnOnce() -> Graph,
    ) -> Graph {
        let key = ReplicaKey { graph: graph.clone(), scope: scope.normalized() };
        let idx = self.shard_of(&key);
        {
            let shard = self.shards[idx].read().expect("replica-cache shard poisoned");
            if let Some(replica) = shard.map.get(&key) {
                return replica.fork();
            }
        }
        let built = build();
        let mut shard = self.shards[idx].write().expect("replica-cache shard poisoned");
        if let Some(replica) = shard.map.get(&key) {
            return replica.fork(); // a concurrent builder won the race
        }
        let handed_out = built.fork();
        shard.insert(key, built);
        handed_out
    }

    /// Drop every cached replica across every shard — the write-path invalidation (see
    /// the module docs for the two seams that call it).
    pub(crate) fn clear(&self) {
        for shard in &self.shards {
            let mut s = shard.write().expect("replica-cache shard poisoned");
            s.map.clear();
            s.order.clear();
        }
    }

    /// Total cached replicas across all shards (test/inspection only).
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().expect("poisoned").map.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    //! [SONNET-4.6] sq-nc3c6 — DIRECT unit tests for the cache mechanics, independent of
    //! the WAC/ACP auth path: build-once/hit, normalization sharing, key separation,
    //! bounded eviction, and `clear`.
    use super::*;
    use crate::pattern_scope::ScopePattern;
    use oxrdf::NamedNode;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    fn replica(nt: &str) -> Graph {
        Graph::load_dataset(nt, "ntriples").unwrap()
    }

    fn one_triple() -> Graph {
        replica("<http://ex/s> <http://ex/p> <http://ex/o> .")
    }

    /// A scope naming `preds` as deny patterns, in the given order.
    fn deny(preds: &[&str]) -> GraphScope {
        GraphScope::deny_within(
            preds.iter().map(|p| ScopePattern::new(None, Some(iri(p)), None)).collect(),
        )
    }

    #[test]
    fn build_runs_once_then_hits_are_warm() {
        let cache = ReplicaCache::new();
        let calls = AtomicUsize::new(0);
        let build = || {
            calls.fetch_add(1, Ordering::SeqCst);
            one_triple()
        };
        let a = cache.get_or_build(&iri("http://ex/g"), &deny(&["http://ex/x"]), build);
        let b = cache.get_or_build(&iri("http://ex/g"), &deny(&["http://ex/x"]), build);
        assert_eq!(calls.load(Ordering::SeqCst), 1, "build ran exactly once");
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1, "the fork carries the cached replica's triples");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn normalized_scopes_share_one_replica_regardless_of_order() {
        // The SAME mask written in the opposite pattern order must hit, not rebuild —
        // that is what `GraphScope::normalized` buys (design record §6).
        let cache = ReplicaCache::new();
        let calls = AtomicUsize::new(0);
        let build = || {
            calls.fetch_add(1, Ordering::SeqCst);
            one_triple()
        };
        cache.get_or_build(&iri("http://ex/g"), &deny(&["http://ex/a", "http://ex/b"]), build);
        cache.get_or_build(&iri("http://ex/g"), &deny(&["http://ex/b", "http://ex/a"]), build);
        // …and a duplicated pattern describes the same mask too.
        cache.get_or_build(
            &iri("http://ex/g"),
            &deny(&["http://ex/b", "http://ex/a", "http://ex/b"]),
            build,
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1, "one replica for one scope class");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn distinct_graphs_and_distinct_scopes_cache_independently() {
        let cache = ReplicaCache::new();
        let build = one_triple;
        cache.get_or_build(&iri("http://ex/g1"), &deny(&["http://ex/x"]), build);
        cache.get_or_build(&iri("http://ex/g2"), &deny(&["http://ex/x"]), build);
        cache.get_or_build(&iri("http://ex/g1"), &deny(&["http://ex/y"]), build);
        assert_eq!(cache.len(), 3, "keyed by BOTH the graph name and the scope");
    }

    #[test]
    fn bounded_eviction_caps_each_shard() {
        // Mint far more scope classes than the whole cache can hold; every shard must
        // stay within SHARD_CAP and the total within SHARDS * SHARD_CAP.
        let cache = ReplicaCache::new();
        for n in 0..(SHARDS * SHARD_CAP * 2) {
            cache.get_or_build(&iri("http://ex/g"), &deny(&[&format!("http://ex/p{}", n)]), one_triple);
        }
        let mut evicting = 0;
        for (i, shard) in cache.shards.iter().enumerate() {
            let len = shard.read().unwrap().map.len();
            assert!(len <= SHARD_CAP, "shard {} bounded ({} <= {})", i, len, SHARD_CAP);
            assert_eq!(len, shard.read().unwrap().order.len(), "order record per live entry");
            if len == SHARD_CAP {
                evicting += 1;
            }
        }
        assert!(cache.len() <= SHARDS * SHARD_CAP, "whole cache bounded");
        // Non-vacuity: at least one shard actually hit the cap, so eviction really ran.
        assert!(evicting > 0, "no shard reached the cap — the bound was never exercised");
    }

    #[test]
    fn clear_drops_every_replica() {
        let cache = ReplicaCache::new();
        for n in 0..8 {
            cache.get_or_build(&iri("http://ex/g"), &deny(&[&format!("http://ex/p{}", n)]), one_triple);
        }
        assert!(cache.len() >= 1);
        cache.clear();
        assert_eq!(cache.len(), 0, "clear drops all shards");
        // …and the next call rebuilds rather than serving a dropped entry.
        let calls = AtomicUsize::new(0);
        cache.get_or_build(&iri("http://ex/g"), &deny(&["http://ex/p0"]), || {
            calls.fetch_add(1, Ordering::SeqCst);
            one_triple()
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1, "rebuilt after clear");
    }

    #[test]
    fn shard_selection_is_stable_and_in_range() {
        let cache = ReplicaCache::new();
        let key = ReplicaKey { graph: iri("http://ex/g"), scope: deny(&["http://ex/x"]).normalized() };
        assert_eq!(cache.shard_of(&key), cache.shard_of(&key), "deterministic");
        assert!(cache.shard_of(&key) < SHARDS, "in range");
    }
}
