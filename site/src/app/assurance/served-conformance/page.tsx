// [OPUS-4.8] sq-ro6if (PSS #1415/#1478) — /assurance/served-conformance: the served-surface
// SPARQL 1.1 Protocol conformance dashboard. A data-driven page (mirroring the /benchmarks +
// /assurance precedents) that renders the committed snapshot from
// src/data/served-conformance.generated.json — the machine-readable per-assertion outcomes +
// counts the `served-conformance-report` binary emits from the two EXISTING served-surface CI
// lanes (sq-jaj38 http-protocol + sq-1uuxz Service-Description/Graph-Store-Protocol).
//
// HONESTY. These are CONFORMANCE COUNTS, not performance numbers (no timings appear here, so the
// no-perf-numbers gate is satisfied). Floors are ratchet floors; documented divergences are shown
// per-assertion and never summed into the pass count. The page states plainly that it is a
// SNAPSHOT and that the authoritative versioned reports attach to each GitHub Release.

import type { Metadata } from "next";
import Link from "next/link";
import { ArrowUpRight, CheckCircle2, ShieldCheck, TriangleAlert, XCircle } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  servedConformance,
  shortCommit,
  type ConformanceOutcome,
  type ConformanceSuite,
} from "@/data/served-conformance";

export const metadata: Metadata = {
  title: "Served-surface SPARQL 1.1 Protocol conformance",
  description:
    "What sparq-server's served HTTP /sparql surface passes of the SPARQL 1.1 Protocol and the Service-Description + Graph-Store-Protocol — machine-readable per-assertion outcomes, published per release.",
};

const REPO = "https://github.com/sparq-org/sparq";
const RELEASES = `${REPO}/releases`;
const REPORT_SRC = `${REPO}/blob/main/site/src/data/served-conformance.generated.json`;

const { provenance, totals, suites } = servedConformance;

/** An outcome pill: pass (green), divergence (amber), fail (red). */
function OutcomePill({ outcome }: { outcome: ConformanceOutcome }) {
  const map: Record<ConformanceOutcome, { cls: string; label: string }> = {
    pass: {
      cls: "border-[var(--success)]/40 bg-[var(--success)]/10 text-[var(--success)]",
      label: "pass",
    },
    divergence: {
      cls: "border-amber-500/40 bg-amber-500/10 text-amber-600 dark:text-amber-400",
      label: "divergence",
    },
    fail: {
      cls: "border-red-500/40 bg-red-500/10 text-red-600 dark:text-red-400",
      label: "fail",
    },
  };
  const { cls, label } = map[outcome];
  return (
    <span
      className={`inline-flex items-center whitespace-nowrap rounded-full border px-2 py-0.5 text-xs font-medium ${cls}`}
    >
      {label}
    </span>
  );
}

/** A count chip in a suite header. */
function Count({
  icon,
  n,
  label,
  tone,
}: {
  icon: React.ReactNode;
  n: number;
  label: string;
  tone: string;
}) {
  return (
    <span className={`inline-flex items-center gap-1.5 text-sm ${tone}`}>
      {icon}
      <strong className="font-semibold">{n}</strong>
      <span className="text-muted-foreground">{label}</span>
    </span>
  );
}

