//! Continuous queries: the RSP-QL pipeline glued together.
//!
//! [`ContinuousQuery`] owns one [`WindowedStream`] (the S2R operator), one
//! registered SPARQL SELECT (validated at [`register`](ContinuousQuery::register)
//! time), and one [`R2S`] relation-to-stream operator. Every push drives the
//! window; every window that closes is **materialised into a fresh
//! [`sparq_core::Graph`]** (set semantics — duplicate triples in the window
//! collapse, as in any RDF graph), evaluated by `sparq_engine`, run through the
//! R2S operator, and handed to the caller's callback as a [`WindowResult`].
//!
//! # R2S semantics (multiset, row-hash based)
//!
//! SPARQL SELECT results are multisets of rows. The three operators:
//!
//! * **RSTREAM** — every window emits its FULL result. Stateless.
//! * **ISTREAM** — every window emits the rows *added* relative to the previous
//!   window's result: the multiset difference `cur ∖ prev`. The first window
//!   diffs against the empty multiset, i.e. emits everything.
//! * **DSTREAM** — the rows *removed*: `prev ∖ cur`. The first window emits
//!   nothing. Note DSTREAM needs empty windows to be reported (they are — see
//!   [`crate::window`]): a result only "disappears" when a later, possibly
//!   empty, window closes without it.
//!
//! Diffs are computed over 64-bit **row hashes** (`FxHasher` over the bound
//! terms), counted as multisets, so a row appearing twice in `cur` and once in
//! `prev` ISTREAMs exactly once. Emission order is deterministic: ISTREAM
//! preserves the engine's row order of the current window, DSTREAM the row
//! order of the previous window. (A hash collision between two *different*
//! rows inside one query's results could suppress a diff; with 64-bit hashes
//! this is vanishingly unlikely and accepted for speed.)

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use oxrdf::{Term, Variable};
use sparq_core::dict::Dict;
use sparq_core::Graph;
use spargebra::{Query, SparqlParser};

use crate::window::{Window, WindowSpec, WindowedStream};

/// The relation-to-stream operator: what part of each window's result is
/// streamed to the callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum R2S {
    /// The full result of every window (the RSP-QL default).
    #[default]
    RStream,
    /// Rows added relative to the previous window (multiset difference).
    IStream,
    /// Rows removed relative to the previous window (multiset difference).
    DStream,
}

/// One R2S-filtered query result, delivered per closed window.
#[derive(Debug, Clone)]
pub struct WindowResult {
    /// Window bounds, as reported by the window operator: time windows are
    /// half-open `[start, end)`; count windows carry the inclusive
    /// `[first.ts, last.ts]` of their content (see [`crate::window`]).
    pub start: u64,
    /// See `start`.
    pub end: u64,
    /// Projected variables, in SELECT order.
    pub vars: Vec<Variable>,
    /// The emitted rows (full / added / removed per [`R2S`]); each row has one
    /// entry per `vars` position, `None` = unbound.
    pub rows: Vec<Vec<Option<Term>>>,
}

/// A registered continuous query: window spec + SPARQL SELECT + R2S operator.
///
/// ```
/// use oxrdf::{NamedNode, Term, Literal};
/// use sparq_rsp::{ContinuousQuery, WindowSpec};
///
/// let mut q = ContinuousQuery::register(
///     "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/value> ?v }",
///     WindowSpec::time(10, 10), // tumbling: RANGE 10 STEP 10
/// ).unwrap();
///
/// let t = |n: i32| -> [Term; 3] {
///     [NamedNode::new_unchecked("http://ex/s").into(),
///      NamedNode::new_unchecked("http://ex/value").into(),
///      Literal::from(n).into()]
/// };
/// let mut results = Vec::new();
/// q.push(t(2), 1, |r| results.push(r)).unwrap();
/// q.push(t(4), 5, |r| results.push(r)).unwrap();
/// q.push(t(9), 12, |r| results.push(r)).unwrap(); // ts 12 closes [0,10)
/// assert_eq!(results.len(), 1);
/// // AVG(2,4) — integer averages come back as xsd:decimal per SPARQL typing.
/// let avg = Literal::new_typed_literal("3.0", oxrdf::vocab::xsd::DECIMAL);
/// assert_eq!(results[0].rows[0][0], Some(avg.into()));
/// ```
pub struct ContinuousQuery {
    sparql: String,
    window: WindowedStream,
    r2s: R2S,
    /// Previous window's full result, in engine row order, with row hashes —
    /// the ISTREAM/DSTREAM diff base. Unused (empty) under RSTREAM.
    prev_rows: Vec<Vec<Option<Term>>>,
    prev_hashes: Vec<u64>,
}

