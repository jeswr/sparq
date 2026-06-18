// [OPUS-4.8] sq-8thu — live wasm loader for the in-browser SPARQL REPL.
//
// The lean sparq-wasm bundle (core parser + triplestore + SPARQL engine) is built
// by `js/`'s `wasm-pack build --target web` and copied to `public/wasm/`. We load
// the wasm-pack glue (`sparq_wasm.js`) at RUNTIME with a dynamic import that webpack
// is told to ignore, so the ~1.2 MB wasm never enters the page bundle and the glue's
// `new URL('./sparq_wasm_bg.wasm', import.meta.url)` resolution is bypassed — we pass
// the wasm bytes' URL explicitly, prefixed with the Pages basePath. This is genuinely
// live: every query runs the real Rust engine compiled to wasm, in your tab.

import {
  ALL_QUADS_BODY,
  ALL_QUADS_QUERY,
  isDatasetFormat,
  rowsToNQuads,
} from "./repl-dataset";

/**
 * [OPUS-4.8] sq-5teq — a forward-only cursor over a SELECT result's solution rows,
 * mirroring the wasm `SolutionCursor` export ({@link WasmStore.queryCursor}). Each
 * {@link next} yields the next BATCH of up to `batchSize` solutions as a SELF-CONTAINED
 * SPARQL 1.1 JSON document (so it can be `JSON.parse`d on its own), or `undefined` once
 * every solution has been yielded. The consumer never holds more than one batch, so
 * peak JS memory is bounded by `batchSize` rather than by the whole result. `vars`,
 * `rowCount` and `batchSize` introspect the (wasm-side materialised) result; `free`
 * releases the cursor (call it if you stop early).
 */
export interface WasmSolutionCursor {
  next(): string | undefined;
  vars(): string[];
  rowCount(): number;
  batchSize(): number;
  free(): void;
}

export interface WasmStore {
  readonly size: number;
  query(sparql: string): string; // SELECT/ASK -> SPARQL 1.1 JSON results document
  queryQuads(sparql: string): string; // CONSTRUCT/DESCRIBE -> N-Triples document
  // [OPUS-4.8] sq-vfbm — the remaining lean-bundle query surface the REPL drives:
  // SPARQL Update (mutates this store in place) and the EXPLAIN introspection forms.
  updateInPlace(sparql: string): void; // INSERT/DELETE/LOAD/graph-mgmt -> mutate store
  explain(sparql: string): string; // planning-only plan text (every query form)
  explainAnalyze(sparql: string): string; // plan + per-operator trace (SELECT/ASK only)
  // [OPUS-4.8] sq-5teq — the @jeswr/sparq browser-API surface the /surface/javascript-wasm
  // page demos live: a streaming cursor, a materialisation-free count, the ASK fast path,
  // and an incremental quad-level delta (applied in place through the engine's overlay).
  count(sparql: string): number; // SELECT solution count, read from the index where possible
  ask(sparql: string): boolean; // ASK fast path: a plain boolean, no SELECT materialised
  queryCursor(sparql: string, batchSize: number): WasmSolutionCursor; // batched row cursor
  applyDelta(inserts: string, deletes: string): void; // O(batch) N-Quads delta, in place
  // [OPUS-4.8] sq-egy6 — the SHACL `validate` binding. Present only when the bundle
  // is built with `--features shacl` (the site's default `build:wasm`, and the
  // published `@jeswr/sparq`, both enable it). Stateless: it does NOT consult the
  // receiver's stored triples — it parses `data`/`shapes` and validates one-shot.
  validate?: (data: string, shapes: string, format: string) => string;
}

interface WasmModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  Store: {
    /** Loads RDF, FOLDING any named graphs into the default graph. */
    load(text: string, format: string): WasmStore;
    /** Loads RDF, PRESERVING named graphs (N-Quads / TriG) as separate graphs. */
    loadDataset(text: string, format: string): WasmStore;
  };
}

