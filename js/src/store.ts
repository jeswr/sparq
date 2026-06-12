/**
 * `SparqStore`: an RDF/JS-flavoured wrapper around the sparq wasm engine — an
 * immutable-index, dictionary-encoded in-memory store with SPARQL SELECT/ASK
 * (both evaluated natively by the engine) and SPARQL Update (rebuild semantics).
 */
import type * as RDF from '@rdfjs/types';
import { Bindings } from './bindings.js';
import {
  detectQueryForm,
  quadsToNQuads,
  termFromSparqlJson,
  termToNT,
  type SparqlJsonResults,
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
}

type MatchTerm = RDF.Term | null | undefined;

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
   * Named graphs are folded into the default graph.
   */
  static async fromString(data: string, format: RdfFormat = 'turtle', options: SparqStoreOptions = {}): Promise<SparqStore> {
    await init();
    const inner = options.compressed ? WasmStore.loadCompressed(data, format) : WasmStore.load(data, format);
    return new SparqStore(inner);
  }

  /** Builds a store from RDF/JS quads (serialised internally to N-Quads). */
  static async fromQuads(quads: Iterable<RDF.Quad>, options: SparqStoreOptions = {}): Promise<SparqStore> {
    return SparqStore.fromString(quadsToNQuads(quads), 'nquads', options);
  }

  /** The number of (deduplicated) triples in the store. */
  get size(): number {
    return this.#inner.size;
  }

  /** A rough estimate of the store's in-memory footprint, in bytes. */
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
    return rows.map(row => {
      const entries: [RDF.Variable, RDF.Term][] = [];
      for (const [name, term] of Object.entries(row)) {
        entries.push([new Variable(name), termFromSparqlJson(term)]);
      }
      return new Bindings(entries);
    });
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
   * RDF/JS `match`-style triple lookup via a generated SELECT.
   * `null`/`undefined`/Variable positions are wildcards; blank-node positions
   * are matched by label (scanned as a variable and filtered, since SPARQL
   * cannot name a specific bnode). All quads are in the default graph, so a
   * named `graph` argument matches nothing.
   */
  match(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): Quad[] {
    if (graph && graph.termType !== 'Variable' && graph.termType !== 'DefaultGraph') return [];

    const s = position(subject, 's');
    const p = position(predicate, 'p');
    const o = position(object, 'o');
    const vars = [s, p, o].filter(x => !x.fixed).length > 0 ? '*' : '?s';
    // When all three positions are constants, probe with a variable subject
    // and post-filter (the engine needs at least one projected variable).
    const sparqlSubject = vars === '?s' ? '?s' : s.sparql;
    const sparql = `SELECT ${vars === '?s' ? '?s' : '*'} WHERE { ${sparqlSubject} ${p.sparql} ${o.sparql} }`;

    const quads: Quad[] = [];
    for (const row of this.queryBindings(sparql)) {
      const subjectTerm = s.fixed && vars !== '?s' ? s.fixed : row.get('s')!;
      const predicateTerm = p.fixed ?? row.get('p')!;
      const objectTerm = o.fixed ?? row.get('o')!;
      // Post-filters: a fixed blank node compares by label; the all-constant
      // probe compares the scanned subject against the requested one.
      if (subject?.termType === 'BlankNode' && !subject.equals(subjectTerm)) continue;
      if (predicate?.termType === 'BlankNode') continue; // predicates are never bnodes
      if (object?.termType === 'BlankNode' && !object.equals(objectTerm)) continue;
      if (vars === '?s' && subject && !subject.equals(subjectTerm)) continue;
      quads.push(
        new Quad(
          subjectTerm as RDF.Quad_Subject,
          predicateTerm as RDF.Quad_Predicate,
          objectTerm as RDF.Quad_Object,
        ),
      );
    }
    return quads;
  }

  /** `match(…).length` without materialising terms when all-wildcard. */
  countQuads(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): number {
    if (graph && graph.termType !== 'Variable' && graph.termType !== 'DefaultGraph') return 0;
    const needsPostFilter =
      subject?.termType === 'BlankNode' || object?.termType === 'BlankNode' || predicate?.termType === 'BlankNode';
    if (needsPostFilter) return this.match(subject, predicate, object, graph).length;
    const s = position(subject, 's');
    const p = position(predicate, 'p');
    const o = position(object, 'o');
    if (!s.fixed || !p.fixed || !o.fixed) {
      return this.#inner.count(`SELECT * WHERE { ${s.sparql} ${p.sparql} ${o.sparql} }`);
    }
    return this.match(subject, predicate, object, graph).length;
  }

  /**
   * Applies a SPARQL 1.1 Update (`INSERT DATA`, `DELETE DATA`, `CLEAR`,
   * `DELETE/INSERT … WHERE`; default graph only). The engine rebuilds the
   * immutable index (O(store) per update batch), and this wrapper swaps the
   * handle in place, so the `SparqStore` mutates as callers expect.
   */
  update(sparql: string): void {
    const next = this.#inner.update(sparql);
    this.#inner.free();
    this.#inner = next;
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
