# GenAI design synthesis: one prioritized plan (T7)

**Status:** design synthesis, 2026-06-10. Synthesizes the five research reports —
[`genai-iri-similarity.md`](genai-iri-similarity.md), [`genai-nl-to-sparql.md`](genai-nl-to-sparql.md),
[`genai-ontology-introspection.md`](genai-ontology-introspection.md),
[`genai-kg-embeddings-vectorindex.md`](genai-kg-embeddings-vectorindex.md),
[`genai-benchmarks-and-synthesis.md`](genai-benchmarks-and-synthesis.md) — into ONE prioritized
design. The reports remain the references; this document is the plan of record.

## 0. Invariants (inherited, non-negotiable)

- Every GenAI feature is an **opt-in crate**, trivially removable, **zero perf/memory/binary impact**
  on the default engine. No core crate is modified; gaps in public APIs are captured as
  beads (`bd`), not in-repo `TODO.md` files. <!-- [OPUS-4.8] doc-sweep: align with AGENTS.md bead policy -->
- Approximate signals **never serve BGP answers** (data-structures §R6/R7): they power similarity /
  retrieval / linking / planner *estimates* only. Exact stays exact.
- Every feature ships with **accuracy AND performance benchmarks** (machine spec inline, M1 Air
  house style per `genai-benchmarks-and-synthesis.md` §A.4.4).
- The shared substrate is the **dict u32 id-space + six sorted permutation indexes**: signatures,
  vectors, stats and linking indexes are all keyed by dict id — no side join tables.

## 1. The four crates and their priority order

Priority = (impact × uniqueness) / (effort × external-dependency risk). Training-free,
index-native features first; model-dependent features last.

| # | Crate | What it is | Why this rank |
|---|-------|------------|---------------|
| **1** | **`sparq-sim`** | **Training-free structural similarity** from the existing permutation indexes: structural signatures = (predicate, neighbor) sets from SPO (out) + OPS/OSP (in); pred-IDF-weighted Jaccard; index-driven candidate generation. Later: FST lexical tier over the sorted dictionary, MinHash/LSH sketches. | The **novel edge** (iri-similarity §7.2): no other store gets this for free; zero models, zero network, WASM-trivial, incremental (indexes are the feature store). Everything downstream (linking, hybrid retrieval) consumes it. |
| **2** | **`sparq-introspect`** | Effective-schema mining by sorted scans: predicate histograms (PSO/POS), class extents (POS @ rdf:type), **characteristic sets** (one SPO scan), observed domain/range, VoID export, token-budgeted "schema cards" for LLM grounding. | Nearly free given sorted permutations (introspection §0.1); dual-use — CS feeds the planner's cardinality model *and* the NL→SPARQL prompt. Pure index ops, no models. Hard dependency of crate 3. |
| **3** | **`sparq-nlq`** | NL→SPARQL: lean retrieve → generate → validate (spargebra parse) → execute → repair loop (SPARQL-LLM-style, not a sprawling agent); schema cards from `sparq-introspect`, entity/relation linking from `sparq-sim`; LLM behind a trait (record/replay cache, offline CI). Later: grammar-constrained decoding against the live dictionary (synthesis N2). | Highest product value but depends on 1 + 2 and an external LLM; the engine-side contribution (linking + validation + execution) is benchmarked LLM-excluded (synthesis §A.4.3). |
| **4** | **`sparq-vectors`** | Embedding storage + ANN: mmap `vectors.bin` keyed by dict id (`id*dim` offset), HNSW (`usearch`-class) in-RAM/WASM, DiskANN-style out-of-core later; embeddings produced out-of-process (OpenAI-compatible `/v1/embeddings`) or imported (PyKEEN dumps); optional GNCE-style planner cardinality hook. | Heaviest external surface (models, ANN deps, quantization); the other crates degrade gracefully without it (sim is the no-model fallback). Hybrid RRF fusion (lexical + structural + vector) lands here-ish, once ≥2 signals exist. |

## 2. Phasing

- **Phase 1 (now): `sparq-sim` v1** — structural signatures + weighted-Jaccard similarity +
  `most_similar` with index-driven candidate generation; pred-IDF from existing `PredStat`s;
  olympics + synthetic-taxonomy accuracy gate; latency gate. *No new dependencies beyond
  `sparq-core` + `oxrdf`.*
- **Phase 2: `sparq-introspect` v1** — predicate/class histograms, characteristic sets, VoID export,
  schema cards with a token budget. Then `sparq-sim` v1.1: FST lexical tier + T-box-aware signatures
  (closure via `sparq-reason`, opt-in per iri-similarity §7.4).
- **Phase 3: `sparq-nlq` v1** — linking (sim + introspect) → prompt → spargebra-validate → execute →
  repair (≤3 rounds); QALD-style harness with record/replay LLM cache; report exec-acc and
  LLM-excluded latency. Then N2 grammar-constrained decoding.
- **Phase 4: `sparq-vectors` v1** — mmap vector store + HNSW + import path; hybrid RRF
  (`lexical + struct + vector`); zero-impact proof bench. MinHash/LSH sketches in `sparq-sim` if
  `most_similar` candidate generation hits its scaling wall first.

