# KG embeddings & the vector-index data layer for sparq — design + literature map

**Topic.** The engine internals needed to embed RDF terms/triples and serve
nearest-neighbour (ANN) queries, integrated with sparq's dictionary-encoded,
out-of-core, six-permutation design. This is the **storage + compute layer** that
the separately-researched *similarity-search* and *NL→SPARQL* features sit on top of.

**Hard constraint inherited from the core design.** `research/data-structures.md`
§R6/R7 and `research/bit-level-encoding.md` already settled that **KG embeddings are
categorically incompatible with exact BGP evaluation** — they return *probable*, not
*present*, triples and would break sparq's no-row-cap / exact-term-equality / #Diverge
gate. Therefore **everything here is opt-in, in a separate crate, gated by a cargo
feature, with ZERO perf/memory impact on the default exact engine, trivially
removable**, and confined to *new* approximate features (vector/semantic search,
embedding-based candidate generation, NL retrieval) plus — answer-safe — the planner's
*cardinality estimate* (GNCE-style, `research/data-structures.md` §S7). The vector
layer never sits on the exact triple-serving hot path.

**Tags.** `[established]` = peer-reviewed / SOTA result with a primary source;
`[claimed]` = authors' own numbers, not independently reproduced; `[novel]` = our
proposal for sparq; `[measured]` = a number sparq or a cited benchmark measured.
No numbers are fabricated; where a number is from secondary reporting it says so.

---

## 0. Executive summary

1. KG embeddings + a vector index belong in an **opt-in `sparq-vectors` crate**, never
   the exact core. They power similarity / semantic / NL retrieval and answer-safe
   cardinality — never BGP answers (which stay exact).
2. **Structural model:** **RotatE** (best quality/parameter on FB15k-237 & WN18RR among
   classic KGEs) or **ComplEx** (cheapest strong baseline); **RDF2Vec** as the
   walk-based, label-free alternative. Inference is trivially cheap: one dict-id → one
   vector lookup — a perfect fit for sparq's u32-id space.
3. **Text model:** a small sentence-transformer (BGE-small / all-MiniLM-L6-v2)
   embedding `rdfs:label`/`skos:prefLabel`/descriptions, run from Rust via **`ort`
   (ONNX Runtime)** or **`candle`** (the WASM-capable path). Combine with structural
   vectors only when labels are informative.
4. **ANN index:** **DiskANN / Vamana** (on-disk, single-node billion-scale, PQ-in-RAM +
   full-vectors-on-SSD) for native — it *is* an external-memory ANN, matching sparq's
   philosophy exactly. **HNSW** (`usearch`/`hnsw_rs`) for the in-RAM and browser tiers.
5. **One vector per dict u32 id:** a memory-mapped, fixed-stride vector store keyed
   directly by `ValueId`. The dictionary IS the embedding key space — no side mapping,
   zero-copy mmap load, mirrors the permutation block format.
6. **Quantization** (PQ / scalar / binary) is mandatory at billion-scale; DiskANN keeps
   PQ codes in RAM and full vectors on SSD, reranking from disk. IVF-PQ is the
   memory-minimal alternative; HNSW the recall-maximal in-RAM one.
7. **[novel]** Build the **Vamana graph with sparq's own external-merge-sort
   infrastructure**, adjacency keyed by dict u32 ids — reuse the permutation-build
   sorter and mmap block format rather than a second I/O stack.
8. **[novel]** **Reuse permutation-index structural features** (degree, characteristic
   sets, predicate-context Roaring bitmaps) as a *training-free* "structural sketch"
   vector: zero GPU, incrementally maintained for free, a cold-start/fallback embedding.
9. **[novel]** **Hybrid structural+text vectors** via late-fusion ANN, with **PQ
   codebooks trained per dict section** (SO/S/O/P) to exploit dictionary locality.
10. **WASM:** ship a small **quantized HNSW** (`usearch` / `candle` both compile to
    wasm32) + a pre-computed vector blob fetched separately — mirroring the engine +
    index-blob split already designed for the exact engine.
