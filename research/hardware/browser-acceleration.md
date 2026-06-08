# sparq in the browser — runtime-adaptive hardware acceleration

How a browser WASM build of sparq can detect and use the best acceleration
available on whatever device it runs on — SIMD128 (→ native NEON/SSE/AVX), WASM
threads, and WebGPU compute — with honest per-mechanism verdicts, a
feature-detection + dispatch design, and a prioritized roadmap tied to sparq's
measured bottlenecks.

This report builds on, and reconciles with, two prior reports:

- `research/hardware/consumer-and-targets.md` — device targeting; measured that
  `+simd128` gives a free ~3.6% bundle-size win but **no end-to-end query
  speedup today** because the engine has zero hand-written SIMD (only incidental
  autovectorization), and recommended writing a `core::simd` numeric-FILTER
  kernel first.
- `research/hardware/gpu-and-cloud.md` — GPU/cloud strategy; contains a
  **measured wgpu filter+count spike on this M1/Metal** (crossover ~16–64M
  elements; the host→device transfer tax is 4–12× and kills naive per-query
  offload). It concluded WebGPU is a "portable co-processor for big resident
  aggregates," not a join accelerator.

**Honesty contract.** Section 0 and the SIMD128 table in §1.1 are **measured on
this machine today** (Apple M1, 8 cores, 16 GB, macOS, Node v25.1.0, rustc
nightly 1.91, `wasm32-unknown-unknown`). The wgpu numbers in §1.3 are quoted from
`gpu-and-cloud.md`'s measured spike (same machine, same day) and labelled as
carried-over. Everything about browser API availability and other-vendor hardware
is public knowledge as of the Jan-2026 cutoff and is labelled **estimate** or
**fact (dated)** where it matters. Measured ≠ estimated, and I do not fabricate.

---

## 0. The new measured spike — does an *explicit* SIMD128 kernel beat scalar?

The prior consumer report measured the *whole engine* with `+simd128` on/off and
found no query-time win — but it explicitly flagged that the value "is locked
behind writing explicit vectorized kernels" and that this was **untested**. This
spike tests exactly that missing piece: an isolated, hand-written `core::simd`
numeric range-FILTER over a column of `i64` inline-numeric ValueId payloads — the
M4 hot path (`FILTER(?x in [lo,hi))` on inline numerics, no dictionary touch) —
built to wasm twice (baseline vs `+simd128`) and run in Node.

Scratch crate: `/tmp/simd128-spike` (standalone, **not** a sparq workspace
member, per the no-modify-crates constraint). The kernel:

```rust
// core::simd → lowers to WASM SIMD128 under +simd128 → lowers to NEON on M1.
let m = v.simd_ge(lov) & v.simd_lt(hiv);   // 2× i64 lanes per v128
acc -= m.to_int();                          // true lane = -1, subtract to count
```

> **MEASURED — i64 range-filter count, ~12.5% selectivity, Node v25.1.0, M1,
> best-of-12 ms, 3 runs (stable). Lower = better.**
> `base.*` = the no-simd128 build; `simd.*` = the `+simd128` build. `.scalar` =
> the plain `if v>=lo && v<hi` loop; `.simd` = the explicit `core::simd` kernel.
> All four counts verified equal every run.
>
> | elems | base.scalar | base.simd | simd.scalar | **simd.simd** | best-simd / best-scalar |
> |--:|--:|--:|--:|--:|--:|
> | 1 M  | 0.52 | 0.51 | 0.33 | **0.33** | ~0.97× |
> | 4 M  | 2.00 | 1.99 | 1.31 | **1.30** | ~0.99× |
> | 16 M | 7.99 | 7.94 | 5.22 | **5.16** | ~0.99× |
> | 64 M | 32.5 | 33.0 | 20.7 | **20.6** | ~0.99× |

**The honest, slightly surprising reading — three findings:**

1. **`core::simd` does nothing without the flag.** `base.scalar ≈ base.simd` at
   every size: in the no-simd128 build the portable-SIMD kernel compiles down to
   the same scalar code. So `core::simd` is necessary-but-not-sufficient — the
   `+simd128` target feature is the actual switch.

