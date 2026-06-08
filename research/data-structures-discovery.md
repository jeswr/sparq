# Data-structures discovery sweep — triage for sparq

An open-ended discovery sweep mined adjacent fields (stringology / pangenomics,
IR / sketches, recent DB + theory venues, emerging paradigms, citation-chasing)
for structures sparq's blueprint does **not** already name. This report dedupes
those finds against the known set, groups the genuinely-novel ones by subsystem,
names the top-5 to investigate, and parks the speculative long tail.

**Scope / method.** Triaged against `research/ARCHITECTURE.md` (the decided
design) and `research/bit-level-encoding.md` (a prior, *measured* assessment of
the bitwise/succinct corner). No `research/data-structures.md` existed at write
time, so dedup is against those two documents plus the architecture's findings
index. Evidence tags are carried verbatim from the sweep and re-checked for
honesty: **[measured]** = benchmarked in the cited paper, **[claimed]** =
authors assert without independent benchmark, **[literature]** = established
result, **[speculation]** = the *transfer to RDF/sparq* is our inference, not
demonstrated. URLs preserved. Nothing here is fabricated; where a celebrated
structure does **not** fit sparq's regime, that is stated.

**sparq's regime, restated (the bar every find is judged against).** In-memory,
**memory-bandwidth/latency-bound** (not ALU-bound — established by the hardware
suite and the bit-level spike), dictionary-encoded, **six sorted permutations**
as the storage substrate, **LFTJ/WCOJ + merge/hash** join engine, billion-scale
aspiring, **browser/WASM** target (2–4 GB heap, small bundle, no AVX intrinsics —
WASM128 only), with **future mutability/snapshots** as a stated M5 goal. The
recurring high-value insight across scouts: **the six permutation columns are six
near-identical reorderings of one triple multiset → highly repetitive, run-rich
integer streams**, which is exactly the regime repetition-aware / move-structure
indexes were built for.

---

## 0. What was deduped OUT (already known — not novel)

These appeared in the sweep (or are trivial variants of sweep items) but are
**already in sparq's docs**, so they are *not* counted as discoveries:

- **The Ring** (BWT + wavelet-tree single-permutation WCOJ self-index) —
  `ARCHITECTURE.md` `[ring-csa-single-perm]`, evaluated at length in
  `bit-level-encoding.md` §2.3 (M5+ compression endgame). *RDFCSA and CompactLTJ
  below are adjacent to the Ring but structurally distinct — kept, flagged.*
- **k²-trees / k²-triples / wavelet trees / BitMat / Roaring / WAH / EWAH / BSI**
  — all evaluated in `bit-level-encoding.md` (with a measured spike). Any sweep
  item that is a pure variant (IK2-tree, qdags, BMatrix) is **out**.
