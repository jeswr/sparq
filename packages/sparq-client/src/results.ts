// [OPUS-4.8] sq-x0kp — framework-agnostic SPARQL-results formatting for the /try REPL
// results panel (GUI MVP item 2 in research/gui-design.md §"Phase 1").
//
// WHY HERE. The GUI design record's load-bearing property is that the PURE part of the GUI
// "transfers" to any host (the Next.js site today, the proposed Tauri 2 webview later). The
// React result panel is host-shaped, but the LOGIC that turns a SPARQL 1.1 JSON results
// document into a typed table, a CSV/TSV export, or a pretty-printed raw view is pure data
// shaping with no DOM and no framework — so it lives in this shared `@sparq/client` package
// (alongside `formatTerm` / `streamQueryRows` / the SPARQL-JSON shapes) and both hosts draw
// the same cells. This file is the single source of truth for "how a result renders".
//
// SCOPE. It shapes the three answer shapes the lean wasm bundle returns through its query
// surface: a SELECT result (`Store.query` -> SPARQL-JSON with `head.vars` + bindings), an
// ASK result (the same document carrying a top-level `boolean`), and — for completeness so
// the panel has one place to ask "what shape is this?" — a discriminator over both. The
// CONSTRUCT/DESCRIBE (N-Triples) and EXPLAIN (plan text) shapes are already plain strings the
// engine returns, so they need no shaping here; the panel renders them verbatim.
//
// No performance claim is made anywhere here.

import type { SparqlResults, SparqlTerm, SparqlBinding } from "./index.js";
import { formatTerm } from "./index.js";

// ---------------------------------------------------------------------------
// Reading a SPARQL-JSON results document.
// ---------------------------------------------------------------------------

/** The variable names a SELECT result projects (its `head.vars`), in order. */
export function resultVars(results: SparqlResults): string[] {
  return results.head?.vars ?? [];
}

/** The solution rows of a SELECT result (each a var→term binding map), in order. */
export function resultRows(results: SparqlResults): SparqlBinding[] {
  return results.results?.bindings ?? [];
}

/**
 * Whether a parsed SPARQL-JSON document is an ASK answer — it carries a top-level
 * `boolean`. (A SELECT document has `head.vars` + `results.bindings` and no `boolean`.)
 */
export function isAskResult(results: SparqlResults): boolean {
  return typeof results.boolean === "boolean";
}

/** The boolean of an ASK answer, or `null` if the document is not an ASK result. */
export function askValue(results: SparqlResults): boolean | null {
  return typeof results.boolean === "boolean" ? results.boolean : null;
}

// ---------------------------------------------------------------------------
// The typed TABLE view (columns from the head vars, cells via formatTerm).
// ---------------------------------------------------------------------------

/** A SELECT result shaped as a table: the column headers plus one string row per solution. */
export interface ResultTable {
  /** The column headers — the projection's variable names, in `head.vars` order. */
  vars: string[];
  /**
   * One row per solution. Each row has exactly `vars.length` cells (columns in `vars`
   * order); an unbound projection variable in a solution renders as the empty string.
   */
  rows: string[][];
}

/**
 * [OPUS-4.8] sq-x0kp — shape a SELECT result set into a {@link ResultTable}: the columns are
 * the projected variables (in `head.vars` order) and each cell is the term rendered by the
 * shared {@link formatTerm} (so a URI shows as `<…>`, a typed literal keeps its `^^xsd:…`
 * suffix, a lang literal its `@lang`, and an unbound variable is the empty string). This is
 * the single extraction both the site table and the Tauri webview table draw from.
 *
 * Passing an ASK document yields an empty table (`vars: []`, `rows: []`): a table is not the
 * right view for a boolean, and the caller should branch on {@link isAskResult} first.
 */
