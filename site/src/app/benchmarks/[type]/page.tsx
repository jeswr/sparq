// [OPUS-4.8] sq-vjn4 — a per-TYPE benchmark page. Renders the type's suites as
// collapsible groups; each collapsed header carries a LIVE-COMPUTED competitive summary
// (speedup band vs next-best, OR an honest "competitor baseline pending" / "sparq numbers
// only"). Expanding shows the metric table, any same-box cross-engine table, and any
// cross-machine reference baselines. The SPARQL type is the big one (LUBM / WatDiv /
// SP2Bench / BSBM / Operators / Synthetic). Static-exported via generateStaticParams.
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";

import { Button } from "@/components/ui/button";
import { SuiteGroup } from "@/components/benchmarks/suite-group";
import { ZkCircuitCostBreakdown } from "@/components/benchmarks/zk-circuit-costs";
import {
  FAMILIES,
  LATEST,
  fullSuiteGroupsForFamily,
} from "@/data/benchmarks";

export function generateStaticParams() {
  return FAMILIES.map((f) => ({ type: f.key }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ type: string }>;
}): Promise<Metadata> {
  const { type } = await params;
  const f = FAMILIES.find((x) => x.key === type);
  return {
    title: f ? `${f.title} benchmarks` : "Benchmarks",
    description: f?.blurb,
  };
}

export default async function BenchmarkTypePage({
  params,
}: {
  params: Promise<{ type: string }>;
}) {
  const { type } = await params;
  const family = FAMILIES.find((f) => f.key === type);
  if (!family) notFound();

  const groups = fullSuiteGroupsForFamily(family.key);
  const commit = LATEST.commit ? LATEST.commit.slice(0, 8) : "unknown";

  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/benchmarks">
          <ArrowLeft className="size-4" aria-hidden />
          All benchmark types
        </Link>
      </Button>

      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">{family.title}</h1>
        <p className="measure text-sm text-muted-foreground">{family.blurb}</p>
        <p className="text-xs text-muted-foreground">
          Latest commit {commit} · every metric smaller-is-better · each group header shows
          a competitive summary computed live from real same-box competitor numbers (or an
          honest placeholder where none exist yet).
        </p>
      </header>

      {/* [OPUS-4.8] sq-1s2 — the ZK family carries a deterministic circuit gate/time cost
          breakdown sourced from src/data/zk-circuit-costs.json, ABOVE the CI-feed groups
          (which are empty until a ci-bench hook emits zk_* metrics). It is the snapshot of
          record for the family's per-member gate counts, independent of the CI feed. */}
      {family.key === "zk" && <ZkCircuitCostBreakdown />}

      {groups.length === 0 ? (
        <div className="rounded-lg bg-muted/40 p-6 text-sm text-muted-foreground">
          {family.key === "zk" ? (
            <>
              No CI-feed timing metrics are reported for this benchmark type yet (the
              gate/time breakdown above is the committed snapshot). The CI benchmark
              workflow will add live <code className="font-mono">zk_*</code> metrics as
              soon as it emits — kept visible so coverage is honest, never fabricated.
            </>
          ) : (
            <>
              No metrics are reported for this benchmark type yet. The CI benchmark workflow
              will populate it as soon as it emits — this section is kept visible so coverage
              is honest, never fabricated.
            </>
          )}
        </div>
      ) : (
        <div className="space-y-3">
          {groups.map((g, i) => (
            <SuiteGroup key={g.suite} data={g} defaultOpen={i === 0} />
          ))}
        </div>
      )}
    </div>
  );
}
