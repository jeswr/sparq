---
name: streaming-rsp
description: Use when running continuous/standing SPARQL over a live RDF triple stream with the sparq engine — sliding/tumbling time windows (RANGE/STEP), count (ROWS) windows, RSTREAM/ISTREAM/DSTREAM output, RSP-QL surface syntax (REGISTER STREAM, FROM NAMED WINDOW ... ON ... RANGE/STEP), and multi-window joins (WINDOW <w1>{} JOIN WINDOW <w2>{}). Covers the sparq-rsp crate's ContinuousQuery / ContinuousConstruct / ContinuousAsk / ContinuousMultiQuery / RspqlQuery / WindowSpec.
---

# sparq-streaming-rsp

`sparq-rsp` runs **windowed continuous SPARQL** (RSP-QL-style RDF Stream Processing) over a stream of `(triple, timestamp)` elements, as a deterministic **synchronous library** — no async runtime, no wall clock, no service. You push timestamped triples; it closes windows on a watermark and fires your callback once per closed window with the SELECT / CONSTRUCT / ASK result. It is a fully isolated, opt-in crate: nothing else in the workspace depends on it, the core engine and the **lean** `sparq-wasm` bundle carry zero streaming code (streaming ships as a *separate*, lazy-loaded `sparq-rsp-wasm` bundle — see below), and there are **no cargo features** — you engage it simply by depending on the crate.

## Quickstart

`Cargo.toml`:

```toml
[dependencies]
sparq-rsp = { path = "../sparq/crates/sparq-rsp" } # or version = "0.1.0" once published
oxrdf = { version = "0.3", features = ["rdf-12"] } # the term model (Term/NamedNode/Literal)
```

```rust
use oxrdf::{Literal, NamedNode, Term};
use sparq_rsp::{ContinuousQuery, WindowSpec};

// Average reading per TUMBLING 60-tick window, tolerating 5 ticks of disorder.
let mut q = ContinuousQuery::register(
    "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }",
    WindowSpec::time(60, 60).with_max_delay(5), // RANGE 60 STEP 60, lateness 5
)?;

// A stream element is a subject/predicate/object [Term; 3] plus a u64 timestamp.
let reading = |v: i32| -> [Term; 3] {
    [NamedNode::new_unchecked("http://ex/sensor1").into(),
     NamedNode::new_unchecked("http://ex/reading").into(),
     Literal::from(v).into()]
};

// push() fires on_result once per window CLOSED by this push (0+ times, oldest first).
q.push(reading(10), 0,  |r| println!("[{},{}) -> {:?}", r.start, r.end, r.rows))?;
q.push(reading(20), 30, |_| {})?;
// flush() = end-of-stream: close everything up to the last timestamp seen.
q.flush(|r| println!("final [{},{}) -> {:?}", r.start, r.end, r.rows))?;
// Integer AVG comes back as xsd:decimal "15.0" per SPARQL aggregate typing.
# Ok::<(), String>(())
```

The whole pipeline is a pure function of the pushed `(triple, ts)` sequence: replayable, unit-testable, wasm-safe. Wrapping pushes in tokio / a thread / a browser timer is your one-liner, not this crate's dependency.

### In-tab live streaming: the tier-b `sparq-rsp-wasm` ("W-rsp") bundle ([OPUS-4.8] sq-nzcb)

Because `sparq-rsp` reads no wall clock and runs no async runtime, it compiles to `wasm32-unknown-unknown` and ships as a **separate, lazy-loaded** wasm bundle (`crates/sparq-rsp-wasm`) — NOT folded into the lean `sparq-wasm` triplestore bundle (the `sparq-reason-wasm` "W-reason" pattern). It exposes a single stateful JS handle, `Rsp`, for the showcase site's `/surface/streaming-rsp` page, where the **browser tab drives the logical clock**:

```js
import init, { Rsp } from "./sparq_rsp_wasm.js";
await init();
const q = Rsp.select("SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }",
                     60, 60, 0, "rstream"); // range, step, maxDelay, "rstream"|"istream"|"dstream"
const closed = JSON.parse(q.push("<http://ex/s1>", "<http://ex/reading>", "10", 0)); // -> "[]" until a window closes
JSON.parse(q.flush()); // end-of-stream; q.lateDropped() = arrivals too late for any window
```

