# sparq on consumer hardware — targets and how to optimize for them

A research report on which consumer-hardware platforms a "highly performant
triplestore for consumer devices" should optimize for, and the concrete engine
techniques per target. sparq already has two delivery vehicles: the native Rust
engine (`sparq-core` + `sparq-engine`, six sorted permutations, merge/hash/WCOJ,
the M1–M5 roadmap in `research/ARCHITECTURE.md`) and a minimal-bundle WebAssembly
build (`crates/sparq-wasm`, ~210 KB brotli today). This report is grounded in
those docs and in a measured spike (Section 5).

**Honesty contract.** Everything in Section 5 is measured on this machine.
RAM/arch/bundle constraints in Sections 1–2 are from public spec knowledge and
are labelled as estimates where not directly measured here. Numbers carried over
from `research/BENCHMARKS.md` (sparq vs QLever/Oxigraph) are cited as such.

---

## 0. TL;DR

- **The browser/WASM path is the highest-reach target by far** and it is the one
  sparq has already invested in. Optimize it first.
- A measured spike (build with `RUSTFLAGS="-C target-feature=+simd128"` vs
  without, benchmarked in Node) shows **WASM SIMD128 does NOT meaningfully speed
  up sparq today** — best case ~6 % on a pure scan, everything else inside
  run-to-run noise — because **the engine contains zero hand-written SIMD**; the
  only gains are incidental LLVM autovectorization. The one reliable SIMD128 win
  is that the bundle got **~3.6 % smaller** (805 KB → 777 KB raw). The lesson:
  SIMD on consumer devices is a real lever, but sparq has to *write* the
  vectorized kernels (M3/M4) before `+simd128` pays off.
- The cross-cutting wins that help **every** consumer target are the same three
  the architecture already prioritizes for the server: **smaller index via
  compression** (M3), **portable SIMD kernels that lower to both NEON and
  WASM-SIMD128** (M4), and **memory-bounded streaming** (M4). Consumer hardware
  just raises their priority because RAM and bundle size are the binding
  constraints, not disk or core count.
- **Recommended first 4 targets:** (1) WASM single-thread on Apple Silicon / mid
  laptops, (2) WASM single-thread on mobile Safari/Chrome (A-series / Snapdragon
  8-gen), (3) WASM threads behind cross-origin-isolation for high-end laptops,
  (4) native ARM64 (Apple Silicon, mobile, Raspberry Pi 4/5) as the same codebase
  minus the bundle constraint.

---

## 1. Top consumer-hardware platforms for a client-side/edge triplestore

Ranked roughly by global install base × relevance to an in-browser/edge RDF
store. CPU/RAM figures are typical-device estimates from public specs, not
measured here.

