// [OPUS-4.8] sq-17nw — pure dataset helpers for the live SPARQL REPL.
//
// The REPL used to load EVERY uploaded document with `Store.load(text, format)`,
// which folds the named graphs of N-Quads / TriG into the default graph — so a
// `GRAPH <iri> { … }` query against an uploaded dataset returned nothing. These
// helpers let the REPL instead:
//   • pick the named-graph-preserving `Store.loadDataset` for the quad formats
//     (`isDatasetFormat`), and
//   • serialise a *whole* loaded dataset (default graph + every named graph) back
//     to N-Quads (`ALL_QUADS_QUERY` + `rowsToNQuads`) so the format-agnostic
//     "add to current" merge re-loads through `loadDataset` without losing graphs.
//
// They are framework-free and side-effect-free (no React, no DOM, no wasm) so they
// unit-test directly under `node --test`; the engine-level named-graph behaviour is
// covered by the Rust tests + js/test/store.test.mjs.

import type { SparqlTerm } from "./sparq-wasm";

// [OPUS-4.8] sq-dvyi: JSON-LD can carry named graphs (`@graph` with an outer `@id`),
// so it joins nquads/trig as a dataset format routed through `loadDataset`.
/** Formats whose payload can carry named graphs (and so need `loadDataset`). */
const DATASET_FORMATS = new Set(["nquads", "trig", "jsonld"]);

/**
 * True for the quad-bearing RDF formats (`nquads` / `trig` / `jsonld`). Triple-only
 * formats (`turtle` / `ntriples`) have no named graphs, so the cheaper `Store.load` is
 * both correct and slightly faster for them.
 */
export function isDatasetFormat(format: string): boolean {
  return DATASET_FORMATS.has(format);
}

// [OPUS-4.8] sq-dvyi: the RDF formats the wasm engine's `load`/`loadDataset` accept,
// keyed by file extension. Lives here (not in the React component) so it is framework-
// free and unit-testable. JSON-LD is parsed engine-side (oxjsonld) in the lean wasm
// bundle. The component renders these as the upload/URL format choices.
export const FORMAT_OPTIONS: { value: string; label: string; exts: string[] }[] = [
  { value: "turtle", label: "Turtle (.ttl)", exts: ["ttl", "turtle"] },
  { value: "ntriples", label: "N-Triples (.nt)", exts: ["nt", "ntriples"] },
  { value: "nquads", label: "N-Quads (.nq)", exts: ["nq", "nquads"] },
  { value: "trig", label: "TriG (.trig)", exts: ["trig"] },
  { value: "jsonld", label: "JSON-LD (.jsonld)", exts: ["jsonld", "json-ld"] },
];

/** Best-effort engine format guess from a filename/URL; falls back to Turtle. */
export function guessFormat(name: string): string {
  const ext = name.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "";
  const hit = FORMAT_OPTIONS.find((f) => f.exts.includes(ext));
  return hit?.value ?? "turtle";
}

// Map a response `Content-Type` to an engine format, ignoring any `; charset=…`
// parameter. A served media type is more reliable than the URL extension for the
// auto-detect URL load (e.g. a JSON-LD endpoint with no `.jsonld` suffix), so the URL
// path prefers it when present. Returns `undefined` for an unknown/absent type so the
// caller can fall back to the extension guess.
const CONTENT_TYPE_FORMATS: Record<string, string> = {
  "text/turtle": "turtle",
  "application/n-triples": "ntriples",
  "application/n-quads": "nquads",
  "application/trig": "trig",
  "application/ld+json": "jsonld",
};

/** Engine format for a response `Content-Type`, or `undefined` if unrecognised. */
export function formatFromContentType(
  contentType: string | null,
): string | undefined {
  if (!contentType) return undefined;
  const mime = contentType.split(";")[0].trim().toLowerCase();
  return CONTENT_TYPE_FORMATS[mime];
}

/**
 * The graph-pattern that matches the WHOLE dataset: the default graph
 * (`{ ?s ?p ?o }`) UNION every named graph (`GRAPH ?g { ?s ?p ?o }`). Named-graph
 * solutions additionally bind `?g`; default-graph solutions leave it unbound.
 * Folding-only `{ ?s ?p ?o }` would miss the named graphs entirely — the bug this
 * fixes. Reused by the row enumeration and the COUNT-based total-size query.
 */
export const ALL_QUADS_BODY = "{ ?s ?p ?o } UNION { GRAPH ?g { ?s ?p ?o } }";

/** A SELECT that enumerates the whole dataset (see {@link ALL_QUADS_BODY}). */
export const ALL_QUADS_QUERY = `SELECT ?s ?p ?o ?g WHERE { ${ALL_QUADS_BODY} }`;

/** Escapes a literal's lexical form for an N-Quads quoted string (matches the
 * engine's own serialiser: backslash, double-quote, LF, CR — a raw tab is legal). */
function escapeLiteral(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r");
}

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

/** Serialises one SPARQL-JSON term to its N-Triples / N-Quads token. */
function termToNT(term: SparqlTerm): string {
  switch (term.type) {
    case "uri":
      return `<${term.value}>`;
    case "bnode":
      return `_:${term.value}`;
    case "literal": {
      const quoted = `"${escapeLiteral(term.value)}"`;
      const lang = term["xml:lang"];
      if (lang) return `${quoted}@${lang}`;
      if (term.datatype && term.datatype !== XSD_STRING) {
        return `${quoted}^^<${term.datatype}>`;
      }
      return quoted;
    }
  }
}

/** One solution row from {@link ALL_QUADS_QUERY}: bound `s`/`p`/`o`, optional `g`. */
type QuadRow = {
  s?: SparqlTerm;
  p?: SparqlTerm;
  o?: SparqlTerm;
  g?: SparqlTerm;
};

/**
 * Serialises {@link ALL_QUADS_QUERY} solution rows to an N-Quads document: a triple
 * line (`s p o .`) when `?g` is unbound (default graph), a quad line
 * (`s p o g .`) when `?g` is bound (a named graph). Re-loading the result with
 * `Store.loadDataset(_, "nquads")` reconstructs the dataset with its named graphs
 * intact. Rows missing any of `s`/`p`/`o` are skipped defensively.
 */
export function rowsToNQuads(rows: QuadRow[]): string {
  const lines: string[] = [];
  for (const row of rows) {
    if (!row.s || !row.p || !row.o) continue;
    const spo = `${termToNT(row.s)} ${termToNT(row.p)} ${termToNT(row.o)}`;
    lines.push(row.g ? `${spo} ${termToNT(row.g)} .` : `${spo} .`);
  }
  return lines.join("\n");
}
