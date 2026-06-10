# Text-embedding practices for knowledge-graph entities (T7 follow-up)

**Status:** research distillation, 2026-06-10. How production KG and vector-DB systems
turn an entity's literal properties (`rdfs:label`, descriptions, names, …) into the text
that gets embedded, and how text vectors are fused with structural/lexical signals.
Feeds the verbalization layer in `crates/sparq-vectors` (`EntityTextConfig` /
`embed_entities`) and the hybrid recipe in its README.

## 1. Entity verbalization: what the big systems concatenate

- **Wikidata Embedding Project / Vector Database** (Wikimedia Deutschland + Jina.AI):
  each item is "textualized" by concatenating **label + description + aliases +
  selected statements (property label: value, incl. qualifiers)** into one passage,
  e.g. *"Douglas Adams, English science fiction writer and humorist (1952–2001), also
  known as …. Attributes include: instance of: human; occupation: …"*. Embedded with
  Jina Embedding V3 (multilingual, 8192-token window), `retrieval.passage` task for
  items and `retrieval.query` for queries. **One vector store per language** — the
  text for each language's shard is built from that language's labels/descriptions.
  <https://www.wikidata.org/wiki/Wikidata:Vector_Database>,
  <https://www.wikidata.org/wiki/Wikidata:Embedding_Project>
- **BLINK-style entity linking** (Facebook Research; the canonical bi-encoder recipe):
  the entity side of the bi-encoder is **`title [SEP] description`** — a short name
  plus a sentence-or-paragraph description, embedded once per entity; mentions are
  embedded separately and matched by dot product, then re-ranked by a cross-encoder.
  Labels alone are too ambiguous; the description disambiguates.
  <https://github.com/facebookresearch/BLINK>,
  <https://aclanthology.org/2022.naacl-industry.38.pdf>
- **Microsoft GraphRAG**: entity nodes carry an LLM-summarized **description**, and the
  default pipeline **embeds `name + description`** per entity (plus community-report
  and text-unit embeddings). The takeaway is the same shape: a short identifier plus a
  descriptive sentence is the embedded unit.
  <https://microsoft.github.io/graphrag/index/default_dataflow/>,
  <https://neo4j.com/blog/developer/global-graphrag-neo4j-langchain/>
- **Neo4j vector indexes / GraphRAG ecosystem**: the vector index sits on an embedding
  property that is "a vector of **combined text properties**" of the node; the
  `Neo4jVector` integration takes an explicit list of text node properties to embed,
  and hybrid retrievers pair the vector index with a keyword index.
  <https://neo4j.com/docs/cypher-manual/current/indexes/semantic-indexes/vector-indexes/>,
  <https://neo4j.com/developer/genai-ecosystem/vector-search/>

## 2. Vector-DB conventions for choosing/weighting properties

- **Weaviate text2vec** is the most explicit convention: by default it vectorizes
  **only `text`-typed properties**, sorted alphabetically, values concatenated with
  spaces, class name prepended, all lowercased; per-property `skip` excludes a
  property, `vectorizePropertyName` optionally prepends the property name to its value
  (the "prefix" idea), and named vectors take `source_properties` /
  `exclude_properties` lists. Numbers/dates/refs are **not** vectorized — they stay
  filterable fields.
  <https://weaviate.io/blog/pulling-back-the-curtains-on-text2vec>,
  <https://docs.weaviate.io/weaviate/manage-collections/vector-config>
- **Qdrant / pgvector / Vertex-style stores** have no schema-level vectorizer: the
  application renders a text template per record and embeds it client-side; scalar
  fields ride along as **payload/metadata for filtering**, not as embedded text
  (Vertex: `restricts` / `numeric_restricts`).
  <https://docs.cloud.google.com/vertex-ai/docs/vector-search/using-metadata>

## 3. Which literal properties matter

- **Label-like** (one short name; pick ONE by priority): `rdfs:label`,
  `skos:prefLabel`, `foaf:name`, `schema:name`, `dcterms:title`.
