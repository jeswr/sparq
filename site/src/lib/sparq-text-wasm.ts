// [OPUS-4.8] sq-xoxu — live loader for the tier-b "W-text" wasm bundle that drives
// /surface/full-text. This is a SEPARATE, lazy-loaded bundle from the lean sparq-wasm
// triplestore bundle (crates/sparq-text-wasm → public/wasm/text): the owned BM25 inverted
// index + the `text:` magic predicates of sparq-text over an RDF document. The lean
// landing page never pays for it; the full-text page loads it on first interaction,
// exactly as the inference page loads the W-reason bundle and the streaming page the W-rsp
// bundle.
//
// As with src/lib/reason-wasm.ts the wasm-pack glue is imported at RUNTIME with a
// webpackIgnore dynamic import, so the wasm never enters the page bundle and we pass the
// wasm bytes' URL explicitly (prefixed with the Pages basePath) rather than let the glue's
// `new URL('…_bg.wasm', import.meta.url)` resolution run.

/**
 * The JS-facing `TextSearch` surface the W-text bundle exports (mirrors
 * crates/sparq-text-wasm). Every method is a STATELESS one-shot: it parses the supplied
 * document, builds a positions-enabled BM25 index over it, runs the request, and returns a
 * string — so the JS side never holds a long-lived index handle. Document `format` is one
 * of `"turtle"` | `"ntriples"` | `"nquads"` | `"trig"`.
 */
export interface WasmTextSearch {
  /**
   * Parse `data`, build the index, rewrite the `text:` magic predicates in `sparql` into
   * plain SPARQL, evaluate, and return a canonical SPARQL 1.1 JSON results document. A
   * query with no `text:` patterns runs unchanged.
   */
  query(data: string, format: string, sparql: string): string;
  /**
   * Report the index footprint over `data` as a small JSON object
   * `{"docs","tokens","heapBytes","hasPositions"}` — the indexed-literal count, the
   * distinct-token count, the index's estimated in-memory size in bytes, and whether
   * positions are recorded.
   */
  indexStats(data: string, format: string): string;
}

interface TextModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  TextSearch: WasmTextSearch;
}

let modulePromise: Promise<TextModule> | null = null;

function basePath(): string {
  // next.config.ts sets basePath '/sparq'; mirror it for the runtime asset URLs.
  return process.env.NEXT_PUBLIC_BASE_PATH ?? "/sparq";
}

/**
 * [OPUS-4.8] sq-xoxu — loads + initialises the W-text bundle once; subsequent calls reuse
 * it. The fetch + compile + instantiate is the expensive cold start. {@link prewarmText}
 * kicks this off eagerly on route mount so the first "Search" pays no cold start. A failed
 * load resets the cache so a later call retries.
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
      modulePromise = null; // allow retry on transient failure
    });
  }
  const mod = await modulePromise;
  return mod.TextSearch;
}

/**
 * [OPUS-4.8] sq-xoxu — eagerly pre-warm the W-text bundle (fetch + compile + instantiate)
 * without blocking render. Safe to call on route mount and repeatedly; it shares the single
 * {@link loadTextSearch} promise, so the cold start happens at most once.
 */
export function prewarmText(): Promise<unknown> {
  return loadTextSearch();
}

/**
 * [OPUS-4.8] sq-xoxu — the index footprint the full-text page shows: `docs` indexed string
 * literals, `tokens` distinct terms across them, `heapBytes` estimated in-memory size, and
 * whether `hasPositions` (the bundle always builds positions on, so `text:phrase` /
 * `text:near` are answerable). Mirrors `TextSearch.indexStats`'s JSON.
 */
export interface TextIndexStats {
  docs: number;
  tokens: number;
  heapBytes: number;
  hasPositions: boolean;
}

/**
 * [OPUS-4.8] sq-xoxu — run a `text:`-predicate SPARQL query over an RDF document, entirely
 * in your tab via the W-text bundle. `format` is one of `"turtle"` | `"ntriples"` |
 * `"nquads"` | `"trig"`. Returns the parsed SPARQL 1.1 JSON results. Throws if the bundle is
 * missing the `TextSearch` export (wrong artifact), or the document / query fails to parse,
 * or the query misuses a `text:` predicate.
 */
export async function sparqTextQuery(
  data: string,
  sparql: string,
  format = "turtle",
): Promise<import("./sparq-wasm").SparqlResults> {
  const TextSearch = await loadTextSearch();
  if (typeof TextSearch?.query !== "function") {
    throw new Error(
      "This wasm bundle does not export the full-text `TextSearch` handle. " +
        "Build crates/sparq-text-wasm with `wasm-pack build --target web`.",
    );
  }
  return JSON.parse(
    TextSearch.query(data, format, sparql),
  ) as import("./sparq-wasm").SparqlResults;
}

/**
 * [OPUS-4.8] sq-xoxu — report the BM25 index footprint over an RDF document via the W-text
 * bundle. Same `format` set as {@link sparqTextQuery}. Throws if the bundle is the wrong
 * artifact or the document fails to parse.
 */
export async function sparqTextIndexStats(
  data: string,
  format = "turtle",
): Promise<TextIndexStats> {
  const TextSearch = await loadTextSearch();
  if (typeof TextSearch?.indexStats !== "function") {
    throw new Error(
      "This wasm bundle does not export the full-text `TextSearch` handle. " +
        "Build crates/sparq-text-wasm with `wasm-pack build --target web`.",
    );
  }
  return JSON.parse(TextSearch.indexStats(data, format)) as TextIndexStats;
}
