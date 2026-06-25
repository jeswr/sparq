// [OPUS-4.8] sq-vw3ax.5 — the bold homepage HERO, faithful to the approved mockup
// (sparq-design-system/proposals/web-home.html `.hero`). Built from the merged dark-first
// foundation utilities (.bg-atmos / .bg-grid / .display-1 / .text-gradient / .kicker) and
// the count-badge variant — NO new hue, NO globals.css edit. The atmospheric ground is
// mounted here, page-scoped (the utilities are fixed/inset:0/z-index:-1, so they layer
// behind the AppShell content without touching the shell chrome).
//
// REAL DATA / HONESTY: the four structural stats are STRUCTURAL FACTS, not performance
// claims — the SPARQL query forms + RDF text formats the engine ships, the surface count
// derived from the single GROUPS source (data/surfaces.ts), and "0 servers" (the engine
// runs in-tab). No number is fabricated or a timing/throughput claim.

import Link from "next/link";
import { Boxes, Cpu, Github, Play, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ALL_SURFACES, GROUPS } from "@/data/surfaces";
import { RDF_TEXT_FORMATS } from "@/data/home-facts";

const REPO_URL = "https://github.com/jeswr/sparq";

// Derived from the single IA source so the headline counts can never drift from the nav,
// the capability gallery, or Cmd-K. (No literal "16"/"5" hardcoded here.)
const SURFACE_COUNT = ALL_SURFACES.length;
const THEME_COUNT = GROUPS.length;
const FORMAT_COUNT = RDF_TEXT_FORMATS.length;

interface Stat {
  /** The leading number/value (mono, the `--primary` accent unit lives in `unit`). */
  value: string;
  unit?: string;
  /** Whether the value should render after the unit (e.g. "4 formats"). */
  unitLeads?: boolean;
  label: string;
}

const STATS: Stat[] = [
  {
    value: "SPARQL",
    unit: "1.1/1.2",
    label: "SELECT · ASK · CONSTRUCT · DESCRIBE · UPDATE",
  },
  {
    value: "formats",
    unit: String(FORMAT_COUNT),
    unitLeads: true,
    label: RDF_TEXT_FORMATS.join(" · "),
  },
  {
    value: "surfaces",
    unit: String(SURFACE_COUNT),
    unitLeads: true,
    label: `across ${THEME_COUNT} capability themes`,
  },
  {
    value: "servers",
    unit: "0",
    unitLeads: true,
    label: "the engine runs entirely in your tab",
  },
];

export function Hero() {
  return (
    <section className="relative -mt-8 pb-2 pt-10 md:-mt-10 md:pt-14">
      {/* Atmospheric ground — mounted page-scoped. Fixed + inert, behind all content. */}
      <div className="bg-atmos" aria-hidden />
      <div className="bg-grid" aria-hidden />

      <Badge
        variant="success"
        className="h-7 gap-2 rounded-full px-3 text-[0.78rem]"
      >
        <span
          className="size-1.5 animate-pulse rounded-full bg-[var(--success)]"
          aria-hidden
        />
        The live REPL below runs in your browser tab — nothing is sent to a server
      </Badge>

      <h1 className="display-1 mt-5 max-w-[17ch] font-semibold">
        A state-of-the-art RDF engine,{" "}
        <span className="text-gradient">running live in your browser tab.</span>
      </h1>

      <p className="mt-5 max-w-[56ch] text-lg leading-relaxed text-muted-foreground">
        sparq is a SOTA Rust triplestore and SPARQL 1.1/1.2 engine, compiled to
        WebAssembly. The same code ships as{" "}
        <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-[0.92em] text-foreground">
          @jeswr/sparq
        </code>{" "}
        — streaming cursors, property paths, RDF 1.2 triple terms — and answers
        your queries in-tab, on the real engine.
      </p>

      <div className="mt-7 flex flex-wrap gap-3">
        <Button asChild size="lg">
          <Link href="#repl">
            <Play className="size-4" aria-hidden />
            Run a query now
          </Link>
        </Button>
        <Button asChild size="lg" variant="outline">
          <a href={REPO_URL} target="_blank" rel="noopener noreferrer">
            <Github className="size-4" aria-hidden />
            GitHub
          </a>
        </Button>
      </div>

      <div className="mt-6 flex flex-wrap gap-2 text-sm text-muted-foreground">
        <Pill icon={Cpu} text="Rust core → WebAssembly" />
        <Pill icon={Boxes} text="RDF/JS @jeswr/sparq API" />
        <Pill icon={ShieldCheck} text="ZK + MPC privacy surfaces" />
      </div>

      {/* Structural stat strip — a single hairline-divided grid, mono figures. */}
      <dl
        className="mt-11 grid grid-cols-2 gap-px overflow-hidden rounded-xl bg-border ring-1 ring-foreground/10 md:grid-cols-4"
        aria-label="What the engine answers in-tab"
      >
        {STATS.map((stat) => (
          <div
            key={stat.label}
            className="bg-[color-mix(in_oklch,var(--card)_80%,var(--background))] px-5 py-4"
          >
            <dt className="block font-mono text-2xl font-semibold tracking-tight">
              {stat.unitLeads ? (
                <>
                  <span className="text-primary">{stat.unit}</span> {stat.value}
                </>
              ) : (
                <>
                  {stat.value}{" "}
                  {stat.unit ? (
                    <span className="text-primary">{stat.unit}</span>
                  ) : null}
                </>
              )}
            </dt>
            <dd className="mt-1 text-xs text-muted-foreground">{stat.label}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function Pill({
  icon: Icon,
  text,
}: {
  icon: React.ComponentType<{ className?: string }>;
  text: string;
}) {
  return (
    <span className="inline-flex items-center gap-1.5 rounded-full border bg-card px-3 py-1.5">
      <Icon className="size-3.5 text-primary" aria-hidden />
      {text}
    </span>
  );
}
