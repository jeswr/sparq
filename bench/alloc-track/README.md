# bench/alloc-track — deterministic allocation-count instrument

> 🤖 SPARQ agent. sq-7d3dj.22 (epic sq-7d3dj,
> `research/optimization-program-delta-2026-07.md`).

The `op_*` measurement plans (sq-7d3dj.6/.7/.19/.16) all cite allocation counts, but
no counting allocator existed anywhere in the sparq estate. This harness fills that
gap: a `#[global_allocator]` counting wrapper emits **{allocs, peak\_bytes}** per
pinned `op_*` workload as a **TREND-ONLY** series, without touching any production
code or gated ratchet.

Like `bench/serve-throughput`, this is a **standalone cargo project** (own
`[workspace]`) so the repo's root workspace and its clippy/test gate are untouched.

## What it measures

For each of the 28 SPARQL operator queries in `bench/operators/queries/`:

| column | what |
|---|---|
| `allocs` | number of heap allocations made during `sparq_engine::query(…)` |
| `peak_bytes` | peak live heap bytes from a per-query-call baseline of 0 |
| `rows` | result row count (correctness anchor — must be stable) |
| `status` | `ok` (counts matched across iters) or `band[min,max]` (variance) |

The dataset (a deterministic WatDiv-flavoured social graph, default 2 000 entities) is
built once outside the measurement window; the per-query counters reset before each
`query(…)` call.

## Determinism

* **`allocs` (allocation count)** is fully cross-run deterministic: it depends only on
  the code paths exercised, not on wall-clock timing or heap layout. Identical across
  separate process invocations with the same binary.

* **`peak_bytes`** is **within-run** deterministic (the internal two-iteration check
  always agrees), but may vary ±O(10) bytes between separate process invocations. The
  root cause is that the system allocator may service the same logical sizes with
  slightly different internal alignment across process starts, shifting the
  `LIVE_BYTES` high-water mark by a few bytes. This does NOT affect `allocs`, which is
  the primary stable comparator. **Documented variance band**: ±~10 bytes on typical
  query workloads (< 0.1% of the measured peak).

Each query is run once as a warmup (to flush lazy initialisation) and then
`--iters - 1` times with counting. If all measured runs agree → `status = ok`. If they
diverge → the harness prints the variance band and exits 1.

Rayon is clamped to **one thread** (`ThreadPoolBuilder::num_threads(1)`) before any
work, so there are no concurrent allocator calls that could perturb the counts.

## Run

```sh
# in bench/alloc-track/
cargo run --release --bin alloc_track                    # default (scale=2000, iters=3)
cargo run --release --bin alloc_track -- --scale 2000 --iters 3
cargo run --release --bin alloc_track -- --smoke         # fast run (scale=200, iters=2)

# unit tests for the counting allocator invariants
cargo test --bin alloc_track
```

Flags: `--scale N` (number of synthetic entities; must be > 100 for all queries to
find their pinned subjects), `--iters N` (>= 2; first is warmup, rest are measured),
`--smoke` (sets scale=200, iters=2 for fast gate verification).

## Honesty — BENCH-ONLY, TREND-ONLY, NON-CANONICAL

Allocation counts are deterministic on the **same binary and same host**. They are NOT
canonical across compiler versions, std releases, or architecture changes. Use them
to compare before/after a single code change on the same box — never to claim an
absolute number. Nothing here is wired into `scripts/perf-gate.py` or any ratchet.

## Registration

Registered in `bench/benchmarks.toml` as `alloc-track` (category `query`, `featured =
false`). The `featured = false` flag marks it as an internal micro-bench that is
exempt from the dashboard-row requirement.

## Unblocks

sq-7d3dj.6 (ingest alloc reduction), sq-7d3dj.7 (bind-join alloc reduction),
sq-7d3dj.16, sq-7d3dj.19 — all of which cite allocation counts as the primary metric.

## License

MIT (workspace default).
