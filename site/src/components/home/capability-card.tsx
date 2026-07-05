// [OPUS-4.8] sq-vw3ax.5 — the bold homepage capability THEME card (the mockup's `.cap`):
// a numbered chip + theme title, the theme description, and a wrapped list of the theme's
// REAL surfaces, each prefixed by its honesty-tier dot. All data is the single GROUPS source
// (data/surfaces.ts) — theme label, description and every surface title/tier verbatim, links
// resolved exactly as the prior Home did (deep pages keep their route, others go to the
// /capabilities#theme anchor). The privacy theme's research-grade caveat already lives in its
// GROUPS description, so the not-externally-audited caveat is rendered here unchanged.

import Link from "next/link";

import { Card } from "@/components/ui/card";
import {
  DEEP_PAGE_SLUGS,
  capabilityAnchor,
} from "@/data/capabilities";
import { type Surface, type SurfaceGroup } from "@/data/surfaces";
import { TierDot } from "@/components/home/tier-dot";

/** A surface's link on Home → its post-redesign home (a deep page, or /capabilities#theme). */
function surfaceHref(surface: Surface, groupId: string): string {
  if (DEEP_PAGE_SLUGS.has(surface.slug)) return surface.href;
  return capabilityAnchor(groupId);
}

export function CapabilityCard({
  group,
  index,
}: {
  group: SurfaceGroup;
  index: number;
}) {
  const num = String(index + 1).padStart(2, "0");
  return (
    <Card className="flex h-full flex-col p-5 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-elevation-2 hover:ring-primary/30">
      <div className="flex items-center gap-2.5">
        <span className="grid size-6 flex-none place-items-center rounded-md font-mono text-xs text-primary ring-1 ring-primary/35">
          {num}
        </span>
        <h3 className="text-base font-semibold">
          <Link href={capabilityAnchor(group.id)} className="hover:text-primary">
            {group.label}
          </Link>
        </h3>
      </div>
      <p className="mt-3 flex-1 text-sm leading-relaxed text-muted-foreground">
        {group.description}
      </p>
      <ul className="mt-4 flex flex-wrap gap-x-2.5 gap-y-2 border-t border-[var(--hairline)] pt-3.5">
        {group.surfaces.map((surface) => (
          <li key={surface.slug}>
            <Link
              href={surfaceHref(surface, group.id)}
              className="inline-flex items-center gap-1.5 text-sm text-foreground/80 underline-offset-4 hover:text-primary hover:underline"
            >
              <TierDot tier={surface.tier} />
              {surface.title}
            </Link>
          </li>
        ))}
      </ul>
    </Card>
  );
}
