# Feature research: VECTOR / semantic-retrieval / GenAI (epic sq-3183)

**Status:** deep-research, 2026-06-15. `[OPUS-4.8]` Authored under Opus 4.8 (re-review when
Fable returns). One of a set of sibling feature-research reports; this one owns the
**vector / ANN / KG-embedding / GenAI-retrieval** surface. Companion design records:
[`genai-design.md`](genai-design.md), [`genai-kg-embeddings-vectorindex.md`](genai-kg-embeddings-vectorindex.md),
[`genai-text-embedding-practices.md`](genai-text-embedding-practices.md),
[`genai-iri-similarity.md`](genai-iri-similarity.md), [`genai-nl-to-sparql.md`](genai-nl-to-sparql.md),
[`genai-benchmarks-and-synthesis.md`](genai-benchmarks-and-synthesis.md).

Invariants inherited from the engine (non-negotiable): every GenAI/vector feature is an
**opt-in crate**, trivially removable, **zero perf/memory/binary impact** on the default exact
engine; approximate signals **never serve BGP answers** (they feed retrieval / linking / planner
*estimates*, re-validated against exact triples); the shared substrate is the **dict u32 id-space**
— vectors, signatures, stats are keyed by dict id with no side join tables.

Tags: `[established]` peer-reviewed/SOTA with a primary source · `[claimed]` authors' own numbers ·
`[novel]` our proposal · `[measured]` a number sparq or a cited benchmark measured. No numbers are
fabricated; secondary reporting is marked.

---

## 1. What sparq does today (the baseline — so we can find GAPS)

Grounded in `crates/sparq-vectors` (+ `-sim`, `-introspect`, `-nlq`, `-text`) and the skills/research.

**`sparq-vectors` (the ANN crate).** Opt-in, separate crate; the default + wasm builds don't compile it.
- **Store:** one f32 embedding per dict **term id**, flat mmap `.spqv` (id → `header + id·stride`, O(1),
  zero-copy). Sparse by design (entities, not every literal). `StreamingWriter` for O(1)-RAM builds;
  `open_from_bytes` for filesystem-less. A **graph fingerprint** guards against dict-id-shifting rebuilds.
- **Search:** exact brute-force (`nearest_exact`), in-RAM **HNSW** (`VectorIndex`, via `instant-distance`),
  persistent on-disk **DiskANN/Vamana** (`.spqg`, build-once/open-no-rebuild). All cosine-identical.
- **Quantization:** `ScalarQuantizer` (f32→u8, 4×) and `ProductQuantizer` (PQ, asymmetric distance / ADC,
  8–32×) with a PQ-filter → full-precision re-rank recipe. (PQ candidate cache exists/tested but is **not
  yet wired into** `DiskAnnIndex.search_slots` — manual loop only.)
- **Embeddings:** produced **out-of-process** (provider-agnostic `Embedder` trait; opt-in `provider` /
  `embeddings` features carry an OpenAI-compatible `/v1/embeddings` client). `verbalize` /
  `embed_entities` render `<label>. a <type>. <description>` (Wikidata/BLINK shape, Weaviate prefix
  convention, multilingual, char-budgeted). Bulk import of external matrices (`.npy` / flat dump) via a
  row→dict-id contract.
- **Hybrid fusion:** `fuse_rrf` / `fuse_rrf_weighted` / `fuse_scores` (RRF k=60 + min-max relative-score),
  `hybrid_search` drives N retriever closures off one query and fuses by item. Designed to fuse text
  vectors with `sparq-sim` structural Jaccard.

**Adjacent retrieval estate (share the dict id-space):**
- `sparq-sim` — **training-free** structural similarity (pred-IDF-weighted Jaccard over SPO/OPS/OSP
  signatures); the no-model fallback / fusion partner.
- `sparq-introspect` — effective-schema mining + **exact characteristic sets** in dict-id space; already
  feeds the engine planner (`--features cs-planner`, `CsTable`). The hook a GNCE estimator would extend.
- `sparq-nlq` — lean NL→SPARQL (ground→generate→validate→execute→repair), LLM behind a record/replay trait.
- `sparq-text` — **BM25 + positional FTS in SPARQL via `text:` magic predicates** (`text:matches`,
  `text:phrase`, `text:near`, `text:score`), rewritten to inline `VALUES`. **This is the precedent for a
  `vec:` magic predicate — and the proof sparq can do query-language-level retrieval.**

### 1.1 The three GAPS that define this report

