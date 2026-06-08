# Bit-level graph encoding for sparq — an honest assessment

**Question (verbatim intent).** Can we *encode graph properties at the bit level*
— rather than letting numeric ids merely *name* nodes — so that graph mutation and
query optimisation map to arithmetic / bitwise operations (which modern hardware,
shaped by neural workloads, executes very fast)? First try to design it; if it
doesn't pan out, survey how the database world (relational, non-relational, neural)
actually does bit-level optimisation.

**Short answer.** Part of the idea is already real and already in sparq: inline
tagged `ValueId`s push numeric *values* into the id bits so FILTER/range become
arithmetic in id-space (`crates/sparq-core/src/dict.rs:27`, `INLINE_BASE = 1<<30`).
The bigger idea — represent the **graph structure** as bit-matrices so that a BGP
becomes a chain of bitwise-AND + popcount — is a real, published architecture
(**BitMat**), and a *compressed-bitmap* version of it (**Roaring**) is genuinely
worth adopting for sparq, but **only for one specific subsystem** (dense,
low-cardinality adjacency: `rdf:type`, booleans, a handful of hot predicates),
**not** as a replacement for the sorted-permutation + WCOJ core. The succinct
indexes (k²-trees, the Ring) are the *space* frontier, not the *speed* frontier.
The "neural hardware" angle is, honestly, a dead end for *exact* graph ops:
matmul/tensor units are for dense float; exact set intersection is bitwise, and the
"join = sparse matrix multiply" line (MAGiQ) is a *scale-out / portability* play,
not a single-node speed win. Embeddings are approximate and unsuitable for exact
SPARQL — but learned **cardinality estimation** is a legitimate, separable upgrade
to the planner.

This report is grounded in: sparq's own code and docs
(`research/ARCHITECTURE.md`, `research/BENCHMARKS.md`, `dict.rs`, `store.rs`); the
prior hardware-research suite (`research/hardware/`, which already measured that
this workload is **memory-latency/bandwidth bound** and that a NEON
merge-intersection gives 1.6–2.8×); the cited literature (URLs in §7); and a
**measured spike** I ran on this machine (Apple M1, aarch64, 2026-06-08) in
`/tmp/bit-spike` (§5). Every number is tagged **[measured]**, **[literature]**, or
**[speculation]**. No numbers are fabricated.

---

## 1. What "bit-level encoding" can and can't mean here

There are two distinct ideas hiding in the question, and conflating them is the
main trap:

1. **Bit-encode the values** (already done). An id whose *bits are the value*
   (`id = INLINE_BASE + n` for small `xsd:integer`s) lets `FILTER(?x > k)`,
   arithmetic, and ORDER BY run as integer comparisons on the id, with no
   dictionary touch, and — because the ids then sort by value in the permutations —
   lets a range FILTER **binary-search to the passing sub-slice** instead of
   scanning (`research/BENCHMARKS.md`: q06 `FILTER(?a>90)` 4.8 → 1.61 ms, scanning
   140k rows not 1.25M; now 1.24× faster than native QLever). This is real,
   measured, shipped, and is exactly QLever's tagged-`ValueId` idea
   `[qlever-encoded-value-ids]`.

2. **Bit-encode the *structure*** (the open question). Represent the *adjacency* of
   the graph as bits so that **traversal/join becomes bitwise AND/OR + popcount**,
   not a sorted-merge over id lists. This is BitMat, Roaring-over-columns,
   k²-trees, the Ring. This is what the rest of the report evaluates.

The crucial framing fact, established by the prior hardware research and confirmed
by my spike: **sparq's hot path is memory-bandwidth/latency bound, not
ALU-bound.** A streaming scan on this M1 already runs at ~3.8 G rows/s contiguous
(`research/hardware/README.md`); the CPU is *waiting on memory*, not on integer
units. That single fact decides almost everything below: a bit representation helps
**iff it moves fewer bytes** (compression / density), and hurts whenever it forces
**materialisation** of bytes that a sorted scan would have skipped. "More arithmetic
throughput" is not the lever; "fewer cache lines touched per result" is.

---

## 2. The structural techniques, each assessed against sparq

### 2.1 BitMat — the literal answer to the question

**What it is** `[bitmat-www2010]`, `[bitmat-ssws09]`. BitMat (Atre, Chaoji,
Zaki, Hendler; *Matrix "Bit" loaded*, WWW 2010) stores an RDF graph as a 3-D bit
cube over (S, P, O): a bit at (s,p,o) is 1 iff the triple exists. In practice it is
materialised as 2-D bit-matrix *slices* — e.g. for each predicate `p`, an
S×O bit-matrix `BM_p`; plus the transposes, so you can slice on any bound
dimension. A triple pattern selects a slice/row/column. The two core operations are:

