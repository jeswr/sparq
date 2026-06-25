import Link from "next/link";

// [OPUS-4.8] sq-vw3ax.5 — the BOLD homepage redesign, built on the merged dark-first
// foundation (#1261) and faithful to the approved mockup
// (sparq-design-system/proposals/web-home.html). It reimagines the landing as:
//   1. an atmospheric teal HERO (.bg-atmos/.bg-grid + .text-gradient .display-1 headline +
//      mono .kicker rhythm) with a four-stat STRUCTURAL strip (count-badge style figures);
//   2. the live-REPL "moment" — the heavy <ReplLazy> framed (USED AS-IS; the /try redesign
//      owns the REPL internals) as the killer artifact, immediately after the hero;
//   3. the bold "See it work" flagship showcase (the real FLAGSHIPS) and the five capability
//      THEME cards with per-surface honesty-tier dots; and
//   4. the "How it runs" honesty strip — the tier legend + a "How tiers work" <details>.
//
// REAL DATA / HONESTY (load-bearing): every figure is a STRUCTURAL fact derived from the
// single GROUPS/FLAGSHIPS source (data/surfaces.ts) or a query-form/format list — NOT a
// performance/timing claim and NOT fabricated. The REPL's results come from the real engine
// at runtime; nothing here hardcodes a result row, triple count, or timing.
import { ReplLazy } from "@/components/repl-lazy";
import { Badge } from "@/components/ui/badge";
import {
  FLAGSHIPS,
  GROUPS,
  TIER_LABEL,
  TIER_VARIANT,
  type Tier,
} from "@/data/surfaces";
import { Hero } from "@/components/home/hero";
import { SectionHeader } from "@/components/home/section-header";
import { FlagshipCard } from "@/components/home/flagship-card";
import { CapabilityCard } from "@/components/home/capability-card";

const REPO_URL = "https://github.com/jeswr/sparq";

// The honesty tiers, told ONCE here (the per-surface badge/dot is the per-page pointer).
// Order mirrors the strongest → most-hedged execution model. Each is a real Tier so the
// legend can never drift from the badges the rest of the site renders.
const TIER_LEGEND: { tier: Tier; note: string }[] = [
  { tier: "live", note: "the real Rust engine, compiled to wasm, in your browser tab" },
  { tier: "live-bbjs", note: "in-tab proving via the 3rd-party bb.js UltraHonk prover" },
  { tier: "live-sim", note: "a faithful in-tab JS simulation of a native protocol" },
  { tier: "hosted", note: "a small hosted sparq-server where a wasm rebuild is uneconomic" },
  { tier: "walkthrough", note: "real, captured engine output replayed (native-only surface)" },
];