## 3. APIs (sketch; stable surface kept minimal)

```rust
// sparq-sim (Phase 1 — implemented now)
Sim::new(&Graph) -> Sim;  Sim::with_config(&Graph, SimConfig) -> Sim
sim.signature(&Term) -> Option<Signature>            // (dir, pred, neighbor) set, IDF-weighted
sim.similarity(&Term, &Term) -> f64                  // weighted Jaccard in [0,1]
sim.most_similar(&Term, k) -> Vec<(Term, f64)>       // index-driven candidates, exact re-score
sim.similar_by_signature(&Signature, k) -> Vec<(Term, f64)>
// SimConfig: mode (Predicates | PredicateNeighbor), idf on/off, excluded predicates,
//            candidate-frequency cap (the documented approximation knob)

// sparq-introspect (Phase 2)
Introspect::new(&Graph); .predicates() / .classes() -> ranked histograms
.characteristic_sets(top_k); .observed_domain_range(pred); .void_document()
.schema_cards(TokenBudget) -> String   // the LLM grounding deck

// sparq-nlq (Phase 3)
Nlq::new(&Graph, Box<dyn LlmBackend>); nlq.ask(&str) -> Answer { sparql, rows, repairs, provenance }
// LlmBackend: generate(prompt) -> String; ReplayBackend for offline CI

// sparq-vectors (Phase 4)
VectorStore::build(&Graph, EmbeddingSource) / ::open(dir); .nearest(id | &[f32], k, Filter)
```

## 4. Benchmark gates (each phase merges only if green)

Protocol per `genai-benchmarks-and-synthesis.md`: machine spec inline; accuracy AND performance;
**zero-impact proof** — workspace SPARQL benches unchanged with the new crate present-but-unused
(it is a separate crate, so the default build does not even compile it).

| Crate | Accuracy gate | Performance gate |
|---|---|---|
| `sparq-sim` | rdf:type-as-ground-truth (type triples excluded from signatures): same-class entities rank above cross-class — **AUC > 0.9** on the generated taxonomy; on olympics (1.78M triples, stratified minority classes) **precision@10 > 0.7** for `most_similar` and **AUC > 0.8** in the role-similarity (`Predicates`) mode. (Pairwise AUC in `PredicateNeighbor` mode is *not* gated: two arbitrary same-class entities usually share no concrete neighbor — that mode measures shared context, and is judged by retrieval precision.) **Measured (M1, 2026-06-10): precision@10 = 0.999, Predicates-AUC = 1.000, neighbor-mode pairwise AUC = 0.61 — green.** | `most_similar(k=10)` **ms-level** (p50 < 10 ms, p95 < 100 ms) on olympics scale, measured + recorded in the crate README. **Measured: p50 0.13 ms, p95 4.9 ms, max 7.1 ms — green.** |
| `sparq-introspect` | CS-based cardinality estimates beat the current `PredStat` estimator (q-error, WatDiv); schema-card facts spot-checked exact (they are counts, not guesses). | Full introspection scan ≤ load time of the dataset; schema card < 100 ms at olympics scale. |
| `sparq-nlq` | Exec-accuracy on QALD-10-style harness with pinned dump + recomputed gold; report oracle-linking vs end-to-end separately; ≥ baseline of the same LLM without grounding (the grounding must pay for itself). | LLM-excluded store latency (linking + validate + execute) p50 < 50 ms; repair loop ≤ 3 rounds. |
| `sparq-vectors` | recall@10 vs exact NN ≥ 0.9 at the default operating point; ann-benchmarks-style recall/QPS curve, not a single point. | Build time + RSS at 10M/100M; query p50 sub-ms in-RAM; mmap open near-instant. |

## 5. Decisions resolved against the reports

1. **Lead structural, not vector** (vs iri-similarity §9 which leads lexical): the structural tier is
   the *headline novelty* (§7.2) and needs zero new deps; the FST lexical tier is additive in v1.1.
2. **Lean NLQ loop, not agent** (nl-to-sparql §0.1): retrieve + generate + validate + repair.
3. **Embeddings out-of-process** (iri-similarity §6.2, kg-embeddings §0.11): sparq stores + searches
   vectors; it never runs a model in-process in v1 (`candle`/`ort` revisited for WASM later).
4. **Characteristic sets are built once, used twice** (introspection §0.2): schema summary + planner.
5. **rdf:type-exclusion eval discipline**: any eval that scores against type ground truth must
   exclude type triples from the input signal — leakage rule, documented in each harness.
6. **Signature direction**: both directions (SPO out + OPS/OSP in) — role similarity needs in-edges
   (iri-similarity §7.2); direction is part of the signature element, not folded away.

## 6. Risks

- **Hub pairs** blow up `most_similar` candidate generation → frequency cap (documented approximation;
  IDF makes capped pairs low-information anyway); MinHash/LSH sketches are the Phase-4 escape hatch.
- **Skewed class ground truth** (olympics is 98% foaf:Person) → stratified per-class sampling in evals.
- **LLM nondeterminism in CI** → record/replay cache committed to the repo (synthesis §A.5).
- **WASM bloat** → only `sparq-sim` (and later the lexical FST) targets WASM by default; vectors ship
  precomputed blobs.
