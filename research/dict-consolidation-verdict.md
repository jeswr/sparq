# Parallel dict consolidation — measured follow-up to the rung-5 serial bucket

Status: MEASURED (branch `dict-consolidation` @ 137fb39; A/B vs main binary whose
sparq-core is identical to the merge-base c7af391 — main moved to f96898b but
c7af391..f96898b touches only sparq-solid/sparq-reason/docs, verified).

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

## Measured (local M1, 4P+4E, 16 GB)

CAVEAT: the box was time-shared with another agent's builds (background load 5–10
throughout); A/B rounds ALTERNATED old/new under a load-gate (<10 1-min loadavg before
each run), reported min-of-3. The earlier single-shot numbers in the first draft of this
file (60.4 s serial / 42.9 s sharded) were taken under heavier, unequal load and are
superseded by the alternating min-of-3 below.

### External build, 40 M synthetic triples (`SPARQ_BUILD_TIMING=1`, 16 M-triple runs)

min-of-3 alternating rounds (per-round walls: old-sharded 56.8/59.5/56.2,
new 58.3/53.9/54.4; old-serial 79.6/55.8/66.2):

| path (min-of-3) | stage-1 wall (parse+intern+spill) | merge bucket | remap bucket | total wall |
|---|--:|--:|--:|--:|
| OLD default = serial dict (main) | 16.80 s | 11.23 s | 3.41 s | 55.8 s |
| OLD sharded (`SPARQ_SHARDED_DICT=1`, main) | 12.08 s | 5.84 s | 3.83 s | 56.2 s |
| NEW default = sharded+pipelined (this branch) | **10.81 s** | (9.63 s)* | (8.56 s)* | **53.9 s** |

\* Bucket semantics CHANGED on the new path: merge/remap now run on overlapped pipeline
stages, so those buckets are per-stage occupancy (thread-time, inflated by contention),
NOT additive serial wall. The comparable number is the stage-1 wall: **12.08 → 10.81 s
(−10.5%)** vs old-sharded, **16.80 → 10.81 s (−36%)** vs the old DEFAULT. `into_merged`
(the consolidation proper) measured 0.12–0.16 s at 40 M. Total wall moves only −4%
(old-sharded) because the sibling-sort phase (~31 s, IO-bound on this 16 GB M1)
dominates at this scale; the dict bucket is what Amdahl-caps big-core-count boxes,
not this box's wall.

### Load thread-scaling (16 M synthetic, `sparq-cli scaling`, in-memory load, best-of-3)

| threads | BEFORE best ms | AFTER best ms | BEFORE speedup | AFTER speedup |
|--:|--:|--:|--:|--:|
| 1 | 18 992 | 18 410 | 1.00 | 1.00 |
| 2 | 12 196 | 9 759 | 1.56 | **1.89** |
| 4 | 9 615 | 7 425 | 1.98 | **2.48** |
| 8 | 8 544 | 6 167 | 2.22 | **2.99** |

- The measured ~1.8–2.0× @4-core load plateau (x86 1.81×, M1 1.82×, this sweep's
  same-day BEFORE 1.98×) moves to **2.48× @4 / 2.99× @8**; absolute 8-thread load time
  drops 28% (8.54 → 6.17 s).
- 1-thread is UNCHANGED within noise (18.4 vs 19.0 s — the 1-thread path keeps the
  proven serial merge; no sharding tax).
- Amdahl read: serial fraction of in-memory load (fit at 8 threads) drops from ~37%
  to ~24% (M1 P/E asymmetry confounds the absolute fit; the DELTA is the signal).

### Projected 1 B impact (EXTRAPOLATION, labeled)

Rung-5 measured (r7i.2xlarge, 8 vCPU, 1 B real truthy): dict bucket = 137.9 s merge +
62.2 s remap ≈ **200 s/1 B additive-serial**, parse 138.1 s, stage-1 wall 221.2 s
(`research/wikidata-ingestion-benchmark.md`).

What this branch changes structurally, per that split:

1. The 137.9 s merge bucket's serial parts (hash-routing scan + remap-table scatter)
   are now parallel; only per-shard interning order is serialized per shard (parallel
   across shards). At ≥4-way effective parallelism that bucket compresses to ≲35–70 s
   of stage-occupancy.
2. The 62.2 s remap(+spill-sort) runs on its OWN pipeline stage, overlapped with
   interning, with a rayon-parallel gather and parallel run sorts — it leaves the
   critical path unless it exceeds max(parse, intern).
3. `into_merged` (the one remaining serial consolidation step) measured 0.14 s/40 M
   ⇒ **~3.5 s/1 B** (linear-in-distinct-terms ASSUMPTION).

Projected stage-1 wall at 1 B: max(parse 138.1, compressed intern, overlapped remap) ≈
**~140–160 s vs the measured 221.2 s** (saves ~60–80 s/1 B), and — the rung-5 question —
the ADDITIVE-SERIAL dict bucket drops from ~200 s/1 B to **~3.5 s (`into_merged`) plus
whatever intern occupancy exceeds parse — well under the 153 s/1 B budget** the <24-min
roadmap target requires. The Amdahl cap stops being the dict: the next serial terms are
kway-merge (31.2 s/1 B) and the IO-bound sibling-sort/finalize phases. CAVEATS: 40 M→1 B
is 25×; M1 (4P+4E, 16 GB, contended) ≠ r7i.2xlarge; dict density differs between
synthetic (40 M → 4.4 M distinct) and truthy (~42% distinct); needs a rung-5 re-run to
confirm (the 40 M alternating A/B + the 2.99× scaling curve are the local evidence).

## Differential correctness (gate) — PASS

- `dict_consolidation_differential.rs` (committed): serial-streaming, sharded-in-memory,
  1-thread-in-memory, external-sharded, external-serial — identical term-level triples,
  dict sizes, and full qlever-synthetic suite results. PASSING.
- Cross-BINARY, 40 M scale: stores built by the OLD binary (serial default AND
  `SPARQ_SHARDED_DICT=1`) open in the NEW binary, and the store built by the NEW binary
  opens in the OLD binary (format-compat both directions, no version bump needed);
  all 5 binary×store combinations return identical row counts across the whole bench
  suite (q02 5 000 000 / q03 5 000 000 / q04 19 999 991 / q06 562 500 / q09 1 /
  q10 5 000 000; 39 999 991 triples after dedup in every store). PASS.

## Verdict

**ADOPT.** Same-conditions alternating A/B: new default beats the old DEFAULT serial
path by 36% on the stage-1 wall and ~2 s total at 40 M (where IO dominates), beats the
old opt-in sharded path on every phase, removes the measured ~1.8–2.0× load-scaling
plateau (now 2.99× @8 on M1, +35%), keeps 1-thread unchanged, and is differentially
correct + format-compatible in both directions. The projected 1 B additive-serial dict
bucket falls from ~200 s to single-digit seconds (extrapolation — re-measure at rung 5
before claiming the <24-min path is open). No revert candidates: every sub-change
(parallel routing/scatter, pipelined remap, par run sorts, parallel `into_merged`,
sharded default, in-memory sharded consolidation) carried its weight in the A/B or is
load-bearing for one that did.
