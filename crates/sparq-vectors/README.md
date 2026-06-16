# sparq-vectors

<p>
  <a href="https://crates.io/crates/sparq-vectors"><img src="https://img.shields.io/crates/v/sparq-vectors.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-vectors"><img src="https://docs.rs/sparq-vectors/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Opt-in embedding store + nearest-neighbour search** for the [sparq](../../README.md) RDF
engine (GenAI phase 4).

One f32 embedding per dictionary term id, in a flat memory-mapped `.spqv` file (sparse by
design — only entities, not every literal). Query top-`k` by cosine with an exact brute-force
scan, an in-RAM HNSW index, or a persistent on-disk DiskANN/Vamana graph. Embeddings are
produced **outside** the engine; the crate verbalizes entities (label + type + description)
to text, embeds via a provider-agnostic trait, and fuses with another ranked signal for
hybrid search. It is a **separate crate** — nothing in the workspace (or the wasm build)
depends on it.

## 🚀 Quickstart

```rust
# use oxrdf::{NamedNode, Term};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let ttl = "<http://ex/a> <http://www.w3.org/2000/01/rdf-schema#label> \"Alpha\" .";
# let some_term: Term = NamedNode::new("http://ex/a")?.into();
# // [OPUS-4.8] Unique temp path so this doctest leaves no artifact in the crate root and
# // doesn't collide with parallel doctests; cleaned up below. (No tempfile dev-dep.)
# let nanos = std::time::SystemTime::now()
#     .duration_since(std::time::UNIX_EPOCH)?.as_nanos();
# let path = std::env::temp_dir()
#     .join(format!("sparq-vectors-quickstart-{}-{}.spqv", std::process::id(), nanos));
use sparq_core::Graph;
use sparq_vectors::{embed_labels, HashEmbedder, VectorIndex, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let embedder = HashEmbedder::new(64);              // test-only; bring your own Embedder
let mut store = VectorStore::create(&path, 64)?;
embed_labels(&graph, &mut store, &embedder)?;       // rdfs:label > skos:prefLabel > …
store.finalize()?;                                  // handle becomes mmap-backed

let index = VectorIndex::build(&store);             // HNSW
let _neighbours = index.nearest_term(&some_term, &graph, &store, 10);
# // [OPUS-4.8] drop the mmap handle before removing the backing file.
# drop(index); drop(store); std::fs::remove_file(&path).ok();
# Ok(()) }
```

## ✨ Features

- **`VectorStore`** — memory-mapped `.spqv` store; `get` is one binary search + a contiguous
  `&[f32]`; corrupt files are rejected up front. `open_from_bytes` for filesystem-less use.
- **Graph-staleness guard** — stores are keyed by dictionary id, so a store built against one
  graph silently mis-resolves after a dict-id-shifting rebuild. Both `.spqv`/`.spqg` headers
  embed a graph **fingerprint** (`with_fingerprint` / `build_for`); the checked query paths
  (`nearest_term_exact_checked`, `DiskAnnIndex::nearest_term_checked`, `check_graph`) return a
  descriptive error on a mismatch instead of wrong neighbours.
- **`StreamingWriter`** — build stores bigger than RAM with O(1) build-phase memory;
  byte-identical output to the in-RAM builder.
- **Search** — `nearest_exact` (ground-truth baseline), in-RAM HNSW (`VectorIndex`), and the
  persistent on-disk `DiskAnnIndex` (`.spqg`, build once / open with no rebuild).
- **Predicate-constrained (filtered) ANN (opt-in `filtered-ann`)** — restrict the search to the
  graph nodes a SPARQL BGP admits: build an `IdMask` from the BGP-selected dict-ids and call
  `nearest_exact_filtered` / `DiskAnnIndex::nearest_filtered`. Lean feature (no new dependency, no
  engine pull). Every returned id is in the mask; empty mask → no results; full mask → the
  unfiltered search.
