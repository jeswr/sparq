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
import {
  METRIC_BADGE_WIDEST_LABEL,
  metricBadgeLabel,
} from "@/lib/metric-badge";

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
                    {/* [SONNET-4.6] sq-hfd82 — data-vr-mask: the per-family metric COUNT is
                        derived from the committed benchmarks snapshot, which the benchmark
                        workflow refreshes continuously (the metric total grew by more than half
                        again within three days of the last baseline re-mint). sq-mcsbp masked the
                        provenance strip above for exactly this reason but left these badges
                        unmasked, so benchmarks-index.png went stale again within days — the
                        fastest-recurring cause of the nightly visual red. Mask it ("mask, don't
                        chase"); the card grid layout is what the snapshot actually guards.

                        Masking alone is not enough: a Playwright mask tracks the element's LIVE
                        bounding box, so a pill that resizes on a digit or singular/plural change
                        moves the pixels around it. The invisible ghost below reserves the widest
                        label the pill can ever show (see @/lib/metric-badge), and the live label
                        is centred over it — so the pill's geometry, and the mask over it, are
                        identical for every count. */}
                    <Badge
                      variant="muted"
                      className="relative tabular-nums"
                      data-vr-mask
                    >
                      <span aria-hidden className="invisible">
                        {METRIC_BADGE_WIDEST_LABEL}
                      </span>
                      <span className="absolute inset-0 flex items-center justify-center">
                        {metricBadgeLabel(count)}
                      </span>
                    </Badge>
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