2. **`+simd128` gives a real ~1.5× on this kernel — but via autovectorization,
   not the intrinsics.** `simd.scalar` (the *plain loop*, +simd128 build) is
   ~1.5× faster than `base.scalar` (32.5 → 20.7 ms at 64M). And the explicit
   `core::simd` kernel (`simd.simd`) is **statistically identical to the
   autovectorized scalar** (within 1%). For a loop this trivial — count rows
   matching a range — LLVM's autovectorizer already does as well as hand-written
   SIMD once `+simd128` lets it emit v128 ops. (SIMD opcodes confirmed: the
   `0xFD` prefix count rose from 3 in baseline to 42 in the `+simd128` build.)

3. **Why this is not a contradiction of the prior "no speedup" result.** The
   prior report benchmarked the *real engine*, where the filter is buried in
   per-row work — materialize the id to a term, parse the number, branch into the
   dictionary — which is branchy, data-dependent, dictionary-touching code that
   **does not autovectorize and cannot be hand-vectorized either** while it stays
   in that shape. This spike removed that work by assuming the column is already
   `i64` inline numerics (the M4 representation). The result reframes the lever:

   > **The SIMD128 win is gated on the M4 *data layout* (inline-numeric ValueIds
   > in a contiguous column), not on writing SIMD intrinsics. Once the data is in
   > that shape, even a plain Rust loop gets ~1.5× from `+simd128` for free;
   > explicit `core::simd` buys little *extra* on simple count/compare kernels but
   > is still worth writing for the loops LLVM won't auto-vectorize (selection-
   > vector compaction with `vpcompressd`-style gather, varint/PFor bit-unpacking,
   > galloping/branchless merge intersection).**

So the corrected priority order is: **M4 columnar inline-numeric layout first
(unlocks the ~1.5× autovectorized filter on every device), `+simd128` flag on
(free), then explicit `core::simd` only for the non-autovectorizable kernels.**
This is a sharper, measured refinement of the prior report's recommendation.

---

## 1. What hardware acceleration a browser WASM module can actually reach

A precise inventory. The browser does **not** expose raw ISA, GPU drivers, or the
Neural Engine. It exposes four portable abstractions that the runtime lowers to
native hardware: SIMD128, threads (SAB+atomics), WebGPU, and (irrelevant here)
WebNN. Here is what each genuinely reaches and what sparq op it helps.

### 1.1 WASM SIMD128 — the primary, broadly-available lever

**What it is and what it reaches (fact).** SIMD128 is a fixed **128-bit** vector
ISA in the WebAssembly spec. The browser's wasm compiler lowers each v128 op to
the host's **native** SIMD: **NEON on Apple Silicon / ARM**, **SSE2–AVX2 on
x86**. So a single SIMD128 kernel **automatically "uses M1 NEON" in the browser**
— no separate ARM path, no detection beyond "is SIMD128 supported." This is the
cleanest answer to the user's "can the WASM environment access M1 features": yes,
NEON, transparently, via SIMD128. Availability is **universal on modern browsers**
(Chrome/Edge/Firefox/Safari all ship it; Safari since 16.4, 2023) — it is safe to
ship as the default with a tiny scalar fallback for ancient engines. `core::simd`
(portable SIMD, nightly) and `std::arch::wasm32` intrinsics both lower to it.

**Hard limit:** 128 bits only. SIMD128 does **not** reach wider vector units —
**AVX-512 on x86, SVE/SVE2 on ARMv9** are unreachable from WASM. On Apple Silicon
that costs nothing (M-series NEON *is* 128-bit, with multiple NEON units the
compiler can fill), but on an AVX-512 desktop the browser caps you at 1/4 the
native vector width. `relaxed-SIMD` (below) recovers some FMA/swizzle ops but not
width.

**Which sparq ops it helps, and the realistic gain (measured + estimated):**

| sparq op | SIMD128 fit | realistic gain | basis |
|---|---|---|---|
| **numeric FILTER over inline-`i64`/`f64` ValueIds (M4)** | excellent | **~1.5×** over no-simd128 scalar | **measured** §0; mostly autovectorization once data is columnar |
| **comparison / range pre-check** (block firstTriple/lastTriple, ORDER BY key compare) | excellent | ~1.3–2× | estimate; same lane-compare shape as §0 |
| **varint / PForDelta / bit-unpack decompression (M3)** | very good — but **needs explicit `core::simd`/intrinsics**, won't autovectorize | 3–8× decode throughput | estimate; the `bitpacking`/SIMD-BP128 path `[simd-01]`; LLVM can't auto-vectorize variable-length decode |
| **selection-vector compaction** (mask → dense indices) | good with explicit shuffle/`vpcompressd`-equivalent | 2–4× | estimate; explicit kernel required |
| **galloping / branchless merge-join intersection** | good (branchless lane compare) | 1.5–3× on the intersect inner loop | estimate; `[join-02]` |
| **dictionary `string→id` scan/compare** | medium | 1.2–1.5× | estimate; bounded by the binary-search, not the compare |

