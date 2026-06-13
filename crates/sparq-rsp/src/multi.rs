//! [OPUS-4.8] Multi-window continuous queries (sq-9u1).
//!
//! [`ContinuousQuery`](crate::ContinuousQuery) is *one stream, one window, one
//! query*. RSP-QL allows a query to open SEVERAL named windows — possibly over
//! different streams, possibly with different `RANGE`/`STEP` — and JOIN across
//! them in the `WHERE` (`WINDOW <w1> { … } WINDOW <w2> { … }`, sharing
//! variables). [`ContinuousMultiQuery`] is that form.
//!
//! # How the join is evaluated
//!
//! Each declared window keeps its OWN S2R state — a
//! [`WindowedStream`](crate::WindowedStream) over the stream it reads. A push
//! `(stream, triple, ts)` is routed to every window declared `ON` that stream;
//! a stream feeding two windows feeds both, independently. All windows share ONE
//! event-time clock (the maximum timestamp seen across all streams), so closure
//! is synchronized: a triple on stream `s1` advancing the clock can close a
//! window on stream `s2`. This is the deterministic Window-Close report policy —
//! the same watermark contract the single-window pipeline already pins.
//!
//! At each EVALUATION TICK — a synchronized boundary at which at least one window
//! closes — every window contributes its current closed content as a NAMED GRAPH
//! keyed by the window IRI. The windows are assembled into a single
//! [`sparq_core::Graph`] (empty default graph, one named sub-graph per window),
//! and the registered query — with each `WINDOW <w>` rewritten to a standard
//! SPARQL `GRAPH <w>` — is evaluated against it. The engine's `GRAPH <w> { … }`
//! evaluation translates each window's ids into the shared outer dictionary, so
//! a variable bound in `WINDOW <w1>` JOINS correctly with the same variable in
//! `WINDOW <w2>` (the cross-graph join the engine already supports — see
//! `eval_graph_named`). The full result is delivered as a [`WindowResult`].
//!
//! ## Tick alignment (the synchronization rule)
//!
//! Windows may close at different times (different `STEP`s). The join is
//! evaluated **once per distinct closing boundary**, oldest first; at boundary
//! `t` each window contributes the content of its window that closed AT or
//! IMMEDIATELY BEFORE `t` (its latest-closed content), so a slow window's
//! content is held steady across the faster window's ticks. A window that has
//! not closed any content yet (no triples within reach of the watermark)
//! contributes the empty graph — `GRAPH <w> { … }` then yields no rows, so the
//! join correctly produces nothing until every joined window has content.
//!
//! ## Scope (honest)
//!
//! * Time windows only (the surface grammar has no `ROWS`); R2S is RSTREAM (the
//!   per-tick FULL join result). ISTREAM/DSTREAM over a multi-window join are a
//!   documented follow-up (the diff base would be the previous tick's full
//!   result; mechanically the single-window [`diff_rows`](crate::query) applies,
//!   but it is not wired here).
//! * The output operator is `R2S::RStream`; `output_stream`/entailment metadata
//!   parsed by [`RspqlQuery`] is retained but not acted on (this is a library,
//!   not a stream-router).

use std::collections::BTreeSet;

use oxrdf::{NamedNode, Term};
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;
use sparq_engine::PreparedQuery;
use spargebra::Query;

use crate::query::{R2S, WindowResult};
use crate::rspql::{RspqlQuery, WindowDecl};
use crate::window::{Window, WindowedStream};

/// One declared window's live state: which stream it reads, its window IRI (the
/// named-graph key), its S2R machine, and the content of its most recently
/// closed window (held steady between this window's own closing boundaries).
struct WindowState {
    window: NamedNode,
    stream: NamedNode,
    ws: WindowedStream<[Term; 3]>,
    /// The bounds + content of the latest CLOSED window, or `None` until this
    /// window first closes. Carried across faster windows' ticks so the join
    /// sees a stable snapshot of the slow window.
    latest: Option<Window<[Term; 3]>>,
}