/**
 * [OPUS-4.8] sq-17nw — parses RDF into a store, PRESERVING named graphs for the
 * quad-bearing formats (N-Quads / TriG) by routing them through `loadDataset` instead
 * of `load` (which folds every named graph into the default graph). Triple-only formats
 * (`turtle` / `ntriples`) carry no named graphs, so the cheaper `load` is used. This is
 * the single load entry point the REPL uses for every source (default, picker, upload,
 * URL, and the re-load step of a merge).
 */
export function loadIntoStore(
  Store: WasmModule["Store"],
  text: string,
  format: string,
): WasmStore {
  return isDatasetFormat(format)
    ? Store.loadDataset(text, format)
    : Store.load(text, format);
}

/**
 * [OPUS-4.8] sq-17nw — serialises a store's WHOLE dataset (default graph PLUS every
 * named graph) to N-Quads, by selecting {@link ALL_QUADS_QUERY} and emitting one
 * triple/quad line per solution. Used to merge two stores format-agnostically when the
 * user chooses "add to current": concatenate both stores' N-Quads and re-load with
 * `Store.loadDataset(_, "nquads")` so the named graphs survive the round-trip. (The
 * previous `CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }` saw only the default graph, so
 * merging silently dropped every named graph — the folding bug this fixes.)
 */
export function storeToNQuads(store: WasmStore): string {
  const json = store.query(ALL_QUADS_QUERY);
  const parsed = JSON.parse(json) as SparqlResults;
  return rowsToNQuads(parsed.results.bindings);
}

/**
 * [OPUS-4.8] sq-17nw — the TOTAL number of triples in a store across the default graph
 * and all named graphs. The wasm `store.size` getter counts the DEFAULT graph only
 * (see `loadDataset`'s doc), so it under-reports a dataset; this counts the whole thing
 * via {@link ALL_QUADS_QUERY} for an accurate "N triples" badge. Falls back to
 * `store.size` if the count query fails for any reason.
 */
export function datasetSize(store: WasmStore): number {
  try {
    const json = store.query(`SELECT (COUNT(*) AS ?n) WHERE { ${ALL_QUADS_BODY} }`);
    const parsed = JSON.parse(json) as SparqlResults;
    const n = parsed.results.bindings[0]?.n?.value;
    const parsedN = n != null ? Number.parseInt(n, 10) : NaN;
    return Number.isFinite(parsedN) ? parsedN : store.size;
  } catch {
    return store.size;
  }
}

// [OPUS-4.8] sq-5teq — the RDF/JS `match()` term shape the demo uses: `null` (and the
// empty string in the UI) is a wildcard; a non-empty string is an N-Triples term
// (`<iri>`, `"literal"`, `"v"^^<dt>` …) inlined verbatim into the generated SELECT.
export type MatchTerm = string | null;

/** One position of a generated triple pattern: a constant term or a fresh variable. */
function patternPosition(term: MatchTerm, variable: string): string {
  const t = term?.trim();
  return t ? t : `?${variable}`;
}

/**
 * [OPUS-4.8] sq-5teq — RDF/JS `match()`-style quad lookup over a wasm {@link WasmStore},
 * the same generated-SELECT strategy `@jeswr/sparq`'s `SparqStore.match` uses: each of
 * subject/predicate/object/graph is either a wildcard (a fresh `?s`/`?p`/`?o`/`?g`
 * variable) or an inlined constant N-Triples term. The graph wildcard spans the default
 * graph AND every named graph. Returns the matching rows as SPARQL-JSON bindings.
 */
export function matchQuads(
  store: WasmStore,
  subject: MatchTerm = null,
  predicate: MatchTerm = null,
  object: MatchTerm = null,
  graph: MatchTerm = null,
): SparqlResults["results"]["bindings"] {
  const json = store.query(matchSelect(subject, predicate, object, graph));
  return (JSON.parse(json) as SparqlResults).results.bindings;
}

