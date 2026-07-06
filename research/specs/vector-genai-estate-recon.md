# Vector + GenAI Estate Recon

> Grounding for the Vector/GenAI SPARQL Proposed Spec (bead sq-rvgr2.4). Read-only recon; not itself normative.

## Summary

Opt-in `crates/sparq-vectors` (one f32 embedding per dict term-id, flat mmap `.spqv`) plus
adjacent `sparq-nlq`/`sparq-introspect` NL crates. The **ONLY parsed+executed vector SPARQL
surface** is two magic predicates in namespace `http://sparq.dev/vec#`: `?node vec:nearest (
query k )` and `( ?node ?score ) vec:search ( query k )`, an algebra-to-VALUES rewrite behind
the `vec-predicate` feature with the engine unchanged (`src/rewrite.rs`).

Search is an answer-exact full scan plus opt-in approximate HNSW / DiskANN-Vamana / PQ
backends whose recall is explicitly under 1.0. Filtered-ANN (BGP-to-`IdMask`, cost-model
pre/post-filter, transitive connected-component constraint, cross-prepare mask cache) is built
and tested.

Key missing provenance: the `.spqv` header records dimension (u32) and a graph fingerprint
(dict_len+triple_count+content_hash) but records **NO embedding model-id, model-version, or
distance metric**; cosine is implicit in the mainline path. Vendor precedent (GraphDB
`similarity:`, Stardog Voicebox, Neptune, Neo4j, Qdrant/Weaviate/pgvector/Milvus) is catalogued
in `research/`, framing `vec:`-in-SPARQL as the headline parity gap with V4/V5/V6/V8/V9/GNCE
designed-but-unbuilt.

---

## What Is Built and Tested

- **`.spqv` mmap store**: SPQV magic + version-2 header (dim u32 at off 8, count u64, 12B
  reserved, 24B graph fingerprint at off 32, dense f32 data, sorted id-to-slot index)
  `src/store.rs:1–17,62–70`; `StreamingWriter` O(1)-RAM build + `open_from_bytes`;
  `tests/store.rs`, `staleness_contract.rs`.

- **`vec:nearest` / `vec:search` magic predicates**, the ONLY parsed+executed vector SPARQL
  surface. NS `http://sparq.dev/vec#` `lib.rs:112–123`; spargebra-algebra to inline VALUES
  rewrite, engine/planner/wasm unchanged, `src/rewrite.rs:1–79` dispatch `404–412`; entry
  points `query_vec`/`prepare_vec`/`rewrite_query` `rewrite.rs:170–289` behind `vec-predicate`.

- **Approximate `vec:` twins** `query_vec_approx`/`prepare_vec_approx` over an on-disk
  `DiskAnnIndex`, `rewrite.rs:214–257`, recall under 1.0, staleness-guarded, gated
  `approx-ann`.

- **Exact + approximate ANN**: `nearest_exact`/`nearest_term_exact(_checked)` `src/ann.rs`;
  in-RAM HNSW `VectorIndex` via `instant-distance` (`approx-ann`); on-disk DiskANN/Vamana
  `.spqg` SPQG (`src/diskann.rs`) with PQ candidate cache.

- **Filtered (predicate-constrained) ANN**: `IdMask` + `nearest_exact_filtered`
  `src/filter.rs`; BGP-to-`IdMask` derivation over the join-connected connected-component
  (transitive/cyclic), engine-evaluated mask `rewrite.rs:768–825`,
  `connected_component 943–979`; pre/post-filter `CostModel` `src/cost.rs`; cross-prepare mask
  cache keyed by (sub-BGP, graph `Fingerprint`) with soundness proof + adversarial
  invalidation tests `rewrite.rs:849–1211`.

- **Iterative over-fetch filtered path + pluggable `AnnBackend`** (`ExactBackend` answer-exact,
  `ApproxBackend` recall under 1.0) `src/backend.rs`.

- **Quantisation**: `ScalarQuantizer` (f32 to u8, 4×) + `ProductQuantizer`
  (ADC/DistanceTable) with persistable codebook + `EncodedStore`, `src/quant.rs`.

- **Embedder seam**: provider-agnostic `Embedder` trait (one `dim()`-length finite vector per
  input, in order) + test-only `HashEmbedder` (NO semantics) `src/embed.rs:10–83`; `provider`
  feature = OpenAI-compatible `/v1/embeddings` request/response shape with caller `Transport`
  (`embed.rs:108–146`); `embeddings` feature = concrete reqwest `RemoteEmbedder::from_env`
  reading `SPARQ_EMBEDDINGS_API_KEY` / `SPARQ_EMBEDDINGS_BASE_URL` / `SPARQ_EMBEDDINGS_MODEL`
  (`embed.rs:159–253`).