| Gap | Today | Why it matters |
|---|---|---|
| **G1 — `vec:` in SPARQL** | NONE. `sparq-vectors` is a standalone Rust library with **no SPARQL hook** (no `SERVICE`, function, or magic predicate), explicitly stated in the skill. `sparq-text` already has `text:`; vectors don't. | Every commercial RDF store exposes vectors **inside SPARQL** — GraphDB `similarity:` index queried in SPARQL `[established]`, Stardog Voicebox, Neo4j `db.index.vector.queryNodes`. sparq is the **only one without it.** Highest-leverage, lowest-novelty-risk gap. |
| **G2 — filtered-ANN** | NONE. The only filter in `ann.rs`/`diskann.rs` is self-exclusion (`n != id`). No predicate/payload-constrained search. | RDF vectors are attached to dict ids that **already have types/predicates** — "nearest `foaf:Person` to X" is the natural query. Filtered-ANN (ACORN/NaviX/PathFinder) is the 2024–25 frontier and an **RDF-native differentiator** because the constraint is just a BGP over the same store. |
| **G3 — KGE-for-cardinality (GNCE)** | NONE. `genai-design`/`benchmarks-synthesis` N1 *propose* it; not implemented. `cs-planner` uses exact characteristic sets, no learned/embedding estimator. | GNCE (arXiv:2303.01140) `[established]` shows KGE features predict conjunctive-query cardinality over RDF. Dict ids make KGE a colocated array → answer-safe planner lift with **zero core-engine answer changes.** Improves the **core product**, not just GenAI. |

---

## 2. Landscape (cite)

### 2.1 ANN / vector-index advances (the frontier sparq's docs predate or under-weight)

- **Filtered-ANN (predicate-constrained search) — the hottest 2024–25 area.**
  - **ACORN** (Patel et al., SIGMOD 2024) — *predicate-agnostic* filtered search: traverse the HNSW
    subgraph induced by nodes passing an **arbitrary** predicate; over-build degree (`M·γ`) so any
    predicate subgraph still approximates an HNSW. Beats pre-/post-filter baselines, handles
    low-selectivity filters and disjunctions. `[established]` <https://arxiv.org/html/2403.04871v1>,
    <https://dl.acm.org/doi/10.1145/3654923>
  - **NaviX** (Sehgal & Salihoglu, 2025) — adaptive heuristic on a *standard* HNSW: decide per-query
    whether to expand 1-hop vs 2-hop neighbours from local **selectivity estimates** (cheaper index than
    ACORN's over-build). `[established]`
  - **PathFinder** (2024/25) — materialise extra edges to preserve density under selective filters;
    supports **conjunctions + disjunctions** of predicates. `[established]` <https://arxiv.org/html/2511.00995v1>
  - Surveys/benchmarks: *Survey of Filtered ANN over Vector-Scalar Hybrid Data*
    <https://arxiv.org/html/2505.06501v1>; transformer-embedding FANN benchmark
    <https://arxiv.org/pdf/2507.21989>; *JAG: Joint Attribute Graphs* <https://arxiv.org/pdf/2602.10258>.
  - **Why RDF-native here:** the "scalar predicate" in these papers is exactly an RDF triple pattern
    (`?e a foaf:Person`, `?e :country :FR`). sparq can compute the candidate id-set **exactly** from its
    permutation indexes and feed it as the ACORN/NaviX visit-mask — a capability a bolt-on vector DB
    (which must mirror metadata) can't match.

- **Quantization frontier — RaBitQ.** RaBitQ (Gao & Long, SIGMOD 2024) `[established]`: JL-rotation then
  1-bit/dim with a **provably unbiased estimator, error O(1/√D)**; distance = bitwise AND + popcount
  (single-cycle). 768-d recall >94% at 1-bit; multi-bit 4/5/7-bit ≈ 90/95/99% recall **without** rerank
  (secondary, Elastic/Milvus). Shipped as Milvus `IVF_RABITQ`. **Frontier vs sparq's current SQ(4×)/PQ —
  RaBitQ is the modern binary tier sparq lacks.** <https://dl.acm.org/doi/pdf/10.1145/3654970>,
  <https://github.com/VectorDB-NTU/RaBitQ-Library>, <https://www.elastic.co/search-labs/blog/rabitq-explainer-101>,
  <https://milvus.io/docs/ivf-rabitq.md>

- **Multi-vector / late interaction (ColBERT family).** ColBERTv2 (token-level multi-vector, MaxSim) +
  **PLAID** (centroid pruning) `[established]` <https://arxiv.org/pdf/2112.01488>; **MUVERA** (NeurIPS 2024)
  — fixed-dimensional encodings turn multi-vector retrieval into **single-vector MIPS** (use any ANN
  index) <https://proceedings.neurips.cc/paper_files/paper/2024/file/b71cfefae46909178603b5bc6c11d3ae-Paper-Conference.pdf>;
  **WARP** (2025) efficient multi-vector engine <https://arxiv.org/pdf/2501.17788>. *Honest:* high effort,
  storage-heavy; lower priority for an entity-retrieval store than for document RAG.

- **Hybrid dense+sparse (BM25/SPLADE + vectors, RRF/fusion).** SPLADE learned-sparse + dense (Contriever)
  fused by score-normalised average / RRF **consistently beats either alone on BEIR/MS-MARCO**
  `[established]` <https://en.wikipedia.org/wiki/Learned_sparse_retrieval>,
  <https://www.emergentmind.com/topics/dense-sparse-hybrid-retrieval>. RRF k=60 is the de-facto default
  (Azure/Elastic/ParadeDB). **sparq already has both signals** (`sparq-text` BM25 + `sparq-vectors`
  cosine) and the fuse helpers — but fusion is **library-only, not exposed in SPARQL** (see G1).

- **Cross-encoder / reranking (two-stage).** Retrieve N∈[50,200] cheaply, rerank with a query-aware
  cross-encoder: +up-to-10 nDCG over bi-encoders on MS-MARCO; the standard production pipeline
  `[established]` <https://towardsdatascience.com/advanced-rag-retrieval-cross-encoders-reranking/>,
  <https://arxiv.org/pdf/2510.04757>. sparq's RRF over-fetch is stage-1; a reranker seam is the missing
  stage-2 (model out-of-process, like embeddings).

- **Index families / streaming.** HNSW (in-RAM recall champ) vs IVF-PQ (memory-minimal) vs DiskANN/Vamana
  (external-memory billion-scale) — all already in sparq's design. **Incremental/streaming updates** remain
  the weak spot of graph indexes (HNSW insert cheap, delete awkward; DiskANN build is batch). GPU ANN
  (CAGRA) is out of scope for the CPU+SSD single-box target. ann-benchmarks is the recall-QPS frontier
  reference <https://github.com/erikbern/ann-benchmarks>.

### 2.2 RDF + vectors (the differentiator)

- **KG embeddings colocated with dict ids.** RotatE / ComplEx / TransE one-vector-per-entity → a single
  id-indexed lookup, identical in shape to sparq's dict (covered exhaustively in
  `genai-kg-embeddings-vectorindex.md`). FB15k-237 / WN18RR are the link-prediction gates; PyKEEN/Marius
  train offline, sparq imports + serves. `[established]`