- **fold(BitMat, retainDim) → bit-vector**: OR-reduce the matrix along the
  non-retained dimension, yielding the set of values that appear in the retained
  dimension (e.g. "which subjects have *any* matching object" = fold an S×O slice
  along O).
- **`MaskBitArray = AND` of folds**: a BGP is processed by (a) folding each pattern
  to the bit-vectors of its join variable's live domain, (b) **AND-ing** those
  bit-vectors across patterns sharing a variable to prune each variable's domain to
  the values that *can* participate, (c) iterating the prune to a fixpoint
  (semi-join reduction in bit-space), then (d) a final subgraph-matching pass that
  enumerates results **without decompressing** the matrix. BitMat's matrices are
  gap-compressed bit-runs, and the AND/OR/fold run directly on the compressed runs.

So: **a BGP's filtering phase is literally a sequence of bitwise ANDs + OR-folds**,
which is exactly the user's intuition. BitMat scaled to 1.33 B triples in main
memory on a single node (2010), which was the largest single-node result of its
day `[bitmat-www2010]`.

**Where it wins.** (i) **Multi-pattern AND with shared variables** over **dense**
slices: the fold+AND prunes whole domains in one pass, and the AND of two dense
bit-vectors is branch-free and SIMD-friendly. (ii) **Memory**, when slices are
dense (a dense predicate's S×O matrix at 1 bit/cell beats a 12-byte triple). (iii)
The semi-join *pruning* phase is a clean, cache-friendly fixpoint — morally the
same thing Yannakakis/U-SIP do in sparq, but in bits.

**Where it loses, and why it is not sparq's core.** (i) **Sparsity.** Real RDF
predicate matrices are *extremely* sparse — Wikidata's S and O domains are 10^8–10^9
wide and most predicates touch a tiny fraction. An uncompressed S×O bit-matrix is
|S|·|O|/8 bytes, astronomically larger than the triples; you *must* run-compress,
and once you do, your "bitwise AND" is AND-over-runs — i.e. you have reinvented a
**compressed-bitmap intersection**, which is exactly Roaring (§2.2), and at low
density Roaring **loses to sorted-merge** (my spike, §5). (ii) **Result
materialisation / enumeration.** BitMat is great at *deciding which values survive*
but the final result tuples still have to be enumerated; for high-output BGPs the
enumeration dominates and you are back to iterating id lists. (iii) **Variable
predicate** (`?s ?p ?o`) and **path** queries don't fit the per-predicate-slice
model cleanly. (iv) **Worst-case-optimality.** BitMat's pruning is a powerful
heuristic but is **not** the AGM/ρ\* guarantee that sparq's Leapfrog Triejoin gives
on cyclic queries; on a triangle, the bit-prune helps but the enumeration can still
chase intermediates. sparq's WCOJ is asymptotically safer there (`ARCHITECTURE.md`
§4).

**Verdict on BitMat.** It is the *closest* published match to the idea and it
*works*, but its win is concentrated on **dense, multi-pattern, low-output** BGPs,
and its dense form is killed by RDF sparsity. The *useful, transplantable kernel*
of BitMat for sparq is its **semi-join-in-bit-space domain pruning** — and that
kernel is best realised with **Roaring**, not raw matrices. Don't build BitMat;
steal its pruning idea for the dense-predicate subsystem (§6).

### 2.2 Compressed bitmaps — Roaring / WAH / EWAH

**What they are** `[roaring-spe2016]`, `[roaring-spe2018]`. A compressed bitmap
stores a set of u32s so that **set algebra is bit algebra**: A∩B = bitmap AND, A∪B
= OR, |A| = popcount, all SIMD-accelerated. **WAH / EWAH** are run-length word
encodings; **Roaring** (the modern standard, used by ClickHouse, Doris, Lucene,
Druid) partitions the u32 space into 2^16 chunks and stores each chunk as the
*best* of {sorted array (sparse), bitset (dense), run list (clustered)}, switching
representation per chunk. CRoaring ships AVX2/AVX-512/NEON kernels; the Roaring
paper reports intersections **up to 900× faster than WAH/Concise** and ~2× better
compression `[roaring-spe2016]`.

**How they'd map onto sparq.** Two candidate uses:

