# Natural-Language Conversation with the Knowledge Base: Text-to-SPARQL / KBQA for `sparq`

**A design + literature reference for an opt-in, pluggable, WASM-compatible natural-language query layer over the `sparq` RDF/SPARQL engine.**

Scope: turning natural-language (NL) questions into executable SPARQL 1.1 queries (Text-to-SPARQL / Knowledge-Base Question Answering, KBQA), grounding on the actual schema of the loaded dataset, validating/repairing generated queries against `spargebra`'s grammar + algebra, and carrying conversational context across turns. The engine itself stays LLM-agnostic; everything here lives in a separate, feature-gated crate with **zero perf/memory impact on the default engine** and benchmarks for both accuracy and latency.

Primary sources are cited inline with URLs and (where known) dates. Numbers are taken from the papers' own tables where quoted.

---

## 0. Executive summary (load-bearing facts)

1. **There is no single winning technique — the SOTA is a *pipeline / agent loop*.** Every strong 2024-2025 system combines (entity/relation linking) + (schema retrieval into the prompt) + (LLM generation, usually few-shot ICL) + (execute → repair loop). The Text2SPARQL'25 winner *mKGQAgent* is a modular agent doing "planning, entity/relation linking, template grounding, iterative refinement" with vector+full-text schema search and live querying. <https://arxiv.org/pdf/2507.16971> **But** SPARQL-LLM (2025) shows a *lean* retrieve+repair loop beats those agents by **+24% F1 on DBpedia at 36× lower latency and ≤$0.01/question** — so `sparq` should default to a tight loop, not a sprawling agent. <https://arxiv.org/html/2512.14277v1>
2. **Fine-tuned seq2seq still tops *closed-schema* benchmarks; LLM-ICL wins *open-domain / robustness*.** Triplet-order-sensitive T5 (TosT5) reports **95.4% F1 / 93.5% QM on LC-QuAD 2.0** — a fixed-schema benchmark — beating prompting. But on open Wikidata/DBpedia (QALD-10, Text2SPARQL'25) agentic LLM systems with KG exploration win, because the schema is huge and shifts. <https://arxiv.org/abs/2410.05731>
3. **The two failure modes are (a) syntactically invalid SPARQL and (b) valid-but-wrong IRIs.** (a) is *fully solvable for free* in `sparq`: we already embed `spargebra`, so we can (i) parse-validate every candidate and (ii) — the novel bit — **constrain decoding to the SPARQL grammar** so invalid tokens are never emitted. (b) is the hard part and needs entity/relation linking against our dictionary + a similarity-search module.
4. **Execution-guided / self-correction loops are the highest-ROI accuracy lever.** Run the query; feed back parse errors, unknown-IRI errors, and empty-result signals; let the model revise. SPARQL-specific work shows convergence "in up to three revision steps." <https://arxiv.org/html/2512.14277v1>
5. **Live KG exploration beats static pre-linking on hard questions.** Spinach (GPT-4o, 0-shot) sets SOTA on QALD-7/9+/10 (e.g. QALD-10 69.5 F1, +10pp) and crushes prior agents on its own hard dataset (45.3 vs 7.2 F1) by *iteratively building the query against the live endpoint*. `sparq`'s cheap index introspection makes this exploration essentially free locally (no network round-trips). <https://arxiv.org/html/2407.11417v1>
6. **`sparq` has three structural advantages over generic Text-to-SPARQL stacks:** (i) `spargebra` gives us a *formal grammar + algebra* to validate and constrain against; (ii) the dictionary + 6 permutation indexes make *schema introspection and IRI lookup* cheap (we can answer "does this predicate exist? how many triples use it?" in O(log n)); (iii) the **materialized-inference module** (`sparq-reason`) lets us answer questions a raw query can't (e.g. transitive `rdfs:subClassOf`, `owl:sameAs`) — a differentiator no pure-LLM stack has.
7. **Opt-in is natural here.** The NL layer is pure orchestration around the existing public query API plus an *external* LLM call. It compiles out entirely behind a cargo feature; the default engine never links an HTTP client or a tokenizer. WASM works because the LLM call is just `fetch` to an API (or `wasm`-hosted small model via an injected callback).

---

## 1. Problem framing

**Text-to-SPARQL** = map a NL question `q` (plus optional conversation history `H`) to a SPARQL query `s` such that executing `s` over the KG yields the intended answer `A`. Two evaluation philosophies:

- **Query-match (QM / EM):** does generated `s` equal the gold query (up to normalization)? Strict, schema-coupled, brittle to equivalent rewrites.
- **Execution / answer F1:** execute `s`, compare the *answer set* to gold answers. Robust to query paraphrase; this is what QALD and Text2SPARQL grade on. Preferred.

**Why it's hard (the four gaps):**
1. **Lexical gap** — "movies" ↦ class `dbo:Film`; "directed by" ↦ `dbo:director`. (schema/relation linking)
2. **Entity gap** — "Tarantino" ↦ `wd:Q3772` / `dbr:Quentin_Tarantino`. (entity linking, ambiguous)
3. **Structural gap** — multi-hop paths, aggregation, filters, ordering, `OPTIONAL`, negation, property paths.
4. **Grounding gap** — the model hallucinates IRIs/predicates that don't exist in *this* dataset.

`sparq`'s job is to make gaps 1, 2, 4 cheap (introspection + similarity search + grammar validation) and gap 3 reliable (algebra validation + repair).

---

## 2. SOTA approaches (literature map)

### 2.1 LLM in-context / few-shot prompting with schema linking — *(a)*

Given a question, retrieve `k` most-similar (NL, SPARQL) examples and the relevant schema fragment, put them in the prompt, let the LLM emit SPARQL, execute. This is the dominant open-domain approach.

- **ICSU — In-Context Schema Understanding** (Liu et al., 2023): LLM generates SPARQL directly via ICL with examples retrieved by different strategies; treats schema linking as an in-context task rather than a separate trained model. <https://arxiv.org/pdf/2310.14174>
- **Dynamic Few-Shot Learning for KGQA** (Lehmann et al., 2024): selects few-shot exemplars *dynamically* by similarity to the input question — a recurring, high-impact trick. <https://arxiv.org/pdf/2407.01409>
- **Investigating LLMs for Text-to-SPARQL** (KnowledgeNLP @ NAACL 2025): systematic study of prompting strategies / model sizes for SPARQL. <https://aclanthology.org/2025.knowledgenlp-1.5.pdf>
- Schema-linking taxonomy (from Text-to-SQL, transfers directly): **string-matching**, **neural**, and **ICL-based** schema linking. <https://arxiv.org/pdf/2510.14296>

**Pros:** no training, adapts to new schemas instantly, strongest on open-domain. **Cons:** token cost per query, sensitive to prompt/example selection, hallucinates IRIs without grounding.

### 2.2 Fine-tuned seq2seq (T5 / BART / LLaMA) — *(b)*

Train an encoder-decoder (or decoder-only) end-to-end on (question → SPARQL) pairs.

- **TosT5 — Triplet-order-sensitive pre-training** (2024): **95.4% F1 / 93.5% QM on LC-QuAD 2.0**, SOTA across model sizes; key insight is that triple-pattern order matters and pre-training should be order-aware. <https://arxiv.org/html/2410.05731v1>
- **FLAN-T5 text2sparql** checkpoints (InfAI) — practical baselines, incl. custom SPARQL tokenizers. <https://huggingface.co/InfAI/flan-t5-text2sparql-naive>
- **Small LMs for Text2SPARQL** (Diomedi/Hogan-style; 2024) — argues SLMs improve resilience of AI assistance for SPARQL, relevant to WASM/on-device. <https://arxiv.org/html/2405.17076v1>
- **Correcting triplets** to enhance SPARQL generation (Appl. Sci. 2024). <https://www.mdpi.com/2076-3417/14/4/1521>
- **AMR → SPARQL transpilation** (IBM, 2021) — structured intermediate representation route. <https://arxiv.org/pdf/2112.07877>

**Pros:** cheap+fast at inference, high accuracy on *fixed* schema, deployable offline (good for WASM). **Cons:** needs training data per schema, brittle to schema drift, weak zero-shot on unseen KGs.

### 2.3 Retrieval-augmented generation grounding on ontology + example queries — *(c)*

Retrieve from (i) the schema (classes/properties + labels/descriptions) and (ii) a corpus of example queries, then condition generation on both.

- **OG-RAG — Ontology-Grounded RAG**: injects ontology entities/relations/constraints into every stage of retrieval+generation. <https://www.emergentmind.com/topics/ontology-grounded-retrieval-augmented-generation-og-rag>
- **Ontotext / GraphDB RAG**, **HippoRAG + graph semantics** (Graphwise) — production KG-RAG patterns. <https://graphwise.ai/blog/from-retrieval-to-reasoning-enhancing-hipporag-with-graph-based-semantics/>
- **"Ontologies to the Rescue"** (Allemang & Sequeda, 2024): ontology context measurably raises LLM QA accuracy on enterprise KGs. <https://arxiv.org/pdf/2405.11706>

**Pros:** grounds the model in the *actual* schema, reduces IRI hallucination, adapts without fine-tuning. **Cons:** retrieval quality is the bottleneck; needs good labels/embeddings of the schema.

### 2.4 Execution-guided decoding & query-repair loops — *(d)*

Execute the (partial or full) query, feed errors/empties back, regenerate.

- **Execution-Guided Decoding for Text-to-SQL** (Wang et al., 2018) — original idea: discard partial decodes that already produce runtime errors / empty intermediate results. <https://arxiv.org/pdf/1807.03100>
- **SPARQL-LLM** (2025-12): 8-step pipeline — decompose into sub-questions → embed concepts → similarity-retrieve context/schema → build prompt → generate → **validate + correct iteratively** (human-readable errors naming wrong classes/predicates + alternatives, fed back) → execute → interpret. Converges "in up to three revision steps." Triplestore-agnostic; evaluated on DBpedia, a private Corporate KG, and bio KGs (UniProt/Cellosaurus/Bgee). Reports **+24% F1 over the Text2SPARQL'25 challenge winners on DBpedia, 36× faster, ≤ $0.01/question** — i.e. a well-engineered retrieve+repair loop can beat heavyweight agents at a fraction of the cost. <https://arxiv.org/html/2512.14277v1>
- **Self-healing SQL pipelines** (2026) — formal error taxonomies + zero-shot self-debug; transfers to SPARQL. <https://arxiv.org/pdf/2604.16511>

**Pros:** biggest single accuracy lift on top of any generator; turns "valid-but-wrong" into "corrected." **Cons:** extra latency (N executions), needs careful loop bounding to avoid runaway.

### 2.5 Grammar-constrained / constrained decoding using the SPARQL grammar — *(e)*

Restrict the decoder so only tokens that keep the output on a valid grammar path can be emitted; partial sequences are validated by an incremental parser.

- **Grammar Prompting for DSL generation** (Wang et al., NeurIPS 2023) — give the BNF in-prompt + constrain. <https://arxiv.org/pdf/2305.19234>
- Constrained-decoding tutorials for structured output (parser-in-the-loop token masking). <https://mbrenndoerfer.com/writing/constrained-decoding-structured-llm-output>
- Ontology-guided hybrid prompt learning for KGQA generalization (2025). <https://arxiv.org/pdf/2502.03992>

**`sparq` angle:** we already link `spargebra` (SPARQL grammar + algebra). Two strengths: (1) **post-hoc validation is free** — parse every candidate, reject non-parsing ones before they ever touch the engine; (2) **constrained decoding** is feasible for *local* models where we control logits (mask to grammar-valid tokens) and partially feasible for API models via grammar-aware sampling APIs (e.g. JSON/GBNF-style). See §8 (novel ideas).

### 2.6 Entity & relation linking — *(f)*

Map NL mentions to IRIs. Often the single largest error source on open-domain.

- **GENRE** (autoregressive entity retrieval) — generate the canonical entity name, constrained to KG titles. (referenced widely in KGQA linking)
- **Generative Relation Linking** (IBM, 2021) — seq2seq for relation linking on QALD/LC-QuAD. <https://arxiv.org/pdf/2108.07337>
- **Spinach** (Stanford OVAL, 2024): agentic SPARQL-based information navigation. A ReAct loop with five actions (search Wikidata, fetch entity page, view property examples, run SPARQL, stop) that *builds the query incrementally* ("start simple, gradually build toward the complete query") using **the full expressiveness of SPARQL for exploration**, not one-edge-at-a-time. Reports (0-shot, GPT-4o): **QALD-7 62.2% EM / 74.6% F1 (+30.1pp), QALD-9-plus 58.3 / 71.6 (+27.0pp), QALD-10 63.1 / 69.5 (+10.0pp), WikiWebQuestions 61.2 / 72.3** (within 1.6pp of a *fine-tuned* baseline). Introduces the **Spinach dataset** (320 hard real Wikidata-forum questions, avg 8.89 clauses/query vs 2.63 for WikiWebQuestions); on it Spinach scores 16.4 EM / 45.3 F1 vs ToG+GPT-4 at 1.8 / 7.2 (**+38.1pp F1**). This is the strongest evidence that **live KG exploration beats static pre-linking** on hard open-domain. <https://arxiv.org/html/2407.11417v1>
- **mKGQAgent** (Perevalov et al., 2025-07; Text2SPARQL'25 **winner**): human-inspired *modular agent* — planning, entity/relation linking, **template grounding**, and iterative SPARQL refinement, with utility tools (vector search + full-text search over the schema + live endpoint querying) and an **experience pool** for ICL. 1st on both DBpedia and Corporate tracks. <https://arxiv.org/pdf/2507.16971>, <https://ceur-ws.org/Vol-4094/>
- **ARUQULA** (Text2SPARQL'25): ReAct + KG-exploration utilities — same agentic family. <https://arxiv.org/html/2510.02200v1>

**`sparq` angle:** entity/relation linking maps cleanly onto our **dictionary + similarity-search module**: build a label→IRI index (rdfs:label, skos:prefLabel, schema:name) with embeddings or trigram/edit-distance fallback; the 6 permutation indexes let us verify candidate predicates and rank by frequency.

### 2.7 Comparison table

| Approach | Repr. systems | Best regime | Acc. (reported) | Cost / latency | Robust to schema drift | Needs training | WASM-friendly | Grounding |
|---|---|---|---|---|---|---|---|---|
| **LLM few-shot ICL + schema link** (a) | ICSU, Dynamic-Few-Shot | open-domain | QALD-10 mid (~0.4 F1 median range) | high (LLM/query) | ✅ | no | ✅ (API) | via retrieval |
| **Fine-tuned seq2seq** (b) | TosT5, FLAN-T5 | fixed schema | **95.4% F1 LC-QuAD2** | low (one fwd pass) | ❌ | yes (per KG) | ✅ (small model on device) | learned, weak |
| **RAG on ontology** (c) | OG-RAG, GraphDB | enterprise/custom KG | +large gains vs no-ctx | medium | ✅ | no | ✅ | strong (explicit) |
| **Exec-guided / repair** (d) | SPARQL-LLM, EG-SQL | any (add-on) | +5–15pp over base | high (N execs) | ✅ | no | ✅ | via execution |
| **Grammar-constrained** (e) | Grammar-Prompting | any (add-on) | guarantees validity; +acc low-resource | low overhead | ✅ | no | ⚠️ (needs logit access) | syntactic only |
| **Entity/relation linking** (f) | GENRE, GenRL, Spinach | open-domain (critical) | dominates error budget | medium | ✅ | optional | ✅ | strong (IRI) |
| **Agentic (compose all)** | **mKGQAgent**, ARUQULA, Spinach | open-domain SOTA | **Text2SPARQL'25 1st** | highest (multi-step) | ✅ | no | ✅ | strongest |

> **Takeaway for `sparq`:** ship an **agentic pipeline** (compose a+c+d+f) as the default NL backend, with grammar-constrained validation (e) bolted on for free via `spargebra`, and an *optional* fine-tuned small-model fast path (b) for fixed-schema / offline / WASM deployments.

---

## 3. Schema linking & grounding (how to fill the prompt)

The model can only use IRIs it's told about. Strategy:

1. **Schema summary from introspection** (separate `sparq` module): enumerate classes (`?c a rdfs:Class`/`owl:Class`, plus `?x a ?c` distinct), properties (distinct predicates), with **counts** (cheap via indexes), **labels/descriptions** (`rdfs:label`, `rdfs:comment`, `skos:definition`), and domain/range where declared. Counts let us prioritize *frequently-used* predicates.
2. **Retrieve the relevant slice** for a given question: embed labels+descriptions of classes/properties; retrieve top-`m` by cosine to the question (reuse the similarity-search module). Fall back to BM25/trigram when no embeddings. This is exactly the Text-to-SQL schema-linking pattern. <https://arxiv.org/pdf/2510.14296>
3. **Few-shot example selection by similarity** — retrieve top-`k` (question, SPARQL) exemplars from a curated/auto-mined pool by question similarity; dynamic selection beats static. <https://arxiv.org/pdf/2407.01409>
4. **Labels/descriptions matter** — they are the bridge across the lexical gap; OG-RAG and "Ontologies to the Rescue" both show explicit schema text lifts accuracy. <https://arxiv.org/pdf/2405.11706>
5. **Budgeting:** schema can be huge → retrieve, don't dump. For small/fixed KGs, dump the whole schema (cheaper than retrieval and lossless).

---

## 4. Benchmarks

### 4.1 Datasets (target KG + what they test)

| Dataset | KG | Size / nature | Tests | Metric(s) |
|---|---|---|---|---|
| **QALD-9 / QALD-9-plus** | DBpedia (+Wikidata in plus) | ~400 Q, multilingual (10 langs in plus) | open-domain, multilingual | Macro P/R/F1 (answer) |
| **QALD-10** | **Wikidata** | ~400 Q, harder, multilingual | open-domain Wikidata shift | answer F1 (median ~0.16→0.42 w/ good systems) |
| **LC-QuAD 1.0** | DBpedia | 5k Q, template-grounded | structural complexity | QM + answer F1 |
| **LC-QuAD 2.0** | Wikidata + DBpedia | **30k Q** | complex, large | **QM + F1 (TosT5: 95.4 F1)** |
| **Text2SPARQL'25 (DB25 / Corporate)** | DBpedia 2015-10 + Corporate | 200 Q (EN+ES) + private | scalability, multilingual, domain adapt | challenge score (answer-based) |
| **WikiWebQuestions** | Wikidata | WebQSP ported to WD | real user questions | answer F1 |
| **WikiSP / WikiQA (Spinach)** | Wikidata | hard real-world | live-exploration QA | answer F1 |
| **GrailQA** | Freebase | 64k Q | i.i.d. / compositional / **zero-shot** generalization | EM + F1 |
| **KQA Pro** | Wikidata-subset (dense) | 120k Q | compositional reasoning, KoPL programs | answer acc |
| **CSQA** | Wikidata | 1.6M QA / 200k dialogs | **conversational** (coref, ellipsis) | answer F1 / acc |
| **SPICE** | Wikidata (12.8M ent) | CSQA + **SPARQL parses**, avg 9.5 turns | conversational semantic parsing | EM / exec F1 |
| **Spider** | relational DBs (SQL) | 10k Q, 200 DBs | *technique transfer only* (cross-DB generalization, schema linking) | EM / exec acc |

Sources: QALD-10 <https://www.researchgate.net/publication/376009186>; LC-QuAD2/TosT5 <https://arxiv.org/html/2410.05731v1>; Text2SPARQL'25 <https://ceur-ws.org/Vol-4094/>; Spinach <https://arxiv.org/html/2407.11417v1>; SPICE/CSQA <https://aclanthology.org/2023.emnlp-main.535.pdf>; KGQA leaderboard <https://arxiv.org/pdf/2201.08174>.

### 4.2 Metrics

- **Query Match (QM / EM):** normalized string/AST equality to gold SPARQL. We can do a *stronger* AST-level match via `spargebra` (parse both, compare algebra up to variable renaming) — fairer than string EM.
- **Execution / answer F1:** macro precision/recall/F1 over answer sets; the QALD standard. Handles `ASK`/`SELECT`/`COUNT` shapes.
- **Validity rate:** % of generated queries that parse (and that reference only existing IRIs). A `sparq`-specific metric our grammar-constraint should push toward 100%.
- **Latency / cost:** wall-clock + LLM tokens (and # of repair iterations) per question — report alongside accuracy (the "opt-in must ship perf benchmarks" rule).

### 4.3 Reproducible offline harness (plan)

- Vendor each dataset's test split into `bench/nlq/<dataset>/` (questions + gold SPARQL + gold answers).
- **Pin the KG snapshot** (DBpedia 2015-10 for DB25; a fixed Wikidata truthy dump or subset). Load into `sparq`; record dataset hash.
- Run with a **deterministic LLM config** (temperature 0, fixed model id + version) and also a **local-small-model** config for fully-offline reproducibility (no network). Record model + prompt template hashes.
- Grade with our own `spargebra`-based QM + an answer-F1 scorer that *executes* gold and predicted on the *same* `sparq` instance (eliminates endpoint drift — a known QALD reproducibility pain).
- Emit a CSV: `dataset, model, F1, QM, validity, mean_iters, p50_latency_ms, tokens`. Gate CI on no-regression for the offline small-model path.

---

## 5. Multi-turn / conversational

- **Phenomena:** coreference ("his movies"), ellipsis ("and the ones after 2010?"), topic shift, result-referencing ("which of those won an Oscar?"). CSQA is the canonical source of these. <https://arxiv.org/pdf/1910.05069>
- **Datasets:** **CSQA** (1.6M QA / 200k dialogs) and **SPICE** (CSQA + SPARQL parses, avg 9.5 turns, 12.8M-entity KG) are the targets; SPICE is the one with executable SPARQL. <https://aclanthology.org/2023.emnlp-main.535.pdf>
- **Two architectural patterns:**
  1. **Question rewriting** — rewrite the follow-up into a standalone question using history (Ke et al., EMNLP'22 self-training rewriter), then run single-turn Text-to-SPARQL. Simplest; decouples conversation from generation. <https://xiaojingzi.github.io/publications/EMNLP22-Ke-et-al-ConversationKBQA.pdf>
  2. **Contextual semantic parsing** — carry a *dynamic context graph* / prior SPARQL + prior result entities into the prompt; resolve coref against prior bindings. <https://aclanthology.org/2023.emnlp-main.535.pdf>
- **`sparq` plan:** keep a `Conversation` struct holding (prior questions, prior SPARQL ASTs, prior result entity IRIs, prior schema slice). On each turn: (i) cheap LLM "is this a follow-up?" + rewrite-to-standalone, **and** (ii) expose prior result IRIs as candidate bindings for coref ("those" ↦ last result set). Carry the retrieved schema slice forward to save retrieval cost.

---

## 6. Validation & safety

1. **Syntactic validation (free):** `spargebra::Query::parse` every candidate; reject/repair non-parsing ones *before* execution.
2. **Schema validation:** walk the parsed algebra, collect all IRIs in predicate/class position, check each against the dictionary + introspection index. Unknown IRI → targeted repair message ("predicate `dbo:directr` not found; did you mean `dbo:director`?"). This is the SPARQL-LLM repair pattern, but we get the *candidate alternatives for free* from our label index. <https://arxiv.org/html/2512.14277v1>
3. **Runaway-query prevention:** before executing an LLM-authored query, enforce limits — inject/enforce a `LIMIT`, set a **time budget + row-count cap** (the engine already has out-of-core execution; add a cancellation token + cost estimate from index cardinalities). Reject unbounded cross-products (no shared variable) with an explanation.
4. **"I don't know":** if after the repair budget the query still doesn't parse, references no valid IRIs, or returns empty *and* the model's confidence is low, surface an honest abstention with the closest schema terms — never a fabricated answer. Empty result ≠ failure (could be a true negative), so distinguish "query invalid" from "query valid, zero rows."
5. **Read-only guarantee:** the NL layer must only emit `SELECT`/`ASK`/`CONSTRUCT`/`DESCRIBE`; reject `UPDATE`/`LOAD`/`CLEAR`/`DROP` at the algebra level so a prompt-injected question can't mutate the store.

---

## 7. Recommended architecture for `sparq`

### 7.1 Crate layout (opt-in, zero-impact)

New crate `crates/sparq-nlq` (mirrors how `sparq-reason` is separated), behind a workspace cargo feature `nlq`. The default engine never compiles it.

```text
crates/sparq-nlq/
  src/
    lib.rs            // ask(), Conversation, public API
    backend.rs        // LlmBackend trait (pluggable: API / local / wasm-callback)
    schema.rs         // introspection: classes, properties, counts, labels  (reads engine)
    link.rs           // entity/relation linking via dictionary + similarity-search module
    retrieve.rs       // schema-slice + few-shot example retrieval
    prompt.rs         // prompt assembly / templates
    validate.rs       // spargebra parse + algebra walk + IRI check + safety
    repair.rs         // execution-guided self-correction loop
    constrain.rs      // (novel) grammar-constrained decoding driver
```

- **Zero perf/memory impact:** nothing in `sparq-core`/`sparq-engine` depends on `sparq-nlq`; it depends on *them* (one-directional). No HTTP client, tokenizer, or embedding lib enters the default build. Removing the crate is deleting a directory + one `members`/feature line.
- **WASM:** `LlmBackend` is a trait; the WASM build injects a JS callback (`fetch` to an API, or an in-browser small model via transformers.js / WebLLM). No native-only deps in the WASM path (the linking/validation/retrieval are pure Rust over the in-memory store).

### 7.2 Pluggable LLM backend

```rust
pub trait LlmBackend {
    /// Free-form completion (API or local model).
    fn complete(&self, prompt: &str, cfg: &GenCfg) -> Result<String>;
    /// OPTIONAL: grammar-constrained generation. If the backend supports
    /// logit masking / GBNF, sparq supplies a SPARQL grammar driver.
    fn complete_constrained(&self, prompt: &str, grammar: &SparqlGrammar, cfg: &GenCfg)
        -> Option<Result<String>> { None }  // default: unsupported
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;  // for retrieval/linking
}
```

Reference impls: `ApiBackend` (OpenAI/Anthropic/Ollama-compatible HTTP), `WasmBackend` (JS callback), optional `LocalBackend` (a small fine-tuned T5/LLaMA via `candle`, feature `nlq-local`). The engine ships **no API keys** and makes the backend mandatory-to-supply.

### 7.3 The `ask` pipeline (agentic, composes a+c+d+e+f)

```text
ask(question, &mut conversation) -> Answer { sparql, results, explanation, confidence }
```

1. **Conversation resolve** (if history): rewrite follow-up → standalone; stage prior result IRIs as coref candidates. (§5)
2. **Link** mentions → candidate IRIs via `link.rs` (label index + similarity search + index-frequency ranking). (§2.6)
3. **Retrieve** schema slice (top-`m` classes/props by question similarity) + top-`k` few-shot examples. (§3)
4. **Generate** SPARQL with the LLM:
   - if backend supports it → **grammar-constrained** generation (`constrain.rs`, §8.1) — output is guaranteed to parse;
   - else → plain generation.
5. **Validate** (`validate.rs`): `spargebra` parse → algebra walk → IRI existence check → safety (read-only, bounded). (§6)
6. **Repair loop** (`repair.rs`, ≤N iters): on parse error / unknown IRI / empty-and-low-confidence, build a structured feedback message (with `did-you-mean` alternatives from the label index) and re-prompt. (§2.4) Bounded by `N` (default 3) and a wall-clock budget.
6b. **(optional, hard questions) Local KG exploration** — Spinach-style: instead of one-shot generation, let the LLM issue cheap *probe* queries against the in-memory store (neighbors of a linked entity, example values of a predicate, "does class C have property P?") and build the final query incrementally. Because these probes hit local indexes (no network), the per-probe cost is microseconds vs Spinach's HTTP round-trips. Gate this behind a `--explore` flag (it costs more LLM turns). (§2.6, §8.7)
7. **Execute** on the engine under a time/row cap; optionally **expand with inference** (`sparq-reason`) when the raw query is empty but a materialized closure would answer it. (§8.2)
8. **Explain**: return the final SPARQL, the results, an NL explanation (LLM verbalizes the query + how it maps to the question), and a confidence (function of repair iters, IRI-link scores, result size).

### 7.4 API surface

```rust
let mut conv = Conversation::new();
let ans = engine.nlq(&backend).ask("Who directed Pulp Fiction?", &mut conv)?;
// ans.sparql, ans.results, ans.explanation, ans.confidence
let ans2 = engine.nlq(&backend).ask("And what year was it released?", &mut conv)?; // follow-up
```

Also expose lower-level building blocks (`to_sparql`, `validate_sparql`, `schema_summary`) so users can compose their own flows or BYO generator and still use our validation/repair.

---

## 8. Novel ideas (sparq-specific, clearly separated)

### 8.1 Grammar-constrained SPARQL decoding via `spargebra`
We already embed the SPARQL grammar. Build an **incremental SPARQL acceptor** from `spargebra`'s grammar so that, at each decode step on a *local* backend, we mask the logits to tokens that keep the prefix on a valid parse path (GBNF/Earley-style parser-in-the-loop). Result: **100% syntactic validity by construction**, no wasted repair iterations on syntax. For API backends that accept a supplied grammar (GBNF / structured-output), emit a SPARQL GBNF derived from the same grammar. This is the cleanest "we have the grammar, use it" win and, to our knowledge, no shipped Text-to-SPARQL system constrains against a real production SPARQL grammar + then *also* algebra-validates IRIs.

### 8.2 Inference-augmented answering (use `sparq-reason`)
A raw SPARQL query over the explicit triples can return empty when the answer is only *entailed* (e.g. transitive `rdfs:subClassOf`, `owl:sameAs`, property chains). Novel loop: when a validated query returns empty, **materialize the relevant closure** (`sparq-reason`) and re-execute — answering questions a pure Text-to-SPARQL stack *cannot*. The LLM can even be told "these predicates are inferable" so it writes simpler queries and lets the reasoner do the multi-hop. No LLM-only competitor has a built-in reasoner.

### 8.3 Index-grounded entity/relation linking with cardinality priors
Because we have the dictionary + 6 permutation indexes, linking can rank candidate predicates/classes by **actual usage frequency** (cardinality) and verify a candidate triple pattern *exists* before committing — e.g. prefer `dbo:director` over `dbo:directors` because the former has 10⁵ uses and the latter 0. This is grounding that endpoint-blind LLM stacks can't cheaply do.

### 8.4 Algebra-level query-equivalence for fair QM
Use `spargebra` to compare predicted vs gold queries at the **algebra level up to variable renaming / BGP reordering**, not string EM — a more honest QM metric, and reusable as a *self-consistency* signal (sample K queries, keep the algebra-modal one).

### 8.5 Schema-slice caching across a conversation
Retrieve the schema slice once per topic and **carry it across turns**; only re-retrieve on detected topic shift. Cuts per-turn retrieval+token cost in multi-turn sessions.

### 8.6 Cost-bounded planning preview
Before executing an LLM query, use the engine's **cardinality estimates** to predict cost; if it explodes (unbounded join), *return the estimate to the LLM* and ask it to add selectivity (a `FILTER`/`LIMIT`/more specific class) — execution-guided *planning*, not just error-guided repair.

### 8.7 Zero-latency local KG exploration (Spinach, but in-process)
Spinach's gains come from probing the *live* endpoint to discover IRIs and structure — but each probe is a network round-trip. In `sparq`, the "endpoint" is the in-process store, so a probe (`neighbors(entity)`, `sample_values(predicate)`, `count(pattern)`) is a microsecond index lookup. This makes iterative query-building cheap enough to run *by default* on hard questions, and we can expose a small, safe **exploration toolset** to the LLM (read-only, bounded, no `UPDATE`) rather than free-form SPARQL during exploration.

---

## 9. Recommended build order

1. **M0 (foundation):** `schema.rs` introspection + `validate.rs` (`spargebra` parse + IRI check + read-only/bounded safety). Useful even without an LLM (BYO-SPARQL validation). Add the algebra-level QM scorer (§8.4).
2. **M1 (single-turn baseline):** `ApiBackend` + retrieval (§3) + plain generation + repair loop (§2.4/§6). Benchmark on QALD-10 + LC-QuAD 2.0 + Text2SPARQL DB25 (offline harness §4.3).
3. **M2 (grounding):** `link.rs` index-grounded linking (§8.3) + few-shot dynamic selection. Re-benchmark.
4. **M3 (conversational):** `Conversation` + follow-up rewrite + coref via prior bindings; benchmark on SPICE.
5. **M4 (novel):** grammar-constrained decoding (§8.1) for a local backend; inference-augmented answering (§8.2); WASM backend.

---

## 10. Open questions / risks

- **Grammar-constrained decoding needs logit access** — only works on local/open backends; API backends get GBNF-if-supported or fall back to post-hoc validation. Quantify the validity gap.
- **Entity linking is the dominant error source** on open-domain — the similarity-search module's quality caps end-to-end accuracy; budget effort there.
- **Reproducibility of answer-F1** depends on a pinned KG snapshot; vendor exact dumps/subsets.
- **Latency budget** for the agentic loop (multi-step + N repairs) — must report it; offer a one-shot fast path (fine-tuned small model, no loop) for latency-sensitive/WASM use.

---

## 11. Source index (primary, with dates where known)

- TEXT2SPARQL'25 proceedings (CEUR Vol-4094, ESWC 2025-06-01): <https://ceur-ws.org/Vol-4094/>
- ARUQULA (ReAct + KG exploration, 2025): <https://arxiv.org/html/2510.02200v1>
- Multilingual human-inspired Text-to-SPARQL (2025-07): <https://arxiv.org/abs/2507.16971>
- TosT5 triplet-order pre-training (2024-10): <https://arxiv.org/html/2410.05731v1>
- ICSU in-context schema understanding (2023-10): <https://arxiv.org/pdf/2310.14174>
- Dynamic Few-Shot for KGQA (2024-07): <https://arxiv.org/pdf/2407.01409>
- Investigating LLMs for Text-to-SPARQL (NAACL KnowledgeNLP 2025): <https://aclanthology.org/2025.knowledgenlp-1.5.pdf>
- Execution-Guided Decoding for Text-to-SQL (2018): <https://arxiv.org/pdf/1807.03100>
- SPARQL-LLM real-time generation + repair (2025-12): <https://arxiv.org/html/2512.14277v1>
- Grammar Prompting for DSLs (NeurIPS 2023): <https://arxiv.org/pdf/2305.19234>
- Ontology-Guided Hybrid Prompt Learning (2025-02): <https://arxiv.org/pdf/2502.03992>
- "Ontologies to the Rescue" (2024-05): <https://arxiv.org/pdf/2405.11706>
- Generative Relation Linking (IBM, 2021): <https://arxiv.org/pdf/2108.07337>
- AMR→SPARQL transpilation (2021): <https://arxiv.org/pdf/2112.07877>
- Spinach agentic SPARQL navigation (2024-07): <https://arxiv.org/html/2407.11417v1>
- KGQA leaderboard (2022): <https://arxiv.org/pdf/2201.08174>
- QALD-10 (2023): <https://www.researchgate.net/publication/376009186>
- SPICE conversational semantic parsing (EMNLP 2023): <https://aclanthology.org/2023.emnlp-main.535.pdf>
- Conversational KBQA question rewriter (EMNLP 2022): <https://xiaojingzi.github.io/publications/EMNLP22-Ke-et-al-ConversationKBQA.pdf>
- Multi-task conversational KBQA / CSQA (2019): <https://arxiv.org/pdf/1910.05069>
- Schema-linking bidirectional retrieval (2025-10): <https://arxiv.org/pdf/2510.14296>
- Small LMs for Text2SPARQL (2024-05): <https://arxiv.org/html/2405.17076v1>
- Small LMs / SLM Text2SPARQL checkpoints (InfAI): <https://huggingface.co/InfAI/flan-t5-text2sparql-naive>
