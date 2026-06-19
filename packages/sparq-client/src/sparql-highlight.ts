// [OPUS-4.8] sq-n5aw — a framework-agnostic SPARQL tokenizer for syntax highlighting.
//
// This is the highlighting CORE the query-editor uplift (`research/gui-design.md` §2, MVP
// item 1) is built on. It is deliberately DEPENDENCY-FREE and FRAMEWORK-AGNOSTIC: a pure
// `tokenizeSparql(text)` over a string, returning a flat, gap-free token list a renderer can
// wrap in styled spans. No DOM, no React — so the SAME tokenizer serves the Next.js site and
// the Tauri 2 webview (and is unit-testable in `node:test` with no browser). It is a
// pragmatic LEXER for highlighting, not a SPARQL parser: it never rejects input and always
// reproduces the source exactly when the token texts are concatenated (a load-bearing
// property the overlay editor relies on — the highlight layer must align with the textarea
// glyph-for-glyph).
//
// Scope: the SPARQL 1.1/1.2 surface the lean wasm bundle answers — keywords, prefixed names
// (`foaf:name`), variables (`?x` / `$x`), IRIs (`<…>`), string literals (single/long/quoted),
// numbers, comments (`# …`), the `a` keyword, booleans, and punctuation. Unknown text falls
// through as plain. No performance claim is made here (this repo's work box is non-canonical).

/** The lexical class of a highlighted SPARQL token. */
export type SparqlTokenType =
  | "keyword" // SELECT, WHERE, FILTER, PREFIX, …  (and the `a` shorthand, booleans)
  | "variable" // ?x  $x
  | "iri" // <http://…>
  | "prefixed" // foaf:name   ex:alice   rdf:type
  | "string" // "…"  '…'  """…"""  '''…'''  (with lang/datatype handled as adjacent tokens)
  | "number" // 42   3.14   1.0e9
  | "comment" // # … to end of line
  | "punctuation" // { } ( ) [ ] ; , . ^^ etc.
  | "plain"; // whitespace and anything unclassified

/** One token: its source `text` and lexical `type`. Concatenating every `text` in order
 * reproduces the input string exactly. */
export interface SparqlToken {
  text: string;
  type: SparqlTokenType;
}

// SPARQL 1.1 keywords (case-insensitive in SPARQL). Kept as an uppercase set; the lexer
// upsuper-cases the candidate before lookup. Includes the SPARQL 1.2 additions the engine
// accepts and the aggregate / modifier / update keywords. `a` (the rdf:type shorthand) and
// the boolean literals are highlighted as keywords too (handled in the lexer, not this set).
const KEYWORDS = new Set<string>([
  "BASE", "PREFIX", "SELECT", "DISTINCT", "REDUCED", "CONSTRUCT", "DESCRIBE",
  "ASK", "FROM", "NAMED", "WHERE", "ORDER", "BY", "ASC", "DESC", "LIMIT",
  "OFFSET", "VALUES", "GROUP", "HAVING", "AS", "BIND", "SERVICE", "OPTIONAL",
  "UNION", "MINUS", "GRAPH", "FILTER", "EXISTS", "NOT", "IN", "STR", "LANG",
  "LANGMATCHES", "DATATYPE", "BOUND", "IRI", "URI", "BNODE", "RAND", "ABS",
  "CEIL", "FLOOR", "ROUND", "CONCAT", "STRLEN", "UCASE", "LCASE",
  "ENCODE_FOR_URI", "CONTAINS", "STRSTARTS", "STRENDS", "STRBEFORE", "STRAFTER",
  "YEAR", "MONTH", "DAY", "HOURS", "MINUTES", "SECONDS", "TIMEZONE", "TZ", "NOW",
  "UUID", "STRUUID", "MD5", "SHA1", "SHA256", "SHA384", "SHA512", "COALESCE",
  "IF", "STRLANG", "STRDT", "SAMETERM", "ISIRI", "ISURI", "ISBLANK",
  "ISLITERAL", "ISNUMERIC", "REGEX", "SUBSTR", "REPLACE", "COUNT", "SUM", "MIN",
  "MAX", "AVG", "SAMPLE", "GROUP_CONCAT", "SEPARATOR",
  // SPARQL Update
  "LOAD", "CLEAR", "DROP", "CREATE", "ADD", "MOVE", "COPY", "INSERT", "DELETE",
  "DATA", "WITH", "USING", "SILENT", "DEFAULT", "ALL", "INTO", "TO",
  // EXPLAIN introspection forms the engine exposes
  "EXPLAIN", "ANALYZE",
]);

