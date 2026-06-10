# @jeswr/sparq

> **Package name is a placeholder** — the final npm name is the maintainer's call
> (e.g. `sparq`, `@rdfjs/sparq`, `sparq-wasm`). Update `package.json#name` before
> the first publish.

[RDF/JS](https://rdf.js.org/)-style bindings for **sparq**, a Rust RDF
triplestore + SPARQL engine compiled to WebAssembly. One ~1 MB wasm artifact,
zero runtime npm dependencies; works in Node ≥ 18 and the browser.

- Dictionary-encoded, immutable in-memory store (optionally block-compressed:
  about half the index memory for a bounded scan cost).
- SPARQL 1.1 SELECT (BGPs with worst-case-optimal joins, FILTER, OPTIONAL,
  UNION, MINUS, BIND, VALUES, aggregates, ORDER BY, DISTINCT/LIMIT/OFFSET,
  sub-SELECT) and ASK (evaluated via the engine's no-materialise count path).
- SPARQL 1.1 Update: `INSERT DATA`, `DELETE DATA`, `DELETE/INSERT … WHERE`,
  `CLEAR` (default graph).
- Results as RDF/JS Query-spec `Bindings` (Map-like, `.get(variable)`), terms as
  spec-compliant RDF/JS `Term`s (typed against `@rdfjs/types`).

## Install / build

The package ships the wasm artifact; from a source checkout build it first:

```sh
npm run build   # wasm-pack build ../crates/sparq-wasm + tsc
npm test        # node --test against the built dist/
```

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

// Memory-constrained devices: block-compressed index
const compact = await SparqStore.fromString(bigTurtle, 'turtle', { compressed: true });
compact.heapBytes(); // rough wasm-side footprint

store.free(); // release wasm memory (also `using store = …` via Symbol.dispose)
```

### Notes & limits

- The store is **triple-scoped**: named graphs in the input (TriG/N-Quads or
  `fromQuads`) are folded into the default graph; `match(s, p, o, namedGraph)`
  returns nothing for a named graph.
- ASK is rewritten to `SELECT *` and answered from the engine's count path
  (the engine itself currently evaluates SELECT only).
- CONSTRUCT / DESCRIBE / federated queries are not supported (see `TODO.md`).
- A specific blank node in `match()` is matched by label via a post-filter
  (SPARQL itself cannot reference a particular bnode).
- `update()` consumes and replaces the wasm store handle; concurrent use of an
  old handle is impossible (the old one is freed).

## Benchmarks

`npm run bench` (see `bench/vs-oxigraph.mjs`) compares load + SELECT workloads
against [oxigraph](https://www.npmjs.com/package/oxigraph)'s npm package when it
is installed (`npm i --no-save oxigraph`); it skips gracefully otherwise.
