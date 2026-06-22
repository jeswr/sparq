// [OPUS-4.8] sq-iwhl8 (#1116) — the RDF/JS Stream-spec surface for `@jeswr/sparq`:
// a browser-safe quad `Stream`, and a `Source` / `Sink` / `Store` adapter over a
// {@link SparqStore} so the engine is usable wherever an RDF/JS Stream interface is expected
// (https://rdf.js.org/stream-spec/). This completes the term/factory/dataset surface with the
// Stream/Source/Sink corner the conformance audit flagged.
//
// The store itself exposes the synchronous, materialised `match(...)` → `Quad[]` shape (the
// shape SPARQL consumers want); these adapters re-view that as the spec's EVENT-BASED `Stream`
// (a `data`-per-quad-then-`end` EventEmitter) WITHOUT pulling in `node:events`, so they run
// unchanged in the browser.
import type * as RDF from '@rdfjs/types';
import { DefaultGraph, NamedNode, type Quad } from './terms.js';
import type { SparqStore } from './store.js';

const DEFAULT_GRAPH = new DefaultGraph();

/** RDF/JS `match()` term position: a concrete term, or `null`/`undefined` for a wildcard. */
type MatchTerm = RDF.Term | null | undefined;

/** An event listener registered on {@link QuadStream}. */
type StreamListener = (arg?: unknown) => void;

/**
 * A minimal, browser-safe RDF/JS `Stream<Quad>` over an in-memory array of quads. It implements
 * exactly the `EventEmitter` subset the RDF/JS Stream spec exercises (`on` / `once` /
 * `removeListener` / `off` / `emit` / `read`) rather than depending on `node:events`, so it runs
 * unchanged in the browser. It emits a `data` event per quad then `end` on the next microtask, so
 * listeners attached synchronously after construction still receive every event; it is also
 * `read()`-able per the spec (pulls the next buffered quad, or `null`).
 *
 * It implements the `EventEmitter` SUBSET the RDF/JS Stream spec actually exercises rather than
 * the full `node:events` surface (so it stays browser-safe); call sites bridge to the full
 * `RDF.Stream` type via {@link asStream}.
 */
export class QuadStream {
  readonly #quads: RDF.Quad[];
  #i = 0;
  #flushed = false;
  readonly #listeners = new Map<string, Set<StreamListener>>();