11. **Build pipeline:** offline batch. Import pretrained vectors (PyKEEN / DGL-KE
    dumps) for M-V1, or train out-of-core via **Marius** (billion-edge KGE on one box,
    sparq's exact philosophy). GPU optional, never required; inference is CPU/SSD-only.
12. **Benchmarks:** accuracy = link-prediction **Hits@k / MRR on FB15k-237 & WN18RR**
    + retrieval **recall@k** vs a brute-force oracle; performance = build time, RAM +
    SSD footprint, and **ANN query p50/p99 latency** at 1M / 100M / 1B vectors.

---

## 1. KG embedding models — signal, cost, memory, incremental behaviour, quality

A KG-embedding model (KGE) maps each entity and relation to a vector (or matrix), with
a *scoring function* `f(h,r,t)` trained so true triples score higher than corrupted
ones (negative sampling). For sparq the decisive properties are: **(i)** is inference a
cheap per-id vector lookup (yes for all "shallow" KGEs — they store one vector per
entity); **(ii)** memory = `|entities| × dim × bytes`; **(iii)** can a *new* entity be
embedded without full retraining (the incremental story); **(iv)** link-prediction
quality on the standard benchmarks.

### 1.1 Link-prediction quality — the canonical table

Filtered MRR / Hits@10 on the two de-facto benchmarks. **FB15k-237** (14 541 entities,
237 relations, 310k triples — dense, many relation patterns) and **WN18RR** (40 943
entities, 11 relations — sparse, hierarchical, symmetry-heavy). Numbers from the RotatE
paper Table 5 unless noted `[established]`
(<https://arxiv.org/pdf/1902.10197>, ICLR 2019):

| Model | Family | FB15k-237 MRR | FB237 H@10 | WN18RR MRR | WN18RR H@10 | Notes |
|---|---|--:|--:|--:|--:|---|
| **TransE** (2013) | translational | .294 | .465 | .226 | .501 | cannot model symmetry / 1-to-N |
| **DistMult** (2015) | bilinear | .241 | .419 | .43 | .49 | symmetric only (`f` is symmetric in h,t) |
| **ComplEx** (2016) | bilinear, ℂ | .247 | .428 | **.44** | .51 | models antisymmetry; cheap; strong on WN18RR |
| **ConvE** (2018) | neural (CNN) | .325 | .501 | .43 | .52 | 2-D conv on reshaped embeddings |
| **RotatE** (2019) | rotation in ℂ | **.338** | **.533** | **.476** | **.571** | models symmetry+antisymmetry+inversion+composition |

A second large-scale unified re-evaluation (Ali et al., *Bringing Light Into the
Dark*, PyKEEN, IEEE TPAMI 2021, <https://arxiv.org/abs/2006.13365>) `[established]`
showed that **with proper hyperparameter tuning the gap between models largely
collapses** — older models like TransE/DistMult/ComplEx, retuned, match or beat the
headline numbers of newer ones (e.g. ComplEx ~.53 H@10 on WN18RR). **Takeaway for
sparq:** model *choice* matters less than tuning + negative sampling; pick the cheapest
strong one (**ComplEx**) or the best all-rounder (**RotatE**) and invest the budget in
*loading* good vectors, not chasing the SOTA leaderboard.

### 1.2 Model-by-model: signal, cost, incremental behaviour

| Model | Params/entity | Scoring | Training cost | Incremental (new entity) | Best for |
|---|---|---|---|---|---|
| **TransE** | `d` floats | `−‖h+r−t‖` | lowest | retrain or fit-1-entity by SGD | baseline; 1-to-1 relations |
| **TransH / TransR** | `d` (+ rel-space) | project to relation hyperplane/space | medium (TransR has `d²` per relation) | as TransE | 1-to-N, N-to-N |
| **DistMult** | `d` | `⟨h,r,t⟩` (trilinear) | low | as TransE | symmetric relations only |
| **ComplEx** | `2d` (ℂ) | `Re⟨h,r,t̄⟩` | low | as TransE | antisymmetry; strong cheap baseline |
| **RotatE** | `2d` (ℂ) | `−‖h∘r−t‖`, `\|r\|=1` | low-medium | as TransE | symmetry+inversion+composition |
| **RESCAL** | `d` ent, `d²` rel | `hᵀ Mr t` | high (`d²`/relation) | as TransE | expressive, few relations |
| **ConvE** | `d` + CNN weights | CNN over reshaped (h,r) | medium-high | needs CNN forward, not just lookup | parameter-efficient deep |

**Key structural facts for the data layer.** All "shallow" KGEs (TransE…RotatE,
RESCAL) store **exactly one vector per entity** → inference is a single id-indexed
lookup, identical in shape to sparq's dictionary. Memory is `|E|·dim·4 B` (f32) or
`·2 B` (f16); for full Wikidata (~100M entities, dim 200) that is ~80 GB f32 / 40 GB
f16 → **quantization is mandatory** (§3.5, §4.4). ConvE and GNN decoders additionally
need a forward pass, so they are *not* a pure lookup — prefer shallow models for the
serving layer; if a GNN is used for *training*, **materialise its output vectors** to
the same flat store.

### 1.3 Walk-based: RDF2Vec, DeepWalk, node2vec

**RDF2Vec** (Ristoski & Paulheim 2016; family survey Portisch & Paulheim, *Semantic
Web* 2024, <https://content.iospress.com/articles/semantic-web/sw233514>)
`[established]`: extract random walks over the RDF graph (entities + predicates as
tokens), feed the walk "sentences" to **word2vec** (skip-gram). Captures graph
*neighbourhood* structure. On classification/clustering RDF2Vec-SG is competitive with
or beats TransE/TransH/TransR/ComplEx/DistMult
(<https://www.rdf2vec.org/>) `[claimed]`. **node2vec/DeepWalk** are the
property-graph cousins (no typed edges). Pros for sparq: **directly consumes sparq's
permutation indexes** to generate walks (forward walk = SPO scan; the OPS permutation
gives backward walks) — no separate graph build. Cons: word2vec training is its own
pipeline; predicates are tokens, so relation *semantics* (composition/inversion) are
weaker than RotatE. Rust: `rust-word2vec`/`finalfusion` exist but training is usually
offline in Python (`pyrdf2vec`).

### 1.4 GNNs: R-GCN, CompGCN, GraphSAGE

- **R-GCN** (Schlichtkrull et al. 2018, <https://arxiv.org/abs/1703.06103>)
  `[established]`: relation-specific message passing; an *encoder* producing entity
  embeddings, usually with a DistMult decoder. ~29.8% MRR lift on FB15k-237 over a
  decoder-only baseline `[claimed]`. Cost: relation-count parameter blow-up (mitigated
  by basis/block decomposition); training needs the graph in (GPU) memory.
- **CompGCN** (Vashishth et al., ICLR 2020, <https://arxiv.org/pdf/1911.03082>)
  `[established]`: composition operators (sub/mult/circular-correlation) jointly embed
  entities **and** relations; ConvE decoder. Stronger than R-GCN on FB15k-237/WN18RR.
- **GraphSAGE** (Hamilton et al. 2017) `[established]`: **inductive** — embeds *unseen*
  nodes by sampling+aggregating neighbour features. This is the one GNN property sparq
  would actually want (embed a freshly inserted entity without retraining), but it
  needs node *features* (e.g. text embeddings of labels) to be inductive.

**Verdict for sparq:** GNNs give the best quality and the only clean *inductive* story,
but training is heavy and inference needs neighbour gathering (not a pure lookup). Use
them **offline** to produce vectors, then store the materialised output in the flat
per-id store. For incremental inserts at serve time prefer (a) GraphSAGE-style
aggregation over already-stored neighbour vectors, or (b) the **[novel] structural
sketch** of §6.5 as a zero-training fallback.

### 1.5 Libraries & out-of-core training (sparq cares about scale)

| Library | Lang | Scale story | Notes |
|---|---|---|---|
| **PyKEEN** | Python/PyTorch | single-GPU/CPU, 40+ models | best for *evaluation* + pretrained dumps; the unified-eval reference (<https://github.com/pykeen/pykeen>) |
| **DGL-KE** | Python/C++ | multi-GPU + **distributed** | 86M nodes / 338M edges in ~100 min on 8 GPUs, ~30 min on a 4-machine cluster `[claimed]` (<https://arxiv.org/abs/2004.08532>) |
| **Marius / MariusGNN** | C++/Python | **out-of-core, single machine, billion-edge** | pipelining + a data-replacement policy across the full memory hierarchy to keep one GPU busy — **the closest match to sparq's external-memory philosophy** (<https://github.com/marius-team/marius>, OSDI 2021) |
| **GraphVite** | C++/CUDA | CPU-GPU hybrid, node embeddings | 334× node2vec speedup `[claimed]`; **GPU-memory-bound** (~12 GB), so not billion-vertex on one card (<https://arxiv.org/abs/1903.00757>) |
| **AmpliGraph** | Python/TF | usability-focused | good API, smaller scale |

**Rust-native KGE training** is essentially absent (no mature crate) — confirming the
design choice: **train offline (Python/Marius), serve in Rust**. sparq imports vector
dumps; it does not need a Rust KGE trainer for M-V1. (A future `sparq-vectors`
*structural-sketch* path, §6.5, needs no trainer at all.)

**Incremental-update reality across all KGEs:** shallow models embed a new entity only
by SGD-fitting its vector against existing (frozen) neighbours, or by full periodic
retraining; relations rarely change. So the data layer must support **append-only
vector inserts + periodic offline rebuild**, exactly like sparq's static-read-core +
M5 delta-overlay pattern (`research/data-structures.md` §B6). The structural-sketch
(§6.5) is the only *truly* incremental option.

---

## 2. Text embeddings of terms

Pure-structural KGEs know nothing about what an entity *means* lexically; two entities
with identical graph neighbourhoods but different labels collapse. **Text embeddings**
of `rdfs:label` / `skos:prefLabel` / `schema:name` / `dct:description` fix this and are
essential for **NL→entity retrieval** (the NL→SPARQL feature) and for *inductive*
embedding of brand-new entities (a label exists before any edges do).

**Models:** small sentence-transformers — `all-MiniLM-L6-v2` (384-d, ~22M params),
`BAAI/bge-small-en-v1.5` (384-d), `bge-base` (768-d). These dominate the
quality/size frontier for short-text retrieval.

**Running them from Rust** (no Python at serve time):

| Path | Crate | Notes |
|---|---|---|
| **ONNX Runtime** | **`ort`** (+ `fastembed`/`embed_anything`) | hardware-accelerated; export ST model to ONNX; the pragmatic default for native batch embedding (<https://docs.rs/ort>) |
| **candle** | `candle-transformers` | pure-Rust, HF-native, **compiles to WASM** → the only path that embeds text *in-browser* (<https://github.com/huggingface/candle>) |
| **llama.cpp** | `llama-cpp-rs` / GGUF | for larger embedding LLMs; heavier, GPU-optional |

**Structural vs text — when to use which / combine.** Structural KGEs win for
*link-prediction*-style "what entity fits this graph hole"; text wins for *lexical /
NL* retrieval and cold-start. **Combine** when labels are informative AND graph
structure matters (most KGs): two strategies — (a) **concatenate + reproject** (train a
small projection so the fused vector lives in one space), or (b) **late fusion** —
index structural and text vectors separately and merge ranked lists at query time
(simpler, no retraining, lets the query pick the modality). sparq's design (§6.6) uses
**late fusion** by default (modular, no joint training, removable) with concat as an
offline option.

---

## 3. Vector index data structures

The ANN index is the hot part of this layer. The axes are **recall vs latency vs
memory**, and — decisively for sparq — **in-RAM vs on-disk (external-memory)**.

### 3.1 HNSW (Hierarchical Navigable Small World)

Malkov & Yashunin 2016/2018 (<https://arxiv.org/abs/1603.09320>) `[established]`. A
multi-layer proximity graph; greedy search descends layers. **The in-RAM recall/latency
champion** — typically the highest recall@k at a given latency. Cost: the *graph plus
full vectors live in RAM*; build is `O(N log N)` with high constant; updates are
incremental (insert is cheap, delete is awkward). Memory ≈ `N·(dim·4 + M·8)` bytes
(M = max degree, ~16–64). Best when the whole index fits RAM and you want max recall.

### 3.2 IVF-PQ (inverted file + product quantization)

FAISS's workhorse. Partition vectors into `nlist` Voronoi cells (coarse quantizer);
within each cell store **PQ codes** (sub-vector codebooks) not full vectors. Query
probes `nprobe` cells and scans compressed codes. **The memory-minimal option** — a
128-d f32 vector (512 B) → ~8–16 B of PQ codes (a ~97% cut)
(<https://www.pinecone.io/learn/series/faiss/product-quantization/>) `[established]`.
Trade: at 100M scale IVF-PQ used **~7.2× less memory than HNSW but reached only ~70% of
its recall**, recoverable to 90–95% with more `nprobe` + a rerank step (secondary
reporting via Milvus). Best when RAM is the binding constraint.

### 3.3 ScaNN

Google (Guo et al., ICML 2020, <https://arxiv.org/abs/1908.10396>) `[established]`.
IVF+PQ plus two wins: **anisotropic / score-aware quantization** (a loss that preserves
the *ordering* of true neighbours, not raw reconstruction error) and **FastScan**
(SIMD 4-bit PQ lookups). State-of-the-art recall-at-latency for in-RAM at scale; C++
core, no first-class Rust binding (port the anisotropic objective if needed).

### 3.4 DiskANN / Vamana — **the on-disk one that fits sparq**

Subramanya et al., NeurIPS 2019 (<https://suhasjs.github.io/files/diskann_neurips19.pdf>),
Microsoft (<https://github.com/microsoft/DiskANN>) `[established]`. A *single graph*
(Vamana) built with **α-RobustPrune** (a relaxed-monotonic pruning giving a smaller
search radius than HNSW/NSG → fewer hops → fewer disk reads). At serve time: **PQ codes
in RAM** for navigation, **full vectors + adjacency on SSD** read by beam search, with
a final exact rerank from disk. Result: **>5000 QPS, <3 ms mean, 95%+ recall@1 on
SIFT1B (1 billion points) on a 16-core box**, where FAISS/IVFOADC plateau near 50%
recall@1 `[claimed]`. **This is an external-memory ANN — the exact analogue of sparq's
mmap'd out-of-core triple store**, and is the recommended native index.

- **SPANN** (Chen et al., NeurIPS 2021,
  <https://www.microsoft.com/en-us/research/wp-content/uploads/2021/11/SPANN_finalversion1.pdf>)
  `[established]`: an inverted-index hybrid (centroids in RAM, posting lists on disk)
  — competitive billion-scale alternative; more moving parts than a single Vamana
  graph, so DiskANN is preferred for sparq's "one mmap'd file" aesthetic.
- **AiSAQ** (2024, <https://arxiv.org/pdf/2404.06004>): all-in-storage DiskANN keeping
  even PQ codes on SSD (DRAM-free) — relevant to sparq's *most* memory-constrained
  billion-scale target.

### 3.5 Quantization primitives (orthogonal to the index)

- **Product Quantization (PQ):** split a `d`-vector into `m` sub-vectors, k-means a
  256-entry codebook per sub-space → `m` bytes/vector. Asymmetric distance via lookup
  tables. ~97% memory cut, controllable recall loss `[established]`.
- **Scalar quantization (SQ):** per-dimension int8/int4; ~4×/8× cut, near-lossless at
  int8, trivially fast — the *first* thing sparq should ship (cheap, high recall).
- **Binary quantization:** 1 bit/dim + Hamming distance; ~32× cut, fast, lower recall;
  good for a coarse first stage + rerank.

### 3.6 Rust crate landscape — maturity, mmap, WASM

| Crate | Algorithm | mmap / on-disk | WASM | Maturity | Notes |
|---|---|---|---|---|---|
| **`usearch`** | HNSW | yes (memory-mapped index, view mode) | **yes** (compiles to wasm) | high (multi-lang core) | user-defined metrics, f16/i8/PQ; **recommended in-RAM + WASM** (<https://docs.rs/usearch>) |
| **`hnsw_rs`** (jean-pierreBoth) | HNSW | **mmap on dumped vectors** (not graph) | partial | high, pure-Rust | mature, good for large vectors (<https://crates.io/crates/hnsw_rs>) |
| **`hnswlib-rs`** | HNSW | decoupled vector store (BYO `VectorStore`) | partial | medium | graph/vectors decoupled — **fits sparq's "vectors keyed by ValueId" store** |
| **`instant-distance`** (cloudflare) | HNSW | serde dump | partial | medium | simple, production-used |
| **`arroy` / `hannoy`** (Meilisearch) | trees → graph, **LMDB-backed mmap** | **yes (mmap, > RAM)** | no | high | on-disk, KV-backed; `hannoy` = graph successor (<https://blog.kerollmops.com/from-trees-to-graphs-speeding-up-vector-search-10x-with-hannoy>) |
| **`diskann_rs` / infinilabs `diskann`** | **Vamana / DiskANN** | **yes (mmap, billion-scale)** | no | emerging | α-pruning + PQ + beam search; "6–10× lower memory, mmap"; pure-Rust DiskANN (<https://github.com/infinilabs/diskann>) — **the native on-disk choice** |
| **`faiss` (faiss-rs)** | all FAISS (IVF-PQ, HNSW…) | yes | no | high (C++ FFI) | reference perf, but a C++ dependency (fights the small-WASM goal) |

**Selection:** native on-disk billion-scale → **`diskann_rs`/infinilabs `diskann`**
(or port from microsoft/DiskANN); in-RAM + browser → **`usearch`** (single crate spans
both, has mmap + WASM + PQ). Keep `faiss-rs` only as a *benchmark oracle*, not a ship
dependency (its C++ build conflicts with the sub-1.5 MB WASM target).

---

## 4. Storage integration with the dict-encoded, out-of-core design

The whole point of this layer is to **reuse sparq's existing infrastructure** rather
than bolt on a foreign vector DB.

### 4.1 The key insight: the dictionary IS the embedding key space

sparq already assigns every term a monotonic `ValueId` (u32 in M1, tagged u64 later),
with contiguous lexicographic sections **SO / S-only / O-only / P**
(ARCHITECTURE §3.1). **Store exactly one vector per id, in id order, as a flat
fixed-stride array.** Then:

- vector for id `i` lives at byte offset `header + i·stride` — **O(1), no hash, no side
  index**;
- the array is **mmap'd and zero-copy cast** (`bytemuck`/`zerocopy`) exactly like a
  permutation block (ARCHITECTURE §6, `rust-04-mmap-zerocopy`);
- only entities (SO + S + O sections) get vectors; predicates (P section) get relation
  embeddings in a tiny separate array — the section boundaries make this a trivial
  range split;
- **incremental insert = append** a vector at the next id (ids are assigned in load
  order), matching the static-core + append-delta pattern.

### 4.2 File format (mmap) — sketch

```text
sparq-vectors file  (.spqv)  — little-endian, 64-byte-aligned, mmap/zero-copy
┌──────────────────────────────────────────────────────────────────────┐
│ Header { magic, version, dim, dtype(f32|f16|i8|pq), stride,           │
│          n_entity_vecs, n_relation_vecs, id_section_offsets[4],       │
│          model_tag, pq_codebook_offset, ann_index_offset, checksum }  │
├──────────────────────────────────────────────────────────────────────┤
│ EntityVectors  [n_entity_vecs · stride]   (id-indexed, dense)         │
│ RelationVectors[n_relation_vecs · stride]                             │
│ PQ codebooks   (optional: m · 256 · subdim · f32)                     │
│ ANN index blob (DiskANN/Vamana adjacency, dict-id keyed — §6.4)       │
│   or HNSW graph (browser tier)                                        │
└──────────────────────────────────────────────────────────────────────┘
```

This mirrors the permutation file: a small RAM-resident header + metadata, large
payload left on disk/mmap, decoded lazily. The exact engine never opens this file
(feature-gated); removing the crate removes the file.

### 4.3 Avoiding the hot path

- The vector store and ANN index are **separate files / separate mmaps**; the triple
  permutations are untouched.
- The exact query path has **zero references** to `sparq-vectors` (feature-gated at the
  crate boundary — the optional crate depends on `sparq-core` for the dictionary, not
  vice-versa). This satisfies "ZERO perf/memory impact, trivially removable."
- Vector search is exposed as a *new* operator (e.g. a magic predicate
  `sparq:nearest`, or a `SERVICE`/function), producing candidate `ValueId`s that then
  feed the **exact** engine — so results are re-validated against real triples and the
  #Diverge gate still holds for whatever exact BGP wraps the candidates.

### 4.4 Bounding memory at billion-scale

- f32 dim-200 × 100M entities ≈ 80 GB → **store f16 on disk** (40 GB) and/or **PQ**
  (8–16 B/vec → ~1–2 GB of codes resident, full vectors on SSD, DiskANN-style).
- DiskANN keeps PQ codes RAM-resident for navigation + full f16 vectors on SSD for
  rerank → fits the €2.5k box exactly as the exact store does.
- **[novel] dict-locality PQ (§6.3):** train PQ codebooks **per dict section** (SO / S
  / O), so codes for entities that share lexical neighbourhood share a codebook →
  better reconstruction at the same byte budget.

---

## 5. Compute pipeline + WASM story

### 5.1 Where embeddings come from (offline batch; GPU optional)

1. **Export edges from sparq** in id space — literally a scan of one permutation →
   `(h_id, r_id, t_id)` triples (no string round-trip; the dictionary ids *are* the
   training ids). This is the cleanest possible KGE training input.
2. **Train offline** with PyKEEN (small graphs, evaluation), **DGL-KE** (multi-GPU), or
   **Marius** (billion-edge, one machine — matches sparq's philosophy). Or **import
   pretrained vectors** (e.g. public Wikidata embeddings, KGvec2go/PyKEEN dumps).
3. **Text vectors** computed by `ort`/`candle` over labels, in id order.
4. **Pack** into the `.spqv` file (vectors in id order, optionally PQ-encoded) and
   **build the ANN index** (Vamana/HNSW) over it.
5. sparq **mmaps `.spqv` zero-copy** at startup if the `vectors` feature is on.

GPU is used *only* in step 2 and only if available; **inference/serving is CPU+SSD
only** (lookup + ANN beam search), so the deployed engine needs no GPU — consistent
with the €2.5k single-box target.

### 5.2 Exposing to queries

A new approximate operator, **gated behind the feature**, e.g.:
- `?e sparq:nearestTo (?seed 10)` → top-10 entities by ANN to `?seed`'s vector;
- a text-entry point `sparq:textSearch("query string", 20)` → embed the string
  (candle/ort) → ANN → candidate `ValueId`s;
- the candidates flow into the normal exact engine as a `VALUES`-like bound set, so any
  surrounding BGP is evaluated exactly.

This composes cleanly with the **similarity-search** and **NL→SPARQL** features
(researched separately): this layer is their *retrieval primitive*; they own
query-understanding and ranking.

### 5.3 WASM

- **Vectors:** ship a **quantized** (`i8`/PQ) `.spqv` blob fetched separately, mirroring
  the exact engine's "small wasm + separately-fetched index blob" split (ARCHITECTURE
  §6). At browser scale (≤ a few M vectors) an **in-RAM HNSW** is fine.
- **Index:** **`usearch`** compiles to wasm32 and supports mmap-view + quantized
  vectors; **`candle`** compiles to wasm32 to embed the *query* text in-browser (so NL
  search needs no server). Both fit the small-bundle constraint better than `faiss`
  (C++) or `diskann_rs` (assumes a real filesystem).
- DiskANN's *disk* premise doesn't hold in the browser; for WASM use HNSW over a
  quantized blob held in an `ArrayBuffer`.

---

## 6. Proposed `sparq-vectors` crate design (established stack + novel ideas)

### 6.1 Crate shape (satisfies the opt-in contract)

- New workspace crate **`sparq-vectors`**, behind cargo feature `vectors` (and
  `vectors-wasm`). It **depends on `sparq-core`** (for the dictionary + id space +
  external-merge-sort + mmap block utilities); **nothing in `sparq-core`/`sparq-engine`
  depends on it**. Deleting the crate + feature removes the whole capability with no
  diff to the exact engine. Shipped with the accuracy + perf benchmarks of §7.
- Modules: `store` (the `.spqv` mmap format + id-indexed vector access), `ann`
  (Vamana on-disk / HNSW in-RAM behind a trait), `quant` (SQ/PQ, dict-section
  codebooks), `embed` (ort/candle text embedding; importers for PyKEEN/DGL-KE/Marius
  dumps), `sketch` (the training-free structural sketch, §6.5), `op` (the
  `sparq:nearest`/`textSearch` operators wiring candidates back into the exact engine).

### 6.2 The ANN choice & justification

- **Native:** **DiskANN/Vamana** (`diskann_rs`/infinilabs `diskann`, or a port of
  microsoft/DiskANN). Justification: it is the *only* mature ANN that is genuinely
  **external-memory** (PQ-in-RAM + vectors-on-SSD), so its resource profile matches
  sparq's out-of-core triple store; it hits billion-scale on one box at 95%+ recall@1;
  and its build (α-RobustPrune over batches) maps onto sparq's external merge-sort.
- **In-RAM / browser:** **HNSW** via **`usearch`** (mmap + WASM + quantization in one
  crate). Behind a `trait AnnIndex { build, search(&query, k) -> Vec<(ValueId, dist)> }`
  so the two are interchangeable and benchmarkable head-to-head.

### 6.3 [novel] Quantization exploiting dictionary locality

PQ codebooks are normally global. sparq's dictionary is **sorted into SO/S/O/P sections
with lexicographic locality** — entities near in id are often lexically/typed-related.
**Train a PQ codebook per section (or per id-range bucket)** so each codebook
specialises to a tighter sub-distribution → lower reconstruction error at the *same*
bytes/vector, or fewer bytes at equal recall. Bucket id is derivable from the id
itself (it's just a range check), so it costs nothing at query time. *Honest risk:*
gains depend on how much vector geometry actually correlates with lexical id order —
**measure reconstruction MSE per-section-PQ vs global-PQ** before committing.

### 6.4 [novel] Vamana graph keyed by dict ids, built on sparq's external-merge infra

DiskANN builds Vamana by inserting points and α-pruning their candidate edges. **Key
the adjacency list by dict `ValueId` and build it with sparq's existing external
merge-sort / paired-permutation builder** (ARCHITECTURE §3.2 "external merge-sort"):
the graph is "for each id, a sorted list of neighbour ids" — *structurally identical to
a permutation relation* (leading column = source id, payload = neighbour ids), so it
can be stored in the **same column-major, optionally-ZSTD'd block format** with the
same mmap/binary-search machinery. Benefits: one I/O + mmap + compression stack instead
of two; neighbour-id deltas compress with the same codecs; the graph file is a sibling
of the permutation files. *Honest risk:* ANN beam search is random-access
pointer-chasing (the bandwidth-bound concern of `data-structures.md` §4.1) — but it's
on the **opt-in approximate path**, not the exact hot path, so the regime constraint is
relaxed here by design.

### 6.5 [novel] Permutation-derived structural-sketch vectors (training-free)

Reuse features sparq **already computes** for the planner as a cheap, GPU-free,
*incrementally maintainable* embedding:
- **degree signatures** (in/out degree per node — free from 1-prefix counts,
  ARCHITECTURE §3.2 aggregated indexes);
- **characteristic-set membership** — sparq already builds
  `HashMap<BTreeSet<PredId>, stats>` for star cardinality (ARCHITECTURE §3.4); a node's
  predicate-set is a sparse high-signal structural descriptor;
- **predicate-context Roaring bitmaps** — the dense-predicate tier (`bit-level` P1)
  already has per-predicate subject/object bitmaps; a node's "which dense predicates
  touch me" bitvector is a cheap structural fingerprint.

Hash/project these (e.g. random-projection or a tiny learned MLP) into a fixed-dim
sketch vector. **Properties:** zero training, zero GPU, **updates incrementally as
triples are inserted** (the only truly-incremental embedding here), and serves as a
**cold-start vector** for entities with no trained embedding yet, or a **cheap fallback
modality** in late fusion. *Honest expectation:* lower link-prediction quality than
RotatE — its job is *coverage + incrementality + cost*, not beating SOTA; **measure it
as a baseline and as a fusion input**, not as a replacement for trained KGEs.

### 6.6 [novel] Hybrid structural+text via late fusion

Index structural vectors and text vectors as **two `.spqv` stores + two ANN indexes**.
At query time, search both, and **merge by reciprocal-rank fusion** (or a tunable
weighted score). Advantages over joint training: no retraining when either modality
changes, per-query modality weighting (NL query → weight text; "more like this entity"
→ weight structural), and clean removability. Offline **concat+reproject** stays
available as a higher-quality, less-flexible option to A/B against late fusion.

### 6.7 How it composes with the separately-researched features

- **Similarity search** → directly `sparq:nearest` over structural/text/hybrid vectors.
- **NL→SPARQL** → uses `embed` (candle/ort) + text ANN as its **entity/relation linking
  retriever**, returning `ValueId`s the generated SPARQL binds exactly.
- **Planner cardinality (answer-safe, §S7)** → a GNCE-style estimator can consume these
  same entity/relation vectors for path-cardinality estimation **without** ever
  affecting answers (planner-only). This is the *one* place embeddings touch the core,
  and it is provably answer-safe.

---

## 7. Benchmark plan

Two independent axes, mirroring the brief and sparq's measurement culture
(`BENCH-PROTOCOL`): **accuracy** (does the embedding capture the graph?) and
**performance** (does the index serve fast within memory?).

### 7.1 Accuracy — link prediction (the embedding quality gate)

- **Datasets:** **FB15k-237** and **WN18RR** (the de-facto KGE benchmarks); add a
  Wikidata subset (e.g. Wikidata5M) for scale-realism.
- **Metric:** **filtered MRR** and **Hits@{1,3,10}** under the standard
  corrupt-head/corrupt-tail protocol. **Targets to match** (RotatE paper / PyKEEN
  unified eval): RotatE FB237 MRR ~.338 / H@10 ~.533; WN18RR MRR ~.476 / H@10 ~.571;
  ComplEx WN18RR MRR ~.44–.48 retuned `[established]`.
- **Process:** import a PyKEEN/DGL-KE-trained model → load into `.spqv` → re-score
  link-prediction *through sparq's own vector store and distance kernels* → confirm the
  numbers reproduce (validates the storage/quantization didn't degrade quality).
- **Quantization ablation:** MRR/Hits@10 at f32 vs f16 vs SQ-int8 vs PQ-{8,16} B —
  quantify the recall cost of each memory tier (the §4.4 / §6.3 decision data).
- **Structural-sketch baseline (§6.5):** its standalone MRR + its lift as a fusion
  input vs trained-KGE-alone (does cheap structure help?).

### 7.2 Performance — retrieval & serving

- **Retrieval recall@k** vs a **brute-force exact-NN oracle** (recall@{1,10,100}) — the
  ANN-quality metric, separate from embedding quality.
- **Scales:** 1M, 100M, 1B vectors (dim 128–200), on the €2.5k box.
- **Metrics per scale:** **build time**; **RAM resident** + **SSD footprint**;
  **query latency p50/p99** at fixed recall (e.g. 0.95); **QPS** single- and
  multi-thread. **Targets:** DiskANN's published SIFT1B bar — >5000 QPS, <3 ms mean,
  95%+ recall@1 on 16 cores `[claimed]` — as the billion-scale yardstick.
- **Index A/B:** DiskANN/Vamana (on-disk) vs HNSW (in-RAM, `usearch`) vs IVF-PQ
  (`faiss-rs` oracle) on the recall/latency/RAM frontier; confirm the §6.4 dict-keyed
  Vamana build is no slower than a stock Vamana build.
- **WASM:** bundle size (target: small index ≤ a few MB) + in-browser query latency for
  an in-RAM quantized HNSW over a ≤1–5M-vector blob; candle text-embed latency in-browser.
- **Zero-impact proof (the contract):** build sparq **without** the `vectors` feature
  and confirm **byte-identical** exact-engine binary size / benchmark numbers vs main —
  the literal demonstration of "ZERO perf/memory impact, trivially removable."

### 7.3 Decision rules

- Ship **ComplEx or RotatE** structural + **bge-small/MiniLM** text by default; pick
  per-dataset by the §7.1 MRR table.
- Ship **DiskANN/Vamana** native, **HNSW (`usearch`)** in-RAM/WASM; default quantization
  **SQ-int8** (high recall, 4× cut), escalate to **PQ** only where §7.2 shows RAM is the
  binding constraint, validated by the §7.1 quantization ablation.
- Adopt **dict-locality PQ (§6.3)** only if its reconstruction-MSE A/B beats global PQ;
  adopt **dict-keyed Vamana (§6.4)** only if build time ≈ stock; keep the
  **structural sketch (§6.5)** as the incremental/cold-start fallback regardless.

---

## 8. Sources

**KG embedding models & evaluation.**
TransE — Bordes et al., NeurIPS 2013
<https://proceedings.neurips.cc/paper/2013/file/1cecc7a77928ca8133fa24680a88d2f9-Paper.pdf> ·
DistMult — Yang et al., ICLR 2015 <https://arxiv.org/abs/1412.6575> ·
ComplEx — Trouillon et al., ICML 2016 <https://arxiv.org/abs/1606.06357> ·
RotatE — Sun et al., ICLR 2019 <https://arxiv.org/pdf/1902.10197> ·
ConvE — Dettmers et al., AAAI 2018 <https://arxiv.org/abs/1707.01476> ·
RESCAL — Nickel et al., ICML 2011
<https://icml.cc/2011/papers/438_icmlpaper.pdf> ·
Unified large-scale eval (PyKEEN) — Ali et al., IEEE TPAMI 2021
<https://arxiv.org/abs/2006.13365>,
<https://backend.orbit.dtu.dk/ws/files/262633429/Bringing_Light_Into_the_Dark_A_Large_scale_Evaluation_of_Knowledge_Graph_Embedding_Models_under_a_Unified_Framework.pdf>

**Walk-based & GNN.**
RDF2Vec — Ristoski & Paulheim, ISWC 2016; family survey Portisch & Paulheim, *Semantic
Web* 2024 <https://content.iospress.com/articles/semantic-web/sw233514>,
<https://www.rdf2vec.org/> · pyrdf2vec <https://pyrdf2vec.readthedocs.io/> ·
node2vec — Grover & Leskovec, KDD 2016 <https://arxiv.org/abs/1607.00653> ·
DeepWalk — Perozzi et al., KDD 2014 <https://arxiv.org/abs/1403.6652> ·
R-GCN — Schlichtkrull et al., ESWC 2018 <https://arxiv.org/abs/1703.06103> ·
CompGCN — Vashishth et al., ICLR 2020 <https://arxiv.org/pdf/1911.03082> ·
GraphSAGE — Hamilton et al., NeurIPS 2017 <https://arxiv.org/abs/1706.02216>

**Training libraries / out-of-core.**
PyKEEN <https://github.com/pykeen/pykeen> · DGL-KE — Zheng et al., SIGIR 2020
<https://arxiv.org/abs/2004.08532>, <https://github.com/awslabs/dgl-ke> ·
Marius — Mohoney et al., OSDI 2021 <https://github.com/marius-team/marius> ·
GraphVite — Zhu et al., WWW 2019 <https://arxiv.org/abs/1903.00757>,
<https://github.com/DeepGraphLearning/graphvite> ·
AmpliGraph <https://github.com/Accenture/AmpliGraph>

**Text embeddings from Rust.**
candle <https://github.com/huggingface/candle> · ort (ONNX Runtime)
<https://docs.rs/ort> · fastembed-rs / embed_anything
<https://crates.io/crates/embed_anything> · Sentence-Transformers in Rust (Burn/ONNX/
candle) <https://dev.to/mayu2008/building-sentence-transformers-in-rust-a-practical-guide-with-burn-onnx-runtime-and-candle-281k>
· BGE/MiniLM models (HF: BAAI/bge-small-en-v1.5, sentence-transformers/all-MiniLM-L6-v2)

**Vector indexes & quantization.**
HNSW — Malkov & Yashunin, TPAMI 2018 <https://arxiv.org/abs/1603.09320> ·
DiskANN/Vamana — Subramanya et al., NeurIPS 2019
<https://suhasjs.github.io/files/diskann_neurips19.pdf>,
<https://www.microsoft.com/en-us/research/publication/diskann-fast-accurate-billion-point-nearest-neighbor-search-on-a-single-node/>,
<https://github.com/microsoft/DiskANN> ·
SPANN — Chen et al., NeurIPS 2021
<https://www.microsoft.com/en-us/research/wp-content/uploads/2021/11/SPANN_finalversion1.pdf>
· AiSAQ — 2024 <https://arxiv.org/pdf/2404.06004> ·
ScaNN (anisotropic) — Guo et al., ICML 2020 <https://arxiv.org/abs/1908.10396>,
<https://milvus.io/blog/a-brief-introduction-to-the-scann-index.md> ·
Product Quantization — Jégou et al., TPAMI 2011;
<https://www.pinecone.io/learn/series/faiss/product-quantization/>

**Rust ANN crates.**
usearch <https://docs.rs/usearch>, <https://github.com/unum-cloud/usearch> ·
hnsw_rs <https://crates.io/crates/hnsw_rs>,
<https://github.com/jean-pierreBoth/hnswlib-rs> ·
instant-distance <https://github.com/instant-labs/instant-distance> ·
arroy / hannoy
<https://blog.kerollmops.com/from-trees-to-graphs-speeding-up-vector-search-10x-with-hannoy>
· diskann_rs <https://crates.io/crates/diskann_rs>,
<https://github.com/lukaesch/diskann-rs>, infinilabs DiskANN (pure Rust)
<https://github.com/infinilabs/diskann> · faiss-rs <https://github.com/Enet4/faiss-rs>

**Internal cross-references.**
`research/data-structures.md` (§R6/R7 embeddings rejected for exact BGP; §S7 GNCE
answer-safe cardinality; §B6 static-core + append-delta; §4.1 bandwidth-bound /
pointer-chasing regime), `research/ARCHITECTURE.md` (§3.1 dictionary + SO/S/O/P
sections + tagged ValueId; §3.2 permutations + external merge-sort + aggregated
indexes; §3.4 characteristic sets; §6 WASM engine + index-blob split),
`research/bit-level-encoding.md` (Roaring dense-predicate tier P1; KG-embedding
rejection for exact joins), `research/inference-sota.md` (offline-train / serve split
precedent).
