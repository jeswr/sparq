# T24d — GPU execution: the measured verdict

**Recommendation: PARK.** Do not wire a GPU backend into the engine now. Keep
`crates/sparq-gpu` as the measured artifact + re-test harness; re-open only
under the conditions in §5.

This is the completion of roadmap T24d: working wgpu compute kernels for
sparq's three hot-path primitive shapes, benchmarked CPU vs GPU with the
host→device transfer tax charged explicitly. It supersedes the estimate-heavy
parts of `research/hardware/gpu-and-cloud.md` (whose single measured spike —
filter+count, crossover ~16–64M rows — these results confirm and extend).

**Honesty contract.** Every number in §3 was measured on this machine (Apple
M1, MacBookAir10,1, 8 cores, 16 GB unified memory, macOS, wgpu 22 on Metal,
rustc stable, `--release`) on 2026-06-11 by
`cargo run -p sparq-gpu --release --example gpu_bench`. Methodology in §2,
including a contention caveat (§2.1). Nothing here is extrapolated except
§4's discrete-GPU note, which is labelled as such.

---

## 1. What was built

`crates/sparq-gpu` — opt-in, depended on by nothing, zero engine coupling
(kernels take plain `&[u32]`/`&[f64]` columns: the exact shape of sparq-core's
permutation-index object columns and dense numeric cache). Four WGSL kernels
spanning the compute-intensity axis:

| kernel | shape | intensity |
|---|---|---|
| `filter_count_u32` | FILTER `lo ≤ v < hi` + COUNT | compute-light (~1 cmp / 4 B) |
| `filter_count_f64_gt` | FILTER `v > t` + COUNT, exact via bit-order keys (WGSL has no f64) | compute-light (~1 cmp / 8 B) |
| `hash_probe` | hash-join probe vs resident open-addressing table, COUNT + SUM(payload) | compute-medium (hash + data-dependent walk) |
| `group_aggregate` | COUNT + SUM GROUP BY (≤512 dense groups), two-level atomics, u64 via carry emulation | compute-dense for a scan |

All kernels reduce **on-device** and read back O(1)/O(groups) bytes — the
GPU's best case. Materialising selected rows would only make the GPU look
worse, so everything below is an **upper bound** on GPU benefit.

Correctness: 4 GPU-vs-CPU tests on random data incl. IEEE-754 edges
(NaN/±inf/−0.0/subnormals) — exact equality asserted, all pass on the M1.
`Gpu::new()` returns `None` when no adapter exists (runtime check), so the
tests skip-pass on GPU-less CI.

## 2. Methodology

Four legs per kernel × column size, interleaved best-of-N (each round runs
every leg once, so thermal/cache drift hits all legs; N=5 at 1M/10M, N=3 at
100M; checksums asserted equal across legs every round):

- **cpu1** — single-thread scalar Rust (what one sparq scan thread runs today)
- **cpuN** — rayon over all 8 cores
- **gpu resident** — dispatch + tiny readback; columns already in device
  memory (what a residency cache would buy)
- **gpu e2e** — re-upload **all** inputs + dispatch + readback (the per-query
  streaming reality)

Scales: 1M / 10M / 100M elements. 100M fits comfortably (16 GB host; Metal
max storage binding here is 4 GiB − 1; largest resident input is the 800 MB
f64 column) — no scale-down was needed.

### 2.1 Contention caveat

The machine ran concurrent agent workloads (rustc jobs in sibling worktrees +
an antivirus scanner); 1-min load average ranged ~33–60 during the run.
Best-of-N minima and interleaving blunt but do not eliminate this. A repeat
run under lower load (§3.5) reproduced every ratio's *sign* (win/lose) and
ranking; absolute CPU times in the noisier run are pessimistic by up to ~2×
in the worst single cell. The verdict does not change between runs — the
contention noise is smaller than the margins it would need to overturn.

## 3. Results (milliseconds, best-of-N; ratios >1.0× = GPU wins)

### 3.1 FILTER u32 (~12.5% selectivity) — compute-light

| elems | cpu1 | cpuN | gpu resident | gpu e2e | cpuN/resident | cpuN/e2e |
|--:|--:|--:|--:|--:|--:|--:|
| 1M | 0.58 | 0.30 | 1.64 | 2.68 | 0.18× | 0.11× |
| 10M | 6.68 | 1.25 | 3.54 | 19.36 | 0.35× | 0.06× |
| 100M | 65.65 | 12.42 | 17.89 | 200.55 | 0.69× | 0.06× |

### 3.2 FILTER f64 (v > t, exact bit-order compare) — compute-light

| elems | cpu1 | cpuN | gpu resident | gpu e2e | cpuN/resident | cpuN/e2e |
|--:|--:|--:|--:|--:|--:|--:|
| 1M | 0.50 | 0.32 | 1.79 | 5.68 | 0.18× | 0.06× |
| 10M | 3.89 | 2.12 | 4.64 | 36.14 | 0.46× | 0.06× |
| 100M | 51.45 | 20.77 | 21.50 | 531.16 | 0.97× | 0.04× |

### 3.3 HASH-JOIN probe (build = n/4 ids, load 0.5, ~50% hit) — compute-medium

| elems | cpu1 | cpuN | gpu resident | gpu e2e | cpuN/resident | cpuN/e2e |
|--:|--:|--:|--:|--:|--:|--:|
| 1M | 86.96 | 11.99 | 4.29 | 6.74 | 2.79× | 1.78× |
| 10M | 599.20 | 99.67 | 31.13 | 70.11 | 3.20× | 1.42× |
| 100M | 10360.46 | 1437.53 | 939.97 | 1375.60 | 1.53× | 1.05× |