/**
 * [OPUS-4.8] sq-5teq — `match(…).length` without materialising the matched terms, via the
 * engine's index-level {@link WasmStore.count} (the count of a generated SELECT over the
 * same pattern {@link matchQuads} builds). This is `countQuads()` in `@jeswr/sparq`.
 */
export function countQuads(
  store: WasmStore,
  subject: MatchTerm = null,
  predicate: MatchTerm = null,
  object: MatchTerm = null,
  graph: MatchTerm = null,
): number {
  return store.count(matchSelect(subject, predicate, object, graph));
}

/** The generated SELECT a `match()`/`countQuads()` lookup runs (shared so both agree). */
function matchSelect(
  subject: MatchTerm,
  predicate: MatchTerm,
  object: MatchTerm,
  graph: MatchTerm,
): string {
  const s = patternPosition(subject, "s");
  const p = patternPosition(predicate, "p");
  const o = patternPosition(object, "o");
  const triple = `${s} ${p} ${o}`;
  const g = graph?.trim();
  const where = g
    ? `{ GRAPH ${g} { ${triple} } }`
    : `{ { ${triple} } UNION { GRAPH ?g { ${triple} } } }`;
  return `SELECT * WHERE ${where}`;
}

/** One batch a {@link streamQueryRows} pull surfaced: the rows plus the running totals. */
export interface CursorBatch {
  /** 1-based index of this batch. */
  index: number;
  /** The rows in THIS batch (at most `batchSize`). */
  rows: SparqlResults["results"]["bindings"];
  /** Total rows yielded so far, across all batches pulled. */
  cumulative: number;
}

/**
 * [OPUS-4.8] sq-5teq — stream a SELECT query's rows one BATCH at a time through the wasm
 * `queryCursor` cursor (mirrors `@jeswr/sparq`'s `queryBindingsStream`): pull a batch of
 * up to `batchSize` self-contained SPARQL-JSON rows, hand it to `onBatch`, drop it, pull
 * the next — so the consumer never holds more than one batch. Returns the cursor's
 * introspection (`vars`, `rowCount`, `batchSize`) once drained. The wasm cursor is always
 * freed, even if `onBatch` throws. An empty result yields exactly one empty batch.
 */
export function streamQueryRows(
  store: WasmStore,
  sparql: string,
  batchSize: number,
  onBatch?: (batch: CursorBatch) => void,
): { vars: string[]; rowCount: number; batchSize: number } {
  const cursor = store.queryCursor(sparql, batchSize);
  const meta = {
    vars: cursor.vars(),
    rowCount: cursor.rowCount(),
    batchSize: cursor.batchSize(),
  };
  try {
    let index = 0;
    let cumulative = 0;
    for (;;) {
      const chunk = cursor.next();
      if (chunk === undefined) break;
      const rows = (JSON.parse(chunk) as SparqlResults).results.bindings;
      cumulative += rows.length;
      onBatch?.({ index: ++index, rows, cumulative });
    }
  } finally {
    cursor.free();
  }
  return meta;
}

let modulePromise: Promise<WasmModule> | null = null;

function basePath(): string {
  // next.config.ts sets basePath '/sparq'; mirror it for the runtime asset URLs.
  // Empty in local `next dev` without basePath, '/sparq' on Pages.
  return process.env.NEXT_PUBLIC_BASE_PATH ?? "/sparq";
}

/**
 * [OPUS-4.8] Loads + initialises the wasm engine once; subsequent calls reuse it.
 *
 * The fetch + compile + instantiate is the expensive cold start. {@link prewarmSparq}
 * kicks this off eagerly on route mount so the first `Run query` pays no cold start.
 */
export async function loadSparq(): Promise<WasmModule["Store"]> {
  if (!modulePromise) {
    modulePromise = (async () => {
      const base = basePath();
      const gluePath = `${base}/wasm/sparq_wasm.js`;
      const wasmPath = `${base}/wasm/sparq_wasm_bg.wasm`;
      // webpackIgnore keeps the glue out of the bundle: it's fetched as a plain
      // ESM module from /public at runtime.
      const mod = (await import(/* webpackIgnore: true */ gluePath)) as WasmModule;
      await mod.default({ module_or_path: new URL(wasmPath, window.location.origin) });
      return mod;
    })();
    modulePromise.catch(() => {
      modulePromise = null; // allow retry on transient failure
    });
  }
  const mod = await modulePromise;
  return mod.Store;
}

