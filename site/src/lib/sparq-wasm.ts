// [OPUS-4.8] sq-2e93 — the site's wasm-engine entry point.
//
// The framework-agnostic core — the ONE `WasmStore` type declaration, the wasm loaders
// (`loadSparq`/`prewarmSparq`), the SPARQL-JSON + SHACL shapes, the `match()`/cursor query
// helpers, `sparqShaclValidate` and `formatTerm` — now lives in the shared
// `@sparq/client` package (`packages/sparq-client`). This removes the hand-redeclared
// `WasmStore` drift that `research/gui-design.md` §0/§4 flagged: there is now a single
// source for that surface, which the site re-exports here so every existing
// `@/lib/sparq-wasm` import keeps resolving unchanged, and which the (proposed) Tauri 2 GUI
// consumes directly rather than becoming a third hand-copy.
//
// What stays HERE is only the site-specific glue that is coupled to the site's
// dataset-format knowledge (`./repl-dataset`): the named-graph-preserving load entry point
// and the all-quads (de)serialisation used by the "add to current" merge. Those take an
// opinion on which RDF formats carry named graphs, which is a site concern, not part of the
// engine's framework-agnostic surface.

import {
  type SparqlResults,
  type WasmModule,
  type WasmStore,
} from "@sparq/client";

import {
  ALL_QUADS_BODY,
  ALL_QUADS_QUERY,
  isDatasetFormat,
  rowsToNQuads,
} from "./repl-dataset";

// Re-export the shared framework-agnostic client surface so existing `@/lib/sparq-wasm`
// importers across the site keep working with no call-site change.
export {
  type CursorBatch,
  type LoadSparqOptions,
  type MatchTerm,
  type ShaclReport,
  type ShaclResult,
  type SparqlBinding,
  type SparqlResults,
  type SparqlTerm,
  type WasmModule,
  type WasmSolutionCursor,
  type WasmStore,
  type WasmStoreCtor,
  countQuads,
  formatTerm,
  loadSparq,
  matchQuads,
  prewarmSparq,
  sparqShaclValidate,
  streamQueryRows,
} from "@sparq/client";

/**
 * [OPUS-4.8] sq-17nw — parses RDF into a store, PRESERVING named graphs for the
 * quad-bearing formats (N-Quads / TriG / JSON-LD) by routing them through `loadDataset`
 * instead of `load` (which folds every named graph into the default graph). Triple-only
 * formats (`turtle` / `ntriples`) carry no named graphs, so the cheaper `load` is used.
 * This is the single load entry point the REPL uses for every source (default, picker,
 * upload, URL, and the re-load step of a merge). The dataset-format decision lives in the
 * site (`./repl-dataset`), so this stays out of the framework-agnostic `@sparq/client`.
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
 * `Store.loadDataset(_, "nquads")` so the named graphs survive the round-trip.
 */
export function storeToNQuads(store: WasmStore): string {
  const json = store.query(ALL_QUADS_QUERY);
  const parsed = JSON.parse(json) as SparqlResults;
  return rowsToNQuads(parsed.results.bindings);
}

/**
 * [OPUS-4.8] sq-17nw — the TOTAL number of triples in a store across the default graph
 * and all named graphs. The wasm `store.size` getter counts the DEFAULT graph only, so it
 * under-reports a dataset; this counts the whole thing via {@link ALL_QUADS_QUERY} for an
 * accurate "N triples" badge. Falls back to `store.size` if the count query fails.
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
