// [OPUS-4.8] sq-rnwc — unit tests for the /surface/http-server walkthrough data + helpers.
// This surface is tier-e (no backend behind the static site) so it replays REAL captured
// I/O. These tests pin the captured frames' SHAPE *and SERIALIZATION* so the page can never
// silently drift into a mock: the recipe catalogue covers the protocol surface, every
// captured response is non-empty and matches its declared media, the live-subscription
// transcript has the snapshot-then-diff shape (a sequence-0 full snapshot followed by a
// sequence-1 incremental diff), AND — the honesty-regression guard added after a hand-edited
// transcript dropped the literal datatype / reordered the SSE id line — every SSE literal
// binding carries its xsd:integer datatype exactly as the server's term_json emits it, the
// `id:` line follows `data:`, the CONSTRUCT recipe is the real two-triple full-IRI output,
// and the default-build UPDATE head carries no time-travel-only header. Run via
// `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  RECIPES,
  SUBSCRIPTION_TRANSCRIPT,
  SEED_TURTLE,
  ENDPOINT,
  transcriptUpTo,
  transcriptSequences,
  recipeById,
} from "../src/lib/http-server.ts";

// Pull the JSON `data:` payload out of a captured SSE frame body. The stored bodies are the
// RAW wire lines, exactly as the server emits them: `event:` then a single-line `data:` JSON
// then (for a notification) an `id:` line. The `data:` payload is therefore the single line
// that follows the `data: ` marker; slice from the marker to the next newline (so a trailing
// `id:` line is not swallowed into the JSON).
function extractSseData(body) {
  const idx = body.indexOf("data: ");
  assert.ok(idx >= 0, "frame body has no data: marker");
  const rest = body.slice(idx + "data: ".length);
  const nl = rest.indexOf("\n");
  return nl >= 0 ? rest.slice(0, nl) : rest;
}

test("the recipe catalogue covers the core protocol surface", () => {
  const ids = RECIPES.map((r) => r.id);
  // SPARQL 1.1 Protocol query forms + update, Graph Store write, EXPLAIN, /metrics.
  for (const want of [
    "select",
    "ask",
    "csv",
    "construct",
    "update",
    "gsp-put",
    "explain",
    "metrics",
  ]) {
    assert.ok(ids.includes(want), `missing recipe: ${want}`);
  }
});

test("every recipe has a curl request and a non-empty captured response", () => {
  for (const r of RECIPES) {
    assert.match(r.curl, /^curl /, `${r.id} curl does not start with curl`);
    assert.ok(r.curl.includes(ENDPOINT), `${r.id} curl does not target the endpoint`);
    assert.ok(r.response.trim().length > 0, `${r.id} response is empty`);
    assert.ok(r.blurb.trim().length > 0, `${r.id} blurb is empty`);
  }
});

test("captured responses match their declared media", () => {
  // JSON results parse as JSON and carry the SPARQL-results head.
  for (const r of RECIPES.filter((x) => x.lang === "json")) {
    const parsed = JSON.parse(r.response);
    assert.ok("head" in parsed, `${r.id} JSON has no head`);
  }
  // The SELECT JSON is a real SPARQL-results object with bindings.
  const select = JSON.parse(recipeById("select").response);
  assert.deepEqual(select.head.vars, ["s", "age"]);
  assert.equal(select.results.bindings.length, 3);
  // ASK returns a boolean envelope.
  const ask = JSON.parse(recipeById("ask").response);
  assert.equal(ask.boolean, true);
  // The HTTP-head recipes show the status line + the always-on hardening headers. (The
  // default-build UPDATE head carries NO Sparq-Generation line — that is asserted in its
  // own dedicated test below; here we only pin the always-on hardening header set.)
  const update = recipeById("update");
  assert.match(update.response, /^HTTP\/1\.1 204 No Content/);
  assert.match(update.response, /x-content-type-options: nosniff/);
  // The PUT shows a Graph-Store create.
  assert.match(recipeById("gsp-put").response, /^HTTP\/1\.1 201 Created/);
  // CSV is comma-separated with the var header row.
  assert.match(recipeById("csv").response, /^s,age\n/);
});

test("recipeById returns undefined for an unknown id", () => {
  assert.equal(recipeById("does-not-exist"), undefined);
});

