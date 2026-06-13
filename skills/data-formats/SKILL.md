---
name: data-formats
description: Parse and load RDF into a sparq Graph (Turtle/N-Triples/N-Quads/TriG via sparq-core, and HDT incl. compressed .hdt.gz/.hdt.zst/.hdt.bz2 via sparq-hdt), do streaming/parallel/external-memory ingest of compressed dumps, and take cheap immutable copy-on-write Graph snapshots. Use when ingesting RDF files, choosing a loader, wiring HDT, or snapshotting a graph for serving.
---

# sparq data formats

How to get RDF *into* a sparq `Graph` and how to snapshot one cheaply. Text formats
(Turtle / N-Triples / N-Quads / TriG) and the in-memory + streaming + external-memory
loaders live in `sparq-core`; the binary HDT archive format (including content-sniffed
`.hdt.gz` / `.hdt.zst` / `.hdt.bz2`) lives in the opt-in `sparq-hdt` crate.

> Direction note: these crates **parse RDF in**. sparq-core ships no RDF *text serializer*
> — to write RDF out, iterate `Graph::iter_ids()`, materialize terms with `Dict::term`,
> and feed them to an `oxttl` serializer (oxttl is already a dependency). See the recipe.

## Quickstart

Add the dependency (HDT is a separate, native-only crate):

```toml
# Cargo.toml
[dependencies]
sparq-core = "0.1"            # default features include `parallel` (native)
sparq-hdt  = "0.1"            # OPTIONAL — only if you load .hdt archives
oxrdf      = { version = "0.3", features = ["rdf-12"] }  # for Term/NamedNode at call sites
```

```rust
use sparq_core::Graph;

// `format`: "turtle"|"ttl" | "ntriples"|"n-triples" | "nquads"|"n-quads" | "trig"|"application/trig"
let ttl = r#"@prefix ex: <http://ex/> .
ex:alice ex:knows ex:bob ."#;
let g = Graph::load_str(ttl, "turtle").expect("parse");
assert_eq!(g.len(), 1);

// HDT archive (sniffs .hdt.gz/.hdt.zst/.hdt.bz2 by magic bytes, not file name):
// let g = sparq_hdt::load("dataset.hdt").unwrap();
```

Or from the CLI:

```bash
sparq-cli query data.ttl turtle 'SELECT * WHERE { ?s ?p ?o } LIMIT 5'
# HDT needs the opt-in feature (its MSRV is 1.87, above the 1.85 workspace floor):
cargo build -p sparq-cli --features hdt
./target/.../sparq-cli query dataset.hdt hdt 'SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'
```

## Key APIs

`sparq_core::Graph` — construction / ingest (all take a `format: &str`):

```rust
// In-memory, whole document in a &str. Parallel-chunked for N-Triples & Turtle
// when the `parallel` feature is on (default native).
pub fn load_str(text: &str, format: &str) -> Result<Graph, String>
pub fn load_str_with_base(text: &str, format: &str, base: &str) -> Result<Graph, String>

// Dataset load (N-Quads / TriG) preserving NAMED GRAPHS as separate sub-graphs
// (so `GRAPH ?g {…}` works). Other formats defer to load_str. In-memory only.
pub fn load_dataset(text: &str, format: &str) -> Result<Graph, String>

// Streaming from any reader (e.g. a decompression stream) — serial.
pub fn load_reader<R: std::io::Read>(reader: R, format: &str) -> Result<Graph, String>

// Streaming + PARALLEL (needs `parallel`): pipelined 32 MiB block parse, never
// materializes the whole decompressed doc. N-Triples gets the fast path; other
// formats fall back to serial load_reader.
pub fn load_reader_parallel<R: std::io::Read + Send>(reader: R, format: &str) -> Result<Graph, String>

// Parse WITHOUT building indexes — the seam reasoning/transform hooks use.
pub fn parse_to_triples(text: &str, format: &str) -> Result<(Dict, Vec<[Id; 3]>), String>
pub fn from_parts(dict: Dict, triples: Vec<[Id; 3]>) -> Graph

// EXTERNAL-MEMORY build (needs `mmap`): stream-parse, spill sorted runs to disk,
// k-way merge — bounded RAM for datasets larger than memory. `chunk` = triples per run.
pub fn build_external<R: std::io::Read + Send>(reader: R, format: &str, dir: &Path, chunk: usize) -> Result<(), String>
```

`sparq_core::Graph` — cheap snapshots / forks (the serving pattern):

```rust
pub fn snapshot(&self) -> GraphSnapshot   // immutable COW view; O(pending delta), not O(triples)
pub fn fork(&self) -> Graph               // mutable structural fork (Arc-shares base storage)
```

`sparq_core::GraphSnapshot` — `Send + Sync`, `Deref<Target = Graph>` (every read method:
`len`, `id_of`, `pattern`, `iter_ids`, `store`/`dict`), **no** mutating methods.
`into_graph(self) -> Graph` / `as_graph(&self) -> &Graph`.

`sparq_hdt` (native-only; opt-in):

```rust
pub fn load(path: impl AsRef<Path>) -> Result<Graph, Error>         // sniffs .hdt.gz/.hdt.zst/.hdt.bz2
pub fn load_reader<R: BufRead>(reader: R) -> Result<Graph, Error>   // any buffered source
pub fn header(path: impl AsRef<Path>) -> Result<Graph, Error>       // HDT header metadata as a Graph
pub fn header_reader<R: BufRead>(reader: R) -> Result<Graph, Error>
// also: load_reader_via_upstream (differential oracle), graph_from_hdt, graph_from_reader
```

## Common recipes

**1. Load a compressed N-Triples dump with the fast parallel streaming path.**
Decompress on the fly and parse in parallel without ever holding the full text:

```rust
use sparq_core::Graph;
let file = std::fs::File::open("dump.nt.gz")?;
let reader = flate2::read::MultiGzDecoder::new(file);     // .bz2 -> bzip2::read::MultiBzDecoder
                                                          // .zst -> zstd::stream::read::Decoder::new
let g = Graph::load_reader_parallel(reader, "ntriples")?; // N-Triples gets the pipelined parser
```

**2. Load a dataset with named graphs (TriG / N-Quads).**

```rust
let trig = r#"<http://ex/g> { <http://ex/a> <http://ex/p> <http://ex/b> . }"#;
let g = sparq_core::Graph::load_dataset(trig, "trig")?;
assert_eq!(g.named.len(), 1);          // each named graph is its own sub-Graph
```

**3. Load an HDT archive (and read its header metadata).**

```rust
let g    = sparq_hdt::load("dataset.hdt")?;        // .hdt / .hdt.gz / .hdt.zst / .hdt.bz2 (by magic bytes)
let meta = sparq_hdt::header("dataset.hdt")?;      // VoID stats / provenance as a queryable Graph
// from memory: sparq_hdt::load_reader(std::io::Cursor::new(bytes))?
```

**4. Out-of-core build for a dataset larger than RAM** (requires `sparq-core` `mmap` feature):

```rust
use std::path::Path;
let reader = flate2::read::MultiGzDecoder::new(std::fs::File::open("huge.nt.gz")?);
// stream-parse + external-sort the six permutations onto disk; ~16M triples per spill run:
sparq_core::Graph::build_external(reader, "ntriples", Path::new("/data/idx"), 16_000_000)?;
let g = sparq_core::Graph::open(Path::new("/data/idx"))?;   // query with indexes memory-mapped
```

**5. Cheap snapshot for serving (one mutable master + immutable readers per commit).**
`snapshot()` Arc-shares the base indexes/dictionary — it copies neither, and later
mutation of the master is invisible to the snapshot:

```rust
let mut master = sparq_core::Graph::load_str(doc, "turtle")?;
let snap = master.snapshot();          // immutable, Send+Sync, point-in-time
master.apply_delta(&inserts, &deletes)?;   // master moves on
assert_eq!(snap.len(), original_len);  // snapshot frozen at snapshot time
// publish `snap` to query threads; it Derefs to &Graph
let mutable_copy = master.snapshot().into_graph();  // a snapshot you can then mutate
```

**6. Serialize a graph back out to N-Triples (no built-in serializer — use oxttl).**

```rust
use oxrdf::{Triple, Subject, NamedOrBlankNode};
use oxttl::NTriplesSerializer;
let mut out = Vec::new();
let mut w = NTriplesSerializer::new().for_writer(&mut out);
for [s, p, o] in g.iter_ids() {                 // dictionary-id triples, S,P,O order
    let (st, pt, ot) = (g.dict.term(s), g.dict.term(p), g.dict.term(o));
    // reconstruct an oxrdf::Triple from the terms (match on NamedNode/BlankNode/Literal)
    // then: w.serialize_triple(&triple)?;
}
w.finish()?;
```

## Gotchas / feature flags / prerequisites

- **Format strings are matched literally.** Accepted: `"turtle"`/`"ttl"`,
  `"ntriples"`/`"n-triples"`, `"nquads"`/`"n-quads"`, `"trig"`/`"application/trig"`.
  `parse_to_triples` (and the `_with_base` variants) treat any **unknown** format as
  Turtle (the `_ =>` arm) — pass the exact string.
- **Parallel paths need the `parallel` feature** (on by default natively). The parallel
  fast path applies to N-Triples and Turtle in `load_str`; `load_reader_parallel`'s
  pipelined parser is **N-Triples only** (other formats silently fall back to serial
  `load_reader`). With `parallel` off (e.g. the wasm build, `--no-default-features`),
  everything parses serially.
- **`build_external` / `open` / `save` require the `mmap` feature** (native only). The
  N-Triples external path also honors `SPARQ_SHARDED_DICT` (default on with ≥2 threads)
  and, with the `dict-spill` feature + `SPARQ_DICT_SPILL` env, bounds peak build RSS by
  spilling the term dictionary to disk (byte-identical output). External build folds
  N-Quads/TriG named graphs into the default graph (only `load_dataset` preserves them).
- **HDT is opt-in and native-only.** `sparq-hdt` MSRV is **1.87** (the wrapped `hdt` crate),
  above the workspace's 1.85 — in the CLI it is gated behind `--features hdt`. It carries
  zero code into the wasm build. Compression containers are detected by **magic bytes, not
  file extension**, so a mislabeled `.hdt` still loads; all three (`gz`/`zst`/`bz2`) decode
  in a streaming fashion.
- **HDT format coverage:** standard v1.0 layout only (FourSectionDictionary / Plain Front
  Coding + BitmapTriples, SPO order) as emitted by hdt-cpp / hdt-java. Exotic submission
  layouts are rejected with an error. **Writing HDT is not supported** (blocked upstream).
- **`load_dataset` is in-memory only.** Numeric/temporal filter caches are built on load
  in all in-memory paths; `into_compressed()` / `load_str_compressed()` trade a small
  per-scan decode for ~2.5× more triples per byte of RAM (browser target).

## See also

- `fused-decompress-parse` — choosing gzip vs zstd vs bzip2 and fusing decode with parse (measured numbers).
- `rust-parallel-parsing` — how the chunk-parallel N-Triples/Turtle scanners work, and when NOT to parallelize.
- `hdt-format` — the HDT binary layout internals and the `hdt`-crate wrapping/decode performance.
