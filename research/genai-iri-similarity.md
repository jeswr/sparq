# Similarity Search over IRIs / Entities in `sparq`

**Status:** living research document (in progress). Last updated 2026-06-09.
**Scope:** literature survey + concrete opt-in design for entity/IRI similarity search in `sparq`, the Rust dictionary-encoded RDF + SPARQL 1.1 engine.

> Design constraint repeated throughout: any feature here ships as a **separate, feature-gated crate** (`sparq-sim`), trivially removable, with **zero perf/memory impact on the default engine**, and benchmarked for **both accuracy and performance**. The core engine never links a vector index, an embedding model, or a tokenizer unless the `sim` feature is enabled.

---

## 0. Executive framing

"Similarity search over IRIs" is an overloaded phrase. It really means three distinct retrieval problems that callers conflate:

1. **Lexical similarity** — "find entities whose labels/local-names look like this string" (typo-tolerant autocomplete, entity linking, dedup). Signal lives in `rdfs:label` / `skos:prefLabel` / `altLabel` / `dc:description` and in the IRI local name.
2. **Semantic (textual) similarity** — "find entities that *mean* something like this text," via sentence-embedding vectors over labels+context. Signal is in a pretrained language model.
3. **Structural / relational similarity** — "find entities that play the same *role* in the graph" (same neighbours, same predicates, analogy). Signal is in the graph topology itself — exactly what `sparq`'s 6 permutation indexes already encode.

A serious system offers all three and a hybrid re-ranker. The big architectural insight for `sparq`: **#3 is nearly free** because the structural features are already materialised as sorted `[u32;3]` permutation indexes keyed by dict id. That is our differentiator and is developed in §7.

---

## 1. What TopQuadrant & Bergwinkl actually did

### 1.1 TopQuadrant — TopBraid EDG "TopBraid AI Service"

TopQuadrant added a **vector-store-backed semantic search** to TopBraid EDG (the "TopBraid AI Service"), shipping prominently in EDG 8.2+ ("Knowledge Graphs as the Foundation for Secure AI"). Concretely, from the EDG 8.3 reference docs (`TopBraidAIServiceSection`):

- **Vector DB:** **Weaviate** is the underlying vector store. The docs expose "the underlying Weaviate index configuration" and point at Weaviate's **HNSW** index parameters — so it's HNSW under the hood, configurable via a JSON "Vector Store Config."
- **Embeddings:** pluggable **external embedding service** via an "Embedding Service URL" + "Embedding Service Model." Example given is OpenAI `text-embedding-3-small`; any OpenAI-compatible endpoint (incl. local) works. So **no proprietary RDF embedding** — they embed *text* derived from resources.
- **What gets embedded:** text rendered from each asset (labels/descriptions/glossary text). SHACL is the *schema layer* that determines which properties exist and are "display"/"search" relevant; EDG is wholly SHACL-described (since EDG 5.2 SHACL is the modeling language for classes/properties), so the text materialised per node is SHACL-driven, not the vectors themselves.
- **Features built on it:** **AutoClassifier** (suggest taxonomy/glossary terms for an asset by vector similarity) and **Crosswalks** (align/link assets across collections by similarity). These are *governance* features — entity linking, glossary mapping, dedup — not raw kNN APIs.
- **Positioning:** "knowledge graphs as the foundation for secure AI" — i.e. RAG/grounding where the KG supplies trustworthy structured context and the vector store supplies fuzzy recall.

**Takeaways for us:** (a) the pragmatic, productised answer is *external text embeddings + HNSW (Weaviate)*; (b) SHACL is used to decide *what text represents an entity*; (c) the killer apps are classification/linking/crosswalk, not a bare `similar()` call. We can reproduce (a) and (b) cheaply, and we can do better on *structural* similarity (§7) which Weaviate-style text embeddings ignore.