- **Verbalisation + label pipelines** `verbalize`/`embed_entities`/`embed_labels` (label chain,
  multilingual, budgeted) `src/verbalize.rs`, `src/labels.rs`.

- **Hybrid fusion helpers LIBRARY-ONLY** (NOT in SPARQL): `fuse_rrf`/`fuse_rrf_weighted`/
  `fuse_scores`/`hybrid_search` (RRF k=60) `src/fuse.rs`.

- **Bulk import** `import_npy`/`import_numeric_dump` via a row-to-dict-id contract, fail-closed
  on dtype/shape/order/length/non-finite, bounded `.npy` header, `src/import.rs`.

- **Incremental delta sidecar** (add/remove/update/compact + crash-durable `.spqd` SPQD
  sidecar, generation-tied) `src/delta.rs` (`delta` feature).

- **Structure-aware vectorisation epic sq-0wo9e** (behind `structure`/`structure-shacl`/`kge`/
  `neuro-symbolic` features, zero default-build code): typed-literal encoders + self-describing
  `.spqs` `SchemaHeader` (SPQS, per-block `Metric` tag Euclidean/NonEuclidean + metric-
  correctness guard) `src/encode.rs:433–558`; enum `Codebook` + QUDT unit normaliser;
  SHACL/OWL prior extractor; taxonomy DAG + Euclidean/hyperbolic encoders + `GeometryGate` +
  `DisjointnessOracle`; closure + type-constrained negative sampler `structure.rs`.

- **KGE trainer + eval harness** (`kge` feature, hand-rolled SGD): DistMult/ComplEx `train.rs`
  + filtered link-prediction ablation; explicitly NO accuracy claim.

- **Provenance-weight reader** `w(t)` (`structure` feature): mines `pkg:confidence` /
  `pkg:assurance` (`secx:Proven/Claimed/Conjectured`) / `prov:wasDerivedFrom` /
  `pkg:discoveredFrom`, fail-open 1.0 on plain graphs, `WeightMode` on/off ablation,
  `src/provenance.rs:1–67`.

- **Neuro-symbolic propose-then-verify** (`neuro-symbolic` feature): vector ANN proposes
  candidates then a deductive gate admits only if no new SHACL violation and no new OWL
  inconsistency, fail-closed, `src/verify.rs`.

- **Compose read-path URI-hiding view** + model-free fidelity/cost `ab_report` + measured K4
  real-model blinded paired A/B, `src/compose.rs` (`compose` feature).

- **Adjacent NL-retrieval**: `sparq-nlq` NL-to-SPARQL loop (ground/generate/validate/execute/
  repair; `ReplayLlm`/`RecordingLlm`/live `AnthropicLlm`/`nlq-endpoint`) + `sparq_nlq::eval`
  answer-F1 harness + `citations`/`qualify` provenance features; `sparq-introspect` schema
  card/VoID/SHACL/characteristic-sets; `skills/genai-retrieval/SKILL.md`.

---

## Designed, Not Built

- **Richer `vec:` forms**: only `vec:nearest`/`vec:search` exist. `vec:textNearest`,
  `vec:score`, `vec:hybrid` named only in `research/feature-research-vector-genai.md:177,180`
  (V1/V4), no parser/executor.

- **V4 in-query hybrid + weighted-RRF in SPARQL**: fusion is library-only (`src/fuse.rs`); no
  SPARQL exposure. `feature-research-vector-genai.md:180,204`.

- **V5 RaBitQ binary quantisation** (1-bit + popcount) NOT built; only SQ(4×)/PQ exist.

- **V6 cross-encoder rerank stage-2 seam** NOT built.

- **V8 named / multi-vector per dict id** NOT built; store holds exactly one f32 vector per
  id.

- **V3 GNCE KGE-for-cardinality planner hook** proposed but NOT wired to `cs-planner` (KGE
  trainer exists, not connected).

- **V9 RDF-native provenance-carrying GraphRAG retriever** is a composition, not a built unit.

- **SERVICE-form and standard-function-form vector search**: `research/genai-kg-embeddings-
  vectorindex.md:351–352` lists a `sparq:nearest` predicate OR a SERVICE/function as
  alternatives; only the magic-predicate path was built.

- **Embedding model-id / model-version / metric provenance in the store** NOT recorded in any
  `.spqv`/`.spqg`/`.spqd` header (only dim + graph fingerprint); the provider model string
  (`embed::ProviderConfig.model`, `DEFAULT_MODEL embed.rs:168`) is runtime-only, never
  persisted.

