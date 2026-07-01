// [FABLE-5] sq-rvgr2.4 — SPARQL Vector & GenAI Extension, an Unofficial Proposal Draft.
//
// GROUNDING. Every normative statement in this draft is grounded in the read-only estate
// recon for the vector/GenAI surface (research/specs/vector-genai-estate-recon.md, branch
// docs/fable-program-recon-records): the built-and-tested surface of the opt-in sparq-vectors
// crate plus the adjacent sparq-nlq / sparq-introspect crates. Surfaces that are DESIGNED but
// NOT BUILT are labelled with a "Proposal" aside and confer no conformance obligation on any
// existing implementation. Code-level details the recon digest does not pin are marked with an
// "Editor's note" rather than guessed.
//
// HONESTY. This draft cites NO measured performance or accuracy figures: every measurement in
// the underlying estate is work-box non-canonical, and the spec factory's build-boundary
// honesty scan enforces the no-perf-numbers rule over this source. Approximation, uncalibrated
// confidence, and no-accuracy-claim features are stated plainly in the normative text.

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "SPARQL Vector & GenAI Extension")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

// A "Proposal" aside: marks designed-but-unbuilt surface. Mirrors _lib/spec.typ's note()
// (HTML aside + PDF block) so both targets render it, with an amber accent to distinguish a
// proposal (a design decision offered for review) from an informative note.
#let proposal(body) = context {
  if target() == "html" {
    html.elem("aside", attrs: (class: "note proposal"))[
      #html.elem("span", attrs: (class: "note-title"))[Proposal]
      #body
    ]
  } else {
    block(width: 100%, inset: 8pt, radius: 3pt, stroke: (left: 3pt + rgb("#c80")), fill: rgb("#fdf8ee"))[
      #strong[Proposal] — #body
    ]
  }
}

// An "Editor's note" aside: a detail this draft does not yet pin (the grounding digest does
// not capture it); to be confirmed against the implementation before the draft advances.
#let ednote(body) = context {
  if target() == "html" {
    html.elem("aside", attrs: (class: "note ednote"))[
      #html.elem("span", attrs: (class: "note-title"))[Editor's note]
      #body
    ]
  } else {
    block(width: 100%, inset: 8pt, radius: 3pt, stroke: (left: 3pt + rgb("#88a")), fill: rgb("#f4f5fb"))[
      #strong[Editor's note] — #body
    ]
  }
}

#spec-head()

#intro-section("abstract", "Abstract")[
  This document specifies a vector-search and generative-AI extension to SPARQL 1.1 as
  implemented by the sparq engine's opt-in vector estate. It defines the `vec:` vocabulary and
  the two magic-predicate graph patterns `vec:nearest` and `vec:search` (k-nearest-neighbour
  retrieval with score exposure as ordinary solution bindings); the separation of answer-exact
  and approximate execution modes; the answer-safety obligation for predicate-filtered vector
  search; the persisted vector-store container formats and their graph-staleness contract; the
  embedding-acquisition contract and remote-provider protocol; a provenance-weighted retrieval
  vocabulary; and the grounded-generation (natural-language-to-SPARQL) processing decisions.
  It further proposes — clearly labelled as proposals — an embedding provenance and versioning
  vocabulary required for result reproducibility, and surfaces the open design questions
  (metric selection, variable-driven queries, SERVICE and function forms, hybrid fusion in the
  query surface) that a future Working or Community Group would need to settle.
]

#sotd()

= Introduction

RDF stores increasingly pair symbolic query answering with dense-vector retrieval: entities
and literals are embedded into a vector space by a (typically external) embedding model, and a
query can ask for the _k_ nearest neighbours of a node or of an ad-hoc vector, optionally
constrained by ordinary graph patterns. Every major vendor has grown a proprietary idiom for
this — GraphDB's `similarity:` index queries, Amazon Neptune Analytics' vector functions,
Neo4j's vector indexes, Stardog's Voicebox, and the ecosystem of dedicated vector databases
(Qdrant, Weaviate, Milvus, pgvector) — but there is no standard SPARQL surface. This draft
writes down one coherent, implemented design so it can be reviewed, criticised, and compared.

The design philosophy is deliberately minimal:

- The #strong[only] vector query surface is a pair of _magic predicates_ in a dedicated
  namespace, recognised inside ordinary basic graph patterns. No new grammar productions, no
  new algebra operators, and no change to the SPARQL parser are required.
- Vector search results enter the solution multiset as ordinary bindings — a neighbour
  variable and an optional score variable — so every downstream SPARQL construct (joins,
  `FILTER`, `ORDER BY`, aggregation, federation) applies unchanged.
- Answer-exact execution is the default. Approximate nearest-neighbour (ANN) acceleration is
  an explicit opt-in whose non-exactness must be surfaced, never silent.

