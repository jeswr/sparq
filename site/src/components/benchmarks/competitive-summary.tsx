// [OPUS-4.8] sq-vjn4 — the LIVE-COMPUTED competitive summary badge + the honest variants.
// The speedup band is computed at render time from REAL same-box competitor numbers only
// (see src/data/benchmarks.ts competitiveSummary). Pending / sparq-only states render an
// explicit honest note — never a fabricated ratio.
import { Gauge, Hourglass, Minus, Scale } from "lucide-react";

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
  // [OPUS-4.8] sq-1sa9r — honest canonical same-box badge: report the count-checked
  // win-rate, NOT a bald multiplier (the engines use different measurement modes).
  if (summary.kind === "same-box") {
    return (
      <Badge
        variant={summary.wins === summary.total ? "success" : "muted"}
        className="gap-1"
        title={
          "Canonical" +
          (summary.canonical ? " " : " (non-canonical) ") +
          "same-box gather; absolute cross-engine ratios include measurement-mode overhead — see detail."
        }
      >
        <Scale aria-hidden />
        {summary.canonical ? "canonical " : ""}same-box · fastest on {summary.wins}/
        {summary.total}
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
  if (summary.kind === "same-box") {
    return (
      <div className="space-y-2 text-sm text-muted-foreground">
        <p>
          {summary.canonical ? (
            <strong className="text-foreground">Canonical</strong>
          ) : (
            "Non-canonical"
          )}{" "}
          same-box comparison on{" "}
          <span className="text-foreground">{summary.host}</span> at{" "}
          <span className="text-foreground">{summary.scale}</span> (git{" "}
          <code className="text-[11px]">{summary.gitCommit}</code>). Across{" "}
          <strong className="text-foreground">{summary.total}</strong> quer
          {summary.total === 1 ? "y" : "ies"} whose solution{" "}
          <em>count cross-checked</em> across engines, sparq returned the correct count on
          every one and was the fastest engine on{" "}
          <strong className="text-foreground">{summary.wins}</strong>
          {summary.losses > 0 ? (
            <>
              {" "}
              (a competitor was faster on{" "}
              <strong className="text-foreground">{summary.losses}</strong>)
            </>
          ) : null}
          . The fastest competitor ranged{" "}
          <strong className="text-foreground">
            {summary.min}×–{summary.max}×
          </strong>{" "}
          sparq&rsquo;s time (median{" "}
          <strong className="text-foreground">{summary.median}×</strong>).
        </p>
        <p className="italic">
          Measurement modes differ — sparq
          {summary.cliCompetitors.length > 0
            ? ` and ${summary.cliCompetitors.join(" / ")}`
            : ""}{" "}
          run in-process via the CLI, while {summary.httpEngines.join(" / ")} answer over an
          HTTP SPARQL adapter — so on small result sets these absolute cross-engine ratios
          reflect per-query harness/transport overhead as much as engine speed; read them
          for cross-engine ordering, not as an audited head-to-head.
          {summary.failedEngines.length > 0 && (
            <>
              {" "}
              {summary.failedEngines.join(", ")} failed to load (shown as{" "}
              <span className="not-italic">failed</span>, never blank).
            </>
          )}
          {summary.diffQueries.length > 0 && (
            <>
              {" "}
              {summary.diffQueries.join(", ")} {summary.diffQueries.length === 1 ? "was" : "were"}{" "}
              excluded from the ratio because engines disagreed on the solution count.
            </>
          )}
        </p>
      </div>
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
