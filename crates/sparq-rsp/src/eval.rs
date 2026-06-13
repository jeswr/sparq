//! Window materialisation strategies (the R2R step's graph production):
//! how each closed window becomes the [`sparq_core::Graph`] the engine
//! evaluates. See [`EvalMode`] for the three strategies and the crate README
//! for the measured throughput of each.

use oxrdf::Term;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{is_inline, Dict, Id};
use sparq_core::Graph;

use crate::window::{Window, WindowSpec, WindowedStream};

/// How each closed window's graph is materialised for evaluation.
///
/// All three modes produce IDENTICAL query results (pinned by tests); they
/// differ only in how much per-window work is redone. Numbers below are the
/// crate benchmark (`examples/throughput.rs`, see README).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvalMode {
    /// v1 baseline: every closed window interns its terms into a FRESH
    /// dictionary and builds fresh indexes. Honest, allocation-heavy; kept as
    /// the benchmark baseline and for streams with unbounded vocabulary
    /// (nothing persists between windows, so memory is bounded by one window).
    Rebuild,
    /// ONE dictionary for the continuous query's lifetime: every term is
    /// interned once at push time and each closed window builds its indexes
    /// from already-interned `[Id; 3]`s (`Graph::from_parts`), removing term
    /// hashing/allocation from the window loop. The benchmark winner across
    /// every scenario; the default.
    ///
    /// [OPUS-4.8] The dictionary no longer grows monotonically with the
    /// stream's vocabulary. Liveness is exact: a term is reachable iff some
    /// live window (a buffered/open window, the count ring, or a closed window
    /// not yet taken) still references it; terms that have aged out of every
    /// window are unreachable, so their dictionary entries are reclaimable. A
    /// bounded amount of post-eviction churn triggers a COMPACTION pass that
    /// rebuilds the dictionary from only the still-live terms and atomically
    /// remaps the live window indexes onto the fresh dense ids (see
    /// [`WindowEval::compact_dict`]). Bound: the dictionary stays O(distinct
    /// terms across the live windows), not O(distinct terms over all time. So a
    /// long-running query over an unbounded vocabulary holds memory proportional
    /// to its window content, not its history.
    #[default]
    PersistentDict,
    /// ONE live graph maintained by `Graph::apply_delta(inserts, deletes)` per
    /// slide (the set-semantic diff between consecutive windows), compacted
    /// when the pending overlay outgrows the live window. Measured SLOWER than
    /// `PersistentDict` in every benchmark scenario (the per-slide work is
    /// term-level — interning inserts, `id_of` lookups for deletes — and
    /// overlay rows are re-sorted per scan); kept as an option because its
    /// per-slide cost is O(changes), so it can win when windows are huge and
    /// the engine evaluation itself is cheap relative to an index rebuild.
    /// Like `PersistentDict`, the live graph's dictionary grows monotonically.
    Delta,
}

/// Compact the delta-mode overlay when the cumulative churn since the last
/// compaction exceeds the live graph size (clamped: tiny windows shouldn't
/// compact on every slide).
const MIN_COMPACT_CHURN: usize = 256;

/// [OPUS-4.8] Persistent-dictionary compaction trigger. After each batch of
/// closed windows, the dictionary holds `live + dead` distinct terms, where
/// `live` are still referenced by some window and `dead` have aged out. A
/// compaction rebuild is O(live + dead) (it walks the live payloads and
/// reconstructs the survivors), so triggering it on EVERY slide would add a
/// full-dictionary pass per window — defeating the point of the persistent
/// dictionary. Instead we let the dead set grow and compact only when it is
/// both ABSOLUTELY large enough to matter and a SUBSTANTIAL fraction of the
/// dictionary, so the amortised compaction cost per interned term is O(1) and
/// the steady-state dictionary size is bounded by `live · (1 + DEAD_RATIO)`.
///
/// `MIN_DEAD` keeps small/low-churn streams from ever compacting (the rebuild
/// would cost more than the few bytes it reclaims); `DEAD_RATIO` makes the
/// reclaimed-fraction worthwhile when we do pay it.
const MIN_DEAD_TERMS: usize = 1024;
/// Compact once the estimated dead terms reach this fraction of the live set.
const DEAD_RATIO_NUM: usize = 1;
const DEAD_RATIO_DEN: usize = 2;

