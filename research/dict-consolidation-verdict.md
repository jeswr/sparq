# Parallel dict consolidation — measured follow-up to the rung-5 serial bucket

Status: IN PROGRESS (numbers being filled from local A/B runs; branch `dict-consolidation`).

## The measured problem (inputs)

- Rung-5 @1 B real truthy triples (r7i.2xlarge, 8 vCPU): dict bucket = `merge_remap(serial)`
  **137.9 s** + `triple-remap(serial)` **62.2 s** ≈ **200 s/1 B SERIAL** even with
  `SPARQ_SHARDED_DICT=1`; CPU 249%/800% ⇒ Amdahl caps full-truthy ingest at ≥31 min on ANY
  core count; <24-min roadmap target needs the bucket ≤153 s/1 B
  (`research/wikidata-ingestion-benchmark.md` "Rung 5 MEASURED").
- Same serialization as the LOAD plateau: 1.81× at 4 identical x86 cores (M1: 1.82×) —
  `research/hardware-validation-blocked.md`.

## Where the serial time actually was (recon, code-level)

With `SPARQ_SHARDED_DICT=1`, stage 3 of `build_external_ntriples_sharded` ran on ONE
thread and did, per batch:

1. a SERIAL hash-routing scan over every partial-dict term (`intern_partials`),
2. the parallel per-shard interning (the only parallel part),
3. a SERIAL scatter of resolved temp ids into the remap table,
4. a SERIAL triple-remap loop + `spill_run` (which itself did a SERIAL
   `sort_unstable` of up to `chunk` (= 64 M at rung 5) triples per spill),

and `ShardedDict::into_merged` (the consolidation proper) moved every distinct term's
arena slot serially. The in-memory loaders (`load_str`/`load_reader_parallel` — what
`sparq-cli scaling` measures) didn't use the sharded dict at all: fully serial
`merge_remap` per block.

## What was changed (branch `dict-consolidation`)

1. **`ShardedDict::intern_partials` is now fully parallel**: routing runs per-partial in
   parallel (per-partial per-shard sub-buckets, no shared state); each shard walks the
   partials in order (per-shard id assignment — and so every downstream byte — unchanged)
   and scatters temp ids straight into the remap table via disjoint writes.
2. **Stage-3 split into a 2-stage pipeline** (intern ∥ remap+spill) inside the external
   sharded build: the critical path is now max(intern, remap), not the sum. The remap
   gather is rayon-parallel; run-file boundaries are preserved exactly (byte-identical
   output vs. the old sharded path).
3. **`spill_run` uses `par_sort_unstable`** (byte-identical run files): also parallelises
   each sibling-perm external sort (the 409.6 s/1 B IO+sort phase gets the sort half
   parallel).
4. **`into_merged` arena move parallel** (per-shard disjoint target slices, offsets =
   the existing `base` prefix sums).
5. **In-memory N-Triples loads consolidate through the same sharded dict** at ≥2 rayon
   threads, with a parallel-hash `Dict::build_table` rebuild so query-time
   `lookup`/`intern` still work. 1 thread keeps the proven serial merge (no sharding
   overhead, and stays the byte-reference).
6. **Sharded dict is now the DEFAULT** for parallel N-Triples external builds (roadmap
   task #22): `SPARQ_SHARDED_DICT=0|off` opts out, `=1` still forces. The on-disk FORMAT
   is unchanged (same writers/layouts; only term-id assignment differs) ⇒ NO
   format-version bump; stores from either path open interchangeably, old files load.

## Guards

- Differential test `crates/sparq-engine/tests/dict_consolidation_differential.rs`:
  serial streaming, sharded in-memory, 1-thread in-memory, external-sharded and
  external-serial builds of the same document must agree on term-level triples, dict
  size, and the full qlever-synthetic bench query suite. PASSING.
- `build_external_matches_in_memory` (byte-identity, perm files): PASSING — sharded
  id assignment is chunking-independent (per-shard order = stream first-occurrence
  order), so the in-memory sharded save and the external sharded build still agree
  byte-for-byte when built in the same rayon pool.

## Measured (local M1, 4P+4E, 16 GB — fill-in)

CAVEAT: the box was time-shared with another agent's builds during early runs; all
A/B numbers below are min-of-N with load checked between runs.

### External build phase split, 40 M synthetic triples (`SPARQ_BUILD_TIMING=1`)

| path | merge bucket | remap bucket | wall |
|---|--:|--:|--:|
| BEFORE serial-dict (main @c7af391) | 17.17 s | 4.02 s | 60.4 s |
| BEFORE sharded (env=1, main) | 11.63 s | 5.33 s | 42.9 s |
| AFTER sharded (default, this branch) | TBD | TBD | TBD |

### Load thread-scaling (16 M synthetic, `sparq-cli scaling`, in-memory load)

| threads | BEFORE best ms | AFTER best ms | BEFORE speedup | AFTER speedup |
|--:|--:|--:|--:|--:|
| 1 | TBD | TBD | 1.00 | 1.00 |
| 2 | TBD | TBD | TBD | TBD |
| 4 | TBD | TBD | TBD | TBD |
| 8 | TBD | TBD | TBD | TBD |

### Projected 1 B impact (EXTRAPOLATION, labeled)

TBD after the 40 M A/B: scale the per-40 M serial-bucket delta by 25× and re-Amdahl
against the rung-5 phase split.

## Verdict

TBD.