function SuiteCard({ suite }: { suite: ConformanceSuite }) {
  return (
    <section className="space-y-4 rounded-xl border p-4">
      <div className="space-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="text-base font-semibold">{suite.label}</h3>
          <Badge variant="outline">{suite.family}</Badge>
          <Badge variant="outline">
            <code className="font-mono">{suite.feature}</code>
          </Badge>
        </div>
        <div className="flex flex-wrap items-center gap-x-5 gap-y-1">
          <Count
            icon={<CheckCircle2 className="size-4 text-[var(--success)]" aria-hidden />}
            n={suite.counts.pass}
            label="pass"
            tone="text-foreground"
          />
          <Count
            icon={<TriangleAlert className="size-4 text-amber-500" aria-hidden />}
            n={suite.counts.divergence}
            label="documented divergence"
            tone="text-foreground"
          />
          <Count
            icon={<XCircle className="size-4 text-red-500" aria-hidden />}
            n={suite.counts.fail}
            label="fail"
            tone="text-foreground"
          />
          <span className="text-sm text-muted-foreground">
            {suite.counts.total} assertions · ratchet floor{" "}
            <strong className="text-foreground">{suite.floor ?? "?"}</strong> (
            {suite.floor_basis})
          </span>
        </div>
      </div>
      <p className="text-sm text-muted-foreground">{suite.note}</p>
      <div className="overflow-x-auto rounded-lg border">
        <table className="w-full border-collapse text-left text-sm">
          <thead>
            <tr className="border-b bg-muted/40 text-xs uppercase tracking-wide text-muted-foreground">
              <th scope="col" className="px-3 py-2 font-medium">
                Assertion
              </th>
              <th scope="col" className="px-3 py-2 font-medium">
                Outcome
              </th>
            </tr>
          </thead>
          <tbody>
            {suite.tests.map((t, i) => (
              <tr key={i} className="border-b align-top last:border-0">
                <td className="px-3 py-2 text-muted-foreground">
                  {t.label}
                  {t.detail ? (
                    <span className="mt-0.5 block text-xs italic text-muted-foreground/80">
                      {t.detail}
                    </span>
                  ) : null}
                </td>
                <td className="px-3 py-2">
                  <OutcomePill outcome={t.outcome} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

export default function ServedConformancePage() {
  return (
    <div className="space-y-10">
      {/* ── Header ──────────────────────────────────────────────── */}
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <ShieldCheck className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">
              Served-surface SPARQL 1.1 Protocol conformance
            </h1>
            <p className="text-sm text-muted-foreground">
              What the served HTTP <code className="font-mono">/sparql</code> surface passes —
              published per release.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">SPARQL 1.1 Protocol</Badge>
          <Badge variant="outline">Service Description</Badge>
          <Badge variant="outline">Graph Store Protocol</Badge>
        </div>
      </header>

      {/* ── Lead ────────────────────────────────────────────────── */}
      <section className="measure space-y-3 text-sm text-muted-foreground">
        <p>
          A consumer that talks to sparq over its{" "}
          <strong className="text-foreground">served HTTP surface</strong> (the SPARQL 1.1
          Protocol <code className="font-mono">/sparql</code> endpoint) depends on the WIRE
          contract, which can diverge from the engine&rsquo;s internal semantics. This page
          publishes what that served surface passes, per release: the machine-readable outcomes
          the two served-surface CI lanes emit — the SPARQL 1.1 Protocol lane and the
          Service-Description + Graph-Store-Protocol lane — so an HTTP consumer can trust the
          contract without re-verifying it against every commit.
        </p>
        <p>
          Every number here is a <strong className="text-foreground">conformance count</strong>{" "}
          (pass / documented-divergence / fail assertion tallies), never a performance number.
          Documented divergences are shown per-assertion and are{" "}
          <strong className="text-foreground">never summed into the pass count</strong>, so a
          documented gap can never inflate the conformance number.
        </p>
      </section>

      {/* ── Summary + provenance ────────────────────────────────── */}
      <section className="space-y-3 rounded-xl border bg-card p-4">
        <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
          <Count
            icon={<CheckCircle2 className="size-4 text-[var(--success)]" aria-hidden />}
            n={totals.pass}
            label="pass"
            tone="text-foreground"
          />
          <Count
            icon={<TriangleAlert className="size-4 text-amber-500" aria-hidden />}
            n={totals.divergence}
            label="documented divergence"
            tone="text-foreground"
          />
          <Count
            icon={<XCircle className="size-4 text-red-500" aria-hidden />}
            n={totals.fail}
            label="fail"
            tone="text-foreground"
          />
          <span className="text-sm text-muted-foreground">
            {totals.total} assertions across {suites.length} served suites
          </span>
        </div>
        <dl className="grid grid-cols-1 gap-x-6 gap-y-1 text-xs text-muted-foreground sm:grid-cols-2">
          <div>
            <dt className="inline font-medium text-foreground">Snapshot of: </dt>
            <dd className="inline">
              <a
                href={`${REPO}/commit/${provenance.commit}`}
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono text-primary hover:underline"
              >
                {shortCommit(provenance.commit)}
              </a>{" "}
              ({provenance.version})
            </dd>
          </div>
          <div>
            <dt className="inline font-medium text-foreground">Generated: </dt>
            <dd className="inline">{provenance.generated_at}</dd>
          </div>
        </dl>
        <p className="rounded-lg border border-amber-500/30 bg-amber-500/5 px-3 py-2.5 text-xs text-muted-foreground">
          <strong className="text-foreground">This is a snapshot.</strong> The authoritative,
          versioned per-release reports (
          <code className="font-mono">served-conformance-&lt;version&gt;.json</code>) attach to
          each <Artifact href={RELEASES} label="GitHub Release" /> and are SLSA build-provenance
          attested; the same JSON is emitted as a build artifact by the{" "}
          <code className="font-mono">service-federation-conformance</code> CI job. This page
          renders the committed{" "}
          <Artifact href={REPORT_SRC} label="snapshot JSON" />, which may lag a release.
        </p>
      </section>

      {/* ── Per-suite detail ────────────────────────────────────── */}
      <section className="space-y-5">
        {suites.map((s) => (
          <SuiteCard key={s.id} suite={s} />
        ))}
      </section>

      {/* ── What these numbers mean ─────────────────────────────── */}
      <section className="space-y-3">
        <h2 className="text-lg font-semibold">What these numbers mean</h2>
        <ul className="space-y-3 text-sm text-muted-foreground">
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">Floors are floors.</strong> Each suite&rsquo;s{" "}
            ratchet floor may only rise; a pass count equal to the floor is the honest current
            state, not an aspirational target. The floor comes from the same central registry
            that gates CI.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">Divergences are part of the claim.</strong> A
            documented divergence is a W3C-permitted behaviour distinct from a naive expectation
            (for example, an absent <code className="font-mono">Accept</code> defaulting to
            SPARQL-results JSON). It is reported, never hidden, and never counted as a pass.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">Served ≠ engine.</strong> This measures the HTTP
            wire contract specifically — the surface an out-of-process consumer actually depends
            on — which is why it is published separately from the engine conformance ratchets.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">Fail-closed publication.</strong> The report
            generator refuses to emit a report that silently omits a suite: a vacuous, failing,
            or below-floor run blocks the release rather than publishing a misleading artifact.
          </li>
        </ul>
      </section>

      {/* ── CTAs ────────────────────────────────────────────────── */}
      <section className="flex flex-wrap gap-3 border-t pt-6">
        <Button asChild variant="outline" size="sm">
          <Link href="/assurance">Back to Assurance</Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a href={RELEASES} target="_blank" rel="noopener noreferrer">
            Per-release reports (GitHub Releases)
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a href={REPORT_SRC} target="_blank" rel="noopener noreferrer">
            Snapshot JSON
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
      </section>
    </div>
  );
}

/** An external link to a repo artifact, with the small up-right glyph. */
function Artifact({ href, label }: { href: string; label: string }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 text-primary hover:underline"
    >
      {label}
      <ArrowUpRight className="size-3 opacity-70" aria-hidden />
    </a>
  );
}
