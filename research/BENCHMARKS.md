# Benchmarks

> **On the numbers in this document (read first).** Per AGENTS.md, benchmark
> figures are not the source of truth — the generated data is. The tables below
> are **machine-specific, point-in-time snapshots** kept for the *analysis* (the
> QLever-gap narrative, the negative results, the methodology); they drift and are
> not maintained by hand. For current figures:
> - **Per-commit CI metrics** (deterministic: wasm bundle bytes, store B/triple,
>   dict B/term) → the perf dashboard <https://sparq.jeswr.org/dev/bench>
>   (orphan `benchmark-data` branch).
> - **The benchmark registry + exact invocations** → `bench/benchmarks.toml`
>   and `bench/CATALOG.md`. Regenerate any figure by running the named harness.
> - **The QLever comparison baselines** are single-sourced in
>   `bench/qlever-baselines.md`; the **rung5 1B Wikidata ingest** result is
>   single-sourced in `research/wikidata-ingestion-benchmark.md`.
>
> Each section below names the harness/CLI command that produces its numbers; run
> that command for live data rather than citing the snapshot.

`crates/sparq-bench` runs an identical dataset and query workload through **sparq**
and through **Oxigraph** (a mature, widely-used Rust SPARQL engine), and:

1. checks that both return the **same number of solutions** for every query — a
   differential correctness cross-check against an independent implementation;
2. reports **load time**, **per-query time** (min of *K* iterations, warm), and
   **peak process RSS** (measured in a clean subprocess, right after load).

```text
cargo run -p sparq-bench --release -- --scale 50000 --iters 4
```

## Methodology & honesty notes

- **What this is.** A like-for-like, in-process comparison on a synthetic
  WatDiv-flavoured social graph (each entity is a `ex:Person` with name/age/city
  attributes and `ex:follows` edges → star joins, chains, triangles).
- **What this is *not* (yet).** Oxigraph is *a* SOTA-class engine but not the
  fastest; QLever and RDFox are the engines to beat on the public benchmarks, and
  those comparisons (SP2Bench, WatDiv, WDBench, etc. from docs.qlever.dev) are not
  yet run (tracked in beads). We compare against Oxigraph first because it embeds
  cleanly as a Rust crate, giving a continuous, reproducible correctness + perf gate.
- **Memory caveat.** Oxigraph's *in-memory* `Store` (`Store::new()`, no RocksDB)
  is measured. Oxigraph's primary, optimised backend is on-disk RocksDB; the
  in-memory store is not its most space-efficient mode, so the memory comparison
  flatters sparq somewhat. Peak RSS also includes the parse buffer (the same
  Turtle bytes for both). sparq additionally self-reports its store footprint
  (dictionary + six permutation indexes).
- Timings are min-of-K (warm cache). Measured on the development machine
  (Apple Silicon, darwin); absolute numbers are machine-specific, ratios less so.

## Results — scale 50 000 entities (~400 k triples)

All seven queries returned **identical solution counts** to Oxigraph.

| stage / query | sparq | oxigraph | speedup |
|---|--:|--:|--:|
| load          | 577 ms | 615 ms | 1.07× |
| scan-type     | 4.27 ms | 21.7 ms | **5.08×** |
| star-3        | 20.8 ms | 79.7 ms | **3.82×** |
| chain-2 (800k rows) | 170 ms | 568 ms | **3.35×** |
| triangle (cyclic → WCOJ) | 310 ms | 679 ms | **2.19×** |
| filter-age    | 7.96 ms | 11.5 ms | 1.44× |
| count-edges   | 6.72 ms | 22.3 ms | **3.32×** |
| optional      | 20.6 ms | 48.4 ms | 2.34× |
| **peak RSS (after load)** | **71 MiB** | 178 MiB | **2.5× lighter** |

(sparq store self-estimate: 42.9 MiB for 400 k triples ≈ 6 permutations × 12 B +
dictionary.)

### Reading the numbers

- sparq is **1.4×–5.1× faster** on every query in the workload, and ~par on load
  (it builds six sorted permutations, which costs more than a single index but
  buys single-range pattern scans and merge-join-ready orderings).
- The **triangle** is the WCOJ case: Leapfrog Triejoin avoids the large
  intermediate a binary plan would build. It still wins here (2.19×); the gap
  grows on denser/skewed cyclic data (where binary intermediates blow up).
- Memory is *better* than Oxigraph's in-memory store at rest despite six
  permutations — but see the caveat above; the real memory test is vs QLever's
  compressed columns, and column compression (M3) is the lever there.

## The real target: QLever (primary benchmark for M3+M4)

Oxigraph is the *embeddable first gate* — it links as a crate, so it gives a
continuous correctness + perf check on every commit. It is **not** the engine to
beat. **QLever (ad-freiburg) is dramatically faster than Oxigraph** (often
orders of magnitude on large datasets), and is the benchmark that matters:

- QLever wiki perf comparison:
  <https://github.com/ad-freiburg/qlever/wiki/QLever-performance-evaluation-and-comparison-to-other-SPARQL-engines>
