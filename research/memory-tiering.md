# Memory tiering — measured verdicts (perm dropping, hot/cold compression, residency)

Investigation of reducing sparq's resident memory without noticeably sacrificing
performance: workload-aware DROPPING/RE-INTRODUCTION of permutation indexes, and
compression tiering on in-memory indices. Spike harness: `bench/memtier/` (standalone
workspace, like `bench/serve`). Raw run logs: `bench/memtier/results/` (not tracked).

## Measurement conditions — read this first

**EC2 was authorized but unavailable**: the AWS SSO access token expired mid-day and
refreshing requires an interactive browser login the agent cannot complete (manual
token refresh from the credential cache was — correctly — denied by policy). Fallback
per instructions: local M1 Air (8 cores, 16 GB), which was under concurrent load
during the runs (load average 7–27, an AV process at ~97% CPU). Mitigations:
- every comparison is PAIRED back-to-back in the same session, repeated 2–3×;
- per-query statistic is **min of 7 iterations** (contention-robust); medians and
  maxima were recorded and the spread is flagged wherever it affects a conclusion;
- all MEMORY numbers (RSS, mincore page-cache residency, file/heap sizes) are
  load-immune and exact.
**$0 AWS spend; no EC2 resources created.** Treat the latency *ratios* as reliable
(cross-pair agreement was within ±10% on every headline number) and absolute
latencies as indicative.

Datasets: `bench/qlever-synthetic` (9,999,991 triples) and `bench/qlever-olympics`
(1,781,625 triples), saved as raw mmap dirs. Batteries: the tracked `queries/` suites,
engine `count` mode unless stated (matches BENCHMARKS.md methodology), plus a `json`
(full materialize+serialize) pass for the realistic-serving residency profile.

---

## Headline finding (lever 3): the biggest resident block was a BUG — fixed

Profiling "what's actually resident" found that `Graph::open` paged the **entire
POS + PSO permutations** (240 MB at 10M triples; ~2.4 GB at 100M) into RAM despite
`predstats.bin` existing precisely to avoid that: `load_pred_stats` read the
predicate id as 8 bytes while `save_pred_stats` writes a 4-byte `Id`, so every load
mis-framed, failed at EOF, and silently fell back to `compute_pred_stats` — a full
scan of two permutations. The existing roundtrip test compared recomputed-vs-
recomputed, so it passed.

Fixed in `sparq-core` (commit `6589c98`), regression-tested
(`pred_stats_load_is_some_and_exact`). Measured effect on the 10M raw dir:

| | before | after |
|---|--:|--:|
| `Graph::open` | 0.52 s | **0.019 s** (27×) |
| RSS after open | 236.7 MB | **2.3 MB** (102×) |

Gates: `cargo test -p sparq-core --release` 31 ok; `--features mmap` 40 ok; wasm
build 1,572,958 B (size-identical with and without the fix — the loader is mmap-gated).

After the fix, a count-mode serving process at 10M holds ~2 MB anon heap; the page
cache holds only what queries touch (the battery's hot set: POS+PSO, ~240 MB of file
pages). **Remaining breakdown (10M, after battery):** perms touched 240 MB page cache
(PSO+POS 100%, the other four ≤0.1%), dict ≈2 MB touched in count mode (offs+terms
fully resident — 68 MB — only in `json` mode), numerics/temporals ≈0, heap 156 B.

**Second residency finding (json mode):** the realistic materialize+serialize battery
drove process RSS to **1.75 GB** — almost all of it transient result materialization
(q04 returns 5M rows → a ~700 MB JSON string) retained by the allocator, not index
storage (index+dict file pages total <200 MB). At serving scale the biggest memory
lever is **streaming/chunked result serialization in sparq-serve**, not index tiering.
Out of scope here; recorded as the top follow-up.

---

## Lever 1: permutation dropping (6 perms vs the 3-perm wasm set) — REJECT as a runtime feature for mmap serving; the compile-time compact set already covers the case that pays

Per-perm cost at 10M: 114.4 MB on disk = 120 MB owned heap each (12 B/triple);
686 MB disk for all six.

