# Cross-platform hardware benchmark results

Measured on real silicon (AWS eu-west-2, June 2026) to validate the hardware-specific
optimizations against the M1 dev machine. Harnesses: `scripts/hw-bench.sh` (per-`-Ctarget-cpu`
tier sweep, end-to-end) and `sparq-cli bench-remap` (isolated dict-remap gather, the path the
per-ISA software prefetch targets). Each platform was provisioned, measured, and terminated;
total instance uptime ≈ 1 h.

## Platforms

| label | instance | CPU | ISA |
|-------|----------|-----|-----|
| M1 | (dev) | Apple M1 | aarch64 / Apple |
| x86 | c7i.4xlarge | Intel Xeon Platinum 8488C (Sapphire Rapids), 16 vCPU | x86-64-v4 (full AVX-512 + AMX) |
| ARM | c7g.4xlarge | AWS Graviton3 (Neoverse-V1), 16 vCPU | aarch64 / Neoverse, SVE+bf16+i8mm |

## Finding 1 — compile-time `-Ctarget-cpu` tiers give NO measurable uplift

End-to-end ingest / query / serialise / infer, built at each tier (4M-triple dataset). All deltas
are within run-to-run noise (±2%) on both architectures, including on a CPU with full AVX-512:

x86 (c7i):
| tier | load M/s | json µs | join µs | infer s |
|------|---------|---------|---------|---------|
| baseline | 2.15 | 147606 | 333261 | 0.389 |
| v2 | 2.16 | 151561 | 334688 | 0.391 |
| v3 (AVX2) | 2.12 | 149520 | 334959 | 0.394 |
| v4 (AVX-512) | 2.17 | 151639 | 333527 | 0.406 |

ARM (c7g):
| tier | load M/s | json µs | join µs | infer s |
|------|---------|---------|---------|---------|
| baseline | 1.90 | 126367 | 308772 | 0.283 |
| neoverse-n1 | 1.93 | 126426 | 302914 | 0.295 |
| neoverse-v1 | 1.91 | 124362 | 309090 | 0.287 |

**Why:** the hot loops are bandwidth-bound (dict gather, scans) or already use runtime-dispatched
SIMD (hashbrown for joins/interning); the formatting/filter loops do not autovectorize into
wider-register wins. Newer ISA codegen changes nothing measurable.

**Implication:** the 10-tier `-Ctarget-cpu` build/release matrix in `.github/workflows/dist.yml`
buys ~0 performance. It can be collapsed to one baseline binary per OS/arch (correctness only —
a v4 binary SIGILLs on a non-AVX-512 host, so a single baseline build is also the *safe* choice).
Kept as-is for now pending the maintainer's call; the perf cost of removing it is zero.

## Finding 2 — the per-ISA software prefetch is genuinely hardware-specific (helps OR hurts)

Isolated dict-remap gather (`bench-remap`, 20M triples scattered over a 50M-entry table), software
prefetch ON vs OFF, best of repeated runs:

| platform | prefetch ON | prefetch OFF | effect of ON |
|----------|------------|--------------|--------------|
| M1 (Apple aarch64) | 68.7 M/s | 56.0 M/s | **+22%** |
| x86 Sapphire Rapids | 27.9 M/s | 25.7 M/s | **+7.5%** |
| Graviton3 / Neoverse-V1 | 31.5 M/s | 35.2 M/s | **−10%** (hurts!) |

The *same* `prfm pldl1keep` hint that wins big on Apple silicon and modestly on Intel **slows
Graviton3 down** — its hardware prefetcher already saturates the gather, so explicit hints only
add instruction overhead. This is the textbook case for per-hardware tuning.

**Action taken** (`crates/sparq-core/src/lib.rs`, `PREFETCH_DEFAULT`): the prefetch now defaults
ON for x86_64 and Apple aarch64, OFF for aarch64-Linux (Graviton/Neoverse), overridable at runtime
with `SPARQ_PREFETCH=1` / `SPARQ_NO_PREFETCH=1` for re-tuning on new silicon.

## Cross-platform aside

Graviton3 beats the Xeon on the compute-bound query+inference paths (json 126µs vs 148µs, infer
0.28s vs 0.39s) while the Xeon edges ingest. The Xeon's random-gather throughput (~28 M/s) is far
below M1's (~69 M/s) — Apple's memory subsystem handles the cache-missing remap dramatically
better.

## Reproduce

```
scripts/hw-bench.sh [scale] [out.csv]          # per-tier sweep on the host
sparq-cli bench-remap [n] [dict] [iters]       # isolated remap gather; set SPARQ_NO_PREFETCH=1 to A/B
```