impl ContinuousQuery {
    /// Registers a continuous SELECT over a windowed stream, with the RSP-QL
    /// default output operator (RSTREAM). The query string is parsed and
    /// validated NOW — a malformed or non-SELECT query is rejected at
    /// registration, not at the first window.
    pub fn register(sparql: &str, spec: WindowSpec) -> Result<Self, String> {
        let q = SparqlParser::new().parse_query(sparql).map_err(|e| e.to_string())?;
        if !matches!(q, Query::Select { .. }) {
            return Err("continuous queries must be SELECT queries".into());
        }
        Ok(ContinuousQuery {
            sparql: sparql.to_owned(),
            window: WindowedStream::empty(spec),
            r2s: R2S::RStream,
            prev_rows: Vec::new(),
            prev_hashes: Vec::new(),
        })
    }

    /// Sets the relation-to-stream operator (builder style):
    /// `ContinuousQuery::register(q, spec)?.with_r2s(R2S::IStream)`.
    pub fn with_r2s(mut self, r2s: R2S) -> Self {
        self.r2s = r2s;
        self
    }

    /// The registered SPARQL text.
    pub fn sparql(&self) -> &str {
        &self.sparql
    }

    /// The window specification.
    pub fn spec(&self) -> WindowSpec {
        self.window.spec()
    }

    /// Pushes one stream element; `on_result` fires once per window this
    /// closes (zero or more times), oldest window first.
    ///
    /// Errors are evaluation errors from the engine (the query itself was
    /// validated at registration). On error, remaining closed windows are
    /// dropped.
    pub fn push(
        &mut self,
        triple: [Term; 3],
        ts: u64,
        mut on_result: impl FnMut(WindowResult),
    ) -> Result<(), String> {
        self.window.push(triple, ts);
        let closed = self.window.take_closed();
        self.emit(closed, &mut on_result)
    }

    /// End-of-stream: closes every window up to the last timestamp seen
    /// (ignoring `max_delay`) and delivers the results. See
    /// [`WindowedStream::flush`] for the exact closure rule.
    pub fn flush(&mut self, mut on_result: impl FnMut(WindowResult)) -> Result<(), String> {
        let closed = self.window.flush();
        self.emit(closed, &mut on_result)
    }

    /// Arrivals dropped as too late (every covering window already closed).
    pub fn late_dropped(&self) -> u64 {
        self.window.late_dropped()
    }

    fn emit(
        &mut self,
        closed: Vec<Window>,
        on_result: &mut impl FnMut(WindowResult),
    ) -> Result<(), String> {
        for w in closed {
            let graph = materialize(&w);
            let result = sparq_engine::query(&graph, &self.sparql)?;
            let rows = match self.r2s {
                R2S::RStream => result.rows,
                R2S::IStream | R2S::DStream => self.diff(result.rows),
            };
            on_result(WindowResult { start: w.start, end: w.end, vars: result.vars, rows });
        }
        Ok(())
    }

    /// Applies ISTREAM/DSTREAM: multiset row-hash diff of the current window's
    /// full result against the previous window's, then advances the diff base.
    fn diff(&mut self, cur_rows: Vec<Vec<Option<Term>>>) -> Vec<Vec<Option<Term>>> {
        let cur_hashes: Vec<u64> = cur_rows.iter().map(|r| row_hash(r)).collect();
        let emitted = match self.r2s {
            // cur ∖ prev, in current row order.
            R2S::IStream => multiset_minus(&cur_rows, &cur_hashes, &self.prev_hashes),
            // prev ∖ cur, in previous row order.
            R2S::DStream => multiset_minus(&self.prev_rows, &self.prev_hashes, &cur_hashes),
            R2S::RStream => unreachable!("diff() is only called for ISTREAM/DSTREAM"),
        };
        self.prev_rows = cur_rows;
        self.prev_hashes = cur_hashes;
        emitted
    }
}

/// `keep ∖ minus` as multisets of row hashes: each hash in `minus` cancels at
/// most that many occurrences in `keep`; survivors are returned in `keep`'s
/// row order.
fn multiset_minus(
    keep_rows: &[Vec<Option<Term>>],
    keep_hashes: &[u64],
    minus: &[u64],
) -> Vec<Vec<Option<Term>>> {
    let mut budget: HashMap<u64, usize> = HashMap::with_capacity(minus.len());
    for h in minus {
        *budget.entry(*h).or_insert(0) += 1;
    }
    keep_rows
        .iter()
        .zip(keep_hashes)
        .filter(|(_, h)| match budget.get_mut(h) {
            Some(n) if *n > 0 => {
                *n -= 1;
                false // cancelled by an occurrence on the other side
            }
            _ => true, // survives the difference: emit
        })
        .map(|(row, _)| row.clone())
        .collect()
}

/// 64-bit hash of one result row (bound terms + unbound positions).
fn row_hash(row: &[Option<Term>]) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    row.hash(&mut h);
    h.finish()
}

/// Materialises one window's content into a fresh dictionary-encoded graph.
/// RDF graphs are sets: a triple present at several timestamps in the window
/// appears once.
fn materialize(w: &Window) -> Graph {
    let mut dict = Dict::new();
    let ids: Vec<[sparq_core::dict::Id; 3]> = w
        .triples
        .iter()
        .map(|t| [dict.intern(&t.triple[0]), dict.intern(&t.triple[1]), dict.intern(&t.triple[2])])
        .collect();
    Graph::from_parts(dict, ids)
}