test("the live-subscription transcript has the snapshot-then-diff shape", () => {
  // Opens with a client GET, then a server `subscribed` ack, then notifications.
  assert.equal(SUBSCRIPTION_TRANSCRIPT[0].side, "client");
  assert.match(SUBSCRIPTION_TRANSCRIPT[0].label, /subscriptions\/sse/);
  assert.match(SUBSCRIPTION_TRANSCRIPT[1].body, /"subscribed"/);

  // The server notification sequence numbers are strictly increasing from 0 (a full
  // snapshot first, then incremental diffs) — exactly what the engine emits.
  const seqs = transcriptSequences();
  assert.deepEqual(seqs, [0, 1]);
  for (let i = 1; i < seqs.length; i++) {
    assert.ok(seqs[i] > seqs[i - 1], "sequence ids must strictly increase");
  }

  // There is an UPDATE action between the snapshot and the diff (the thing that fires it).
  const note = SUBSCRIPTION_TRANSCRIPT.find((f) => f.side === "note");
  assert.ok(note, "transcript has no UPDATE action frame");
  assert.match(note.body, /sparql-update/);
  assert.match(note.body, /INSERT DATA/);

  // The sequence-1 frame is an incremental ADD (one new binding), not a re-snapshot.
  const diff = SUBSCRIPTION_TRANSCRIPT.find((f) => f.sequence === 1);
  const payload = JSON.parse(extractSseData(diff.body));
  assert.equal(payload.notification.sequence, 1);
  assert.equal(payload.notification.addedResults.results.bindings.length, 1);
  assert.equal(payload.notification.removedResults.results.bindings.length, 0);

  // The sequence-0 frame is the full snapshot (all four seeded+written subjects).
  const snap = SUBSCRIPTION_TRANSCRIPT.find((f) => f.sequence === 0);
  const snapPayload = JSON.parse(extractSseData(snap.body));
  assert.equal(snapPayload.notification.sequence, 0);
  assert.equal(snapPayload.notification.addedResults.results.bindings.length, 4);
});

// [OPUS-4.8] sq-rnwc honesty-regression guard. The original transcript was a HAND-EDITED
// mock that dropped the literal `datatype` and reordered the SSE `id:`/`data:` lines, while
// the page promised byte-for-byte real output. These assertions pin the SSE term
// serialization to what the server actually emits (subscriptions.rs::term_json) so the
// payload can never silently drift back to a non-verbatim mock — the previous test only
// checked frame SHAPE, which is how the fabrication slipped through.
test("SSE notification frames are raw wire: event/data/id order, with id AFTER data", () => {
  for (const f of SUBSCRIPTION_TRANSCRIPT.filter(
    (x) => x.side === "server" && typeof x.sequence === "number",
  )) {
    const lines = f.body.split("\n");
    assert.equal(lines[0], "event: notification", `${f.label}: first line must be event:`);
    assert.ok(lines[1].startsWith("data: "), `${f.label}: second line must be data:`);
    // The SSE id line follows the data line (subscriptions.rs::notification_event), it does
    // NOT precede it — the exact ordering the mock got wrong.
    assert.equal(lines[2], `id: ${f.sequence}`, `${f.label}: id: line must follow data: and carry the sequence`);
  }
});

test("every SSE literal binding carries its datatype, matching the SELECT recipe serialization", () => {
  // The SELECT recipe is the canonical SPARQL-JSON term serialization. Pull its `age`
  // binding (a typed xsd:integer literal) and use it as the reference shape.
  const selectBindings = JSON.parse(recipeById("select").response).results.bindings;
  const refAge = selectBindings[0].age;
  assert.deepEqual(
    Object.keys(refAge).sort(),
    ["datatype", "type", "value"],
    "SELECT age literal must serialize as {datatype,type,value}",
  );
  assert.equal(refAge.type, "literal");
  assert.equal(refAge.datatype, "http://www.w3.org/2001/XMLSchema#integer");

  // EVERY age literal across EVERY SSE notification frame must serialize the same way —
  // a non-string literal ALWAYS carries its datatype (term_json), so a bare
  // {value,type} (the old fabrication) must fail here.
  let seen = 0;
  for (const f of SUBSCRIPTION_TRANSCRIPT.filter(
    (x) => x.side === "server" && typeof x.sequence === "number",
  )) {
    const payload = JSON.parse(extractSseData(f.body));
    for (const b of payload.notification.addedResults.results.bindings) {
      assert.ok(b.age, `${f.label}: binding missing age`);
      assert.equal(b.age.type, "literal", `${f.label}: age must be a literal`);
      assert.equal(
        b.age.datatype,
        "http://www.w3.org/2001/XMLSchema#integer",
        `${f.label}: age literal must carry xsd:integer datatype (not a bare string)`,
      );
      assert.ok(
        "value" in b.age,
        `${f.label}: age literal must carry a value`,
      );
      seen++;
    }
  }
  assert.ok(seen >= 5, "expected at least the 4 snapshot + 1 diff age literals");
});

