---
name: data-formats
description: Parse and load RDF into a sparq Graph (Turtle/N-Triples/N-Quads/TriG via sparq-core, and HDT incl. compressed .hdt.gz/.hdt.zst/.hdt.bz2 via sparq-hdt), do streaming/parallel/external-memory ingest of compressed dumps, and take cheap immutable copy-on-write Graph snapshots. Use when ingesting RDF files, choosing a loader, wiring HDT, or snapshotting a graph for serving.
---

# sparq data formats

How to get RDF *into* a sparq `Graph` and how to snapshot one cheaply. Text formats
(Turtle / N-Triples / N-Quads / TriG) and the in-memory + streaming + external-memory
loaders live in `sparq-core`; the binary HDT archive format (including content-sniffed
`.hdt.gz` / `.hdt.zst` / `.hdt.bz2`) lives in the opt-in `sparq-hdt` crate.

> Direction note: these crates **parse RDF in**. To write RDF *out*, sparq-engine ships the
> **RDF writer matrix** behind its opt-in `serialize-rdf` feature — Turtle / TriG / N-Quads /
> JSON-LD 1.1 writers (`sparq_engine::serialize::*`); the N-Triples writer
> (`triples_to_ntriples`) is always on. See recipe 6.

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
// when the `parallel` feature is on (default native); on ≥2 threads both fan the
// per-chunk dictionaries into one sharded-dict merge (the serial `merge_remap`
// loop is kept only on a single thread). Output is identical to the serial parse.
pub fn load_str(text: &str, format: &str) -> Result<Graph, String>
pub fn load_str_with_base(text: &str, format: &str, base: &str) -> Result<Graph, String>

// Dataset load (N-Quads / TriG) preserving NAMED GRAPHS as separate sub-graphs
// (so `GRAPH ?g {…}` works). Other formats defer to load_str. In-memory only.
// N-Quads is parallel-chunked (per-graph routing + sharded-dict merge) when the
// `parallel` feature is on (default native); TriG uses the serial path. Named
// graphs come out in first-occurrence document order (deterministic).
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

// WRITE (opt-in `write` feature): Graph -> .hdt (honours .gz/.zst/.bz2 by extension)
#[cfg(feature = "write")]
pub fn save(graph: &Graph, path: impl AsRef<Path>) -> Result<(), Error>
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
// WRITE back (opt-in `write` feature): sparq_hdt::save(&g, "out.hdt.gz")?;
//   (currently a temp-N-Triples round-trip through the wrapped builder — see
//    sparq-hdt/UPSTREAM.md for the queued in-memory-builder contribution)
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

**6. Serialize a graph back out (the RDF writer matrix).** The Turtle / TriG / N-Quads /
JSON-LD writers live in `sparq-engine` behind the opt-in **`serialize-rdf`** feature (it pulls
in ZERO new dependencies — the default dep graph is byte-for-byte unchanged when off, *and*
unchanged even when on: the JSON-LD writer emits JSON by hand, no json-ld/serde crate). The
N-Triples writer (`triples_to_ntriples`) is always on. Enable with
`sparq-engine = { version = "0.1", features = ["serialize-rdf"] }`.

```rust
use sparq_engine::serialize::{graph_to_turtle, graph_to_trig, graph_to_nquads,
                              graph_to_jsonld, JsonLdForm};

let g = sparq_core::Graph::load_dataset(trig_src, "trig")?;
let ttl = graph_to_turtle(&g);   // Turtle: @prefix header, `a` for rdf:type, predicate-object
                                 //   lists; DEFAULT graph only.
let tg  = graph_to_trig(&g);     // TriG: default graph + `GRAPH <g> { … }` blocks (whole dataset).
let nq  = graph_to_nquads(&g);   // N-Quads: default graph (3 cols) + named graphs (4th column).
let jx  = graph_to_jsonld(&g, JsonLdForm::Expanded);    // JSON-LD 1.1, fully-expanded node-object
                                                        //   array, no @context (whole dataset).
let jf  = graph_to_jsonld(&g, JsonLdForm::Flattened);   //   node-merged, `@graph`-framed.
let jc  = graph_to_jsonld(&g, JsonLdForm::Compacted);   //   basic prefix `@context` (default_prefixes).
```

