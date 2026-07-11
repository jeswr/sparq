// [FABLE-5] sq-ixc3.14 — pure helpers for federated SERVICE query execution in the Query tool.
//
// The in-tab WASM engine fundamentally cannot evaluate a SPARQL 1.1 `SERVICE` clause (the
// browser has no blocking cross-origin HTTP — CORS), so the Query tool routes SERVICE-bearing
// SELECT/ASK runs to the desktop shell's native engine over Tauri IPC (`query_service`,
// gui/src-tauri/src/federation.rs), which evaluates them under the engine's STRICT fail-closed
// egress allowlist (the per-workspace federation setting). On the hosted web build the same
// detection produces an HONEST degradation message instead — per the tools.ts tier taxonomy
// this half of the Query tool is native-only there (the `walkthrough` tier's framing), never a
// silent wrong answer or a hang.
//
// Everything here is a PURE function (no React, no IPC) so the routing decision, the refusal
// classification, and the native-result parse are unit-testable in the node test runner.

import {
  formatSparqlJson,
  isAskResult,
  askValue,
  type SparqlResults,
} from "@sparq/client";

/**
 * The stable marker substring every engine SERVICE egress-refusal error carries — byte-for-byte
 * `sparq_engine::SERVICE_EGRESS_REFUSED_MARKER` (crates/sparq-engine-service/src/service.rs),
 * the same contract sparq-server uses to classify a refusal as policy (403) rather than fault.
 */
export const SERVICE_EGRESS_REFUSED_MARKER = "SERVICE egress refused";

/**
 * The honest web-build degradation message (the tools.ts taxonomy's "native-only" framing):
 * the browser cannot dial cross-origin SPARQL endpoints, and pretending otherwise would hang
 * or silently drop the SERVICE pattern. Shown instead of running the query.
 */
export const WEB_FEDERATION_MESSAGE =
  "Federated SERVICE queries are native-only: the browser build cannot dial remote SPARQL " +
  "endpoints (cross-origin HTTP is blocked by CORS, and the in-tab WASM engine ships no " +
  "federation client). Run this query in the sparq desktop app, where it executes on the " +
  "native engine under the workspace's federation allowlist.";

/**
 * Strip SPARQL string literals (single/double/long), `#` comments, and IRI refs from a query,
 * so keyword detection cannot be fooled by `"SERVICE"` inside a literal, a comment, or an IRI
 * like `<http://example.org/service>`. A `<` is treated as an IRI opener only when a
 * whitespace-free IRI body up to `>` follows (so a comparison `?x < 5` is untouched).
 */
function stripLiteralsCommentsAndIris(sparql: string): string {
  let out = "";
  let i = 0;
  const n = sparql.length;
  while (i < n) {
    const c = sparql[i];
    if (c === "#") {
      // Comment: skip to end of line.
      while (i < n && sparql[i] !== "\n") i++;
      continue;
    }
    if (c === "'" || c === '"') {
      // String literal: honour long (triple-quoted) forms and backslash escapes.
      const quote = c;
      const long = sparql.startsWith(quote.repeat(3), i);
      const close = long ? quote.repeat(3) : quote;
      i += close.length;
      while (i < n) {
        if (sparql[i] === "\\") {
          i += 2;
          continue;
        }
        if (sparql.startsWith(close, i)) {
          i += close.length;
          break;
        }
        i++;
      }
      out += " ";
      continue;
    }
    if (c === "<") {
      // IRI ref: skip only a whitespace-free <...> body (IRIs cannot contain whitespace),
      // leaving comparison operators like `?x < 5` alone.
      const m = /^<[^<>"{}|^`\\\s]*>/.exec(sparql.slice(i));
      if (m) {
        i += m[0].length;
        out += " ";
        continue;
      }
    }
    out += c;
    i++;
  }
  return out;
}

/**
 * Whether a query uses the SPARQL 1.1 `SERVICE` keyword (outside strings/comments/IRIs) and
 * should therefore route to the native federated path (desktop) or degrade honestly (web).
 * A pragmatic keyword scan, not a full parse: a false positive merely routes a query to the
 * native engine (which evaluates it identically), never to a wrong answer.
 */
export function queryUsesService(sparql: string): boolean {
  return /\bSERVICE\b/i.test(stripLiteralsCommentsAndIris(sparql));
}

/**
 * Classify a native-run error: an egress REFUSAL (the engine's stable marker) becomes an
 * actionable fail-closed message pointing at the per-workspace allowlist; anything else
 * returns `null` (surface the engine's own message unchanged).
 */
export function describeServiceRefusal(message: string): string | null {
  if (!message.includes(SERVICE_EGRESS_REFUSED_MARKER)) return null;
  return (
    "SERVICE endpoint refused (fail-closed): it is not on this workspace's federation " +
    "allowlist. Add the endpoint's host (or host:port) in the Federation control to allow " +
    `it. Engine: ${message}`
  );
}

/** The parsed outcome of a native `query_service` run (SELECT or ASK results JSON). */
export type ServiceResultsOutcome =
  | {
      kind: "select";
      results: SparqlResults;
      rawJson: string;
      rowCount: number;
      totalRows: number;
      truncated: boolean;
    }
  | { kind: "ask"; value: boolean; rawJson: string }
  | { kind: "error"; message: string };

/**
 * Parse the SPARQL 1.1 JSON results document the native `query_service` command returns into
 * the Query tool's outcome shape, applying the SAME row cap the streamed in-tab path applies
 * (rows beyond `rowCap` are counted but dropped, and the outcome records `truncated`) so the
 * result panel behaves identically wherever the query ran.
 */
export function parseServiceResults(json: string, rowCap: number): ServiceResultsOutcome {
  let parsed: SparqlResults;
  try {
    parsed = JSON.parse(json) as SparqlResults;
  } catch {
    return {
      kind: "error",
      message: "The native engine returned an unparseable result document.",
    };
  }
  if (isAskResult(parsed)) {
    return { kind: "ask", value: askValue(parsed) ?? false, rawJson: formatSparqlJson(parsed) };
  }
  const bindings = parsed.results?.bindings ?? [];
  const kept = bindings.slice(0, rowCap);
  const capped: SparqlResults = {
    head: { vars: parsed.head?.vars ?? [] },
    results: { bindings: kept },
  };
  return {
    kind: "select",
    results: capped,
    rawJson: formatSparqlJson(capped),
    rowCount: kept.length,
    totalRows: bindings.length,
    truncated: bindings.length > kept.length,
  };
}
