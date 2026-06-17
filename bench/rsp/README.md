<!-- [OPUS-4.8] sq-b1hn — RSP-QL streaming benchmark suite. Design: research/capability-benchmark-program.md §3.6. -->
# RSP-QL streaming suite

The RSP-QL analogue of the LUBM / SHACL / FTS template — but with a deliberately
**different shape**, because `sparq-rsp` is a **deterministic, clock-free
library**, not a wall-clock service engine. The suite is therefore built around a
**correctness gate**, not a throughput head-to-head.

It exercises `sparq-rsp`: windowed continuous SPARQL (S2R `RANGE/STEP` windows →
R2R per-window materialisation → R2S `RSTREAM`), the three `EvalMode`s (Rebuild /
PersistentDict / Delta), and the RSP-QL surface syntax + multi-window
named-graph joins.

## Why no competitor perf column (honest)

The natural RSP peers — **C-SPARQL, CQELS, RSP4J / YASPER** (and the
**SRBench / CityBench / LSBench / RSPLab** harnesses) — are **service/runtime
engines with wall-clock windows**: a window closes when *real time* passes.
`sparq-rsp` is clock-free — a window closes when the *pushed-timestamp watermark*
passes, so its output is a pure function of the `(triple, ts)` sequence with no
scheduler, no real clock. **Any throughput head-to-head across the two is
apples-to-oranges (different time model)** — it is the surface where a competitor
*perf* column is least meaningful. So this suite has **no competitor perf
column**. The honest comparison is **correctness / expressivity**, run as the
SRBench correctness oracle below (the EYE role). Perf head-to-head is explicitly
**out of scope**.

## The gate: per-window result-row counts (clock-free → deterministic)

Because the pipeline is clock-free, replaying a **fixed `(triple, ts)` script**
yields a fully **deterministic** per-window result. The gate is a diff of the
per-window **result-row count** against `expected.tsv` — a **stronger** gate than
any wall-clock RSP benchmark can offer (no timing flake, no scheduler
nondeterminism). `run.sh` asserts every metric and exits non-zero on any drift,
exactly like LUBM's count diff.

Two metric families, both asserted:

| family | what | meaning of the count |
|---|---|---|
| `rsp_<scenario>_<mode>_w<k>_rows` | single-window scenario × `EvalMode` × window `k` | result rows that window emitted |
| `rsp_srbench_<q>_w<k>_rows` | SRBench oracle query × synchronized window `k` | rows the multi-window join produced |

### Single-window scenarios (× three EvalModes)

A fixed 13-element sensor script (multi-sensor, a duplicate triple for set
semantics, an empty window for the gap case, boundary-internal arrivals) replayed
through three scenarios:

- **`tumbling_avg`** — `AVG` over `RANGE 10 STEP 10` (one aggregate row per
  window, including empty windows).
- **`sliding_sum`** — `SUM` per sensor over `RANGE 20 STEP 10` (50 % overlap; one
  row per distinct sensor present in each overlapping window).
- **`tumbling_groupby_join`** — room ⋈ value `GROUP BY` over `RANGE 20 STEP 20`.

Each runs under **all three `EvalMode`s** (Rebuild / PersistentDict / Delta), and
`expected.tsv` pins them to the **same** per-window counts — so these rows ALSO
encode the **three-EvalMode-equivalence** assert (a regression in any one mode
fails the gate). This complements the differential streamed-equals-batch oracle
already in `crates/sparq-rsp/tests/window_oracle.rs` (which proves each window
equals the batch query over its tuples); the bench pins the *shape* (window
count + row count) of a fixed replay so a regression alerts on the dashboard.

### SRBench correctness oracle (the EYE role)

[SRBench](https://link.springer.com/chapter/10.1007/978-3-642-35176-1_40) is the
canonical RSP correctness/expressivity suite over real LOD **weather** streams:
sensor **observations** joined to **station metadata**. We reproduce that *shape*
deterministically — an observations stream (`<station> <value> v`) and a
station-metadata stream (`<station> <state> <usstate>`, re-announced each window
the way a metadata stream emits a per-window snapshot), joined per synchronized
window via `ContinuousMultiQuery` (cross-named-graph join, the RSP-QL `WINDOW
<w1> { … } WINDOW <w2> { … }` form). Two queries:

- **`srbench_join`** — annotate each observation with its station's US state
  (the canonical "enrich the stream from the background graph" pattern). An
  observation from a station with **no metadata** is correctly dropped by the
  inner join; a station observed twice in a window is deduplicated to set
  semantics.
- **`srbench_groupby_state`** — readings per US state per window (the join feeds
  a `GROUP BY` — aggregate-after-join expressivity).

This is correctness/expressivity coverage only; the time model differs from a
wall-clock RSP engine, so it carries **no perf comparison**.

## Advisory throughput (trend-only, NOT a gate)

The runner also emits a single `rsp_persistentdict_triples_per_s` line: the
PersistentDict (default) `EvalMode` throughput pushing 200 k synthetic readings
through a `RANGE 1000` continuous query. It is **machine-sensitive and trend-only**
— NOT in the perf gate. The deterministic per-window counts above are the real
gate. The fuller `EvalMode` head-to-head (1 M readings, all scenarios) lives in
`crates/sparq-rsp/examples/throughput.rs` (the `rsp-throughput` registry entry).

## Files

| file | role |
|---|---|
| `crates/sparq-rsp/examples/rsp_oracle.rs` | the TSV-emitting runner (the crate is isolated — not a `sparq-cli` dependency, so the runner is a crate example, like FTS's `bench_text`) |
| `expected.tsv` | the deterministic per-window row-count gate (single-window × 3 EvalModes + SRBench oracle) |
| `run.sh` | self-asserting entry point CI calls: run the example, assert every `*_rows` metric vs `expected.tsv`, forward the 3-column `<metric>\t<value>\t<unit>` hook contract |

## Run it

```sh
cargo build --release -p sparq-rsp --example rsp_oracle
bench/rsp/run.sh                       # asserts + prints the metric TSV; exit 1 on any drift
```

A divergence in `expected.tsv` means the RSP windowing / materialisation /
eval-mode / multi-window-join semantics changed — regenerate only after confirming
the change is intended (and update the differential oracle test if so).