| # | Platform | CPU arch / SIMD | Typical RAM | Browser / WASM reality | What limits an RDF store here |
|---|---|---|---|---|---|
| 1 | **Mid-range Android** (Snapdragon 6/7-gen, MediaTek Dimensity 700–8000) | ARMv8-A, **NEON** (128-bit); no SVE in this tier | 4–8 GB, but per-tab budget tiny | Chrome/WebView; WASM SIMD128 + threads broadly available on recent Chrome; **tab can be killed under memory pressure** | RAM + GC pressure + thermal throttle; this is the *binding* constraint tier |
| 2 | **High-end Android** (Snapdragon 8-gen 1/2/3, Dimensity 9000+) | ARMv9-A, **NEON**, **SVE2** (but WASM can't reach SVE) | 8–16 GB | Same as above, faster cores; big.LITTLE means a JS/WASM thread may land on a little core | Thermal throttling on sustained work; big.LITTLE scheduling unpredictability |
| 3 | **Apple iPhone (A-series)** | ARMv8.5+/v9, **NEON** (no SVE exposed) | 6–8 GB | **Safari/WebKit**: WASM SIMD128 yes; **threads require COOP/COEP cross-origin isolation** and SAB; Safari historically lagged here | iOS Safari per-tab memory limits are aggressive; WebKit JIT/WASM quirks |
| 4 | **Apple Silicon iPad / Mac** (M1–M4) | ARMv8.5+, **NEON** (128-bit), wide OoO cores, big caches | 8–24 GB (16+ common) | Best-in-class WASM perf; SIMD128 + threads supported; **native** target trivial | Almost nothing at consumer graph sizes — this is the comfortable case |
| 5 | **x86-64 laptop, mainstream** (Intel Core i5/i7, AMD Ryzen 5/7) | x86-64, **SSE/AVX2** everywhere, **AVX-512** only on some (Zen4/5, a few Intel) | 8–16 GB | All browsers; WASM SIMD128 maps to SSE/AVX; threads via SAB | Mid laptops are RAM-limited at 8 GB; AVX-512 not assumable |
| 6 | **ARM Windows laptop** (Snapdragon X / X Elite, older SQ-series) | ARMv8/v9, **NEON** | 16 GB typical (X Elite) | Edge/Chrome; WASM SIMD128 + threads; x86 apps run under emulation but WASM is native | Emerging tier, small base; NEON path is the same as Apple Silicon |
| 7 | **Raspberry Pi 4 / 5 and Pi-class SBCs** | ARM Cortex-A72 (Pi4) / A76 (Pi5), **NEON**; no SVE | 2–8 GB | Native target (Linux ARM64); browser possible but slow | RAM (2–4 GB common) + modest memory bandwidth; the classic "edge node" |
| 8 | **Chromebook** (ARM or low-end x86) | ARM NEON or x86 SSE/AVX2 | 4–8 GB | Chrome-first; WASM SIMD128 + threads | Low RAM, often the cheapest silicon; pure browser workload |
| 9 | **Smart TV / set-top / console browser** | ARM (mostly), NEON; weak cores | 1.5–4 GB usable | Cut-down WebKit/Chromium; **WASM threads/SIMD support is spotty and old**; SAB often unavailable | Very tight RAM, old browser engines, no threads — assume the *baseline* WASM profile |
| 10 | **Wearables / low-end IoT** (some Cortex-A, some Cortex-M) | ARM; Cortex-M has no MMU, can't run general WASM runtimes well | <1 GB | Mostly out of scope for a general triplestore | Memory and the absence of a real browser; treat as non-target for now |

**Reach observation.** Targets 1–8 are all reachable by **one artifact**: the
WASM build. That is the strategic point — optimizing the WASM path lifts the
entire mobile + laptop + Chromebook fleet simultaneously, where shipping native
binaries would mean per-OS, per-arch builds and an install step the browser does
not require.

---

## 2. What to target, and the technique per target

### 2.1 The WASM path (targets 1–8, the high-reach lane)

**WASM SIMD128.** A single 128-bit vector ISA that the engine writes once and the
browser lowers to **NEON on ARM** and **SSE/AVX on x86** — exactly the portability
property sparq wants. `core::simd` (portable SIMD) and many hand-written kernels
lower to it cleanly. *But see Section 5: it only helps if you actually write
vector kernels.* sparq's hottest consumer ops are the right shape for it:
- numeric **FILTER** over a column of inline ValueIds (M4) — compare 4×f64 or
  2×i64 lanes per instruction;
- **merge-join intersection** of two sorted id runs (branchless lane compare);
- **block decompression** (the `bitpacking`/SIMD-BP128 path the architecture
  already names for M3/`[simd-01-simd-bp128-fastpfor]`);
- dictionary **scan/compare** during `string→id`.

**Bundle size vs speed.** Today's bundle is **210 KB brotli** (`README.md`), and
the spike shows `+simd128` *reduces* raw size ~3.6 %, so SIMD128 is not in tension
with the minimal-bundle goal — turn it on. The real bundle cost is the SPARQL
parser (`spargebra`/`peg`) + `oxttl`/`oxrdf` + the unused `rand` path
(`README.md`); those, not SIMD, are where size-reduction effort belongs (drop
`rand`, parse-on-demand, `opt-level="z"`, `twiggy` pruning).

**Threads via SharedArrayBuffer.** Web Workers + `wasm-bindgen-rayon` give
parallel scans/joins, but **only under cross-origin isolation** — the page must
send `COOP: same-origin` and `COEP: require-corp` headers, which breaks
third-party embeds and is impossible on many static hosts. So threads must be an
*optional, feature-detected* build (`crossOriginIsolated === true`), never the
default. The architecture already states this (`§6` WASM end-state). For a
Solid/RDFJS drop-in that has no control over hosting headers, **single-thread is
the realistic default**; threads are an opt-in upgrade for first-party apps.

**Memory limits.** `wasm32` is a 32-bit address space: the hard ceiling is **4 GB
linear memory**, and many browsers/tabs cap well below that (often **~2 GB**, less
on mobile). This is *the* constraint that decides whether sparq runs at all on a
phone. Consequences for the engine:
- the in-memory `Vec<[u32;3]>` permutation store (M1) is **6× redundant and
  uncompressed** — fine on a server, ruinous in a 2 GB tab. M3 **column
  compression** is not a "nice to have" for browsers, it is the gate.
- prefer **shipping a prebuilt compressed `.sparq` index** and zero-copy-loading
  it from an `ArrayBuffer` over parsing Turtle client-side, both to save the parser
  in the bundle and to avoid the transient parse buffer doubling memory
  (`§6`).
- `memory64` (wasm64) lifts the 4 GB cap but is not broadly shipping and doubles
  pointer width; not a near-term consumer target.

**Persistence.** **OPFS** (Origin Private File System) gives synchronous,
fast file handles in a Worker and is the right backing store for a memory-mapped
`.sparq` index in the browser; **IndexedDB** is the universally-available fallback
(async, slower, but works without COOP/COEP). A consumer sparq should support
"fetch index once → persist to OPFS → zero-copy load on subsequent visits."

### 2.2 Native mobile / SBC ARM (targets 1–4, 7 as native)

Same Rust source, no bundle constraint, full `std`.
- **NEON** directly via `std::arch::aarch64` or `core::simd`; the *same* kernels
  written for WASM-SIMD128 lower here, so the work is shared (see §4).
- **big.LITTLE scheduling:** a morsel-parallel engine must not assume homogeneous
  cores. A work-stealing pool (the architecture's morsel design) naturally adapts
  — fast cores steal more — which is the right primitive; **pinning to specific
  cores is counterproductive on big.LITTLE** because the OS migrates threads for
  power/thermal reasons. Prefer rayon-style work-stealing over `core_affinity`
  pinning on mobile.
- **Thermal throttling:** sustained scans will downclock a phone within seconds.
  This rewards **doing less work per query** (compression → fewer bytes touched,
  streaming → stop early on LIMIT/ASK) far more than raw parallelism, which just
  heats the SoC faster. Memory-bounded streaming is the mobile-friendly design.
- **SVE/SVE2** exists on ARMv9 (target 2) but is **not reachable from WASM** and is
  awkward to use portably even natively (vector-length-agnostic). Not worth
  targeting for a consumer engine; NEON is the portable floor.

### 2.3 Where a portable approach pays off across many devices

- **Portable SIMD (`core::simd`)** is the highest-leverage choice: one kernel →
  NEON + SSE/AVX2 + WASM-SIMD128. Write filters/intersections/decompression once.
- **wgpu / WebGPU for GPU offload** is *portable in principle* (Vulkan/Metal/DX12
  native, WebGPU in-browser) but is a **poor fit for sparq's workload**: RDF joins
  are pointer-chasing and memory-latency bound (`ARCHITECTURE.md` §1, §3.4), not
  the dense-compute GPUs excel at; data-transfer and kernel-launch overhead would
  dominate at consumer graph sizes; and WebGPU availability on mobile Safari is
  still partial. **Recommendation: do not invest in GPU for the join path.** The
  only plausible GPU use is bulk operations (sort during index build, large
  aggregation), which is not the consumer hot path. Park it.

---

## 3. Prioritized device-target matrix

Score = Reach × Feasibility × Performance-upside, each 1–5 (5 best),
Total = product (max 125). Reach = install base reachable. Feasibility = how
ready sparq is to ship there *today*. Upside = how much headroom optimization
unlocks.

| Target | Reach | Feasibility | Upside | Score | Notes |
|---|--:|--:|--:|--:|---|
| **WASM single-thread, Apple Silicon / mid laptop** | 5 | 5 | 4 | **100** | Already builds & runs (spike). Fastest WASM hosts. The proving ground. |
| **WASM single-thread, mobile (A-series / SD 8-gen)** | 5 | 4 | 5 | **100** | Highest reach. RAM cap is the gate → M3 compression unlocks it. |
| **Native ARM64 (Apple Silicon / mobile / Pi 4–5)** | 4 | 5 | 4 | **80** | Same code, no bundle limit; NEON kernels shared with WASM. |
| **WASM threads (COOP/COEP), high-end laptop** | 3 | 3 | 5 | **45** | Big upside but gated on hosting headers + feature detection. |
| **x86-64 native/desktop** | 3 | 5 | 3 | **45** | Easy but least "consumer-edge"; overlaps the server target. |
| **Smart TV / set-top WASM** | 2 | 2 | 2 | **8** | Old engines, no threads, tiny RAM → baseline profile only. |
| **GPU via wgpu/WebGPU** | 2 | 1 | 1 | **2** | Wrong workload shape; do not invest now. |

**Recommended first 3–4, with reasoning:**

1. **WASM single-thread (Apple Silicon / mid laptops).** Already works; it is the
   development and benchmarking surface for every WASM optimization. Concrete
   engine work: **WASM-SIMD128 numeric-filter kernel** over inline ValueIds (M4),
   which the spike shows is currently *missing* and is sparq's worst measured gap
   (numeric FILTER `q06` was up to 20× slower than QLever at 10M triples,
   `BENCHMARKS.md`).
2. **WASM single-thread on mobile.** Same artifact, highest reach. The gating work
   is **front-coded / compressed vocab + column-compressed permutations (M3)** so
   a useful graph fits inside a ~2 GB mobile tab. Pair with **shipping a prebuilt
   `.sparq` index** so the client never holds parse buffers.
3. **Native ARM64.** Captures Pi-class edge nodes and any native-app embedding for
   the cost of one extra build target, and — critically — its **NEON kernels are
   the same `core::simd` kernels as the WASM build**, so this target *amortizes*
   the SIMD investment rather than competing with it.
4. **WASM threads (opt-in).** Only after 1–3: add **Web-Worker parallel joins /
   morsel scans** behind a `crossOriginIsolated` feature gate for first-party apps
   that can set COOP/COEP. High upside on high-end laptops, but zero reach where
   hosting headers can't be controlled, so it must never be the default path.

Concrete engine-change mapping:
- "WASM-SIMD128 numeric filter" → target 1, closes the measured FILTER gap.
- "front-coded compressed vocab to fit mobile RAM" → target 2, M3 work.
- "Web-Worker parallel joins" → target 4, opt-in only.
- "portable `core::simd` intersection/decompression kernels" → shared by 1, 2, 3.

---

## 4. Cross-cutting wins (help EVERY consumer target)

1. **Smaller index via compression (M3).** Less memory → fits a 2 GB mobile tab;
   fewer bytes touched → less work → less heat and faster on memory-bandwidth-
   limited SBCs; smaller `.sparq` download. This single lever improves *reach*
   (runs at all), *thermals*, and *latency* at once. It is already the engine's
   #1 measured gap vs QLever (`BENCHMARKS.md`: "the gap WIDENS" at 10M because the
   store is flat `Vec<[u32;3]>`).
2. **Portable SIMD that maps to both NEON and WASM-SIMD128 (M4).** Write the
   filter / merge-intersection / decompression kernels once in `core::simd`; they
   lower to NEON natively and SIMD128 in-browser. The spike proves the *implicit*
   path (autovectorization) is near-zero — the value is entirely in writing the
   kernels, and writing them once covers ~every consumer target.
3. **Memory-bounded streaming (M4).** Lazy/streaming operators with a memory
   budget and disk/OPFS spill mean LIMIT/ASK stop early (great for interactive
   mobile UIs), large intermediates never blow the wasm32 4 GB / 2 GB-tab ceiling,
   and the engine degrades gracefully instead of OOM-killing the tab. This is the
   same design that fixes QLever's OPTIONAL/DISTINCT OOM on the server — consumer
   constraints just make it mandatory rather than merely better.

These three are *already* the architecture's M3/M4 priorities for the server. The
consumer angle does not add new work — it **reorders** it: for consumer devices,
RAM and bundle size are binding, so M3 (compression) and the M4 streaming/SIMD
pieces should land before server-only concerns like out-of-core billion-triple
loading.

---

## 5. The spike — measured results (this machine)

**Setup.** Built `crates/sparq-wasm` two ways with `wasm-pack build --target
nodejs --release` and benchmarked in Node v25.1.0 on this Apple Silicon (darwin)
dev machine. Toolchain: wasm-pack 0.13.1, rustc 1.89.0, `wasm32-unknown-unknown`,
wasm-opt `-Oz` (bundled by wasm-pack).
- **baseline:** default flags (no SIMD).
- **simd:** `RUSTFLAGS='--cfg getrandom_backend="wasm_js" -C target-feature=+simd128'`
  (the `getrandom` cfg is required to keep the browser RNG backend; command-line
  RUSTFLAGS overrides `.cargo/config.toml`, so it must be repeated).

Workload (harness committed at `crates/sparq-wasm/test/bench.cjs`): a synthetic
20 000-person graph = **100 000 triples** (name/age/city + two `follows` edges
each), queried with scan, 3-star, 2-chain, triangle (WCOJ path), and two numeric
FILTERs. Per query: 3 warmup + 25 timed iters, min + median; whole suite run 3×.
Both builds passed the existing smoke test and returned **identical row counts**.

**Bundle size (measured).** SIMD128 is slightly *smaller*, not larger:

| build | raw wasm | gzip -9 | brotli -q11 |
|---|--:|--:|--:|
| baseline | 805 478 B | 284 056 B | 215 716 B |
| **+simd128** | **776 610 B** | **277 787 B** | **213 045 B** |
| delta | **−3.6 %** | −2.2 % | −1.2 % |

SIMD opcodes confirmed present: count of the `0xFD` SIMD-prefix byte jumped from
157 (baseline, incidental) to **5 460** (simd) — LLVM did autovectorize scalar
loops; the bundle still shrank because vectorized loops are fewer, denser
instructions.

**Query time (measured, best-of-3-runs min, ms).** Lower is better;
`simd/baseline` < 1 means SIMD faster.

| query | baseline min | simd min | simd/baseline |
|---|--:|--:|--:|
| scan-type (20k rows) | 4.33 | 4.07 | **0.94×** |
| star-3 (20k rows) | 15.65 | 15.64 | 1.00× |
| chain-2 (80k rows) | 35.48 | 35.67 | 1.01× |
| triangle (WCOJ path) | 27.16 | 29.61 | 1.09× |
| filter-age (`?a>50`) | 3.67 | 3.90 | 1.06× |
| filter-heavy (`30<?a<70`) | 9.68 | 10.07 | 1.04× |
| load (100k triples) | 195.9 | 195.1 | 1.00× |

**Honest reading.**
- The **only** query that reliably improved is the pure scan (~6 %). Everything
  else is **within run-to-run noise** (medians across the 3 runs swung wider than
  the baseline↔simd gap — e.g. triangle median ranged 33–47 ms within the *simd*
  build alone), and several are nominally *slower* under SIMD. **There is no
  trustworthy query-time win from `+simd128` on sparq as it stands.**
- **Why:** the codebase has **no hand-written SIMD** (`grep` for
  `simd`/`core::simd`/`target_feature` across `crates/` returns nothing). The
  filter path still materializes ids and parses numbers per row (the exact
  inefficiency `BENCHMARKS.md` flags for `q06`), which is branchy, dictionary-
  touching, scalar code that does not autovectorize. `+simd128` only lets LLVM
  vectorize `memcpy`-ish and trivial loops, which are not the bottleneck.
- **The reliable, free win is bundle size** (−3.6 % raw / −1.2 % brotli) — so
  there is no downside to enabling `+simd128` in the wasm release profile today.
- **The actionable conclusion:** the value of WASM SIMD is locked behind writing
  explicit vectorized kernels (M4 numeric-filter on inline ValueIds; M3 SIMD
  block decompression). Flipping the compiler flag is necessary but nowhere near
  sufficient.

**Caveats.** Single dev machine, Node (not a browser engine — Safari/Chrome WASM
runtimes differ); 100k triples fits L2/L3 so this understates the memory-bound
regime where compression matters most; the triangle query returns 0 rows for this
graph shape but still exercises the WCOJ code path so its timing is representative
of the operator, not of result materialization.

---

## 6. Recommended concrete first step

**Enable `+simd128` in the wasm release profile now (it is free — smaller bundle,
no regression), then build and land a single hand-written `core::simd`
numeric-FILTER kernel over inline/cached numeric ValueIds, and measure it with the
committed `test/bench.cjs` harness.**

Why this first:
- It attacks sparq's **single worst measured gap** — numeric FILTER, up to **20×
  slower than QLever** at 10M triples (`BENCHMARKS.md`) — on the **highest-reach
  target** (WASM, every phone + laptop + Chromebook).
- The kernel is written in **portable `core::simd`**, so the *same* code also
  accelerates the native ARM64/NEON and x86/AVX2 targets — it amortizes across
  three of the top four recommended targets at once.
- The spike has already proven the inverse: turning on the flag *without* the
  kernel does nothing for query time. This first step is precisely the missing
  piece, and `bench.cjs` already exists to quantify it (baseline filter-age
  ~3.7 ms / filter-heavy ~9.7 ms in WASM today — those are the numbers to beat).

After that, in order: M3 column compression (unlocks the 2 GB mobile-tab tier and
the memory/thermal wins), prebuilt `.sparq` + OPFS zero-copy load (drops the
parser from the hot path and the bundle), then opt-in Web-Worker threads behind a
`crossOriginIsolated` feature gate for first-party high-end-laptop apps.