Sources:
- https://www.topquadrant.com/doc/8.3/reference/TopBraidAIServiceSection.html
- https://www.topquadrant.com/resources/topquadrant-launches-topbraid-edg-8-2-knowledge-graphs-as-the-foundation-for-secure-ai/
- https://www.topquadrant.com/doc/9.1/user_guide/guidance_specific_to_asset_collection_type/working_with_ontologies/shacl_enablement.html

### 1.2 Thomas Bergwinkl (bergi / bergos, Zazuko CTO)

Bergwinkl is the rdfjs-community engineer behind RDF-Ext / RDF/JS. His directly relevant artefact is **`github.com/bergos/embedding-server`**: a small **FastAPI** service that wraps **Hugging Face sentence-transformer models** behind an **OpenAI-compatible `/v1/embeddings` API**, Dockerised (CPU PyTorch), models chosen via a `MODELS` env var, models baked in at image-build time.

The significance is architectural, and it matches TopQuadrant's choice: in the rdfjs world the idiomatic path to entity embeddings is **"render text per resource → call a local OpenAI-compatible sentence-transformer endpoint → store/search vectors."** `embedding-server` is exactly the local, self-hosted embedding endpoint that a SPARQL pipeline (his other RDF/JS data-processing work — graph traversal, PageRank, shortest-path over RDF/JS) would POST node text to. It deliberately decouples the embedding model from the RDF stack via a stable HTTP contract — the same decoupling we adopt (§6: embeddings are produced out-of-process; `sparq-sim` only *stores and searches* them, keyed by dict id).

Bergwinkl's broader RDF/JS "data processing" writing (bergnet.org, 2023–2024) argues RDF/JS should support graph *algorithms* (shortest path, PageRank) over the abstract data model — i.e. structural signals — which aligns with our §7 structural-similarity proposal.

Sources:
- https://github.com/bergos/embedding-server
- https://www.bergnet.org/2024/01/rdfjs-data-processing/
- https://github.com/bergos
- https://www.w3.org/community/rdfjs/author/tbergwin/

**Net:** Neither party invented an RDF-native embedding. Both converged on *external sentence-transformer text embeddings + HNSW*. The open ground — RDF-native **structural** similarity and exploiting existing triple indexes — is where `sparq` can be novel.

---

## 2. Lexical similarity over IRIs / labels

Lexical methods are cheap, training-free, language-agnostic-ish, incrementally updatable, and **frequently beat embeddings** for: exact-ish entity lookup, autocomplete, code/identifier-like IRIs, rare proper nouns and IDs (where LM tokenizers smear meaning), and any setting where the user typed a near-exact label. They also need no GPU and run in WASM trivially.

Building blocks:

- **URI local-name tokenization.** Strip namespace, split the local name on `camelCase`, `snake_case`, `kebab-case`, `/`, `#`, digits; expand known prefixes; lowercase/fold. (`fooBar_baz/Qux` → `foo bar baz qux`.) This recovers human words from opaque IRIs and is the single highest-ROI preprocessing step.
- **Character n-gram / trigram index.** Index each label's character trigrams; candidate = labels sharing ≥t trigrams; rank by Jaccard/overlap. This is what Postgres `pg_trgm` and many entity linkers use; robust to typos and morphological variation. Maps cleanly to an inverted index keyed by dict id.
- **Edit distance (Levenshtein/Damerau, Jaro-Winkler).** Use as a *re-ranker* on the trigram candidate set, not as a primary index (O(n·m) per pair). Jaro-Winkler is good for short names; Damerau handles transpositions.
- **BM25 / TF-IDF over label+description text.** Treat each entity as a "document" = concatenation of its `prefLabel`+`altLabel`+`description`+local-name. Classic sparse retrieval; excellent for longer descriptive text and multi-word queries; gives strong, explainable lexical recall. Tantivy (Rust Lucene) is the natural engine.
- **Phonetic / fuzzy** (Soundex/Metaphone) — niche, mostly for person/place names.

