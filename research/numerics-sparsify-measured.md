# Sparsifying the native numerics cache — measured verdict (sq-7d3dj.8)

> 🤖 SPARQ agent — measure-first record for roadmap item 12 of
> `research/optimization-audit-2026-07.md`. [OPUS-5]

**Verdict: DO NOT flip the default. The dense numerics cache stays.** The A/B arm and the
harness are landed so the measurement is reproducible (and re-runnable on EC2), but the
default `Graph::build` keeps the dense `Vec<f64>` exactly as it was. A recorded negative is
the outcome the bead asked for.

## The question

`Graph::build` (`crates/sparq-core/src/lib.rs`) gives every dictionary term 8 bytes of
numerics cache — a dense `Vec<f64>` holding `NaN` for the non-numeric majority — while the
*temporal* cache next to it already sparsifies at build via `into_sparse_if_worthwhile`
(≤25% of terms carrying a value ⇒ a map instead of a vec). The obvious symmetry argument
says numerics should do the same on string-heavy data.

The counter-argument, and the reason the audit flagged this **measure-first**, is that the
dense layout is deliberate: `NumData::lookup` is an O(1) array index, `#[inline(always)]`
so LTO can plant it inside numeric gather loops, and it is the FILTER / ORDER BY / MIN-MAX
fast path. Sparsifying turns every probe — including the misses, which dominate on exactly
the string-heavy data the change targets — into an `FxHashMap` lookup.

So: does the footprint win pay for the probe cost?

## The A/B arm

`sparq-core`'s opt-in `sparse-numerics` feature (OFF by default) applies the SAME
`into_sparse_if_worthwhile` heuristic to the numerics cache at build. Both arms come from
one source tree, so nothing but the cache layout differs:

```sh
cargo run -p sparq-core --release --example bench_numerics                              # arm A: dense
cargo run -p sparq-core --release --features sparse-numerics --example bench_numerics   # arm B: sparse
```

Two corpora at 200k rows (~400k dictionary terms), differing only in numeric density —
`string_heavy` at 2% numeric objects (under the heuristic's cut, so arm B sparsifies) and
`numeric_heavy` at 80% (over the cut, so arm B DECLINES and must be indistinguishable from
arm A). Objects are distinct `xsd:double` literals; small integers are inline-encoded into
their id and never reach the cache, so they would make the A/B vacuous. Per corpus:
`Graph::heap_bytes()`, a full random-order probe pass over every id (coprime-stride
permutation — the cache-hostile shape a FILTER/ORDER BY key extraction really has), and a
full sequential pass (the MIN/MAX aggregate shape).

Result equivalence is pinned two ways: `sparse_numerics_ab_arms_agree_on_every_id` asserts
`numeric_value` matches a fresh dense recomputation for every id in **both** feature states,
and the harness's `count` column (cached numeric literals found) must be identical across
arms or the timing comparison is void — it was.

## Measured (work box, NON-CANONICAL — advisory magnitudes, unambiguous direction)

Run on the shared work box, `--release`, best-of-9, two independent runs per arm. Timings
from this box are **not canonical** and are quoted only as a bracket; footprint cites
`Graph::heap_bytes()` (the store's own self-accounting), never process RSS.

| corpus | metric | arm A (dense) | arm B (sparse) | delta |
|---|---|---:|---:|---|
| `string_heavy` (2% numeric) | `Graph::heap_bytes()` | 40,827,783 B | 37,749,631 B | **−3.08 MB (−7.5%)** |
| `string_heavy` | random-order probe pass | 1,490 / 1,492 µs | 2,219 / 2,221 µs | **≈1.5× slower** |
| `string_heavy` | sequential probe pass | 645 / 645 µs | 2,161 / 2,164 µs | **≈3.4× slower** |
| `numeric_heavy` (80% numeric) | `Graph::heap_bytes()` | 39,111,783 B | 39,111,783 B | 0 (declined) |
| `numeric_heavy` | random-order probe pass | 3,075 / 3,103 µs | 3,021 / 3,023 µs | within noise |
| `numeric_heavy` | sequential probe pass | 2,172 / 2,175 µs | 2,207 / 2,208 µs | within noise |

Run-to-run spread was well under 1% on every row, so the string-heavy regression is roughly
two orders of magnitude larger than the measurement noise.

The footprint delta is exactly the dense vec: 400,001 terms × 8 B = 3,200,008 B replaced by
a map holding 4,000 entries (121,856 B of `FxHashMap` capacity at ~17 B/slot).

## Findings

1. **The adoption criterion fails.** The bead's condition was "adopt only if the numeric
   fast path holds within noise". It does not: on the very corpus sparsification is meant to
   help, the probe path costs ~1.5× (random order) to ~3.4× (sequential) more. Trading that
   for 7.5% of graph heap is the wrong side of the trade for a query engine.
2. **The sequential regression is the bigger one, and it is structural.** The dense arm's
   ordered scan is a linear walk over contiguous f64s with perfect prefetch; the sparse arm
   pays a hash + probe per id *and* loses locality entirely. MIN/MAX and ORDER BY key
   extraction are exactly that shape.
3. **Most probes are misses, and misses are the expensive case.** On string-heavy data only
   a tiny fraction of ids are numeric — but a numeric FILTER still probes every candidate
   id. A dense miss is one array read plus a NaN test; a sparse miss is a full hashmap
   lookup that fails. Sparsifying makes the *common* operation on the *targeted* dataset
   slower, which is why the intuition ("mostly NaN, so mostly waste") is misleading.
4. **The heuristic itself is sound, and the decline path works.** `numeric_heavy` is
   byte-identical and timing-identical across arms — `into_sparse_if_worthwhile` correctly
   refuses to sparsify dense data. The problem is not the cut point; it is that the sparse
   side of the cut is not worth taking for numerics.
5. **The memory-constrained target is already served.** `Graph::into_compressed` — the
   browser / RAM-bound build — *already* sparsifies both caches. The only thing this bead
   could have added is sparsification for the *uncompressed native* build, which is the
   configuration that chose latency over footprint in the first place. So the marginal value
   of defaulting it on was small even before the timings came in.
6. **No gated metric moves either way.** Numerics bytes sit outside the ratcheted metrics
   (`store_bytes_per_triple`, `dict_bytes_per_term`, `wasm_bundle_bytes`), and the default
   build is unchanged, so nothing to re-baseline.

## Disposition

- The default stays dense. `sq-7d3dj.8` closes as a **recorded negative**.
- `sparse-numerics` stays in-tree as the reproducible A/B arm (a CI leg runs its tests, and
  the decline-path witness guards the heuristic), and as a narrow opt-in for a native
  deployment that knowingly wants footprint over numeric-probe latency on string-heavy data.
  It is **not** a candidate for defaulting on; anyone reconsidering should re-run
  `bench_numerics` rather than re-derive the argument.
- An EC2 re-run is cheap now that the arm exists and would firm up the magnitudes, but it is
  not needed to overturn the direction: a 1.5–3.4× probe regression does not become noise on
  a quieter box.
- Honest limit of this measurement: it exercises the cache probe through
  `Graph::numeric_value` in isolation, not an end-to-end `op_filter-numeric` / ORDER BY /
  MIN-MAX query through `sparq-engine`. That isolation is deliberate (it varies only the
  layout), but it means the *end-to-end* fraction attributable to the cache is unmeasured
  here. It can only make the sparse arm look better by dilution, never worse — which does
  not change a negative verdict.