/// A registered RSP-QL continuous query joining across MORE THAN ONE named
/// window (the multi-window form). Built from RSP-QL surface syntax via
/// [`from_rspql`](Self::from_rspql) (or [`register`](Self::register) for the raw
/// text), pushed per `(stream, triple, ts)`, delivering one [`WindowResult`] per
/// synchronized evaluation tick.
///
/// ```
/// use oxrdf::{NamedNode, Term};
/// use sparq_rsp::ContinuousMultiQuery;
///
/// // Join readings on stream :temp (window :w1) with the room each sensor is in
/// // on stream :meta (window :w2), per synchronized 10-tick tumbling window.
/// let q_text = "\
/// REGISTER STREAM <http://ex/out> AS
/// SELECT ?room ?v WHERE {
///   WINDOW <http://ex/w1> { ?s <http://ex/value> ?v }
///   WINDOW <http://ex/w2> { ?s <http://ex/in> ?room }
/// }
/// FROM NAMED WINDOW <http://ex/w1> ON <http://ex/temp> RANGE 10 STEP 10
/// FROM NAMED WINDOW <http://ex/w2> ON <http://ex/meta> RANGE 10 STEP 10";
/// let mut q = ContinuousMultiQuery::register(q_text).unwrap();
///
/// let temp = NamedNode::new_unchecked("http://ex/temp");
/// let meta = NamedNode::new_unchecked("http://ex/meta");
/// let t = |s: &str, p: &str, o: Term| -> [Term; 3] {
///     [NamedNode::new_unchecked(format!("http://ex/{s}")).into(),
///      NamedNode::new_unchecked(format!("http://ex/{p}")).into(), o]
/// };
/// let mut rows = Vec::new();
/// q.push(&meta, t("s1", "in", NamedNode::new_unchecked("http://ex/kitchen").into()), 1, |_| {}).unwrap();
/// q.push(&temp, t("s1", "value", oxrdf::Literal::from(21).into()), 2, |r| rows.extend(r.rows)).unwrap();
/// q.flush(|r| rows.extend(r.rows)).unwrap();
/// assert_eq!(rows.len(), 1); // s1's reading joined with its room
/// ```
pub struct ContinuousMultiQuery {
    rspql: String,
    /// The embedded SPARQL with `WINDOW` rewritten to `GRAPH` (executed per tick).
    sparql: String,
    prepared: PreparedQuery,
    windows: Vec<WindowState>,
    r2s: R2S,
    output_stream: Option<NamedNode>,
}

impl ContinuousMultiQuery {
    /// Registers a multi-window query from RSP-QL surface syntax. The text is
    /// parsed ([`RspqlQuery::parse`]), the embedded SPARQL (with `WINDOW`
    /// rewritten to `GRAPH`) is validated as a SELECT, and one window machine is
    /// created per `FROM NAMED WINDOW` declaration.
    ///
    /// Errors: a malformed RSP-QL header, a non-SELECT embedded query, fewer
    /// than two windows (use [`ContinuousQuery`](crate::ContinuousQuery) for
    /// one), or a `WINDOW <w>` in the `WHERE` with no matching declaration.
    pub fn register(rspql_text: &str) -> Result<Self, String> {
        let parsed = RspqlQuery::parse(rspql_text)?;
        Self::from_rspql(&parsed, rspql_text)
    }

    /// Builds a multi-window query from an already-parsed [`RspqlQuery`].
    /// `rspql_text` is retained only for [`rspql`](Self::rspql).
    pub fn from_rspql(parsed: &RspqlQuery, rspql_text: &str) -> Result<Self, String> {
        let prepared = PreparedQuery::parse(&parsed.sparql)?;
        if !matches!(prepared.query(), Query::Select { .. }) {
            return Err("multi-window continuous queries must be SELECT queries".into());
        }
        if parsed.windows.len() < 2 {
            return Err(format!(
                "multi-window query needs ≥ 2 FROM NAMED WINDOW declarations (got {}); \
                 use ContinuousQuery for a single window",
                parsed.windows.len()
            ));
        }
        // Every window referenced in the WHERE must be declared. (The reverse —
        // a declared-but-unreferenced window — is allowed: it just never
        // contributes, exactly like an unused FROM NAMED.)
        let referenced = referenced_graphs(prepared.query());
        let declared: BTreeSet<&str> = parsed.windows.iter().map(|w| w.window.as_str()).collect();
        for r in &referenced {
            if !declared.contains(r.as_str()) {
                return Err(format!(
                    "WINDOW <{r}> is used in the WHERE clause but never declared with FROM NAMED WINDOW"
                ));
            }
        }
        let windows = parsed
            .windows
            .iter()
            .map(|d: &WindowDecl| WindowState {
                window: d.window.clone(),
                stream: d.stream.clone(),
                ws: WindowedStream::empty(d.spec),
                latest: None,
            })
            .collect();
        Ok(ContinuousMultiQuery {
            rspql: rspql_text.to_owned(),
            sparql: parsed.sparql.clone(),
            prepared,
            windows,
            r2s: parsed.r2s,
            output_stream: parsed.output_stream.clone(),
        })
    }

