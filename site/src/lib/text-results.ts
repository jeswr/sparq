// [OPUS-4.8] sq-xoxu — pure helpers for the live /surface/full-text playground: shape the
// SPARQL-1.1-JSON result set the wasm `TextSearch.query` binding returns into framework-free
// rows for rendering. The BM25 ranking + `text:` rewrite SEMANTICS themselves are proven by
// crates/sparq-text(/ -wasm)'s tests; here we only test the JS render of what the binding
// returns. Kept separate from the React component so node --test can exercise it with no DOM.

import type { SparqlResults, SparqlTerm } from "./sparq-wasm";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

/**
 * [OPUS-4.8] sq-xoxu — render a SPARQL-JSON term for display, with a compact datatype/lang
 * suffix. A self-contained copy of the REPL's `formatTerm` so this pure helper carries NO
 * cross-module runtime import (the unit-test ts-loader does not rewrite extensionless
 * specifiers, and a value import would not resolve). An undefined term — an unbound
 * projection variable in a row — renders as the empty string.
 */
export function formatTerm(t: SparqlTerm | undefined): string {
  if (!t) return "";
  if (t.type === "uri") return `<${t.value}>`;
  if (t.type === "bnode") return `_:${t.value}`;
  if (t["xml:lang"]) return `"${t.value}"@${t["xml:lang"]}`;
  if (t.datatype && t.datatype !== XSD_STRING) {
    const short = t.datatype.replace("http://www.w3.org/2001/XMLSchema#", "xsd:");
    return `"${t.value}"^^${short}`;
  }
  return `"${t.value}"`;
}

/** The variable names the result set projects (its SELECT projection), in order. */
export function resultVars(results: SparqlResults): string[] {
  return results.head?.vars ?? [];
}

/** The solution rows of the result set (each a var→term map). */
export function resultRows(
  results: SparqlResults,
): Record<string, SparqlTerm>[] {
  return results.results?.bindings ?? [];
}

/**
 * [OPUS-4.8] sq-xoxu — render a result set as plain string cells (one row per solution,
 * columns in {@link resultVars} order), reusing {@link formatTerm} so the datatype/lang
 * suffixes match the rest of the site. An unbound variable in a row renders as the empty
 * string. This is what the playground draws into its ranked-hits table.
 */
export function resultCells(results: SparqlResults): {
  vars: string[];
  rows: string[][];
} {
  const vars = resultVars(results);
  const rows = resultRows(results).map((binding) =>
    vars.map((v) => formatTerm(binding[v])),
  );
  return { vars, rows };
}

/**
 * [OPUS-4.8] sq-xoxu — the raw numeric value of a bound variable in a row, or `null` when
 * it is unbound or not a finite number. Used to read a `text:score` cell for the relevance
 * bar without re-formatting it (the score binds as an xsd:double in the SPARQL JSON).
 */
export function numericValue(
  binding: Record<string, SparqlTerm> | undefined,
  variable: string,
): number | null {
  const t = binding?.[variable];
  if (!t || t.type !== "literal") return null;
  const n = Number(t.value);
  return Number.isFinite(n) ? n : null;
}