- Benchmark datasets/queries: <https://docs.qlever.dev/benchmarks>

Beating Oxigraph 2–5× says nothing about beating QLever. **M3+M4 must benchmark
against QLever directly** and study its architecture closely.

### What makes QLever fast (to match or beat)

- **Compressed permutation indexes** — block-compressed column storage (prefix /
  PForDelta-style), not flat `Vec<[u32;3]>`. This is the big memory + scan win
  → our M3.
- **Tagged ValueIds** — datatype + value packed inline (numerics, dates), so
  numeric filters/joins never touch the dictionary → our M4.
- **Compressed on-disk vocabulary** with an in-memory cache (prefix/front-coding).
- **Lazy / streaming block-based operators** — results flow in blocks; early
  LIMIT/ASK stop work. We currently fully materialise — a streaming executor is
  needed to compete on first-result latency and memory.
- **External-memory parallel index build**; highly tuned merge joins.

### Our potential genuine edge

- **WCOJ (Leapfrog Triejoin)** for cyclic / skewed BGPs — QLever uses binary
  joins, so triangle/cycle-heavy queries are a place we can win asymptotically
  (already implemented; needs to be shown on a skewed benchmark).
- **Memory-bounded streaming** as a first-class design goal.

### Plan

1. Stand up QLever (Docker image or source build) + load a standard dataset
   (start with SP2Bench or WatDiv; then a WDBench/Wikidata subset).
2. Run the standard query sets through sparq and QLever; report per-query latency
   (cold + warm), QMpH, peak RSS, and index size.
3. Profile the gaps; drive M3 (column compression) and M4 (tagged ValueIds,
   streaming operators, property paths, vectorization) from the measured deltas.
4. Honest reporting: where we win, where we lose, by how much — no unverified
   victory claims. Out-competing QLever broadly is a serious, multi-stage effort.

## Measured: sparq vs QLever — Olympics (~1.78M triples), first run

Harness in `bench/qlever-olympics/` (`qlever` CLI builds + serves QLever in
Docker; `compare.py` runs both engines; `queries/` and `queries-count/` are the
workloads). Reproduce:

```sh
cd bench/qlever-olympics
../../.qlever-venv/bin/qlever setup-config olympics && qlever get-data && qlever index && qlever start
../../.qlever-venv/bin/python compare.py 5 endtoend   # fair end-to-end (both serialise JSON)
../../.qlever-venv/bin/python compare.py 5 compute    # fair compute-only
```

Min of 5 **cold** runs each (QLever cache cleared every run; sparq has no query
cache). **All 10 queries returned result sizes identical to QLever**, and in the
compute pass sparq's solution counts equal QLever's `COUNT(*)` values exactly — a
strong correctness cross-check against an independent engine.

**Two measurements, because output serialisation dominates and must be matched:**

| pass | what both do | queries | geomean (qlever/sparq) | winner |
|---|---|--:|--:|---|
| end-to-end (full SPARQL JSON) | compute **+ serialise all rows to JSON** | 10 | 2.56× | sparq faster |
| **compute only** (COUNT-wrapped) | compute the join, return the count | 8† | **0.31×** | **QLever ~3.2× faster** |

† the 8 BGP/scan/filter/OPTIONAL queries; the two queries that are themselves
aggregates (`q01 count-all`, `q09 group-by`) have no COUNT-wrapped form and are
covered by the end-to-end pass only.

### The honest reading

- **QLever's query *engine* is ~3.2× faster than sparq** on these joins/scans
  (compute-only pass), winning every query — and most dramatically the numeric
  **FILTER (q06): 14× faster**. sparq evaluates a filter by materialising each id
  to a term and parsing the number per row; QLever's tagged ValueIds + columnar
  scan avoid both. This is the single clearest signal pointing at **M4 (tagged
  ValueIds)** and **M3 (column compression + vectorised scan)**.
- sparq only "wins" the **end-to-end** pass because **QLever's JSON export is slow
  in this setup** (Docker-on-macOS), not because sparq computes faster. When both
  serialise, sparq's hand-rolled JSON writer happens to be quicker here — but
  that says nothing about engine quality.
- **This setup *handicaps* QLever** and still it wins on compute: it runs in
  Docker-on-macOS (VM syscall/IO overhead, slow mmap of its disk index), on a
  tiny dataset that fits entirely in RAM — sparq's home turf. QLever is engineered
  for **billion-triple, out-of-core** workloads where sparq would simply OOM.
- Pure aggregates over a full scan (`q01 count-all`, `q09 group-by-gender`) go to
  QLever even end-to-end (0.16×, 0.34×) — its columnar count is hard to beat.

**Conclusion:** beating Oxigraph said nothing about QLever, exactly as expected.
QLever's engine is genuinely faster; closing that gap is what M3/M4 are for. No
"we beat QLever" claim — the opposite, on the metric that matters (compute).

### Progress — compute gap closed from 3.2× to 1.2× with two targeted changes

