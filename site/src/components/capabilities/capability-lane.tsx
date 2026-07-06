// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign LANE (server component).
//
// Faithful to the approved mockup (web-capabilities.html §LANE): each theme is a horizontal
// "lane" with a sticky NUMBERED spine (01 / 05) + a per-theme accent (chart-1…5) + a tile grid,
// so the five capability areas read as five distinct places — scannable without reading every
// blurb. The tiles are the existing CapabilityRowItem (lazy-mount machinery preserved); the
// query-data lane carries the "open the workbench" affordance, and the privacy lane carries the
// explicit research-grade caveat strip (LIVE privacy-claims honesty, made loud not hidden).

import { AlertTriangle, PlayCircle } from "lucide-react";

import { type CapabilityTheme } from "@/data/capabilities";
import { withBasePath } from "@/lib/base-path";
import { CapabilityRowItem } from "@/components/capabilities/capability-row";
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

      {/* The tile grid. */}
      <div className="grid gap-3 sm:grid-cols-2">
        {rows.map(({ surface, kind }) => (
          <CapabilityRowItem
            key={surface.slug}
            slug={surface.slug}
            kind={kind}
            accent={accent}
          />
        ))}

        {/* Privacy lane: the explicit research-grade caveat strip (made loud, not a footnote). */}
        {group.id === "privacy" && (
          <div className="flex items-start gap-3 rounded-[var(--radius-lg)] border border-dashed border-[color-mix(in_oklch,var(--warning)_45%,var(--border))] bg-[color-mix(in_oklch,var(--warning)_7%,transparent)] px-4 py-3 text-[12.7px] text-muted-foreground sm:col-span-2">
            <AlertTriangle
              className="mt-0.5 size-4 shrink-0 text-[var(--warning)]"
              aria-hidden
            />
            <span>
              <span className="font-semibold text-foreground">
                Research-grade, labelled as such.
              </span>{" "}
              The v1 ZK verifier is not externally audited — sound as landed under its
              stated threat model, pending re-review. MPC runs as a faithful in-tab
              simulation of the native protocol, not a proof of correctness. Indicative
              engineering, not an audited cryptographic guarantee.
            </span>
          </div>
        )}
      </div>
    </section>
  );
}
