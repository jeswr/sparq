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
> JSON-LD 1.1 writers (`sparq_engine::serialize::*`), plus deterministic **pretty** (indented)
> variants — Turtle / TriG (`graph_to_turtle_pretty`, the long-term engine home for the site
> formatter) and JSON-LD (`graph_to_jsonld_pretty`); the N-Triples writer (`triples_to_ntriples`)
> is always on. See recipe 6.

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

// `format` (plus media-type aliases): "turtle"|"ttl"|"text/turtle"|"application/turtle" |
//   "ntriples"|"n-triples"|"nt"|"application/n-triples" | "nquads"|"n-quads"|"nq"|"application/n-quads" |
//   "trig"|"application/trig". An UNRECOGNISED format string is an `Err` — NOT silently
//   parsed as Turtle (sq-m2pc). JSON-LD ("jsonld"/"json-ld"/"application/ld+json") needs the
//   `jsonld` feature on the `sparq-core` LIBRARY dep (OFF by default — the library stays lean);
//   without it those strings also error rather than mis-parsing. [OPUS-4.8] sq-oy1f.4: the
//   `sparq-cli` and `sparq-server` BINARIES enable `jsonld` by DEFAULT (a maintainer-directed
//   exception), so they read/write JSON-LD out of the box; a library embedder opts in explicitly.
//   [OPUS-4.8] sq-f47w1 (survey §B1): RDF/XML ("rdfxml"/"rdf-xml"/"application/rdf+xml")
//   likewise needs the OPT-IN `rdfxml` feature on `sparq-core` (OFF by default — it links
//   `oxrdfxml`/`quick-xml`, kept off the lean wasm bundle); without it those strings error
//   rather than mis-parsing. RDF/XML has no named-graph syntax, so it loads via `load_str` /
//   `parse_to_triples` (not the dataset path), serial-only (XML is not line-delimited).
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
// materializes the whole decompressed doc. N-Triples gets this parallel pipelined
// path; other formats fall back to serial load_reader. (This is the PARALLEL-LOADER
// fast path — distinct from the byte-level IRI-validation fast path noted below, which
// is a separate, opt-in mechanism that covers N-Triples AND N-Quads.)
pub fn load_reader_parallel<R: std::io::Read + Send>(reader: R, format: &str) -> Result<Graph, String>

// [OPUS-4.8] sq-7d3dj.18 — the byte-level N-Triples/N-Quads fast path does NOT validate IRIs
// against RFC-3987 by default (it trusts input; the serial oxttl path DOES validate). The
// OPT-IN `iri-fast` feature on `sparq-core` (OFF by default) turns that validation on for the
// byte parser via a prefix-memoized fast path IN FRONT of `oxiri` — a last-N validated
// `scheme://authority/` memo + a one-pass ASCII `iunreserved`/sub-delims suffix scan, falling
// back to the full `oxiri` automaton on any non-ASCII / `%`-escape / `#` / delimiter anomaly.
// It accepts EXACTLY what `oxiri::Iri::parse` accepts (an absolute IRI) — a MANDATORY
// differential-fuzz gate (`sparq_core::iri`, run under `--features iri-fast`) asserts
// `fast_accept ⇒ oxiri` and `validate == oxiri` over a committed adversarial corpus + a
// deterministic generator, so a malformed IRI can never be wrong-accepted into the store.
// The `oxiri` dep is already in-tree (via oxttl), so the feature adds zero new compilation.

// Parse WITHOUT building indexes — the seam reasoning/transform hooks use.
pub fn parse_to_triples(text: &str, format: &str) -> Result<(Dict, Vec<[Id; 3]>), String>
pub fn from_parts(dict: Dict, triples: Vec<[Id; 3]>) -> Graph

// EXTERNAL-MEMORY build (needs `mmap`): stream-parse, spill sorted runs to disk,
// k-way merge — bounded RAM for datasets larger than memory. `chunk` = triples per run.
pub fn build_external<R: std::io::Read + Send>(reader: R, format: &str, dir: &Path, chunk: usize) -> Result<(), String>

// Empty, in-memory graph — the trivial infallible constructor (no `load_str("", …)`
// workaround, no error handling). `default()` and `new()` are interchangeable.
pub fn new() -> Graph                 // also: Graph::default()
```

`sparq_core::Graph` — single-triple mutation from `oxrdf` terms (the ergonomic one-triple
case over `apply_delta`; each position takes anything that converts into a `Term`:
`NamedNode` / `Literal` / `BlankNode` / a `Term`). Set-valued (re-insert / absent-remove is a
no-op) and, for a directory-backed graph, WAL-logged + fsync'd — same semantics as `apply_delta`.
For several triples at once prefer ONE `apply_delta` batch (one WAL append) over a loop:

```rust
use sparq_core::Graph;
use oxrdf::{NamedNode, Literal, Term};