- **HDT BitmapTriples, RDF-3X leaf byte format, Elias-Fano (basic), Gap/Delta +
  SIMD-BP128 / FastPFOR, characteristic sets, LFTJ / Generic Join / COLT,
  Free Join, factorized DBs, DPhyp, AGM/ρ\*, FSST + Plain Front-Coding (vocab),
  PFOR** — all named in `ARCHITECTURE.md`. (FSST is *known for the vocab*; the
  sweep's **OnPair** successor is the only novel slice — see Dictionary below.)
- **Learned RMI/PGM, GNN cardinality (GNCE), KG embeddings, HNSW/IVF, GPU/tensor
  SpMV (MAGiQ)** — evaluated/rejected in `bit-level-encoding.md` §3–4.

**Consolidations** (the sweep listed several of these 2–3× under different
scouts; merged here): the **Move structure** (move-r / Movi / b-move / MIOV /
Nishimoto–Tabei) → one entry; **COLOR / Color framework** → one entry;
**Degree-Sequence Bound + Lp-norm/LpBound** → one entry; **SuRF + the modern
range-filter family (Grafite, Oasis, SNARF, Proteus, Memento, …)** → one entry.

---

## 1. Novel discoveries, grouped by sparq subsystem

Each item: **promise (sweep score 1–5)** · evidence · source · honest
applicability note. "Adjacent" = close to a known structure but materially
different; kept with the caveat called out.

### Storage / compact in-RAM representation

- **Move structure (move-r / Movi / b-move / MIOV; Nishimoto–Tabei).** Promise 5.
  A run-length BWT reformulated as a **single flat interval table** giving **O(1)
  LF/locate steps in O(r) space simultaneously** (the r-index could only get one
  or the other), with **1–2 cache misses per step** and explicit prefetch.
  Movi: up to **30× faster than r-index** purely by minimising cache misses;
  move-r: locate **2–35× (typ 15×)** faster than any locate-capable r-index at
  **0.8–2.5× (typ 2×)** its size. **[measured]** for genomics. *Fit:* the cost
  model **is cache misses**, which is sparq's exact bottleneck; sparq's six
  permutations are "runny" permutations (SPO→OSP mapping is full of long runs
  because consecutive triples share S or P), so a move table could run-compress
  them with O(1) cache-friendly stepping. *Honest note:* **[speculation]** for
  RDF — no one has built a move structure over triple permutations; backward-search
  semantics must be mapped onto triple-pattern range scans, and the win depends on
  real run counts `r` in RDF permutations (unmeasured). Rust impls exist.
  Source: https://drops.dagstuhl.de/storage/00lipics/lipics-vol301-sea2024/LIPIcs.SEA.2024.1/LIPIcs.SEA.2024.1.pdf ,
  https://pmc.ncbi.nlm.nih.gov/articles/PMC10635132/

- **Suffixient arrays / suffixient sets.** Promise 5. A tiny subset of the suffix
  array sufficient to locate one occurrence of any MEM by binary search over
  compressed random-access text; its size is a **new repetitiveness measure**
  (≪ r on repetitive data). **[measured]** **3.5 ns/char** query — within ~3× of
  the machine's raw RAM throughput (1.18 ns/char) — while orders of magnitude
  smaller than the suffix array and 1–2 orders smaller *and* faster than the
  r-index. *Fit:* the **closest any index gets to the RAM-bandwidth floor**, which
  is precisely sparq's wall. *Honest note:* **[speculation]** on direct triple
  mapping — designed for text MEM-location, not 6-order triple scans; needs a
  compressed-random-access backing for the permutation and a membership guard.
  Newest/most-research-grade of the set. Source: https://arxiv.org/abs/2407.18753

- **Relative Lempel-Ziv (RLZ) index + RLZ-r / LZ-End-r.** Promise 4. Compress each
  sequence as an LZ77 parse **relative to a fixed reference**; ~0.1 bits/base on 80
  yeast genomes with µs-scale locate; 2025 RLZ-r/LZ-End-r enhance move-r's SA
  sampling. **[measured]** (genomics). *Fit:* **the textbook scenario for sparq's
  6× redundancy** — pick one permutation as reference, store the other five as RLZ
  phrases against it, collapsing 6× toward ~1×+deltas while keeping random access.
  The single most direct attack on the "six indexes" storage blow-up. *Honest
  note:* **[speculation]** for permutation columns specifically — nobody has RLZ'd
  permutation indexes; random-access cost inside a phrase must stay scan-friendly.
  Source: https://ieeexplore.ieee.org/document/8712748/ , https://arxiv.org/pdf/2507.17300

- **Block Trees (1D) + Two-Dimensional Block Trees.** Promise 4. Recursive blocking
  reaching LZ-close compression with **O(log n) random access + rank/select** — a
  drop-in for wavelet trees on repetitive data; the 2D variant compresses adjacency
  matrices and **beats k²-tree space by up to 50%** with forward+reverse neighbour
  queries. **[measured/literature]**. *Fit:* a 1D block tree stores each permutation
  column near LZ entropy yet random-accesses for binary-search scans; a 2D block
  tree gives **two triple-pattern orders (SP→O and reverse) from one structure**
  with rank/select cardinality for free. *Honest note:* rank/select is
  pointer-chasing (latency-bound) — same caveat as k²-trees/Ring in
  `bit-level-encoding.md`; trades scan speed for space. **[speculation]** on RDF.
  Source: https://arxiv.org/pdf/1803.01362 , https://arxiv.org/html/2512.23314

- **RDFCSA (RDF Compressed Suffix Array) + 2024 LTJ extension.** Promise 5,
  **adjacent to the Ring**. Views each triple as a **cyclic string of length 3**
  and indexes the whole set with a CSA (a cycle-forced Ψ permutation + bitvector,
  Ψ run-length+Huffman compressed). Triple-pattern lookup = prefix search. The 2024
  multijoin paper adds a **second CSA (spo+ops)** with `leap()` to support LFTJ.
  **[claimed/measured]** the Chilean group's own paper shows RDFCSA-as-LTJ-backend
  is **~2× the Ring's space but ~4× faster** — because Ψ is more branch/cache-friendly
  than wavelet-tree rank/select. *Fit:* self-indexes (replaces the raw triples),
  serves all orders from **two** structures not six, has a **versioned variant
  (v-RDFCSA)** answering the snapshot goal in the same structure. *Honest note:*
  **adjacent to `[ring-csa-single-perm]`** but the CSA/Ψ substrate is materially
  different from the Ring's wavelet trees, and the "beats the Ring" result directly
  challenges the bit-level doc's framing of the Ring as the compact-WCOJ endpoint.
  Source: https://arxiv.org/pdf/2408.00558 (multijoin §4) , https://arxiv.org/abs/2009.10045

- **CompactLTJ functional tries (LOUDS one-bit-per-edge) + partial tries + trie
  switching.** Promise 5, **adjacent to LFTJ + wavelet/k²**. Each of the 6 LTJ
  tries as a LOUDS ordinal tree at **one bit per edge** (encoding `0^(d-1)1`, leaves
  removed → ~⅓ topology saving); "partial tries" store only *some* orders and
  reconstruct the rest by **trie switching** (re-enter another trie at its root
  instead of descending). **[measured]** most compact variant uses **5–6× less
  space** than classic WCOJ implementations while matching the fastest WCOJ time
  and **30–40× faster than non-WCOJ systems**. *Fit:* directly attacks the cost of
  **six full permutation indexes** for a 2–4 GB browser build; the partial-trie /
  trie-switching knob is exactly the space/time dial sparq needs. *Honest note:*
  LOUDS rank/select is latency-bound (per `bit-level-encoding.md`'s succinct
  caveat); a real win must beat sparq's flat sorted Vec on cache-resident data.
  Source: https://users.dcc.uchile.cl/~gnavarro/ps/grades24.pdf ,
  https://link.springer.com/article/10.1007/s00778-025-00945-5

- **Wheeler graphs + BWT for arbitrary labelled graphs via co-lex orders
  (Cotumaccio; p-sortable / CFS).** Promise 4. Wheeler graphs: a directed
  edge-labelled graph storable at **O(1) bits/edge** with path-coherent rank/select
  if its nodes admit a co-lex total order; Cotumaccio generalises to **all** labelled
  graphs via a width parameter `p` (interpolating linear↔quadratic match time), and
  `p` **quantifies how indexable a given graph is**. **[literature]**. *Fit:* an RDF
  graph *is* an edge-labelled digraph; this is the principled single-index path/pattern
  framework, and `p` doubles as a built-in cost/cardinality knob. *Honest note:* the
  **deepest conceptual fit but the highest risk** — construction is O(|E|²+|V|^{5/2}),
  `p` can be large on messy KGs, and RDF graphs are generally *not* Wheeler.
  **[speculation]** for a production engine; best used as the *lens* to evaluate the
  other BWT finds. Source: https://arxiv.org/abs/2111.04595 , https://arxiv.org/pdf/2402.16205 ,
  https://pmc.ncbi.nlm.nih.gov/articles/PMC5727778/

### Scan / range access / numeric range-pruning

- **SuRF / Fast Succinct Trie + the modern range-filter family (Grafite, Oasis,
  SNARF, Proteus, REncoder, bloomRF, Memento).** Promise 5. Succinct-trie / learned
  / Bloom-hierarchy structures answering **"does any key exist in [lo,hi]?"** in a
  few bits/key with one-sided error. SuRF (LOUDS-DS, ~10 bits/node) doubles as an
  order-preserving index + range-count. **Grafite** gives a **provable FPR bound for
  any (even adversarial) query** (SuRF/SNARF degrade on correlated workloads).
  **Memento** is the **first dynamic+robust** range filter. **[measured/literature]**.
  *Fit:* a per-block range filter lets a triple-pattern scan **skip whole regions of
  the flat Vec without touching them** (bandwidth saving) and prunes empty LFTJ
  branches before descent; tiny enough for WASM; Grafite's adversarial robustness
  matters for a public endpoint; Memento survives the future mutability goal.
  *Honest note:* sparq's **inline order-preserving ValueIds already give range
  FILTER via sorted block pruning** (`bit-level-encoding.md` §2.4: q06 4.8→1.6 ms) —
  so the win is **incremental** over what ships, concentrated on (a) adversarial-FPR
  guarantees and (b) the *dynamic* filter for updates, not on basic range pruning.
  Source: https://db.cs.cmu.edu/papers/2019/20_srf-zhang.pdf ,
  https://arxiv.org/pdf/2311.15380 (Grafite) , https://www.vldb.org/pvldb/vol17/p1911-luo.pdf (Oasis) ,
  https://www.vldb.org/pvldb/vol15/p1632-vaidya.pdf (SNARF) , https://arxiv.org/pdf/2408.05625 (Memento)

- **GraCT — grammar-compressed index with MBR-annotated nonterminals.** Promise 4.
  Re-Pair grammar where **each nonterminal carries a bounding box**, enabling spatial
  range/NN queries **directly on the grammar without decompression**; **[measured]**
  ~4–7% of raw, 2 orders smaller than classic spatio-temporal indexes. *Fit:* the
  transferable kernel is **annotate grammar phrases (or block-tree blocks) with the
  min/max id range they cover**, so a range scan / numeric filter prunes whole phrases
  unexpanded — serving sparq's range-pruning + cardinality. *Honest note:*
  **[speculation]** for RDF; it's the *annotation mechanism* that transfers, not the
  spatio-temporal structure. Source: https://users.dcc.uchile.cl/~gnavarro/ps/isci19.pdf

- **FastLanes Unified Transposed Layout (1024-bit virtual SIMD).** Promise 5. A
  columnar layout that reorders tuples into a common `04261537` interleaving for a
  virtual 1024-bit register, so **the same bit-(un)packing code auto-vectorizes for
  any lane width** — the earlier FastLanes layout decodes **>100 B integers/s with
  SCALAR code**; adds partial decompression and cross-column correlation encoding.
  **[measured/claimed]**. *Fit:* **the headline solves sparq's "no AVX in WASM"
  problem** — near-SIMD integer decode from plain scalar code; ideal for the u32
  ID columns; partial decompression lets range scans skip blocks; cross-column
  encoding could exploit S–P/P–O correlation. *Honest note:* a **layout/format**, not
  an index — competes with/augments sparq's planned ZSTD+bitpacking blocks, and the
  auto-vectorization claim should be re-measured under the WASM128 backend specifically.
  Source: https://www.vldb.org/pvldb/vol18/p4629-afroozeh.pdf

- **PDX (Partition Dimensions Across).** Promise 4. Dimension-major block layout
  ("PDXearch") whose kernel **auto-vectorizes float32 without SIMD intrinsics**,
  2.3–4.6× faster, no recall loss. **[measured]** (vector search). *Fit:* same
  transferable idea as FastLanes — store a permutation **attribute-major** (blocks of
  all-S, then all-P, then all-O) so a one-column scan/filter auto-vectorizes for WASM.
  *Honest note:* **[speculation]** — demonstrated for float distance, **not integer
  ID scanning**; gains may not carry. Overlaps FastLanes in intent; keep one, spike
  the other. Source: https://arxiv.org/pdf/2503.04422

### Join engine (WCOJ / LFTJ / multiway)

- **Fair intersection of seekable iterators (Arntzenius, 2025).** Promise 5. A new
  multiway sorted-set intersection for LFTJ that enforces **fairness** (no iterator
  races ahead), bounding total seeks and improving cache locality vs classic
  leapfrog, with analysis + empirical gains when intersection cardinality ≪ inputs.
  **[claimed/measured]**. *Fit:* **near drop-in for sparq's LFTJ inner loop** — pure
  algorithmic upgrade, needs **no new index**, and is memory-bandwidth-friendly (the
  exact bottleneck). *Honest note:* very new (Oct 2025 preprint); gains are workload-
  dependent (best when output is selective). Lowest-integration-risk join find.
  Source: https://arxiv.org/pdf/2510.26016

- **CoCo index / HoneyComb (eager-sort WCOJ trie + all-variable HyperCube).**
  Promise 5. An **eagerly-built** trie for WCOJ (fully sort each relation once)
  replacing the lazily-built hash-trie, + HyperCube partitioning of **all** query
  variables (not just the top loop) + a plan-rewrite factoring out redundant WCOJ
  subcomputations. **[literature/claimed]**. *Fit:* sparq **already has six fully
  sorted permutations** — exactly the "eager sort" CoCo wants, so its trie is a thin
  view over an existing permutation **at no extra build cost**; the all-variable
  HyperCube is a direct recipe to parallelize LFTJ across cores/web-workers without
  the work-skew lazy tries cause. *Honest note:* HyperCube parallelism shines at
  multi-core scale; browser-single-thread benefit is the redundancy-factoring rewrite,
  not the partitioning. Source: https://arxiv.org/abs/2502.06715

- **VecDict / SmallVecDict / SortedDict trie nodes (unified binary+WCOJ, extends
  Free Join).** Promise 5, **adjacent to Free Join/COLT (known)**. Specialized trie
  nodes: **SortedDict** = binary-searchable node over consecutive ranges (no hash
  table); VecDict/SmallVecDict avoid hash/heap overhead for small/intermediate sets;
  compiled via SDQL, unifying hash- and sort-based WCOJ with binary joins; adds
  redundant-offset elimination + aggregation hoisting. **[measured]** 1.42–4.78× over
  Free Join, 1.49–3.14× over Generic Join on JOB/LSQB. *Fit:* **SortedDict is exactly
  how an LFTJ level over a sorted `Vec<[u32;3]>` should behave** — range scans, no
  hash table for base relations (bandwidth-optimal); directly informs sparq's
  hash-vs-merge-vs-LTJ unification. *Honest note:* it's a *compilation* result (SDQL→C++);
  the transferable part is the **node taxonomy and the "no hash for base relations"
  principle**, which sparq can adopt in its vectorized interpreter without a compiler.
  Source: https://arxiv.org/html/2505.19918v1

- **Tetris-Reloaded / box-cover certificates (beyond-WCOJ geometric joins) +
  PANDA/Jaguar (submodular-width).** Promise 5. Reformulate multiway joins as
  resolving a **box cover** of output gaps; runtime bounded by **certificate size**
  (smallest gap subset covering the output), which can fall **far below AGM**;
  PANDAExpress (2025)/Jaguar (2026) give **subw-time** cyclic-query evaluation
  **asymptotically below the AGM/WCOJ bound** LFTJ targets. **[literature]**. *Fit:*
  sparq's six sorted permutations **are** the tries/B+trees box covers are derived
  from → a box-cover layer makes joins **instance/certificate-optimal** (beyond-WCOJ)
  on the cyclic/skew BGPs where sparq's WCOJ is supposed to win; certificate gaps
  double as cardinality/pruning hints. The striking meta-point: **for cyclic patterns
  (triangles), plain WCOJ is no longer the theoretical frontier** — which the
  architecture treats as the gold standard. *Honest note:* deep theory, **high build
  complexity**, mostly **[literature]** without production-grade RDF implementations;
  a long-horizon research bet, not an M2 item. Source:
  https://arxiv.org/pdf/1909.12102 (Box covers, ICDT 2021) , https://arxiv.org/pdf/1404.0703 (Tetris)

- **Trie-Compressed Intersectable Sets (Arroyuelo, 2022).** Promise 5. An adaptive
  binary-trie set-intersection proven **alternation-adaptive (instance-optimal, not
  merely worst-case)** — the 1978 Trabb-Pardo trie intersection made compact.
  **[measured]** **1.07–1.91× faster than Roaring AND 1.55–3.32× faster than
  Partitioned Elias-Fano**, i.e. beats *both* baselines sparq would reach for. *Fit:*
  a **ready-made instance-optimal replacement for the leapfrog inner loop** of every
  one of the six sorted indexes, squarely outside the known set. *Honest note:* the
  bit-level spike found sorted-merge beats Roaring below ~5–10% density anyway, so the
  relevant comparison for sparq is **trie-intersect vs sorted-merge/gallop** (the
  shipped path), which the paper does not isolate — must be re-measured.
  Source: https://arxiv.org/abs/2212.00946

### Cardinality estimation (the planner's weak spot)

- **Degree-Sequence Bound (DSB) + Lp-norm bounds / LpBound estimator.** Promise 5.
  **Pessimistic, never-underestimating** join-size bounds that **strictly dominate
  AGM and the polymatroid bound** by using the full **degree sequences** (sorted
  value-frequency vectors), stored compactly as **staircase / piecewise-constant**
  functions; Lp-norm generalisation gives **non-trivial bounds for acyclic AND cyclic**
  queries (AGM degenerates on acyclic); **LpBound is a built, evaluated estimator**
  for multi-join + selections + group-by. **[literature → built system]**. *Fit:*
  sparq's sorted permutations make **per-(predicate,position) degree sequences cheap
  to materialise and compress** (Elias-Fano the frequency vector); gives **provable
  bounds to drive LFTJ variable ordering** on skew/cyclic BGPs where AGM point-estimates
  mislead — pure integer math, WASM-friendly, **transfer risk low** (LpBound already
  works on real queries). *Honest note:* complements (does not replace) characteristic
  sets, which sparq already plans for *star* subqueries; DSB/Lp adds the *chain/cyclic*
  coverage characteristic sets lack. Strongest, lowest-risk planner upgrade.
  Source: https://par.nsf.gov/servlets/purl/10428131 ,
  https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICDT.2023.8 ,
  https://arxiv.org/abs/2306.14075 (Lp-norms) , https://www.researchgate.net/publication/392802739 (LpBound)

- **COLOR / Color framework (graph-coloring lifted-graph estimator).** Promise 5.
  Partition vertices into ~32 colors (color-refinement / 1-WL) so same-color nodes
  have similar inter-color degree; store a **tiny lifted graph** of per-color counts,
  min/avg/max inter-color degrees, **and cycle-closure probabilities**; estimate
  acyclic queries exactly under stable colorings. **[measured/claimed]** up to **1000×
  more accurate**, **sub-ms inference**, small memory, **graceful under updates**.
  *Fit:* RDF subgraph cardinality **is** sparq's BGP-estimation problem; the lifted
  graph is **resident and tiny** (ships in a browser bundle), the **cycle-closure term
  handles the cyclic BGPs where WCOJ matters**, far cheaper than the GNN estimators
  `bit-level-encoding.md` already rules out, and update-graceful (matches mutability
  goal). *Honest note:* color-refinement build cost over a billion-edge graph is
  non-trivial; accuracy on RDF specifically is the open question.
  Source: https://arxiv.org/abs/2405.06767

- **Path-centric Summary Graph (PSG).** Promise 4. A summary whose size depends only
  on the **number of vertex/edge TYPES** (not graph size), composing **short-path
  statistics** to estimate arbitrary subgraph cardinalities. **[literature]**. *Fit:*
  RDF predicates **are** edge types → PSG is naturally tiny for RDF and ships easily
  in WASM; composing short-path stats matches how SPARQL chains triple patterns; a
  clean alternative/complement to COLOR. *Honest note:* type-count-bounded size assumes
  a modest predicate count — true for most KGs, but Wikidata has ~10k predicates.
  Source: https://www.vldb.org/pvldb/vol18/p3063-yin.pdf

- **UltraLogLog / ExaLogLog + CPC sketch.** Promise 4. **~28% less memory than
  HyperLogLog** at equal accuracy, mergeable, constant-time insert, near-Cramér-Rao
  estimator; CPC reaches ~2.31 memory value (near the conjectured 1.98 bound).
  **[measured]**. *Fit:* the planner needs per-(predicate)/(s,p)/(p,o) **distinct-value
  counts** for LFTJ ordering — UltraLogLog gives them at a fraction of HLL bytes,
  **mergeable across index shards/snapshots**, cheap to keep WASM-resident. *Honest
  note:* sparq's block metadata **already yields exact 1-/2-prefix counts for free**
  (`ARCHITECTURE.md` §3.2), so the win is only for **per-snapshot mergeable** distinct
  counts under future mutability, or for higher-arity distinct stats the prefix counts
  don't cover. Incremental. Source: https://arxiv.org/pdf/2308.16862

- **Theta / Tuple sketches (set-op sketches for join sizes).** Promise 4. KMV-style
  distinct-count sketches that **also support union/intersection/difference/Jaccard**
  while staying sketches; Tuple variants attach a payload for **approximate join-size
  and join-aggregate** estimation directly. **[literature]** (production DataSketches).
  *Fit:* a Theta sketch per join-key column estimates **intersection size = join
  cardinality** without materializing; mergeable across snapshots. *Honest note:*
  overlaps DSB/Lp (which give *provable bounds*, stronger for planning); sketches give
  *unbiased estimates* — useful where a bound is too loose. Source:
  https://datasketches.apache.org/docs/Theta/ThetaSketches.html

### Filter / semijoin pruning (AMQ)

- **Binary Fuse filters.** Promise 4. Static AMQ, **~13% smaller than Xor**, well
  below Bloom, faster. **[measured]** (mature `xorf` Rust crate). *Fit:* **drop-in for
  any Bloom sparq uses in semijoin / predicate-transfer pruning over id sets**;
  static matches sparq's immutable-snapshot model; WASM-ready. *Honest note:* sparq's
  U-SIP already plans Bloom domain filters — this is a **better constant**, not a new
  capability. Source: https://arxiv.org/pdf/1912.08258

- **Ribbon / BuRR retrieval + Homogeneous Ribbon filter.** Promise 4. Sparse-XOR
  retrieval (key→r-bit value) and AMQ at **<1% space overhead**, faster than Bloom;
  **production-proven (ships in RocksDB)**. **[measured]**. *Fit:* (a) AMQ smaller than
  Bloom for join pruning; (b) **retrieval** mapping a perfect-hash slot → small payload
  (datatype tag, cardinality bucket) at near-zero overhead. *Honest note:* the
  retrieval use overlaps CARAMEL/CSF below; pick one. Rust ports exist.
  Source: https://arxiv.org/abs/2109.01892

- **Predicate Transfer (generalized multi-table Bloom-join / SIP).** Promise 4.
  Generalises 2-table Bloom-join to a **multi-table pre-filter pass**: transfer a
  Bloom/predicate filter along join edges to transitively prune many relations before
  the real join; **[measured]** 3.1× over Bloom-join on TPC-H; optimized variant adds
  a cost-based transfer schedule. *Fit:* a SPARQL BGP **is** a multiway join → a
  predicate-transfer pass with binary-fuse/ribbon filters over sparq's id columns
  **shrinks each pattern's working set before merge/hash/LFTJ**, cutting the dominant
  memory bandwidth; pairs naturally with the sorted indexes (filters built from index
  ranges). *Honest note:* this **generalizes sparq's already-planned U-SIP** — adopt
  it as the principled multi-hop version of what's already scoped. Source:
  https://www.cidrdb.org/cidr2024/papers/p22-yang.pdf

### Dictionary (term vocab + per-term metadata)

- **PtrHash (MPHF at RAM throughput).** Promise 5. Minimal perfect hash via fixed
  8-bit pilots + cuckoo bumping; **~37 ns/key (2× faster than any prior MPHF)** at
  **2.1–3.0 bits/key, single memory access, batch prefetch**; Rust impl by the author.
  **[measured]**. *Fit:* sparq is bandwidth-bound → an MPHF gives a **collision-free
  id space with NO stored keys**, so the string→id side becomes MPHF + packed value
  array; 37 ns/key + batch prefetch match the scan/join hot loops; no_std → WASM. *Honest
  note:* sparq's **id must be the lexicographic rank** (order-preserving, for range
  FILTER → block pruning) — a plain MPHF is **not order-preserving**, so it can serve
  *membership/lookup* but **cannot replace the sorted vocab rank** without an MMPHF
  (below). Source: https://arxiv.org/html/2502.15539

