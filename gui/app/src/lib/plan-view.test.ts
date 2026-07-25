// [FABLE-5] sq-ixc3.19 — unit tests for the plan explorer's pure view logic.
//
// Covers: qErrorSeverity bucketing (the heat thresholds), maxQError / planSummary
// derivation, hotOperatorPath (jump-to-hot, incl. the honest "no measured time → no hot
// operator" case), and the honest formatters (0 wasm nanos renders as unmeasured "—",
// never "0 ns"). The rendering itself is covered by the Playwright mocked-IPC spec
// (e2e-playwright/specs/plan-explorer.web.spec.ts).
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { PlanNode } from "@sparq/client";

import {
  formatCardinality,
  formatNanos,
  formatQError,
  hotOperatorPath,
  maxQError,
  planNodeId,
  planSummary,
  qErrorSeverity,
} from "./plan-view.js";

/** Terse node builder (children default to none). */
function node(partial: Partial<PlanNode> & { operator: string }): PlanNode {
  return {
    estimated: null,
    actual: null,
    nanos: null,
    qError: null,
    children: [],
    ...partial,
  };
}

// ---------------------------------------------------------------------------
// qErrorSeverity — the stated heat thresholds (<2 ok, <10 warn, ≥10 hot).
// ---------------------------------------------------------------------------

test("qErrorSeverity – buckets per the stated thresholds", () => {
  assert.equal(qErrorSeverity(null), "none");
  assert.equal(qErrorSeverity(1), "ok");
  assert.equal(qErrorSeverity(1.99), "ok");
  assert.equal(qErrorSeverity(2), "warn");
  assert.equal(qErrorSeverity(9.99), "warn");
  assert.equal(qErrorSeverity(10), "hot");
  assert.equal(qErrorSeverity(4000), "hot");
});

test("qErrorSeverity – non-finite degrades to none, never a fake bucket", () => {
  assert.equal(qErrorSeverity(Number.POSITIVE_INFINITY), "none");
  assert.equal(qErrorSeverity(Number.NaN), "none");
});

// ---------------------------------------------------------------------------
// maxQError / planSummary.
// ---------------------------------------------------------------------------

const TREE: PlanNode = node({
  operator: "Project ?n",
  actual: 2,
  nanos: 5_000,
  children: [
    node({
      operator: "Join",
      actual: 2,
      nanos: 4_000,
      qError: 1.5,
      children: [
        node({ operator: "BGP a", estimated: 40, actual: 2, nanos: 3_000, qError: 20 }),
        node({ operator: "BGP b", estimated: 2, actual: 2, nanos: 500, qError: 1 }),
      ],
    }),
  ],
});

test("maxQError – the worst q-error anywhere in the subtree", () => {
  assert.equal(maxQError(TREE), 20);
  assert.equal(maxQError(node({ operator: "Filter" })), null);
});

test("planSummary – derives operator count, root rows/nanos, worst q-error", () => {
  const s = planSummary(TREE);
  assert.equal(s.operators, 4);
  assert.equal(s.rootRows, 2);
  assert.equal(s.rootNanos, 5_000);
  assert.equal(s.worstQError, 20);
});

// ---------------------------------------------------------------------------
// hotOperatorPath — jump-to-hot.
// ---------------------------------------------------------------------------

test("hotOperatorPath – the largest measured nanos wins (a child, not the root)", () => {
  // Root has the largest INCLUSIVE time; the hot operator is still root here (5000).
  assert.deepEqual(hotOperatorPath(TREE), []);
  // Make a grandchild the hottest.
  const t2: PlanNode = node({
    operator: "root",
    nanos: 10,
    children: [
      node({ operator: "a", nanos: 20 }),
      node({
        operator: "b",
        nanos: 30,
        children: [node({ operator: "b0", nanos: 900 })],
      }),
    ],
  });
  assert.deepEqual(hotOperatorPath(t2), [1, 0]);
});

test("hotOperatorPath – no measured time (dry run / wasm zeros) → null, no fake hot op", () => {
  assert.equal(hotOperatorPath(node({ operator: "BGP" })), null);
  // A wasm ANALYZE: every node reads 0 nanos — unmeasured, so no hot operator.
  const zeros: PlanNode = node({
    operator: "root",
    nanos: 0,
    actual: 5,
    children: [node({ operator: "a", nanos: 0, actual: 5 })],
  });
  assert.equal(hotOperatorPath(zeros), null);
});

test("planNodeId – stable row ids from the child-index path", () => {
  assert.equal(planNodeId([]), "root");
  assert.equal(planNodeId([1, 0]), "root.1.0");
});

// ---------------------------------------------------------------------------
// Honest formatting.
// ---------------------------------------------------------------------------

test("formatNanos – 0/null render as unmeasured '—', real nanos scale units", () => {
  assert.equal(formatNanos(null), "—");
  assert.equal(formatNanos(0), "—"); // wasm32 reads 0 = unmeasured, NOT "0 ns"
  assert.equal(formatNanos(999), "999 ns");
  assert.equal(formatNanos(1_500), "1.5 µs");
  assert.equal(formatNanos(2_500_000), "2.5 ms");
  assert.equal(formatNanos(3_210_000_000), "3.21 s");
});

test("formatCardinality – null → '—', digits grouped", () => {
  assert.equal(formatCardinality(null), "—");
  assert.equal(formatCardinality(1234567), "1,234,567");
  assert.equal(formatCardinality(2), "2");
});

test("formatQError – precision steps with magnitude", () => {
  assert.equal(formatQError(null), "—");
  assert.equal(formatQError(1.5), "1.50×");
  assert.equal(formatQError(12.34), "12.3×");
  assert.equal(formatQError(4000), "4000×");
});
