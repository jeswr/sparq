---
name: vector-search
description: "Semantic / ANN vector search over a sparq RDF graph: build a memory-mapped per-term-id embedding store (.spqv), then run cosine top-k with an in-RAM HNSW, a persistent on-disk DiskANN/Vamana graph (.spqg), or an exact brute-force baseline; verbalize entities (label+type+description) for embedding, scalar/product quantize (SQ/PQ) for large stores, fuse with another ranked signal (RRF / score blend) for hybrid retrieval, run predicate-constrained (filtered) ANN over a BGP-selected dict-id mask behind the opt-in `filtered-ann` feature, and — behind the opt-in `vec-predicate` feature — run k-NN INSIDE plain SPARQL via the `vec:nearest` / `vec:search` magic predicates. Use when adding embedding/semantic-search/nearest-neighbour over a sparq Graph in the sparq-vectors crate."
---

# sparq-vector-search

`sparq-vectors` is an **opt-in** crate that adds embedding storage + nearest-neighbour
(ANN) search to the sparq RDF engine. You store one f32 embedding per dictionary **term
id** in a flat memory-mapped `.spqv` file, then query top-`k` by **cosine** with an exact
scan, an in-RAM HNSW index, or a persistent on-disk DiskANN/Vamana graph — all
cosine-identical so their scores are directly comparable. Embeddings are produced
**out-of-process** (you supply the `Embedder`); the crate never runs a model and the
default engine build does not even compile it.

## Quickstart

`crates/sparq-vectors/Cargo.toml` (it consumes `sparq-core`; no features needed for the
core flow):

```toml
[dependencies]
sparq-core = { path = "../sparq-core" }
sparq-vectors = { path = "../sparq-vectors" }
oxrdf = "*"   # for oxrdf::{NamedNode, Term} in term-keyed queries
```

Embed entity labels, finalize the store, query the nearest neighbours of a term:

```rust
use sparq_core::Graph;
use sparq_vectors::{embed_labels, nearest_term_exact, HashEmbedder, VectorStore};
use oxrdf::{NamedNode, Term};

# fn main() -> Result<(), String> {
let ttl = r#"
  @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
  @prefix ex:   <http://example.org/> .
  ex:bolt  rdfs:label "Usain Bolt" ; a ex:Athlete .
  ex:bolt2 rdfs:label "Usain Bolt Junior" ; a ex:Athlete .
"#;
let graph = Graph::load_str(ttl, "turtle")?;

let embedder = HashEmbedder::new(64);             // TEST-ONLY embedder (see Gotchas)
let mut store = VectorStore::create("graph.spqv", 64)?; // dim must match embedder.dim()
embed_labels(&graph, &mut store, &embedder)?;     // rdfs:label > skos:prefLabel > foaf:name > schema:name > dc:title
store.finalize()?;                                // writes the file, handle becomes mmap-backed

// Exact brute-force search by term (default build — no third-party ANN dep):
let bolt = Term::NamedNode(NamedNode::new("http://example.org/bolt").map_err(|e| e.to_string())?);
let neighbours: Vec<(Term, f32)> = nearest_term_exact(&store, &graph, &bolt, 10); // (Term, cosine), best first, query term excluded
// For the APPROXIMATE in-RAM HNSW at scale, enable the opt-in `approx-ann` feature and build a
// `VectorIndex` (recall < 1.0; see "Exact vs HNSW" below) — gated so the heavy crate stays out of
// the default build.
# Ok(()) }
```

## Key APIs

