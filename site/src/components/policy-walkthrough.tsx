"use client";

// [OPUS-4.8] sq-vw3ax.14 — the ODRL usage-control capability walkthrough on /capabilities.
//
// Captured-output tier: `sparq-policy` is an OPT-IN native crate the lean wasm bundles never
// carry, and the static GitHub-Pages site has no backend — so this REPLAYS the real
// `sparq_policy::evaluate` decision for each (policy, request) pair rather than running the
// evaluator in your tab. Every verdict is pinned by a NAMED crate test (see the HONESTY
// CONTRACT + per-variant `test` provenance in src/lib/policy.ts, which is framework-free and
// unit-tested). Pick a policy, then send different requests at it and watch the fail-closed
// ALLOW / DENY decision — with the matched rule(s) and the unmet-constraint explanation —
// change. Single-node ODRL; the federated ODRL→MPC composition is deferred (no ZK/MPC claim).

import * as React from "react";
import {
  CheckCircle2,
  FileLock2,
  ListChecks,
  ShieldX,
  XCircle,
} from "lucide-react";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  SCENARIOS,
  requestLine,
  type PolicyScenario,
  type RequestVariant,
} from "@/lib/policy";

const REPO = "https://github.com/sparq-org/sparq";

/** The ALLOW / DENY verdict pill — the one thing a visitor scans for first. */
function VerdictBadge({ allow }: { allow: boolean }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-[13px] font-semibold ring-1",
        allow
          ? "bg-[color-mix(in_oklch,var(--success)_15%,transparent)] text-[var(--success-on-tint)] ring-[var(--success)]/30"
          : "bg-destructive/10 text-destructive ring-destructive/30",
      )}
    >
      {allow ? (
        <CheckCircle2 className="size-4" aria-hidden />
      ) : (
        <ShieldX className="size-4" aria-hidden />
      )}
      {allow ? "PERMIT" : "DENY"}
    </span>
  );
}

/** The evaluated request + its decision — the right-hand result column. */
function DecisionPanel({ variant }: { variant: RequestVariant }) {
  const { decision } = variant;
  return (
    <div className="space-y-3">
      <div>
        <div className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
          The request
        </div>
        <code className="block overflow-x-auto rounded-md bg-muted px-2.5 py-2 font-mono text-[11.5px] leading-relaxed text-foreground">
          {requestLine(variant)}
        </code>
        <p className="mt-1.5 text-[12.5px] text-muted-foreground">{variant.note}</p>
      </div>

      <div className="flex items-center gap-2 border-t pt-3">
        <span className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
          Decision
        </span>
        <VerdictBadge allow={decision.allow} />
      </div>

      {decision.matched.length > 0 && (
        <div>
          <div className="mb-1 flex items-center gap-1.5 text-[12px] font-semibold text-foreground/90">
            <ListChecks className="size-3.5 text-primary" aria-hidden />
            Matched rule{decision.matched.length > 1 ? "s" : ""}
          </div>
          <ul className="space-y-1">
            {decision.matched.map((m) => (
              <li
                key={m}
                className="rounded bg-primary/5 px-2 py-1 font-mono text-[11.5px] text-foreground"
              >
                {m}
              </li>
            ))}
          </ul>
        </div>
      )}

      {decision.unmet.length > 0 && (
        <div>
          <div className="mb-1 flex items-center gap-1.5 text-[12px] font-semibold text-foreground/90">
            <XCircle className="size-3.5 text-destructive" aria-hidden />
            Why it did not grant
          </div>
          <ul className="space-y-1">
            {decision.unmet.map((u) => (
              <li
                key={u}
                className="rounded bg-destructive/5 px-2 py-1 font-mono text-[11px] leading-snug text-foreground/90"
              >
                {u}
              </li>
            ))}
          </ul>
        </div>
      )}

      <p className="border-t pt-2 text-[11px] text-muted-foreground">
        Real{" "}
        <code className="font-mono text-[10.5px]">evaluate</code> output, pinned by{" "}
        <code className="font-mono text-[10.5px]">{variant.test}</code>.
      </p>
    </div>
  );
}

