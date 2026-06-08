# Hardware-optimization findings — sparq on Apple M1 (measured)

Adds to `m1-apple-silicon.md` the three things it lacked: **measured compiler-flag impact
on the real benchmarks, PGO/BOLT, and a distribution strategy for hardware-optimised
builds.** Measured on the live 672 MB / 10M-triple `synthetic.nt`, `rustc 1.89.0`, current
`[profile.release]` (`opt-level=3, lto=fat, codegen-units=1, panic=abort`), via `RUSTFLAGS`
env + `/tmp` binaries (no source/config edits).

## 1. Compiler flags — headline NEGATIVE result

**`-C target-cpu=native` and `-C target-cpu=apple-m1` give NO measurable speedup on
aarch64-apple-darwin.** `rustc --print cfg` for the default target, `native`, and
`apple-m1` all enable the **identical 28 target-features** (neon, aes, sha3, dotprod,
fp16, lse, rcpc2, …) — the Apple-Darwin target already ships the full Apple-silicon
baseline, so `native` unlocks **zero new instructions**; it only changes `-mtune`
scheduling. Interleaved best-of-5 (to cancel thermal drift on the fanless Air):

| path | baseline | native | delta |
|---|--:|--:|--:|
| load/parse 10M | 3.029 s | 3.045 s | −0.5% (noise) |
| q03 star COUNT | 14.2 ms | 14.2 ms | 0% |
| q04 JSON (700 MB) | 1062 ms | 1055 ms | noise |

(An early *sequential* run showed a fake ~6–9% load gain — a warm-cache artifact that
vanished under interleaving.) The engine is **bandwidth/allocator-bound, not
instruction-scheduling-bound** (scans run ~55–62 GB/s on one P-core ≈ the whole CPU), so
codegen changes are inconsequential here.

- **Do NOT use `target-cpu=native`/`apple-m1` on macOS aarch64** — no benefit, and it
  costs portability. On **x86-64 / aarch64-linux it's a real ISA unlock** — keep there.
- The **current release profile is optimal** — keep `lto=fat` + `codegen-units=1` (the big
  lever: cross-crate inlining of the `#[inline]` hot fns), `panic=abort`, `opt-level=3`.
- Minor: add `panic="abort"` to `[profile.bench]` (it has lto/cgu=1 but not panic).

## 2. NEON SIMD — reconciled with the actual hot loops

| hot loop | verdict |
|---|---|
| **merge-join / LFTJ intersection** (`exec.rs` `merge_join`, leapfrog `seek`) | **HIGHEST ROI — the one true win. ~1.6–2.8× (repo spike).** Hand-write a block-vs-block sorted-set intersection (Lemire/Schlegel `vqtbl1q` shuffle-LUT) over the u32 columns; LLVM cannot autovectorize a branchy two-cursor merge. Pure-Rust, wasm128-portable. |
| **newline scan in parser** (`nt.rs`) | Low ROI — use `memchr` (already transitive, NEON path), don't hand-roll. Load is dict/sort-bound, not scan-bound. <5% end-to-end. |
| **LEB128 varint decode** (`compress.rs`) | Native-irrelevant — the native store keeps raw `[[u32;3]]` and never decodes. Hot only in browser/out-of-core; there the `bitpacking` crate (NEON+wasm128) needs a format change (bit-packed-FOR vs LEB128). |
| **binary search** (`store.rs` `lower_bound`/`upper_bound`) | Skip — latency-bound (cache-miss/probe), SIMD can't hide DRAM latency. **Prefetch** is the lever (§5). |
| **JSON escaping** (`json.rs`) | Skip — already bulk-`push_str` per safe run; allocator/bandwidth-bound (700 MB String growth), not compute. |

**Net: implement exactly ONE NEON kernel — sorted-set intersection for merge-join/LFTJ.**

**Caveat found when scoping it (refines the 1.6–2.8× estimate).** That speedup assumes two
**contiguous `&[u32]`** key arrays. But sparq's actual hot loops do not have that layout:
- `merge_join` (`exec.rs`) operates on **row-major `Bindings`** (`Vec<SmallVec<[Id;4]>>`); the
  join key is `row[col]` — *strided by the row width*, not contiguous.
- `GroupStream`/the store scans operate on **interleaved `[[u32;3]]`**; the join column is
  every 3rd `u32` — also strided.

So plugging in the SIMD intersection requires first **extracting the key column into a
contiguous `u32` buffer** (a deinterleave/gather), then intersecting, then mapping indices
back. On the **bandwidth-bound** M1 that extra gather pass plausibly eats the SIMD gain.
Conclusion: the NEON intersection is **not a bounded drop-in** — its real payoff is coupled
to a **columnar Bindings / columnar scan** representation (the M3 structural bet), where the
keys are already contiguous. Standalone, it's an uncertain win; the right sequencing is
columnar-execution first, *then* the SIMD kernel falls out naturally. Measure-first verdict:
do not hand-write the kernel against the current row-major layout.

## 3. PGO / BOLT — MEASURED: marginal, below adoption threshold

**PGO tried end-to-end on this M1 (`llvm-tools-preview`, instrument → train on the 10M
synthetic with `bench … json` → merge → rebuild → interleaved best-of-5 A/B). Result: net
−2.0% query time, noisy and mixed — NOT worth defaulting.**