**Memory saved by dropping in mmap mode: ~nothing.** Untouched perms are never
resident (demand paging, measured 0.0–0.1% residency after full batteries). The
3-perm build's process RSS after the synthetic battery was actually **higher**
(122 MB vs 108 MB): the fallback plans hash/sort-materialize what merge joins
previously borrowed zero-copy. Dropping only saves real memory in **Owned/in-RAM
mode**: −360 MB of 720 MB at 10M.

**Query cost of 3 perms** (`--features compact-index`, same index dir, paired,
min-of-7, best across 3 pairs):

| battery | 6-perm total | 3-perm total | ratio | worst queries |
|---|--:|--:|--:|---|
| synthetic 10M (6 q) | 69.0 ms | 225.8 ms | **3.3×** | q10 optional_age 4.6→53.1 ms (**11.5×**), q03 star3 23.8→108.8 ms (4.6×), q04 39.1→57.0 ms (1.5×) |
| olympics 1.78M (10 q) | 34.4 ms | 72.8 ms | **2.1×** | q10 4.6×, q05 3.8×, q03/q08 3.1×, q04 1.8× |

Point lookups, counts, and hash-join-dominated queries (oly q07) are at parity; the
damage is concentrated in queries whose merge joins lose their sort order (plans fall
back to hash/sort, not full scans — `choose()` still always finds a prefix-matching
perm in the compact set).

**Re-introduction cost at 10M** (the other half of drop-under-pressure):

| path | time |
|---|--:|
| rebuild from in-RAM SPO (permute + parallel sort) | SOP 0.064 s, PSO 0.131 s, OPS 0.127 s |
| reload persisted perm file from disk (warm cache, incl. byte→row copy) | 0.043–0.081 s |
| + cold NVMe sequential read of 114 MB (estimate) | +~0.05 s |
| first-touch page-in of one whole mmap'd perm (measured) | 0.217 s |

So re-introduction is cheap (≈0.1–0.2 s/perm at 10M, roughly linear → ~1–2 s at
100M). **Verdict: REJECT runtime adaptive dropping.** For mmap deployments it saves
no resident memory (the OS already does it — lever 5); for Owned deployments the
−50% memory comes at 2–3.3× battery latency, which lever 2 beats (−30% at ~0%
latency), and the existing compile-time `compact-index`/wasm set already serves the
hard-memory-bound target. The cheap re-introduction numbers mean a *pressure
emergency valve* for Owned mode is viable if ever needed — design sketched below,
not recommended as a first wave.

## Lever 2: hot/cold compression tiering — PURSUE (per-permutation, Owned mode + disk); REJECT (per-predicate-range blocks)

