# sparq-rsp

RSP-QL-style RDF stream processing for the [sparq](../../README.md) engine:
windowed **continuous SPARQL queries** over timestamped triple streams, as a
deterministic **library** — no async runtime, no wall clock, no service.

```rust
use oxrdf::{Literal, NamedNode, Term};
use sparq_rsp::{ContinuousQuery, R2S, WindowSpec};

// Average reading per tumbling 60-tick window, tolerating 5 ticks of disorder.
let mut q = ContinuousQuery::register(
    "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }",
    WindowSpec::time(60, 60).with_max_delay(5),   // RANGE 60 STEP 60
)?
.with_r2s(R2S::RStream);                          // the default: full result per window

q.push(triple, ts, |result| {                     // fires once per CLOSED window
    println!("[{}, {}): {:?}", result.start, result.end, result.rows);
})?;
q.flush(|result| { /* end-of-stream: close everything up to max ts */ })?;
```

The classic RSP-QL pipeline, one type each:

| stage | type | role |
|---|---|---|
| stream | `TripleStream` / push API | `(triple, u64 timestamp)` elements, app-supplied timestamps |
| S2R | `WindowSpec` + `WindowedStream` | `RANGE w STEP s [t0]` time windows / `ROWS n [SLIDE s]` count windows, incremental |
| R2R | registered SPARQL | each closed window materialised into a `sparq_core::Graph` (per `EvalMode`, below), evaluated by `sparq-engine` |
| R2S | `R2S::{RStream, IStream, DStream}` | full / added / removed rows per window, delivered as `WindowResult` callbacks |

Three query forms: `ContinuousQuery` (SELECT → `WindowResult` rows),
`ContinuousConstruct` (CONSTRUCT → `GraphResult` triples — stream-to-stream
transformation, with R2S as exact set diffs over the constructed graphs), and
`ContinuousAsk` (ASK → one `AskResult` boolean per window).

## Design: deterministic by construction

- **The engine never reads a clock.** Timestamps are application-supplied
  `u64`s (logical ticks, epoch millis, sequence numbers — your choice of
  scale). Time advances only through pushed timestamps, via the watermark
  `max_ts_seen − max_delay`. The entire pipeline is a pure function of the
  pushed `(triple, ts)` sequence: replayable, unit-testable, wasm-safe.
- **No tokio / threads.** Synchronous push + callback. Wrapping pushes in an
  async runtime, a thread, or a browser timer is the embedder's one-liner, not
  this crate's dependency tree.
- **Isolated crate** (the `sparq-reason` / `sparq-shacl` pattern): nothing in
  the workspace depends on it; the core engine and the wasm build carry zero
  streaming code.

## RSP-QL surface syntax + multi-window joins

Beyond the programmatic API above, the crate parses the **RSP-QL textual query
language** and joins across **multiple named windows**:

```rust
use sparq_rsp::ContinuousMultiQuery;

// Join sensor READINGS (stream :temp, window :w1) with the ROOM each sensor is
// in (stream :meta, window :w2), per synchronized tumbling window.
let mut q = ContinuousMultiQuery::register("\
REGISTER STREAM <http://ex/out> AS
SELECT ?room ?v WHERE {
  WINDOW <http://ex/w1> { ?s <http://ex/value> ?v }
  WINDOW <http://ex/w2> { ?s <http://ex/in> ?room }
}
FROM NAMED WINDOW <http://ex/w1> ON <http://ex/temp> RANGE 10 STEP 10
FROM NAMED WINDOW <http://ex/w2> ON <http://ex/meta> RANGE 10 STEP 10")?;

q.push(&meta_stream, triple, ts, |r| { /* full join result per tick */ })?;
```

- **Parser** (`RspqlQuery::parse`): `REGISTER [STREAM|RSTREAM|ISTREAM|DSTREAM]
  <out> AS`, `FROM NAMED WINDOW <w> ON <s> [RANGE <dur> [STEP <dur>]]` (tumbling
  when `STEP` is omitted), and `WINDOW <w> { … }` rewritten to standard SPARQL
  `GRAPH <w> { … }` — spargebra parses the embedded BGP/algebra. Durations are
  ISO-8601 (`PT10S`, `PT1M30S`, `P1D`; seconds resolution) or bare integers
  (logical ticks). IRIs may be `<…>` or prefixed names resolved against the
  body's `PREFIX`/`BASE`. **Scoped out:** window variables (`WINDOW ?w`), `ROWS`
  count windows, the `t0`/`max_delay` parameters, and relative `NOW-PT…TO…`
  window bounds (use the programmatic `WindowSpec` for those).
