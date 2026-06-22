/**
 * `SparqStore`: an RDF/JS-flavoured wrapper around the sparq wasm engine — an
 * immutable-index, dictionary-encoded in-memory store with SPARQL SELECT/ASK
 * (both evaluated natively by the engine) and SPARQL Update / quad-level
 * deltas (applied in place through the engine's delta overlay, O(batch)).
 */
import type * as RDF from '@rdfjs/types';
import { Bindings } from './bindings.js';
import { decompressToString, type CompressionCodec } from './decompress.js';
import {
  detectQueryForm,
  parseNTriples,
  quadsToNQuads,
  SparqlJsonRowsParser,
  termFromSparqlJson,
  termToNT,
  type SparqlJsonResults,
  type SparqlJsonTerm,
} from './sparql.js';
import { DataFactory, Quad, Variable } from './terms.js';
import { init, WasmStore } from './wasm.js';

// [OPUS-4.8] sq-dvyi: `'jsonld'` added — the engine parses JSON-LD (oxjsonld) when the
// wasm bundle is built with the OPT-IN `jsonld` feature, which the published `@jeswr/sparq`
// bundle and the site REPL both enable (js `build:wasm`). The JS surface accepts it like
// the other syntaxes; the default lean wasm bundle leaves oxjsonld out (perf-gate floor).
export type RdfFormat = 'turtle' | 'ntriples' | 'nquads' | 'trig' | 'jsonld';

/**
 * [OPUS-4.8] sq-pxls (#162): one SHACL validation result — the per-violation
 * shape PSS's ADR-0014 `ShaclValidator` seam consumes (a drop-in for
 * `rdf-validate-shacl`'s result records). `focusNode`/`value`/`sourceShape`
 * are N-Triples term strings (the lexical form sparq's `Term` `Display`
 * produces); `path` is a SHACL Turtle path expression;
 * `sourceConstraintComponent`/`severity` are full IRIs (e.g.
 * `http://www.w3.org/ns/shacl#Violation`); `message` is the first declared
 * `sh:message` text or a generated default. `path`/`value`/`message` are
 * `null` when the underlying result carries none.
 */
export interface ValidationResult {
  focusNode: string;
  path: string | null;
  value: string | null;
  sourceShape: string;
  sourceConstraintComponent: string;
  severity: string;
  message: string | null;
}

/**
 * [OPUS-4.8] sq-pxls (#162): a SHACL validation report — the typed parse of
 * the JSON the wasm `Store.validate` binding returns. `conforms` counts EVERY
 * result regardless of severity (the W3C-suite notion); filter `results` by
 * `severity` for a violations-only gate.
 */
export interface ValidationReport {
  conforms: boolean;
  results: ValidationResult[];
}

export interface SparqStoreOptions {
  /**
   * Store the index block-compressed (~half the index memory at a bounded
   * per-scan decode cost) — the right default when RAM, not CPU, binds.
   */
  compressed?: boolean;
  /**
   * Preserve named graphs from N-Quads / TriG (or `fromQuads`) as separate
   * graphs instead of folding them into the default graph — enables
   * `GRAPH <iri> { … }` / `GRAPH ?g { … }`, `FROM (NAMED)` and updates with
   * `GRAPH` blocks, and makes `match()` graph-aware. Not combinable with
   * `compressed` (the engine has no compressed dataset loader yet).
   */
  dataset?: boolean;
}

type MatchTerm = RDF.Term | null | undefined;

/** One SPARQL-JSON solution row as an RDF/JS `Bindings`. */
function bindingsFromRow(row: Record<string, SparqlJsonTerm>): Bindings {
  const entries: [RDF.Variable, RDF.Term][] = [];
  for (const [name, term] of Object.entries(row)) {
    entries.push([new Variable(name), termFromSparqlJson(term)]);
  }
  return new Bindings(entries);
}

