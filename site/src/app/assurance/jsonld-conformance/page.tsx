// [GPT-5.6] sq-ztdez — static public scoreboard for the six JSON-LD lanes.
import type { Metadata } from "next";
import { CheckCircle2, FlaskConical } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { jsonLdConformanceLanes } from "@/data/jsonld-conformance";

export const metadata: Metadata = {
  title: "JSON-LD 1.1 conformance floors",
  description: "Measured, rise-only W3C JSON-LD 1.1 conformance floors for sparq.",
};

export default function JsonLdConformancePage() {
  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <FlaskConical className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">JSON-LD 1.1 conformance floors</h1>
            <p className="text-sm text-muted-foreground">
              Six independently ratcheted lanes against pinned W3C test suites.
            </p>
          </div>
        </div>
        <Badge variant="outline">W3C JSON-LD 1.1</Badge>
      </header>

      <p className="max-w-3xl rounded-xl border border-amber-500/30 bg-amber-500/5 p-4 text-sm text-muted-foreground">
        <strong className="text-foreground">Measured floors, not conformance claims.</strong>{" "}
        Each pass/total value is a rise-only measured minimum at the pinned suite revision.
        A lane below its total is not claimed conformant; uncounted cases remain documented
        failures or skips in the test harness.
      </p>

      <section aria-labelledby="lanes-heading" className="space-y-4">
        <h2 id="lanes-heading" className="text-lg font-semibold">
          Ratcheted lanes
        </h2>
        <ul className="grid gap-3 sm:grid-cols-2">
          {jsonLdConformanceLanes.map((lane) => {
            const complete = lane.floor === lane.total;
            return (
              <li key={lane.id} className="rounded-xl border bg-card p-4">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <h3 className="font-semibold">{lane.label}</h3>
                    <code className="text-xs text-muted-foreground">{lane.id}</code>
                  </div>
                  <div className="text-right">
                    <p className="text-xl font-semibold tabular-nums" aria-label={`${lane.floor} passes out of ${lane.total}`}>
                      {lane.floor}/{lane.total}
                    </p>
                    <p className="text-xs text-muted-foreground">measured floor</p>
                  </div>
                </div>
                {complete ? (
                  <p className="mt-3 flex items-center gap-1.5 text-xs text-muted-foreground">
                    <CheckCircle2 className="size-3.5 text-[var(--success)]" aria-hidden />
                    Full score at the pinned revision
                  </p>
                ) : (
                  <p className="mt-3 text-xs text-muted-foreground">Not claimed conformant</p>
                )}
              </li>
            );
          })}
        </ul>
      </section>
    </div>
  );
}

