// [FABLE-5] sq-ixc3.16 — unit tests for the Streaming tool's workspace-feed pure helpers.
//
// Covers: parseNQuadsLine (the N-Quads term tokenizer — IRIs, blank nodes, literals with
// spaces / escaped quotes / language tags / datatypes, graph folding, malformed rejection)
// and diffAddedTriples (baseline diff, cross-graph dedup, deletion + re-add semantics,
// malformed accounting). The end-to-end feed (storeEpoch → diff → Rsp.push → closed window)
// is exercised by e2e-playwright/specs/streaming-workspace-feed.web.spec.ts.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import { parseNQuadsLine, diffAddedTriples, snapshotKeys } from "./rsp-feed.js";

// ---------------------------------------------------------------------------
// parseNQuadsLine
// ---------------------------------------------------------------------------

test("parseNQuadsLine – plain IRI triple", () => {
  assert.deepEqual(parseNQuadsLine("<http://ex/s> <http://ex/p> <http://ex/o> ."), [
    "<http://ex/s>",
    "<http://ex/p>",
    "<http://ex/o>",
  ]);
});

test("parseNQuadsLine – folds a named-graph label away", () => {
  assert.deepEqual(parseNQuadsLine("<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g> ."), [
    "<http://ex/s>",
    "<http://ex/p>",
    "<http://ex/o>",
  ]);
});

test("parseNQuadsLine – literal object with spaces and escaped quotes", () => {
  assert.deepEqual(parseNQuadsLine('<http://ex/s> <http://ex/p> "a \\"quoted\\" phrase" .'), [
    "<http://ex/s>",
    "<http://ex/p>",
    '"a \\"quoted\\" phrase"',
  ]);
});

test("parseNQuadsLine – language-tagged literal, in a named graph", () => {
  assert.deepEqual(parseNQuadsLine('<http://ex/s> <http://ex/p> "salut"@fr-BE <http://ex/g> .'), [
    "<http://ex/s>",
    "<http://ex/p>",
    '"salut"@fr-BE',
  ]);
});

test("parseNQuadsLine – typed literal keeps its full datatype IRI", () => {
  assert.deepEqual(
    parseNQuadsLine(
      '<http://ex/s> <http://ex/reading> "10"^^<http://www.w3.org/2001/XMLSchema#integer> .',
    ),
    ["<http://ex/s>", "<http://ex/reading>", '"10"^^<http://www.w3.org/2001/XMLSchema#integer>'],
  );
});

test("parseNQuadsLine – blank-node subject and graph", () => {
  assert.deepEqual(parseNQuadsLine("_:b0 <http://ex/p> _:b1 _:g0 ."), [
    "_:b0",
    "<http://ex/p>",
    "_:b1",
  ]);
});

test("parseNQuadsLine – rejects malformed lines (fail closed)", () => {
  // Missing terminating dot.
  assert.equal(parseNQuadsLine("<http://ex/s> <http://ex/p> <http://ex/o>"), null);
  // Literal predicate.
  assert.equal(parseNQuadsLine('<http://ex/s> "p" <http://ex/o> .'), null);
  // Literal subject.
  assert.equal(parseNQuadsLine('"s" <http://ex/p> <http://ex/o> .'), null);
  // Unterminated literal.
  assert.equal(parseNQuadsLine('<http://ex/s> <http://ex/p> "unterminated .'), null);
  // Trailing garbage after the dot.
  assert.equal(parseNQuadsLine("<http://ex/s> <http://ex/p> <http://ex/o> . junk"), null);
  // Only two terms.
  assert.equal(parseNQuadsLine("<http://ex/s> <http://ex/p> ."), null);
});

// ---------------------------------------------------------------------------
// diffAddedTriples
// ---------------------------------------------------------------------------

const BASE = [
  "<http://ex/s1> <http://ex/reading> \"10\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
  '<http://ex/s1> <http://ex/label> "sensor one" .',
].join("\n");

test("diffAddedTriples – empty baseline reports every folded triple once", () => {
  const { added, keys, malformed } = diffAddedTriples(new Set(), BASE);
  assert.equal(added.length, 2);
  assert.equal(keys.size, 2);
  assert.equal(malformed, 0);
});

test("diffAddedTriples – only NEW triples stream on the next update", () => {
  const baseline = snapshotKeys(BASE);
  const next = BASE + '\n<http://ex/s2> <http://ex/reading> "20"^^<http://www.w3.org/2001/XMLSchema#integer> .';
  const { added, keys } = diffAddedTriples(baseline, next);
  assert.deepEqual(added, [
    [
      "<http://ex/s2>",
      "<http://ex/reading>",
      '"20"^^<http://www.w3.org/2001/XMLSchema#integer>',
    ],
  ]);
  assert.equal(keys.size, 3);
});

test("diffAddedTriples – the same triple in two named graphs streams ONCE (folded dedup)", () => {
  const doc = [
    "<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g1> .",
    "<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g2> .",
  ].join("\n");
  const { added, keys } = diffAddedTriples(new Set(), doc);
  assert.equal(added.length, 1);
  assert.equal(keys.size, 1);
});

test("diffAddedTriples – a deleted-then-readded triple streams again", () => {
  const full = snapshotKeys(BASE);
  // The triple is deleted (snapshot without it)…
  const withoutLabel = '<http://ex/s1> <http://ex/reading> "10"^^<http://www.w3.org/2001/XMLSchema#integer> .';
  const afterDelete = diffAddedTriples(full, withoutLabel);
  assert.equal(afterDelete.added.length, 0);
  assert.equal(afterDelete.keys.size, 1);
  // …then re-added: it is NEW relative to the previous snapshot and streams again.
  const afterReadd = diffAddedTriples(afterDelete.keys, BASE);
  assert.equal(afterReadd.added.length, 1);
  assert.deepEqual(afterReadd.added[0][1], "<http://ex/label>");
});

test("diffAddedTriples – blank/comment lines skipped, malformed lines counted", () => {
  const doc = ["", "# a comment", "not an nquads line", BASE].join("\n");
  const { added, malformed } = diffAddedTriples(new Set(), doc);
  assert.equal(added.length, 2);
  assert.equal(malformed, 1);
});

test("diffAddedTriples – empty snapshot yields an empty delta", () => {
  const { added, keys, malformed } = diffAddedTriples(new Set(), "");
  assert.equal(added.length, 0);
  assert.equal(keys.size, 0);
  assert.equal(malformed, 0);
});