enum Materializer {
    Rebuild {
        window: WindowedStream<[Term; 3]>,
    },
    PersistentDict {
        /// `None` only transiently while the dictionary is lent to the
        /// per-window graph (`Graph::from_parts` takes it by value; it is
        /// moved back out of `graph.dict` after evaluation).
        dict: Option<Dict>,
        window: WindowedStream<[Id; 3]>,
        /// [OPUS-4.8] `dict.len()` (distinct non-inline terms) at the last
        /// compaction. The cheap compaction trigger: once `dict.len()` has
        /// grown by at least [`MIN_DEAD_TERMS`] beyond this, enough terms have
        /// been interned since the last rebuild that a liveness scan is worth
        /// running (and `compact_dict` then resets this to the post-rebuild
        /// length). Scanning only every `MIN_DEAD_TERMS` interns keeps the
        /// amortised compaction cost per interned term O(1).
        dict_len_at_compact: usize,
    },
    Delta {
        window: WindowedStream<[Term; 3]>,
        /// Boxed: a `Graph` (dictionary + indexes) is much larger than the
        /// other variants' state.
        graph: Box<Graph>,
        /// The previous window's content as a deduplicated triple set (the
        /// diff base for the next slide).
        prev: Vec<[Term; 3]>,
        /// Inserts + deletes applied since the last compaction.
        churn: usize,
    },
}

/// One windowed stream + one [`EvalMode`]: pushes elements, and surfaces every
/// closed window as a `(start, end, &Graph)` callback.
pub(crate) struct WindowEval {
    mode: EvalMode,
    m: Materializer,
}

impl WindowEval {
    pub fn new(spec: WindowSpec, mode: EvalMode) -> Self {
        let m = match mode {
            EvalMode::Rebuild => Materializer::Rebuild { window: WindowedStream::empty(spec) },
            EvalMode::PersistentDict => Materializer::PersistentDict {
                dict: Some(Dict::new()),
                window: WindowedStream::empty(spec),
                dict_len_at_compact: 0,
            },
            EvalMode::Delta => Materializer::Delta {
                window: WindowedStream::empty(spec),
                graph: Box::new(Graph::from_parts(Dict::new(), Vec::new())),
                prev: Vec::new(),
                churn: 0,
            },
        };
        WindowEval { mode, m }
    }

    pub fn mode(&self) -> EvalMode {
        self.mode
    }

    pub fn spec(&self) -> WindowSpec {
        match &self.m {
            Materializer::Rebuild { window } | Materializer::Delta { window, .. } => window.spec(),
            Materializer::PersistentDict { window, .. } => window.spec(),
        }
    }

    pub fn late_dropped(&self) -> u64 {
        match &self.m {
            Materializer::Rebuild { window } | Materializer::Delta { window, .. } => {
                window.late_dropped()
            }
            Materializer::PersistentDict { window, .. } => window.late_dropped(),
        }
    }

    /// [OPUS-4.8] The number of distinct non-inline terms currently held by the
    /// persistent dictionary, or `None` for the modes without a lifetime
    /// dictionary ([`EvalMode::Rebuild`] builds and drops one per window;
    /// [`EvalMode::Delta`] keeps the live graph's dictionary, not exposed here).
    /// An observability hook: this is the quantity that compaction bounds, so an
    /// embedder running a long continuous query can watch it stay bounded
    /// instead of growing with the all-time vocabulary.
    pub fn dict_len(&self) -> Option<usize> {
        match &self.m {
            Materializer::PersistentDict { dict, .. } => {
                Some(dict.as_ref().expect("dict restored between evaluations").len())
            }
            _ => None,
        }
    }

    pub fn push(&mut self, triple: [Term; 3], ts: u64) {
        match &mut self.m {
            Materializer::Rebuild { window } | Materializer::Delta { window, .. } => {
                window.push(triple, ts)
            }
            Materializer::PersistentDict { dict, window, .. } => {
                let d = dict.as_mut().expect("dict is always restored after evaluation");
                let ids = [d.intern(&triple[0]), d.intern(&triple[1]), d.intern(&triple[2])];
                window.push(ids, ts);
            }
        }
    }