1. **Adjacency bitmaps as a join primitive.** For a bound (p, o), the set of
   subjects `{s : (s,p,o) ∈ G}` is a u32 set → store it (or compute it) as a
   Roaring bitmap; a star/path join on that variable becomes **bitmap AND** across
   patterns. This is the BitMat fold+AND, done with a production bitmap library.
2. **Compress the permutation columns.** A permutation's leading-key runs are
   sorted id lists; the *object* (or subject) column within a `(p, ·)` run is a
   sorted u32 set → Roaring-encodable, and a scan↔scan join on it becomes a bitmap
   AND.

**The honest crossover — measured (§5).** My spike intersects two synthetic sorted
u32 sets (universe 2^26) at increasing density, sorted-merge vs Roaring AND:

| density | set size | merge ns/op | roaring AND ns/op | **roaring speedup** | roaring/vec bytes |
|--:|--:|--:|--:|--:|--:|
| 0.0001 | 6,710 | 36,441 | 66,605 | **0.55× (loses)** | 0.805 |
| 0.001 | 67,108 | 364,407 | 718,470 | **0.51× (loses)** | 0.531 |
| 0.01 | 671,088 | 3.62 M | 7.74 M | **0.47× (loses)** | 0.503 |
| 0.05 | 3.36 M | 21.0 M | 34.5 M | **0.61× (loses)** | 0.501 |
| **0.10** | 6.71 M | 33.4 M | **7.43 M** | **4.5× (wins)** | 0.329 |
| 0.25 | 16.8 M | 73.4 M | 9.48 M | **7.7× (wins)** | 0.141 |
| 0.50 | 33.6 M | 141 M | **1.15 M** | **123× (wins)** | 0.079 |
| 0.90 | 60.4 M | 151 M | 0.89 M | **170× (wins)** | 0.053 |

The crossover is sharp and lands around **5–10 % density**. Below it, sorted-merge
is *faster* (it streams two cache-friendly arrays; Roaring pays container overhead
and the result is still sparse). Above it, Roaring wins by **1–2 orders of
magnitude** because (a) the dense chunks become bitset-AND (branch-free, ~1 cycle /
64 bits) and (b) — crucially for a bandwidth-bound machine — **Roaring's size stays
flat at 16.8 MB while the Vec grows to 318 MB**; the merge is moving 19× more bytes.
That is the memory-bound thesis made visible.

**Asymmetric (selective ∩ large) — measured:**

| small | large | merge ns/op | gallop ns/op | roaring ns/op |
|--:|--:|--:|--:|--:|
| 100 | 33.5 M | 71.9 M | 14,991 | **5,819** |
| 1,000 | 33.5 M | 72.4 M | 117,043 | **23,824** |
| 10,000 | 33.5 M | 72.9 M | 4.60 M | **48,694** |

When one side is tiny, **never** linear-merge (72 ms!). sparq already galloping-joins
here, which is ~4800× better for a 100-probe set; Roaring is **another ~2.6× faster
than gallop** and degrades far more gracefully as the probe grows (at 10k probes,
gallop 4.6 ms vs Roaring 49 µs). So even where sparq's gallop is "good enough,"
Roaring is better — *if the large side is already a resident bitmap*.