```rust
// --- store (src/store.rs) ---  Id = sparq_core::dict::Id (= u32)
VectorStore::create<P: Into<PathBuf>>(path, dim: usize) -> Result<VectorStore, String>
VectorStore::open<P: AsRef<Path>>(path) -> Result<VectorStore, String>          // mmap; validates header+index up front
VectorStore::open_from_bytes(bytes: Vec<u8>) -> Result<VectorStore, String>     // filesystem-less
impl VectorStore {
    fn put(&mut self, id: Id, vector: &[f32]) -> Result<(), String>             // build phase only; rejects all-zero / non-finite / dup id / dim mismatch
    fn finalize(&mut self) -> Result<(), String>                                // write + reopen mmap; idempotent
    fn get(&self, id: Id) -> Option<&[f32]>;  fn iter(&self) -> impl Iterator<Item=(Id, &[f32])>
    fn dim(&self) -> usize;  fn len(&self) -> usize;  fn is_empty(&self) -> bool
}
StreamingWriter::create(path, dim);  fn put(&mut self, id, &[f32]); fn finalize(self) -> Result<VectorStore, String>  // O(1) build RAM, byte-identical output

// --- bulk import of externally-computed embeddings (src/import.rs) --- bring-your-own vectors; reuses the store writer
ImportSpec { spqv_path, dim, ids: &[Id], binding: ImportBinding }   // ids[i] keys row i (row -> dict-id contract); ids.len() = rows
ImportBinding::Unbound | ImportBinding::Graph(&Graph)              // graph binding for the fingerprint header (Unbound = unverifiable)
VectorStore::import_npy(ImportSpec, npy_path) -> Result<VectorStore, String>          // 2-D C-order little-endian f4/f8 .npy; f8 narrowed to f32
VectorStore::import_numeric_dump(ImportSpec, dump_path) -> Result<VectorStore, String> // header-less flat rows*dim*4 LE f32 (row-major)
// both: fail-closed on dim/row-count/dtype/order/length mismatch; .npy header bounded (sq-tzwa) before any body alloc; NOT on wasm (std::fs)

// --- embedders (src/embed.rs) ---
trait Embedder { fn dim(&self) -> usize; fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String>; }
HashEmbedder::new(dim) / ::with_seed(dim, seed)                                 // TEST-ONLY lexical hashing, NO semantics
// feature = "provider": provider::{Transport, ProviderConfig, RemoteEmbedder<T: Transport>}  (caller supplies HTTP)

// --- text → vectors (src/labels.rs, src/verbalize.rs) ---
embed_labels(&Graph, &mut VectorStore, &impl Embedder) -> Result<usize, String> // store left UNFINALIZED
embed_labels_with(&Graph, &mut VectorStore, &impl Embedder, &LabelConfig) -> Result<usize, String>
verbalize(&Graph, &Term, &EntityTextConfig) -> Option<String>                   // inspect the passage before embedding
embed_entities(&Graph, &mut VectorStore, &impl Embedder, &EntityTextConfig) -> Result<usize, String>
// EntityTextConfig{ groups: Vec<PropertyGroup>, languages, naming_predicates, separator, max_chars, batch }
// PropertyGroup::literal(preds) / ::entity_label(preds) .with_prefix("a ") .with_max_values(3); ObjectKind::{Literal, EntityLabel}

// --- graph-staleness guard (src/fingerprint.rs) --- store is keyed by dict id; a rebuild can shift ids
VectorStore::create(p, dim)?.with_fingerprint(&Graph)   // bind store to its graph; finalize embeds it in the .spqv header
StreamingWriter::create_with_fingerprint(p, dim, &Graph) // same for the streaming builder
store.fingerprint() -> Option<Fingerprint>;  store.check_graph(&Graph) -> Result<(), String>   // None / mismatch -> Err
// fingerprint = dict_len + triple_count + content_hash over the dict term SET in a dict-id-order-INDEPENDENT (sorted) order
//   (sq-xhiv: stable across RAYON_NUM_THREADS, so re-loading the same RDF at a different thread count does NOT spuriously
//   mismatch; a genuine term add/remove/edit still does); legacy v1 files open but check_graph errs
// ID-KEYED STALENESS CONTRACT (sq-wlzi): the store is keyed by RAW dict id, so a passing check_graph is NECESSARY but NOT
//   SUFFICIENT -- the thread-count-stable fingerprint ALSO passes a re-parse of the same RDF whose ids merely permuted, which
//   then serves the WRONG vector. To SERVE a persisted .spqv/.spqg: persist its graph (Graph::save) and reopen THAT graph
//   (Graph::open -- mmaps the FROZEN id order) to resolve query terms; NEVER re-parse the source RDF (Graph::load_str etc.).
//   (Graph::save/open need sparq-core's `mmap` feature.) Round-trip vs re-parse trap pinned in tests/staleness_contract.rs.

// --- incremental add/remove/update (src/delta.rs) --- feature = "delta" ONLY; LEAN, no new dep [OPUS-4.8] sq-pi44
// In-RAM DELTA SIDECAR over the immutable base: append map + tombstone set. No file rebuild; a single graph change no
// longer forces a full re-embed. get/iter/len (hence search) transparently union base+delta and honour tombstones.
store.add(id, &[f32]) -> Result<(), String>      // NEW id; errs if id already present (use update) or on the put validation
store.remove(id) -> bool                          // tombstone id (true if it had a vector); a removed id can be re-added
store.update(id, &[f32]) -> Result<(), String>   // replace an EXISTING id's vector; errs if absent (use add)
store.compact(out_path, &Graph) -> Result<VectorStore, String>  // fold delta into a FRESH base == a from-scratch rebuild
store.has_delta() -> bool;  store.delta() -> Option<&VectorDelta>;  store.take_delta() -> Option<VectorDelta>
store.apply_delta(VectorDelta) -> Result<(), String>  // GENERATION TIE: rejects a delta whose fingerprint != this base's
// --- PERSISTED on-disk delta sidecar (.spqd) --- [OPUS-4.8] sq-7e50; same `delta` feature, no new dep
store.save_delta() -> Result<PathBuf, String>          // persist the in-RAM delta to a sibling .spqd (tmp+fsync+rename)
store.save_delta_to(path) -> Result<(), String>        // …to an explicit path; an unstarted delta persists empty+gen-bound
VectorStore::open_with_delta(base) -> Result<VectorStore, String>   // open base mmap + replay the persisted sibling .spqd
VectorStore::open_with_delta_at(base, delta_path) -> Result<..>     // …from an explicit delta path (no sidecar ⇒ bare base)
VectorStore::sibling_delta_path(&Path) -> PathBuf;  VectorStore::has_persisted_delta(&Path) -> bool
// `.spqd` = little-endian header (magic SPQD, v1, dim, append/tomb counts, BASE fingerprint @ off 32) + appends + tombstones.
// CRASH-DURABLE (tmp+fsync+atomic rename, StreamingWriter discipline); PARTIAL-WRITE detected (exact-length check, fail-closed);
// open routes through apply_delta so a sidecar from a DIFFERENT graph generation is REJECTED, never silently mis-keyed.
// `put` is UNCHANGED (still errs after finalize) — `add` is the additive path; `compact` is the in-base durability path,
//   `save_delta`/`open_with_delta` the survive-a-restart-without-compact path. On a build-phase store add==put.

// --- search (src/ann.rs) --- all return cosine in [-1,1], best first; zero query -> empty
nearest_exact(&VectorStore, query: &[f32], k) -> Vec<(Id, f32)>                 // ground-truth full scan
nearest_term_exact(&VectorStore, &Graph, &Term, k) -> Vec<(Term, f32)>          // UNCHECKED: stale store -> silently wrong
nearest_term_exact_checked(&VectorStore, &Graph, &Term, k) -> Result<Vec<(Term, f32)>, String>  // errs on stale store
cosine(a: &[f32], b: &[f32]) -> f32
// HNSW = the APPROXIMATE backend: feature = "approx-ann" ONLY [OPUS-4.8] sq-ip3a (the ONLY thing
// pulling instant-distance; default build has NO third-party ANN dep). recall < 1.0 — NOT exact.
VectorIndex::build(&store) / ::build_with(&store, HnswConfig{ef_search, ef_construction, seed})
impl VectorIndex { fn nearest(&self, query: &[f32], k) -> Vec<(Id, f32)>;
                   fn nearest_term(&self, &Term, &Graph, &VectorStore, k) -> Vec<(Term, f32)>;
                   fn nearest_term_checked(..) -> Result<Vec<(Term, f32)>, String> }

// --- persistent on-disk ANN (src/diskann.rs) ---
DiskAnnIndex::build(&VectorStore, path) / ::build_with(&store, path, VamanaConfig{degree, build_beam, search_beam, alpha, seed})
DiskAnnIndex::build_for(&store, path, &Graph) / ::build_with_for(&store, path, cfg, &Graph)  // embeds the graph fingerprint
DiskAnnIndex::build_with_pq(&store, path, cfg, PqConfig) -> Result<..>          // [OPUS-4.8] sq-qamd: + a PQ candidate cache (search on codes, re-rank off mmap); persisted as a trailing .spqg section (encoding tag 1)
DiskAnnIndex::open(path) -> Result<DiskAnnIndex, String>                        // mmap + header check, NO rebuild (reloads any PQ section)
impl DiskAnnIndex { fn nearest(&self, &[f32], k) -> Vec<(Id, f32)>; fn nearest_term(..) -> Vec<(Term, f32)>; fn len()/dim();
                    fn has_pq_cache() -> bool;                                  // [OPUS-4.8] sq-qamd: PQ-guided search when true
                    fn fingerprint() -> Option<Fingerprint>; fn check_graph(&store, &Graph) -> Result<(), String>;
                    fn nearest_term_checked(&Term, &Graph, &store, k) -> Result<Vec<(Term, f32)>, String> }
sibling_graph_path(&Path) -> PathBuf                                            // foo.spqv -> foo.spqg

// --- predicate-constrained (filtered) ANN (src/filter.rs) --- feature = "filtered-ann" ONLY; LEAN, no new dep [OPUS-4.8] sq-1wc1
IdMask::new() / ::from_ids(impl IntoIterator<Item=Id>) / FromIterator<Id>   // the BGP-selected "visit mask" of permitted dict-ids
impl IdMask { fn insert(&mut self, Id) -> &mut Self; fn contains(Id)->bool; fn len()/is_empty(); fn iter() -> impl Iterator<Item=Id> }
nearest_exact_filtered(&VectorStore, query: &[f32], &IdMask, k) -> Vec<(Id, f32)>   // EXACT ground truth = pre-filter strategy (scan only the mask)
FilterConfig { prefilter_fraction: f32 /*0.01*/, prefilter_floor: usize /*256*/, traversal_beam_factor: usize /*4*/ }
impl FilterConfig { fn prefer_prefilter(mask_len, store_len) -> bool }       // selectivity crossover: <= max(frac*store, floor) -> pre-filter
impl DiskAnnIndex { fn nearest_filtered(&self, &[f32], &IdMask, &VectorStore, k) -> Vec<(Id, f32)>     // ACORN/NaviX filtered traversal OR pre-filter, by selectivity
                    fn nearest_filtered_with(&self, &[f32], &IdMask, &VectorStore, k, FilterConfig) -> Vec<(Id, f32)> }
impl VectorIndex  { fn nearest_filtered(&self, &[f32], &IdMask, &VectorStore, k) -> Vec<(Id, f32)> }   // HNSW: exact pre-filter only (instant-distance adjacency not exposed)
// empty mask -> no results; full mask -> equals the unfiltered search; every returned id is guaranteed in the mask

// --- pre-filter vs post-filter cost model (src/cost.rs) --- feature = "filtered-ann" ONLY [OPUS-4.8] sq-7hx6 (subsumes sq-ic0n)
CostModel { scatter_penalty: f32 /*2.0*/ }                                       // scattered masked-row cost vs sequential full-scan row; >= 1.0
impl CostModel { fn decide(mask_len, store_len, k) -> CostEstimate }             // pre-filter iff mask_len * scatter_penalty <= store_len
Strategy::{PreFilter, PostFilter}                                                // the chosen branch (assert the decision)
CostEstimate { mask_len, store_len, k, prefilter_cost, postfilter_cost, strategy }   // the modelled estimate behind a decision
postfilter_exact(&VectorStore, query: &[f32], &IdMask, k) -> Vec<(Id, f32)>      // scan WHOLE store, drop non-masked -> IDENTICAL to nearest_exact_filtered (no over-fetch boundary: full ranking)
nearest_filtered_costed(&VectorStore, &[f32], &IdMask, k, &CostModel) -> (Vec<(Id, f32)>, CostEstimate)   // decide + run chosen branch
overfetch_target(k, mask_len, store_len) -> usize                               // ceil(k/selectivity) clamped; the FIRST fetch size for the iterative over-fetch path below (exact backend never under-fills, so it's a no-op there)
// HEURISTIC over an ESTIMATE, not optimal: scatter_penalty is one modelled constant; pre/post return the IDENTICAL top-k either way (answer-safe)

// --- pluggable ANN backend + iterative over-fetch FILTERED path (src/backend.rs) --- feature = "filtered-ann" [OPUS-4.8] sq-ip3a (follow-up to sq-7hx6)
trait AnnBackend { fn candidates(&self, query: &[f32], fetch) -> Vec<(Id, f32)>; fn len()/is_empty(); }  // ranked top-`fetch`, best-first, prefix-stable
ExactBackend::new(&VectorStore)                                                  // answer-EXACT (full scan); fetch>=len => complete ranking => recall 1.0; NO third-party dep
ApproxBackend::new(&DiskAnnIndex)                                                // feature = "approx-ann" ALSO; APPROXIMATE (bounded beam) => recall < 1.0 — NOT exact
nearest_filtered_overfetch(&impl AnnBackend, query, &IdMask, k, max_rounds) -> Vec<(Id, f32)>           // fetch overfetch_target, post-filter, if <k DOUBLE the fetch & retry up to backend size / max_rounds
nearest_filtered_overfetch_default(&backend, query, &IdMask, k) -> Vec<(Id, f32)>                       // = ..._overfetch(.., DEFAULT_MAX_ROUNDS=24)
// FILTERED approx under-fills a bounded list when admitted ids cluster below the prefix; iterative over-fetch FILLS k whenever k admitted vectors exist & the backend surfaces them.
// HONESTY: over-fetch fixes UNDER-FILL, NOT recall — ApproxBackend recall stays < 1.0 (measured floor 0.90@10 in tests/overfetch.rs, NOT exactness). Only ExactBackend is answer-exact.

// --- quantization for large stores (src/quant.rs) ---
ScalarQuantizer::fit(dim, vectors: impl IntoIterator<Item=&[f32]>) -> Result<ScalarQuantizer, String>   // f32->u8, 4x
ProductQuantizer::fit(dim, vectors, PqConfig{m, k, iters, seed}) -> Result<ProductQuantizer, String>    // M bytes/vec, 8-32x
ProductQuantizer::to_bytes() -> Vec<u8> / ::from_bytes(&[u8]) -> Result<ProductQuantizer, String>       // [OPUS-4.8] sq-qamd: persist/reload the codebook (e.g. in a .spqg PQ section)
impl {Scalar,Product}Quantizer { fn encode(&self, &[f32]) -> Vec<u8>; fn reconstruct(&self, &[u8]) -> Vec<f32>;
                                  fn encode_store(&self, &VectorStore) -> Result<EncodedStore, String> }
DistanceTable::new(&ProductQuantizer, query: &[f32]);  fn distance(&self, code)->f32; fn cosine(&self, code)->f32  // ADC
EncodedStore::rank_pq(&self, &DistanceTable, k) -> Vec<(Id, f32)>;  cosine_from_sq_dist(sq: f32) -> f32
EncodedStore::from_parts(ids, codes, stride) -> Result<EncodedStore, String> / ::codes() -> &[u8]       // [OPUS-4.8] sq-qamd: reload a persisted candidate cache

// --- hybrid fusion (src/fuse.rs) --- lists are (item, f64) best-first; deterministic ties
fuse_rrf(lists: &[&[(T, f64)]], k: f64 /*RRF_K=60.0*/, top_k) -> Vec<(T, f64)>
fuse_rrf_weighted(lists: &[(&[(T, f64)], f64)], k, top_k) -> Vec<(T, f64)>      // weight 0.0 mutes a list entirely
fuse_scores(a: &[(T,f64)], b: &[(T,f64)], alpha /*1.0=a only*/, top_k) -> Vec<(T, f64)>
// one-call hybrid: run N retriever closures on one query, fuse by item via RRF, dedup
hybrid_search(query: &Q, top_k, k /*RRF_K*/, &mut [Retriever<'_, Q, T>]) -> Vec<(T, f64)>   // [OPUS-4.8] lifetime on alias use
//   Retriever<'r, Q, T> = &'r mut dyn FnMut(&Q) -> Vec<(T, f64)>  (e.g. nearest_term / most_similar closures)

// --- `vec:` magic predicate (src/rewrite.rs) --- feature = "vec-predicate" ONLY; pulls sparq-engine [OPUS-4.8] sq-k6ex
query_vec(&Graph, sparql: &str, &VectorStore) -> Result<QueryResult, String>      // parse + rewrite + evaluate
query_vec_with_budget(&Graph, &str, &VectorStore, &QueryBudget) -> Result<QueryResult, String>
prepare_vec(&Graph, &str, &VectorStore) -> Result<PreparedQuery, String>          // compose with engine *_prepared entry points
rewrite_query(Query, &Graph, &VectorStore) -> Result<Query, String>              // spargebra-algebra rewrite only
// re-exported when the feature is on: query_prepared, PreparedQuery, QueryBudget, QueryResult (no direct sparq-engine dep needed)
// vocab: vec::{VEC_NS, NEAREST, SEARCH}  (http://sparq.dev/vec#)  — exact-scan (nearest_exact) KNN
// [OPUS-4.8] sq-z589: with `approx-ann` ALSO on, the *_approx twins take a &DiskAnnIndex and run the
//   UNFILTERED vec: k-NN through that Vamana index instead of the full scan (APPROXIMATE, recall < 1.0):
query_vec_approx(&Graph, &str, &VectorStore, &DiskAnnIndex) -> Result<QueryResult, String>   // feature = "vec-predicate" + "approx-ann"
query_vec_approx_with_budget(&Graph, &str, &VectorStore, &DiskAnnIndex, &QueryBudget) -> Result<QueryResult, String>
prepare_vec_approx(&Graph, &str, &VectorStore, &DiskAnnIndex) -> Result<PreparedQuery, String>
//   The FILTERED path is unchanged (still cost-model'd nearest_filtered_costed); approx seam = unfiltered scan only.
// [OPUS-4.8] sq-36ol: with `filtered-ann` ALSO on, the BGP→IdMask a constrained `vec:` neighbour
//   derives is CACHED across prepares, keyed by (constraining sub-BGP, graph Fingerprint). The
//   fingerprint folds dict_len + triple_count + a content hash over the dict term SET in a
//   dict-id-order-INDEPENDENT (sorted) order (sq-xhiv), so ANY genuine graph change misses the cache
//   and recomputes — a stale mask is never served (invalidation is SOUND; when in doubt it misses) —
//   while a thread-count-only dict-id permutation of an unchanged graph correctly HITS (same mask).
//   The cache is thread-local and transparent (no API change; same answers).
```

