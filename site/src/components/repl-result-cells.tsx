"use client";

// [OPUS-4.8] sq-vw3ax — the data-tool result rendering (a SELECT table + its inline aggregate
// mini-viz), now used by the home hero's in-fold runner (src/components/home/hero-runner.tsx)
// after the /try REPL was removed (sq-4hiqe). Faithful to the mockup
// (sparq-design-system/proposals/web-try.html): typed-value colouring that reuses the
// foundation's `--chart-*` syntax tokens (IRI / literal / number), CURIE abbreviation of IRIs,
// and a tiny bar strip derived from the LIVE result set (never a hardcoded shape).
//
// REAL DATA / HONESTY (load-bearing): every cell here renders a real `SparqlTerm` from the
// engine's result document, and the mini-viz is computed PURELY from those rows — a numeric
// aggregate column charted against a label column the engine actually returned. If the result
// has no chartable (label, number) shape, the viz simply does not render. No illustrative
// figures, no fabricated bars.

import * as React from "react";

import type { SparqlResults, SparqlTerm } from "@/lib/sparq-wasm";

const XSD = "http://www.w3.org/2001/XMLSchema#";
const NUMERIC_XSD = new Set(
  [
    "integer",
    "decimal",
    "double",
    "float",
    "long",
    "int",
    "short",
    "byte",
    "nonNegativeInteger",
    "positiveInteger",
    "unsignedInt",
    "unsignedLong",
  ].map((t) => XSD + t),
);

// Well-known prefixes for CURIE display in result cells — the same vocabularies the built-in
// datasets use. Display-only abbreviation; the raw exports keep full IRIs.
const DISPLAY_PREFIXES: [string, string][] = [
  ["ex:", "http://example.org/"],
  ["foaf:", "http://xmlns.com/foaf/0.1/"],
  ["rdf:", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"],
  ["rdfs:", "http://www.w3.org/2000/01/rdf-schema#"],
  ["xsd:", XSD],
  ["dc:", "http://purl.org/dc/elements/1.1/"],
  ["dct:", "http://purl.org/dc/terms/"],
  ["owl:", "http://www.w3.org/2002/07/owl#"],
];

/** Abbreviate an IRI to a CURIE when it sits under a well-known prefix (display only). */
function curie(iri: string): string {
  for (const [prefix, ns] of DISPLAY_PREFIXES) {
    if (iri.startsWith(ns)) return prefix + iri.slice(ns.length);
  }
  return iri;
}

function isNumericLiteral(t: SparqlTerm): boolean {
  return (
    t.type === "literal" &&
    typeof t.datatype === "string" &&
    NUMERIC_XSD.has(t.datatype) &&
    Number.isFinite(Number(t.value))
  );
}

/**
 * Render one result cell with typed-value colouring. Reuses the foundation's syntax-highlight
 * tokens (`--chart-1` IRI, `--chart-3` number, `--chart-4` string) via the shared `.sq-tok-*`
 * classes so the table reads like the editor. An unbound projection variable renders as an
 * em-dash placeholder.
 */
export function ResultCell({ term }: { term: SparqlTerm | undefined }) {
  if (!term) {
    return (
      <span className="text-muted-foreground/50" aria-label="unbound">
        —
      </span>
    );
  }
  if (term.type === "uri") {
    return (
      <span className="sq-tok-iri" title={term.value}>
        {curie(term.value)}
      </span>
    );
  }
  if (term.type === "bnode") {
    return <span className="text-muted-foreground">_:{term.value}</span>;
  }
  // Literal.
  if (isNumericLiteral(term)) {
    return <span className="sq-tok-number tabular">{term.value}</span>;
  }
  const lang = term["xml:lang"];
  const dt =
    term.datatype && term.datatype !== `${XSD}string` ? term.datatype : undefined;
  return (
    <span className="sq-tok-string">
      &quot;{term.value}&quot;
      {lang && <span className="text-muted-foreground">@{lang}</span>}
      {dt && (
        <span className="sq-tok-prefixed" title={dt}>
          ^^{curie(dt)}
        </span>
      )}
    </span>
  );
}

// ── Inline aggregate mini-viz ─────────────────────────────────────────────────────────────

export interface VizBar {
  label: string;
  value: number;
}

/**
 * [OPUS-4.8] sq-vw3ax — derive a small bar chart from the LIVE result set, or `null` when the
 * result has no chartable shape. The heuristic, applied to the engine's OWN rows:
 *   • find the first column whose every bound cell is a numeric literal — the VALUE axis;
 *   • find the first NON-numeric column — the LABEL axis;
 *   • chart up to `max` rows of (label, value).
 * This matches the mockup's "count per city" aggregate viz without ever inventing a figure: a
 * SELECT that has no numeric column (e.g. the default `?name ?age ?friend` join, where ?age is
 * the only number but there is no distinct label) still charts ?age-per-?friend honestly; a
 * result with no numbers at all charts nothing. The caller hides the strip when this is null.
 */
export function deriveViz(results: SparqlResults, max = 12): VizBar[] | null {
  const vars = results.head?.vars ?? [];
  const rows = results.results?.bindings ?? [];
  if (vars.length < 2 || rows.length === 0) return null;

  // The value axis: a column whose every bound value is a numeric literal (and at least one is).
  const valueVar = vars.find((v) => {
    let sawNumber = false;
    for (const row of rows) {
      const t = row[v];
      if (!t) continue;
      if (!isNumericLiteral(t)) return false;
      sawNumber = true;
    }
    return sawNumber;
  });
  if (!valueVar) return null;

  // The label axis: the first column that is not the value column.
  const labelVar = vars.find((v) => v !== valueVar);
  if (!labelVar) return null;

  const bars: VizBar[] = [];
  for (const row of rows) {
    const valTerm = row[valueVar];
    if (!valTerm) continue;
    const value = Number(valTerm.value);
    if (!Number.isFinite(value)) continue;
    const labelTerm = row[labelVar];
    const label = labelTerm ? labelTerm.value : "—";
    bars.push({ label: curie(label), value });
    if (bars.length >= max) break;
  }
  return bars.length > 0 ? bars : null;
}

/**
 * The inline aggregate mini-viz strip. Bars are scaled to the max value in the (real) set; the
 * exact value sits above each bar and the (abbreviated) label below. Purely presentational —
 * it charts what {@link deriveViz} extracted from the engine's result.
 */
export function ResultMiniViz({ bars }: { bars: VizBar[] }) {
  const max = Math.max(...bars.map((b) => b.value), 1);
  return (
    <div
      data-result-viz
      aria-label="Result shape preview"
      className="flex h-24 items-end gap-2.5 border-t bg-muted/15 px-4 pt-6 pb-5"
    >
      {bars.map((b, i) => {
        const pct = Math.max(8, Math.round((b.value / max) * 100));
        return (
          <div
            key={`${b.label}-${i}`}
            className="relative min-w-0 flex-1 rounded-t-[5px] bg-[var(--hero-grad)] opacity-90"
            style={{ height: `${pct}%` }}
            title={`${b.label}: ${b.value}`}
          >
            <b className="absolute -top-4 inset-x-0 text-center font-mono text-[10px] font-normal text-foreground">
              {b.value}
            </b>
            <span className="absolute -bottom-4 inset-x-0 truncate text-center font-mono text-[10px] text-muted-foreground">
              {b.label}
            </span>
          </div>
        );
      })}
    </div>
  );
}
