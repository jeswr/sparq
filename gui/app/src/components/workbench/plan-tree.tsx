"use client";

// [FABLE-5] sq-ixc3.19 — the navigable EXPLAIN / EXPLAIN ANALYZE operator tree.
//
// One renderer for all three plan sources (in-tab wasm, desktop-native, server endpoint):
// a collapsible operator tree over the typed `PlanNode` schema (the sq-jbqh4 contract),
// with per-operator est-vs-actual cardinality, wall time, and q-error — q-error rows
// heat-tinted with the Badge `color-mix` idiom on the shared `--warning`/`--destructive`
// tokens (no invented hues), plus jump-to-hot-operator. All view DERIVATION (severity
// thresholds, hot path, honest formatting) lives in lib/plan-view.ts where it is
// unit-tested; this file is the thin renderer.
//
// HONESTY: a wasm ANALYZE reads 0 wall nanos (no monotonic clock on wasm32) — those render
// as "—" with an explicit note, never as "0 ns"; the tree itself is never synthesised from
// the text plan (a text-only source renders as text, upstream).

import * as React from "react";
import { ChevronRight, Crosshair } from "lucide-react";
import type { PlanNode } from "@sparq/client";

import { cn } from "@/lib/utils";
import {
  formatCardinality,
  formatNanos,
  formatQError,
  hotOperatorPath,
  planNodeId,
  qErrorSeverity,
  type QErrorSeverity,
} from "@/lib/plan-view";

/** Row tint per q-error bucket — the Badge `color-mix` idiom over the shared tokens. */
const HEAT_ROW: Record<QErrorSeverity, string> = {
  none: "",
  ok: "",
  warn: "bg-[color-mix(in_oklch,var(--warning)_14%,transparent)]",
  hot: "bg-[color-mix(in_oklch,var(--destructive)_16%,transparent)]",
};

/** q-error cell text per bucket (matches the server tool's severity text idiom). */
const HEAT_TEXT: Record<QErrorSeverity, string> = {
  none: "text-muted-foreground",
  ok: "text-muted-foreground",
  warn: "text-[var(--warning)]",
  hot: "text-destructive",
};

/** The shared row grid: operator · est · actual · q-error · time. */
const ROW_GRID = "grid grid-cols-[minmax(0,1fr)_5.5rem_5.5rem_5rem_5.5rem] items-center gap-2";

export interface PlanTreeProps {
  tree: PlanNode;
  /** True for EXPLAIN ANALYZE (actual/nanos/qError populated); false for the dry run. */
  analyzed: boolean;
  /**
   * Where the plan came from — drives the wall-time honesty note: `wasm` ANALYZE times
   * read 0 (unmeasured), only `native` / `endpoint` measure real per-operator time.
   */
  source?: "wasm" | "native" | "endpoint";
}