export function extractTable(results: SparqlResults): ResultTable {
  if (isAskResult(results)) return { vars: [], rows: [] };
  const vars = resultVars(results);
  const rows = resultRows(results).map((binding) =>
    vars.map((v) => formatTerm(binding[v])),
  );
  return { vars, rows };
}

// ---------------------------------------------------------------------------
// CSV / TSV export (MVP: "table + raw SPARQL-JSON + N-Triples, plus CSV/TSV export").
// ---------------------------------------------------------------------------

/**
 * The RAW lexical value of a term for a data export, WITHOUT the display decoration
 * {@link formatTerm} adds (no `<>` around a URI, no surrounding quotes / `^^`/`@` on a
 * literal). A CSV/TSV consumer wants the value itself (`http://ex/a`, `42`, `Alice`), and the
 * cell-level quoting/escaping is handled by {@link csvCell} / {@link tsvCell}. An unbound
 * projection variable exports as an empty cell.
 */
function exportValue(t: SparqlTerm | undefined): string {
  return t ? t.value : "";
}

/**
 * One CSV cell, quoted per RFC 4180: a field containing a comma, double-quote, CR or LF is
 * wrapped in double-quotes with each embedded double-quote doubled. Other fields pass through
 * verbatim. (CRLF is the RFC line terminator; {@link resultsToCsv} joins rows with it.)
 */
export function csvCell(value: string): string {
  return /[",\r\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
}

/**
 * One TSV cell: tab/CR/LF are NOT representable inside a TSV field, so they are replaced with
 * a single space (the conventional lossless-enough TSV escape — TSV has no quoting). This
 * keeps every solution on exactly one line with one column per tab.
 */
export function tsvCell(value: string): string {
  return value.replace(/[\t\r\n]+/g, " ");
}

/**
 * [OPUS-4.8] sq-x0kp — export a SELECT result as CSV (RFC 4180): a header row of the
 * projected variable names, then one row per solution of the terms' RAW lexical values
 * ({@link exportValue}), each cell quoted by {@link csvCell}, rows joined with CRLF. An ASK
 * document (no projection) exports as the empty string. A result with columns but no
 * solutions exports just the header row. This is the "Download CSV" payload.
 */
export function resultsToCsv(results: SparqlResults): string {
  if (isAskResult(results)) return "";
  const vars = resultVars(results);
  const header = vars.map(csvCell).join(",");
  const body = resultRows(results).map((binding) =>
    vars.map((v) => csvCell(exportValue(binding[v]))).join(","),
  );
  return [header, ...body].join("\r\n");
}

/**
 * [OPUS-4.8] sq-x0kp — export a SELECT result as TSV: a tab-separated header row of the
 * projected variable names, then one tab-separated row per solution of the terms' RAW
 * lexical values ({@link exportValue}) sanitised by {@link tsvCell}, rows joined with LF. An
 * ASK document exports as the empty string; a result with columns but no solutions exports
 * just the header row. This is the "Download TSV" payload.
 */
export function resultsToTsv(results: SparqlResults): string {
  if (isAskResult(results)) return "";
  const vars = resultVars(results);
  const header = vars.map(tsvCell).join("\t");
  const body = resultRows(results).map((binding) =>
    vars.map((v) => tsvCell(exportValue(binding[v]))).join("\t"),
  );
  return [header, ...body].join("\n");
}

// ---------------------------------------------------------------------------
// The RAW SPARQL-JSON view (a pretty-printed copy of the engine's document).
// ---------------------------------------------------------------------------

/**
 * [OPUS-4.8] sq-x0kp — pretty-print a parsed SPARQL-JSON results document for the raw-JSON
 * toggle: a stable 2-space-indented re-serialisation of the EXACT document the engine
 * returned (`head` + `results`/`boolean`). Re-serialising the parsed object (rather than
 * threading the original string) guarantees the panel shows valid, canonically-indented JSON
 * regardless of how the engine spaced its output.
 */
export function formatSparqlJson(results: SparqlResults): string {
  return JSON.stringify(results, null, 2);
}
