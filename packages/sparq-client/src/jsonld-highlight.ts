// [OPUS-4.8] sq-ixc3.1 — a framework-agnostic JSON-LD tokenizer for syntax highlighting.
//
// The third sibling of `sparql-highlight.ts` / `turtle-highlight.ts`, completing the set of
// formats the data-formats picker accepts: JSON-LD (`.jsonld` / `application/ld+json`). Unlike
// Turtle, JSON-LD is JSON, so this is a small forgiving JSON LEXER — keys, string values,
// numbers, the JSON literals (`true` / `false` / `null`) and structural punctuation
// (`{ } [ ] : ,`), with the JSON-LD KEYWORDS (`@context`, `@id`, `@type`, `@graph`, `@value`,
// `@language`, …) coloured distinctly so the linked-data structure stands out.
//
// Like its siblings it is deliberately DEPENDENCY-FREE and FRAMEWORK-AGNOSTIC (no DOM, no
// React, no JSON.parse) so the SAME tokenizer serves the Next.js site and the Tauri 2 webview,
// and is unit-testable under `node:test`. It is a pragmatic lexer for highlighting, not a JSON
// parser/validator: it never rejects input and always reproduces the source EXACTLY when the
// token texts are concatenated (the load-bearing property the overlay editor relies on — the
// highlight layer must align with the textarea glyph-for-glyph).
//
// The token TYPE set is the shared {@link SparqlTokenType} so the existing `.sq-tok-*` CSS
// classes style it — no new palette, theme-following highlighting for free. The mapping:
//   * `keyword`     — JSON-LD `@…` keywords (`@context`/`@id`/`@type`/…) AND `true`/`false`/`null`
//   * `prefixed`    — an OBJECT KEY (a string in key position, i.e. immediately before a `:`)
//   * `string`      — a string VALUE
//   * `number`      — a numeric value
//   * `punctuation` — `{ } [ ] : ,`
//   * `plain`       — whitespace and anything unclassified
// `iri` / `variable` / `comment` are unused by JSON (kept in the shared type for one renderer).
// No performance claim is made here (this repo's work box is non-canonical).

import type { SparqlToken, SparqlTokenType } from "./sparql-highlight.js";

/** The lexical class of a highlighted JSON-LD token. Aliased to {@link SparqlTokenType} so one
 * renderer + one CSS palette styles SPARQL, Turtle and JSON-LD alike. */
export type JsonLdTokenType = SparqlTokenType;

/** One token: its source `text` and lexical `type`. Concatenating every `text` in order
 * reproduces the input string exactly. Aliased to {@link SparqlToken}. */
export type JsonLdToken = SparqlToken;

// The JSON-LD keyword set (JSON-LD 1.1 §1.7 "Syntax Tokens and Keywords"). These are the `@…`
// terms that carry structural meaning regardless of the active context. A `"@something"` not in
// this set is treated as a key/value string like any other (forgiving — we never reject input).
const JSONLD_KEYWORDS = new Set<string>([
  "@base", "@container", "@context", "@direction", "@graph", "@id", "@import",
  "@included", "@index", "@json", "@language", "@list", "@nest", "@none",
  "@prefix", "@propagate", "@protected", "@reverse", "@set", "@type", "@value",
  "@version", "@vocab",
]);

// The bare JSON literals (`true` / `false` / `null`), coloured as keywords (the same class the
// SPARQL/Turtle lexers give their boolean literals).
const JSON_LITERALS = new Set<string>(["true", "false", "null"]);

const isWs = (c: string): boolean =>
  c === " " || c === "\t" || c === "\n" || c === "\r" || c === "\f" || c === "\v";
const isDigit = (c: string): boolean => c >= "0" && c <= "9";

/**
 * Tokenize a JSON-LD (JSON) document for HIGHLIGHTING. Pure: no side effects, no DOM, no
 * `JSON.parse`. The returned tokens are gap-free and ordered — `tokens.map(t => t.text)
 * .join("")` equals the input exactly (so an overlay renderer aligns with the source). This is
 * a forgiving lexer, not a validator: malformed input (an unterminated string, a stray
 * character) is still tokenized to end-of-input rather than throwing.
 *
 * Key vs value distinction: a string is classified `prefixed` (a KEY) when the next significant
 * character after it is `:`, else `string` (a VALUE). A JSON-LD `@…` keyword (in either
 * position) is `keyword`. This single-pass lookahead is a heuristic, not a grammar — but a
 * highlighter only needs it to read right, and it does for well-formed JSON-LD.
 */
