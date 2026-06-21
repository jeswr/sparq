// [OPUS-4.8] sq-3hrc — shared layout for a real (non-placeholder) surface page.
//
// [OPUS-4.8] sq-vw3ax.4 — rebuilt SCAN-FIRST per research/website-redesign.md §4
// ("CAPABILITY DEEP PAGE"). The old template was a rigid sequence of always-open
// cards (intro prose → 6-card capability grid → "How this runs" card → "Honest
// caveats" card) that could not shrink — the density root cause the redesign
// answers. The new order leads with the demo and makes every content block an
// OPTIONAL prop, so a simple surface renders just statement + demo + one-line note:
//
//   (1) title + tier badge + ONE-sentence statement
//   (2) the interactive demo IMMEDIATELY (`children`)
//   (3) a tight Capabilities list (bolded lead term + short clause, max ~5)
//   (4) "How this runs" as a SINGLE sentence whose authority is the tier badge
//   (5) caveats as a CLOSED <details>
//
// Content is grounded in each surface's skills/*/SKILL.md. Honesty discipline:
// the tier badge (TIER_LABEL / TIER_VARIANT) is the load-bearing claim — the
// runsNote sentence only restates it, it must not over-claim past the badge.

import type { ReactNode } from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { type Tier, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export interface SurfaceContentProps {
  icon: LucideIcon;
  title: string;
  /** One-line capability statement — always shown under the title. */
  statement: string;
  tier: Tier;
  /** Optional extra badge label (e.g. "Native-only", "Research-grade"). */
  extraBadge?: string;
  /**
   * The interactive demo. Rendered IMMEDIATELY after the header so the proof is
   * the first thing a visitor scans. Optional — a surface with no in-tab demo
   * simply omits it.
   */
  children?: ReactNode;
  /**
   * Optional lead prose. Most deep pages no longer need this — the demo plus the
   * capabilities list carry the page. Kept for surfaces that genuinely need a
   * sentence of framing before the list.
   */
  intro?: ReactNode;
  /**
   * The concrete capabilities this surface offers, as a tight scan-list. Each
   * entry is a bolded lead term + a short clause. Keep it to ~5 — long lists are
   * the density the redesign cuts. Optional: omit for a bare statement+demo page.
   */
  capabilities?: { term: string; body: string }[];
  /**
   * "How this runs" — a SINGLE sentence whose authority is the tier badge above
   * (e.g. "Runs live in your tab via the lean wasm bundle"). Optional. Renders as
   * one muted line, NOT a card.
   */
  runsNote?: ReactNode;
  /**
   * Honest caveat / what is NOT covered. Rendered inside a CLOSED <details> so it
   * is available without dominating the page. Optional.
   */
  caveat?: ReactNode;
  /** Optional CTA links beyond the source link. */
  links?: { href: string; label: string; external?: boolean }[];
}

export function SurfaceContent({
  icon: Icon,
  title,
  statement,
  tier,
  extraBadge,
  children,
  intro,
  capabilities,
  runsNote,
  caveat,
  links,
}: SurfaceContentProps) {
  return (
    <div className="space-y-8">
      {/*
        Back-link targets the overview. The redesign's /capabilities gallery
        (sq-vw3ax.3) is the intended parent once it lands on main; until then this
        points to "/" so the link never resolves to a 404 on the static export.
      */}
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to overview
        </Link>
      </Button>

      {/* (1) title + tier badge + ONE-sentence statement */}
      <header className="space-y-3">
        <div className="flex items-start gap-3">
          <span className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Icon className="size-5" aria-hidden="true" />
          </span>
          <div className="space-y-1">
            <div className="flex flex-wrap items-center gap-2">
              <h1 className="text-2xl font-semibold">{title}</h1>
              <Badge variant={TIER_VARIANT[tier]}>{TIER_LABEL[tier]}</Badge>
              {extraBadge && <Badge variant="muted">{extraBadge}</Badge>}
            </div>
            <p className="text-muted-foreground">{statement}</p>
          </div>
        </div>
      </header>

      {/* Optional one-sentence framing, before the demo. */}
      {intro && (
        <section className="measure space-y-3 text-sm text-muted-foreground">
          {intro}
        </section>
      )}

      {/* (2) the interactive demo IMMEDIATELY */}
      {children}

      {/* (3) a tight Capabilities list — bolded lead term + short clause, max ~5 */}
      {capabilities && capabilities.length > 0 && (
        <section className="space-y-3">
          <h2 className="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
            Capabilities
          </h2>
          <ul className="measure space-y-2 text-sm">
            {capabilities.map((c) => (
              <li key={c.term} className="flex gap-2">
                <span
                  className="mt-2 size-1.5 shrink-0 rounded-full bg-primary/60"
                  aria-hidden="true"
                />
                <span className="text-muted-foreground">
                  <strong className="font-semibold text-foreground">
                    {c.term}
                  </strong>{" "}
                  — {c.body}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {/* (4) "How this runs" as a SINGLE sentence — authority is the badge above */}
      {runsNote && (
        <section className="measure text-sm text-muted-foreground">
          <span className="font-medium text-foreground">How this runs: </span>
          {runsNote}
        </section>
      )}

      {/* (5) caveats as a CLOSED <details> */}
      {caveat && (
        <details className="measure rounded-lg border bg-muted/20 p-3 text-sm text-muted-foreground">
          <summary className="cursor-pointer select-none font-medium text-foreground">
            Caveats &amp; what is not covered
          </summary>
          <div className="mt-3 space-y-2">{caveat}</div>
        </details>
      )}

      <section className="flex flex-wrap gap-2 pt-2">
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