**Bundle cost (measured, prior report):** `+simd128` makes the bundle **~3.6%
smaller** (vectorized loops are fewer denser ops), so it is *not* in tension with
the minimal-bundle goal. **Turn it on in the wasm release profile now.**

**Verdict:** SIMD128 is the primary lever — broadly available, free on bundle
size, and it transparently uses M1 NEON. But its query-time payoff is gated on the
**M4 columnar inline-numeric layout** (measured §0); the flag alone does nothing
to the current per-row-materializing filter.

### 1.2 WASM threads — SharedArrayBuffer + Web Workers + wasm atomics

**What it is (fact).** Parallelism in WASM = a `SharedArrayBuffer` (SAB) backing
the wasm linear memory, shared across **Web Workers**, with **wasm atomics** for
synchronization (`wasm-bindgen-rayon` packages this for Rust). This gives sparq
the same morsel work-stealing parallelism as native: parallel scans, parallel
hash-join build/probe, parallel index build.

**The gating constraint (fact, and it is severe).** SAB is only available when the
page is **cross-origin isolated** — the server must send
`Cross-Origin-Opener-Policy: same-origin` **and**
`Cross-Origin-Embedder-Policy: require-corp`. This:
- **cannot be set on many static hosts** (GitHub Pages historically couldn't;
  needs server header control or a service-worker shim);
- **breaks third-party embeds** (every cross-origin resource must opt in via
  CORP/CORS) — fatal for a Solid/RDFJS *drop-in* that loads into someone else's
  page.

So threads **cannot be the default**. Feature-detect `globalThis.crossOriginIsolated
=== true` (verified in §2; it is `undefined` outside isolation) and only then load
the threaded build. For a first-party app that controls its headers, threads are a
real upside (near-linear on scan/join up to the core count, minus big.LITTLE
scheduling noise on phones); for an embed, single-thread is the realistic default.

**Verdict:** high upside, opt-in only, behind a `crossOriginIsolated` gate and a
separate (larger) build artifact. Matches `ARCHITECTURE.md §6`.

### 1.3 WebGPU compute — `navigator.gpu`

**What it is and what it reaches (fact).** WebGPU (`navigator.gpu`) is the
browser's modern GPU-compute API. The browser runs WGSL compute shaders on the
**real device GPU** via the platform backend: **Metal on Apple Silicon**, D3D12 on
Windows, Vulkan on Linux/Android. So in the browser on this M1, a WebGPU compute
kernel runs on the **M1 GPU via Metal** — the exact thing the user asked about.
Availability (fact, dated): shipping in Chrome/Edge (since 113, 2023) and Safari
(since 17.4 on Apple platforms, 2024); Firefox rolling out. **Mobile Safari/Chrome
support is partial/recent** — do not assume it on phones.

**Reconciling with the prior "WebGPU is the wrong fit."** The prior reports are
correct *for the join path* and I do not overturn that. The precise statement is:

> WebGPU is **poor for latency-bound, pointer-chasing binary joins** (the bulk of
> SPARQL): irregular memory access, thread divergence on trie descent, and a
> ~1 ms fixed kernel-launch latency that exceeds whole small queries. But it is
> **potentially good for bulk, embarrassingly-parallel, throughput-bound ops over
> millions of resident rows** — and sparq has a few of those.

**Per-op honest verdict (compute = data already in VRAM; e2e includes upload):**

