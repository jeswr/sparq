// [OPUS-4.8] sq-hsyg — per-metric TREND charts (metric over commits), ported from the
// standalone bench/dashboard's Chart.js trend cards (dashboard.js trendCard/trendPoints).
// One card per metric in the suite, showing its value across the committed history window.
// Numbers are exactly the existing benchmark series (CI-runner band — INDICATIVE, the same
// labelling the rest of the in-site benchmarks carry); we never fabricate a missing point —
// a metric measured in fewer commits simply plots fewer points, a single-point metric shows
// one marker. The "Show trends" disclosure keeps first paint light when a suite has many
// metrics. Client component (collapsible state); the series is computed server-side + passed.
"use client";

import * as React from "react";
import { ChevronRight } from "lucide-react";

import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";
import { LineChart, type ChartPoint } from "@/components/benchmarks/line-chart";
import type { TrendSeries } from "@/data/benchmarks";

const CHART_VARS = ["--chart-1", "--chart-2", "--chart-3", "--chart-4", "--chart-5"];

function fmtDate(ms: number): string {
  if (!Number.isFinite(ms)) return "";
  return new Date(ms).toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

export function TrendCharts({ series }: { series: TrendSeries[] }) {
  const [open, setOpen] = React.useState(false);
  // Only metrics with at least one history point are chartable (always true here, but keep
  // the guard so an empty history degrades to "nothing to plot" rather than a broken card).
  const chartable = series.filter((s) => s.points.length > 0);
  if (chartable.length === 0) return null;

  const multiPoint = chartable.filter((s) => s.points.length > 1).length;

  return (
    <div className="space-y-3">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        className="flex items-center gap-2 text-sm font-medium hover:text-primary"
      >
        <ChevronRight
          aria-hidden
          className={cn("size-4 transition-transform", open && "rotate-90")}
        />
        Trends over commits ({chartable.length} metric{chartable.length === 1 ? "" : "s"})
      </button>

      {open && (
        <>
          <p className="text-xs text-muted-foreground">
            Value across the last {chartable[0].points.length} recorded commit
            {chartable[0].points.length === 1 ? "" : "s"} (every metric smaller-is-better;
            indicative CI-runner numbers — see the note above).
            {multiPoint === 0 && " History is sparse — only the latest point exists so far."}
          </p>
          <div className="grid gap-3 sm:grid-cols-2">
            {chartable.map((s, i) => {
              const points: ChartPoint[] = s.points.map((p, idx) => ({
                x: p.date ?? idx,
                y: p.value,
                label: p.date != null ? fmtDate(p.date) : `commit ${idx + 1}`,
              }));
              return (
                <Card key={s.name} className="gap-1 p-3">
                  <div className="flex items-baseline justify-between gap-2">
                    <span className="truncate text-sm font-medium" title={s.label}>
                      {s.label}
                    </span>
                    <span className="shrink-0 text-xs text-muted-foreground">{s.unit}</span>
                  </div>
                  <code className="text-[11px] text-muted-foreground">{s.name}</code>
                  <LineChart
                    points={points}
                    unit={s.unit}
                    colorVar={CHART_VARS[i % CHART_VARS.length]}
                    xTickFormat={(x) => fmtDate(x)}
                    ariaLabel={`Trend of ${s.label} over commits`}
                  />
                </Card>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
