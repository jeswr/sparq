# Stored QLever baselines (this machine: 2020 MacBook Air M1, 16GB, native QLever 0.5.47)

Recorded so we DO NOT re-run QLever for every sparq iteration. QLever's compute
times are stable enough on this machine to use as a fixed reference; sparq is
benchmarked in the background and compared against these numbers. Re-measure QLever
only when the dataset or QLever version changes (note the date + commit if so).

All times are **min-of-N cold** query-time-ms reported by QLever (`query-time-ms`),
COUNT(*)-wrapped queries (the `queries-count/` set) — i.e. compute-only, no JSON
serialisation. Machine load varies ±30%, so treat these as the reference band.

## 10M synthetic (bench/qlever-synthetic, 1.25M entities × 8)

| query | QLever compute (ms) | notes |
|---|--:|---|
| q02_type_person   | 4   | COUNT ?s a Person |
| q03_star3         | 74  | 3-pattern star on ?s |
| q04_follows_name  | 56  | follows · name |
| q06_filter_age    | 2   | FILTER(?a > 90) |
| q10_optional_age  | 60  | OPTIONAL |

## 100M synthetic (bench/qlever-100m, 12.5M entities × 8) — recorded 2026-06-08

| query | QLever compute (ms) | observed range |
|---|--:|---|
| q02_type_person   | 35  | 30–47 |
| q03_star3         | 900 | 829–1195 |
| q04_follows_name  | 600 | 529–1098 |
| q06_filter_age    | 8   | 6–10 |
| q10_optional_age  | 650 | 591–866 |

QLever index build (100M, native): parse ≈ 0.15 M/s (~13 min — slow single-stream
parser on macOS), then permutations; "Index build completed" ~15 min total.

## How to compare sparq without re-running QLever

Run sparq alone (background):
  `target/release/sparq-cli bench bench/qlever-100m/synthetic100m.nt ntriples \
     bench/qlever-100m/queries <iters> count`
then read each query's µs from stdout and divide the QLever ms above by it.
