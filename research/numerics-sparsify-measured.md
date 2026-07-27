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

Two corpora at the same row count, differing only in numeric density —
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

## What was measured (work-box run, NON-CANONICAL)

Run on the shared work box, `--release`, best-of-9, two independent runs per arm. Per the
repo's *No hard-coded performance numbers* rule, and because work-box timings are not
canonical, no magnitudes are transcribed here — re-run the two commands above and read the
harness's own structured output (the house 3-column `name<TAB>count<TAB>us` TSV, one row per
corpus × workload). Footprint cites `Graph::heap_bytes()` (the store's own self-accounting),
never process RSS.

The direction the run established, which is what the verdict rests on:

- `string_heavy`: arm B sparsifies and does shrink `Graph::heap_bytes()`, but **both** probe
  passes — random-order and sequential — are materially slower, the sequential pass by more
  than the random-order one. The regression is far outside the run-to-run spread, which was
  small enough on every row to leave no ambiguity about the sign.
- `numeric_heavy`: arm B DECLINES, and is byte-identical and timing-identical to arm A.

The footprint delta is accounted for entirely by the dense `Vec<f64>` (one f64 per dictionary
term) being replaced by an `FxHashMap` holding only the numeric minority; the harness prints
both arms' `heap_bytes` so the arithmetic is checkable from a re-run.

## Findings

1. **The adoption criterion fails.** The bead's condition was "adopt only if the numeric
   fast path holds within noise". It does not: on the very corpus sparsification is meant to
   help, the probe path costs multiples more on both access shapes, while the footprint win
   is a small fraction of graph heap — the wrong side of the trade for a query engine.
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
- An EC2 re-run is cheap now that the arm exists and is the only way to get canonical
  magnitudes, but it is not needed to overturn the direction: a multiple-× probe regression
  does not become noise on a quieter box.
- Honest limit of this measurement: it exercises the cache probe through
  `Graph::numeric_value` in isolation, not an end-to-end `op_filter-numeric` / ORDER BY /
  MIN-MAX query through `sparq-engine`. That isolation is deliberate (it varies only the
  layout), but it means the *end-to-end* fraction attributable to the cache is unmeasured
  here. It can only make the sparse arm look better by dilution, never worse — which does
  not change a negative verdict.
