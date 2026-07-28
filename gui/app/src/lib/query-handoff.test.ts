// [OPUS-5] sq-ixc3.24 — unit tests for the "open this in the Query tool" handoff bridge.
//
// The invariants that matter: a live subscriber is notified; a handoff published before the
// Query tab has ever mounted is still delivered on mount; and it is delivered EXACTLY ONCE, so
// re-opening the tab later never re-applies an old handoff over the user's own edits.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  publishQuery,
  resetQueryHandoff,
  subscribeToQueryHandoff,
  takePendingQuery,
} from "./query-handoff.js";

test("a live subscriber receives the published query", () => {
  resetQueryHandoff();
  const seen: string[] = [];
  const unsubscribe = subscribeToQueryHandoff((q) => seen.push(q));
  assert.equal(publishQuery("SELECT * WHERE { }"), 1);
  assert.deepEqual(seen, ["SELECT * WHERE { }"]);
  unsubscribe();
  publishQuery("SELECT ?s WHERE { }");
  assert.deepEqual(seen, ["SELECT * WHERE { }"]);
});

test("a handoff delivered live parks nothing for a later mount to replay", () => {
  resetQueryHandoff();
  const unsubscribe = subscribeToQueryHandoff(() => {});
  publishQuery("SELECT ?live WHERE { }");
  unsubscribe();
  // Re-opening the tab later must not resurrect a query the open tab already received.
  assert.equal(takePendingQuery(), null);
});

test("every live subscriber is notified", () => {
  resetQueryHandoff();
  const seen: string[] = [];
  subscribeToQueryHandoff((q) => seen.push(`a:${q}`));
  subscribeToQueryHandoff((q) => seen.push(`b:${q}`));
  assert.equal(publishQuery("ASK { }"), 2);
  assert.deepEqual(seen, ["a:ASK { }", "b:ASK { }"]);
});

test("a query published with no subscribers is still pending for the tab's first mount", () => {
  resetQueryHandoff();
  assert.equal(publishQuery("SELECT ?person WHERE { }"), 0);
  assert.equal(takePendingQuery(), "SELECT ?person WHERE { }");
});

test("a pending handoff is consumed exactly once", () => {
  resetQueryHandoff();
  publishQuery("SELECT ?a WHERE { }");
  assert.equal(takePendingQuery(), "SELECT ?a WHERE { }");
  // A later remount must NOT re-apply it over whatever the user has since typed.
  assert.equal(takePendingQuery(), null);
});

test("the newest handoff replaces an unconsumed older one", () => {
  resetQueryHandoff();
  publishQuery("SELECT ?old WHERE { }");
  publishQuery("SELECT ?new WHERE { }");
  assert.equal(takePendingQuery(), "SELECT ?new WHERE { }");
});

test("unsubscribing twice is harmless", () => {
  resetQueryHandoff();
  const unsubscribe = subscribeToQueryHandoff(() => {});
  unsubscribe();
  unsubscribe();
  assert.equal(publishQuery("SELECT * WHERE { }"), 0);
});
