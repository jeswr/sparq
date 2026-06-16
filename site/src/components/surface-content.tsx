// [OPUS-4.8] sq-3hrc — shared layout for a real (non-placeholder) surface page.
// Renders a consistent structure per research/feature-showcase-site-design.md §2:
// capability statement → tier badge → capability list → "how this works" →
// honest caveat → CTAs. Content is grounded in each surface's skills/*/SKILL.md.

import type { ReactNode } from "react";
import Link from "next/link";
import { ArrowLeft, Check, Info, TriangleAlert } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { type Tier, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export interface SurfaceContentProps {
  icon: LucideIcon;
  title: string;
  /** One-line capability statement. */
  statement: string;
  tier: Tier;
  /** Optional extra badge label (e.g. "Native-only", "Research-grade"). */
  extraBadge?: string;
  /** Lead paragraphs of prose. */
  intro: ReactNode;
  /** The concrete capabilities this surface offers (bullet list). */
  capabilities: { title: string; body: string }[];
  /** "How this runs" honest note. */
  runsNote: ReactNode;
  /** Honest caveat / what is NOT covered. */
  caveat: ReactNode;
  /** Optional CTA links beyond the source link. */
  links?: { href: string; label: string; external?: boolean }[];
  /** Anything extra (e.g. an embedded REPL or table). */
  children?: ReactNode;
}

export function SurfaceContent({
  icon: Icon,
  title,
  statement,
  tier,
  extraBadge,
  intro,
  capabilities,
  runsNote,
  caveat,
  links,
  children,
}: SurfaceContentProps) {
  return (
    <div className="space-y-10">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to overview
        </Link>
      </Button>

      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Icon className="size-5" aria-hidden="true" />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">{title}</h1>
            <p className="text-sm text-muted-foreground">{statement}</p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant={TIER_VARIANT[tier]}>{TIER_LABEL[tier]}</Badge>
          {extraBadge && <Badge variant="muted">{extraBadge}</Badge>}
        </div>
      </header>

      <section className="measure space-y-3 text-muted-foreground">{intro}</section>

      <section className="space-y-4">
        <h2 className="text-xl font-semibold">Capabilities</h2>
        <div className="grid gap-3 sm:grid-cols-2">
          {capabilities.map((c) => (
            <div
              key={c.title}
              className="flex gap-3 rounded-xl bg-muted/40 p-4 ring-1 ring-foreground/10"
            >
              <Check
                className="mt-0.5 size-4 shrink-0 text-[var(--success)]"
                aria-hidden="true"
              />
              <div>
                <div className="text-sm font-semibold">{c.title}</div>
                <div className="text-sm text-muted-foreground">{c.body}</div>
              </div>
            </div>
          ))}
        </div>
      </section>

      {children}

      <section className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader className="flex-row items-center gap-2 space-y-0">
            <Info className="size-5 text-primary" aria-hidden="true" />
            <CardTitle className="text-base">How this runs</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-muted-foreground">
            {runsNote}
          </CardContent>
        </Card>
        <Card className="ring-[var(--warning)]/30">
          <CardHeader className="flex-row items-center gap-2 space-y-0">
            <TriangleAlert
              className="size-5 text-[var(--warning)]"
              aria-hidden="true"
            />
            <CardTitle className="text-base">Honest caveats</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-muted-foreground">
            {caveat}
          </CardContent>
        </Card>
      </section>

      <section className="flex flex-wrap gap-2">
        {links?.map((l) =>
          l.external ? (
            <Button key={l.href} asChild variant="outline" size="sm">
              <a href={l.href} target="_blank" rel="noopener noreferrer">
                {l.label}
              </a>
            </Button>
          ) : (
            <Button key={l.href} asChild variant="outline" size="sm">
              <Link href={l.href}>{l.label}</Link>
            </Button>
          ),
        )}
        <Button asChild variant="outline" size="sm">
          <a
            href="https://github.com/jeswr/sparq"
            target="_blank"
            rel="noopener noreferrer"
          >
            Source on GitHub
          </a>
        </Button>
      </section>
    </div>
  );
}