- **Multi-window join** (`ContinuousMultiQuery`): each declared window keeps its
  own S2R state over the stream it reads; pushes are tagged with their stream.
  All windows share one event-time clock (a triple on one stream advances the
  watermark of windows on the others, via `WindowedStream::advance`), so closure
  is synchronized. At each tick every window contributes its latest-closed
  content as a **named graph** keyed by the window IRI; the engine's
  cross-named-graph join then binds variables shared between `WINDOW <w1>` and
  `WINDOW <w2>`. RSTREAM (full per-tick result) for now; ISTREAM/DSTREAM over a
  multi-window join is a documented follow-up (see `TODO.md`).

## Window semantics (exact, pinned by tests)

**Time windows** (`WindowSpec::time(range, step)`):

- Windows are **half-open intervals `[t0 + k·step, t0 + k·step + range)`**,
  `k = 0, 1, 2, …` — start bound **inclusive**, end bound **exclusive**.
  `RANGE 10 STEP 10` yields `[0,10) [10,20) …`: a triple at `ts = 10` belongs
  to `[10,20)` only — tumbling windows partition the timeline with no double
  counting. The RSP-QL window origin `t0` defaults to 0 and is set with
  `.with_t0(t0)`; an arrival before `t0` belongs to no window (not counted
  late — no window ever covered it) but still advances the watermark.
- `step < range` ⇒ sliding windows **overlap**: with `RANGE 10 STEP 5`, a
  triple at `ts = 7` is in both `[0,10)` and `[5,15)`.
- A window **closes** — its content is frozen, the query runs, the callback
  fires — when the watermark (`max_ts − max_delay`) reaches its end.
  `with_max_delay(d)` is the out-of-order tolerance: arrivals up to `d` ticks
  behind the newest timestamp still land in their windows. A triple whose
  *every* covering window has closed is dropped and counted
  (`late_dropped()`).