- **Calibrated confidence for `qualify`/`citations`** deferred; band reflects ASSERTED
  assurance, no reliability-diagram measurement.

- **Provenance-weighting adoption** gated behind on/off ablation with an explicit adopt-only-
  on-measured-lift-else-ABANDON bar; lift currently UNPROVEN (`provenance.rs:34–43`).

- **Grammar/logit-constrained NL-to-SPARQL decoding** NOT implemented; only a post-hoc N2
  dictionary constraint exists.

---

## Candidate Normative Surface

1. **Vocabulary**: `vec:` NS IRI is `http://sparq.dev/vec#` (`lib.rs:114`); recognised
   predicate IRIs are `http://sparq.dev/vec#nearest` and `http://sparq.dev/vec#search`
   (`lib.rs:119,122`).

2. **Pattern**: `?node vec:nearest ( query k )` MUST bind `?node` to the k nearest neighbours
   best-first; `( ?node ?score ) vec:search ( query k )` MUST additionally bind `?score` to
   the cosine similarity as an `xsd:double` (`rewrite.rs:38–52`).

3. **Argument grammar**: `( ... )` are ordinary SPARQL RDF collections (`rdf:first`/`rdf:rest`);
   the object list MUST be exactly `( query k )` and the `vec:search` subject exactly
   `( ?node ?score )` (`rewrite.rs:566–596`).

4. **Typing**: `query` MUST be a constant node IRI (its stored vector, seed self-excluded) OR a
   comma-separated float literal whose dimension MUST equal store dim; `k` MUST be a
   non-negative integer literal constant (`rewrite.rs:54–64,599–638`, dim-check `719–725`).

5. **MUST-fail (hard query error) obligations**: unknown `vec:` IRI (`rewrite.rs:407–411`);
   neighbour position not a variable (`require_var 641–646`); non-constant query/k; wrong
   argument arity (`589–595`); query-vector dim not equal store dim (`720–725`); malformed/
   dangling/cyclic argument list (`ListCells::elements 523–550`).

6. **Result ordering**: VALUES rows carry no order through joins; a consumer MUST `ORDER BY
   DESC(?score)` over a `vec:search` score to recover best-first (`rewrite.rs:76–79`).

7. **Semantics**: an absent/unembedded seed IRI MUST yield zero rows (`rewrite.rs:687–693`);
   an all-zero query MUST yield no results; the exact scan is the answer-exact default,
   approximate backends MUST be flagged recall under 1.0.

8. **Formats (little-endian)**: `.spqv` v2 magic SPQV, version u32=2, dim u32, count u64, 12B
   reserved, 24B fingerprint (dict_len, triple_count, content_hash u64), dense f32, sorted
   (id u32, slot u32) index (`store.rs:1–17,62–70`); siblings `.spqg` SPQG, `.spqd` SPQD,
   `.spqs` SPQS (`encode.rs:552`).

9. **Staleness contract**: fingerprint = dict_len + triple_count + content_hash over the dict
   term SET in dict-id-order-INDEPENDENT order, thread-count-stable; `check_graph` is
   NECESSARY-but-NOT-SUFFICIENT; to serve a persisted store the graph MUST be `Graph::save`/
   `Graph::open` round-tripped (frozen id order), NEVER re-parsed (`store.rs:26–36`).

10. **Distance metric**: cosine in −1..1 L2-normalised (cos = 1 − d²/2) is the single implicit
    mainline metric; the structure-feature `.spqs` `SchemaHeader` records a per-block `Metric`
    in `{Euclidean, NonEuclidean}` with a guard that MUST reject a Euclidean/cosine distance
    on a NonEuclidean block (`encode.rs:437–447`).

11. **Embedder contract**: MUST return exactly one finite `dim()`-length vector per input, in
    input order; a live provider response MUST be rejected on wrong count/dim/non-finite/
    duplicate-index/missing-index/out-of-range index (`embed.rs:10–15,401–453`).

12. **Provider protocol**: OpenAI-compatible `POST <endpoint>` (JSON body model+input, Bearer
    auth, honouring response index); the endpoint is the full URL in `SPARQ_EMBEDDINGS_BASE_URL`
    — POSTed as-is, nothing appended — env vars `SPARQ_EMBEDDINGS_API_KEY` (required) /
    `SPARQ_EMBEDDINGS_BASE_URL` / `SPARQ_EMBEDDINGS_MODEL` with defaults
    `https://api.openai.com/v1/embeddings` and `text-embedding-3-small` (`embed.rs:163–253`).