- **Vector-augmented SPARQL — competitors have it, sparq doesn't (G1).** GraphDB *Semantic similarity
  search*: a similarity index built from a SPARQL SELECT, then **queried from SPARQL** (real/complex/binary
  semantic vectors, IDF term-weighting) `[established]`
  <https://graphdb.ontotext.com/documentation/11.3/semantic-similarity-searches.html>. Stardog **Voicebox**:
  embed ontology terms → vector-match → SPARQL gen `[established]`. Neo4j vector index + Cypher
  `db.index.vector.queryNodes`. The W3C sparql-dev *Inventory of SPARQL 1.1 extensions* tracks vendor
  extension functions <https://github.com/w3c-cg/sparql-dev/wiki/Inventory-of-existing-extensions-to-SPARQL-1.1>.
- **KGE-for-query-optimisation — GNCE (G3).** arXiv:2303.01140 `[established]`: KG-embedding features
  predict conjunctive-query cardinality over RDF (q-error 1.2–4.2 vs CSET 1e29–1e47 in the in-repo
  data-structures note). Answer-safe (planner-only). sparq's exact characteristic-set table is the natural
  feature substrate.
- **RAG-over-KG / GraphRAG / HybridRAG.** Microsoft GraphRAG (community summaries; local = vector+traversal,
  global = map-reduce) + LazyGraphRAG (~0.1% index cost) `[established]`; **HybridRAG** (arXiv:2408.04948)
  fuses VectorRAG + GraphRAG contexts — 2025 reporting ~85% vs ~70% accuracy for vector-only on
  knowledge-intensive multi-hop tasks (secondary)
  <https://arxiv.org/pdf/2408.04948>, <https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/>.
  **Almost all GraphRAG is property-graph/Cypher + LLM-built graphs with weak/no formal semantics** — the
  opening for an RDF+OWL/N3, out-of-core, **provenance-carrying** GraphRAG.