## Common recipes

### 1. Verbalize entities (label + type + description), then embed

Better than label-only when the graph has descriptions/types — the default
`EntityTextConfig` renders `"<label>. a <type>. <description>"`:

```rust
use sparq_core::Graph;
use sparq_vectors::{embed_entities, verbalize, EntityTextConfig, HashEmbedder, VectorStore};
use oxrdf::{NamedNode, Term};

let graph = Graph::load_str(ttl, "turtle")?;
let cfg = EntityTextConfig::default();

// Eyeball what would be embedded BEFORE paying a real model:
let bolt = Term::NamedNode(NamedNode::new("http://example.org/bolt").map_err(|e| e.to_string())?);
println!("{:?}", verbalize(&graph, &bolt, &cfg));   // Some("Usain Bolt. a athlete. Jamaican sprinter...")

let mut store = VectorStore::create("graph.spqv", 64)?;
let n = embed_entities(&graph, &mut store, &HashEmbedder::new(64), &cfg)?;  // entities with no literal text are skipped
store.finalize()?;
```

Tailor the template — add a categorical literal group, set the language chain:

```rust
use sparq_vectors::{EntityTextConfig, PropertyGroup};
use oxrdf::NamedNode;

let mut cfg = EntityTextConfig::default();
cfg.languages = vec!["en".into(), "".into()];   // "en" also matches en-GB and RDF 1.2 en--ltr; "" = untagged
cfg.groups.push(
    PropertyGroup::literal(vec![NamedNode::new("http://example.org/occupation").unwrap()])
        .with_prefix("occupation: ")            // Weaviate property-name prefix convention
        .with_max_values(3),
);
cfg.max_chars = 1024;                           // char budget; the leading label always survives
// Keep raw numbers/dates OUT of the text (models do not order numbers); verbalize only categorical values.
```