let mut g = Graph::new();
g.insert_triple(
    NamedNode::new_unchecked("http://ex/alice"),
    NamedNode::new_unchecked("http://schema.org/name"),
    Term::Literal(Literal::new_simple_literal("Alice")),
)?;
g.remove_triple(
    NamedNode::new_unchecked("http://ex/alice"),
    NamedNode::new_unchecked("http://schema.org/name"),
    Term::Literal(Literal::new_simple_literal("Alice")),
)?; // absent ⇒ no-op, never an error
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

// MEASUREMENT-ONLY (bench/parse's HDT stage split — NOT a production loader):
// identical decode to graph_from_reader, but records a per-stage wall-clock split.
// Production callers use the plain loaders above; the decode is byte-for-byte the
// same (timing only reads Instant::now() at the existing stage boundaries), so
// decoder behaviour is unchanged.
//   COARSE three: dict (control/header + 4-section PFC decode+intern+merge), scan
//   (triples-section read + SPO id-translation walk), build (Graph::from_parts).
//   FINER (sq-7ge0): dict_{shared,subjects,objects,predicates} are the four per-PFC-
//   section decode walls — in a multi-threaded pool they run CONCURRENTLY so they
//   OVERLAP and need NOT sum to `dict`; dict_merge is the section-order merge. scan
//   splits into scan_read (triples bitmaps/sequences) + scan_walk (the SPO walk),
//   which sum EXACTLY to `scan`.
pub struct StageTimings {
    pub dict: Duration, pub scan: Duration, pub build: Duration,
    pub dict_shared: Duration, pub dict_subjects: Duration, pub dict_objects: Duration,
    pub dict_predicates: Duration, pub dict_merge: Duration,
    pub scan_read: Duration, pub scan_walk: Duration,
}
pub fn graph_from_reader_timed<R: BufRead>(reader: R, t: &mut StageTimings) -> Result<Graph, Error>

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
JSON-LD writers live in `sparq-engine` behind the **`serialize-rdf`** feature (it pulls
in ZERO new dependencies — the default dep graph is byte-for-byte unchanged when off, *and*
unchanged even when on: the JSON-LD writer emits JSON by hand, no json-ld/serde crate). For a
LIBRARY embedder it is opt-in (the engine library default stays lean): enable with
`sparq-engine = { version = "0.1", features = ["serialize-rdf"] }`. The `sparq-cli` and
`sparq-server` BINARIES pull it into their default build via the default-on `jsonld` feature
([OPUS-4.8] sq-oy1f.4), so `dump …` and the server's `application/ld+json` work out of the box.
The N-Triples writer (`triples_to_ntriples`) is always on. Internally the writer matrix is
housed in the `sparq-engine-serialize` sub-crate (`publish = false`, [FABLE-5] sq-6vshe.4) and
re-exported verbatim — the `sparq_engine::serialize::*` paths and feature names above are
unchanged and remain the supported surface.

```rust
use sparq_engine::serialize::{graph_to_turtle, graph_to_trig, graph_to_nquads,
                              graph_to_jsonld, graph_to_jsonld_pretty, JsonLdForm};

let g = sparq_core::Graph::load_dataset(trig_src, "trig")?;
let ttl = graph_to_turtle(&g);   // Turtle: @prefix header, `a` for rdf:type, predicate-object
                                 //   lists; DEFAULT graph only.
let tg  = graph_to_trig(&g);     // TriG: default graph + `GRAPH <g> { … }` blocks (whole dataset).
let nq  = graph_to_nquads(&g);   // N-Quads: default graph (3 cols) + named graphs (4th column).
let jx  = graph_to_jsonld(&g, JsonLdForm::Expanded);    // JSON-LD 1.1, fully-expanded node-object
                                                        //   array, no @context (whole dataset).
let jf  = graph_to_jsonld(&g, JsonLdForm::Flattened);   //   node-merged, `@graph`-framed.
let jc  = graph_to_jsonld(&g, JsonLdForm::Compacted);   //   basic prefix `@context` (default_prefixes).
let jp  = graph_to_jsonld_pretty(&g, JsonLdForm::Expanded); // same docs, indented (multi-line).
```

