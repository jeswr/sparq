// [SONNET-4.6] sq-hfd82 — the label shown in the metric-count pill on the /benchmarks
// cards, plus the count-INDEPENDENT box that pill reserves for it.
//
// WHY A RESERVED BOX. The pill is masked at visual-regression capture (`data-vr-mask`)
// because its count comes from the continuously-refreshed benchmark snapshot. But a
// Playwright mask follows the element's LIVE bounding box: if the pill resizes when the
// count gains a digit — or crosses the singular/plural boundary — the mask resizes with
// it and the surrounding card pixels move, which is exactly the baseline drift the mask
// is there to stop. `tabular-nums` alone does NOT fix this: it equalises DIGIT advance
// widths (so "10 metrics" and "99 metrics" match) but leaves "9 metrics" vs "10 metrics"
// and "1 metric" vs "2 metrics" different total widths.
//
// So the pill reserves a box sized for `METRIC_BADGE_WIDEST_LABEL` — rendered as an
// invisible ghost, so the reservation is exact in whatever font actually loads rather
// than a guessed pixel width — and centres the live label inside it. Every label this
// module can return fits that box, including counts past `METRIC_BADGE_MAX_COUNT`, which
// degrade to "999+ metrics" rather than growing the pill. The pill is therefore a
// constant, slightly wider pill than a snug one; that is the price of a mask that never
// moves.

/** Largest count rendered exactly; above this the label degrades to `999+ metrics`. */
export const METRIC_BADGE_MAX_COUNT = 999;

/**
 * The widest label `metricBadgeLabel` can return — every other label has no more digits
 * and no more non-digit characters. Rendered as an invisible ghost to size the pill.
 */
export const METRIC_BADGE_WIDEST_LABEL = `${METRIC_BADGE_MAX_COUNT}+ metrics`;

/** The pill's label: "soon" for a family with no data, else "N metric(s)". */
export function metricBadgeLabel(count: number): string {
  if (!Number.isFinite(count) || count <= 0) return "soon";
  if (count > METRIC_BADGE_MAX_COUNT) return METRIC_BADGE_WIDEST_LABEL;
  return `${count} metric${count === 1 ? "" : "s"}`;
}
