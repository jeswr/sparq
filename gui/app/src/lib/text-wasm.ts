// [SONNET-4.6] sq-9nwab — the GUI-side loader for the tier-b "W-text" wasm bundle
// (crates/sparq-text-wasm), the SAME BM25 full-text search engine the marketing site's
// /surface/full-text page runs (site/src/lib/text-wasm.ts). It is a SEPARATE, lazy-loaded
// bundle from the lean sparq-wasm triplestore engine the workbench queries with: the BM25
// inverted index + text: magic predicates, built on demand over the live store's serialised
// TriG so a query can match an indexed literal.
//
// Like the reason-wasm loader, the wasm-pack glue is imported at RUNTIME with a webpackIgnore
// dynamic import from /public, so the wasm never enters the page bundle; the asset URLs are
// prefixed with the SAME NEXT_PUBLIC_BASE_PATH the rest of the GUI keys off (@/lib/base-path),
// so they resolve under both the Tauri root-relative export and the hosted "/app" sub-path.
// The bundle is OPTIONAL: a build that did not sync it surfaces an honest "search unavailable"
// state at runtime rather than crashing.

import { basePath } from "@/lib/base-path";

/**
 * The JS-facing `TextSearch` surface the W-text bundle exports (mirrors crates/sparq-text-wasm
 * and the site loader). Every method is a STATELESS one-shot: it parses the supplied document,
 * builds the BM25 index, and returns a string — so the JS side never holds a long-lived index
 * handle. Document `format` is one of `"turtle"` | `"ntriples"` | `"nquads"` | `"trig"`
 * (named graphs are folded into the default graph so the index covers every literal).
 */
export interface WasmTextSearch {
  /** Run a SPARQL query with text: magic predicates; returns SPARQL-1.1-JSON string. Throws JsError. */
  query(data: string, format: string, sparql: string): string;
  /** Index footprint: `{"docs":N,"tokens":M,"heapBytes":B,"hasPositions":bool}` JSON. Throws JsError. */
  indexStats(data: string, format: string): string;
}

interface TextModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  TextSearch: WasmTextSearch;
}

let modulePromise: Promise<TextModule> | null = null;

/**
 * [SONNET-4.6] sq-9nwab — load + initialise the W-text bundle once; subsequent calls reuse it.
 * The fetch + compile + instantiate is the expensive cold start (paid the first time the full-text
 * tool is opened). A failed load resets the cache so a later attempt retries (e.g. after a build
 * that syncs the bundle). Rejects when the bundle is not present in this build.
 */
export async function loadTextSearch(): Promise<WasmTextSearch> {
  if (!modulePromise) {
    modulePromise = (async () => {
      const base = basePath();
      const gluePath = `${base}/wasm/text/sparq_text_wasm.js`;
      const wasmPath = `${base}/wasm/text/sparq_text_wasm_bg.wasm`;
      const mod = (await import(/* webpackIgnore: true */ gluePath)) as TextModule;
      await mod.default({ module_or_path: new URL(wasmPath, window.location.origin) });
      return mod;
    })();
    modulePromise.catch(() => {
      modulePromise = null; // allow retry on transient failure / a later synced build
    });
  }
  const mod = await modulePromise;
  return mod.TextSearch;
}
