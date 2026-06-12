//! Window materialisation strategies (the R2R step's graph production):
//! how each closed window becomes the [`sparq_core::Graph`] the engine
//! evaluates. See [`EvalMode`] for the three strategies and the crate README
//! for the measured throughput of each.

use oxrdf::Term;
use rustc_hash::FxHashSet;
use sparq_core::dict::{Dict, Id};
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
    /// hashing/allocation from the window loop. The dictionary grows
    /// monotonically with the stream's distinct vocabulary (an accepted,
    /// documented trade — see the crate TODO for the compaction follow-up).
    /// The benchmark winner across every scenario; the default.
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

    pub fn push(&mut self, triple: [Term; 3], ts: u64) {
        match &mut self.m {
            Materializer::Rebuild { window } | Materializer::Delta { window, .. } => {
                window.push(triple, ts)
            }
            Materializer::PersistentDict { dict, window } => {
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
            Materializer::PersistentDict { dict, window } => {
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
