# Ontology / Schema Introspection APIs for `sparq`

**Automatically understanding the ontologies and shapes a knowledge graph uses, and producing compact, token-budget-bounded summaries that ground an LLM for NL→SPARQL and help humans explore.**

A design + literature document for the `sparq` engine (dictionary-encoded RDF, 6 sorted permutation indexes SPO/SOP/PSO/POS/OSP/OPS, out-of-core mmap, SPARQL 1.1, WASM port). The introspection feature is **opt-in**, lives in a **separate crate** gated by a cargo feature, with **zero perf/memory impact** on the default engine, and ships with **accuracy AND performance benchmarks**.

> Status: living document — written incrementally so partial work survives interruption.

---

## 0. Executive summary (load-bearing facts)

1. **The 6 permutation indexes already contain the entire effective schema.** A predicate histogram is one linear scan of **PSO** (each P-block start gives `|{s : (s,p,·)}|`); class extents are runs in **POS** for `p = rdf:type`; per-subject out-degree and per-object in-degree fall out of **SPO**/**OSP** block boundaries. Almost every statistic the schema-summarization literature computes with expensive SPARQL `GROUP BY` queries is, in `sparq`, a **sorted-scan with no hashing**. This is the core thesis of this document: **schema introspection is nearly free given sorted permutations.**

2. **Characteristic Sets (Neumann & Moerkotte, ICDE 2011) are the single most valuable structure** — they serve a *dual* purpose: (a) the SOTA join-cardinality estimator for the query planner, and (b) a compact, data-derived schema ("entity types defined by their property set"). One structure feeds both the cost model and the LLM grounding prompt. **Recommend: build them once at index-build time, expose via the introspection API, and wire them into the planner.**

3. **An "effective schema" mined from instance data beats the declared ontology** for NL→SPARQL grounding, because real KGs are under-specified, mis-typed, and use properties outside their declared domain/range. Prior art (VoID, Loupe, ABSTAT, SchemaCrawler-style profiling, LSQ query logs) all converge on the same primitives: classes-by-type-count, properties-by-usage, *observed* domain/range, cardinalities, sample values, and class–property co-occurrence patterns `(C, p, D)`.

4. **Vocabulary detection grounds prefixes in human language.** Map namespaces → known vocabularies via a *bundled offline* LOV/prefix.cc snapshot (WASM-safe, no network), then optionally dereference unknown ontology IRIs to pull `rdfs:label`/`rdfs:comment`/`skos:definition`. This turns `wdt:P31` into "instance of (Wikidata)".

5. **Shapes are the bridge to validation.** Inferred SHACL node shapes (per characteristic set / per class) both *describe* the graph to the LLM and *validate* generated SPARQL (does the query touch a property that exists on that class? with the right datatype/cardinality?). QSE/SHACTOR show shape mining scales to Wikidata.

6. **The output is a token-bounded "schema card" deck.** The deliverable for an NL→SPARQL prompt is not the whole schema (Wikidata ≈ 10k properties) but a *ranked, budgeted* textual data-dictionary: top classes by frequency, their characteristic property sets, observed ranges, sample IRIs/labels, and prefix glossary — selected greedily under a token budget, with a retrieval mode for "give me the schema around these entities/terms."

7. **Zero-impact opt-in.** Stats are computed on demand by scanning existing indexes (no new state in the hot engine) OR persisted as an optional sidecar file built post-ingest. The default build links none of it.

---

## 1. Literature map

### 1.1 Dataset/ontology description vocabularies (the "what to compute" baseline)

- **VoID — Vocabulary of Interlinked Datasets** (W3C Interest Group Note, 2011). RDF Schema vocabulary for dataset metadata: `void:triples`, `void:entities`, `void:classes`, `void:properties`, and crucially `void:classPartition` (all triples about entities of a given `rdf:type`) and `void:propertyPartition` (all triples with a given predicate), each carrying its own count statistics. This is the canonical *shape* of a statistical schema summary. Spec: <https://www.w3.org/TR/void/>. Design paper (Alexander, Cyganiak, Hausenblas, Zhao): <https://www.researchgate.net/publication/228974311>. VoID-ext adds finer stats: <http://akswnc7.informatik.uni-leipzig.de/dstreitmatter/archivo/ldf.fi/void-ext/>.
  - **Relevance to sparq:** VoID's class/property partitions map *exactly* onto runs in POS (`rdf:type`) and PSO (predicate blocks). `sparq` can emit a VoID+VoID-ext document essentially for free as one export format of the introspection API.

- **Loupe** (Mihindukulasooriya et al., ISWC 2015 demo; SDSVoc 2016 model). Online tool + model that inspects a dataset for **implicit triple patterns** of the form `(subjectType, predicate, objectType)` and explicit ontology axioms. Computes per-class instance counts + associated properties with cardinalities; per-property triple counts + *estimated* domains/ranges + value ranges/patterns. Uses parameterized SPARQL templates. Paper: <https://ceur-ws.org/Vol-1486/paper_113.pdf>. Model: <https://www.w3.org/2016/11/sdsvoc/SDSVoc16_paper_7>.
  - **Relevance:** Loupe's "abstract triple pattern" `(C, p, D)` is the central unit; in `sparq` it is computable from POS+type-joins. Its "estimated domain/range" = the OSP/POS histogram idea below.

- **ABSTAT** (Spahiu, Porrini, Palmonari, Rula, Maurino — ESWC 2015 + ISWC 2016). Ontology-driven linked-data summaries combining **abstraction** (lift instances to their minimal types via the ontology) with **statistics**. Key idea: **minimal type patterns** — a pattern `(C, p, D)` is kept only if there is no more specific `(C', p, D')` subsuming the same assertions (pattern minimalization via the type hierarchy), which dramatically shrinks the summary while preserving coverage. Output is RDF, queryable via SPARQL, navigable in a UI to help users **write SPARQL**. Papers: <https://link.springer.com/chapter/10.1007/978-3-319-25639-9_25>, <http://km.aifb.kit.edu/ws/sumpre2016/paper_3.pdf>.
  - **Relevance:** Minimalization is exactly the trick needed to keep a *token-bounded* summary informative — don't list `(Thing, p, Thing)` if `(Person, p, Email)` is what the data shows. Requires the class hierarchy (declared or materialized).

- **SchemaCrawler / relational-style data profiling.** General DB profiling (column cardinality, null rate, value distribution, candidate keys, FK detection). The RDF analogue: treat each predicate as a "column," compute distinct-subject / distinct-object cardinality, datatype distribution, min/max/sample, and functional-property detection (is `p` ≤ 1 per subject?). Establishes the *vocabulary of statistics* a profiler should expose.

### 1.2 Characteristic Sets — the keystone (dual: schema + cardinality)

- **Neumann & Moerkotte, "Characteristic sets: Accurate cardinality estimation for RDF queries with multiple joins," ICDE 2011, pp. 984–994.** PDF: <https://www.csd.uoc.gr/~hy561/papers/storageaccess/optimization/Characteristic%20Sets.pdf>. DBLP: <https://dblp.org/rec/conf/icde/NeumannM11.html>.
  - **Definition.** For a subject `s`, its characteristic set `S_C(s) = { p : ∃o (s,p,o) ∈ G }` — the *set of predicates emitted by s*. The schema-level object is the **set of distinct characteristic sets** over all subjects, each annotated with `count` (how many subjects have exactly this set) and, per predicate in the set, the average multiplicity (`Σ` of objects / count). A star query `?s p1 ?o1 . ?s p2 ?o2 …` selecting predicate set `Q` has cardinality `Σ_{C ⊇ Q} count(C) · Π_{p∈Q} mult(C,p)` — provably accurate because it captures *predicate correlations* that independence-assuming estimators miss.
  - **Why it is also a schema.** A characteristic set *is* an emergent entity type: "things that have {`rdf:type`, `foaf:name`, `foaf:mbox`}". The frequent CSs are the de-facto classes of the data, even when no ontology declares them. This is the basis of the **"schema card" novelty** in §7.
  - **Scaling note.** The number of distinct CSs can be large (long tail); standard practice merges/prunes rare CSs and bounds the count (Neumann-Moerkotte bound the top-K and bucket the remainder). See also sampling-based estimation: "Estimating Characteristic Sets for RDF Dataset Profiles based on Sampling" (ESWC 2020): <https://preprints.2020.eswc-conferences.org/121230144.pdf>.
  - **sparq computation.** One scan of **SPO**: subjects are contiguous; for each subject emit the sorted predicate list (dedup adjacent), hash it to an id, accumulate count + per-predicate multiplicity. O(|G|) time, O(#distinct CS) memory. No join. This is the single most important build-time artifact.

- **Extensions:** "Estimating the Cardinality of Conjunctive Queries over RDF" (arXiv:1801.09619, <https://arxiv.org/pdf/1801.09619>) generalizes CSs to handle bound objects and chains; **SumRDF**, **LMKG** (learned, arXiv:2102.10588) are the learned alternatives. For `sparq` the exact CS table is preferred (cheap given sorted indexes, interpretable, dual-use).

### 1.3 Vocabulary / ontology detection

- **LOV — Linked Open Vocabularies** (Vandenbussche, Atemezing, Poveda-Villalón, Vatant; Semantic Web Journal 2017). Curated catalogue of ~700 reusable vocabularies with metadata, term search, and a public API (Search Term / Search Vocab). Paper: <https://semantic-web-journal.net/system/files/swj974.pdf>. API: <https://lov.linkeddata.es/dataset/lov/api>.
- **prefix.cc** — crowd-sourced prefix↔namespace registry (e.g. `foaf:` → `http://xmlns.com/foaf/0.1/`). LOV aligns its prefixes with prefix.cc. Polysemy/synonymy tolerated. Site: <https://prefix.cc/>.
  - **Relevance:** `sparq` should bundle an **offline snapshot** (prefix→namespace→human-name + vocab title/description) so namespace identification works in WASM with no network, with optional live dereferencing as an enhancement.
- **Ontology dereferencing:** for namespaces not in the snapshot, fetch the ontology IRI (follow `Link: rel=describedby`/content-negotiation), parse `owl:Ontology`/`rdfs:label`/`rdfs:comment`/`skos:prefLabel`/`skos:definition`/`rdfs:domain`/`rdfs:range`/`sh:*` and cache. This yields human descriptions for `rdfs:`/`owl:`/`skos:`/`shacl:` terms actually used.

### 1.4 SHACL / shapes mining

- **QSE — Quality Shapes Extraction** (Rabbani, Lissandrini, Hose; VLDB 2023, "Extraction of Validating Shapes from very large Knowledge Graphs"). PDF: <https://www.vldb.org/pvldb/vol16/p1023-rabbani.pdf>. Repo: <https://github.com/dkw-aau/qse>. Site: <https://relweb.cs.aau.dk/qse/>. Extracts SHACL node/property shapes from huge KGs, annotating each constraint with **support** (how many nodes satisfy it) and **confidence** (fraction of the class exhibiting it), enabling pruning of spurious shapes. **12× faster** extraction than prior work, filters up to **93%** spurious shapes, **first to extract complete shapes from Wikidata.** Pre-extracted shapes for Wikidata/DBpedia/YAGO-4/LUBM on Zenodo: <https://zenodo.org/records/7598613>.
- **SHACTOR** (Rabbani et al., SIGMOD 2023 demo) — web tool over QSE that visualizes extraction + support/confidence and flags erroneous/missing triples. <https://people.cs.aau.dk/~matteo/pdf/SIGMOD23-SHACTOR-demo.pdf>.
- **SHACLearner** — learns SHACL constraints via inductive logic programming style mining (deriving shapes from positive instances). (To verify/cite below.)
  - **Relevance:** A characteristic set + its per-predicate multiplicity + observed object types/datatypes ≈ a SHACL **node shape** with `sh:property` entries (`sh:path`, `sh:class`/`sh:datatype`, `sh:minCount`/`sh:maxCount` from multiplicity, `sh:in` for small value sets). `sparq`'s CS table is most of the way to QSE-style shapes. Support/confidence are direct from counts.

### 1.5 Query logs (LSQ) and usage-driven schema

- **LSQ — Linked SPARQL Queries** (Saleem et al.). A corpus/representation of real SPARQL query logs (DBpedia, SWDF, etc.). Usage-driven signal: which classes/properties/joins users *actually* query is a strong prior for what to surface to an LLM and for the planner's workload-aware tuning. <http://lsq.aksw.org/>.
  - **Relevance:** Optional `sparq` enhancement — if a query log is available, rank schema-card contents by query frequency (most-asked-about classes/properties first), and mine frequent join patterns to refine CS-based estimates.

### 1.6 Other graph-summary lines (context)

- **ExpLOD** — summaries of interlinking + class/predicate usage via bisimulation labels. **SchemaPainter / structural graph summaries** (Blume, Scherp) — (k)-bisimulation quotient summaries, parallel algorithms (arXiv:2111.12493). **QLever's Wikidata autocompletion / schema introspection** — context-sensitive SPARQL autocompletion driven by precomputed predicate/co-occurrence stats (Bast, Kalmbach, Klumpp, Bäurle): a live example of exactly the "stats-for-grounding" use case at Wikidata scale. (URLs to verify below.)

---

## 2. The core insight: permutation indexes ⇒ near-free statistics

Given the 6 sorted permutations, here is the **stat → index → cost** table that the whole API is built on. "Run" = a maximal block of identical leading key(s) in the sorted order; block boundaries are found by scanning or by binary search over a block-start array.

| Statistic | Index | How | Cost |
|---|---|---|---|
| **Predicate histogram** `count(p)=\|{(s,o):(s,p,o)}\|` | PSO or POS | each P-block length | one scan (or precomputed block index) |
| **Distinct subjects per predicate** `\|{s:(s,p,·)}\|` | PSO | count S-runs within each P-block | one scan |
| **Distinct objects per predicate** `\|{o:(·,p,o)}\|` | POS | count O-runs within each P-block | one scan |
| **Class extent** `\|{s: (s,rdf:type,C)}\|` | POS (p=type) | O-run lengths within the `rdf:type` P-block | sub-scan |
| **Subject out-degree dist.** | SPO | S-run lengths | one scan |
| **Object in-degree dist.** | OSP | O-run lengths | one scan |
| **Characteristic set of `s`** | SPO | predicates in the S-run (already sorted, dedup adjacent) | part of the SPO scan |
| **Predicate co-occurrence** `(p,q)` on same subject | SPO | within each S-run, all predicate pairs (or feed CS table) | SPO scan |
| **Observed range histogram** of `p` (object's classes/datatypes) | POS→type join, or OPS | for each object in p's block, look up its type(s) | scan + type lookup |
| **Observed domain histogram** of `p` (subject's classes) | PSO→type join | for each subject in p's block, look up its type(s) | scan + type lookup |
| **Functional check** (is `p` ≤1 per subject?) | PSO | are all S-runs length 1 within p's block? | sub-scan |
| **Inverse-functional** (is `p` ≤1 per object?) | POS | are all O-runs length 1? | sub-scan |
| **Sample values of `p`** | PSO/POS | first/last/stride-k objects in p's block (sorted ⇒ min/max free) | O(k) |
| **Class–property matrix** `(C,p)` | join type-extent with PSO | for class C's subjects, which p-blocks they appear in | scan |

The headline: **no hash aggregation, no GROUP BY, no extra index.** Everything is a sorted scan over data the engine already stores. Min/max/quantile samples are *free* because the relevant permutation is sorted on the value. This is why `sparq` can offer an introspection API that competitors implement as slow SPARQL workloads.

---

## 3. Recommended API surface

Crate `sparq-introspect`, gated by cargo feature `introspect`. It depends on `sparq-core` (dict + permutation index read API) but the default engine never links it. All methods take a borrowed read-only handle to the store; none mutate engine state.

### 3.1 Data structures (the persisted/derived artifacts)

```rust
/// One emergent entity type = a distinct characteristic set. The keystone struct.
pub struct CharSet {
    pub id: CsId,                       // dense id into the CS table
    pub predicates: Box<[PredId]>,      // sorted, deduped predicate ids
    pub count: u64,                     // #subjects whose CS == exactly this set
    pub per_pred: Box<[PredStat]>,      // aligned with `predicates`
    pub typed_as: Box<[(ClassId, u64)]>,// declared rdf:type(s) of these subjects + support
    pub label: Option<StrId>,           // human name if a dominant class gives one
}
pub struct PredStat {
    pub multiplicity_sum: u64,          // Σ objects over subjects in this CS (avg = sum/count)
    pub max_mult: u32,                  // observed sh:maxCount
    pub min_present: bool,              // present on every subject ⇒ sh:minCount 1
    pub range: RangeProfile,            // observed object profile
}
pub enum RangeProfile {
    Iri { top_classes: Box<[(ClassId,u64)]> },     // object's rdf:type histogram (top-k)
    Literal { datatype: DtId, samples: Box<[StrId]>, min: Option<StrId>, max: Option<StrId> },
    Mixed { iri_frac: f32, /* ... */ },
}