/** One policy scenario: the ODRL Turtle on the left, a request selector + decision on the right. */
function ScenarioCard({ scenario }: { scenario: PolicyScenario }) {
  const [active, setActive] = React.useState(0);
  const variant = scenario.variants[active];

  return (
    <Card>
      <CardHeader className="space-y-2 pb-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="flex items-center gap-2 text-base">
            <FileLock2 className="size-4 shrink-0 text-primary" aria-hidden />
            {scenario.title}
          </CardTitle>
          <Badge variant="muted" className="font-mono text-[10.5px]">
            {scenario.feature}
          </Badge>
        </div>
        <p className="text-[13px] leading-snug text-muted-foreground">{scenario.summary}</p>
      </CardHeader>
      <CardContent className="grid gap-4 lg:grid-cols-2">
        {/* The policy. */}
        <div>
          <div className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
            The ODRL 2.2 policy
          </div>
          <pre className="overflow-x-auto rounded-md border bg-muted/40 p-3 font-mono text-[11px] leading-relaxed text-foreground">
            {scenario.turtle}
          </pre>
        </div>

        {/* The request selector + the decision. */}
        <div className="space-y-3">
          <div>
            <div className="mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
              Send a request
            </div>
            <div
              role="tablist"
              aria-label={`Requests for ${scenario.title}`}
              className="flex flex-wrap gap-1.5"
            >
              {scenario.variants.map((v, i) => (
                <button
                  key={v.id}
                  role="tab"
                  type="button"
                  aria-selected={i === active}
                  onClick={() => setActive(i)}
                  className={cn(
                    "inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-[12.5px] font-medium ring-1 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60",
                    i === active
                      ? "bg-primary/10 text-primary ring-primary/40"
                      : "bg-card text-muted-foreground ring-border hover:text-foreground",
                  )}
                >
                  {v.decision.allow ? (
                    <CheckCircle2 className="size-3.5" aria-hidden />
                  ) : (
                    <XCircle className="size-3.5" aria-hidden />
                  )}
                  {v.label}
                </button>
              ))}
            </div>
          </div>
          <DecisionPanel variant={variant} />
        </div>
      </CardContent>
    </Card>
  );
}

export function PolicyWalkthrough() {
  return (
    <div className="space-y-5">
      <p className="text-sm text-muted-foreground">
        <span className="font-medium text-foreground">Usage control above access control.</span>{" "}
        Where WAC/ACP answers &ldquo;may this agent <em>read</em> graph G?&rdquo;, ODRL answers
        &ldquo;may this party <em>use</em> this asset — for purpose P, disclosing only to recipient
        R, with obligation O, until time T?&rdquo; Every decision is <em>fail-closed</em>: no
        matching, duty-discharged permission (or any matching prohibition) means DENY.
      </p>

      <div className="space-y-4">
        {SCENARIOS.map((s) => (
          <ScenarioCard key={s.id} scenario={s} />
        ))}
      </div>

      <div className="rounded-[var(--radius-lg)] border bg-muted/20 p-4 text-[13px] leading-relaxed text-muted-foreground">
        <p>
          <span className="font-semibold text-foreground">From decision to enforcement.</span>{" "}
          The <code className="font-mono text-[12px]">sparq-solid</code> odrl-bridge materializes
          these decisions onto access-controlled queries — see the{" "}
          <Link href="/showcase/solid-pairs" className="text-primary underline-offset-4 hover:underline">
            Solid (user, app)-pair result sets
          </Link>{" "}
          showcase, where one Pod returns different result sets per requester, enforced by the
          engine. The evaluator is conformance-ratcheted against the SolidLab ODRL Test Suite
          (67/68 through the real{" "}
          <code className="font-mono text-[12px]">evaluate</code> path).
        </p>
        <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-[12.5px]">
          <a
            className="text-primary underline-offset-4 hover:underline"
            href={`${REPO}/tree/main/crates/sparq-policy`}
            target="_blank"
            rel="noopener noreferrer"
          >
            sparq-policy crate ↗
          </a>
          <a
            className="text-primary underline-offset-4 hover:underline"
            href={`${REPO}/blob/main/skills/usage-control-policy/SKILL.md`}
            target="_blank"
            rel="noopener noreferrer"
          >
            usage-control-policy SKILL.md ↗
          </a>
        </div>
      </div>
    </div>
  );
}