- **Monotone Minimal Perfect Hashing (MMPHF) / learned MMPHF.** Promise 4. Maps each
  key to its **rank in sorted order** in **~O(log log u) bits/key** — far less than
  storing keys; learned variants shrink further. **[literature]**. *Fit:* sparq's
  vocab id **is** a sorted rank → an MMPHF maps a term to its position **without storing
  the sorted array**, enabling order-preserving locate/rank at tiny space — the rare
  find that respects sparq's order-preserving-id constraint (unlike PtrHash). *Honest
  note:* MMPHF returns **garbage on absent keys** → needs a membership guard (a fuse/
  ribbon filter), and front-coding already compresses the vocab well; the win is the
  *additional* drop from O(string) to O(log log u) bits for the index side.
  Source: https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2023.46

- **RecSplit + the 2025 "Modern MPHF" survey.** Promise 4. Most space-efficient
  practical MPHF, **~1.56 bits/key** (breaks the 2-bit barrier) via recursive splitting;
  the survey benchmarks the whole family. **[measured]**. *Fit:* where the smallest
  possible dictionary footprint matters (WASM over billions of terms), RecSplit
  minimizes MPHF overhead; the survey is a ready decision-guide (RecSplit vs PtrHash vs
  PTHash per deployment). *Honest note:* same order-preserving caveat as PtrHash; pick
  per deployment-size from the survey. Source: https://arxiv.org/pdf/2506.06536

