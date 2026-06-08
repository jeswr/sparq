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

## 3. PGO / BOLT

- **PGO: worth one try (~3–8%), low effort.** Branchy planner code (`eval_bgp_binary`,
  `NumCmp::test`, `term`/`term_parts` match arms) is where PGO's branch-layout wins land.
  ```
  RUSTFLAGS="-Cprofile-generate=/tmp/pgo" cargo build --release -p sparq-cli
  ./target/release/sparq-cli bench …/synthetic.nt ntriples …/queries 5 json   # train (JSON touches joins+materialise+escape)
  llvm-profdata merge -o /tmp/pgo/merged.profdata /tmp/pgo/*.profraw
  RUSTFLAGS="-Cprofile-use=/tmp/pgo/merged.profdata" cargo build --release -p sparq-cli
  ```
- **BOLT: skip** — Mach-O unsupported; D-cache/bandwidth-bound anyway.

## 4. Distribution strategy for hardware-optimised builds

The §1 finding reshapes this: **on Apple silicon there are no tiers to ship** (OS target
enables every feature). Tiering is real only on x86-64 / generic aarch64-linux.

- **aarch64-apple-darwin: ship ONE default binary** (no `target-cpu`) — runs identically on
  M1/M2/M3/M4.
- **x86-64: ship microarch tiers** — `x86-64` baseline + **`x86-64-v3`** (AVX2/BMI2/FMA,
  ~95% of live servers; AVX2 genuinely unlocks the autovectorized loops) + optional v4.
- **aarch64-linux: one `neoverse-n1`/`+lse` build** (LSE atomics help contended rayon).
- **Mechanism: pre-built tiers** (3–4 CI artifacts; a launcher picks the x86 tier from
  `/proc/cpuinfo`) **+ the `multiversion` crate on JUST the intersection kernel** (a few KB)
  so a single binary picks NEON/AVX2 vs scalar at runtime. Don't multiversion
  whole-program (bloats `.text` for no gain on bandwidth-bound bulk). `target-cpu=native`
  per-machine only for self-compiling power users on Linux/x86.

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
4. PGO once (JSON training set); keep if ≥5%. No BOLT.
5. Distribution: single arm64-darwin binary; tiered `x86-64-v3`/baseline + `neoverse`
   Linux; runtime-detect only the intersection kernel.

The structural wins (column compression, tagged ValueIds — M3/M4) remain far larger than
anything here.
