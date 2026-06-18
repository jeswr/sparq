// [OPUS-4.8] sq-rnwc — unit tests for the /surface/http-server walkthrough data + helpers.
// This surface is tier-e (no backend behind the static site) so it replays REAL captured
// I/O. These tests pin the captured frames' SHAPE so the page can never silently drift into
// a mock: the recipe catalogue covers the protocol surface, every captured response is
// non-empty and matches its declared media, and the live-subscription transcript has the
// snapshot-then-diff shape (a sequence-0 full snapshot followed by a sequence-1 incremental
// diff) that the engine actually emits. Run via `npm run test:unit`.
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

// Pull the JSON `data:` payload out of a captured SSE frame body. In our stored
// representation the JSON is pretty-printed across several lines after the `data: ` marker
// (the `id:` line, if any, precedes it), so take everything from the `data: ` marker on.
function extractSseData(body) {
  const idx = body.indexOf("data: ");
  assert.ok(idx >= 0, "frame body has no data: marker");
  return body.slice(idx + "data: ".length);
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
  // The HTTP-head recipes show the status line + the always-on hardening headers.
  const update = recipeById("update");
  assert.match(update.response, /^HTTP\/1\.1 204 No Content/);
  assert.match(update.response, /x-content-type-options: nosniff/);
  assert.match(update.response, /sparq-generation: \d+/);
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