- **CARAMEL / AutoCSF (compressed static function lookup).** Promise 4. Read-only
  key→value table at space **proportional to the value-set ENTROPY (no keys stored)**,
  via per-value prefix codes in a ribbon retrieval system; AutoCSF handles skew
  near-optimally. **[literature]**. *Fit:* sparq maps term-id → metadata (datatype tag,
  language tag, lexical-form pointer, small-int flag) — CARAMEL stores that at entropy
  cost **with no key storage**, much smaller than a parallel array when the metadata
  distribution is skewed (it is, for RDF datatypes). Directly shrinks the dictionary's
  value side for WASM. *Honest note:* static (read-only) — fine for snapshots, not the
  mutable tier. Overlaps Ribbon retrieval; CARAMEL is the entropy-optimal specialization.
  Source: https://arxiv.org/pdf/2305.16545

- **OnPair (short-string compression, Re-Pair-flavoured FSST successor).** Promise 4,
  **adjacent to FSST (known for vocab)**. Re-Pair-style successor to FSST improving
  ratio for short strings with fast random access. **[measured]**. *Fit:* IRIs/literals
  are extremely prefix/substring-repetitive → OnPair shrinks the vocab further than the
  already-planned FSST while keeping O(1) random access by id. *Honest note:* a
  **marginal upgrade** over FSST which `ARCHITECTURE.md` already names — measure ratio
  vs FSST before adopting. Source: https://arxiv.org/pdf/2508.02280