  constructor(quads: RDF.Quad[]) {
    this.#quads = quads;
    queueMicrotask(() => this.#flush());
  }

  read(): RDF.Quad | null {
    return this.#i < this.#quads.length ? this.#quads[this.#i++]! : null;
  }

  on(event: string | symbol, listener: (...args: never[]) => void): this {
    const key = String(event);
    let set = this.#listeners.get(key);
    if (!set) this.#listeners.set(key, (set = new Set()));
    set.add(listener as StreamListener);
    return this;
  }

  once(event: string | symbol, listener: (...args: never[]) => void): this {
    const wrapper: StreamListener = (arg) => {
      this.removeListener(event, wrapper as (...args: never[]) => void);
      (listener as StreamListener)(arg);
    };
    return this.on(event, wrapper as (...args: never[]) => void);
  }

  removeListener(event: string | symbol, listener: (...args: never[]) => void): this {
    this.#listeners.get(String(event))?.delete(listener as StreamListener);
    return this;
  }

  off(event: string | symbol, listener: (...args: never[]) => void): this {
    return this.removeListener(event, listener);
  }

  emit(event: string | symbol, ...args: unknown[]): boolean {
    const set = this.#listeners.get(String(event));
    if (!set || set.size === 0) return false;
    for (const listener of [...set]) listener(args[0]);
    return true;
  }

  #flush(): void {
    if (this.#flushed) return;
    this.#flushed = true;
    for (const q of this.#quads) this.emit('data', q);
    this.emit('end');
  }
}

/**
 * Bridges a browser-safe {@link QuadStream} (which implements only the `EventEmitter` subset the
 * RDF/JS Stream spec exercises) to the full `RDF.Stream<Quad>` interface (whose nominal type
 * extends the whole `node:events` `EventEmitter`). The subset is exactly what consumers use, so
 * the cast is faithful — see the same pattern in `dataset.ts`.
 */
function asStream(s: QuadStream): RDF.Stream<RDF.Quad> {
  return s as unknown as RDF.Stream<RDF.Quad>;
}

/**
 * [OPUS-4.8] sq-iwhl8 (#1116) — an RDF/JS **`Source`** + **`Sink`** + **`Store`** adapter over a
 * {@link SparqStore}. `Source.match` returns a quad `Stream` (the spec's event-based shape, not
 * the store's synchronous `Quad[]`); `Sink.import` / `Store.remove` consume a quad `Stream` and
 * return an EventEmitter that signals `end` / `error`; `removeMatches` / `deleteGraph` mutate by
 * pattern. The backing store stays the source of truth — the adapter only re-views its
 * synchronous primitives as the streaming interface, so a sparq store drops into any RDF/JS
 * pipeline that speaks the Stream spec (a parser sink, a serializer source, …).
 */
export class SparqSource implements RDF.Store<RDF.Quad> {
  readonly #store: SparqStore;

  constructor(store: SparqStore) {
    this.#store = store;
  }

  /** The backing {@link SparqStore} — the full SPARQL + delta surface. */
  get store(): SparqStore {
    return this.#store;
  }

  /** RDF/JS `Source.match`: a quad `Stream` of the quads matching the pattern (wildcards are `null`). */
  match(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): RDF.Stream<RDF.Quad> {
    return asStream(new QuadStream(this.#store.match(subject, predicate, object, graph) as Quad[]));
  }

  /**
   * RDF/JS `Sink.import` / `Store` insert: consumes the quad `Stream`, buffering its quads and
   * applying them as one O(batch) delta on `end`. Returns an EventEmitter that emits `end` once
   * the batch is applied (or `error` if it fails) — the spec's fire-and-forget consume contract.
   */
  import(stream: RDF.Stream<RDF.Quad>): RDF.Stream<RDF.Quad> {
    return this.#consume(stream, (quads) => this.#store.addQuads(quads));
  }

  /** RDF/JS `Store.remove`: consumes the quad `Stream` and removes its quads as one O(batch) delta. */
  remove(stream: RDF.Stream<RDF.Quad>): RDF.Stream<RDF.Quad> {
    return this.#consume(stream, (quads) => this.#store.removeQuads(quads));
  }

  /** RDF/JS `Store.removeMatches`: removes every quad matching the pattern; signals `end` when done. */
  removeMatches(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): RDF.Stream<RDF.Quad> {
    const out = new QuadStream([]);
    queueMicrotask(() => {
      try {
        const toRemove = this.#store.match(subject, predicate, object, graph);
        if (toRemove.length > 0) this.#store.removeQuads(toRemove);
        out.emit('end');
      } catch (err) {
        out.emit('error', err instanceof Error ? err : new Error(String(err)));
      }
    });
    return asStream(out);
  }

  /**
   * RDF/JS `Store.deleteGraph`: removes every quad in the given graph. A `NamedNode`/`BlankNode`
   * (or a non-empty string IRI) targets that named graph; `DefaultGraph` or `''` targets the
   * default graph. Signals `end` when the batch is applied.
   */
  deleteGraph(graph: RDF.Quad_Graph | string): RDF.Stream<RDF.Quad> {
    const g: RDF.Quad_Graph =
      typeof graph === 'string' ? (graph === '' ? DEFAULT_GRAPH : new NamedNode(graph)) : graph;
    return this.removeMatches(undefined, undefined, undefined, g);
  }

  #consume(stream: RDF.Stream<RDF.Quad>, apply: (quads: RDF.Quad[]) => void): RDF.Stream<RDF.Quad> {
    const out = new QuadStream([]);
    const buffered: RDF.Quad[] = [];
    stream.on('data', (quad: RDF.Quad) => buffered.push(quad));
    stream.on('error', (err: Error) => out.emit('error', err));
    stream.on('end', () => {
      try {
        if (buffered.length > 0) apply(buffered);
        out.emit('end');
      } catch (err) {
        out.emit('error', err instanceof Error ? err : new Error(String(err)));
      }
    });
    return asStream(out);
  }
}
