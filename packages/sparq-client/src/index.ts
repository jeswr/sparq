// [OPUS-4.8] sq-2e93 — framework-agnostic sparq WASM client.
//
// This is the ONE TypeScript declaration of the sparq-wasm `Store` surface plus the
// loaders / query helpers around it. It exists to kill the drift liability the GUI design
// record (`research/gui-design.md` §0 / §4) flags: the WASM `Store` TS interface was
// hand-redeclared inside `site/src/lib/sparq-wasm.ts`, so any future GUI would have become
// a THIRD hand-copy. Both the Next.js site and the (proposed) Tauri 2 GUI now consume this
// single source.
//
// It is deliberately FRAMEWORK-AGNOSTIC: no React, no Next.js, no DOM beyond the two
// globals the wasm-pack glue needs at load time (`window.location`, dynamic `import()`).
// Everything here ports unchanged to any GUI framework the maintainer later prefers — the
// design's load-bearing "part (A) transfers" property.
//
// It does NOT re-implement the engine. `loadSparq()` fetches the same wasm-pack glue the
// site already syncs into `public/wasm/` (the lean `@jeswr/sparq` bundle, built with
// `--features shacl,jsonld`), so this stays a thin, honest wrapper. No performance claim is
// made anywhere here (this repo's work box is non-canonical); a caller MAY time a single
// query with `performance.now()` and label it as a measured per-query latency, but must
// never bake in a benchmark number.

// ---------------------------------------------------------------------------
// The WASM `Store` surface (the single declaration; mirrors crates/sparq-wasm).
// ---------------------------------------------------------------------------

/**
 * A forward-only cursor over a SELECT result's solution rows, mirroring the wasm
 * `SolutionCursor` export ({@link WasmStore.queryCursor}). Each {@link next} yields the
 * next BATCH of up to `batchSize` solutions as a SELF-CONTAINED SPARQL 1.1 JSON document
 * (so it can be `JSON.parse`d on its own), or `undefined` once every solution has been
 * yielded. The consumer never holds more than one batch, so peak JS memory is bounded by
 * `batchSize` rather than by the whole result. `vars`, `rowCount` and `batchSize`
 * introspect the (wasm-side materialised) result; `free` releases the cursor (call it if
 * you stop early).
 */
export interface WasmSolutionCursor {
  next(): string | undefined;
  vars(): string[];
  rowCount(): number;
  batchSize(): number;
  free(): void;
}

/**
 * The sparq-wasm `Store` surface, as exposed across the wasm-bindgen boundary. This is the
 * ONE hand-written mirror of the wasm-pack-generated `sparq_wasm.d.ts`; the GUI design
 * record's §4 plan to re-export the generated `.d.ts` directly is deferred (see the package
 * README) — for now this single copy replaces the two that previously existed.
 */
export interface WasmStore {
  readonly size: number;
  query(sparql: string): string; // SELECT/ASK -> SPARQL 1.1 JSON results document
  queryQuads(sparql: string): string; // CONSTRUCT/DESCRIBE -> N-Triples document
  // SPARQL Update (mutates this store in place) and the EXPLAIN introspection forms.
  updateInPlace(sparql: string): void; // INSERT/DELETE/LOAD/graph-mgmt -> mutate store
  explain(sparql: string): string; // planning-only plan text (every query form)
  explainAnalyze(sparql: string): string; // plan + per-operator trace (SELECT/ASK only)
  // The @jeswr/sparq browser-API surface: a streaming cursor, a materialisation-free
  // count, the ASK fast path, and an incremental quad-level delta (applied in place
  // through the engine's overlay).
  count(sparql: string): number; // SELECT solution count, read from the index where possible
  ask(sparql: string): boolean; // ASK fast path: a plain boolean, no SELECT materialised
  queryCursor(sparql: string, batchSize: number): WasmSolutionCursor; // batched row cursor
  applyDelta(inserts: string, deletes: string): void; // O(batch) N-Quads delta, in place
  // The SHACL `validate` binding. Present only when the bundle is built with
  // `--features shacl` (the site's default `build:wasm`, and the published `@jeswr/sparq`,
  // both enable it). Stateless: it does NOT consult the receiver's stored triples — it
  // parses `data`/`shapes` and validates one-shot.
  validate?: (data: string, shapes: string, format: string) => string;
}