### Updates / snapshots / mutability (the M5 goal)

- **RadixGraph (radix vertex index + snapshot-log edge store).** Promise 4. In-memory
  dynamic-graph structure: space-optimized radix-tree vertex index + **snapshot-log
  edge architecture** (immutable snapshot + append-only delta log, periodically merged);
  **[claimed]** up to **16.27× faster updates, ~40% less memory** vs GraphOne/LiveGraph/
  Teseo/GTX/SortledTon. *Fit:* a **concrete blueprint for cheap updates + MVCC snapshots
  over flat permutation indexes without rebuilding** — directly serves the M5 mutability
  goal; the radix vertex index could index the term dictionary. *Honest note:* very new
  (2026 preprint, **[claimed]** not independently verified); a *pattern to import*, not
  a structure to drop in. Source: https://arxiv.org/pdf/2601.01444

- **Tentris hypertrie + incremental insert/delete (ISWC 2025).** Promise 4. The
  hash-based order-3 tensor trie that **implicitly provides all six orders from one
  structure**, extended with a **fast incremental insert/delete** algorithm for online
  + bulk updates at throughput comparable-or-better than traditional triple stores.
  **[claimed]**. *Fit:* the **most relevant negative-space result** — it shows a WCOJ
  index (the chief obstacle to mutability) **can** support fast online updates; the
  single-structure-gives-all-orders idea is an alternative to six sorted Vecs. *Honest
  note:* hypertrie is **hash-based** (different from sparq's sorted Vec) → the transfer
  is **at the algorithm level** (the incremental-update recipe), not the structure.
  Source: https://papers.dice-research.org/2025/ISWC_Tentris-WCOJ-Update/public.pdf

- **Adaptive factorization via linear-chained hash tables (CIDR 2025, DuckDB).**
  Promise 4. A cache-friendly **linear-chained** hash table building **factorized**
  join results adaptively at runtime (factor-vs-flatten decision + result caching).
  **[measured]** 1.25× without caching, up to **17.58× with caching**. *Fit:* avoids
  materializing Cartesian products on skewed RDF stars; linear-chaining is more
  bandwidth-friendly than bucket-chaining for hash-join paths; caching factored
  sub-results could accelerate repeated SPARQL sub-patterns. *Honest note:*
  factorized output is already named in `ARCHITECTURE.md` §4 (fhtw); this is the
  **adaptive runtime mechanism**, the novel slice. Source:
  https://vldb.org/cidrdb/papers/2025/p21-gro.pdf

### Browser / WASM-specific (cross-cutting)

The WASM-load-bearing finds are tagged in their home subsystems: **FastLanes** and
**PDX** (scalar auto-vectorization → no-AVX decode), **PtrHash / RecSplit / MMPHF /
CARAMEL** (tiny dictionary), **CompactLTJ / RDFCSA** (collapse six indexes for the
2–4 GB heap), **binary-fuse / ribbon** (small filters). No browser-only structure was
found that isn't better placed in one of the above.

---

## 2. TOP 5 — "we should look into this"

Selected for **novel-to-us AND plausibly high-leverage** for an in-memory,
bandwidth-bound, billion-scale-aspiring, browser-targeting WCOJ triplestore.
Biased toward (a) attacking sparq's *stated* bottleneck (cache misses / bandwidth /
six-index blow-up), (b) low integration risk, (c) something a cheap spike can settle.

