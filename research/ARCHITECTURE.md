# sparq — Architecture Blueprint

A from-scratch RDF + SPARQL engine in Rust, designed to match or beat QLever,
RDFox, Virtuoso, GraphDB, Blazegraph and MillenniumDB on the standard
benchmarks, on a single commodity machine.

This document is the engineering blueprint: the decided architecture, the
numbers to beat, implementation-grade component designs, the optimality theory,
the phased roadmap, and the WASM end-state. Every major decision cites the
research findings it derives from (IDs in brackets, e.g. `[qlever-six-permutations]`).

---

## 1. Executive summary — the chosen architecture

sparq is a **read-optimised, dictionary-encoded, sorted-permutation triplestore
with a vectorized, merge-join-centric query engine and a hybrid (binary +
worst-case-optimal) join planner**, built to run a full Wikidata (≈20B triples)
on one ~€2.5k machine. The shape is deliberately QLever-like — because QLever is
the engine to beat across every benchmark — with three targeted improvements
where QLever is known to be weak: **memory-bounded streaming** (QLever OOMs on
OPTIONAL/DISTINCT at full scale), **worst-case-optimal joins** for cyclic BGPs
(QLever has no WCOJ; MillenniumDB beats binary plans ~1.6× there), and
**stricter RDF-term correctness** (every competitor has known answer divergences).

The decisions, and why:

- **Storage / index: six sorted ID permutations** (PSO, POS, SPO, SOP, OPS, OSP),
  each a sequence of column-major, ZSTD-compressed blocks with rich per-block
  skip metadata (`firstTriple`/`lastTriple`, per-column offset+size, `numRows`).
  Six permutations make *every* triple pattern a single contiguous sorted scan
  and *every* two-pattern join a sort-free merge join; they also give Leapfrog
  Triejoin the sorted orders it needs on any variable. Despite 6× redundancy the
  index is *smaller* than competitors because of block compression + dictionary
  encoding (QLever: 8 GB on DBLP-390M vs Oxigraph's 67 GB).
  `[qlever-six-permutations]`, `[qlever-compressed-blocks]`,
  `[perm-coverage-3of6]`, `[rdf3x-storage-model]`, `[wcoj-index-orders]`,
  `[BENCH-DBLP-500M]`.

- **Scans = binary search over block metadata.** A triple pattern resolves in
  O(log #blocks) against the in-RAM sorted `[firstTriple,lastTriple]` array; only
  overlapping blocks are decompressed, only the needed columns. The same metadata
  yields near-free cardinality estimates (sum `numRows`) with zero data reads.
  `[qlever-block-binary-search-scan]`.

- **Dictionary: tagged 64-bit `ValueId` + sorted, front-coded/FSST string vocab.**
  Numbers, booleans, dates and geo-points are packed *inline* in the id (datatype
  tag in the high bits, value in the low bits) so FILTER ranges and arithmetic run
  in id space with zero dictionary lookups, and never consume vocab space. Only
  IRIs/blank-nodes/large literals get a `VocabIndex` into a sorted dictionary
  compressed with Plain Front-Coding (and later FSST). ID order matches value
  order so range FILTERs map to block-metadata pruning.
  `[qlever-encoded-value-ids]`, `[oxigraph-inline-ids]`, `[stor-01-dictionary-encoding-id-layout]`,
  `[DICT-01]`, `[qlever-fsst-squared-vocab]`, `[dictionary-pfc-frontcoding]`, `[DICT-03]`.

- **Join engine: merge join by default, gallop when one side ≫ the other, hash
  join when inputs are not co-sorted, Leapfrog Triejoin (WCOJ) for cyclic/skewed
  multiway joins.** Because permutations are pre-sorted, the common case is a
  sort-free merge; scan↔scan joins prune blocks against each other before
  decompressing. WCOJ is spliced in only where the planner predicts intermediate
  blow-up (cyclic subqueries or heavy same-variable skew), per the Free Join /
  GraphflowDB hybrid result that pure-WCOJ regresses on acyclic low-skew queries.
  `[qlever-merge-gallop-hash-join]`, `[qlever-lazy-block-join]`,
  `[wcoj-leapfrog-triejoin]`, `[wcoj-when-which]`, `[wcoj-hybrid-optimizer]`,
  `[join-01-lockfree-hash-join-tagged]`, `[join-02-merge-join-sorted-intersection]`.

- **Planner: spargebra → own physical IR → filter pushdown → cardinality-cost greedy
  (GOO) join ordering by DEFAULT.** An OPT-IN `dp-planner` feature (bead `sq-iywur`)
  adds a connected-subgraph-complement-pair (DPccp) DP that finds a `Cout`-optimal
  *bushy* tree, falling back to GOO above a connected-subgraph-count budget (and on
  disconnected BGPs); it is order-only (result-identical to GOO) and OFF by default.
  Hypergraph DPhyp and interesting-orders-in-the-DP-table (below) remain future work.
  Connected components are planned independently and combined by cartesian product.
  Cardinality comes from block-metadata counts (free, exact for small
  scans), per-relation multiplicities (stored at build), and **characteristic
  sets** for star subqueries — the single highest-value RDF estimator.
  `[plan-01]`, `[plan-02]`, `[qlever-dp-goo-planner]`, `[plan-04]`, `[plan-13]`,
  `[plan-05]`, `[plan-03]`, `[qlever-cardinality-multiplicity]`, `[qlever-prefilter-pushdown]`.

- **Execution: vectorized, morsel-driven, lazy/streaming.** Operators pass
  column-major `DataChunk`s of ~2048 u64 ValueIds with selection vectors; the
  engine runs entirely on ids and materialises strings only at projection. Each
  operator's output is either a materialised table or a lazy stream of chunks, so
  LIMIT/OFFSET push into scans and short-circuit, and large intermediates never
  fully buffer (the fix for QLever's OOM). Parallelism is morsel work-stealing
  over a pinned thread pool; the hash join is lock-free with tagged-pointer probe
  + software prefetch. SIMD effort goes to decompression and the hash probe, not
  selection scans (only ~10% end-to-end).
  `[vec-01-columnar-vector-batches]`, `[vec-02-selection-vector-vs-compaction]`,
  `[qlever-lazy-results]`, `[morsel-01-morsel-driven-parallelism]`,
  `[decide-vec-vs-compile]`, `[simd-01-simd-bp128-fastpfor]`.

**Why this shape wins.** On the published DBLP-390M head-to-head, QLever loads in
231 s to an 8 GB index and answers in 0.7 s avg; Oxigraph (the existing Rust
engine, RocksDB row storage) is the *slowest* at 93 s avg / 67 GB index
`[BENCH-DBLP-500M]`. The lesson is explicit: *Rust alone is not enough — the
storage substrate is the differentiator.* Compressed dictionary-encoded sorted
permutations + merge joins are the proven substrate; we adopt it and add WCOJ +
streaming + correctness as the wedge over QLever itself.

**What we deliberately do *not* build first.** A JIT (Cranelift) — the workload
is memory-latency bound, where vectorization already wins and compilation mainly
helps cache-resident compute `[decide-vec-vs-compile]`. The RDFox linked-list +
lock-free-insert mutable store — only needed if we target datalog
materialisation/reasoning (LUBM); deferred behind a separate write/reasoning tier
`[rdfox-triple-table]`, `[INDEX-02]`. The Ring/CSA single-permutation index and
k²-trees — research-grade compression tiers for later `[ring-csa-single-perm]`,
`[k2tree-triples]`.

---

## 2. Target scorecard — numbers to beat

Hardware baseline for all comparisons: single commodity machine, AMD Ryzen 9
7950X/9950X (16 cores), 128–192 GB RAM, NVMe SSD (~€2,500)
`[qlever-hardware-design-philosophy]`, `[BENCH-PROTOCOL]`. Measurement protocol:
cold-start via `drop_caches`, time after engine warm, report mean + median +
quartiles + max + #Error/#Timeout/#Diverge, three result modes (full / COUNT /
DISTINCT), no silent row caps, LIMIT 10M, 60 s and 600 s timeouts `[BENCH-PROTOCOL]`.

| Benchmark | Dataset / size | Metric | Best published number to beat | Source |
|---|---|---|---|---|
| **DBLP-390M** (first local target) | 390M triples, 1.8 GB gz | load time / index size / avg query | **QLever 231 s / 8 GB / 0.7 s.** Oxigraph (Rust) 640 s / 67 GB / 93 s — the Rust bar to crush | `[BENCH-DBLP-500M]` |
| **WDQS-298** (primary) | full Wikidata ≈20B | mean / median (unadj. ms) | **QLever mean 4583, median 103, q3 559, max 301655, 3 err, 0 TO, 12 diverge.** Adjusted: QLever 2290 vs MillenniumDB 15482 | `[BENCH-WDQS-298]` |
| WDBench (categories) | truthy 1.257B → full 20B | avg/median per category, #TO | QLever: single-BGP 769 ms, multi-BGP 4630, paths 4276, **optionals 17018 (127 errs, OOM — exploitable)**. Virtuoso paths avg 4.71 s (but buggy) | `[BENCH-WDBENCH]` |
| WGPB (pure BGP / WCOJ) | full Wikidata | mean ms | **QLever 12 ms**, MillenniumDB 21, Blazegraph 644, Virtuoso 978 | `[BENCH-WGPB-SCHOLIA]` |
| Scholia (complex real queries) | full Wikidata | adjusted ms | QLever fastest (~2/3 Blazegraph); MillenniumDB ~2.7× QLever | `[BENCH-WGPB-SCHOLIA]` |
| Sparqloscope (feature coverage) | DBLP 500M + Wikidata 8B | per-feature ms, #failed, serialization cost | Per-feature vs QLever in the public web app; target zero failures (MINUS/EXISTS/aggregates/functions) | `[BENCH-SPARQLOSCOPE]` |
| WatDiv (scaling/plan quality) | SF 10M/100M/1B, L/S/F/C templates | per-template latency, throughput | Controlled estimator/plan-quality test; star templates reward WCOJ | `[BENCH-WATDIV]` |
| SP2Bench (operator coverage) | 250k–25M synthetic | per-query at scale | Optimizer unit-tests (Q4 long join, Q6/Q7 NOT-EXISTS anti-join, Q5 early DISTINCT) | `[BENCH-SP2BENCH]` |
| BSBM (throughput/concurrency) | 100M | QMpH | **GraphDB 11.2: Explore 1-thread 62,170; 4-thread 217,509 QMpH** | `[BENCH-BSBM]` |
| LUBM (reasoning — optional tier) | LUBM-8000 ≈1B → 1T | materialization rate, bytes/triple, query | **RDFox 6.1M triples/s reasoning, 1M/s import, 36.9 B/triple, 0.49 s incremental.** Virtuoso 1B load ~30–37 Kt/s | `[BENCH-LUBM]`, `[rdfox-parallel-materialisation]` |
| UniProt / OSM (ultimate scale) | 100B+ / geospatial | load throughput, index size; spatial joins | QLever public endpoints; >1T triples on one PC | `[BENCH-UNIPROT-OSM]` |

**Correctness gate (#Diverge), to weaponise.** Every competitor diverges:
Virtuoso silently caps at 1,048,576 rows and emits duplicate path results;
QLever wrongly unifies `'1'^^xsd:integer` and `'01'^^xsd:integer`; MillenniumDB
drops path-alternative duplicates `[BENCH-WDQS-298]`, `[BENCH-PROTOCOL]`. sparq
must implement exact RDF-term equality (no premature numeric folding), no row
caps, and correct property-path duplicate semantics — and cross-check answer sets
against QLever/Virtuoso as a CI gate.

**Build-throughput bars** `[PARSE-02]`, `[DICT-02]`, `[DICT-04]`: QLever full
Wikidata index build 4.4 h (parse 1.4 h, vocab 0.4 h, global ids 0.2 h, 6
permutations 2.4 h); load ~1.7M triples/s on DBLP-390M; serd is the fastest
Turtle reader at constant memory (the parsing bar) `[PARSE-01]`.

---

## 3. Component designs (implementation-grade)

Workspace crates (existing scaffold, kept):
`sparq-core` (dictionary + permutation store + bulk loader), `sparq-engine`
(algebra → physical plan → execution), `sparq-cli` (loader/query runner). Term
model and parsing reuse `oxrdf`/`oxttl`; SPARQL syntax via `spargebra`; store,
planner and execution are our own.

### 3.1 Dictionary

**Encoding — tagged 64-bit `ValueId`** `[qlever-encoded-value-ids]`,
`[DICT-01]`, `[oxigraph-inline-ids]`, `[stor-01-dictionary-encoding-id-layout]`.

```rust
#[repr(transparent)]
pub struct ValueId(u64);          // top 4 bits = Datatype tag, low 60 = payload
enum Datatype {                   // tag values 0..15
  Undefined, Bool, Int, Double, Date, GeoPoint,
  VocabIndex, LocalVocabIndex, BlankNodeIndex, /* ... */
}
```

- **Inline (no dictionary entry, no lookup):** Bool (1 bit), Int (60-bit signed,
  range ±2^59), Double (mantissa truncated to fit 60 bits, sign-bit flipped so
  bit-order == numeric order), Date/dateTime (bit-cast into 60 bits), GeoPoint
  (lat/long packed). FILTER(`?x > 100`), arithmetic and ORDER BY on these run in
  id space and prune blocks via `firstTriple`/`lastTriple`.
- **Dictionary (VocabIndex):** IRIs, blank nodes, lang/typed/large string
  literals. The id is the **lexicographic rank** in the sorted global vocabulary,
  so id order == term order (enables range scans and prefix compression).
- `Undefined` is a reserved sentinel for OPTIONAL/UNDEF-aware joins.
- Tag ordering chosen so cross-type SPARQL ORDER BY is respected where possible.
- `LocalVocabIndex` covers values newly created mid-query (BIND of a string, etc.)
  carried in a per-chunk `LocalVocab`.

Decision: we use **monotonic lexicographic ids** (not Oxigraph's 128-bit string
hashes) because sorted ids are what unlock gap/delta compression of the
permutation columns and prefix-compressed vocab — the storage win that beats
Oxigraph. We accept that this ties ids to one collation (Unicode codepoint order,
for determinism `[DICT-02]`) and constrains id reclamation on delete.

**String vocabulary compression** `[dictionary-pfc-frontcoding]`, `[DICT-03]`,
`[qlever-fsst-squared-vocab]`.

- Four lexicographic sections with contiguous id ranges: **SO** (terms used as
  both subject and object — lowest ids, so subject↔object join variables range
  only over `[1,|SO|]`), **S**-only, **O**-only, **P** (predicates). Real data
  has huge S/O overlap (HDT: up to 60% in SO), shrinking the index and join
  domains.
- M1–M3: **Plain Front-Coding** — sorted terms in buckets of `b`=16; first term
  verbatim, rest stored as `VByte(shared-prefix-len) + suffix`. `id→string`:
  jump to bucket `id/b`, decode `id mod b` steps. `string→id`: binary-search
  bucket headers, scan within bucket. A `LogArray` (bit-packed offsets) indexes
  bucket starts.
- M4: upgrade hot vocab to **FSST** (random-access string compression, ~1 codebook
  per 1M words) for extra ratio without full-block decode; keep the full vocab
  on disk (mmap) with every k-th word + hot words cached in RAM for ~log₂(1M)
  decompressions per binary-search lookup. Split internal (hot) vs external
  (cold) vocab as QLever does (cold vocab is 206 GB on Wikidata).
- Rust crates: `fst` (Burntsushi) as an alternative immutable FST for ordered
  prefix/range queries; an FSST crate for M4; `rustc-hash`/`hashbrown` for the
  build-time term→id maps.

**Bulk dictionary build — partial vocabs → external merge → global ids**
`[DICT-02]`, `[qlever-parallel-build]`.

1. **Partial vocabs:** as triples stream in, batch (~10M triples), build a
   per-batch `term→local-id` map (per-thread `rustc_hash::FxHashMap` or
   `lasso`/`ThreadedRodeo` to avoid contention), record terms + frequencies, sort
   by Unicode codepoint, dedup adjacent, spill the sorted partial vocab to disk
   async while parsing continues. Triples are emitted with *local* ids.
2. **Merge:** k-way heap-merge all sorted partials with dedup; global id = rank
   in the merged order.
3. **Convert:** reload per-batch local→global maps and rewrite the parsed triples
   to global `ValueId`s in parallel (`rayon`).

Throughput target: QLever's vocab 0.4 h + global-id 0.2 h on Wikidata.

### 3.2 Permutation indexes

**Which permutations.** Ship all **six** (PSO, POS, SPO, SOP, OPS, OSP) for full
SPARQL 1.1 (variable predicates are common in real queries; six give every join
input pre-sorted on every variable for sort-free merge joins and supply LFTJ its
orders). The 3-cover {SPO, POS, OSP} answers all 8 abstract patterns and is the
memory-constrained fallback. For quads/RDF 1.2 triple terms, add a 4th graph column and the
extra orders later. `[qlever-six-permutations]`, `[perm-coverage-3of6]`,
`[wcoj-index-orders]`, `[engine-permutation-survey]`.

**Block layout** `[qlever-compressed-blocks]`, `[INDEX-01]`. Each permutation is
a relation-grouped sequence of blocks (a "relation" = maximal run with equal
leading column):

- The **leading column is elided** from blocks (recovered from per-relation
  metadata) — drops the most redundant column entirely. Only columns 2 and 3
  (+ optional graph) live in the payload.
- Blocks are **column-major** (~250 KB/column uncompressed target) and each column
  is **independently ZSTD-compressed** (level 3) so a scan decodes only the
  columns it needs. Rust: `zstd` crate.
- Per-relation metadata: `{ col0, numRows, multiplicityCol1, multiplicityCol2,
  offsetInBlock }`; `multiplicity == 1.0` ⇒ functional predicate.
- Per-block metadata (kept RAM-resident / mmap, ~0.5 GB at Wikidata scale):
  `{ firstTriple, lastTriple, numRows, perColumn(offset,compressedSize),
  optional graphInfo (≤20 ids), blockIndex }`.

**Scans = binary search over block metadata** `[qlever-block-binary-search-scan]`.
A pattern resolves via `partition_point` on the sorted `[firstTriple,lastTriple]`
array (`equal_range` with comparator `a.lastTriple < b.firstTriple`), returning
the contiguous run of overlapping blocks. Only the first/last blocks of the run
may be partial; interiors are fully in-result. Two paths: exact-size scan, and an
**estimate-only path that reads no data** (sums `numRows`, divides partial blocks
by a fudge factor) for the planner.

**Compression of sorted id lists** `[gap-delta-pfor]`, `[simd-01-simd-bp128-fastpfor]`,
`[rdf3x-leaf-byte-format]`. Default: plain ZSTD-3 over column-major u64s (QLever's
choice — captures most redundancy, simple). Improvement opportunity to measure:
**delta/gap pre-transform then SIMD-BP128 / FastPFOR before ZSTD** for extra ratio
and >1B ints/s decode (Rust `bitpacking` crate = port of Lemire's simdcomp, with
scalar fallback). For fine-grained per-triple random seek, RDF-3X's byte-delta
leaf format (1-byte fast path when only the object increments) is the alternative;
we prefer block ZSTD for simplicity and add bitpacking where scan throughput
dominates.

**Aggregated indexes for stats** `[rdf3x-storage-model]`, `[plan-08]`. The six
2-prefix counts (SP/PS/SO/OS/PO/OP → count) and three 1-prefix counts (S/P/O →
count) are cheap (one row per distinct pair/value) and double as optimizer
cardinalities and as answers to 2-bound COUNT patterns. Largely free since the
permutation block metadata already yields exact 1-/2-prefix counts.

**Build — external merge-sort, paired permutations** `[DICT-04]`,
`[qlever-parallel-build]`. Sort the global-id triple array under each ordering via
an external k-way merge sorter (bounded RAM: only 2 sorters concurrent). Build in
pairs sharing a leading column (PSO→POS, SPO→SOP, OSP→OPS) so each pair's first
sort feeds the sibling cheaply; compress blocks on a worker pool; reuse small-
relation blocks across twins. Radix/MSD sort on the leading u64 (dense ids).
Rust: `rayon` + an external-sort run-generation/merge over fixed 24-byte records.

### 3.3 Parser + bulk-load pipeline

`[PARSE-01]`, `[PARSE-02]`, `[PARSE-03]`, `[PARSE-04]`.

- **Front-end:** memory-map the dump (`memmap2`) → one `&[u8]`, constant memory
  (serd model). SIMD delimiter scanning via `memchr`/`memchr2`/`memchr3` for
  `>`, `"`, `\n`, whitespace, and `\\` (escape detection).
- **Tokenizer:** hand-written **byte-level DFA** with a 256-entry char-class
  lookup table and a resumable `recognize_next_token(&data[pos..], is_ending) ->
  Option<(consumed, Result<Token,Err>)>` (`None` = need more bytes, for
  streaming). Two scratch buffers; return borrowed `Cow<str>` slices into the
  input when no escape is present (the common case) — zero-copy. This supersedes
  N3.js's ~30-regex-per-token lexer. We can reuse `oxttl` (which already
  implements exactly this) for M1 and only hand-roll if profiling demands.
- **Parallel N-Triples/N-Quads:** line-oriented and context-free → split into
  ~8–16 MB chunks, snap each boundary forward to the next `\n` via `memchr`, parse
  chunks on `rayon` workers into per-thread arenas emitting pre-encoded local ids.
- **Parallel Turtle/TriG:** stateful, so prescan @prefix declarations
  single-threaded, then chunk at top-level `.`-boundaries (track `<>"()[]` nesting
  depth) and hand each chunk the shared prefix map. A relaxed/trusted-bulk mode
  (disable @base, mid-stream @prefix redefinition, multiline strings) enables safe
  fan-out; fall back to strict serial parsing when assumptions are violated. Gate
  behind `--trusted-bulk`.
- **End-to-end load pipeline:** mmap → chunk boundaries (memchr) → parallel
  tokenize (zero-copy) → per-batch local dictionary + ValueId encoding → spill
  sorted partial vocabs + local-id triples → k-way merge → global ids → rewrite
  triples → external merge-sort into paired permutations → column-major ZSTD-3
  blocks with metadata. Bounded channels between stages (parse/encode/sort/compress
  overlap).

### 3.4 Query engine

#### SPARQL algebra — reuse `spargebra`

`[plan-01]`, `[plan-02]`. Parse SPARQL 1.1/1.2 with `spargebra` (already a
workspace dependency, `sparql-12` + `sep-0006`) into the W3C `GraphPattern`
algebra (`Bgp`, `Path`, `Join`, `LeftJoin{expr}`, `Filter`, `Union`, `Graph`,
`Extend`, `Minus`, `Values`, `Group`, `Project`, `Distinct`, `Reduced`, `Slice`,
`OrderBy`, `Service`; `PropertyPathExpression` variants). **Do not write our own
grammar.** Lower this logical IR to our own **physical IR**:

```rust
enum PhysicalOp {
  IndexScan{ pattern, permutation, prefiltered_blocks },
  MergeJoin, HashJoin, MultiwayLeapfrog,   // join family
  Filter, Bind, LeftJoin, Minus, Union,
  Aggregate, Sort, Distinct, Slice, PathEval, CartesianProduct,
}
```

Flatten spargebra's nested binary `Join` tree into a join *set* before reordering.
OPTIONAL → `LeftJoin(P1,P2,F)` (F = filters textual inside the OPTIONAL, evaluated
over merged bindings — never pushed below the left join). FILTER scopes to the
whole group; pushdown rules per Oxigraph `push_filters`: into a join side iff all
its vars are bound there (replicate into both if bound in both); LeftJoin
right-side filters go into the LeftJoin's `expression` slot; filters using an
`Extend` var stay above it; Union filters replicate into both branches.

#### Physical operators

- **IndexScan** `[qlever-block-binary-search-scan]`, `[qlever-prefilter-pushdown]`:
  binary-search block metadata; FILTER predicates compiled to a small
  `PrefilterExpression` algebra evaluable against `(firstTriple,lastTriple,
  graphInfo)` are pushed *into* the scan to drop blocks before decompression;
  LIMIT/OFFSET threaded in to prune at source.
- **MergeJoin** `[qlever-merge-gallop-hash-join]`, `[join-02-merge-join-sorted-intersection]`,
  `[plan-09]`: default zipper merge over two sorted streams; UNDEF-aware.
  Sideways-information-passing (U-SIP): a per-variable shared cell (atomic max /
  min-max+Bloom domain filter) lets a downstream merge `seek()` past
  non-matching blocks. Galloping/exponential seek inside leaves.
- **GallopingJoin:** when `size_a/size_b > 1000` and no UNDEF, binary-search the
  small side's keys into the large side `[qlever-merge-gallop-hash-join]`.
- **HashJoin** `[join-01-lockfree-hash-join-tagged]`, `[rust-02-hashbrown-ahash]`:
  for inputs not co-sorted. Build on the smaller side; single shared
  open-addressing table via `hashbrown::RawTable`; 16-bit tag in the high bits of
  the bucket pointer as a Bloom fast-reject; **software-prefetch** the next
  chunk's bucket addresses while comparing the current chunk's tags (RDF joins
  are latency-bound). Fast non-crypto hasher (term-ids are internal → no HashDoS;
  `id.wrapping_mul(0x9E3779B97F4A7C15)` or `ahash`/`foldhash`). Lock-free CAS
  build for parallel morsels.
- **Scan↔scan lazy join** `[qlever-lazy-block-join]`: when both inputs are index
  scans on the same join variable, intersect their block `[firstTriple,lastTriple]`
  ranges first (`getBlocksForJoin`), decompress only mutually-overlapping blocks,
  stream-merge incrementally. Scan↔materialised: derive a key range/set from the
  materialised side to prune scan blocks.
- **MultiwayLeapfrog (WCOJ)** `[wcoj-leapfrog-triejoin]`, `[wcoj-generic-join]`,
  `[wcoj-colt-vectorize]`, `[plan-10]`: fix a global variable order; at each
  level run a leapfrog multiway intersection over the depth's sorted trie
  iterators of all patterns binding that variable, bind, descend, backtrack.
  Iterator trait over B-tree/block cursors (no physical trie needed):
  ```rust
  trait TrieIterator {
    fn key(&self) -> Option<ValueId>;
    fn next(&mut self); fn seek(&mut self, k: ValueId);
    fn open(&mut self); fn up(&mut self); fn at_end(&self) -> bool;
  }
  ```
  `seek` = galloping search in-leaf + tree descent (amortised O(1+log(N/m))).
  Inner intersection uses SIMD UINT/BITSET set-intersection over dictionary-
  encoded ids `[wcoj-emptyheaded]`. COLT-style lazy trie materialisation avoids
  building tries for the iterated side `[wcoj-colt-vectorize]`.
- **Filter** `[simd-02-selection-scan-compress]`: vectorized predicate over a
  chunk → bitmask → selection vector (AVX-512 `vpcompressd` or AVX2 shuffle-LUT
  compaction); inline-id comparisons need zero dictionary access.
- **OPTIONAL / MINUS** `[qlever-optional-minus]`: OPTIONAL = left-outer merge
  emitting left+UNDEFs on no right match, with a fast no-UNDEF specialization and
  galloping when right ≫ left. MINUS = three-way merge comparator dropping left
  rows with a right match on shared variables. Both UNDEF-aware.
- **UNION:** concatenate/interleave child streams, aligning to a common variable
  schema (UNDEF for missing).
- **Aggregation / GROUP BY** `[rust-02-hashbrown-ahash]`: hash-grouping into a
  `hashbrown` table keyed by the group-by id tuple; streaming partial aggregates
  per morsel then merged. HAVING as a post-filter. Sparqloscope/BSBM-BI stress
  this.
- **Property paths** `[plan-11]`, `[BENCH-WDBENCH]`: BFS/DFS driven from the bound
  endpoint, choosing the cheaper direction via the forward/reverse permutations
  (magic-set bound-propagation intuition); `*`/`+` as fixpoint over reachable
  nodes; bound-both = reachability check. **Correct duplicate semantics**
  (Virtuoso's bug) is a hard requirement. Cardinality is hardest here — pair with
  adaptive execution later `[plan-12]`.
- **DISTINCT / Slice:** hash-set distinct (early pushdown where legal, SP2Bench
  Q5); LIMIT/OFFSET short-circuit through the lazy result model.

#### Planner — cost model, cardinality, join ordering

- **Cardinality estimation, layered** `[qlever-cardinality-multiplicity]`,
  `[plan-03]`, `[plan-06]`, `[plan-07]`, `[plan-08]`:
  - Per-pattern scan size = block-metadata `numRows` sum (free; exact for small
    scans). Bootstrap with Oxigraph's boundness constant table before stats land.
  - Join size ≈ `leftSize × rightMultiplicity` using per-relation multiplicities
    stored at build; `isFunctional` for multiplicity 1.0.
  - **Characteristic Sets** for star subqueries (same-subject patterns) — the
    highest-value RDF estimator. Build a `HashMap<BTreeSet<PredId>, CharSetStats>`
    at load: per distinct predicate-set, `distinct` (#subjects) and per-predicate
    `count`; `StarJoinCardinality` sums `distinct(S)·∏ multiplicity` over supersets,
    using the most-selective bound object. Cap at 10k sets (merge rare into
    smallest superset); index sets by predicate to prune superset search.
  - Non-star (path/chain) edges: M4 upgrade to SumRDF possible-world summary
    and/or Virtuoso-style compile-time index sampling (probe permutations with the
    query's constants) `[plan-12]`.
- **Cost model** `[plan-06]`, `[plan-13]`: objective = `Cout` (sum of intermediate
  cardinalities), decomposable for DP. **Sort-aware**: a merge join over an
  already-sorted permutation is far cheaper than a hash join — track each plan's
  output sort order and keep *interesting orders* in the DP table, or we lose the
  whole merge-join advantage.
- **Join ordering** `[plan-04]`, `[qlever-dp-goo-planner]`, `[plan-05]`,
  `[plan-13]`: build the query graph (node per pattern, edge per shared variable,
  cap ~64 patterns as u64/u128 bitsets), split into connected components, plan
  each independently, combine via cartesian product. Per component: **DPccp →
  DPhyp** bottom-up DP keeping multiple interesting-orders per subset, under a
  tunable subgraph budget (default ~1500); **greedy (GOO)** above the budget
  (repeatedly join the pair with smallest estimated result). HSP star-collapsing
  (characteristic-set super-nodes) shrinks huge queries to small skeletons.
  Equality filters become joins where legal.
- **WCOJ-vs-binary decision** `[wcoj-when-which]`, `[wcoj-hybrid-optimizer]`: route
  to MultiwayLeapfrog when a subquery is **cyclic** (chordless cycle in the
  variable graph) *or* when estimated max intermediate exceeds estimated output
  (AGM/ρ*) by a threshold (same-variable skew). Otherwise binary merge/hash. Keep
  the hybrid path — pure WCOJ regresses on acyclic low-skew workloads (JOB/LSQB).
  i-cost (sum of intersected adjacency-list sizes) ranks WCOJ plans.

#### Execution model — vectorized, morsel-driven, lazy

`[vec-01-columnar-vector-batches]`, `[vec-02-selection-vector-vs-compaction]`,
`[morsel-01-morsel-driven-parallelism]`, `[qlever-lazy-results]`,
`[exec-rof-prefetch]`, `[decide-vec-vs-compile]`.

- **Batch model:** operators pass `DataChunk { cols: SmallVec<[Vector;8]>, count }`
  where `Vector { data: Vec<u64>, validity: Option<Bitmap>, sel: Option<SelVector> }`,
  ~2048 ValueIds/chunk (tuned so in-flight columns fit L2). Selection vectors
  carry filtered rows without compacting; compact adaptively when <~50% survive
  several stages. Everything runs on u64 ids; strings materialise only at
  projection.
- **Lazy/streaming results:** each operator's output is `Result =
  Materialized(IdTable) | Lazy(impl Iterator<Item=(IdTable, LocalVocab)>)`.
  Pipeline merge joins and group-bys over chunks; LIMIT/OFFSET push into scans and
  short-circuit. This is the fix for QLever's OOM on OPTIONAL/DISTINCT — large
  intermediates never fully buffer; spill to disk when a streaming op must
  materialise beyond a memory budget.
- **Parallelism:** morsel-driven work-stealing. A morsel = a contiguous range of a
  permutation, fed through the vectorized pipeline in chunks. `rayon` for M3 (with
  `with_min_len(morsel_size)`); graduate the hot loop to a pinned pool
  (`core_affinity`) with a shared atomic morsel cursor + per-pipeline barrier when
  profiling shows scheduling/NUMA overhead. NUMA-local morsel placement deferred
  to multi-socket targets.
- **SIMD priorities** `[decide-vec-vs-compile]`: spend it on **decompression**
  (`bitpacking`) and the **hash-probe prefetch**, not selection scans (~10%
  end-to-end). Relaxed Operator Fusion stages a cache-resident buffer before
  random-access ops (hash probe, dictionary decode) to issue group prefetches.
- **No JIT in early milestones:** memory-bound workload ⇒ vectorization wins;
  reserve Cranelift-compiled fused pipelines for cache-resident compute-heavy
  fragments (arithmetic FILTER/BIND trees, small-table star joins, hot repeated
  queries) only if profiling demands.
- **Rust micro-arch** `[rust-03-smallvec-arena-bounds]`, `[rust-04-mmap-zerocopy]`:
  `SmallVec<[u64;8]>` for row-oriented edges (projection, paths); `bumpalo` arena
  per query for transient plan/buffer allocations; iterator-chain inner loops for
  bounds-check elimination + auto-vectorization; mmap permutation/dict files and
  zero-copy cast to typed slices with `bytemuck`/`zerocopy` (`#[repr(C)]` block
  headers, little-endian, 16/64-byte aligned for SIMD).

---

## 4. Optimality theory

**AGM bound (output-size lower bound).** Model a BGP as a hypergraph H=(V,E):
V = query variables, one hyperedge per triple pattern over its variable set. A
fractional edge cover x=(x_F) assigns x_F ≥ 0 with Σ_{F∋v} x_F ≥ 1 for every
variable v. The Atserias–Grohe–Marx inequality bounds the output:

  |Q| = |⋈_F R_F| ≤ ∏_F |R_F|^{x_F}, and the tightest bound is
  |Q| ≤ 2^{ρ*(Q,D)} where ρ*(Q,D) = min Σ_F (log₂|R_F|)·x_F subject to the cover
  constraints (a small LP).

AGM proved this is tight: for infinitely many input sizes there is an instance
whose output matches the bound. So no join algorithm can run faster than its
output, and the output can be as large as 2^{ρ*}; an algorithm running in
Õ(2^{ρ*}) is therefore **worst-case optimal**. Worked: triangle
R(x,y)⋈S(y,z)⋈T(z,x) with x_F=½ each ⇒ |Q| ≤ √(|R||S||T|) = N^{3/2}; n-clique ⇒
N^{n/2}. `[wcoj-agm-bound]`.

**WCOJ guarantee.** Generic Join / Leapfrog Triejoin compute any conjunctive
query in Õ(2^{ρ*}) = Õ(AGM) — within a polylog factor of the bound — for *any*
valid fractional cover and *any* variable order, by intersecting one variable at
a time and never materialising a sub-join larger than the final output
(Veldhuizen ICDT'14, Thm 2 / Cor 3; Ngo–Ré–Rudra Generic Join, proved via the
query-decomposition lemma / Hölder's inequality). sparq computes ρ* per BGP from
per-pattern cardinality estimates (log₂ of estimated scan sizes as LP
coefficients) — solvable with a tiny LP (`good-lp`/`minilp`) or a greedy
enumerative heuristic over ≤~20 edges — and uses it both as a memory budget /
output-size bound and as the optimality yardstick. `[wcoj-generic-join]`,
`[wcoj-leapfrog-triejoin]`, `[wcoj-agm-bound]`.

**What is and isn't provably optimal.**

- *Provable:* the **per-BGP join** under the WCOJ family is worst-case optimal in
  data complexity (Õ(AGM)), given all six sorted permutations so any variable
  order has its sorted input `[wcoj-index-orders]`. A **single triple-pattern
  scan** is output-optimal: O(log #blocks) to locate + O(result) to emit
  `[qlever-block-binary-search-scan]`. **Acyclic** queries are
  input+output-linear-optimal via Yannakakis (semijoin reduction), which we use
  across WCOJ bags in a GHD when a query is cyclic-in-parts but acyclic-overall
  `[wcoj-emptyheaded]`, `[wcoj-factorised-db]`.
- *Not provable / heuristic:* **join ordering / plan choice** is cost-model-driven
  and depends on cardinality *estimates*, which for RDF are routinely off by
  orders of magnitude — there is no optimality guarantee, only better estimators
  (characteristic sets, SumRDF) and adaptivity (re-optimization, compile-time
  sampling) `[plan-03]`, `[plan-07]`, `[plan-12]`. **Property-path / RPQ**
  cardinality has no good bound. WCOJ is also more *robust* to bad plan choices
  than binary joins (it can't blow up past AGM), which is itself a reason to
  prefer it on uncertain cyclic subqueries `[wcoj-when-which]`.
- *Factorised lower bound:* when results can stay factorised, the right complexity
  target is **fhtw** (fractional hypertree width), with 1 ≤ fhtw ≤ ρ* ≤ |Q| and
  the gap as large as |Q|; a d-representation can be polynomially smaller than the
  flat WCOJ output `[wcoj-factorised-db]`. We emit factorised/vector-group results
  with constant-delay enumeration for large outputs where the variable order
  permits (M4+).

**Per-operator lower bounds to target.** Scan: O(log #blocks + |result|), zero
data reads for size estimates. Two-pattern join on sorted inputs: O(N_min ·
log(N_max/N_min)) merge/gallop (no sort). Multiway cyclic join: Õ(2^{ρ*}).
Aggregation/distinct: O(input) hash. Dictionary lookup: O(log(1M) decompressions)
with FSST + sampled-RAM. `[qlever-merge-gallop-hash-join]`,
`[wcoj-leapfrog-triejoin]`, `[qlever-fsst-squared-vocab]`.

---

## 5. Phased roadmap

Each milestone lists the findings it realises and the benchmark that validates it.

### M1 — First working engine (current target)

**Goal:** correct, single-threaded, in-memory end-to-end SELECT.

- Dictionary with **u32 ids** (M1 simplification; widen to tagged 64-bit ValueId
  in M4), term↔id via `oxrdf` terms + `FxHashMap`/`Vec` `[DICT-01 (subset)]`,
  `[stor-01-dictionary-encoding-id-layout]`.
- **Six sorted permutation indexes** as plain sorted `Vec<[u32;3]>` (uncompressed),
  range-scan by binary search `[qlever-six-permutations]`, `[perm-coverage-3of6]`,
  `[rdf3x-storage-model]`.
- Parse via `spargebra` → own physical IR; **filter pushdown** normalization
  `[plan-01]`, `[plan-02]`.
- BGP via **greedy-ordered hash joins** (Oxigraph `reorder_joins` ported) + a
  basic constant cost model `[plan-05]`, `[plan-06]`, `[join-01-lockfree-hash-join-tagged (scalar)]`.
- **MergeJoin** on co-sorted permutation scans where orders line up
  `[qlever-merge-gallop-hash-join]`, `[join-02-merge-join-sorted-intersection]`.
- SELECT + a **FILTER subset** (comparisons, logical, basic functions) +
  DISTINCT/LIMIT/OFFSET; bulk load from N-Triples/Turtle via `oxttl` `[PARSE-01]`.
- **Validate:** SP2Bench 250k–1M (operator coverage), WatDiv 10M (correctness +
  scaling), DBLP-390M load+query for a first head-to-head vs Oxigraph
  `[BENCH-SP2BENCH]`, `[BENCH-WATDIV]`, `[BENCH-DBLP-500M]`.

### M2 — WCOJ + better planner + more operators

- **Leapfrog Triejoin** over the `TrieIterator` trait on permutation cursors;
  **hybrid WCOJ-vs-binary** routing (cyclic / skew detection, AGM/ρ* threshold,
  i-cost) `[wcoj-leapfrog-triejoin]`, `[wcoj-generic-join]`, `[wcoj-when-which]`,
  `[wcoj-hybrid-optimizer]`, `[plan-10]`.
- **DPccp → DPhyp** join ordering with interesting-orders DP table under a budget,
  greedy above `[plan-04]`, `[plan-13]`, `[qlever-dp-goo-planner]`.
- **Characteristic-set** star cardinality `[plan-03]`, `[qlever-cardinality-multiplicity]`.
- **OPTIONAL, MINUS, UNION, aggregation/GROUP BY/HAVING, BIND**, galloping join,
  U-SIP `[qlever-optional-minus]`, `[plan-09]`.
- AGM/ρ* LP + memory budgeting `[wcoj-agm-bound]`.
- **Validate:** WGPB (pure-BGP/WCOJ, target QLever 12 ms / MillenniumDB 21 ms),
  WatDiv stars/snowflakes/complex, SP2Bench Q4/Q5/Q6/Q7 `[BENCH-WGPB-SCHOLIA]`,
  `[BENCH-WATDIV]`, `[BENCH-SP2BENCH]`.

### M3 — Compression + parallelism + bulk load at scale

- **Column-major ZSTD-3 blocks** with leading-key elision + per-block
  first/last-triple skip metadata + binary-search scans + scan↔scan lazy block
  join `[qlever-compressed-blocks]`, `[INDEX-01]`, `[qlever-block-binary-search-scan]`,
  `[qlever-lazy-block-join]`.
- **Parallel bulk load:** mmap + memchr chunking + parallel tokenize + partial-
  vocab/external-merge/global-id pipeline + paired-permutation external sort
  `[PARSE-02]`, `[PARSE-03]`, `[PARSE-04]`, `[DICT-02]`, `[DICT-04]`, `[qlever-parallel-build]`.
- **Morsel-driven parallel execution**; lock-free tagged-pointer hash join with
  prefetch `[morsel-01-morsel-driven-parallelism]`, `[join-01-lockfree-hash-join-tagged]`,
  `[rust-01-rayon-and-pinned-pool]`.
- **mmap + zero-copy** index/dict load; front-coded vocab `[rust-04-mmap-zerocopy]`,
  `[dictionary-pfc-frontcoding]`, `[DICT-03]`.
- **Validate:** DBLP-390M (beat QLever 231 s load / 8 GB index / 0.7 s),
  WDBench truthy 1.257B, BSBM 100M throughput sweep, LUBM-8000 load
  `[BENCH-DBLP-500M]`, `[BENCH-WDBENCH]`, `[BENCH-BSBM]`, `[BENCH-LUBM]`.

### M4 — Advanced: full scale, correctness, vectorization

- **Tagged 64-bit ValueId** with inline numerics/dates/geo + order-preserving
  encoding `[qlever-encoded-value-ids]`, `[DICT-01]`, `[oxigraph-inline-ids]`.
- **Property paths** (forward/reverse-driven, correct duplicate semantics) and
  navigational/C2RPQ `[plan-11]`, `[BENCH-WDBENCH]`.
- **Vectorized DataChunk execution** + selection vectors + SIMD decompression
  (`bitpacking`) + lazy/streaming results with spill (the QLever-OOM fix)
  `[vec-01-columnar-vector-batches]`, `[vec-02-selection-vector-vs-compaction]`,
  `[simd-01-simd-bp128-fastpfor]`, `[qlever-lazy-results]`.
- **FILTER prefilter-pushdown into scans** against block metadata
  `[qlever-prefilter-pushdown]`.
- **Estimator upgrades:** SumRDF and/or compile-time index sampling; one
  re-optimization checkpoint `[plan-07]`, `[plan-12]`, `[plan-08]`.
- **Fast result serialization:** streaming TSV/JSON + Arrow IPC / native binary
  ValueId columns + memoised id→string decode `[SER-01]`.
- **Validate:** full Wikidata 20B — WDQS-298 (beat QLever mean 4583 / median 103),
  Scholia, Sparqloscope per-feature + zero failures; #Diverge correctness gate
  `[BENCH-WDQS-298]`, `[BENCH-WGPB-SCHOLIA]`, `[BENCH-SPARQLOSCOPE]`, `[BENCH-PROTOCOL]`.

### M5+ — Optional tiers (scope-gated)

- **Reasoning/mutable tier:** RDFox-style triple-table + linked-list + hash
  indexes, lock-free insert, triple-at-a-time parallel semi-naive materialisation,
  B/F incremental maintenance, DeltaTriples overlay for SPARQL UPDATE — validated
  by LUBM/OWL-RL and LDBC SPB writes `[rdfox-triple-table]`, `[rdfox-hash-indexes]`,
  `[rdfox-lockfree-insert]`, `[rdfox-parallel-materialisation]`,
  `[rdfox-incremental-bf-dred]`, `[qlever-delta-triples-updates]`, `[INDEX-02]`,
  `[BENCH-LUBM]`, `[BENCH-DBPSB-LDBC]`.
- **Ultra-compressed read tiers:** HDT BitmapTriples cold segment, k²-triples,
  Ring/CSA single-permutation WCOJ index `[hdt-bitmaptriples]`, `[k2tree-triples]`,
  `[ring-csa-single-perm]`.
- **Full-text + geospatial:** QLever-style text index (`contains-word`/
  `contains-entity`); GeoSPARQL R-tree/S2 for OSM `[qlever-text-index]`,
  `[BENCH-UNIPROT-OSM]`.

---

## 6. WASM end-state

**Goal:** run sparq in the browser (Solid/RDFJS context) as a fast in-memory
SPARQL engine over modest graphs, with a small bundle.

**What ports cleanly.** The whole core is pure Rust over `u64`/`u32` arrays with
no OS dependency except mmap and threads — so the *query path* compiles to
`wasm32-unknown-unknown` essentially as-is: dictionary, in-memory permutation
indexes, merge/hash/Leapfrog joins, the planner, vectorized execution, and the
`spargebra` parser (already Rust). The lazy/streaming result model maps to JS
async iterators.

**What changes for WASM.**

- **No mmap / no OS page cache.** Load indexes from an `ArrayBuffer` (fetched
  `.sparq` file) and `bytemuck`/`zerocopy`-cast in place — same zero-copy story,
  different backing store `[rust-04-mmap-zerocopy]`. Keep ZSTD block decompression
  (zstd has a wasm build; or use a smaller wasm-friendly codec / `bitpacking`'s
  scalar path) `[qlever-compressed-blocks]`, `[simd-01-simd-bp128-fastpfor]`.
- **Threads optional.** Default to single-threaded morsel execution (no
  cross-origin-isolation requirement); offer a `wasm-bindgen-rayon`/Web-Workers
  build behind `crossOriginIsolated` for parallel scans `[morsel-01-morsel-driven-parallelism]`,
  `[rust-01-rayon-and-pinned-pool]`. SIMD via WASM128 (`std::simd` lowers to it).
- **Bulk load in-browser** uses the streaming `oxttl`/byte-DFA path (no parallel
  file chunking); for large graphs prefer shipping a **prebuilt `.sparq` index**
  (permutations + front-coded dict, mmap-format, zero-copy loadable) rather than
  parsing client-side `[PARSE-01]`, `[DICT-03]`.

**Bundle-size strategy.**

- **Feature-gate** the engine: a `wasm` profile excludes the reasoning tier, FSST,
  Ring/CSA, parallel-build, and external sort (build-only). Ship query + in-memory
  store + front-coded dict + ZSTD decode.
- `opt-level = "z"`/`"s"` + `lto = "fat"` + `codegen-units = 1` + `panic = "abort"`
  + `wasm-opt -Oz` (Binaryon) + `twiggy`/`wasm-snip` to trim. `wee_alloc` or the
  default allocator depending on speed/size trade.
- Keep `spargebra` (small) but drop heavyweight optional deps in the wasm feature
  set (no `zstd` C build if a pure-Rust/wasm codec is smaller; no `memmap2`).
- Target: a sub-~1.5 MB gzipped wasm for the query engine + a separately-fetched
  index blob, so it slots into a Solid/RDFJS app as a drop-in fast SPARQL backend.

---

### Appendix: findings index (most load-bearing)

Storage/index: `qlever-six-permutations`, `qlever-compressed-blocks`,
`qlever-block-binary-search-scan`, `perm-coverage-3of6`, `rdf3x-storage-model`,
`hexastore-shared-vectors`, `rdf3x-leaf-byte-format`, `gap-delta-pfor`,
`hdt-bitmaptriples`, `triplebit-matrix`, `k2tree-triples`, `ring-csa-single-perm`,
`engine-permutation-survey`. Dictionary: `qlever-encoded-value-ids`,
`qlever-fsst-squared-vocab`, `dictionary-pfc-frontcoding`, `oxigraph-inline-ids`,
`DICT-01..04`, `stor-01..02`. Joins: `qlever-merge-gallop-hash-join`,
`qlever-lazy-block-join`, `qlever-optional-minus`, `wcoj-*`, `join-01..02`,
`rdfox-inlj-sip-queryplan`. Planner: `qlever-dp-goo-planner`,
`qlever-cardinality-multiplicity`, `qlever-prefilter-pushdown`, `plan-01..13`.
Execution: `qlever-lazy-results`, `vec-01..02`, `morsel-01`, `simd-01..02`,
`decide-vec-vs-compile`, `exec-rof-prefetch`, `compile-01`, `rust-01..04`.
Build/parse: `qlever-parallel-build`, `PARSE-01..04`, `SER-01`. Reasoning/updates:
`rdfox-*`, `qlever-delta-triples-updates`. Benchmarks: `BENCH-*`. Hardware:
`qlever-hardware-design-philosophy`.
