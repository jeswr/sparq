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
//!    [`sparq_core::Graph`] and the registered SPARQL SELECT is evaluated by
//!    `sparq_engine`;
//! 3. **R2S** — an [`R2S`] operator (RSTREAM / ISTREAM / DSTREAM) maps the
//!    result back onto the output stream, delivered through a callback as
//!    [`WindowResult`]s.
//!
//! [`ContinuousQuery::register`] wires the three together.
//!
//! # Design constraints (deliberate)
//!
//! * **No async runtime, no wall clock.** Timestamps are application-supplied
//!   `u64`s and time only advances through pushed timestamps (watermark =
//!   `max_ts − max_delay`). The whole pipeline is a pure function of the
//!   pushed `(triple, ts)` sequence: deterministic, unit-testable, and
//!   trivially embeddable under tokio / threads / wasm timers *by the caller*.
//! * **Exact window semantics are documented** in [`window`] (boundary
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

mod query;
mod stream;
mod window;

pub use query::{ContinuousQuery, R2S, WindowResult};
pub use stream::{TimestampedTriple, TripleStream};
pub use window::{Window, WindowSpec, WindowedStream};