### #1 — Move structure (move-r / Movi) over the permutation columns
- **Why:** the only index family whose **cost model literally is cache misses**
  (1–2/step) — a bullseye for sparq's bandwidth-bound profile — that *also* gets O(1)
  steps **and** run-compressed O(r) space, attacking both the latency wall and the 6×
  storage blow-up at once. Mature Rust impls.
- **Read next:** move-r SEA 2024 (LIPIcs.SEA.2024.1) for the flat-table construction
  and the move-vs-r-index space/time Pareto; Movi (iScience 2024) for the cache-miss
  measurement methodology and prefetch scheme.
- **Cheap validating spike (the decisive measurement):** on a real Wikidata/DBLP
  permutation, **count the run count `r`** of the SPO→OSP (and other) permutation
  mappings — this is a few lines over the existing sorted arrays and **immediately
  bounds the achievable compression** (`O(r)` only helps if `r ≪ n`). Then microbench
  a flat move-table LF-step vs sparq's binary-search range-scan step, measuring
  **cache misses (perf counters) and ns/step**, on cache-resident vs RAM-resident
  columns. If `r/n` is small and ns/step ≤ binary search, it is a storage *and* speed
  win; if `r/n` is large (permutations not as runny as hoped), it is dead — and the
  spike says so for ~a day's work.