Driven by the measured deltas, in order:

1. **Numeric value cache (partial M4).** A dictionary-parallel `Vec<f64>` of every
   term's numeric value (NaN = non-numeric) lets numeric FILTER / comparison /
   ORDER BY skip the per-row term materialisation + string parse; numeric literal
   constants are folded the same way. → q06 filter 14.15 → 5.22 ms; compute
   geomean **0.31× → 0.41×**.
2. **Inline rows (`SmallVec<[Id;4]>`).** Replaced the per-row `Vec<Vec<Id>>` — one
   heap allocation per solution row — with rows inlined up to 4 columns, so joins
   produce no per-row heap allocation. This was the dominant join cost. →
   q04 40.8 → 13.5 ms, q05 53.4 → 19.3 ms, q02 4.7 → 1.2 ms; compute geomean
   **0.41× → 0.81×**.

Compute pass now (Olympics, min-of-7 cold; QLever in Docker-on-macOS):

| query | sparq | qlever | speedup |
|---|--:|--:|--:|
| q02 type-scan | 1.21 ms | 3 ms | **2.48×** |
| q03 star-4 | 17.3 ms | 17 ms | 0.98× |
| q04 athlete-age | 13.4 ms | 13 ms | 0.97× |
| q05 result-star-3 | 19.6 ms | 19 ms | 0.97× |
| q06 filter | 3.87 ms | 1 ms | 0.26× |
| q07 medal join | 14.5 ms | 10 ms | 0.69× |
| q08 label-age | 6.96 ms | 9 ms | **1.29×** |
| q10 OPTIONAL | 25.9 ms | 9 ms | 0.35× |
| **geomean** | | | **0.81×** |

sparq is now at parity-or-better with (Dockerized) QLever on the scan/star/chain
queries, and still behind on `q06` (numeric filter — small absolute numbers) and
`q10` (OPTIONAL/left-join overhead). Result counts remain identical to QLever.

3. **Numeric FILTER pushdown + sorted-column scan (M4 first step).** The hardware
   research measured q06's bottleneck as a *random gather* into the `Vec<f64>`
   numeric cache (8–15× slower than contiguous). Fix: push a sargable `?v OP const`
   FILTER into the pattern scan, scanning via that column's permutation so the
   numeric access is *sequential*, and apply the predicate inline so failing rows
   are never materialised. Measured: **olympics q06 3.4 → 0.27 ms (12.5×)**;
   **10M synthetic q06 42 → 3.9 ms (10.7×)** — its gap to native QLever went from
   20× to ~2× (0.05× → 0.51×). Results identical to QLever. This is the first,
   bounded piece of M4; the full tagged-ValueId columnar layout is the larger
   architectural follow-on (it removes the gather entirely).

4. **Sort-merge left outer join (OPTIONAL).** OPTIONAL used a hash left-join whose
   table build over the whole right side dominated. For the common single-shared-
   variable case it now sorts both sides (near-linear — the scans are already
   key-sorted) and merges, like QLever. Measured: **10M q10 324 → 48 ms (6.75×)** —
   now **faster than native QLever** (58 ms, 1.21×); olympics 17.5 → 8.3 ms.
   10M compute-pass geomean vs native QLever: **0.39× → 0.49×** (QLever's lead
   3.7× → ~2×; sparq now wins q10 and is near-parity on q06). Remaining 10M gaps
   are scan/materialisation bound (q02 full-scan 0.23×, q04 5M-row join 0.29×) —
   the columnar-output / M3-compression territory.

### sparq now wins/ties ALL 5 compute queries vs native QLever (10M)

5. **Tagged ValueIds (inline integers) + range-pruning.** Canonical small
   `xsd:integer`s are encoded *inline in the u32 id* (`id = INLINE_BASE + value`,
   `INLINE_BASE = 1<<30`): no dictionary entry, no string parse, and they **sort by
   value** in the permutations. Numeric FILTER then range-prunes — binary-search to
   the passing value sub-slice instead of scanning the whole relation. Measured:
   **q06 `FILTER(?a > 90)` 4.8 → 1.61 ms** (scans 140k rows, not 1.25M) — now
   **1.24× faster than native QLever** (was 0.39×). Memory-neutral (integers leave
   the dictionary) and correct (all tests + QLever result counts match; only the
   canonical lexical form inlines, so term identity is preserved).

**10M compute pass, vs native QLever — sparq wins or ties every query:**

| query | sparq | qlever | speedup |
|---|--:|--:|--:|
| q02 count (lazy) | ~0 ms | 4 ms | huge |
| q03 star join | ~85 ms | 73 ms | ~0.85× |
| q04 join count (count-only) | 31 ms | 54 ms | **1.5×** |
| q06 numeric FILTER (range-pruned) | 1.6 ms | 2 ms | **1.24×** |
| q10 OPTIONAL (sort-merge) | 48 ms | 59 ms | **1.23×** |

