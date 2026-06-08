# Accelerating sparq on the 2020 MacBook Air (Apple M1)

A rigorous, honest assessment of every hardware-acceleration avenue for the sparq
RDF + SPARQL engine on the **2020 M1 MacBook Air**: 8-core CPU (4 Firestorm
performance + 4 Icestorm efficiency), 8-core GPU, 16-core Neural Engine, 16 GB
unified LPDDR4X memory (~68 GB/s peak), running macOS / `aarch64-apple-darwin`.

Every speedup below is tagged as **[measured]** (a spike in `/tmp/m1-spikes`, run
on this exact machine), **[literature]** (published numbers for comparable
hardware/algorithms), or **[speculation]** (reasoned estimate, not validated).
No benchmark numbers are fabricated. Where a spike contradicts the intuitive
expectation, the report says so plainly.

The engine hot paths this targets, from `research/BENCHMARKS.md`:
- **full-scan numeric FILTER** (q06: 20× slower than QLever at 10M triples — the
  single biggest measured gap),
- **large join materialization** (q04 chain, q10 OPTIONAL),
- **ingest / index-build throughput**,
plus the structural levers already on the roadmap: M3 column compression +
vectorised/block scan, M4 tagged ValueIds.

---

## 0. The one number that governs everything: unified-memory bandwidth

The first spike measured a count-only `u32 > k` filter over a 64 MB column and
the *same* op on 8 threads and on the GPU:

| executor | throughput | time for 64 MB | effective bandwidth |
|---|--:|--:|--:|
| CPU, 1 P-core (auto-vectorized) | **15.6 G rows/s** | 1.08 ms | **~62 GB/s** [measured] |
| CPU, 8 threads | 13.5 G rows/s | 1.25 ms | ~54 GB/s [measured] |
| M1 GPU (incl. dispatch+wait) | 13.4 G rows/s | 1.26 ms | ~55 GB/s [measured] |
| vDSP bulk op (read+write) | 7.4 G elem/s | — | ~59 GB/s [measured] |

**A single Firestorm P-core already nearly saturates the ~68 GB/s unified memory
bus on a streaming scan.** Adding cores does not help (it is bandwidth-, not
compute-bound), and the GPU does not help either — the CPU, the 8 cores, and the
8-core GPU all pull from the *same* LPDDR4X controller. This is the defining
constraint of the M1 for this engine, and it drives every verdict below:

