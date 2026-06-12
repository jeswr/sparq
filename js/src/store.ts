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
  quadsToNQuads,
  SparqlJsonRowsParser,
  termFromSparqlJson,
  termToNT,
  type SparqlJsonResults,
  type SparqlJsonTerm,
} from './sparql.js';
import { DataFactory, Quad, Variable } from './terms.js';
import { init, WasmStore } from './wasm.js';

export type RdfFormat = 'turtle' | 'ntriples' | 'nquads' | 'trig';

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
   * `format`: `"turtle"` (default) | `"ntriples"` | `"nquads"` | `"trig"`.
   * Named graphs are folded into the default graph unless `options.dataset`
   * is set, in which case they are preserved as separate graphs.
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
   * CONSTRUCT/DESCRIBE are not supported by the engine yet.
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
   * Runs an ASK query natively in the engine (evaluation early-exits at the
   * first solution; a single-pattern ASK is answered straight from the index).
   * Returns the `boolean` of the engine's SPARQL 1.1 JSON results form.
   */
  queryBoolean(sparql: string): boolean {
    const json = JSON.parse(this.#inner.query(sparql)) as SparqlJsonResults;
    if (typeof json.boolean !== 'boolean') throw new Error('queryBoolean() requires an ASK query');
    return json.boolean;
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
   * Counts the solutions of a SELECT query without materialising them
   * (read straight from the index where possible).
   */
  count(sparql: string): number {
    return this.#inner.count(sparql);
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