### #2 — Degree-Sequence / Lp-norm bounds (LpBound) for the join planner
- **Why:** **provable, never-underestimating** cardinality bounds that **strictly
  dominate AGM**, covering the **chain/cyclic** BGPs that characteristic sets (already
  planned, star-only) miss — directly improving LFTJ variable ordering where bad
  estimates currently mislead. Pure integer math, WASM-friendly, **already a built
  system** (low transfer risk), computable from stats sparq already materializes.
- **Read next:** ICDT 2023 DSB (staircase representation) + LpBound SIGMOD 2025 (the
  evaluated estimator + compressed degree-sequence tradeoffs).
- **Cheap validating spike:** materialize per-(predicate,position) degree sequences as
  staircases from one permutation; on a handful of WatDiv/WGPB cyclic + chain queries,
  compute the DSB/Lp bound and compare to (a) the true output size and (b) sparq's
  current estimate — measure **q-error**. A bound that is tight where AGM is loose, at
  trivial cost, justifies wiring it into the cost model.

### #3 — Fair intersection of seekable iterators for the LFTJ inner loop
- **Why:** a **pure algorithmic upgrade to the join engine with NO new index** — sparq's
  LFTJ already *is* seekable iterators over sorted permutations — that bounds seeks and
  improves cache locality (the bottleneck), exactly where sparq spends its WCOJ time.
- **Read next:** Arntzenius 2025 (arXiv:2510.26016) — the fairness invariant, the seek
  bound, and the empirical regime (selective intersections).
- **Cheap validating spike:** implement the fair-leapfrog intersection beside sparq's
  current leapfrog; on cyclic/skew BGPs (triangles, WGPB) measure **total seeks +
  cache misses + wall time** vs the classic loop. Low risk because it's swappable and
  answer-preserving — a regression just reverts the inner loop.

### #4 — FastLanes Unified Transposed Layout for WASM-friendly integer decode
- **Why:** **directly solves "no AVX intrinsics in WASM"** — its layout auto-vectorizes
  integer bit-(un)packing **from plain scalar code** (>100 B ints/s), so the browser
  build gets near-SIMD decode for free over the u32 ID columns; partial decompression
  lets range scans skip blocks. This is the highest-leverage *browser* find.
- **Read next:** FastLanes PVLDB vol18 (the `04261537` transposed layout + partial
  decompression API); the earlier FastLanes paper for the scalar-decode benchmark.
- **Cheap validating spike:** bit-pack a real permutation column in the FastLanes
  transposed layout, compile the unpack kernel to **wasm32 (WASM128 off and on)**, and
  measure **decode GB/s + ints/s vs sparq's planned ZSTD+bitpacking scalar path** in
  the browser. The question to settle: does the auto-vectorization survive the WASM
  backend, and does it beat the current scalar bitpacking there.

### #5 — RDFCSA + CompactLTJ to collapse six indexes for the browser heap
- **Why:** the two strongest candidates to **collapse sparq's 6× permutation storage
  toward ~1–2×** while keeping triple-pattern scans in all orders and **supporting
  LFTJ** — the enabling structure for a billion-triple-aspiring 2–4 GB browser build.
  RDFCSA's own paper reports it **beats the Ring** (sparq's currently-parked endgame)
  ~4× on speed at ~2× space, and has a **versioned variant** for snapshots; CompactLTJ
  reports **5–6× less space** than classic WCOJ at matching speed.
- **Read next:** "New Compressed Indices for Multijoins" 2024 §4 (RDFCSA-as-LTJ-backend,
  the spo+ops `leap()`); CompactLTJ VLDB-J 2025 (one-bit-per-edge LOUDS + partial tries
  / trie switching).
- **Cheap validating spike:** the cheapest *decision* first — on a real permutation,
  measure the **per-op latency of a wavelet-tree/CSA rank-select-style traversal vs the
  flat sorted-Vec binary search** on cache-resident data (a microbench), because
  `bit-level-encoding.md` already warns succinct rank/select is latency-bound and may
  *regress* sparq's in-RAM wins. Only if the per-op penalty is acceptable does the full
  RDFCSA build pay off. This spike is a *go/no-go gate* on the entire succinct-collapse
  direction.

