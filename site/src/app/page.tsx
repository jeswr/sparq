import Link from "next/link";
import { ArrowRight, Cpu, ShieldCheck, Boxes, Github } from "lucide-react";

// [OPUS-4.8] sq-vw3ax.5 — the Home rewrite: route + prove in under one screen.
//
// CUT (research/website-redesign.md §3, §4): the 14-card "Every surface" grid (moved to
// /capabilities) and the 4-sentence hero honesty preamble (now ONE success badge + one
// #how-it-runs tier strip + a "How tiers work" <details>). KEPT: the live REPL as the killer
// artifact, immediately after the hero.
//
// Top-to-bottom: hero (1 sentence + badge + 2 buttons + 3 pills) → live REPL → 3 flagship
// cards (→ /showcase) → 5 capability THEME cards (each listing its surfaces as links to
// /capabilities#<theme>) → #how-it-runs tier-legend strip + "How tiers work" details.
import { ReplLazy } from "@/components/repl-lazy";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  FLAGSHIPS,
  GROUPS,
  TIER_LABEL,
  TIER_VARIANT,
  type Surface,
  type Tier,
} from "@/data/surfaces";
import { DEEP_PAGE_SLUGS, capabilityAnchor } from "@/data/capabilities";

const REPO_URL = "https://github.com/jeswr/sparq";

// The honesty tiers, told ONCE here (the per-surface badge is the per-page pointer). Order
// mirrors the strongest → most-hedged execution model. Each is a real Tier so the legend can
// never drift from the badges the rest of the site renders.
const TIER_LEGEND: { tier: Tier; note: string }[] = [
  { tier: "live", note: "the real Rust engine, compiled to wasm, in your browser tab" },
  { tier: "live-bbjs", note: "in-tab proving via the 3rd-party bb.js UltraHonk prover" },
  { tier: "live-sim", note: "a faithful in-tab JS simulation of a native protocol" },
  { tier: "hosted", note: "a small hosted sparq-server where a wasm rebuild is uneconomic" },
  { tier: "walkthrough", note: "real, captured engine output replayed (native-only surface)" },
];

/** A surface's link on Home → its post-redesign home (deep page or /capabilities#theme). */
function surfaceHref(surface: Surface, groupId: string): string {
  if (DEEP_PAGE_SLUGS.has(surface.slug)) return surface.href;
  if (surface.slug === "try") return surface.href;
  return capabilityAnchor(groupId);
}

