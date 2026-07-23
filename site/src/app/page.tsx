import Link from "next/link";

// [OPUS-4.8] sq-vw3ax.11 — the homepage, with the SPLIT hero making the in-browser runner the
// hero artifact (the full REPL workbench, which was duplicated with /try, is REMOVED from home).
// The landing is now:
//   1. a SPLIT atmospheric HERO — text on the left, a lightweight LIVE in-browser runner
//      (HeroQueryRunner) on the right, so Run is in-fold (≤3 interactions to a live result);
//   2. a slim four-stat STRUCTURAL band under the hero (StatBand);
//   3. the bold "See it work" flagship showcase (the real FLAGSHIPS) and the five capability
//      THEME cards with per-surface honesty-tier dots; and
//   4. the "How it runs" honesty strip — the tier legend + a "How tiers work" <details>.
//
// REAL DATA / HONESTY (load-bearing): every figure is a STRUCTURAL fact derived from the
// single GROUPS/FLAGSHIPS source (data/surfaces.ts) or a query-form/format list — NOT a
// performance/timing claim and NOT fabricated. The hero runner's results come from the real
// engine at runtime; nothing here hardcodes a result row, triple count, or timing.
import { Badge } from "@/components/ui/badge";
import {
  FLAGSHIPS,
  GROUPS,
  TIER_LABEL,
  TIER_VARIANT,
  type Tier,
} from "@/data/surfaces";
import { Hero } from "@/components/home/hero";
import { StatBand } from "@/components/home/stat-band";
import { SectionHeader } from "@/components/home/section-header";
import { FlagshipCard } from "@/components/home/flagship-card";
import { CapabilityCard } from "@/components/home/capability-card";

const REPO_URL = "https://github.com/sparq-org/sparq";

// The honesty tiers, told ONCE here (the per-surface badge/dot is the per-page pointer).
// Order mirrors the strongest → most-hedged execution model. Each is a real Tier so the
// legend can never drift from the badges the rest of the site renders.
const TIER_LEGEND: { tier: Tier; note: string }[] = [
  { tier: "live", note: "the real Rust engine, compiled to wasm, in your browser tab" },
  { tier: "live-bbjs", note: "in-tab proving via the 3rd-party bb.js UltraHonk prover" },
  { tier: "live-sim", note: "a faithful in-tab JS simulation of a native protocol" },
  { tier: "hosted", note: "a small hosted sparq-server where a wasm rebuild is uneconomic" },
  // [GPT-5.6] sq-vw3ax.15 — the dot used by built opt-in crates has an explicit legend.
  { tier: "native", note: "a built, opt-in Rust crate with code and docs linked from this site" },
  { tier: "walkthrough", note: "real, captured engine output replayed (native-only surface)" },
];

export default function HomePage() {
  return (
    <div className="space-y-20">
      {/* 1 — Split hero + slim stat band (the band sits close under the hero, one visual group). */}
      <div className="space-y-6">
        <Hero />
        <StatBand />
      </div>

      {/* 3 — See it work: the real flagship showcases, large. */}
      <section className="space-y-6">
        <SectionHeader
          kicker="See it work"
          title="Three end-to-end privacy demonstrations, each runnable"
          note={
            <>
              From a live in-tab proof to a faithful protocol simulation — pick one,
              or{" "}
              {/* [OPUS-4.8] sq-ymr2e.4 — persistent underline so the inline link is distinguishable
                  without relying on colour (WCAG 2.1 §1.4.1 / axe `link-in-text-block`). */}
              <Link
                href="/examples"
                className="text-primary underline underline-offset-4"
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
                className="text-primary underline underline-offset-4"
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
                className="text-primary underline underline-offset-4"
                href={REPO_URL}
                target="_blank"
                rel="noopener noreferrer"
              >
                Source on GitHub
              </a>
              <Link
                className="text-primary underline underline-offset-4"
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