An implementation technique note: the sparq implementation realises these patterns as a
query-rewrite from the recognised pattern to inline data (a `VALUES`-equivalent block) over
the algebra, leaving the engine, planner, and WASM build unchanged. The technique is
informative; only the observable semantics in this document are normative.

This document also covers the surrounding estate a vector surface needs to be trustworthy:
persisted store formats and their staleness contract (@sec-formats), the embedding
acquisition contract (@sec-embedding), embedding provenance (@sec-provenance-model), and the
grounded-generation loop that turns natural-language questions into validated SPARQL
(@sec-grounded). Where the estate has a designed-but-unbuilt surface, this draft records the
design as a labelled #emph[Proposal] rather than implying it exists.

= Terminology and conformance

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED], #strong[MAY],
and #strong[OPTIONAL] in this document are to be interpreted as described in
#cite("RFC2119") and #cite("RFC8174") when, and only when, they appear in all capitals, as
shown here.

This document uses the following terms:

- A #dfn[vector store] is a component that persists a mapping from RDF term identifiers to
  dense vectors (one vector per term in the mainline model) and answers nearest-neighbour
  queries over it.
- A #dfn[vec-extended processor] is a SPARQL query processor that recognises and evaluates the
  graph patterns of @sec-patterns against a vector store.
- An #dfn[embedding client] is a component that obtains vectors for text inputs from an
  embedding model under the contract of @sec-embedding.
- A #dfn[grounded-generation processor] is a component that answers natural-language questions
  by generating, validating, and executing SPARQL against an RDF dataset per @sec-grounded.
- #dfn[Answer-exact] execution returns exactly the mathematically correct top-k under the
  declared metric; #dfn[approximate] execution may miss true neighbours (its recall is
  strictly below one).

Conformance is claimed per class: an implementation MAY implement any subset of the four
classes above, but MUST satisfy every obligation of each class it claims. Material marked
#emph[Proposal] or #emph[Editor's note], all examples, and all notes are non-normative.

= The vec: vocabulary <sec-vocab>

The vector extension vocabulary lives in a single namespace:

```turtle
PREFIX vec: <http://sparq.dev/vec#>
```

Two predicate IRIs are recognised in this draft:

#table(
  columns: 2,
  align: (left, left),
  table.header[Predicate][Role],
  [`vec:nearest` (`http://sparq.dev/vec#nearest`)], [k-nearest-neighbour pattern binding the
    neighbour variable only (@sec-patterns).],
  [`vec:search` (`http://sparq.dev/vec#search`)], [k-nearest-neighbour pattern additionally
    binding a similarity score (@sec-patterns).],
)

A vec-extended processor MUST raise a query error when a triple pattern uses any other IRI in
the `vec:` namespace as its predicate. Unrecognised `vec:` terms are reserved for future
revisions of this specification; treating them as ordinary (data-matched) predicates would
silently change meaning between revisions.

= Vector search graph patterns <sec-patterns>

== Recognised patterns

A vec-extended processor MUST recognise the following two triple-pattern shapes inside a basic
graph pattern:

```sparql
?node vec:nearest ( query k ) .
( ?node ?score ) vec:search ( query k ) .
```

Evaluating `vec:nearest` MUST bind `?node` to the `k` nearest neighbours of `query` in the
vector store, best-first at generation time (see @sec-ordering for what "best-first" does and
does not guarantee downstream). Evaluating `vec:search` MUST produce the same neighbour
bindings and MUST additionally bind `?score` to the cosine similarity between the query vector
and each neighbour's vector, as an `xsd:double` literal.

== Argument grammar

The parenthesised forms are #strong[ordinary SPARQL RDF collections] — syntactic sugar for
`rdf:first`/`rdf:rest` lists per SPARQL 1.1 #cite("SPARQL11-QUERY") and RDF 1.1
#cite("RDF11-CONCEPTS") — not new grammar. The extension recognises the list structurally:

- The object of both predicates MUST be a two-element collection, exactly `( query k )`.
- The subject of `vec:search` MUST be a two-element collection, exactly `( ?node ?score )`.
- The neighbour position (`?node`) MUST be a variable.

#ednote[
  The grounding digest pins a hard error for a non-variable #emph[neighbour] position; it does
  not capture whether a non-variable #emph[score] position is likewise a hard error or is
  handled otherwise. To be confirmed against the implementation before this draft advances.
]

== Argument typing <sec-typing>

The `query` argument MUST be a constant, one of:

+ a #strong[node IRI]: the query vector is that node's stored vector, and the seed node itself
  is excluded from the results; or
+ a #strong[literal] whose lexical form is a comma-separated list of floating-point values:
  the query vector is parsed from the literal, and its dimension MUST equal the store's
  declared dimension.