| query | base µs | pgo µs | Δ |
|---|--:|--:|--:|
| q04_follows_name (1.36 s — dominant join) | 1364771 | 1384785 | **+1.5%** |
| q03_star3 | 418993 | 340708 | −18.7% |
| q10_optional_age | 308759 | 327260 | **+6.0%** |
| q02_type_person | 83568 | 81379 | −2.6% |
| q06_filter_age | 20386 | 19271 | −5.5% |
| q09_count_edges (100 µs — noise) | 597 | 116 | −80.5% |
| **TOTAL query µs** | 2197074 | 2153519 | **−2.0%** |

The **dominant** query (q04, 62% of the total) is flat-to-worse, and two heavy queries
(q04, q10) regress — so the −2.0% net is within run-to-run noise and below the ≥5% keep
bar. This **confirms §1**: the engine is bandwidth/latency-bound, so PGO's branch-layout
wins have little surface area on M1. **Verdict: do NOT make PGO the default** (it needs
training data + two builds + `llvm-tools`, for a sub-threshold non-robust gain). It is kept
as an **opt-in** `PGO=1 scripts/build-dist.sh <tier>` for users to try on their own x86
hardware/workload, where branchy planner code (`eval_bgp_binary`, `NumCmp::test`,
`term`/`term_parts` match arms) has more to gain.
- **BOLT: skip** — Mach-O unsupported; D-cache/bandwidth-bound anyway.

## 4. Distribution strategy for hardware-optimised builds

The §1 finding reshapes this: **on Apple silicon there are no tiers to ship** (OS target
enables every feature). Tiering is real only on x86-64 / generic aarch64-linux.

- **aarch64-apple-darwin: ship ONE default binary** (no `target-cpu`) — runs identically on
  M1/M2/M3/M4.
- **x86-64: ship microarch tiers** — `x86-64` baseline + **`x86-64-v3`** (AVX2/BMI2/FMA,
  ~95% of live servers; AVX2 genuinely unlocks the autovectorized loops) + optional v4.
- **aarch64-linux: one `neoverse-n1`/`+lse` build** (LSE atomics help contended rayon).
- **Mechanism — now IMPLEMENTED** (this is the deliverable for "distribute builds optimised
  to particular hardware"):
  - **`.github/workflows/dist.yml`** — builds all 5 tiers on **native** runners (no
    cross-toolchain): `arm64-darwin` (macos-14), `x64-baseline/v3/v4` (ubuntu),
    `arm64-linux` neoverse+lse (ubuntu-arm). Triggers on `v*` tags + manual dispatch only.
  - **`scripts/build-dist.sh`** — host-aware local builder (`PGO=1` for the opt-in
    profile-guided variant); builds host-native tiers, prints recipes for the rest.
  - **`scripts/sparq-launch.sh`** — the runtime dispatcher: reads `/proc/cpuinfo` and
    exec's the richest tier the CPU advertises (v4→v3→baseline), with fallback. Ship it as
    the "fat package" so one download self-selects the optimal binary.
  - **Future** `multiversion` crate on JUST the intersection kernel (a few KB) once that
    NEON/AVX2 kernel exists (coupled to the columnar layout, §2) — so a single binary picks
    SIMD-vs-scalar at runtime. Don't multiversion whole-program (bloats `.text` for no gain
    on bandwidth-bound bulk). `target-cpu=native` per-machine only for self-compiling power
    users on Linux/x86.

## 5. Cache / memory (128-byte lines)

- **Keep the 12-byte `[u32;3]` triple — do NOT pad to 16 B** (would inflate the store 33%
  and move 33% more bytes on a bandwidth-bound bus — a net loss on M1). ~10.6 triples/line.
- **Software prefetch is the high-value memory lever** for the hash-join probe and the
  `lower_bound`/`upper_bound` binary search: prefetch the next probe 8–16 iterations ahead
  (`core::arch::aarch64::_prefetch`/`prfm`). M1's deep OoO overlaps many in-flight misses
  → literature 1.2–1.6× on latency-bound joins/probes. Attacks the *actual* bottleneck.
- The biggest measured gap (q06 filter 20× behind QLever) is the `numeric_value(id)`
  random `Vec<f64>[id]` gather — latency-bound; the fix is the tagged-ValueId contiguous
  numeric column (M4), not a flag/SIMD tweak.

## Priority summary

1. Keep the release profile; add `panic="abort"` to `[profile.bench]`. **Drop
   `target-cpu=native` from any macOS distribution plan.**
2. ONE NEON kernel: sorted-set intersection for merge-join/LFTJ (1.6–2.8×), multiversioned
   for the Linux/x86 build.
3. Prefetch the hash-probe + binary search (1.2–1.6×, low risk).
4. PGO: **tried, net −2.0% (noisy, below the ≥5% bar) — not default.** Kept as `PGO=1`
   opt-in in `build-dist.sh` for x86/own-workload users. No BOLT.
5. Distribution: **implemented** — `dist.yml` (5 tiers on native runners) + `build-dist.sh`
   + `sparq-launch.sh` (runtime `/proc/cpuinfo` tier dispatch). Single arm64-darwin binary;
   tiered `x86-64-v3`/`v4`/baseline + `neoverse` Linux. (Per-kernel multiversion later.)

The structural wins (column compression, tagged ValueIds — M3/M4) remain far larger than
anything here.
