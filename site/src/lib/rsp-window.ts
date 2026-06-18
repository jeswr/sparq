// [OPUS-4.8] sq-11zy — pure helpers for the live /surface/streaming-rsp playground:
// parse the closed-window JSON array the wasm `Rsp.push`/`Rsp.flush` bindings return,
// and render each window's SPARQL-1.1-JSON result table into framework-free rows. The
// wasm windowing semantics themselves (boundaries, lateness, R2S diffs) are proven by
// crates/sparq-rsp's tests; here we only test the JS parse/render of what it returns.
// Kept separate from the React component so node --test can exercise it with no DOM.

import type { SparqlResults, SparqlTerm } from "./sparq-wasm";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

/**
 * [OPUS-4.8] sq-11zy — render a SPARQL-JSON term for display, with a compact
 * datatype/lang suffix. A self-contained copy of the REPL's `formatTerm` so this pure
 * helper carries NO cross-module runtime import (the unit-test ts-loader does not rewrite
 * extensionless specifiers, and a value import would not resolve). An undefined term —
 * an unbound projection variable in a row — renders as the empty string.
 */
function formatTerm(t: SparqlTerm | undefined): string {
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

/**
 * [OPUS-4.8] sq-11zy — one CLOSED window the engine fired: the half-open `[start, end)`
 * logical-time bounds plus the R2S-filtered SELECT table as a self-contained SPARQL 1.1
 * JSON document. Mirrors the `{"start","end","results"}` shape `windows_json` emits in
 * crates/sparq-rsp-wasm.
 */
export interface ClosedWindow {
  start: number;
  end: number;
  results: SparqlResults;
}

/**
 * [OPUS-4.8] sq-11zy — parse the JSON string a single `push`/`flush` returns into the
 * array of windows it closed (oldest first, possibly empty). Throws a clear error if the
 * payload is not the expected `{start,end,results}[]` shape, so a malformed binding return
 * surfaces loudly rather than rendering as an empty stream.
 */
export function parseClosedWindows(json: string): ClosedWindow[] {
  const parsed: unknown = JSON.parse(json);
  if (!Array.isArray(parsed)) {
    throw new Error("expected a JSON array of closed windows");
  }
  return parsed.map((w, i) => {
    if (
      typeof w !== "object" ||
      w === null ||
      typeof (w as Record<string, unknown>).start !== "number" ||
      typeof (w as Record<string, unknown>).end !== "number" ||
      typeof (w as Record<string, unknown>).results !== "object"
    ) {
      throw new Error(`closed window ${i} is not {start,end,results}`);
    }
    const rec = w as { start: number; end: number; results: SparqlResults };
    return { start: rec.start, end: rec.end, results: rec.results };
  });
}

/** The variable names a window's result table projects (its SELECT projection). */
export function windowVars(window: ClosedWindow): string[] {
  return window.results.head?.vars ?? [];
}

/** The solution rows of a window's result table (each a var→term map). */
export function windowRows(
  window: ClosedWindow,
): Record<string, SparqlTerm>[] {
  return window.results.results?.bindings ?? [];
}

/**
 * [OPUS-4.8] sq-11zy — render a window's result table as plain string cells (one row per
 * solution, columns in `windowVars` order), reusing the REPL's {@link formatTerm} so the
 * datatype/lang suffixes match the rest of the site. An unbound variable in a row renders
 * as the empty string. This is what the playground draws into its per-window table.
 */
export function windowCells(window: ClosedWindow): {
  vars: string[];
  rows: string[][];
} {
  const vars = windowVars(window);
  const rows = windowRows(window).map((binding) =>
    vars.map((v) => formatTerm(binding[v])),
  );
  return { vars, rows };
}

/** A compact `[start, end)` label for a window header. */
export function windowLabel(window: ClosedWindow): string {
  return `[${window.start}, ${window.end})`;
}

/**
 * [OPUS-4.8] sq-11zy — a one-line human summary of a window: its bounds and its row count
 * ("empty" when the watermark jumped a gap and the window fired with no solutions —
 * which `sparq-rsp` reports so DSTREAM can observe results disappear).
 */
export function windowSummary(window: ClosedWindow): string {
  const n = windowRows(window).length;
  if (n === 0) return `${windowLabel(window)} — empty window`;
  return `${windowLabel(window)} — ${n} row${n === 1 ? "" : "s"}`;
}