13. **Provenance-weight vocabulary**: `pkg:confidence` (`https://sparq.dev/ns/pkg#confidence`),
    `pkg:assurance` (objects `secx:Proven/Claimed/Conjectured`), `prov:wasDerivedFrom`,
    `pkg:discoveredFrom`; `w(t) = clamp(assurance_mult*confidence*source_reliability, floor,
    1.0)`, fail-open 1.0 (`provenance.rs:30–67`).

14. **Filtered-ANN answer-safety obligation**: the derived `IdMask` is the connected-component
    projection of the neighbour variable; filtered top-k MUST equal post-filtering the
    unfiltered top-k by that transitive constraint; every returned id MUST be in the mask;
    empty mask yields no results (`rewrite.rs:740–767`).

---

## Gaps (Spec Must Address)

1. **No embedding-model provenance**: the store cannot detect that two `.spqv` files came from
   different embedding models or dimensions; a spec should mandate recording model-id +
   model-version + metric + normalisation and MUST-reject a query vector from an incompatible
   model.

2. **Metric is unspecified in the mainline query surface**: cosine is hardcoded; there is no
   way to request Euclidean/dot via `vec:`.

3. **Query argument is constant-only**: per-row/correlated query vector or a variable seed is
   forbidden at bind-time (`rewrite.rs:66–79`); a WG spec must decide how a variable-driven /
   lateral k-NN is expressed.

4. **No standard SERVICE or extension-function form**; only a magic predicate; no alignment to
   vendor idioms (GraphDB `similarity:`, Neo4j `db.index.vector.queryNodes`, Stardog Voicebox,
   pgvector operators).

5. **Scores exposed only as an unordered VALUES column** via `vec:search`; no defined tie-break/
   determinism contract, no k=0/negative normative text.

6. **No named/multi-vector model** (one vector per id); no in-query hybrid/rerank surface
   (V4/V6/V8).

7. **Approximate backends recall under 1.0 = non-answer-exact**; a spec must separate
   answer-serving (exact) from retrieval/estimate (approximate) as a normative mode.

8. **KGE / structure-aware / provenance-weight features carry NO accuracy claim** and are gated
   on unmeasured lift; no pinned accuracy floor is citable.

9. **All measured numbers (recall, A/B, token) are work-box NON-CANONICAL**; no canonical
   benchmark authority exists.

---

## Honesty Flags

- **Approximate search** (HNSW, DiskANN/Vamana, PQ, `query_vec_approx`) has recall under 1.0;
  only `nearest_exact`/`ExactBackend`/`query_vec` are answer-exact; the measured over-fetch
  floor 0.90 at 10 and recall gates are explicitly NON-CANONICAL.

- **`HashEmbedder` is TEST-ONLY** and captures NO semantics (car and automobile unrelated);
  MUST NEVER be shipped as a retrieval feature.

- **KGE trainer makes NO accuracy claim, INDICATIVE only**; DistMult is symmetric so near-
  random on directional data.

- **Provenance-weighting makes NO accuracy claim**; literature reports inconsistent gains;
  ships behind an on/off ablation with an adopt-only-on-measured-lift-else-ABANDON bar
  (`provenance.rs:34–43`).

- **Compose K4 real-model A/B win is CONDITIONAL** (rescues opaque-id bindings, roughly zero
  benefit where the IRI slug is already informative) and is NOT a token-saver; numbers are
  work-box NON-CANONICAL.

- **`qualify`/`citations` confidence band REFLECTS ASSERTED assurance**, NOT a calibrated
  confidence; no reliability measurement exists.

- **`sparq_nlq::eval` exec-accuracy scores in CI** come only from scripted/recorded
  (`ReplayLlm`) completions, validating the harness/mechanism, NOT a real model's accuracy.

- **Recall is anchored against hnswlib** via a committed capture (`tests/ref_lib_verify.rs`)
  but the recall figures are NON-CANONICAL and the live re-capture is `#[ignore]`d.

- **Vendor-precedent claims** (GraphDB `similarity:`, Stardog Voicebox, Neptune, Neo4j,
  Qdrant/Weaviate/pgvector/Milvus) are catalogued from external docs in `research/` and tagged
  `established`/`secondary`, not sparq measurements.

---

> **Empirical-honesty reminder**: ZK and MPC estates are NOT production-sound until the
> external cryptographer audit sq-qhy4 completes. All work-box benchmarks are non-canonical;
> do not hard-code them in documentation or tests.

---

*Recon captured by Sonnet 4.6 under the Fable program; [SONNET-4.6]*