**The catch (don't gloss it):** these numbers exclude bitmap *build* time and
assume the bitmap is **resident**. Building a Roaring bitmap from a cold sorted scan
costs an O(n) pass; if you build-then-AND-then-throw-away per query, the build
swamps the win at low density. Roaring pays off when the bitmap is a **persistent
index artifact** (built once at load, AND'd many times) over a **dense** column.
That is precisely the dense-predicate / `rdf:type` case, and nothing else.

**WAH/EWAH vs Roaring:** Roaring dominates both on the workloads that matter
(random/clustered), per the paper and universal industry adoption; there is no
reason to pick WAH/EWAH for sparq. If a bitmap tier is built, build Roaring.

### 2.3 Succinct indexes — k²-trees, wavelet trees, the Ring

These attack **space**, and pose adjacency/intersection as **rank/select** bit
operations rather than AND/popcount.

- **k²-tree RDF (k²-triples)** `[k2triples-2011]`, `[k2-revisited-2020]`. Apply the
  k²-tree (a recursive 2-D quadtree over the adjacency bit-matrix, stored as a
  bit-sequence with rank/select) to vertical-partitioned RDF: one k²-tree per
  predicate over its S×O matrix. Forward (`s p ?o`) and backward (`?s p o`)
  neighbour queries are rank/select descents; joins are coordinated tree traversals.
  Reported as an **ultra-compressed, full-in-memory** RDF store that answers SPARQL
  *without decompression*. The 2020 follow-up (BMatrix) beats k²-triples on space
  for some datasets. **Assessment:** k²-trees are the right tool when **space is the
  binding constraint** (the mobile 2 GB tab ceiling in
  `research/hardware/consumer-and-targets.md`, or fitting >1 T triples on one box).
  But rank/select descents are **pointer-chasing**, i.e. *latency-bound* — the worst
  fit for sparq's already-bandwidth-bound profile — so they trade **scan speed for
  space**. They belong in `ARCHITECTURE.md`'s already-listed "ultra-compressed read
  tiers (M5+)", not the hot path.

- **The Ring** `[ring-tods2024]`, `[ring-pvldb-wco]`. Arroyuelo, Hogan, Navarro et
  al. Treat each triple as a *cyclic* string of length 3 and build a **BWT-based
  self-index** (wavelet trees over the BWT). It is **worst-case-optimal for SPARQL
  BGPs** (it gives Leapfrog-Triejoin's `seek`/`next`/`open` primitives via
  wavelet-tree rank/select) while storing **only one permutation's worth of data** —
  it reports **4–6× less space** than Jena/RDF-3X/Virtuoso/Blazegraph and **2–6×
  faster** average BGP time than those (non-WCOJ) systems, in *near-graph space*.
  This is the most intellectually exciting result for sparq, because it is the
  *same* WCOJ guarantee sparq's LFTJ targets but at a **fraction of the six-permutation
  memory** (sparq stores 6× redundancy; the Ring stores ≈1×). **Assessment:** the
  Ring is the **long-term compression endgame** for the WCOJ core — it would
  collapse sparq's six `Vec<[u32;3]>` (or six ZSTD column sets) into one succinct
  structure with the *same* asymptotic join guarantee. The cost is real: wavelet-tree
  `rank/select` is `O(log σ)` bit-twiddling per step with heavy pointer-chasing,
  typically a **constant-factor slower per-op** than a flat sorted array on
  cache-resident data — so on the small/dense in-RAM datasets where sparq currently
  *wins*, the Ring would likely **regress latency to buy memory**. It is the
  technique to adopt when sparq is **memory-bound at billion-triple scale**, not
  before. It is already (correctly) parked in `ARCHITECTURE.md` as
  `[ring-csa-single-perm]`, M5+.

- **Wavelet trees** are the substrate under the Ring (and give per-symbol rank/select
  for free); not a standalone choice for sparq beyond that.

### 2.4 Bit-Sliced Indexes (BSI) — the numeric-FILTER angle

**What it is** `[bsi-rinfret-oneil]`. Store N integer values of B bits as **B
bitplanes**: bitplane `k` is a bitmap whose bit `i` = bit `k` of value `i`. A range
predicate `value > k`, `MIN`, `SUM`, top-k, and even `X+Y` are computed by
**O(B) bitwise passes** (AND/OR/XOR/NOT) over the planes, ending in a result bitmap
+ popcount — pure bit/arithmetic, no per-row branch. This is the most *direct*
mapping of "numeric filter → bit ops," and it's directly relevant to sparq's inline
`ValueId` range filter.

**Measured (§5)** — `value > midpoint`, BSI bitplanes vs a contiguous scalar
`Vec<u32>` scan, ns **per element**:

| n | bits | scalar ns/el | BSI ns/el | **BSI vs scalar** | BSI/scalar bytes |
|--:|--:|--:|--:|--:|--:|
| 1 M | 8 | 0.149 | **0.091** | **1.64× faster** | 0.250 |
| 1 M | 16 | 0.153 | 0.166 | 0.92× (par) | 0.500 |
| 1 M | 30 | 0.202 | 0.586 | **0.34× (slower)** | 0.938 |
| 10 M | 8 | 0.157 | **0.090** | **1.75× faster** | 0.250 |
| 10 M | 16 | 0.154 | 0.180 | 0.86× | 0.500 |
| 10 M | 30 | 0.156 | 0.302 | **0.52× (slower)** | 0.938 |

**Honest reading.** BSI's cost is **O(bits) passes over the column**, so it only
beats a single scalar pass when the value domain is *narrow* (≤ ~8 bits → 1.6–1.8×
faster **and** 4× smaller). At 16 bits it's a wash; at 30 bits (sparq's inline-int
range, `INLINE_BASE = 1<<30`) it is **2–3× slower** and barely smaller. **So BSI is
the wrong tool for sparq's wide inline integers.** Two caveats that matter:

1. The scalar baseline here is a **contiguous** scan (~0.15 ns/el ≈ 6.6 G/s). sparq's
   *current* real filter does a **random gather** into the `Vec<f64>` numeric cache,
   measured at 8–15× slower (`research/hardware/README.md`). BSI would *beat* that
   gather — but so does the **already-shipped** fix (sorted-column scan + range
   pruning, `BENCHMARKS.md` item 5), which is simpler and also gives 1.24× over
   QLever. BSI competes with the *fast* version and loses on wide domains.
2. BSI shines for **narrow categorical/boolean** columns and for **SUM/MIN/top-k**
   aggregates over them — which sparq does not yet special-case. That is a niche,
   not a core lever.

**Verdict on BSI:** not for wide numeric FILTER (sorted range-pruning already wins);
*maybe* later for narrow-domain aggregates. Low priority.

---

## 3. The hardware angle — does "neural hardware" help exact graph ops?

This is where the question's hopeful premise needs the bluntest answer.

- **Matmul / tensor units (Apple AMX/ANE, NVIDIA Tensor Cores, TPUs) are for dense,
  low-precision *float* GEMM.** Exact RDF set intersection is **integer bitwise**
  (AND + popcount) over **sparse** sets. These are different primitives. You cannot
  route an exact set-intersection through a tensor core and get the right answer
  faster; the prior research already tried and dropped AMX/vDSP/ANE as "wrong shape"
  and "honestly infeasible" for an exact-integer triplestore
  (`research/hardware/README.md`, `m1-apple-silicon.md`). **[literature + measured]**

- **"Join as sparse matrix multiply" (MAGiQ / GSmart)** `[magiq-eurosys2019]`,
  `[magiq-vldb2018]`. This line *is* real: represent the graph as a sparse boolean
  matrix and express a SPARQL join as **sparse matrix–vector / matrix–matrix
  multiplication** over a semiring (GraphBLAS / SuiteSparse). MAGiQ scales to **512 B
  triples** across **CPUs, GPUs, and a Cray supercomputer** from one matrix-algebra
  program. **But be precise about *why* it exists:** its value is **portability and
  distributed scale-out** ("write the query once, run on any GraphBLAS backend"), not
  single-node latency. On a single commodity box, a tuned sorted-merge/WCOJ engine
  (QLever, sparq) *beats* the SpMV formulation — the survey
  literature is explicit that MAGiQ targets the massively-distributed regime. The
  SpMV *does* use the GPU's sparse-linear-algebra units (which are bit/integer-capable,
  not the float tensor cores), so it is "bit-ish hardware," but it is **not** the
  neural-tensor-unit win the question imagines, and it does not help sparq's target
  (one €2.5k machine). **[literature]**