The `k` argument MUST be a constant, non-negative integer literal.

#ednote[
  Because `k` is constrained to be non-negative, a negative `k` is excluded by typing; however
  the behaviour at `k = 0` has no pinned normative text in the implemented surface. This draft
  proposes (see @sec-open) that `k = 0` evaluate to zero solutions rather than an error.
]

== Error conditions

A vec-extended processor MUST fail the query with a hard query error (not an empty result) in
each of the following cases:

+ the predicate is an unrecognised IRI in the `vec:` namespace (@sec-vocab);
+ the neighbour position is not a variable;
+ the `query` or `k` argument is not a constant;
+ either argument list has the wrong number of elements;
+ the query vector's dimension does not equal the store's dimension;
+ an argument list is malformed, dangling, or cyclic (the `rdf:first`/`rdf:rest` structure
  does not form a well-formed finite list).

Failing loudly is a design decision: every one of these conditions indicates a query that
cannot mean what its author intended, and returning an empty (or worse, partial) result would
convert an authoring bug into a silent wrong answer.

== Result multiplicity and ordering <sec-ordering>

The solutions produced by a `vec:` pattern enter the surrounding query as an #strong[unordered]
solution multiset, exactly as if they had been written as an inline `VALUES` block: SPARQL
joins do not preserve generation order. Consequently:

- A consumer that requires best-first order MUST apply an explicit
  `ORDER BY DESC(?score)` over a `vec:search` score variable.
- A processor MUST NOT promise any implicit ordering of `vec:nearest` results after joins.

The score is the cosine similarity in the closed interval from −1 to 1 (see @sec-metric),
exposed as `xsd:double`.

#ednote[
  No tie-break rule is pinned when two neighbours have equal similarity: the implemented
  surface defines no determinism contract across runs or across implementations for ties.
  @sec-open proposes settling a deterministic tie-break in a future revision.
]

== Empty and degenerate queries

The following degenerate cases MUST be handled as stated (all are implemented and tested
behaviour, not proposals):

- If the `query` argument is a node IRI that is absent from the store (the node exists in the
  graph but was never embedded, or does not exist at all), the pattern MUST yield zero
  solutions — not an error, and not arbitrary neighbours.
- If the query vector is all-zero, the pattern MUST yield zero solutions. (Cosine similarity
  against the zero vector is undefined; refusing to rank is the only honest answer.)

== Worked example

The following query retrieves the ten reports most similar to a seed document, keeps only
those in a thematic class, and recovers best-first order explicitly:

```sparql
PREFIX vec: <http://sparq.dev/vec#>
PREFIX ex:  <http://example.org/>

SELECT ?report ?score WHERE {
  ( ?report ?score ) vec:search ( ex:seed-document 10 ) .
  ?report a ex:ClimateReport .
}
ORDER BY DESC(?score)
```

#note[
  The class constraint `?report a ex:ClimateReport` interacts with the vector pattern under
  the filtered-search semantics of @sec-filtered: the ten neighbours are the top ten
  #emph[within] the constrained candidate set, not the global top ten post-filtered (which
  could return fewer than ten).
]

= Answer-exact and approximate execution modes <sec-modes>

This specification distinguishes two normative execution modes:

+ #strong[Answer-exact mode] (the default). The processor MUST return exactly the true top-k
  under the declared metric. The implemented default is an exact full scan of the store, and
  the exact backend is the only backend a processor MAY use without an explicit opt-in.
+ #strong[Approximate mode] (explicit opt-in). The processor MAY accelerate retrieval with an
  approximate index — the implemented estate includes an in-memory HNSW graph
  #cite("HNSW"), an on-disk DiskANN/Vamana graph #cite("DiskANN"), and product-quantisation
  candidate generation #cite("PQ") — whose recall is strictly below one.

For approximate mode, a processor:

- MUST require an explicit opt-in (a distinct entry point, feature flag, or equivalent) —
  approximate execution MUST NOT be substituted silently for answer-exact execution;
- MUST surface, in its API contract and documentation, that results are approximate with
  recall below one;
- MUST NOT represent approximate results as answer-exact to any downstream consumer.

The rationale is the project's empirical-honesty rule: a SPARQL answer is a claim about the
data, and an ANN index can silently drop true answers. Retrieval-and-estimate workloads may
accept that trade; answer-serving workloads must not have it made for them.

#note[
  This document deliberately cites no recall or throughput figures: every measurement in the
  underlying estate is machine-local and non-canonical. A future revision backed by a
  canonical benchmark authority could add an informative recall annex; the normative content
  here does not depend on one.
]

Approximate-mode entry points in the implemented estate are additionally staleness-guarded
against the store contract of @sec-staleness.

