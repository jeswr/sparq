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

export interface WasmStore {
  readonly size: number;
  query(sparql: string): string; // SPARQL 1.1 JSON results document
  queryQuads(sparql: string): string; // CONSTRUCT/DESCRIBE -> N-Triples
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