/** The static `Store` constructor surface (the two named-graph load modes). */
export interface WasmStoreCtor {
  /** Loads RDF, FOLDING any named graphs into the default graph. */
  load(text: string, format: string): WasmStore;
  /** Loads RDF, PRESERVING named graphs (N-Quads / TriG) as separate graphs. */
  loadDataset(text: string, format: string): WasmStore;
}

/** The wasm-pack ESM module surface this client loads at runtime. */
export interface WasmModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  Store: WasmStoreCtor;
}

// ---------------------------------------------------------------------------
// SPARQL 1.1 JSON + SHACL report shapes (for rendering).
// ---------------------------------------------------------------------------

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

/** One row of a SELECT result (a variable → term binding map). */
export type SparqlBinding = SparqlResults["results"]["bindings"][number];

/**
 * One SHACL validation result, the per-violation record the wasm `Store.validate` binding
 * emits (a drop-in for `rdf-validate-shacl`'s results). `focusNode`/`value`/`sourceShape`
 * are N-Triples term strings; `path` is a SHACL Turtle path expression;
 * `sourceConstraintComponent`/`severity` are full IRIs. `path`/`value`/`message` are `null`
 * when the result carries none.
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
 * A SHACL validation report. `conforms` counts EVERY result regardless of severity (the
 * W3C-suite notion); `results` is the per-violation list.
 */
export interface ShaclReport {
  conforms: boolean;
  results: ShaclResult[];
}

// ---------------------------------------------------------------------------
// The runtime loader (the single cold-start path).
// ---------------------------------------------------------------------------

/**
 * Configuration for {@link loadSparq}. `basePath` is the URL prefix under which the
 * wasm-pack glue (`sparq_wasm.js`) and the wasm binary (`sparq_wasm_bg.wasm`) are served —
 * e.g. `"/sparq"` on GitHub Pages, `""` for a local dev server, or a `tauri://`/`file://`
 * origin for the desktop GUI. Defaulted from `NEXT_PUBLIC_BASE_PATH` (falling back to
 * `"/sparq"`) when not given, so the site keeps its current behaviour with no call-site
 * change.
 */
export interface LoadSparqOptions {
  basePath?: string;
}

function defaultBasePath(): string {
  // The Next.js site sets basePath '/sparq'; mirror it for the runtime asset URLs.
  // Empty in local `next dev` without basePath, '/sparq' on Pages. A GUI host passes its
  // own (e.g. "") explicitly via {@link LoadSparqOptions}.
  const env =
    typeof process !== "undefined" ? process.env?.NEXT_PUBLIC_BASE_PATH : undefined;
  return env ?? "/sparq";
}

let modulePromise: Promise<WasmModule> | null = null;
let modulePromiseBase: string | null = null;

/**
 * Loads + initialises the wasm engine once; subsequent calls reuse it.
 *
 * The fetch + compile + instantiate is the expensive cold start. {@link prewarmSparq}
 * kicks this off eagerly on route mount so the first query pays no cold start. The glue is
 * imported with `webpackIgnore` so it never enters a bundler's graph — it is fetched as a
 * plain ESM module at runtime, and the wasm URL is passed explicitly (the glue's own
 * `new URL('./sparq_wasm_bg.wasm', import.meta.url)` resolution is bypassed). If the
 * resolved base path changes between calls (e.g. site vs GUI), the module is re-loaded.
 */
export async function loadSparq(opts: LoadSparqOptions = {}): Promise<WasmStoreCtor> {
  const base = opts.basePath ?? defaultBasePath();
  if (!modulePromise || modulePromiseBase !== base) {
    modulePromiseBase = base;
    modulePromise = (async () => {
      const gluePath = `${base}/wasm/sparq_wasm.js`;
      const wasmPath = `${base}/wasm/sparq_wasm_bg.wasm`;
      // webpackIgnore keeps the glue out of the bundle: it's fetched as a plain ESM
      // module from the static asset root at runtime.
      const mod = (await import(/* webpackIgnore: true */ gluePath)) as WasmModule;
      const origin =
        typeof window !== "undefined" ? window.location.origin : undefined;
      await mod.default({
        module_or_path: origin ? new URL(wasmPath, origin) : wasmPath,
      });
      return mod;
    })();
    modulePromise.catch(() => {
      modulePromise = null; // allow retry on transient failure
      modulePromiseBase = null;
    });
  }
  const mod = await modulePromise;
  return mod.Store;
}

