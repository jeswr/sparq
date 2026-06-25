// [OPUS-4.8] sq-vw3ax.5 — the bold homepage "See it work" flagship card, faithful to the
// mockup's `.flag` (sparq-design-system/proposals/web-home.html): an icon tile + tier badge
// row, a starred title, the blurb, and an "Open demo →" affordance, with the hover-lift +
// radial-sheen treatment from the merged foundation (shadow-elevation-2, the `::before`
// glow rendered here as a sibling layer). Data is the real FLAGSHIPS row from data/surfaces.ts
// (route, blurb, tier all verbatim) — no fabricated copy. Home-scoped presentational only.

import Link from "next/link";
import { ArrowRight, Star } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { TIER_LABEL, TIER_VARIANT, type Surface } from "@/data/surfaces";

export function FlagshipCard({ surface }: { surface: Surface }) {
  const Icon = surface.icon;
  return (
    <Link
      href={surface.href}
      className="group block rounded-xl focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background"
    >
      <Card className="relative h-full overflow-hidden p-6 transition-all duration-200 group-hover:-translate-y-0.5 group-hover:shadow-elevation-2 group-hover:ring-primary/40">
        {/* Radial sheen — the mockup's `.flag::before`, top-right teal glow on hover. */}
        <div
          className="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-300 group-hover:opacity-100"
          style={{
            background:
              "radial-gradient(28rem 16rem at 100% 0%, color-mix(in oklch, var(--primary) 16%, transparent), transparent 60%)",
          }}
          aria-hidden
        />
        <div className="relative flex items-start justify-between gap-2">
          <span className="flex size-10 items-center justify-center rounded-lg bg-teal-soft text-primary">
            <Icon className="size-5" aria-hidden />
          </span>
          <Badge variant={TIER_VARIANT[surface.tier]}>
            {TIER_LABEL[surface.tier]}
          </Badge>
        </div>
        <h3 className="relative mt-4 flex items-center gap-1.5 text-base font-semibold">
          <Star className="size-4 fill-[var(--warning)] text-[var(--warning)]" aria-hidden />
          {surface.title}
        </h3>
        <p className="relative mt-2 text-sm leading-relaxed text-muted-foreground">
          {surface.blurb}
        </p>
        <span className="relative mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-primary">
          Open demo
          <ArrowRight
            className="size-4 transition-transform group-hover:translate-x-0.5"
            aria-hidden
          />
        </span>
      </Card>
    </Link>
  );
}
