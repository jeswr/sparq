"use client";

// [OPUS-4.8] sq-vjn4 — a collapsible SUITE group (LUBM / WatDiv / SP2Bench / BSBM / …).
// COLLAPSED: header shows the suite name, the metric count, and the LIVE-COMPUTED
// competitive summary pill (speedup band / "competitor baseline pending" / "sparq numbers
// only"). EXPANDED: the summary detail, the per-benchmark metric table, any same-box
// cross-engine table, and any cross-machine reference baselines. State is client-side;
// the summary itself is computed server-side and passed in (no fabrication client-side).
import * as React from "react";
import { ChevronRight } from "lucide-react";

import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";
import { MetricTable } from "@/components/benchmarks/metric-table";
import { SameBoxTable } from "@/components/benchmarks/same-box-table";
import { HttpPanelTable } from "@/components/benchmarks/http-panel-table";
import { ReferencesNote } from "@/components/benchmarks/references-note";
import { TrendCharts } from "@/components/benchmarks/trend-charts";
import { ScalingCharts } from "@/components/benchmarks/scaling-charts";
import {
  SummaryDetail,
  SummaryPill,
} from "@/components/benchmarks/competitive-summary";
import type {
  CompetitiveSummary,
  MetricRow,
  ReferenceBaseline,
  SameBoxComparison,
  ScalingFamily,
  TrendSeries,
} from "@/data/benchmarks";

export interface SuiteGroupData {
  suite: string;
  rows: MetricRow[];
  summary: CompetitiveSummary;
  sameBox?: SameBoxComparison;
  // [OPUS-4.8] sq-7d3dj.34.3 — the canonical same-mode HTTP panel (full-request + TTFB) +
  // its own honest wins/losses summary, rendered below the CLI matrix when present.
  httpSameBox?: SameBoxComparison;
  httpSummary?: CompetitiveSummary;
  references: ReferenceBaseline[];
  trends: TrendSeries[];
  scaling: ScalingFamily[];
}

export function SuiteGroup({
  data,
  defaultOpen = false,
}: {
  data: SuiteGroupData;
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = React.useState(defaultOpen);
  const bodyId = `suite-${data.suite.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`;

  return (
    <Card className="gap-0 overflow-hidden py-0">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        aria-controls={bodyId}
        className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-muted/40"
      >
        <ChevronRight
          aria-hidden
          className={cn(
            "size-4 shrink-0 text-muted-foreground transition-transform",
            open && "rotate-90",
          )}
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <span className="font-semibold">{data.suite}</span>
          <span className="text-xs text-muted-foreground">
            {data.rows.length > 0
              ? `${data.rows.length} benchmark${data.rows.length === 1 ? "" : "s"}`
              : data.sameBox
                ? `${data.sameBox.rows.length} same-box quer${data.sameBox.rows.length === 1 ? "y" : "ies"}`
                : "0 benchmarks"}
          </span>
        </div>
        <SummaryPill summary={data.summary} />
      </button>

      {open && (
        <div id={bodyId} className="space-y-4 border-t px-4 py-4">
          <SummaryDetail suite={data.suite} summary={data.summary} />
          {/* [FABLE-5] sq-hmd7l.28 — a comparison-only group (a new-axis same-box gather with
              no CI-feed metric yet) carries zero metric rows; skip the empty per-metric table
              and let the same-box cross-engine table below be the group's content. */}
          {data.rows.length > 0 && <MetricTable rows={data.rows} />}
          <TrendCharts series={data.trends} />
          <ScalingCharts families={data.scaling} />
          {data.sameBox && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Same-box cross-engine comparison</h4>
              <SameBoxTable comparison={data.sameBox} />
            </div>
          )}
          {data.httpSameBox && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium">
                Same-box HTTP-server panel — full-request + TTFB
              </h4>
              <HttpPanelTable
                comparison={data.httpSameBox}
                summary={data.httpSummary}
              />
            </div>
          )}
          <ReferencesNote references={data.references} />
        </div>
      )}
    </Card>
  );
}
