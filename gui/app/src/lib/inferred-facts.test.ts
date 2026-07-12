// [FABLE-5] sq-ixc3.20 — unit tests for the canonical inferred-fact identity logic
// (node test runner). The load-bearing property: the SAME triple keyed from a SPARQL-JSON
// binding (decoded lexical form) and from a parsed N-Triples line (verbatim-escaped) yields
// the SAME key — regardless of `^^xsd:string` suppression or escape spelling.

import { strict as assert } from "node:assert";
import { test } from "node:test";

import { parseNTriples, type SparqlTerm } from "@sparq/client";

import {
  entailedKeysFromClosure,
  keyOfRdfTerm,
  keyOfSparqlTerm,
  termToNT,
  tripleKeyOfBindings,
  tripleKeyOfTerms,
  tripleKeysOfNTriples,
  unescapeNT,
} from "./inferred-facts.js";

/** Key a one-line N-Triples statement through the PARSED (RdfTerm) path. */
function keyOfLine(line: string): string {
  const { statements } = parseNTriples(line);
  assert.equal(statements.length, 1, `must parse: ${line}`);
  const st = statements[0];
  return tripleKeyOfTerms(st.s, st.p, st.o);
}

test("unescapeNT decodes the N-Triples escapes and keeps malformed ones verbatim", () => {
  assert.equal(unescapeNT('a\\"b\\nc\\\\d\\te'), 'a"b\nc\\d\te');
  assert.equal(unescapeNT("caf\\u00e9"), "café");
  assert.equal(unescapeNT("\\U0001F600"), "😀");
  assert.equal(unescapeNT("plain"), "plain");
  assert.equal(unescapeNT("bad\\uZZ"), "bad\\uZZ");
});

test("SPARQL-JSON and parsed-N-Triples keys agree for every term shape", () => {
  const iri: SparqlTerm = { type: "uri", value: "http://ex/s" };
  const bnode: SparqlTerm = { type: "bnode", value: "b0" };
  const plain: SparqlTerm = { type: "literal", value: "hi" };
  const typedString: SparqlTerm = {
    type: "literal",
    value: "hi",
    datatype: "http://www.w3.org/2001/XMLSchema#string",
  };
  const langed: SparqlTerm = { type: "literal", value: "hi", "xml:lang": "en" };
  const typed: SparqlTerm = {
    type: "literal",
    value: "5",
    datatype: "http://www.w3.org/2001/XMLSchema#integer",
  };
  const escaped: SparqlTerm = { type: "literal", value: 'a"b\nc' };

  // xml:lang / datatype normalisation: an explicit ^^xsd:string equals a plain literal.
  assert.equal(keyOfSparqlTerm(plain), keyOfSparqlTerm(typedString));

  for (const t of [iri, bnode, plain, typedString, langed, typed, escaped]) {
    const line = `${termToNT(iri)} ${termToNT(iri)} ${termToNT(t)} .`;
    const { statements } = parseNTriples(line);
    assert.equal(statements.length, 1, `round-trip parse: ${line}`);
    assert.equal(
      keyOfRdfTerm(statements[0].o),
      keyOfSparqlTerm(t),
      `parsed key must equal binding key for ${JSON.stringify(t)}`,
    );
  }
});

test("triple keys agree across the binding and parsed paths", () => {
  const s: SparqlTerm = { type: "uri", value: "http://ex/rex" };
  const p: SparqlTerm = { type: "uri", value: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" };
  const o: SparqlTerm = { type: "uri", value: "http://ex/Animal" };
  assert.equal(
    tripleKeyOfBindings(s, p, o),
    keyOfLine(`${termToNT(s)} ${termToNT(p)} ${termToNT(o)} .`),
  );
});

test("tripleKeysOfNTriples folds named graphs and skips junk lines", () => {
  const doc = [
    "<http://ex/a> <http://ex/p> <http://ex/b> .",
    "<http://ex/a> <http://ex/p> <http://ex/b> <http://ex/g> .", // same s/p/o, named graph
    "this line is not a statement",
    "",
  ].join("\n");
  const keys = tripleKeysOfNTriples(doc);
  assert.equal(keys.size, 1, "graph term ignored + dedup + junk skipped");
});

test("entailedKeysFromClosure is exactly closure minus base", () => {
  const base = tripleKeysOfNTriples("<http://ex/a> <http://ex/p> <http://ex/b> .");
  const closure = [
    "<http://ex/a> <http://ex/p> <http://ex/b> .",
    "<http://ex/a> <http://ex/q> <http://ex/b> .",
  ].join("\n");
  const entailed = entailedKeysFromClosure(closure, base);
  assert.equal(entailed.size, 1);
  assert.ok(entailed.has(keyOfLine("<http://ex/a> <http://ex/q> <http://ex/b> .")));
  // Membership is the affordance gate: the asserted triple is NOT marked.
  assert.ok(!entailed.has(keyOfLine("<http://ex/a> <http://ex/p> <http://ex/b> .")));
});
