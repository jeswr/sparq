"use client";

// [FABLE-5] sq-vw3ax.13 — the federation capability walkthrough on /capabilities.
// Captured-output tier: the three federation crates (sparq-engine-service's SERVICE
// transport + SSRF egress allowlist, sparq-fedclient's streaming SPARQL/brTPF/TPF
// consumer, sparq-fedplan's cost-based planner) are opt-in NATIVE crates — the wasm
// bundles carry zero federation code and the static site has no backend — so this
// replays REAL sparq-fedplan planner output (selection + three plans), captured
// verbatim by crates/sparq-fedplan/examples/capture_surface_federation.rs over the
// declared fixture shown on the page. All data lives in src/lib/federation.ts
// (framework-free, unit-tested); the numbers are the planner's abstract estimates
// over declared statistics, not benchmark results.

import * as React from "react";
import { Database, Filter, ListTree, Play, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  FED_QUERY,
  PATTERNS,
  SCENARIOS,
  SELECTION,
  SOURCES,
  patternPredicate,
  pruneReason,
  type JoinAlgo,
  type PlanScenario,
} from "@/lib/federation";

const fmt = (n: number) => n.toLocaleString("en-US");

const ALGO_STYLE: Record<JoinAlgo, string> = {
  Bind: "bg-primary/10 text-primary ring-primary/30",
  Hash: "bg-[var(--warning)]/10 text-[var(--warning)] ring-[var(--warning)]/30",
  Streaming: "bg-[var(--success)]/10 text-[var(--success)] ring-[var(--success)]/30",
};

function AlgoChip({ algo }: { algo: JoinAlgo }) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 font-mono text-[11px] font-semibold ring-1",
        ALGO_STYLE[algo],
      )}
    >
      {algo}
    </span>
  );
}