Lower-level entry points take `&[oxrdf::Triple]` (e.g. CONSTRUCT output) directly:
`write_turtle(triples, &prefixes)`, `write_trig(&named_graphs, &prefixes)`,
`write_nquads(&named_graphs)`, `write_jsonld(&named_graphs, JsonLdForm::Expanded, &prefixes)`;
`default_prefixes()` supplies the common namespaces, or pass your own `Prefixes` (a
`BTreeMap<String, String>`) — `prefixes_from_pairs([(prefix, iri), …])` builds one from an
ordered `(prefix, iri)` pair list (e.g. a query's parsed `PREFIX` lines or a `[[prefix, iri], …]`
array). The `graph_to_*_with` convenience wrappers (`graph_to_turtle_with`, `graph_to_trig_with`,
`graph_to_jsonld_with`, and the pretty `*_with`) take that same map, so the whole-graph path can
also serialise under an explicit prefix policy. Only prefixes actually used are emitted (the
Turtle/TriG header, or the JSON-LD compacted `@context`). Round-trip (parse → serialize → re-parse) is isomorphic
for every form. **JSON-LD specifics:** `xsd:string`/`rdf:langString` stay implicit
(`@value` + optional `@language`); every other datatype is preserved as `@type`; canonical
`xsd:integer`/`xsd:boolean` literals coerce to native JSON scalars only when lossless (leading
zeros, `xsd:double`/`decimal`, etc. stay typed strings). A *well-formed, single-referenced*
`rdf:first`/`rdf:rest`/`rdf:nil` blank-node chain is collapsed into a native `@list` array
(nested lists collapse recursively); anything that fails the conservative safety predicate —
a list cell referenced more than once, carrying an extra predicate, cyclic, or not terminated
by `rdf:nil` — is left as ordinary `rdf:first`/`rdf:rest` triples, so the round-trip stays
lossless either way (the empty list `()` stays an `rdf:nil` reference, never `@list`).

