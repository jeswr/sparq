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
  degenerate inputs.
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
- **`embed_labels`** — embeds one human-readable label per entity
  (`rdfs:label` > `skos:prefLabel` > `foaf:name` > `schema:name` >
  `dc:title`, configurable via `LabelConfig`), scanning each predicate's
  contiguous index block rather than the whole graph.

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

## Usage

```rust
use sparq_core::Graph;
use sparq_vectors::{embed_labels, HashEmbedder, VectorIndex, VectorStore};

let graph = Graph::load_str(ttl, "turtle")?;
let embedder = HashEmbedder::new(64); // test-only; bring your own Embedder
let mut store = VectorStore::create("graph.spqv", 64)?;
embed_labels(&graph, &mut store, &embedder)?;
store.finalize()?; // writes the file, handle becomes mmap-backed

let index = VectorIndex::build(&store); // HNSW, rebuilt per process
let neighbours = index.nearest_term(&some_term, &graph, &store, 10);
```

## Accuracy gate

`tests/recall.rs` asserts **HNSW recall@10 ≥ 0.95 vs exact brute force** on
50 000 random 32-d vectors (100 queries), as an ordinary `cargo test` (the
workspace raises the dev opt-level for this crate so the gate stays fast).

Measured: **recall@10 = 0.998–0.999** with the default `HnswConfig`
(`ef_search = ef_construction = 100`, fixed seed).

## Throughput

Measured by `tests/throughput.rs` —
`cargo test -p sparq-vectors --release --test throughput -- --ignored --nocapture` —
on an Apple M1, 50 000 × 32-d vectors, k = 10, 200 queries. Ranges span two
runs (quiet vs heavily loaded machine); rerun the harness on your hardware:

| Operation                  | Time            |
| -------------------------- | --------------- |
| `put` 50k vectors (RAM)    | 7–10 ms         |
| `finalize` (write + mmap)  | 7–22 ms         |
| `open` (mmap + index validation) | 0.6–0.9 ms |
| HNSW build (rayon)         | 33–83 s         |
| exact top-10 (full scan)   | 6.5–11.8 ms/query (85–150 q/s) |
| HNSW top-10                | 1.6–2.5 ms/query (400–630 q/s) |

Below ~10⁵ vectors the exact scan is a fine default; the HNSW index pays for
its build once query volume is high. The index is rebuilt from the mmap'd
store per process rather than persisted; out-of-core persistent ANN
(DiskANN-style) is the recorded follow-up for 10M+ stores.

## Tests

- `tests/store.rs` — `.spqv` round-trip, sparse/unsorted ids, build-phase
  error paths, garbage/truncated/overflowing-header/corrupt-index rejection.
- `tests/recall.rs` — the recall@10 ≥ 0.95 gate plus exact/HNSW agreement on
  separable clusters.
- `tests/labels.rs` — `embed_labels` predicate priority, literal filtering,
  and term-keyed lookup end to end on a small graph.
- `tests/olympics.rs` — the real 1.78M-triple olympics dataset (137k labels):
  embed → finalize → HNSW → `nearest_term`, with a same-type sanity check on
  the neighbours. **Skips (passes with a stderr note) when
  `bench/qlever-olympics/olympics.nt` is absent**; override the path with
  `SPARQ_OLYMPICS_NT`.
- `tests/throughput.rs` — the `#[ignore]`d measurement behind the table above.