**When lexical beats embeddings (BEIR, Thakur et al. 2021):** the BEIR zero-shot benchmark showed dense retrievers trained on one domain frequently *underperform BM25* out-of-domain — BM25 "nails exact-match queries but fails on paraphrase/synonymy," while dense returns semantically adjacent docs that lack the exact token. Strongest BM25 wins: out-of-domain corpora, rare entities/IDs, exact-match intent, low-resource languages. Even where modern dense models now lead in-domain, **hybrid (BM25 + dense) remains the production standard** (§5). BEIR: https://arxiv.org/abs/2104.08663

Rust libraries: `tantivy` (BM25, also has an ngram tokenizer), `strsim` (Levenshtein/Jaro-Winkler/Damerau), `fst` (Levenshtein-automaton fuzzy over a sorted term set — *ideal* because our dict is already a sorted term set), `rust-stemmers`, `whatlang` (lang detect). All compile to WASM.

> `fst` deserves emphasis: `sparq`'s term dictionary is a sorted set of strings → build a `fst::Map` from local-name/label → dict id and run **Levenshtein-automaton** fuzzy queries directly against it. Near-zero extra memory, no separate index, WASM-friendly. This is a "free" lexical layer (§7.1).

---

## 3. Embedding-based entity similarity

Three families, capturing *different* notions of "similar." Pick per use-case; ideally fuse (§5).

### 3.1 Textual / sentence-transformer embeddings (lexico-semantic)
Render a text surface per entity (labels + types + key descriptive predicates, optionally 1-hop context) and embed with a sentence-transformer (e.g. `all-MiniLM-L6-v2` 384-d, `bge-small/base`, `e5-small`, `gte`). Captures **meaning of labels/descriptions**, cross-lingual-ish with multilingual models, zero graph structure. This is what TopQuadrant and Bergwinkl's `embedding-server` do.
- Pros: best for "find entities about X" text queries, semantic dedup, RAG grounding. Cheap incremental update (embed only changed nodes). No training.
- Cons: ignores graph topology; quality depends on label/description richness; needs a model (GPU helps but MiniLM is fine on CPU).

### 3.2 RDF2Vec (walk-based, structural-lexical)
Random/biased walks over the RDF graph → sequences of IRIs → word2vec (skip-gram). Captures **graph context / neighbourhood co-occurrence**. Mature ecosystem: `rdf2vec.org`, **pyRDF2Vec** (Vandewiele et al., ESWC 2023). Walk strategies matter a lot (Cochez et al. biased walks; "Walk Extraction Strategies for Node Embeddings", ISWC 2021 — n-gram walks best on average for node classification). RDF-star2vec extends to RDF-star.
- Pros: purely structural, label-free (works on opaque IRIs), strong on node classification/clustering.
- Cons: **transductive** — adding nodes/edges requires re-walking + re-training (poor incremental story); walk sampling is the cost driver; no text semantics.
- Note: "More is not Always Better" (Ristoski et al. / Portisch) shows A-box materialization can *hurt* RDF2Vec — relevant to our "inference + similarity" idea (§7.4): materialize selectively.

### 3.3 Knowledge-graph embeddings (relational / link-prediction)
Train per-entity and per-relation vectors so that scoring functions predict triples:
- **TransE** (translational, `h + r ≈ t`) — simple, fast, struggles with 1-N/symmetry.
- **DistMult** (bilinear diagonal) — symmetric relations only.
- **ComplEx** (complex bilinear) — handles asymmetry/antisymmetry.
- **RotatE** (rotation in complex space, Sun et al. 2019) — the only one of these that models **symmetry/antisymmetry + inversion + composition** simultaneously (TransE can't do symmetry; DistMult can't do antisymmetry/inversion; ComplEx does symmetry/antisymmetry/inversion but not composition). Strong on WN18RR (symmetry/composition-heavy) and FB15k-237.
Capture **relational/role similarity** and enable analogy & link prediction. Tooling: PyKEEN, DGL-KE, AmpliGraph. RotatE: https://arxiv.org/abs/1902.10197 ; KGE link-prediction benchmark survey: https://www.mdpi.com/2079-9292/11/23/3866
- Pros: best "same role/relation" similarity; link prediction for free; compact.
- Cons: transductive (new entity = retrain or fold-in); training cost scales with |triples|·epochs; need negative sampling; no text.