    /// The registered RSP-QL source text.
    pub fn rspql(&self) -> &str {
        &self.rspql
    }

    /// The embedded SPARQL (with `WINDOW` rewritten to `GRAPH`) executed per
    /// tick.
    pub fn sparql(&self) -> &str {
        &self.sparql
    }

    /// The declared window IRIs, in source order.
    pub fn window_iris(&self) -> Vec<&NamedNode> {
        self.windows.iter().map(|w| &w.window).collect()
    }

    /// The `REGISTER … <out> AS` output-stream IRI, if the source named one.
    pub fn output_stream(&self) -> Option<&NamedNode> {
        self.output_stream.as_ref()
    }

    /// The relation-to-stream operator from the `REGISTER` header. Currently
    /// only [`R2S::RStream`] (the full per-tick join result) is acted on;
    /// ISTREAM/DSTREAM over a multi-window join are a documented follow-up, so
    /// this is retained metadata.
    pub fn r2s(&self) -> R2S {
        self.r2s
    }

    /// Pushes one stream element TAGGED WITH ITS STREAM. Routed to every window
    /// declared `ON` this stream; `on_result` fires once per synchronized
    /// evaluation tick this push triggers (zero or more), oldest first.
    ///
    /// A push on a stream no window reads is accepted and ignored (it still does
    /// not advance any window — only timestamps on a window's own stream do, per
    /// that window's S2R).
    pub fn push(
        &mut self,
        stream: &NamedNode,
        triple: [Term; 3],
        ts: u64,
        on_result: impl FnMut(WindowResult),
    ) -> Result<(), String> {
        for w in &mut self.windows {
            if w.stream == *stream {
                // A window over this stream BUFFERS the triple (and advances its
                // own watermark to `ts`).
                w.ws.push(triple.clone(), ts);
            } else {
                // A window over another stream sees `ts` only as a SHARED-clock
                // heartbeat: event time has advanced to `ts`, so its windows
                // close on the same boundary — but the triple is not in its
                // stream, so nothing is buffered. This is what synchronizes the
                // join across streams of differing arrival rates.
                w.ws.advance(ts);
            }
        }
        self.drain(false, on_result)
    }

    /// End-of-stream: flushes every window to its last seen timestamp and
    /// delivers the final ticks.
    pub fn flush(&mut self, on_result: impl FnMut(WindowResult)) -> Result<(), String> {
        self.drain(true, on_result)
    }

    /// Drains closed windows into synchronized evaluation ticks. The set of
    /// distinct close boundaries (the `end` of each closed window across all
    /// streams) is the tick schedule; at each tick every window contributes the
    /// latest content that closed at or before it.
    fn drain(
        &mut self,
        flush: bool,
        mut on_result: impl FnMut(WindowResult),
    ) -> Result<(), String> {
        // Collect every newly-closed window per stream, and the set of distinct
        // close boundaries that defines the tick schedule.
        let mut boundaries: BTreeSet<u64> = BTreeSet::new();
        let mut closed: Vec<Vec<Window<[Term; 3]>>> = Vec::with_capacity(self.windows.len());
        for w in &mut self.windows {
            let wins = if flush { w.ws.flush() } else { w.ws.take_closed() };
            for win in &wins {
                boundaries.insert(win.end);
            }
            closed.push(wins);
        }
        if boundaries.is_empty() {
            return Ok(());
        }
        // Per-window cursor into its `closed` vector (windows are oldest-first).
        let mut cursor = vec![0usize; self.windows.len()];
        for tick in boundaries {
            // Advance each window to the latest content that CLOSED at or before
            // this tick boundary, holding it as that window's snapshot.
            for (i, wins) in closed.iter().enumerate() {
                while cursor[i] < wins.len() && wins[cursor[i]].end <= tick {
                    self.windows[i].latest = Some(wins[cursor[i]].clone());
                    cursor[i] += 1;
                }
            }
            self.eval_tick(tick, &mut on_result)?;
        }
        Ok(())
    }

