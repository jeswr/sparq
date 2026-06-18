# @jeswr/sparq


[RDF/JS](https://rdf.js.org/)-style bindings for **sparq**, a Rust RDF
triplestore + SPARQL engine compiled to WebAssembly. One compact wasm artifact,
one tiny runtime npm dependency ([`fzstd`](https://www.npmjs.com/package/fzstd),
dynamically imported only when ingesting zstd); works in Node ≥ 18 and the
browser. (The wasm bundle bytes are tracked per-commit on the perf dashboard,
<https://jeswr.github.io/sparq/dev/bench>.)

- Dictionary-encoded in-memory store (optionally block-compressed: substantially
  less index memory for a bounded scan cost).
- SPARQL 1.1 SELECT (BGPs with worst-case-optimal joins, FILTER, OPTIONAL,
  UNION, MINUS, BIND, VALUES, aggregates, ORDER BY, DISTINCT/LIMIT/OFFSET,
  sub-SELECT) and ASK — both evaluated natively (ASK early-exits at the first
  solution).
- SPARQL 1.1 CONSTRUCT / DESCRIBE: `queryQuads()` returns the constructed
  graph as RDF/JS `Quad`s, `queryQuadsString()` as raw N-Triples, and
  `queryQuadsStream()` streams a large graph one quad at a time.
- Named graphs (`options.dataset`): `GRAPH <iri>` / `GRAPH ?g` patterns,
  `FROM` / `FROM NAMED`, graph-aware `match()`.
- SPARQL 1.1 Update over the full dataset (`INSERT/DELETE DATA` with `GRAPH`
  blocks, `DELETE/INSERT … WHERE` with graph templates, `CLEAR`/`DROP`/
  `CREATE`/`ADD`/`COPY`/`MOVE`), applied in place through the engine's delta
  overlay — O(batch), no index rebuild — plus quad-level `applyDelta` /
  `addQuads` / `removeQuads`.
- Streaming results: `queryBindingsStream()` yields one solution at a time
  (`for…of` or `for await…of`) from ~64 KiB wasm-boundary chunks — no giant
  JSON blob, no giant array; `queryJsonChunks()` exposes the raw chunks.
- Compressed ingest: `fromCompressed()` decodes `.nt.zst` (including
  multi-frame zstd from sparq's `CompressedSink`) via pure-JS `fzstd`, and
  `.gz` via the platform.
- Dictionary-fetch protocol client (`SparqDictionaryClient`): zstd
  vocabulary-dictionary negotiation with a sparq server — content-addressed
  dictionary caching, background warm-up, pluggable dict-capable decoder.
- Results as RDF/JS Query-spec `Bindings` (Map-like, `.get(variable)`), terms as
  spec-compliant RDF/JS `Term`s (typed against `@rdfjs/types`).

## Install / build

The package ships the wasm artifact; from a source checkout build it first:

```sh
npm run build   # wasm-pack build ../crates/sparq-wasm (--features shacl) + tsc
npm test        # node --test against the built dist/
```

### Pinning a git build (before the npm release)

Until `@jeswr/sparq` is published to npm under its settled name, depend on it by
**pinning a git build**. The package's `prepare` script compiles the wasm engine
+ TypeScript on install, so a git pin yields a working binding (the registry
tarball ships those prebuilt). Add to the consumer's `package.json`:

```jsonc
"dependencies": {
  // pin an immutable commit; `directory: "js"` is read from this package.json
  "@jeswr/sparq": "github:jeswr/sparq#<commit-sha>"
}
```

A git-pinned install needs the Rust → wasm toolchain on the build machine
(`rustup target add wasm32-unknown-unknown` + `cargo install wasm-pack`); without
it `prepare` fails loudly with the install command rather than silently shipping
an engine-less binding. After install, verify the engine actually landed:

```sh
node -e "import('@jeswr/sparq').then(m=>m.SparqStore.fromString('<a> <b> <c> .','ntriples')).then(s=>{s.free?.();console.log('ok')})"
```

Maintainers run the publish guardrail this repo gates on — `npm run
check:package` — which proves `prepare` is wired, the `files` allowlist is
intact, and the packed tarball actually ships `dist/` + `wasm/*_bg.wasm`.

The default build (and the published bundle) ships with `--features shacl` so
`SparqStore.validate` works out of the box. SHACL is not free in the wasm
binary — it pulls in the SHACL engine + `regex` + the SPARQL query path for
`sh:sparql`, which roughly **doubles** the `.wasm` (measured ~1.21 MiB → ~2.19
MiB, +~1.0 MiB / +85%, before gzip). If you do not need validation and bundle
size matters, build the lean variant — `npm run build:wasm:lean` — which omits
SHACL entirely (`SparqStore.validate` then throws a clear error if called).

## Usage

```js
import { SparqStore, DataFactory as DF } from '@jeswr/sparq';

const store = await SparqStore.fromString(`
  @prefix ex: <http://ex/> .
  ex:alice ex:name "Alice" ; ex:knows ex:bob .
  ex:bob   ex:name "Bob"@en .
`, 'turtle'); // or 'ntriples' | 'nquads' | 'trig'

// SELECT → RDF/JS bindings
for (const row of store.queryBindings(
  'PREFIX ex: <http://ex/> SELECT ?s ?name WHERE { ?s ex:name ?name }',
)) {
  console.log(row.get('s').value, '→', row.get('name').value);
}

// ASK → boolean
store.queryBoolean('PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }'); // true

// CONSTRUCT / DESCRIBE → RDF/JS quads (default graph)
for (const quad of store.queryQuads(
  'PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:label ?n } WHERE { ?s ex:name ?n }',
)) {
  console.log(quad.subject.value, quad.predicate.value, quad.object.value);
}
store.queryQuadsString('DESCRIBE <http://ex/bob>'); // raw N-Triples string

// RDF/JS-style triple lookup (generated SELECT under the hood)
store.match(null, DF.namedNode('http://ex/name'), null); // → Quad[]
store.countQuads(null, DF.namedNode('http://ex/name'));  // → 2 (no materialisation)

// Build a store from RDF/JS quads (serialised to N-Quads internally)
const fromQuads = await SparqStore.fromQuads([
  DF.quad(DF.namedNode('http://ex/s'), DF.namedNode('http://ex/p'), DF.literal('o')),
]);

// SPARQL Update (the engine rebuilds its immutable index; the handle swaps in place)
store.update('PREFIX ex: <http://ex/> INSERT DATA { ex:carol ex:name "Carol" }');

// Raw SPARQL 1.1 JSON results, skipping JS-side term materialisation
const json = JSON.parse(store.queryJson('SELECT * WHERE { ?s ?p ?o } LIMIT 10'));

// Stream large results one solution at a time (no whole-result JSON/array):
for await (const row of store.queryBindingsStream('SELECT ?s ?o WHERE { ?s ?p ?o }')) {
  console.log(row.get('s').value);
}

// Incremental, O(batch) quad-level updates (no index rebuild):
store.addQuads([DF.quad(DF.namedNode('http://ex/d'), DF.namedNode('http://ex/p'), DF.literal('o'))]);
store.removeQuads(store.match(null, DF.namedNode('http://ex/p')));
store.applyDelta(insertQuads, deleteQuads); // deletes applied first, then inserts

// Named graphs: load as a DATASET (N-Quads / TriG / fromQuads keep their graphs)
const ds = await SparqStore.fromString(nquads, 'nquads', { dataset: true });
ds.queryBindings('SELECT ?g ?s WHERE { GRAPH ?g { ?s ?p ?o } }');
ds.update('INSERT DATA { GRAPH <http://ex/g> { <http://ex/s> <http://ex/p> "o" } }');
ds.match(null, null, null, DF.namedNode('http://ex/g')); // graph-aware lookup

// Compressed ingest: .nt.zst (multi-frame OK) / .ttl.gz, codec sniffed by magic
const fromZst = await SparqStore.fromCompressed(zstBytes, 'ntriples');

// Memory-constrained devices: block-compressed index
const compact = await SparqStore.fromString(bigTurtle, 'turtle', { compressed: true });
compact.heapBytes(); // rough wasm-side footprint

// SHACL validation (data graph vs shapes graph) → a typed ValidationReport.
// Stateless one-shot: does NOT consult the store's own triples (a drop-in for
// rdf-validate-shacl). conforms counts every result; filter by severity for a gate.
const report = store.validate(dataTurtle, shapesTurtle, 'turtle'); // format defaults to 'turtle'
report.conforms;                                                    // boolean
for (const r of report.results) {
  console.log(r.focusNode, r.path, r.severity, r.message);          // per-violation fields
}

store.free(); // release wasm memory (also `using store = …` via Symbol.dispose)
```

### Talking to a sparq server: dictionary-fetch protocol

Sparq servers compress small SPARQL responses with shared zstd *vocabulary
dictionaries* (markedly smaller on small bodies) — but only when the client proves
it already holds the dictionary, so no request ever waits on one.
`SparqDictionaryClient` wraps `fetch` with the whole negotiation:

```js
import { SparqDictionaryClient } from '@jeswr/sparq';

const client = new SparqDictionaryClient({
  // fzstd cannot decode dictionary frames — supply a dict-capable decoder
  // (zstd-wasm in the browser; node:zlib in Node). Without it the client
  // simply never advertises dictionaries and responses stay plain zstd.
  decodeWithDictionary: (body, dict) => zstdDecompressSync(body, { dictionary: Buffer.from(dict) }),
});
const { body, dictionary } = await client.fetch('https://host/sparql?query=…');
```

Held dictionaries are advertised via `Sparq-Dictionary`; the response echoes
the one used and `Sparq-Dictionary-Current` triggers a background, content-
verified warm-up from `GET /dictionary/{dict-id}` for the *next* request.

### Notes & limits

- Named graphs are folded into the default graph **unless** the store is
  loaded with `options.dataset`; `size`/`heapBytes` always report the default
  graph (use `countQuads()` for dataset totals). `dataset` is not combinable
  with `compressed` yet.
- Federated (`SERVICE`) queries are not exposed at the JS wrapper layer (tracked in beads — `bd list -l area:js`).
- `validate()` (SHACL) runs in-process and is best for small documents
  (~10–100 triples); for large data graphs validate server-side via the
  `sparq-server` HTTP `validate` path. It needs a `--features shacl` bundle
  (shipped by default; `build:wasm:lean` omits it — see *Install / build*).
- `REGEX`/`REPLACE` are compiled out of the wasm build (the engine's
  non-default `regex` cargo feature) to keep the bundle small — use
  `CONTAINS`/`STRSTARTS`/… or a custom build.
- A specific blank node in `match()` is matched by label via a post-filter
  (SPARQL itself cannot reference a particular bnode); `applyDelta` deletes
  also address bnodes by label (unlike SPARQL `DELETE DATA`).
- `update()` and `applyDelta()` mutate in place through the engine's delta
  overlay (append-only dictionary growth; deletes are overlay tombstones until
  the wasm store is reloaded).
- Browsers silently truncate **multi-member gzip** to the first member —
  `fromCompressed` uses `node:zlib` in Node (which loops members) and
  `DecompressionStream` in the browser (single-member only); multi-frame
  **zstd** decodes fully everywhere via the bundled fzstd.
- **SPARQL-injection guard.** `match()`/`countQuads()` build a query string and
  `addQuads`/`applyDelta`/`fromQuads` build N-Quads, both embedding RDF/JS terms
  via `termToNT`. Hostile term values cannot break out of their token: IRIs are
  percent-encoded over the full `IRIREF`-illegal set (`< > " { } | ^` `` ` ``
  `\` and `#x00–#x20`, so a `>` in an ACL-pointer IRI becomes `%3E`) and literal
  values escape `"`, `\` and all control chars — the same rules QLever's lexer
  enforces. This is proved end-to-end against the engine's real parser in
  `test/injection.test.mjs`. Note percent-encoding is canonicalising: an IRI
  value that *contains* illegal chars stores under its encoded form.

## Benchmarks

`npm run bench` (see `bench/vs-oxigraph.mjs`) compares load + SELECT workloads
against [oxigraph](https://www.npmjs.com/package/oxigraph)'s npm package when it
is installed (`npm i --no-save oxigraph`); it skips gracefully otherwise.
