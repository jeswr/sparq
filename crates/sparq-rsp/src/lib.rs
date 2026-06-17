//! sparq-rsp: deterministic RSP-QL-style RDF stream processing for the sparq
//! engine — windowed continuous SPARQL queries as a **library**, not a service.
//!
//! The pipeline is the classic RSP-QL trio:
//!
//! 1. **S2R** — a [`WindowSpec`] (`RANGE w STEP s` time windows or `ROWS n`
//!    count windows) turns a [stream](TripleStream) of
//!    [`TimestampedTriple`]s into a sequence of finite [`Window`]s,
//!    maintained incrementally by [`WindowedStream`];
//! 2. **R2R** — each closed window is materialised into a
//!    [`sparq_core::Graph`] (by the configured [`EvalMode`]: rebuild /
//!    persistent dictionary / delta — see [`EvalMode`] and the README
//!    throughput table) and the registered SPARQL query is evaluated by
//!    `sparq_engine`;
//! 3. **R2S** — an [`R2S`] operator (RSTREAM / ISTREAM / DSTREAM) maps the
//!    result back onto the output stream, delivered through a callback as
//!    [`WindowResult`]s.
//!
//! [`ContinuousQuery::register`] wires the three together for SELECT;
//! [`ContinuousConstruct`] (stream-to-stream transformation: each window's
//! result is itself a graph, as [`GraphResult`]s) and [`ContinuousAsk`] (one
//! boolean per window, as [`AskResult`]s) are the CONSTRUCT / ASK forms.
//!
//! # RSP-QL surface syntax + multi-window joins ([OPUS-4.8], sq-9u1)
//!
//! The above is the *programmatic* API (build a [`WindowSpec`], register plain
//! SPARQL). [`RspqlQuery::parse`] additionally parses the **RSP-QL textual query
//! language** — `REGISTER … AS`, `FROM NAMED WINDOW <w> ON <s> [RANGE … STEP …]`,
//! and `WINDOW <w> { … }` graph patterns — into that representation, reusing
//! spargebra for the embedded SPARQL. [`ContinuousMultiQuery`] is the front end
//! for queries that open MORE THAN ONE named window and JOIN across them: each
//! window keeps its own S2R state over its stream, and at each synchronized tick
//! the windows are assembled as named graphs and joined by the engine. See
//! [`RspqlQuery`] and [`ContinuousMultiQuery`] for the supported subset and the
//! scoped-out constructs.
//!
//! # Design constraints (deliberate)
//!
//! * **No async runtime, no wall clock.** Timestamps are application-supplied
//!   `u64`s and time only advances through pushed timestamps (watermark =
//!   `max_ts − max_delay`). The whole pipeline is a pure function of the
//!   pushed `(triple, ts)` sequence: deterministic, unit-testable, and
//!   trivially embeddable under tokio / threads / wasm timers *by the caller*.
//! * **Exact window semantics are documented** in `window` (boundary
//!   inclusivity, lateness, empty windows, gaps) and pinned by tests.
//! * **Isolation:** nothing else in the workspace depends on this crate; the
//!   core engine and the wasm build carry zero streaming code.
//!
//! ```
//! use oxrdf::{Literal, NamedNode, Term};
//! use sparq_rsp::{ContinuousQuery, WindowSpec};
//!
//! // Tumbling 60-tick windows, average reading per window.
//! let mut q = ContinuousQuery::register(
//!     "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }",
//!     WindowSpec::time(60, 60).with_max_delay(5), // tolerate 5 ticks of disorder
//! ).unwrap();
//!
//! let reading = |v: i32| -> [Term; 3] {
//!     [NamedNode::new_unchecked("http://ex/sensor1").into(),
//!      NamedNode::new_unchecked("http://ex/reading").into(),
//!      Literal::from(v).into()]
//! };
//! q.push(reading(10), 0, |_| {}).unwrap();
//! q.push(reading(20), 30, |_| {}).unwrap();
//! let mut avgs = Vec::new();
//! q.flush(|r| avgs.push(r.rows[0][0].clone())).unwrap();
//! // Integer averages come back as xsd:decimal per SPARQL AVG typing.
//! let avg = Literal::new_typed_literal("15.0", oxrdf::vocab::xsd::DECIMAL);
//! assert_eq!(avgs, vec![Some(avg.into())]);
//! ```
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

mod eval;
mod multi;
mod query;
mod rspql;
mod stream;
mod window;

pub use eval::EvalMode;
pub use multi::ContinuousMultiQuery;
pub use query::{
    AskResult, ContinuousAsk, ContinuousConstruct, ContinuousQuery, GraphResult, R2S, WindowResult,
};
pub use rspql::{RspqlQuery, WindowDecl};
pub use stream::{Timestamped, TimestampedTriple, TripleStream};
pub use window::{Window, WindowSpec, WindowedStream};