Lower-level entry points take `&[oxrdf::Triple]` (e.g. CONSTRUCT output) directly:
`write_turtle(triples, &prefixes)`, `write_trig(&named_graphs, &prefixes)`,
`write_nquads(&named_graphs)`, `write_jsonld(&named_graphs, JsonLdForm::Expanded, &prefixes)`;
`default_prefixes()` supplies the common namespaces, or pass your own `Prefixes` (a
`BTreeMap<String, String>`) — only prefixes actually used are emitted (the Turtle/TriG header,
or the JSON-LD compacted `@context`). Round-trip (parse → serialize → re-parse) is isomorphic
for every form. **JSON-LD specifics:** `xsd:string`/`rdf:langString` stay implicit
(`@value` + optional `@language`); every other datatype is preserved as `@type`; canonical
`xsd:integer`/`xsd:boolean` literals coerce to native JSON scalars only when lossless (leading
zeros, `xsd:double`/`decimal`, etc. stay typed strings). RDF lists are emitted as plain
triples (no `@list` collapsing — tracked in bead sq-e3pj follow-up).

From the CLI (opt-in `serialize-rdf` feature) — re-serialize a loaded document to stdout:

```bash
cargo build -p sparq-cli --features serialize-rdf
./target/.../sparq-cli dump data.trig trig nquads
#   out-format: turtle | trig | nquads | ntriples | jsonld[-expanded|-flattened|-compacted]
#   (bare `jsonld` == jsonld-expanded)
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
- **RDF 1.2 triple terms / RDF-star are first-class.** A triple term `<<( s p o )>>` is a
  real `oxrdf::Term::Triple` (object position only — RDF 1.2 makes triple terms object-only;
  a `<<( … )>>` in subject/predicate position is rejected with a precise error). Triple terms
  may nest, take blank-node/literal components, and are content-addressed (an identical triple
  term shares one dict id). They load through **every** path — serial, parallel-chunked,
  streaming-pipelined, and the sharded external builder — at full parallelism (the in-memory
  parallel merge no longer drops to a serial fallback when they are present). The **dict-spill**
  external builder is the one exception: it rejects triple terms with a clear error (its
  content-only on-disk records can't encode them — bead sq-jvbr); use the default sharded path
  or an in-memory load for RDF-star + dict-spill datasets. In **Turtle/TriG**, the SPARQL-1.2
  reification sugar is supported via the Turtle parser: the reifying triple `<< s p o >>`
  (subject or object position, optionally `<< s p o ~ reifier >>`) and the annotation block
  `s p o {| … |}` desugar to the standard `rdf:reifies <<( s p o )>>` form (the annotation block
  also **asserts** the base triple; a bare `<< … >>` reifier does not). N-Triples/N-Quads carry
  only the desugared `<<( … )>>` triple-term form (no `<<>>`/`{| |}` sugar — per the line-format
  grammar). Nested triple terms are **depth-bounded** (`MAX_TRIPLE_TERM_DEPTH = 128` in
  `sparq-core::nt`): the byte-level N-Triples/N-Quads parser is the only native-recursion RDF
  parse path, so a pathologically nested `<<( … )>>` chain returns a clean parse error rather
  than overflowing the stack (ASVS V5.5.2 / sq-53s1). The Turtle/TriG/N-Quads path via `oxttl`
  is a heap-stack pushdown automaton and cannot recurse the native stack. The SPARQL parser has
  the matching `MAX_RECURSION_DEPTH = 128` cap (groups/expressions/paths/collections/triple
  terms). 128 is far deeper than any real data/query nests.
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
  layouts are rejected with an error.
- **Writing HDT is opt-in** (`sparq-hdt`'s `write` feature): `save(&graph, path)` emits the
  same standard v1.0 layout. The path currently round-trips through a **temporary
  N-Triples file** via the wrapped crate's builder (`Hdt::read_nt` + `Hdt::write`, behind
  its `sophia` feature) — correct and interoperable, but it re-serialises + re-parses the
  whole graph, so it is NOT free. A direct in-memory builder (no text round-trip) is queued
  upstream (`sparq-hdt/UPSTREAM.md`). HDT carries a single default graph, so `save` ignores
  named graphs.
- **`load_dataset` is in-memory only.** Numeric/temporal filter caches are built on load
  in all in-memory paths; `into_compressed()` / `load_str_compressed()` trade a small
  per-scan decode for ~2.5× more triples per byte of RAM (browser target).

## See also

- `fused-decompress-parse` — choosing gzip vs zstd vs bzip2 and fusing decode with parse (measured numbers).
- `rust-parallel-parsing` — how the chunk-parallel N-Triples/Turtle scanners work, and when NOT to parallelize.
- `hdt-format` — the HDT binary layout internals and the `hdt`-crate wrapping/decode performance.
