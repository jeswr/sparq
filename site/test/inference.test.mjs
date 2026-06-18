// [OPUS-4.8] sq-0po6 — unit tests for the pure inference helpers that back the live
// /surface/inference page: the N-Triples line parser, the materializeStats / why() JSON
// parsers, the CURIE shortening, and the proof-tree flattening. The wasm `Reasoner`
// bindings themselves are proven by the Rust tests in crates/sparq-reason-wasm; here we
// only test the framework-free JS shaping of their string outputs. Run via
// `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  parseNTriplesLine,
  parseNTriples,
  parseStats,
  parseProof,
  shortenTerm,
  formatTriple,
  proofRows,
  ruleLabel,
} from "../src/lib/inference.ts";

test("parseNTriplesLine: an IRI triple", () => {
  const t = parseNTriplesLine(
    "<http://ex/Socrates> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Mortal> .",
  );
  assert.deepEqual(t, {
    subject: "<http://ex/Socrates>",
    predicate: "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>",
    object: "<http://ex/Mortal>",
  });
});

test("parseNTriplesLine: a literal with datatype, lang, and escapes", () => {
  const dt = parseNTriplesLine(
    '<http://ex/a> <http://ex/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .',
  );
  assert.equal(dt?.object, '"30"^^<http://www.w3.org/2001/XMLSchema#integer>');

  const lang = parseNTriplesLine('<http://ex/a> <http://ex/n> "hi"@en-GB .');
  assert.equal(lang?.object, '"hi"@en-GB');

  // A quote and a space inside the literal must NOT split the term early.
  const esc = parseNTriplesLine('<http://ex/a> <http://ex/n> "a \\"b\\" c" .');
  assert.equal(esc?.object, '"a \\"b\\" c"');
});

test("parseNTriplesLine: blank node subject/object", () => {
  const t = parseNTriplesLine("_:b0 <http://ex/p> _:b1 .");
  assert.deepEqual(t, {
    subject: "_:b0",
    predicate: "<http://ex/p>",
    object: "_:b1",
  });
});

test("parseNTriplesLine: rejects blank, comment, and malformed lines", () => {
  assert.equal(parseNTriplesLine(""), null);
  assert.equal(parseNTriplesLine("   "), null);
  assert.equal(parseNTriplesLine("# a comment"), null);
  assert.equal(parseNTriplesLine("<a> <b> <c>"), null); // missing terminator
  assert.equal(parseNTriplesLine("<a> <b> ."), null); // only two terms
  assert.equal(parseNTriplesLine("<a> <b> <c> <d> ."), null); // four terms
  assert.equal(parseNTriplesLine('<a> <b> "unterminated .'), null);
});

test("parseNTriples: parses a document, skipping blanks", () => {
  const doc = [
    "<http://ex/Socrates> <http://ex/p> <http://ex/Human> .",
    "",
    "# comment",
    "<http://ex/Socrates> <http://ex/p> <http://ex/Mortal> .",
  ].join("\n");
  assert.equal(parseNTriples(doc).length, 2);
});

test("parseStats: valid and invalid", () => {
  const s = parseStats(
    '{"profile":"rdfs","baseTriples":2,"closureTriples":5,"entailed":3}',
  );
  assert.deepEqual(s, {
    profile: "rdfs",
    baseTriples: 2,
    closureTriples: 5,
    entailed: 3,
  });
  assert.throws(() => parseStats('{"profile":"rdfs"}'));
  assert.throws(() => parseStats("null"));
});

test("parseProof: a tree, the null literal, and a malformed doc", () => {
  const json =
    '{"root":2,"nodes":[' +
    '{"id":0,"conclusion":["<a>","<sco>","<b>"],"rule":"asserted","premises":[]},' +
    '{"id":1,"conclusion":["<s>","<type>","<a>"],"rule":"asserted","premises":[]},' +
    '{"id":2,"conclusion":["<s>","<type>","<b>"],"rule":"rdfs9","premises":[0,1]}]}';
  const tree = parseProof(json);
  assert.equal(tree?.root, 2);
  assert.equal(tree?.nodes.length, 3);

  assert.equal(parseProof("null"), null);
  assert.throws(() => parseProof('{"nodes":[]}')); // no root
});

test("shortenTerm: well-known prefixes, leaves the rest alone", () => {
  assert.equal(shortenTerm("<http://ex/Socrates>"), "ex:Socrates");
  assert.equal(
    shortenTerm("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"),
    "rdf:type",
  );
  assert.equal(
    shortenTerm("<http://www.w3.org/2002/07/owl#sameAs>"),
    "owl:sameAs",
  );
  assert.equal(shortenTerm("<http://other/X>"), "<http://other/X>");
  assert.equal(shortenTerm('"a literal"'), '"a literal"'); // untouched
});

test("formatTriple: CURIE-compacts all three positions", () => {
  assert.equal(
    formatTriple({
      subject: "<http://ex/Socrates>",
      predicate: "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>",
      object: "<http://ex/Mortal>",
    }),
    "ex:Socrates rdf:type ex:Mortal",
  );
});

test("proofRows: indents premises and back-references shared sub-proofs", () => {
  // node 0 is a shared premise of both node 1 and node 2 (the root).
  const tree = {
    root: 2,
    nodes: [
      { id: 0, conclusion: ["<a>", "<p>", "<b>"], rule: "asserted", premises: [] },
      { id: 1, conclusion: ["<c>", "<p>", "<d>"], rule: "rdfsX", premises: [0] },
      { id: 2, conclusion: ["<e>", "<p>", "<f>"], rule: "rdfsY", premises: [1, 0] },
    ],
  };
  const rows = proofRows(tree);
  // Root first (depth 0), then its first premise (depth 1) and ITS premise (depth 2),
  // then the root's second premise — node 0 again — as a repeated back-ref (depth 1).
  assert.equal(rows[0].node.id, 2);
  assert.equal(rows[0].depth, 0);
  assert.equal(rows.at(-1).node.id, 0);
  assert.equal(rows.at(-1).repeated, true);
  // node 0 appears exactly twice: once expanded, once as a back-ref.
  const zero = rows.filter((r) => r.node.id === 0);
  assert.equal(zero.length, 2);
  assert.equal(zero.filter((r) => r.repeated).length, 1);
});

test("proofRows: tolerates an out-of-range premise index", () => {
  const tree = {
    root: 1,
    nodes: [
      { id: 0, conclusion: ["<a>", "<p>", "<b>"], rule: "asserted", premises: [] },
      { id: 1, conclusion: ["<c>", "<p>", "<d>"], rule: "rdfsX", premises: [0, 99] },
    ],
  };
  // The bogus premise 99 is skipped, not a crash.
  const rows = proofRows(tree);
  assert.equal(rows.length, 2);
});

test("ruleLabel: leaf vs derived", () => {
  assert.equal(ruleLabel("asserted"), "asserted");
  assert.equal(ruleLabel("rdfs9"), "rdfs9");
});