### 2. Exact vs HNSW (pick by scale)

HNSW (`VectorIndex`) is the APPROXIMATE backend behind the **opt-in `approx-ann` feature** — the
only thing pulling `instant-distance`, so the default build has NO third-party ANN dep (lean core).
It is approximate: recall < 1.0 (NOT answer-exact). Build with `--features approx-ann`.

```rust
# #[cfg(feature = "approx-ann")] {
use sparq_vectors::{nearest_exact, VectorIndex, HnswConfig};

let store = sparq_vectors::VectorStore::open("graph.spqv")?;

// Below ~10^5 vectors the exact scan is a fine default (no build cost, answer-exact):
let hits = nearest_exact(&store, query, 10);          // Vec<(Id, f32)>

// At higher query volume, build HNSW once (rayon-parallel inside instant-distance):
let index = VectorIndex::build_with(&store, HnswConfig { ef_search: 100, ef_construction: 100, seed: 0 });
let approx = index.nearest(query, 10);                // APPROXIMATE: ef_search must be >= k; recall@10 < 1.0 (run tests/recall.rs)
# }
```

### 3. Persistent on-disk index that survives process restart

```rust
use sparq_vectors::{DiskAnnIndex, VectorStore};

let store = VectorStore::open("entities.spqv")?;
let _ = DiskAnnIndex::build(&store, "entities.spqg")?;  // build + persist ONCE (single-threaded Vamana)
// every later run: no rebuild, near-instant (mmap + header check):
let index = DiskAnnIndex::open("entities.spqg")?;
let hits = index.nearest(query, 10);                    // Vec<(Id, cosine)>, recall@10 ~0.966
```

### 4. Quantize a large store, then PQ-filter + full-precision re-rank (the DiskANN loop)

PQ codes are a coarse RAM-resident *filter*, not a final ranking — re-rank the candidates
against the full-precision store. [OPUS-4.8] sq-qamd: `DiskAnnIndex::build_with_pq` now drives
this loop INSIDE the index (rank each visited node's neighbours on the in-RAM codes, re-rank the
final beam off the mmap — `nearest` reports the exact full-precision cosine), and persists the
codebook + codes as a trailing `.spqg` section so `open` reloads the cache with no rebuild:

```rust
use sparq_vectors::{DiskAnnIndex, PqConfig, VamanaConfig, VectorStore};
let store = VectorStore::open("entities.spqv")?;
let idx = DiskAnnIndex::build_with_pq(&store, "entities.spqg", VamanaConfig::default(), PqConfig::default())?;
assert!(idx.has_pq_cache());
let top10 = idx.nearest(query, 10);                    // PQ-guided traversal, exact re-ranked cosine
```

To drive the same coarse-filter-then-re-rank loop by hand (e.g. over the exact store with no graph):

```rust
use sparq_vectors::{cosine, DistanceTable, ProductQuantizer, PqConfig, VectorStore};

let store = VectorStore::open("entities.spqv")?;
let pq = ProductQuantizer::fit(store.dim(), store.iter().map(|(_, v)| v), PqConfig::default())?; // M=16,K=256 -> 16B codes, 8x at dim=32
let cache = pq.encode_store(&store)?;                  // count x M bytes, RAM-resident
let table = DistanceTable::new(&pq, query);            // asymmetric distance (query stays full-precision)

let candidates = cache.rank_pq(&table, 50);            // RAM-only coarse top-50  (PQ-alone recall ~0.60)
let mut rescored: Vec<(u32, f32)> = candidates
    .into_iter()
    .map(|(id, _)| (id, cosine(query, store.get(id).unwrap())))   // re-score on full precision
    .collect();
rescored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
let top10: Vec<_> = rescored.into_iter().take(10).collect();       // recall@10 ~0.98
```

`ScalarQuantizer` (4×, `f32→u8`) is the cheaper alternative; `reconstruct(code)` gives the
lossy preview for re-ranking. Both quantizers and all searchers share the L2-normalized
cosine convention (`cos = 1 − d²/2`).

### 5. Hybrid retrieval — fuse text vectors with another ranked signal

Neither crate depends on the other; the fusion helpers take plain `(item, score)` lists.
Over-fetch each signal (e.g. 50) for a top-10 fusion so RRF has overlap to reward.

```rust
use sparq_sim::Sim;                                    // structural similarity (separate crate)
use sparq_vectors::{fuse_rrf, fuse_scores, RRF_K};

let text: Vec<(oxrdf::Term, f64)> =
    index.nearest_term(&query, &graph, &store, 50).into_iter().map(|(t, s)| (t, s as f64)).collect();
let structural: Vec<(oxrdf::Term, f64)> = Sim::new(&graph).most_similar(&query, 50);

let hybrid  = fuse_rrf(&[&text, &structural], RRF_K, 10);   // rank-only; right default for differing score scales
let blended = fuse_scores(&text, &structural, 0.7, 10);     // min-max normalize then blend; alpha 1.0 = text only
```

`hybrid_search` packages the common RRF case — drive N retriever closures off one query
`Term` and fuse by `Term` in a single call (dedups; a term in only one list still surfaces;
RRF ignores the score scales). Widen `nearest_term`'s `f32` to `f64` so both lists share the
`(Term, f64)` shape:

