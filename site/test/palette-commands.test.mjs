// [OPUS-4.8] sq-ixc3.10 — unit tests for the keyboard-first spine's PURE command-model helpers
// (site/src/lib/palette-commands.ts): the recent-query ring, the one-line query preview, and the
// named-graph label. These are the only logic-bearing pieces of the command palette's operational
// layer; the React wiring (the registry provider + the REPL's command list) is exercised by the
// site build's typecheck + the existing REPL e2e, but the data-shaping here is unit-tested
// directly (no DOM), exactly like results.test.mjs / repl-dataset.test.mjs. Run via
// `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  pushRecentQuery,
  previewQuery,
  graphLabel,
  RECENT_QUERY_LIMIT,
  PALETTE_COMMAND_GROUP_ORDER,
} from "../src/lib/palette-commands.ts";

test("pushRecentQuery prepends the newest query, most-recent first", () => {
  let ring = [];
  ring = pushRecentQuery(ring, "SELECT 1", 1000);
  ring = pushRecentQuery(ring, "SELECT 2", 2000);
  assert.deepEqual(
    ring.map((r) => r.query),
    ["SELECT 2", "SELECT 1"],
  );
  assert.equal(ring[0].ranAt, 2000);
});

test("pushRecentQuery de-duplicates by trimmed text, bumping the dupe to the front", () => {
  let ring = [];
  ring = pushRecentQuery(ring, "SELECT *", 1000);
  ring = pushRecentQuery(ring, "ASK {}", 2000);
  // Re-run the first query (with stray whitespace) — it should move to the front, not duplicate.
  ring = pushRecentQuery(ring, "  SELECT *  ", 3000);
  assert.deepEqual(
    ring.map((r) => r.query),
    ["SELECT *", "ASK {}"],
  );
  assert.equal(ring[0].ranAt, 3000, "the bumped entry carries the new timestamp");
});

test("pushRecentQuery ignores an empty / whitespace-only query", () => {
  let ring = pushRecentQuery([], "SELECT 1", 1000);
  ring = pushRecentQuery(ring, "   \n  ", 2000);
  assert.deepEqual(
    ring.map((r) => r.query),
    ["SELECT 1"],
  );
});

test("pushRecentQuery caps the ring at the limit, dropping the oldest", () => {
  let ring = [];
  for (let i = 0; i < RECENT_QUERY_LIMIT + 3; i++) {
    ring = pushRecentQuery(ring, `SELECT ${i}`, 1000 + i);
  }
  assert.equal(ring.length, RECENT_QUERY_LIMIT);
  // Newest is the last pushed; the three oldest were evicted.
  assert.equal(ring[0].query, `SELECT ${RECENT_QUERY_LIMIT + 2}`);
  assert.equal(ring[ring.length - 1].query, "SELECT 3");
});

test("pushRecentQuery does not mutate the input array", () => {
  const prev = [{ query: "SELECT 1", ranAt: 1000 }];
  const next = pushRecentQuery(prev, "SELECT 2", 2000);
  assert.equal(prev.length, 1, "the original array is untouched");
  assert.notEqual(prev, next);
});

test("pushRecentQuery stores the TRIMMED text", () => {
  const ring = pushRecentQuery([], "  SELECT *  ", 1000);
  assert.equal(ring[0].query, "SELECT *");
});

test("previewQuery collapses whitespace to one line", () => {
  assert.equal(
    previewQuery("SELECT ?s\n  WHERE { ?s ?p ?o }"),
    "SELECT ?s WHERE { ?s ?p ?o }",
  );
});

test("previewQuery truncates long queries with an ellipsis at the cap", () => {
  const long = "SELECT " + "?x".repeat(100);
  const out = previewQuery(long, 20);
  assert.equal(out.length, 20, "the visible width stays at the cap");
  assert.ok(out.endsWith("…"));
});

test("previewQuery leaves a short query unchanged (no ellipsis)", () => {
  assert.equal(previewQuery("ASK {}", 64), "ASK {}");
});

test("graphLabel takes the local name after the last # or /", () => {
  assert.equal(graphLabel("http://example.org/graphs/people"), "people");
  assert.equal(graphLabel("http://example.org/onto#Person"), "Person");
  // A hash AFTER the last slash wins (it is the rightmost separator).
  assert.equal(graphLabel("http://example.org/g/onto#Thing"), "Thing");
});

test("graphLabel falls back to the full IRI when there is no usable separator", () => {
  assert.equal(graphLabel("urn:isbn:0451450523"), "urn:isbn:0451450523");
  // A trailing slash has nothing after it → fall back to the whole IRI.
  assert.equal(graphLabel("http://example.org/g/"), "http://example.org/g/");
});

test("PALETTE_COMMAND_GROUP_ORDER lists the operational groups in render order", () => {
  assert.deepEqual(PALETTE_COMMAND_GROUP_ORDER, [
    "Actions",
    "Recent queries",
    "Named graphs",
    "Workspaces",
  ]);
});
