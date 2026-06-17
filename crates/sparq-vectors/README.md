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
scan or a persistent on-disk DiskANN/Vamana graph by default, or an in-RAM HNSW index behind the
opt-in `approx-ann` feature (the only third-party ANN dependency; recall < 1.0). Embeddings are
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
use sparq_vectors::{embed_labels, nearest_term_exact, HashEmbedder, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let embedder = HashEmbedder::new(64);              // test-only; bring your own Embedder
let mut store = VectorStore::create(&path, 64)?;
embed_labels(&graph, &mut store, &embedder)?;       // rdfs:label > skos:prefLabel > …
store.finalize()?;                                  // handle becomes mmap-backed

// Exact brute-force search — the default build (no third-party ANN crate). For the
// approximate in-RAM HNSW index, enable the opt-in `approx-ann` feature and build a
// `VectorIndex` (recall < 1.0 — see the Search bullet below).
let _neighbours = nearest_term_exact(&store, &graph, &some_term, 10);
# // [OPUS-4.8] drop the mmap handle before removing the backing file.
# drop(store); std::fs::remove_file(&path).ok();
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
- **Search** — `nearest_exact` (answer-exact ground-truth baseline) and the persistent on-disk
  `DiskAnnIndex` (`.spqg`, build once / open with no rebuild) in the default build; the in-RAM HNSW
  `VectorIndex` behind the **opt-in `approx-ann` feature** (see below).
- **Approximate backend (opt-in `approx-ann`)** — the **only** feature that pulls a third-party ANN
  crate (`instant-distance`, HNSW), so the **default build carries no heavy ANN dependency** — just
  the exact scan and the hand-rolled on-disk Vamana graph (lean core). Approximate search is
  **APPROXIMATE**: its recall is **`< 1.0`** (measured against `nearest_exact`, the ground truth) —
  only the exact path is answer-exact. A **pluggable backend trait** (`AnnBackend`) abstracts the
  two: `ExactBackend` (answer-exact, no extra dep) and `ApproxBackend` (over `DiskAnnIndex`,
  `approx-ann`). Composed with `filtered-ann`, the filtered approximate path uses **iterative
  over-fetch** (`nearest_filtered_overfetch`): post-filtering a *bounded* approximate candidate list
  can under-fill `k` when admitted ids cluster below the fetched prefix (the recall boundary the
  `sq-7hx6` cost model documented), so it fetches `overfetch_target(k, selectivity)`, post-filters,
  and **if fewer than `k` survive, doubles the fetch and retries** up to the backend size — filling
  `k` whenever `k` admitted vectors exist *and the backend surfaces them*. **Honest caveat:**
  over-fetch fixes *under-fill*, not the approximate index's inherent misses — recall stays `< 1.0`
  (a measured floor asserted in `tests/overfetch.rs`, **not** claimed as exactness). The A1-paper
  recall evidence is for the **exact / transitive answer-safe** path (`sq-7hx6`) and does not
  transfer to this approximate path.
- **Predicate-constrained (filtered) ANN (opt-in `filtered-ann`)** — restrict the search to the
  graph nodes a SPARQL BGP admits: build an `IdMask` from the BGP-selected dict-ids and call
  `nearest_exact_filtered` / `DiskAnnIndex::nearest_filtered`. Lean feature (no new dependency, no
  engine pull). Every returned id is in the mask; empty mask → no results; full mask → the
  unfiltered search.
- **k-NN inside SPARQL (opt-in `vec-predicate`)** — the `vec:nearest` / `vec:search` magic
  predicates run vector k-NN in plain SPARQL via a spargebra-algebra rewrite (the engine is
  unchanged). Composed with `filtered-ann`, a `vec:` neighbour variable that is **constrained by
  ordinary BGP patterns** is searched as a *filtered* ANN automatically — the **join-connected
  sub-BGP** of the neighbour variable (its connected component in the BGP join-graph, so
  **transitive / multi-variable** constraints like `?node :owns ?x . ?x a :Vehicle` are honoured)
  derives the candidate `IdMask`, so the filtered top-k equals post-filtering the unfiltered top-k
  by that same constraint. A direct-mention-only constraint is the single-variable special case;
  disconnected patterns never narrow the mask; each `vec:` request gets its own component mask.
  That derived `IdMask` is **cached across prepares** (sq-36ol), keyed by the connected sub-BGP
  plus the graph `Fingerprint` — so the cache wraps the **transitive** derivation: repeated queries
  against an unchanged graph reuse the component's mask instead of re-running the constraint
  SELECT, and **any** graph change (added/removed/changed triple or term, or a dict-id shift)
  changes the fingerprint and misses the cache — so a stale mask is never served (the invalidation
  is sound; when in doubt it recomputes). The cache is transparent (no API change). With the mask in
  hand, a per-query **cost model** (`CostModel`, sq-7hx6) chooses **pre-filter** (scan only the mask)
  vs **post-filter** (scan the whole store, drop non-members) by the mask's selectivity — pre-filter
  iff `mask_len · scatter_penalty ≤ store_len` (default crossover ≈ half the store). Both branches
  return the **byte-identical** top-k (asserted in tests), so the choice trades only throughput,
  never the answer; it is a heuristic over a cost estimate, not an optimum.
  Composed instead with `approx-ann` (sq-z589), the `query_vec_approx` / `prepare_vec_approx` twins
  take an on-disk `DiskAnnIndex` and run the **unfiltered** `vec:` k-NN through that Vamana graph
  rather than the full scan — for large `.spqv` stores. **APPROXIMATE (recall `< 1.0`)**, never
  claimed as exact; use the exact `query_vec` when answer-exactness matters. The filtered path above
  is unaffected (still cost-model'd) — the approximate seam swaps only the unfiltered scan.
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
- **Bulk import (bring-your-own embeddings)** — `import_npy` / `import_numeric_dump` load a
  matrix computed elsewhere (Python/sentence-transformers, a vendor batch job) straight into a
  `.spqv` keyed by dictionary id, with no model in-process and no new dependency. Fail-closed on
  any dtype/shape/length mismatch (see below).

## 🔌 Live embeddings (opt-in `embeddings` feature)

The default build opens no sockets. To embed against a real OpenAI-compatible
`/v1/embeddings` endpoint without writing any HTTP glue, enable the **non-default**
`embeddings` feature (it pulls in a blocking `reqwest` client; the default build does not):

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

## 📦 Bulk import of externally-computed embeddings (sq-xsq9)

When your vectors come from an out-of-process pipeline rather than an in-process `Embedder`,
import the matrix directly. It reuses the standard store writer
(`create` / `put` / `finalize`), so the output is byte-for-byte a normal `.spqv`.
**No new dependency** — the `.npy` header is parsed by hand — and the import functions need
`std::fs`, so they are compiled off the wasm target (`#[cfg(not(target_arch = "wasm32"))]`);
nothing in the default build pulls extra weight.

**Row → dict-id contract.** An external matrix carries no term identity, so you supply a
**parallel slice of dictionary ids** (`ImportSpec::ids`): row `i` of the matrix is stored for
`ids[i]`. `ids.len()` must equal the matrix's row count. Resolve each id with `graph.id_of(&term)`;
the natural source is the same scan that produced the texts you embedded — emit `(id, text)`
pairs, embed the texts, then import the matrix with the ids in that same order.

**Supported formats.**

- **NumPy `.npy`** (`import_npy`) — a **2-D, C-order, little-endian `f4`/`f8`** array, i.e. what
  `numpy.save(path, arr.astype(np.float32))` writes. `f8` rows are narrowed to `f32` (the store is
  `f32`). `shape` must be `(ids.len(), dim)`.
- **Header-less flat dump** (`import_numeric_dump`) — exactly `rows · dim · 4` bytes of contiguous
  row-major little-endian `f32`, i.e. `arr.astype("<f4").tofile(path)` or raw `Vec<f32>` bytes from
  any language. `rows` is taken from `ids.len()` and `dim` from the spec, so the file length is
  fully determined and checked.

```rust,ignore
use sparq_vectors::{ImportBinding, ImportSpec, VectorStore};

// row i of vecs.npy lines up with ids[i]
let ids: Vec<u32> = vec![/* alice */ 10, /* bob */ 20, /* carol */ 30];

// (a) NumPy .npy — bind the store to its source graph for the staleness guard.
let store = VectorStore::import_npy(
    ImportSpec { spqv_path: "graph.spqv", dim: 384, ids: &ids, binding: ImportBinding::Graph(&graph) },
    "vecs.npy",
)?;

// (b) header-less flat f32 dump — Unbound when no graph is on hand.
let store = VectorStore::import_numeric_dump(
    ImportSpec { spqv_path: "graph.spqv", dim: 384, ids: &ids, binding: ImportBinding::Unbound },
    "vecs.f32",
)?;
```

**Fail-closed.** A dtype / byte-order / fortran-order / dimensionality mismatch, a `shape[1] != dim`,
a `shape[0] != ids.len()`, a wrong dump length, a duplicate / zero / non-finite row, or a
malformed / oversized `.npy` header is an `Err` — never a silent reinterpretation or truncation. The
`.npy` declared header length is bounded (`MAX_NPY_HEADER_LEN`, 64 KiB) and the declared shape is
checked against the real file size **before any body allocation** (the sq-tzwa bounded-read
discipline), so a hostile header cannot force a large pre-allocation. An empty matrix
(`shape (0, dim)` / a 0-byte dump with an empty `ids`) is a valid no-op import that yields a
well-formed empty store.

**`ImportBinding`.** `Graph(&graph)` writes the graph fingerprint into the store header so
`check_graph` can later reject a stale store after a dict-id-shifting rebuild; pass the SAME graph
whose ids the rows are keyed by. `Unbound` leaves the store unverifiable (use only when no graph is
on hand at import time).

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
