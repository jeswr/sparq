// [FABLE-5] sq-ixc3.20 — unit tests for the pure proof-view logic (node test runner).

import { strict as assert } from "node:assert";
import { test } from "node:test";

import {
  buildProofDisplay,
  ntTermLabel,
  parseProofJson,
  proofSummary,
  ruleKind,
  ruleLabel,
} from "./proof-view.js";

/** A realistic two-step RDFS proof: rdfs9 over an asserted subclass edge + typing, with the
 *  SHARED subclass premise referenced twice (a DAG, not a tree). */
const PROOF = JSON.stringify({
  root: 3,
  nodes: [
    {
      id: 0,
      conclusion: ["<http://ex/Dog>", "<http://www.w3.org/2000/01/rdf-schema#subClassOf>", "<http://ex/Mammal>"],
      rule: "asserted",
      premises: [],
    },
    {
      id: 1,
      conclusion: ["<http://ex/rex>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://ex/Dog>"],
      rule: "asserted",
      premises: [],
    },
    {
      id: 2,
      conclusion: ["<http://ex/rex>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://ex/Mammal>"],
      rule: "rdfs9",
      premises: [0, 1],
    },
    {
      // Contrived: references node 0 AGAIN so the repeat/see-step path is exercised.
      id: 3,
      conclusion: ["<http://ex/rex>", "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>", "<http://ex/Animal>"],
      rule: "rdfs9",
      premises: [0, 2],
    },
  ],
});

test("parseProofJson: a valid proof parses; the JSON literal null is null", () => {
  const tree = parseProofJson(PROOF);
  assert.ok(tree);
  assert.equal(tree.root, 3);
  assert.equal(tree.nodes.length, 4);
  assert.equal(parseProofJson("null"), null);
});

test("parseProofJson: shape drift throws (never a fabricated proof)", () => {
  // root out of range
  assert.throws(() => parseProofJson('{"root":9,"nodes":[{"id":0,"conclusion":["a","b","c"],"rule":"asserted","premises":[]}]}'));
  // a premise NOT strictly before its node (would make the expansion cyclic)
  assert.throws(() =>
    parseProofJson(
      '{"root":0,"nodes":[{"id":0,"conclusion":["a","b","c"],"rule":"r","premises":[0]}]}',
    ),
  );
  // a 2-term conclusion
  assert.throws(() =>
    parseProofJson('{"root":0,"nodes":[{"id":0,"conclusion":["a","b"],"rule":"r","premises":[]}]}'),
  );
  // not an object at all
  assert.throws(() => parseProofJson("[1,2,3]"));
});

test("buildProofDisplay: expands from the root; a shared node repeats as a reference", () => {
  const tree = parseProofJson(PROOF)!;
  const root = buildProofDisplay(tree);
  assert.equal(root.id, 3);
  assert.equal(root.kind, "rule");
  assert.equal(root.children.length, 2);
  // Depth-first: node 0 expands under the root first…
  const first = root.children[0];
  assert.equal(first.id, 0);
  assert.equal(first.repeat, false);
  // …and its SECOND occurrence (under node 2) is a repeat reference with no children.
  const second = root.children[1].children.find((c) => c.id === 0)!;
  assert.ok(second);
  assert.equal(second.repeat, true);
  assert.equal(second.children.length, 0);
});

test("proofSummary counts steps / rule firings / asserted leaves", () => {
  const s = proofSummary(parseProofJson(PROOF)!);
  assert.deepEqual(s, { steps: 4, ruleFirings: 2, assertedLeaves: 2 });
});

test("ruleKind + ruleLabel classify honestly", () => {
  assert.equal(ruleKind("asserted"), "asserted");
  assert.equal(ruleKind("axiom-rdfs"), "axiom");
  assert.equal(ruleKind("rdfs9"), "rule");
  assert.equal(ruleKind("n3-rule-0"), "rule");
  assert.equal(ruleLabel("n3-rule-0"), "N3 rule #1");
  assert.equal(ruleLabel("cax-sco"), "cax-sco");
});

test("ntTermLabel: IRIs abbreviate, literals shorten, bnodes verbatim", () => {
  assert.equal(ntTermLabel("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"), "rdf:type");
  assert.equal(ntTermLabel("<http://ex/Mortal>"), "Mortal");
  assert.equal(ntTermLabel("_:b0"), "_:b0");
  assert.equal(ntTermLabel('"hi"@en'), '"hi"');
  assert.equal(ntTermLabel('"5"^^<http://www.w3.org/2001/XMLSchema#integer>'), '"5"');
  // An escaped quote inside the lexical form does not truncate the label early.
  assert.equal(ntTermLabel('"a\\"b"'), '"a\\"b"');
  const long = `"${"x".repeat(40)}"`;
  assert.ok(ntTermLabel(long).endsWith('…"'));
});
