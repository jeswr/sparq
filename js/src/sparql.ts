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

/**
 * Incremental parser for a *chunked* SPARQL 1.1 JSON results document (the
 * engine's chunk sequence concatenates to one valid document): feed chunks
 * with `push()`, get back the solution rows completed by each chunk. Holds at
 * most one chunk plus a partial row in memory — never the whole document.
 * Rows are extracted with a string-aware brace scanner, so it stays correct
 * even if a future engine splits a chunk mid-row.
 */
export class SparqlJsonRowsParser {
  #buf = '';
  #inBindings = false;
  #done = false;

  /** Set when the document is the ASK boolean form (it then has no rows). */
  boolean: boolean | undefined;

  /** Feeds the next chunk; returns the rows it completed. */
  push(chunk: string): Record<string, SparqlJsonTerm>[] {
    if (this.#done) return [];
    this.#buf += chunk;
    if (!this.#inBindings) {
      const start = this.#buf.indexOf('"bindings":[');
      if (start < 0) {
        // The ASK boolean form has no bindings array at all.
        const bool = /"boolean"\s*:\s*(true|false)/.exec(this.#buf);
        if (bool) {
          this.boolean = bool[1] === 'true';
          this.#done = true;
          this.#buf = '';
        }
        return []; // head not complete yet — keep buffering
      }
      this.#buf = this.#buf.slice(start + '"bindings":['.length);
      this.#inBindings = true;
    }
    return this.#scanRows();
  }

  /** Extracts the complete top-level `{…}` row objects currently buffered. */
  #scanRows(): Record<string, SparqlJsonTerm>[] {
    const rows: Record<string, SparqlJsonTerm>[] = [];
    const buf = this.#buf;
    let i = 0;
    while (i < buf.length) {
      const c = buf[i]!;
      if (c === ',' || c === ' ' || c === '\n' || c === '\t' || c === '\r') {
        i++;
      } else if (c === ']') {
        this.#done = true; // end of the bindings array; trailing "}}"' ignored
        this.#buf = '';
        return rows;
      } else if (c === '{') {
        // Scan to the matching close brace, JSON-string-aware.
        let depth = 0;
        let inString = false;
        let j = i;
        for (; j < buf.length; j++) {
          const ch = buf[j]!;
          if (inString) {
            if (ch === '\\') j++;
            else if (ch === '"') inString = false;
          } else if (ch === '"') inString = true;
          else if (ch === '{') depth++;
          else if (ch === '}' && --depth === 0) break;
        }
        if (j >= buf.length) break; // row incomplete — wait for the next chunk
        rows.push(JSON.parse(buf.slice(i, j + 1)) as Record<string, SparqlJsonTerm>);
        i = j + 1;
      } else {
        throw new Error(`malformed SPARQL JSON results: unexpected ${JSON.stringify(c)} in bindings array`);
      }
    }
    this.#buf = buf.slice(i);
    return rows;
  }
}

// --- RDF/JS terms → N-Triples / SPARQL syntax ---------------------------------------------------

/**
 * Escapes a literal lexical form for the inside of a `"…"` N-Triples /
 * SPARQL `STRING_LITERAL_QUOTE`. Escapes `\`, `"`, and the whole C0 control
 * range plus DEL — the SPARQL grammar forbids a raw `#x22`/`#x5C`/`#xA`/`#xD`
 * inside a single-quoted string, and a raw control byte (e.g. TAB or NUL) is
 * exactly what an attacker would use to slip past a naive `"`-only escaper.
 * `\n`/`\r`/`\t`/`\b`/`\f` use their short forms; every other control char
 * becomes a `\uXXXX` escape.
 *
 * This is the SPARQL-injection guard for literal *values*: the output cannot
 * close its own quote or emit a raw newline, so a hostile value (e.g. an
 * ACL-derived label) stays confined to the literal token when the result is
 * re-parsed by sparq's own SPARQL lexer.
 */
function escapeLiteral(value: string): string {
  // " \  and C0 controls (#x00–#x1F) + DEL (#x7F).
  // eslint-disable-next-line no-control-regex
  return value.replace(/["\\\x00-\x1f\x7f]/g, (c) => {
    switch (c) {
      case '\\':
        return '\\\\';
      case '"':
        return '\\"';
      case '\n':
        return '\\n';
      case '\r':
        return '\\r';
      case '\t':
        return '\\t';
      case '\b':
        return '\\b';
      case '\f':
        return '\\f';
      default:
        return `\\u${c.charCodeAt(0).toString(16).toUpperCase().padStart(4, '0')}`;
    }
  });
}

/**
 * Percent-encodes the characters the SPARQL/Turtle `IRIREF` production forbids
 * inside `<…>` — `< > " { } | ^` `` ` `` `\` and every codepoint in `#x00–#x20`
 * (controls and space). Without this a hostile IRI value — most importantly one
 * carrying a `>`, e.g. an ACL pointer IRI taken from untrusted input — could
 * close its own `<…>` bracket and inject arbitrary SPARQL.
 *
 * This is the SPARQL-injection guard for IRI *values*; it matches the illegal
 * set QLever's lexer rejects, so a value that round-trips here parses to the
 * same single IRI term in sparq's parser. Percent-encoding is IRI-preserving:
 * an endpoint dereferences the encoded form to the same resource.
 */
function escapeIri(value: string): string {
  // IRIREF illegal set: < > " { } | ^ ` \ and #x00–#x20 (controls + space).
  // eslint-disable-next-line no-control-regex
  return value.replace(/[<>"{}|^`\\\x00-\x20]/g, (c) => {
    const code = c.charCodeAt(0);
    return `%${code.toString(16).toUpperCase().padStart(2, '0')}`;
  });
}

const XSD_STRING = 'http://www.w3.org/2001/XMLSchema#string';

/** Serialises a term to its N-Triples (and SPARQL constant) form. */
export function termToNT(term: RDF.Term): string {
  switch (term.termType) {
    case 'NamedNode':
      return `<${escapeIri(term.value)}>`;
    case 'BlankNode':
      return `_:${term.value}`;
    case 'Literal': {
      const quoted = `"${escapeLiteral(term.value)}"`;
      if (term.language !== '') {
        const dir = term.direction != null && term.direction !== '' ? `--${term.direction}` : '';
        return `${quoted}@${term.language}${dir}`;
      }
      if (term.datatype.value !== XSD_STRING) return `${quoted}^^<${escapeIri(term.datatype.value)}>`;
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