- **Provenance-carrying grounded answers.** SPARQL `GRAPH` + RDF-star justification → every answer triple
  carries source + (for inferred triples) rule + premises. The antidote to "trust-me" RAG; KGs reduce LLM
  hallucination (survey arXiv:2311.07914) `[established]`.

### 2.3 Commercial vector-DB features (what sparq's vector query should gain)

Qdrant 2025: **score-boosting reranking** (blend similarity + business signals), **strict filtering during
HNSW traversal** (= filtered-ANN, G2), **named vectors + multivectors**, multitenancy
<https://qdrant.tech/blog/2025-recap/>. Weaviate: **native hybrid (BM25+vector, configurable α)**, named
vectors, built-in **multi-tenancy** <https://cipherprojects.com/blog/posts/weaviate-vs-qdrant-vector-database-comparison-2025/>.
Across Qdrant/Weaviate/Milvus/pgvector/Vespa/LanceDB the recurring feature set is: **payload/metadata
filtering, hybrid search, reranking, named/multi-vector, quantization, metadata indexes**. sparq has
quantization + (library) hybrid; it lacks **payload filtering (G2), in-query hybrid/rerank (G1), named
vectors**.

---

## 3. CANDIDATE FEATURE TABLE

FIT legend: `clear-fit:<component>` (slots into an existing crate) · `new-component-but-fits` (new crate/module,
consistent with the architecture) · `ambiguous-ask-user`. Impact 1–5 (5 = product-defining). Effort S/M/L.
**Bold rows = the RDF-native differentiators** called out in the brief.

