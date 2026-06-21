/**
 * [OPUS-4.8] sq-lii76 (#981) — `Dataset`: a small RDF/JS **`DatasetCore`** surface over the
 * sparq wasm engine, so the import the issue's `<script type="module">` snippet uses by name —
 *
 *     import { Dataset } from "https://esm.sh/@jeswr/sparq"
 *
 * — resolves to a real, spec-shaped entry, not just the low-level `Store` class. It is a thin
 * wrapper over {@link SparqStore} (which already wraps the engine), adding nothing the engine
 * does not already do: it exposes the four RDF/JS `DatasetCore` members (`add` / `delete` /
 * `has` / `match`), the `size` getter and `[Symbol.iterator]`, each mapped onto the store's
 * existing quad-level delta / `match` / count bindings.
 *
 * LAZY WASM INIT (the #981 posture). `DatasetCore`'s instance methods are SYNCHRONOUS by the
 * RDF/JS spec, so the wasm must already be instantiated by the time an instance exists. The
 * `Dataset` constructor is therefore PRIVATE; you obtain one through the async factories
 * {@link Dataset.create} / {@link Dataset.fromString} / {@link Dataset.fromQuads}, each of
 * which `await`s the engine's (memoised) `init()` FIRST. Until you call one of them the
 * ~MB wasm binary is never fetched — so a `<script type="module">` that merely imports the
 * name pays nothing, and the engine streams in only on the first `await Dataset.create(...)`.
 * The cold start is paid at most once across the whole page (the underlying `init()` is
 * memoised), so two datasets created in the same tab share one engine instance.
 *
 * This is a SUPERSET-by-reference, not a fork: a `Dataset` exposes its backing
 * {@link Dataset.store} so the full SPARQL surface (`queryBindings`, `update`,
 * `queryQuads`, SHACL `validate`, streaming cursors, …) stays one accessor away — `Dataset`
 * is the RDF/JS-`DatasetCore` ergonomic entry, `SparqStore` is the SPARQL engine handle.
 */
import type * as RDF from '@rdfjs/types';
import { SparqStore, type RdfFormat, type SparqStoreOptions } from './store.js';

/** RDF/JS `match()` term position: a concrete term, or `null`/`undefined` for a wildcard. */
type MatchTerm = RDF.Term | null | undefined;

/**
 * An RDF/JS **`DatasetCore`** backed by the sparq wasm engine. Construct via the async
 * factories ({@link Dataset.create} / {@link Dataset.fromString} / {@link Dataset.fromQuads}),
 * which lazily instantiate the wasm engine on first use; thereafter the `DatasetCore` members
 * are synchronous per the spec. Mutations (`add` / `delete`) go through the engine's O(batch)
 * delta overlay — no index rebuild.
 */
export class Dataset implements RDF.DatasetCore {
  readonly #store: SparqStore;

  private constructor(store: SparqStore) {
    this.#store = store;
  }

  /**
   * An EMPTY dataset. Awaits the (memoised) wasm `init()` first — this is the lazy-load
   * point: the engine binary is fetched here, not when the module is imported. `options`
   * are forwarded to {@link SparqStore.fromString} (e.g. `{ dataset: true }` to keep named
   * graphs addressable; the default folds them into the default graph).
   */
  static async create(options: SparqStoreOptions = {}): Promise<Dataset> {
    return new Dataset(await SparqStore.fromString('', 'ntriples', options));
  }

  /**
   * Parses an RDF document into a dataset (lazily instantiating the wasm engine first).
   * `format` and `options` mirror {@link SparqStore.fromString} — pass `{ dataset: true }`
   * to preserve named graphs from N-Quads / TriG / a JSON-LD `@graph`.
   */
  static async fromString(
    data: string,
    format: RdfFormat = 'turtle',
    options: SparqStoreOptions = {},
  ): Promise<Dataset> {
    return new Dataset(await SparqStore.fromString(data, format, options));
  }

  /**
   * Builds a dataset from RDF/JS quads (lazily instantiating the wasm engine first). With
   * `{ dataset: true }` each quad's graph is preserved; otherwise all quads land in the
   * default graph.
   */
  static async fromQuads(
    quads: Iterable<RDF.Quad>,
    options: SparqStoreOptions = {},
  ): Promise<Dataset> {
    return new Dataset(await SparqStore.fromQuads(quads, options));
  }

  /**
   * The backing {@link SparqStore} — the full SPARQL engine surface (`queryBindings`,
   * `update`, `queryQuads`, SHACL `validate`, streaming cursors, …) for when `DatasetCore`'s
   * four members are not enough. The `Dataset` and its store share one wasm handle.
   */
  get store(): SparqStore {
    return this.#store;
  }

  /**
   * The number of quads across the default graph AND every named graph (RDF/JS
   * `DatasetCore.size` spans the whole dataset). Computed via a `COUNT(*)` over a
   * graph-spanning pattern so a dataset loaded with `{ dataset: true }` reports its named
   * graphs too (the store's plain `size` counts the default graph only).
   */
  get size(): number {
    return this.#store.countQuads(null, null, null, null);
  }

  /**
   * Adds a quad through the engine's O(batch) delta overlay (idempotent — the store is a
   * set, so re-adding an existing quad is a no-op). Returns `this` per the RDF/JS spec.
   */
  add(quad: RDF.Quad): this {
    this.#store.addQuads([quad]);
    return this;
  }

  /**
   * Removes a quad through the engine's O(batch) delta overlay (a no-op if absent). Returns
   * `this` per the RDF/JS spec.
   */
  delete(quad: RDF.Quad): this {
    this.#store.removeQuads([quad]);
    return this;
  }

  /** Whether the dataset contains `quad` (graph-aware). */
  has(quad: RDF.Quad): boolean {
    return (
      this.#store.countQuads(quad.subject, quad.predicate, quad.object, quad.graph) > 0
    );
  }

  /**
   * Returns a NEW {@link Dataset} with the quads matching the given term pattern
   * (`null`/`undefined` positions are wildcards; the graph wildcard spans the default graph
   * and every named graph), per RDF/JS `DatasetCore.match`. The result is an independent
   * dataset (its own engine handle); mutating it does not affect this one. Always preserves
   * named graphs so a graph-spanning match round-trips faithfully.
   */
  match(
    subject?: MatchTerm,
    predicate?: MatchTerm,
    object?: MatchTerm,
    graph?: MatchTerm,
  ): Dataset {
    const quads = this.#store.match(subject, predicate, object, graph);
    // `match()` is synchronous per the RDF/JS spec, and the wasm engine is already
    // initialised by the time any Dataset instance exists (the async factories awaited
    // `init()`), so the SYNC `fromQuadsSync` is safe here — it never re-fetches the engine.
    return new Dataset(SparqStore.fromQuadsSync(quads, { dataset: true }));
  }

  /**
   * Iterates every quad across the default graph and all named graphs. The materialised
   * array is built once via the store's graph-spanning `match()`; for a very large dataset
   * prefer the backing {@link store}'s streaming surface (`queryQuadsStream` /
   * `queryBindingsStream`) instead.
   */
  [Symbol.iterator](): Iterator<RDF.Quad> {
    return this.#store.match(null, null, null, null)[Symbol.iterator]();
  }

  /** Releases the wasm-side memory. The dataset must not be used afterwards. */
  free(): void {
    this.#store.free();
  }

  [Symbol.dispose](): void {
    this.free();
  }
}