Quantised representations (a scalar quantiser from 32-bit floats to 8-bit integers, and a
product quantiser with an asymmetric-distance table and a persistable codebook) exist as
storage/acceleration options; a processor using them for candidate generation is in
approximate mode and carries that mode's obligations.

= Filtered vector search <sec-filtered>

When a `vec:` pattern occurs in a basic graph pattern alongside ordinary triple patterns that
constrain the neighbour variable, the processor MAY evaluate the search #emph[filtered]: only
candidates satisfying the constraints are eligible neighbours, so the top-k is taken within
the eligible set. This is the standard "filtered ANN" problem, and it carries the
specification's central soundness obligation:

- The #dfn[candidate mask] MUST be derived from the #strong[connected component] of the
  neighbour variable in the join graph of the surrounding basic graph pattern — every triple
  pattern transitively join-connected to the neighbour variable participates, including
  cyclic sub-patterns.
- The filtered top-k MUST equal the result of post-filtering the #emph[unfiltered] ranking by
  that same transitive constraint — filtering is an optimisation, never a semantic change.
- Every returned identifier MUST be a member of the mask.
- An empty mask MUST yield zero solutions.

A processor MAY choose between pre-filtering (masking candidates before ranking) and
post-filtering (ranking then discarding) on cost grounds; the choice MUST NOT be observable in
the result set. The implemented estate drives this with a cost model and an iterative
over-fetch strategy for approximate backends, both informative here.

A processor MAY cache derived masks across query preparations. A cached mask MUST be keyed by
both the constraining sub-pattern and the graph's fingerprint (@sec-staleness), and MUST be
invalidated when the fingerprint changes; serving a mask computed against a different graph
state is unsound.

= Distance metric <sec-metric>

The mainline metric is #strong[cosine similarity over L2-normalised vectors], with scores in
the closed interval from −1 to 1. On normalised vectors the cosine relates to Euclidean
distance by `cos = 1 − d²/2`, which is how an implementation may derive it from a
Euclidean-native index.

The mainline query surface (@sec-patterns) offers #strong[no metric selection]: cosine is
implicit. This is an acknowledged gap (@sec-open).

The structure-aware store extension (@sec-formats) records a per-block `Metric` tag with
values `Euclidean` and `NonEuclidean`. A processor reading such a store MUST reject an attempt
to apply a Euclidean or cosine distance to a block tagged `NonEuclidean` (for example, a
hyperbolic taxonomy embedding): the numbers a Euclidean comparison would produce on such a
block are meaningless, and rejecting is the metric-correctness guard.

= Persisted vector store formats <sec-formats>

== The primary container

The primary container (file extension `.spqv`) is a flat, memory-mappable file holding
exactly one 32-bit-float vector per embedded RDF term identifier. All integers are
little-endian. The version-2 layout is, in order:

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Field][Type / size][Meaning],
  [magic], [4 bytes], [the ASCII bytes `SPQV`],
  [version], [`u32`], [container version; this draft describes version 2],
  [dimension], [`u32`, at byte offset 8], [vector dimensionality, one value for the whole
    store],
  [count], [`u64`], [number of stored vectors],
  [reserved], [12 bytes], [reserved for future use],
  [fingerprint], [24 bytes, at byte offset 32], [graph fingerprint: dictionary length,
    triple count, and content hash, each `u64` (@sec-staleness)],
  [data], [dense `f32`], [the vectors, densely packed],
  [index], [sorted pairs of `u32`], [an (identifier, slot) index sorted by identifier],
)

A store builder SHOULD support a streaming write path whose memory use is independent of the
store size, and a reader SHOULD support opening from a byte slice (enabling memory-mapped and
in-memory use alike). Both exist in the implemented estate.

The header records the dimension and the graph fingerprint — and, in version 2,
#strong[nothing about the embedding model that produced the vectors]. @sec-provenance-model
proposes closing that gap; it is the most consequential change this draft asks for.

== Sibling containers

Three sibling containers share the same magic-plus-version discipline:

- `.spqg` (magic `SPQG`): an on-disk DiskANN/Vamana proximity graph for approximate mode.
- `.spqd` (magic `SPQD`): a crash-durable incremental delta sidecar (add, remove, update,
  compact), tied to the store generation it patches.
- `.spqs` (magic `SPQS`): a self-describing schema header for structure-aware vectorisation
  (typed-literal encoder blocks, each carrying the per-block `Metric` tag of @sec-metric).

#ednote[
  The byte-level layouts of the sibling containers are not pinned by the grounding digest and
  are therefore not specified here; only their existence, roles, and the `.spqs` metric tag
  are. A future revision should specify them or delegate to a registry.
]

== Graph fingerprint and staleness contract <sec-staleness>