/// VoID-style flat stats (cheap, always available).
pub struct ClassProfile  { pub class: ClassId, pub instances: u64,
                           pub props: Box<[(PredId,u64)]> }      // class-property matrix row
pub struct PredProfile   { pub pred: PredId, pub triples: u64,
                           pub distinct_s: u64, pub distinct_o: u64,
                           pub functional: bool, pub inverse_functional: bool,
                           pub domain_hist: Box<[(ClassId,u64)]>,// observed domain
                           pub range_hist:  Box<[(ClassId,u64)]>,// observed range
                           pub literal_dts: Box<[(DtId,u64)]>,
                           pub samples: Box<[StrId]> }

/// Detected vocabulary (from bundled LOV/prefix.cc snapshot + optional deref).
pub struct VocabUse { pub namespace: StrId, pub prefix: Option<String>,
                      pub title: Option<String>, pub description: Option<String>,
                      pub term_count: u64, pub source: VocabSource }
```

### 3.2 Methods

```rust
impl Introspector<'_> {
    // ---- flat profiling (one permutation scan each; cacheable) ----
    fn classes(&self, top_k: usize) -> Vec<ClassProfile>;        // by instance count (POS,type)
    fn properties(&self, top_k: usize) -> Vec<PredProfile>;      // by triple count (PSO)
    fn properties_of(&self, c: ClassId) -> Vec<(PredId,u64,RangeProfile)>; // class-property
    fn sample_values(&self, p: PredId, k: usize) -> Vec<Term>;   // incl. min/max for free
    fn domain_of(&self, p: PredId) -> Vec<(ClassId,u64)>;        // observed (PSO→type)
    fn range_of(&self, p: PredId)  -> Vec<(ClassId,u64)>;        // observed (POS→type / OPS)

    // ---- characteristic sets (keystone; dual-use with planner) ----
    fn characteristic_sets(&self, opts: CsOpts) -> CsTable;      // SPO scan, bounded top-K
    fn schema_cards(&self, budget: CardBudget) -> Vec<SchemaCard>; // §7 novelty

    // ---- vocabulary detection ----
    fn vocabularies(&self) -> Vec<VocabUse>;                     // namespace → LOV/prefix.cc
    fn prefix_glossary(&self) -> Vec<(String,String,String)>;    // prefix, ns, human gloss

    // ---- shapes ----
    fn shacl_shapes(&self, opts: ShapeOpts) -> ShapesDoc;        // node+property shapes w/ support/conf
    fn validate_query(&self, q: &Algebra) -> Vec<SchemaWarning>; // does query fit the schema?

    // ---- the headline export: token-bounded summary ----
    fn schema_summary(&self, budget: SummaryBudget) -> SchemaSummary;   // §4
    fn schema_summary_for(&self, seeds: &[Term], budget: SummaryBudget) -> SchemaSummary; // retrieval mode

    // ---- standard exports ----
    fn to_void(&self) -> Graph;          // VoID + VoID-ext document
    fn to_shacl(&self) -> Graph;         // inferred shapes
    fn to_markdown(&self, budget: SummaryBudget) -> String;      // the LLM data dictionary
}
```

Design notes:
- **Everything is a scan, nothing is a join engine call.** `properties_of`/`domain_of`/`range_of` do the type-lookup via the dict + POS(`rdf:type`) extent membership, not via the SPARQL evaluator, so there is no planner re-entrancy and the cost is predictable.
- **`schema_summary_for(seeds, …)`** is the RAG/retrieval entry point: given seed terms (entities the user's question mentions, resolved by label search), return only the *local* schema — the classes those entities belong to, their characteristic sets, and the properties/ranges around them. This is how you handle Wikidata's ~10k properties without blowing the token budget.
- **`validate_query`** consumes the parsed SPARQL algebra and flags: predicate never co-occurs with the constrained class; object datatype mismatch vs `range_of`; cardinality misuse; unknown namespace. This is the "use shapes to validate generated queries" loop.

---

## 4. Token-bounded schema-summary format (the "data dictionary")

The summary is a **deck of schema cards** rendered to compact Markdown, selected greedily under a token budget. Selection priority (highest first): (1) classes by instance count, (2) within a class, characteristic-set properties by `min_present` then frequency, (3) prefix glossary for namespaces actually used, (4) one representative sample value per property. ABSTAT-style **minimalization** drops a `(C,p,D)` line when a more specific subtype already covers it; rare CSs are bucketed into a "… and N rarer property patterns" tail line so the LLM knows coverage is incomplete.

Each line carries the *counts*, because counts are what let the LLM pick selective patterns and what the planner reuses.

````markdown
# Schema summary — DBpedia subset (4.1M triples, 312 classes, 1,104 props)

## Prefixes
dbo:  http://dbpedia.org/ontology/      (DBpedia ontology)
dbr:  http://dbpedia.org/resource/      (DBpedia resources / instances)
foaf: http://xmlns.com/foaf/0.1/        (FOAF — people & agents)
geo:  http://www.w3.org/2003/01/geo/…   (WGS84 lat/long)

## Top classes
### dbo:Person  — 412,907 instances
  properties (count · cardinality · range):
  - rdfs:label        412,907 · 1..1  · xsd:string (lang-tagged)   e.g. "Ada Lovelace"@en
  - dbo:birthDate     381,442 · 0..1  · xsd:date                   e.g. 1815-12-10
  - dbo:birthPlace    377,901 · 0..* · →dbo:Place (92%), dbo:City  e.g. dbr:London
  - dbo:occupation    210,338 · 0..* · →dbo:Occupation             e.g. dbr:Mathematician
  - foaf:name         402,110 · 1..* · xsd:string                  e.g. "Ada Lovelace"
  characteristic sets (emergent subtypes):
  - {label,birthDate,birthPlace,deathDate,deathPlace}  ×138,402  → "deceased person"
  - {label,birthDate,birthPlace,occupation}            × 96,771  → "living notable"
  … and 240 rarer property patterns.

### dbo:Place — 388,210 instances
  - rdfs:label   388,210 · 1..1 · xsd:string
  - geo:lat      301,884 · 0..1 · xsd:double      e.g. 51.5074
  - geo:long     301,884 · 0..1 · xsd:double
  - dbo:country  274,330 · 0..1 · →dbo:Country
  …

## Cross-class join hints (most frequent (C,p,D) patterns)
  dbo:Person —dbo:birthPlace→ dbo:Place   (377,901)
  dbo:Person —dbo:spouse→ dbo:Person      ( 41,228, ~symmetric)
  dbo:Film   —dbo:director→ dbo:Person    ( 88,540)
````

**Budgeting knobs (`SummaryBudget`):** `max_tokens`, `max_classes`, `props_per_class`, `samples_per_prop`, `include_char_sets: bool`, `include_join_hints: bool`. A token estimator (chars/4 heuristic or a real BPE counter behind a feature) drives greedy truncation; the header always states totals so the model knows what was elided.

**Why this beats dumping the OWL ontology:** (i) it reflects *actual* usage (under-specified/mis-typed ontologies are common); (ii) ShEx/SHACL-like per-class shape blocks use fewer tokens and are syntactically closer to SPARQL than OWL axioms (consistent with current LLM-text-to-SPARQL practice, §1.5); (iii) counts give the model and planner selectivity signal for free.

---

## 5. Opt-in / zero-impact architecture, build-time vs on-demand, WASM

### 5.1 Zero-impact guarantee
- **Separate crate, cargo feature.** `sparq-introspect` is compiled only when `--features introspect`. `sparq-core`/`sparq-engine` have **no** dependency on it and no fields/branches for it; removing the crate is a one-line `Cargo.toml` deletion. No bytes added to the default index, no branches in the hot query path.
- **Read-only over existing indexes.** The introspector borrows the same mmap'd permutations the engine uses. It allocates only its *own* output (CS table, profiles), which is freed when dropped.

### 5.2 Build-time vs on-demand
Two tiers, user's choice:
1. **On-demand (default).** Every method scans the relevant permutation when called. A predicate histogram or class list is a single linear pass — fine interactively for up to ~10⁸–10⁹ triples; cache results in an in-process LRU keyed by (method, args).
2. **Sidecar (opt-in `--build-introspect`).** Post-ingest, compute the CS table, flat profiles, and class-property matrix **once** and persist to a sidecar file (`*.introspect`, mmap-friendly, dict-id encoded). Subsequent `schema_summary()` calls are O(output). The sidecar is **separate from the main index** — deleting it costs nothing, and the engine ignores it. The CS table is *also* loaded by the planner when `--features cs-planner` is on (the dual-use wiring, §7).

Block-start arrays (per-permutation P-block / S-run offsets) are an *optional* tiny acceleration the sidecar may cache so even on-demand histograms become O(#predicates) instead of O(|G|); these are pure derived data, never required.

### 5.3 WASM feasibility
- **Fully feasible and arguably the killer use case** (an in-browser engine that can describe its own loaded graph to a client-side LLM). All operations are CPU + sequential scans over the already-loaded indexes — no syscalls, no threads required.
- **Vocabulary detection ships an offline LOV/prefix.cc snapshot** (a few hundred KB of prefix→namespace→gloss, optionally compressed) bundled in the WASM artifact, so namespace identification needs no network. Live ontology dereferencing is an *optional* path guarded by a `fetch` capability (browser `fetch`/`Cache`), off by default for determinism and offline use.
- **Memory:** the CS table dominates; bound it (`CsOpts::top_k`, merge rare sets) so it stays a few MB even for Wikidata-scale inputs. Stream the SPO scan; never materialize all subjects.

---

## 6. Benchmark plan (accuracy + performance)

### 6.1 Accuracy — "does the summary capture the true schema, and does it improve NL→SPARQL?"
Datasets with *ground-truth* schemas: **LUBM** (synthetic, fully known ontology + generator stats), **DBpedia** (curated ontology + mappings), **YAGO-4** (typed, SHACL available), **Wikidata** truthy subset. QSE published pre-extracted shapes for LUBM/DBpedia/YAGO-4/Wikidata (Zenodo 7598613) — use as a **shape-extraction gold standard**.

Metrics:
1. **Schema coverage / fidelity.** Precision/recall of inferred `(C,p,D)` patterns and inferred SHACL shapes vs (a) the declared ontology `rdfs:domain/range` and (b) QSE's extracted shapes. Report at multiple support/confidence thresholds (precision–recall curves), mirroring QSE's support/confidence framing.
2. **Cardinality fidelity (CS dual-use).** q-error of CS-based star-join cardinality estimates vs true counts, on a workload of star queries — directly comparable to Neumann-Moerkotte and to `sparq`'s existing estimator. (Bridges to the planner benchmark.)
3. **NL→SPARQL uplift (the real target).** Use established text-to-SPARQL benchmarks — **QALD-9/QALD-10**, **LC-QuAD 2.0**, **Spider-style** KG sets — and measure execution-match / answer-F1 of an LLM prompted with: (baseline) nothing/raw ontology, vs (ours) the budgeted schema summary, vs (ablations) summary − char-sets, summary − sample-values, summary − counts. Also report **token cost** of each prompt (uplift-per-token). Include the **retrieval mode** (`schema_summary_for(seeds)`) vs whole-schema at fixed budget on Wikidata to show it scales past 10k properties.
4. **Query-validation usefulness.** On LLM-generated queries, measure how many *schema-invalid* queries `validate_query` catches (predicate-not-on-class, datatype mismatch) and the end-to-end accuracy gain when the validator drives a correction loop.

### 6.2 Performance — compute time + memory over large graphs
- **Build time & memory** for the CS table + flat profiles vs graph size: LUBM-{1,100,1000,8000}, DBpedia, Wikidata truthy. Plot vs |G|; expect ~linear (single SPO scan). Compare against the SPARQL-query way of computing the same stats (the Loupe/ABSTAT approach) — expected order(s)-of-magnitude speedup is the headline performance result.
- **On-demand latency** for `classes()`, `properties()`, `sample_values()`, `schema_summary()` — interactive (<1s) target on ≤10⁸ triples without the sidecar; with sidecar, O(output).
- **WASM**: same metrics in-browser on a Wikidata subset that fits the WASM heap; report bundle-size delta from the LOV snapshot.
- **Zero-impact proof:** benchmark the default engine *with* and *without* `--features introspect` compiled in — must be byte-identical index and within noise on query latency (the regression gate).

---

## 7. Novel ideas (separated)

These are contributions specific to `sparq`'s permutation-index substrate; each is cheap *because* of the 6 sorted indexes.

1. **Characteristic-Set Schema Cards (CS²).** Treat each frequent characteristic set as an *emergent class* and render it as a self-contained "schema card": property list with observed cardinalities + ranges + samples + a synthesized human label (from the dominant `rdf:type`, else from a salient property). One artifact, three consumers: the **LLM prompt**, the **SHACL exporter** (a CS ≈ a node shape), and the **cost planner** (the CS table *is* the star-join estimator). Novelty: unifying schema-summarization, shape-mining, and cardinality-estimation behind a single SPO-scan-built structure, and ranking cards by a **value-per-token** score (`count × distinctiveness / token_len`) for budget packing.

2. **Instant observed domain/range histograms via OSP/POS.** Declared `rdfs:domain`/`range` are frequently wrong or absent. Compute the *empirical* ones: for predicate `p`, walk its POS block; for each object, fetch its type(s) via the `rdf:type` extent (a membership test in a sorted run) and accumulate a class histogram — the **observed range**. Symmetrically PSO→type gives the **observed domain**. Because objects in POS are sorted, identical objects are adjacent (lookup once per distinct object). This yields per-predicate range/domain *distributions* (not single classes) at scan cost, which is strictly more informative than `rdfs:range` and feeds both the card and join-cardinality estimates.

3. **Schema summary that is literally the planner's cost model.** The numbers in the human-readable card (predicate counts, distinct-subject/object, CS multiplicities, observed join cardinalities for `(C,p,D)`) are exactly the inputs a selectivity-based optimizer wants. Persist *one* statistics sidecar and (a) render it to Markdown for the LLM and (b) load it into the planner. Guarantees the explanation the LLM sees and the estimates the planner uses are **consistent**, and amortizes the build cost across two features.

4. **Co-occurrence-driven "join hints."** From the SPO scan, record frequent predicate pairs per subject (correlated properties) and, via the type joins, frequent `(C, p, D)` cross-class edges with counts. Surface the top ones as "join hints" in the summary (§4) — they tell the LLM which classes are *connected* and how selectively, which is the information most missing from a flat class/property list. Same table refines the planner's correlated-predicate estimates beyond independence.

5. **Retrieval-mode schema summaries (`schema_summary_for`).** For huge schemas, don't summarize everything — resolve the question's entities to classes (label search), then emit only their CS² cards + 1-hop neighboring classes via join hints. Turns an intractable 10k-property prompt into a focused few-hundred-token context. This is RAG, but the "documents" are characteristic-set cards built from sorted scans rather than embedded text chunks (cheaper, exact, no vector DB).

6. **Support/confidence on every line, computed exactly.** Because counts are free, every inferred shape constraint carries QSE-style support and confidence *exactly* (not sampled). The LLM can be instructed to trust high-confidence constraints as hard and treat low-confidence ones as optional, and the validator can threshold on them — a calibrated, data-grounded notion of "what's reliable in this graph."

7. **Min/max/quantile samples for free.** Since each permutation is sorted on its leading keys, the first and last object in a predicate's POS block are its min and max values with **zero extra work**, and stride sampling gives quantiles. Numeric ranges and date spans (e.g. "`dbo:birthDate` ranges 1700-01-01 … 2020-12-31") drop straight into the card and help the LLM write range filters correctly — a detail other profilers compute with sort/aggregate passes.

---

## 8. References (primary sources)

- VoID spec: <https://www.w3.org/TR/void/> · design paper: <https://www.researchgate.net/publication/228974311> · VoID-ext: <http://akswnc7.informatik.uni-leipzig.de/dstreitmatter/archivo/ldf.fi/void-ext/>
- Loupe: <https://ceur-ws.org/Vol-1486/paper_113.pdf> · model: <https://www.w3.org/2016/11/sdsvoc/SDSVoc16_paper_7>
- ABSTAT: <https://link.springer.com/chapter/10.1007/978-3-319-25639-9_25> · <http://km.aifb.kit.edu/ws/sumpre2016/paper_3.pdf>
- Characteristic Sets (Neumann & Moerkotte, ICDE 2011): <https://www.csd.uoc.gr/~hy561/papers/storageaccess/optimization/Characteristic%20Sets.pdf> · DBLP: <https://dblp.org/rec/conf/icde/NeumannM11.html>
- CS sampling (ESWC 2020): <https://preprints.2020.eswc-conferences.org/121230144.pdf> · Conjunctive-query cardinality (arXiv:1801.09619): <https://arxiv.org/pdf/1801.09619> · LMKG (arXiv:2102.10588): <https://arxiv.org/pdf/2102.10588>
- LOV (SWJ 2017): <https://semantic-web-journal.net/system/files/swj974.pdf> · API: <https://lov.linkeddata.es/dataset/lov/api> · prefix.cc: <https://prefix.cc/>
- QSE (VLDB 2023): <https://www.vldb.org/pvldb/vol16/p1023-rabbani.pdf> · repo: <https://github.com/dkw-aau/qse> · shapes dataset: <https://zenodo.org/records/7598613> · SHACTOR (SIGMOD 2023 demo): <https://people.cs.aau.dk/~matteo/pdf/SIGMOD23-SHACTOR-demo.pdf>
- SHACLearner / "Learning SHACL Shapes from Knowledge Graphs" (SWJ): <https://www.semantic-web-journal.net/system/files/swj2906.pdf>
- LSQ (Linked SPARQL Queries): <http://lsq.aksw.org/>
- ExpLOD (ESWC 2010): <https://link.springer.com/chapter/10.1007/978-3-642-13489-0_19> · usage summaries: <http://www.cs.toronto.edu/~shahan/swc2011/UnderstandingBillionsofTriples-swc2011-extended.pdf>
- QLever context-sensitive SPARQL autocompletion (arXiv:2104.14595): <https://arxiv.org/pdf/2104.14595> · QLever: <https://github.com/ad-freiburg/qlever>
- Graph summarization survey (Čebirić, Goasdoué et al., VLDB J 2019): <https://link.springer.com/article/10.1007/s00778-018-0528-3> · RDFQuotient / first-sight: <https://link.springer.com/article/10.1007/s00778-020-00611-y>
- Structural graph summaries / k-bisimulation (Blume, Scherp; arXiv:2111.12493): <https://arxiv.org/pdf/2111.12493>
- LLM text-to-SPARQL over federated KGs (arXiv:2410.06062 — ShEx-per-class, retrieval-augmented prompting): <https://www.alphaxiv.org/overview/2410.06062v4>
- Counterfactual validation for text-to-SPARQL (arXiv:2508.01815): <https://arxiv.org/html/2508.01815>
