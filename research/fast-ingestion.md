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
| **1** | **Parallel decompression** — `lbzip2 -dc` (bzip2 is block-based → ~7×, ~21 min) or recompress source to **zstd** once (`bzcat \| zstd -9 -T8`, ~16 min, parallelizable) | E2E decompress 147 → ~16–21 min | Low | **TODO** (needs `lbzip2` install or one-time recompress) |
| **2** | **Overlap decompress with parse** — decompress on its own thread feeding a bounded channel, so it runs concurrently with parse+spill instead of additively | hides the parse under the decompress | Low | **DONE** (commit 48b9c22 — `build_external_ntriples_parallel`; measured ~0.39 → ~0.96 M/s on a `.bz2`, ~2.4×) |
| **3** | **Radix sort** the permutations (MSD on the leading u32 — ids are dense) instead of serial `sort_unstable` in `build_raw_perms`/`external_sort` | build stage 2–4× | Medium | TODO |
| **4** | **Parallelize `external_sort`** across the 5 sibling permutations (and `par_sort` each run); start sibling sorts before the SPO merge completes | build 2–3× more (eff. cores 1.4 → 8) | Medium | TODO |

`lbzip2`/zstd (#1) is the single biggest win — bzip2 is 70% of E2E wall-time and ~7×
parallelizable. #2 (done) overlaps the rest. #3–#4 then stop the build stage from being
the second wall once decompression is parallel.

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