| op | WebGPU verdict | crossover (this M1, measured) |
|---|---|---|
| **full-column predicate scan / FILTER → COUNT/aggregate** | **good** — few rows out, embarrassingly parallel | wins above **~16–64 M rows** resident; ~2–3× all-core CPU at 64 M `[measured, gpu-and-cloud.md]` |
| **bulk COUNT / GROUP BY aggregation** | **good** — reduction is a GPU strength | same threshold; small result leaves GPU |
| **parallel sort (index build, merge-join prep)** | **good** — radix sort is GPU's best trick (5–20× resident, lit.) — **but** sparq's permutations are *already sorted*, so the win is only at *build* time | build-time only; large key arrays |
| **bulk dictionary encode (build)** | **poor** — branchy, variable-length string hashing + dedup; build is parse/IO-bound | n/a |
| **decompression** | **bit-unpack/PForDelta good; ZSTD poor** (Huffman/FSE stage is serial per block) | adopt bit-packing (M3) for a GPU path; keep ZSTD on CPU |
| **WCOJ multiway intersection** | **maybe** — warp-level set-intersection helped on datacentre GPUs (lit.), but irregular trie descent diverges; unproven on integrated GPUs | research-grade; not near-term |
| **binary merge/hash join** | **poor** — latency-bound pointer chasing; the prior verdict stands | never on integrated GPU |

**The data-size threshold and why it dominates (measured, carried over).** From
the wgpu spike in `gpu-and-cloud.md` on this exact M1: GPU **compute-only** beats
the all-core CPU only at **~16–64 M elements** (at 64 M, ~2–3× CPU). Below ~4 M,
the ~1 ms launch latency makes the GPU **slower than a single CPU core**.
Critically, **GPU end-to-end (re-upload the column per query) is 4–12× slower than
CPU at every size** — even on M1 where "upload" is just a unified-memory `memcpy`,
not a bus transfer. On a discrete GPU across PCIe it is strictly worse. **Add the
JS↔WASM↔GPU handoff** in the browser (copy wasm linear memory → a
`GPUBuffer` via `device.queue.writeBuffer`, dispatch, map-read back) and the
fixed per-call tax is *higher* than the native spike measured.

> **Therefore the only viable browser-WebGPU design is data-resident:** upload the
> relevant permutation column(s) / numeric-value cache into `GPUBuffer`s once at
> load, and dispatch many filter/aggregate kernels against them across queries.
> Per-query column streaming is a non-starter. This restricts WebGPU to a
> **resident bulk-aggregate co-processor for large (>~10 M-row) datasets** — i.e.
> the analytics tail, not interactive point queries. For the consumer graph sizes
> sparq targets in a browser tab (often <few M triples, RAM-capped at ~2 GB),
> **the GPU rarely clears its own crossover.** It earns its place only on the
> large-dataset, aggregate-heavy minority — and there it is a genuine, novel
> capability (no competitor ships a browser SPARQL engine that offloads to
> WebGPU).

### 1.4 Other mechanisms — honest verdicts

- **`Atomics`** (fact): the synchronization primitive for §1.2 threads; also
  enables lock-free morsel cursors. No standalone value without SAB.
- **`Memory64` / wasm64** (fact, dated — Phase 4 / shipping behind flags, broad
  availability still maturing in 2025–26): lifts the **4 GB linear-memory ceiling**
  that caps how big a graph fits in a tab, at the cost of 64-bit pointers
  (larger heap, slightly slower). **Relevant** to sparq because wasm32's 4 GB (and
  many tabs' ~2 GB practical cap) is *the* gate on dataset size in-browser. Verdict:
  **watch and adopt when broadly shipping**; until then, M3 compression + a
  prebuilt `.sparq` index is the way to fit more in 32-bit space. Not a near-term
  default.
- **`relaxed-SIMD`** (fact, dated — shipping in Chrome/Firefox; Safari later):
  adds FMA, relaxed swizzle/lane-select, and dot-product ops on top of SIMD128.
  Modest, *non-deterministic* gains (results may differ per host for FMA), so only
  for ops where bit-exactness doesn't matter (not FILTER correctness). **Low
  priority**; helps decompression/scan math marginally.
- **JS `Float16` / `f16`** (fact): a JS numeric type, not a wasm-accel path
  relevant to a triplestore (RDF numerics are i64/f64/decimal/date). **Irrelevant.**