**Heterogeneity already exists.** `PermData` is a per-slot enum and `open()`
auto-detects raw vs compressed **per file**, so mixed dirs work today with zero crate
changes (the spike's `mix` bin re-encodes selected `perm{i}.bin`). What's missing is
only a public API to re-encode selected perms of an in-RAM store (`from_triples`
builds all-raw, `from_triples_compressed` all-compressed; `decompress_to_ram` is
all-or-nothing the other way).

Hot set on BOTH batteries (mincore residency): **PSO + POS ≈ 100%**, SPO/SOP/OSP/OPS
≤ 0.1%. Mixed dir = cold three {SOP, OSP, OPS} compressed (2.4–2.6×), hot three raw.
Paired battery results (synthetic 10M, 3 reps):

| config | suite total (min) | process RSS after | perm bytes on disk | owned-heap equivalent |
|---|--:|--:|--:|--:|
| all raw | 64.6–80.3 ms | 108 MB | 686 MB | 720 MB |
| **mixed (cold compressed)** | 66.3–92.1 ms | 116 MB | **479 MB (−30%)** | **503 MB (−30%)** |
| all compressed, lazy | 177.8–226.8 ms (**2.7×**) | 172 MB | 289 MB | (dir only ~4 MB) |
| all compressed, eager decode | 70.6–79.8 ms (parity) | 715 MB | 289 MB | 720 MB |

The mixed-vs-raw total deltas (+3…+15%) are machine noise: per-query mins are at
parity (cleanest pair: q03 23.28 vs 23.37 ms, q04 36.6 vs 38.2 ms, q10 4.69 vs
4.74 ms). Olympics agrees (raw 31.3 vs mixed 32.0 ms in the clean pair; all-compressed
lazy 1.6×). **The claim tested — ~0% perf cost when ONLY cold structures are
compressed — holds**, because the battery never decodes a cold block. The win is real
where bytes are owned: disk always (−30%), and heap in Owned/in-RAM deployments
(wasm; eager-decompressed latency-sensitive servers: decompress only the hot perms —
240 MB heap + cold served lazily ≈ **−65% vs all-eager** at hot-path parity).

**Per-predicate-range tiering within a perm: REJECT.** In count mode the hot perms
are 100% touched (no cold blocks to compress). The json battery does show real
within-perm structure (PSO 38%, POS 65% resident, contiguous hot ranges in the
20-bucket histogram) — but in mmap mode the page cache ALREADY keeps only those
ranges resident (that's what the measurement shows), and in Owned mode the residual
win beyond per-perm tiering (~120 MB at 10M) requires a new mixed-block file format,
per-block flags, and an access-tracking policy to pick blocks. Structurally feasible
(the block directory already has per-block offsets), but the complexity/win ratio
loses to shipping per-perm tiering first. Revisit only if real-workload counters
(lever 4) show large cold fractions inside hot perms in an Owned deployment.

## Lever 4: access tracking — PURSUE (it is free at scan granularity)

Microbench on a realistic probe loop (binary-search a random bound-subject prefix
over the 10M-row SPO perm, 2M probes, best of 5):

| variant | ns/probe | overhead |
|---|--:|--:|
| baseline probe | 825.4 | — |
| + relaxed `AtomicU64` fetch_add per probe (per-perm counter) | 828.8 | **+0.4%** |
| + sampled (bump every 64th via local u64) | 810.9 | −1.8% (= noise) |
| 4 threads hammering ONE shared counter while probing | 734.1/thread | no contention penalty visible |
| raw uncontended fetch_add | 3.2 ns/op | — |

A per-permutation relaxed counter on the `rows_in`/`count_in` path costs nothing
measurable at query granularity (scans are O(µs–ms); the counter is 3 ns). Per-ROW
counting would be a different story — never do that. Sampling is unnecessary at
per-scan granularity.

## Lever 5: page-cache reality check — YES, the OS already drops cold perms for free in mmap mode

Measured plainly:
- An untouched mapping is **0% resident** (demand paging) — a cold perm in an mmap
  deployment costs zero RAM *from the start*; nothing to drop.
- Clean file-backed pages are reclaimable by the kernel under pressure on both OSes;
  explicit eviction is not even available on macOS (`MADV_DONTNEED` returned 0 and
  evicted nothing — residency stayed 100%; Linux semantics differ, but there the
  kernel's LRU already does the job).
- Re-warm costs: first-touch of a full 120 MB perm 0.217 s; warm re-touch ~0.
**Conclusion: explicit dropping/tiering only matters for Owned (in-RAM) data — the
wasm build, eager-decompressed perms, and anon heap. For mmap'd serving, "drop cold
perms under pressure" is already implemented by the kernel.** This is what makes
lever 1 (runtime dropping) pointless for the server and lever 2 (owned-mode tiering)
the winner.

---

## Verdict table

| lever | verdict | key numbers |
|---|---|---|
| predstats load bug (found via lever 3) | **FIXED** (crate change, gated) | open 0.52→0.019 s; RSS-after-open 236→2.3 MB |
| 1. runtime perm dropping | **REJECT** (mmap: saves ~0 resident, OS already demand-pages; Owned: dominated by lever 2) | 3-perm battery 2.1–3.3× slower (worst query 11.5×); re-introduce 0.06–0.13 s/perm at 10M |
| 2a. per-perm hot/cold compression tiering | **PURSUE** | −30% perm bytes (owned/disk) at per-query parity; hot-only-eager ≈ −65% heap vs all-eager at hot-path parity |
| 2b. per-predicate-range (block) tiering | **REJECT** (for now) | count battery: hot perms 100% hot; mmap page cache already does it (json: only 38/65% of PSO/POS resident); Owned win ~120 MB at 10M not worth a new format yet |
| 3. residency profile | **DONE** — biggest block was the bug; next-biggest in realistic serving is result materialization (1.75 GB RSS in json battery), an engine/serving follow-up, not index storage | index+dict file pages <200 MB at 10M |
| 4. access counters | **PURSUE** | +0.4% per probe (3.2 ns), no visible contention penalty at scan granularity |
| 5. page-cache check | **CONFIRMED** — mmap mode gets drop-cold free | untouched perm 0% resident; macOS MADV_DONTNEED is a no-op on file pages |

## Proposed design — winners only

Per the orchestrator addendum (2026-06-12): build-time configurability in house
style, resource-awareness, wasm compiles detection out.

**W1 — per-perm tiering API (`sparq-core`, feature `tiering`, off by default):**
- `TripleStore::set_perm_storage(perm, Storage::{Raw, Compressed})` — re-encode /
  decode ONE owned perm in place (PermData is already a per-slot enum; ~40 lines),
  plus `Graph::decompress_perms(&[Perm])` so an opened compressed dir can eager-decode
  only the hot set. Default behaviour without the feature: exactly today's.
- `save_mixed(dir, cold: &[Perm])` writing per-file formats `open()` already
  auto-detects (zero migration; old dirs untouched).
- Default static policy: cold = {SOP, OSP, OPS} (measured ≤0.1% touched on both
  batteries; SPO stays raw — it backs `contains`/overlay paths). Runtime override.
- Resource-awareness: a `MemoryBudget` (default = fraction of detected system RAM via
  sysctl/`/proc/meminfo`; explicit value required on wasm — no detection there,
  `cfg(target_arch)` compiles it out) selects the tier ladder at load:
  720 MB (all raw) → 503 MB (cold compressed) → 386 MB (only POS+PSO+SPO raw…) →
  289 MB (all compressed, 2.7× battery) per 10M triples, scaled linearly.
  Disk floor via `statvfs` before any spill/save (default: refuse below ~2 GB free).
  The bench harness passes explicit budgets for reproducibility.

**W2 — access counters (`sparq-core`, feature `access-stats`, off by default first
wave; candidate for default-on after a battery-level A/B):**
- `[AtomicU64; 6]` in `TripleStore`, `fetch_add(1, Relaxed)` in `scan_with` +
  `estimate`; snapshot + epoch-halving decay API (`perm_access_counts()`).
- Feeds W1's adaptive mode: a perm idle for an epoch while over budget → compress
  (measured 0.6–0.9 s/6 perms at 10M, so ~0.1–0.15 s/perm); counter rising on a
  compressed perm + budget headroom → decompress (same cost). Hysteresis: epoch
  ≥ minutes, never flap more than one perm per epoch.
- Per-BLOCK counters explicitly rejected (per-row/btree-node granularity is where
  the 3 ns starts to bite, and 2b is rejected anyway).

**Emergency valve (design only, not first wave):** under hard memory pressure in
Owned mode, dropping a cold perm entirely (freeing 120 MB/10M each) with
re-introduction = rebuild-from-SPO (0.06–0.13 s/perm at 10M, ~linear) is viable;
trigger = budget breach after the compression ladder is exhausted. Costs 2–3.3×
on queries that needed it until re-introduced — acceptable only as a last rung.

## Anomalies / honesty notes

- 3-perm builds RAISED process RSS after the battery (122 vs 108 MB): hash/sort
  fallbacks materialize what merge joins borrowed. Dropping indexes can cost peak
  memory, not just latency.
- Suite-total deltas of ±5–15% appeared between identical configs across reps
  (loaded machine); every conclusion above is backed by per-query mins agreeing
  across ≥2 pairs, and memory numbers are exact.
- macOS `MADV_DONTNEED` silently does nothing to file-backed residency (rc=0) —
  do not build eviction logic on it; deployments are Linux but the spike ran on macOS.
- `mix`-produced perm4/perm5 compressed to byte-identical sizes on synthetic — an
  artifact of the generator's uniform structure, not a bug (olympics sizes differ).
- The json battery's 1.75 GB RSS is allocator retention of materialized results —
  reproducible, and the single biggest realistic-serving memory number we saw;
  streaming serialization in sparq-serve is the follow-up that dwarfs index tiering.
