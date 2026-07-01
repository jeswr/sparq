"use client";

// [OPUS-4.8] sq-tp1m (#757) — the INFERENCE tool: a real, working panel (no longer a stub) for
// the per-workspace entailment regime. It drives the SAME engine + workspace state as the
// compact toolbar selector (InferenceControl), and additionally explains the active regime and
// shows the MEASURED base / closure / entailed triple counts the engine's forward-chaining
// reasoner (sparq-reason, via the tier-b W-reason wasm bundle) produced over the live store.
//
// HONESTY: reasoning is REAL (not a captured walkthrough), but it is a QUERY-TIME regime — it
// never mutates the persisted store, and named graphs are folded into the default graph for
// entailment. RDFS + OWL 2 RL are wired; N3 rule reasoning is deferred (it needs a rules-
// authoring surface + a store that can hold rule/formula terms — see the caveat below).

import * as React from "react";
import { Brain, Info } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { useEngine } from "@/lib/engine-context";
import { useWorkspace } from "@/lib/workspace-context";
import { INFERENCE_MODE_META } from "@/lib/reason-wasm";
import { toolById, TIER_META } from "@/data/tools";
import { InferenceControl } from "@/components/workbench/inference-control";

/** One labelled measured count in the stats strip. */
function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex flex-col rounded-md border bg-card px-3 py-2">
      <span className="tabular text-lg font-semibold">{value.toLocaleString()}</span>
      <span className="text-[11px] uppercase tracking-wide text-muted-foreground">{label}</span>
    </div>
  );
}

export function InferenceTool() {
  const { inferenceStatus, status } = useEngine();
  const { inference } = useWorkspace();
  const tool = toolById("inference");
  const tier = tool ? TIER_META[tool.tier] : null;
  const modeMeta = INFERENCE_MODE_META[inference];

  return (
    <div className="h-full overflow-auto">
      <div className="mx-auto max-w-2xl space-y-5 p-6">
        {/* Header. */}
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <Brain className="size-5 text-primary" />
            <h2 className="text-lg font-semibold">Inference</h2>
            {tier ? (
              <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <span className={`size-2 rounded-full ${tier.dot}`} aria-hidden />
                {tier.label}
              </span>
            ) : null}
          </div>
          <p className="text-sm text-muted-foreground">
            Apply an RDFS / OWL 2 RL entailment regime to your queries over the live store. The
            engine forward-chains the deductive closure with the real{" "}
            <code className="rounded bg-muted px-1 py-0.5 text-[11px]">sparq-reason</code> reasoner,
            so a query can match an <em>entailed</em> triple, not just an asserted one. The choice
            is saved per workspace.
          </p>
        </div>

        {/* The selector (shared with the query toolbar). */}
        <div className="rounded-lg border bg-background p-4">
          <InferenceControl />
          <p className="mt-3 text-xs text-muted-foreground">{modeMeta.blurb}</p>
        </div>

        {/* Live measured stats when a regime is active + ready. */}
        {inferenceStatus.kind === "ready" ? (
          <div className="space-y-2" data-inference-stats>
            <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Closure over the live store
            </h3>
            <div className="grid grid-cols-3 gap-2">
              <Stat label="Asserted" value={inferenceStatus.info.baseTriples} />
              <Stat label="After reasoning" value={inferenceStatus.info.closureTriples} />
              <Stat label="Entailed" value={inferenceStatus.info.entailed} />
            </div>
            <p className="text-[11px] text-muted-foreground">
              Measured counts (distinct triples; named graphs folded into the default graph for
              entailment). Reasoning runs at query time and does not change the stored data.
            </p>
          </div>
        ) : inferenceStatus.kind === "loading" ? (
          <p className="text-sm text-muted-foreground">Materialising the closure…</p>
        ) : inferenceStatus.kind === "error" ? (
          <div className="rounded-md border border-[var(--warning)]/40 bg-[var(--warning)]/5 p-3 text-xs text-muted-foreground">
            The reasoning bundle could not run: {inferenceStatus.message} It is a separate wasm
            bundle that must be synced into this build — rebuild it with{" "}
            <code className="rounded bg-muted px-1 py-0.5">npm run build:reason-wasm</code> in{" "}
            <code className="rounded bg-muted px-1 py-0.5">js/</code>. Until then, queries run over
            the asserted data.
          </div>
        ) : status.kind !== "ready" ? (
          <p className="text-sm text-muted-foreground">Waiting for the engine to warm…</p>
        ) : (
          <p className="text-sm text-muted-foreground">
            Inference is off — queries run over the asserted data. Pick RDFS or OWL 2 RL above to
            reason over the live store.
          </p>
        )}

        {/* Honest caveats. */}
        <div className="space-y-1.5 rounded-md border bg-muted/30 p-3 text-xs text-muted-foreground">
          <div className="flex items-center gap-1.5 font-medium text-foreground">
            <Info className="size-3.5" /> How this works
          </div>
          <ul className="list-disc space-y-1 pl-5">
            <li>
              Real forward-chaining: OWL 2 RL includes RDFS. The closure is (re)materialised when
              you change data (import / update) or switch regime.
            </li>
            <li>
              Query-time only: reasoning never mutates the persisted store — turning inference off
              returns you to the asserted data exactly.
            </li>
            <li>
              N3 rule reasoning is not offered as a store-wide regime: it needs an N3 rules-
              authoring surface and a store that can carry rule / formula terms, which the in-tab
              ground-triple store cannot represent (tracked as a follow-up).
            </li>
          </ul>
          <div className="pt-1">
            <Badge variant="outline">sq-tp1m</Badge>
          </div>
        </div>
      </div>
    </div>
  );
}