export function tokenizeJsonLd(input: string): JsonLdToken[] {
  const tokens: JsonLdToken[] = [];
  const n = input.length;
  let i = 0;
  const push = (text: string, type: JsonLdTokenType): void => {
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

    // String: `"…"` with `\`-escapes (JSON has no single-quoted strings). After scanning, peek
    // past whitespace for a `:` — if found, this string is an object KEY (so colour it
    // `prefixed`, distinct from a value); a JSON-LD `@…` keyword is `keyword` either way.
    if (c === '"') {
      const j = scanJsonString(input, i);
      const text = input.slice(i, j);
      push(text, classifyString(text, input, j));
      i = j;
      continue;
    }

    // Number: an optional `-`, integer part, optional fraction, optional exponent. JSON does
    // not allow a leading `+` or a leading `.`, so only `-`/digit can start a number here.
    if (isDigit(c) || (c === "-" && i + 1 < n && isDigit(input[i + 1]))) {
      const j = scanJsonNumber(input, i);
      if (j > i) {
        push(input.slice(i, j), "number");
        i = j;
        continue;
      }
    }

    // Bare word: a JSON literal (`true`/`false`/`null`). Any other unquoted word is invalid
    // JSON but tokenized forgivingly as `plain` so the lexer stays lossless.
    if ((c >= "a" && c <= "z") || (c >= "A" && c <= "Z")) {
      let j = i + 1;
      while (j < n && ((input[j] >= "a" && input[j] <= "z") || (input[j] >= "A" && input[j] <= "Z"))) {
        j++;
      }
      const word = input.slice(i, j);
      push(word, JSON_LITERALS.has(word) ? "keyword" : "plain");
      i = j;
      continue;
    }

    // Structural punctuation: `{ } [ ] : ,` (and anything else, forgivingly, one char at a time).
    push(c, "punctuation");
    i += 1;
  }

  return tokens;
}

/** Scan a JSON string starting at the opening `"` (`start`), returning the index PAST the
 * closing `"` (or end of input for an unterminated literal). Handles `\`-escapes; a JSON string
 * cannot span a raw newline, so an unterminated string ends at EOL (forgiving). */
function scanJsonString(input: string, start: number): number {
  const n = input.length;
  let j = start + 1;
  while (j < n) {
    const c = input[j];
    if (c === "\\") {
      j += 2; // skip the escaped char (`\"`, `\\`, `\n`, `\uXXXX`, …)
      continue;
    }
    if (c === '"') return j + 1;
    if (c === "\n") return j; // unterminated — stop at the newline (forgiving)
    j++;
  }
  return n;
}

/** Classify a just-scanned `"…"` token. A JSON-LD `@…` keyword is `keyword`; otherwise a string
 * in KEY position (the next significant char is `:`) is `prefixed`, a string VALUE is `string`.
 * `at` is the index just past the closing quote. */
function classifyString(text: string, input: string, at: number): JsonLdTokenType {
  // The string's content without the surrounding quotes (the closing quote may be absent for an
  // unterminated literal, hence the defensive slice).
  const content = text.endsWith('"') && text.length >= 2 ? text.slice(1, -1) : text.slice(1);
  if (content.startsWith("@") && JSONLD_KEYWORDS.has(content)) return "keyword";
  return nextSignificantIsColon(input, at) ? "prefixed" : "string";
}

/** True if the next non-whitespace character at or after `at` is `:` (so the preceding string
 * is an object key). */
function nextSignificantIsColon(input: string, at: number): boolean {
  const n = input.length;
  let j = at;
  while (j < n && isWs(input[j])) j++;
  return j < n && input[j] === ":";
}

/** Scan a JSON number at `start` (a leading digit, or `-` then a digit); returns the index past
 * it, or `start` if no digit was actually consumed. Follows the JSON grammar: optional `-`,
 * an integer part, an optional `.fraction`, an optional `e`/`E` exponent. */
function scanJsonNumber(input: string, start: number): number {
  const n = input.length;
  let j = start;
  if (input[j] === "-") j++;
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