/** A position of a triple pattern: either an inline SPARQL constant or a variable. */
function position(term: MatchTerm, variable: string): { sparql: string; fixed: RDF.Term | undefined } {
  // RDF/JS match() semantics: null/undefined and Variable are wildcards.
  if (!term || term.termType === 'Variable') return { sparql: `?${variable}`, fixed: undefined };
  // A specific blank node cannot be written in SPARQL (a bnode in a query is a
  // fresh variable), so scan with a variable and post-filter on the label.
  if (term.termType === 'BlankNode') return { sparql: `?${variable}`, fixed: undefined };
  return { sparql: termToNT(term), fixed: term };
}

export class SparqStore {
  #inner: WasmStore;

  private constructor(inner: WasmStore) {
    this.#inner = inner;
  }

  /**
   * Parses an RDF document into a store.
   * `format`: `"turtle"` (default) | `"ntriples"` | `"nquads"` | `"trig"` | `"jsonld"`.
   * Named graphs (N-Quads / TriG / a JSON-LD `@graph`) are folded into the default
   * graph unless `options.dataset` is set, in which case they are preserved as
   * separate graphs.
   */
  static async fromString(data: string, format: RdfFormat = 'turtle', options: SparqStoreOptions = {}): Promise<SparqStore> {
    await init();
    if (options.dataset && options.compressed) {
      throw new Error('options.dataset cannot be combined with options.compressed (no compressed dataset loader yet)');
    }
    const inner = options.dataset
      ? WasmStore.loadDataset(data, format)
      : options.compressed
        ? WasmStore.loadCompressed(data, format)
        : WasmStore.load(data, format);
    return new SparqStore(inner);
  }

  /**
   * [OPUS-4.8] sq-lii76: parses an RDF document into a store SYNCHRONOUSLY — the same as
   * {@link fromString} but WITHOUT the `await init()`. The wasm engine must ALREADY be
   * initialised (a prior `await init()` / `await SparqStore.fromString(...)` has resolved),
   * otherwise the wasm binding throws. This is the building block for the synchronous RDF/JS
   * `DatasetCore` members ({@link Dataset.match}), where the engine is guaranteed already up;
   * prefer the async {@link fromString} for first construction.
   */
  static fromStringSync(data: string, format: RdfFormat = 'turtle', options: SparqStoreOptions = {}): SparqStore {
    if (options.dataset && options.compressed) {
      throw new Error('options.dataset cannot be combined with options.compressed (no compressed dataset loader yet)');
    }
    const inner = options.dataset
      ? WasmStore.loadDataset(data, format)
      : options.compressed
        ? WasmStore.loadCompressed(data, format)
        : WasmStore.load(data, format);
    return new SparqStore(inner);
  }

  /**
   * [OPUS-4.8] sq-lii76: builds a store from RDF/JS quads SYNCHRONOUSLY (see
   * {@link fromStringSync} — the wasm engine must already be initialised).
   */
  static fromQuadsSync(quads: Iterable<RDF.Quad>, options: SparqStoreOptions = {}): SparqStore {
    return SparqStore.fromStringSync(quadsToNQuads(quads), 'nquads', options);
  }

  /** Builds a store from RDF/JS quads (serialised internally to N-Quads). */
  static async fromQuads(quads: Iterable<RDF.Quad>, options: SparqStoreOptions = {}): Promise<SparqStore> {
    return SparqStore.fromString(quadsToNQuads(quads), 'nquads', options);
  }

  /**
   * Parses a COMPRESSED RDF document — `.nt.zst` / `.ttl.gz` / the
   * multi-frame zstd streams sparq's `CompressedSink` emits — decompressing
   * on the JS side (zstd via pure-JS `fzstd`, dynamically imported; gzip via
   * the platform) and loading with [`fromString`]. The codec is sniffed from
   * the payload's magic number unless `options.codec` names it.
   */
  static async fromCompressed(
    bytes: Uint8Array,
    format: RdfFormat = 'turtle',
    options: SparqStoreOptions & { codec?: CompressionCodec } = {},
  ): Promise<SparqStore> {
    const { codec, ...rest } = options;
    return SparqStore.fromString(await decompressToString(bytes, codec), format, rest);
  }

