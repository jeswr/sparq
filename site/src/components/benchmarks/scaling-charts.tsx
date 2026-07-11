// [OPUS-4.8] sq-hsyg — SCALING charts (metric vs dataset size/depth), ported from the
// standalone bench/dashboard's scaling view (dashboard.js sizeAxisOf/buildScalingFamilies/
// renderScaling). For SIZE-PARAMETRISED metrics — e.g. the Deep Taxonomy depth series
// (deeptax_d1000_* / deeptax_d10000_*) — the harness encodes the size in the metric name;
// we plot the latest value vs that axis so the reader sees how it SCALES. Where the data has
// no size-parametrised metrics for a family, this renders nothing (no fabricated curve). The
// axis is the real size token; numbers are the existing indicative CI series.
"use client";

import * as React from "react";
import { ChevronRight } from "lucide-react";

import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";
import { LineChart, type ChartPoint } from "@/components/benchmarks/line-chart";
import { fmtNum } from "@/lib/fmt-num";
import type { ScalingFamily } from "@/data/benchmarks";

const CHART_VARS = ["--chart-2", "--chart-4", "--chart-1", "--chart-5", "--chart-3"];

function fmtAxis(n: number): string {
  if (n >= 1_000_000) return n / 1_000_000 + "M";
  if (n >= 1000) return n / 1000 + "k";
  return String(n);
}

export function ScalingCharts({ families }: { families: ScalingFamily[] }) {
  const [open, setOpen] = React.useState(false);
  if (families.length === 0) return null;

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
        Scaling ({families.length} size-parametrised metric
        {families.length === 1 ? "" : "s"})
      </button>

      {open && (
        <>
          <p className="text-xs text-muted-foreground">
            Latest value vs dataset size/depth, derived from the size token in each metric
            name (smaller-is-better; indicative CI-runner numbers).
          </p>
          <div className="grid gap-3 sm:grid-cols-2">
            {families.map((f, i) => {
              const points: ChartPoint[] = f.points.map((p) => ({
                x: p.axis,
                y: p.value,
                label: fmtAxis(p.axis),
              }));
              return (
                <Card key={f.base} className="gap-1 p-3">
                  <div className="flex items-baseline justify-between gap-2">
                    <span className="truncate text-sm font-medium" title={f.label}>
                      {f.label}
                    </span>
                    <span className="shrink-0 text-xs text-muted-foreground">{f.unit}</span>
                  </div>
                  <code className="text-[11px] text-muted-foreground">{f.base}</code>
                  <LineChart
                    points={points}
                    unit={f.unit}
                    colorVar={CHART_VARS[i % CHART_VARS.length]}
                    xTickFormat={fmtAxis}
                    ariaLabel={`Scaling of ${f.label} vs ${f.axisLabel}`}
                  />
                  <p className="text-[11px] text-muted-foreground">
                    x: {f.axisLabel} ·{" "}
                    {f.points.map((p) => `${fmtAxis(p.axis)}→${fmtNum(p.value)}`).join(", ")}
                    {f.points.length === 1 && " (single point — no scaling curve yet)"}
                  </p>
                </Card>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
