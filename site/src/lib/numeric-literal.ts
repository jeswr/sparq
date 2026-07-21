// [GPT-5] Shared SPARQL-result numeric detection. Keep result renderers on one XSD datatype set.
import type { SparqlTerm } from "@/lib/sparq-wasm";

import { XSD } from "./curie";

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
  ].map((datatype) => XSD + datatype),
);

/** Whether a bound SPARQL result term is a finite XSD numeric literal. */
export function isNumericLiteral(term: SparqlTerm | undefined): boolean {
  return (
    term?.type === "literal" &&
    typeof term.datatype === "string" &&
    NUMERIC_XSD.has(term.datatype) &&
    Number.isFinite(Number(term.value))
  );
}
