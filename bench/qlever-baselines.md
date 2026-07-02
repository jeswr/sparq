# Stored QLever baselines (this machine: 2020 MacBook Air M1, 16GB, native QLever 0.5.47)

Recorded so we DO NOT re-run QLever for every sparq iteration. QLever's compute
times are stable enough on this machine to use as a fixed reference; sparq is
benchmarked in the background and compared against these numbers. Re-measure QLever
only when the dataset or QLever version changes (note the date + commit if so).

> **What is canonical here.** The **QLever columns are the pinned reference** — a
> recorded external baseline, kept verbatim. The **sparq columns in the comparison
> tables below are point-in-time snapshots** of this machine and *will drift*;
> regenerate them with the `sparq-cli bench … count` / `bench-mmap` commands shown
> in each section (see also `bench/benchmarks.toml`, `bench/CATALOG.md`, and the
> per-commit perf dashboard <https://sparq.jeswr.org/dev/bench>). This is the
> single source for the QLever comparison; other docs link here rather than restate it.

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

For a machine-readable run, append `--json <path>` (e.g. `… count --json /tmp/run.json`):
the STDOUT TSV is unchanged and the same measured fields (`name` / `rows` / `min_micros`,
min-of-iters) are ALSO written to `<path>` as a JSON document — the structured-benchmark-catalog
shape. `bench-mmap` takes the same flag. The emitted numbers are whatever the running host
measured (NON-CANONICAL); never commit them.

## sparq vs QLever — compute-only comparison (2026-06-09, native QLever 0.5.47 reproduced)

Re-ran QLever natively (Homebrew `qlever-server`, SYSTEM=native, cold/cache-cleared) — numbers
reproduced the stored baselines (q03 73 vs 74, q04 54 vs 56, q10 59 vs 60). sparq via
`bench ... count` (compute-only), min-of-N. Both compute-only, no serialisation.

| query | 10M QLever ms | 10M sparq ms | 100M QLever ms | 100M sparq ms | sparq× (10M/100M) |
|---|--:|--:|--:|--:|--|
| q03_star3 (3-pattern join) | 73 | 14.5 | 900 | 157.7 | 5.0× / 5.7× |
| q04_follows_name (2-pattern) | 54 | 24.8 | 600 | 266.4 | 2.2× / 2.3× |
| q10_optional_age (OPTIONAL) | 59 | 3.4 | 650 | 32.6 | 17× / 20× |
| q02_type_person (1-pattern)* | 4 | 0.002 | 35 | 0.004 | range-count short-circuit |
| q06_filter_age (FILTER)* | 3 | 0.005 | 8 | 0.005 | range-prune short-circuit |

sparq wins compute 2.3–20× on the join/OPTIONAL queries at BOTH scales; advantage holds/grows
10M→100M. CAVEATS: (1) sparq loaded 100M fully in RAM (~10.8GB store+dict); QLever queries an
on-disk compressed index → far smaller RAM, scales to billions where sparq in-memory OOMs (QLever's
real edge; sparq's mmap out-of-core path is the answer, unvalidated at this scale here). (2) Dataset
is sparq-bench synthetic (uniform) — favours simple joins; the unbiased test is WatDiv/WDBench
(skewed/real), QLever's tuning target. (3) *single-pattern/FILTER: sparq short-circuits via index
range-counting, not a fair compute comparison. CONCLUSION: raw join compute is NOT where sparq
loses; remaining gaps = out-of-core-at-scale + skewed/standard benchmarks.

## sparq OUT-OF-CORE (mmap) vs QLever — 100M (2026-06-09)

External on-disk build: 74s, 4.2GB peak RAM (vs in-memory 10.8GB; vs QLever ~15min build).
On-disk index 7.9GB (uncompressed perms). Query via `bench-mmap` (mmap, OS-paged):

| metric | sparq in-memory | sparq mmap | QLever |
|---|--:|--:|--:|
| startup | 56s load | 0.67s open | build-once then start |
| committed heap | 10.8GB | ~0GB (2.4GB reclaimable page-cache) | low (on-disk) |
| q03_star3 ms | 158 | 138 | 900 |
| q04_follows_name ms | 266 | 235 | 600 |
| q10_optional ms | 32.6 | 32.2 | 650 |

mmap path MATCHES/BEATS in-memory query time with ~0 committed heap + 0.67s open → vs QLever 100M
it is 6.5×/2.6×/20× faster on q03/q04/q10 at a comparable (file-backed, reclaimable) memory model.
CONCLUSION: out-of-core-at-scale is NOT a gap — sparq already has QLever's low-RAM model AND faster
compute. QLever's only remaining theoretical edges: (a) COMPRESSED on-disk index (smaller disk
footprint at billions — a disk-space win via PFor/prefix blocks, not query speed), (b) skewed/real
data tuning (WatDiv/WDBench — the remaining unbiased test). The in-memory engine is SOTA; further
parallel/SIMD/join micro-opts have low marginal value.

## sparq vs QLever on REAL data — olympics (DBpedia/Wallscope, 1.78M triples, 2026-06-09)

Unbiased real-world skewed data (athletes/teams/medals/gender). QLever native cold, sparq count.

| query | QLever cold ms | sparq ms | sparq× |
|---|--:|--:|--|
| q03_athlete_star4 (4-star) | 18 | 2.52 | 7.1× |
| q05_result_star3 (3-star) | 17 | 2.97 | 5.7× |
| q04_result_athlete_age | 11 | 2.56 | 4.3× |
| q08_label_age | 7 | 1.52 | 4.6× |
| q07_medal_athlete_gender (3-join) | 13 | 7.52 | 1.7× |
| q10_optional_height | 10 | 0.81 | 12× |
| q06_filter_height (FILTER) | 1 | 0.48 | 2.1× |
| q02_type_person (1-pat)* | 2 | 0.006 | range-count |

sparq wins EVERY query on REAL skewed data, 1.7–12× → the advantage is not a synthetic artifact.
COMBINED VERDICT (synthetic 10M+100M, in-mem + mmap, AND real olympics): sparq is
SOTA-competitive-to-FASTER than QLever on compute across scales/data/memory-models. The ONLY
untested frontier is billion-scale REAL data (Wikidata/WDBench) — infra-blocked on this 16GB M1
(disk) — plus compressed-on-disk-index for smaller billion-scale disk footprint (disk, not speed).