**If I could spike only ONE: #1, the Move structure.** It is the single find whose
core claim (1–2 cache misses/step in O(r) space) maps *directly* onto sparq's
explicitly-stated, measured bottleneck (memory bandwidth/latency), it attacks the
6×-index storage problem *simultaneously*, and — crucially — the **decisive
measurement is cheap and unambiguous**: counting the run count `r` of the
permutation mappings is a few lines over data sparq already has, and it tells us in
under a day whether the entire repetition-aware-index thesis (which underwrote the
sweep's most surprising finds) is real for RDF permutations or a mirage. Every other
top find is either a refinement of something already planned (#2 over characteristic
sets, #3 over LFTJ, #4 over bitpacking) or gated by a latency risk the bit-level doc
already flagged (#5). #1 is the one with the largest upside *and* the cheapest
truth-test.

---

## 3. Long tail — intriguing but speculative (for the record)

Kept for completeness; lower promise, higher transfer-risk, or strongly overlapping
something above/known. Not recommended for a near-term spike.

- **SBWT (Spectral BWT) + subset-rank queries** (promise 4, **[measured]** genomics,
  <3 bits/k-mer). The subset-rank primitive (does the i-th alphabet-subset contain c)
  could encode "does this (s,p) prefix continue with object X" compactly. **Speculative**
  RDF mapping; membership-oriented, a poor fit for high-output enumeration.
  https://www.biorxiv.org/content/10.1101/2022.05.19.492613v2 , https://lib.rs/crates/sbwt

- **Wheeler graphs / co-lex (Cotumaccio) as the unifying lens** (promise 4,
  **[literature]**). Deepest conceptual fit (RDF *is* an edge-labelled digraph) but
  O(|E|²+|V|^{5/2}) construction and large `p` on messy KGs make it a **research lens**,
  not a build target. Listed in Storage above; repeated here as the long-horizon idea.

- **Tetris-Reloaded / box covers / PANDA / Jaguar** (promise 5 on *ambition*,
  **[literature]**). Theoretically thrilling — instance/certificate-optimal and
  **sub-AGM for cyclic queries**, i.e. *past* the WCOJ frontier sparq treats as gold —
  but no production-grade RDF implementation; a multi-quarter research bet, not an M2/M3
  item. Flagged in Join above.

- **GraCT MBR-annotated grammar** (promise 4, **[measured]** for trajectories). Only the
  *annotation-on-grammar-phrase* kernel transfers (range-prune compressed phrases
  unexpanded); **speculative** for RDF permutation columns.

- **PDX dimension-major layout** (promise 4, **[measured]** for *float* vector search).
  Overlaps FastLanes (#4) in intent; the integer-ID transfer is **unproven**. Spike
  FastLanes first; only reach for PDX if FastLanes' transposed layout disappoints under
  WASM.

- **UltraLogLog / Theta / Tuple sketches** (promise 4, **[measured]**/production). Mostly
  **redundant** with sparq's free exact prefix-counts; the genuine slice is *mergeable
  distinct counts across snapshots* once the mutable tier exists — defer to M5.

- **OnPair vs FSST** (promise 4, **[measured]**). Marginal vocab-ratio upgrade over the
  already-planned FSST; a measurement task, not a research direction.

- **RadixGraph / Tentris-incremental / adaptive-factorization** (promise 4,
  **[claimed]**/**[measured]**). All three are **patterns for the M5 mutable tier**, not
  structures to adopt now; revisit when the write tier is scoped. RadixGraph in
  particular is an unverified 2026 preprint.

- **Binary-fuse / Ribbon-BuRR / CARAMEL / Predicate Transfer** (promise 4,
  **[measured]**/production). All **better constants** over filters/SIP sparq already
  plans (U-SIP, Bloom domain filters), not new capabilities — adopt opportunistically
  during implementation, no spike needed. Predicate Transfer is the one worth treating
  as a *design upgrade* (the principled multi-hop generalization of U-SIP).

- **RT-core RMQ / differential-dataflow shared arrangements / DBSP Z-sets** (from the
  emerging-paradigms scout). RT-core range-min (RTXRMQ, 2.3–5× **[measured]**) is
  **GPU-only, not browser-viable** — wrong tier for sparq's headline target. Differential
  dataflow / DBSP (snapshot isolation + O(Δ) incremental maintenance, **[literature]**,
  Rust/WASM-compilable, Z-set weights = exact cardinalities) is a genuinely interesting
  **mutability-tier** substrate but a large architectural commitment; park with the M5
  update finds.

---

## 4. One honest caveat on the whole sweep

The sweep was **deliberately recall-biased** and most "fit" claims are
**[speculation]** — the *transfer* of a genomics/IR/theory structure to RDF triple
permutations is almost never demonstrated, only argued by structural analogy
("permutations are runny", "RDF is an edge-labelled graph"). The bit-level study
already showed this regime punishes optimistic transfers: Roaring **loses** to
sorted-merge below ~5–10% density, BSI **loses** at sparq's 30-bit inline width, and
every succinct rank/select structure trades **latency for space** in exactly the
in-RAM regime where sparq currently *wins*. The same skepticism applies to the
succinct/compressed finds here (#5, block trees, CompactLTJ, RDFCSA): they are the
**space frontier, not the speed frontier**, and must beat a *flat sorted Vec on
cache-resident data*, not just an asymptotic baseline, before adoption. The finds with
the cleanest path past that bar are the ones whose cost model is *already* cache
misses or pure planning math — **#1 (move), #2 (degree bounds), #3 (fair intersection)**
— which is why they top the list.