- **Neural Engine (ANE)** (fact, important honest answer): **the ANE is NOT
  reachable from a browser.** Not from WASM, not from WebGPU, not from any web API.
  The *only* indirect path to Apple's ML accelerators from the web is **WebNN**
  (`navigator.ml`), which schedules **neural-network graphs** (conv, matmul,
  activations) onto CoreML/NPU/GPU. WebNN is for ML inference; **a sorted-merge /
  WCOJ triplestore has no neural-net-shaped op**, so WebNN is **almost certainly
  irrelevant to sparq**. The one theoretical stretch — casting set-intersection or
  aggregation as a matmul (the MAGiQ "SPARQL as sparse linear algebra" idea) —
  would be a *different engine*, not a bolt-on, and WebNN's API is not designed for
  sparse integer ops anyway. **Verdict: do not pursue WebNN/ANE.** The user asked;
  the honest answer is "the ANE is walled off from the web, and the bridge that
  exists is the wrong shape for this workload."

---

## 2. Runtime feature-detection + dispatch design for sparq-wasm

The goal: one small default bundle that runs everywhere, **probes the device at
startup**, and dispatches each op to the best available path with graceful
fallback to the portable scalar/SIMD128 WASM path. Verified detection surfaces
(probed in Node v25 as a sanity check — Node has no `navigator.gpu` and
`crossOriginIsolated === undefined` outside isolation, exactly as a non-isolated
browser tab without WebGPU would report):

```js
const caps = {
  // SIMD128: validate a tiny module that uses a v128 op (cheap, synchronous).
  simd128: WebAssembly.validate(SIMD_PROBE_BYTES),         // ~universally true now
  // Threads: SAB requires cross-origin isolation. Both must hold.
  threads: (typeof SharedArrayBuffer === "function") && (globalThis.crossOriginIsolated === true),
  // WebGPU: navigator.gpu exists AND an adapter is grantable (async).
  webgpu: !!(navigator.gpu),                               // confirm with requestAdapter()
};
```

**Sketch API — an async `Store.create()` that probes capabilities.** The current
API (`Store.load(text, format)` static, synchronous; `query()` → JSON string)
stays as the simple path. Add an async builder that picks the engine profile:

```js
import init, { Store } from "./sparq_wasm.js";   // default: SIMD128 + scalar fallback
await init();

// Capability-aware constructor. Probes SIMD128 (sync), crossOriginIsolated (sync),
// and navigator.gpu.requestAdapter() (async). Lazy-imports heavy paths.
const store = await Store.create(turtleText, "turtle", {
  threads: "auto",   // "auto" → use only if crossOriginIsolated; "off" forces single-thread
  webgpu:  "auto",   // "auto" → probe adapter; only engages for large resident datasets
});

const json = store.query("SELECT ... ");   // dispatches per-op internally
```

**Internal dispatch, per operation (the routing table):**

| op | path chosen at runtime |
|---|---|
| numeric FILTER / compare / aggregate over inline numerics | **WebGPU** if `caps.webgpu && resident && rows > ~10 M`; else **SIMD128 kernel** (autovectorized + `core::simd`); else **scalar** |
| scan / merge-join / hash-join | **threads** (morsel work-stealing) if `caps.threads`; else single-thread; SIMD128 inner loops always |
| decompression (M3) | SIMD128 `core::simd` bit-unpack if `caps.simd128`; else scalar |
| everything else (planner, parser, path eval) | portable scalar/SIMD128 — no special path |

**Keeping the bundle small (the critical constraint — today 210 KB brotli):**

1. **SIMD128 + scalar in the default bundle.** SIMD128 is free on size (−3.6%
   measured) and universal; the scalar fallback is the same Rust compiled without
   the kernel hot path. One artifact covers ~every device.
2. **Threads = a *separate* artifact**, lazy-imported only when
   `crossOriginIsolated`. `wasm-bindgen-rayon` needs a worker-spawn glue and an
   atomics-enabled build; never ship it to non-isolated pages.
3. **WebGPU = a *separately lazy-loaded* JS+WGSL module.** The WGSL shaders and
   the `GPUBuffer`-management glue are JS, not wasm, so they add **zero** to the
   wasm bundle and are `import()`-ed only after `navigator.gpu.requestAdapter()`
   succeeds **and** the dataset clears the size threshold. A device with no WebGPU,
   or a small graph, never downloads them.

**The portability multiplier — one kernel serves browser + native.** This is the
strategic core of the user's directive:

- A **`core::simd` kernel** compiles to **WASM SIMD128 in the browser** *and* to
  **NEON on native ARM (Apple Silicon, mobile, Pi)** *and* **SSE/AVX2 on native
  x86** — one source, every CPU target. (§0 measured this lowering works.)
