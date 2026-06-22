/**
 * [OPUS-4.8] sq-lii76 (#981) / #1047 — `Dataset`: a full RDF/JS **`Dataset`** surface (the
 * complete set algebra, not just `DatasetCore`) over the sparq wasm engine, so the import the
 * issue's `<script type="module">` snippet uses by name —
 *
 *     import { Dataset } from "https://esm.sh/@jeswr/sparq"
 *
 * — resolves to a real, spec-shaped entry, not just the low-level `Store` class. It is a thin
 * wrapper over {@link SparqStore} (which already wraps the engine): the `DatasetCore` members
 * (`add` / `delete` / `has` / `match` / `size` / `[Symbol.iterator]`) map onto the store's
 * existing quad-level delta / `match` / count bindings, and the `Dataset` set algebra
 * (`union` / `intersection` / `difference` / `addAll` / `contains` / `equals` / `filter` /
 * `map` / `forEach` / `some` / `every` / `reduce` / `import` / `toStream` / `toCanonical` / …)
 * is built on top of those primitives.
 *
 * INTEROP (#1047, the maintainer's hard requirement). The binary set ops — `union`,
 * `intersection`, `difference`, `addAll`, `contains`, `equals` — accept an operand that is
 * EITHER another sparq `Dataset` OR a FOREIGN RDF/JS dataset/store (e.g. an `N3.Store`, an
 * `@rdfjs/dataset`). The operand's source is detected through the one member every RDF/JS
 * dataset exposes — `[Symbol.iterator]` (mandated by `DatasetCore`) — so a sparq {@link Dataset}
 * iterates the SAME way an `N3.Store` does; a sparq `Dataset` additionally takes a FAST NATIVE
 * path on {@link union} (its quads bulk-serialise once through the engine). Both paths produce a
 * sparq `Dataset` result, so the engine stays the source of truth for the set semantics.
 *
 * LAZY WASM INIT (the #981 posture). `Dataset`'s instance methods are SYNCHRONOUS by the RDF/JS
 * spec, so the wasm must already be instantiated by the time an instance exists. The `Dataset`
 * constructor is therefore PRIVATE; you obtain one through the async factories
 * {@link Dataset.create} / {@link Dataset.fromString} / {@link Dataset.fromQuads}, each of which
 * `await`s the engine's (memoised) `init()` FIRST. Until you call one of them the ~MB wasm binary
 * is never fetched. The cold start is paid at most once across the whole page (the underlying
 * `init()` is memoised), so two datasets created in the same tab share one engine instance.
 *
 * This is a SUPERSET-by-reference, not a fork: a `Dataset` exposes its backing
 * {@link Dataset.store} so the full SPARQL surface (`queryBindings`, `update`, `queryQuads`,
 * SHACL `validate`, streaming cursors, …) stays one accessor away.
 */
import type * as RDF from '@rdfjs/types';
import { quadsToNQuads } from './sparql.js';
import { SparqStore, type RdfFormat, type SparqStoreOptions } from './store.js';

/** RDF/JS `match()` term position: a concrete term, or `null`/`undefined` for a wildcard. */
type MatchTerm = RDF.Term | null | undefined;

/**
 * The lowest common denominator a binary set op accepts: any RDF/JS dataset/store is iterable
 * over its quads (`DatasetCore` mandates `[Symbol.iterator]`), so an `Iterable<RDF.Quad>` covers
 * a sparq {@link Dataset}, an `N3.Store`, an `@rdfjs/dataset`, or a plain `Quad[]`.
 */
type QuadSource = Iterable<RDF.Quad>;

/**
 * An RDF/JS **`Dataset`** backed by the sparq wasm engine. Construct via the async factories
 * ({@link Dataset.create} / {@link Dataset.fromString} / {@link Dataset.fromQuads}), which
 * lazily instantiate the wasm engine on first use; thereafter the `Dataset` members are
 * synchronous per the spec. Mutations (`add` / `delete` / `addAll` / `deleteMatches`) go through
 * the engine's O(batch) delta overlay — no index rebuild.
 */
export class Dataset implements RDF.Dataset {
  readonly #store: SparqStore;