const BOOLEANS = new Set<string>(["true", "false"]);

// Character-class helpers (ASCII fast paths; PN_CHARS_BASE/U is approximated for highlighting).
const isWs = (c: string): boolean => c === " " || c === "\t" || c === "\n" || c === "\r" || c === "\f" || c === "\v";
const isDigit = (c: string): boolean => c >= "0" && c <= "9";
const isNameStart = (c: string): boolean =>
  (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || c === "_" || c.charCodeAt(0) > 127;
const isNameChar = (c: string): boolean => isNameStart(c) || isDigit(c) || c === "-" || c === "." || c === "·";

/**
 * Tokenize a SPARQL string for HIGHLIGHTING. Pure: no side effects, no DOM. The returned
 * tokens are gap-free and ordered — `tokens.map(t => t.text).join("")` equals the input
 * exactly (so an overlay renderer aligns with the source). This is a forgiving lexer, not a
 * validator: malformed input (an unterminated string/IRI, a stray character) is still
 * tokenized to end-of-input rather than throwing.
 */
export function tokenizeSparql(input: string): SparqlToken[] {
  const tokens: SparqlToken[] = [];
  const n = input.length;
  let i = 0;
  const push = (text: string, type: SparqlTokenType): void => {
    if (text.length > 0) tokens.push({ text, type });
  };

  while (i < n) {
    const c = input[i];

    // Whitespace run.
    if (isWs(c)) {
      let j = i + 1;
      while (j < n && isWs(input[j])) j++;
      push(input.slice(i, j), "plain");
      i = j;
      continue;
    }

    // Comment: `#` to end of line.
    if (c === "#") {
      let j = i + 1;
      while (j < n && input[j] !== "\n") j++;
      push(input.slice(i, j), "comment");
      i = j;
      continue;
    }

    // IRI ref: `<` … `>` (no whitespace/`<` inside per the grammar; stop forgivingly at EOL).
    if (c === "<") {
      let j = i + 1;
      while (j < n && input[j] !== ">" && input[j] !== "\n" && input[j] !== "<") j++;
      if (j < n && input[j] === ">") {
        push(input.slice(i, j + 1), "iri");
        i = j + 1;
        continue;
      }
      // Not a closed IRI (e.g. a `<` operator) — emit one punctuation char and continue.
      push(c, "punctuation");
      i += 1;
      continue;
    }

    // Variable: `?x` or `$x`.
    if (c === "?" || c === "$") {
      let j = i + 1;
      while (j < n && isNameChar(input[j])) j++;
      // A lone `?` (e.g. a property-path `?`) with no name is punctuation.
      if (j === i + 1) {
        push(c, "punctuation");
        i += 1;
      } else {
        push(input.slice(i, j), "variable");
        i = j;
      }
      continue;
    }

    // String literal: '...', "...", '''...''', """...""" (with \-escapes).
    if (c === '"' || c === "'") {
      const j = scanString(input, i, c);
      push(input.slice(i, j), "string");
      i = j;
      continue;
    }

    // Number: a leading digit, or a `+`/`-`/`.` immediately followed by a digit.
    if (
      isDigit(c) ||
      ((c === "+" || c === "-" || c === ".") && i + 1 < n && isDigit(input[i + 1]))
    ) {
      const j = scanNumber(input, i);
      // A bare `.` (statement terminator) is punctuation, not a number — scanNumber returns
      // i unchanged in that case (it only advances when it actually consumed digits).
      if (j > i) {
        push(input.slice(i, j), "number");
        i = j;
        continue;
      }
    }

    // Name-like run: a keyword, a prefixed name (`foaf:name`), a bare prefix (`foaf:`), the
    // `a` shorthand, a boolean, or `PREFIX:local`. Also handles a leading `:local` (empty
    // prefix) and a leading `_:bnode`.
    if (isNameStart(c) || c === ":") {
      const j = scanNameLike(input, i);
      const text = input.slice(i, j);
      push(text, classifyNameLike(text));
      i = j;
      continue;
    }

    // `_:bnode` blank-node label.
    if (c === "_" && i + 1 < n && input[i + 1] === ":") {
      let j = i + 2;
      while (j < n && isNameChar(input[j])) j++;
      push(input.slice(i, j), "prefixed");
      i = j;
      continue;
    }

    // Multi-char punctuation we want as one token: `^^`, `||`, `&&`, `<=`, `>=`, `!=`.
    const two = input.slice(i, i + 2);
    if (two === "^^" || two === "||" || two === "&&" || two === "<=" || two === ">=" || two === "!=") {
      push(two, "punctuation");
      i += 2;
      continue;
    }

    // Single-char punctuation / operators.
    push(c, "punctuation");
    i += 1;
  }

  return tokens;
}

/** Scan a quoted string starting at `start` (quote char `q`), returning the index PAST the
 * closing quote (or end of input for an unterminated literal). Handles `'''`/`"""` long
 * forms and `\`-escapes. */
function scanString(input: string, start: number, q: string): number {
  const n = input.length;
  const isLong = input[start + 1] === q && input[start + 2] === q;
  if (isLong) {
    let j = start + 3;
    while (j < n) {
      if (input[j] === "\\") {
        j += 2;
        continue;
      }
      if (input[j] === q && input[j + 1] === q && input[j + 2] === q) return j + 3;
      j++;
    }
    return n;
  }
  let j = start + 1;
  while (j < n) {
    if (input[j] === "\\") {
      j += 2;
      continue;
    }
    if (input[j] === q) return j + 1;
    if (input[j] === "\n") return j; // single-line string ends at a newline (forgiving)
    j++;
  }
  return n;
}

/** Scan a numeric literal at `start`; returns the index past it, or `start` if no digit was
 * actually consumed (so a bare `.`/`+`/`-` operator is NOT misread as a number). */
function scanNumber(input: string, start: number): number {
  const n = input.length;
  let j = start;
  if (input[j] === "+" || input[j] === "-") j++;
  let sawDigit = false;
  while (j < n && isDigit(input[j])) {
    j++;
    sawDigit = true;
  }
  if (j < n && input[j] === ".") {
    j++;
    while (j < n && isDigit(input[j])) {
      j++;
      sawDigit = true;
    }
  }
  // Exponent.
  if (sawDigit && j < n && (input[j] === "e" || input[j] === "E")) {
    let k = j + 1;
    if (k < n && (input[k] === "+" || input[k] === "-")) k++;
    if (k < n && isDigit(input[k])) {
      k++;
      while (k < n && isDigit(input[k])) k++;
      j = k;
    }
  }
  return sawDigit ? j : start;
}

/** Scan a name-like run (keyword / prefixed name / bare prefix / `a` / boolean), including an
 * optional `:` and a local part. Returns the index past it. */
function scanNameLike(input: string, start: number): number {
  const n = input.length;
  let j = start;
  // Optional leading prefix label (or empty for `:local`).
  while (j < n && isNameChar(input[j])) j++;
  // A `:` makes it a prefixed name — consume the colon and the local part.
  if (j < n && input[j] === ":") {
    j++;
    while (j < n && (isNameChar(input[j]) || input[j] === ":" || input[j] === "%")) j++;
  }
  // Trim a trailing `.` that is really a statement terminator (not part of a local name end)
  // — but only when the run is NOT a prefixed name (a dot inside `ex:a.b` is legal). A bare
  // keyword/`a` followed by `.` must not swallow the dot.
  if (input.slice(start, j).indexOf(":") === -1) {
    while (j > start && input[j - 1] === ".") j--;
  }
  return Math.max(j, start + 1);
}

/** Classify a name-like run already extracted by {@link scanNameLike}. */
function classifyNameLike(text: string): SparqlTokenType {
  if (text.indexOf(":") !== -1) return "prefixed"; // foaf:name, ex:alice, :local, rdf:type
  if (text === "a") return "keyword"; // the rdf:type shorthand
  const upper = text.toUpperCase();
  if (KEYWORDS.has(upper)) return "keyword";
  if (BOOLEANS.has(text.toLowerCase())) return "keyword";
  return "plain";
}
