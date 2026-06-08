# Data structures for sparq — the definitive design-space report

**Scope.** Seven structured family surveys (rdf-engine-indexes, succinct-self-indexes,
ordered-mutable-trees, wcoj-and-factorized, columnar-vectorized-compression,
learned-and-adaptive, graph-native-and-gpu) covering ~50 graph/RDF data structures.
This report maps the design space against sparq's committed architecture, gives a
prioritised honest adoption plan, names the 2–3 highest-leverage bets and what to
measure, and is explicit about what does **not** fit sparq's regime.

**The regime sparq is built for (the lens for every judgement below).** In-RAM,
dictionary-encoded, **sparse, high-cardinality**, bulk-load-only read core;
**memory-bandwidth/latency bound** (not ALU-bound — established by the prior
hardware research, ~3.8 G rows/s contiguous scan on the M1, ~62 GB/s ceiling);
**correctness-first** (exact RDF-term equality, no row caps, correct path duplicate
semantics — the #Diverge weapon); two deployment targets that pull in different
directions — **native billion-scale** (full Wikidata ≈20B on one ~€2.5k box) and
**browser/WASM** (sub-~1.5 MB engine + a separately-fetched index blob, 2–4 GB
address-space ceiling).

**The committed core (what everything is measured against).** Six sorted ID
permutations (PSO/POS/SPO/SOP/OPS/OSP) → column-major ZSTD-3 blocks with
leading-column elision + per-block `firstTriple`/`lastTriple` skip metadata (M3);
tagged 64-bit inline `ValueId`s (M4); merge/gallop/hash + Leapfrog-Triejoin WCOJ
with a GraphflowDB-style hybrid router; vectorized morsel-driven lazy execution;
DPccp/DPhyp + characteristic-set cardinality. Plus the bit-level report's standing
recommendation: a **Roaring dense-predicate tier (P1)** for the few dozen densest
predicates (`rdf:type`, booleans) where the measured crossover (~5–10 % density)
favours bitmap-AND over sorted-merge.

**The one-paragraph verdict.** **No structure in any of the seven families beats
sparq's core as a wholesale replacement — because sparq's core IS the convergent
winner of the rdf-engine-index literature** (RDF-3X, Hexastore, QLever,
MillenniumDB all converge on "dictionary-encode → keep sorted permutations →
sort-free merge joins → compress hard," and sparq took QLever's best version and
added WCOJ + streaming + correctness). The families pay off **per-subsystem**:
**finish M3** (the measured 3.7× compute gap at 10M is unbuilt compression, not a
wrong design); add **per-block Bloom filters** and **Elias-Fano compressed-seek
columns** as the two highest-leverage *new* storage bets; adopt **COLT/Free Join**
to win the join subsystem; reserve **factorised execution** for the M4+ many-to-many
/ streaming wedge; reserve **succinct (Ring/CompactLTJ)** for the M5+ billion-scale /
browser compression endgame; and confine all the **mutable trees** (ART/BS-tree) and
**RDFox-style overlays** strictly to the M5 write tier. Tags: **[measured]** (a
real number sparq or a survey measured), **[literature]** (peer-reviewed result),
**[claimed-by-authors]** (authors' own numbers, not independently reproduced),
**[speculation]** (our inference). No numbers are fabricated.

---

## 1. The design space — families × fit

Per-dimension fit for sparq's regime: **5** = already the core / clear win,
**4** = strong, adopt-after-spike or for one subsystem, **3** = useful component or
specific tier, **2** = niche / contrast, **1** = reject. "Browser" = fit under the
2–4 GB WASM ceiling + small bundle. The *Fit* column is the representative/best
member's headline score from the surveys.

| Family (representative members) | Storage | Scan | Join | Filter | Update | Cardinality | Browser | Fit | Regime note |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|---|
| **RDF-engine perm-indexes** — RDF-3X, Hexastore, QLever, MillenniumDB | 5 | 5 | 4 | 4 | 2 | 5 | 4 | 5 | **This IS sparq's core.** QLever = the adopted target; RDF-3X aggregated indexes + byte-delta leaves are the transplants; fewer-than-6 covers license the WASM build. |
| **Columnar codecs** — dict, ZSTD blocks, Elias-Fano/PEF, FOR/bit-pack, SIMD-BP128, Stream-VByte, RLE, Arrow | 5 | 5 | 4 | 4 | 2 | 4 | 4 | 5 | sparq's storage substrate, refined. **EF is the one differentiated new bet** (compressed seek). ZSTD = the M3 default everything must beat. |
| **WCOJ + factorized** — COLT/Free Join, factorised f/d-reps, Kuzu vectors, GraphflowDB, EmptyheadeD, CompactLTJ | 3 | 3 | **5** | 2 | 2 | 4 | 3 | 5 | **Beats sparq on JOIN.** COLT = drop-in win now; factorised reps = the M4+ m-n/streaming wedge; succinct LTJ = M5+ space tier. |
| **Succinct self-indexes** — the Ring, C-Ring, RDFCSA, URing, Qdag, HDT, wavelet trees | **5** | 2 | 4 | 2 | 1 | 3 | **5** | 4 | **Beats sparq on STORAGE+BROWSER, loses on cache-resident latency** (rank/select pointer-chasing). The Ring = M5+ compression endgame; HDT = browser snapshot. |
| **Ordered mutable trees** — ART, BS-tree, HOT, Masstree, ALEX, B-ε/LSM, FAST, MillenniumDB B+tree | 3 | 3 | 3 | 3 | **5** | 2 | 2 | 4 | **Only the UPDATE subsystem (M5).** BS-tree = best read-preserving mutable; ART (congee) = fastest to prototype. Flat sorted array still wins the read core. |
| **Learned + adaptive** — PGM, RMI, RadixSpline, ALEX, GNCE cardinality, cracking, multi-dim | 2 | 2 | 1 | 2 | 3 | **3** | 2 | 3 | **Only PLANNER (GNCE cardinality) and maybe M5 writes (dynamic PGM).** Search-step accelerators lose in-cache; cracking rejected (1). |
| **Graph-native + GPU** — CSR/CSC, EmptyHeaded SIMD sets, GraphBLAS/MAGiQ, Kuzu, Neo4j, GPU CSR (Gunrock), gStore-GPU | 2 | 3 | 4 | 2 | 3 | 2 | 1 | 4 | **Only EmptyHeaded's SIMD set-intersection kernel is on-regime.** CSR ⊆ sparq permutations; GPU gated by transfer tax; Neo4j is an anti-pattern. |

**Reading the table.** Three families touch the **core** (perm-indexes, columnar
codecs, WCOJ) and that is where the action is. Three families touch **exactly one
peripheral subsystem each** — succinct → storage/browser tier, mutable trees →
updates, learned → planner. One family (graph-native/GPU) contributes a single
kernel (SIMD set-intersection) and is otherwise analytics/scale-out, not BGP
serving. The Update column is uniformly weak across every *fast read* structure —
the family-wide honest gap that M5 must fill with an overlay, never by mutating the
read core.

---

## 2. Prioritised adoption plan

Each item: **expected win**, **implementation cost**, **how it composes** with the
existing core (dict + 6 permutations + WCOJ + inline ValueIds + the Roaring
dense-tier), and which **goal** it serves (native-billion-scale vs browser). The
overriding rule from the bit-level report and the hardware research: a structure
helps **iff it moves fewer bytes per result**; it hurts whenever it forces
materialisation a sorted scan would skip, or adds pointer-chasing on a
bandwidth-bound machine.

### (a) Adopt now

These are already committed or are small additive wins with measured/literature
backing, on-regime, and answer-safe.

**A1. Finish M3: column-major ZSTD-3 blocks + leading-column elision + per-block
skip metadata.** *(perm-indexes / columnar — QLever, COTTAS)*
- **Win:** closes the **measured 3.7× compute gap** at 10M triples (BENCHMARKS.md:
  q02 full-scan 0.23×, q04 5M-row join 0.29× vs native QLever) — that gap is
  *unbuilt compression*, not a wrong design. **[measured]** Memory: QLever 8 GB vs
  Oxigraph 67 GB on DBLP-390M is the bar. **[measured]**
- **Cost:** medium — it is the M3 milestone (one dep: `zstd`; block layout +
  metadata + binary-search scan + scan↔scan lazy block join). The substrate is
  triply confirmed (QLever source, RDF-3X, COTTAS ISWC'25). mmap/zero-copy + WASM
  build both ship.
- **Composes:** it is the storage layer *under* the 6 permutations; WCOJ/merge/hash
  read decompressed blocks; inline ValueIds live in the columns unchanged; the
  Roaring dense tier sits beside it for dense predicates.
- **Goal:** both. The default that A2/A3/B1 must beat in a measured A/B.
- **Honest weakness it leaves open:** ZSTD has **no compressed seek** — a block must
  be fully decoded even for a point/seek touching one row. This is the exact gap
  Bloom filters (A2) and Elias-Fano (B1) fill.

**A2. Per-block Bloom filters on the high-NDV (subject/object) columns.**
*(perm-indexes — COTTAS ISWC'25)*
- **Win:** the min/max (`firstTriple`/`lastTriple`) zone map already prunes **range**
  patterns, but for an **equality-bound constant** whose id falls inside many blocks'
  overlapping `[min,max]` ranges — the common case for high-cardinality subject/object
  columns — only a Bloom filter can skip those blocks. Directly attacks sparq's own
  measured weak spot (full-scan materialisation). **[literature/claimed-by-authors;
  COTTAS showed ~50% smaller than HDT with Bloom-augmented zone maps]**
- **Cost:** small — a few hundred LOC + one bitset per block, built in the loader.
- **Composes:** purely additive to the M3 block metadata; falls back to the existing
  scan; no join/correctness impact.
- **Goal:** both (a flat per-block bitset is WASM-trivial). **This is the single
  cheapest new pruning win and the recommended first prototype.**

**A3. Promote RDF-3X aggregated indexes to first-class cardinality oracles.**
*(perm-indexes — RDF-3X)*
- **Win:** a dedicated tiny aggregated B+-tree / sorted array of `(SP→count)`,
  `(PO→count)`, … is the cheapest *exact* star/selectivity estimator and doubles as
  a **2-bound COUNT answerer**. sparq currently plans to *derive* 1-/2-prefix counts
  from block metadata; RDF-3X shows the dedicated structure is near-free and exact.
  **[literature]**
- **Cost:** small — largely already available since permutation block metadata yields
  exact 1-/2-prefix counts (ARCHITECTURE §3.2 "aggregated indexes for stats"); the
  work is making them a first-class planner input + COUNT fast-path.
- **Composes:** feeds the DPccp/DPhyp planner and the AGM/ρ* LP; complements
  characteristic sets (which cover stars) on the cheaper exact-pair side.
- **Goal:** both.

**A4. COLT — Column-Oriented Lazy Trie (Free Join).** *(WCOJ — Wang/Willsey/Suciu,
PACMMOD 2023, in Rust)*
- **Win:** the literal answer to "WCOJ without rebuilding a trie per query." A COLT
  leaf is a vector of offsets into a permutation block; an internal node is a
  *lazily-forced* `FxHashMap<ValueId, COLT>` — initialised as `[0..n-1]` with **no
  trie built**, forcing a level only when probed. It **eliminates the per-query
  trie-materialisation** that is sparq's known WCOJ bottleneck and **unifies
  merge/hash/Leapfrog into one operator** over the permutations sparq *already has*.
  An algorithm converts any cost-based binary plan into a Free Join plan that runs
  as-fast-or-faster. DuckDB: up to **19.36× over binary join / 31.6× over Generic
  Join** on acyclic JOB, **15.45× / 4.08×** on cyclic LSQB. **[claimed-by-authors,
  arxiv 2301.10841 — on relational JOB/LSQB, single-thread M1, NOT RDF]**
- **Cost:** medium — replace the materialised-trie `TrieIterator` with a COLT over
  the existing per-permutation column slices; run the Free Join conversion over the
  existing DP/GOO binary plans; keep the hybrid router but let Free Join subsume both
  branches. Implemented in Rust → WASM-portable.
- **Composes:** **additive and low-risk** — falls back to today's plans; the
  persistent index is unchanged (the 6 permutations); GraphflowDB's i-cost becomes
  the cost model that *drives* Free Join plan selection. The build-elimination on the
  left relation is exactly "don't move bytes you won't probe" — on-regime for a
  bandwidth-bound machine.
- **Goal:** both. **The honest risk to measure:** the published wins are vs a
  row-store-ish baseline; the build-elimination is the part most likely to transfer,
  the hashmap-probe locality less so — **must be measured on sparq's own
  triangle/star/chain benches** before committing.

**A5. (Cross-ref, already standing) Roaring dense-predicate tier.** *(bit-level
report P1)* — Keep as planned: build subject/object Roaring bitmaps at load for the
few dozen predicates above the ~5–10 % density crossover; route star-AND /
domain-pruning to bitmap-AND, fall back to sorted-merge otherwise. **[measured:
123–170× on dense AND, 0.5× loss on sparse]**. EmptyHeaded's adaptive uint/bitset
layout (A6) is the WCOJ-side generalisation of this same crossover.

**A6. EmptyHeaded SIMD set-intersection + adaptive uint/bitset layout — for the WCOJ
inner loop.** *(graph-native / WCOJ — Aberger SIGMOD'16)*
- **Win:** a SIMD-vectorized uint-intersection kernel for the leapfrog inner
  intersection, plus an adaptive bitset path for **dense join variables** that
  auto-selects when bitset-AND beats sorted-merge — the same crossover the bit-report
  measured for Roaring. ARCHITECTURE.md already flags `[wcoj-emptyheaded]` as a
  target; the hardware research measured **NEON merge-intersection at 1.6–2.8×**.
  **[measured (NEON); claimed-by-authors (EmptyHeaded ~3 orders on triangle/clique)]**
- **Cost:** small-medium — replace the scalar inner-loop multiway intersection in
  `MultiwayLeapfrog` with a portable `core::simd` kernel + a density-gated bitset
  path. ~a few hundred LOC; additive (falls back).
- **Composes:** serves the WCOJ subsystem sparq differentiates on vs QLever;
  converges with the Roaring P1 dense recommendation (shared density-crossover
  validation). **Do NOT** adopt EmptyHeaded's full persistent trie store (redundant
  with the permutations) or its GHD code-generator (ARCHITECTURE defers JIT).
- **Goal:** native primarily; the bitset path degrades gracefully under WASM's
  128-bit SIMD ceiling (it wins on bytes-moved, not lane width).

### (b) Prototype / spike

Promising, on-regime, but the win is workload-dependent and must be measured per-
relation against the ZSTD/binary-search default before adoption.

**B1. Elias-Fano (then Partitioned-EF) as an optional second column codec — the
single most promising new storage bet.** *(columnar — Vigna; Ottaviano/Venturini
SIGIR'14)*
- **Win:** EF is **alone in this family in supporting near-O(1) `NextGEQ(x)` (seek/
  successor) directly on compressed data** — exactly the primitive sparq's
  merge-gallop, scan↔scan block pruning, and Leapfrog `TrieIterator::seek()` all
  call. Every other high-ratio codec (PForDelta, SIMD-BP128, Stream-VByte, ZSTD) is
  decode-then-search and **forfeits compressed seek**. It is quasi-succinct
  (~2+log₂(u/n) bits/elem), so it could **shrink storage AND accelerate seek-heavy
  joins AND shrink the WASM footprint simultaneously** — the only structure spanning
  four subsystems at once. The fully-developed RDF form (permuted-trie+PEF,
  Pibiri/Venturini ICDE'21) lands **55–83 bits/triple, 2–81× faster than HDT-FoQ/
  TripleBit** on pattern lookups. **[literature; permuted-trie numbers measured]**
- **Cost:** medium — build a per-relation EF over the trailing column's deltas;
  expose `next_geq` to the existing `TrieIterator`/merge `seek`. A/B against ZSTD-3
  on (a) scan throughput, (b) seek-heavy merge/WCOJ, (c) bytes resident, in **native
  AND wasm32+simd128**. PEF is the production form once plain-EF is promising.
- **Composes:** a codec choice *under* a permutation column, beside ZSTD; chosen per
  relation from build-time NDV/clustering stats (ZSTD stays the default). Removes the
  "materialise-then-gallop" step for selective patterns.
- **Goal:** both — and **especially the browser**: EF's wins come from *moving fewer
  bytes* and *scalar select*, not wide SIMD, so it survives WASM's 128-bit ceiling
  better than SIMD-BP128. **Honest risk:** select/rank is some pointer-chasing on a
  bandwidth-bound machine, and streaming-scan ratio may trail ZSTD on dense runs —
  hence measure per-column, keep ZSTD as default.

**B2. FOR / bit-packing + delta pre-transform before ZSTD — the lowest-risk M3
increment.** *(columnar — Lemire/Boytsov; `bitpacking` Rust crate)*
- **Win:** a delta+bit-pack pre-transform is a small, measurable ratio+speed tweak;
  on numeric columns it often improves **both** ZSTD ratio and decode speed. Ready
  Rust crate (`bitpacking`, port of simdcomp) with a scalar WASM fallback.
  **[literature]**
- **Cost:** small. ARCHITECTURE.md already names it (`[gap-delta-pfor]`,
  `[simd-01-simd-bp128-fastpfor]`).
- **Composes:** a pre-transform inside the M3 block path; gives **no compressed
  seek** (that is EF's job) — it is a refinement, not the differentiator.
- **Goal:** both. Spike it in the same A/B as ZSTD vs EF.
- **Note on SIMD-BP128/FastPFOR specifically:** native-only refinement. Its win
  needs **wide lanes**; WASM simd128 is 128-bit (~SSE3, ~4× slower than AVX2 at large
  payloads), so the native advantage largely evaporates in the browser. And on a
  bandwidth-bound machine at ~3.8 G rows/s, a 2.5 G int/s SIMD decode may not be the
  binding constraint vs just moving fewer bytes. Lower priority than EF.

**B3. RDF-3X byte-delta leaf format vs ZSTD blocks — a seek-heavy A/B.**
*(perm-indexes — RDF-3X)*
- **Win:** RDF-3X's header-byte + 1-byte-fast-path per-triple delta gives
  **fine-grained random seek inside a leaf**, where ZSTD forces a full-block decode.
  Worth benchmarking on a seek-heavy (WCOJ leapfrog) workload. **[literature]**
- **Cost:** small-medium spike. Largely subsumed by B1 (EF gives the same
  compressed-seek property with better ratio) — run B3 only if EF disappoints and
  block-granularity decode shows up as a leapfrog bottleneck.
- **Composes:** alternative leaf codec under the permutations.
- **Goal:** native primarily.

**B4. Adaptive variable-elimination-order for WCOJ — a planner-only spike.**
*(succinct survey — URing, "New Compressed Indices", arXiv 2408.00558)*
- **Win:** recompute the WCOJ variable order *mid-join* from sparq's free
  block-metadata cardinalities instead of fixing one global order up front. In the
  succinct setting this gave **first-1000-results 4–13× faster**. The *idea* is
  transplantable to sparq's existing permutation-based LFTJ with **no succinct
  storage** — a separable, answer-safe planner upgrade for skewed/cyclic BGPs.
  **[claimed-by-authors]**
- **Cost:** small-medium — a planner change inside `MultiwayLeapfrog`, harvested from
  the paper's algorithm only.
- **Composes:** strengthens the M2 hybrid WCOJ router; orthogonal to storage.
- **Goal:** both.

**B5. PGM-index A/B — run it to CONFIRM binary search already wins in-cache.**
*(learned — Ferragina/Vinciguerra VLDB'20; Rust `pgm_index`)*
- **Win/expectation:** PGM is the only learned index that is answer-safe (a wrong
  model only widens local search), provably bounded, <1% space, with a mature Rust
  crate **and** a dynamic variant for M5. SOSD shows ~2.8–3.6× on the **search
  primitive** — but **only on large out-of-cache arrays**; the VLDB'25 critiques
  ("Why Learned Indexes Are Sometimes Ineffective", "Binary Search Is All You Need")
  and PGM's own "subpar on 1000 keys" say that on **cache-resident** arrays
  (sparq's few-hundred-K-entry block-metadata array, even at Wikidata scale) binary
  search's minimal footprint wins. **[measured (SOSD); literature (critiques)]**
- **Cost:** small spike behind a feature flag — swap `partition_point` in scan-open
  and Leapfrog `seek()` for model-predict + ε local search; A/B on (a) the small
  block-metadata array (predicted loss) and (b) the full out-of-cache row array
  (possible partial win) with cold and warm caches.
- **Why spike a likely-loser:** to get the **number** that justifies *not* adding a
  learned index to the read core, and to evaluate the **dynamic PGM** as an M5
  mutable-index candidate. Do not ship to the read core unless the out-of-cache A/B
  surprises.
- **Goal:** native (irrelevant to the small WASM bundle goal otherwise).

**B6. (M5 spike) An in-RAM layered update tier: ordered mutable delta over the static
read core.** *(mutable-trees — ART via `congee` + BS-tree; LSM-merge pattern)*
- **Win:** the family's one genuine contribution is the subsystem sparq deferred —
  **updates (M5)**. Keep the read core static (6 ZSTD/flat permutations + WCOJ +
  inline ValueIds); land writes in a small **ordered mutable delta** merged at scan
  time via a unioning cursor (delta ∪ base), periodically compacted back. **[the
  decisive experiment:** measure long-scan/merge-join throughput of each tree vs
  sparq's contiguous flat-array sweep (~3.8 G rows/s) on a real permutation column —
  every source (ALEX's ~1000-key scan crossover, the bandwidth-bound finding, RDF's
  deep shared-prefix keys) predicts the trees lose the long-scan hot path, confining
  them to the small write-absorbing delta.**]**
- **Cost:** medium-large, but **M5-gated** — prototype TWO substrates: **(i) ART via
  `congee`** (8-byte key/permutation, SIMD nodes, OLC concurrency, `range()` scans —
  lowest-effort, production-proven, maps onto the existing TrieIterator/LFTJ cursor);
  **(ii) a BS-tree-style** gapped + SIMD + (CBS-)compressed contiguous-leaf B-tree —
  the best read/scan-preserving mutable option (beats ART on reads, near-flat-array
  density) but needs a Rust reimplementation (2025 research, AVX-512-tuned, no crate;
  port to NEON/AVX2/WASM scalar).
- **Composes:** the overlay pattern is RDFox-style (lock-free triple-table + hash
  index) but **ordered** — RDFox's own hash store is unordered and can't serve
  sorted scans, so sparq must layer an *ordered* delta. Roaring is the better delta
  *set* substrate (bit-level P2). Dynamic-PGM and ALEX are also candidates here.
- **Goal:** native (M5+); a mutable WASM build is a later concern.

### (c) Keep on the shelf (research-grade tiers, well-understood placement)

Adopt only when a specific binding constraint (extreme scale, browser ceiling) makes
the trade pay — not before.

**S1. The Ring / CompactLTJ / C-Ring — the WCOJ compression endgame.** *(succinct)*
- **What:** a BWT/wavelet-tree self-index that gives sparq's **exact LFTJ
  seek/next/open contract in ~1 permutation's space instead of six** (the Ring:
  16.41 bpt, ~8% over raw integer data, 4–6× smaller than Jena/Virtuoso/Blazegraph,
  2–6× faster than those *non-WCOJ* stores at billion-edge Wikidata). CompactLTJ does
  the six-order LTJ in ~3.3× raw-triple size and **claims update tolerance** (rare for
  succinct). C-Ring halves the space at ~3× the latency. **[claimed-by-authors /
  measured (CompactLTJ space)]**
- **Why shelved:** rank/select is **O(log σ) dependent pointer-chasing per step** —
  the *worst* fit for sparq's measured bandwidth/latency-bound profile, so it
  **regresses per-op latency on the small/dense cache-resident data where sparq
  currently wins** (identical verdict to the bit-level report's M5+ parking). It is
  bulk-load-only (the Ring) and the storage win erodes to ~2× for the multi-copy
  BGP-complete variants (RDFCSA/URing).
- **When to adopt:** when sparq is **memory-bound at billion-triple scale** (6 raw
  32-bit permutations of 15B-triple Wikidata exceed a terabyte) **or** under the WASM
  ceiling — there, collapsing 6 → ~1 is the difference between fitting and not.
  Build it behind the **same `TrieIterator` (seek/next/open/up) contract** so the
  planner routes to it transparently. CompactLTJ's update claim makes it the more
  interesting of the two for an M5 mutable+compact future. Goal: native-billion-scale
  + browser.

**S2. HDT (Header-Dictionary-Triples) + HDT-FoQ — the cold/exchange + browser-snapshot
tier.** *(succinct)*
- **What:** the battle-tested, **Rust-available (`hdt-rs`)** compact RDF exchange
  format; SO/S/O/P front-coded dictionary ≈ what sparq already plans (dictionary
  ideas cross-pollinate **now**). Bitmap-Triples is a succinct SPO adjacency; HDT-FoQ
  adds PSO+OPS indexes. **[literature]**
- **Why shelved for the core:** **no WCOJ, no join optimality, read-only** — a
  scan/lookup index, not a join index.
- **When to adopt:** as the M5+ cold segment (already parked as `[hdt-bitmaptriples]`)
  and — **the credible browser play** — shipping a prebuilt HDT-FoQ blob is a "fast
  queryable snapshot under 2–4 GB" with no decompression. Goal: browser.

**S3. TripleBit's 2-order predicate-partitioned layout — a memory-constrained build
shape.** *(perm-indexes)*
- **What:** SO+OS per predicate at ~5 B/triple answers most real queries; its
  ID-Chunk min/max-per-chunk skip metadata **validates sparq's planned per-block
  skip array at 64 KB granularity** and shows the fitted-line-segment trick to shrink
  the chunk-location index to bytes. **[claimed-by-authors]**
- **Why shelved:** for the **full-fat in-RAM build, 6 orders give sort-free merges +
  LFTJ orders** that 2 predicate-partitioned orders can't. Do **not** adopt the
  literal bit-matrix (RDF sparsity kills it — bit-level report).
- **When to adopt:** a memory-constrained / WASM build could ship the **3-cover
  {SPO,POS,OSP}** (already the planned fallback) or a **4-order WCOJ-tuned set
  (à la MillenniumDB)** instead of 6, trading variable-predicate/merge coverage for
  ~2–3× less RAM. Empirically licensed by Virtuoso (2), GraphDB (PSO+POS), TripleBit
  (2), COTTAS (1) all shipping <6. Goal: browser + memory-constrained native.

**S4. Factorised f-/d-representations + Kuzu factorized vectors + ASP-Join — the
m-n / streaming wedge.** *(WCOJ-and-factorized)*
- **What:** keep intermediates *factorised* instead of as flat solution tables —
  size **O(\|D\|^fhtw)** vs O(\|D\|^ρ\*) flat, with **constant-delay enumeration** and
  **free COUNT/aggregate over factorised stars**. Kuzu proves it ships in a real
  columnar graph engine with CSR-like adjacency (the closest sibling to sparq) and
  gives a copyable "factorized vector group" + ASP-Join/sip blueprint.
  **[literature (theory); claimed-by-authors (Kuzu)]**
- **Why shelved (M4+, not now):** it is a **deep change to the vectorized execution
  model** (a new intermediate type + a factorisation-aware planner over f-trees), and
  the win only materialises when results actually factorise (cyclic/Boolean queries
  may not).
- **When to adopt:** it targets two real sparq weaknesses — **high-fan-out
  intermediates** (QLever's OOM regime; ARCHITECTURE already lists factorised/
  vector-group output at M4+) and **large result serialisation** (emit factorised +
  enumerate). Adopt the Kuzu blueprint when M4 tackles m-n/streaming intermediates.
  Goal: native (the OOM fix) + result-export.

**S5. Hexastore shared-third-level lists — a cheap dedup.** *(perm-indexes)* — twin
permutations sharing a leading column (e.g. SPO/SOP) could share physical
object-run storage to claw back ~1/6 of the 6× redundancy without changing the scan
path. Lower priority than M3 compression, but a clean low-risk memory win. **[claimed-
by-authors]**

**S6. Virtuoso compile-time index sampling — a planner estimator.** *(perm-indexes,
already `[plan-12]`)* — probe the permutations with the query's constants at plan
time. Borrow for the planner; the relational/row-store quad model itself is heavier
than sparq's flat arrays and is not a storage change. **[claimed-by-authors]**

**S7. GNCE-style learned cardinality — only after classical estimators prove
insufficient.** *(learned, already bit-level P4)* — answer-safe (planner-only),
beats characteristic-sets on **path queries by orders of magnitude in q-error**
(GNCE 1.2–4.2 vs CSET 1e29–1e47). But it needs offline training + whole-dictionary
embeddings (heavy for arbitrary user data, degrades on unseen entities, adds plan-
time latency + a model dep fighting the small-bundle goal). **Exhaust the planned
classical upgrades first** (characteristic sets + compile-time sampling + SumRDF);
reach for the GNN only if WDBench paths still misplan. **[measured (q-error)]**

### (d) Reject (wrong regime, will regress or fight the core)

**R1. Database cracking / adaptive indexing.** Never catches a pre-built sorted index
(**~40% slower after 1000 queries**), unsolved on concurrency/robustness, can't crack
a ZSTD block in place, and targets the "no time to build an index" scenario that is
the *opposite* of sparq's bulk-built read-optimised design (build throughput is a
design target sparq intends to **win**). **[literature]** Fit 1.

**R2. Multi-dimensional learned indexes (Flood/Tsunami/LISA).** A learned 3-D grid
layout **destroys the interesting sort orders** that make sparq's merge/WCOJ work;
1-/2-bound prefix patterns (the common case) are already served optimally by a sorted
permutation with zero model; Flood/Tsunami need a representative workload and don't
support updates. Fights the core. **[claimed-by-authors]** Fit 1.

**R3. Neo4j-style index-free adjacency (pointer-chasing record store).** Trades
cache-friendly sorted scans for **random pointer dereferences** — directly fights the
bandwidth-bound finding; ~34 B/edge vs sparq's compressed columns; no two-bound range
scans or variable predicates without secondary indexes; no WCOJ. Anti-pattern.
**[literature]** Fit 1.

**R4. Qdag (succinct quadtree-DAG).** Smallest space, but **"hundreds of times
slower" than the Ring** (which is already a latency regression vs sparq's flat
permutations). Wrong end of every trade for the in-RAM regime. **[claimed-by-authors]**
Fit 1.

**R5. GraphBLAS/MAGiQ SpMV-join and all GPU CSR (Gunrock/Wukong+G/TripleID-Q/
gStore-GPU).** MAGiQ is a **scale-out + hardware-portability** architecture (512B
triples across CPU/GPU/Cray), **not a single-node latency win** — on one box a tuned
merge/WCOJ engine beats the SpGEMM formulation, and one-matrix-per-predicate **can't
handle variable predicates**. GPU CSR is gated by the **measured 4–12× host→device
transfer tax + 16–64 M-row crossover** (prior gpu-and-cloud.md spike) and is wrong
for latency-sensitive SPARQL with large variable results that must leave the device;
VRAM caps rule out sparq's target datasets; gStore-GPU's signature-graph subgraph-
isomorphism is a filter-then-verify **heuristic with no AGM guarantee** (sparq's whole
point). The one transplantable idea — SIMD/warp set-intersection — is harvested on
**CPU via EmptyHeaded (A6)** for sparq's actual regime. **[literature; transfer-tax
measured]** Fit 1–2.

**R6. RocksDB/LSM read core (Oxigraph's substrate).** The LSM read-amplification tax
is **exactly why Oxigraph is the slowest engine** in the scorecard (67 GB / 93 s vs
QLever 8 GB / 0.7 s on DBLP-390M). Adopting it would regress the in-RAM scan/merge
core and bloat memory. The *layered-merge idea* survives only as the lightweight
in-RAM M5 overlay (B6); the on-disk KV substrate is rejected for the read core.
**[measured]** Fit 2 (updates-only, and B6 does it better in-RAM).

**R7. BSI (bit-sliced index) for wide numeric FILTER, tensor/neural units for exact
joins, KG embeddings for exact BGP** — all settled by the bit-level report:
**[measured]** BSI is 2–3× *slower* than the contiguous scalar scan at sparq's 30-bit
inline-int width and sparq's shipped sorted range-pruning already beats QLever 1.24×;
tensor cores are dense-float (wrong primitive for exact bitwise set ops); embeddings
are approximate (return probable, not exact, triples). Cross-referenced, not re-done.

---

## 3. The 2–3 highest-leverage structures to try next, and what to measure

Ranked by (expected win × confidence × on-regime fit ÷ cost), distinguishing the two
goals.

### #1 — Per-block Bloom filters (A2). *Prototype this first.*
**Why highest-leverage:** smallest cost (a few hundred LOC + one bitset/block),
purely additive, answer-safe, and it attacks sparq's **own measured weak spot** — the
full-scan materialisation on high-NDV columns (q02 0.23×, q04 0.29× vs QLever) — that
the min/max zone map structurally cannot fix (overlapping ranges on high-cardinality
subject/object columns). It is independent of the join engine and ships on both
native and WASM. It is the cheapest path to closing part of the M3 gap *before* the
full block-compression work lands. **[literature/claimed-by-authors — COTTAS]**

**Measure:** on a real Wikidata permutation column (high-NDV subject/object) —
(i) **block-skip rate**: fraction of blocks the Bloom filter skips that the min/max
zone map does *not*, for equality-bound constants; (ii) **end-to-end latency** on
q02/q04-style scan+join queries with vs without the filter; (iii) **false-positive
rate vs bits/key** (tune to ~1% at ~10 bits/key); (iv) **build-time + memory
overhead** in the loader. Success = measurable block-skip on bound-but-out-of-cluster
constants with <2% index-size growth.

### #2 — Elias-Fano compressed-seek column codec (B1). *The single most promising new bet.*
**Why:** it is the **only structure that can improve storage AND scan AND join AND
browser-footprint at once**, because `NextGEQ` is the exact `seek()` primitive sparq's
merge-gallop, scan↔scan pruning, and Leapfrog `TrieIterator` already use — closing
ZSTD's one structural gap (no compressed seek) **without** abandoning the column
store. And it is the *right browser bet*: its wins are bytes-moved + scalar-select,
which survive WASM's 128-bit ceiling where SIMD-BP128 does not. The RDF endgame
(permuted-trie+PEF) is already measured at 55–83 bpt and 2–81× over HDT-FoQ/TripleBit.
**[literature; permuted-trie measured]**

**Measure** (per-relation, native AND wasm32+simd128, A/B vs ZSTD-3 default and vs
B2 FOR+bitpack): (i) **bytes resident** per relation (EF/PEF vs ZSTD vs FOR) keyed on
NDV/clustering; (ii) **streaming-scan throughput** (the likely EF weakness — must not
trail ZSTD badly on dense runs); (iii) **seek-heavy throughput** on a WCOJ-leapfrog /
selective-pattern workload (the EF win — does compressed `NextGEQ` beat
decode-block-then-gallop?); (iv) **end-to-end chunk-fill throughput** feeding the
Arrow-like execution layer (judge the codec by chunk-fill, not standalone decode
microbench). Decision rule: **codec-per-column, chosen at build time from per-relation
stats; ZSTD stays the default; EF/PEF adopted only where the A/B wins.** **Honest
risk to confirm or kill:** does rank/select pointer-chasing cost more than the
bytes-moved saving on a bandwidth-bound machine?

### #3 — COLT / Free Join (A4). *Wins the join subsystem.*
**Why:** it is the literal answer to the brief's WCOJ question — eliminates the
per-query trie-materialisation that is sparq's known WCOJ bottleneck, unifies
merge/hash/Leapfrog over the **persistent index sparq already has**, is implemented in
Rust (WASM-portable), and is additive (falls back to today's plans). It is the part of
the WCOJ-and-factorized family that beats the current design and is adoptable *now*,
unlike factorised reps (deep M4+ rewrite) or succinct LTJ (M5+ space tier). **[claimed-
by-authors]**

**Measure** (on sparq's **own** triangle/star/chain benches — the published numbers
are relational, not RDF): (i) **build-elimination win** — confirm the left relation
builds no auxiliary structure and that this is where the gain comes from (the part
most likely to transfer to a bandwidth-bound engine); (ii) **COLT vs current
materialised-trie LFTJ** on cyclic (triangle) and acyclic (star) BGPs at 10M and at
scale; (iii) **hashmap-probe locality** — the part *least* likely to transfer; does it
regress vs flat-array merge on low-skew acyclic queries (keep the hybrid router as the
safety net)? Success = ≥ parity on acyclic + a clear win on cyclic/high-fan-out,
with no regression once the GraphflowDB i-cost router gates it.

**Goal split.** #1 and #3 serve **both** goals. #2 serves both but is *disproportionately*
the **browser** lever (compressed-seek under a small bundle + the 2–4 GB ceiling). The
**native-billion-scale** goal additionally leans on S1 (the Ring/CompactLTJ collapsing
6→1 permutation when memory-bound) and S4 (factorised intermediates for the OOM
regime); the **browser** goal additionally leans on S2 (a prebuilt HDT-FoQ snapshot)
and S3 (the 3-/4-order memory-constrained build).

---

## 4. What does NOT fit the in-memory, sparse, WCOJ, correctness-first regime

Made explicit so the engine isn't pulled off-design by a structure that benchmarks
well in *someone else's* regime.

1. **Anything that pointer-chases on the hot path.** rank/select succinct indexes
   (Ring/CompactLTJ/RDFCSA/wavelet trees), index-free adjacency (Neo4j), and tries
   over long scans (ART/HOT/Masstree for the *read core*) all do **O(log σ) or
   per-node dependent memory accesses** where sparq's flat sorted array does **one
   binary-search probe + a linear prefetcher-friendly sweep**. On a machine **measured
   to be bandwidth/latency-bound**, that is the worst possible trade for SCAN and
   merge-JOIN. These belong to *space* tiers (succinct, when memory-bound) or the
   *write* tier (trees), never the read hot path. **[measured: bandwidth-bound;
   literature: succinct ~1 order slower than pointer-based on cache-resident data]**

2. **Anything that destroys sorted order.** Multi-dimensional learned grids (Flood/
   Tsunami), hash-id dictionaries (Oxigraph's 128-bit hashes), and any unordered store
   forfeit the **sort-free merge joins, LFTJ seek, and block min/max range-pruning**
   that are the entire reason for six monotonic-id permutations. sparq's monotonic
   lexicographic ids are load-bearing for EF/delta/FOR/range-pruning *and* WCOJ.

3. **Anything whose win requires bytes that aren't there (RDF sparsity).** The literal
   bit-matrix (BitMat dense form) and dense-bitset adjacency on sparse columns:
   uncompressed S×O is |S|·|O|/8 bytes; once run-compressed it *is* Roaring, and
   **below ~5–10% density sorted-merge beats Roaring (measured 0.5×)**. Dense bitmaps
   are a *targeted tier* for the few dozen dense predicates (P1/A5/A6), not a core
   layout.

4. **Anything that is a scale-out / portability / analytics architecture, not a
   single-node latency engine.** GraphBLAS/MAGiQ SpMV (distributed scale-out, can't do
   variable predicates), GPU CSR (4–12× transfer tax, VRAM cap, wrong for large
   variable results leaving the device), and property-graph traversal engines (Neo4j/
   Kùzu/CSR tuned for n-hop BFS/PageRank, weak on the variable-predicate + two-bound-
   constant BGP patterns that dominate SPARQL). sparq's target is **one €2.5k box,
   latency-sensitive BGP serving** — a different problem. **[measured transfer tax;
   literature for MAGiQ scale-out positioning]**

5. **Anything that trades a built index for per-query write amplification.** Database
   cracking and learned-index online adaptation: sparq **bulk-builds once, queries
   millions of times read-only** — the exact workload where a full pre-built sorted
   index dominates (cracking is ~40% slower after 1000 queries and unsolved on
   concurrency, which fights morsel-parallel reads). **[literature]**

6. **Anything approximate where correctness is the differentiator.** KG embeddings
   (TransE/RotatE) for BGP evaluation return *probable* triples (including ones not in
   the graph) — categorically incompatible with sparq's no-row-cap, exact-term-equality,
   correct-path-duplicate #Diverge gate. Embeddings are confined to the **planner's
   cardinality estimate** (answer-safe, GNCE/S7), never the execution path. **[literature]**

7. **Anything whose speed comes from wide SIMD, for the browser build.** SIMD-BP128/
   FastPFOR's win needs AVX2/AVX-512 lanes; WASM simd128 is 128-bit (~SSE3, ~4× slower
   at large payloads). For the browser, prefer codecs that win on **bytes moved +
   scalar select** (Elias-Fano, FOR/bit-packing, Stream-VByte's scalar path) over
   lane-width-dependent ones. **[literature]**

**The unifying principle.** sparq's core is the *convergent answer* of the RDF-engine
literature for its regime; the right structures to add are **(a) the compression and
seek codecs that move fewer bytes** (M3 + Bloom + EF), **(b) the join structure that
stops building bytes it won't probe** (COLT), and **(c) per-subsystem specialists**
behind the same contracts — succinct (`TrieIterator`) for the space/browser tier,
ordered-mutable for the write tier, learned for the planner. Everything that
pointer-chases the hot path, destroys sort order, assumes density that RDF lacks,
targets scale-out/analytics, cracks instead of builds, or approximates the answer is
correctly **out of regime**.

---

## 5. Sources (URLs preserved from the surveys; tags as cited)

**Perm-indexes.** RDF-3X <https://cs.uwaterloo.ca/~gweddell/cs848/papers/RDF-3X.pdf>,
<https://link.springer.com/article/10.1007/s00778-009-0165-y> · Hexastore
<http://www.vldb.org/pvldb/vol1/1453965.pdf> · TripleBit
<http://www.vldb.org/pvldb/vol6/p517-yuan.pdf> · Virtuoso
<https://vos.openlinksw.com/owiki/wiki/VOS/VOSArticleWebScaleRDF> · RDFox
<https://www.cs.ox.ac.uk/boris.motik/pubs/npmhwb15RDFox-scalable.pdf> · QLever
<https://ad-publications.cs.uni-freiburg.de/CIKM_qlever_BB_2017.pdf>,
<https://github.com/ad-freiburg/qlever/wiki/QLever-performance-evaluation-and-comparison-to-other-SPARQL-engines>
· COTTAS <https://sferrada.com/publication/2025-iswc-arenas-guerrero-cottas/2025-iswc-arenas-guerrero-cottas.pdf>
· MillenniumDB <https://aidanhogan.com/docs/millenniumdb.pdf> · GraphDB
<https://graphdb.ontotext.com/documentation/11.3/storage.html> · Stardog
<https://docs.stardog.com/operating-stardog/database-administration/managing-databases>

**Succinct self-indexes.** The Ring <https://dl.acm.org/doi/10.1145/3644824>,
<https://aidanhogan.com/docs/ring-graph-wco.pdf> · C-Ring / Qdag
<https://aidanhogan.com/docs/wco-ring.pdf>, <https://arxiv.org/pdf/1908.01812> ·
RDFCSA <https://users.dcc.uchile.cl/~gnavarro/ps/spire15.pdf>,
<https://link.springer.com/article/10.1007/s11227-022-04890-w> · URing / adaptive-VEO
<https://arxiv.org/abs/2408.00558> · HDT
<https://www.sciencedirect.com/science/article/abs/pii/S1570826813000036>,
<https://github.com/KonradHoeffner/hdt> · CompactLTJ
<https://users.dcc.uchile.cl/~gnavarro/ps/vldbj25.pdf>,
<https://users.dcc.uchile.cl/~gnavarro/ps/grades24.pdf> · wavelet trees
<https://users.dcc.uchile.cl/~gnavarro/ps/cpm12.pdf>

**Ordered mutable trees.** ART <https://www.db.in.tum.de/~leis/papers/ART.pdf>,
congee <https://github.com/XiangpengHao/congee> · BS-tree
<https://arxiv.org/html/2505.01180v1> · MillenniumDB B+tree
<https://arxiv.org/pdf/2111.01540> · FAST
<http://kaldewey.com/pubs/FAST__SIGMOD10.pdf> · Masstree
<https://pdos.csail.mit.edu/papers/masstree:eurosys12.pdf> · HOT
<https://15721.courses.cs.cmu.edu/spring2019/papers/08-oltpindexes2/p521-binna.pdf> ·
ALEX <https://arxiv.org/pdf/1905.08898> · B-ε / LSM
<https://www.usenix.org/publications/login/oct15/bender>, Oxigraph
<https://github.com/oxigraph/oxigraph/wiki/Architecture>

**WCOJ + factorized.** COLT/Free Join <https://arxiv.org/pdf/2301.10841>,
<https://dl.acm.org/doi/10.1145/3589295>, <https://arxiv.org/html/2505.19918v1> ·
factorised f/d-reps <http://www.cs.ox.ac.uk/dan.olteanu/papers/oz-icdt12.pdf>,
<https://fdbresearch.github.io/principles.html> · Kuzu
<https://www.cidrdb.org/cidr2023/papers/p48-jin.pdf> · GraphflowDB
<https://www.vldb.org/pvldb/vol12/p1692-mhedhbi.pdf> · EmptyHeaded
<https://ppl.stanford.edu/papers/emptyheaded.pdf> · CSR/CSR++
<https://labs.oracle.com/pls/apex/f?p=LABS:0:::APPLICATION_PROCESS=GETDOC_INLINE:::DOC_ID:1580>

**Columnar codecs.** Dictionary / C-Store
<http://www.cs.umd.edu/~abadi/papers/abadisigmod06.pdf> · Arrow
<https://arrow.apache.org/docs/format/Columnar.html> · Elias-Fano
<https://vigna.di.unimi.it/ftp/papers/QuasiSuccinctIndices.pdf>,
<http://groups.di.unipi.it/~ottavian/files/elias_fano_sigir14.pdf> · permuted-trie+PEF
<https://pages.di.unipi.it/rossano/assets/pdf/papers/ICDE21.pdf>,
<https://github.com/jermp/rdf_indexes> · FOR/bit-pack/PForDelta/SIMD-BP128
<https://arxiv.org/pdf/1209.2137>, <https://lemire.me/en/publication/spe2015simd/>,
<https://github.com/quickwit-oss/bitpacking> · Stream-VByte
<https://arxiv.org/abs/1709.08990>

**Learned + adaptive.** PGM <https://www.vldb.org/pvldb/vol13/p1162-ferragina.pdf>,
<https://github.com/gvinciguerra/PGM-index> · RMI <https://arxiv.org/pdf/1712.01208> ·
RadixSpline <https://ar5iv.labs.arxiv.org/html/2004.14541> · "learned indexes
sometimes ineffective" <https://www.vldb.org/pvldb/vol18/p2886-liu.pdf> · GNCE
<https://arxiv.org/html/2303.01140v2>, <https://dl.acm.org/doi/10.1145/3639299> ·
cracking <http://www.vldb.org/pvldb/vol7/p97-schuhknecht.pdf> · multi-dim
<https://arxiv.org/pdf/2006.13282>

**Graph-native + GPU.** CSR survey <https://arxiv.org/pdf/2102.13027> · GraphBLAS/
MAGiQ <https://arxiv.org/pdf/1807.07691>, <http://www.vldb.org/pvldb/vol11/p1978-jamour.pdf>
· Gunrock <https://gunrock.github.io/gunrock/>, <https://arxiv.org/pdf/1501.05387> ·
TripleID-Q <https://arxiv.org/pdf/1807.01409> · Wukong+G
<https://www.usenix.org/system/files/conference/atc18/atc18-wang-siyuan.pdf> ·
gStore-GPU <https://dl.acm.org/doi/10.14778/2002974.2002976>

**Internal cross-references.** `research/ARCHITECTURE.md` (§3.2 permutations + M3
plan, §3.4 planner + WCOJ, §5 roadmap, §6 WASM), `research/bit-level-encoding.md`
(Roaring dense-tier P1/P2, Ring M5+ P3, BSI/tensor/embedding rejections, the measured
density crossover + bandwidth-bound thesis), `research/BENCHMARKS.md` (the measured
QLever gaps: 10M compute 0.27× geomean / 3.7×, q02 0.23×, q04 0.29×, q06 range-prune
1.24× win, q10 sort-merge 1.23× win), `research/hardware/` (bandwidth-bound finding,
NEON merge-intersection 1.6–2.8×, GPU 4–12× transfer tax / 16–64M crossover).
