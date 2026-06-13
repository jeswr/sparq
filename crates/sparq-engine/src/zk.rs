//! zk-trace seam (plan §4.E, stage 2 — SKELETON).
//!
//! Records, for an executed SELECT/ASK, the **per-obligation input sets** the
//! ZK witness builder needs: for each BGP triple pattern the matched triples
//! (materialized), the executed scan/join order, and FILTER obligations with
//! their per-row operand bindings. The consumer (`sparq-zk`) resolves the
//! recorded triples to leaf indices in the per-graph commitment orderings and
//! applies the Q6 cross-graph bnode-join guard.
//!
//! Pattern: exactly the EXPLAIN-ANALYZE operator trace's (`exec::trace`) —
//! a thread-local recorder armed by [`install`], drained by [`take`], with a
//! drop guard so a failed query never leaks a trace. The recording hooks in
//! `exec.rs` sit behind `#[cfg(feature = "zk")]` (this whole module does):
//! default builds carry ZERO zk code, and a `zk` build without an installed
//! recorder pays one thread-local `Cell<bool>` read per *scan* (not per row)
//! — the same cost class as `trace::enabled()` / the budget check.
//!
//! Scope honesty (skeleton): the hooks cover BGP scans (`scan_to_bindings`),
//! index-nested-loop rescans (`bind_join`), WCOJ trie builds (`build_trie`)
//! and FILTER applications (`apply_filter`) — i.e. the monotone
//! BGP + FILTER fragment the stage-1 derived-credential queries use.
//! Property paths, EXISTS-internal scans (which do flow through
//! `scan_to_bindings` but are not labelled as such), and non-monotone
//! operators (OPTIONAL / MINUS) record their scans but their *sufficiency*
//! semantics are a later deliverable (with flat commitments the whole graph
//! is in-circuit anyway; plan §2.2).

use oxrdf::{Term, Variable};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

/// One slot of a recorded triple pattern: a constant term or a variable name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlotPattern {
    Term(Term),
    Var(String),
    /// The slot was an unnamed position (no variable, no constant id — does
    /// not occur for store scans, kept for totality).
    Wildcard,
}

impl std::fmt::Display for SlotPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotPattern::Term(t) => write!(f, "{t}"),
            SlotPattern::Var(v) => write!(f, "?{v}"),
            SlotPattern::Wildcard => write!(f, "_"),
        }
    }
}

/// A recorded triple pattern (the dedup key for matched-triple sets).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternKey {
    pub slots: [SlotPattern; 3],
}

/// The matched-triple input set of one BGP triple pattern.
#[derive(Debug, Clone)]
pub struct PatternMatches {
    pub pattern: PatternKey,
    /// Matched triples (s, p, o), deduplicated, in first-match order.
    pub triples: Vec<[Term; 3]>,
}

/// One FILTER obligation: the expression, the binding variables in scope and
/// the per-row operand bindings with the filter's verdict.
#[derive(Debug, Clone)]
pub struct FilterObligation {
    /// Debug rendering of the `spargebra` expression (informational; the
    /// obligation compiler re-derives structure from the query text).
    pub expr: String,
    /// Variables of the filtered binding set, in column order.
    pub vars: Vec<String>,
    /// Per input row: the operand bindings (one per var, `None` = unbound)
    /// and whether the row passed the filter.
    pub rows: Vec<(Vec<Option<Term>>, bool)>,
}

/// One step of the executed order, in execution sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// A full index scan of a pattern (also: a WCOJ trie build).
    Scan { pattern: usize },
    /// An index-nested-loop (bind-join) rescan of a pattern against the
    /// running result.
    BindJoin { pattern: usize },
    /// A FILTER application.
    Filter { filter: usize },
}

/// The drained zk trace of one executed query.
#[derive(Debug, Clone, Default)]
pub struct ZkTrace {
    /// Per-pattern matched-triple input sets (`Step::Scan`/`Step::BindJoin`
    /// indices point here).
    pub patterns: Vec<PatternMatches>,
    /// FILTER obligations (`Step::Filter` indices point here).
    pub filters: Vec<FilterObligation>,
    /// Executed scan / join / filter order.
    pub steps: Vec<Step>,
}

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(Default)]
struct State {
    patterns: Vec<PatternMatches>,
    seen: Vec<HashSet<[Term; 3]>>,
    filters: Vec<FilterObligation>,
    steps: Vec<Step>,
}

/// Disarms the recorder (and clears any partial trace) when the installing
/// entry point returns — also on error/unwind.
pub struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        ENABLED.with(|e| e.set(false));
        STATE.with(|s| *s.borrow_mut() = State::default());
    }
}