### 3.4 GROUP BY COUNT+SUM (256 groups) — compute-dense for a scan

| elems | cpu1 | cpuN | gpu resident | gpu e2e | cpuN/resident | cpuN/e2e |
|--:|--:|--:|--:|--:|--:|--:|
| 1M | 2.02 | 4.70 | 3.19 | 7.31 | 1.47× | 0.64× |
| 10M | 20.71 | 13.64 | 13.47 | 47.66 | 1.01× | 0.29× |
| 100M | 531.82 | 45.66 | 105.50 | 650.84 | 0.43× | 0.07× |

### 3.5 Confirmation run (lower load)

(repeat run, same protocol — see §2.1)

| kernel | scale | cpuN | gpu resident | gpu e2e | cpuN/resident | cpuN/e2e |
|---|--:|--:|--:|--:|--:|--:|
| TBD | | | | | | |

## 4. Reading the numbers

1. **Compute-light scans never pay, even resident.** On unified memory the M1
   GPU sees the *same* DRAM as the CPU; for a ~1-op-per-word scan the 8 CPU
   cores already saturate bandwidth (FILTER u32 100M: 12.4 ms ≈ 32 GB/s).
   The GPU's only lever — more FLOPs — buys nothing, and dispatch+readback
   latency (~1.5 ms floor) drowns small columns. Best case observed: 0.97×
   (a tie) at 100M f64. **This kills the "GPU FILTER" idea on this class of
   hardware in the most GPU-favourable (count-only) formulation.**
2. **The transfer tax is fatal everywhere it applies.** gpu-e2e loses by
   9–25× on scans. And this is *unified memory* — a `queue.write_buffer`
   memcpy at tens of GB/s. A discrete card over PCIe 3.0/4.0 (8–32 GB/s,
   plus driver latency) is strictly worse; the prior XPS-targeted estimates
   in `gpu-and-cloud.md` stand. Per-query streaming offload is dead on
   arrival; only a residency model is even discussable.
3. **Hash-probe is the one real win** — the only kernel with enough work per
   byte (hash + data-dependent dependent-load walk) for the GPU's
   latency-hiding to beat the CPU's cache hierarchy: 1.5–3.2× over 8-core
   rayon when resident, and still ≥1× even charging the full re-upload.
   But: (a) the build side stays on the CPU; (b) sparq's real joins are
   mostly *merge* joins on sorted permutations (the hash path is the
   fallback); (c) a 1.5–3× win on one operator class does not amortise a
   residency cache, a scheduler that knows what's resident, wgpu in the
   dependency tree, and a second backend to keep correct.
4. **GROUP BY flips against the GPU at scale** (0.43× at 100M): per-element
   shared-memory atomics with key-skew contention scale worse than the CPU's
   private 256-entry arrays + tree merge. The u64-carry emulation (WGSL has
   no 64-bit atomics) adds per-element cost the CPU doesn't pay.
5. **Break-even points** (resident, vs 8-core CPU): FILTER u32/f64 — none
   observed up to 100M (trend approaches 1× around ~100M–1B extrapolated, on
   a workload nobody should ship); hash-probe — wins from ≤1M onward; GROUP
   BY — narrow ~10M window, gone by 100M. Against *single-thread* CPU the GPU
   looks much better everywhere ≥10M, but cpu1 is not the honest baseline:
   rayon is already in sparq-bench and parallel scans are far cheaper to
   adopt than a GPU backend.

## 5. Verdict

**PARK** (measured-and-rejected for now; not a permanent rejection):

- **Reject** per-query GPU offload (the e2e rows) on all hardware — the
  transfer tax never pays at realistic scales, even with zero-copy-class
  unified memory.
- **Reject** GPU FILTER/scan on Apple-silicon-class unified memory — CPU
  cores saturate the same DRAM; there is nothing for the GPU to add.
- **Park** the hash-probe result. It is real (1.5–3.2× resident) but
  single-operator, fallback-path, and worth far less than its integration
  cost today. The kernels and harness stay in-tree as the re-test rig.

**Re-open T24d if any of these become true:**

1. sparq targets a machine with a **discrete GPU with ≥8 GB VRAM and high
   bandwidth** (≥400 GB/s) where datasets fit resident — then re-run this
   harness there (the crate is portable: Vulkan/DX12/Metal/WebGPU); the
   resident hash-probe and possibly scans change regime when GPU bandwidth is
   5–10× host. Note the caveat cuts both ways: discrete = better bandwidth
   *and* worse transfer tax.
2. The engine grows a **resident-column cache** for another reason (e.g.
   numeric vector workloads in sparq-vectors) — then GPU hash-probe piggy-
   backs on already-paid residency and the 1.4–3× becomes nearly free.
3. **WebGPU in-browser** becomes the only parallelism available (wasm threads
   unavailable): against *cpu1* (the wasm reality today) the resident GPU is
   3.7–11× on hash-probe and ~3–5× on big scans — a different calculus, but
   blocked on f64 absence and buffer-size limits; treat as its own task.

### Apple-silicon caveat (explicit)

These unified-memory numbers are the **most transfer-friendly** measurement
possible: "upload" is a DRAM-to-DRAM memcpy. On any discrete-GPU machine the
e2e columns get worse (PCIe), while the resident columns may get better
(GDDR/HBM bandwidth). Neither shift rescues per-query offload; both make the
residency-model question hardware-specific — which is exactly why the verdict
is *park with a portable re-test harness* rather than *delete*.

## 6. Reproduce

```sh
cargo test -p sparq-gpu                                  # skips w/o adapter
cargo run -p sparq-gpu --release --example gpu_bench     # ~3–5 min on an M1
cargo tree -p sparq-wasm --target wasm32-unknown-unknown | grep -i wgpu  # must be empty
```