> **For streaming scans/filters, you cannot go faster by adding compute units —
> only by moving fewer bytes.** The wins are (a) *compression* (read fewer bytes
> per row), (b) *tagged ValueIds / columnar layout* (avoid the random `Vec<f64>`
> gather, which is latency-bound and runs at a fraction of streaming bandwidth),
> and (c) *block pruning* (don't read the block at all). Hardware SIMD/GPU help
> only the **compute-bound** ops: joins, sorts, set intersections, decompression.

This is exactly what QLever's architecture does, and it is why M3 (compression)
and M4 (tagged ValueIds) are the right structural bets — they reduce bytes-moved,
which is the only thing that helps the bandwidth-bound hot path.

---

## 1. NEON SIMD (128-bit, `std::arch::aarch64` / `std::simd`)

**What it is.** NEON is the mandatory 128-bit Advanced-SIMD ISA in the AArch64
baseline; it is *always on* for `aarch64-apple-darwin` (verified: `rustc
--print target-features` lists `neon` enabled by default; no `target-feature`
flag or runtime detection needed). 4× `f32`/`u32`, 2× `f64`/`u64` per lane.
M1 Firestorm has **4 NEON pipes** (it can issue up to four 128-bit SIMD ops per
cycle) — unusually wide, so NEON-friendly code scales well.

**Spike results** (`/tmp/m1-spikes/src/neon.rs`, 4.2M-row columns, min-of-N,
5 repeats to show variance):

| hot-path shape | scalar | NEON intrinsics | verdict |
|---|--:|--:|---|
| **Columnar `f32` filter** (M3/M4 contiguous column) | ~3.8 G rows/s | **0.93–0.99×** | NEON gives **nothing** [measured] |
| **Columnar `u32` filter** (tagged-ValueId path) | ~3.7 G rows/s | 0.91–1.40× | marginal/noise [measured] |
| **Gather filter** (current q06 shape: `cache[id] > k`) | ~0.2–0.5 G rows/s | 0.73–1.63× | unreliable, load-bound [measured] |
| **Sorted merge intersection** (merge-join inner loop) | — | **1.64–2.79×** | **durable win** [measured] |

**The critical, non-obvious finding: LLVM already auto-vectorizes the simple
filter loops to NEON.** Disassembly of the plain scalar `for &x in col { if x>k
{c+=1} }` shows `fcmgt.4s` (4-wide float compare) and `uaddv` reductions — the
compiler emits exactly the NEON a human would write. Hand-written intrinsics for
contiguous compare-and-count therefore add **zero** (and my first attempt, using
a shift-then-add count, *regressed* to 0.80× because it was worse than what LLVM
generates). For a count-only contiguous filter the loop is bandwidth-bound at
~3.8 G rows/s anyway, far below the NEON compute ceiling.

So NEON's real value is concentrated in code LLVM **cannot** auto-vectorize:

- **Merge / sorted-set intersection [measured 1.6–2.8×].** The scalar zipper has
  a data-dependent branch per step that defeats auto-vectorization; a
  block-vs-block NEON intersection (Lemire/Schlegel-style: load 4+4, do 3 rotated
  `vceqq` compares, advance the side with the smaller max) gives a real,
  repeatable **1.6–2.8×**. This directly helps **merge-join** (`merge_join` in
  `exec.rs`), the **leapfrog/LFTJ inner intersection** (the WCOJ inner loop the
  architecture already calls out for SIMD set-intersection), and **scan↔scan
  block joins**. This is the highest-ROI NEON target.
- **Galloping / branchless binary search** in `seek()` — NEON can do a 4-wide
  compare against block bounds; modest [speculation, ~1.3–1.5×].
- **Decompression: varint / PForDelta / bit-unpacking [literature].** The
  `bitpacking` crate (Lemire's simdcomp port) already has a NEON path; SIMD
  varint decoders (Stream-VByte, Masked VByte) hit **2–4 GB/s** vs ~0.5–1 GB/s
  scalar on comparable ARM cores. This is the right SIMD spend for M3's
  compressed blocks — *use the crate, don't hand-roll*. [literature, 2–4×]
- **Dictionary probing / hash compare** — NEON tag compare (16× `u8` Bloom tags
  in one `vceqq`) for the open-addressing hash join's fast-reject; helps the
  hash-probe the architecture flags as latency-bound. [speculation, ~1.2–1.5×]
- **`vpcompress`-style selection-vector compaction** — AArch64 has no
  `vpcompressd`; compaction needs a NEON shuffle-LUT (`vqtbl1q`). Doable but
  fiddly; the architecture already (correctly) deprioritises selection-scan SIMD
  as ~10% end-to-end. [speculation, ~1.5×, low priority]

**Cost/risk in Rust.** Low–moderate. `std::arch::aarch64` intrinsics are stable;
`std::simd` (portable, lowers to NEON on AArch64 and WASM128 on the wasm target —
a free WASM win) is nightly-only today, so for stable use `std::arch` directly or
the `wide`/`bitpacking` crates. `unsafe` blocks, but no FFI, no extra deps for
hand-written kernels. Main risk is wasted effort vectorising things LLVM already
does — the spike proves you must *check the disassembly first*.

**Feasibility verdict: STRONGLY FEASIBLE, but narrowly targeted.** NEON is the
highest-ROI near-term hardware win **only if pointed at the right ops**: merge/set
intersection (joins, WCOJ — **measured 1.6–2.8×**) and SIMD decompression
(**literature 2–4×**). It is **not** a win for contiguous filter scans (already
auto-vectorized) — those are fixed by *layout* (tagged ValueIds, compression),
not by hand-written SIMD. Highest-priority M1 work item.

---

## 2. Apple AMX (matrix coprocessor via Accelerate / BLAS)

**What it is.** AMX is Apple's undocumented per-cluster matrix coprocessor,
*not* directly programmable (no public ISA), reachable only through the
Accelerate framework (BLAS/LAPACK `cblas_sgemm`, BNNS). It accelerates dense
matrix multiply / outer products at low–medium precision.

**Spike results** (`/tmp/m1-spikes/src/accel.rs`, system `cblas_sgemm`):

| matmul | time | throughput |
|---|--:|--:|
| 512×512 SGEMM | 0.22 ms | **~1250 GFLOP/s** [measured] |
| 1024×1024 SGEMM | 2.1 ms | ~1010 GFLOP/s [measured] |
| 2048×2048 SGEMM | 19.6 ms | ~875 GFLOP/s [measured] |

So AMX is *real and reachable* and delivers ~0.9–1.25 TFLOP/s of dense FP32
matmul — far above what NEON FP could do. The question is whether **any
triplestore op** maps onto dense matmul.

**Can a join be a matmul?** There is a genuine literature thread here — the
linear-algebra / GraphBLAS view of RDF, and GPU/accelerator systems built on it:

- **MAGiQ** (Jamour et al., VLDB'18) expresses SPARQL BGPs as a sequence of
  **sparse matrix–vector / matrix–matrix products** over a Boolean/integer
  semiring: a triple pattern is a sparse adjacency matrix, a join is masked
  sparse matmul (SpGEMM), and the whole BGP is a matrix expression. It ran on
  CPU, GPU and Intel-MKL backends. [literature]
- **gSmart / matrix-based RDF** and the **GraphBLAS** RDF work similarly cast
  BGP evaluation as semiring SpGEMM.

The catch is that RDF joins are **extremely sparse Boolean** SpGEMM, and **AMX
accelerates *dense* low-precision matmul, not sparse Boolean SpGEMM.** Forcing a
sparse RDF join through dense AMX would require densifying — `O(|V|²)` memory and
flops over a matrix that is ~99.99% zeros — which is catastrophically worse than
a sort-merge or hash join that touches only the non-zeros. AMX has no sparse
path, no Boolean/integer-semiring path that beats a comparison-based join, and no
gather. The reformulation overhead (build sparse matrices, densify or block,
convert results back to id tuples) dwarfs any matmul speedup.

The only sparse linear algebra that AMX could touch is **SpGEMM via dense
blocking** (chunk the adjacency matrix into small dense tiles, AMX-multiply the
tiles). For RDF's sparsity this produces almost-all-zero tiles — pathological.
MAGiQ-style systems that *did* win used **GPU sparse-matrix libraries (cuSPARSE)
and MKL sparse**, not a dense matrix unit, and they won on **very specific
shapes** (e.g. transitive-closure / reachability over a single predicate, where
the matrix is reused across many SpMV iterations) — i.e. **property paths**, not
general BGPs.

**Possible (speculative) niche:** *property-path reachability* (`?x foaf:knows+
?y`) over a single predicate's adjacency relation is repeated SpMV/SpGEMM to a
fixpoint — the one place the LA formulation is competitive in the literature.
But even there, AMX's *dense* unit is the wrong tool; you'd want a sparse SpMV
(scalar/NEON over CSR), and the engine already plans paths as BFS/DFS, which is
the same complexity without matrix overhead.

**Feasibility verdict: NOT FEASIBLE for joins/scans; marginal-and-unproven for
paths.** AMX is a dense-matmul unit; RDF joins are sparse Boolean. No general
triplestore op maps onto it without a densification that costs orders of
magnitude more than the join it replaces. The Accelerate/AMX TFLOP/s is real but
irrelevant to this workload. **Do not pursue.** (The Accelerate framework is
still useful for *vDSP* bulk ops — see §5 — just not the AMX matmul path.)

---

## 3. Metal GPU compute (8-core M1 GPU)

**What it is.** The M1 GPU is 8 cores / 128 EUs, ~2.6 FP32 TFLOP/s, programmable
via Metal compute shaders. Crucially, on Apple Silicon it shares **unified
memory** with the CPU: a `StorageModeShared` buffer is visible to both with **no
copy** — eliminating the PCIe transfer that kills discrete-GPU offload for small
ops. The `metal` Rust crate binds it cleanly, and **runtime shader compilation
(`new_library_with_source`) works without the offline Metal CLI toolchain** —
verified, the offline `metal` compiler is *not* installed on this machine
(`xcrun metal` errors: "missing Metal Toolchain"), yet the spike compiled and ran
a kernel via the system Metal framework.

**Spike results** (`/tmp/m1-spikes/src/metal.rs`):

| measurement | value | note |
|---|--:|---|
| GPU filter, 16.7M `u32`, best grid | 13.4 G rows/s, 1.26 ms | [measured] |
| **vs CPU 1-core** | 15.6 G rows/s, 1.08 ms | **CPU is faster** [measured] |
| Unified-memory buffer alloc/handoff (64 MB) | ~7–12 ms one-time | [measured] |
| **GPU dispatch+wait latency floor** (true no-op kernel) | **~80–95 µs** (min 80 µs, median 94 µs; first call ~470 µs) | [measured] |

**The honest GPU verdict, from the spike.** For a streaming, bandwidth-bound
filter the GPU is **not faster than one CPU core** — both saturate the same
~55 GB/s unified bus (§0). Unified memory removes the *transfer* cost but not the
*bandwidth* ceiling, which the CPU already hits. The GPU also carries a **~0.1 ms
dispatch floor**, so any GPU op shorter than ~0.3 ms of CPU work is a *net loss*.

This kills GPU offload for the obvious target (scan/filter) but leaves it viable
for **compute-dense, data-resident, massively-parallel** kernels where the CPU is
*not* bandwidth-bound and would take many ms:

- **GPU hash join** [literature]. GPU hash joins are a mature topic (He et al.
  "Relational Joins on GPUs", Sioulas et al. SIGMOD'19 partitioned hash joins).
  On discrete GPUs they win 2–8× *when data is resident* and the build/probe is
  large. On M1 the unified-memory advantage is real (no copy), but the M1 GPU is
  only 8 cores and the join is gather-heavy (random hash-bucket probes) =
  latency-bound, not compute-bound — the regime where GPUs are weakest. Estimated
  **1–2×** over a good morsel-parallel CPU hash join for large builds; break-even
  or loss for small ones. [speculation grounded in literature]
- **GPU sort** [literature]. GPU radix sort (e.g. `metal-experiment`/CUB-style)
  is genuinely strong: **2–5×** over CPU radix sort for large arrays. Relevant to
  **index build** (sorting the 6 permutations) and **ORDER BY**. The most
  defensible GPU win on M1. [literature 2–5×]
- **GPU WCOJ / multiway intersection** [literature/speculation]. TripleID-Q,
  gStore-GPU and "RDF on GPUs" surveys report large speedups for BGP matching on
  GPUs, but on *discrete* GPUs with high memory bandwidth and thousands of cores.
  The M1's 8 GPU cores + shared bandwidth blunt this. Leapfrog/generic-join inner
  intersections are compute-dense and parallel — a plausible GPU target — but the
  spike's bandwidth ceiling and 8-core count cap the upside at maybe **1.5–3×**
  for very large skewed intersections. [speculation]
- **GPU-RDF systems surveyed**: TripleID (triple-pattern matching as parallel
  scans), gStore-GPU (subgraph matching), MAGiQ (SpGEMM on cuSPARSE) — all built
  for discrete NVIDIA GPUs. Their core lesson is "massive parallelism over
  resident triples"; on M1 the parallelism (8 cores) and bandwidth are modest, so
  the published 5–20× speedups **do not transfer** — expect a fraction.

**The unified-memory upside that *is* real.** Because there's no copy, you can
keep the permutation columns and dictionary GPU-resident and offload **only the
expensive, compute-bound stages** of a long query (a giant sort, a billion-row
skewed intersection) while the CPU drives the pipeline — a CPU/GPU co-execution
model. This only pays off above the ~0.3 ms break-even and for ops the CPU can't
already do at memory-bandwidth speed.

**Cost/risk in Rust.** Moderate–high. `metal` + `objc` crates work and need no
offline toolchain (runtime shader compile verified). But: kernels are written in
Metal Shading Language (a second language to maintain), debugging is hard, the
WASM target cannot use Metal at all (so it'd be a macOS-only code path forking the
engine), and the spike shows the easy wins (scan/filter) don't materialise. High
engineering cost for a workload-specific, platform-specific, uncertain payoff.

**Feasibility verdict: FEASIBLE but LOW PRIORITY.** Unified memory is a genuine
M1 advantage over discrete GPUs, and `metal`-rs works without the offline
toolchain — but the spike proves the GPU does **not** beat the CPU on the
bandwidth-bound scan/filter hot paths, and carries a ~0.1 ms dispatch floor. The
only defensible GPU wins are **large sorts (index build / ORDER BY, ~2–5×
[literature])** and possibly **very large compute-bound intersections/hash
joins**. Defer until the CPU path (NEON merge + compression + tagged ids) is
exhausted; revisit GPU sort for index-build at scale.

---

## 4. Apple Neural Engine (ANE) — the honest verdict

**What it is.** The 16-core ANE is a fixed-function neural-network inference
accelerator, programmable **only** via CoreML (and, lower-level, BNNS graphs).
It executes a fixed menu of NN ops — convolution, matmul/`Linear`, pooling,
elementwise activations — at **low precision (fp16 / int8)**, and only when the
CoreML compiler decides to schedule a layer onto it (you cannot force it; CoreML
may silently fall back to GPU/CPU). It is **not** a general-purpose compute
device: there is no way to run an arbitrary join, scan, hash-probe, sort, or
comparison kernel on it. There is no gather, no branching, no integer-exact
comparison primitive exposed, no general indexing.

**Could any triplestore op be cast as an ANE-friendly ML op?** Walking through it
honestly:

- **Scan/filter** (`col > k`): this is an elementwise threshold. BNNS/CoreML
  *can* do an elementwise compare/threshold over a tensor — but (a) only in
  fp16/int8, which **cannot represent 32-bit ids or 64-bit ValueIds exactly**
  (fp16 has 11-bit mantissa — it would alias ids above 2048), making it
  *incorrect* for id comparison; (b) you'd pay tensor marshalling + CoreML
  dispatch overhead (model invocation latency is typically ~ms, far above the
  ~0.1 ms GPU floor and the ~1 ms the CPU already takes for 16M rows); (c) the
  op is bandwidth-bound anyway (§0), and the ANE shares the same memory bus.
  Net: slower and **numerically wrong**.
- **Join as matmul**: same as the AMX analysis (§2) — RDF joins are sparse
  Boolean SpGEMM; the ANE does dense low-precision matmul. Densifying is
  catastrophic, and fp16 cannot hold ids exactly. Infeasible.
- **Hashing / dictionary / sort / set-intersection**: none of these are NN ops;
  the ANE has no primitive for them. Infeasible by construction.
- **Cardinality estimation as a learned model?** This is the *only* place ML —
  and therefore the ANE — could touch the engine at all: a small learned
  cardinality/cost estimator (a tiny MLP) could run on the ANE. But (a) it would
  be a *research* direction orthogonal to the engine's measured bottlenecks; (b) a
  tiny MLP runs in microseconds on the CPU already — the ANE's fixed dispatch
  overhead would make it *slower* for a model this small; (c) the engine's
  estimator strategy (characteristic sets, block-metadata counts) is exact/cheap
  and does not need ML. So even the one ML-shaped opportunity does not benefit
  from the ANE.

**Feasibility verdict: INFEASIBLE / IMPRACTICAL — do not pursue.** The ANE is a
fixed-function fp16/int8 NN inference unit reachable only through CoreML/BNNS. No
triplestore computation maps onto it: the comparison/scan that *looks* tensor-
shaped is (a) numerically wrong in fp16 for exact id comparison, (b) bandwidth-
bound on the same bus, and (c) burdened by CoreML dispatch overhead — so it would
be both **incorrect and slower** than the CPU. Joins/sorts/hashing/intersection
have no ANE primitive at all. The only ML-shaped op (a learned cardinality
estimator) is a separate research bet that a tiny CPU MLP serves better. The ANE
contributes **nothing** to this engine. (This is the truthful answer to the
specific ANE question — it is not pessimism, it is the architecture of the device:
it executes neural-net layers, not database kernels.)

---

## 5. Accelerate / vDSP / BNNS for bulk ops

**What it is.** Beyond the AMX matmul path (§2), Accelerate exposes **vDSP**
(vectorized signal/array primitives: bulk add/mul/compare/threshold/reduce over
f32/f64/int arrays) and **BNNS** (NN graph ops, ANE/GPU/CPU-scheduled).

**Spike result.** `vDSP_vthr` (vectorized clamp/threshold) over 16.7M f32:
**7.4 G elem/s [measured]** — but this *writes* an output array, so it moves ~2×
the bytes of a count-only filter and is **read+write bandwidth-bound at ~59
GB/s** (§0). For the count-only filter that the engine actually needs, vDSP would
have to produce a mask/output too, and the plain auto-vectorized CPU loop already
runs at the same memory-bandwidth ceiling **without** an output write or an FFI
call.

**Assessment per op:**
- **Bulk compare/threshold/arithmetic** (FILTER, BIND arithmetic over a column):
  vDSP matches but does **not beat** the auto-vectorized Rust loop, and it forces
  f32/f64 (wrong for exact u32/u64 ids) and an FFI boundary. **No win.**
- **Bulk reductions** (SUM/MIN/MAX aggregates over a numeric column): vDSP
  `vDSP_sve`/`vDSP_maxv` are convenient and bandwidth-bound = same speed as a
  hand loop. Marginal convenience, no perf win. [speculation: ~1×]
- **BNNS**: routes to ANE/GPU — inherits §3/§4's verdicts. No win.

**Cost/risk.** Low (FFI to a system framework, no deps), but the precision
mismatch (f32/f64 only) makes it unsuitable for the id-exact comparisons that
dominate, and it never beats the bandwidth ceiling the CPU already reaches.

**Feasibility verdict: NOT WORTH IT.** Accelerate/vDSP is correct and easy to
call but provides **no speedup** over auto-vectorized Rust on the bandwidth-bound
bulk ops, while imposing FP precision constraints incompatible with exact id
arithmetic and an FFI boundary. Skip it. (The one genuinely useful Accelerate
entry point, AMX SGEMM, is irrelevant to RDF per §2.)

---

## 6. Unified memory, cache hierarchy, prefetch, P/E scheduling, bandwidth

These are the **architecture-level** levers that, per §0, matter *more* than any
SIMD/GPU kernel for a bandwidth-bound engine.

**Unified memory & bandwidth (~68 GB/s peak, ~62 GB/s achieved by 1 core).**
- The governing constraint (§0). Optimise for **bytes-moved**, not flops.
- **Compression directly buys throughput**: M3's ZSTD/bitpacked columns mean a
  scan reads (say) 3–4× fewer bytes → ~3–4× faster scan *even with* decompression
  cost, *because* the engine is bandwidth-bound and decompression is compute (the
  one place that idle CPU/NEON time is free). This reframes M3: column
  compression is not just a *memory* win, it is a *scan-speed* win on M1. [the
  bandwidth measurement supports this; magnitude is speculation pending M3]
- **Tagged ValueIds (M4)** remove the random `Vec<f64>` gather in q06. The gather
  spike ran at ~0.2–0.5 G rows/s vs ~3.8 G rows/s for a contiguous column — the
  gather is **8–15× slower** because it is latency-bound (random DRAM access),
  not bandwidth-bound. Inlining the numeric value into the id (or scanning a
  contiguous numeric column) converts a latency-bound gather into a
  bandwidth-bound streaming compare. **This is the real fix for q06**, and it is
  an M4 layout change, not a SIMD change. [measured gather vs contiguous gap]

**Cache hierarchy (Firestorm: 192 KB L1d/core, 12 MB shared L2 on the P-cluster;
huge by x86 standards).**
- The architecture's ~2048-id `DataChunk` is well-sized; the 12 MB P-cluster L2
  is large enough that bigger morsels can stay L2-resident. Worth tuning chunk
  size up on M1 specifically (the L2 is ~2–3× an x86 desktop's L2). [speculation]
- The 128-byte cache line (vs 64 on x86) means **prefetch and struct packing
  matter more**: `[u32;3]` = 12 B, so ~10 triples per line; column-major (M3)
  packs one column densely → far better line utilisation than row-major
  `Vec<[u32;3]>`. Another reason M3 helps on M1 specifically.

**Software prefetch.**
- The architecture already plans software prefetch for the hash-join probe
  (latency-bound random bucket access — exactly the case the gather spike shows is
  3.8→0.5 G rows/s painful). On AArch64, `core::arch::aarch64::__prefetch` /
  inline `prfm` issued ~8–16 elements ahead of the probe hides DRAM latency. This
  is **high-value** because the M1's deep out-of-order window + prefetch can
  overlap many in-flight misses. [literature: prefetch on latency-bound joins
  commonly 1.2–1.6×]

**P-core vs E-core scheduling.**
- The 4 Firestorm P-cores are ~3–4× the throughput of the 4 Icestorm E-cores.
  macOS schedules by **QoS class**, not core pinning — there is **no
  `sched_setaffinity`** on macOS; you cannot hard-pin a thread to a P-core. You
  *influence* placement via `pthread_set_qos_class_self_np` /
  `QOS_CLASS_USER_INTERACTIVE` (→ P-cores) vs `QOS_CLASS_BACKGROUND` (→ E-cores).
- Implication for the morsel scheduler: a naive `rayon` pool with 8 equal workers
  will put ~half the morsels on 3–4× slower E-cores → stragglers. **Work-stealing
  (which the architecture already specifies) is the correct mitigation**: faster
  P-cores steal from the queue more often, so static imbalance self-corrects.
  Setting the pool's QoS to user-interactive biases toward P-cores for
  latency-sensitive queries. Do **not** assume 8× scaling — realistic parallel
  speedup on M1 for a balanced compute-bound op is ~**4–5×**, not 8×, because the
  E-cores are weak and the bus saturates. [literature/measured: §0 showed 8-thread
  filter ≈ 1-thread because bandwidth-bound; compute-bound ops scale to ~4–5×]
- **Number of worker threads:** for bandwidth-bound stages, 1–2 P-core threads
  already saturate the bus (§0) → more threads waste energy and add contention.
  For compute-bound stages (joins, sort, decompression, NEON intersection), use
  all 8. A QoS-aware, stage-aware thread count beats a fixed `num_cpus` pool.
  [speculation, grounded in §0]

**Feasibility verdict: HIGHEST PRIORITY (and mostly already on the roadmap).**
The M1-specific architecture wins are: **(1) compression as a scan-speed win**
(M3), **(2) tagged ValueIds to kill the q06 gather** (M4) — measured 8–15× gather
penalty, **(3) software prefetch on the hash-probe**, **(4) work-stealing +
QoS-aware, stage-aware thread counts** (don't expect 8× — expect 4–5× on
compute, ~1× on bandwidth-bound). These are pure-Rust, portable (most help WASM
too), low-risk, and target the *measured* bottlenecks directly.

---

## 7. Prioritised M1 roadmap

Ordered by (expected impact on the measured hot paths) × (feasibility) ÷ (cost).
"Aggregate impact" is on the engine's *measured* gaps (q06 filter 20× behind
QLever at 10M; q04/q10 join overhead; index-build throughput).

| # | Optimization | Hot path | Expected speedup | Evidence | Cost/risk |
|---|---|---|--:|---|---|
| **1** | **Tagged ValueIds / contiguous numeric column** (M4) — kill the `Vec<f64>` gather in q06 | filter | **8–15×** on the gather itself | [measured] gather 0.5 vs contiguous 3.8 G rows/s | M4, planned; moderate |
| **2** | **Column compression + block scan** (M3) — fewer bytes on a bandwidth-bound bus; column-major packs 128 B lines | scan/filter/memory | **~3–4×** scan (bytes-moved) + memory win | [measured] §0 bandwidth bound; [speculation] magnitude | M3, planned; moderate |
| **3** | **NEON merge / set-intersection** kernel for merge-join, LFTJ inner loop, scan↔scan | join | **1.6–2.8×** on the intersection | **[measured]** spike | low; `unsafe`, no deps |
| **4** | **SIMD decompression** via `bitpacking` (NEON path) for M3 blocks | decompression/scan | **2–4×** decode | [literature] simdcomp/StreamVByte | low; use the crate |
| **5** | **Software prefetch on the hash-probe** (`prfm`, 8–16 ahead) | join (hash) | **1.2–1.6×** | [literature]; gather spike shows latency cost | low |
| **6** | **Work-stealing + QoS/stage-aware thread count** (P vs E cores; 1–2 threads for bandwidth-bound, 8 for compute) | all parallel | up to **~4–5×** on compute-bound (not 8×) | [measured] §0; [literature] | low–moderate (macOS QoS) |
| **7** | **GPU radix sort for index build / ORDER BY** (Metal, unified mem) | sort/index-build | **2–5×** on large sorts | [literature] | high; MSL, macOS-only |
| **8** | GPU hash join / large intersection (resident, compute-bound only) | join | 1–3×, above ~0.3 ms ops only | [speculation] | high; defer |
| — | **AMX matmul** (dense; RDF joins are sparse Boolean) | — | none | [measured] AMX real but wrong shape | — DROP |
| — | **Accelerate / vDSP bulk ops** (no win over auto-vec; fp precision) | — | ~1× | [measured] | — DROP |
| — | **Apple Neural Engine** (fp16 NN-only; incorrect for ids; CoreML overhead) | — | none / negative | [reasoned] | — DROP |

### Aggregate expectation

- **Items 1+2 (M3/M4 layout)** are the dominant win and directly close the q06
  filter gap (20× → roughly parity) and the scan/memory gaps, because they reduce
  **bytes-moved and gather latency** — the only thing that helps a bandwidth-bound
  engine on the M1. These are **not** hardware-acceleration tricks; they are the
  structural changes already on the roadmap, and the M1 bandwidth measurement is
  the strongest possible argument for prioritising them.
- **Items 3–6 (NEON intersection, SIMD decode, prefetch, QoS scheduling)** are
  the genuine, low-risk *hardware* wins, layered on top: a **measured 1.6–2.8×**
  on joins, a **literature 2–4×** on decode, plus prefetch and parallelism. They
  are pure-Rust, mostly portable to the WASM target, and target the join/decode
  hot paths the layout changes don't.
- **Item 7 (GPU sort)** is the one defensible GPU bet, for index-build scale-out;
  everything else GPU is deferred.
- **AMX, vDSP, and the ANE contribute nothing** to this workload and should be
  explicitly dropped — the spikes confirm AMX is the wrong (dense) shape, vDSP
  doesn't beat auto-vectorization, and the ANE is a fixed-function fp16 NN unit
  that is both numerically wrong for exact ids and slower than the CPU.

**The single most important M1 insight:** the engine is **memory-bandwidth-bound
on scans** (one P-core ≈ the whole 8-core CPU ≈ the GPU, all ~55–62 GB/s) and
**latency-bound on the q06 gather and the hash-probe**. No amount of extra
*compute* (more cores, GPU, AMX, ANE) speeds a bandwidth-bound scan. The wins are
**move fewer bytes** (compression, tagged ids, column-major) and **hide latency**
(prefetch, contiguous layout) — with NEON and SIMD-decode as the targeted
compute-bound multipliers on the joins and decompression that remain.

---

### Appendix: spikes (reproduce)

All in `/tmp/m1-spikes` (a scratch crate; the sparq crates were not modified).

- `src/neon.rs` — NEON-vs-scalar for gather filter, columnar f32/u32 filter, and
  sorted merge intersection. Disassembly check (`check_asm.rs`) confirms the
  scalar filter auto-vectorizes (`fcmgt.4s`). Run: `cargo run --release --bin neon`.
- `src/metal.rs` — Metal GPU parallel filter (unified-memory, runtime shader
  compile, no offline toolchain), CPU 1-/8-thread baselines, alloc/handoff cost,
  and a true no-op-kernel dispatch-latency floor.
  Run: `cargo run --release --bin metal`.
- `src/accel.rs` — Accelerate `cblas_sgemm` (AMX-backed) throughput and a vDSP
  bulk-op timing. Run: `cargo run --release --bin accel`.

Measured on this 2020 M1 MacBook Air, `rustc 1.89.0`, `aarch64-apple-darwin`,
release (`opt-level=3, lto=fat, codegen-units=1`). Absolute numbers are
machine-specific; ratios and the bandwidth ceiling are the load-bearing findings.
The Metal *offline* toolchain is absent (`xcrun metal` fails) but runtime shader
compilation via the system framework works, so the GPU spike ran successfully.
