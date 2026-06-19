// [OPUS-4.8] sq-3was — unit tests for the /surface/genai (GenAI / NLQ) walkthrough.
// This surface is tier-e (no backend behind the static site) so it replays REAL captured
// output from the sparq-nlq loop (ReplayLlm fixture + the real engine) over the real
// 1.78M-triple Olympics dataset. These tests pin the captured data's SHAPE *and
// SERIALIZATION* so the page can never silently drift into a fabrication — the exact
// failure mode the sibling http-server page was caught in (dropped datatypes, invented
// rows). In particular: every literal result cell carries its datatype / language tag
// EXACTLY as oxrdf::Term::Display emits it; the captured row counts match the crate's
// own CI cross-check (crates/sparq-nlq/tests/olympics_eval.rs); the one repair case has
// the ["ParseError","Ok"] transcript; and the ASK is the unit-row encoding. Run via
// `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  QUESTIONS,
  SCHEMA_SUMMARY,
  DATASET,
  IS_LIVE_LLM,
  questionByText,
  totalRepairs,
} from "../src/lib/genai.ts";

test("the model step is honestly flagged as NOT a live LLM call", () => {
  // The page MUST NOT claim the NL→SPARQL generation is a live model. This flag drives
  // the page's "scripted fixture · not a live model" labelling; if it ever flips to true
  // the page would need a real live capture (and this guard forces that decision).
  assert.equal(IS_LIVE_LLM, false);
});

