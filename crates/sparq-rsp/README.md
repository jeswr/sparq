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
| S2R | `WindowSpec` + `WindowedStream` | `RANGE w STEP s` time windows / `ROWS n [SLIDE s]` count windows, incremental |
| R2R | registered SPARQL SELECT | each closed window materialised into a `sparq_core::Graph`, evaluated by `sparq-engine` |
| R2S | `R2S::{RStream, IStream, DStream}` | full / added / removed rows per window, delivered as `WindowResult` callbacks |

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

## Window semantics (exact, pinned by tests)

**Time windows** (`WindowSpec::time(range, step)`):

- Windows are **half-open intervals `[k·step, k·step + range)`**, `k = 0, 1, 2, …`
  — start bound **inclusive**, end bound **exclusive**. `RANGE 10 STEP 10`
  yields `[0,10) [10,20) …`: a triple at `ts = 10` belongs to `[10,20)` only —
  tumbling windows partition the timeline with no double counting.
  (RSP-QL parameterises the window origin `t0`; we fix `t0 = 0` — a documented
  divergence that keeps window identity canonical.)
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
  closed at the initial watermark (first accepted `ts − max_delay`) are skipped
  (a stream starting at `ts = 10⁹` does not replay a billion empties), and the
  lateness contract holds across the first push: a first push at `ts = 12` with
  `max_delay = 5` leaves `[0,10)` open for a later `ts = 8`.
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

## Throughput

`cargo run --release -p sparq-rsp --example throughput` — 1 M synthetic sensor
readings (100 sensors, 1 triple per tick), tumbling time windows, Apple M1,
rustc 1.89:

| scenario | throughput | windows/s |
|---|---|---|
| windowing only (no query), RANGE 1000 | 2.26 M triples/s | 2 257 |
| `AVG(?v)` per window, RANGE 100 (100 triples/window) | 1.14 M triples/s | 11 446 |
| `AVG(?v)` per window, RANGE 1000 | 1.66 M triples/s | 1 661 |
| `AVG(?v)` per window, RANGE 10000 | 1.66 M triples/s | 166 |
| `AVG(?v)` per sensor (GROUP BY), RANGE 1000 | 1.45 M triples/s | 1 454 |

The dominant cost is per-window: each closed window builds a fresh
dictionary-encoded graph and runs the engine, so throughput scales with
triples-per-window until materialisation dominates (~1.6 M triples/s plateau).
Windowing alone (push + eviction) is `BTreeMap` insert + range-read +
`split_off` per window — the 2.3 M triples/s line.

## Tests

`cargo test -p sparq-rsp` — 22 integration tests + 2 doctests pinning:
boundary inclusivity (`[start, end)`), tumbling partition / sliding overlap,
empty-window reporting, `step > range` gaps, out-of-order within `max_delay`
vs. too-late drops, ROWS / SLIDE / arrival-order membership, scripted ISTREAM
and DSTREAM traces (including disappearance via an empty window), multiset
diff semantics, set-semantic materialisation, register-time validation, and
end-to-end AVG-per-window.