| # | Feature | FIT | Impact | Effort | Rationale + source |
|---|---|---|--:|:--:|---|
| **V1** | **`vec:` magic predicate — vector KNN inside SPARQL** (`?e vec:nearest (?seed 10)`, `vec:textNearest ("query" 20)`, `vec:score ?s`), rewritten to `VALUES` like `text:`; candidates feed the **exact** engine | **clear-fit:sparq-vectors (+`text:`-style rewriter; new `sparq-vectors`↔engine seam)** | **5** | **M** | The single biggest gap (G1). `text:` already proves the pattern (`sparq-text/src/rewrite.rs`). Every competitor exposes vectors in SPARQL — GraphDB `similarity:`, Stardog, Neo4j; sparq is the only one without. Closes the headline "vectors aren't a SPARQL citizen" gap. `[established]` GraphDB docs; in-repo `genai-kg-embeddings-vectorindex.md` §4.3/5.2 |
| **V2** | **Filtered-ANN over dict-ids** — predicate-constrained KNN ("nearest `foaf:Person` to X"); compute the exact allowed-id set from permutation indexes, use it as an ACORN/NaviX visit-mask in HNSW/Vamana | **clear-fit:sparq-vectors (`ann.rs`/`diskann.rs`)** | **5** | **M** | RDF-native differentiator (G2). The "scalar filter" in ACORN/NaviX **is** an RDF triple pattern; sparq computes it exactly from its own indexes — a bolt-on vector DB must mirror metadata and can't. The natural way to combine `vec:` with a BGP. `[established]` ACORN SIGMOD'24 (arXiv:2403.04871); NaviX 2025; survey arXiv:2505.06501 |
| **V3** | **KGE-for-cardinality (GNCE) planner hook** — answer-safe learned cardinality from colocated KGE/CS features feeding `cs-planner` | **clear-fit:sparq-introspect + sparq-engine `cs-planner` (opt-in)** | **5** | **L** | RDF-native differentiator (G3). Improves the **core engine** (join ordering), not just GenAI; provably answer-safe (planner-only). `cs-planner`/`CsTable` is the existing seam. GNCE q-error 1.2–4.2 vs CSET blow-up. `[established]` GNCE arXiv:2303.01140; in-repo data-structures.md §S7 |
| **V4** | **In-query hybrid + RRF in SPARQL** — fuse `text:` BM25 + `vec:` cosine (+ `sparq-sim`) at the query layer (`vec:hybrid`/weighted-RRF), not just the Rust library | clear-fit:sparq-vectors `fuse.rs` + the V1 rewriter | 4 | M | Dense+sparse hybrid beats either alone on BEIR/MS-MARCO `[established]`; sparq already has **both signals + the fuse helpers** — only the SPARQL exposure is missing. Weaviate/Qdrant make this a first-class query. <https://en.wikipedia.org/wiki/Learned_sparse_retrieval>; in-repo `genai-text-embedding-practices.md` §4 |
| **V5** | **RaBitQ binary quantization tier** — 1-bit + popcount with the unbiased estimator; multi-bit 4/5/7 levels; replaces/augments the binary path next to SQ/PQ | clear-fit:sparq-vectors `quant.rs` | 4 | M | Modern binary tier sparq lacks; AND+popcount is single-cycle, 768-d recall >94% at 1-bit, theoretical error bound. Closes the quantization frontier vs Milvus `IVF_RABITQ`. `[established]` RaBitQ SIGMOD'24 (10.1145/3654970) |
| **V6** | **Cross-encoder rerank seam (stage-2)** — over-fetch N via ANN/RRF, rerank by an out-of-process reranker behind a trait (mirrors the `Embedder`/`Llm` seam) | clear-fit:sparq-vectors (new `rerank` trait) | 3 | S | Standard two-stage RAG; +up-to-10 nDCG over bi-encoders, K∈[50,200] `[established]`. Cheap, model out-of-process, matches sparq's "model lives outside the engine" discipline. <https://arxiv.org/pdf/2510.04757> |
| **V7** | **Wire the PQ candidate cache into `DiskAnnIndex.search_slots`** — finish the designed-but-unwired PQ-filter → full-precision rerank loop inside the on-disk index | clear-fit:sparq-vectors `diskann.rs` | 3 | S | Already-built/tested `quant.rs` cache is **not yet in** `search_slots` (skill "DiskANN honest scope"); finishing it makes DiskANN truly external-memory at billion-scale. Low-risk completion of existing work. In-repo skill + crate beads |
| **V8** | **Named / multi-vector per entity** — store >1 vector per dict id (e.g. structural KGE + text + per-language) under named slots; per-query slot selection | new-component-but-fits (`.spqv` format ext) | 3 | M | Commercial table stakes (Qdrant/Weaviate named vectors); the late-fusion design (`genai-kg-embeddings-vectorindex.md` §6.6) wants ≥2 modalities. Currently one f32/id. Per-language stores are the Wikidata pattern. <https://qdrant.tech/blog/2025-recap/> |
| **V9** | **RDF-native GraphRAG retriever** — `vec:`/filtered-ANN seed entities → exact graph-neighbourhood expansion (SPARQL) → provenance-carrying context pack, all local/WASM | new-component-but-fits (compose V1+V2+`sparq-nlq`/`-introspect`) | 4 | L | GraphRAG today is property-graph + LLM-built, weak semantics; sparq offers RDF+OWL/N3 reasoning + **provenance** + browser-local. HybridRAG (vector+graph) ~85% vs ~70% vector-only (secondary). The product-defining composition. `[established]` MS GraphRAG; arXiv:2408.04948 |
| V10 | **MUVERA fixed-dim encoding for late-interaction** — if multi-vector ever needed, encode to single-vector MIPS so the existing ANN index serves it | ambiguous-ask-user (only if doc-RAG enters scope) | 2 | L | Multi-vector (ColBERT) is doc-retrieval-shaped, not entity-shaped; MUVERA avoids a bespoke MaxSim index. Park until a doc-RAG use-case is confirmed. `[established]` MUVERA NeurIPS'24; WARP 2025 |
| V11 | **Streaming / incremental ANN inserts** — append-only `.spqv` insert + periodic offline Vamana rebuild (the static-core + delta-overlay pattern) | clear-fit:sparq-vectors + sparq-core delta pattern | 2 | M | Graph-index updates are the known weak spot; `sparq-sim`'s structural sketch is the only truly-incremental embedding (cold-start). Match the engine's M5 delta-overlay. `[established]` (general ANN) |

---

## 4. Top recommendations + prioritisation

**Tier 1 — the three RDF-native differentiators (do these first; they are why sparq-vectors beats a bolt-on vector DB):**

1. **V1 `vec:` in SPARQL** (impact 5, effort M) — the headline gap; the `text:` rewriter is a working
   template, and it's the prerequisite that makes V2/V4 expressible. **Start here.**
2. **V2 filtered-ANN over dict-ids** (5, M) — the differentiator: the predicate constraint is a BGP sparq
   evaluates exactly, fed as the ACORN/NaviX mask. Lands naturally as the way V1 composes with a graph pattern.
