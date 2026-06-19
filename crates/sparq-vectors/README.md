<!-- [OPUS-4.8] sq-inzv: README brought to template. -->
# sparq-vectors

<p>
  <a href="https://crates.io/crates/sparq-vectors"><img src="https://img.shields.io/crates/v/sparq-vectors.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-vectors"><img src="https://docs.rs/sparq-vectors/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Opt-in embedding store + nearest-neighbour search** for the [sparq](../../README.md)
RDF engine (GenAI phase 4).

One f32 embedding per dictionary term id, in a flat memory-mapped `.spqv` file (sparse by
design). Query top-`k` by cosine with an exact brute-force scan or a persistent on-disk
DiskANN/Vamana graph by default, or an in-RAM HNSW index behind the opt-in `approx-ann`
feature (the only third-party ANN dependency; **recall < 1.0**). Embeddings are produced
**outside** the engine; the crate verbalizes entities to text, embeds via a
provider-agnostic trait, and fuses with another ranked signal for hybrid search. It is a
**separate crate** — nothing in the workspace (or the wasm build) depends on it.

## 🚀 Quickstart

```rust
# use oxrdf::{NamedNode, Term};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let ttl = "<http://ex/a> <http://www.w3.org/2000/01/rdf-schema#label> \"Alpha\" .";
# let some_term: Term = NamedNode::new("http://ex/a")?.into();
# let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
# let path = std::env::temp_dir()
#     .join(format!("sparq-vectors-quickstart-{}-{}.spqv", std::process::id(), nanos));
use sparq_core::Graph;
use sparq_vectors::{embed_labels, nearest_term_exact, HashEmbedder, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let embedder = HashEmbedder::new(64);              // test-only; bring your own Embedder
let mut store = VectorStore::create(&path, 64)?;
embed_labels(&graph, &mut store, &embedder)?;       // rdfs:label > skos:prefLabel > …
store.finalize()?;                                  // handle becomes mmap-backed

// Exact brute-force search — the default build (no third-party ANN crate).
let _neighbours = nearest_term_exact(&store, &graph, &some_term, 10);
# drop(store); std::fs::remove_file(&path).ok();
# Ok(()) }
```

## ✨ Features

- **`VectorStore`** — memory-mapped `.spqv`; `get` is one binary search + a contiguous
  `&[f32]`; corrupt files rejected up front. `StreamingWriter` builds stores bigger than
  RAM in O(1) memory, byte-identical to the in-RAM builder. `open_from_bytes` for
  filesystem-less use.
- **Exact vs approximate search** — `nearest_exact` (answer-exact ground truth) and the
  persistent on-disk `DiskAnnIndex` in the default build; the in-RAM HNSW `VectorIndex`
  behind the **opt-in `approx-ann`** feature, the **only** third-party-ANN dependency
  (`instant-distance`). **Approximate search has recall `< 1.0`** (measured against
  `nearest_exact`) — only the exact path is answer-exact; the approximate path is never
  claimed as exact.
- **Predicate-constrained ANN (opt-in `filtered-ann`)** — restrict the search to the
  dict-ids a SPARQL BGP admits via an `IdMask` (lean, no new dependency). Composed with
  `approx-ann`, filtered over-fetch fills `k` whenever `k` admitted vectors exist *and
  the backend surfaces them* — **honest caveat:** over-fetch fixes *under-fill*, not the
  approximate index's inherent misses (recall stays `< 1.0`, asserted in
  `tests/overfetch.rs`).
- **k-NN inside SPARQL (opt-in `vec-predicate`)** — `vec:nearest` / `vec:search` magic
  predicates via a spargebra-algebra rewrite (the engine is unchanged). A constrained
  neighbour variable is searched as a *filtered* ANN over its join-connected sub-BGP,
  with the derived `IdMask` cached by sub-BGP + graph fingerprint (any graph change
  misses the cache, so a stale mask is never served). A per-query cost model chooses
  pre- vs post-filter; both return the **byte-identical** top-k.
- **Graph-staleness guard** — `.spqv`/`.spqg` headers embed a dictionary **fingerprint**
  (thread-count-stable: re-loading the same RDF at a different `RAYON_NUM_THREADS`
  fingerprints identically). The checked query paths reject a mismatch instead of serving
  wrong neighbours. **Id-keyed staleness contract (sq-wlzi):** a passing `check_graph` is
  **necessary but not sufficient** — to serve a persisted store, `Graph::save` its graph
  and reopen **that** (`Graph::open`, frozen id order); **never** re-parse the source RDF
  (pinned in `tests/staleness_contract.rs`).
- **Verbalization, quantization, hybrid fusion** — `verbalize` / `embed_entities` render
  per-entity text (multilingual, char-budgeted, single named graph at a time);
  `ScalarQuantizer` (4×) and `ProductQuantizer` for large stores; `fuse_rrf` /
  `hybrid_search` combine vectors with another ranked signal (e.g.
  [`sparq-sim`](../sparq-sim)).
- **Bring-your-own embeddings** — `import_npy` / `import_numeric_dump` load an
  externally-computed matrix straight into a `.spqv` keyed by dict id (no model
  in-process, no new dependency). **Fail-closed**: any dtype / byte-order / shape /
  length / non-finite-row mismatch is an `Err`, never a silent reinterpretation, and the
  declared `.npy` header is bounded before any body allocation.
- **Live embeddings (opt-in)** — the default build is **socket-free**. The non-default
  `provider` feature carries the OpenAI-compatible `/v1/embeddings` shape with a
  caller-supplied `Transport`; `embeddings` adds a concrete reqwest client +
  `RemoteEmbedder::from_env(dim)`. Never enters the wasm bundle.

## 📚 Learn more

- **How-to** — [`skills/vector-search/SKILL.md`](../../skills/vector-search/SKILL.md)
  (label / verbalized / hybrid pipelines, DiskANN, quantization, bulk import, the full
  API surface and the `.spqv` / `.spqg` formats).
- **API reference** — [docs.rs/sparq-vectors](https://docs.rs/sparq-vectors).
- **Design** — [`research/genai-text-embedding-practices.md`](../../research/genai-text-embedding-practices.md).
- **Accuracy & throughput** — not baked into docs; the recall / DiskANN / PQ /
  throughput gates are `cargo test`s (`tests/recall.rs`, `tests/diskann.rs`,
  `tests/quant.rs`, `tests/throughput.rs`), with live numbers on the
  [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
