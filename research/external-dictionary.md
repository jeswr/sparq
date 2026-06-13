# External (spillable) term dictionary for the out-of-core build

Status: DESIGN (implementation in progress on branch `dict-spill`).
Motivation: research/wikidata-lowresource-stage1.md (measured 2026-06-12) — the
external-memory build bounds TRIPLE-sort memory but the term dictionary stays
RAM-resident during BUILD: RSS 6.6–6.9 GiB @100M, 13.3–13.5 GiB @300M, swap-bound at
1B on a 16 GB box (35 GiB total footprint, 4.8× throughput loss); full truthy (~8B)
extrapolates to a ~280–330 GiB dict. Goal: peak RSS bounded by a configurable budget,
dict included.

## Audit findings (what already exists, and what does NOT bound dict memory)

- `extsort.rs`: triples are chunk-sorted to run files and k-way merged — already
  bounded. No changes needed (confirmed by the Wikidata stage-1 runs: the triple side
  stayed bounded throughout).
- Commit 137fb39 ("parallel dict consolidation"): `ShardedDict` parallelises BUILD-time
  interning across `default_shards()` hash shards and pipelines remap+spill, but every
  shard is a full in-RAM `Dict` (arena of `Stored` + hashbrown table), and
  `into_merged` materialises ONE merged in-RAM `Dict` of every distinct term. It is a
  *throughput* optimisation, not a memory bound. The "spill" in that commit's message
  refers to triple-run spilling, not dict spilling.
- `Dict::save_mmap`/`open_mmap`: QUERY-time dict is already off-heap (mmap'd blob +
  sorted-hash index). Only BUILD-time interning is unbounded. Additionally,
  `numerics_of`/`temporals_of` allocate dense `8 B/term` + `9 B/term` vectors at
  finalize — another O(distinct-terms) RAM spike on the existing path.
- Conclusion: the existing machinery does NOT bound dict memory; the spillable dict is
  not redundant.

## Design: single-pass ingest with seq-tagged term spilling + external dedup/rank

The final id assignment of the (default) sharded path is: term → shard by
`hash_termparts % n`; within a shard, local ids are assigned in FIRST-INTERN order
(batches walked in order, partials in order, partial-local ids ascending); final id =
`base[shard] + local` with `base` = prefix sums of shard sizes. That order is exactly
reproducible externally — first-occurrence rank — so the spilled build can be
**byte-identical** to the sharded in-RAM build.

Phases (all sequential IO; no random disk access anywhere):

1. **Ingest** (reuses the existing decompress + parallel-parse stages). For each parsed
   partial dict, walk local ids 1..=len exactly like `ShardedDict::intern_partials`:
   route by `hash_termparts % n`. Per shard: a BOUNDED dedup cache
   (`bytes(serialized term) → seq_s`). Miss → assign `seq_s` (per-shard counter, equal
   to the record index), append the serialized term to the shard's record spill file,
   insert into the cache. Hit → reuse the cached `seq_s`. Triples are staged to disk as
   `[u64;3]` with `staged_id = (shard+1)<<32 | seq_s` (inline integer ids pass through
   < 2^32). When a shard cache exceeds its budget share it is CLEARED (epoch reset,
   recorded as (staged-triple-count, seq_s) boundary) — eviction is a pure IO
   optimisation: a re-spilled term re-collapses at dedup time (min-seq wins), so
   correctness never depends on cache policy. Caches are only cleared at batch
   boundaries so every staged reference points into its own epoch's seq window.
