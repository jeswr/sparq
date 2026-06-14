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
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;
use sparq_vectors::{embed_labels, HashEmbedder, VectorIndex, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let embedder = HashEmbedder::new(64);              // test-only; bring your own Embedder
let mut store = VectorStore::create("graph.spqv", 64)?;
embed_labels(&graph, &mut store, &embedder)?;       // rdfs:label > skos:prefLabel > …
store.finalize()?;                                  // handle becomes mmap-backed

let index = VectorIndex::build(&store);             // HNSW
let _neighbours = index.nearest_term(&some_term, &graph, &store, 10);
# Ok(()) }
# const ttl: &str = "<http://ex/a> <http://www.w3.org/2000/01/rdf-schema#label> \"Alpha\" .";
# use oxrdf::{NamedNode, Term}; let some_term: Term = NamedNode::new("http://ex/a").unwrap().into();
```

## ✨ Features

- **`VectorStore`** — memory-mapped `.spqv` store; `get` is one binary search + a contiguous
  `&[f32]`; corrupt files are rejected up front. `open_from_bytes` for filesystem-less use.
- **`StreamingWriter`** — build stores bigger than RAM with O(1) build-phase memory;
  byte-identical output to the in-RAM builder.
- **Search** — `nearest_exact` (ground-truth baseline), in-RAM HNSW (`VectorIndex`), and the
  persistent on-disk `DiskAnnIndex` (`.spqg`, build once / open with no rebuild).
- **Verbalization** — `verbalize` / `embed_entities` render `<label>. a <type>. <description>`
  per entity (multilingual, char-budgeted); `embed_labels` is the label-only special case.
- **Quantization** — `ScalarQuantizer` (4×) and `ProductQuantizer` (asymmetric distance) for
  large stores; `Embedder` is provider-agnostic (the crate never opens a socket).
- **Hybrid fusion** — `fuse_rrf` / `fuse_rrf_weighted` / `fuse_scores` combine text vectors
  with another ranked signal (e.g. [`sparq-sim`](../sparq-sim) structural similarity).

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
