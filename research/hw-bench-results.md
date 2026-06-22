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
| x86 Sapphire Rapids (Intel) | 27.9 M/s | 25.7 M/s | **+7.5%** |
| x86 EPYC 9R14 (AMD Zen 4) | 25.8 M/s | 25.1 M/s | **+3%** |
| Graviton3 / Neoverse-V1 | 31.5 M/s | 35.2 M/s | **−10%** (hurts!) |

The *same* `prfm pldl1keep`/`prefetcht0` hint that wins on Apple silicon and both x86 vendors
(Intel +7.5%, AMD +3% — consistent every run) **slows Graviton3 down** — its hardware prefetcher
already saturates the gather, so explicit hints only add instruction overhead. This is the textbook
case for per-hardware tuning. Four silicon families measured (Apple, Intel, AMD, Graviton) cover the
consumer + server landscape; the `x86_64 → ON` default is validated across *both* x86 vendors, and
the aarch64 split (Apple ON / Neoverse-Linux OFF) is the only divergence.

**Action taken** (`crates/sparq-core/src/lib.rs`, `PREFETCH_DEFAULT`): the prefetch now defaults
ON for x86_64 and Apple aarch64, OFF for aarch64-Linux (Graviton/Neoverse), overridable at runtime
with `SPARQ_PREFETCH=1` / `SPARQ_NO_PREFETCH=1` for re-tuning on new silicon.

## Cross-platform aside

Graviton3 beats the Xeon on the compute-bound query+inference paths (json 126µs vs 148µs, infer
0.28s vs 0.39s) while the Xeon edges ingest. The Xeon's random-gather throughput (~28 M/s) is far
below M1's (~69 M/s) — Apple's memory subsystem handles the cache-missing remap dramatically
better.

## Reproduce

```text
scripts/hw-bench.sh [scale] [out.csv]          # per-tier sweep on the host
sparq-cli bench-remap [n] [dict] [iters]       # isolated remap gather; set SPARQ_NO_PREFETCH=1 to A/B
```

## Graviton 16-vCPU thread sweep (c7g.4xlarge, 2026-06-10, commit d7e401a)

First *uniform-core server* datapoint for the scaling work (no E-cores, single socket — isolates
algorithmic serial fractions + bandwidth from the M1's heterogeneity and from NUMA). 8M triples,
`sparq-cli scaling`, threads 1→16, best-of-3. Speedup at 16 threads (parallel efficiency):

| subsystem | 16-thread speedup (eff) |
|---|---|
| **GROUP BY COUNT (radix build + T1.0b)** | **6.10× (0.38)** — still climbing at 16 |
| GROUP_CONCAT | 3.37× (0.21) |
| BIND new-strings (T1.0b) | 2.45× (0.15) |
| 2-pattern join (merge) | 2.50× (0.16) |
| 3-star join | 2.24× (0.14) |
| OPTIONAL | 2.01× (0.13) |
| load (parse+build) | 2.19× (0.14) |

**Reading:** the rebuilt aggregation pipeline scales 2–3× better than every other subsystem —
direct validation of the radix-group + parallel-resolve work. Everything else plateaus ~2.2–2.5×
by 8 threads on UNIFORM cores, so the remaining ceiling is memory bandwidth (a 16-vCPU slice of
the chip) + the still-serial merge/outer joins + per-operator materialization — i.e. exactly the
Tier-3 (morsel/columnar) territory, NOT NUMA or core heterogeneity. Cost of the datapoint: ~$0.30.
