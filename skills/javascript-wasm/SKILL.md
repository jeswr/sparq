---
name: javascript-wasm
description: Use the sparq RDF+SPARQL engine from JavaScript/TypeScript (Node >=18 or the browser) via its WebAssembly build and the @jeswr/sparq RDF-JS wrapper — load Turtle/N-Triples/N-Quads/TriG, run SPARQL 1.1 SELECT/ASK, stream large results, count without materialising, apply SPARQL Update / quad deltas, do RDF-JS match()/countQuads(), and ingest gzip/zstd-compressed RDF. Reach for this when wiring sparq into a Node service, browser tab, or RDF-JS pipeline.
---

# sparq from JavaScript / WebAssembly

sparq is a Rust RDF triplestore + SPARQL engine compiled to a single ~886 KB (314 KB gzip) WebAssembly artifact. The npm package **`@jeswr/sparq`** wraps it in an idiomatic [RDF/JS](https://rdf.js.org/) surface (`SparqStore`, Map-like `Bindings`, spec terms via `@rdfjs/types`); it runs unchanged in Node >= 18 and the browser. There is also a thin raw wasm class (`Store`) if you want SPARQL-JSON strings with no JS-side term materialisation.

Use `SparqStore` (the high-level wrapper) by default. Drop to the raw `Store` only for CONSTRUCT/DESCRIBE (the wrapper does not expose those yet) or to skip term materialisation entirely.

## Quickstart

```js
import { SparqStore, DataFactory as DF } from '@jeswr/sparq';

// init() runs lazily on first construction; nothing else to await.
const store = await SparqStore.fromString(`
  @prefix ex: <http://ex/> .
  ex:alice ex:name "Alice" ; ex:knows ex:bob .
  ex:bob   ex:name "Bob"@en .
`, 'turtle'); // 'turtle' (default) | 'ntriples' | 'nquads' | 'trig'

// SELECT -> RDF/JS Bindings[]
for (const row of store.queryBindings(
  'PREFIX ex: <http://ex/> SELECT ?s ?name WHERE { ?s ex:name ?name }',
)) {
  console.log(row.get('s').value, '->', row.get('name').value);
}

// ASK -> boolean (native early-exit at first solution)
store.queryBoolean('PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }'); // true

// count without materialising rows
store.count('PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?o }'); // 2

store.free(); // release wasm memory (or: `using store = await SparqStore.fromString(...)`)
```

Building from a source checkout of the sparq repo (the package ships the wasm prebuilt, but a checkout must build it):

```sh
cd js
npm run build   # = wasm-pack build --target web --release  +  tsc
npm test        # node --test against the built dist/
```

## Key APIs

`SparqStore` (from `@jeswr/sparq`) — the high-level store:

```ts
// Construction (all async; each runs wasm init() once, memoised)
static fromString(data: string, format?: RdfFormat, opts?: SparqStoreOptions): Promise<SparqStore>
static fromQuads(quads: Iterable<RDF.Quad>, opts?: SparqStoreOptions): Promise<SparqStore>     // serialised to N-Quads internally
static fromCompressed(bytes: Uint8Array, format?: RdfFormat,
                      opts?: SparqStoreOptions & { codec?: 'zstd' | 'gzip' }): Promise<SparqStore>

type RdfFormat = 'turtle' | 'ntriples' | 'nquads' | 'trig';
interface SparqStoreOptions { compressed?: boolean; dataset?: boolean; } // NOT combinable

// Reading
get size(): number                                  // deduped triples in the DEFAULT graph
heapBytes(): number                                 // rough wasm-side footprint
query(sparql): Bindings[] | boolean                 // dispatches: SELECT->Bindings[], ASK->boolean
queryBindings(sparql): Bindings[]                    // SELECT -> one Bindings per solution
queryBoolean(sparql): boolean                        // ASK (native path; rejects non-ASK)
queryJson(sparql): string                            // raw SPARQL 1.1 JSON string (SELECT or ASK)
count(sparql): number                                // solution count, no materialisation
queryBindingsStream(sparql): Generator<Bindings>     // stream solutions, for...of / for await...of
queryJsonChunks(sparql): Generator<string>           // raw ~64 KiB JSON chunks (concat == queryJson)

// RDF/JS quad lookup (null/undefined/Variable = wildcard; generated SELECT under the hood)
match(s?, p?, o?, g?): Quad[]
countQuads(s?, p?, o?, g?): number

// Mutation (all IN PLACE through the engine's O(batch) delta overlay)
update(sparql): void                                 // SPARQL 1.1 Update (INSERT/DELETE DATA, DELETE/INSERT WHERE, CLEAR/DROP/CREATE/ADD/COPY/MOVE)
applyDelta(inserts: Iterable<RDF.Quad>, deletes?: Iterable<RDF.Quad>): void  // deletes first, then inserts
addQuads(quads): void                                // applyDelta(quads, [])
removeQuads(quads): void                             // applyDelta([], quads) — bnodes matched BY LABEL
free(): void                                         // also Symbol.dispose
```

`Bindings` (RDF/JS Query-spec, Map-like): `.get('var') -> RDF.Term | undefined`, `.has`, `.keys()`, `.values()`, `.entries()`, `.size`, `.equals`, immutable `.set/.delete/.filter/.map/.merge`, iterable. Terms: `.termType` (`'NamedNode' | 'Literal' | 'BlankNode' | ...`), `.value`, plus `.language` / `.datatype` on literals.

Other named exports from `@jeswr/sparq`: `DataFactory` (RDF/JS factory: `namedNode`, `blankNode`, `literal`, `variable`, `quad`, ...) and term classes `NamedNode/BlankNode/Literal/Variable/DefaultGraph/Quad`; `init` (idempotent wasm bootstrap); compression helpers `decompress / decompressToString / sniffCodec`; SPARQL helpers `termFromSparqlJson / termToNT / quadsToNQuads / detectQueryForm / askToSelect / SparqlJsonRowsParser`; and the `SparqDictionaryClient` (server dictionary-fetch protocol).

Raw wasm `Store` (from `../wasm/sparq_wasm.js`, re-exported as `WasmStore` internally) — use only when you need CONSTRUCT/DESCRIBE, batch cursors, or **query-plan introspection**. Methods return SPARQL-JSON / N-Triples / plan-text **strings**, not RDF/JS terms: `Store.load/loadDataset/loadCompressed(text, format)`, `.query(sparql)`, `.queryChunks(sparql)`, `.queryCursor(sparql, batchSize)`, `.queryQuads(sparql)` (CONSTRUCT/DESCRIBE -> N-Triples), `.queryQuadsChunks(sparql, batchSize)`, `.count`, `.ask`, `.askWithMaxRows(sparql, maxRows)`, `.explain(sparql)`, `.explainAnalyze(sparql)`, `.update`, `.updateInPlace`, `.applyDelta(inserts, deletes)`, `.size`, `.heapBytes()`.

## Common recipes

**Stream a large SELECT without holding the whole result.** One solution at a time, from ~64 KiB wasm-boundary chunks; `break` frees the cursor.

```js
for await (const row of store.queryBindingsStream('SELECT ?s ?o WHERE { ?s ?p ?o }')) {
  console.log(row.get('s').value);
}
```

**RDF/JS match() / countQuads().** Wildcards are `null`/`undefined`/Variable.

```js
store.match(null, DF.namedNode('http://ex/name'), null);     // -> Quad[]
store.countQuads(null, DF.namedNode('http://ex/name'));      // -> 2 (no materialisation)
```

**Mutate in place (O(batch), no index rebuild).**

```js
store.update('PREFIX ex: <http://ex/> INSERT DATA { ex:carol ex:name "Carol" }');
store.addQuads([DF.quad(DF.namedNode('http://ex/d'), DF.namedNode('http://ex/p'), DF.literal('o'))]);
store.removeQuads(store.match(null, DF.namedNode('http://ex/p')));
store.applyDelta(insertQuads, deleteQuads); // deletes applied first, then inserts
```

**Named graphs (load as a dataset).** Without `dataset: true`, all quads fold into the default graph and `GRAPH`/named-graph lookups see nothing.

```js
const ds = await SparqStore.fromString(nquads, 'nquads', { dataset: true });
ds.queryBindings('SELECT ?g ?s WHERE { GRAPH ?g { ?s ?p ?o } }');
ds.update('INSERT DATA { GRAPH <http://ex/g> { <http://ex/s> <http://ex/p> "o" } }');
ds.match(null, null, null, DF.namedNode('http://ex/g')); // graph-aware
```

**Compressed ingest + memory-tight index.** `fromCompressed` sniffs the codec by magic number (`.nt.zst`, `.ttl.gz`); `{ compressed: true }` halves index memory at a small per-scan decode cost.

```js
const fromZst = await SparqStore.fromCompressed(zstBytes, 'ntriples');           // codec auto-sniffed
const compact = await SparqStore.fromString(bigTurtle, 'turtle', { compressed: true });
```

**CONSTRUCT / DESCRIBE (raw wasm only).** Not on `SparqStore`; drop to the raw `Store`, which returns N-Triples (a valid Turtle subset).

```js
import init, { Store } from '@jeswr/sparq/wasm/sparq_wasm.js'; // path is into the shipped wasm/ dir
await init();
const raw = Store.load(turtle, 'turtle');
const nt  = raw.queryQuads('PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:label ?n } WHERE { ?s ex:name ?n }');
```

**EXPLAIN / EXPLAIN ANALYZE (query-plan introspection, raw wasm only).** Same plan text the Rust API (`sparq_engine::explain`) and the HTTP endpoint (`?explain` / `?explain=analyze`, or `Accept: text/x-sparq-explain`) return — returned to JS as a plain string. `explain` is a planning-only dry run (no execution; every query form); `explainAnalyze` runs the query (SELECT/ASK only) and appends a per-operator row-count trace.

```js
const plan = raw.explain('PREFIX ex: <http://ex/> SELECT ?n ?a WHERE { ?s ex:name ?n . ?s ex:age ?a }');
// "EXPLAIN (SELECT) — planning-only dry run; nothing is executed.\n...Plan:\n  ..."
const trace = raw.explainAnalyze('PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }');
// plan + per-operator output rows (wall times read 0 on wasm32 — no monotonic clock)
```

## Gotchas / feature flags / prerequisites

- **ESM only, Node >= 18.** The package is `"type": "module"`; `init()` is idempotent and runs automatically on the first `SparqStore.from*` call (in Node it reads the wasm bytes from disk; in the browser/Deno it `fetch`es relative to the module). One runtime dep: `fzstd` (~8 KB, dynamically imported only when decoding zstd).
- **`SparqStore` exposes SELECT/ASK only.** Its `query()` returns `Bindings[]` (SELECT) or `boolean` (ASK). CONSTRUCT/DESCRIBE and federated (`SERVICE`) queries are **not** exposed at the JS wrapper layer — use the raw `Store.queryQuads` for CONSTRUCT/DESCRIBE.
- **`REGEX` / `REPLACE` are compiled out** of the wasm build (the engine's non-default `regex` cargo feature is off to keep the bundle small). Use `CONTAINS`/`STRSTARTS`/`STRENDS`/... or a custom wasm build with `--features regex`.
- **`options.dataset` is not combinable with `options.compressed`** — there is no compressed dataset loader yet (the constructor throws). `compact-index` (3 permutations, ~half the memory) is auto-selected for wasm32 regardless; `compressed` adds block compression on top.
- **`size` / `heapBytes` report the DEFAULT graph only.** For dataset totals use `countQuads()` (its graph wildcard spans named graphs).
- **Mutation is overlay-based.** `update()` / `applyDelta()` write through an append-only delta overlay: the dictionary only grows, and deletes are tombstones until the wasm store is reloaded. Blank nodes in `applyDelta`/`removeQuads` are matched **by label** (so bnode triples can be retracted — impossible via SPARQL `DELETE DATA`).
- **Browser gzip truncation.** Browsers silently truncate **multi-member gzip** to the first member; `fromCompressed` uses `node:zlib` in Node (loops members) and `DecompressionStream` in the browser (single-member only). Multi-frame **zstd** decodes fully everywhere via fzstd. fzstd cannot decode zstd **dictionary** frames — for those supply a dict-capable decoder via `SparqDictionaryClient`'s `decodeWithDictionary` hook.
- **Lifetime.** Call `.free()` (or use `using`) to release wasm linear memory; the store and any held cursors must not be used afterward. wasm32 caps linear memory at 4 GB (a real tab is happier under ~2 GB): ~30 M triples raw, ~75 M with `compressed`.
- **Raw `Store` budget knobs are wasm-portable only.** `askWithMaxRows` bounds the working set by row count; the engine's wall-clock deadline budget is native-only (`std::time::Instant` is unusable on wasm32).
- **`explainAnalyze` wall times read 0 on wasm32.** There is no monotonic clock in the wasm bundle, so the per-operator trace reports 0 for every wall time; the per-operator **row counts are exact**. `explainAnalyze` executes the query (SELECT/ASK only) — CONSTRUCT/DESCRIBE/UPDATE are rejected; use `explain` (a non-executing dry run that accepts every query form) for those.

## See also

- `fused-decompress-parse`, `rust-parallel-parsing` — server/native ingest internals behind the codecs you feed `fromCompressed`.
- `hdt-format` — loading `.hdt` archives (native crate, not in the wasm bundle).
- `noir-circuit-patterns`, `noir-optimisation`, `mpc-protocols`, `verifiable-credentials-zk`, `sparql-formal-semantics` — the ZK/MPC estate (separate `zk` feature; not part of the JS/wasm surface).