```rust
use sparq_vectors::{hybrid_search, RRF_K};
let fused = hybrid_search(&query, 10, RRF_K, &mut [
    &mut |t: &oxrdf::Term| index.nearest_term(t, &graph, &store, 50)
        .into_iter().map(|(t, s)| (t, s as f64)).collect(),   // ANN (cosine)
    &mut |t: &oxrdf::Term| sparq_sim::Sim::new(&graph).most_similar(t, 50),  // structural (Jaccard) [OPUS-4.8] FQ path: block has no `use Sim`
]);
```

### 6. Out-of-RAM build / filesystem-less open

```rust
use sparq_vectors::{StreamingWriter, VectorStore};

let mut w = StreamingWriter::create("/tmp/big.spqv", 384)?;  // O(1) build memory; appends straight to disk
w.put(7, &[0.1; 384])?;                                      // duplicate ids reported at finalize, not here
let store = w.finalize()?;                                   // a normal, validated VectorStore

let bytes = std::fs::read("/tmp/big.spqv").unwrap();         // or fetched/embedded
let store2 = VectorStore::open_from_bytes(bytes)?;           // identical validation, no filesystem
```

### 7. Bulk-import embeddings computed ELSEWHERE (NumPy `.npy` / flat dump)

When vectors come from an external pipeline (Python/sentence-transformers, a vendor batch job)
rather than an in-process `Embedder`, load the matrix straight into a `.spqv`. The matrix carries
no term identity, so you supply a **parallel slice of dict ids** — row `i` is stored for `ids[i]`
(the **row → dict-id contract**). Emit `(id, text)` pairs from the same scan, embed the texts
out-of-process, then import the matrix with the ids in the same order.

```rust
use sparq_vectors::{ImportBinding, ImportSpec, VectorStore};

// row i of vecs.npy lines up with ids[i]; resolve each id with graph.id_of(&term).
let ids: Vec<u32> = vec![/* alice */ 10, /* bob */ 20, /* carol */ 30];

// (a) NumPy: numpy.save("vecs.npy", arr.astype(np.float32))  -> 2-D C-order <f4/<f8
let store = VectorStore::import_npy(
    ImportSpec { spqv_path: "graph.spqv", dim: 384, ids: &ids, binding: ImportBinding::Graph(&graph) },
    "vecs.npy",
)?;                       // finalized, mmap-backed; binding=Graph embeds the fingerprint

// (b) header-less dump: arr.astype("<f4").tofile("vecs.f32")  -> exactly rows*dim*4 LE bytes
let store = VectorStore::import_numeric_dump(
    ImportSpec { spqv_path: "graph.spqv", dim: 384, ids: &ids, binding: ImportBinding::Unbound },
    "vecs.f32",
)?;
```

Fail-closed: a dtype/byte-order/fortran-order/dimensionality mismatch, a `shape[1] != dim`, a
`shape[0] != ids.len()`, a wrong dump length, a duplicate/zero/non-finite row, or a
malformed/oversized `.npy` header is an `Err` — never a silent reinterpretation. The `.npy` header
length and declared shape are bounded against the actual file size before any body allocation
(sq-tzwa hardening). `import_*` need `std::fs` and are compiled off the wasm target.

### 8. k-NN INSIDE SPARQL — the `vec:` magic predicate (opt-in, feature = `vec-predicate`)

Express nearest-neighbour search in plain SPARQL, mirroring `sparq-text`'s `text:` predicate:
the rewrite runs the k-NN, inlines the hit nodes as a `VALUES` table at the spargebra-algebra
level, and evaluates through the engine's prepared-query seam — so the **engine, planner,
executor and wasm bundle are unchanged**. This is the **only** SPARQL-level hook, and it is
**OFF by default**: the feature is the only thing that pulls `sparq-engine` into this crate, so
without it `sparq-vectors` is a pure storage+ANN crate and the base query path carries zero
`vec:` code (`cargo tree -p sparq-vectors` lists no `sparq-engine`/`spargebra`).

```toml
sparq-vectors = { path = "../sparq-vectors", features = ["vec-predicate"] }
```

```rust
use sparq_vectors::{query_vec, VectorStore};   // query_vec only exists with the feature on
// store built/finalized as above, keyed by the SAME graph's dict ids.

// vec:nearest — bind ?node to the k nearest neighbours of a query.
//   query = a node IRI (its stored vector; the seed is excluded) OR a "f1,f2,…" vector literal.
let r = query_vec(&graph,
    "PREFIX vec: <http://sparq.dev/vec#>
     SELECT ?label WHERE {
       ?node vec:nearest ( <http://example.org/bolt> 5 ) .   # 5 neighbours of bolt
       ?node rdfs:label ?label .                              # join to ordinary triples
     }", &store)?;

// vec:search — also bind a cosine score; ORDER BY DESC recovers best-first (VALUES is unordered).
let r = query_vec(&graph,
    "PREFIX vec: <http://sparq.dev/vec#>
     SELECT ?node ?score WHERE {
       ( ?node ?score ) vec:search ( \"0.1,0.9,…\" 10 )
     } ORDER BY DESC(?score)", &store)?;
```

The argument lists `( … )` are ordinary SPARQL RDF collections (spargebra lowers them to
`rdf:first`/`rdf:rest`, which the rewrite walks). Hard query errors (not silent mismatches):
the neighbour position(s) must be variables; `query`/`k` must be constants; the object list must
be exactly `( query k )` and the `vec:search` subject exactly `( ?node ?score )`; a query-vector
literal's dimension must match the store; any other `vec:` IRI is unknown. An absent/unembedded
seed IRI yields no rows. By default the unfiltered search is the **exact** `nearest_exact` scan
(deterministic, answer-exact — a fine default below ~10⁵ vectors).

**Approximate `vec:` for large stores (opt-in, ALSO `approx-ann`, sq-z589).** With BOTH
`vec-predicate` *and* `approx-ann` on, `query_vec_approx` / `prepare_vec_approx` take an extra
on-disk `DiskAnnIndex` argument and run the **unfiltered** k-NN through that Vamana index instead
of the full scan — for large `.spqv` stores where brute force is the bottleneck. Everything else
(parse, rewrite, VALUES inlining, joins, error checks, seed-self-exclusion) is identical to
`query_vec`. **APPROXIMATE: recall < 1.0** — the index can miss a true neighbour; use `query_vec`
(exact) when answer-exactness matters. Pass an index built with `DiskAnnIndex::build_for(&store,
path, &graph)` (and a store bound via `.with_fingerprint(&graph)`) so the staleness guard catches a
stale index. The **filtered** path (below) is unaffected — it still uses the cost-model'd filtered
search; the approximate seam is only the unfiltered scan.

```rust
# #[cfg(all(feature = "vec-predicate", feature = "approx-ann"))] {
use sparq_vectors::{query_vec_approx, DiskAnnIndex};
let index = DiskAnnIndex::build_for(&store, "entities.spqg", &graph)?;   // build/open the Vamana graph once
let r = query_vec_approx(&graph,
    "PREFIX vec: <http://sparq.dev/vec#>
     SELECT ?node WHERE { ?node vec:nearest ( \"0.1,0.9,…\" 10 ) }",
    &store, &index)?;                                                     // APPROXIMATE top-10
# }
```

**Automatic filtered ANN (compose with `filtered-ann`, sq-bvmd + sq-3tjd).** When **both**
`vec-predicate` *and* `filtered-ann` are on, the rewrite derives the candidate id-set
([`IdMask`], recipe 9) automatically: if the `vec:` neighbour variable is **constrained by
ordinary patterns in the same BGP**, those patterns carve out the eligible nodes and the k-NN is
run as a **filtered** search (`nearest_exact_filtered`) over just that set — no separate API call.

