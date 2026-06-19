// [OPUS-4.8] sq-8uew — a framework-agnostic Turtle/TriG/N-Triples/N-Quads tokenizer for
// syntax highlighting.
//
// The sibling of `sparql-highlight.ts`: a pure `tokenizeTurtle(text)` over a string returning
// a flat, gap-free token list a renderer wraps in styled spans. It is deliberately
// DEPENDENCY-FREE and FRAMEWORK-AGNOSTIC (no DOM, no React) so the SAME tokenizer serves the
// Next.js site results/dataset views and the Tauri 2 webview. It is a pragmatic LEXER for
// highlighting, not an RDF parser: it never rejects input and always reproduces the source
// exactly when the token texts are concatenated (the load-bearing property the overlay editor
// relies on — the highlight layer must align with the source glyph-for-glyph).
//
// Coverage — the family of RDF text serialisations the lean wasm bundle reads and writes:
//   * Turtle / TriG — `@prefix` / `@base` directives, the SPARQL-style `PREFIX` / `BASE`
//     directives (Turtle 1.1), the `a` keyword, prefixed names (`foaf:name`, `:local`),
//     IRIs (`<…>`), string literals (single / long / `^^datatype` / `@lang`), numbers,
//     booleans, blank-node labels (`_:b`), `#` comments, and the `{ } [ ] ( ) ; , .`
//     structural punctuation. TriG named-graph braces are just punctuation.
//   * N-Triples / N-Quads — a strict subset of the above (only IRIs, strings, `_:` bnodes,
//     and the `.` terminator; N-Quads adds a trailing graph IRI), highlighted by the same
//     rules.
//
// The token TYPE set is intentionally identical to `SparqlTokenType` so the existing
// `.sq-tok-*` CSS classes style both — no new palette, theme-following highlighting for free.
// No performance claim is made here (this repo's work box is non-canonical).

import type { SparqlToken, SparqlTokenType } from "./sparql-highlight.js";

/** The lexical class of a highlighted Turtle/TriG/N-Triples/N-Quads token. Aliased to
 * {@link SparqlTokenType} so one renderer + one CSS palette styles both languages. */
export type TurtleTokenType = SparqlTokenType;

/** One token: its source `text` and lexical `type`. Concatenating every `text` in order
 * reproduces the input string exactly. Aliased to {@link SparqlToken}. */
export type TurtleToken = SparqlToken;

// The directive / keyword set Turtle (and TriG) recognise. `@prefix` / `@base` are the
// classic `@`-prefixed forms; `PREFIX` / `BASE` are the case-insensitive SPARQL-style forms
// Turtle 1.1 also accepts. `a` (the rdf:type shorthand) and the boolean literals are handled
// in the lexer, not this set. `GRAPH` is not a Turtle/TriG keyword (TriG uses bare graph
// labels + braces), so it is deliberately absent.
const TURTLE_DIRECTIVES = new Set<string>(["PREFIX", "BASE"]);

const BOOLEANS = new Set<string>(["true", "false"]);

// Character-class helpers (ASCII fast paths; PN_CHARS_BASE/U is approximated for highlighting,
// matching the SPARQL lexer's policy).
const isWs = (c: string): boolean =>
  c === " " || c === "\t" || c === "\n" || c === "\r" || c === "\f" || c === "\v";