  /**
   * The number of (deduplicated) triples in the DEFAULT graph. For a store
   * loaded with `options.dataset`, count the whole dataset with
   * `countQuads()` (whose graph wildcard spans named graphs too).
   */
  get size(): number {
    return this.#inner.size;
  }

  /** A rough estimate of the store's in-memory footprint, in bytes (default graph). */
  heapBytes(): number {
    return this.#inner.heapBytes();
  }

  /**
   * Runs a SPARQL query: returns `Bindings[]` for SELECT, `boolean` for ASK.
   * For the graph-valued forms (CONSTRUCT / DESCRIBE) use {@link queryQuads}
   * (or {@link queryQuadsString} for the raw N-Triples) — routing one through
   * `query()` throws, since this method only yields solution tables.
   */
  query(sparql: string): Bindings[] | boolean {
    const form = detectQueryForm(sparql)?.form;
    if (form === 'ASK') return this.queryBoolean(sparql);
    return this.queryBindings(sparql);
  }

  /** Runs a SELECT query, returning one RDF/JS `Bindings` per solution. */
  queryBindings(sparql: string): Bindings[] {
    const json = JSON.parse(this.#inner.query(sparql)) as SparqlJsonResults;
    const rows = json.results?.bindings ?? [];
    return rows.map(bindingsFromRow);
  }

  /**
   * [OPUS-4.8] #1123 — runs a SELECT query and returns the solutions in the OXIGRAPH JS shape:
   * an array of plain `Map<string, Term>`, each keyed on the variable NAME (no `?`) with RDF/JS
   * `Term` values — exactly what Oxigraph's `Store.query` yields for a SELECT, so Oxigraph code
   * (`for (const binding of store.querySolutions(q)) binding.get("s").value`) ports unchanged.
   *
   * It is a thin O(n) re-view of {@link queryBindings}'s RDF/JS `Bindings` (one `Map` allocation
   * per solution, no extra wasm round-trip), so the engine's SPARQL-JSON path — not a second
   * code path — stays the single source of truth. Prefer {@link queryBindings} for an RDF/JS
   * pipeline (richer immutable `Bindings`); use this for drop-in Oxigraph migration.
   */
  querySolutions(sparql: string): Map<string, RDF.Term>[] {
    return this.queryBindings(sparql).map((b) => b.toMap());
  }

  /**
   * [OPUS-4.8] #1123 — the streaming counterpart of {@link querySolutions}: yields one
   * Oxigraph-shaped `Map<string, Term>` solution at a time without materialising the whole
   * result (see {@link queryBindingsStream} for the streaming contract). For porting Oxigraph
   * code that iterates `store.query(...)` lazily.
   */
  *querySolutionsStream(sparql: string): Generator<Map<string, RDF.Term>, void, undefined> {
    for (const b of this.queryBindingsStream(sparql)) yield b.toMap();
  }

  /**
   * Streams a SELECT query's solutions as RDF/JS `Bindings`, one at a time,
   * without ever materialising the whole result on the JS side — neither as
   * one JSON string nor as one `Bindings[]`. The engine serialises in ~64 KiB
   * chunks that cross the wasm boundary piecewise; at most one chunk (plus
   * one partial row) is held at a time. Works with both `for…of` and
   * `for await…of`; abandoning the iterator early (`break`) frees the
   * wasm-side cursor.
   */
  *queryBindingsStream(sparql: string): Generator<Bindings, void, undefined> {
    const cursor = this.#inner.queryChunks(sparql);
    try {
      const parser = new SparqlJsonRowsParser();
      for (;;) {
        const chunk = cursor.next();
        if (chunk === undefined) break;
        for (const row of parser.push(chunk)) yield bindingsFromRow(row);
      }
      if (parser.boolean !== undefined) {
        throw new Error('queryBindingsStream() requires a SELECT query (got an ASK boolean result)');
      }
    } finally {
      cursor.free();
    }
  }

  /**
   * Streams the raw SPARQL 1.1 JSON results document as the engine's chunk
   * sequence (~64 KiB each, split only at solution-row boundaries); the
   * concatenation of the chunks is byte-identical to `queryJson()`. For
   * forwarding large results to a network sink or incremental parser with no
   * JS-side term materialisation.
   */
  *queryJsonChunks(sparql: string): Generator<string, void, undefined> {
    const cursor = this.#inner.queryChunks(sparql);
    try {
      for (;;) {
        const chunk = cursor.next();
        if (chunk === undefined) break;
        yield chunk;
      }
    } finally {
      cursor.free();
    }
  }

  /**
   * Runs an ASK query through the engine's NATIVE ask path: evaluation
   * early-exits at the first solution (the pattern runs under an implicit
   * `LIMIT 1`), a single-pattern ASK is answered straight from the index, and
   * the result crosses the wasm boundary as a plain `boolean` — no SELECT is
   * materialised, no SPARQL-JSON string is built or parsed. A non-ASK query is
   * rejected with a clear error.
   */
  queryBoolean(sparql: string): boolean {
    return this.#inner.ask(sparql);
  }

  /**
   * The raw engine output for a SELECT or ASK query: a SPARQL 1.1 JSON
   * results string (`application/sparql-results+json` — the boolean form for
   * ASK), with no JS-side term materialisation.
   */
  queryJson(sparql: string): string {
    return this.#inner.query(sparql);
  }

  /**
   * [OPUS-4.8] sq-1gkw: runs a graph-valued query (CONSTRUCT / DESCRIBE),
   * returning the constructed/described graph as RDF/JS {@link Quad}s in the
   * default graph. CONSTRUCT instantiates its template once per WHERE solution
   * (template blank nodes freshened per solution; triples with unbound or
   * RDF-illegal slots dropped per SPARQL §16.2); DESCRIBE returns each
   * resource's concise bounded description. A SELECT / ASK query is rejected
   * with a clear error — use {@link query} / {@link queryBindings} /
   * {@link queryBoolean} for those. For a large graph, stream it with
   * {@link queryQuadsStream}, or get the raw N-Triples with
   * {@link queryQuadsString}.
   */
  queryQuads(sparql: string): Quad[] {
    return parseNTriples(this.#inner.queryQuads(sparql));
  }

  /**
   * Like {@link queryQuads} but returns the constructed/described graph as the
   * engine's raw N-Triples string (a syntactic subset of Turtle, so also a
   * valid `text/turtle` document) — for forwarding to a serializer sink or
   * writing to a file without RDF/JS term materialisation.
   */
  queryQuadsString(sparql: string): string {
    return this.#inner.queryQuads(sparql);
  }

  /**
   * Streams a graph-valued query's result as RDF/JS {@link Quad}s, one at a
   * time, with the constructed graph crossing the wasm boundary in batches of
   * `batchSize` triples (default 1024) so a large graph is never held whole on
   * the JS side. Works with both `for…of` and `for await…of`; abandoning the
   * iterator early (`break`) frees the wasm-side cursor. (Caveat: the engine
   * materialises the full graph inside wasm before the first batch; the bound
   * is on the JS-side copy.)
   */
  *queryQuadsStream(sparql: string, batchSize = 1024): Generator<Quad, void, undefined> {
    const cursor = this.#inner.queryQuadsChunks(sparql, batchSize);
    try {
      for (;;) {
        const chunk = cursor.next();
        if (chunk === undefined) break;
        yield* parseNTriples(chunk);
      }
    } finally {
      cursor.free();
    }
  }

  /**
   * Counts the solutions of a SELECT query without materialising them
   * (read straight from the index where possible).
   */
  count(sparql: string): number {
    return this.#inner.count(sparql);
  }

  /**
   * [OPUS-4.8] sq-pxls (#162): validates an RDF **data graph** against a SHACL
   * **shapes graph**, returning a typed SHACL {@link ValidationReport}.
   *
   * Both arguments are RDF documents in the same `format` {@link fromString}
   * accepts (`"turtle"` (default) | `"ntriples"` | `"nquads"` | `"trig"`);
   * they are parsed identically (named graphs folded into the default graph).
   * This is a **stateless one-shot** — it does NOT consult the receiver's
   * stored triples — so it is the ergonomic drop-in for `rdf-validate-shacl`'s
   * `validate(dataDataset, { shapes })`, running through `sparq-shacl`'s SHACL
   * Core + SHACL-SPARQL (`sh:sparql`, §5.2) engine inside wasm. The JSON the
   * binding returns is parsed into the typed report here, so the caller never
   * touches the raw string.
   *
   * `report.conforms` counts EVERY result regardless of severity (the W3C-suite
   * notion); filter `report.results` by `severity` for a violations-only gate.
   * Throws only if a graph fails to parse; malformed shapes are skipped by the
   * engine, never surfaced. Validation is in-process and best for small
   * documents (~10–100 triples); validate large graphs server-side via the
   * `sparq-server` HTTP `validate` path instead (the other half of #162).
   *
   * Requires a `shacl`-enabled wasm bundle (the published `@jeswr/sparq` ships
   * one); a `shacl`-less custom build throws a clear error here rather than a
   * cryptic "not a function".
   */
  validate(data: string, shapes: string, format: RdfFormat = 'turtle'): ValidationReport {
    if (typeof this.#inner.validate !== 'function') {
      throw new Error(
        'SparqStore.validate requires a SHACL-enabled wasm bundle (build sparq-wasm with --features shacl)',
      );
    }
    return JSON.parse(this.#inner.validate(data, shapes, format)) as ValidationReport;
  }

  /**
   * The WHERE clause scoping `pattern` to the requested `graph` position:
   * `null`/`undefined`/Variable span the default graph AND every named graph
   * (binding `?g`); `DefaultGraph` scopes to the default graph; a `NamedNode`
   * to that graph; a `BlankNode` graph name scans `GRAPH ?g` for a label
   * post-filter (SPARQL cannot name a specific bnode).
   */
  static #graphScope(pattern: string, graph: MatchTerm): string {
    if (!graph || graph.termType === 'Variable') {
      return `{ { ${pattern} } UNION { GRAPH ?g { ${pattern} } } }`;
    }
    if (graph.termType === 'DefaultGraph') return `{ ${pattern} }`;
    return `{ GRAPH ${graph.termType === 'BlankNode' ? '?g' : termToNT(graph)} { ${pattern} } }`;
  }

  /**
   * RDF/JS `match`-style quad lookup via a generated SELECT.
   * `null`/`undefined`/Variable positions are wildcards (the graph wildcard
   * spans the default graph and all named graphs); blank-node positions are
   * matched by label (scanned as a variable and filtered, since SPARQL cannot
   * name a specific bnode). Stores loaded without `options.dataset` hold all
   * quads in the default graph, so a named `graph` argument matches nothing
   * there.
   */
  match(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): Quad[] {
    if (graph && !['Variable', 'DefaultGraph', 'NamedNode', 'BlankNode'].includes(graph.termType)) return [];

    const s = position(subject, 's');
    const p = position(predicate, 'p');
    const o = position(object, 'o');
    const allFixed = Boolean(s.fixed && p.fixed && o.fixed);
    // When all three triple positions are constants, probe with a variable
    // subject and post-filter (the engine needs ≥1 projected variable).
    const pattern = `${allFixed ? '?s' : s.sparql} ${p.sparql} ${o.sparql}`;
    const sparql = `SELECT * WHERE ${SparqStore.#graphScope(pattern, graph)}`;

    const quads: Quad[] = [];
    for (const row of this.queryBindings(sparql)) {
      const subjectTerm = s.fixed && !allFixed ? s.fixed : row.get('s')!;
      const predicateTerm = p.fixed ?? row.get('p')!;
      const objectTerm = o.fixed ?? row.get('o')!;
      // ?g is bound in named-graph branches; unbound (default graph) otherwise.
      const graphTerm = graph?.termType === 'NamedNode' ? graph : (row.get('g') as RDF.Quad_Graph | undefined);
      // Post-filters: a fixed blank node compares by label; the all-constant
      // probe compares the scanned subject against the requested one.
      if (subject?.termType === 'BlankNode' && !subject.equals(subjectTerm)) continue;
      if (predicate?.termType === 'BlankNode') continue; // predicates are never bnodes
      if (object?.termType === 'BlankNode' && !object.equals(objectTerm)) continue;
      if (graph?.termType === 'BlankNode' && !graph.equals(graphTerm)) continue;
      if (allFixed && subject && !subject.equals(subjectTerm)) continue;
      quads.push(
        new Quad(
          subjectTerm as RDF.Quad_Subject,
          predicateTerm as RDF.Quad_Predicate,
          objectTerm as RDF.Quad_Object,
          graphTerm,
        ),
      );
    }
    return quads;
  }

  /** `match(…).length` without materialising terms where possible. */
  countQuads(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): number {
    if (graph && !['Variable', 'DefaultGraph', 'NamedNode', 'BlankNode'].includes(graph.termType)) return 0;
    const needsPostFilter =
      subject?.termType === 'BlankNode' ||
      object?.termType === 'BlankNode' ||
      predicate?.termType === 'BlankNode' ||
      graph?.termType === 'BlankNode';
    if (needsPostFilter) return this.match(subject, predicate, object, graph).length;
    const s = position(subject, 's');
    const p = position(predicate, 'p');
    const o = position(object, 'o');
    if (!s.fixed || !p.fixed || !o.fixed) {
      return this.#inner.count(`SELECT * WHERE ${SparqStore.#graphScope(`${s.sparql} ${p.sparql} ${o.sparql}`, graph)}`);
    }
    return this.match(subject, predicate, object, graph).length;
  }

  /**
   * Applies a SPARQL 1.1 Update (`INSERT DATA`, `DELETE DATA`, `CLEAR`,
   * `DELETE/INSERT … WHERE`, `DROP`/`CREATE`/`ADD`/`COPY`/`MOVE`) over the
   * full dataset — `GRAPH` blocks and graph templates address named graphs
   * (load with `options.dataset` to start from a dataset). Applied IN PLACE
   * through the engine's delta overlay: data operations are O(batch) per
   * target graph (no index rebuild), and the dictionary grows append-only.
   */
  update(sparql: string): void {
    this.#inner.updateInPlace(sparql);
  }

  /**
   * Incremental quad-level delta, mirroring the Rust `Graph::apply_delta`
   * API: applies `deletes` first, then `inserts`, as one O(batch) batch
   * through the delta overlay — no index rebuild — routed per graph (named
   * graphs are auto-created on first insert; load with `options.dataset` for
   * named-graph data to be addressable). Unlike SPARQL `DELETE DATA`, blank
   * nodes here denote concrete nodes BY LABEL, so bnode triples can be
   * retracted.
   */
  applyDelta(inserts: Iterable<RDF.Quad>, deletes: Iterable<RDF.Quad> = []): void {
    this.#inner.applyDelta(quadsToNQuads(inserts), quadsToNQuads(deletes));
  }

  /** `applyDelta(quads, [])` — incremental O(batch) insertion. */
  addQuads(quads: Iterable<RDF.Quad>): void {
    this.applyDelta(quads, []);
  }

  /** `applyDelta([], quads)` — incremental O(batch) removal (bnodes by label). */
  removeQuads(quads: Iterable<RDF.Quad>): void {
    this.applyDelta([], quads);
  }

  /** Releases the wasm-side memory. The store must not be used afterwards. */
  free(): void {
    this.#inner.free();
  }

  [Symbol.dispose](): void {
    this.free();
  }
}

export { Bindings, DataFactory };
