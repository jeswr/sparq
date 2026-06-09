# GenAI-Native RDF Store: Benchmark Suite + Product Synthesis

**A reference + product-strategy document for `sparq` (Rust, dict-encoded, out-of-core, 6-index, SPARQL 1.1 on spargebra, opt-in reasoning, WASM port).**

Scope: cross-cutting **evaluation** (Part A — reproducible accuracy + performance benchmarks for GenAI-native KG features) and **product synthesis + novel research agenda** (Part B). Sibling agents cover the individual features (IRI/entity similarity, NL→SPARQL, ontology introspection, KG-embeddings + vector index). This document owns the *measurement* and the *vision*.

Design constraints inherited from the engine: every GenAI feature must be **opt-in** (separate crate/feature flag), **trivially removable**, **zero perf/memory impact** on the core engine when disabled, and shipped with **both accuracy and performance benchmarks**. The dev machine is a **fanless Apple M1 Air** — hardware normalization is non-negotiable, and the design must respect a memory- and thermal-constrained baseline.

> Status: LIVING DOCUMENT. Numbers are extracted from primary sources with URLs + dates. Where a number is paraphrased from a secondary source it is marked *(secondary)*.

---

## 0. Executive summary (load-bearing facts)

1. **There is no single GenAI-KG benchmark.** You must assemble a suite: KBQA/NL→SPARQL (QALD-10, LC-QuAD 2.0, Text2SPARQL'25, GrailQA, KQA Pro, CronQuestions), link-prediction (FB15k-237, WN18RR, CoDEx, YAGO3-10), retrieval (BEIR-adapted, recall@k/nDCG/MRR), and performance (build/query at 10M/100M/1B). Each has its own metric and its own leakage traps.
2. **NL→SPARQL must report TWO numbers: (a) accuracy with the LLM in the loop, (b) end-to-end latency with the LLM EXCLUDED** (i.e. the store-side cost: entity/relation linking + query execution), so the engine's contribution is isolable from a swappable LLM.
3. **Entity-linking leakage and Wikidata-version drift are the dominant validity threats** for KBQA. Pin the KG dump by date; report whether gold entities were provided ("oracle linking") or predicted.
4. **The QALD Macro-F1 measure** (per-question P/R/F1 then averaged) is the canonical KBQA metric; **execution accuracy** (does the generated SPARQL return the gold answer set) is the cleaner, store-centric metric and the one `sparq` should headline.
5. **Link prediction is filtered MRR + Hits@{1,3,10}** under the "filtered" protocol. WN18/FB15k are *deprecated* (test leakage via inverse relations); use **WN18RR / FB15k-237**. SOTA FB15k-237 MRR ≈ 0.35–0.40, WN18RR MRR ≈ 0.48–0.50.
6. **`sparq`'s unique edge** = out-of-core dict-encoded integer triples + 6 permutations + reasoning, in Rust/WASM. This is exactly the substrate for: (a) **embeddings-as-an-index for the query planner**, (b) **inference-augmented retrieval** (materialize, then embed/retrieve over the closure), (c) **grammar-constrained SPARQL decoding** against the real dictionary, and (d) a **browser-local private KG assistant** (WASM, no server, data never leaves device).
7. **Hardware normalization protocol**: report wall-clock on the M1 Air *and* a normalized "per-core-GHz" / "per-GB/s-membw" figure, plus the machine spec inline with every table (the in-repo `inference-sota.md` already sets this house style — match it).

---

## PART A — THE BENCHMARK SUITE

### A.1 NL→SPARQL / KBQA

#### A.1.1 Datasets (version-pinned)

| Dataset | KG | Size (test) | Question type | Canonical metric | Notes |
|---|---|---|---|---|---|
| **QALD-9** | DBpedia | 150 | multilingual, complex | Macro F1 (QALD) | classic; DBpedia 2016-10 |
| **QALD-9-plus** | DBpedia + Wikidata | 150 | multilingual | Macro F1 | adds Wikidata gold + more languages |
| **QALD-10** | **Wikidata** | 394 | multilingual (en/de/ru/zh), complex | Macro F1 (QALD) | Wikidata Truthy dump; version drift is acute |
| **LC-QuAD 1.0** | DBpedia | 1,000 | template-generated, multi-hop | Macro F1 / answer F1 | DBpedia 04-2016 |
| **LC-QuAD 2.0** | Wikidata + DBpedia | 6,046 | 30k pairs total, complex/dual | F1, BLEU, exact-match | larger, harder |
| **Text2SPARQL'25** | DBpedia + corporate KG | 100 (en/es) + 50 (en) | challenge, incl. unknown KG | F1 (challenge harness) | ESWC 2025 workshop |
| **WikiSP / WikiWebQuestions** | Wikidata | ~3k | from WebQuestions, Wikidata-mapped | answer F1 / exec acc | Stanford OVAL |
| **GrailQA** | Freebase | 13,231 test (64,331 total) | i.i.d. / compositional / **zero-shot** generalization | EM, F1 (per-split) | 3,720 relations, 86 domains; the generalization benchmark |
| **KQA Pro** | Wikidata (FB15k-style dense subset) | 11,797 | program + SPARQL, compositional | accuracy (answer) | has KoPL programs too |
| **CronQuestions** | Wikidata (temporal) | 410k (large) | **temporal** reasoning | Hits@1 / answer acc | temporal KGQA |

#### A.1.2 Metrics — exact definitions

- **QALD Macro F1**: for each question compute precision/recall against the gold answer set, F1 = 2PR/(P+R); average F1 over all questions ("Macro F1 QALD"). Unanswered → counted (P=R=0 unless the system *abstains* correctly). Source: Usbeck et al., *QALD-10*, Semantic Web Journal 2024, https://journals.sagepub.com/doi/10.3233/SW-233471 .
- **Macro F1 (strict) vs Micro F1**: Macro averages per-question; Micro pools all answer entities then computes one P/R/F1. Report **both**; Macro is QALD-canonical.
- **Execution accuracy (exec acc / answer accuracy)**: fraction of questions where the *executed* generated SPARQL returns exactly the gold answer set (set equality). This is the **store-centric headline metric** — it folds in linking + generation + execution and is reproducible offline against a pinned dump.
- **Exact-match (EM) on query strings**: brittle (many SPARQLs are answer-equivalent); report only as a secondary, normalized (canonicalized variable names, ordering) figure.
- **Answer F1**: as above but token/entity-level F1 on the answer set (used by WebQuestions-derived sets).

#### A.1.3 Known SOTA scores (primary where possible)

- **QALD-10 (English)**: DFSL-MQ (2024) ≈ **0.622 F1 / 0.632 exec acc**; COT-SPARQL (2024) ≈ 0.498 F1 / 0.508 exec acc *(secondary; verify against source paper tables)*. QALD-10 dataset paper: Usbeck et al. 2024, https://journals.sagepub.com/doi/10.3233/SW-233471 .
- **Text2SPARQL'25**: **mKGQAgent** took 1st place on combined DBpedia + Corporate KGQA. ARUQULA (ReAct + KG exploration) is a strong open system: arXiv:2510.02200, https://arxiv.org/abs/2510.02200 (2025). AIRI team system: CEUR Vol-4094 paper5, https://ceur-ws.org/Vol-4094/paper5.pdf .
- **LC-QuAD**: large variance across systems; LLM-based systems report sizeable exec-acc gains on LC-QuAD 2.0 / QALD-10 (see KnowledgeNLP 2025, https://aclanthology.org/2025.knowledgenlp-1.5.pdf ).
- Leaderboard aggregators: KGQA Leaderboard (https://kgqa.github.io/leaderboard/), RUCAIBox Awesome-KBQA (https://github.com/RUCAIBox/Awesome-KBQA).

#### A.1.4 How to run against a local store (offline, reproducible)

1. **Pin the KG.** Wikidata: a dated Truthy `.nt` dump (e.g. `wikidata-truthy-2024-XX-XX`); DBpedia: 2016-10. Record the exact dump URL + sha256. *This is the #1 reproducibility control* — Wikidata drifts daily and gold answers rot.
2. **Load into sparq**, build the 6 indexes. Persist a snapshot so the harness is deterministic.
3. **Two evaluation modes**:
   - **Oracle-linking mode**: feed gold entity/relation IRIs to the generator → isolates *query construction* from *entity linking*. Report separately.
   - **End-to-end mode**: NL question → (linker) → SPARQL → execute on sparq → compare answer set. Report exec acc + Macro F1.
4. **Execute the gold SPARQL too** on the pinned dump and cache its answer set — defends against drift (gold answers recomputed against the *same* dump the system queries).
5. **Abstention handling**: define behavior for "no answer" / out-of-KG; QALD counts these.

#### A.1.5 Validity threats / caveats (must be reported)

- **Entity-linking leakage**: many papers quietly use gold entities ("oracle linking"). Always state which. An engine number that uses oracle linking is *not* comparable to an end-to-end number.
- **Wikidata-version drift**: gold answer sets computed against a different dump → spurious mismatches. Mitigate by recomputing gold from the pinned dump.
- **Answer-equivalent SPARQL**: never headline string EM.
- **Train/test KG overlap & memorization**: LLMs may have memorized Wikidata; report a held-out / private-KG split (Text2SPARQL'25 corporate KG is designed for exactly this).
- **Timeouts**: a query that times out ≠ wrong query; report timeout rate separately.

### A.2 Entity similarity / link prediction / entity resolution

#### A.2.1 Datasets

| Dataset | Domain | #Entities / #Rel / #Triples | Use | Caveat |
|---|---|---|---|---|
| **FB15k-237** | Freebase subset | 14,541 / 237 / 310k | link prediction | inverse-relation leakage removed (vs FB15k) |
| **WN18RR** | WordNet | 40,943 / 11 / 93k | link prediction | leakage removed (vs WN18); sparse, lexical |
| **CoDEx (S/M/L)** | Wikidata | up to 78k / 69 / 612k | link pred + triple classification | hard negatives, less leakage; recommended modern set |
| **YAGO3-10** | YAGO | 123k / 37 / 1.08M | link prediction | denser, more entities |
| **OGB: ogbl-wikikg2, ogbl-biokg** | Wikidata / biomedical | millions | scalable link pred | for the 10M+ scale story |
| **OAEI tracks** | various ontologies | varies | **entity/ontology alignment (ER)** | F1 on reference alignments |

#### A.2.2 Metrics — exact definitions

- **Filtered MRR**: for each true test triple, corrupt head (and tail) over all entities, *filter out* other true triples from train/valid/test, rank the true entity, MRR = mean(1/rank). "Filtered" is mandatory; "raw" is deprecated. Source convention: Bordes et al. TransE (NeurIPS 2013); filtered setting standard since.
- **Hits@K (K∈{1,3,10})**: fraction of test triples whose true entity ranks ≤ K (filtered).
- **Tie-handling**: use the **random / mean rank** tie policy (NOT optimistic "min rank") — the 2020 reproducibility crisis (Sun et al., "A Re-evaluation of Knowledge Graph Completion Methods", ACL 2020, https://aclanthology.org/2020.acl-main.489/) showed optimistic tie-breaking inflated scores massively. Report the policy.
- **Entity resolution (OAEI)**: precision/recall/F1 against reference alignment.

#### A.2.3 Known SOTA (filtered)

- **FB15k-237**: MRR ≈ **0.35–0.40**, Hits@10 ≈ 0.53–0.56 (strong models: e.g. NBFNet, modern transformers). CAB-KGC (2024) reports Hits@1 ≈ 0.322 *(secondary)*.
- **WN18RR**: MRR ≈ **0.48–0.50**, Hits@10 ≈ 0.57–0.60. CAB-KGC (2024) reports MRR ≈ 0.685 / Hits@1 ≈ 0.637 *(secondary — unusually high; verify, possible different protocol)*.
- Reference baselines (TransE/DistMult/ComplEx/RotatE/TuckER) are the honest comparison points for a *store-embedded* (not SOTA-chasing) embedder; PyKEEN reproduces all of these with documented hyperparameters: https://github.com/pykeen/pykeen .

> **Recommendation for sparq**: do NOT chase KGE SOTA. Ship a *fast, in-store* embedder (RotatE/ComplEx/DistMult class) and report its MRR/Hits within a few points of PyKEEN baselines, with build-time + memory at scale. The product story is "embeddings *colocated with* the store and the reasoner", not "best MRR".

### A.3 Retrieval (semantic search over the KG)

- **Metrics**: recall@k, MRR, **nDCG@{10,100}**, MAP. nDCG needs graded relevance.
- **Methodology (BEIR-adapted)**: BEIR (Thakur et al., NeurIPS 2021 D&B, https://github.com/beir-cellar/beir) is the IR-retrieval standard (corpus / queries / qrels, report nDCG@10). Adapt to KG by defining the "document" unit: an **entity's concise bounded description (CBD)** or a **verbalized neighborhood** (1-hop star). Queries = NL or entity mentions; qrels = relevance judgments.
- **Building ground truth offline** (no manual annotation):
  - **Property-based silver labels**: entities sharing type + key properties = relevant (e.g. `wdt:P31` class + shared `wdt:P106`).
  - **Link-prediction-as-retrieval**: for a (h, r, ?) query, gold = true tails (filtered) → reuse FB15k-237/CoDEx splits as retrieval qrels.
  - **Alias/redirect sets**: Wikidata `skos:altLabel` / DBpedia redirects = relevant duplicates for entity-similarity.
  - **Human spot-check** a small slice to validate the silver labels (report agreement).
- **ANN recall** is a *system* metric (recall of the index vs exact NN), distinct from *task* recall@k — report both; never conflate.

### A.4 Performance benchmarks

#### A.4.1 What to measure (per feature)

| Capability | Metric | Scales |
|---|---|---|
| Embedding build | wall time, peak RSS, on-disk size | 10M / 100M / 1B triples |
| ANN index build | build time, RSS, index size | 10M / 100M / 1B |
| ANN query | p50/p95/p99 latency, QPS, **recall@k vs latency curve** | per index |
| NL→SPARQL e2e | latency **LLM-excluded** (linking+exec) AND **LLM-included** | per dataset |
| Linking | latency, accuracy (P/R/F1 of linked entities) | — |
| Introspection | compute time for schema/stats summary, memory | per dataset size |
| Reasoning-augmented retrieval | closure materialization time + Δ retrieval quality | — |

#### A.4.2 The recall/latency tradeoff (ANN)

Report the canonical **recall@10 vs QPS Pareto curve** (the ann-benchmarks.com methodology, Aumüller et al., https://github.com/erikbern/ann-benchmarks). Sweep the index parameter (HNSW `efSearch`, IVF `nprobe`) to trace the curve; a single (recall, QPS) point is not publishable. Compare against `hnswlib` / `faiss` as the external reference.

#### A.4.3 NL→SPARQL latency decomposition (CRITICAL)

Break end-to-end into: **(1) LLM generation** (model + tokens; report model + hardware, exclude from engine number), **(2) entity/relation linking** (sparq's job — ANN/dictionary lookup), **(3) SPARQL parse+plan+execute** (sparq's job), **(4) post-processing**. Headline **(2)+(3)+(4)** as the *store* latency; report **(1)** separately and note it's LLM/hardware-dependent and swappable. This is how the engine's contribution stays honest and the LLM stays a pluggable dependency.

#### A.4.4 Hardware normalization protocol (owner is sensitive here)

- **State the machine inline with every table** (match `inference-sota.md` house style): "Apple M1 (4P+4E), 16 GB LPDDR4X-4266 (~68 GB/s), fanless, macOS, single-threaded unless noted."
- Report **wall-clock** (primary) + a **normalized figure**: throughput per P-core, and for memory-bound builds, GB processed per GB/s of bandwidth (sequential external-memory build is membw-bound — normalize by ~68 GB/s).
- **Thermal caveat**: the fanless M1 Air throttles on sustained load; report whether a run is sustained (throttled) or burst, and pin a cooldown between iterations. Min-of-K warm is fine for query latency; for *build* (sustained), report a single cold run + note thermal state.
- Provide a **second reference machine** number where possible (a Linux x86 server) so ratios, not absolutes, carry the claim — ratios travel across hardware, absolutes do not.
- Never compare sparq-on-M1 to a competitor-on-server without the normalization caveat printed in the same table.

### A.5 Proposed harness layout (version-pinned, scriptable)

```
crates/sparq-genai-bench/            # opt-in; feature = "genai-bench"; not in default workspace build
  Cargo.toml                         # pins every dataset version via a manifest
  manifest.toml                      # dataset name -> {url, sha256, date, license}
  src/
    datasets/                        # downloaders + loaders (cache to ~/.sparq-bench/)
      qald.rs  lcquad.rs  text2sparql.rs  grailqa.rs  kqapro.rs  cron.rs
      fb15k237.rs  wn18rr.rs  codex.rs  yago310.rs
      beir_kg.rs                     # KG-adapted retrieval builder + silver-label generator
    runners/
      kbqa.rs                        # oracle-linking + e2e modes; LLM behind a trait (mockable, offline-cacheable)
      linkpred.rs                    # filtered MRR/Hits, documented tie policy
      retrieval.rs                   # recall@k / nDCG / MRR + ANN recall-vs-QPS sweep
      perf.rs                        # build/query timing + RSS at 10M/100M/1B
    metrics/
      qald_f1.rs  exec_acc.rs  filtered_mrr.rs  ndcg.rs  ann_recall.rs
    llm/
      trait.rs                       # LlmBackend trait; record/replay cache so CI is offline + deterministic
      replay.rs                      # cached (prompt -> completion) fixtures committed to repo
    report.rs                        # emits the canonical results table (markdown + JSON)
bench/genai/
  results/                           # committed result JSON + rendered markdown, per machine
  fixtures/llm/                      # committed LLM record/replay cache (no network in CI)
```

**Canonical results table format** (one per dataset; machine spec in caption):

```
### <dataset> — <metric>  (machine: M1 Air 8-core, 16GB, single-thread)
| system        | mode        | metric1 | metric2 | build s | RSS MiB | notes        |
|---------------|-------------|--------:|--------:|--------:|--------:|--------------|
| sparq         | oracle-link |   0.xx  |  0.xx   |   ...   |  ...    | dump sha=... |
| sparq         | e2e         |   0.xx  |  0.xx   |   ...   |  ...    | LLM=...      |
| <baseline>    | ...         |   ...   |  ...    |   ...   |  ...    | from <cite>  |
```

**Reproducibility rules**: every dataset entry pins {url, sha256, date}; LLM calls go through a committed record/replay cache so CI runs offline and deterministically; every perf table prints the machine spec + thermal state; tie/abstention/filtering policies stated in the metric module docstring.

---

## PART B — SYNTHESIS & NOVEL AGENDA

### B.1 The GenAI + Knowledge Graph landscape (what's SOTA, what's missing)

#### B.1.1 GraphRAG family (LLM builds + queries a graph from text)

- **Microsoft GraphRAG** (mid-2024): LLM extracts entities+relations from documents → builds a KG → Leiden community detection → **community summaries**. Two retrievers: *local* (vector similarity + graph traversal around seed entities) and *global* (map-reduce over community summaries for corpus-wide questions). Blog: https://www.microsoft.com/en-us/research/blog/graphrag-unlocking-llm-discovery-on-narrative-private-data/ . Repo: https://github.com/microsoft/graphrag .
- **LazyGraphRAG** (late 2024): defers the expensive LLM summarization; **indexing cost ≈ 0.1% of full GraphRAG** (≈ vector-RAG cost) while matching local-query quality, ~700× lower global-query cost. Blog: https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/ .
- **Framework integrations**: LangChain (`Neo4jGraph`, `GraphCypherQAChain`), LlamaIndex (`KnowledgeGraphIndex`, `PropertyGraphIndex`), Neo4j's GraphRAG package. Almost all are **Cypher / property-graph centric**, not RDF/SPARQL.
- **Evidence**: GraphRAG-Bench (2025) — GraphRAG beats vanilla vector RAG on **multi-hop reasoning + global summarization**, gap narrows for simple fact lookup. So the value is *structure-dependent* questions.

> **Gap for sparq**: GraphRAG today is mostly *property-graph + LLM-constructed* graphs with weak/no formal semantics, no reasoning, and in-memory/server-bound. sparq offers **RDF + OWL/N3 reasoning + out-of-core scale + SPARQL provenance** — a *grounded, queryable, inferable* graph rather than an LLM-summarized blob.

#### B.1.2 "Talk to your graph" products (NL→query over an existing KG)

- **Stardog Voicebox** (2024): LLM identifies semantic concepts → **embedded vector store matches them to the ontology/model** → ontology concepts handed back to the LLM → structured SPARQL generated → executed → answer. Agentic, enterprise. Docs: https://docs.stardog.com/voicebox/ ; launch: https://venturebeat.com/ai/stardog-launches-voicebox-an-llm-powered-layer-to-query-enterprise-data .
- **Ontotext GraphDB "Talk to Your Graph"** (GraphDB 11.x, 2024–25): chat over the KG using SPARQL generation + similarity search + full-text + ChatGPT retrieval connector. Docs: https://graphdb.ontotext.com/documentation/11.3/talk-to-graph.html .
- **RDFox + LLM**: RDFox is the materialization/reasoning leader (see in-repo `inference-sota.md`); LLM-facing NL layer is comparatively nascent — an opening.
- **Vector-native graph/DBs**: Neo4j (HNSW vector index + Cypher), Weaviate / Qdrant (vector-first, RDF only via mapping), TigerGraph, KuzuDB. RDF-native vector colocation is rare.

> **The recurring pattern across all of these**: (1) embed the *schema/ontology terms* into a vector store; (2) retrieve relevant terms for the NL question; (3) feed terms to an LLM that emits SPARQL/Cypher; (4) execute; (5) verbalize. sparq can implement this loop **fully local (WASM)** and **grounded in reasoning** — neither Voicebox nor Talk-to-Your-Graph runs in the browser with the data never leaving the device.

#### B.1.3 KG ↔ LLM virtuous loop, hallucination reduction, provenance

- **"KG grounds the LLM"**: retrieving verified triples as context reduces hallucination by constraining generation to facts the store can vouch for. Survey: *Can Knowledge Graphs Reduce Hallucinations in LLMs?* arXiv:2311.07914 — https://arxiv.org/abs/2311.07914 . 2025 survey on Retrieval-And-Structuring Augmented Generation: arXiv:2509.10697.
- **"LLM queries the KG"**: NL→SPARQL turns the LLM into a *query front-end*; the *answer* is computed by the engine (deterministic, exact), not generated → hallucination-free answers when the SPARQL is correct.
- **The loop**: LLM proposes structure (entities, query) → KG executes + verifies + returns provenance → LLM verbalizes grounded answer → optionally writes new validated triples back. sparq's reasoning crate closes the loop: **inferred** facts are also grounded (with rule-derivation provenance).
- **Provenance is sparq's differentiator**: every answer triple can carry (a) source graph/IRI, (b) for inferred triples, the rule + premises (RDF-star / N3 justification). This is the antidote to "trust me" RAG. SPARQL's `GRAPH` + RDF-star make provenance first-class; LLM RAG over text cannot match this.

### B.2 Where a Rust, out-of-core, dict-encoded engine has a UNIQUE edge

1. **Browser-local private KG assistant (WASM)** — the standout. sparq already has a WASM port. A NL→SPARQL + vector-similarity + reasoning stack compiled to WASM = a **fully client-side "chat with your knowledge graph"** where the data *never leaves the device*. No competitor (Voicebox, GraphDB, Neo4j) runs the store + retrieval + reasoning in-browser. Killer for healthcare/legal/personal-data ("private KG assistant").
2. **Out-of-core scale for retrieval ground-truth** — embeddings + ANN over **1B-triple** KGs on a laptop via mmap, where Python KGE pipelines OOM. The perf story ("KGE + ANN at 1B triples, 16 GB RAM") is unique.
3. **Dict-encoded u32 ids = embeddings are just a `Vec<f32>` keyed by id** — zero join/lookup overhead between the symbolic store and the vector store; they share the same id space. Most stacks bolt a separate vector DB alongside; sparq's vector index *is* an array indexed by dictionary id.
4. **Reasoning colocated with retrieval** — *inference-augmented retrieval*: materialize the OWL/N3 closure, then embed/retrieve over the **closure**, surfacing entailed-but-not-asserted matches. RDFox reasons but has no embedding/ANN; vector DBs retrieve but don't reason. sparq does both, sharing the id space.
5. **Grammar-constrained SPARQL against the REAL dictionary** — sparq owns the SPARQL grammar (spargebra) *and* the dictionary, so it can constrain LLM decoding to (a) grammatical SPARQL 1.1 *and* (b) IRIs/predicates that actually exist in this store. See B.3.

### B.3 Novel research agenda (prioritized by impact / effort)

Legend: Impact (1–5), Effort (1–5, lower=cheaper). Sort = high impact, low effort first.

| # | Idea | Impact | Effort | Why sparq specifically |
|---|---|--:|--:|---|
| **N1** | **Embeddings-as-an-index for the query planner** (KGE features → cardinality estimation / join order) | 5 | 3 | GNCE (arXiv:2303.01140) shows KGE features predict conjunctive-query cardinality over RDF; sparq's dict ids make KGE a colocated array → better join ordering with *zero* core-engine changes (opt-in planner hook). Improves the **core** product, not just GenAI. |
| **N2** | **Grammar-constrained SPARQL decoding against the live dictionary** | 5 | 2 | sparq owns spargebra grammar + dictionary. Emit a GBNF/XGrammar constraint that forces (a) valid SPARQL 1.1, (b) only existing IRIs/predicates. Near-eliminates syntactically-invalid + dangling-IRI generations. Cheap, huge correctness win. Refs: Geng et al. GCD (EMNLP'23), arXiv:2502.05111; Outlines/XGrammar. |
| **N3** | **Local WASM private-KG assistant** (NL→SPARQL + vector sim + verbalize, in-browser, offline) | 5 | 3 | Unique to sparq's WASM port; data never leaves device. Product-defining. LLM via WebGPU/wasm small model or remote-optional. |
| **N4** | **Inference-augmented retrieval** (embed the reasoning closure, not just asserted triples) | 4 | 3 | sparq has the reasoning crate + shared id space. Surfaces entailed matches no vector-DB-only or reasoner-only system can. Benchmark: Δ recall on link-pred when retrieving over closure vs base. |
| **N5** | **Provenance-carrying answers** (every answer triple → source graph + rule justification for inferred) | 4 | 2 | RDF-star / N3 justification + SPARQL `GRAPH`. Directly attacks LLM hallucination distrust; auditable GenAI. Mostly plumbing on existing features. |
| **N6** | **Self-correcting NL→SPARQL loop** (generate → execute → if empty/error, feed the engine's error + schema neighborhood back to the LLM → repeat) | 4 | 2 | ARUQULA/ReAct-style (arXiv:2510.02200) but grounded in sparq's *real* execution errors and dictionary neighborhoods. Big exec-acc lift for small extra latency. |
| **N7** | **Schema/ontology auto-introspection as the linking index** (build the term-embedding index from VoID/SHACL/owl + property co-occurrence stats the engine already computes) | 3 | 2 | sparq computes index statistics anyway; expose them as the entity/predicate-linking vector store (the Voicebox pattern, but free from existing engine stats). |
| **N8** | **KGE-as-a-store-service** (RotatE/ComplEx trained in-store, queried via SPARQL extension function `sparq:similar(?e, k)`) | 3 | 4 | Embeddings become a first-class SPARQL citizen; hybrid symbolic+vector queries in one language. Differentiator vs bolt-on vector DBs. |
| **N9** | **Hybrid retrieve-then-reason ranking** (ANN candidate set → reasoner verifies/filters → exact answer) | 3 | 3 | Combines vector recall with logical precision; report precision lift from the reasoning filter stage. |
| **N10** | **Drift-robust KBQA harness with recomputed gold** (recompute gold answers from the pinned dump, the Part-A control) shipped as a reusable public benchmark | 3 | 2 | Turns Part A into a *community* contribution; positions sparq as the reproducibility-first KGQA platform. |

#### Recommended sequencing

- **Phase 1 (cheap, high-impact, mostly core-positive)**: N2 (grammar-constrained decoding), N5 (provenance), N6 (self-correcting loop), N1 (KGE planner features — *this one also speeds the core engine*). Ship each behind `--features genai` with both an accuracy and a perf benchmark from the Part-A harness.
- **Phase 2 (product-defining)**: N3 (WASM local assistant), N4 (inference-augmented retrieval), N7 (introspection linking index).
- **Phase 3 (research-flagship)**: N8 (KGE-in-SPARQL), N9 (retrieve-then-reason), N10 (public drift-robust KBQA benchmark).

#### The pitch in one line

**sparq = the only RDF engine where the symbolic store, the OWL/N3 reasoner, the KGE/vector index, and the SPARQL grammar all share one dictionary id-space — enabling grounded, provenance-carrying, inference-augmented GenAI that runs out-of-core on a laptop *and* fully private in the browser.**

---

## Appendix: Primary sources

- QALD-10: Usbeck et al., *Semantic Web Journal* 2024 — https://journals.sagepub.com/doi/10.3233/SW-233471
- Text2SPARQL'25 / ARUQULA: arXiv:2510.02200 — https://arxiv.org/abs/2510.02200 (2025)
- KGQA leaderboards: https://kgqa.github.io/leaderboard/ ; https://github.com/RUCAIBox/Awesome-KBQA
- KGE re-evaluation (tie-breaking): Sun et al., ACL 2020 — https://aclanthology.org/2020.acl-main.489/
- PyKEEN (reproducible KGE baselines): https://github.com/pykeen/pykeen
- BEIR: https://github.com/beir-cellar/beir
- ann-benchmarks: https://github.com/erikbern/ann-benchmarks
- Microsoft GraphRAG: https://www.microsoft.com/en-us/research/blog/graphrag-unlocking-llm-discovery-on-narrative-private-data/ ; https://github.com/microsoft/graphrag
- LazyGraphRAG: https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/
- Stardog Voicebox: https://docs.stardog.com/voicebox/ ; https://venturebeat.com/ai/stardog-launches-voicebox-an-llm-powered-layer-to-query-enterprise-data
- Ontotext GraphDB "Talk to Your Graph": https://graphdb.ontotext.com/documentation/11.3/talk-to-graph.html
- KGs reduce hallucination (survey): arXiv:2311.07914 — https://arxiv.org/abs/2311.07914
- Retrieval-And-Structuring Augmented Generation survey (2025): arXiv:2509.10697
- GNCE — cardinality estimation over RDF with KG embeddings: arXiv:2303.01140 — https://arxiv.org/abs/2303.01140
- Grammar-constrained decoding: Geng et al. EMNLP'23; "Flexible and Efficient GCD" arXiv:2502.05111; Outlines (Willard & Louf); XGrammar
- CoDEx (Wikidata KGE benchmark): Safavi & Koutra, EMNLP 2020 — https://github.com/tsafavi/codex
- GrailQA: Gu et al., WWW 2021 — https://dki-lab.github.io/GrailQA/ ; https://github.com/dki-lab/GrailQA
- CronQuestions (temporal KGQA): Saxena et al., ACL 2021 — https://aclanthology.org/2021.acl-long.520/
- KQA Pro: Cao et al., ACL 2022 — https://github.com/shijx12/KQAPro_Baselines
- GraphRAG-Bench (2025): evaluation of GraphRAG vs vector RAG on reasoning/summarization