  private constructor(store: SparqStore) {
    this.#store = store;
  }

  // --- factories ---------------------------------------------------------------------------------

  /**
   * An EMPTY dataset. Awaits the (memoised) wasm `init()` first — this is the lazy-load point:
   * the engine binary is fetched here, not when the module is imported. The dataset is created
   * with named-graph awareness so the set algebra round-trips quads in any graph faithfully;
   * pass `{ dataset: false }` to fold named graphs into the default graph.
   */
  static async create(options: SparqStoreOptions = {}): Promise<Dataset> {
    return new Dataset(await SparqStore.fromString('', 'ntriples', { dataset: true, ...options }));
  }

  /**
   * Parses an RDF document into a dataset (lazily instantiating the wasm engine first).
   * `format` and `options` mirror {@link SparqStore.fromString}. Named graphs (from
   * N-Quads / TriG / a JSON-LD `@graph`) are preserved by default so `match`/`size`/the set
   * algebra are graph-aware; pass `{ dataset: false }` to fold them into the default graph.
   */
  static async fromString(
    data: string,
    format: RdfFormat = 'turtle',
    options: SparqStoreOptions = {},
  ): Promise<Dataset> {
    return new Dataset(await SparqStore.fromString(data, format, { dataset: true, ...options }));
  }

  /**
   * Builds a dataset from RDF/JS quads (lazily instantiating the wasm engine first). Each quad's
   * graph is preserved by default (`{ dataset: true }`).
   */
  static async fromQuads(quads: Iterable<RDF.Quad>, options: SparqStoreOptions = {}): Promise<Dataset> {
    return new Dataset(await SparqStore.fromQuads(quads, { dataset: true, ...options }));
  }

  /**
   * Wraps an already-built {@link SparqStore} as a `Dataset` (no re-parse). The wasm engine is
   * assumed already initialised. Internal — used to materialise the result of a set op.
   */
  private static fromStore(store: SparqStore): Dataset {
    return new Dataset(store);
  }

  /** A NEW sparq-backed dataset over `quads`, built synchronously (engine already up). */
  private static of(quads: QuadSource): Dataset {
    return new Dataset(SparqStore.fromQuadsSync(quads, { dataset: true }));
  }

  /**
   * [OPUS-4.8] sq-iwhl8 (#1116) — a SYNCHRONOUS dataset factory, the building block for the
   * RDF/JS {@link RDF.DatasetCoreFactory} / {@link RDF.DatasetFactory} ({@link datasetFactory})
   * the conformance harness drives. Like {@link fromQuads} but WITHOUT `await init()`: the wasm
   * engine must ALREADY be initialised (a prior `await init()` / `await Dataset.create()` has
   * resolved), so the spec's synchronous `dataset(quads?)` factory shape can be satisfied. Prefer
   * the async {@link create} / {@link fromQuads} for first construction.
   */
  static fromQuadsSync(quads: Iterable<RDF.Quad> = []): Dataset {
    return Dataset.of(quads);
  }

  // --- DatasetCore -------------------------------------------------------------------------------

  /**
   * The backing {@link SparqStore} — the full SPARQL engine surface (`queryBindings`, `update`,
   * `queryQuads`, SHACL `validate`, streaming cursors, …). The `Dataset` and its store share one
   * wasm handle.
   */
  get store(): SparqStore {
    return this.#store;
  }

  /** The number of quads across the default graph AND every named graph (graph-spanning). */
  get size(): number {
    return this.#store.countQuads(null, null, null, null);
  }

