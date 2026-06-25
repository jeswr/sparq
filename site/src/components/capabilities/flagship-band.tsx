// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign FLAGSHIP band (server component).
//
// Faithful to the approved mockup (web-capabilities.html §FLAGSHIPS): the three flagship demos
// promoted to a hero band ("start here") as large depth cards, BEFORE the long tail. Each card
// is a real Surface from FLAGSHIPS (data/surfaces.ts) — real title, blurb, tier badge, and the
// real /showcase/<slug> route. The lead card gets extra glow + a span. NO fabricated metrics:
// the card shows only the honest blurb + tier badge; the proof itself runs on the linked page.

import Link from "next/link";
import { ArrowRight, Star } from "lucide-react";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { FLAGSHIPS, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export function FlagshipBand() {
  return (
    <section aria-labelledby="flagships-heading" className="space-y-5">
      <div className="space-y-2">
        <p className="kicker">Start here</p>
        <h2 id="flagships-heading" className="display-2 font-semibold">
          Three proofs that the breadth is real
        </h2>
        <p className="measure text-[15px] text-muted-foreground">
          Not feature checkboxes — complete, runnable demos that each prove one hard
          thing. Open one and walk away having seen it work.
        </p>
      </div>

      <ul className="grid gap-4 lg:grid-cols-[1.25fr_1fr_1fr]">
        {FLAGSHIPS.map((surface, i) => {
          const Icon = surface.icon;
          const lead = i === 0;
          return (
            <li key={surface.slug} className={cn(lead && "lg:row-span-1")}>
              <Link
                href={surface.href}
                className={cn(
                  "group relative flex h-full flex-col overflow-hidden rounded-[var(--radius-2xl)] border bg-card p-6 transition-all duration-200",
                  "hover:-translate-y-0.5 hover:border-primary/45 hover:shadow-elevation-glow",
                  lead &&
                    "bg-[radial-gradient(110%_90%_at_100%_0%,color-mix(in_oklch,var(--primary)_16%,transparent),transparent_55%),radial-gradient(90%_90%_at_0%_100%,color-mix(in_oklch,var(--chart-5)_12%,transparent),transparent_55%),var(--card)]",
                )}
              >
                <span className="absolute right-5 top-5 inline-flex items-center gap-1 font-mono text-[11px] text-primary">
                  <Star className="size-3 fill-current" aria-hidden />
                  {lead && "flagship"}
                </span>

                <span className="flex size-11 items-center justify-center rounded-[var(--radius-md)] border border-primary/30 bg-primary/15 text-primary">
                  <Icon className="size-5" aria-hidden />
                </span>

                <h3 className="mt-4 text-lg font-semibold tracking-tight">
                  {surface.title}
                </h3>
                <p className="mt-2 flex-1 text-sm text-muted-foreground">
                  {surface.blurb}
                </p>

                <div className="mt-4 flex items-center justify-between gap-3">
                  <Badge
                    variant={TIER_VARIANT[surface.tier]}
                    className="h-5 px-2 text-[11px]"
                  >
                    {TIER_LABEL[surface.tier]}
                  </Badge>
                  <span className="inline-flex items-center gap-1.5 text-[13.5px] font-semibold text-primary">
                    Open demo
                    <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-0.5" />
                  </span>
                </div>
              </Link>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
