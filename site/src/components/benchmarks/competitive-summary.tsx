// [OPUS-4.8] sq-vjn4 — the LIVE-COMPUTED competitive summary badge + the honest variants.
// The speedup band is computed at render time from REAL same-box competitor numbers only
// (see src/data/benchmarks.ts competitiveSummary). Pending / sparq-only states render an
// explicit honest note — never a fabricated ratio.
import { Gauge, Hourglass, Minus } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import type { CompetitiveSummary } from "@/data/benchmarks";

export function SummaryPill({ summary }: { summary: CompetitiveSummary }) {
  if (summary.kind === "speedup") {
    const band =
      summary.min === summary.max
        ? `${summary.min}×`
        : `${summary.min}×–${summary.max}×`;
    return (
      <Badge variant="success" className="gap-1">
        <Gauge aria-hidden />
        {band} faster than next-best
      </Badge>
    );
  }
  if (summary.kind === "pending") {
    return (
      <Badge variant="warning" className="gap-1">
        <Hourglass aria-hidden />
        competitor baseline pending
      </Badge>
    );
  }
  return (
    <Badge variant="muted" className="gap-1">
      <Minus aria-hidden />
      sparq numbers only
    </Badge>
  );
}

// A fuller explanation shown inside an expanded group — states the computation + honesty.
export function SummaryDetail({ suite, summary }: { suite: string; summary: CompetitiveSummary }) {
  if (summary.kind === "speedup") {
    return (
      <p className="text-sm text-muted-foreground">
        Across <strong className="text-foreground">{summary.n}</strong> benchmark
        {summary.n === 1 ? "" : "s"} in {suite}, sparq is{" "}
        <strong className="text-foreground">
          {summary.min}×–{summary.max}×
        </strong>{" "}
        faster than the next-best of {summary.engines.join(" / ")} (median{" "}
        <strong className="text-foreground">{summary.median}×</strong>), computed at
        render time as the per-benchmark ratio of the fastest competitor over sparq.{" "}
        {summary.nonCanonical && (
          <span className="italic">
            Measured {summary.provenance}; absolute timings across host classes are not
            directly comparable.
          </span>
        )}
      </p>
    );
  }
  if (summary.kind === "pending") {
    return (
      <p className="text-sm text-muted-foreground">
        {summary.reason}
        {summary.provenance ? ` (${summary.provenance})` : ""} sparq&rsquo;s absolute
        numbers are shown below.
      </p>
    );
  }
  return <p className="text-sm text-muted-foreground">{summary.reason}</p>;
}
