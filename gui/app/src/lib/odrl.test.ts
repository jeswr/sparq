// [FABLE-5] sq-ixc3.15 — unit tests for odrl.ts pure helpers.
//
// Covers: boundValues / hiddenGraphs (the pane-diff logic that reports which named graphs a
// policy hid from a requester) and describeMalformedPolicy (the fail-closed banner). The IPC
// round-trip is exercised by the Playwright mocked-IPC spec (e2e-playwright/specs/
// odrl-tool.spec.ts) and, against the REAL evaluator + enforcement store, by
// gui/src-tauri/src/odrl.rs's native-lane tests.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults } from "@sparq/client";

import {
  boundValues,
  hiddenGraphs,
  describeMalformedPolicy,
  SAMPLE_DATASET_TRIG,
  SAMPLE_POLICY_TTL,
  SAMPLE_QUERY,
} from "./odrl.js";

/** Build a SELECT results doc binding `?g` (and `?title`) per row. */
function results(rows: Array<{ g: string; title?: string }>): SparqlResults {
  return {
    head: { vars: ["g", "title"] },
    results: {
      bindings: rows.map((r) => ({
        g: { type: "uri", value: r.g },
        ...(r.title ? { title: { type: "literal", value: r.title } } : {}),
      })),
    },
  } as SparqlResults;
}

const PUBLIC_G = "http://example.org/public";
const SECRET_G = "http://example.org/secret";

// ---------------------------------------------------------------------------
// boundValues
// ---------------------------------------------------------------------------

test("boundValues – distinct values in first-seen order", () => {
  const r = results([{ g: SECRET_G }, { g: PUBLIC_G }, { g: SECRET_G }]);
  assert.deepEqual(boundValues(r, "g"), [SECRET_G, PUBLIC_G]);
});

test("boundValues – unbound variable / empty results yield an empty list", () => {
  assert.deepEqual(boundValues(results([]), "g"), []);
  assert.deepEqual(boundValues(results([{ g: PUBLIC_G }]), "nope"), []);
});

// ---------------------------------------------------------------------------
// hiddenGraphs — the "what did the policy hide?" diff
// ---------------------------------------------------------------------------

test("hiddenGraphs – a prohibition-hidden graph shows up in the diff", () => {
  const ungated = results([
    { g: PUBLIC_G, title: "Public report" },
    { g: SECRET_G, title: "Secret memo" },
  ]);
  const alicePane = results([{ g: PUBLIC_G, title: "Public report" }]);
  assert.deepEqual(hiddenGraphs(ungated, alicePane), [SECRET_G]);
});

test("hiddenGraphs – an ungated-equal pane hides nothing (honest empty diff)", () => {
  const both = results([{ g: PUBLIC_G }, { g: SECRET_G }]);
  assert.deepEqual(hiddenGraphs(both, both), []);
});

test("hiddenGraphs – deny-everything (empty pane) hides every ungated graph", () => {
  const ungated = results([{ g: PUBLIC_G }, { g: SECRET_G }]);
  assert.deepEqual(hiddenGraphs(ungated, results([])), [PUBLIC_G, SECRET_G]);
});

// ---------------------------------------------------------------------------
// describeMalformedPolicy — the fail-closed banner
// ---------------------------------------------------------------------------

test("describeMalformedPolicy – states deny-everything AND carries the verbatim reason", () => {
  const msg = describeMalformedPolicy("expected '.' at line 3");
  assert.match(msg, /fail-closed/i);
  assert.match(msg, /denying everything/i);
  assert.ok(msg.includes("expected '.' at line 3"), "the parser reason must be verbatim");
});

// ---------------------------------------------------------------------------
// Sample fixtures — the out-of-the-box demo must stay coherent
// ---------------------------------------------------------------------------

test("sample fixtures – the policy targets the sample graphs and the query is graph-scoped", () => {
  // The prohibition demo only works if the policy targets the graphs the dataset defines
  // and the query iterates named graphs (PodStore's default graph is EMPTY by design).
  assert.ok(SAMPLE_DATASET_TRIG.includes("ex:public {"));
  assert.ok(SAMPLE_DATASET_TRIG.includes("ex:secret {"));
  assert.ok(SAMPLE_POLICY_TTL.includes("odrl:prohibition"));
  assert.ok(SAMPLE_POLICY_TTL.includes("odrl:target ex:secret"));
  assert.match(SAMPLE_QUERY, /GRAPH \?g/);
});
