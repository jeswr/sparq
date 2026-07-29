// [FABLE-5] sq-ixc3.19 — pure view logic for the plan explorer (no React, no DOM).
//
// Everything a rendered operator row derives from the typed EXPLAIN tree (the sq-jbqh4
// `PlanNode` schema contract, `@sparq/client`) lives here so it is unit-testable with the
// gui/app node test runner: q-error severity bucketing (the heat scale), the hot-operator
// path (jump-to-hot + auto-expand), honest number formatting (0 wall nanos are
// sub-resolution/unmeasured, never "free"), and the plan summary line. The React tree component
// (components/workbench/plan-tree.tsx) is a thin renderer over these.

import type { PlanNode } from "@sparq/client";

// ---------------------------------------------------------------------------
// q-error heat.
// ---------------------------------------------------------------------------

/**
 * The heat bucket of one operator's q-error (= max(est/actual, actual/est), ≥ 1 when
 * present). Buckets, not a continuous ramp, so the colours stay legible and the
 * thresholds are honest, stated numbers rather than a normalised gradient:
 *   * `none` — no q-error (no estimate on this operator, or a plan-only dry run);
 *   * `ok`   — q-error < 2 (estimate within 2× of reality — a good estimate);
 *   * `warn` — 2 ≤ q-error < 10 (a real misestimate; worth a look);
 *   * `hot`  — q-error ≥ 10 (an order-of-magnitude misestimate — the panel's headline).
 */
export type QErrorSeverity = "none" | "ok" | "warn" | "hot";

/** Bucket one operator's q-error (see {@link QErrorSeverity} for the thresholds). */
export function qErrorSeverity(qError: number | null): QErrorSeverity {
  if (qError === null || !Number.isFinite(qError)) return "none";
  if (qError < 2) return "ok";
  if (qError < 10) return "warn";
  return "hot";
}

/** The worst q-error anywhere in the subtree, or null when nothing carries one. */
export function maxQError(node: PlanNode): number | null {
  let worst = node.qError;
  for (const child of node.children) {
    const c = maxQError(child);
    if (c !== null && (worst === null || c > worst)) worst = c;
  }
  return worst;
}

// ---------------------------------------------------------------------------
// The hot operator (jump-to-hot).
// ---------------------------------------------------------------------------

/**
 * Path (child indexes from the root; `[]` = the root itself) to the operator with the
 * LARGEST wall nanos — the panel's "jump to hot operator" target. Returns `null` when no
 * node carries a positive `nanos` (a plan-only dry run, or work below the host timer's
 * resolution): there is no honest hot operator to jump to then.
 */
export function hotOperatorPath(node: PlanNode): number[] | null {
  let best: { path: number[]; nanos: number } | null = null;
  const walk = (n: PlanNode, path: number[]) => {
    if (n.nanos !== null && n.nanos > 0 && (best === null || n.nanos > best.nanos)) {
      best = { path, nanos: n.nanos };
    }
    n.children.forEach((c, i) => walk(c, [...path, i]));
  };
  walk(node, []);
  return best === null ? null : (best as { path: number[]; nanos: number }).path;
}

/** The stable per-row DOM id for a node path (used by expand/scroll targeting). */
export function planNodeId(path: number[]): string {
  return path.length === 0 ? "root" : `root.${path.join(".")}`;
}

// ---------------------------------------------------------------------------
// Honest formatting.
// ---------------------------------------------------------------------------

/**
 * Format an operator's wall time. `null` (not analyzed) and `0` both render as `"—"`:
 * a 0 is sub-resolution or unmeasured, and claiming "0 ns" would misread as "free".
 * Real (positive) nanos scale to the readable
 * unit. The panel-level source note explains WHY times may be absent.
 */
export function formatNanos(nanos: number | null): string {
  if (nanos === null || nanos <= 0) return "—";
  if (nanos < 1_000) return `${nanos} ns`;
  if (nanos < 1_000_000) return `${(nanos / 1_000).toFixed(1)} µs`;
  if (nanos < 1_000_000_000) return `${(nanos / 1_000_000).toFixed(1)} ms`;
  return `${(nanos / 1_000_000_000).toFixed(2)} s`;
}

/** Format a cardinality (estimate or actual): `null` → `"—"`, else grouped digits. */
export function formatCardinality(n: number | null): string {
  if (n === null || !Number.isFinite(n)) return "—";
  return new Intl.NumberFormat("en-US").format(n);
}

/** Format a q-error: `null` → `"—"`, else `×`-suffixed with sensible precision. */
export function formatQError(q: number | null): string {
  if (q === null || !Number.isFinite(q)) return "—";
  const s = q >= 100 ? q.toFixed(0) : q >= 10 ? q.toFixed(1) : q.toFixed(2);
  return `${s}×`;
}

// ---------------------------------------------------------------------------
// Summary.
// ---------------------------------------------------------------------------

/** The one-line plan summary the panel header shows. All values derived, none fabricated. */
export interface PlanSummary {
  /** Total operators in the tree. */
  operators: number;
  /** The root operator's output rows (ANALYZE only; null on a dry run). */
  rootRows: number | null;
  /** The root operator's wall nanos (ANALYZE with a real clock only; see formatNanos). */
  rootNanos: number | null;
  /** The worst q-error in the tree (null when nothing carries one). */
  worstQError: number | null;
}

/** Derive the {@link PlanSummary} of a plan tree. */
export function planSummary(root: PlanNode): PlanSummary {
  let operators = 0;
  const count = (n: PlanNode) => {
    operators += 1;
    n.children.forEach(count);
  };
  count(root);
  return {
    operators,
    rootRows: root.actual,
    rootNanos: root.nanos,
    worstQError: maxQError(root),
  };
}
