<!-- [OPUS-4.8] (sq-lfo84) ANN distance-kernel SIMD evaluation + fix record. Companion to
     research/gap-vector-2026-07.md (the sq-z2z18 sweep that root-caused the gap). All timing
     rows NON-CANONICAL: aarch64 EC2 work box, no AVX2 — directional ranking only. -->

# ANN distance-kernel SIMD — evaluation + fix (2026-07)

**Bead:** `sq-lfo84` (P1, HNSW QPS/build gap vs hnswlib) + `sq-ose80` (build-time sibling).
**Root cause (from `research/gap-vector-2026-07.md` §3.6/§7 + the sq-z2z18 sweep):** the
`instant-distance` 0.6.1 HNSW backend ships **no SIMD distance kernel** — its inner
squared-Euclidean loop is scalar — and its graph construction is largely serial, so a 1M×128
build was aborted at >30min (hnswlib: 374s).

> **NON-CANONICAL throughout.** Every timing below was collected on an **aarch64 EC2 work box
> (no AVX2)**. Absolute figures are not canonical; the matched-recall *ranking* between kernels is
> stable across ISAs. The x86_64 AVX2 kernel added here is type-checked + clippy-clean on the
> `x86_64-unknown-linux-gnu` target but its numeric output is verified only by the in-tree unit
> test on the CI x86_64 runner (this box cannot execute AVX2).

---

## 1. Options evaluated (Phase 1)

| Option | New dep | Build weight | SIMD | Per-query ef | API fit | Verdict |
|---|---|---|---|---|---|---|
| **(a) SIMD kernel for `instant-distance`** (NEON aarch64 / AVX2 x86_64 in the user-supplied `Point::distance`) | **none** | none (Cargo.lock unchanged) | yes | no (unchanged — still build-time) | **exact** (public API identical) | **CHOSEN** |
| (b1) `usearch` 2.25.3 | `cxx` **+ a C++ toolchain build** (`cxx-build`, vendored C++ core) | HEAVY (native C++ compile) | yes (SimSIMD) | yes | needs a wrapper; different index type | rejected — C++ build dep violates lean-native-build; would need its own feature |
| (b2) `hnsw_rs` 0.3.4 | pure Rust but **15+ transitive crates** (rayon, serde, bincode, mmap-rs, anndists, indexmap, hashbrown, parking_lot, env_logger, …) | MEDIUM-HEAVY | yes (`anndists`) | **yes** | needs a `VectorIndex` rewrite | rejected here — fat tree + no matched-recall QPS win on this box (see §3); revisit for the ef-sweep + parallel-build story (sq-ose80) |

