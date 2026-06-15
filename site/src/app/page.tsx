import Link from "next/link";
import { ArrowRight, Cpu, ShieldCheck, Boxes } from "lucide-react";

import { Repl } from "@/components/repl";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  ALL_SURFACES,
  FLAGSHIPS,
  TIER_LABEL,
  TIER_VARIANT,
} from "@/data/surfaces";

export default function HomePage() {
  return (
    <div className="space-y-12">
      {/* Hero */}
      <section className="space-y-4">
        <Badge variant="success" className="h-6 px-3">
          The live REPL below runs in your browser tab
        </Badge>
        <h1 className="text-3xl font-semibold tracking-tight md:text-4xl">
          A state-of-the-art RDF + SPARQL engine,{" "}
          <span className="text-primary">live in your tab</span>.
        </h1>
        <p className="measure text-muted-foreground md:text-lg">
          sparq is a SOTA Rust RDF triplestore and SPARQL 1.1/1.2 engine with a
          browser WASM port. This site demonstrates real sparq output — no mocks.
          Each surface is honestly labelled by how it runs: live in your tab,
          live via a 3rd-party prover, a faithful in-tab simulation, hosted, or a
          captured-I/O walkthrough.
        </p>
        <div className="flex flex-wrap gap-2 pt-1 text-sm text-muted-foreground">
          <Feature icon={Cpu} text="Rust core, compiled to WebAssembly" />
          <Feature icon={Boxes} text="RDF/JS @jeswr/sparq API" />
          <Feature icon={ShieldCheck} text="ZK + MPC privacy surfaces" />
        </div>
      </section>

      {/* Marquee live demo */}
      <section className="space-y-3">
        <Repl />
        <p className="text-xs text-muted-foreground">
          This REPL loads the lean sparq wasm bundle (~1.2 MB) and runs your
          SPARQL against the sample graph using the real Rust engine — the same
          code that ships as <code className="font-mono">@jeswr/sparq</code>.
          Nothing is sent to a server.
        </p>
      </section>

      {/* Flagship showcases */}
      <section className="space-y-4">
        <div className="flex items-end justify-between">
          <div>
            <h2 className="text-2xl font-semibold">Flagship showcases</h2>
            <p className="text-sm text-muted-foreground">
              Three end-to-end privacy demonstrations.
            </p>
          </div>
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          {FLAGSHIPS.map((f) => (
            <SurfaceCard
              key={f.href}
              href={f.href}
              title={f.title}
              blurb={f.blurb}
              tier={f.tier}
              icon={f.icon}
              flagship
            />
          ))}
        </div>
      </section>

      {/* Full surface grid */}
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-semibold">Every surface</h2>
          <p className="text-sm text-muted-foreground">
            The full sparq feature set. Pages still being built link to an honest
            placeholder.
          </p>
        </div>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {ALL_SURFACES.filter((s) => s.slug !== "try" && s.slug !== "about").map(
            (s) => (
              <SurfaceCard
                key={s.href}
                href={s.href}
                title={s.title}
                blurb={s.blurb}
                tier={s.tier}
                icon={s.icon}
              />
            ),
          )}
        </div>
      </section>
    </div>
  );
}

function Feature({
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

function SurfaceCard({
  href,
  title,
  blurb,
  tier,
  icon: Icon,
  flagship,
}: {
  href: string;
  title: string;
  blurb: string;
  tier: keyof typeof TIER_LABEL;
  icon: React.ComponentType<{ className?: string }>;
  flagship?: boolean;
}) {
  return (
    <Link href={href} className="group block focus-visible:outline-none">
      <Card className="h-full transition-colors group-hover:ring-primary/40 group-focus-visible:ring-3 group-focus-visible:ring-ring/50">
        <CardHeader className="gap-2">
          <div className="flex items-start justify-between gap-2">
            <span className="flex size-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <Icon className="size-4.5" />
            </span>
            <Badge variant={TIER_VARIANT[tier]}>{TIER_LABEL[tier]}</Badge>
          </div>
          <CardTitle className="flex items-center gap-1.5">
            {flagship && <span className="text-[var(--warning)]">★</span>}
            {title}
          </CardTitle>
          <CardDescription className="leading-relaxed">{blurb}</CardDescription>
        </CardHeader>
        <CardContent>
          <span className="inline-flex items-center gap-1 text-sm font-medium text-primary">
            Open
            <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-0.5" />
          </span>
        </CardContent>
      </Card>
    </Link>
  );
}
