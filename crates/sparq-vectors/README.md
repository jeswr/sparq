# sparq-vectors

Opt-in **embedding storage + nearest-neighbour search** for the
[sparq](https://github.com/jeswr/sparq) RDF engine (GenAI phase 4):

- **`VectorStore`** — one f32 embedding per dictionary term id in a flat,
  memory-mapped `.spqv` file. Coverage is **sparse by design** (only entities
  get embeddings, not every literal): vectors are stored densely in insertion
  slots and a trailing sorted `(id, slot)` index maps term ids to slots, so a
  `get` on an mmap'd store is one binary search plus a contiguous `&[f32]` —
  no per-vector allocation. `open` eagerly reads only the header and the
  index section (validated up front with checked arithmetic, so a corrupt
  file is rejected rather than panicking later); the vector data — the bulk
  of the file — is paged in by the OS on access. All-zero vectors are
  rejected at `put` (cosine is undefined on them), and an all-zero *query*
  returns no results from both searchers, so exact and HNSW agree even on
  degenerate inputs. `open_from_bytes` opens a store held entirely in memory
  (filesystem-less environments), with identical validation.
- **`StreamingWriter`** — builds stores **bigger than RAM**: each `put`
  appends the vector straight to the file's data section and spills the id to
  a sidecar, so build-phase memory is O(1) (`finalize` transiently holds the
  8-byte-per-vector index to sort it — 192× less than the 384-d f32 data the
  in-RAM builder would hold). Output is byte-identical to
  `create`/`put`/`finalize`; duplicate ids are reported at `finalize` (the
  sort reveals them) instead of at `put`.
- **`nearest_exact` / `VectorIndex`** — exact brute-force cosine top-`k` (the
  ground-truth baseline) and an HNSW approximate index
  ([`instant-distance`](https://crates.io/crates/instant-distance), pure
  Rust). The HNSW index stores L2-normalized copies and searches Euclidean,
  which is rank-equivalent to cosine on unit vectors; scores are converted
  back to cosine so both searchers are directly comparable.
  `nearest_term` / `nearest_term_exact` query by `oxrdf::Term`, resolving
  through a `Graph`'s dictionary and excluding the query term itself.
- **`Embedder`** — provider-agnostic embedding trait. Embeddings are produced
  **outside** the engine (design decision: out-of-process). `HashEmbedder` is
  the deterministic, offline, **test-only** implementation (lexical n-gram
  hashing — no semantics); the non-default `provider` feature carries the API
  shape for an OpenAI-compatible `/v1/embeddings` endpoint with a
  **caller-supplied** HTTP transport (this crate never opens a socket).
- **`verbalize` / `embed_entities`** — the entity **verbalization layer**:
  renders one text passage per entity from its literal properties (label +
  type + description + extra prefixed literals, multilingual-aware,
  char-budgeted) per a configurable `EntityTextConfig`, then embeds it. This
  is how production KG/vector systems build entity vectors (Wikidata's vector
  database, BLINK entity linking, GraphRAG, Weaviate's text2vec — sources in
  [`research/genai-text-embedding-practices.md`](../../research/genai-text-embedding-practices.md)).
- **`embed_labels`** — the label-only special case: one human-readable label
  per entity (`rdfs:label` > `skos:prefLabel` > `foaf:name` > `schema:name`
  > `dc:title`, configurable via `LabelConfig`), scanning each predicate's
  contiguous index block rather than the whole graph.
- **`fuse_rrf` / `fuse_rrf_weighted` / `fuse_scores`** — rank/score **fusion
  for hybrid search**: combine the text-vector ranking with another ranked
  signal (typically [`sparq-sim`](../sparq-sim)'s structural similarity) with
  reciprocal-rank fusion (optionally per-list weighted, Elasticsearch-style —
  down-weight a noisier signal without dropping it) or a normalized
  alpha-blend.

This is a **separate crate** by design: nothing in the workspace depends on
it (the wasm build in particular is untouched), and the default engine build
does not even compile it. It consumes only sparq-core's public read API.

## `.spqv` file format (version 1, little-endian)

```text
offset 0   magic        b"SPQV"                          4 bytes
offset 4   version      u32 = 1                          4 bytes
offset 8   dim          u32                              4 bytes
offset 12  count        u64                              8 bytes
offset 20  reserved     zero padding                     12 bytes
offset 32  data         [count × dim] f32, dense         count·dim·4 bytes
offset 32+count·dim·4   id→slot index                    count·8 bytes
           (id: u32, slot: u32) pairs sorted by id ascending
```

Both sections start at multiples of 4 from the page-aligned map, so the f32
casts are always aligned. Big-endian targets are rejected at `create`/`open`.

## How to: index entity labels

The minimal pipeline — one vector per labeled entity, embedded from the label
text alone. Enough for lexical lookup ("find me things called roughly X"):

```rust
use sparq_core::Graph;
use sparq_vectors::{embed_labels, HashEmbedder, VectorIndex, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let embedder = HashEmbedder::new(64); // test-only; bring your own Embedder
let mut store = VectorStore::create("graph.spqv", 64)?;
embed_labels(&graph, &mut store, &embedder)?; // rdfs:label > skos:prefLabel > …
store.finalize()?; // writes the file, handle becomes mmap-backed

let index = VectorIndex::build(&store); // HNSW, rebuilt per process
let neighbours = index.nearest_term(&some_term, &graph, &store, 10);
```

Use `embed_labels_with(&graph, &mut store, &embedder, &LabelConfig { .. })`
to change the label predicates or batch size. Label-only vectors cannot tell
two "John Smith"s apart — when the graph has descriptions, types, or other
text worth embedding, index verbalized entities instead.

## How to: index verbalized entities (labels + descriptions + types)

`embed_entities` embeds a **text passage** per entity, the way production KG
systems do (Wikidata vector DB: label + description + statements; BLINK:
`title [SEP] description`; GraphRAG: `name + description` — see
[`research/genai-text-embedding-practices.md`](../../research/genai-text-embedding-practices.md)).
The default `EntityTextConfig` renders **`<label>. a <type>. <description>`**:
the first label by predicate priority and language preference, the type as a
*word* (the type IRI's own label, falling back to its local name), and the
first description-like literal:

```rust
use sparq_core::Graph;
use sparq_vectors::{embed_entities, verbalize, EntityTextConfig, HashEmbedder, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let cfg = EntityTextConfig::default(); // label. a type. description

// Eyeball what would be embedded BEFORE paying for a model:
println!("{:?}", verbalize(&graph, &bolt, &cfg));
// Some("Usain Bolt. a athlete. Jamaican sprinter, eight-time Olympic champion.")

let embedder = HashEmbedder::new(64); // test-only; bring your own Embedder
let mut store = VectorStore::create("graph.spqv", 64)?;
let n = embed_entities(&graph, &mut store, &embedder, &cfg)?;
store.finalize()?;
```

Tailor the template per dataset:

```rust
use sparq_vectors::{EntityTextConfig, PropertyGroup};
use oxrdf::NamedNode;

let mut cfg = EntityTextConfig::default();
// Multilingual graphs: prefer en, then plain literals; anything else is a
// last resort ("en" also matches en-GB and the RDF 1.2 `en--ltr` form).
cfg.languages = vec!["en".into(), "".into()];
// Domain literals worth embedding: short, categorical, human-readable
// values, each with a property-name prefix (the Weaviate convention).
cfg.groups.push(
    PropertyGroup::literal(vec![NamedNode::new("http://example.org/occupation")?])
        .with_prefix("occupation: ")
        .with_max_values(3),
);
cfg.max_chars = 1024; // char budget; the leading label always survives
```

Rules of thumb (research-backed, sources in the research doc):

- **One value per slot by predicate priority** — `rdfs:label` beats
  `skos:prefLabel` for the same entity; groups are templates, not bags.
- **Include a description if the graph has one** — it is the disambiguator
  every production system embeds (labels alone are too ambiguous).
- **Keep raw numbers and dates OUT** of the text — embedding models do not
  order numbers; leave them to structured filters and verbalize only
  categorical values, with a prefix.
- Entities with **no literal text are skipped** (a bare "a athlete" passage
  matches every athlete and nothing else); `verbalize` returning `None`
  means `embed_entities` will skip that entity.

## How to: hybrid search (text vectors × structural similarity)

Text vectors know what things *mean*; [`sparq-sim`](../sparq-sim)'s
structural similarity knows how things are *connected*. Fuse the two ranked
lists — neither crate depends on the other; the fusion helpers take plain
`(item, score)` lists:

```rust
use sparq_sim::Sim;
use sparq_vectors::{fuse_rrf, fuse_scores, VectorIndex, RRF_K};

// Signal 1: text-vector neighbours (cosine, best first).
let text: Vec<(Term, f32)> = index.nearest_term(&query, &graph, &store, 50);
let text: Vec<(Term, f64)> = text.into_iter().map(|(t, s)| (t, s as f64)).collect();

// Signal 2: structural neighbours (weighted Jaccard, best first).
let sim = Sim::new(&graph);
let structural: Vec<(Term, f64)> = sim.most_similar(&query, 50);

// Default: Reciprocal Rank Fusion — rank-based, no normalization, nothing
// to tune. k = 60 is the industry-standard constant.
let hybrid = fuse_rrf(&[&text, &structural], RRF_K, 10);

// Tunable: min-max normalize each list, then alpha-blend.
// alpha = 1.0 → text only; 0.0 → structure only.
let blended = fuse_scores(&text, &structural, 0.7, 10);
```

Recipe notes:

- **Over-fetch each signal** (k = 50 for a top-10 fusion): RRF rewards items
  that appear in *both* lists, so give it overlap to work with.
- **`fuse_rrf` is the right default** when score scales differ (cosine in
  [-1, 1] vs Jaccard in [0, 1]) — it only uses ranks. Use `fuse_scores` when
  you want magnitudes to matter (a runaway top hit stays a runaway) and are
  willing to tune `alpha`.
- Both functions are deterministic: ties break by first appearance.

## Accuracy gate

`tests/recall.rs` asserts **HNSW recall@10 ≥ 0.95 vs exact brute force** on
50 000 random 32-d vectors (100 queries), as an ordinary `cargo test` (the
workspace raises the dev opt-level for this crate so the gate stays fast). The
measured recall with the default `HnswConfig` (`ef_search = ef_construction = 100`,
fixed seed) prints from the test — run it for the current figure.

## Throughput

`tests/throughput.rs` measures `put` / `finalize` / `open` / HNSW build / exact
vs HNSW top-10 query rates on an Apple M1, 50 000 × 32-d vectors, k = 10, 200
queries. Run it for the numbers (they are machine- and load-dependent):

```sh
cargo test -p sparq-vectors --release --test throughput -- --ignored --nocapture
```

Below ~10⁵ vectors the exact scan is a fine default; the HNSW index pays for
its build once query volume is high. The HNSW index is rebuilt from the mmap'd
store per process; for a **persistent** index that survives restart without a
rebuild, use `DiskAnnIndex` (below).

## Persistent on-disk ANN (`DiskAnnIndex`)

`DiskAnnIndex` (`src/diskann.rs`) is a **Vamana / DiskANN-style on-disk graph**
(`.spqg`): build it once, then `open` it on later runs with no rebuild — just an
mmap and a header check. It is a second, self-contained index (the
`instant-distance` HNSW is a closed graph whose adjacency can't be persisted),
with cosine semantics identical to the HNSW/exact searchers.

```rust
use sparq_vectors::{DiskAnnIndex, VectorStore};
let store = VectorStore::open("entities.spqv")?;
// Build + persist once (writes entities.spqg):
let _ = DiskAnnIndex::build(&store, "entities.spqg")?;
// On every later run — no rebuild, near-instant:
let index = DiskAnnIndex::open("entities.spqg")?;
let hits = index.nearest(&query, 10); // Vec<(Id, cosine)>, best first
```

Each on-disk node record co-locates the node's id, its (normalized) vector, and
its ≤ `R` neighbour slots, so one greedy-search hop is one contiguous page read.
Recall@10 vs exact brute force on the 50k×32 synthetic set is asserted by
`tests/diskann.rs` (run it for the figure), and a reopened handle returns
byte-identical neighbours to the freshly built one.
*Scope note:* full-precision vectors are searched straight from the mmap; the
PQ-compressed in-RAM candidate cache that *full* DiskANN ranks on now exists as
a standalone, tested layer (`src/quant.rs`, below) but is **not yet wired into
`search_slots`** — that integration is tracked as bead `sq-qamd`. <!-- [OPUS-4.8] -->
The
Vamana build is single-threaded and a little slower than the rayon HNSW build,
but the **open** is what this buys you: no per-process rebuild.

## Quantization for large stores (`ScalarQuantizer` / `ProductQuantizer`)

`src/quant.rs` shrinks per-vector memory so 100M-scale stores fit a candidate
cache in RAM, at a measured cost in recall. Both encoders work over the same
**L2-normalized / squared-Euclidean** cosine convention as the searchers
(`cos = 1 − d²/2`), so a ranking over quantized codes is directly comparable to
the exact/HNSW/DiskANN results.

- **Scalar quantization (SQ)** — `ScalarQuantizer`: each dimension is linearly
  mapped from its learned `[min, max]` onto 256 levels (`f32 → u8`, **4×**
  smaller); `reconstruct` inverts it for re-ranking. Worst per-component error
  is half a quantization step (measured 0.00096 on normalized random vectors),
  reconstruction cosine > 0.99.
- **Product quantization (PQ)** — `ProductQuantizer`: the vector is split into
  `M` subspaces, a k-means codebook of `K` centroids is learned per subspace,
  and a vector becomes its `M` nearest-centroid ids (`M` bytes; default `M=16`,
  `K=256`). Distance to a full-precision query uses **asymmetric distance
  computation** (`DistanceTable`): precompute the query↔centroid distances per
  subspace, then a code's estimated distance is `M` table look-ups, no decode.

```rust
use sparq_vectors::{DistanceTable, ProductQuantizer, PqConfig, VectorStore};
let store = VectorStore::open("entities.spqv")?;
let pq = ProductQuantizer::fit(store.dim(), store.iter().map(|(_, v)| v), PqConfig::default())?;
let cache = pq.encode_store(&store)?;          // count × M bytes, resident in RAM
let table = DistanceTable::new(&pq, &query);   // per-subspace query↔centroid table
let candidates = cache.rank_pq(&table, 50);    // RAM-only coarse ranking → Vec<(Id, cosine)>
// …then re-rank `candidates` against full-precision vectors for the final top-k.
```

Recall@10 at 8× compression (`M=16`) is measured by the crate's quantization
tests for two rankings — PQ-alone (a coarse filter) and PQ candidate filter +
full-precision re-rank (which recovers near-exact recall, the reason DiskANN
re-ranks the beam on disk); run the tests for the figures. Compression trades
against PQ-alone recall (higher `M` / lower compression lifts it). `encode_store`
produces the `EncodedStore` in-RAM cache that `DiskAnnIndex` would rank on; the
wiring into `search_slots` is tracked in beads.

## Tests

- `tests/store.rs` — `.spqv` round-trip, sparse/unsorted ids, build-phase
  error paths, garbage/truncated/overflowing-header/corrupt-index rejection,
  streaming-writer byte-identity with the in-RAM builder (+ sidecar cleanup,
  duplicate-at-finalize detection), and `open_from_bytes` parity/validation.
- `tests/recall.rs` — the recall@10 ≥ 0.95 gate plus exact/HNSW agreement on
  separable clusters.
- `tests/diskann.rs` — the on-disk `DiskAnnIndex`: recall@10 ≥ 0.90 vs exact
  brute force on 50k×32, **reopen-without-rebuild returns byte-identical
  neighbours** (the persistence acceptance gate), parity with the in-RAM HNSW
  on separable clusters, empty/singleton round-trip, and corrupt/wrong-magic
  rejection.
- `tests/quant.rs` — quantization gates: SQ round-trip half-step error bound +
  reconstruction cosine, PQ encode/decode idempotence, the ADC ==
  reconstructed-distance identity, uneven-subspace coverage, config/degenerate
  validation, tie-break determinism, and the headline **PQ recall@10 vs exact**
  on a 10k clustered set (PQ-alone floor + the PQ-filter-plus-re-rank ≥ 0.95
  DiskANN-loop gate).
- `tests/labels.rs` — `embed_labels` predicate priority, literal filtering,
  and term-keyed lookup end to end on a small graph.
- `tests/verbalize.rs` — `verbalize`/`embed_entities` on a fixture graph:
  template shape, language preference + fallback, predicate priority, type
  naming (label vs local name), prefixes and value caps, char-budget
  truncation, type-only skip, embed coverage/determinism/dim mismatch.
- `src/fuse.rs` unit tests — hand-computed RRF scores (plain and weighted,
  incl. unit-weight equivalence and zero-weight muting), rank-only
  invariance, min-max blend, tie-break determinism, input validation.
- `tests/olympics.rs` — the real 1.78M-triple olympics dataset (137k labels):
  embed → finalize → HNSW → `nearest_term`, with a same-type sanity check on
  the neighbours. **Skips (passes with a stderr note) when
  `bench/qlever-olympics/olympics.nt` is absent**; override the path with
  `SPARQ_OLYMPICS_NT`.
- `tests/throughput.rs` — the `#[ignore]`d measurement behind the table above.