export function PlanTree({ tree, analyzed, source }: PlanTreeProps) {
  // Collapsed row ids (default: everything expanded — plans are small and scanning is
  // the point). Jump-to-hot clears collapses along the hot path and highlights the row.
  const [collapsed, setCollapsed] = React.useState<ReadonlySet<string>>(new Set());
  const [highlightId, setHighlightId] = React.useState<string | null>(null);
  const rootRef = React.useRef<HTMLDivElement>(null);

  const hotPath = React.useMemo(() => (analyzed ? hotOperatorPath(tree) : null), [analyzed, tree]);

  const toggle = React.useCallback((id: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const jumpToHot = React.useCallback(() => {
    if (!hotPath) return;
    // Expand every ancestor of the hot row, then scroll + highlight it.
    setCollapsed((prev) => {
      const next = new Set(prev);
      for (let i = 0; i < hotPath.length; i++) {
        next.delete(planNodeId(hotPath.slice(0, i)));
      }
      return next;
    });
    const id = planNodeId(hotPath);
    setHighlightId(id);
    // After the expand renders, scroll the row into view.
    requestAnimationFrame(() => {
      rootRef.current
        ?.querySelector(`[data-plan-node-id="${id}"]`)
        ?.scrollIntoView({ block: "center" });
    });
  }, [hotPath]);

  // A wasm ANALYZE cannot measure wall time — say so rather than showing a column of "—"
  // with no explanation (and never render "0 ns").
  const wasmTimeNote = analyzed && source === "wasm";

  return (
    <div ref={rootRef} className="text-xs" data-plan-tree data-plan-source={source ?? "unknown"}>
      <div className="flex items-center justify-between gap-2 border-b border-border px-3 py-1.5">
        <div className={cn(ROW_GRID, "min-w-0 flex-1 font-medium text-muted-foreground")}>
          <span>Operator</span>
          <span className="text-right">Est. rows</span>
          <span className="text-right">Actual</span>
          <span className="text-right">q-error</span>
          <span className="text-right">Time</span>
        </div>
        {hotPath !== null && (
          <button
            type="button"
            className="inline-flex shrink-0 items-center gap-1 rounded border border-border px-1.5 py-0.5 text-[11px] text-muted-foreground hover:bg-accent hover:text-foreground"
            onClick={jumpToHot}
            data-plan-jump-hot
            title="Scroll to the operator with the largest measured wall time"
          >
            <Crosshair className="size-3" aria-hidden />
            Hot operator
          </button>
        )}
      </div>
      {wasmTimeNote && (
        <p className="border-b border-border px-3 py-1.5 text-[11px] text-muted-foreground" data-plan-time-note>
          Wall times are unmeasured in the in-tab engine (wasm has no monotonic clock) — row
          counts and q-error are exact. Use the desktop app&apos;s native ANALYZE or a server
          endpoint for real per-operator times.
        </p>
      )}
      <ul role="tree" aria-label="Query plan operators" className="py-1">
        <PlanRow
          node={tree}
          path={[]}
          depth={0}
          analyzed={analyzed}
          collapsed={collapsed}
          highlightId={highlightId}
          onToggle={toggle}
        />
      </ul>
    </div>
  );
}

interface PlanRowProps {
  node: PlanNode;
  path: number[];
  depth: number;
  analyzed: boolean;
  collapsed: ReadonlySet<string>;
  highlightId: string | null;
  onToggle: (id: string) => void;
}

function PlanRow({ node, path, depth, analyzed, collapsed, highlightId, onToggle }: PlanRowProps) {
  const id = planNodeId(path);
  const severity = qErrorSeverity(node.qError);
  const hasChildren = node.children.length > 0;
  const isCollapsed = collapsed.has(id);

  return (
    <li role="treeitem" aria-expanded={hasChildren ? !isCollapsed : undefined}>
      <div
        className={cn(
          ROW_GRID,
          "px-3 py-0.5",
          HEAT_ROW[severity],
          highlightId === id && "outline outline-1 outline-primary",
        )}
        data-plan-node
        data-plan-node-id={id}
        data-plan-severity={severity}
      >
        <span
          className="flex min-w-0 items-center gap-1"
          style={{ paddingLeft: `${depth * 14}px` }}
        >
          {hasChildren ? (
            // The rail's canonical collapse idiom: chevron rotates 90° when open.
            <button
              type="button"
              aria-label={isCollapsed ? "Expand operator" : "Collapse operator"}
              onClick={() => onToggle(id)}
              className="shrink-0 text-muted-foreground hover:text-foreground"
            >
              <ChevronRight
                className={cn("size-3 transition-transform", !isCollapsed && "rotate-90")}
                aria-hidden
              />
            </button>
          ) : (
            <span className="inline-block size-3 shrink-0" aria-hidden />
          )}
          <span className="min-w-0 flex-1 truncate font-mono" title={node.operator}>
            {node.operator}
          </span>
        </span>
        <span className="text-right font-mono tabular-nums text-muted-foreground">
          {formatCardinality(node.estimated)}
        </span>
        <span className="text-right font-mono tabular-nums">
          {analyzed ? formatCardinality(node.actual) : "—"}
        </span>
        <span className={cn("text-right font-mono tabular-nums", HEAT_TEXT[severity])}>
          {formatQError(node.qError)}
        </span>
        <span className="text-right font-mono tabular-nums text-muted-foreground">
          {formatNanos(node.nanos)}
        </span>
      </div>
      {hasChildren && !isCollapsed && (
        <ul role="group">
          {node.children.map((child, i) => (
            <PlanRow
              key={planNodeId([...path, i])}
              node={child}
              path={[...path, i]}
              depth={depth + 1}
              analyzed={analyzed}
              collapsed={collapsed}
              highlightId={highlightId}
              onToggle={onToggle}
            />
          ))}
        </ul>
      )}
    </li>
  );
}