    /// Materialises and evaluates every window that closed since the last
    /// call (all remaining windows when `flush` is true), oldest first. On an
    /// `Err` from `f`, remaining closed windows are dropped.
    pub fn eval_closed(
        &mut self,
        flush: bool,
        f: &mut impl FnMut(u64, u64, &Graph) -> Result<(), String>,
    ) -> Result<(), String> {
        match &mut self.m {
            Materializer::Rebuild { window } => {
                for w in take(window, flush) {
                    let graph = materialize(&w);
                    f(w.start, w.end, &graph)?;
                }
            }
            Materializer::PersistentDict { dict, window, dict_len_at_compact } => {
                for w in take(window, flush) {
                    let ids: Vec<[Id; 3]> = w.triples.iter().map(|t| t.triple).collect();
                    // Lend the persistent dictionary to the per-window graph;
                    // `Graph::from_parts` dedups (RDF set semantics) and builds
                    // the indexes from the already-interned ids — no term
                    // hashing or allocation here.
                    let d = dict.take().expect("dict is always restored after evaluation");
                    let graph = Graph::from_parts(d, ids);
                    let r = f(w.start, w.end, &graph);
                    *dict = Some(graph.dict);
                    r?;
                }
                // [OPUS-4.8] Reclaim the dictionary entries of terms that have
                // aged out of every live window. Safe HERE: every closed window
                // for this call has been evaluated and dropped (the `take`
                // vector is gone), no `Graph` borrows the dictionary, and the
                // only remaining references are the live window payloads —
                // which `compact_dict` remaps atomically.
                let len = dict.as_ref().expect("dict restored above").len();
                if len >= dict_len_at_compact.saturating_add(MIN_DEAD_TERMS) {
                    Self::compact_dict(dict, window, dict_len_at_compact);
                }
            }
            Materializer::Delta { window, graph, prev, churn } => {
                for w in take(window, flush) {
                    // Set-semantic diff against the previous window: a triple
                    // present at several timestamps counts once, so inserts /
                    // deletes are computed over the DISTINCT triple sets.
                    let cur: FxHashSet<&[Term; 3]> = w.triples.iter().map(|t| &t.triple).collect();
                    let old: FxHashSet<&[Term; 3]> = prev.iter().collect();
                    let inserts: Vec<[Term; 3]> =
                        cur.iter().filter(|t| !old.contains(**t)).map(|t| (*t).clone()).collect();
                    let deletes: Vec<[Term; 3]> =
                        old.iter().filter(|t| !cur.contains(**t)).map(|t| (*t).clone()).collect();
                    let next_prev: Vec<[Term; 3]> = cur.iter().map(|t| (*t).clone()).collect();
                    drop(cur);
                    drop(old);
                    *prev = next_prev;
                    *churn += inserts.len() + deletes.len();
                    graph.apply_delta(&inserts, &deletes)?;
                    // Fold the overlay back into the base before it outgrows
                    // the live window (overlay rows are re-sorted per scan).
                    if *churn >= graph.len().max(MIN_COMPACT_CHURN) {
                        graph.compact()?;
                        *churn = 0;
                    }
                    f(w.start, w.end, graph)?;
                }
            }
        }
        Ok(())
    }

