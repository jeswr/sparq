# Fast Wikidata ingestion — bottleneck analysis + pipeline (measured on M1)

Goal: ingest the full Wikidata "truthy" N-Triples dump *as fast as physically possible*
on a consumer machine (Apple M1, 8 cores, 16 GB). Aspiration: subsecond. Reference:
RDFox reports ~24 min. All numbers below are **measured on this M1** against the real
dump and the actual sparq release binary (except where marked "est").

## Headline

- The dump is **~9.4 B triples / ~1.08 TB decompressed** (42.8 GB `.bz2`, an unusually
  high ~25× ratio).
- **Subsecond is physically impossible.** Even reading the decompressed bytes once at
  3 GB/s is ~6 min; the 42.8 GB → 1.08 TB decompress is irreducible. **Honest, defensible
  target on this M1: ~20–25 min — RDFox-competitive**, gated almost entirely by parallel
  decompression.
- **bzip2 is the bottleneck**: single-stream `bzcat` runs at **123 MB/s out ≈ 1.06 M
  triples/s** → ~147 min for the full file no matter how fast everything downstream is.

## The four levers (prioritized)

| # | Change | Est. impact | Difficulty | Status |
|--|--|--|--|--|
| **1** | **Fast decompression** — recompress source to **zstd** once (`zstd -9 -T0`, parallel), then `.zst` input decompresses ~12× faster than bzip2 so it stops being the bottleneck (the `lbzip2` alternative needs an install that isn't present) | E2E decompress 147 → ~16 min | Low | **DONE** (commit 4903da1 — `.zst`/`.zstd` input in build/ingest; 10M build from `.zst` 4.6 s vs `.bz2` 10.4 s vs `.nt` 4.4 s — zstd decompress fully hidden under the parse by the #2 overlap pipeline, so compressed ingest now runs at ~uncompressed speed) |
| **2** | **Overlap decompress with parse** — decompress on its own thread feeding a bounded channel, so it runs concurrently with parse+spill instead of additively | hides the parse under the decompress | Low | **DONE** (commit 48b9c22 — `build_external_ntriples_parallel`; measured ~0.39 → ~0.96 M/s on a `.bz2`, ~2.4×) |
| **3** | ~~**Radix sort** the permutations (MSD on the leading u32 — ids are dense) instead of `sort_unstable` in `build_raw_perms`/`external_sort`~~ | build stage 2–4× (claimed) | Medium | **REJECTED — measured (sq-56z).** The premise (replace a *serial* sort) is stale: both sites already use `par_sort_unstable` under the default `parallel` feature, against which a best-effort MSD radix is a **regression**. See "Radix-sort verdict (sq-56z)" below. |
| **4** | **Parallelize `external_sort`** across the 5 sibling permutations (each in its own tmp subdir, sharing the chunk budget) | build stage faster (eff. cores → ~5) | Medium | **DONE** (commit c391dde — external build 10M from `.nt` 6.8s → 4.4s, −35%) |

zstd (#1, now done) is the single biggest win — bzip2 was 70% of E2E wall-time. With #2
(overlap, done) hiding the now-fast zstd decompress under the parse, compressed ingest runs
at ~uncompressed speed (measured 4.6 s vs 4.4 s on 10M). #3 (radix) was the last open lever
and is now **measured and rejected** (see below); #4 (parallel sibling sorts, done) already
removed the build stage as the second wall.

### Radix-sort verdict (sq-56z) — REJECTED after measurement

The bead flagged this as "measure-first / questionable on the bandwidth-bound M1", and the
measurement confirms the doubt. The sort hot paths — `build_raw_perms` (SPO dedup + sibling
permutations) and the `spill_run` tail under `external_sort` — **already use rayon
`par_sort_unstable`** under the default `parallel` feature (they did `sort_unstable` when this
doc was first written; that was changed independently). A direct comparison on synthetic
`[u32;3]` permutation rows across four distributions (Wikidata-like, dense, small per-run
chunk, and POS-like low-leading-cardinality) gives, *relative to the real baseline*:

- A serial MSD radix beats the **serial** `sort_unstable` by ~1.3–2.0× — but no hot path
  uses the serial sort in the shipped (parallel) build, so that win is unreachable.
- Every parallel MSD radix variant we could write (naive leading-byte partition; and a
  load-balanced variant that does a parallel chunked histogram and even falls back to
  `par_sort_unstable` for the big buckets) is **slower than plain `par_sort_unstable`** —
  roughly **0.5–0.7×** its throughput (i.e. `par_sort_unstable` is ~1.4–1.8× faster), with
  no distribution where radix wins.

Why: the radix scatter pass is a random-write memory shuffle, which is exactly the
bandwidth-bound bottleneck the bead named — and rayon's `par_sort_unstable` (a cache-aware
parallel driftsort) is already very well tuned. The "Sort traffic floor" row above
(6 perms × 3 passes) is the same memory-traffic ceiling radix would have to beat and cannot.
(Ratios are relative algorithm comparisons measured on the non-canonical -aws work box; they
are directional, not the canonical M1 throughput numbers, and so are stated as ratios only.)

**Decision:** keep `par_sort_unstable`; do not add a radix path. Re-open only if a future
profile shows the index BUILD stage (not decompress/parse) is the binding ingest constraint
*and* a radix variant is shown by measurement to beat `par_sort_unstable` on real id rows.

**Recipe for the full Wikidata dump:** `zstd -9 -T0` the 42.8 GB `.bz2` → `.zst` once
(parallel, one-time), then `sparq-cli build dump.nt.zst ntriples out 8` — ingest is now
parse/sort-bound (no decompress wall), projected ~30 min for 9.4 B triples,
RDFox-competitive (their 24 min).

## Stage measurements (10M-class, this M1)

| Stage | Throughput | Notes |
|---|--|--|
| bzip2 single-stream decompress | 123 MB/s ≈ 1.06 M/s | the serial ceiling |
| in-process `bzip2::MultiBzDecoder` (old, serial) | even slower than `bzcat`, no overlap | was ~70% of E2E |
| **external build from `.bz2`, OLD serial** | **~0.39 M/s** | decompress + parse additive |
| **external build from `.bz2`, overlapped (now)** | **~0.96 M/s (~2.4×)** | decompress hidden under parse |
| external build from decompressed `.nt` | ~1.13 M/s | parse+build, no decompress |
| custom parallel byte parser (parse+intern, no decompress) | 2.49 M/s | already zero-copy, no `oxrdf::Term` |
| parse alone (est) | ~5 M/s | SIMD/`memchr` newline scan → 8+ M/s, but not the bottleneck |

## Theoretical floors

| Floor | Time |
|---|--|
| Read 42.8 GB `.bz2` off SSD | ~14 s (negligible) |
| Read 1.08 TB decompressed once @ 3 GB/s | ~6 min |
| Sort traffic floor (6 perms × 3 passes @ 60 GB/s) | ~34 s |
| RDFox reference (24 min) | = 6.5 M/s |

## The one mandatory serialization point

`Dict::merge_remap` — it assigns global ids, so the per-block partial-dict merge must be
serial. Everything else (decompress, parse, the 6 permutation sorts) parallelizes. The
parser SIMD work (#6 in the agent list) is low-priority — it's not the binding constraint
until decompression and sort are fixed.