- **SIMD popcount/AND (AVX-512 `vpopcntq`, NEON) is the one that genuinely helps** —
  and it helps *bitmaps and merge-intersection*, both already on sparq's roadmap.
  CRoaring's NEON/AVX kernels are why the §5 dense-AND numbers are so fast; the prior
  research measured a **NEON merge-intersection at 1.6–2.8×** (`README.md`). This is
  the real, bankable hardware lever, and it is **orthogonal** to "neural hardware" —
  it's plain data-parallel integer SIMD. **[measured, prior]**

- **GPU bitmap joins** help only as a **resident columnar co-processor** for huge
  bulk scans/aggregates; the measured wgpu spike (`gpu-and-cloud.md`) found a
  16–64 M-row crossover and a 4–12× host→device transfer tax that kills per-query
  offload. A Roaring-on-GPU AND has the same shape: worth it only if the bitmaps
  *live* on the device. Not a near-term sparq lever. **[measured, prior]**

**Bottom line on hardware:** the only bit-level hardware win that pays for a
single-node engine is **integer SIMD (NEON/AVX) over compressed bitmaps and sorted
columns** — which is *already* the plan. Tensor/neural units do not do exact graph
joins; SpMV is a scale-out story; GPU helps only resident.

---

## 4. Neural / embedding databases — honest scope

