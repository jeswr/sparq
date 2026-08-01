// [OPUS-4.8] sq-3hrc — shared layout for a real (non-placeholder) surface page.
//
// [OPUS-4.8] sq-vw3ax.8 — REBUILT to the scan-first, disclosure-based template from the
// website-redesign record (research/website-redesign.md §4 "CAPABILITY DEEP PAGE"):
//
//   1. title + tier badge + ONE-sentence statement (the badge carries the per-page truth,
//      so prose does not have to repeat it);
//   2. the interactive demo IMMEDIATELY (children) — show-don't-tell, before any prose;
//   3. an OPTIONAL one-line lead (`intro`) — deep pages trim their old multi-paragraph
//      intros to a single sentence; the long form moves to the crate README / SKILL.md;
//   4. a TIGHT capabilities list (bolded lead term + short clause, max ~5) — NOT the old
//      always-open 6 ring-cards;
//   5. "How this runs" as a SINGLE sentence whose authority is the badge, with an optional
//      reproduce-it line — NOT an always-open co-equal card;
//   6. caveats behind a CLOSED <details> ("Caveats & limitations") — the qualifier is
//      preserved (scripts/check-privacy-claims.sh stays green), just relocated off the
//      first screen;
//   7. UNIVERSAL depth links — every deep page links its crate README + SKILL.md (the
//      reference material moved off-site lives there), alongside any page-specific CTAs.
//
// Every content block is an OPTIONAL prop: a simple surface renders just statement + demo +
// a one-line note. Reference-grade detail (full capability matrices, edge-case boundaries,
// feature-gating minutiae) belongs in the README / SKILL.md / research, NOT here.

import type { ReactNode } from "react";
import Link from "next/link";
import { ArrowLeft, BookOpen, Check, ExternalLink, FileText } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { type Tier, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export interface SurfaceContentProps {
  icon: LucideIcon;
  title: string;
  /** One-line capability statement. */
  statement: string;
  tier: Tier;
  /** Optional extra badge label (e.g. "Native-only", "Research-grade"). */
  extraBadge?: string;
  /**
   * Optional one-line (1–2 sentence) lead. Keep it short — the long-form intro,
   * full matrices and edge-case boundaries move to the crate README / SKILL.md.
   */
  intro?: ReactNode;
  /** The concrete capabilities this surface offers — keep to ≤5, bolded lead term. */
  capabilities?: { title: string; body: string }[];
  /** "How this runs" — a SINGLE sentence; its authority is the tier badge. */
  runsNote?: ReactNode;
  /** Optional reproduce-it line shown beside the runs note (e.g. a cargo command). */
  reproduce?: string;
  /**
   * Honest caveat / what is NOT covered — rendered behind a CLOSED <details>.
   * The qualifier is preserved (privacy-claims gate), just off the first screen.
   */
  caveat?: ReactNode;
  /** Label for the caveat disclosure (default "Caveats & limitations"). */
  caveatLabel?: string;
  /** Crate README link — the moved-off-site reference material lives here. */
  readmeHref?: string;
  /** SKILL.md link — the usage guide for this surface. */
  skillHref?: string;
  /** Optional CTA links beyond the README / SKILL / source links. */
  links?: { href: string; label: string; external?: boolean }[];
  /** Anything extra (e.g. an embedded REPL or table) — shown IMMEDIATELY after the header. */
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
  reproduce,
  caveat,
  caveatLabel = "Caveats & limitations",
  readmeHref,
  skillHref,
  links,
  children,
}: SurfaceContentProps) {
  return (
    <div className="space-y-8">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/capabilities">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to Capabilities
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

      {/* Show, then tell: the interactive demo comes IMMEDIATELY after the header. */}
      {children}

      {intro && (
        <section className="measure space-y-3 text-muted-foreground">{intro}</section>
      )}

      {capabilities && capabilities.length > 0 && (
        <section className="space-y-3">
          <h2 className="text-lg font-semibold">What it does</h2>
          <ul className="space-y-2">
            {capabilities.map((c) => (
              <li key={c.title} className="flex gap-2.5 text-sm">
                <Check
                  className="mt-0.5 size-4 shrink-0 text-[var(--success)]"
                  aria-hidden="true"
                />
                <span className="text-muted-foreground">
                  <span className="font-semibold text-foreground">{c.title}</span>
                  {" — "}
                  {c.body}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {(runsNote || reproduce) && (
        <section className="space-y-2 text-sm text-muted-foreground">
          {runsNote && <p>{runsNote}</p>}
          {reproduce && (
            <p>
              Reproduce:{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-[12px] text-foreground">
                {reproduce}
              </code>
            </p>
          )}
        </section>
      )}

      {caveat && (
        <details className="group rounded-xl border bg-muted/30 px-4 py-3 text-sm">
          <summary className="cursor-pointer select-none font-medium text-foreground/90">
            {caveatLabel}
          </summary>
          <div className="mt-3 space-y-2 text-muted-foreground">{caveat}</div>
        </details>
      )}

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
        {readmeHref && (
          <Button asChild variant="outline" size="sm">
            <a href={readmeHref} target="_blank" rel="noopener noreferrer">
              <FileText className="size-4" aria-hidden="true" />
              Crate README
              <ExternalLink className="size-3.5 opacity-60" aria-hidden="true" />
            </a>
          </Button>
        )}
        {skillHref && (
          <Button asChild variant="outline" size="sm">
            <a href={skillHref} target="_blank" rel="noopener noreferrer">
              <BookOpen className="size-4" aria-hidden="true" />
              SKILL.md
              <ExternalLink className="size-3.5 opacity-60" aria-hidden="true" />
            </a>
          </Button>
        )}
        <Button asChild variant="outline" size="sm">
          <a
            href="https://github.com/sparq-org/sparq"
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