The constraint is the **join-connected sub-BGP** of the neighbour variable (sq-3tjd): not only
patterns that *directly mention* it, but every pattern reachable from it through shared variables
— so the mask honours **transitive / multi-variable** constraints. A direct-mention-only BGP is a
special case of this (the connected component is just those patterns), so the single-variable
behaviour from sq-bvmd is unchanged.

```rust
// Direct (single-variable): ?node is the neighbour AND is itself a :Car —
// the search only ever considers Cars, so a closer non-Car is never returned.
let r = query_vec(&graph,
    "PREFIX vec: <http://sparq.dev/vec#>
     SELECT ?node WHERE {
       ?node vec:nearest ( \"1,0\" 5 ) .
       ?node <http://ex/kind> <http://ex/Car> .   # ← carves out the candidate id-set
     }", &store)?;

// Transitive (multi-variable, 2-hop): ?node is restricted to subjects that own a
// :Vehicle even though ?node never appears in the `?x a :Vehicle` pattern — the
// connected component {?node :owns ?x, ?x a :Vehicle} is evaluated and ?node projected.
let r = query_vec(&graph,
    "PREFIX vec: <http://sparq.dev/vec#> PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
     SELECT ?node WHERE {
       ?node vec:nearest ( \"1,0\" 5 ) .
       ?node <http://ex/owns> ?x .
       ?x rdf:type <http://ex/Vehicle> .          # ← reached transitively through ?x
     }", &store)?;
```

