// [OPUS-4.8] sq-dwdm — unit tests for the /surface/vector (Vector / ANN) walkthrough.
// This surface is tier-e (no backend behind the static site) so it replays REAL captured
// output from the sparq-vectors binary (the capture harness
// crates/sparq-vectors/examples/capture_surface_vector.rs, built with `--features
// vec-predicate`) over a tiny declared in-memory fixture, with the answer-EXACT backend.
// These tests pin the captured data's SHAPE *and SERIALIZATION* so the page can never
// silently drift into a fabrication — the exact failure mode the sibling http-server page
// (sq-rnwc) was caught in (dropped datatypes, invented rows). In particular: every result
// cell is the engine's verbatim oxrdf::Term::Display string; the vec:search cosine carries
// its xsd:double datatype EXACTLY; the BGP-joined labels are PLAIN string literals (no
// invented datatype); and the captured Bolt cosines are pinned. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  BOLT,
  IS_EXACT_BACKEND,
  IS_SEMANTIC_EMBEDDER,
  RECALL_NOTES,
  UNIT_CIRCLE,
  VEC_NS,
  VEC_QUERIES,
  vecQueryById,
} from "../src/lib/vector.ts";

test("the embedder is honestly flagged as NOT a semantic model, backend is exact", () => {
  // The page MUST NOT claim the captures show semantic retrieval quality. The HashEmbedder
  // is a deterministic lexical hash; the captured search is the answer-exact scan. If
  // either flag ever flips, the page would need a real re-capture (this forces the choice).
  assert.equal(IS_SEMANTIC_EMBEDDER, false);
  assert.equal(IS_EXACT_BACKEND, true);
});

test("the vec: namespace matches the crate's vocabulary", () => {
  assert.equal(VEC_NS, "http://sparq.dev/vec#");
});

// ── The Usain Bolt label-embedding capture ──────────────────────────────────────────────
test("the Bolt run embeds all five labeled entities and excludes the seed", () => {
  assert.equal(BOLT.fixture.length, 5);
  assert.equal(BOLT.embedded, 5);
  assert.equal(BOLT.seed, "<http://example.org/bolt>");
  // The seed must NOT appear among its own neighbours (self-exclusion is real, not faked).
  for (const n of BOLT.neighbours) {
    assert.notEqual(n.term, BOLT.seed, "seed must be excluded from its neighbours");
  }
});

test("the captured Bolt neighbours are verbatim, best-first, with pinned cosines", () => {
  // Pin the EXACT captured output (term + f32 cosine) so a hand-edit can't invent a
  // neighbour or a score. Captured from the real binary 2026-06-19.
  assert.deepEqual(
    BOLT.neighbours.map((n) => n.term),
    [
      "<http://example.org/bolt2>",
      "<http://example.org/blake>",
      "<http://example.org/powell>",
      "<http://example.org/coubertin>",
    ],
  );
  // bolt2 (shared n-grams) is the nearest; cosine pinned to the engine's exact f32.
  assert.equal(BOLT.neighbours[0].cosine, 0.8762895);
  // Cosines are in [-1, 1] and strictly non-increasing (best-first).
  let prev = Infinity;
  for (const n of BOLT.neighbours) {
    assert.ok(n.cosine >= -1.0001 && n.cosine <= 1.0001, `cosine out of range: ${n.cosine}`);
    assert.ok(n.cosine <= prev, "cosines must be non-increasing (best-first)");
    prev = n.cosine;
    // Every neighbour term is a verbatim IRI (<...>), not a bare value.
    assert.match(n.term, /^<[^>]+>$/, `neighbour is not a verbatim IRI: ${n.term}`);
  }
});

// ── The vec: magic-predicate captures ───────────────────────────────────────────────────
test("the unit-circle store is the dim-2 fixture the captures ran over", () => {
  assert.equal(UNIT_CIRCLE.length, 5);
  for (const e of UNIT_CIRCLE) {
    assert.equal(e.vec.length, 2, `${e.iri}: expected a dim-2 vector`);
  }
});

