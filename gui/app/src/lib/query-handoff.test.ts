// [SONNET-4.6] sq-ixc3.24 (#2700) — the builder → Query-editor handoff latch.
//
// The invariant that matters: a query handed off while the Query tab is CLOSED is not lost — it
// is delivered when the editor mounts, exactly once.

import assert from "node:assert/strict";
import test from "node:test";
import {
  requestQueryHandoff,
  resetQueryHandoff,
  subscribeQueryHandoff,
} from "./query-handoff.js";

test("a handoff to a mounted editor is delivered immediately", () => {
  resetQueryHandoff();
  const seen: string[] = [];
  const unsubscribe = subscribeQueryHandoff((q) => seen.push(q));
  requestQueryHandoff("SELECT * WHERE { ?s ?p ?o }");
  assert.deepEqual(seen, ["SELECT * WHERE { ?s ?p ?o }"]);
  unsubscribe();
});

test("a handoff requested while the editor is closed is latched, then delivered once", () => {
  resetQueryHandoff();
  requestQueryHandoff("ASK { ?s ?p ?o }");

  const first: string[] = [];
  const unsubscribeFirst = subscribeQueryHandoff((q) => first.push(q));
  assert.deepEqual(first, ["ASK { ?s ?p ?o }"], "delivered on mount");
  unsubscribeFirst();

  // The latch is consumed — remounting must not replay a stale query over the user's edits.
  const second: string[] = [];
  const unsubscribeSecond = subscribeQueryHandoff((q) => second.push(q));
  assert.deepEqual(second, []);
  unsubscribeSecond();
});

test("a newer latched handoff replaces an undelivered one", () => {
  resetQueryHandoff();
  requestQueryHandoff("SELECT ?a WHERE { ?a ?p ?o }");
  requestQueryHandoff("SELECT ?b WHERE { ?b ?p ?o }");

  const seen: string[] = [];
  const unsubscribe = subscribeQueryHandoff((q) => seen.push(q));
  assert.deepEqual(seen, ["SELECT ?b WHERE { ?b ?p ?o }"]);
  unsubscribe();
});

test("unsubscribing stops delivery and re-latches later requests", () => {
  resetQueryHandoff();
  const seen: string[] = [];
  const unsubscribe = subscribeQueryHandoff((q) => seen.push(q));
  unsubscribe();

  requestQueryHandoff("DESCRIBE <http://example.org/a>");
  assert.deepEqual(seen, []);

  const later: string[] = [];
  const unsubscribeLater = subscribeQueryHandoff((q) => later.push(q));
  assert.deepEqual(later, ["DESCRIBE <http://example.org/a>"]);
  unsubscribeLater();
});
