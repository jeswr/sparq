# Hardware-optimization research — synthesis

Four independent research agents (each running its own **measured spikes** on this
2020 M1 MacBook Air) studied how to accelerate sparq on real hardware. Their
reports:

- [`m1-apple-silicon.md`](m1-apple-silicon.md) — M1 CPU/GPU/AMX/Neural-Engine
- [`gpu-and-cloud.md`](gpu-and-cloud.md) — Nvidia/CUDA, Dell XPS, AWS/cloud + a wgpu spike
- [`remote-access-setup.md`](remote-access-setup.md) — runbook to give the agent SSH access to the XPS
- [`consumer-and-targets.md`](consumer-and-targets.md) — phones/laptops/SBCs, device-target matrix
- [`browser-acceleration.md`](browser-acceleration.md) — WASM SIMD128 / threads / WebGPU, runtime dispatch

## The unanimous conclusion

**This workload is memory-latency / bandwidth bound, not compute bound — so the
single highest-leverage optimization is the M4 inline-numeric *columnar* layout
(tagged ValueIds), and it is the gate for every hardware win.**

The measured evidence (all from spikes, not literature):

- **M1 is bandwidth-bound on scans (~62 GB/s); one P-core ≈ the whole 8-core CPU
  ≈ the GPU.** Adding compute (cores, GPU, AMX, ANE) does not speed a streaming
  filter. *(m1 agent)*
- **The real cost is layout:** sparq's q06 numeric FILTER does a *random* gather
  into the `Vec<f64>` numeric cache — measured ~0.5 G rows/s vs ~3.8 G/s
  contiguous: an **8–15× latency penalty**. Inline numerics in a contiguous
  column (M4) remove the gather. *(m1 agent)*
- **SIMD only pays once the data is columnar:** `+simd128` / `core::simd` give
  ~1.5× on a *contiguous* filter (via LLVM autovectorization), but **nothing** on
  the current per-row dictionary-materializing path. So SIMD is gated on M4, not
  on writing intrinsics. *(browser agent)*
- **GPU (Metal/CUDA/WebGPU) loses end-to-end** below ~16–64M rows and whenever
  data must be uploaded per query (upload memcpy ≈ 10× the compute, even on M1
  unified memory). It is only viable as a **resident columnar co-processor** for
  large bulk scan/aggregate/sort — which again requires the columnar layout.
  *(gpu + browser agents)*
- **ANE / WebNN: unreachable / wrong shape** for exact-integer triplestore ops —
  honestly infeasible. **AMX / vDSP: dropped** (dense-matrix units, wrong shape).
  *(m1 agent)*

## Where each technique *does* earn its keep (after M4)

| technique | helps | measured/expected | when |
|---|---|---|---|
| **M4 inline-numeric columnar layout** | FILTER, numeric join, ORDER BY | removes the 8–15× gather penalty | **first — the gate** |
| `core::simd` / `+simd128` (→ NEON / AVX2 / WASM-SIMD) | filter, **bit-unpack decode**, **merge-intersection**, selection compaction | ~1.5× filter; NEON merge-intersect **1.6–2.8× measured**; decode 2–4× (lit) | after M4 / with M3 |
| **M3 column compression** (PForDelta / front-coded vocab) | memory + scan bandwidth + mobile 2 GB cap | the scaling lever vs QLever | with M4 |
| prefetch on hash-probe | hash join | latency hiding | M4-era |
| WASM threads (SAB + COOP/COEP) | parallel join/scan/build | opt-in only (cross-origin isolation) | later, lazy |
| WebGPU / wgpu (one kernel → Metal/Vulkan/CUDA + browser) | **resident** bulk scan/aggregate/sort on huge data | 2–3× >16–64M rows | later, lazy, data-resident |
| GPU (discrete / cloud) | same, at scale | — | only as resident co-processor |

## Hardware-target verdict

- **Best CPU target for a hosted store:** AWS **Graviton memory-optimized (r7g/r8g)
  + NVMe (im4gn/i4i)** for out-of-core — *not* GPU instances. *(gpu/cloud agent)*
- **Dell XPS 9500 (GTX 1650 Ti, 4 GB):** a GPU **dev/validation** target only — every
  dataset sparq cares about dwarfs 4 GB VRAM. Use it to validate a portable `wgpu`
  kernel, not for perf numbers.
- **Consumer / browser:** the **WASM single-thread** artifact has the highest reach
  (one build covers mobile + laptops + Chromebooks); M3 compression is mandatory
  for the 2 GB mobile-tab memory ceiling. *(consumer/browser agents)*
- **Cross-cutting multiplier:** one `core::simd` kernel serves WASM-SIMD + NEON +
  AVX2; one `wgpu` kernel serves WebGPU + Metal + Vulkan + CUDA. Write portable
  kernels once.

## The plan this produces

1. **M4 — tagged/inline-numeric ValueIds + columnar value access** (the gate).
2. **M3 — column compression (PForDelta) + front-coded vocab + parallel bulk load**
   (memory + bandwidth; also the Wikidata-ingestion lever, since bz2 decompression
   is the current bottleneck).
3. **Portable `core::simd` kernels** for filter / bit-unpack / merge-intersection
   (serve native NEON/AVX + WASM-SIMD).
4. **Then** opt-in WASM threads and a lazy `wgpu` resident bulk co-processor.

GPU/cloud/XPS work follows the portable-kernel path; the XPS SSH runbook is ready
for when GPU validation is wanted.