- **Empty windows are reported** (evaluated and delivered) when the watermark
  jumps a gap — DSTREAM requires observing results *disappear*. Windows wholly
  closed at the initial watermark (the first arrival's `ts − max_delay`) are
  skipped (a stream starting at `ts = 10⁹` does not replay a billion empties),
  and the lateness contract holds across the first push: a first push at
  `ts = 12` with `max_delay = 5` leaves `[0,10)` open for a later `ts = 8`.
- `step > range` leaves uncovered gaps; a gap timestamp belongs to no window
  and is not "late" — but it still advances the watermark (event time passed),
  closing earlier windows.
- `flush()` is end-of-stream: closes every window up to the last timestamp
  seen, ignoring `max_delay`.

**Count windows** (`WindowSpec::count(rows)`, CQL-style): the last
`min(rows, arrivals)` triples in **arrival order** (timestamps carried but
irrelevant to membership), reported on every arrival — or every `slide`-th
with `.with_slide(s)`. Reported bounds are the inclusive `[first.ts, last.ts]`
of the content.

**Materialisation is set-semantic:** a window is an RDF *graph*, so the same
triple at several timestamps within one window counts once.

## R2S semantics

SPARQL SELECT results are multisets of rows; diffs respect that:

- **RSTREAM** — the full result of every window (RSP-QL default, stateless).
- **ISTREAM** — multiset difference `current ∖ previous`: rows *added* since
  the previous window. First window diffs against empty (emits everything).
- **DSTREAM** — `previous ∖ current`: rows *removed*. Emits nothing for the
  first window; a row "disappears" only when a later (possibly empty) window
  closes without it — rows in the final window are never DSTREAMed.

Diffs are computed as **64-bit row hashes** (`FxHasher` over the bound terms)
counted as multisets — O(rows) per window, no row sorting. Emission order is
deterministic: ISTREAM keeps the engine's row order of the current window,
DSTREAM that of the previous window. (A 64-bit hash collision between two
distinct rows of one query could suppress a diff; accepted as vanishingly
unlikely.)

## Evaluation modes (R2R materialisation)

How each closed window becomes the graph the engine evaluates is the
`EvalMode` (`.with_mode(…)`, same results in all three — pinned by tests):

- **`Rebuild`** — the v1 baseline: fresh dictionary + fresh indexes per
  window. Nothing persists between windows, so memory is bounded by one
  window even on unbounded-vocabulary streams.
- **`PersistentDict`** (default) — ONE dictionary for the continuous query's
  lifetime: terms interned once at push time, per-window graphs built from
  already-interned `[Id; 3]`s via `Graph::from_parts`. Removes term
  hashing/allocation from the window loop. The dictionary is COMPACTED as terms
  age out of every live window (refcount-exact liveness: a term is kept iff some
  live window still references it), so it stays bounded by the live window
  vocabulary rather than growing with the all-time vocabulary — a 30 000-tick
  churning-vocabulary stream (all-time vocab 60 001) peaks at ~2 063 dictionary
  terms. `ContinuousQuery::dict_len()` exposes the live size.
- **`Delta`** — ONE live graph + `Graph::apply_delta(inserts, deletes)` per
  slide (set-semantic diff between consecutive windows), compacted when the
  pending overlay outgrows the window. Measured slower than `PersistentDict`
  everywhere (see below); kept because its per-slide work is O(changes).

## Throughput

`cargo run --release -p sparq-rsp --example throughput` — 1 M synthetic sensor
readings (100 sensors, 1 triple per tick), time windows, Apple M1, rustc 1.93
(triples/s; windowing only — no query — runs at 2.33 M triples/s):

| scenario | Rebuild (v1) | PersistentDict | Delta |
|---|---|---|---|
| `AVG(?v)`, RANGE 100 STEP 100 (tumbling) | 1.13 M | **1.51 M** | 1.07 M |
| `AVG(?v)`, RANGE 1000 STEP 1000 (tumbling) | 1.55 M | **1.93 M** | 1.87 M |
| `AVG(?v)`, RANGE 10000 STEP 10000 (tumbling) | 1.61 M | **3.08 M** | 1.70 M |
| `AVG(?v)`, RANGE 1000 STEP 100 (sliding 10×) | 0.26 M | **0.95 M** | 0.29 M |
| `AVG(?v)`, RANGE 10000 STEP 1000 (sliding 10×) | 0.27 M | **1.44 M** | 0.36 M |
| `AVG` per sensor (GROUP BY), RANGE 1000 | 1.68 M | **2.52 M** | 2.01 M |

`PersistentDict` wins every scenario (1.2–5.3× over the v1 rebuild — the
sliding rows are exactly the "~90 % of each build is redone work" case the v1
TODO predicted) and is the default. `Delta` never wins: `apply_delta` works at
the term level (interning inserts, `id_of` per delete) and overlay rows are
re-sorted per scan, so its savings are eaten before the engine runs. The
remaining per-window cost in `PersistentDict` is the index build
(`TripleStore::from_triples`) plus the numeric/temporal caches (O(dictionary)
per window) — removing those needs the core cheap-snapshot seam (see TODO).

The registered query is parsed ONCE at `register` time into a
`sparq_engine::PreparedQuery`; each window executes the prepared algebra
(no per-window parse). Parsing the AVG query above costs ~2.6 µs, so this
only shows up at very high window rates: RANGE 10 tumbling windows dropped
~17 % per window (11.8 → 9.8 µs median, interleaved A/B); at RANGE 100 and
above the saving is within run-to-run noise.

## Tests

`cargo test -p sparq-rsp` — 33 integration tests + 3 doctests pinning:
boundary inclusivity (`[start, end)`), tumbling partition / sliding overlap,
window origin `t0` (shifted bounds, pre-origin arrivals, anchor far from the
origin), empty-window reporting, `step > range` gaps, out-of-order within
`max_delay` vs. too-late drops, ROWS / SLIDE / arrival-order membership,
scripted ISTREAM and DSTREAM traces (including disappearance via an empty
window), multiset diff semantics, set-semantic materialisation, three-way
eval-mode equivalence (incl. delta compaction + multi-timestamp eviction),
PersistentDict dictionary compaction (bounded growth under high vocabulary
churn, results identical to the uncompacted reference),
CONSTRUCT (RSTREAM/ISTREAM/DSTREAM set diffs) and ASK per window,
register-time validation of all three query forms, and end-to-end
AVG-per-window.