### 3.4 GNN encoders (R-GCN, CompGCN) and node2vec/DeepWalk
- **node2vec/DeepWalk**: like RDF2Vec but on the (typed-or-not) graph; transductive, similar tradeoffs.
- **R-GCN / CompGCN**: relation-aware message passing; can be **inductive** (embed unseen nodes from features/neighbourhood) and incorporate node features (e.g. the §3.1 text vectors as input features → fuses text+structure). Higher training/infra cost; heaviest option.

### 3.5 What each captures — summary table

| Method | Captures | Needs text? | Inductive / incremental? | Train cost | WASM-friendly to *use*? |
|---|---|---|---|---|---|
| Sentence-transformer (§3.1) | label/description meaning | yes | yes (re-embed changed node) | none (pretrained) | inference heavy, but vectors precomputed → search is WASM-fine |
| RDF2Vec (§3.2) | neighbourhood co-occurrence | no | no (retrain) | medium (walks+w2v) | search-only fine |
| TransE/DistMult/ComplEx/RotatE (§3.3) | relational role, link prediction | no | no (fold-in/retrain) | medium–high | search-only fine |
| node2vec/DeepWalk (§3.4) | proximity/community | no | no | medium | search-only fine |
| R-GCN/CompGCN (§3.4) | structure + features, inductive | optional | yes (inductive) | high | hard |

**Recommendation for v1:** lead with §3.1 (text) for the productised path (matches TopQuadrant/Bergwinkl, best incremental story, no training), offer §3.3 ComplEx/RotatE as an *optional* "structural embeddings" mode for relational similarity, and provide the **free** structural features from permutation indexes (§7) as a third, training-free structural signal.

Sources:
- RDF2Vec: https://www.rdf2vec.org/ ; pyRDF2Vec (ESWC 2023) https://link.springer.com/chapter/10.1007/978-3-031-33455-9_28 ; walk strategies (ISWC 2021) https://link.springer.com/chapter/10.1007/978-3-030-87101-7_8 ; arXiv https://arxiv.org/pdf/2009.04404
- A-box materialization caveat: https://arxiv.org/pdf/2009.00318
- Vec2SPARQL (SPARQL + KG embeddings): https://ceur-ws.org/Vol-2275/paper12.pdf

---

## 4. Vector index internals in Rust

Goal: an ANN index that (a) keys vectors by `u32` dict id, (b) supports `sparq`'s **out-of-core mmap** design, (c) optionally compiles to WASM, (d) supports incremental insert/delete (RDF graphs mutate).

### 4.1 Candidate libraries

| Library | Algo | On-disk / mmap | Incremental | Quantization | WASM | Notes |
|---|---|---|---|---|---|---|
| **`arroy`** (Meilisearch, ex-Spotify Annoy) | random-projection trees | **yes, LMDB-backed mmap** | yes (LMDB insert/remove) | **binary quant** (7× faster index, big disk save); filtered ANN | possible (LMDB caveat) | best fit for out-of-core + filtered search by id subset |
| **`hannoy`** (Meilisearch, successor) | **HNSW**, DiskANN-inspired, LMDB | **yes, mmap** | yes, online updates | yes | — | now Meilisearch default; better recall/latency than arroy |
| **`usearch`** | HNSW | yes (serialize/mmap view) | yes | f16/i8/b1 quant, custom metrics | **yes (has WASM)** | single-file, many langs, very portable; strong default choice |
| **`hnsw_rs`** (jean-pierreBoth) | HNSW | mmap dump/reload | yes | — | likely | mature, ann-benchmarks-validated |
| **`instant-distance`** (InstantDomain) | HNSW | serde serialize | rebuild-ish | — | likely | simple, pure-Rust |
| **`faiss` (Rust bindings)** | IVF/PQ/HNSW/OPQ | yes | varies | **full PQ/SQ/OPQ** | no | most complete ANN toolbox; C++ dep, no WASM |
| `rust-cv/hnsw` | HNSW (hamming/binary) | — | yes | binary | yes | good for binary/Hamming over quantized/hash vectors |