The mask is exactly the set the engine binds to the neighbour variable when that connected
sub-BGP is evaluated and the neighbour variable projected, so the filtered top-k is **identical to
post-filtering the unfiltered top-k** by that same (now transitive) constraint — and therefore a
subset of the unfiltered result. A pattern **disconnected** from the neighbour variable (no
shared-variable path) is excluded, so it never narrows the mask. Each `vec:` request in a BGP gets
its **own** connected-component mask, derived independently. If the neighbour variable is
**unconstrained** (no pattern mentions it) the search falls back to the plain unfiltered
`nearest_exact` (recipe 8's exact behaviour, unchanged). With `filtered-ann` **off**, the `vec:`
predicate is always unfiltered — this composition adds nothing to the `vec-predicate`-only build.
**Cyclic** join sub-BGPs (a back-edge to the neighbour variable like `?node :owns ?x . ?x
:ownedBy ?node`, or a longer cycle among intermediates) are handled correctly (sq-p5oy): the
connected-component computation is a per-pattern fixpoint so it always **terminates** on a cycle,
and the cyclic sub-BGP is evaluated through the standard engine, so the mask is exactly the engine's
(cycle-correct) binding — `filtered == post-filter` holds for cyclic constraints too.

**Pre-filter vs post-filter — the cost model (sq-7hx6, subsumes sq-ic0n).** Once the mask is derived,
the rewrite still has a *choice*: **pre-filter** (scan only the masked ids, `nearest_exact_filtered`)
or **post-filter** (scan the whole store unfiltered, then drop the disallowed hits, `postfilter_exact`).
Both return the **byte-identical** top-k (same ids, same order — see recipe 10), so the choice is purely
about throughput. The rewrite picks per query via `CostModel` (recipe 10): pre-filter when the mask is
selective (`mask_len · scatter_penalty ≤ store_len`, default crossover ≈ half the store), post-filter
when it is broad. This is a HEURISTIC over a cost ESTIMATE, not an optimum — it never affects the answer,
only the work. (Deriving the mask itself is unconditional: we must, to stay answer-safe; the cost model
only decides how to *use* it.)

### 9. Predicate-constrained (filtered) ANN — only neighbours a BGP admits (opt-in, feature = `filtered-ann`)

The RDF-native vector differentiator: a SPARQL BGP carves out the *eligible* graph nodes (e.g.
`?node a :Car ; :seats ?s`), and the nearest-neighbour search returns only neighbours **inside that
candidate set**. You compute the exact id-set from the BGP (via `graph.id_of` / the permutation
indexes / a `query`'s solution `?node` column) and hand it to the search as an `IdMask` — the
"visit mask". OPT-IN and OFF by default; the feature is **lean** — it adds *no new dependency* (the
mask reuses the in-tree `rustc-hash` set) and pulls in neither the engine nor any heavy crate.

```toml
sparq-vectors = { path = "../sparq-vectors", features = ["filtered-ann"] }
```

```rust
use sparq_vectors::{nearest_exact_filtered, DiskAnnIndex, IdMask, VectorIndex, VectorStore};

let store = VectorStore::open("entities.spqv")?;

// 1. Build the mask from whatever the BGP selected — here, ids of nodes typed :Car.
let car_ids = /* graph.query("SELECT ?node { ?node a :Car }") -> ?node column -> graph.id_of */;
let mask: IdMask = car_ids.into_iter().collect();          // empty mask => no results

// 2a. Exact filtered (ground truth; the pre-filter strategy) — scans only the masked ids:
let hits = nearest_exact_filtered(&store, query, &mask, 10); // every id is guaranteed in the mask

// 2b. On-disk DiskANN filtered: picks the strategy by SELECTIVITY (FilterConfig::default):
//   selective mask (<= 1% of store, or <= 256) -> exact pre-filter scan;
//   broad mask -> ACORN/NaviX filtered traversal (walk the graph through ALL nodes for
//   connectivity, ACCEPT only masked ones; the beam is widened so k accepted hits are collected).
let idx = DiskAnnIndex::open("entities.spqg")?;
let approx = idx.nearest_filtered(query, &mask, &store, 10); // recall@10 vs exact-filtered ~0.998 on a ~50% mask

// 2c. In-RAM HNSW filtered: instant-distance adjacency is NOT exposed, so this is the exact
//     pre-filter (the right path for a selective mask; use DiskAnnIndex for a broad mask):
let vidx = VectorIndex::build(&store);
let hnsw = vidx.nearest_filtered(query, &mask, &store, 10);
```

Tune the crossover / beam with `nearest_filtered_with(query, &mask, &store, k, FilterConfig { .. })`.
You build the mask yourself here, which keeps this filtered API a pure sparq-vectors surface with
zero engine coupling. The engine-side **automatic** BGP → mask wiring — deriving the `IdMask` from
the surrounding BGP and running a filtered `vec:` search — is wired in recipe 8 (sq-bvmd, when both
`vec-predicate` and `filtered-ann` are on), and is built on exactly this `nearest_exact_filtered`
seam.

### 10. Pre-filter vs post-filter — the cost model (opt-in, feature = `filtered-ann`)

Deriving a mask is one cost; *using* it is another. With a mask in hand there are two ways to the
**same** top-k: **pre-filter** (scan only the masked ids) or **post-filter** (scan the whole store
unfiltered, then drop the disallowed hits). `CostModel` estimates both and picks the cheaper per
query. The `vec:` rewrite (recipe 8) calls this automatically; you can also drive it directly.

```rust
use sparq_vectors::{nearest_filtered_costed, postfilter_exact, CostModel, Strategy};

// Decision only (no search): pre-filter iff mask_len * scatter_penalty <= store_len.
let est = CostModel::default().decide(/*mask_len*/ 50, /*store_len*/ 1000, /*k*/ 10);
assert_eq!(est.strategy, Strategy::PreFilter);   // 50*2 = 100 <= 1000 → selective → pre-filter
//   est.prefilter_cost / est.postfilter_cost expose WHY.

// Decide + run the chosen branch in one call; the second tuple element is the estimate.
let (hits, est) = nearest_filtered_costed(&store, query, &mask, 10, &CostModel::default());

// Post-filter directly (the broad-mask branch). IDENTICAL result to nearest_exact_filtered:
let post = postfilter_exact(&store, query, &mask, 10);
assert_eq!(post, sparq_vectors::nearest_exact_filtered(&store, query, &mask, 10));
```

**The model.** `m = |mask|`, `n = store_len`. Pre-filter touches `m` *scattered* rows (random-access
gather into the mmap), post-filter streams `n` rows sequentially; a scattered row is modelled as
`scatter_penalty ×` (default `2.0`) a sequential one. **Pre-filter iff `m · scatter_penalty ≤ n`** —
default crossover ≈ half the store. `k`/`dim` cancel out of the exact-scan comparison (both branches
do `dim`-width work per touched row, then take `k`), so they do not enter the crossover; they are
surfaced in `CostEstimate` for a future approximate backend.

**Why it is answer-safe.** `nearest_exact` returns a *complete* ranking of the store, so post-filtering
it and pre-filtering the mask reduce to ranking the same admitted ids by the same comparator — the
top-k is **byte-identical** (asserted in `tests/cost_model.rs`). There is **no over-fetch recall
boundary** for this exact backend: a full ranking can only under-fill `k` when the mask admits fewer
than `k` stored vectors, and then the pre-filter scan is equally short. (`overfetch_target(k, m, n) =
ceil(k/selectivity)` sizes the first fetch for the **approximate** backend, whose bounded candidate
list CAN under-fill — the iterative over-fetch path that handles it is recipe 11.)

**Honest scope.** This is a **HEURISTIC over a cost estimate, not an optimum**: `scatter_penalty` is a
single modelled constant (non-canonical, not a measured fit), real cache/SIMD behaviour is
hardware-dependent, and a planner that has *not* yet evaluated the sub-BGP would feed an *estimated*
cardinality (here the mask is materialised, so `m` is exact). A mis-estimate costs only throughput —
never the answer.

### 11. Approximate filtered ANN with iterative over-fetch (opt-in, feature = `filtered-ann` + `approx-ann`)

The cost model (recipe 10) is for the **exact** backend, which post-filters a *complete* ranking and
so never under-fills. An **approximate** backend returns a *bounded* candidate list: post-filtering it
can under-fill `k` when the admitted ids cluster *below* the fetched prefix (the recall boundary).
`nearest_filtered_overfetch` removes that boundary by growing the fetch and retrying.

```rust
# #[cfg(all(feature = "filtered-ann", feature = "approx-ann"))] {
use sparq_vectors::{
    nearest_filtered_overfetch_default, ApproxBackend, ExactBackend, DiskAnnIndex, VectorStore,
};
let store = VectorStore::open("entities.spqv")?;
let idx = DiskAnnIndex::open("entities.spqg")?;

// EXACT backend: answer-exact (recall 1.0) — the ground-truth reference, no third-party dep.
let exact = ExactBackend::new(&store);
let exact_hits = nearest_filtered_overfetch_default(&exact, query, &mask, 10);

// APPROXIMATE backend over the on-disk Vamana graph: recall < 1.0 (NOT exact).
let approx = ApproxBackend::new(&idx);
let approx_hits = nearest_filtered_overfetch_default(&approx, query, &mask, 10);
// fetch overfetch_target(k, selectivity); post-filter; if < k survive, DOUBLE the fetch & retry
// up to the backend size — fills k whenever k admitted vectors exist AND the index surfaces them.
# let _ = (exact_hits, approx_hits); }
```

**Honesty (load-bearing).** Iterative over-fetch fixes **under-fill** (returning < k when k exist); it
does **not** make the approximate backend exact. Even a full-store fetch from `ApproxBackend` is the
*approximate* ranking, so a true admitted neighbour the index never visits is still missed — **recall
stays < 1.0** (measured floor 0.90@10 in `tests/overfetch.rs`, NOT claimed as exactness). Only
`ExactBackend` is answer-exact. The A1-paper recall evidence is for the **exact/transitive** answer-safe
path (recipe 10), and does **not** transfer to this approximate path. Implement your own backend by
`impl`-ing `AnnBackend` (one `candidates(query, fetch)` method + `len`).

### 12. Incremental add / remove / update without a full rebuild (opt-in, feature = `delta`)

A `.spqv` is build-once-immutable, so one new/removed entity would otherwise force a full re-embed +
rebuild. The `delta` feature adds a **delta sidecar** over the immutable base — `add` / `remove`
/ `update` write an append map + tombstone set, and `get` / `iter` / `len` (hence search) transparently
union base+delta and honour tombstones. `compact` folds it back into a fresh base. Lean (no new dep).

```rust,ignore
# // cargo build -p sparq-vectors --features delta
use sparq_vectors::VectorStore;
let mut store = VectorStore::open("graph.spqv")?;   // a finalized base

store.add(new_id, &vec_for_new_id)?;                 // NEW id (errs if already present)
store.update(existing_id, &new_vec)?;                // replace an existing id (errs if absent)
store.remove(stale_id);                              // tombstone (returns bool; re-add allowed)

// search transparently sees the delta — no rebuild, no extra API:
let hits = sparq_vectors::nearest_exact(&store, &query, 10);

// PERSIST the delta so the mutations survive a restart WITHOUT a compact (sq-7e50):
store.save_delta()?;                                 // crash-durable .spqd sibling (tmp+fsync+rename)
// …later, in a fresh process: reopen the base AND replay the persisted delta:
let reopened = VectorStore::open_with_delta("graph.spqv")?;  // == the (base + delta) view it had before

// or fold the delta into a fresh base == a from-scratch rebuild over the final vector set:
let compacted = store.compact("graph.spqv", &graph)?;   // bound to graph's fingerprint generation
# Ok::<(), String>(())
```

**Generation tie.** A delta carries the graph **generation** (the sq-32i5 fingerprint) it was built
against; `apply_delta` rejects a delta whose generation differs from the receiving base, so its dict-ids
can never mis-key onto a different graph. The persisted `.spqd` header carries that same generation, so
`open_with_delta` routes the replay through `apply_delta` and a sidecar written against a *different*
graph generation is rejected, never silently mis-keyed. **Durability (sq-7e50).** Two durability paths:
`compact` materializes the delta into a fresh validated `.spqv`; `save_delta` persists the *delta itself*
to a crash-durable `.spqd` (write-tmp + `fsync` + atomic rename) that `open_with_delta` replays on reopen
— so incremental mutations survive a process restart with no compact. A truncated/torn sidecar is
detected (an exact-length check) and rejected, never read out of bounds. `put` is unchanged (still errors
after `finalize`) — `add` is the additive path.

## Gotchas / feature flags / prerequisites

- **Opt-in.** Nothing in the workspace depends on `sparq-vectors`; the default engine
  build does not compile it. The core Rust flow (store/ANN/embed) is a standalone library
  over sparq-core's public read API — you wire it into application code yourself. The **one**
  SPARQL-level integration is the optional `vec:` magic predicate (recipe 8 below), behind
  the **non-default `vec-predicate`** feature; with it OFF this crate has zero `sparq-engine`
  dependency and the base engine/core query path is byte-identical (see recipe 8). There is
  still no `SERVICE`/function binding. Predicate-constrained (filtered) ANN (recipe 9) is a
  separate **non-default `filtered-ann`** feature — also lean (no new dep, no engine pull).
- **Incremental add/remove/update is the non-default `delta` feature (sq-pi44, persistence sq-7e50).**
  Also lean (no new dep, no engine pull). With it OFF the store is build-once-immutable as before
  (`put` errors after `finalize`; no `add`/`remove`/`update`/`compact`/`save_delta`); with it ON those
  methods write a delta the read paths consult transparently (recipe 12). The delta lives in RAM on the
  handle; `save_delta` persists it to a crash-durable `.spqd` sidecar and `open_with_delta` replays it,
  so mutations survive a restart without a `compact` (the two durability paths).
- **`approx-ann` is the ONLY heavy ANN dependency, and it is OFF by default (sq-ip3a).** The HNSW
  index (`VectorIndex`/`HnswConfig`) and the `ApproxBackend` are gated behind it — it is the only
  thing pulling `instant-distance`. With it OFF the default build is lean: exact brute-force
  (`nearest_exact`, answer-exact) + the hand-rolled on-disk Vamana graph (`DiskAnnIndex`, no extra
  dep). Approximate search is **APPROXIMATE** — recall < 1.0, NOT answer-exact (recipes 2 & 11).
- **Reference-library verification (sq-6te5).** The recall floors are anchored not only against
  this crate's own exact searcher but against an **established ANN library** (hnswlib/FAISS):
  `tests/ref_lib_verify.rs` loads a committed fixture (`tests/fixtures/hnswlib_ref.tsv`, a REAL
  hnswlib capture) and asserts (a) `nearest_exact` reproduces numpy's exact-kNN oracle exactly —
  same metric, same corpus — and (b) sparq-vectors' DiskANN (and HNSW under `approx-ann`) clears
  hnswlib's OWN recall on the shared oracle. **This runs in CI with no native deps** — the corpus
  is regenerated from a splitmix64 seed on both sides (bit-identical Python and Rust), so the
  fixture is just neighbour ids. The **live** hnswlib re-capture
  (`scripts/capture_hnswlib_ref.py`, needs numpy + hnswlib) is `#[ignore]`d / **gather-only**: run
  `cargo test -p sparq-vectors --test ref_lib_verify -- --ignored` (optionally
  `VECTOR_PY=<venv>/bin/python`). Recall numbers are work-box NON-CANONICAL and are recomputed off
  the ids, never baked.
- **`HashEmbedder` is TEST-ONLY.** It is lexical n-gram hashing with **no semantics**
  ("car" and "automobile" are unrelated). Use it to exercise the store/ANN/pipeline; bring
  a real `Embedder` for actual retrieval.
- **Live embeddings — two opt-in tiers, default build is socket-free.** For a real
  OpenAI-compatible `/v1/embeddings` endpoint:
  - **`provider`** (non-default) carries the API shape only — it builds the request body and
    parses the response but **never opens a socket**; you supply a `Transport` impl over your
    HTTP client (reqwest/ureq/recorded cassette). `cargo build -p sparq-vectors --features provider`.
  - **`embeddings`** (non-default; enables `provider` + a blocking reqwest `Transport`) needs
    **no caller glue**: `RemoteEmbedder::from_env(dim)` reads `SPARQ_EMBEDDINGS_API_KEY`
    (required), `SPARQ_EMBEDDINGS_BASE_URL` (default `https://api.openai.com/v1/embeddings`),
    and `SPARQ_EMBEDDINGS_MODEL` (default `text-embedding-3-small`), then `embed_entities`
    works against the live endpoint. Mirrors `sparq-nlq`'s `live` feature; requests carry a
    hard timeout. `cargo build -p sparq-vectors --features embeddings`.
  The **default build pulls no HTTP client** (no reqwest) and the crate never enters the wasm
  bundle, so neither tier affects the default or wasm builds.
- **Keys are dictionary term ids** (`sparq_core::dict::Id`, a `u32`). Resolve a `Term` with
  `graph.id_of(&term) -> Option<Id>` and an `Id` back with `graph.dict.term(id)`. Coverage
  is **sparse by design** — only entities get vectors, not every literal.
- **Graph scoping (sq-quuu).** Every `&Graph` entry point (`verbalize`, `embed_entities`,
  `embed_labels`, `nearest_term*`) reads the store of the `Graph` it is handed — the
  **default graph** for the top-level `&graph`, or a **single named graph** when you pass
  that graph's sub-`Graph`. On a quad dataset (N-Quads / TriG via `Graph::load_dataset`)
  each named graph is a self-contained `Graph`; fetch one by name with
  `graph.named_graph(&name)` and build a store **per graph** (or over the default graph).
  Verbalization labels never cross graphs and there is no union-of-all-graphs mode — the
  quads are not merged for you. Because keys are dict ids and a named graph has its **own**
  dictionary, a store is bound to **one** `Graph`: the per-named-graph store and the
  default-graph store carry distinct fingerprints, so `check_graph` catches a cross-graph
  mix-up.
- **Dim must match.** `VectorStore::create(path, dim)` fixes `dim`; `embed_*` errors if
  `embedder.dim() != store.dim()`. `embed_labels`/`embed_entities` leave the store
  **unfinalized** — call `store.finalize()` before opening/searching (or before a fresh
  `VectorStore::open`).
- **Degenerate vectors:** `put` rejects all-zero and non-finite vectors (cosine is
  undefined); a zero **query** returns no results from every searcher, so exact/HNSW/DiskANN
  agree on the degenerate case.
- **HNSW is per-process.** `VectorIndex` is rebuilt from the mmap on `build` (~33–83 s for
  50k×32 on an M1). Use `DiskAnnIndex` for a persistent index that reopens with no rebuild;
  its **build** is single-threaded (slower than the rayon HNSW build) — only the **open** is
  cheap. `ef_search` / `search_beam` must be ≥ the `k` you query.
- **DiskANN honest scope:** `DiskAnnIndex` searches full-precision vectors straight from the
  mmap by default. The PQ-compressed in-RAM candidate cache that *full* DiskANN ranks on
  (`quant.rs`) exists, is tested, and **is now wired into** `search_slots` (sq-qamd / #620):
  when a PQ cache is present, `search_slots` dispatches to `search_slots_pq` — rank on the
  RAM-resident codes, re-rank the final beam off the mmap. Build it via
  `DiskAnnIndex::build_with_pq` and check `has_pq_cache()`; recipe 4 shows the same loop the
  index now drives internally. <!-- [OPUS-4.8] de-staled: PQ cache wired into search_slots (sq-qamd/#620) -->
  Without a PQ cache, search stays full-precision-from-mmap as above.
- **Filtered ANN (`filtered-ann` feature):** predicate-constrained search returns only ids in the
  `IdMask`, and **every returned id is guaranteed in the mask**. An **empty** mask -> no results
  (distinct from passing no mask = an unfiltered search); a **full** mask -> identical to the
  unfiltered search. `DiskAnnIndex::nearest_filtered` is **exact** when it pre-filters (selective
  mask) and **approximate** when it traverses (broad mask) — but pre-filter is exact, so erring
  toward it (the conservative default crossover) only costs mild throughput, never recall. The
  in-RAM `VectorIndex::nearest_filtered` is **always** the exact pre-filter (instant-distance
  adjacency is not exposed, so the HNSW graph cannot be walked with predicate-aware acceptance);
  use `DiskAnnIndex` for filtered traversal over a broad mask. Lean feature: no new dependency, no
  engine pull. NON-CANONICAL timing.
- **Little-endian only.** `.spqv`/`.spqg` reject big-endian targets at create/open.
- **Determinism:** ties break on ascending id (searchers) or first appearance (fusion);
  HNSW/Vamana/PQ seeds are fixed by default so builds are reproducible.

## See also

- `sparql-formal-semantics` — the SPARQL algebra the engine evaluates (vectors are not part
  of it).
- `hdt-format`, `fused-decompress-parse`, `rust-parallel-parsing` — getting RDF into the
  `Graph` you then embed.
- `mpc-protocols`, `noir-circuit-patterns` — unrelated sibling skills in this workspace.
