# sparq — GPU and Cloud Acceleration Strategy

Honest assessment of where GPU and cloud hardware help (and hurt) a
dictionary-encoded, sorted-permutation RDF + SPARQL engine, with one **measured
local GPU spike** on this M1 Mac and clearly-flagged **literature estimates** for
everything else.

> **Measured vs estimated.** Everything in the `MEASURED` boxes was run on the
> development M1 Mac (Apple M1, 8 cores, 16 GB unified memory, macOS, wgpu 22 on
> Metal) on 2026‑06‑08. Everything else is sourced literature or reasoned
> extrapolation and is labelled as such. No GPU benchmark here was run on the
> XPS — that machine is not yet reachable (see `remote-access-setup.md`).

---

## 0. The measured spike — does a portable GPU kernel even work, and when does it win?

The single strongest GPU strategy for a Rust engine that must run on **Metal (this
Mac), Vulkan/CUDA (the XPS), and DX12/WebGPU (browser/WASM)** is one portable
compute kernel via **wgpu**. I built and ran exactly that: a parallel
predicate-filter + count over a `u32` column — sparq's actual hot path (the object
column of a permutation index, the `FILTER(?o in [lo,hi))` / range-scan case that
`BENCHMARKS.md` flags as the biggest loss vs QLever, q06: 14–20× slower than
QLever today).

The spike lives at `research/hardware/wgpu-spike/` (standalone crate with its own
`[workspace]`, deliberately *not* a member of the sparq workspace so it can't
perturb the engine build). Reproduce with `cd research/hardware/wgpu-spike &&
cargo run --release`. It compiled and ran first-try on Metal. WGSL kernel core:

```wgsl
@compute @workgroup_size(256)
fn main(...) {
    let i = flat_wg * 256u + lid.x;          // 2D-tiled grid (WebGPU caps each
    if (i < params.n) {                      // dispatch dim at 65535 workgroups)
        let v = col[i];
        if (v >= params.lo && v < params.hi) { atomicAdd(&local_count, 1u); }
    }
    // workgroup-local atomic, flushed to a per-workgroup slot, host reduces
}
```

> **MEASURED — wgpu filter+count, M1 / Metal, ~12.5% selectivity, best-of-N, three runs.**
> Columns: `cpu1` = single-thread scalar Rust; `cpuN` = rayon all-8-cores;
> `gpu compute` = kernel + tiny readback, **column already resident in VRAM**;
> `gpu e2e` = **re-upload the whole column each call** + kernel + readback.
>
> | elems | cpu1 ms | cpuN ms | gpu compute ms | gpu e2e ms | cpu1/gpu | cpuN/gpu |
> |--:|--:|--:|--:|--:|--:|--:|
> | 1 M  | 0.16 | 0.20 | 1.2–1.4 | 1.7–2.0 | 0.13× | 0.16× |
> | 4 M  | 0.62 | 0.60 | 1.3 | 4.7 | 0.48× | 0.48× |
> | 16 M | 2.7–4.4 | 2.1–3.4 | 2.6–4.0 | 17–20 | 0.7–1.7× | 0.6–1.3× |
> | 64 M | 11–17 | 8.5–15 | 5.1–8.2 | 58–84 | 2.2–3.3× | 1.7–2.9× |
>
> (Counts verified equal CPU vs GPU every run.)

**What this measured spike proves (and these are the load-bearing conclusions):**

1. **One wgpu kernel is genuinely portable.** It built and ran unmodified on
   Metal; the same source targets Vulkan (hence the XPS's CUDA GPU via the Vulkan
   driver) and DX12 and WebGPU. This de-risks the "write the GPU path once" bet —
   sparq does not need separate CUDA and Metal code paths to start.
2. **GPU compute wins only on large, *resident* columns.** Crossover is ~16–64 M
   elements. Below ~4 M, the ~1 ms fixed dispatch/launch latency makes the GPU
   *slower than a single CPU core*. At 64 M (256 MB column) GPU compute is ~2–3×
   the all-core CPU.