// [OPUS-4.8] sq-rnwc — pin the CONSTRUCT recipe to the engine's ACTUAL output: the
// CONSTRUCT { ?s ?p ?o } WHERE { ?s ex:knows ?o . ?s ?p ?o } pattern over the seed binds
// ?p to ex:knows only (the join forces ?p = ex:knows on a 5-triple seed with no shared
// subject between knows and age beyond the constrained ?s), so it returns exactly the two
// ex:knows triples. The Turtle writer registers a fixed common-prefix set but NOT ex:, so
// those IRIs render in full. The original recipe FABRICATED an @prefix ex: decl, ex:-
// compacted output, and two extra ex:age triples the engine never produces.
test("the CONSTRUCT recipe is the real two-triple, full-IRI engine output (no fabricated ex: rows)", () => {
  const ttl = recipeById("construct").response;
  // A nine-line common-prefix preamble, NONE of which is ex:.
  assert.ok(!/@prefix ex:/.test(ttl), "CONSTRUCT must NOT declare an ex: prefix");
  assert.match(ttl, /@prefix rdf: <http:\/\/www\.w3\.org\/1999\/02\/22-rdf-syntax-ns#> \./);
  assert.match(ttl, /@prefix xsd: <http:\/\/www\.w3\.org\/2001\/XMLSchema#> \./);
  // Exactly the two ex:knows triples, in FULL-IRI form (no ex: compaction).
  assert.match(ttl, /^<http:\/\/ex\/alice> <http:\/\/ex\/knows> <http:\/\/ex\/bob> \.$/m);
  assert.match(ttl, /^<http:\/\/ex\/carol> <http:\/\/ex\/knows> <http:\/\/ex\/alice> \.$/m);
  // And NO ex:age data triples — the original mock invented alice 30 / carol 41 rows.
  assert.ok(!/<http:\/\/ex\/age>/.test(ttl), "CONSTRUCT must not contain ex:age triples");
  const tripleLines = ttl
    .split("\n")
    .filter((l) => l.length > 0 && !l.startsWith("@prefix"));
  assert.equal(tripleLines.length, 2, "CONSTRUCT returns exactly two triples");
});

// [OPUS-4.8] sq-rnwc — the default-build UPDATE 204 head carries NO Sparq-Generation header
// (that header exists only under the opt-in `time-travel` feature). The original recipe
// fabricated a `sparq-generation: 1` line for a build that does not emit one.
test("the UPDATE 204 head has the default-build hardening headers and no time-travel-only header", () => {
  const head = recipeById("update").response;
  assert.match(head, /^HTTP\/1\.1 204 No Content/);
  assert.match(head, /x-content-type-options: nosniff/);
  assert.match(head, /content-security-policy: default-src 'none'; frame-ancestors 'none'/);
  assert.match(head, /x-frame-options: DENY/);
  assert.match(head, /referrer-policy: no-referrer/);
  // The time-travel-only generation header is NOT in the default build.
  assert.ok(!/sparq-generation/i.test(head), "default-build UPDATE head must not carry Sparq-Generation");
});

test("transcriptUpTo clamps and reveals a growing prefix", () => {
  assert.deepEqual(transcriptUpTo(-5), []);
  assert.equal(transcriptUpTo(0).length, 0);
  assert.equal(transcriptUpTo(2).length, 2);
  assert.deepEqual(transcriptUpTo(2), SUBSCRIPTION_TRANSCRIPT.slice(0, 2));
  // Over-stepping returns the whole transcript, never more.
  assert.equal(
    transcriptUpTo(999).length,
    SUBSCRIPTION_TRANSCRIPT.length,
  );
});

test("the seed dataset is the graph the captured I/O was recorded against", () => {
  // The captured SELECT rows (bob 25, alice 30, carol 41) come from this seed.
  assert.match(SEED_TURTLE, /ex:alice ex:age 30/);
  assert.match(SEED_TURTLE, /ex:bob\s+ex:age 25/);
  assert.match(SEED_TURTLE, /ex:carol ex:age 41/);
});