3. **V3 KGE-for-cardinality / GNCE** (5, L) — the one that improves the **core engine** (answer-safe join
   ordering), reusing the `cs-planner` seam. Higher effort/research-risk → sequence after V1/V2 but flag as
   the strategic core-positive bet.

**Tier 2 — close the commercial feature parity (cheap, high-leverage):**
4. **V4 in-query hybrid+RRF** — both signals + fuse helpers already exist; only SPARQL exposure is missing
   (rides on V1).
5. **V7 wire the PQ cache into DiskANN** — finish existing, tested code (effort S).
6. **V5 RaBitQ** + **V6 cross-encoder rerank seam** — modern quantization + the standard stage-2; both
   models/codecs stay out-of-process.

**Tier 3 — product-defining composition:**
7. **V9 RDF-native, provenance-carrying, browser-local GraphRAG** — composes V1+V2 with `sparq-nlq`/
   `-introspect`/reasoning; the strongest market story (no competitor runs store+retrieval+reasoning fully
   in-browser), but L effort and depends on Tier-1.

**Park:** V8 named-vectors (do when late-fusion needs ≥2 modalities), V10 MUVERA (only if doc-RAG enters
scope), V11 streaming inserts (sketch fallback covers cold-start for now).

**Non-canonical timing note (per project env):** any latency/throughput figures measured on the EC2 work
box are NON-canonical; gate on the crate's deterministic recall/q-error tests, not wall-clock here.

---

## 5. Sources

ANN / filtered-ANN: ACORN SIGMOD'24 <https://arxiv.org/html/2403.04871v1>, <https://dl.acm.org/doi/10.1145/3654923> ·
PathFinder <https://arxiv.org/html/2511.00995v1> · Survey of Filtered ANN <https://arxiv.org/html/2505.06501v1> ·
FANN benchmark <https://arxiv.org/pdf/2507.21989> · JAG <https://arxiv.org/pdf/2602.10258> ·
ann-benchmarks <https://github.com/erikbern/ann-benchmarks>.
Quantization: RaBitQ SIGMOD'24 <https://dl.acm.org/doi/pdf/10.1145/3654970>, <https://github.com/VectorDB-NTU/RaBitQ-Library>,
<https://www.elastic.co/search-labs/blog/rabitq-explainer-101>, Milvus IVF_RABITQ <https://milvus.io/docs/ivf-rabitq.md>.
Multi-vector: ColBERTv2 <https://arxiv.org/pdf/2112.01488> · MUVERA NeurIPS'24
<https://proceedings.neurips.cc/paper_files/paper/2024/file/b71cfefae46909178603b5bc6c11d3ae-Paper-Conference.pdf> ·
WARP <https://arxiv.org/pdf/2501.17788>.
Hybrid/sparse + rerank: SPLADE / learned-sparse <https://en.wikipedia.org/wiki/Learned_sparse_retrieval> ·
dense-sparse hybrid <https://www.emergentmind.com/topics/dense-sparse-hybrid-retrieval> ·
cross-encoder rerank <https://towardsdatascience.com/advanced-rag-retrieval-cross-encoders-reranking/>,
<https://arxiv.org/pdf/2510.04757>.
RDF + vectors: GNCE arXiv:2303.01140 <https://arxiv.org/abs/2303.01140> ·
GraphDB similarity search <https://graphdb.ontotext.com/documentation/11.3/semantic-similarity-searches.html> ·
W3C SPARQL-extension inventory <https://github.com/w3c-cg/sparql-dev/wiki/Inventory-of-existing-extensions-to-SPARQL-1.1> ·
HybridRAG arXiv:2408.04948 <https://arxiv.org/pdf/2408.04948> · LazyGraphRAG
<https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/> ·
KGs reduce hallucination arXiv:2311.07914 <https://arxiv.org/abs/2311.07914>.
Commercial: Qdrant 2025 recap <https://qdrant.tech/blog/2025-recap/> ·
Weaviate vs Qdrant 2025 <https://cipherprojects.com/blog/posts/weaviate-vs-qdrant-vector-database-comparison-2025/>.
Internal: `research/genai-kg-embeddings-vectorindex.md`, `genai-text-embedding-practices.md`,
`genai-benchmarks-and-synthesis.md` (N1/GNCE), `genai-design.md`; `crates/sparq-vectors`,
`crates/sparq-text/src/rewrite.rs` (the `text:` precedent), `crates/sparq-introspect` (`cs-planner` seam).