- **Description-like** (the disambiguator; BLINK/Wikidata/GraphRAG all include one):
  `schema:description`, `rdfs:comment`, `dcterms:description`, `skos:definition`,
  `skos:note`.
- **Type**: verbalized as text ("…, a Person") rather than left as an IRI — Wikidata
  includes `instance of` in the passage; type words give the embedding model the
  category signal that pure labels lack. The type IRI's own label (or local name) is
  the text.
- **Other literals**: include **short, categorical, human-readable** values with a
  property-name prefix (the Weaviate `vectorizePropertyName` convention — "occupation:
  sprinter"). **Numeric and date literals are normally NOT embedded raw**: embedding
  models do not order numbers, so `"height: 195"` adds noise — verbalize them only if
  bucketed/categorical, otherwise leave them to structured filters (the universal
  vector-DB metadata-filtering pattern).
  <https://callsphere.ai/blog/vector-search-filtering-semantic-similarity-metadata-constraints>,
  <https://neo4j.com/blog/developer/graph-metadata-filtering-vector-search-rag/>
- **Multilingual labels**: pick by a **language-preference chain** (e.g. `en` >
  plain-literal > anything), or — at Wikidata scale — build one store per language.
  Mixing languages inside one passage degrades monolingual models; multilingual
  models (Jina V3 class) tolerate it but per-language selection is still the norm.

## 4. Hybrid scoring: fusing text vectors with structural/lexical signals

- **Reciprocal Rank Fusion (RRF)** — `score(d) = Σ_i w_i / (k + rank_i(d))`, ranks
  1-based, `k = 60` is the standard, empirically robust constant (Azure AI Search,
  Elasticsearch, MariaDB, ParadeDB all default to it). Rank-based, so it needs **no
  score normalization** — the right default when the signals' score scales differ
  (cosine in [-1,1] vs Jaccard in [0,1]). Per-list weights = "weighted RRF"
  (Elasticsearch).
  <https://learn.microsoft.com/en-us/azure/search/hybrid-search-ranking>,
  <https://www.elastic.co/search-labs/blog/weighted-reciprocal-rank-fusion-rrf>,
  <https://www.paradedb.com/learn/search-concepts/reciprocal-rank-fusion>
- **Relative score fusion** — min-max normalize each list to [0,1], then a convex
  combination `alpha·text + (1−alpha)·other` (Weaviate's `relativeScoreFusion`,
  MongoDB's classic hybrid recipe). Preserves score *magnitudes* (a runaway best hit
  stays a runaway), so it can beat RRF when the underlying scores are meaningful;
  `alpha` is the tuning knob.
  <https://medium.com/mongodb/reciprocal-rank-fusion-and-relative-score-fusion-classic-hybrid-search-techniques-3bf91008b81d>
- **Graph-specific hybrids**: Neo4j GraphRAG pairs vector retrieval with keyword and
  graph traversal; GraphRAG "local search" fuses entity-embedding hits with their
  graph neighborhoods. For sparq the two native signals are `sparq-vectors` cosine
  (text) × `sparq-sim` weighted Jaccard (structure) — fusion recipe now in the
  `sparq-vectors` README, with a dependency-free `fuse` helper.

## 5. What sparq adopts (decisions)

1. **Template = label, type, description, extra prefixed literals**, in that order,
   ONE value per slot by predicate priority (Wikidata/BLINK shape; Weaviate prefix
   convention) — `EntityTextConfig` / `verbalize` in `sparq-vectors`.
2. **Language preference chain** (`["en", ""]` default; `""` = plain literal; stored
   `lang--dir` slots match on the tag before `--`; unmatched-language fallback so
   non-English-only graphs still embed).
3. **Char budget** (`max_chars`) — passage-sized text, label never silently dropped.
4. **Numbers/dates stay out of the default template**; opt-in per predicate with a
   prefix, documented as "verbalize only categorical values".
5. **Hybrid = RRF by default, alpha-blend (min-max normalized) when tuning** —
   `fuse_rrf` / `fuse_scores`, taking ranked lists so `sparq-vectors` never depends
   on `sparq-sim`.