A vector store is only meaningful relative to the exact dictionary state of the graph it was
built from: the store maps #emph[term identifiers], not terms, to vectors. The staleness
contract is therefore normative:

- The fingerprint MUST comprise the dictionary length, the triple count, and a content hash
  computed over the dictionary's term #emph[set] in an identifier-order-independent manner.
  The hash MUST be stable across thread counts (parallel and serial ingest of the same data
  MUST fingerprint identically).
- A fingerprint check before serving is #strong[necessary but not sufficient]: two graphs with
  identical term sets can assign identifiers in different orders, and the fingerprint —
  being deliberately order-independent — cannot tell them apart.
- To serve queries from a persisted store, the graph MUST have been round-tripped through the
  engine's native save/open path, which freezes identifier order. A graph #strong[re-parsed]
  from a source serialisation MUST NOT be paired with a previously persisted store, even when
  the fingerprint matches: identifier assignment may have changed, and every vector would
  silently attach to the wrong term.

The last clause is the sharpest edge in the whole design; see also the integrity discussion in
@sec-security.

== Bulk import validation

A bulk import path (from `.npy` arrays or raw numeric dumps, under an explicit
row-to-identifier contract) MUST be fail-closed: the importer MUST reject wrong element type,
wrong shape, wrong row order, wrong length, and non-finite values, and MUST bound the bytes it
reads while parsing the `.npy` header. Fail-open import would let a malformed artifact
manufacture plausible-looking neighbours.

= Embedding acquisition <sec-embedding>

== The embedding client contract

An embedding client MUST return, for a batch of text inputs, #strong[exactly one] vector per
input, #strong[in input order], each of exactly the declared dimension and containing only
finite values.

When the vectors come from a live remote provider, the client MUST validate the response and
MUST reject it on any of: wrong vector count, wrong dimension, non-finite values, a duplicate
result index, a missing result index, or an out-of-range result index. Silently re-ordering
or padding a provider response would attach vectors to the wrong terms — the same failure
class as the staleness edge of @sec-staleness.

== Remote provider protocol

The implemented remote protocol is OpenAI-compatible #cite("OPENAI-EMB"):

- `POST` to the provider base URL's `/v1/embeddings` endpoint with a JSON body carrying the
  model identifier and the input batch, authenticated with a bearer token;
- the response's per-item #emph[index] field MUST be honoured when reassembling the batch
  (order of arrival is not trusted).

Client configuration is drawn from the environment: `SPARQ_EMBEDDINGS_API_KEY` (REQUIRED),
`SPARQ_EMBEDDINGS_BASE_URL`, and `SPARQ_EMBEDDINGS_MODEL`, the latter two with implementation
defaults (the public OpenAI endpoint and a small text-embedding model, respectively). The
defaults are informative; the validation obligations above are not.

== Test-only embedders

The estate ships a hash-based embedder for deterministic tests. It captures #strong[no]
semantics whatsoever — semantically related words are unrelated vectors under it. It MUST NOT
be shipped or configured as a retrieval feature; it exists so the machinery around embeddings
can be tested without a model.

== Verbalisation

Entities are turned into embeddable text by a verbalisation pipeline (label-chain resolution
with multilingual support and a length budget). Verbalisation choices change what a vector
means; @sec-provenance-model folds the verbalisation regime into the proposed provenance
record. Note that verbalised text is exactly what leaves the deployment when a remote
provider is used (@sec-security).

= Embedding provenance and versioning <sec-provenance-model>

Version 2 of the primary container records the vector dimension and the graph fingerprint —
and #strong[no embedding-model identity, version, distance metric, or normalisation regime].
The provider's model string exists only as runtime configuration and is never persisted.
Consequently two stores built by different models (or the same model at different versions,
or with different normalisation) are indistinguishable to a processor as long as their
dimensions agree, and comparing a query vector from one model against stored vectors from
another produces well-formed, meaningless numbers.

#proposal[
  A future container version SHOULD make embedding provenance mandatory, and this draft
  proposes the following as the normative shape for that revision:

  + The store header MUST record: an embedding #strong[model identifier], a #strong[model
    version] (or content digest where the provider exposes one), the #strong[distance
    metric], the #strong[normalisation regime], and the #strong[dimension] (already present
    in version 2). It SHOULD record the verbalisation regime used to produce the embedded
    text.
  + A processor MUST reject a query whose query-vector provenance is incompatible with the
    store — a different model identifier, model version, metric, normalisation, or dimension
    — rather than return well-formed meaningless results. Literal query vectors
    (@sec-typing) carry no provenance and SHOULD at minimum be dimension- and
    normalisation-checked; a processor MAY require an explicit override to accept them.
  + The same record SHOULD be expressible in RDF so datasets can describe their vector
    indexes; a natural home is the `vec:` namespace (for example, terms for the model
    identifier, model version, metric, normalisation, and dimension of a store), aligned with
    PROV-O #cite("PROV-O") for derivation. The exact term set is left to a future revision.

  None of this is implemented today; it is the single most important gap this draft asks
  reviewers to weigh, because without it vector answers are not reproducible even in
  principle: the store cannot say what function produced it.
]