    /// [OPUS-4.8] Persistent-dictionary compaction: rebuild the lifetime
    /// dictionary from ONLY the terms still referenced by a live window, and
    /// remap the live window payloads onto the fresh dense ids — bounding the
    /// dictionary at the live vocabulary instead of the all-time vocabulary.
    ///
    /// Correctness contract:
    /// * **No dangling ids.** Every live payload is remapped through `old → new`
    ///   in [`WindowedStream::remap_live_payloads`]; the remap covers exactly
    ///   the ids gathered by [`WindowedStream::for_each_live_payload`], so after
    ///   the swap no live id points into the freed dictionary.
    /// * **A still-referenced term stays resolvable.** Every live id is
    ///   re-interned (its term reconstructed from the old dict, interned into
    ///   the new one), so its value survives; only ids referenced by NO live
    ///   window are dropped.
    /// * **Results are unchanged.** The remap is a bijection on live ids, so
    ///   per-window triple sets, membership, ordering and counts are identical;
    ///   only the integer encoding changes. (Inline-integer ids carry their
    ///   value in the id itself and are never dictionary entries, so they map to
    ///   themselves and need no rebuild.)
    /// * **Atomic.** The new dict + remap are built fully before any live
    ///   payload is rewritten; the rebuild cannot fail (it only reconstructs and
    ///   re-interns terms already present), so there is no partial state.
    ///
    /// Skips the rebuild when nothing would be reclaimed (dead set below
    /// [`MIN_DEAD_TERMS`] or below the [`DEAD_RATIO_NUM`]/[`DEAD_RATIO_DEN`]
    /// fraction of the live set), but always advances `dict_len_at_compact` so
    /// the next liveness scan is again `MIN_DEAD_TERMS` interns away.
    fn compact_dict(
        dict: &mut Option<Dict>,
        window: &mut WindowedStream<[Id; 3]>,
        dict_len_at_compact: &mut usize,
    ) {
        let old = dict.as_ref().expect("dict present when compacting");

        // Gather the DISTINCT live (non-inline) ids referenced by any window.
        let mut live: FxHashSet<Id> = FxHashSet::default();
        window.for_each_live_payload(|ids| {
            for &id in ids {
                if !is_inline(id) {
                    live.insert(id);
                }
            }
        });

        // Decide whether reclaiming is worth a full rebuild. `dead` is the
        // number of dictionary entries no live window references any more.
        let total = old.len();
        let dead = total.saturating_sub(live.len());
        let worth_it = dead >= MIN_DEAD_TERMS
            && dead * DEAD_RATIO_DEN >= live.len().max(1) * DEAD_RATIO_NUM;
        if !worth_it {
            // Defer: record the current size so the next scan is MIN_DEAD_TERMS
            // interns away (otherwise a once-large-but-now-mostly-live dict
            // would re-scan on every batch).
            *dict_len_at_compact = total;
            return;
        }

        // Rebuild: a fresh dictionary holding only the survivors, plus the
        // old→new id remap. Re-intern via the reconstructed term so the new
        // dict's prefix/datatype tables and value caches are rebuilt correctly.
        let mut fresh = Dict::with_capacity(live.len());
        let mut remap: FxHashMap<Id, Id> = FxHashMap::with_capacity_and_hasher(live.len(), Default::default());
        for &id in &live {
            let new_id = fresh.intern(&old.term(id));
            remap.insert(id, new_id);
        }

        // Atomically swing every live payload onto the fresh ids. Inline ids
        // pass through unchanged (they are not in `remap`); every non-inline
        // live id is present in `remap` by construction.
        window.remap_live_payloads(|ids| {
            [
                remap_id(ids[0], &remap),
                remap_id(ids[1], &remap),
                remap_id(ids[2], &remap),
            ]
        });

        *dict = Some(fresh);
        *dict_len_at_compact = live.len();
    }
}

/// Maps one id through the compaction remap: inline-integer ids (and only
/// those) are absent from the remap and carry their value in the id, so they
/// map to themselves; every live dictionary id is present by construction.
#[inline]
fn remap_id(id: Id, remap: &FxHashMap<Id, Id>) -> Id {
    if is_inline(id) {
        id
    } else {
        *remap.get(&id).expect("every live non-inline id is in the compaction remap")
    }
}

fn take<T: Clone>(window: &mut WindowedStream<T>, flush: bool) -> Vec<Window<T>> {
    if flush {
        window.flush()
    } else {
        window.take_closed()
    }
}

/// Materialises one window's content into a fresh dictionary-encoded graph
/// (the [`EvalMode::Rebuild`] baseline). RDF graphs are sets: a triple present
/// at several timestamps in the window appears once.
fn materialize(w: &Window<[Term; 3]>) -> Graph {
    let mut dict = Dict::new();
    let ids: Vec<[Id; 3]> = w
        .triples
        .iter()
        .map(|t| [dict.intern(&t.triple[0]), dict.intern(&t.triple[1]), dict.intern(&t.triple[2])])
        .collect();
    Graph::from_parts(dict, ids)
}