/** One endpoint's declared descriptor — everything the planner knows about it. */
function SourceCard({ index }: { index: number }) {
  const s = SOURCES[index];
  return (
    <Card>
      <CardHeader className="flex-row items-center gap-2 space-y-0 pb-2">
        <Database className="size-4 shrink-0 text-primary" aria-hidden="true" />
        <CardTitle className="min-w-0 text-sm">
          <span className="font-mono">{s.label}</span>
          <span className="ml-2 break-all font-mono text-[11px] font-normal text-muted-foreground">
            {s.endpoint}
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-1 font-mono text-[11.5px]">
        <div className="text-muted-foreground">
          void:triples {fmt(s.totalTriples)}
        </div>
        {s.partitions.map((p) => (
          <div key={p.predicate} className="flex flex-wrap gap-x-2">
            <span className="text-foreground">{p.predicate}</span>
            <span className="text-muted-foreground">
              {fmt(p.triples)} triples · {fmt(p.distinctSubjects)} subj ·{" "}
              {fmt(p.distinctObjects)} obj
            </span>
          </div>
        ))}
        {s.charSets.map((c) => (
          <div key={c.predicates.join()} className="text-muted-foreground">
            cs:{"{"}
            {c.predicates.join(", ")}
            {"}"} — {fmt(c.subjects)} subjects × mult [{c.avgMultiplicity.join(", ")}]
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

/** Phase 1 — per-pattern recall-safe selection: retained + estimates, pruned + why. */
function SelectionTable() {
  return (
    <div className="overflow-x-auto rounded-lg border">
      <table className="w-full border-collapse text-[12px]">
        <thead>
          <tr className="border-b bg-muted/60 text-left">
            <th className="px-3 py-1.5 font-semibold">triple pattern</th>
            <th className="px-3 py-1.5 font-semibold">retained (est. rows)</th>
            <th className="px-3 py-1.5 font-semibold">pruned — provably empty</th>
          </tr>
        </thead>
        <tbody className="font-mono">
          {SELECTION.map((sel) => (
            <tr key={sel.pattern} className="border-b align-top last:border-0">
              <td className="px-3 py-1.5 whitespace-nowrap">{PATTERNS[sel.pattern]}</td>
              <td className="px-3 py-1.5">
                {sel.retained.map((c) => (
                  <div key={c.source} className="whitespace-nowrap">
                    {SOURCES[c.source].label}{" "}
                    <span className="text-muted-foreground">
                      ~{fmt(c.estimatedCardinality)}
                    </span>
                  </div>
                ))}
              </td>
              <td className="px-3 py-1.5 text-muted-foreground">
                {sel.pruned.map((s) => (
                  <div key={s} title={pruneReason(sel.pattern, s)}>
                    <s>{SOURCES[s].label}</s>{" "}
                    <span className="font-sans text-[11px]">
                      — no {patternPredicate(sel.pattern)} partition
                    </span>
                  </div>
                ))}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Phase 2 — the planned left-deep join tree for one captured scenario. */
function PlanView({ scenario }: { scenario: PlanScenario }) {
  const first = SELECTION.find((s) => s.pattern === scenario.joinOrder[0]);
  return (
    <div className="space-y-3">
      <div className="font-mono text-[11.5px] text-muted-foreground">
        {scenario.options}
      </div>
      <ol className="space-y-2">
        <li className="flex flex-wrap items-center gap-2 text-[12px]">
          <span className="w-14 shrink-0 text-right font-mono text-muted-foreground">
            start
          </span>
          <span className="font-mono">{PATTERNS[scenario.joinOrder[0]]}</span>
          <span className="text-muted-foreground">
            — most selective leaf, ~{fmt(first?.total ?? 0)} rows
          </span>
        </li>
        {scenario.steps.map((st, i) => (
          <li key={st.right} className="flex flex-wrap items-center gap-2 text-[12px]">
            <span className="w-14 shrink-0 text-right font-mono text-muted-foreground">
              join {i + 1}
            </span>
            <AlgoChip algo={st.algo} />
            <span className="font-mono">{PATTERNS[st.right]}</span>
            <span className="text-muted-foreground">
              → ~{fmt(st.cardinality)} rows · cost {fmt(st.cost)}
            </span>
          </li>
        ))}
      </ol>
      <p className="text-xs text-muted-foreground">
        Total estimated cost <span className="font-mono">{fmt(scenario.totalCost)}</span>{" "}
        (abstract planner units) · estimated result ~
        <span className="font-mono">{fmt(scenario.rootCardinality)}</span> rows.{" "}
        {scenario.caption}
      </p>
    </div>
  );
}

export function FederationWalkthrough() {
  const [selected, setSelected] = React.useState(0);
  const [planned, setPlanned] = React.useState(false);
  const scenario = SCENARIOS[selected];

  return (
    <div className="space-y-8">
      {/* ── The fixture: what the planner sees ──────────────────────────────────── */}
      <section className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Database className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">Three endpoints, one query</h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real captured planner output
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          <code className="font-mono">sparq-fedplan</code> plans over{" "}
          <strong className="text-foreground">served descriptors</strong> — VoID property
          partitions plus mined characteristic sets — never the network. Everything it
          knows about the three endpoints is declared below; the selection and every plan
          are the real planner&rsquo;s verbatim output over exactly this input.
        </p>
        <div className="grid gap-3 lg:grid-cols-3">
          {SOURCES.map((_, i) => (
            <SourceCard key={i} index={i} />
          ))}
        </div>
        <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
          {FED_QUERY}
        </pre>
      </section>

      {/* ── Phase 1: recall-safe source selection ───────────────────────────────── */}
      <section className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Filter className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">1 · Source selection — prune, never lose</h2>
        </div>
        <p className="measure text-sm text-muted-foreground">
          HiBISCuS-style pruning is <strong className="text-foreground">recall-safe</strong>:
          an endpoint is dropped for a pattern only when its descriptor{" "}
          <em>proves</em> it holds no matching triple (here: its complete predicate set
          lacks the pattern&rsquo;s predicate). The <span className="font-mono">geo</span>{" "}
          endpoint is never contacted; the name pattern honestly fans out to{" "}
          <em>both</em> endpoints that hold <span className="font-mono">foaf:name</span>.
          Per-source row estimates are CostFed-style, from the declared statistics.
        </p>
        <SelectionTable />
      </section>

      {/* ── Phase 2: cost-based join planning ───────────────────────────────────── */}
      <section className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <ListTree className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">2 · Join planning — bind vs hash vs stream</h2>
        </div>
        <p className="measure text-sm text-muted-foreground">
          The planner joins greedily from the most selective pattern and prices each join
          two ways: a <strong className="text-foreground">bind join</strong> (one bound
          SERVICE/VALUES request per row) vs a{" "}
          <strong className="text-foreground">hash join</strong> (scan both sides once,
          join locally). Turn the real <code className="font-mono">PlanOptions</code> knobs
          and watch the choice flip — same statistics, different cost model:
        </p>
        <div className="flex flex-wrap gap-2">
          {SCENARIOS.map((sc, i) => (
            <button
              key={sc.id}
              type="button"
              onClick={() => {
                setSelected(i);
                setPlanned(false);
              }}
              className={cn(
                "rounded-full px-3 py-1.5 text-left text-[12.5px] ring-1 transition-colors",
                i === selected
                  ? "bg-primary/10 text-primary ring-primary/30"
                  : "bg-muted/40 text-muted-foreground ring-foreground/10 hover:bg-muted",
              )}
            >
              {sc.label}
            </button>
          ))}
        </div>
        <Card>
          <CardHeader className="space-y-0">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <CardTitle className="text-base">{scenario.label}</CardTitle>
              <Button size="sm" onClick={() => setPlanned(true)} disabled={planned}>
                <Play className="size-4" aria-hidden="true" />
                {planned ? "Planned" : "Plan"}
              </Button>
            </div>
          </CardHeader>
          {planned && (
            <CardContent>
              <PlanView scenario={scenario} />
            </CardContent>
          )}
        </Card>
      </section>

      {/* ── The rest of the estate, honestly scoped ─────────────────────────────── */}
      <section className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <ShieldCheck className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">Execution — native, deny-by-default</h2>
        </div>
        <p className="measure text-sm text-muted-foreground">
          Executing the plan is the native crates&rsquo; job, shown here as a walkthrough
          because none of it ships in the wasm bundles:{" "}
          <code className="font-mono">sparq-engine-service</code> runs SPARQL 1.1{" "}
          <code className="font-mono">SERVICE</code> over a streaming results parser behind
          a <strong className="text-foreground">deny-by-default SSRF egress allowlist</strong>{" "}
          (enforced on the resolved IP, DNS-rebinding-safe), and{" "}
          <code className="font-mono">sparq-fedclient</code> consumes heterogeneous sources
          — full SPARQL endpoints, brTPF, TPF — with FedX exclusive groups and
          filter push-down, streaming results through non-blocking operators.
        </p>
      </section>
    </div>
  );
}