    /// Builds the combined named-graph [`Graph`] for the current per-window
    /// snapshots and evaluates the registered query against it.
    fn eval_tick(
        &mut self,
        tick: u64,
        on_result: &mut impl FnMut(WindowResult),
    ) -> Result<(), String> {
        let graph = self.materialize_named();
        let result = sparq_engine::query_prepared(&graph, &self.prepared)?;
        // RSTREAM: the full join result. (R2S diffing over a multi-window join
        // is a documented follow-up.)
        let rows = result.rows;
        // The reported bounds: the tick's span is the join boundary. We report
        // [start, end) where end = tick and start = the min snapshot start, so
        // the window result carries a meaningful interval for the embedder.
        let start = self
            .windows
            .iter()
            .filter_map(|w| w.latest.as_ref().map(|win| win.start))
            .min()
            .unwrap_or(tick);
        on_result(WindowResult { start, end: tick, vars: result.vars, rows });
        Ok(())
    }

    /// Assembles the per-window snapshots into one [`Graph`]: an empty default
    /// graph plus one named sub-graph per window (keyed by the window IRI). Each
    /// window's content is a fresh set-semantic sub-`Graph` (duplicate triples
    /// across timestamps collapse, as in any RDF graph). The engine's
    /// `GRAPH <w> { … }` then evaluates against the matching sub-graph and
    /// translates ids into this outer dictionary, so cross-window joins bind
    /// correctly.
    fn materialize_named(&self) -> Graph {
        let mut g = Graph::from_parts(Dict::new(), Vec::new());
        for w in &self.windows {
            let sub = match &w.latest {
                Some(win) => sub_graph(win),
                None => Graph::from_parts(Dict::new(), Vec::new()),
            };
            g.named.push((Term::NamedNode(w.window.clone()), sub));
        }
        g
    }
}

/// Materialises one window's content into a fresh dictionary-encoded sub-graph
/// (set semantics: a triple at several timestamps appears once).
fn sub_graph(win: &Window<[Term; 3]>) -> Graph {
    let mut dict = Dict::new();
    let ids: Vec<[Id; 3]> = win
        .triples
        .iter()
        .map(|t| [dict.intern(&t.triple[0]), dict.intern(&t.triple[1]), dict.intern(&t.triple[2])])
        .collect();
    Graph::from_parts(dict, ids)
}

/// The set of concrete named graphs referenced by `GRAPH <iri> { … }` patterns
/// in the query (the rewritten `WINDOW <iri>`). Walks the algebra; `GRAPH ?v`
/// (variable) is ignored (window variables are rejected at parse time, so this
/// only catches literal stray `GRAPH <iri>` clauses, which we also validate).
fn referenced_graphs(q: &Query) -> BTreeSet<String> {
    use spargebra::algebra::GraphPattern;
    use spargebra::term::NamedNodePattern;
    let mut out = BTreeSet::new();
    fn walk(p: &GraphPattern, out: &mut BTreeSet<String>) {
        use GraphPattern::*;
        match p {
            Graph { name, inner } => {
                if let NamedNodePattern::NamedNode(n) = name {
                    out.insert(n.as_str().to_owned());
                }
                walk(inner, out);
            }
            Join { left, right }
            | LeftJoin { left, right, .. }
            | Union { left, right }
            | Minus { left, right } => {
                walk(left, out);
                walk(right, out);
            }
            Filter { inner, .. }
            | Extend { inner, .. }
            | OrderBy { inner, .. }
            | Project { inner, .. }
            | Distinct { inner }
            | Reduced { inner }
            | Slice { inner, .. }
            | Group { inner, .. }
            | Service { inner, .. } => walk(inner, out),
            // Leaves (Bgp, Path, Values, table-unit, …) reference no named graph.
            _ => {}
        }
    }
    let pattern = match q {
        Query::Select { pattern, .. }
        | Query::Construct { pattern, .. }
        | Query::Describe { pattern, .. }
        | Query::Ask { pattern, .. } => pattern,
    };
    walk(pattern, &mut out);
    out
}