  /**
   * Adds a quad through the engine's O(batch) delta overlay (idempotent — the store is a set).
   * Returns `this` per the RDF/JS spec.
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
    return this.#store.countQuads(quad.subject, quad.predicate, quad.object, quad.graph) > 0;
  }

  /**
   * Returns a NEW {@link Dataset} with the quads matching the given term pattern
   * (`null`/`undefined` positions are wildcards; the graph wildcard spans the default graph and
   * every named graph), per RDF/JS quad matching. The result is an independent dataset (its own
   * engine handle); mutating it does not affect this one.
   */
  match(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): Dataset {
    return Dataset.of(this.#store.match(subject, predicate, object, graph));
  }

  /**
   * Iterates every quad across the default graph and all named graphs. The materialised array is
   * built once via the store's graph-spanning `match()`; for a very large dataset prefer the
   * backing {@link store}'s streaming surface (`queryQuadsStream` / `queryBindingsStream`).
   */
  [Symbol.iterator](): Iterator<RDF.Quad> {
    return this.#store.match(null, null, null, null)[Symbol.iterator]();
  }

  // --- Dataset: mutation -------------------------------------------------------------------------

  /**
   * Imports the quads (a sparq {@link Dataset}, a foreign RDF/JS dataset, or a `Quad[]`) into
   * THIS dataset, through one O(batch) delta. Unlike {@link union} this mutates the receiver
   * rather than returning a new dataset. Returns `this`.
   */
  addAll(quads: QuadSource | RDF.Quad[]): this {
    this.#store.addQuads(toQuadArray(quads));
    return this;
  }

  /**
   * Removes the quads matching the given pattern from THIS dataset (graph-aware wildcards), per
   * RDF/JS quad matching. Returns `this`.
   */
  deleteMatches(subject?: MatchTerm, predicate?: MatchTerm, object?: MatchTerm, graph?: MatchTerm): this {
    const toDelete = this.#store.match(subject, predicate, object, graph);
    if (toDelete.length > 0) this.#store.removeQuads(toDelete);
    return this;
  }

  // --- Dataset: set algebra (interop-aware) ------------------------------------------------------

  /**
   * Returns a NEW dataset that is the union of this dataset and `quads`. INTEROP: `quads` may be
   * another sparq {@link Dataset} (FAST native path — its quads are bulk-serialised once) or any
   * FOREIGN RDF/JS dataset / `Quad[]` (GENERIC quad-stream path); either way the engine
   * deduplicates as a set.
   */
  union(quads: QuadSource): Dataset {
    const out = SparqStore.fromQuadsSync(this.#store.match(null, null, null, null), { dataset: true });
    out.addQuads(toQuadArray(quads));
    return Dataset.fromStore(out);
  }

  /**
   * Returns a NEW dataset of the quads in this dataset that are ALSO in `quads`. INTEROP: `quads`
   * may be a sparq {@link Dataset} or any foreign RDF/JS dataset / `Quad[]`.
   */
  intersection(quads: QuadSource): Dataset {
    const other = new QuadSet(quads);
    const kept: RDF.Quad[] = [];
    for (const q of this) if (other.has(q)) kept.push(q);
    return Dataset.of(kept);
  }

  /**
   * Returns a NEW dataset of the quads in this dataset that are NOT in `quads`. INTEROP: `quads`
   * may be a sparq {@link Dataset} or any foreign RDF/JS dataset / `Quad[]`.
   */
  difference(quads: QuadSource): Dataset {
    const other = new QuadSet(quads);
    const kept: RDF.Quad[] = [];
    for (const q of this) if (!other.has(q)) kept.push(q);
    return Dataset.of(kept);
  }

  /**
   * Whether this dataset is a SUPERSET of `quads` (every quad of `quads` is present here).
   * INTEROP: `quads` may be a sparq {@link Dataset} or any foreign RDF/JS dataset / `Quad[]`.
   *
   * NOTE the RDF/JS spec says blank nodes "will be normalized"; this implementation compares
   * quads by their concrete terms (blank-node LABELS), not by full RDF-dataset isomorphism. For
   * the common no-blank-node case the result is exact; with blank nodes it is label-sensitive
   * (see {@link toCanonical}). Tracked by bead sq-1dd5t.
   */
  contains(quads: QuadSource): boolean {
    for (const q of toIterable(quads)) if (!this.has(q)) return false;
    return true;
  }

  /**
   * Whether this dataset contains the SAME quads as `quads` (mutual containment). INTEROP:
   * `quads` may be a sparq {@link Dataset} or any foreign RDF/JS dataset / `Quad[]`.
   *
   * Same blank-node caveat as {@link contains}: equality is by concrete terms, not full
   * RDF-dataset isomorphism (bead sq-1dd5t).
   */
  equals(quads: QuadSource): boolean {
    const other = new QuadSet(quads);
    if (other.size !== this.size) return false;
    for (const q of this) if (!other.has(q)) return false;
    return true;
  }

  // --- Dataset: iteration helpers ----------------------------------------------------------------

  /** `Array.prototype.every` over the quads. Short-circuits on the first failing quad. */
  every(iteratee: (quad: RDF.Quad, dataset: this) => boolean): boolean {
    for (const q of this) if (!iteratee(q, this)) return false;
    return true;
  }

  /** `Array.prototype.some` over the quads. Short-circuits on the first passing quad. */
  some(iteratee: (quad: RDF.Quad, dataset: this) => boolean): boolean {
    for (const q of this) if (iteratee(q, this)) return true;
    return false;
  }

  /** A NEW dataset of the quads for which `iteratee` returns truthy (`Array.prototype.filter`). */
  filter(iteratee: (quad: RDF.Quad, dataset: this) => boolean): Dataset {
    const kept: RDF.Quad[] = [];
    for (const q of this) if (iteratee(q, this)) kept.push(q);
    return Dataset.of(kept);
  }

  /** A NEW dataset of `iteratee` applied to each quad (`Array.prototype.map`). */
  map(iteratee: (quad: RDF.Quad, dataset: RDF.Dataset) => RDF.Quad): Dataset {
    const mapped: RDF.Quad[] = [];
    for (const q of this) mapped.push(iteratee(q, this));
    return Dataset.of(mapped);
  }

  /** `Array.prototype.forEach` over the quads. */
  forEach(callback: (quad: RDF.Quad, dataset: this) => void): void {
    for (const q of this) callback(q, this);
  }

  /** `Array.prototype.reduce` over the quads (with or without an initial value). */
  reduce<A = unknown>(callback: (accumulator: A, quad: RDF.Quad, dataset: this) => A, initialValue?: A): A {
    let acc = initialValue;
    let seeded = arguments.length >= 2;
    for (const q of this) {
      if (!seeded) {
        acc = q as unknown as A;
        seeded = true;
      } else {
        acc = callback(acc as A, q, this);
      }
    }
    if (!seeded) throw new TypeError('reduce of empty dataset with no initial value');
    return acc as A;
  }

  // --- Dataset: materialisation ------------------------------------------------------------------

  /** The quads as a host-language array (order arbitrary — a `Dataset` is an unordered set). */
  toArray(): RDF.Quad[] {
    return this.#store.match(null, null, null, null);
  }

  /**
   * Imports all quads from an RDF/JS quad `Stream` into this dataset; resolves with `this` on the
   * stream's `end`, rejects on `error`. Quads are buffered and applied as one O(batch) delta on
   * `end` (the stream-spec contract is fire-and-forget per quad, so a single batch is faithful and
   * far cheaper than per-quad deltas).
   */
  import(stream: RDF.Stream<RDF.Quad>): Promise<this> {
    return new Promise<this>((resolve, reject) => {
      const buffered: RDF.Quad[] = [];
      stream.on('data', (quad: RDF.Quad) => buffered.push(quad));
      stream.on('error', (err: Error) => reject(err));
      stream.on('end', () => {
        try {
          if (buffered.length > 0) this.#store.addQuads(buffered);
          resolve(this);
        } catch (err) {
          reject(err instanceof Error ? err : new Error(String(err)));
        }
      });
    });
  }

  /** A readable RDF/JS `Stream` over every quad of the dataset. */
  toStream(): RDF.Stream<RDF.Quad> {
    // `ArrayQuadStream` implements the EventEmitter SUBSET the RDF/JS stream spec actually
    // exercises (`on`/`once`/`emit`/`removeListener`/`read`) without pulling in `node:events`
    // (so it stays browser-safe); the cast bridges to the full `EventEmitter`-typed interface.
    return new ArrayQuadStream(this.toArray()) as unknown as RDF.Stream<RDF.Quad>;
  }

  /**
   * An N-Quads serialisation of the dataset (no normalization — the RDF/JS `toString` contract).
   * Order is arbitrary.
   */
  toString(): string {
    return quadsToNQuads(this.toArray());
  }

  /**
   * An N-Quads serialisation with the per-quad lines sorted (a deterministic, repeatable string).
   * This is NOT a full RDFC-1.0 / URDNA2015 canonicalisation — blank-node labels are NOT
   * relabelled to a canonical form, so two isomorphic datasets that differ only in blank-node
   * labels canonicalise differently. For datasets WITHOUT blank nodes the output is a true
   * canonical form. Full blank-node canonicalisation is tracked by bead sq-1dd5t.
   */
  toCanonical(): string {
    return this.toArray()
      .map((q) => quadsToNQuads([q]))
      .sort()
      .join('');
  }

  // --- lifecycle ---------------------------------------------------------------------------------

  /** Releases the wasm-side memory. The dataset must not be used afterwards. */
  free(): void {
    this.#store.free();
  }

  [Symbol.dispose](): void {
    this.free();
  }
}

/**
 * [OPUS-4.8] sq-iwhl8 (#1116) — the RDF/JS **`DatasetCoreFactory`** AND **`DatasetFactory`**
 * (the latter extends the former; the sparq {@link Dataset} is a full `Dataset`, so one object
 * satisfies both). Its single `dataset(quads?)` method is SYNCHRONOUS per the spec, so the wasm
 * engine must already be initialised — `await init()` (or any `await Dataset.create()` /
 * `Dataset.fromString(...)`) before the first `dataset()` call, exactly as the synchronous
 * `Dataset` members require. This is what the RDF/JS conformance harness drives as a
 * `DatasetFactory`.
 */
export const datasetFactory: RDF.DatasetCoreFactory<RDF.Quad, RDF.Quad, Dataset> &
  RDF.DatasetFactory<RDF.Quad, RDF.Quad, Dataset> = {
  dataset(quads?: Iterable<RDF.Quad>): Dataset {
    return Dataset.fromQuadsSync(quads ?? []);
  },
};

// --- interop helpers -----------------------------------------------------------------------------

/** Iterates any quad source (a sparq `Dataset`, a foreign RDF/JS dataset/store, or a `Quad[]`). */
function toIterable(quads: QuadSource): Iterable<RDF.Quad> {
  return quads;
}

/** Materialises any quad source to an array (a fast no-op when it already is one). */
function toQuadArray(quads: QuadSource | RDF.Quad[]): RDF.Quad[] {
  return Array.isArray(quads) ? quads : [...quads];
}

/**
 * A membership set over quads keyed on their N-Quads serialisation — the basis for the
 * INTEROP-aware `intersection` / `difference` / `equals`. Building it from a FOREIGN RDF/JS
 * dataset only relies on `[Symbol.iterator]` (which `DatasetCore` mandates), so an `N3.Store`,
 * an `@rdfjs/dataset`, or a plain `Quad[]` all work; a sparq {@link Dataset} iterates the same
 * way. Keying on N-Quads gives RDF/JS `Quad.equals` semantics (term-by-term, blank nodes by
 * label) without an O(n·m) pairwise scan.
 */
class QuadSet {
  readonly #keys: Set<string>;

  constructor(quads: QuadSource) {
    this.#keys = new Set<string>();
    for (const q of toIterable(quads)) this.#keys.add(quadsToNQuads([q]));
  }

  has(quad: RDF.Quad): boolean {
    return this.#keys.has(quadsToNQuads([quad]));
  }

  get size(): number {
    return this.#keys.size;
  }
}

/** An event listener registered on {@link ArrayQuadStream}. */
type StreamListener = (arg?: unknown) => void;

/**
 * A minimal, browser-safe RDF/JS `Stream` over an in-memory array of quads (for {@link
 * Dataset.toStream}). It implements just the `EventEmitter` surface the RDF/JS stream spec needs
 * (`on` / `once` / `emit` / `removeListener`) rather than depending on `node:events`, so it runs
 * unchanged in the browser. Emits `data` for each quad then `end` on the next microtask, so
 * listeners attached synchronously after the call still receive every event. Also `read()`-able
 * per the stream spec.
 */
class ArrayQuadStream {
  #quads: RDF.Quad[];
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