- A **`wgpu` kernel** (WGSL/naga) compiles to **WebGPU in the browser** *and* to
  **Metal / Vulkan / DX12 / CUDA-via-Vulkan natively** — one source, every GPU
  target. (`gpu-and-cloud.md` measured this kernel builds and runs first-try on
  Metal, and the same source targets Vulkan/DX12/WebGPU.)

So the browser-acceleration work is **not** browser-only cost: every kernel
written for the browser path simultaneously accelerates the native engine on the
same class of hardware. That is the whole reason to invest in `core::simd` and
`wgpu` rather than `std::arch::wasm32` intrinsics or CUDA-only code.

---

## 3. Prioritized browser-acceleration roadmap (tied to measured bottlenecks)

sparq's measured pain points (from `BENCHMARKS.md`): the **numeric FILTER is up to
20× slower than QLever at 10M triples** (q06 0.05×), the gap **widens with scale**
because the store is flat uncompressed `Vec<[u32;3]>`, plus **large-result
materialization** and **index build**. The browser adds two binding constraints:
the **~2 GB tab memory cap** and the **no-COOP/COEP-by-default** reality. The
roadmap below orders by (gain × reach ÷ cost) and **prefers cross-cutting kernels
that serve browser + native**.

### Step 1 (do first) — M4 columnar inline-numeric layout + SIMD128 FILTER kernel

- **What:** land the M4 tagged-ValueId inline-numeric representation as a
  *contiguous `i64`/`f64` column*, flip `+simd128` on in the wasm release profile,
  and write the numeric range-FILTER / compare as a `core::simd` kernel.
- **Why first:** it attacks the **single worst measured gap** (20× on numeric
  FILTER) on the **highest-reach target** (every browser device). §0 measured this
  yields **~1.5×** on the kernel from `+simd128` alone once the data is columnar —
  and the data-layout change is the part that matters (the prior report's key
  correction). The kernel is **`core::simd`, so it simultaneously speeds native
  NEON/AVX2** — it amortizes across the top three device targets at once.
- **Gain:** ~1.5× from SIMD128 *plus* the much larger win from killing the per-row
  dictionary materialization the column layout removes (that per-row parse is why
  q06 is 20× off, not the lack of SIMD). **Expected: close most of the 20× gap.**
- **Cost:** SIMD128 flag is free (−3.6% bundle). M4 inline ValueIds are already on
  the roadmap. Kernel complexity: low (≈ the §0 spike).

### Step 2 — M3 column compression + SIMD128 bit-unpack decode (the memory gate)

- **What:** column-major compressed blocks (bit-packing / PForDelta + ZSTD) with a
  `core::simd` bit-unpack decode kernel; ship a **prebuilt `.sparq` index**
  zero-copy-loaded from an `ArrayBuffer` (and persisted to **OPFS**) instead of
  parsing Turtle client-side.
- **Why second:** the **2 GB tab cap is the gate on whether sparq runs at all** on
  a phone; the flat `Vec<[u32;3]>` is 6× redundant and uncompressed. Compression
  is simultaneously the memory fix, the bandwidth/thermal fix, and the
  download-size fix — and it directly addresses the "gap widens at scale" finding.