= Hybrid symbolic–vector retrieval <sec-hybrid>

The implemented hybrid surface is #strong[library-only]: reciprocal-rank fusion
#cite("RRF") (plain and weighted, with the conventional constant of sixty in the rank
denominator) and score fusion exist as host-language helpers that combine a vector ranking
with a symbolic (for example, full-text) ranking. #strong[No SPARQL syntax for hybrid fusion
exists]; within a query, hybrid behaviour today is what you compose yourself from a `vec:`
pattern, other ranked sources, and explicit arithmetic over score bindings.

#proposal[
  The designed-but-unbuilt in-query hybrid surface would add a fusion form (a `vec:hybrid`
  pattern or equivalent) exposing weighted reciprocal-rank fusion over two or more ranked
  sub-retrievals, and a second-stage rerank seam (for a cross-encoder) behind the same
  opt-in-and-flagged discipline as approximate mode (@sec-modes). Named and multi-vector
  stores (more than one vector per term, addressed by name) are likewise designed but
  unbuilt: the version-2 container holds exactly one vector per identifier. These remain
  proposals; no conformance language attaches to them in this draft.
]

= Provenance-weighted retrieval <sec-pkg>

Retrieval over a knowledge graph whose triples carry assurance metadata can weight candidates
by how well-supported they are. The implemented reader recognises the following vocabulary:

#table(
  columns: 2,
  align: (left, left),
  table.header[Term][Meaning],
  [`pkg:confidence` (`https://sparq.dev/ns/pkg#confidence`)], [a numeric confidence asserted
    for a statement],
  [`pkg:assurance`], [an assurance class; recognised objects are `secx:Proven`,
    `secx:Claimed`, and `secx:Conjectured`],
  [`prov:wasDerivedFrom`], [PROV-O derivation #cite("PROV-O"), used to trace a statement to
    its source],
  [`pkg:discoveredFrom`], [the discovery channel a statement arrived through],
)

The triple weight is computed as
`w(t) = clamp(assurance_mult × confidence × source_reliability, floor, 1.0)` and the reader
is #strong[fail-open]: on a plain graph carrying none of this vocabulary every weight is 1.0
and retrieval is unchanged.

#ednote[
  The full namespace IRI bound to the `secx:` prefix is not pinned by the grounding digest
  and is deliberately not stated here; it must be confirmed before this section can be
  implemented from the text alone.
]

#note[
  Provenance weighting ships with #strong[no accuracy claim]. The literature reports
  inconsistent gains from this family of techniques, and the implementation gates adoption
  behind an on/off ablation with an explicit adopt-only-on-measured-lift-else-abandon bar.
  This draft records the vocabulary and the weight formula so they can be reviewed; it does
  not recommend enabling the feature.
]

The adjacent structure-aware vectorisation estate (typed-literal encoders, taxonomy encoders
with Euclidean and hyperbolic geometries, SHACL/OWL prior extraction, knowledge-graph-embedding
training) similarly carries #strong[no accuracy claim] and is out of normative scope for this
draft beyond the `.spqs` metric guard of @sec-metric. A propose-then-verify neuro-symbolic
gate exists — vector search proposes candidate triples and a deductive gate admits a candidate
only if it introduces no new SHACL violation and no new OWL inconsistency, failing closed —
and is likewise recorded here informatively.

= Grounded generation over RDF <sec-grounded>

The generative-AI half of this extension is a natural-language question-answering loop that
is #strong[grounded] in the dataset: a language model writes SPARQL, but only the dataset
answers questions. A grounded-generation processor:

+ MUST ground generation in a machine-derived description of the dataset (the implemented
  estate derives schema cards from introspection: vocabulary usage, VoID-style statistics,
  SHACL shapes, characteristic sets) rather than letting the model guess the schema;
+ MUST validate a generated query before execution, and MUST NOT execute a query that fails
  validation (the loop instead enters repair);
+ MUST derive answers exclusively from the bindings returned by executing the validated query
  against the dataset — never from the language model's parametric knowledge;
+ MAY iterate a bounded generate–validate–execute–repair loop on failure;
+ SHOULD attach citations that trace each answer to the solution bindings (and thus the
  triples) that support it;
+ MUST NOT present an assurance qualifier derived from #emph[asserted] metadata
  (@sec-pkg) as a #emph[calibrated] confidence. The implemented qualifier band reflects what
  the data claims about itself; no reliability measurement backs it, and a future revision
  can only upgrade this once calibration is measured.

