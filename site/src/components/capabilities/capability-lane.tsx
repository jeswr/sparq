// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign LANE (server component).
//
// Faithful to the approved mockup (web-capabilities.html §LANE): each theme is a horizontal
// "lane" with a sticky NUMBERED spine (01 / 05) + a per-theme accent (chart-1…5) + a tile grid,
// so the five capability areas read as five distinct places — scannable without reading every
// blurb. The query-data lane carries the "open the workbench" affordance.
//
// [OPUS-4.8] sq-67qji (issue #1675) — the interactive tile grid + the expanded demo bodies are
// delegated to the CapabilityLaneGrid client component. It renders an opened demo's body at full
// lane width (a `lg:col-span-2` row that reclaims this spine's column) instead of expanding it
// inside a half-column tile, and carries the privacy lane's research-grade caveat strip. The
// lazy-mount machinery + honesty caveats are preserved there verbatim.

import { PlayCircle } from "lucide-react";

import { type CapabilityTheme } from "@/data/capabilities";
import { withBasePath } from "@/lib/base-path";
import { CapabilityLaneGrid } from "@/components/capabilities/capability-lane-grid";
import { laneAccent } from "@/components/capabilities/lane-accents";

export function CapabilityLane({
  theme,
  index,
  total,
}: {
  theme: CapabilityTheme;
  index: number;
  total: number;
}) {
  const { group, rows } = theme;
  const accent = laneAccent(group.id);
  const idx = String(index + 1).padStart(2, "0");
  const count = String(total).padStart(2, "0");

  return (
    <section
      id={group.id}
      // scroll-mt clears the sticky h-16 shell header when an in-page #anchor is targeted.
      className="scroll-mt-24 grid gap-7 border-t py-8 lg:grid-cols-[232px_1fr]"
    >
      {/* Sticky numbered spine — the lane's identity. */}
      <div className="lg:sticky lg:top-24 lg:self-start">
        <div className="font-mono text-[12px] tracking-wide text-primary">
          {idx} / {count}
        </div>
        <h3 className="mt-2.5 text-[22px] font-semibold tracking-tight">
          {group.label}
        </h3>
        <p className="mt-2.5 max-w-[30ch] text-[13.5px] text-muted-foreground">
          {group.description}
        </p>
        <span
          aria-hidden
          className="mt-3.5 block h-[3px] w-9 rounded-full"
          style={{ background: accent }}
        />
        {group.id === "query-data" && (
          // [OPUS-4.8] sq-4hiqe — /app is the single workbench (the /try REPL was removed). /app is
          // a separate overlaid Next app, so this is a HARD anchor, not a next/link soft nav.
          <a
            href={withBasePath("/app/")}
            className="mt-4 inline-flex items-center gap-1.5 text-[13px] font-semibold text-primary underline-offset-4 hover:underline"
          >
            <PlayCircle className="size-4" aria-hidden />
            Open the SPARQL workbench
          </a>
        )}
      </div>

      {/* The interactive tile grid + full-width demo bodies (client). A Surface's lucide `icon`
          is not serializable across the RSC boundary, so we hand down the slug + kind and the
          client resolves the surface. The fragment it returns lands its children as direct grid
          items of this section, so an opened demo body can span `lg:col-span-2` (sq-67qji). */}
      <CapabilityLaneGrid
        rows={rows.map(({ surface, kind }) => ({ slug: surface.slug, kind }))}
        accent={accent}
        groupId={group.id}
      />
    </section>
  );
}