### 4.2 Quantization & tradeoffs
- **Scalar (i8) quantization:** 4× smaller, ~free recall, simplest. Good default.
- **Binary quantization:** 32× smaller, Hamming distance (SIMD-popcount fast), small recall hit; Meilisearch reports 7× faster indexing. Pair with a re-rank on full/SQ vectors. Excellent for out-of-core / WASM memory budgets.
- **Product Quantization (PQ) / OPQ + IVF:** best memory/recall at billion scale (faiss). Heaviest to build; fits `sparq`'s external-memory build philosophy for the truly-huge case.
- General recall/latency/memory: HNSW gives best recall@latency in RAM; IVF-PQ wins memory at scale; DiskANN/disk-HNSW (hannoy) wins when index > RAM — directly matching `sparq`'s mmap mode.

### 4.3 Keying vectors by dict id (the clean part)
Vectors are stored in a **flat `Vec<f32>`/mmap array indexed by dense compaction of `u32` dict id**, plus the ANN graph over those ids. Because the dict id *is* the entity identity in `sparq`, the vector store needs **no string keys, no UUIDs, no join table** — `id → offset` is `id * dim`. Deletes use a tombstone bitmap keyed by id; the §4.1 "filtered ANN" (arroy/hannoy) lets us restrict search to an id subset (e.g. "only entities of rdf:type X") computed cheaply from the POS index.

### 4.4 Recommendation
- **Default native:** **`usearch`** (portable, mmap, i8/binary quant, WASM-capable, custom metrics) **or** **`hannoy`** (best out-of-core HNSW, filtered ANN, LMDB mmap) when out-of-core is primary.
- **Billion-scale build:** optional **faiss IVF-PQ** path, mirroring `sparq`'s external-memory builder.
- **WASM:** `usearch` (or binary-quant + `rust-cv/hnsw`) with vectors precomputed offline.

Sources:
- arroy: https://github.com/meilisearch/arroy ; binary quant https://blog.kerollmops.com/meilisearch-indexes-embeddings-7x-faster-with-binary-quantization ; filtered DiskANN https://blog.kerollmops.com/meilisearch-expands-search-power-with-arroy-s-filtered-disk-ann
- hannoy: https://blog.kerollmops.com/from-trees-to-graphs-speeding-up-vector-search-10x-with-hannoy
- LMDB patch (3× vector store): https://www.meilisearch.com/blog/3xfaster-vector-store
- usearch: https://lib.rs/crates/usearch
- hnsw_rs: https://crates.io/crates/hnsw_rs ; https://github.com/jean-pierreBoth/hnswlib-rs
- rust-cv/hnsw benchmarks: https://github.com/rust-cv/hnsw/blob/main/benchmarks.md

---

## 5. Hybrid retrieval (lexical + vector + structure)

Single-signal retrieval is fragile. Production answer = **fuse**:
1. **Candidate generation** from each available signal: BM25/trigram (lexical), ANN (text vectors), and structural neighbours (§7).
2. **Fusion**: **Reciprocal Rank Fusion (RRF)** — parameter-light, robust, no score calibration needed — as the default; optionally weighted score fusion if scores are calibrated. Meilisearch's "hybrid search" (BM25 + semantic) is a production reference.
3. **Re-rank** the fused top-N with expensive signals: exact edit distance (lexical), full-precision cosine (if quantized index used), and structural overlap features (Jaccard of neighbour sets / shared predicates). Optionally a cross-encoder for the textual case.

This directly mitigates each method's weakness: lexical misses paraphrase, dense misses rare IDs, structure misses text — RRF recovers items any one signal ranks highly.

Sources:
- Meilisearch hybrid search: https://blog.kerollmops.com/spotify-inspired-elevating-meilisearch-with-hybrid-search-and-rust
- Hybrid/semantic vs vector overview: https://www.meilisearch.com/blog/semantic-vs-vector-search

