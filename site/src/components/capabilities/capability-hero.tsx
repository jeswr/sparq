// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign HERO (server component).
//
// Faithful to the approved mockup (sparq-design-system/proposals/web-capabilities.html §HERO):
// a confident display heading with the brand teal→violet gradient (the foundation `.text-gradient`
// / `--hero-grad`, NO new hue), a lede, a DERIVED stat strip (hero-stats.ts — every figure from
// real data), a theme jump-rail of the five lanes, and the REAL prefilled query from
// sample-graph.ts rendered with the shared `.sq-tok-*` highlighter. The "Run query" affordance
// links to /app (the single workbench — sq-4hiqe, the /try REPL was removed) — it never shows a
// fabricated result inline. /app is a separate overlaid Next app, so the CTAs are HARD anchors
// (withBasePath), not next/link soft navigations.

import { ArrowRight, Play } from "lucide-react";

import { GROUPS } from "@/data/surfaces";
import { EXAMPLE_QUERIES } from "@/data/sample-graph";
import { withBasePath } from "@/lib/base-path";
import { Badge } from "@/components/ui/badge";
import { HERO_STATS } from "@/components/capabilities/hero-stats";
import { laneAccent } from "@/components/capabilities/lane-accents";
import { SparqlHighlight } from "@/components/capabilities/sparql-highlight";

// The prefilled default query the live REPL opens with (the ex:alice profile) — REAL, from the
// single sample-graph source. Shown read-only here; "Run query" opens it live in /try.
const HERO_QUERY = EXAMPLE_QUERIES[0].sparql;

export function CapabilityHero() {
  return (
    <header className="relative pb-2 pt-4 sm:pt-8">
      {/* Eyebrow — live pulse + the one-line honest framing. */}
      <span className="kicker inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/[0.07] px-3 py-1.5 normal-case">
        <span className="relative flex size-2">
          <span className="absolute inline-flex size-full animate-ping rounded-full bg-[var(--success)] opacity-60" />
          <span className="relative inline-flex size-2 rounded-full bg-[var(--success)]" />
        </span>
        One engine · {GROUPS.length} themes · honestly labelled
      </span>

      <h1 className="display-1 mt-6 max-w-[16ch] font-semibold">
        The whole RDF stack,{" "}
        <span className="text-gradient">one Rust engine</span> — in your tab.
      </h1>

      <p className="measure mt-5 text-lg text-muted-foreground">
        Every sparq capability, grouped into five themes you can scan in seconds.
        Query, reason, search, prove, and serve — each surface runs the{" "}
        <span className="font-medium text-foreground">real engine</span> and wears
        a badge that says exactly how it runs, from{" "}
        <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-[0.85em] text-foreground">
          live wasm
        </code>{" "}
        to a captured-output walkthrough. No demo pretends to be more than it is.
      </p>

      <div className="mt-9 grid gap-6 lg:grid-cols-[1fr_1.05fr] lg:items-stretch">
        {/* Left: derived stat strip + theme jump-rail. */}
        <div className="flex flex-col justify-between gap-8">
          <dl className="grid grid-cols-2 gap-x-6 gap-y-7 border-t pt-6 sm:grid-cols-4 lg:grid-cols-2">
            {HERO_STATS.map((s) => (
              <div key={s.caption}>
                <dt className="sr-only">{s.caption}</dt>
                <dd>
                  <span className="font-mono text-3xl font-semibold tracking-tight tabular">
                    {s.value}
                    {s.unit && (
                      <span className="ml-1 text-base text-primary">{s.unit}</span>
                    )}
                  </span>
                  <span className="mt-1.5 block text-[12.5px] text-muted-foreground">
                    {s.caption}
                  </span>
                </dd>
              </div>
            ))}
          </dl>

          <nav aria-label="Jump to a capability theme" className="flex flex-wrap gap-2">
            {GROUPS.map((g) => (
              <a
                key={g.id}
                href={`#${g.id}`}
                className="inline-flex items-center gap-2 rounded-full border bg-card px-3 py-1.5 text-[13px] text-muted-foreground transition-colors hover:border-primary/45 hover:text-foreground"
              >
                <span
                  aria-hidden
                  className="size-2.5 rounded-[3px]"
                  style={{ background: laneAccent(g.id) }}
                />
                {g.label}
              </a>
            ))}
          </nav>
        </div>

        {/* Right: a real prefilled query → "Run query" opens it live in the /app workbench. */}
        <div className="overflow-hidden rounded-2xl border border-primary/25 bg-card shadow-elevation-2">
          <div className="flex items-center justify-between gap-3 border-b px-4 py-3">
            <div className="flex items-center gap-2.5">
              <Badge variant="success" className="h-5 px-2 text-[11px]">
                Live in your tab
              </Badge>
              <span className="text-sm font-medium">Live SPARQL</span>
            </div>
            <a
              href={withBasePath("/app/")}
              className="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[13px] font-semibold text-primary-foreground shadow-elevation-glow transition-transform hover:translate-y-px"
            >
              <Play className="size-3.5 fill-current" aria-hidden />
              Run query
            </a>
          </div>
          <SparqlHighlight
            query={HERO_QUERY}
            className="overflow-x-auto bg-[color-mix(in_oklch,var(--background)_55%,var(--card))] px-5 py-4 text-[12.5px] leading-[1.7] text-foreground"
          />
          <p className="border-t px-5 py-2.5 text-[12px] text-muted-foreground">
            The real prefilled query from the sample social graph —{" "}
            <a
              href={withBasePath("/app/")}
              className="inline-flex items-center gap-1 font-medium text-primary underline-offset-4 hover:underline"
            >
              open it in the workbench
              <ArrowRight className="size-3" aria-hidden />
            </a>
            .
          </p>
        </div>
      </div>
    </header>
  );
}
