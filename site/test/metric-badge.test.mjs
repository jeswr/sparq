// [SONNET-4.6] sq-hfd82 — unit tests for the /benchmarks card metric-count pill
// (src/lib/metric-badge.ts). The pill is masked at visual-regression capture, and a
// Playwright mask follows the element's LIVE bounding box — so the pill must occupy the
// SAME box for every count, or a threshold crossing resizes the mask and moves the
// surrounding card pixels (the drift the mask exists to prevent).
//
// The pill reserves that box by rendering METRIC_BADGE_WIDEST_LABEL as an invisible ghost
// and centring the live label over it, so the invariant these tests pin is: every label
// metricBadgeLabel() can return fits inside the ghost. With `tabular-nums` on the pill all
// digits share one advance width, so "fits" reduces to "no more digits AND no more
// non-digit characters than the ghost" — and every numeric label's non-digit tail
// (" metric" / " metrics") is a subsequence of the ghost's ("+ metrics"), so the character
// counts really do dominate the widths. Representative counts cover the singular/plural
// boundary (1 → 2) and both digit-width boundaries (9 → 10, 99 → 100). Run via
// `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
  METRIC_BADGE_MAX_COUNT,
  METRIC_BADGE_WIDEST_LABEL,
  metricBadgeLabel,
} from "../src/lib/metric-badge.ts";

const digitCount = (s) => (s.match(/\d/g) ?? []).length;

test("labels: 'soon' with no data, then singular/plural across the digit boundaries", () => {
  assert.equal(metricBadgeLabel(0), "soon");
  assert.equal(metricBadgeLabel(1), "1 metric");
  assert.equal(metricBadgeLabel(2), "2 metrics");
  assert.equal(metricBadgeLabel(9), "9 metrics");
  assert.equal(metricBadgeLabel(10), "10 metrics");
  assert.equal(metricBadgeLabel(99), "99 metrics");
  assert.equal(metricBadgeLabel(100), "100 metrics");
  assert.equal(metricBadgeLabel(METRIC_BADGE_MAX_COUNT), "999 metrics");
});

test("a count past the exact-display range degrades instead of widening the pill", () => {
  assert.equal(metricBadgeLabel(METRIC_BADGE_MAX_COUNT + 1), METRIC_BADGE_WIDEST_LABEL);
  assert.equal(metricBadgeLabel(123_456), METRIC_BADGE_WIDEST_LABEL);
});

test("every label fits the reserved box, so the masked pill never resizes", () => {
  // 0 .. MAX+2 covers "soon", both digit-width boundaries, the singular/plural boundary
  // and the degraded form — i.e. every branch that could change the pill's width.
  for (let count = 0; count <= METRIC_BADGE_MAX_COUNT + 2; count++) {
    const label = metricBadgeLabel(count);
    assert.ok(
      digitCount(label) <= digitCount(METRIC_BADGE_WIDEST_LABEL),
      `${count}: "${label}" has more digits than the ghost "${METRIC_BADGE_WIDEST_LABEL}"`,
    );
    assert.ok(
      label.length <= METRIC_BADGE_WIDEST_LABEL.length,
      `${count}: "${label}" is longer than the ghost "${METRIC_BADGE_WIDEST_LABEL}"`,
    );
  }
});

test("a non-finite count degrades to 'soon' rather than rendering NaN", () => {
  assert.equal(metricBadgeLabel(Number.NaN), "soon");
  assert.equal(metricBadgeLabel(-1), "soon");
});

test("the committed snapshot still fits the exact-display range", () => {
  // A family's count can never exceed the snapshot's TOTAL bench count, so this is a
  // conservative guard: while it holds, no card can hit the degraded "999+" form. If the
  // whole suite ever outgrows it, revisit METRIC_BADGE_MAX_COUNT (and the ghost with it)
  // deliberately instead of discovering it as silent baseline drift.
  const snapshot = JSON.parse(
    readFileSync(new URL("../src/data/benchmarks.generated.json", import.meta.url), "utf8"),
  );
  const total = snapshot.latest.benches.length;
  assert.ok(
    total <= METRIC_BADGE_MAX_COUNT,
    `snapshot has ${total} metrics in total, past the ${METRIC_BADGE_MAX_COUNT} the pill displays exactly`,
  );
});
