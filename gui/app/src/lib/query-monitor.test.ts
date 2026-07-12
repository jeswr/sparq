// [FABLE-5] sq-ixc3.19 — unit tests for the live query monitor registry.
//
// Covers: track/finish lifecycle (register → listed; finish → delisted), kill (aborts
// the signal but leaves the entry listed until the issuer's finish — the list reflects
// what is genuinely still executing), subscriber notification, and the kill-unknown-id
// no-op. The panel wiring is covered by the Playwright spec.
//
// Run via:   npm run test:unit   (gui/app)
import { test, beforeEach } from "node:test";
import assert from "node:assert/strict";

import {
  getRunning,
  killQuery,
  resetQueryMonitorForTests,
  subscribeRunning,
  trackQuery,
} from "./query-monitor.js";

beforeEach(() => resetQueryMonitorForTests());

test("trackQuery – registers the entry; finish delists it", () => {
  const t = trackQuery("Plan · ANALYZE", "SELECT * WHERE { ?s ?p ?o }", "local");
  assert.equal(getRunning().length, 1);
  assert.equal(getRunning()[0].label, "Plan · ANALYZE");
  assert.equal(getRunning()[0].target, "local");
  assert.equal(t.signal.aborted, false);
  t.finish();
  assert.equal(getRunning().length, 0);
});

test("killQuery – aborts the signal but the entry stays until finish (honest liveness)", () => {
  const t = trackQuery("endpoint explain", "ASK { ?s ?p ?o }", "endpoint");
  const id = getRunning()[0].id;
  assert.equal(killQuery(id), true);
  assert.equal(t.signal.aborted, true, "kill aborts the tracked signal");
  // Still listed: the run is only finished when its issuer observes the abort.
  assert.equal(getRunning().length, 1);
  t.finish();
  assert.equal(getRunning().length, 0);
});

test("killQuery – unknown id is a false no-op, never a throw", () => {
  assert.equal(killQuery(999), false);
});

test("subscribeRunning – notified on track, finish, and unsubscribe stops it", () => {
  let events = 0;
  const unsub = subscribeRunning(() => {
    events += 1;
  });
  const t = trackQuery("q", "SELECT * WHERE { ?s ?p ?o }", "local");
  t.finish();
  assert.equal(events, 2, "track + finish each notify");
  unsub();
  trackQuery("q2", "SELECT * WHERE { ?s ?p ?o }", "local");
  assert.equal(events, 2, "unsubscribed listener is silent");
});

test("getRunning – returns a fresh snapshot reference per change (useSyncExternalStore contract)", () => {
  const before = getRunning();
  trackQuery("q", "SELECT * WHERE { ?s ?p ?o }", "local");
  assert.notEqual(getRunning(), before, "reference changes when the list changes");
  const during = getRunning();
  assert.equal(getRunning(), during, "reference is stable between changes");
});