export default function HomePage() {
  return (
    <div className="space-y-14">
      {/* 1 — Hero: one sentence, one badge, two buttons, three pills. */}
      <section className="space-y-4">
        <Badge variant="success" className="h-6 px-3">
          The live REPL below runs in your browser tab
        </Badge>
        <h1 className="text-3xl font-semibold tracking-tight md:text-4xl">
          A state-of-the-art RDF triplestore &amp; SPARQL 1.1/1.2 engine,{" "}
          <span className="text-primary">live in your browser tab</span>.
        </h1>
        <div className="flex flex-wrap gap-3 pt-1">
          <Button asChild size="lg">
            <Link href="/try">Try the REPL</Link>
          </Button>
          <Button asChild size="lg" variant="outline">
            <a href={REPO_URL} target="_blank" rel="noopener noreferrer">
              <Github className="size-4" aria-hidden />
              GitHub
            </a>
          </Button>
        </div>
        <div className="flex flex-wrap gap-2 pt-1 text-sm text-muted-foreground">
          <Pill icon={Cpu} text="Rust core, compiled to WebAssembly" />
          <Pill icon={Boxes} text="RDF/JS @jeswr/sparq API" />
          <Pill icon={ShieldCheck} text="ZK + MPC privacy surfaces" />
        </div>
      </section>

      {/* 2 — The live REPL: the killer artifact, immediately. */}
      <section className="space-y-3">
        <ReplLazy />
        <p className="text-xs text-muted-foreground">
          This REPL loads the lean sparq wasm bundle and runs your SPARQL against
          the sample graph using the real Rust engine — the same code that ships as{" "}
          <code className="font-mono">@jeswr/sparq</code>. Nothing is sent to a
          server.
        </p>
      </section>

      {/* 3 — See it work: the 3 flagship showcases, large. */}
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-semibold">See it work</h2>
          <p className="text-sm text-muted-foreground">
            Three end-to-end privacy demonstrations, each runnable —{" "}
            <Link
              href="/examples"
              className="text-primary underline-offset-4 hover:underline"
            >
              browse the examples gallery
            </Link>
            .
          </p>
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          {FLAGSHIPS.map((f) => (
            <FlagshipCard key={f.href} surface={f} />
          ))}
        </div>
      </section>

      {/* 4 — What sparq can do: the 5 capability THEME cards. */}
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-semibold">What sparq can do</h2>
          <p className="text-sm text-muted-foreground">
            The feature set in five themes —{" "}
            <Link
              href="/capabilities"
              className="text-primary underline-offset-4 hover:underline"
            >
              browse the full capabilities gallery
            </Link>
            .
          </p>
        </div>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {GROUPS.map((group) => (
            <Card key={group.id} className="flex h-full flex-col">
              <CardHeader className="gap-1.5">
                <CardTitle className="text-base">
                  <Link
                    href={capabilityAnchor(group.id)}
                    className="hover:text-primary"
                  >
                    {group.label}
                  </Link>
                </CardTitle>
                <CardDescription className="leading-relaxed">
                  {group.description}
                </CardDescription>
              </CardHeader>
              <CardContent className="mt-auto">
                <ul className="flex flex-wrap gap-x-3 gap-y-1.5 text-sm">
                  {group.surfaces.map((s) => (
                    <li key={s.slug}>
                      <Link
                        href={surfaceHref(s, group.id)}
                        className="text-foreground/80 underline-offset-4 hover:text-primary hover:underline"
                      >
                        {s.title}
                      </Link>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      {/* 5 — #how-it-runs: ONE tier-legend strip + "How tiers work" details. */}
      <section id="how-it-runs" className="scroll-mt-20 space-y-4">
        <div>
          <h2 className="text-2xl font-semibold">How it runs</h2>
          <p className="text-sm text-muted-foreground">
            Every surface is honestly labelled by how it executes — colour is never
            the only signal, the text label carries the truth.
          </p>
        </div>
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

function Pill({
  icon: Icon,
  text,
}: {
  icon: React.ComponentType<{ className?: string }>;
  text: string;
}) {
  return (
    <span className="inline-flex items-center gap-1.5 rounded-full bg-muted px-3 py-1">
      <Icon className="size-3.5" />
      {text}
    </span>
  );
}

function FlagshipCard({ surface }: { surface: Surface }) {
  const Icon = surface.icon;
  return (
    <Link href={surface.href} className="group block focus-visible:outline-none">
      <Card className="h-full transition-colors group-hover:ring-primary/40 group-focus-visible:ring-3 group-focus-visible:ring-ring/50">
        <CardHeader className="gap-2">
          <div className="flex items-start justify-between gap-2">
            <span className="flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <Icon className="size-5" />
            </span>
            <Badge variant={TIER_VARIANT[surface.tier]}>
              {TIER_LABEL[surface.tier]}
            </Badge>
          </div>
          <CardTitle className="flex items-center gap-1.5">
            <span className="text-[var(--warning)]" aria-hidden>
              ★
            </span>
            {surface.title}
          </CardTitle>
          <CardDescription className="leading-relaxed">
            {surface.blurb}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <span className="inline-flex items-center gap-1 text-sm font-medium text-primary">
            Open demo
            <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-0.5" />
          </span>
        </CardContent>
      </Card>
    </Link>
  );
}