Each `push(s, p, o, ts)` / `flush()` returns a JSON array of the windows that just closed: `{"start","end","results"}`, where `results` is a standard self-contained **SPARQL 1.1 JSON** results document (from the engine's serialiser). Triple terms are **Turtle** syntax — the bare-numeric shorthand (`10`, `10.5`) works, alongside `<iri>`, `"str"`, `"str"@en`, `"v"^^<dt>`, `_:b`. The bundle wraps the single-window `ContinuousQuery` SELECT form only; CONSTRUCT/ASK and `ContinuousMultiQuery` stay native for now. Zero `unsafe`, no serde, no regex (it is the leanest of the wasm bundles); the wasm-deps guard keeps the native-only heavy deps out of its graph.

## Key APIs

All public items are re-exported at the crate root (`sparq_rsp::…`).

```rust
// --- S2R: the window spec (Copy enum + builders) ---
WindowSpec::time(range: u64, step: u64) -> WindowSpec   // panics if range==0 or step==0
WindowSpec::count(rows: usize) -> WindowSpec            // CQL count window; panics if rows==0
  .with_max_delay(d: u64) -> WindowSpec   // out-of-order tolerance (time windows only)
  .with_t0(t0: u64)       -> WindowSpec   // RSP-QL window origin (time windows only; default 0)
  .with_slide(s: usize)   -> WindowSpec   // report cadence (count windows only; default 1)

// --- R2S: relation-to-stream operator (Default = RStream) ---
enum R2S { RStream, IStream, DStream }

// --- R2R materialisation strategy (Default = PersistentDict) ---
enum EvalMode { Rebuild, PersistentDict, Delta }

// --- Continuous SELECT: WindowResult { start, end, vars: Vec<Variable>, rows: Vec<Vec<Option<Term>>> } ---
ContinuousQuery::register(sparql: &str, spec: WindowSpec) -> Result<ContinuousQuery, String>
  .with_r2s(R2S) -> Self            // builder
  .with_mode(EvalMode) -> Self      // builder; call BEFORE first push (resets stream state)
  .push(triple: [Term;3], ts: u64, on_result: impl FnMut(WindowResult)) -> Result<(), String>
  .flush(on_result: impl FnMut(WindowResult)) -> Result<(), String>
  .late_dropped() -> u64            // arrivals dropped because every covering window had closed

// --- Continuous CONSTRUCT: GraphResult { start, end, triples: Vec<Triple> } (stream->stream) ---
ContinuousConstruct::register(sparql: &str, spec: WindowSpec) -> Result<_, String>
  .with_r2s / .with_mode / .push(.., FnMut(GraphResult)) / .flush / .late_dropped

// --- Continuous ASK: AskResult { start, end, value: bool } (one boolean per window) ---
ContinuousAsk::register(sparql: &str, spec: WindowSpec) -> Result<_, String>
  .with_mode / .push(.., FnMut(AskResult)) / .flush / .late_dropped

// --- RSP-QL surface syntax + multi-window joins ---
RspqlQuery::parse(text: &str) -> Result<RspqlQuery, String>
  // fields: output_stream: Option<NamedNode>, r2s: R2S,
  //         windows: Vec<WindowDecl { window, stream, spec }>, sparql: String (WINDOW->GRAPH rewrite)
ContinuousMultiQuery::register(rspql_text: &str) -> Result<ContinuousMultiQuery, String>
  .push(stream: &NamedNode, triple: [Term;3], ts: u64, on_result: impl FnMut(WindowResult)) -> Result<(), String>
  .flush(on_result: impl FnMut(WindowResult)) -> Result<(), String>
  .window_iris() -> Vec<&NamedNode>   .output_stream() -> Option<&NamedNode>   .r2s() -> R2S

// --- Low-level S2R only (no query): WindowedStream<[Term;3]>, Window { start, end, triples } ---
WindowedStream::empty(spec) / ::new(stream: TripleStream, spec)
  .push(triple, ts)  .take_closed() -> Vec<Window>  .flush() -> Vec<Window>  .late_dropped()
```

`register` parses + validates the query **once** (a malformed or wrong-form query is rejected here, not at the first window) and keeps a `sparq_engine::PreparedQuery`; every window executes the prepared algebra with no re-parse.

## Common recipes

**Sliding window + ISTREAM (only newly-appearing rows):** `step < range` overlaps; `IStream` emits the multiset difference `current ∖ previous`.

```rust
use sparq_rsp::{ContinuousQuery, R2S, WindowSpec};
let mut q = ContinuousQuery::register(
    "SELECT ?s WHERE { ?s <http://ex/active> ?o }",
    WindowSpec::time(1000, 100),   // RANGE 1000 STEP 100 — slides 10x within its range
)?.with_r2s(R2S::IStream);
// q.push(.., |r| { /* r.rows = subjects that appeared since the previous window */ })?;
# Ok::<(), String>(())
```
Use `R2S::DStream` for rows that *disappeared* (`previous ∖ current`); DSTREAM relies on empty windows being reported, which they are.

**Count (ROWS) window — last N arrivals, regardless of time:**

```rust
use sparq_rsp::{ContinuousQuery, WindowSpec};
// Last 100 arrivals in arrival order; report every 10th arrival.
let spec = WindowSpec::count(100).with_slide(10);
let mut q = ContinuousQuery::register("SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }", spec)?;
# Ok::<(), String>(())
```

**CONSTRUCT to transform a stream into another stream (each window -> a graph):**

```rust
use sparq_rsp::{ContinuousConstruct, WindowSpec};
let mut q = ContinuousConstruct::register(
    "CONSTRUCT { ?s <http://ex/observed> ?v } WHERE { ?s <http://ex/value> ?v }",
    WindowSpec::time(10, 10),
)?;
// q.push(.., |g| { for t in g.triples { /* emit normalised observation triple */ } })?;
# Ok::<(), String>(())
```

**ASK as a cheap per-window condition watch** (`ContinuousAsk` returns one `bool` per window; the engine early-exits on the first solution).

**RSP-QL textual query — multi-window join across two streams:** `WINDOW <w>` is rewritten to `GRAPH <w>`; each window is materialised as a named graph keyed by its IRI and joined by shared variables. Push is **tagged with the source stream**; a triple on one stream advances the shared watermark of windows on the others, so closure is synchronized.

```rust
use oxrdf::{NamedNode, Literal, Term};
use sparq_rsp::ContinuousMultiQuery;

let mut q = ContinuousMultiQuery::register("\
REGISTER STREAM <http://ex/out> AS
SELECT ?room ?v WHERE {
  WINDOW <http://ex/w1> { ?s <http://ex/value> ?v }
  WINDOW <http://ex/w2> { ?s <http://ex/in> ?room }
}
FROM NAMED WINDOW <http://ex/w1> ON <http://ex/temp> RANGE 10 STEP 10
FROM NAMED WINDOW <http://ex/w2> ON <http://ex/meta> RANGE 10 STEP 10")?;

let temp = NamedNode::new_unchecked("http://ex/temp");
let meta = NamedNode::new_unchecked("http://ex/meta");
let triple = |s: &str, p: &str, o: Term| -> [Term;3] {
    [NamedNode::new_unchecked(format!("http://ex/{s}")).into(),
     NamedNode::new_unchecked(format!("http://ex/{p}")).into(), o] };
q.push(&meta, triple("s1", "in", NamedNode::new_unchecked("http://ex/kitchen").into()), 1, |_| {})?;
q.push(&temp, triple("s1", "value", Literal::from(21).into()), 2, |r| { /* joined rows */ })?;
q.flush(|_| {})?;
# Ok::<(), String>(())
```

**Just want windows, no SPARQL?** Use `WindowedStream` directly: `let mut ws = WindowedStream::empty(WindowSpec::time(10,10)); ws.push(t, ts); for w in ws.take_closed() { /* w.start, w.end, w.triples */ }`.

## Gotchas / feature flags / prerequisites

- **No cargo features, no async, no clock.** The crate has zero feature flags — depend on it and it's on. Timestamps are **application-supplied `u64`s** (logical ticks, epoch millis, sequence numbers — your scale); the engine never reads the wall clock. Time advances only through pushed timestamps. A quiet stream closes nothing until the next push or `flush()`.
- **Window semantics are half-open `[start, end)`** — start inclusive, end exclusive. `RANGE 10 STEP 10` gives `[0,10) [10,20) …` (a triple at `ts=10` is in `[10,20)` only). `step < range` ⇒ overlapping (sliding) windows; `step > range` leaves uncovered gaps (a gap triple enters no window but still advances the watermark). Origin defaults to `t0=0`; pre-`t0` arrivals belong to no window but advance the watermark.
- **Watermark + lateness.** A window closes when `max_ts_seen − max_delay` reaches its `end`. `with_max_delay(d)` is the out-of-order tolerance (default 0 = close at first sight of a newer-window triple). An arrival whose *every* covering window has already closed is dropped and counted in `late_dropped()`. `flush()` ignores `max_delay` and closes everything up to the last timestamp seen.
- **Empty windows are reported** (evaluated + delivered) when the watermark jumps a gap — DSTREAM needs to observe results disappear. Windows wholly closed before the first arrival's watermark are skipped (a stream starting at `ts=10⁹` won't replay a billion empties).
- **Materialisation is set-semantic:** a window is an RDF *graph*, so the same triple at several timestamps within one window counts once. CONSTRUCT results are triple sets (exact set-diff for I/DSTREAM); SELECT results are multisets diffed by 64-bit `FxHasher` row hashes (a hash collision could theoretically suppress a diff — accepted as vanishingly unlikely).
- **`register` rejects the wrong query form:** `ContinuousQuery` requires SELECT, `ContinuousConstruct` requires CONSTRUCT, `ContinuousAsk` requires ASK. Errors come back as `Err(String)` at registration. `push`/`flush` errors are engine evaluation errors.
- **`with_mode` must precede the first push** (switching mode resets stream state). Default `EvalMode::PersistentDict` wins every measured scenario (1.2–5.3× over `Rebuild`) and bounds dictionary memory to the *live* window vocabulary via refcount-exact compaction. `Rebuild` bounds memory to one window. `Delta` never wins in benchmarks (kept for huge-window / cheap-eval cases).
- **RSP-QL parser scope (`RspqlQuery::parse` / `ContinuousMultiQuery`):** parses `REGISTER [STREAM|RSTREAM|ISTREAM|DSTREAM] <out> AS`, `FROM NAMED WINDOW <w> ON <s> [RANGE <dur> [STEP <dur>]]` (tumbling when STEP omitted), and `WINDOW <w> { … }` (rewritten to `GRAPH <w> { … }`). Durations are ISO-8601 (`PT10S`, `PT1M30S`, `PT2H`, `P1D`; **seconds resolution**, years/months/weeks rejected) or bare integers (logical ticks). IRIs may be `<…>` or prefixed names resolved against the body's `PREFIX`/`BASE`. **Scoped out** (use the programmatic `WindowSpec` instead): window *variables* (`WINDOW ?w`), `ROWS` count windows, the `t0`/`max_delay` parameters, and relative `NOW-PT…TO…` bounds. `ContinuousMultiQuery` requires ≥2 windows (use `ContinuousQuery` for one) and currently only RSTREAM; ISTREAM/DSTREAM over a multi-window join is a documented follow-up (see the crate's open beads, `bd list -l area:sparq-rsp`).
- **Term model is `oxrdf`** (`oxrdf::Term`/`NamedNode`/`Literal`); stream elements are `[Term; 3]`. Add `oxrdf` with `features = ["rdf-12"]` to match the workspace.

## See also

- `hdt-format`, `fused-decompress-parse`, `rust-parallel-parsing` — bulk RDF ingest that can feed a stream.
- `sparql-formal-semantics` — the SPARQL algebra the embedded queries are evaluated under.
- The ZK/MPC sibling skills (`noir-circuit-patterns`, `mpc-protocols`, `verifiable-credentials-zk`) cover separate sparq crates; RSP is independent of them.