test("every captured vec: query is real SPARQL with the vec: prefix", () => {
  for (const q of VEC_QUERIES) {
    assert.match(q.sparql, /PREFIX vec: <http:\/\/sparq\.dev\/vec#>/, `${q.id}: no vec: prefix`);
    assert.match(q.sparql, /vec:(nearest|search)/, `${q.id}: no vec: predicate`);
    assert.ok(q.vars.length >= 1, `${q.id}: no projected variables`);
    // Head rows never exceed what a query of this shape returns; arity lines up.
    for (const row of q.rows) {
      assert.equal(row.length, q.vars.length, `${q.id}: row arity != var count`);
    }
  }
});

// ── The honesty-regression guard: term serialization is VERBATIM oxrdf Display. ─────────
// The sibling page dropped a literal's datatype when a "captured" payload was hand-edited.
// Pin every result cell to its EXACT N-Triples serialization so a bare/mistyped value can
// never sneak in: an IRI is `<...>`, a plain literal is `"..."`, a typed literal is
// `"..."^^<datatype-iri>`.
test("every vec: result cell is a verbatim IRI or correctly-serialized literal", () => {
  const IRI = /^<[^>]+>$/;
  const PLAIN = /^"[^"]*"$/;
  const TYPED = /^".*"\^\^<[^>]+>$/s;
  for (const q of VEC_QUERIES) {
    for (const row of q.rows) {
      for (const cell of row) {
        const ok = IRI.test(cell) || PLAIN.test(cell) || TYPED.test(cell);
        assert.ok(ok, `${q.id}: cell is not a verbatim IRI/plain-literal/typed-literal: ${cell}`);
      }
    }
  }
});

test("vec:nearest returns the correct verbatim IRIs (pinned)", () => {
  // "1,0" → the two most +x-aligned: a (exact) then c (near +x).
  const byVec = vecQueryById("nearest-by-vector");
  assert.deepEqual(byVec.rows, [["<http://ex/a>"], ["<http://ex/c>"]]);
  // Neighbours of <a>; a itself excluded → c is nearest.
  const bySeed = vecQueryById("nearest-by-seed");
  assert.deepEqual(bySeed.rows, [["<http://ex/c>"]]);
});

test("BGP-joined labels are PLAIN string literals — verbatim, no invented datatype", () => {
  // The fabrication shape would coerce or strip these; the real engine emits plain string
  // literals because that is exactly how `<http://ex/label>` objects are stored.
  const joined = vecQueryById("nearest-joined");
  assert.deepEqual(joined.rows, [['"epsilon"'], ['"beta"']]);
  for (const row of joined.rows) {
    for (const cell of row) {
      assert.match(cell, /^"[^"]*"$/, `expected a plain string literal, got: ${cell}`);
      assert.ok(!cell.includes("^^"), `joined label must NOT carry a datatype: ${cell}`);
    }
  }
});

test("vec:search binds the cosine as a verbatim xsd:double, ordered best-first", () => {
  const search = vecQueryById("search-score");
  // The exact captured output: a (1.0) > c (~0.994) > e (~0.200), each an xsd:double.
  assert.deepEqual(search.rows, [
    ["<http://ex/a>", '"1"^^<http://www.w3.org/2001/XMLSchema#double>'],
    ["<http://ex/c>", '"0.9938837289810181"^^<http://www.w3.org/2001/XMLSchema#double>'],
    ["<http://ex/e>", '"0.19996000826358795"^^<http://www.w3.org/2001/XMLSchema#double>'],
  ]);
  // Every score cell carries the xsd:double datatype verbatim (the serialization the
  // fabrication blurred), and the parsed scores are non-increasing (ORDER BY DESC).
  const DOUBLE = "^^<http://www.w3.org/2001/XMLSchema#double>";
  let prev = Infinity;
  for (const row of search.rows) {
    const scoreCell = row[1];
    assert.ok(scoreCell.endsWith(DOUBLE), `score missing xsd:double datatype: ${scoreCell}`);
    const val = Number.parseFloat(scoreCell.slice(1));
    assert.ok(val <= prev + 1e-9, "scores must be non-increasing (ORDER BY DESC)");
    prev = val;
  }
});

// ── The illustrative half is honestly labelled. ─────────────────────────────────────────
test("the recall notes are labelled approximate (< 1.0) and point at a test", () => {
  assert.ok(RECALL_NOTES.length >= 2);
  // The exact backend is named as the answer-exact ground truth.
  assert.ok(
    RECALL_NOTES.some((r) => /exact/i.test(r.backend) && /recall 1\.0/.test(r.note)),
    "an exact, answer-exact backend must be listed",
  );
  // Approximate backends must NOT be claimed as exact — each says recall < 1.0 / approximate.
  for (const r of RECALL_NOTES) {
    if (/exact/i.test(r.backend)) continue;
    assert.match(
      r.note,
      /approximate|< 1\.0/,
      `approximate backend "${r.backend}" must be labelled approximate`,
    );
  }
});

test("vecQueryById returns undefined for an unknown id", () => {
  assert.equal(vecQueryById("does-not-exist"), undefined);
});
