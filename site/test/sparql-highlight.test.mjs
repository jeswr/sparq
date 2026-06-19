// [OPUS-4.8] sq-n5aw — unit tests for the framework-agnostic SPARQL highlighting tokenizer
// (packages/sparq-client/src/sparql-highlight.ts), the dependency-free core of the query-editor
// uplift. The two load-bearing invariants: (1) the tokenizer is LOSSLESS — concatenating every
// token's text reproduces the input exactly (the overlay editor aligns the highlight layer with
// the textarea glyph-for-glyph, so any drift would mis-render), and (2) each construct gets the
// right lexical class. Imported via relative path (the node:test TS loader has no `@sparq/client`
// alias). Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  tokenizeSparql,
} from "../../packages/sparq-client/src/sparql-highlight.ts";

/** Concatenate token texts — must equal the input for any string (the lossless invariant). */
function roundtrip(src) {
  return tokenizeSparql(src)
    .map((t) => t.text)
    .join("");
}

/** The token types covering a given source, in order, dropping pure-whitespace `plain` runs. */
function typesOf(src) {
  return tokenizeSparql(src)
    .filter((t) => !(t.type === "plain" && t.text.trim() === ""))
    .map((t) => t.type);
}

/** First token of a given type, or undefined. */
function firstOfType(src, type) {
  return tokenizeSparql(src).find((t) => t.type === type)?.text;
}

test("lossless: concatenating tokens reproduces the input exactly", () => {
  const samples = [
    "",
    "   ",
    "SELECT * WHERE { ?s ?p ?o }",
    `PREFIX foaf: <http://xmlns.com/foaf/0.1/>\nSELECT ?name WHERE {\n  ?s foaf:name ?name .\n} ORDER BY ?name`,
    "ASK { ?s a foaf:Person . FILTER(?age >= 25) } # trailing comment",
    `CONSTRUCT { ?a ex:friendOf ?b } WHERE { ?a foaf:knows ?b }`,
    "SELECT (COUNT(?s) AS ?n) WHERE { ?s ex:city ?city } GROUP BY ?city",
    'SELECT * { ?s ?p "a string with \\" escape" }',
    "SELECT * { ?s ?p '''long\nstring''' }",
    "INSERT DATA { ex:erin a foaf:Person ; foaf:age 28 . }",
    "SELECT * { ?x foaf:knows+ ?y } # path",
    "weird << >> ?? $$ unmatched < iri",
  ];
  for (const s of samples) {
    assert.equal(roundtrip(s), s, `roundtrip mismatch for: ${JSON.stringify(s)}`);
  }
});

test("keywords (case-insensitive) and the `a` shorthand are classified as keyword", () => {
  assert.equal(firstOfType("SELECT ?x", "keyword"), "SELECT");
  assert.equal(firstOfType("select ?x", "keyword"), "select");
  assert.equal(firstOfType("WhErE {}", "keyword"), "WhErE");
  // `a` (rdf:type shorthand) is a keyword; a longer word containing `a` is not.
  const toks = tokenizeSparql("?s a foaf:Person");
  assert.deepEqual(
    toks.filter((t) => t.text === "a").map((t) => t.type),
    ["keyword"],
  );
});

test("variables: ?x and $x, but a lone ? is punctuation", () => {
  assert.equal(firstOfType("?name", "variable"), "?name");
  assert.equal(firstOfType("$age", "variable"), "$age");
  // A property-path `?` (zero-or-one) after a name is punctuation, not a variable.
  const toks = tokenizeSparql("foaf:knows?");
  assert.equal(toks.at(-1).type, "punctuation");
  assert.equal(toks.at(-1).text, "?");
});

test("IRIs, prefixed names, strings, numbers, comments each get their class", () => {
  assert.equal(firstOfType("<http://example.org/a>", "iri"), "<http://example.org/a>");
  assert.equal(firstOfType("foaf:name", "prefixed"), "foaf:name");
  assert.equal(firstOfType(":local", "prefixed"), ":local");
  assert.equal(firstOfType('"hello"', "string"), '"hello"');
  assert.equal(firstOfType("'hi'", "string"), "'hi'");
  assert.equal(firstOfType("42", "number"), "42");
  assert.equal(firstOfType("3.14", "number"), "3.14");
  assert.equal(firstOfType("1.0e9", "number"), "1.0e9");
  assert.equal(firstOfType("# a comment\nSELECT", "comment"), "# a comment");
});

test("a statement-terminating dot is punctuation, not a number or part of the name", () => {
  // `?o .` — the dot must be its own punctuation token (not swallowed by the var or read as a number).
  const toks = tokenizeSparql("?o .");
  const dot = toks.find((t) => t.text === ".");
  assert.ok(dot, "expected a standalone '.' token");
  assert.equal(dot.type, "punctuation");
});

test("a realistic query yields a sensible class sequence", () => {
  const types = typesOf("PREFIX ex: <http://example.org/> SELECT ?x WHERE { ?x a ex:Thing }");
  // PREFIX(keyword) ex:(prefixed) <iri> SELECT(keyword) ?x(var) WHERE(keyword) {(punct)
  assert.equal(types[0], "keyword"); // PREFIX
  assert.equal(types[1], "prefixed"); // ex:
  assert.equal(types[2], "iri");
  assert.equal(types[3], "keyword"); // SELECT
  assert.equal(types[4], "variable"); // ?x
  assert.equal(types[5], "keyword"); // WHERE
  assert.equal(types[6], "punctuation"); // {
  assert.ok(types.includes("keyword")); // `a`
});

test("forgiving: an unterminated string/IRI does not throw and stays lossless", () => {
  const bad = 'SELECT * { ?s ?p "unterminated';
  assert.equal(roundtrip(bad), bad);
  const badIri = "SELECT * { ?s <http://no-close";
  assert.equal(roundtrip(badIri), badIri);
});