---

## 6. Integration design for `sparq` (opt-in, zero-impact)

### 6.1 Crate layout
- New crate **`sparq-sim`**, behind cargo feature `sim` (and sub-features `sim-lexical`, `sim-vector`, `sim-struct`). Default build links **none** of it. No `sparq` core type gains a field; the index lives in side files.
- Core engine exposes (already-existing) read-only access to: the term dictionary (sorted strings ↔ `u32`), the 6 permutation indexes, and the mmap layer. `sparq-sim` consumes these via a trait; removing the crate removes the feature with no diff to hot paths.

### 6.2 Where embeddings come from (no hot-path cost)
- Embeddings are produced **out-of-process** (matching Bergwinkl `embedding-server` / TopQuadrant external service): `sparq-sim` has a `build` step that, for each entity id, renders a text surface (labels/types/description via configurable SHACL-ish property list, computed from POS/PSO indexes), POSTs batches to an OpenAI-compatible `/v1/embeddings` endpoint, and writes a side **`vectors.bin`** (mmap, `id→offset`) + ANN index file.
- **Build-time vs query-time:** embeddings/ANN built offline or incrementally on commit; **query-time** is pure search. Structural features (§7) need no model and can be computed at build *or* lazily at query time from the permutation indexes.
- **Incremental:** on triple add/remove, mark affected entity ids dirty; re-render+re-embed only those (text path supports this; KG-embedding path needs periodic retrain — documented limitation).

### 6.3 API surface
```text
similar(iri, k, mode=Text|Struct|Hybrid, filter=Option<TypeId>) -> [(iri, score)]
search(text, k, mode=Lexical|Text|Hybrid, filter=…)            -> [(iri, score)]
analogy(a, b, c, k)            // KG-embedding mode: b - a + c
explain(query, hit)           // which signal(s) fired (lexical/vector/struct)
```
Also surface as SPARQL extension functions / a property-function / a magic predicate, e.g. `?x sim:similarTo (<iri> 10)` or `(?x ?score) sim:search ("query" 10)`, so similarity composes with normal BGP/join planning (cf. Vec2SPARQL). Gated by `sim`.

### 6.4 WASM feasibility
- **Lexical** (`fst`, `strsim`, `tantivy`-ngram): fully WASM-capable, tiny. Ship by default in the WASM build's `sim-lexical`.
- **Vector search**: WASM-capable if vectors+index are **precomputed offline** and shipped/mmaped (browser: fetched + searched with `usearch`/binary-HNSW). On-device *embedding* of the query is the hard part — either call a remote `/v1/embeddings`, or ship a tiny ONNX/`candle` MiniLM in WASM (heavier, optional).
- **Structural** (§7): pure index ops, fully WASM-capable, no model.

---

## 7. Novel proposals for `sparq` (clearly separated from SOTA above)

These exploit `sparq`'s specific architecture. They are *proposals*, to be validated by the §8 benchmarks.

### 7.1 "Free" lexical layer over the term dictionary (FST + automaton)
Because the dictionary is already a **sorted set of strings**, build a `fst::Map` from (normalized local-name/label → dict id) at feature-build time and serve typo-tolerant prefix/fuzzy lookup via **Levenshtein automata** with essentially **no extra index and no model**. This is a uniquely cheap lexical tier enabled by dict-encoding; most triple stores lack a ready sorted term set.

