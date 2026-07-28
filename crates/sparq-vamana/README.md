# sparq-vamana

A **persistent, memory-mapped Vamana/DiskANN approximate-nearest-neighbour index** in pure Rust.
Build the proximity graph once, write it to a `.spqg` file, and reopen it forever after with an
`mmap` + a header check — **no rebuild on process start**.

It has **no required dependencies** and knows nothing about RDF, SPARQL, or any storage engine:
you hand it vectors through a one-method `VectorSource` trait and it hands back `(id, cosine)`
pairs. (It was extracted from [`sparq-vectors`](../sparq-vectors), which now consumes it.)

## 🚀 Quickstart

```toml
[dependencies]
sparq-vamana = "0.1"
```

```rust
use sparq_vamana::{SliceVectors, VamanaConfig, VamanaIndex};

# fn main() -> Result<(), String> {
// 4 two-dimensional vectors, row-major, with caller-chosen u32 ids.
let src = SliceVectors::new(
    2,
    vec![10, 11, 12, 13],
    vec![1.0, 0.0,  0.9, 0.1,  0.0, 1.0,  0.1, 0.9],
)?;

let path = std::env::temp_dir().join("quickstart.spqg");
// Build + persist once (no staleness token — see "Staleness" below).
let index = VamanaIndex::build(&src, &path, VamanaConfig::default(), None)?;
assert_eq!(index.nearest(&[1.0, 0.0], 1)[0].0, 10);

// ...in a later process: reopen with no rebuild.
let reopened = VamanaIndex::open(&path)?;
assert_eq!(reopened.len(), 4);
# std::fs::remove_file(&path).ok();
# Ok(())
# }
```

## ✨ Features

- **Persistent Vamana graph.** RobustPrune with the DiskANN two-pass α schedule, greedy beam
  search, and a versioned little-endian `.spqg` layout that co-locates each node's vector and its
  adjacency in one record — one contiguous read per visited node.
- **Reopen without rebuilding.** `VamanaIndex::open` is an `mmap` plus header/size validation.
  `open_from_bytes` is the same validator over an owned, f32-aligned buffer for environments with
  no filesystem (wasm); `memmap2` is target-gated out of `wasm32` builds entirely.
- **Product quantization (`quant`).** `ProductQuantizer` (k-means++ codebooks, ADC
  `DistanceTable`) and `ScalarQuantizer`, usable on their own or wired into the index by
  `VamanaIndex::build_with_pq` — DiskANN's *search on PQ in RAM, re-rank on disk* loop, persisted
  as a backwards-compatible trailing `.spqg` section.
- **Opt-in filtered traversal** (`filtered` feature, off by default, adds no dependency).
  `nearest_filtered_by` walks the graph predicate-agnostically for connectivity but accepts only
  ids your predicate admits (ACORN / NaviX style).
- **Bring your own vectors.** Implement `VectorSource` (`dim` / `len` / `iter`) over whatever you
  already have, or use the bundled `SliceVectors`.

### Honest scope

- Recall is **approximate** and depends on `VamanaConfig` (`degree`, `search_beam`, `alpha`); it
  is not an exact k-NN. Measure it on *your* data — this crate's own gate asserts recall@10 against
  a brute-force oracle on a synthetic set, which is a floor, not a promise about your workload.
- The graph **build is single-threaded**; only the open is cheap. Building over millions of
  vectors takes real time.
- The format is **little-endian only** (`.spqg` rejects big-endian targets), and an index is valid
  only against the exact vector generation it was built over.

### Staleness

Node records store your ids and neighbour entries are stored *slots*, so an index served against a
different generation of your data silently resolves the wrong vectors. This crate cannot know what
"the same generation" means for you, so the header carries an **opaque 24-byte `StalenessToken`**
you supply at build time and compare yourself after `open` (`index.staleness_token()`). An index
built without one reads back as `None` — *unverifiable*, never a false match.

## 📚 Learn more

- Full API: `cargo doc -p sparq-vamana --all-features --open`.
- The algorithm: Jayaram Subramanya et al., *DiskANN: Fast Accurate Billion-point Nearest
  Neighbor Search on a Single Node*, NeurIPS 2019.
- The RDF-keyed consumer (dictionary term ids, graph fingerprints, SPARQL `vec:` predicates):
  [`sparq-vectors`](../sparq-vectors) and `skills/vector-search/SKILL.md`.

## License

MIT — see the workspace [LICENSE](../../LICENSE).