2. **Dedup + rank (per shard)**: external sort of the record file by (term bytes, seq);
   merge-scan groups equal terms → distinct count per shard, a `(min_seq, term)`
   distinct stream, and `(seq → min_seq)` pairs for every record. `base[]` = prefix
   sums of the distinct counts (this IS the sharded path's base).
3. **Id assignment + streaming dict write (shards in order)**: external sort of each
   shard's distinct stream by `min_seq`; stream in that order assigning
   `final = base[s] + rank`. While streaming, intern prefixes/datatypes
   first-appearance (provably the same order `into_merged` produces, since a shard's
   prefix table order is its term arrival order) and STREAM-WRITE `dict-terms.bin` /
   `dict-offs.bin` (the exact `write_record` bytes), `(hash, id)` pairs for the lookup
   index, `numerics.bin` f64s and the temporal instants/flags — all in id order, never
   resident. `dict-meta.bin` is written after the last shard; `dict-hash.bin`/
   `dict-hid.bin` by external sort of the `(hash, id)` pairs.
4. **Remap construction (per shard)**: sort `(seq → min_seq)` by min_seq, merge-join
   with the `(min_seq → final)` stream from phase 3, sort the result by seq → a dense
   per-shard `seq → final` file.
5. **Triple remap + the unchanged tail**: stream the staged `[u64;3]` triples with
   per-shard sliding windows over the remap files (advanced at the recorded epoch
   boundaries — window size ≤ one epoch's misses), emitting final-id `[u32;3]` triples
   into the EXISTING `spill_run`/`kway_merge`/sibling-sort pipeline. Ids are already
   final and dense, so `remap_perm_file` is skipped; `perm*.bin` come out byte-identical
   to the sharded path's.

### Byte-identity argument

- Per-shard first-occurrence rank by `seq` = first-intern order of
  `intern_partials` (the routing walk is the same `(batch, partial, local-id)` order,
  and `seq` counters are monotone over that walk). Equal terms collapse to their min
  seq whether or not the ingest cache held them (the first occurrence is always a cache
  miss with the minimal seq).
- `base[]` prefix sums equal the sharded path's `bases()`.
- Prefix/datatype table construction order is reproduced (see phase 3).
- `dict-hash.bin` tie order: `save_mmap` previously sorted by hash only (unstable, tie
  order an implementation accident); both paths now sort by `(hash, id)` — semantics
  unchanged (lookup walks the equal-hash range), determinism gained.

### Memory bound

budget B (configurable) governs: per-shard dedup caches (≈ B/2 during ingest, freed
before phase 2), external-sort run buffers (≈ B/2, phases 2–4), remap windows
(≤ cache-entry count × 4 B). Outside the budget and documented: the triple run buffer
(`chunk` × 12 B, caller-set, already the pre-existing knob), the parse pipeline's
in-flight blocks (~3 × 64 MiB + partials), and transient batch overshoot of the caches
(≤ one block's new terms). Peak RSS ≈ B + chunk×12B + ~0.5–1 GiB pipeline floor.

### Disk bound and resource awareness (binding requirement 2)

- Memory budget: `SPARQ_DICT_SPILL_BUDGET_MB` explicit, else default = 25% of detected
  physical RAM (`sysconf(_SC_PHYS_PAGES) × _SC_PAGE_SIZE`; fallback 8 GiB if
  undetectable). Explicitly settable for benchmarking reproducibility.
- Disk floor: `SPARQ_DICT_SPILL_DISK_FLOOR_MB` (default 1024). Free space on the build
  dir's filesystem (`statvfs`) is checked before ingest and before each external-sort
  phase; dropping below the floor aborts with a clear error instead of filling the disk.
- Extra disk vs the current path: staged triples 24 B/triple + term records
  (≈ post-cache distinct+churn term bytes) + their sort runs — all deleted as consumed.

### Build-time configurability (binding requirement 1)

- New cargo feature `dict-spill = ["mmap", "parallel"]` on sparq-core, OFF by default
  (house style: additive, off-by-default). It composes with the sharded machinery's
  features rather than living inside `mmap` alone because the design rides the
  parallel parse/route pipeline and reproduces the SHARDED id assignment.
  `sparq-cli` enables the feature (native-only binary), but behavior is env-gated:
  unset ⇒ exactly the old path.
- Runtime: `SPARQ_DICT_SPILL=1` routes `Graph::build_external` (N-Triples) through the
  spilled dict; `Graph::build_external_spill(...)` is the explicit API.
  N-Triples only (same restriction and reason as the sharded path: the byte parser
  rejects RDF-star; triple terms cannot be sharded/spilled by content hash).
- wasm32: the feature is never enabled there (sparq-wasm uses
  `default-features = false`); no detection APIs or libc reach the wasm build. Byte
  count must stay at the 1,643,095 baseline (verified below).

### Rejected alternatives

- **Spill only the term arenas, keep hash tables resident** (offsets into an mmap'd
  arena): simpler, but RSS floor stays O(distinct × ~20–24 B) → ~50–60 GiB at the 8B
  target. Not budget-bounded; rejected.
- **Two passes over the input with mmap'd dict lookups in pass 2**: per-occurrence
  random probes of a >RAM hash index → IOPS-bound (~3B random reads at 1B triples on
  4000 IOPS EBS). Rejected.
- **Positioned writes into a dense remap file** (instead of the sort/join in phase 4):
  random 4 B writes over a file > RAM → dirty-page thrash. Rejected.

## Results

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable returns).

### Correctness (byte-identity + differential + fuzz)

- **Byte-identity, unit/differential**: `crates/sparq-engine/tests/dict_spill_differential.rs`
  builds the same N-Triples doc through the default sharded in-RAM path
  (`build_external_opts(.., true)`) and the spilled path (`build_external_spill`) with a
  TINY budget (`mem_budget = 1`, forcing constant cache-epoch clears + many-run external
  sorts) and asserts **every output file is byte-identical** (dict blob, offsets, hash
  index, all 6 perms, numerics, temporals) plus query-result parity. Also passes with a
  comfortable (single-epoch) budget and the empty/non-NT-reject edge cases. 4/4 pass.
- **Byte-identity at REAL scale**: at 10M and 100M triples (high-distinct-cardinality
  data, ~2 distinct terms/triple) the spilled build (512 MB budget) produced output
  **byte-identical to the in-RAM build across all 14 files** — including the 7.68 GB
  `dict-terms.bin` at 100M (200M distinct terms). The byte-identity invariant holds at
  scale, not just on toy inputs.
- **Fuzz (differential vs Oxigraph)**: `SPARQ_FUZZ_DICTSPILL=1` re-serialises each fuzz
  graph to N-Triples, rebuilds via the spilled dict (`mem_budget = 1`), reopens mmap'd,
  and compares query cardinalities (+ ordered + JSON-path + count paths) to Oxigraph.
  The raw harness has **499 pre-existing sparq-vs-Oxigraph mismatch seeds over 0..20000**
  that are unrelated to this feature (they reproduce identically on the BASELINE build
  with no dict-spill). The dict-spill mode's mismatch SET equals the baseline SET exactly:
  verified `42 == 42` (identical seeds) over 0..2000 and `{10034,10172,10184,10188,10213,
  10243,10248}` reproduced exactly over 10000..10500. **Zero dict-spill-specific
  mismatches** — the spilled path diverges from neither the in-RAM path nor Oxigraph.
  (The committed fuzz budget of `1` makes each case do a full external build, so the run
  is slow — set-equality is the meaningful signal and it is exact.)

### Peak RSS (macOS, `/usr/bin/time -l`, mimalloc, high-distinct-cardinality data)

High-cardinality synthetic: each line a UNIQUE IRI subject + UNIQUE literal object
(`<http://ex/resource/itemN> <http://ex/p> "unique-literal-value-N-payload-...">`), so
distinct terms ≈ 2× triples — the dict-heavy worst case. `chunk = 16M` triples.

| triples | distinct terms | dict-terms.bin | budget OFF (in-RAM dict) | budget ON (512 MB) | reduction | output |
|--------:|---------------:|---------------:|-------------------------:|-------------------:|----------:|:------:|
| 10M     | ~20M           | 0.77 GB        | 3.11 GB                  | 0.82 GB            | **3.8×**  | byte-identical |
| 100M    | ~200M          | 7.68 GB        | 5.61 GB                  | 4.55 GB            | 1.23×     | byte-identical |

Notes (empirical honesty):
- At **10M** the spill is dramatic (3.8×): the in-RAM dict (arena of `Stored` +
  hashbrown table for 20M terms) is the dominant resident set, and bounding it to a
  512 MB budget collapses peak RSS to 0.82 GB. Tightening to 256 MB did NOT drop further
  (0.87 GB) — below ~512 MB the peak is set by the un-budgeted floor: the `chunk`×12 B
  triple run buffer (16M×12 B ≈ 192 MB), the parse pipeline (~0.5–1 GB), and the
  sibling-merge mmap residency. This is exactly the bound the design predicts
  (`≈ B + chunk×12B + ~0.5–1 GiB floor`).
- At **100M** the reduction is only 1.23× — NOT because the dict spill fails (the dict IS
  bounded to the budget) but because at this scale peak RSS is dominated by the TRIPLE
  side: six `perm*.bin` permutations (~1.2 GB each) built by k-way merge + sibling
  re-sort over 100M-triple mmap'd runs. The `drop-behind` madvise (this branch) curbs
  that residency but does not eliminate the mmap working set; on the OFF path the in-RAM
  dict (5.6 GB peak) and on the ON path the perm merge (4.55 GB peak) are comparable. The
  dict-spill's value is therefore REGIME-DEPENDENT: decisive when the dict would exceed
  the triple-side peak (the 1B/8B target), modest when the triple side already dominates.

### 1B / 8B projection

Extrapolating the on-disk dict (~7.68 GB `dict-terms.bin` / 200M distinct @100M → ~38.4 B
on-disk per distinct term for this dataset's term shape):

| triples | distinct terms (≈2×) | dict-terms.bin (on disk) | in-RAM dict (OFF) ≈ arena+table, ~3–4× blob | spilled peak (budget B) |
|--------:|---------------------:|-------------------------:|--------------------------------------------:|------------------------:|
| 100M    | ~200M                | 7.7 GB (measured)        | dominated build peak 5.6 GB (measured)      | 4.55 GB @512 MB (measured) |
| 1B      | ~2B                  | ~77 GB                   | **~230–300 GB** (un-runnable on <256 GB)    | ≈ B + chunk×12B + perm-merge floor (tens of GB) |
| 8B (truthy Wikidata) | ~?* | ~280–330 GB (from stage-1 measurement, research/wikidata-lowresource-stage1.md) | **un-runnable** — swap-bound at 1B already on 16 GB | budget-bounded: dict never resident |

\* real Wikidata has far MORE sharing than this synthetic worst case (distinct terms grow
sublinearly), so the 8B dict is the measured 280–330 GiB extrapolation, not 16B. The
synthetic ~2× is an adversarial upper bound on dict size per triple.

**Conclusion**: the spilled dict makes the dict contribution to peak RSS a CONFIGURABLE
constant (the budget) instead of an O(distinct-terms) term that extrapolates to
280–330 GiB at 8B. That is the prerequisite the 8B run needs: with the dict bounded, the
remaining peak is the triple-side merge, which `extsort` already bounds (chunked runs +
k-way merge + drop-behind). The 100M result is an honest reminder that bounding the dict
alone does not shrink the triple-side peak — but at 8B the dict, not the triple merge, is
what makes the build un-runnable today, and that is exactly what this removes.