- **k-NN inside SPARQL (opt-in `vec-predicate`)** — the `vec:nearest` / `vec:search` magic
  predicates run vector k-NN in plain SPARQL via a spargebra-algebra rewrite (the engine is
  unchanged). Composed with `filtered-ann`, a `vec:` neighbour variable that is **also constrained
  by ordinary BGP patterns** is searched as a *filtered* ANN automatically — the surrounding BGP
  derives the candidate `IdMask`, so the filtered top-k equals post-filtering the unfiltered top-k.
- **Verbalization** — `verbalize` / `embed_entities` render `<label>. a <type>. <description>`
  per entity (multilingual, char-budgeted); `embed_labels` is the label-only special case.
- **Quantization** — `ScalarQuantizer` (4×) and `ProductQuantizer` (asymmetric distance) for
  large stores; `Embedder` is provider-agnostic.
- **Live embeddings (opt-in)** — the default build is **socket-free** (no HTTP client). The
  non-default `provider` feature carries the OpenAI-compatible `/v1/embeddings` API shape with
  a caller-supplied `Transport`; the non-default **`embeddings`** feature additionally ships a
  concrete reqwest blocking `Transport` and `RemoteEmbedder::from_env(dim)`, so `embed_entities`
  works against a real endpoint with **no caller glue** (see below). Mirrors `sparq-nlq`'s
  `live` feature; never enters the wasm bundle.
- **Hybrid fusion** — `fuse_rrf` / `fuse_rrf_weighted` / `fuse_scores` combine text vectors
  with another ranked signal (e.g. [`sparq-sim`](../sparq-sim) structural similarity);
  `hybrid_search` runs N retriever closures off one query and fuses by item with RRF.

## 🔌 Live embeddings (opt-in `embeddings` feature)

The default build opens no sockets. To embed against a real OpenAI-compatible
`/v1/embeddings` endpoint without writing any HTTP glue, enable the **non-default**
`embeddings` feature (it pulls in a blocking [`reqwest`] client; the default build does not):

```toml
sparq-vectors = { path = "../sparq-vectors", features = ["embeddings"] }
```

```rust,ignore
use sparq_core::Graph;
use sparq_vectors::{embed_entities, EntityTextConfig, VectorStore};
use sparq_vectors::embed::provider::RemoteEmbedder;

// SPARQ_EMBEDDINGS_API_KEY (required) → Authorization: Bearer …
// SPARQ_EMBEDDINGS_BASE_URL (optional) → endpoint, default https://api.openai.com/v1/embeddings
// SPARQ_EMBEDDINGS_MODEL    (optional) → model,    default text-embedding-3-small
let embedder = RemoteEmbedder::from_env(1536)?;     // dim must match the store
let mut store = VectorStore::create("graph.spqv", 1536)?;
embed_entities(&graph, &mut store, &embedder, &EntityTextConfig::default())?;
store.finalize()?;
```

`from_env` reads the key/base-url/model from the environment (mirroring `sparq-nlq`'s live
client); requests carry a hard timeout. Point `SPARQ_EMBEDDINGS_BASE_URL` at any
OpenAI-compatible server (a self-hosted model, a proxy). The crate still **never opens a
socket in the default build** — only with `--features embeddings`, and it is excluded from the
wasm build (nothing in the workspace depends on this crate).

## 📚 Learn more

- **How-to** — [`skills/vector-search/SKILL.md`](../../skills/vector-search/SKILL.md) (label /
  verbalized / hybrid pipelines, DiskANN, quantization, the full API surface).
- **API reference + `.spqv` / `.spqg` formats** — [docs.rs/sparq-vectors](https://docs.rs/sparq-vectors)
  (the format layout is documented on the `store` / `diskann` modules).
- **Design** — [`research/genai-text-embedding-practices.md`](../../research/genai-text-embedding-practices.md).
- **Accuracy & throughput** — not baked into docs; the HNSW/DiskANN/PQ recall gates and the
  throughput measurement are `cargo test`s (`tests/recall.rs`, `tests/diskann.rs`,
  `tests/quant.rs`, `tests/throughput.rs` — run them for the figures), with live numbers on
  the [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
