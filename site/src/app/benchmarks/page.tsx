// [OPUS-4.8] sq-vjn4 — /benchmarks overview: a card per benchmark TYPE (capability
// family) with its metric count + headline, linking to the per-type page. Honest framing
// of provenance up top.
import type { Metadata } from "next";
import Link from "next/link";
import { ArrowRight, FlaskConical, GitCommitHorizontal } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  FAMILIES,
  LATEST,
  SOURCE,
  familyMetricCount,
} from "@/data/benchmarks";

export const metadata: Metadata = {
  title: "Benchmarks",
  description:
    "sparq's benchmark results — per-commit query/validation/reasoning latencies, with live-computed competitive summaries where real same-box competitor numbers exist, and honest placeholders where they do not.",
};

export default function BenchmarksOverviewPage() {
  const commit = LATEST.commit ? LATEST.commit.slice(0, 8) : "unknown";
  const date = LATEST.date ? new Date(LATEST.date).toLocaleDateString() : "—";

  return (
    <div className="space-y-8">
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <FlaskConical className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">Benchmarks</h1>
            <p className="text-sm text-muted-foreground">
              Real measurements, per benchmark type — honestly labelled where a competitor
              baseline does not yet exist.
            </p>
          </div>
        </div>
        {/* [FABLE-5] sq-mcsbp — this provenance strip (commit sha, report date, refresh mode)
            is data-driven from the benchmark-data branch, so it churns every time the committed
            benchmarks snapshot updates and was the recurring cause of the benchmarks-index visual
            baseline drift. Mask it at capture ("mask, don't chase") so the baseline survives data
            churn — the surrounding static card layout is what the snapshot actually guards. */}
        <div
          data-vr-mask
          className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground"
        >
          <Badge variant="muted" className="gap-1">
            <GitCommitHorizontal aria-hidden />
            commit {commit}
          </Badge>
          <span>·</span>
          <span>latest reported {date}</span>
          <span>·</span>
          <span>source: {SOURCE}</span>
        </div>
      </header>

      <section className="measure space-y-3 text-sm text-muted-foreground">
        <p>
          Each card is a <strong className="text-foreground">benchmark type</strong>. The
          per-commit series (every metric{" "}
          <strong className="text-foreground">smaller is better</strong>) is recorded by
          the CI benchmark workflow on a GitHub-hosted runner. Where a real{" "}
          <strong className="text-foreground">same-box</strong> competitor measurement
          exists, the type page shows a live-computed speedup vs the next-best engine;
          where it does not, it says so plainly and shows sparq&rsquo;s absolute numbers.
          Cross-machine figures are kept separate and clearly labelled — never folded into
          a speedup.
        </p>
      </section>

      <section className="grid gap-4 sm:grid-cols-2">
        {FAMILIES.map((f) => {
          const count = familyMetricCount(f.key);
          return (
            <Link key={f.key} href={`/benchmarks/${f.key}`} className="group">
              <Card className="h-full transition-colors group-hover:ring-foreground/20">
                <CardHeader>
                  <div className="flex items-center justify-between gap-2">
                    <CardTitle className="text-base">{f.title}</CardTitle>
                    {count === 0 ? (
                      <Badge variant="muted">soon</Badge>
                    ) : (
                      <Badge variant="muted">
                        {count} metric{count === 1 ? "" : "s"}
                      </Badge>
                    )}
                  </div>
                </CardHeader>
                <CardContent>
                  <CardDescription className="leading-relaxed">
                    {f.blurb}
                  </CardDescription>
                  <span className="mt-3 inline-flex items-center gap-1 text-sm text-primary">
                    View benchmarks
                    <ArrowRight
                      className="size-3.5 transition-transform group-hover:translate-x-0.5"
                      aria-hidden
                    />
                  </span>
                </CardContent>
              </Card>
            </Link>
          );
        })}
      </section>
    </div>
  );
}