test("the schema card is the real introspection deck over the real dataset", () => {
  // Pure index introspection (to_text_summary), no model — pin the headline counts it
  // reports so a hand-edit can't quietly swap in invented numbers.
  assert.equal(DATASET.triples, 1781625);
  assert.match(SCHEMA_SUMMARY, /^# Schema summary — 1781625 triples, 406700 subjects/);
  assert.match(SCHEMA_SUMMARY, /### foaf:Person — 134730 instances/);
  assert.match(SCHEMA_SUMMARY, /### dbo:Sport — 66 instances/);
  // The budget-truncation markers the crate itself emits at 4000 chars.
  assert.ok(SCHEMA_SUMMARY.includes("…"), "schema card must show the real budget truncation");
  assert.ok(SCHEMA_SUMMARY.length <= 4000, "schema card respects the 4000-char budget");
});

test("the question set matches the crate's olympics eval, with the captured row counts", () => {
  // Cross-check against the EXACT (question → total rows, repairs) the crate's CI test
  // crates/sparq-nlq/tests/olympics_eval.rs asserts against the real engine + dataset.
  // If a hand-edit invents a different count, this fails.
  const expected = new Map([
    ["How many athletes are on each team?", { rows: 1184, repairs: 0 }],
    ["How many athletes are in the dataset?", { rows: 1, repairs: 0 }],
    ["List the year and host city of every Olympic games.", { rows: 52, repairs: 0 }],
    ["How many medals of each type were awarded?", { rows: 3, repairs: 0 }],
    ["What is the average height of the athletes?", { rows: 1, repairs: 0 }],
    ["Which team has the most athletes?", { rows: 1, repairs: 0 }],
    ["List all sports with their labels.", { rows: 66, repairs: 0 }],
    ["How many events does each sport have?", { rows: 66, repairs: 1 }],
    ["Are there any athletes taller than 200 centimetres?", { rows: 1, repairs: 0 }],
  ]);
  assert.equal(QUESTIONS.length, expected.size);
  for (const q of QUESTIONS) {
    const want = expected.get(q.question);
    assert.ok(want, `unexpected question: ${q.question}`);
    assert.equal(q.totalRows, want.rows, `${q.question}: row count`);
    assert.equal(q.repairs, want.repairs, `${q.question}: repair rounds`);
  }
  // Exactly one repair round across the set — the fixture exercises the repair path once.
  assert.equal(totalRepairs(), 1);
});

test("every captured query is a SELECT or ASK with declared prefixes", () => {
  for (const q of QUESTIONS) {
    assert.ok(/\bSELECT\b|\bASK\b/.test(q.sparql), `${q.question}: not a SELECT/ASK`);
    // Each query declares its prefixes (the grounding prompt requires it).
    assert.ok(/PREFIX /.test(q.sparql), `${q.question}: no PREFIX declarations`);
  }
});

// ── The honesty-regression guard: term serialization is VERBATIM oxrdf Display. ─────────
// The sibling page dropped literal datatypes when a "captured" payload was hand-edited.
// Pin every literal cell to its EXACT N-Triples serialization so a bare value can never
// sneak in: a typed literal is `"lex"^^<datatype-iri>`, a language literal is `"lex"@tag`.
test("every literal result cell carries its datatype or language tag verbatim", () => {
  // A non-empty result cell is either an IRI (`<...>`), a typed literal
  // (`"..."^^<...>`), a language literal (`"..."@tag`), or null. NOTHING else — in
  // particular, NO bare `"value"` literal without a type/lang (the fabrication shape).
  const IRI = /^<[^>]+>$/;
  const TYPED = /^".*"\^\^<[^>]+>$/s;
  const LANG = /^".*"@[a-zA-Z-]+$/s;
  for (const q of QUESTIONS) {
    for (const row of q.headRows) {
      for (const cell of row) {
        if (cell === null) continue;
        const ok = IRI.test(cell) || TYPED.test(cell) || LANG.test(cell);
        assert.ok(
          ok,
          `${q.question}: cell is not a verbatim IRI/typed-literal/lang-literal: ${cell}`,
        );
        // A literal (starts with a quote) MUST carry a datatype or a language tag.
        if (cell.startsWith('"')) {
          assert.ok(
            cell.includes("^^<") || /@[a-zA-Z-]+$/.test(cell),
            `${q.question}: literal cell missing datatype/lang tag (the fabrication shape): ${cell}`,
          );
        }
      }
    }
  }
});

test("aggregate COUNTs serialize as xsd:integer and AVG as xsd:decimal", () => {
  // COUNT(...) → xsd:integer (the engine's aggregate type), verbatim.
  const teams = questionByText("How many athletes are on each team?");
  assert.equal(
    teams.headRows[0][1],
    '"9114"^^<http://www.w3.org/2001/XMLSchema#integer>',
    "COUNT must be a verbatim xsd:integer",
  );
  // AVG(...) → xsd:decimal with the engine's full precision, verbatim (NOT rounded).
  const avg = questionByText("What is the average height of the athletes?");
  assert.equal(
    avg.headRows[0][0],
    '"176.316366892317380353"^^<http://www.w3.org/2001/XMLSchema#decimal>',
    "AVG must be the verbatim full-precision xsd:decimal",
  );
});

test("the source xsd:int datatype is preserved (not coerced to xsd:integer)", () => {
  // dbp:year literals are typed xsd:int in the source data; the engine preserves that
  // exact datatype on projection — a coercion to xsd:integer would be a serialization
  // drift. This pins the distinction the http-server fabrication blurred.
  const games = questionByText("List the year and host city of every Olympic games.");
  assert.equal(
    games.headRows[0][1],
    '"1896"^^<http://www.w3.org/2001/XMLSchema#int>',
    "dbp:year must keep its source xsd:int datatype",
  );
});

test("language-tagged labels keep their @en tag verbatim", () => {
  const medals = questionByText("How many medals of each type were awarded?");
  assert.equal(medals.headRows[0][0], '"Gold"@en');
  const sports = questionByText("List all sports with their labels.");
  assert.equal(sports.headRows[1][1], '"Alpine Skiing"@en');
});

test("the repair case has the malformed-then-fixed transcript", () => {
  const repair = questionByText("How many events does each sport have?");
  assert.deepEqual(repair.turnOutcomes, ["ParseError", "Ok"]);
  assert.equal(repair.repairs, 1);
  // The FINAL (post-repair) query is well-formed: the aggregate alias paren is closed.
  assert.match(repair.sparql, /\(COUNT\(\?event\) AS \?count\)/);
  assert.ok(!repair.isAsk);
});

test("the ASK case is the unit-row encoding (zero vars, one empty row iff true)", () => {
  const ask = questionByText("Are there any athletes taller than 200 centimetres?");
  assert.equal(ask.isAsk, true);
  assert.deepEqual(ask.vars, []);
  assert.match(ask.sparql, /^[\s\S]*\bASK\b/);
  // A satisfied ASK encodes as one row with zero cells.
  assert.equal(ask.headRows.length, 1);
  assert.deepEqual(ask.headRows[0], []);
  assert.equal(ask.totalRows, 1);
});

test("head rows never exceed the total, and var counts line up with cells", () => {
  for (const q of QUESTIONS) {
    assert.ok(q.headRows.length <= q.totalRows, `${q.question}: more head rows than total`);
    if (!q.isAsk) {
      for (const row of q.headRows) {
        assert.equal(row.length, q.vars.length, `${q.question}: row arity != var count`);
      }
    }
  }
});

test("questionByText returns undefined for an unknown question", () => {
  assert.equal(questionByText("does not exist"), undefined);
});