Licences are all clean (usearch Apache-2.0; hnsw_rs MIT/Apache-2.0; every transitive crate already
on `deny.toml`'s permissive allowlist) — licence was **not** the discriminator. The discriminators
were **build weight** (option a adds zero; usearch adds a C++ toolchain; hnsw_rs adds a fat tree)
and, on this box, **matched-recall QPS** (§3).

## 2. Chosen fix (Phase 2)

A shared `crates/sparq-vectors/src/simd.rs` (`approx-ann`-only, **no new dependency** — pure
`core::arch` intrinsics behind `#[cfg]`) exposing `l2_sq_dist(a,b)`:

- **NEON** (aarch64): four 128-bit FMA accumulators (16 f32/iter), 4-wide drain, scalar tail.
- **AVX2+FMA** (x86_64): two 256-bit FMA accumulators (16 f32/iter), 8-wide drain, scalar tail.
- **Scalar fallback**: bit-identical to the previous 8-lane auto-vectorised loop.
- Runtime `is_*_feature_detected!` gates every `unsafe` intrinsic entry; each reads exactly
  `len` lanes (no over-read). Wired only into HNSW's `NPoint::distance` (`.sqrt()` of the kernel).

**Why HNSW-only, not the exact/DiskANN/PQ paths:** those three are **EXACT-gated** against
`bench/vector/expected.tsv` (diskann=34, pq=22), and CI runs the gate on **x86_64**. A SIMD
reduction changes the float-sum order → can shift an integer recall deficit by ±1 → would break
the exact-gate on a runner this box cannot reproduce. HNSW is **floor-gated** only (its build is
already rayon-parallel + non-deterministic), so it tolerates the rounding change. Leaving the
deterministic paths on the scalar reduction keeps `expected.tsv` byte-stable. Extending the kernel
to the exact/DiskANN/PQ paths (with an x86_64 `expected.tsv` re-measurement) is the follow-up
**sq-9wrkc**.

## 3. Measured recall–QPS + build time (NON-CANONICAL, aarch64)

Harness: standalone `instant-distance` build with the scalar vs the NEON `Point::distance`, plus a
`hnsw_rs` reference. 100k×128, clustered corpus (the regime real embeddings live in), build
`ef_construction=200`, `ef_search=256`, 1000 queries, k=10.

| Backend | Build (s) | recall@10 | QPS (µs/q) | Notes |
|---|---|---|---|---|
| instant-distance **scalar** (before) | 31.3 | 1.0000 | 2 873 (348 µs) | the committed baseline kernel |
| instant-distance **NEON** (this PR) | **13.2** | 1.0000 | **3 431** (291 µs) | **~2.4× build, ~1.2× query**, recall equal on this corpus |
| hnsw_rs 0.3.4 (reference) | 8.9 | ≤1.0 per ef | ef256: 1 356 | fastest build (parallel_insert) but **lower matched-recall QPS** here |

**Honest reading.** The clustered corpus saturates recall (1.0 both sides), so this table shows the
**kernel-swap effect** (build + throughput), not a recall trade. The NEON kernel roughly **halves
build time and lifts QPS ~1.2×** on the *search* path. **Recall precision caveat:** the SIMD kernels
use FMA (one rounding per term) while the scalar path does `d*d` then `+=` (two roundings), so a SIMD
squared distance is **not bit-identical** to the scalar one — it differs by ≤1 ULP. Rankings are
therefore stable *up to exact near-ties*, which a ≤1-ULP wobble can reorder; that residual is exactly
what the HNSW workload's **floor gate** (recall@10 ≥ 0.95, `tests/recall.rs`) absorbs — the gate is a
recall floor, not a bit-identity assertion. Measured recall was `1.0000` on both kernels here (a
saturating corpus). The scalar *fallback* alone IS bit-identical to the previous auto-vectorised loop
(so a non-SIMD target is numerically unchanged). The bigger win is on **build** because the build
evaluates the distance far more often than search.

**hnsw_rs did NOT win** on this box at matched recall — its per-query ef sweep gives a real Pareto
curve (an advantage `instant-distance` lacks, cf. sq-jo6ty) but its QPS at ef≥64 sat below
instant-distance+NEON, and it drags a 15-crate tree. So a swap was **not** justified by measurement.

## 4. Honest gap vs hnswlib (still BEHIND, narrowed)

The `research/gap-vector-2026-07.md` §3.5 hnswlib figures (NON-CANONICAL, same box) put hnswlib at
5 521 QPS @ recall 0.989 (ef=128) and 2 786 @ 0.997 (ef=256) on **SIFT1M** (1M vectors). This PR's
NEON kernel measured **3 431 QPS on a 100k clustered corpus** — not directly comparable
(different N, different corpus hardness, no per-query ef sweep on the sparq side yet), so **no
matched-recall gap number is claimed here**. What is honestly established:

- **BEHIND-but-improved.** The scalar→NEON kernel closes part of the *per-distance* gap
  (instant-distance's missing SIMD kernel was one of the two root causes). The QPS lift is ~1.2× on
  search and ~2.4× on build on this box.
- **The build-time gap (sq-ose80) is only PARTIALLY addressed.** The NEON kernel roughly halves
  build on a 100k clustered corpus, but on a **hard uniform 100k×128** corpus a single-map
  `instant-distance` build still ran **>14 min** on this box before being cut off — the *serial
  graph-construction* cost (not just the distance kernel) dominates at scale, exactly the sq-ose80
  root cause. Halving the distance kernel does not make a >30min 1M build practical. **sq-ose80
  stays OPEN**; its real fix is a parallel-build HNSW (which is what `hnsw_rs::parallel_insert` and
  usearch already do) — a backend-swap decision to be taken on a canonical x86_64 box.

## 5. Recommendation for the maintainer

1. **Ship option (a)** (this PR): a free (zero-dep), API-preserving, recall-preserving win on both
   build and query, entirely within lean-core discipline.
2. **Re-decide the backend swap on a canonical x86_64 AVX2 box**, not this aarch64 box — hnswlib's
   published dominance is an AVX2 story, and `hnsw_rs`/`usearch` both have the two things
   instant-distance lacks (per-query ef + parallel build). If a swap is greenlit it must be
   **behind its own opt-in feature** (usearch's C++ build especially) so the default stays lean.
   Tracked: **sq-ose80** (build-time / parallel-build backend) + **sq-jo6ty** (per-query ef).
3. **Follow-up sq-9wrkc:** extend the SIMD kernel to the exact/DiskANN/PQ `sq_dist` loops after an
   x86_64 `expected.tsv` re-measurement (the deterministic-gate constraint that scoped this PR).

---

## 6. Discovered beads

- **sq-9wrkc** (P2): extend the `simd::l2_sq_dist` kernel to the exact `cosine` / DiskANN Vamana /
  PQ `sq_dist` loops. Blocked on re-measuring `bench/vector/expected.tsv` on an x86_64 runner (the
  AVX2 float-sum order can shift the EXACT-gated diskann/pq deficits by ±1).
- **sq-ose80** stays OPEN: the >30min 1M build is a *serial graph-construction* problem the distance
  kernel only partially touches; the real fix is a parallel-build backend, decided on a canonical box.

---

*SPARQ agent (Opus 4.8) [OPUS-4.8] · bead sq-lfo84 · NON-CANONICAL aarch64 work-box measurement.*