### 7.2 Structural similarity straight from the 6 permutation indexes (training-free)
For entity `e` (dict id), the permutation indexes give, in O(log n + degree) and cache-friendly scans:
- **out-predicates** = distinct `p` from a SPO range scan on `e`; **in-predicates** from POS/OSP.
- **out-neighbours** = `o` set from SPO; **in-neighbours** = `s` set from OPS/OSP.
- **types** = `o` for `(e, rdf:type, ?)` from SPO.
Define cheap, explainable structural similarity as a weighted blend of:
- **Predicate-profile similarity**: Jaccard/weighted-cosine over the (in+out) predicate multiset — "entities used the same way."
- **Neighbourhood overlap**: Jaccard / Adamic-Adar over shared neighbours (computed by **merge-intersecting two sorted id lists** — the exact primitive `sparq`'s merge-join already implements!).
- **Type-set overlap**.
This is essentially SimRank/feature-based similarity but realised as **sorted-list merge intersections over existing indexes** — no embeddings, no training, fully incremental (indexes update with the data), WASM-trivial. Hypothesis: competitive with RDF2Vec on "same-role" tasks at a fraction of the cost. **This is the headline novel contribution.**

### 7.3 MinHash/SimHash sketches keyed by dict locality
Precompute a small **MinHash signature per entity** over its predicate/neighbour set (from §7.2) and an **LSH band index**. Gives sublinear structural-similarity candidate generation that updates incrementally (recompute only changed entities) and quantizes to bits → cheap mmap + Hamming search (`rust-cv/hnsw`). Bridges §7.2's exactness with §4's ANN scalability; entirely training-free. (Conceptually a structural analogue of the hyperdimensional/hash-vector idea, but grounded in the actual triple sets.)

### 7.4 Inference-aware similarity (materialization × similarity)
`sparq` can do reasoning. Two interacting ideas:
- Compute structural features (§7.2) over the **materialized closure** (e.g. `rdfs:subClassOf`/`subPropertyOf`-expanded types/predicates) so similarity respects the ontology (two entities are "same type" via a common superclass). 
- **But** heed "More is not Always Better" (A-box materialization can *hurt* embeddings): make closure **opt-in per signal** and benchmark both. Likely sweet spot: expand the **T-box** (type/predicate hierarchies) for structural features, but *not* full A-box materialization for text embeddings.

### 7.5 Dict-locality-aware quantization / layout
`sparq` already assigns `u32` ids; if ids are assigned with locality (e.g. grouped by type or by frequency, as many dict builders do), we can (a) co-locate vectors of like entities to improve mmap page-locality during filtered search, and (b) train **per-type PQ codebooks** (entities of one type occupy a tighter subspace → better quantization recall at equal bits). Speculative; to be measured.

### 7.6 Hybrid fusion that includes the free structural signal
RRF over {FST/BM25 lexical, text-vector ANN, §7.2 structural}. The structural channel is unique to `sparq` and free, so even the "no-embeddings, no-model" build (`sim-lexical`+`sim-struct` only) offers a genuine `similar()`/`search()` — important for WASM and for users who won't run a model.

---

## 8. Benchmark plan (accuracy + performance)

**Datasets**
- **Lexical / entity linking:** DBpedia/Wikidata label sets; an autocomplete/typo benchmark (inject edits); a dedup pair set.
- **Structural / node tasks:** standard RDF2Vec eval suites — **AIFB, MUTAG, BGS, AM** (node classification), and **Wikidata/DBpedia** entity-relatedness; Mannheim "KG embedding for ML" benchmark.
- **KG-embedding / link prediction:** **FB15k-237**, **WN18RR**, optionally **YAGO3-10**, **CoDEx**.
- **Scale/perf:** synthetic + real billion-triple (LUBM/WatDiv generator, or a Wikidata truthy dump) to exercise out-of-core mmap.

**Accuracy metrics**
- Retrieval: **Recall@k**, **MRR**, **nDCG@k**, **Hits@k** (k∈{1,5,10}).
- Link prediction: **MRR**, **Hits@{1,3,10}** (filtered).
- Node classification (structural quality): macro-F1 / accuracy via downstream classifier on embeddings/features.
- Ablations: lexical-only vs text-vector-only vs §7.2 structural-only vs hybrid (RRF) — show fusion wins and quantify the *free* structural channel.

**Performance metrics**
- Build: time + peak RAM + on-disk size to embed/index N entities (and incremental: cost to update M changed entities).
- Query: p50/p95/p99 latency and QPS for `similar`/`search` at k, vs **recall** (the recall–latency–memory frontier) across quantization levels (none / i8 / binary).
- **Zero-impact proof:** run the full engine SPARQL benchmark (e.g. WatDiv/BSBM/SP2Bench) with the `sim` feature **off vs on-but-unused**, asserting no regression in binary size, RSS, or query latency. This is the load-bearing benchmark for the opt-in constraint.
- WASM: bundle size and query latency for `sim-lexical` and precomputed-vector search in-browser.

---

## 9. Concrete recommendation (TL;DR)

1. Ship **`sparq-sim`** (feature `sim`), three sub-tiers: `sim-lexical`, `sim-vector`, `sim-struct`. Default engine unaffected; prove it (§8 zero-impact bench).
2. **Lexical (do first, free, WASM):** `fst` Levenshtein-automaton over the dict (§7.1) + optional `tantivy` BM25 over rendered label/description docs; `strsim` re-rank.
3. **Structural (novel, free, WASM):** predicate-profile + neighbour-overlap similarity via sorted-list merge-intersection on the existing 6 indexes (§7.2), with MinHash/LSH sketches for scale (§7.3) and T-box-closure-aware features (§7.4).
4. **Text-vector (productised path, matches TopQuadrant/Bergwinkl):** render SHACL-driven text per entity → external OpenAI-compatible `/v1/embeddings` (e.g. self-hosted `bergos/embedding-server` with `bge-small`/MiniLM) → store `vectors.bin` keyed by dict id → ANN via **`usearch`** (portable/WASM) or **`hannoy`** (out-of-core, filtered ANN). i8 default, binary-quant for WASM/huge.
5. **Optional KG-embeddings (ComplEx/RotatE)** for relational similarity + analogy + link prediction; documented as transductive / periodic-retrain.
6. **Hybrid** RRF over all available signals + cheap re-rank; expose via `similar()`/`search()`/`analogy()` and SPARQL magic predicates.
7. Benchmark accuracy (MRR/Recall@k/Hits@k) **and** performance (recall–latency–memory frontier, incremental update cost, zero-impact proof, WASM bundle).

---

## Appendix: source URLs
- TopQuadrant AI Service: https://www.topquadrant.com/doc/8.3/reference/TopBraidAIServiceSection.html
- TopQuadrant EDG 8.2 launch: https://www.topquadrant.com/resources/topquadrant-launches-topbraid-edg-8-2-knowledge-graphs-as-the-foundation-for-secure-ai/
- Bergwinkl embedding-server: https://github.com/bergos/embedding-server
- Bergwinkl RDF/JS data processing: https://www.bergnet.org/2024/01/rdfjs-data-processing/
- RDF2Vec: https://www.rdf2vec.org/
- pyRDF2Vec (ESWC2023): https://link.springer.com/chapter/10.1007/978-3-031-33455-9_28
- Walk strategies (ISWC2021): https://link.springer.com/chapter/10.1007/978-3-030-87101-7_8 ; https://arxiv.org/pdf/2009.04404
- A-box materialization caveat: https://arxiv.org/pdf/2009.00318
- Vec2SPARQL: https://ceur-ws.org/Vol-2275/paper12.pdf
- arroy: https://github.com/meilisearch/arroy
- arroy binary quant: https://blog.kerollmops.com/meilisearch-indexes-embeddings-7x-faster-with-binary-quantization
- arroy filtered DiskANN: https://blog.kerollmops.com/meilisearch-expands-search-power-with-arroy-s-filtered-disk-ann
- hannoy: https://blog.kerollmops.com/from-trees-to-graphs-speeding-up-vector-search-10x-with-hannoy
- LMDB 3× patch: https://www.meilisearch.com/blog/3xfaster-vector-store
- usearch: https://lib.rs/crates/usearch
- hnsw_rs: https://crates.io/crates/hnsw_rs ; https://github.com/jean-pierreBoth/hnswlib-rs
- rust-cv/hnsw: https://github.com/rust-cv/hnsw/blob/main/benchmarks.md
- Meilisearch hybrid search: https://blog.kerollmops.com/spotify-inspired-elevating-meilisearch-with-hybrid-search-and-rust