const isDigit = (c: string): boolean => c >= "0" && c <= "9";
const isNameStart = (c: string): boolean =>
  (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || c === "_" || c.charCodeAt(0) > 127;
const isNameChar = (c: string): boolean =>
  isNameStart(c) || isDigit(c) || c === "-" || c === "." || c === "·";

/**
 * Tokenize a Turtle / TriG / N-Triples / N-Quads document for HIGHLIGHTING. Pure: no side
 * effects, no DOM. The returned tokens are gap-free and ordered — `tokens.map(t => t.text)
 * .join("")` equals the input exactly (so an overlay renderer aligns with the source). This is
 * a forgiving lexer, not a validator: malformed input (an unterminated string/IRI, a stray
 * character) is still tokenized to end-of-input rather than throwing.
 */
export function tokenizeTurtle(input: string): TurtleToken[] {
  const tokens: TurtleToken[] = [];
  const n = input.length;
  let i = 0;
  const push = (text: string, type: TurtleTokenType): void => {
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

    // Comment: `#` to end of line. (`#` is only a comment outside an IRI/string, which is
    // already handled because those are consumed whole below.)
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
      // Not a closed IRI — emit one punctuation char and continue.
      push(c, "punctuation");
      i += 1;
      continue;
    }

    // String literal: '...', "...", '''...''', """...""" (with \-escapes). A following
    // `@lang` or `^^datatype` is emitted as its own adjacent token (below).
    if (c === '"' || c === "'") {
      const j = scanString(input, i, c);
      push(input.slice(i, j), "string");
      i = j;
      continue;
    }

    // `@prefix` / `@base` directive, or a `@lang` tag immediately after a string literal.
    // Both start with `@`; classify by the directive word that follows.
    if (c === "@") {
      let j = i + 1;
      while (j < n && isNameChar(input[j]) && input[j] !== ".") j++;
      const word = input.slice(i, j);
      // `@prefix` / `@base` are directives (keyword); a bare `@en` / `@en-GB` lang tag is part
      // of the literal so we colour it as a string.
      const bare = word.slice(1).toLowerCase();
      push(word, bare === "prefix" || bare === "base" ? "keyword" : "string");
      i = j;
      continue;
    }

    // `^^` datatype marker — punctuation; the datatype IRI/prefixed name follows as its own
    // token.
    if (c === "^" && input[i + 1] === "^") {
      push("^^", "punctuation");
      i += 2;
      continue;
    }

    // Number: a leading digit, or a `+`/`-`/`.` immediately followed by a digit.
    if (
      isDigit(c) ||
      ((c === "+" || c === "-" || c === ".") && i + 1 < n && isDigit(input[i + 1]))
    ) {
      const j = scanNumber(input, i);
      // A bare `.` (statement terminator) is punctuation, not a number — scanNumber returns
      // i unchanged in that case.
      if (j > i) {
        push(input.slice(i, j), "number");
        i = j;
        continue;
      }
    }

    // `_:bnode` blank-node label. Checked BEFORE the generic name-like run so the `_:` form is
    // coloured as a node (prefixed) rather than mis-split.
    if (c === "_" && input[i + 1] === ":") {
      let j = i + 2;
      while (j < n && isNameChar(input[j])) j++;
      // Trim a trailing `.` that is really a statement terminator.
      while (j > i + 2 && input[j - 1] === ".") j--;
      push(input.slice(i, j), "prefixed");
      i = j;
      continue;
    }

    // Name-like run: a directive (`PREFIX`/`BASE`), a prefixed name (`foaf:name`, `:local`),
    // the `a` shorthand, or a boolean.
    if (isNameStart(c) || c === ":") {
      const j = scanNameLike(input, i);
      const text = input.slice(i, j);
      push(text, classifyNameLike(text));
      i = j;
      continue;
    }

    // Single-char structural punctuation: `{ } [ ] ( ) ; , .`
    push(c, "punctuation");
    i += 1;
  }

  return tokens;
}

/** Scan a quoted string starting at `start` (quote char `q`), returning the index PAST the
 * closing quote (or end of input for an unterminated literal). Handles `'''`/`"""` long
 * forms and `\`-escapes. Identical policy to the SPARQL lexer. */
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
 * actually consumed (so a bare `.`/`+`/`-` is NOT misread as a number). */
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

/** Scan a name-like run (directive / prefixed name / `a` / boolean), including an optional `:`
 * and a local part. Returns the index past it. */
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
  // Trim a trailing `.` that is really a statement terminator (not part of a local-name end)
  // — but only when the run is NOT a prefixed name (a dot inside `ex:a.b` is legal).
  if (input.slice(start, j).indexOf(":") === -1) {
    while (j > start && input[j - 1] === ".") j--;
  }
  return Math.max(j, start + 1);
}

/** Classify a name-like run already extracted by {@link scanNameLike}. */
function classifyNameLike(text: string): TurtleTokenType {
  if (text.indexOf(":") !== -1) return "prefixed"; // foaf:name, ex:alice, :local, rdf:type
  if (text === "a") return "keyword"; // the rdf:type shorthand
  const upper = text.toUpperCase();
  if (TURTLE_DIRECTIVES.has(upper)) return "keyword"; // PREFIX / BASE (case-insensitive)
  if (BOOLEANS.has(text.toLowerCase())) return "keyword";
  return "plain";
}