Deterministic replay of recorded model completions is the implemented testing regime for this
loop. Replay-based scores validate the #emph[mechanism] — the loop, the validator, the
harness — and MUST NOT be quoted as any real model's accuracy.

#proposal[
  Grammar-constrained (logit-masked) decoding of SPARQL — constraining generation itself so
  only grammatical queries and in-dictionary terms can be produced — is designed but not
  implemented; the current implementation applies a post-hoc dictionary constraint only. A
  future revision may specify constrained decoding as a SHOULD for grounded-generation
  processors once implementation experience exists.
]

= Accuracy and determinism considerations <sec-accuracy>

This section states plainly what this extension does #strong[not] promise.

- #strong[No accuracy figures are part of this specification.] Every recall, fidelity, and
  answer-quality measurement in the underlying estate was taken on a non-canonical work
  machine; none is citable as a floor. Conformance to this draft is a property of semantics
  (exactness, error behaviour, validation), never of a benchmark score.
- #strong[Approximate mode is honestly approximate.] Its recall is below one by construction;
  the normative obligations of @sec-modes exist so no consumer mistakes it for answer-exact.
- #strong[Tie order is unspecified.] Equal-similarity neighbours have no defined order or
  selection rule at the top-k boundary (@sec-ordering); cross-implementation determinism for
  ties is an open design question (@sec-open).
- #strong[Embedding models are external nondeterminism.] Two embedding runs are comparable
  only under identical model, version, normalisation, and verbalisation — which the persisted
  format cannot yet attest (@sec-provenance-model). Until it can, "same query, same store"
  reproducibility holds, but "rebuild the store, get the same answers" does not in general.
- #strong[The learned-feature estate makes no claims.] Knowledge-graph-embedding training and
  provenance weighting are explicitly no-accuracy-claim features gated on unmeasured lift
  (@sec-pkg).

= Security and privacy considerations <sec-security>

#strong[Data disclosure to embedding providers.] Under the remote protocol of
@sec-embedding, the verbalised text of entities — labels, literals, and whatever else the
verbalisation budget admits — is transmitted to a third-party provider. Deployments MUST
treat everything embedded through a remote provider as disclosed to that provider, and
SHOULD NOT embed graphs containing personal or confidential data through an external endpoint
absent a data-processing agreement. Bearer credentials are read from the environment
(`SPARQ_EMBEDDINGS_API_KEY`); standard secret-handling hygiene applies, and endpoints SHOULD
be HTTPS (the implementation default is).

#strong[Embedding inversion and membership inference.] Dense embeddings are not an
anonymisation: published research demonstrates partial reconstruction of source text from
embeddings and membership inference against embedding stores. A `.spqv` file derived from
sensitive text SHOULD be access-controlled as if it were the text. (Informative; the
grounding estate takes no position beyond this warning.)

#strong[Malformed artifacts.] The fail-closed import obligations (@sec-formats) — typed,
bounded, finite-checked parsing of foreign arrays — and the fail-closed provider-response
validation (@sec-embedding) are this extension's parsing attack surface controls.

#ednote[
  The trust model for #emph[opening] an untrusted `.spqv`/`.spqg`/`.spqd`/`.spqs` container
  (as opposed to importing foreign numeric data) is not pinned by the grounding digest —
  e.g. whether every header field and index entry is bounds-checked against the mapped
  length. It must be documented before this draft advances.
]

#strong[Integrity via the staleness contract.] Serving vectors against the wrong dictionary
state silently attaches every answer to the wrong term (@sec-staleness). The fingerprint
check and the round-trip-only rule are integrity controls, not performance niceties; the
mask-cache invalidation rule of @sec-filtered is the same control applied to cached derived
state.

#strong[Approximation as an integrity property.] An ANN index can drop true answers. In
adversarial settings the gap between approximate and exact answers is itself a manipulation
surface (an attacker who can influence index construction can influence what goes missing);
answer-serving paths MUST use answer-exact mode per @sec-modes.

#strong[Prompt injection in grounded generation.] A grounded-generation processor feeds
model-bound context derived from the dataset (schema cards, labels), and dataset content can
embed adversarial instructions. The validate-before-execute obligation of @sec-grounded is
the primary control: whatever the model was talked into writing, only a validated SPARQL
query executes, and answers come only from its bindings. Deployments SHOULD additionally
restrict the execution surface available to generated queries (for example, to read-only
operations); this draft does not yet pin that restriction normatively.

#strong[Provenance metadata is attacker-writable data.] The vocabulary of @sec-pkg weights
retrieval by #emph[asserted] confidence and assurance. Anyone who can write
`pkg:confidence` triples can bias retrieval; the weights inherit the write-access-control
story of the graph itself, and the fail-open default means an attacker can also simply omit
the vocabulary. Consumers MUST NOT treat provenance-weighted rank as evidence of truth.

