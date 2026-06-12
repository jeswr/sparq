/**
 * SPARQL-side helpers: converting SPARQL 1.1 JSON result terms to RDF/JS
 * terms, serialising RDF/JS terms to SPARQL/N-Triples syntax, and a small
 * token scanner to detect the query form (SELECT/ASK/…) without a parser.
 */
import type * as RDF from '@rdfjs/types';
import { BlankNode, DataFactory, Literal, NamedNode } from './terms.js';

// --- SPARQL 1.1 JSON results → RDF/JS terms -----------------------------------------------------

export interface SparqlJsonTerm {
  type: 'uri' | 'literal' | 'typed-literal' | 'bnode';
  value: string;
  'xml:lang'?: string;
  datatype?: string;
}

export interface SparqlJsonResults {
  head: { vars?: string[] };
  results?: { bindings: Record<string, SparqlJsonTerm>[] };
  boolean?: boolean;
}

export function termFromSparqlJson(term: SparqlJsonTerm): RDF.Term {
  switch (term.type) {
    case 'uri':
      return new NamedNode(term.value);
    case 'bnode':
      return new BlankNode(term.value);
    case 'literal':
    case 'typed-literal': // legacy alias emitted by some endpoints
      if (term['xml:lang'] !== undefined) return new Literal(term.value, term['xml:lang']);
      if (term.datatype !== undefined) return new Literal(term.value, new NamedNode(term.datatype));
      return new Literal(term.value);
    default:
      throw new Error(`unsupported SPARQL JSON term type: ${(term as { type: string }).type}`);
  }
}

// --- RDF/JS terms → N-Triples / SPARQL syntax ---------------------------------------------------

function escapeLiteral(value: string): string {
  return value
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r');
}

const XSD_STRING = 'http://www.w3.org/2001/XMLSchema#string';

/** Serialises a term to its N-Triples (and SPARQL constant) form. */
export function termToNT(term: RDF.Term): string {
  switch (term.termType) {
    case 'NamedNode':
      return `<${term.value}>`;
    case 'BlankNode':
      return `_:${term.value}`;
    case 'Literal': {
      const quoted = `"${escapeLiteral(term.value)}"`;
      if (term.language !== '') {
        const dir = term.direction != null && term.direction !== '' ? `--${term.direction}` : '';
        return `${quoted}@${term.language}${dir}`;
      }
      if (term.datatype.value !== XSD_STRING) return `${quoted}^^<${term.datatype.value}>`;
      return quoted;
    }
    case 'DefaultGraph':
      return '';
    default:
      throw new Error(`cannot serialise term type ${term.termType} to N-Triples`);
  }
}

/**
 * Serialises quads to N-Quads (a superset of N-Triples; default-graph quads
 * produce plain N-Triples lines). The sparq engine folds named graphs into the
 * default graph on load.
 */
export function quadsToNQuads(quads: Iterable<RDF.Quad>): string {
  let out = '';
  for (const quad of quads) {
    if (quad.subject.termType === 'Quad' || quad.object.termType === 'Quad') {
      throw new Error('RDF-star quoted triples are not supported');
    }
    const graph = quad.graph.termType === 'DefaultGraph' ? '' : ` ${termToNT(quad.graph)}`;
    out += `${termToNT(quad.subject)} ${termToNT(quad.predicate)} ${termToNT(quad.object)}${graph} .\n`;
  }
  return out;
}

// --- query-form detection ------------------------------------------------------------------------

export type QueryForm = 'SELECT' | 'ASK' | 'CONSTRUCT' | 'DESCRIBE';
const FORMS = new Set(['SELECT', 'ASK', 'CONSTRUCT', 'DESCRIBE']);

/**
 * Finds the query form keyword (the first bare SELECT/ASK/CONSTRUCT/DESCRIBE
 * token after the PREFIX/BASE prologue), skipping comments, IRIs and string
 * literals so e.g. `PREFIX ask: <http://ex/ASK#>` cannot confuse it.
 * Returns the form and its character offset, or `undefined` if none is found.
 */
export function detectQueryForm(sparql: string): { form: QueryForm; index: number; length: number } | undefined {
  let i = 0;
  const n = sparql.length;
  while (i < n) {
    const c = sparql[i]!;
    if (c === '#') {
      while (i < n && sparql[i] !== '\n') i++;
    } else if (c === '<') {
      while (i < n && sparql[i] !== '>') i++;
      i++;
    } else if (c === '"' || c === "'") {
      const quote = c;
      const long = sparql.startsWith(quote.repeat(3), i);
      const end = long ? quote.repeat(3) : quote;
      i += end.length;
      while (i < n) {
        if (sparql[i] === '\\') i += 2;
        else if (sparql.startsWith(end, i)) {
          i += end.length;
          break;
        } else i++;
      }
    } else if (/[A-Za-z]/.test(c)) {
      let j = i;
      while (j < n && /[A-Za-z]/.test(sparql[j]!)) j++;
      const word = sparql.slice(i, j).toUpperCase();
      // Not a keyword if part of a prefixed name (`ask:x`, `ex:ASK`) or a
      // variable (`?ask`): inspect the adjacent characters.
      const before = i > 0 ? sparql[i - 1]! : ' ';
      const after = j < n ? sparql[j]! : ' ';
      const isBareToken = !/[:?$@\w.-]/.test(before) && !/[:\w-]/.test(after);
      if (isBareToken && FORMS.has(word)) return { form: word as QueryForm, index: i, length: j - i };
      i = j;
    } else {
      i++;
    }
  }
  return undefined;
}

/**
 * Rewrites an ASK query to an equivalent `SELECT *` query. `ASK WHERE {…}`
 * and `ASK {…}` are both valid SPARQL after substituting `SELECT *`.
 *
 * Legacy helper: the engine now evaluates ASK natively (with first-solution
 * early exit), so `SparqStore` no longer rewrites — kept for callers that
 * target SELECT-only endpoints.
 */
export function askToSelect(sparql: string): string {
  const form = detectQueryForm(sparql);
  if (!form || form.form !== 'ASK') throw new Error('not an ASK query');
  return `${sparql.slice(0, form.index)}SELECT *${sparql.slice(form.index + form.length)}`;
}

export { DataFactory };
