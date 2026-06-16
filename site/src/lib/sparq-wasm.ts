// [OPUS-4.8] sq-8thu — live wasm loader for the in-browser SPARQL REPL.
//
// The lean sparq-wasm bundle (core parser + triplestore + SPARQL engine) is built
// by `js/`'s `wasm-pack build --target web` and copied to `public/wasm/`. We load
// the wasm-pack glue (`sparq_wasm.js`) at RUNTIME with a dynamic import that webpack
// is told to ignore, so the ~1.2 MB wasm never enters the page bundle and the glue's
// `new URL('./sparq_wasm_bg.wasm', import.meta.url)` resolution is bypassed — we pass
// the wasm bytes' URL explicitly, prefixed with the Pages basePath. This is genuinely
// live: every query runs the real Rust engine compiled to wasm, in your tab.

export interface WasmStore {
  readonly size: number;
  query(sparql: string): string; // SPARQL 1.1 JSON results document
  queryQuads(sparql: string): string; // CONSTRUCT/DESCRIBE -> N-Triples
}

interface WasmModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  Store: {
    load(text: string, format: string): WasmStore;
    loadDataset(text: string, format: string): WasmStore;
  };
}

/**
 * [OPUS-4.8] Serialises a store's whole default graph to N-Triples (a valid Turtle
 * subset) via a CONSTRUCT. Used to merge two graphs format-agnostically when the user
 * chooses "add to current": concatenate both stores' N-Triples and re-load as ntriples.
 */
export function storeToNTriples(store: WasmStore): string {
  return store.queryQuads("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }");
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