The 10M gap has gone from **QLever 3.7× faster overall** to **sparq matching or
beating native QLever on all five** — with every result identical to QLever.

### Next (driven by the remaining deltas)

After filter pushdown + merge left-join, the remaining 10M gaps are
scan/materialisation bound: **q02 full-scan-materialise (0.23×)**, **q04 5M-row
join (0.29×)**, **q03 star (0.64×)**.

> **Measured negative result (don't repeat): a flat `Vec<Id>` row buffer.** The
> hypothesis was that `Bindings`' `Vec<SmallVec<[Id;4]>>` (32 B/row) wastes ~8× on
> narrow results vs a flat `Vec<Id>`+width. Implemented and measured: it **regressed**
> q02 13→17.6 ms and q04 158→204 ms, and was reverted. Reason: `SmallVec` rows are
> *inline* (no per-row heap allocation), so `Vec<SmallVec>` is already cheap to
> build (one 32 B memcpy/row); the flat buffer adds a *second* copy (build a temp
> row, then `extend_from_slice` into the buffer). The memory saving only pays if
> rows are read many times downstream, not for build-once-count/materialise. The
> real q02/q04 lever is therefore **lazy/streaming execution** (count/aggregate
> without materialising the full result — what QLever's lazy operators do), a
> larger redesign, **not** the row storage layout.

These point at:

1. **Lazy / streaming operators** — evaluate count/aggregate/LIMIT without
   materialising the whole intermediate (QLever's block-based lazy model); the
   real q02/q04 lever. (COUNT(\*) over a single pattern is already pushed down to
   the range size.)
2. **Full tagged ValueIds** — inline numerics/dates in the id itself (extends the
   numeric cache + the pushdown), removing the gather entirely.
3. **M3 column compression** (PForDelta + front-coded vocab) + **parallel bulk
   load** — the memory lever and the Wikidata-ingestion lever (bz2 decompression
   is the current ingest bottleneck).
4. **Larger datasets** (SP2Bench/WatDiv, then a 100M+ WDBench/Wikidata subset) to
   test QLever's out-of-core regime where sparq must learn to bound memory.
5. Show the **WCOJ edge** on a skew/cycle-heavy benchmark (QLever uses binary
   joins) — our one likely asymptotic win.

## Scaling: 10M triples — the gap WIDENS (the key honest finding)

`bench/qlever-synthetic/` runs the same two-pass comparison on a 10M-triple
synthetic graph (`sparq-bench dump 1250000 synthetic.nt`). All result sizes still
match QLever exactly. Compute pass (min-of-5 cold):

| query | sparq | qlever | speedup |
|---|--:|--:|--:|
| q02 type-scan (1.25M) | 12.8 ms | 7 ms | 0.55× |
| q03 star-3 | 88 ms | 90 ms | 1.02× |
| q04 follows→name (5M rows) | 163 ms | 54 ms | 0.33× |
| q06 filter (1.25M scan) | 42.5 ms | 2 ms | **0.05×** |
| q10 OPTIONAL | 338 ms | 56 ms | 0.17× |
| **geomean** | | | **0.27×** |

**At 10M, QLever is ~3.7× faster on compute — and the gap GREW from the 1.8M
olympics result (0.80×).** sparq's parity at small scale did **not** hold:
in-memory flat `Vec<[u32;3]>` permutations and full-scan filters lose to QLever's
**compressed columnar blocks, lazy/block scan, and tagged ValueIds** as data
grows. The numeric FILTER is now **20× slower** (q06 0.05×) — a full 1.25M-row
materialised scan vs QLever's predicate-pushed block scan. (The end-to-end pass
still shows sparq "winning" 4–12×, but that is entirely QLever's slow JSON export
of millions of rows in Docker — not compute.)

**Conclusion (at the time):** to outcompete QLever *at all scales*, the architecture
itself must change — M3 (column compression + vectorised/block scan + filter pushdown),
M4 (tagged ValueIds), plus out-of-core. Micro-optimisation got sparq to parity on small
in-RAM data; scaling needs the structural work.

### UPDATE (current): that gap has REVERSED — sparq now beats QLever on compute at 10M

The table above is **superseded**. Re-measured after this project's work — the
6-permutation restoration (a feature-unification bug had silently dropped native to 3
perms, crippling merge joins), lazy/streaming counts (N-star, filtered, OPTIONAL), and
the index-nested-loop bind join. Same machine, same `COUNT(*)`-wrapped compute queries,
min-of-8 cold, vs the stored QLever baselines (`bench/qlever-baselines.md`):

| query | sparq compute | QLever | speedup |
|---|--:|--:|--:|
| q02 type-scan (1.25M) | 0.004 ms (lazy: pred-stat) | 4 ms | **~1000×** |
| q03 star-3 | 28 ms | 74 ms | **2.6×** |
| q04 follows→name | 35 ms | 56 ms | **1.6×** |
| q06 filter (1.25M) | 0.005 ms (lazy: range count) | 2 ms | **~400×** |
| q10 OPTIONAL | 54 ms | 60 ms | **1.1×** |

**sparq now matches or beats QLever on compute for all five queries at 10M.** Honest
framing: q02/q06 win by *computing the count without scanning* (characteristic-set
pred-stats; binary-searched filtered range) — a legitimately better algorithm for
COUNT workloads, the same class QLever's own COUNT optimiser targets. q03/q04/q10
*actually execute* the join/OPTIONAL and still win, from the 6-perm merge joins + bind
join. The earlier numbers were real at the time but pre-dated the 6-perm fix (native was
accidentally on 3 perms) — the single biggest factor. The materialise path (returning
millions of rows, not counting) is a separate axis dominated by row construction and not
directly comparable to QLever's Docker JSON export; see below.

### 100M, OUT-OF-CORE — the win holds at scale, committing ~zero heap

The decisive test for "few → billions": query the 100M index **out-of-core** (perms +
numerics + dict + stats all mmap'd/persisted; `sparq-cli bench-mmap`) and compare to the
stored QLever 100M baselines. Open 1.9 s, **index committed heap ~0**; compute (count,
min-of-6):

| query | sparq (out-of-core) | QLever 100M | speedup |
|---|--:|--:|--:|
| q02 type-scan | 0.005 ms (lazy) | 35 ms | ~7000× |
| q03 star-3 | 291 ms | 900 ms | **3.1×** |
| q04 follows→name | 349 ms | 600 ms | **1.7×** |
| q06 filter | 0.005 ms (lazy) | 8 ms | ~1500× |
| q10 OPTIONAL | 517 ms | 650 ms | **1.26×** |

**sparq beats QLever on compute for all five at 100M too — while the index is entirely
memory-mapped (a 16 GB machine querying a dataset whose indexes are ~8.4 GB on disk).**

The first 100M run peaked at ~5.6 GB anonymous memory, which I initially attributed to
"the join compute's working set." **Per-query measurement proved that wrong** — it was
the COUNT machinery, in two specific places, both fixed:
- `COUNT(*)` over an OPTIONAL was not using the lazy left-join count (`count_pushdown`
  returns `None` for a non-conjunctive `LeftJoin`), so it materialised the whole
  left-join just to count it (q10: 196 MB → 43 MB at 10M).
- the multi-pattern star COUNT materialised one `(value,count)` vector per pattern;
  replaced with a k-way intersection merge of lazy `GroupStream`s that accumulates the
  product sum directly (q03 84 MB → 1.2 MB, q04 63 MB → 1.2 MB), and `count_leftjoin`
  likewise streams both sides (q10 43 MB → 1.2 MB).

After the fixes the **whole 100M out-of-core COUNT benchmark peaks at 2.79 MB** of
committed memory (was 5.6 GB), and the queries got *faster* (the streaming merge reads
the group column directly from the permutation row, no triple reconstruction):

| query | sparq (out-of-core) | QLever 100M | speedup |
|---|--:|--:|--:|
| q03 star-3 | 467 ms | 900 ms | **1.9×** |
| q04 follows→name | 568 ms | 600 ms | **1.06×** |
| q10 OPTIONAL | 256 ms | 650 ms | **2.5×** |

**The headline stands and is now clean: across a few triples → 100M, in-RAM and
out-of-core, sparq matches or beats native QLever on compute while committing single-digit
MB regardless of dataset size** — reversing the earlier finding, via the 6-permutation
fix + lazy/streaming counts + bind join. (The *materialise* path — returning millions of
rows — is the separate axis where columnar/streaming execution is still the next bet; it
is not what the COUNT compute above measures.)

## Out-of-core: the "few → billions" range (query AND build past RAM)

The scaling section's conclusion named "out-of-core for the billion-triple regime
where the current in-memory store would OOM" as required for *all scales*. That is
now implemented in two halves, both measured.

**Query past RAM (mmap'd indexes).** The permutation indexes (72 B/triple — the
dominant memory) are persisted as raw `[u32;3]` files and memory-mapped on open, so
the OS pages in only the working set. The dictionary stays in RAM; the large indexes
do not. CLI: `save` then `query-mmap`.

| 10M, follows·name join | in-memory | out-of-core (mmap) |
|---|--:|--:|
| peak RSS | 2.09 GB | **499 MB** (−76%) |
| query time | 30.8 ms | 30.8 ms (same) |

**Build past RAM (external-memory sort/merge).** `save()` of an in-memory load still
needed the whole index in RAM. `Graph::build_external` (CLI `build`) instead sorts
the triples *through disk*: stream-parse + intern → spill `chunk`-sized SPO-sorted
runs → k-way merge (min-heap over mmap'd runs) into the SPO index, dedup → build the
other 5 permutations by external-sorting the SPO file into each order. Peak RAM is one
chunk buffer + the merge heap + the dictionary — not the dataset.

| build | in-memory `save` | external `build` |
|---|--:|--:|
| 10M, peak RSS | 2.02 GB | **486 MB** (−76%), chunk=2M |
| 100M (7.2 GB of perms) | ~10 GB (would thrash/OOM a 16 GB box) | **3.68 GB**, chunk=8M |

The external build's output is **byte-identical** to the in-memory `save` (all six
`perm*.bin` + `dict.bin` sha256-identical on the real 10M set; a unit test asserts
this with a tiny chunk that forces many runs + merge, including dedup of a duplicate
line). The 100M index opens and answers `COUNT(*) = 99,999,990` (100M − 10 dups) and a
real join in <0.1 ms, with the perms entirely mmap'd.

**Parallel parse (−45%).** The external build first parsed single-threaded (oxttl
streaming) while the in-memory load already used the parallel custom byte parser.
Measured: parse+intern is ~64% of the external build (8.2 s of ~12.9 s at 10M) — the
bottleneck, and exactly the "parallelise parsing of the file" requested for large files.
N-Triples is now streamed in newline-aligned ~64 MiB blocks, each parsed + interned in
parallel (per-chunk partial dicts merged into the running dict) before spilling — still
one block resident at a time. **10M external build 12.9 s → 6.6 s (−45%); 100M
175–206 s → 113.7 s (−40%)**, with peak memory *lower* than the sequential build
(2.59 GB vs 3.68 GB — the 64 MiB block + transient partials replace the streaming-parser
buffer). Output stays byte-identical (the byte-identity unit test now exercises the
parallel path) and the 100M index opens + answers `COUNT(*) = 99,999,990`.

Net: a 16 GB machine can now both **construct** and **query** a 100M–1B-triple index
whose permutations are far larger than its RAM — the "billions" end of the range.

**Query with ~zero committed RAM (mmap dict + persisted stats).** Initially the
out-of-core *query* still held two things resident: the dictionary (`Vec<Stored>` +
its lookup table, **1.45 GB at 100M**, rebuilt on open — measured at 100M: term parse
1.39 s + table rebuild 2.40 s) and the per-predicate stats, which `open` recomputed by
**scanning the full POS+PSO permutations** (~2 perms faulted in). Both are now off-heap:

- **Memory-mapped dictionary.** `save_mmap` writes the term-record blob, per-term byte
  offsets, and a hash-**sorted** `(hash,id)` index; `open_mmap` mmaps them. `term(id)`
  parses a record zero-copy from the blob (a borrowed `StoredRef`); `lookup` binary-
  searches the sorted hash index and verifies candidates. Open is just `mmap` — no parse,
  no table rebuild — and the term store is off-heap.
- **Persisted per-predicate stats** (`predstats.bin`), computed once at build, so `open`
  loads them instead of re-scanning POS/PSO.

Each fix exposed the next bottleneck in the measurement: mmap'ing the dict dropped open
7.95 s → 3.77 s, which revealed the stats scan; persisting the stats dropped it to
**1.9 s**. The decisive figure is **committed memory** (macOS "peak memory footprint",
which excludes evictable file-backed mmap pages):

| open + query a dataset out-of-core | committed memory (peak footprint) |
|---|--:|
| 10M (720 MB of perms) | **1.28 MB** |
| 100M (7.2 GB of perms + ~1.2 GB dict) | **2.48 MB** |

Committed memory is **~constant at a few MB regardless of dataset size** — everything
large (perms, numeric cache, dictionary, stats) is mmap'd or persisted, so the process
commits almost no anonymous memory. (The *RSS* still shows GBs, but that is evictable OS
page cache — file-backed mmap pages, warm from the build — not committed heap; under
memory pressure the OS reclaims them.) Validated: 12,000 differential cases vs Oxigraph
through the mmap dict (lookup + materialisation), 0 mismatches. This closes the earlier
caveat — the dictionary no longer has to fit in RAM, so the out-of-core query path scales
to datasets whose dict *and* perms exceed RAM, committing only a few MB.

## Block-compressed permutation index (the browser-memory lever)

The permutation indexes are the dominant memory (72 B/triple = 6 × `[u32;3]`). A
sorted permutation is highly compressible because consecutive triples share prefixes.
`sparq-cli probe-compress` measured the ceiling on real data before any engine change:
a **lexicographic-delta + LEB128-varint** encoding (decodable per 128-triple block, so
scans stay random-accessible) lands at:

| permutation | synthetic 100M | **real Wikidata 20M** |
|---|--:|--:|
| SPO | 5.40 B/triple (45%) | **3.69 B/triple (31%)** |
| POS | 5.00 B/triple (42%) | 5.56 B/triple (46%) |
| OSP | 5.00 B/triple (42%) | 5.00 B/triple (42%) |

i.e. **−55% to −69%**, close to gzip (37%) but ~30× faster to decode (40–320 M/s) and
randomly accessible (+0.16 B/triple for a block directory). Real Wikidata compresses
*better* than synthetic (higher subject/predicate locality).

Built it as an opt-in storage mode (`PermData::Compressed`): scans decode only the
blocks their key-range touches, returning an owned range; the operators are unchanged
(`Scan.rows` became a `Cow` — raw modes still borrow a zero-copy sub-slice). The native
in-memory default stays raw (no decode cost); compressed is for the **memory-bound
browser** and out-of-core.

**Measured end-to-end (10M synthetic, full 6-perm set):**

| | raw | compressed |
|---|--:|--:|
| perm memory | 720 MB (72 B/triple) | **367 MB (36.8 B/triple, −49%)** |
| total graph (perms+dict+numerics) | 0.95 GB | 0.60 GB |
| scan 2M rows → JSON | 222 ms | 243 ms (+10%) |
| scan+filter → JSON | 109 ms | 130 ms (+19%) |
| 2-pattern join 1M → JSON | 212 ms | 281 ms (+33%) |

The browser's 3-perm compact set compresses **−60%** (2.5× more triples per byte of
tab RAM). The latency cost is bounded per-scan decode; the join's +33% is from the bind
join re-decoding the inner pattern's range per outer row — a per-permutation block cache
would cut it (future work). **Correctness:** identical results to the raw store across a
store-level pattern/estimate/pred_stat equality test, **20,000 differential cases vs
Oxigraph through the compressed store** (0 mismatches), and wasm `load==loadCompressed`.

### Measure-first: why NOT a succinct / move-structure permutation index (yet)

The recurring "billions in the browser" idea is a repetition-aware succinct index
(move-r / BWT self-index) that stores the permutations near their entropy with O(1)
random access. Before building one (a multi-week research effort), measure the headroom.
On the 10M SPO permutation:

| | B/triple | properties |
|---|--:|---|
| raw | 12.0 | |
| **delta+varint (shipped)** | **4.94** | lightweight, no dep, random-access, 40–320 M/s decode |
| zstd −19 | 3.55 | ~100 KB+ decoder, block-framed, slower |
| xz −9e (LZMA) | 2.51 | very heavy decoder |

So there *is* entropy headroom — but three measured facts make a heavier index not worth
it **now**: (1) this is synthetic data (sequential ids — pathologically LZMA-compressible);
on **real Wikidata** delta+varint is already 3.69 B/triple, so the real gap is far smaller.
(2) After `Dict::into_blob`, the **permutations are no longer the floor** — in the
compressed wasm store they are ~6 B/triple of ~29–39 total; the dictionary blob + offsets
+ the f64 numeric cache dominate. A succinct *perm* index would shrink <10% of total. (3)
Capturing the headroom needs either a ~100 KB+ zstd/LZMA decoder in the bundle (which
costs browser memory + download — self-defeating) or the move-r research build, for that
<10% gain. The lightweight delta+varint is the right tradeoff for a small-bundle,
fast-decode, randomly-accessible browser index; the next memory levers (if pursued) are
the dict blob (FSST measured at only −20% on real Wikidata — rejected) and a sparse
numeric cache, both modest. Recorded so the succinct index isn't built on intuition.

### Negative result: a block-decode cache for compressed bind joins did NOT help

The compressed join's +33% comes from the bind join re-decoding the inner pattern's
block per outer key. The obvious fix: sort the distinct join keys (so the inner scans
walk the permutation monotonically) and reuse a one-block-span decode cache across
consecutive keys. Built it behind an A/B env toggle and measured on the 10M set:

| join (compressed) | with cache | without cache |
|---|--:|--:|
| follows·name (93 MB out) | 294 ms | 297 ms (neutral) |
| follows·city (476 MB out) | 873 ms | **752 ms** (cache 16% *slower*) |

Reverted (keep-only-if-it-helps). Two reasons it loses: (1) sorting ~1.25M distinct
keys costs more than the re-decode it saves, since (2) a compressed block decode is
already cheap (~128 varints) and these large joins are dominated by *building the
output* (hundreds of MB of JSON), not by inner-scan decode. A cache only pays when
inner-scan decode is a real fraction of the work *and* keys cluster tightly enough to
amortise the sort — not the case here. The compressed join's overhead is best left as
the measured, accepted cost of the memory win; a future win would need cheaper output
materialisation, not a scan cache.

### Honest correction: the persisted-numerics open is *lighter*, not *faster*

A separate change persists + mmaps the numeric-value cache (`numerics.bin`) so `open`
no longer recomputes it. It does cut RAM (the 100M cache is 210 MB, ~2 GB at 1B, now
off-heap). But the 100M open time was **unchanged (~6.7 s)** — it is dominated by
*dictionary* deserialization + hash-table rebuild, not the numeric recompute. The
commit framed it as "faster"; the measured truth is "lighter, same speed." Faster
out-of-core open would need a persisted dict lookup structure (a later lever).

## Negative result: streaming a 2-pattern join to JSON did NOT help

After the streaming single-pattern scan→JSON path won (q02 71→61 ms by skipping the
`Bindings`), the natural next step was to stream a 2-pattern join (hash-join the two
scans, emit JSON per match, no `Bindings`). Implemented and validated correct (90k
differential fuzz cases vs the general path, via an order-independent **bindings-multiset**
check now kept permanently in `sparq-bench fuzz`). But **q04 json stayed at 1517 ms,
unchanged** — so it was reverted (keep-only-if-it-helps, the same discipline that reverted
the flat buffer).

The lesson: for a *join*, the `Bindings` intermediate is NOT the bottleneck — the join
compute (hash build + probe) and the result serialisation (q04 is **701 MB** of JSON)
dominate, so fusing away the intermediate `Vec<Row>` saves nothing measurable. Streaming
only pays where the per-row materialisation is a real fraction of the work — i.e. the
*single-pattern* (scan / scan+filter) case, which is kept.

### The materialise path is OUTPUT-bound, not compute-bound (so "columnar execution" wouldn't help it)

Measured q04 (5M result rows, 10M dataset) across modes — *same join*, different output:

| mode | q04 time | what it does |
|---|--:|---|
| count | **23.5 ms** | lazy join cardinality (Σ over the join var) |
| materialise → Terms | 1127 ms | builds 5M rows + 15M `oxrdf::Term`s |
| query_json | 1276 ms | builds 5M rows + **701 MB** of JSON |

The join *compute* is 23 ms; the 1100+ ms is **producing and serialising 15M output
cells** — inherent to a 5M×3 result, independent of the join algorithm. So the
oft-named "columnar/vectorised execution" lever (a *compute* optimisation) would not move
the materialise path here — it is output-bound, and the output is what the query asked
for. The only real lever is output-conversion efficiency.

**Bounded win taken there:** the JSON string escaper (`escape_into`, called once per
output cell — 15M times for q04) pushed char-by-char; rewritten to bulk-copy maximal
already-safe runs and only handle the rare escape byte individually (byte-level, safe for
UTF-8 since every escaped byte is ASCII). **q04 JSON 1276 → 1034 ms (−19%)**, validated by
50k differential cases (incl. the `query_json` multiset check) + an escaper unit test.
Further materialise gains need a more compact result *format* (an API change) or faster
output — not a columnar *engine*.

## Correctness validation (differential vs Oxigraph) + known limitations

The optimizations were audited two ways against Oxigraph (an independent, mature
SPARQL engine): a random differential fuzzer (`sparq-bench fuzz`, 800k+ cases over
bgp/filter/equality/optional/union/minus/limit/distinct/order, checking solution
count, the lazy `count()` path, and ORDER BY sequence) and an adversarial agent
panel (`sparq-bench diff`, hand-crafted edge cases). The audit **found and fixed a
real bug**: numeric ordering FILTERs against a non-numeric operand (`?v > -1` with
`?v` a string/IRI/boolean) were doing a lexical comparison instead of raising a
SPARQL type error — it hid on the non-sargable residual path (negative thresholds),
which positive-threshold fuzzing never reached. Fixed by modelling SPARQL type
errors (`Value::Error`) + 3-valued `&&`/`||`/`!`; the fuzzer now generates signed
thresholds and a cross-type `=`/`!=` category so it can't regress.

**Exact numeric COMPARISON beyond f64 precision — fixed.** sparq holds numeric values
as `f64` (the cache + inline-ValueId fast path that powers range-pruning), exact for
`|integer| ≤ 2^53` and ≤15-significant-digit decimals — i.e. essentially all real RDF
data. The fuzzer had never exercised the edges (values were `< 120`); extending it to
near-2^53 integers and >15-digit decimals surfaced mismatches (11/30k then 25/40k) —
e.g. `"9007199254740992" = "…993"` and `"0.123456789012345678" = "…679"` (each pair
shares one f64). Now closed with a single exact path. The key property: **f64 rounding
is monotonic, so it only ever collapses distinct values to equal — it never flips an
ordering.** So `cmp_expr`/`equal_expr` re-check exactly **only when the f64 result is
equal** (rare), via `cmp_decimal_str` on the operands' lexical forms (exact for integers
and `xsd:decimal` at any precision); `extract_sargable` declines thresholds with >15
significant digits. Everything f64 orders distinctly is untouched — **zero regression**.
Re-measured: **100,000 differential cases vs Oxigraph** (near-2^53 integers +
high-precision decimals + all categories), **0 mismatches**; the fuzzer generates both
permanently, and `cmp_decimal_str` has its own edge-case unit tests.

**Remaining (deliberate) gap — exact `xsd:decimal` ARITHMETIC.** `0.1 + 0.2 = 0.3` is
false in f64. Comparison is now exact, but arithmetic (`Add`/`Subtract`/… in
`eval_numeric`) is still f64. A full fix needs a decimal type in the arithmetic path;
deferred (sparq does no decimal arithmetic in the measured workloads, and the f64 path
is load-bearing for the filter wins). The narrowest remaining numeric gap — flagged.

## Other next steps

- Decimal arithmetic type to close the exact-`xsd:decimal` *arithmetic* gap.
- Native QLever (not Docker) for the fairest fight; cold-cache and QMpH metrics.
- Re-measure memory against QLever (compressed) and after M3 column compression.
