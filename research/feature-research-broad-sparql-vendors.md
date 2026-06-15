# Broad SPARQL / Semantic-Web Feature Landscape + Commercial Vendor Gap Analysis

<!-- [OPUS-4.8] deep-research sq-3183 — broad landscape + vendor competitive analysis -->

Research record for epic **sq-3183**. This is the *broad* slice of the feature-research
fan-out: the SPARQL/RDF spec frontier, semantic-web tech literature, and a **commercial
vendor competitive gap analysis** — to find features sparq should add. Federation,
ODRL/policy, and vector/embedding retrieval are owned by **sibling agents**; this doc
touches them only where they intersect (e.g. RML mapping is *ingestion*, not federation;
virtual-graph/OBDA is flagged as the sibling's, not re-proposed here).

The candidate table at the end uses the **same format as the sibling agents**: each feature
→ FIT classification → impact (1–5) → effort (S/M/L) → rationale + source.

---

## 0. What sparq already ships (grounding — do NOT re-propose)

Established from `crates/`, the 15 `skills/*/SKILL.md` (plus the root `skills/SKILL.md`),
and `research/ARCHITECTURE.md`:

- **SPARQL 1.1 query + update**, plus **SPARQL 1.2 / RDF 1.2** triple-terms and
  `langdir`/base-direction (`crates/sparq-engine/src/json.rs` emits SPARQL-1.2 JSON
  `its:dir`; `crates/sparq-engine/src/dataset.rs` carries `TripleTerm`s; SERVICE handles
  1.2 triple terms). RDF-star quoted triples.
- **Property paths**, aggregates/GROUP BY/HAVING, subqueries, BIND/VALUES,
  OPTIONAL/MINUS/UNION, DISTINCT, named-graph dataset views, EXISTS.
- **Custom extension functions** (SPARQL 17.6) via `FunctionRegistry` /
  `query_with_functions`; **prepared/parameterised queries** (`PreparedQuery`); query
  **budgets/timeouts**; **EXPLAIN** + `explain_analyze`.
- **Result formats:** SPARQL-1.1/1.2 **JSON, XML, CSV, TSV** (server content-negotiated).
  CONSTRUCT/DESCRIBE → N-Triples; server also offers prefix-Turtle + **RDF/XML** dumps.
- **Reasoning:** RDFS, **OWL 2 RL**, Notation3/EYE rules; forward-chaining
  materialisation; **incremental** maintenance under insert/delete; `why()` proof-trees;
  OWL inconsistency check (`sparq-reason`).
- **SHACL:** Core + SHACL-SPARQL (§5.2) + custom constraint components (§6) (`sparq-shacl`).
- **GeoSPARQL 1.0/1.1 core** (`sparq-geo`): WKT/GML, `geof:` functions, R-tree index.
- **Full-text:** BM25 inverted index + `text:` magic predicates (`sparq-text`).
- **Vector/ANN:** mmap embedding store, HNSW / DiskANN, SQ/PQ, hybrid RRF (`sparq-vectors`)
  — *sibling-owned; listed for completeness.*
- **Streaming RSP-QL** windows (`sparq-rsp`); **ZK query proofs** (`sparq-zk`/`-compose`);
  **MPC** federated SPARQL (`sparq-mpc`) — *MPC/federation sibling-adjacent.*
- **Introspection / GenAI:** schema card, **VoID** export, characteristic-set join hints
  (`sparq-introspect`); NL→SPARQL loop (`sparq-nlq`); structural entity **similarity**
  (`sparq-sim`).
- **Storage/scale:** dictionary-encoded six permutations; mmap out-of-core; compressed
  on-disk format; **HDT** load (`sparq-hdt`); WAL-fsync durable server persistence;
  WASM build; Python + JS/RDF-JS bindings; Prometheus `/metrics`; WS/SSE subscriptions;
  opt-in **time-travel** `?generation` pinning.

Net: sparq is *query/storage*-rich and already at the SPARQL-1.2 frontier. The gaps are
concentrated in **(a) RDF I/O breadth (serializers, mapping, JSON-LD), (b) the
reasoning-profile + SHACL-rules surface, (c) analytics/columnar/window expressivity, and
(d) operational/protocol affordances (service description, autocomplete, transactions,
materialised views) that commercial vendors ship as table stakes.**

---

## 1. SPARQL / RDF spec frontier (W3C, current)

The W3C SPARQL 1.2 + RDF 1.2 suite is the current frontier (RDF-star/SPARQL-star is the
*superseded* predecessor — RDF 1.2 triple terms replace it). Status snapshot:

| Spec | Status (as researched) | sparq |
|---|---|---|
| [SPARQL 1.2 Query](https://www.w3.org/TR/sparql12-query/) — triple terms, `hasLANG`/`hasLANGDIR`/`LANGDIR`/`STRLANGDIR` | Near-final | **Mostly yes** (triple terms + langdir; verify the `has*`/`STRLANGDIR` builtins are all wired) |
| [SPARQL 1.2 Update](https://www.w3.org/TR/sparql12-update/) | WD (Aug 2025) | SPARQL 1.1 Update ✔; 1.2 deltas TBD |
| [SPARQL 1.2 Protocol](https://www.w3.org/TR/sparql12-protocol/) | WD | 1.1 Protocol ✔ |
| [SPARQL 1.2 Service Description](https://www.w3.org/TR/sparql12-service-description) | WD (Mar 2026) | **No SD endpoint** (gap) |
| [Results CSV/TSV](https://www.w3.org/TR/sparql12-results-csv-tsv/), [JSON](https://www.w3.org/TR/sparql12-results-json/), [XML](https://w3c.github.io/sparql-results-xml/spec/) | 1.2 versions | ✔ (CSV/TSV/JSON/XML) |
| [SPARQL 1.2 Entailment Regimes](https://www.w3.org/TR/sparql12-entailment/) | WD | Materialisation-based; no *query-time* entailment-regime declaration |
| [RDF Dataset Canonicalization RDFC-1.0](https://www.w3.org/TR/2024/PR-rdf-canon-20240326/) | **REC (May 2024)** | Implemented **inside `sparq-zk`** only — not a public surface (gap) |

**Frontier items NOT yet in any sparq surface, ordered by leverage:**

1. **SPARQL 1.2 Service Description** — a machine-readable description of the endpoint's
   features (supported functions, entailment, result formats, named graphs). Every serious
   endpoint (QLever, GraphDB, Virtuoso) serves one; agents/clients use it for capability
   discovery. *Small, high-visibility.*
   ([SD spec](https://www.w3.org/TR/sparql12-service-description))
2. **RDFC-1.0 as a first-class public API + CLI** (`canonicalize`, `--canon`, dataset
   hashing/isomorphism check). The algorithm already exists in `sparq-zk`; exposing it
   pays off for **dataset diffing, dedup, short content IDs, and signing** — and it is a
   W3C REC. ([RDFC-1.0 REC](https://www.w3.org/TR/2024/PR-rdf-canon-20240326/))
3. **Window functions** (`WINDOW_AVG`/`WINDOW_SUM`/ranking/`ROW_NUMBER`, "limit per
   resource", moving averages) — the single most-requested SPARQL expressivity gap
   (w3c/sparql-dev #47); shipped by AnzoGraph/Stardog as extensions.
   ([sparql-dev #47](https://github.com/w3c/sparql-dev/issues/47),
   [AnzoGraph window aggregates](https://docs.cambridgesemantics.com/anzo/v5.4/userdoc/functions-window-aggregates.htm))
4. **Custom *aggregates*** — SPARQL's aggregate set is closed by spec; Stardog and Jena/ARQ
   both add a pluggable custom-aggregate registry. sparq has custom *scalar* functions but
   not custom aggregates. ([Jena custom aggregates](https://jena.apache.org/documentation/query/custom_aggregates.html),
   [Stardog aggregates](https://docs.stardog.com/developing/extending-stardog/aggregates))
5. **Entailment-regime query flag** — declare RDFS/OWL-RL entailment *per query* (vs only
   pre-materialise), so a client can ask for entailed answers without a separate closure
   build. ([SPARQL 1.2 Entailment](https://www.w3.org/TR/sparql12-entailment/))

---

## 2. Semantic-web tech literature (beyond the spec)

### 2.1 RDF I/O breadth — the biggest plain-RDF gap

sparq **parses** Turtle/N-Triples/N-Quads/TriG/HDT but ships **no general RDF text
serializer** in core (only a CONSTRUCT N-Triples emitter + the server's RDF/XML dump). The
data-formats skill explicitly says "sparq-core ships no RDF text serializer." Every
competitor (Jena, RDF4J, Oxigraph, Virtuoso) round-trips **Turtle / N-Triples / N-Quads /
TriG / RDF/XML / JSON-LD**. Concretely missing:

- **Turtle / TriG / N-Quads serializers** (prefix-compacted Turtle especially) as a public
  `sparq-core` API + CLI `serialize`/`convert` subcommand.
  ([Oxigraph formats](https://github.com/oxigraph/oxigraph),
  [Jena formats](https://en.wikipedia.org/wiki/Apache_Jena))
- **JSON-LD 1.1** parse + serialize — the dominant web-facing RDF format; Jena and RDF4J
  both have hierarchical JSON-LD writers. Currently absent entirely.
  ([RDF4J JSON-LD](https://rdf4j.org/))

### 2.2 Mapping / lifting — RML / R2RML (ingestion, not federation)

The **R2RML / RML / YARRRML** family lifts relational/CSV/JSON/XML into RDF. This is the
*ETL* on-ramp; Ontop/GraphDB/Stardog all support it (Ontop primarily as *virtual* graphs —
that virtualisation is the **sibling's federation topic**; a **materialising RML loader**
is in-scope here). A `sparq-rml` crate (CSV/JSON/XML → triples via an RML mapping, emitted
straight into a `Graph`) would be a clean opt-in on-ramp without touching the engine.
([R2RML REC](https://www.w3.org/TR/r2rml/),
[RML/Ontop overview](https://github.com/ontop/ontop/wiki/ObdalibRdb2rdf))

A lighter-weight cousin worth noting: **SPARQL-Generate / SPARQL-Anything** turn
heterogeneous documents (JSON/CSV/XML/HTML) into RDF *via SPARQL itself*
(`GENERATE`/iterator functions / Facade-X). sparq's extension-function registry is a
natural host for SPARQL-Generate-style iterator/binding functions.
([SPARQL-Generate](https://link.springer.com/chapter/10.1007/978-3-319-58694-6_16),
[SPARQL Anything](https://sparql-anything.readthedocs.io/stable/))

### 2.3 Reasoning frontier

sparq has RDFS + OWL 2 RL + N3 (materialisation) with incremental maintenance and proof
trees — genuinely strong. Gaps vs the literature/vendors:

- **OWL 2 EL and QL profiles** — sparq does RL only. EL (large biomedical ontologies,
  SNOMED/GO) and QL (query rewriting / OBDA) are distinct, valuable regimes. EL is a
  meaningful classification add; QL overlaps the sibling's OBDA/virtual-graph topic.
  ([RDFox profiles](https://docs.oxfordsemantic.tech/reasoning.html))
- **SWRL rules** — RDFox/Stardog/Protégé ecosystems use SWRL; sparq has N3 but not SWRL
  ingestion. Lower priority (N3/Datalog covers the expressivity).
- **SHACL-AF rules (`sh:rule` / `sh:SPARQLRule` / `sh:TripleRule`)** — SHACL Advanced
  Features turns SHACL into a *rule/inference* engine (CONSTRUCT-style derivations bound to
  shape targets). sparq-shacl validates but does not *execute rules*. This is a natural,
  high-fit extension of the existing SHACL crate + reasoner.
  ([SHACL-AF](https://www.w3.org/TR/shacl-af/),
  [SHACL 1.2 SPARQL](https://www.w3.org/TR/shacl12-sparql/))
- **SHACL node expressions / SHACL-AF functions** — declarative computed values; pairs with
  the rules work.

### 2.4 Provenance, signing, canonicalization

- **RDFC-1.0 public surface** (see §1.2) unlocks **RDF dataset signing / verifiable
  credentials** outside the ZK path, and dataset content-hashing for caching/dedup.
- **PROV-O provenance over named graphs** — capture derivation/who/when for materialised or
  loaded graphs. sparq has named graphs + time-travel; a thin PROV-O recording layer would
  make derivations queryable. Lower priority (vocabulary discipline, not engine work).

### 2.5 Skolemization

Blank-node **skolemization** (`well-known/genid` IRIs) on dump/exchange — needed for stable
external references and round-trips through systems that dislike bnodes. Cheap; pairs with
the serializer work.

---

## 3. Commercial vendor competitive analysis

Deep read of each vendor's *current* feature set, focusing on **what they ship that sparq
lacks**. (Pure-scale/perf claims are excluded — sparq's architecture already targets those;
this is about *feature breadth*.)

### Stardog
First-class **SPARQL + SPARQL\* + GraphQL**; **virtual graphs** (RDBMS/NoSQL/LLM mapped in
situ — *sibling*); **explainable reasoning** (full proof/justification UI); **BI/SQL
Server** (query the KG from BI tools over SQL/JDBC); built-in **graph ML** (similarity +
classification/regression); **connectors** to SQL/NoSQL; **custom aggregates**; agentic-AI
tooling. ([Stardog features](https://www.stardog.com/features/),
[Stardog ML](https://www.stardog.com/platform/features/graph-machine-learning/))
→ **sparq lacks:** GraphQL surface, SQL/BI (JDBC) access, custom aggregates, ML predictive
analytics. (Reasoning *explanation* sparq partially has via `why()`.)

### Ontotext GraphDB
Real-time inference at scale; **connectors** (Lucene/Solr/Elasticsearch/OpenSearch/**Kafka**
— keep external indexes + downstream systems in sync via SPARQL); **vector search**
(via ES/OpenSearch connector); **similarity** plugin; **GraphQL + JDBC** access; **"Talk to
Your Graph"** (NL/LLM, Bedrock/Vertex); RDF4J-API Python client; Ontop-backed virtual
SPARQL (*sibling*). ([GraphDB](https://www.ontotext.com/products/graphdb/),
[Kafka connector](https://graphdb.ontotext.com/documentation/11.2/kafka-graphdb-connector.html))
→ **sparq lacks:** **change-data-capture / sync connectors** (Kafka/Elastic) that mirror
graph mutations to external systems — a genuinely distinctive operational feature; GraphQL;
JDBC. (Similarity sparq has via `sparq-sim`; NL via `sparq-nlq`.)

### Virtuoso
**Hybrid SQL+RDF** in one RDBMS engine (query RDF *as* SQL and vice-versa); **faceted
search/browsing** service; **Sponger** (RDFa/microformat/GRDDL lifting → RDF); full-text;
geospatial; SPARQL endpoint + linked-data publishing.
([Virtuoso](https://virtuoso.openlinksw.com/),
[Virtuoso RDF/SPARQL](https://docs.openlinksw.com/virtuoso/ch-rdfandsparql/))
→ **sparq lacks:** a **faceted-search API** (drill-down counts over a result set — high
value for data-exploration UIs), SQL-over-RDF, content lifting (overlaps RML/Sponger).

### AWS Neptune / Neptune Analytics
Multi-model (**Gremlin + openCypher + SPARQL**); **25+ in-database graph algorithms**
(centrality, community detection, pathfinding) as openCypher procedures; **Neptune ML**
(GNN/DGL embeddings + predictions); **serverless** autoscaling; **MVCC snapshot-isolation
transactions**. ([Neptune features](https://aws.amazon.com/neptune/features/),
[Neptune Analytics algorithms](https://docs.aws.amazon.com/neptune-analytics/latest/userguide/algorithms.html),
[Neptune transactions](https://docs.aws.amazon.com/neptune/latest/userguide/transactions-neptune.html))
→ **sparq lacks:** **graph-analytics algorithms** (PageRank, betweenness/degree centrality,
connected components, community detection, shortest paths beyond property paths) exposed in
SPARQL/CLI; **MVCC transactions** (sparq has WAL durability + time-travel snapshots but no
multi-statement transaction isolation). openCypher/Gremlin is out of scope (RDF engine).

### RDFox
In-memory **parallel Datalog (OWL 2 RL) + SWRL** reasoning with **incremental
materialisation** and efficient `owl:sameAs`; SPARQL over the materialisation; multiple
reasoning profiles. ([RDFox reasoning](https://docs.oxfordsemantic.tech/reasoning.html),
[RDFox paper](https://link.springer.com/chapter/10.1007/978-3-319-25010-6_1))
→ **sparq has the closest match already** (OWL-RL + N3 + incremental). Gaps: **SWRL
ingestion**, **`owl:sameAs` equality-class compaction** (efficient sameAs handling is an
RDFox signature; worth a dedicated optimisation), broader profiles (EL).

### AllegroGraph
**FedShard** (federation/sharding — *sibling*); **Gruff** graph explorer; **GeoTemporal +
n-dimensional geospatial** reasoning; **vector store + LLM** (neuro-symbolic); rule-based
reasoning; **100% ACID transactions** (commit/rollback/checkpoint).
([AllegroGraph](https://allegrograph.com/products/allegrograph/),
[AG LLM/vector](https://franz.com/agraph/support/documentation/llm.html))
→ **sparq lacks:** **temporal reasoning / a temporal-RDF model** (interval/valid-time
queries beyond raw `xsd:dateTime` FILTERs); ACID multi-statement transactions; a graph
explorer UI (out of scope for a library).

### MarkLogic
**Optic API** (query triples *as rows/columns*, join triples↔documents); **bitemporal**
data management (valid-time + system-time, full audit trail); SPARQL 1.1; document+semantic
multi-model. ([MarkLogic Optic](https://docs.progress.com/bundle/marklogic-server-develop-server-side-apps-12/page/topics/OpticAPI.html),
[MarkLogic bitemporal](https://www.progress.com/blogs/marklogic-8-provides-javascript-bitemporal-sparql-1-2))
→ **sparq lacks:** **bitemporal** modelling (sparq's time-travel is *system-time* only — no
valid-time); a relational/row "Optic"-style projection of triples (overlaps SQL-over-RDF +
Arrow export).

### QLever (the perf bar — open source)
Full SPARQL 1.1 (1.2 in progress); **context-sensitive autocompletion** for SPARQL on huge
KGs; **integrated SPARQL+Text** full-text; basic GeoSPARQL + visualisation; **live query
analysis**. ([QLever](https://github.com/ad-freiburg/qlever),
[QLever Wikipedia](https://en.wikipedia.org/wiki/QLever))
→ **sparq lacks:** **query autocompletion** (predicate/subject/object suggestions at the
cursor over the loaded vocab — high value for any query UI; sparq has the sorted
permutation indexes + introspect to make this cheap). sparq's full-text/Geo already match
QLever's.

### Oxigraph / Jena / RDF4J (open-source baselines)
Oxigraph: full SPARQL 1.1 Query/Update/**Federated Query**, in-memory + **RocksDB**
persistence, **all RDF serializations** (Turtle/NT/RDF-XML/NQ/TriG).
Jena/RDF4J: full RDF I/O incl. **JSON-LD**, rule/OWL reasoners, **GeoSPARQL**, Lucene/Solr
text + spatial. ([Oxigraph](https://github.com/oxigraph/oxigraph),
[Jena](https://en.wikipedia.org/wiki/Apache_Jena), [RDF4J](https://rdf4j.org/))
→ **sparq lacks (vs even these baselines):** the **full serializer matrix** (Turtle/TriG/NQ
writers + JSON-LD) — this is the most embarrassing gap because the *parsers* exist and the
peers all round-trip.

---

## 4. Vendor competitive gap matrix

Feature × vendor-that-ships-it × **does sparq have it?**
(`✔` ships / `~` partial / blank = no. *S = sibling-owned topic.*)

| Feature | Stardog | GraphDB | Virtuoso | Neptune | RDFox | AllegroGraph | MarkLogic | QLever | Oxigraph/Jena/RDF4J | **sparq** |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Full RDF serializer matrix (Turtle/TriG/NQ writers) | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ~ | ✔ | **~ (NT only)** |
| JSON-LD 1.1 parse+write | ✔ | ✔ | ✔ | ✔ | | ✔ | ✔ | | ✔ | **✗** |
| RML/R2RML materialising loader | ✔ | ✔ | ✔ | | | | | | ~ (Jena) | **✗** |
| SPARQL Service Description endpoint | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | | ✔ | ✔ | **✗** |
| RDFC-1.0 canonicalization (public API) | ~ | ✔ | | | | | | | ✔ (libs) | **~ (in zk only)** |
| Window functions / ranking | ✔ | | | | | | | | | **✗** |
| Custom aggregates | ✔ | ~ | | | | | | | ✔ (Jena) | **✗** |
| SHACL-AF rules (`sh:rule`) | ~ | ~ | | | ✔(datalog) | ✔ | | | ~ | **✗** |
| OWL 2 EL / QL profiles | ✔ | ✔ | ~ | | ✔ | | | | ~ | **~ (RL+N3)** |
| `owl:sameAs` equality-class handling | ✔ | ✔ | ✔ | | ✔ | ✔ | | | ~ | **✗** |
| Graph analytics algorithms (PageRank/centrality/community) | ~ | ~ | | ✔(25+) | | ✔ | | | | **✗** |
| Query autocompletion | | ~ | | | | | | ✔ | | **✗** |
| Faceted search API | | ~ | ✔ | | | | | | | **✗** |
| MVCC / ACID transactions (isolation) | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | | ~ | **~ (WAL+time-travel, no isolation)** |
| Materialised views / result cache | ✔ | ✔ | ✔ | | ✔ | | | ~ | | **✗** |
| Bitemporal / temporal reasoning | | | | | | ✔ | ✔ | | | **✗** |
| CDC sync connectors (Kafka/Elastic) | | ✔ | | | | | | | | **✗** |
| GraphQL query surface | ✔ | ✔ | | | | | | | | **✗** |
| SQL/BI (JDBC) over RDF / Arrow columnar export | ✔ | ✔ | ✔ | | | | ✔(Optic) | | | **✗** |
| Skolemization on exchange | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | | ✔ | **✗** |
| Virtual graphs / OBDA (*S*) | ✔ | ✔ | ✔ | | | | | | ~ | *S* |
| Federated SERVICE (*S*) | ✔ | ✔ | ✔ | | | ✔ | | | ✔ | ~ (opt-in) |
| Vector/ANN (*S*) | ✔ | ✔ | | ✔ | | ✔ | | | | ✔ |
| Graph ML / GNN embeddings | ✔ | | | ✔ | | ~ | | | | ~ (`sparq-sim` structural) |

**Matrix highlights (biggest, broadest gaps):**
1. **RDF serializer matrix + JSON-LD** — *everyone* round-trips RDF; sparq only writes NT.
   The parsers already exist, so this is the highest leverage-to-effort gap.
2. **Service Description + Skolemization + RDFC-1.0 public API** — three cheap "protocol
   completeness" items that every endpoint/library has; RDFC code already lives in `sparq-zk`.
3. **Window functions + custom aggregates** — the top SPARQL *expressivity* asks; no W3C
   blocker, vendor-proven.
4. **SHACL-AF rules** — turns the existing SHACL crate into an inference engine; high fit.
5. **Graph analytics algorithms** — Neptune ships 25+; sparq's sorted permutations make
   PageRank/CC/centrality cheap to add as a `sparq-graph-algos` crate or `algo:` functions.
6. **MVCC transactions + materialised views/result cache** — operational table-stakes for
   concurrent serving (sparq has the snapshot/time-travel substrate to build on).

---

## 5. CANDIDATE FEATURE TABLE

FIT ∈ {`clear-fit:<component>`, `new-component-but-fits`, `ambiguous-ask-user`}.
Impact 1–5 (5 = closes a broad table-stakes gap vs many vendors). Effort S/M/L.

| # | Feature | FIT | Impact | Effort | Rationale + source |
|---|---|---|---|:-:|---|
| 1 | **RDF serializer matrix** (Turtle/TriG/N-Quads/N-Triples writers, prefix-compacted) + CLI `serialize`/`convert` | `clear-fit:sparq-core` (+ CLI/HTTP/Py/JS) | 5 | M | Parsers exist; the data-formats skill admits "no text serializer." Every peer round-trips RDF. Closes the most embarrassing baseline gap. [Oxigraph](https://github.com/oxigraph/oxigraph), [Jena](https://en.wikipedia.org/wiki/Apache_Jena) |
| 2 | **JSON-LD 1.1** parse + serialize | `clear-fit:sparq-core` | 4 | M | Dominant web-facing RDF format; Jena/RDF4J/Oxigraph all support it; sparq has none. [RDF4J](https://rdf4j.org/) |
| 3 | **SPARQL 1.2 Service Description** endpoint (capabilities/functions/formats/graphs) | `clear-fit:sparq-server` | 4 | S | Machine-readable endpoint discovery; shipped by QLever/GraphDB/Virtuoso; agents rely on it. [SD spec](https://www.w3.org/TR/sparql12-service-description) |
| 4 | **RDFC-1.0 canonicalization** as a public API + CLI (`canonicalize`, dataset hash, isomorphism check) | `clear-fit:sparq-core` (lift from `sparq-zk`) | 4 | S | W3C **REC** (May 2024); code already exists inside `crates/sparq-zk/src/canon.rs` (consumed by `trace.rs`). Unlocks dataset diff/dedup/signing outside ZK. [RDFC-1.0](https://www.w3.org/TR/2024/PR-rdf-canon-20240326/) |
| 5 | **Window functions** (`ROW_NUMBER`/`RANK`/moving aggregates, "limit per resource") | `clear-fit:sparq-engine` | 4 | M | #1 SPARQL expressivity ask (w3c/sparql-dev #47); AnzoGraph/Stardog ship it; no spec blocker. [sparql-dev #47](https://github.com/w3c/sparql-dev/issues/47) |
| 6 | **SHACL-AF rules** (`sh:rule`/`sh:SPARQLRule`/`sh:TripleRule` execution + node expressions) | `clear-fit:sparq-shacl` (+ reasoner seam) | 4 | M | Turns the existing SHACL crate into a rule/inference engine; W3C-defined; Stardog/AllegroGraph parity. [SHACL-AF](https://www.w3.org/TR/shacl-af/) |
| 7 | **Custom aggregates** registry (pluggable, like custom scalar fns) | `clear-fit:sparq-engine` | 3 | S | SPARQL's aggregate set is closed; Stardog + Jena/ARQ both extend it; sparq already has `FunctionRegistry` to mirror. [Jena](https://jena.apache.org/documentation/query/custom_aggregates.html), [Stardog](https://docs.stardog.com/developing/extending-stardog/aggregates) |
| 8 | **Graph-analytics algorithms** (PageRank, degree/betweenness centrality, connected components, community detection, weighted shortest path) exposed via `algo:` fns or CLI | `new-component-but-fits` (`sparq-algos`) | 4 | L | Neptune ships 25+; sparq's sorted six permutations make adjacency scans cheap. Distinct from property paths. [Neptune Analytics](https://docs.aws.amazon.com/neptune-analytics/latest/userguide/algorithms.html) |
| 9 | **RML/R2RML materialising loader** (CSV/JSON/XML/RDB → triples into a `Graph`) | `new-component-but-fits` (`sparq-rml`) | 4 | L | The standard ETL on-ramp; Ontop/GraphDB/Stardog have it. *Materialising* (in-scope) vs *virtual* (sibling). [R2RML REC](https://www.w3.org/TR/r2rml/) |
| 10 | **MVCC / multi-statement transaction isolation** (snapshot isolation over the delta overlay) | `clear-fit:sparq-server`+`sparq-core` | 4 | L | Neptune/Blazegraph/GraphDB ship MVCC snapshot isolation; sparq has WAL + time-travel snapshots to build on, but no isolation contract. [Neptune txns](https://docs.aws.amazon.com/neptune/latest/userguide/transactions-neptune.html), [MVCC](https://en.wikipedia.org/wiki/Multiversion_concurrency_control) |
| 11 | **Query autocompletion** (context-sensitive S/P/O suggestions over loaded vocab) | `new-component-but-fits` (extend `sparq-introspect` + server) | 3 | M | QLever's signature usability feature; sparq's sorted permutations + introspect make it cheap. [QLever](https://en.wikipedia.org/wiki/QLever) |
| 12 | **Materialised views / result cache** (named CONSTRUCT/SELECT cached, invalidated on update) | `new-component-but-fits` (`sparq-engine`+server) | 3 | M | Stardog/GraphDB/RDFox cache; pairs with prepared queries + the delta overlay for invalidation. [Stardog](https://www.stardog.com/features/) |
| 13 | **`owl:sameAs` equality-class compaction** in the reasoner | `clear-fit:sparq-reason` | 3 | M | RDFox's signature reasoning optimisation; avoids quadratic sameAs blow-up. [RDFox](https://docs.oxfordsemantic.tech/reasoning.html) |
| 14 | **OWL 2 EL profile** (in addition to RL) | `clear-fit:sparq-reason` | 3 | L | EL covers SNOMED/GO-scale ontologies RL can't classify; RDFox/GraphDB offer multiple profiles. [RDFox](https://docs.oxfordsemantic.tech/reasoning.html) |
| 15 | **Skolemization** (`/.well-known/genid/` bnode→IRI) on serialize/exchange | `clear-fit:sparq-core` | 3 | S | Stable external references; every peer does it; pairs with the serializer work. [RDF concepts](https://www.w3.org/TR/rdf12-concepts/) |
| 16 | **Apache Arrow / columnar result export** (zero-copy to DuckDB/pandas/BI) | `new-component-but-fits` (serialization) | 3 | M | sparq executes on columnar `DataChunk`s of u64 ids already; emitting Arrow is a natural projection; unlocks the analytics ecosystem (DuckDB/pandas). [Arrow result transfer](https://arrow.apache.org/blog/2025/01/10/arrow-result-transfer/) |
| 17 | **CDC sync connectors** (mirror graph mutations to Kafka / external index via SPARQL) | `new-component-but-fits` (`sparq-connect`) | 3 | L | GraphDB's distinctive operational feature (Kafka/Elastic connectors); fits the server's update/subscription seam. [GraphDB Kafka](https://graphdb.ontotext.com/documentation/11.2/kafka-graphdb-connector.html) |
| 18 | **Faceted-search API** (drill-down value counts over a result set) | `new-component-but-fits` (server endpoint) | 2 | M | Virtuoso's facet service; powers data-exploration UIs; computable from the indexes. [Virtuoso](https://virtuoso.openlinksw.com/) |
| 19 | **SPARQL-Generate iterator/binding functions** (lift JSON/CSV/XML via SPARQL) | `clear-fit:sparq-engine` (FunctionRegistry) | 2 | M | Lighter cousin of RML; hosts cleanly in the existing extension-fn registry. [SPARQL-Generate](https://link.springer.com/chapter/10.1007/978-3-319-58694-6_16) |
| 20 | **PROV-O provenance recording** over named graphs (derivation/who/when) | `ambiguous-ask-user` | 2 | M | Pairs with time-travel; mostly vocabulary discipline. Ask whether this belongs in core or a tool. [PROV-O](https://www.w3.org/TR/prov-o/) |
| 21 | **Bitemporal model** (valid-time alongside system-time time-travel) | `ambiguous-ask-user` | 2 | L | MarkLogic/AllegroGraph differentiator; sparq has system-time only. Scope/demand unclear — ask user. [MarkLogic](https://www.progress.com/blogs/marklogic-8-provides-javascript-bitemporal-sparql-1-2) |
| 22 | **GraphQL query surface** over the graph | `ambiguous-ask-user` | 2 | L | Stardog/GraphDB offer it; large surface, debatable fit for an RDF-first engine. Ask user. [Stardog](https://www.stardog.com/features/) |

---

## 6. Top recommendations

Ordered by **impact ÷ effort** against broad vendor table-stakes (federation/vector/ODRL
deliberately excluded — sibling-owned):

1. **RDF serializer matrix (#1) — do this first.** Highest leverage: parsers already exist,
   *every* peer round-trips RDF, and sparq's own skill admits the gap. Unblocks `convert`,
   Graph-Store writes, and #15 skolemization.
2. **Three cheap "protocol completeness" wins: Service Description (#3) + RDFC-1.0 public
   API (#4) + Skolemization (#15).** All S-effort; #4's code already exists in `sparq-zk`.
   Together they close the "looks like a toy vs a real endpoint" gap.
3. **JSON-LD (#2)** — the web-facing format sparq is conspicuously missing.
4. **Window functions (#5) + custom aggregates (#7)** — the two top SPARQL *expressivity*
   asks, both vendor-proven with no spec blocker, both landing in the existing engine.
5. **SHACL-AF rules (#6)** — converts the existing SHACL crate into a rule engine for an
   M-sized effort; strong fit with the reasoner.
6. **Graph-analytics algorithms (#8)** — the broadest *capability* gap vs Neptune (25+
   algos); larger effort but a new differentiating surface that sparq's storage suits.
7. **MVCC transactions (#10) + materialised views/result cache (#12)** — the operational
   table-stakes for concurrent serving; sparq has the snapshot/delta-overlay substrate, so
   these are evolution, not greenfield.

**Cross-cutting note:** items #1, #2, #15 and the result formats touch the public API on
multiple surfaces (core/CLI/HTTP/Py/JS) — per the AGENTS.md cross-surface-parity rule, each
must update every affected `skills/<surface>/SKILL.md` in the same change.

---

## Sources

- W3C SPARQL 1.2: [Query](https://www.w3.org/TR/sparql12-query/), [Update](https://www.w3.org/TR/sparql12-update/), [Protocol](https://www.w3.org/TR/sparql12-protocol/), [Service Description](https://www.w3.org/TR/sparql12-service-description), [Entailment](https://www.w3.org/TR/sparql12-entailment/), [Results CSV/TSV](https://www.w3.org/TR/sparql12-results-csv-tsv/), [Results JSON](https://www.w3.org/TR/sparql12-results-json/)
- [RDF Dataset Canonicalization RDFC-1.0 (REC)](https://www.w3.org/TR/2024/PR-rdf-canon-20240326/), [W3C news](https://www.w3.org/news/2024/rdf-dataset-canonicalization-is-a-w3c-recommendation/)
- [R2RML REC](https://www.w3.org/TR/r2rml/), [Ontop RDB2RDF](https://github.com/ontop/ontop/wiki/ObdalibRdb2rdf), [SPARQL-Generate](https://link.springer.com/chapter/10.1007/978-3-319-58694-6_16), [SPARQL Anything](https://sparql-anything.readthedocs.io/stable/)
- [SHACL-AF](https://www.w3.org/TR/shacl-af/), [SHACL 1.2 SPARQL](https://www.w3.org/TR/shacl12-sparql/)
- Window functions: [w3c/sparql-dev #47](https://github.com/w3c/sparql-dev/issues/47), [AnzoGraph window aggregates](https://docs.cambridgesemantics.com/anzo/v5.4/userdoc/functions-window-aggregates.htm); custom aggregates: [Jena](https://jena.apache.org/documentation/query/custom_aggregates.html), [Stardog](https://docs.stardog.com/developing/extending-stardog/aggregates)
- Vendors — [Stardog features](https://www.stardog.com/features/) / [Stardog ML](https://www.stardog.com/platform/features/graph-machine-learning/); [GraphDB](https://www.ontotext.com/products/graphdb/) / [Kafka connector](https://graphdb.ontotext.com/documentation/11.2/kafka-graphdb-connector.html) / [ES connector](https://graphdb.ontotext.com/documentation/11.3/elasticsearch-graphdb-connector.html); [Virtuoso](https://virtuoso.openlinksw.com/) / [Virtuoso RDF/SPARQL](https://docs.openlinksw.com/virtuoso/ch-rdfandsparql/); [Neptune features](https://aws.amazon.com/neptune/features/) / [Neptune Analytics algorithms](https://docs.aws.amazon.com/neptune-analytics/latest/userguide/algorithms.html) / [Neptune transactions](https://docs.aws.amazon.com/neptune/latest/userguide/transactions-neptune.html) / [Neptune ML](https://aws.amazon.com/neptune/machine-learning/); [RDFox reasoning](https://docs.oxfordsemantic.tech/reasoning.html) / [RDFox paper](https://link.springer.com/chapter/10.1007/978-3-319-25010-6_1); [AllegroGraph](https://allegrograph.com/products/allegrograph/) / [AG LLM/vector](https://franz.com/agraph/support/documentation/llm.html); [MarkLogic Optic](https://docs.progress.com/bundle/marklogic-server-develop-server-side-apps-12/page/topics/OpticAPI.html) / [MarkLogic bitemporal](https://www.progress.com/blogs/marklogic-8-provides-javascript-bitemporal-sparql-1-2); [QLever](https://github.com/ad-freiburg/qlever) / [Wikipedia](https://en.wikipedia.org/wiki/QLever); [Oxigraph](https://github.com/oxigraph/oxigraph); [Jena](https://en.wikipedia.org/wiki/Apache_Jena); [RDF4J](https://rdf4j.org/)
- [MVCC](https://en.wikipedia.org/wiki/Multiversion_concurrency_control), [Snapshot isolation](https://en.wikipedia.org/wiki/Snapshot_isolation), [Apache Arrow result transfer](https://arrow.apache.org/blog/2025/01/10/arrow-result-transfer/)