**Pretty Turtle / TriG (idiomatic, deterministic).** Alongside the plain writers, the same
feature exposes a *pretty* variant — `graph_to_turtle_pretty(&g)` / `graph_to_trig_pretty(&g)`,
or the lower-level `write_turtle_pretty(triples, &prefixes, &opts)` /
`write_trig_pretty(&named_graphs, &prefixes, &opts)` with a `PrettyOptions { indent, abbreviate }`
(default: two-space indent, abbreviation on). Two differences from `graph_to_turtle`: (1) output
is **emission-order-independent** — subjects, predicates and objects are sorted by their
canonical N-Triples spelling and `rdf:type` (`a`) sorts first in each block, so the same triple
SET always renders to the same bytes (the plain writer preserves the store's `iter_ids()` order,
which is thread-count-dependent); (2) the **output shape** matches the site's `prettyTurtle`
reshaper (`packages/sparq-client/src/pretty-turtle.ts`, sq-gb4o / #805) — a prefix-alphabetical
used-only `@prefix` header, blank-line-separated subject blocks, the subject on its own line,
predicate lines at one indent (`;`-joined, `.`-terminated), and object lists wrapped onto
continuation lines (`,\n` + two indents). It is round-trip-correct (re-parses to the same triple
set) and reuses the same escaping/term helpers, so literals (datatype/lang, implicit `xsd:string`
dropped), blank nodes, and RDF 1.2 triple terms `<<( s p o )>>` are all reproduced losslessly.
This is the long-term engine home for the site-side TS formatter; the wasm-surface exposure
(so the site can drop its formatter) is a deferred follow-up.

**Pretty (indented) JSON-LD.** The JSON-LD writers have a matching pretty variant —
`graph_to_jsonld_pretty(&g, form)` / `write_jsonld_pretty(&named_graphs, form, &prefixes, &opts)`,
or the `graph_to_jsonld_pretty_with(&g, form, &prefixes, &opts)` wrapper — taking
`JsonLdPrettyOptions { indent }` (default: two-space indent). Unlike the Turtle pretty writer
this is **whitespace-only**: it produces the *exact* minified JSON-LD document and re-indents it
(each `{`/`[` opens an indented level, each member/element on its own line; empty `{}`/`[]` stay
compact). So it reuses the minified writer's already-deterministic first-seen subject/predicate
ordering unchanged, round-trips to the identical RDF, and stays dependency-free (a small
hand-written JSON re-indenter — no `serde_json`, which is dev-only). Indentation is presentation
only; the document content is byte-for-byte the minified form plus newlines/indent.

**Streaming Turtle / TriG (opt-in `streaming-serialization`, implies `serialize-rdf`).** The plain
`write_turtle` / `write_trig` build a *full* in-memory `String` for the whole graph, so a large
CONSTRUCT/DESCRIBE holds both the materialised triples and the fully-rendered string at once. The
opt-in `streaming-serialization` feature adds `write_turtle_streaming(triples, &prefixes, &mut w)`
and `write_trig_streaming(&named_graphs, &prefixes, &mut w)` (plus the whole-graph
`graph_to_turtle_streaming(&g, &prefixes, &mut w)` / `graph_to_trig_streaming`) that render the body
directly into any `W: std::io::Write`, buffering only **one subject block at a time** (emitting on
subject change) — so the whole rendered output is never materialised, enabling HTTP chunked
CONSTRUCT/DESCRIBE responses (first bytes flushed after the first subject, not the last). The
streamed bytes are **byte-identical** to the buffered `write_turtle` / `write_trig` for the same
graph (same used-prefix header, same subject grouping, same ordering): both share the prefix-header
and per-subject-block rendering, and graph-sourced triples are subject-contiguous (`iter_ids()` walks
the SPO permutation), which is the precondition for that equality. The memory / time-to-first-byte
payoff is a measurable hypothesis on the canonical host, not a baked-in number. Zero new deps (std
`io::Write` only). Reproduce: `cargo test -p sparq-engine --features streaming-serialization`.

**Full W3C JSON-LD 1.1 Compaction (caller `@context`).** `JsonLdForm::Compacted` above is the
*lighter* prefix-only `@context` (it just abbreviates IRIs to `prefix:local` CURIEs). For the
real **W3C JSON-LD 1.1 Compaction Algorithm** against a caller-supplied `@context`, use
`graph_to_jsonld_compact(&g, &ctx)` (or the slice-level `write_jsonld_compact(&named_graphs, &ctx)`)
— still hand-rolled and **dependency-free** (a tiny internal `Json` AST — `JsonLdValue` — no
`serde_json`, no json-ld crate). It implements: **term definitions** (`{"name":"http://…/name"}`
or expanded `{"@id":…,"@type":…,"@language":…,"@container":…}`), **`@vocab`** (bare vocab-relative
terms for predicates and `@type` values), **type coercion** (a term `@type` matching a value's
datatype collapses the value object to a bare scalar; `@type:@id`/`@vocab` collapse a node
reference to a bare IRI string), **language coercion** (a term `@language` or the document default
`@language` drops a matching `@language`), **`@container`** — `@set` (always an array, no
compact-arrays), `@list` (strips the `{"@list":…}` wrapper to a bare ordered array — but only the
*pure* `{"@list":…}` value: any co-located non-list sibling stays under the property IRI, never
dropped, sq-oy1f.8), `@language` (a `{lang: value(s)}` map — a value with **no** language tag falls
under the reserved `@none` member rather than being dropped, sq-oy1f.9; several values that share
one language slot accumulate into an **array** so none is overwritten, and a **non-string** value —
a typed/numeric literal — routes to a separate non-language key so a strict processor keeps its
datatype rather than rejecting an invalid language-map value, sq-oy1f.12/.14) and `@index` (a
`{index: value(s)}` map; fromRdf has no per-value index, so values share the reserved `@none`
slot, accumulating into an array, sq-oy1f.14). `@id` / `@graph` containers — which the fromRdf
model cannot losslessly populate — **fall back to the default (no-container) framing** rather than
emit a container map a strict processor rejects (sq-oy1f.14). **`@reverse`** terms (forward edges
whose predicate a reverse term maps relocate onto the object node and are emitted as a **forward
member keyed by the reverse term**, NOT inside an `@reverse` block — a reverse-term key inside an
`@reverse` block would double-invert and a strict processor reads the edge backwards, sq-oy1f.12;
an edge whose object is *not* itself a subject is kept as a forward property, never stripped,
sq-oy1f.10), **`@id`/`@type` keyword aliasing** (`{"id":"@id","type":"@type"}`), and **value + node
+ IRI compaction** against the active context (vocab-relative compaction emits the `@vocab`-stripped
form even when the suffix contains `/` or `#` — only a `:` suffix is kept full, since it is ambiguous
with a compact IRI on read-back, sq-oy1f.11; a plain **literal** value under a `@type:@id`-coerced
term moves to a non-coerced key so it does not read back as a node IRI, sq-oy1f.13). Build the
context with `parse_context_json(r#"{…}"#)` (a string → `JsonLdValue`, returns `None` if not a JSON
object) or construct the `JsonLdValue` directly; the parsed `ActiveContext` drives compaction. The
output is a `{"@context":…,"@graph":[…]}` document and the compaction is **lossless** — every
coercion is invertible against the same `@context`, so a JSON-LD-to-RDF round-trip reconstructs the
original triples. *Scope:* this is the fromRdf-then-compact (serialise) path — sparq always emits
RDF, so the input is a `Graph`, not an arbitrary remote document; scoped/typed contexts,
`@propagate`, remote `@context` fetching, `@import`, `@protected` are out of scope (JSON-LD
**Framing** is its own recipe below, sq-oy1f.17). *Strict third-party faithfulness:* the compaction
round-trip is verified both against sparq's own JSON-LD→RDF reader **and** differentially against the
**pyld** W3C reference processor (`expand`/`toRdf`) for the `@reverse`, language-map, `@type:@id`,
and multi-value-container shapes (sq-oy1f.12/.13/.14), so the emitted document re-expands to the
original triples in a strict external processor, not only in sparq. (An `@id`/`@graph`-container
*input* — i.e. compacting an already-indexed document, as distinct from the fromRdf graph sparq
produces — remains out of scope and falls back to default framing as noted above.)

*Native document-level pipeline (in progress, opt-in, not yet wired to the surfaces).* The
scope gaps above (scoped/typed contexts, `@propagate`, `@protected`, remote `@context` +
`@import`) are being closed by a from-scratch, dependency-free **document-level** pipeline in
the `sparq-jsonld` crate (design record `research/jsonld-1.1-design.md`, epic sq-oy1f). Its
Context Processing foundation (sq-oy1f.24) ships today: `sparq_jsonld::context::ActiveContext`
with `process()` (Context Processing + Create Term Definition, fallible with exact spec error
codes), `expand_iri()` (IRI Expansion), RFC 3986 reference resolution, and **deny-by-default**
remote-context loading via `DocumentLoader` (`NoopLoader` refuses every fetch — no ambient
network — while `FsLoader` serves trusted local fixtures). The document-level **Expansion
Algorithm** (sq-oy1f.25) ships too: `sparq_jsonld::expand(&input, &opts, &loader)` returns the
canonical expanded array — Value Expansion, scoped (property/type) contexts,
`@index`/`@id`/`@type`/`@language`/`@graph` container maps, `@nest`, `@reverse`, `@included`,
`@json` literals, keyword aliases, and the drop-null + array-normalisation rules — raising the
exact spec error code on invalid input and threading `frameExpansion`. Compaction / flattening /
framing on this substrate, the surface wiring, and the conformance-lane switch to the normative
document-level expand oracle land in later beads; the RDF-first writers above remain the shipped
emit path until then.

```rust
use sparq_engine::serialize::{graph_to_jsonld_compact, parse_context_json};
let ctx = parse_context_json(r#"{
    "@vocab": "http://schema.org/", "id": "@id", "type": "@type",
    "knows": {"@id": "http://schema.org/knows", "@type": "@id"}
}"#).expect("a JSON object");
let doc = graph_to_jsonld_compact(&g, &ctx); // term defs + @vocab + @type:@id coercion applied
// Indented (multi-line) variant — whitespace-only over the minified document, same triples:
// graph_to_jsonld_compact_pretty(&g, &ctx, &JsonLdPrettyOptions::default()).
```

Exposed on the wasm `Store` as `serializeCompact(context, pretty, indent?)` (the JSON-LD
SKILL note) and on the CLI as the `jsonld-compact[-pretty]` `dump` out-format (sq-oy1f.5).

**Document-level JSON-LD 1.1 pipeline — the `sparq-jsonld` crate (epic sq-oy1f, in progress).**
The RDF-first writers above have a hard conformance ceiling (scoped/typed contexts, `@nest`,
`@index` maps, negative tests — structures an RDF projection has already erased). The full
document-level pipeline (Context Processing, Expansion, Flattening, Compaction, Framing,
from/to-RDF) is being built in the new **opt-in, dependency-free `sparq-jsonld` crate**
(`crates/sparq-jsonld`, design record `research/jsonld-1.1-design.md`). Phase A (sq-oy1f.23)
scaffolds it: the writer's `Json` AST moved out of `sparq-engine` into `sparq_jsonld::Json`
(public API preserved — `sparq_engine::serialize::JsonLdValue` re-exports it, byte-identical
output), alongside the full `JsonLdErrorCode` registry, `JsonLdOptions`, and a **deny-by-default**
`DocumentLoader` (the default `NoopLoader` refuses every remote load — no ambient network). The
crate is `serialize-rdf`-gated on the engine side, so the lean default + wasm builds are unchanged.
The algorithms land in the follow-on beads (sq-oy1f.24+); until then the writer above is the JSON-LD
emit path.

**W3C JSON-LD 1.1 Framing (caller frame).** `graph_to_jsonld_framed(&g, &frame)` (or the
slice-level `write_jsonld_framed(&named_graphs, &frame)`) reshapes the graph into a deterministic
tree matching a caller **frame** document, then compacts it against the frame's `@context`. Build
the frame with `parse_context_json` (a frame is a JSON-LD document). Hand-rolled and
dependency-free, same `Json` AST as compaction — no `serde_json`, no `json-ld` crate (no Rust
JSON-LD crate ships framing). It implements:

* **Node-pattern matching** — `@type` (explicit list / wildcard `{}` / match-none `[]`), property
  *presence* (`{}`), *absence* (`[]`), specific `@value` / `@id` match, and `@id` selection.
  `@requireAll: true` combines the frame's properties with AND; the default with OR (a `{}` frame matches all).
* **`@embed`** — `@once` (default: embed a node the first time it is reached result-wide, a
  `{"@id":…}` reference thereafter), `@always` (re-embed every reference), `@never` (always a
  reference); `@link` is treated as `@always`. A blank-node cycle (`a→b→a`) terminates via the
  embed link table (the back-edge becomes a reference).
* **`@explicit: true`** — keep only the properties the frame names. **`@default`** — the value
  emitted for a framed property absent from the matched node (`@default: @null` → a preserve-`null`
  marker). **`@omitDefault: true`** — drop an absent framed property instead of emitting its
  default/null. **List framing** keeps the `{"@list":…}` wrapper, framing each element.

Output is a `{"@context":…,"@graph":[…]}` document, collapsing to the bare framed node merged with
`@context` for a single matched root (the `omitGraph` default). Each named graph in the dataset is
framed against the same pattern (wrapped `{"@id":<graph>,"@graph":[…]}`). The framed shape is
**differential-tested byte-for-byte against the pyld reference processor's `frame`** across the
flag matrix (sq-oy1f.17), and **ratcheted against the official `w3c/json-ld-framing` suite**
(sq-oy1f.19; see the conformance bullet below) — each `jld:FrameTest` expanded input is framed and
compared by RDF-equivalence to the normative expected output, which quantifies the real gap. *Scope:*
the serialise (fromRdf-then-frame) path — sparq supplies the graph + an inline frame. The W3C suite
documents the current below-floor divergences honestly (value-pattern matching over `@value`
alternative arrays, some `@explicit`/`@default` and named-graph `@graph`-frame shapes); `@reverse`
frames and scoped/typed frame contexts remain follow-up beads (children of sq-oy1f).

```rust
use sparq_engine::serialize::{graph_to_jsonld_framed, parse_context_json};
let frame = parse_context_json(r#"{
    "@context": {"@vocab": "http://schema.org/"},
    "@type": "Person", "knows": {"@embed": "@never"}
}"#).expect("a JSON object");
let doc = graph_to_jsonld_framed(&g, &frame); // selects Person nodes; knows → bare {"@id":…}
```

From the CLI (opt-in `serialize-rdf` feature) — re-serialize a loaded document to stdout:

```bash
cargo build -p sparq-cli --features serialize-rdf
./target/.../sparq-cli dump data.trig trig nquads
#   out-format: turtle | turtle-pretty | trig | trig-pretty | nquads | ntriples
#              | jsonld[-expanded|-flattened|-compacted]
#              | jsonld-pretty[-expanded|-flattened|-compacted]
#              | jsonld-compact[-pretty]   (FULL W3C Compaction; needs --context <ctx.jsonld>)
#   (bare `jsonld` == jsonld-expanded; `turtle-pretty`/`trig-pretty` emit the deterministic,
#    idiomatic Turtle/TriG from recipe 6 — sorted, blank-line-separated subject blocks;
#    the `jsonld-pretty*` forms emit indented JSON-LD, bare `jsonld-pretty` == expanded)
# Full 1.1 Compaction against your own @context (term defs / @vocab / coercion / @reverse):
./target/.../sparq-cli dump data.ttl turtle jsonld-compact --context ctx.jsonld
```

## Gotchas / feature flags / prerequisites

- **Format strings are matched literally — an unknown format is REJECTED, not
  guessed.** Accepted aliases:
  `"turtle"`/`"ttl"`/`"text/turtle"`/`"application/turtle"`,
  `"ntriples"`/`"n-triples"`/`"nt"`/`"application/n-triples"`,
  `"nquads"`/`"n-quads"`/`"nq"`/`"application/n-quads"`, `"trig"`/`"application/trig"`
  (and the `"jsonld"`/`"json-ld"`/`"application/ld+json"` set when the `jsonld` feature is
  on). `parse_to_triples` (and the `_with_base` variants), `load_dataset`/`load_dataset_serial`
  gate on these explicit alias sets: any unknown/typo'd string returns
  `Err("unknown RDF format \"…\" (known: …)")` naming the bad format — it does **not**
  silently fall back to Turtle/TriG (the old `_ =>` catch-all was removed in sq-m2pc /
  sq-01yr; verify: `cargo test -p sparq-core parse_to_triples_rejects_unknown_format
  load_dataset_rejects_unknown_format`). Pass the exact string.
- **Parallel paths need the `parallel` feature** (on by default natively). The parallel
  fast path applies to N-Triples and Turtle in `load_str`; `load_reader_parallel`'s
  pipelined parser is **N-Triples only** (other formats silently fall back to serial
  `load_reader`). With `parallel` off (e.g. the wasm build, `--no-default-features`),
  everything parses serially. The parallel and serial parses are pinned **result-equivalent**
  by a differential test (`crates/sparq-core/tests/parallel_serial_load_differential.rs`,
  sq-bif.13): the same Turtle + N-Triples document loaded under `--features parallel` and under
  `--no-default-features` answers every term-level probe (triple/term counts, full dump,
  pattern scans) identically to a feature-independent reference graph.
- **RDF 1.2 triple terms are first-class.** A triple term `<<( s p o )>>` is a
  real `oxrdf::Term::Triple` (object position only — RDF 1.2 makes triple terms object-only;
  a `<<( … )>>` in subject/predicate position is rejected with a precise error). Triple terms
  may nest, take blank-node/literal components, and are content-addressed (an identical triple
  term shares one dict id). They load through **every** path — serial, parallel-chunked,
  streaming-pipelined, the sharded external builder, and the **dict-spill** external builder
  (sq-jvbr: triple-term occurrences take an in-RAM arena finalised after the spilled leaf
  consolidation, content-addressed by their components' final ids and assigned ids after every
  leaf) — at full parallelism (the in-memory parallel merge no longer drops to a serial
  fallback when they are present). In **Turtle/TriG**, the SPARQL-1.2
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
- **Language tags are normalised to lowercase on ingest.** A `@lang` tag is stored ASCII-lowercased
  across **every** format and path — Turtle/TriG/N-Quads via `oxttl`, and the byte-level
  N-Triples/N-Quads fast parser (`sparq-core::nt`). RDF 1.1/1.2 language tags are case-insensitive,
  so `"x"@en-US` and `"x"@en-us` are the **same** RDF term and intern to one dict id (matching
  `oxrdf::Literal::new_language_tagged_literal`). This makes ingest format-INDEPENDENT: the same
  triple written as N-Triples or as Turtle yields the identical stored term (it did **not** before —
  the byte path used to preserve the written case while every `oxttl` path lowercased it; KamiQuasi
  #1119 / sq-langcase). The RDF 1.2 `--ltr`/`--rtl` direction is unaffected (already lowercase).
- **`build_external` / `open` / `save` require the `mmap` feature** (native only). The
  N-Triples external path also honors `SPARQ_SHARDED_DICT` (default on with ≥2 threads)
  and, with the `dict-spill` feature + `SPARQ_DICT_SPILL` env, bounds peak build RSS by
  spilling the term dictionary to disk (byte-identical output). The spill path's activation
  + mmap-reload edges are pinned by `crates/sparq-core/tests/dict_spill_activation.rs`
  (sq-bif.13): a tiny `mem_budget` (constant cache eviction across many epochs + many spilled
  sort runs) yields a build byte-identical to a comfortable-budget build, a `disk_floor` above
  free disk aborts cleanly through the spill pipeline's resource gate, and a spill-built store
  reloads via `Graph::open` (and re-saves raw/compressed) to identical content. External build
  folds N-Quads/TriG named graphs into the default graph (only `load_dataset` preserves them).
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
- **`block-bloom` (opt-in, OFF by default).** A block-compressed permutation already has an
  implicit min/max zone map (the directory's first-triple) that prunes RANGE scans, but for an
  EQUALITY-BOUND leading column (a point/prefix lookup on a constant subject/object) whose id
  overlaps several blocks' `[min,max]` spans on a high-NDV column, only a per-block Bloom filter
  can skip the blocks that cannot contain it. The `block-bloom` feature builds a tiny per-block
  Bloom bitset over each block's distinct leading ids (built only for high-NDV columns past a
  density gate; pure-std, no new dep) and probes it in `range` before decoding a block. Zero
  false negatives by construction, so results are IDENTICAL to the feature-off path — only fewer
  blocks decode. The filter is in-RAM/build-time only: it is NEVER written to the on-disk
  `SPQCPRM1` format, so a perm built with the feature on persists byte-identically to one built
  with it off, and a memory-mapped perm carries no filter. Whether the hypothesised block-skip
  win materialises is to be confirmed on the canonical perf host (no number is asserted here).
- **`spqcprm2` (opt-in EXPERIMENT, OFF by default).** A block-encoding SPIKE, not a shipped
  format. The default save/open path still writes and auto-detects ONLY the `SPQCPRM1` on-disk
  format (`compress::FILE_MAGIC`); `CompressedPerm::encode` / `from_mmap` always produce a `V1`
  perm. The feature adds an alternate col2-reset encoding under a frame-of-reference scheme: where
  `SPQCPRM1` writes a middle-column-reset col2 id ABSOLUTE, `SPQCPRM2` writes it as a zigzag signed
  delta from the block's first-row col2 (the frame origin the decoder recovers at block start), so
  clustered objects encode as short varints. It surfaces three public items on `sparq_core::compress`
  **only when the feature is on**: `enum Format { V1, V2 }` (which block reader a `CompressedPerm`
  decodes through — a perm tracks its own `format`), `fn encode_v2` (builds a `V2` perm; the only
  producer of one), and `const FILE_MAGIC_V2: [u8; 8]` = `b"SPQCPRM2"` — a RESERVED version marker
  (feature-gated with `mmap`) that the default path NEVER writes; it reserves the marker so a future
  migration can write/auto-detect V2 without colliding with the shipped `SPQCPRM1` magic. A V2 block
  with no middle-column reset is byte-identical to its V1 block. Whether the frame-of-reference
  encoding shrinks the stream enough to adopt is an open question to be settled on the canonical perf
  host (no number is asserted here); until then it stays an opt-in spike behind the feature flag.
- **JSON-LD W3C conformance is RATCHETED (honest baseline, not 100%).** A ratcheted W3C
  JSON-LD 1.1 conformance gate (sq-oy1f.2 + sq-3uos5 + sq-oy1f.19 + sq-oy1f) drives the official
  `w3c/json-ld-api` suite AND the SEPARATE `w3c/json-ld-framing` suite through the real paths:
  **toRdf** through the `jsonld` parser (oxjsonld), **fromRdf** through the `serialize-rdf`
  writer, **compact** through the native Compaction Algorithm (`graph_to_jsonld_compact`),
  **expand** + **flatten** through the shipping writer (`graph_to_jsonld(Expanded|Flattened)`),
  and **frame** through the native Framing Algorithm (`graph_to_jsonld_framed`). toRdf/fromRdf/compact
  are compared by a re-parse RDF-dataset round-trip against the INPUT (`reparse(out) ≡ in`, the
  oxjsonld self-reparse oracle); **expand/flatten/frame** are compared by re-parse RDF-equivalence
  against the suite's NORMATIVE expected output (`reparse(write(D)) ≡ reparse(expected)`) — these
  are JSON-LD normal forms / a SELECT+RESHAPE (they drop/merge/prune/fill), so the oracle anchors on
  the expected document, not the input. The floors only RISE; they reflect ACTUAL current pass
  counts, not full conformance — the
  remaining toRdf divergences are documented oxjsonld limits (remote/`@import` `@context` needs a
  `LoadDocumentCallback`; `expandContext`/`rdfDirection` options not applied; a few
  base-normalization edge cases + leniently-accepted negative tests), and fromRdf misses
  lists whose cells are shared across graphs. The **compact** lane gates on **lossless
  compaction** (parse input → RDF, compact against the case `@context`, require the
  re-parse to reconstruct the SAME RDF); the floor was raised by sq-oy1f.16 after the #978
  faithfulness fixes (the `@type:@id`-coerced-key-vs-plain-string IRI confusion and several
  container round-trips became lossless), with the rest still below the floor (scoped/typed
  contexts, `@nest`, `@index`/`@id` map shapes the writer does not emit) and several SKIPPED
  (negatives sparq's TOTAL compaction does not raise; JSON-LD-1.0-only; non-inline/remote
  `@context`; empty-RDF inputs). The **frame** lane (over the separate framing suite) gates the
  89 positive `jld:FrameTest` cases; below-floor cases are genuine framer divergences (value-pattern
  matching over `@value` alternative arrays, `@explicit`/`@default` fill differences, named-graph
  `@graph` framing shapes, `@list`/`@set` re-emit) tracked for a future RISE, and the 3
  NegativeEvaluationTests are SKIPPED (sparq's framer is TOTAL — it never raises the spec's
  frame-validation errors, so it cannot honestly "pass" by rejecting). The compact/frame oracle
  is oxjsonld self-reparse, so the `@reverse` double-inversion / non-string-language interop gap
  documented above (the compaction interop caveat) is NOT caught by it and is tracked separately.
  The **expand** + **flatten** lanes (sq-oy1f) GRADUATED out of the NOT-IMPLEMENTED bucket: each
  `jld:ExpandTest`/`jld:FlattenTest` input is parsed to RDF and projected through the shipping
  writer, then compared by RDF-equivalence to the normative expected document; below-floor cases
  are honest writer divergences (`@direction`/i18n-datatype, list/coercion shapes) and the SKIP
  buckets are the documented ones (negatives sparq's TOTAL writer does not raise, JSON-LD-1.0-only,
  empty-RDF inputs). Only **html** (script extraction) + **remote-doc** (a `LoadDocumentCallback`)
  remain NOT-IMPLEMENTED buckets the runner reports separately and never fails on (they grow the
  ratchet as those land). The lane is the opt-in
  `jsonld-suite` feature on `sparq-conformance` (forwards to `sparq-core/jsonld` +
  `sparq-engine/serialize-rdf`); OFF it compiles to a self-skip. Reproduce with
  `scripts/fetch-jsonld-tests.sh` + `scripts/fetch-jsonld-framing-tests.sh` then
  `cargo test -p sparq-conformance --features jsonld-suite --test jsonld_suite` (self-skips if
  the gitignored suites are absent). It is registered in the central conformance scoreboard
  (`sparq-conformance-scoreboard`).

## See also

- `fused-decompress-parse` — choosing gzip vs zstd vs bzip2 and fusing decode with parse (measured numbers).
- `rust-parallel-parsing` — how the chunk-parallel N-Triples/Turtle scanners work, and when NOT to parallelize.
- `hdt-format` — the HDT binary layout internals and the `hdt`-crate wrapping/decode performance.