- **KG embeddings (TransE, RotatE, ComplEx)** `[transe-rotate]` map entities/relations
  to vectors so that `link-prediction ≈ vector arithmetic` (TransE: `h + r ≈ t`).
  This is **approximate** — it *scores plausibility*, it does not *enumerate the
  exact answer set*. It is categorically **unsuitable for exact SPARQL BGP
  evaluation** (which must return precisely the triples in the graph, with correct
  duplicate semantics — sparq's whole correctness-gate thesis). Using embeddings to
  answer a BGP would return *probable* triples, including ones not in the graph. Do
  not. **[literature]**

- **Where learned models *do* fit sparq — cardinality estimation.** The single
  acknowledged weak spot of every RDF engine (and of sparq's planner) is
  **cardinality estimation for non-star / path patterns**, which is "routinely off by
  orders of magnitude" (`ARCHITECTURE.md` §4). **GNCE** `[gnce-2024]` uses KG
  embeddings + a GNN to estimate **conjunctive-query cardinality** accurately. This is
  a legitimate, *separable* upgrade: it touches only the **planner's cost model**, not
  the exact execution path, so it cannot corrupt answers — a wrong estimate only
  yields a slower plan. It is a real candidate for sparq's "estimator upgrades (M4)"
  slot, *alongside* the already-planned characteristic-sets / SumRDF. Caveat: it adds
  a trained-model dependency and inference cost at plan time; only worth it if
  characteristic-sets + sampling prove insufficient on path-heavy benchmarks
  (WDBench paths). **[literature]**

- **Vector indexes (HNSW/IVF)** are for ANN similarity search, a *different query
  class* (semantic search) that sparq could offer *next to* SPARQL, not *inside* it.
  Out of scope for the bit-level question.

---

## 5. The spike — what I actually built and measured

**Crate:** `/tmp/bit-spike` (standalone; **no sparq crate was modified**). One
dependency: `roaring = "0.10"`. Run on Apple M1 / aarch64 / macOS, 2026-06-08,
`--release` (LTO, opt-level 3). Reproduce: `cd /tmp/bit-spike && cargo run --release`.

**Experiment A — set intersection** (sparq's join primitive): two synthetic sorted
`Vec<u32>` sets over a 2^26 universe, intersected by (i) classic **sort-merge**, (ii)
**galloping** (exponential search — sparq's asymmetric path), (iii) **Roaring AND**;
all three cross-checked to identical counts (`assert_eq!`). Density swept
0.0001→0.9, plus an asymmetric small∩large sweep. Result tables in §2.2 above.

**Experiment B — range filter** `value > k` (sparq's numeric FILTER): **BSI
bitplanes** (B passes of bitwise ops + popcount) vs a **contiguous scalar
`Vec<u32>` scan**; cross-checked identical counts; n ∈ {0.1M, 1M, 10M}, bits ∈
{8, 16, 30}. Result table in §2.4.

**What the spike confirms / refutes:**

- **Confirms:** Roaring bitmap AND beats sorted-merge **only above ~5–10 % density**,
  and then by **1–2 orders of magnitude**, while staying **constant-size** vs a
  growing Vec — the bandwidth-bound thesis, made concrete. **[measured]**
- **Confirms:** below that density, **sorted-merge wins** — so a wholesale bitmap
  rewrite of sparq's (sparse, high-cardinality) permutation columns would
  **regress**. **[measured]**
- **Refutes** the implicit hope that BSI is a free win for numeric FILTER: BSI is
  **2–3× slower** than a contiguous scalar scan at sparq's 30-bit inline-int width;
  it only wins at ≤8-bit domains. sparq's existing **sorted range-pruning** is the
  better lever. **[measured]**
- **Confirms** (asymmetric): never linear-merge a tiny set against a huge one
  (72 ms); gallop (shipped) or a resident Roaring (5.8 µs) are mandatory. **[measured]**

**Spike limitations (stated honestly):** synthetic *uniform-random* sets — real RDF
columns are *clustered* (sorted ids with locality), which **favours Roaring's
run-containers and sorted-merge's prefetcher both**, so the real crossover density
could differ; values exclude bitmap **build** cost (the win assumes resident
bitmaps); single-thread, single machine (M1, not the target Ryzen/Graviton); no
ZSTD-block comparison (sparq's M3 plan) — Roaring would have to beat *compressed*
columns, not raw `Vec`. These don't change the *direction* of the conclusions but
do mean the exact crossover is workload-specific and should be re-measured on a real
predicate column before committing.

---

## 6. Tradeoff matrix — would bit-level beat sparq's current core?

Against sparq's **sorted-permutation + WCOJ + inline-ValueId** baseline:

| Dimension | Roaring bitmap tier (dense cols) | BitMat (full) | k²-tree / Ring (succinct) | BSI (numeric) |
|---|---|---|---|---|
| **(a) Memory** | **Win on dense/low-card cols** (flat 16.8 MB vs 318 MB @ d=0.5 [measured]); neutral/loss on sparse | Loss unless dense (RDF too sparse) | **Big win** (Ring 4–6× less than non-WCOJ stores [lit]; k²-tree ultra-compact) | Win only ≤8-bit; loss at 30-bit [measured] |
| **(b) BGP join speed** | **123–170× on dense AND [measured]**; **0.5× (loss) on sparse [measured]** | Win dense multi-pattern; loss sparse/high-output | **Loss** per-op (rank/select pointer-chase) but **WCOJ-optimal**; trades latency for space | n/a (not a join) |
| **(c) Numeric filters** | n/a | n/a | n/a | Win ≤8-bit; **loss at sparq's 30-bit [measured]** — sorted range-pruning already wins |
| **(d) Mutation/updates** | **Roaring supports incremental insert/delete** (it's a mutable set) — *better* than re-sorting a permutation Vec | Poor (matrix rebuild) | **Poor** (succinct = static; rebuild to update) | Poor (bitplane rebuild) |
| **Query/data shape it wins** | dense predicate (`rdf:type`, booleans, hot ~dozens of predicates), star-AND, selective∩large | dense, multi-pattern, low-output BGP | billion-triple, memory-bound, mostly-static, WCOJ | narrow categorical range/agg |

**The honest synthesis:** **no single bit-level structure dominates sparq's core.**
The sorted-permutation + WCOJ + inline-ValueId design is *already* the right default
for the sparse, high-cardinality, in-RAM regime sparq targets, and it *wins* there
(`BENCHMARKS.md`). Bitmaps win a **specific corner** (dense columns, multi-pattern
AND); succinct indexes win a **different corner** (memory at extreme scale, static
data). A serious engine adopts them as **tiers/specialisations**, not as a rewrite.

---

## 7. Recommendation — prioritised, with measured vs literature vs speculation

**P1 — A Roaring "dense-predicate" adjacency tier. [measured-supported]**
Build, at load time, a Roaring bitmap of the subject (and object) set for each of the
**few dozen densest predicates** (most-of-all `rdf:type`, and any predicate whose
S- or O-domain coverage exceeds the ~5–10 % crossover the spike measured). Use these
bitmaps for: (a) **`?s rdf:type C` + star joins** → bitmap AND across the shared
subject variable (the BitMat fold+AND kernel, via Roaring); (b) **domain pruning /
semi-join** before a merge/WCOJ pass (Roaring AND to shrink a variable's live domain,
then enumerate from the sorted permutation). Keep the sorted-permutation path as the
default; route to bitmaps only when the planner sees a participating dense predicate.
- *Expected gain:* on dense-AND star queries, **up to 1–2 orders of magnitude** on
  the intersection step **[measured, synthetic]**; in practice gated by enumeration
  and by how dense real predicates are **[speculation until measured on a real
  column]**. Memory: a handful of flat bitmaps, ≤16.8 MB each at 2^26 **[measured]**.
- *Cost:* **medium.** Add `roaring` as a dep; build bitmaps in the loader; a planner
  rule + an execution branch. ~A few hundred LOC. Risk: low (it's additive — falls
  back to the existing path). **First, validate the crossover on an actual Wikidata
  `rdf:type`/hot-predicate column** before building the planner integration.

**P2 — Adopt Roaring as the *update* substrate for the (future) mutable tier.
[literature]** When the M5 mutable/reasoning tier lands, Roaring's efficient
incremental insert/delete + AND makes it a natural delta/overlay index — better than
re-sorting permutation Vecs per write. Defer until the write tier is scoped.
- *Cost:* folded into the M5 write-tier design; no standalone work now.

**P3 — Keep the Ring on the roadmap as the WCOJ *compression endgame*, unchanged.
[literature]** When sparq becomes memory-bound at billion-triple scale, the Ring
gives the **same WCOJ guarantee in ≈1 permutation's space** instead of six. It is a
large research-grade build (BWT + wavelet trees + LFTJ over rank/select) and trades
per-op latency for space, so it is wrong for today's in-RAM small/dense wins. Leave
it at M5+ exactly where `ARCHITECTURE.md` already has it.

**P4 — Consider learned cardinality estimation (GNCE-style) only if path-query plans
prove bad. [literature]** Separable, answer-safe (touches only the planner), but adds
a model dependency. Try characteristic-sets + compile-time sampling first (already
planned); reach for the GNN only if WDBench paths show the estimator is the
bottleneck.

**Do NOT do:**
- **Do not** rewrite the permutation columns as bitmaps wholesale — the spike shows
  sorted-merge **wins below ~5–10 % density**, which is where most RDF columns live.
  **[measured]**
- **Do not** add a BSI for the wide inline-integer FILTER — it is **2–3× slower** than
  the contiguous scalar scan at 30 bits, and sparq's shipped sorted range-pruning
  already beats QLever. **[measured]**
- **Do not** chase tensor/neural-hardware or embeddings for *exact* BGP evaluation —
  wrong primitive (float matmul ≠ exact bitwise set ops) and approximate by
  construction. **[literature + prior measured]**
- **Do not** treat MAGiQ-style SpMV as a single-node speedup — it is a
  scale-out/portability architecture, not a latency win for one €2.5k box.
  **[literature]**

**One-line verdict.** The user's instinct is *half right and already half-built*:
value bits → arithmetic FILTER is shipped and winning; structure bits → bitwise AND
is real (BitMat/Roaring) and worth a **targeted dense-predicate Roaring tier**, but
the sparse, high-cardinality, in-RAM heart of sparq is correctly served by sorted
permutations + WCOJ, and no bitmap/succinct/neural structure should replace it.

---

## 8. Sources

- BitMat — Atre, Chaoji, Zaki, Hendler, *Matrix "Bit" loaded: A Scalable Lightweight
  Join Query Processor for RDF Data*, WWW 2010 —
  <https://archives.iw3c2.org/www2010/proceedings/www/p41.pdf> ;
  *BitMat: A Main Memory Bit-matrix of RDF Triples*, SSWS 2009 —
  <https://ceur-ws.org/Vol-517/ssws09-paper3.pdf>
- Roaring bitmaps — Chambi, Lemire, Kaser, Godin, *Better bitmap performance with
  Roaring bitmaps*, SP&E 2016 — <https://arxiv.org/abs/1402.6407> ; Lemire et al.,
  *Roaring Bitmaps: Implementation of an Optimized Software Library*, SP&E 2018 —
  <https://arxiv.org/pdf/1603.06549> ; CRoaring (SIMD AVX2/AVX-512/NEON) —
  <https://github.com/RoaringBitmap/CRoaring>
- k²-trees for RDF — Álvarez-García, Brisaboa, Fernández, Martínez-Prieto,
  *Compressed k²-Triples for Full-In-Memory RDF Engines*, 2011 —
  <https://arxiv.org/abs/1105.4004> ; *Revisiting compact RDF stores based on
  k²-trees* (BMatrix), 2020 — <https://arxiv.org/pdf/2002.11622>
- The Ring — Arroyuelo, Hogan, Navarro, et al., *The Ring: Worst-Case Optimal Joins
  in Graph Databases using (Almost) No Extra Space*, ACM TODS 2024 —
  <https://dl.acm.org/doi/10.1145/3644824> ; preprint —
  <https://aidanhogan.com/docs/ring-graph-wco.pdf>
- Bit-Sliced Index arithmetic — Rinfret, O'Neil, O'Neil, *Bit-sliced index
  arithmetic*, SIGMOD 2001 — <https://dl.acm.org/doi/10.1145/376284.375669>
- MAGiQ — Jamour, Abdelaziz, Chen, Kalnis, *Matrix Algebra Framework for Portable,
  Scalable and Efficient Query Engines for RDF Graphs*, EuroSys 2019 ; demo VLDB 2018
  — <http://www.vldb.org/pvldb/vol11/p1978-jamour.pdf>
- KG embeddings — Bordes et al. TransE (NeurIPS 2013); Sun et al. RotatE (ICLR 2019);
  overview — <https://www.ontotext.com/knowledgehub/fundamentals/what-are-knowledge-graph-embeddings/>
- Learned cardinality (GNCE) — Schwabe, Acosta, *Cardinality Estimation over
  Knowledge Graphs with Embeddings and Graph Neural Networks*, SIGMOD 2024 —
  <https://arxiv.org/abs/2303.01140>
- RDF stores & SPARQL engines survey (context for MAGiQ/scale) — Ali et al., 2021 —
  <https://arxiv.org/pdf/2102.13027>

*Internal references:* `research/ARCHITECTURE.md`, `research/BENCHMARKS.md`,
`research/hardware/README.md`, `research/hardware/m1-apple-silicon.md`,
`research/hardware/gpu-and-cloud.md`, `crates/sparq-core/src/dict.rs`,
`crates/sparq-core/src/store.rs`. *Spike:* `/tmp/bit-spike` (this machine,
2026-06-08).