- **Gain:** runs on far larger graphs in a tab; faster scans (fewer bytes touched);
  SIMD bit-unpack 3–8× decode (estimate). Decompression is the one place explicit
  `core::simd` clearly beats autovectorization (variable-length, won't auto-vec).
- **Cost:** medium (block format + skip metadata). Already M3 on the roadmap.

### Step 3 — opt-in WASM threads behind `crossOriginIsolated`

- **What:** a *separate* `wasm-bindgen-rayon` build, lazy-loaded only when
  `crossOriginIsolated === true`, for parallel morsel scans / hash-join build.
- **Why third:** big upside on high-end laptops, but **zero reach where hosting
  headers can't be set** (most embeds), so it must be opt-in and never default.
  Lower priority than the universal SIMD/compression wins.
- **Gain:** near-linear on scan/join up to core count for first-party apps.
- **Cost:** medium-high (worker glue, atomics build, COOP/COEP docs); a second
  artifact to maintain.

### Step 4 (opportunistic, large datasets only) — WebGPU resident bulk-scan/sort

- **What:** a lazy-loaded WebGPU path (`wgpu` kernel, shared with native GPU) that
  keeps numeric columns resident in `GPUBuffer`s and dispatches filter/aggregate
  kernels — engaged **only** when `navigator.gpu` exists **and** the dataset
  clears the **~10 M-row** crossover.
- **Why last:** measured crossover is high (~16–64 M rows) and the per-query
  transfer/handoff tax kills naive offload; most browser graphs never clear it. But
  for the large-dataset, aggregate-heavy tail it is a **genuine, novel capability**,
  and the **`wgpu` kernel doubles as the native Metal/Vulkan/CUDA aggregate path**
  (`gpu-and-cloud.md`), so the work is shared with the server-GPU strategy.
- **Gain:** ~2–3× over all-core CPU on large resident aggregate scans (measured on
  M1); novel browser-WebGPU SPARQL capability.
- **Cost:** high (residency management, WGSL, the 65535-workgroup dispatch cap and
  other portability sharp edges the prior spike hit). Lazy-loaded → **zero default
  bundle cost**.

**Cross-cutting emphasis.** Steps 1, 2, and 4 each produce a kernel that serves
**both browser and native**: the `core::simd` FILTER/decode kernels lower to WASM
SIMD128 *and* NEON/AVX2; the `wgpu` aggregate kernel runs on WebGPU *and* native
Metal/Vulkan/CUDA. Writing them for the browser is *the same work* as writing them
for the native engine — that is the leverage that makes this directive worth it
rather than a browser-only side-quest.

---

## 4. Recommended concrete first step (fits the existing M3/M4 plan)

**Implement the M4 inline-numeric columnar layout for the numeric-FILTER path,
enable `+simd128` in the wasm release profile, and add a single `core::simd`
range-FILTER kernel over that column — then measure with the committed
`crates/sparq-wasm/test/bench.cjs` harness (current WASM filter-age ~3.7 ms /
filter-heavy ~9.7 ms are the numbers to beat).**

Why this exact step:

- It attacks sparq's **single worst measured gap** (numeric FILTER, up to **20×**
  slower than QLever at 10M, `BENCHMARKS.md`) on the **highest-reach target**
  (every browser device — phones, laptops, Chromebooks via one artifact).
- §0 **measured** that the win is real (~1.5× from `+simd128`) but **gated on the
  M4 columnar layout**, not on SIMD intrinsics — so the data-layout change is the
  load-bearing part and it is already an M4 deliverable. This is a precise,
  measured correction to the prior report's "write a SIMD kernel" framing: write
  the *layout* first; the SIMD then comes nearly for free.
- The kernel is **portable `core::simd`**, so it lands the same acceleration on
  **native ARM64/NEON and x86/AVX2** — it amortizes across the top targets and
  feeds directly into the M4 milestone in `ARCHITECTURE.md §3.4` (vectorized
  DataChunk execution) rather than being browser-only throwaway work.

Sequence after that, in order: **M3 column compression + SIMD bit-unpack +
prebuilt `.sparq`/OPFS** (unlocks the 2 GB mobile tier and the memory/thermal
wins) → **opt-in threads behind `crossOriginIsolated`** (first-party apps) →
**lazy WebGPU resident bulk-aggregate** for the large-dataset tail (shared kernel
with the native-GPU path). Park WebNN/ANE permanently — the ANE is unreachable
from the web and the workload is the wrong shape for it.

---

### Appendix — reproducing the §0 spike

Standalone crate at `/tmp/simd128-spike` (scratch; **not** a sparq workspace
member). Build both artifacts and benchmark:

```sh
cd /tmp/simd128-spike
RUSTFLAGS="-C target-feature=+simd128" rustup run nightly cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/simd128_spike.wasm spike_simd.wasm
rustup run nightly cargo build --release --target wasm32-unknown-unknown   # baseline
cp target/wasm32-unknown-unknown/release/simd128_spike.wasm spike_base.wasm
node bench.mjs
```

The kernel uses `#![feature(portable_simd)]` (`core::simd`), so it requires a
nightly toolchain; the production engine would use `core::simd` on nightly or
`std::arch::wasm32` intrinsics on stable. The wgpu spike referenced in §1.3 lives
at `research/hardware/wgpu-spike/` (`cargo run --release`).