3. **The transfer tax kills naive offload.** `gpu e2e` (upload the column every
   query) is **4–12× slower than the CPU at every size** — even here, where M1
   "upload" is just a unified-memory `memcpy`, not a real bus transfer. At 64 M
   the upload alone is ~10× the compute. On the XPS's **discrete** 1650 Ti this
   crosses PCIe 3.0 ×8 (~6–7 GB/s realisable) and is strictly worse.
4. **Therefore the only viable GPU design is data-resident**: keep
   permutation/dictionary columns in VRAM across queries and dispatch many kernels
   against them. Per-query host→device streaming is a non-starter. This single
   fact dictates the entire rest of this document.
5. **Bonus honest finding:** the naive 1-D dispatch hit WebGPU's 65535-workgroup
   per-dimension cap at 64 M elems and had to be 2-D-tiled. Portability has real
   sharp edges; budget for them.

A caveat worth stating: the CPU baseline here is a tight scalar/rayon loop and is
*already* memory-bandwidth-bound on M1 (note `cpuN` barely beats `cpu1` — exactly
the "workload is memory-latency bound" prediction in `ARCHITECTURE.md §3.4`). A
real engine scan also does dictionary materialisation and term parsing per row
(that is *why* sparq's q06 is 14–20× off QLever). The GPU's win on the *raw
predicate* understates its potential win on the *full materialise-and-filter* path
— but only if the dictionary numeric cache also lives on the GPU, which again
means resident data, not streaming.

---

## 1. Nvidia GPU acceleration for a triplestore/SPARQL engine

### 1.1 Per-hot-path reality check

Mapping each engine hot path to the GPU, with realistic expectations. "Speedup"
numbers outside the MEASURED box are **literature estimates** for high-end
datacentre GPUs (V100/A100-class) with **data already resident**; they do *not*
transfer to the 1650 Ti and they do *not* include PCIe.

| Hot path (sparq) | GPU fit | Realistic resident speedup (lit.) | When GPU **wins** | When GPU **loses** |
|---|---|---|---|---|
| **Scan + filter** (predicate over a column) | Excellent — embarrassingly parallel, coalesced reads | ~2–3× vs all-core CPU *(MEASURED here at 64 M)*; 5–10× on datacentre GPUs (lit.) | Large resident column, low-medium selectivity, COUNT/aggregate result | Small scan (<~4 M), highly selective (few rows out), or result must leave the GPU |
| **Hash join** | Good but contention-prone (atomics, probe divergence) | ~5–8× resident (Titan X, [He et al.]) | Large build+probe both resident, uniform keys | Skewed keys (atomic hot-spots), small side fits CPU L2, result streamed back |
| **Sort-merge join** | Very good — GPU sort (radix) is a GPU strength | ~10–17× resident (Titan X, [He et al.]) | Both inputs resident & need (re)sorting; large N | sparq's inputs are *already sorted* permutations → the GPU sort advantage largely evaporates; merge of pre-sorted data is bandwidth-bound, modest GPU gain |
| **Worst-case-optimal (Leapfrog Triejoin / multiway)** | Good with warp-level intersection | up to ~67× over prior GPU baselines on V100 ([Accelerating multi-way joins, VLDB'21]); vs CPU more like single-digit–low-double-digit | Cyclic/skewed BGP, many patterns, large intermediates avoided | Acyclic low-skew (CPU LFTJ already cheap); irregular trie descent → thread divergence |
| **Sort** (ORDER BY, index build) | Excellent — radix sort is GPU's best trick | 5–20× resident (lit.) | Large key arrays resident | Result/keys must round-trip; small sorts |
| **Dictionary encode** (build-time term→id) | Poor–medium — hashing + string handling + dedup is branchy, variable-length | rarely worth it; GPU string hashing exists but build is I/O+parse bound | n/a in practice | Almost always: build is dominated by parse + external merge (CPU/IO) |
| **Index build** (sort triples into 6 perms) | Medium — the *sort* step is GPU-friendly, but 4 GB VRAM caps it | GPU radix sort of the id triples could help at scale | Mid-size builds that fit VRAM | Out-of-core builds (the actual target: Wikidata 20 B); VRAM far too small |
| **Block decompression** (ZSTD, M3) | Poor for ZSTD (sequential entropy decode); **good for bit-packing/PForDelta** | bit-unpacking 5–10× resident (lit.) | If sparq adopts SIMD-BP128/FastPFOR (already on the M3 roadmap), GPU bit-unpack is natural | ZSTD's Huffman/FSE stage is inherently serial per block — keep on CPU |

**The cross-cutting truth:** for an *in-memory* engine, the GPU's job is to be a
**resident column co-processor for big aggregate/scan/sort-heavy queries**, not a
general join accelerator and not a streaming offload. This matches the published
GPU-SPARQL systems:

- **TripleID-Q** ([arXiv:1807.01409]) loads the whole dictionary-encoded triple
  table into GPU memory and matches triple patterns with massive parallelism,
  *no index* — reporting up to **108×** vs a traditional CPU RDF tool. The catch
  is exactly the resident-data constraint: it works because the table fits in
  VRAM, and it skips indexing (so it loses on selective patterns where sparq's
  binary-search-on-block-metadata is O(log n), not O(n)).
- **Wukong+G** ([USENIX ATC'18]) is the most relevant real system: GPU graph
  exploration for RDF, and its *entire engineering contribution is managing the
  GPU-memory bottleneck* — caching, pipelining, swapping, prefetching between host
  and GPU, plus RDMA across nodes. That is the same wall the spike hit: the GPU is
  fast, getting data to it is the problem.
- **MAGiQ / GSmart** recast SPARQL as **sparse-matrix algebra** so it runs on GPU
  BLAS/cuSPARSE. This is elegant and portable across hardware, but it is a *very
  different engine* from sparq's sorted-merge/WCOJ design — adopting it would be a
  rewrite, not an accelerator. Worth knowing as a fallback architecture, not a
  bolt-on.
- **gStore-GPU** accelerates its signature-graph subgraph matching on GPU; again,
  a different core data structure than permutation indexes.
- **RAPIDS cuDF** (NVIDIA's production GPU dataframe) reports ~30× over pandas on a
  10 GB join (T4) and processes 1 B rows in ~17 s on an A100 — but those are
  **datacentre GPUs with unified/managed memory and 16–80 GB VRAM**, and the
  comparison baseline is *pandas* (single-thread Python), not a tuned Rust merge
  join. The honest read: cuDF proves GPU joins scale on big iron with lots of
  VRAM; it says little about a 4 GB mobile GPU vs sparq's Rust engine.

### 1.2 The Rust implementation path

Three realistic routes, in recommended order:

1. **wgpu (RECOMMENDED to start).** Pure-Rust, portable: Metal/Vulkan/DX12/GL +
   WebGPU. **Proven to build and run here.** One WGSL/`naga` kernel covers the
   Mac dev box, the XPS (via Vulkan over the NVIDIA driver), *and* the browser
   WASM end-state from `ARCHITECTURE.md §6` — a browser SPARQL engine that can use
   WebGPU is a genuinely novel capability no competitor has. Downsides: WGSL is
   less expressive than CUDA C++ (no warp intrinsics/shuffles exposed portably, no
   `cub`/`thrust`, atomics only), and you hit caps like the 65535-workgroup
   dispatch limit the spike tripped over. For scan/filter/aggregate and simple
   joins it is enough.

2. **cudarc (RECOMMENDED for NVIDIA-max performance).** A safe, **actively
   maintained (2025)** host-side wrapper over the CUDA Driver API + cuBLAS/cuSPARSE/
   cuRAND/NCCL. You write kernels in CUDA C++ (or PTX) and drive them from Rust,
   or call cuSPARSE directly for a MAGiQ-style matrix path. This is the route to
   `thrust`/`cub` radix sort, warp-cooperative intersection for WCOJ, and the last
   ~2× the portable path leaves on the table. NVIDIA-only, so it's a *second*
   backend behind a feature flag, not the primary.

3. **Rust-CUDA / `cust` / `rustc_codegen_nvvm` (WATCH, don't depend yet).** The
   "write the kernel itself in Rust, compile to PTX" project was **rebooted in
   2025** under the Rust GPU org and is improving fast (CUDA 12/13 support landing
   through 2025), and the related "Rust on every GPU" work shares kernel code
   across CUDA and SPIR-V. Promising for a single-language future, but still
   maturing — treat as a future consolidation, not a foundation for a perf-critical
   engine in 2026.

**Concrete recommendation:** prototype on **wgpu** (portable, already works,
unblocks browser-WebGPU too); if and only if NVIDIA benchmarks justify it, add a
**cudarc** backend behind a `--features cuda` flag for `cub` radix sort and
cuSPARSE. Do **not** invest in rust-CUDA kernels yet.

### 1.3 What the XPS's 4 GB GTX 1650 Ti Mobile can realistically do

**The card** (XPS 15 9500 discrete option): NVIDIA GeForce GTX 1650 Ti Mobile —
Turing **TU117**, **1024 CUDA cores**, **4 GB GDDR6** on a **128‑bit** bus
≈ **192 GB/s** VRAM bandwidth, ~**4 TFLOPS FP32** (less in the **Max-Q ~35 W**
trim Dell ships). No tensor cores usable for this workload, no NVLink, PCIe 3.0.

**Honest capability assessment — what it CAN do:**

- Be a **real CUDA/Vulkan dev target**: develop, debug and validate the wgpu and
  cudarc paths against actual NVIDIA hardware/driver (the thing the Mac cannot do).
  This is its single most valuable role.
- Accelerate **resident scan/filter/aggregate and sort** on columns up to **~1–2 GB**
  (leaving headroom for results + the OS using the same GPU for display). Per the
  spike's crossover, expect a win only above ~10–50 M rows per column, and only
  for COUNT/aggregate-shaped queries where little data comes back.
- Run **small-to-mid datasets fully resident**: a dictionary-encoded triple table
  of a few hundred million triples (TripleID-Q style, ~12 B/triple → ~4 GB caps
  you around ~300 M triples *with nothing else*, realistically ~100–150 M with
  room for indexes/results). DBLP-390M does **not** fit; a WatDiv-10M/100M or an
  Olympics-scale graph does.

**Hard limits — what it CANNOT do:**

- **4 GB is the whole story.** sparq's actual targets (DBLP-390M = ~8 GB index;
  WDBench 1.2 B; Wikidata 20 B) are **1–4 orders of magnitude larger than VRAM.**
  Wukong+G's entire paper exists because of this wall; on a 4 GB card the
  swap/prefetch overhead would dominate and almost certainly lose to the CPU.
- The display + OS consume VRAM; under X11/Wayland you realistically have ~3–3.5 GB.
- **35 W Max-Q + laptop thermals** → sustained throughput well below desktop 1650 Ti
  numbers; long-running kernels thermal-throttle.
- PCIe 3.0 ×8/×16 → the transfer tax the spike measured is real and unavoidable
  for anything not resident.

**Verdict on the XPS GPU:** it is a **development and correctness-validation
target for the NVIDIA/Vulkan GPU path, not a performance target.** Use it to prove
the kernels run on CUDA-class hardware and to catch portability bugs (like the
dispatch cap) before they reach a cloud GPU. Do **not** expect it to beat the
engine's CPU path on any dataset sparq actually cares about, and do not benchmark
"sparq-GPU vs QLever" on it — the dataset that fits in 4 GB is precisely the
small-in-RAM regime where the CPU already wins (per the spike's <16 M rows).

---

## 2. Cloud GPU (AWS — user pays)

GPU instances are worth it **only** when the working set is GPU-resident and the
query mix is scan/sort/aggregate-heavy with small results. For sparq specifically,
that means: large in-VRAM columns, COUNT/GROUP BY/analytic workloads, or the WCOJ
path on cyclic skewed BGPs — *not* general low-latency point-lookup SPARQL serving.

| Need | Instance | GPU | VRAM | Why |
|---|---|---|---|---|
| **GPU dev / kernel validation** (cheapest real NVIDIA) | **g4dn.xlarge** | T4 | 16 GB | Cheapest datacentre NVIDIA on AWS (~$0.50/hr on-demand, far less spot). 16 GB VRAM = 4× the XPS, enough to hold DBLP-390M-ish resident. Perfect for "does the cudarc/wgpu path scale past 4 GB?" |
| **GPU dev/perf, current-gen** | **g6.xlarge** / **g6e.xlarge** | L4 | 24 GB | Ada L4, much faster than T4, 24 GB. g6e has more host RAM. Best price/perf for sustained GPU SPARQL experiments. |
| **GPU throughput, balanced** | **g5.xlarge–g5.12xlarge** | A10G | 24 GB ea. | A10G is a workhorse; g5.12xlarge = 4× A10G (96 GB total VRAM) for multi-GPU sharding experiments. |
| **GPU max-scale research** | **p4d.24xlarge** | 8× A100 | 40/80 GB ea. | Only if you are genuinely testing 100B-triple GPU-resident or NVLink multi-GPU joins. Very expensive (~$32/hr). Spot or short bursts only. |
| **Bleeding edge** | **p5.48xlarge** | 8× H100 | 80 GB ea. | Overkill for sparq research; mention only for completeness. |

**Cost/perf reasoning & the honest verdict on GPU-in-the-cloud for SPARQL:**

- **For development:** rent a **g4dn.xlarge or g6.xlarge spot** for a few hours to
  validate that the GPU path scales past the XPS's 4 GB and to get real CUDA-class
  numbers. This is high-ROI and cheap (single-digit dollars per session).
- **For a hosted production triplestore: a GPU instance is almost never the right
  call.** SPARQL serving is dominated by *latency-sensitive, selective* queries
  with *large variable result sets that must leave the engine* — the exact profile
  the spike shows the GPU losing (small/selective + transfer tax). QLever and
  RDFox achieve their numbers on **CPU + lots of RAM + NVMe**, not GPUs. You would
  pay 3–10× the $/hr for a GPU box and lose on most of the query mix.
- **GPU beats a big CPU box only when:** the workload is analytic (heavy COUNT/
  GROUP BY/aggregation over billions of resident rows, few rows out), the data
  fits VRAM (or shards across multiple GPUs' VRAM), and you can amortise residency
  across many queries. That is an *analytics* use case (think "GPU OLAP over RDF"),
  not a general SPARQL endpoint. If you build the GPU aggregate path, **g6e** is
  the instance to demonstrate it on.

---

## 3. Cloud CPU for a hosted triplestore (the actually-recommended path)

This is where a "highly performant triplestore in the cloud" really lives. The
engine is memory-latency-bound and read-optimised; the levers are **RAM capacity,
memory bandwidth, core count, and (for out-of-core) NVMe.**

| Role | Family | Notes for sparq |
|---|---|---|
| **In-memory, best $/perf (RECOMMENDED default)** | **r7g** (Graviton3, ARM) / **r8g** (Graviton4) | 8 GB RAM/vCPU, DDR5, excellent memory bandwidth, ~20% cheaper than x86 equivalents. sparq is pure Rust over u32/u64 arrays → compiles clean to `aarch64`, NEON via `std::simd`. **This is the sweet spot for an in-RAM endpoint.** |
| **In-memory, x86 (max single-core / AVX-512)** | **r7i / r7iz** (Sapphire Rapids) | Use if you adopt AVX-512 (`vpcompressd` selection compaction, bit-unpack) and want the highest per-core clocks. r7iz = high-frequency. |
| **Huge in-memory (full Wikidata 20 B resident)** | **x2idn / x2iedn** (up to 2–4 TB RAM) | When you want the whole 20 B-triple index in RAM with no out-of-core. Expensive but the simplest path to "beat QLever on a single big box." x2iedn also has fast NVMe. |
| **Out-of-core / billion-triple on a budget** | **i4i** (x86) / **im4gn / i4g** (Graviton) | Local **NVMe** is the lever for the out-of-core regime `ARCHITECTURE.md` targets (mmap'd compressed permutations, spill). im4gn = Graviton + NVMe, great $/perf. i4i = highest NVMe IOPS. |
| **Compute-heavy (parallel build, materialisation tier)** | **c7g / c7i** | For bulk index build and the M5 RDFox-style reasoning tier (CPU-bound, parallel). |

**Graviton (ARM) vs x86 — the call for sparq:** **Graviton wins by default.** The
engine has no x86-specific code, the storage substrate is `u32`/`u64` arrays, and
Graviton3/4 give better memory bandwidth per dollar — and the engine is
*bandwidth-bound* (the spike's `cpuN ≈ cpu1` result is the tell). Keep an x86
(r7i/r7iz) variant only if/when AVX-512 decompression and selection-compaction
land and measurably beat NEON. Maintaining both is cheap because it's one Rust
codebase; let benchmarks decide per-workload.

**Recommended target for "highly performant triplestore in the cloud":**

- **Small/medium hosted endpoint (≤ a few hundred M triples, fully in RAM):**
  **r8g.4xlarge–r8g.8xlarge** (Graviton4, 128–256 GB RAM). Best $/perf, beats GPU
  on the real query mix, trivially holds DBLP-390M and WatDiv-1B resident.
- **Large / Wikidata-scale, in-memory:** **x2iedn.8xlarge+** (1 TB+ RAM, NVMe) —
  the "single big box beats QLever" play, mirroring QLever's own "one commodity
  machine" philosophy but with more RAM.
- **Large / out-of-core on a budget:** **im4gn.4xlarge / i4i.4xlarge** — local
  NVMe for mmap'd compressed permutations once M3 compression + out-of-core land.
  This is the path that scales to 20 B triples without a 1 TB-RAM bill.

---

## 4. Prioritized verdict — best ROI hardware targets, in order

1. **Cloud CPU, memory-optimized Graviton (r7g/r8g) — DO THIS.** Highest ROI by a
   wide margin. It's where a hosted sparq endpoint should run, it's where you beat
   QLever/RDFox on the real (latency + varied-result) query mix, and the engine
   runs there unmodified. **First action: stand up an r8g box and re-run the
   QLever head-to-head at 100M–1B triples on Linux/ARM** (the current numbers are
   Docker-on-macOS, which the BENCHMARKS doc itself flags as unfair).

2. **Cloud CPU + NVMe (im4gn/i4i) for out-of-core — DO THIS once M3 lands.** The
   only sane path to the billion-/trillion-triple regime that is sparq's actual
   differentiator vs QLever (memory-bounded streaming). Higher ROI than any GPU.

3. **wgpu GPU compute path (portable) — DO THIS opportunistically.** Already proven
   to build/run here. Targets a resident scan/sort/aggregate co-processor *and*
   unlocks a browser-WebGPU SPARQL engine no competitor has. Validate on a cheap
   cloud T4/L4 before committing. Medium ROI, unique upside, low risk because the
   kernel is portable and small.

4. **The XPS 1650 Ti — use as a GPU *dev/validation* box only.** Real NVIDIA
   hardware to catch CUDA/Vulkan portability bugs the Mac can't. **Not** a perf
   target (4 GB VRAM rules out every dataset sparq cares about). Low direct ROI,
   but near-zero cost (you own it) and it de-risks #3 on actual NVIDIA silicon.

5. **cudarc NVIDIA backend — LATER, behind a flag.** Only if wgpu benchmarks show
   the portable path leaving ≥2× on the table on NVIDIA. Squeezes `cub` radix sort
   + cuSPARSE. Second backend, never the foundation.

6. **Datacentre GPUs (A100/H100 p4d/p5) — RESEARCH BURSTS ONLY.** Justified solely
   if you build a GPU-OLAP-over-RDF aggregate path and want to show 100B-resident
   numbers. Spot/short bursts; never a serving target.

**One-line strategy:** *Win on Graviton CPU + NVMe for the real engine; treat the
GPU as a portable wgpu co-processor for big resident aggregates and as a browser
superpower; use the XPS only to keep the NVIDIA path honest.*

---

## Recommended next step

Provision a **single `r8g.4xlarge` (Graviton4) spot instance**, build sparq for
`aarch64-unknown-linux-gnu`, load a **100M-triple WatDiv/WDBench subset**, and
re-run the QLever head-to-head **natively on Linux/ARM** (not Docker-on-macOS).
This produces the first fair, scale-relevant numbers, directly attacks the "gap
WIDENS at 10M" finding in `BENCHMARKS.md`, and costs a few dollars. Defer all GPU
work until that CPU baseline is trustworthy — then validate the wgpu path on a
cheap **g6.xlarge (L4)** to see whether the resident-aggregate win the spike
showed at 64 M holds past 4 GB on real NVIDIA hardware.

---

### Sources (literature estimates; all GPU speedups outside the MEASURED box are theirs, not ours)

- TripleID-Q: RDF Query Processing Framework using GPU — https://arxiv.org/pdf/1807.01409
- Wukong+G: Fast and Concurrent RDF Queries using RDMA-assisted GPU Graph Exploration (USENIX ATC'18) — https://www.usenix.org/system/files/conference/atc18/atc18-wang-siyuan.pdf
- GSmart / MAGiQ (SPARQL as sparse-matrix algebra) — https://arxiv.org/pdf/2106.14038
- Accelerating multi-way joins on the GPU (GPU LFTJ/MHJ, VLDB Journal 2021) — https://dl.acm.org/doi/abs/10.1007/s00778-021-00708-y
- Leapfrog Triejoin (Veldhuizen, ICDT'14) — https://arxiv.org/pdf/1210.0481
- Fast Equi-Join Algorithms on GPUs (He et al.; sort-merge ~10×, hash ~5.5×, PCIe caveat) — https://www.researchgate.net/publication/317352186_Fast_Equi-Join_Algorithms_on_GPUs_Design_and_Implementation
- RAPIDS cuDF unified memory accelerates pandas up to 30x (10 GB join, T4) — https://developer.nvidia.com/blog/rapids-cudf-unified-memory-accelerates-pandas-up-to-30x-on-large-datasets/
- Scaling to 1B rows with RAPIDS cuDF (A100 17 s / T4 200 s) — https://developer.nvidia.com/blog/processing-one-billion-rows-of-data-with-rapids-cudf-pandas-accelerator-mode/
- Rust CUDA project reboot + 2025 updates (cudarc / cust / rustc_codegen_nvvm) — https://rust-gpu.github.io/blog/2025/01/27/rust-cuda-reboot/ , https://rust-gpu.github.io/blog/2025/08/11/rust-cuda-update/
- Rust running on every GPU (shared CUDA/SPIR-V kernels) — https://rust-gpu.github.io/blog/2025/07/25/rust-on-every-gpu/
- cudarc crate docs — https://docs.rs/cudarc
- Dell XPS 15 9500 specs (discrete GTX 1650 Ti 4 GB GDDR6 option) — https://www.dell.com/support/manuals/en-us/xps-15-9500-laptop/xps-15-9500-setup-and-specifications/gpudiscrete
- GTX 1650 Ti Mobile / Max-Q (Turing TU117, Max-Q ~35 W) — https://www.notebookcheck.net/Dell-XPS-15-9500-i7-10750H-1650-Ti-Max-Q.479329.0.html