/**
 * [OPUS-4.8] Eagerly pre-warm the wasm engine (fetch + compile + instantiate) without
 * blocking render. Safe to call on route mount and to call repeatedly — it shares the
 * single {@link loadSparq} promise, so the cold start happens at most once. Returns a
 * promise that resolves when the engine is ready (or rejects on a load failure, which
 * resets the cache so a later `Run query` can retry).
 */
export function prewarmSparq(): Promise<unknown> {
  return loadSparq();
}

// ---- minimal SPARQL 1.1 JSON shapes for rendering ----

export interface SparqlTerm {
  type: "uri" | "literal" | "bnode";
  value: string;
  datatype?: string;
  "xml:lang"?: string;
}

export interface SparqlResults {
  head: { vars: string[] };
  results: { bindings: Record<string, SparqlTerm>[] };
  boolean?: boolean;
}

// ---- SHACL validation report (mirrors crates/sparq-wasm/src/shacl.rs JSON) ----

/**
 * [OPUS-4.8] sq-egy6 — one SHACL validation result, the per-violation record the
 * wasm `Store.validate` binding emits (a drop-in for `rdf-validate-shacl`'s
 * results). `focusNode`/`value`/`sourceShape` are N-Triples term strings; `path`
 * is a SHACL Turtle path expression; `sourceConstraintComponent`/`severity` are
 * full IRIs. `path`/`value`/`message` are `null` when the result carries none.
 */
export interface ShaclResult {
  focusNode: string;
  path: string | null;
  value: string | null;
  sourceShape: string;
  sourceConstraintComponent: string;
  severity: string;
  message: string | null;
}

/**
 * [OPUS-4.8] sq-egy6 — a SHACL validation report. `conforms` counts EVERY result
 * regardless of severity (the W3C-suite notion); `results` is the per-violation list.
 */
export interface ShaclReport {
  conforms: boolean;
  results: ShaclResult[];
}

/**
 * [OPUS-4.8] sq-egy6 — validate an RDF **data** document against a SHACL **shapes**
 * document, entirely in your tab via the shacl-enabled wasm bundle. Both arguments
 * use the same syntaxes {@link loadIntoStore} accepts. A clear error is thrown if the
 * loaded bundle was built without the `shacl` feature (no `validate` binding), so a
 * caller can distinguish "no SHACL in this bundle" from a parse error.
 */
export async function sparqShaclValidate(
  data: string,
  shapes: string,
  format = "turtle",
): Promise<ShaclReport> {
  const Store = await loadSparq();
  // `validate` is a stateless one-shot, but it is exposed as an instance method on
  // the wasm `Store`, so we need any receiver. An empty store is the cheapest.
  const store = Store.load("", format);
  if (typeof store.validate !== "function") {
    throw new Error(
      "This wasm bundle was built without the SHACL feature (no validate binding). " +
        "Rebuild sparq-wasm with --features shacl.",
    );
  }
  return JSON.parse(store.validate(data, shapes, format)) as ShaclReport;
}

/** Renders a term for display, with a compact datatype/lang suffix. */
export function formatTerm(t: SparqlTerm | undefined): string {
  if (!t) return "";
  if (t.type === "uri") return `<${t.value}>`;
  if (t.type === "bnode") return `_:${t.value}`;
  if (t["xml:lang"]) return `"${t.value}"@${t["xml:lang"]}`;
  if (t.datatype && t.datatype !== "http://www.w3.org/2001/XMLSchema#string") {
    const short = t.datatype.replace("http://www.w3.org/2001/XMLSchema#", "xsd:");
    return `"${t.value}"^^${short}`;
  }
  return `"${t.value}"`;
}
