// [OPUS-4.8] sq-vw3ax.10 — the shared, framework-free CURIE abbreviation used by the live
// SPARQL result renderers (the home hero runner's result table `repl-result-cells.tsx` and the
// node-link Graph view `repl-graph-view.tsx`). Display-only: it abbreviates an IRI to a `prefix:local`
// CURIE when it sits under a well-known vocabulary, and is a no-op otherwise. Raw exports (CSV/TSV/
// JSON) keep the full IRI — this only shortens what a human reads in a cell or a graph node label.
//
// Extracted here so the table and the graph agree on exactly one prefix set (no drift): both import
// `curie` from this module. Pure and React-free so it can be unit-tested under `npm run test:unit`.

export const XSD = "http://www.w3.org/2001/XMLSchema#";

/** Well-known prefixes for CURIE display — the vocabularies the built-in datasets use. */
export const DISPLAY_PREFIXES: [string, string][] = [
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
export function curie(iri: string): string {
  for (const [prefix, ns] of DISPLAY_PREFIXES) {
    if (iri.startsWith(ns)) return prefix + iri.slice(ns.length);
  }
  return iri;
}
