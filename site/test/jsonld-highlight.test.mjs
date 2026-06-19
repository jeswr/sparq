// [OPUS-4.8] sq-ixc3.1 — unit tests for the framework-agnostic JSON-LD highlighting tokenizer
// (packages/sparq-client/src/jsonld-highlight.ts), the third sibling of tokenizeSparql /
// tokenizeTurtle. The two load-bearing invariants mirror the SPARQL/Turtle tests: (1) the
// tokenizer is LOSSLESS — concatenating every token's text reproduces the input exactly (the
// overlay editor aligns the highlight layer glyph-for-glyph, so any drift mis-renders), and
// (2) each construct gets the right lexical class — JSON-LD `@…` keywords + true/false/null as
// `keyword`, object KEYS as `prefixed`, string VALUES as `string`, numbers as `number`, and
// `{ } [ ] : ,` as `punctuation`. Imported via relative path (the node:test TS loader has no
// `@sparq/client` alias). Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  tokenizeJsonLd,
} from "../../packages/sparq-client/src/jsonld-highlight.ts";

/** Concatenate token texts — must equal the input for any string (the lossless invariant). */
function roundtrip(src) {
  return tokenizeJsonLd(src)
    .map((t) => t.text)
    .join("");
}

/** The token types covering a given source, in order, dropping pure-whitespace `plain` runs. */
function typesOf(src) {
  return tokenizeJsonLd(src)
    .filter((t) => !(t.type === "plain" && t.text.trim() === ""))
    .map((t) => t.type);
}

/** First token of a given type, or undefined. */
function firstOfType(src, type) {
  return tokenizeJsonLd(src).find((t) => t.type === type)?.text;
}

test("lossless: concatenating tokens reproduces the input exactly", () => {
  const samples = [
    "",
    "   ",
    "{}",
    "[]",
    '{ "@id": "ex:alice" }',
    `{
  "@context": { "ex": "http://example.org/" },
  "@graph": [
    { "@id": "ex:alice", "@type": "foaf:Person", "foaf:name": "Alice", "age": 30 }
  ]
}`,
    '{ "n": -1.5e10, "ok": true, "no": false, "nil": null }',
    '{ "esc": "a \\"quote\\" and a \\\\ slash" }',
    '{ "unterminated": "no close', // forgiving: no closing quote
    "garbage £ ™ not json",
  ];
  for (const s of samples) {
    assert.equal(roundtrip(s), s, `roundtrip mismatch for: ${JSON.stringify(s)}`);
  }
});

test("JSON-LD @-keywords are classified as keyword (key or value position)", () => {
  // `@context` / `@id` / `@type` / `@graph` / `@value` / `@language` are JSON-LD keywords.
  assert.equal(firstOfType('{ "@context": {} }', "keyword"), '"@context"');
  assert.equal(firstOfType('{ "@id": "x" }', "keyword"), '"@id"');
  // `@type` mapped to `@id` (a keyword in VALUE position too).
  const toks = tokenizeJsonLd('{ "knows": { "@type": "@id" } }');
  const keywords = toks.filter((t) => t.type === "keyword").map((t) => t.text);
  assert.deepEqual(keywords, ['"@type"', '"@id"']);
});

test("an unknown @-token is NOT a keyword (treated as a normal string/key)", () => {
  // `@notakeyword` is not in the JSON-LD keyword set — so it is a key (prefixed), not a keyword.
  assert.equal(firstOfType('{ "@notakeyword": 1 }', "keyword"), undefined);
  assert.equal(firstOfType('{ "@notakeyword": 1 }', "prefixed"), '"@notakeyword"');
});

test("object keys are prefixed; string values are string", () => {
  const toks = tokenizeJsonLd('{ "foaf:name": "Alice" }');
  // "foaf:name" is a key (before a colon) → prefixed; "Alice" is a value → string.
  assert.equal(firstOfType('{ "foaf:name": "Alice" }', "prefixed"), '"foaf:name"');
  assert.equal(firstOfType('{ "foaf:name": "Alice" }', "string"), '"Alice"');
  // The string just before the `:` is the key; verify the value is NOT misclassified as a key.
  const valueTok = toks.find((t) => t.text === '"Alice"');
  assert.equal(valueTok.type, "string");
});

test("a string element inside an array is a value, not a key (no following colon)", () => {
  // ["a", "b"] — neither array element is followed by a colon, so both are string values.
  const types = typesOf('{ "list": ["a", "b"] }');
  // "list"(prefixed) :(punct) [(punct) "a"(string) ,(punct) "b"(string) ](punct) }(punct)
  assert.equal(firstOfType('{ "list": ["a", "b"] }', "prefixed"), '"list"');
  const strings = tokenizeJsonLd('{ "list": ["a", "b"] }')
    .filter((t) => t.type === "string")
    .map((t) => t.text);
  assert.deepEqual(strings, ['"a"', '"b"']);
  assert.ok(types.includes("string"));
});

test("numbers (int, negative, fraction, exponent) are classified as number", () => {
  assert.equal(firstOfType('{ "n": 42 }', "number"), "42");
  assert.equal(firstOfType('{ "n": -7 }', "number"), "-7");
  assert.equal(firstOfType('{ "n": 3.14 }', "number"), "3.14");
  assert.equal(firstOfType('{ "n": -1.5e10 }', "number"), "-1.5e10");
  assert.equal(firstOfType('{ "n": 6.022E23 }', "number"), "6.022E23");
});

test("true / false / null are keywords", () => {
  assert.equal(firstOfType('{ "ok": true }', "keyword"), "true");
  assert.equal(firstOfType('{ "ok": false }', "keyword"), "false");
  assert.equal(firstOfType('{ "ok": null }', "keyword"), "null");
});

test("structural punctuation { } [ ] : , each tokenizes as punctuation", () => {
  const punct = tokenizeJsonLd('{ "a": [1], "b": 2 }')
    .filter((t) => t.type === "punctuation")
    .map((t) => t.text);
  // { : [ ] , : }  — order preserved.
  assert.deepEqual(punct, ["{", ":", "[", "]", ",", ":", "}"]);
});

test("a realistic JSON-LD node yields a sensible class sequence", () => {
  const types = typesOf('{ "@id": "ex:alice", "@type": "foaf:Person" }');
  // {(punct) "@id"(keyword) :(punct) "ex:alice"(string) ,(punct) "@type"(keyword) :(punct) "foaf:Person"(string) }(punct)
  assert.equal(types[0], "punctuation"); // {
  assert.equal(types[1], "keyword"); // "@id"
  assert.equal(types[2], "punctuation"); // :
  assert.equal(types[3], "string"); // "ex:alice"
  assert.equal(types[4], "punctuation"); // ,
  assert.equal(types[5], "keyword"); // "@type"
});

test("forgiving: an unterminated string does not throw and stays lossless", () => {
  const bad = '{ "k": "unterminated';
  assert.equal(roundtrip(bad), bad);
  // It is still tokenized (as a string), not dropped.
  assert.ok(tokenizeJsonLd(bad).some((t) => t.type === "string"));
});