/// Arms the thread-local zk-trace recorder for queries executed on this
/// thread until the guard drops. Drain with [`take`] before dropping.
pub fn install() -> Guard {
    ENABLED.with(|e| e.set(true));
    STATE.with(|s| *s.borrow_mut() = State::default());
    Guard
}

/// Drains the recorded trace (call before the [`Guard`] drops).
pub fn take() -> ZkTrace {
    let state = STATE.with(|s| std::mem::take(&mut *s.borrow_mut()));
    ZkTrace { patterns: state.patterns, filters: state.filters, steps: state.steps }
}

#[inline]
pub(crate) fn enabled() -> bool {
    ENABLED.with(|e| e.get())
}

/// Records the matched triples of one pattern scan (`kind` distinguishes a
/// full scan from a bind-join rescan). Triple sets accumulate per pattern key
/// (a rescanned pattern keeps one merged input set).
pub(crate) fn record_pattern(
    pattern: PatternKey,
    matched: Vec<[Term; 3]>,
    bind_join: bool,
) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let idx = match s.patterns.iter().position(|p| p.pattern == pattern) {
            Some(i) => i,
            None => {
                s.patterns.push(PatternMatches { pattern, triples: Vec::new() });
                s.seen.push(HashSet::new());
                s.patterns.len() - 1
            }
        };
        for t in matched {
            if s.seen[idx].insert(t.clone()) {
                s.patterns[idx].triples.push(t);
            }
        }
        s.steps.push(if bind_join { Step::BindJoin { pattern: idx } } else { Step::Scan { pattern: idx } });
    });
}

/// Records one FILTER application.
pub(crate) fn record_filter(
    expr: String,
    vars: &[Variable],
    rows: Vec<(Vec<Option<Term>>, bool)>,
) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let idx = s.filters.len();
        s.filters.push(FilterObligation {
            expr,
            vars: vars.iter().map(|v| v.as_str().to_string()).collect(),
            rows,
        });
        s.steps.push(Step::Filter { filter: idx });
    });
}

/// Id-level recording entry used by the `exec.rs` hooks: materializes the
/// pattern key and the matched triples through the graph dictionary. Only
/// called when [`enabled`] — materialization cost exists only under an armed
/// recorder (the derived-credential proving path, credential-scale data).
pub(crate) fn record_scan_ids(
    graph: &sparq_core::Graph,
    id_pat: &sparq_core::store::Pattern,
    pos_vars: &[Option<Variable>; 3],
    matched: &[[sparq_core::dict::Id; 3]],
    bind_join: bool,
) {
    let slot = |i: usize| -> SlotPattern {
        match (id_pat[i], &pos_vars[i]) {
            (Some(id), _) => SlotPattern::Term(graph.dict.term(id)),
            (None, Some(v)) => SlotPattern::Var(v.as_str().to_string()),
            (None, None) => SlotPattern::Wildcard,
        }
    };
    let pattern = PatternKey { slots: [slot(0), slot(1), slot(2)] };
    let triples: Vec<[Term; 3]> = matched
        .iter()
        .map(|spo| [graph.dict.term(spo[0]), graph.dict.term(spo[1]), graph.dict.term(spo[2])])
        .collect();
    record_pattern(pattern, triples, bind_join);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_clears_state() {
        let g = install();
        record_pattern(
            PatternKey { slots: [SlotPattern::Wildcard, SlotPattern::Wildcard, SlotPattern::Wildcard] },
            vec![],
            false,
        );
        drop(g);
        let g2 = install();
        let t = take();
        assert!(t.patterns.is_empty() && t.steps.is_empty());
        drop(g2);
    }

    #[test]
    fn dedups_within_pattern_and_orders_steps() {
        let _g = install();
        let key = PatternKey {
            slots: [
                SlotPattern::Var("s".into()),
                SlotPattern::Term(Term::NamedNode(oxrdf::NamedNode::new_unchecked("http://ex/p"))),
                SlotPattern::Var("o".into()),
            ],
        };
        let t1 = [
            Term::NamedNode(oxrdf::NamedNode::new_unchecked("http://ex/a")),
            Term::NamedNode(oxrdf::NamedNode::new_unchecked("http://ex/p")),
            Term::NamedNode(oxrdf::NamedNode::new_unchecked("http://ex/b")),
        ];
        record_pattern(key.clone(), vec![t1.clone()], false);
        record_pattern(key, vec![t1.clone()], true);
        record_filter("test".into(), &[], vec![]);
        let tr = take();
        assert_eq!(tr.patterns.len(), 1);
        assert_eq!(tr.patterns[0].triples.len(), 1, "rescan must not duplicate");
        assert_eq!(
            tr.steps,
            vec![Step::Scan { pattern: 0 }, Step::BindJoin { pattern: 0 }, Step::Filter { filter: 0 }]
        );
    }
}