export default function HomePage() {
  return (
    <div className="space-y-20">
      {/* 1 — Hero: atmospheric ground, gradient display headline, structural stat strip. */}
      <Hero />

      {/* 2 — The live REPL: the killer artifact, framed. ReplLazy is used AS-IS. */}
      <section id="repl" className="scroll-mt-20 space-y-5">
        <SectionHeader
          kicker="The killer artifact"
          title="A live SPARQL REPL — real engine, real graph, real results"
          note="Edit the query, pick an example, hit Run. The lean wasm bundle parses the sample Turtle and answers in-tab."
        />
        <ReplLazy />
        <p className="text-xs text-muted-foreground">
          This REPL loads the lean sparq wasm bundle and runs your SPARQL against
          the sample graph using the real Rust engine — the same code that ships as{" "}
          <code className="font-mono text-foreground">@jeswr/sparq</code>. Nothing
          is sent to a server.
        </p>
      </section>

      {/* 3 — See it work: the real flagship showcases, large. */}
      <section className="space-y-6">
        <SectionHeader
          kicker="See it work"
          title="Three end-to-end privacy demonstrations, each runnable"
          note={
            <>
              From a live in-tab proof to a faithful protocol simulation — pick one,
              or{" "}
              <Link
                href="/examples"
                className="text-primary underline-offset-4 hover:underline"
              >
                browse the examples gallery
              </Link>
              .
            </>
          }
        />
        <div className="grid gap-4 md:grid-cols-3">
          {FLAGSHIPS.map((surface) => (
            <FlagshipCard key={surface.href} surface={surface} />
          ))}
        </div>
      </section>

      {/* 4 — What sparq can do: the 5 capability THEME cards, with honesty-tier dots. */}
      <section className="space-y-6">
        <SectionHeader
          kicker="What sparq can do"
          title="The full feature set, in five themes"
          note={
            <>
              Every surface declares how it runs — the coloured dot before each is
              its honesty tier.{" "}
              <Link
                href="/capabilities"
                className="text-primary underline-offset-4 hover:underline"
              >
                Browse all surfaces
              </Link>
              .
            </>
          }
        />
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {GROUPS.map((group, i) => (
            <CapabilityCard key={group.id} group={group} index={i} />
          ))}
        </div>
      </section>

      {/* 5 — How it runs: ONE tier-legend strip + "How tiers work" details (honesty). */}
      <section id="how-it-runs" className="scroll-mt-20 space-y-6">
        <SectionHeader
          kicker="Honesty by design"
          title="How it runs"
          note="Every surface is honestly labelled by how it executes — colour is never the only signal; the text label carries the truth."
        />
        <div className="flex flex-wrap gap-2">
          {TIER_LEGEND.map(({ tier, note }) => (
            <span
              key={tier}
              className="inline-flex items-center gap-2 rounded-full border bg-card px-3 py-1.5 text-sm"
            >
              <Badge variant={TIER_VARIANT[tier]} className="h-5 px-2 text-[11px]">
                {TIER_LABEL[tier]}
              </Badge>
              <span className="text-muted-foreground">{note}</span>
            </span>
          ))}
        </div>
        <details className="rounded-xl border bg-card px-4 py-3 text-sm">
          <summary className="cursor-pointer select-none font-medium">
            How tiers work
          </summary>
          <div className="mt-3 space-y-3 text-muted-foreground">
            <p>
              sparq is a state-of-the-art Rust RDF triplestore and SPARQL 1.1/1.2
              engine, with a browser WASM port. This site demonstrates its feature
              surfaces with <strong>real engine output</strong>, never mocks — and
              is honest about the seam: every surface declares exactly how it runs.
            </p>
            <p>
              Only the core parser, triplestore, SPARQL engine and the four text
              formats are in the shipped lean wasm bundle today. &ldquo;Live
              everywhere&rdquo; is a three-tier strategy — the lean bundle, optional
              wasm bundles lazy-loaded per surface, and a small hosted server /
              3rd-party prover where a wasm rebuild is uneconomic — plus an in-tab
              simulation for MPC. Every remaining surface degrades to a guided
              walkthrough with real, captured I/O.
            </p>
            <p>
              The ZK verifier is research-grade (v1): sound as landed under its
              stated threat model, but <strong>not externally audited</strong> and
              pending re-review — indicative engineering, not an audited
              cryptographic guarantee. The MPC surface is a faithful in-tab
              simulation of the protocol the native{" "}
              <code className="font-mono">sparq-mpc</code> crate runs, not the
              hardened crate executing in your browser, and not a proof of
              correctness (that layer is a stub today).
            </p>
            <p className="flex flex-wrap gap-4">
              <a
                className="text-primary underline-offset-4 hover:underline"
                href={REPO_URL}
                target="_blank"
                rel="noopener noreferrer"
              >
                Source on GitHub
              </a>
              <Link
                className="text-primary underline-offset-4 hover:underline"
                href="/capabilities"
              >
                Every surface, by theme
              </Link>
            </p>
          </div>
        </details>
      </section>
    </div>
  );
}