= Open design questions <sec-open>

The following are the known open questions a Working or Community Group adopting this draft
would need to settle. Each is a real gap in the implemented design, recorded honestly rather
than papered over:

+ #strong[Variable-driven queries.] The `query` argument is constant-only (@sec-typing); a
  per-row (correlated / lateral) k-NN — "for each ?paper, its five nearest" — is
  inexpressible. Options include lateral-join semantics or relaxing the constant rule with
  defined binding-time semantics.
+ #strong[Surface form.] Only the magic-predicate form exists. A `SERVICE`-based form and an
  extension-function form were considered in the estate's design records; vendor idioms
  (GraphDB `similarity:` #cite("GRAPHDB-SIM"), Neo4j vector index procedures
  #cite("NEO4J-VEC"), Neptune Analytics #cite("NEPTUNE-AN"), Stardog Voicebox
  #cite("STARDOG-VB"), pgvector operators #cite("PGVECTOR")) split across all three shapes.
  A standard should pick one normative form and define the others' equivalence or absence.
+ #strong[Metric selection.] Cosine is implicit and unselectable in the query surface
  (@sec-metric); a metric argument or a store-declared metric (via @sec-provenance-model)
  are the candidate fixes.
+ #strong[`k = 0` and tie-break text.] Neither has pinned normative behaviour today
  (@sec-patterns, @sec-ordering). This draft proposes zero solutions for `k = 0` and a
  deterministic tie-break (for example, by term order) as the future text.
+ #strong[Named / multi-vector stores and in-query hybrid.] Both designed, neither built
  (@sec-hybrid).
+ #strong[Embedding provenance.] The mandatory-provenance container revision of
  @sec-provenance-model — in the editors' view the highest-priority item on this list.
+ #strong[Calibrated confidence.] Upgrading the asserted-assurance band of @sec-grounded to a
  measured, calibrated confidence requires a reliability evaluation that does not yet exist.

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017.]),
  ("SPARQL11-QUERY", [Harris, S.; Seaborne, A. (eds.) #emph[SPARQL 1.1 Query Language].
    W3C Recommendation, 21 March 2013. https://www.w3.org/TR/sparql11-query/.]),
  ("RDF11-CONCEPTS", [Cyganiak, R.; Wood, D.; Lanthaler, M. (eds.) #emph[RDF 1.1 Concepts and
    Abstract Syntax]. W3C Recommendation, 25 February 2014.
    https://www.w3.org/TR/rdf11-concepts/.]),
  ("PROV-O", [Lebo, T.; Sahoo, S.; McGuinness, D. (eds.) #emph[PROV-O: The PROV Ontology].
    W3C Recommendation, 30 April 2013. https://www.w3.org/TR/prov-o/.]),
  ("HNSW", [Malkov, Y.; Yashunin, D. #emph[Efficient and Robust Approximate Nearest Neighbor
    Search Using Hierarchical Navigable Small World Graphs]. IEEE TPAMI, 2020.]),
  ("DiskANN", [Subramanya, S. J. et al. #emph[DiskANN: Fast Accurate Billion-point Nearest
    Neighbor Search on a Single Node]. NeurIPS, 2019.]),
  ("PQ", [Jégou, H.; Douze, M.; Schmid, C. #emph[Product Quantization for Nearest Neighbor
    Search]. IEEE TPAMI, 2011.]),
  ("RRF", [Cormack, G. V.; Clarke, C. L. A.; Büttcher, S. #emph[Reciprocal Rank Fusion
    Outperforms Condorcet and Individual Rank Learning Methods]. SIGIR, 2009.]),
  ("OPENAI-EMB", [OpenAI. #emph[Embeddings API reference (v1/embeddings)].
    https://platform.openai.com/docs/api-reference/embeddings. (Vendor documentation;
    catalogued, not measured.)]),
  ("GRAPHDB-SIM", [Ontotext. #emph[GraphDB Semantic Similarity Searches]. Vendor
    documentation; catalogued, not measured.]),
  ("NEO4J-VEC", [Neo4j. #emph[Vector Indexes]. Vendor documentation; catalogued, not
    measured.]),
  ("NEPTUNE-AN", [Amazon Web Services. #emph[Vector Similarity in Neptune Analytics]. Vendor
    documentation; catalogued, not measured.]),
  ("STARDOG-VB", [Stardog. #emph[Voicebox]. Vendor documentation; catalogued, not
    measured.]),
  ("PGVECTOR", [pgvector contributors. #emph[pgvector: Open-source Vector Similarity Search
    for Postgres]. https://github.com/pgvector/pgvector. (Vendor documentation; catalogued,
    not measured.)]),
))