/**
 * Eagerly pre-warm the wasm engine (fetch + compile + instantiate) without blocking
 * render. Safe to call on route mount and to call repeatedly — it shares the single
 * {@link loadSparq} promise, so the cold start happens at most once. Returns a promise that
 * resolves when the engine is ready (or rejects on a load failure, which resets the cache
 * so a later call can retry).
 */
export function prewarmSparq(opts: LoadSparqOptions = {}): Promise<unknown> {
  return loadSparq(opts);
}

// ---------------------------------------------------------------------------
// RDF/JS `match()`-style helpers (framework-agnostic; no dataset-format coupling).
// ---------------------------------------------------------------------------

// The RDF/JS `match()` term shape: `null` (and the empty string in a UI) is a wildcard; a
// non-empty string is an N-Triples term (`<iri>`, `"literal"`, `"v"^^<dt>` …) inlined
// verbatim into the generated SELECT.
export type MatchTerm = string | null;

/** One position of a generated triple pattern: a constant term or a fresh variable. */
function patternPosition(term: MatchTerm, variable: string): string {
  const t = term?.trim();
  return t ? t : `?${variable}`;
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

/**
 * RDF/JS `match()`-style quad lookup over a wasm {@link WasmStore}, the same generated-SELECT
 * strategy `@jeswr/sparq`'s `SparqStore.match` uses: each of subject/predicate/object/graph
 * is either a wildcard (a fresh `?s`/`?p`/`?o`/`?g` variable) or an inlined constant
 * N-Triples term. The graph wildcard spans the default graph AND every named graph. Returns
 * the matching rows as SPARQL-JSON bindings.
 */
export function matchQuads(
  store: WasmStore,
  subject: MatchTerm = null,
  predicate: MatchTerm = null,
  object: MatchTerm = null,
  graph: MatchTerm = null,
): SparqlBinding[] {
  const json = store.query(matchSelect(subject, predicate, object, graph));
  return (JSON.parse(json) as SparqlResults).results.bindings;
}

/**
 * `match(…).length` without materialising the matched terms, via the engine's index-level
 * {@link WasmStore.count} (the count of a generated SELECT over the same pattern
 * {@link matchQuads} builds). This is `countQuads()` in `@jeswr/sparq`.
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

/** One batch a {@link streamQueryRows} pull surfaced: the rows plus the running totals. */
export interface CursorBatch {
  /** 1-based index of this batch. */
  index: number;
  /** The rows in THIS batch (at most `batchSize`). */
  rows: SparqlBinding[];
  /** Total rows yielded so far, across all batches pulled. */
  cumulative: number;
}

/**
 * Stream a SELECT query's rows one BATCH at a time through the wasm `queryCursor` cursor
 * (mirrors `@jeswr/sparq`'s `queryBindingsStream`): pull a batch of up to `batchSize`
 * self-contained SPARQL-JSON rows, hand it to `onBatch`, drop it, pull the next — so the
 * consumer never holds more than one batch. Returns the cursor's introspection (`vars`,
 * `rowCount`, `batchSize`) once drained. The wasm cursor is always freed, even if `onBatch`
 * throws. An empty result yields exactly one empty batch.
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

// ---------------------------------------------------------------------------
// SHACL validation (stateless one-shot) + term rendering.
// ---------------------------------------------------------------------------

/**
 * Validate an RDF **data** document against a SHACL **shapes** document, entirely in-tab via
 * the shacl-enabled wasm bundle. Both arguments use the syntaxes the loader accepts. A clear
 * error is thrown if the loaded bundle was built without the `shacl` feature (no `validate`
 * binding), so a caller can distinguish "no SHACL in this bundle" from a parse error. The
 * optional `loadOpts` is forwarded to {@link loadSparq} so a GUI host can pass its own
 * `basePath`.
 */
export async function sparqShaclValidate(
  data: string,
  shapes: string,
  format = "turtle",
  loadOpts: LoadSparqOptions = {},
): Promise<ShaclReport> {
  const Store = await loadSparq(loadOpts);
  // `validate` is a stateless one-shot, but it is exposed as an instance method on the wasm
  // `Store`, so we need any receiver. An empty store is the cheapest.
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
