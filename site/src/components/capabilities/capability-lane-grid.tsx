"use client";

// [OPUS-4.8] sq-67qji — the interactive body of a /capabilities lane (issue #1675).
//
// THE BUG. Each lane laid its tiles in a `grid sm:grid-cols-2` inside the lane's `1fr` column,
// and a demo used to expand IN its own tile — so on desktop a wide interactive demo (the ZK /
// MPC / GeoSPARQL result tables) was confined to (viewport − 232px spine − gaps)/2 ≈ ONE THIRD
// of the screen and was cramped and hard to use.
//
// THE FIX. This client component owns the lane's open/mounted state and renders, as siblings
// inside the lane <section>'s `lg:grid-cols-[232px_1fr]` grid:
//   1. the tile grid (unchanged 2-column tiles), and
//   2. the OPENED demo's body as a `lg:col-span-2` row BELOW the grid — reclaiming the
//      numbered-spine column so the demo gets the FULL content width, not a half-column tile.
// Because it returns a fragment (no wrapper DOM node), both land as direct grid items of the
// section, so `lg:col-span-2` spans the whole lane. A collapsed body is `display:none`, which
// removes it from grid layout entirely — so no phantom gap appears below the tiles when nothing
// is open.
//
// LAZY-MOUNT DISCIPLINE PRESERVED (the #1 risk, research/website-redesign.md §3). A demo body
// is only rendered once its slug has been opened (`mounted` set), and stays mounted — hidden
// with CSS on collapse — so re-expanding never re-fetches its code-split chunk. This is exactly
// the invariant e2e/capabilities-lazy.spec.ts + scripts/check-bundle.mjs assert.

import * as React from "react";
import { AlertTriangle } from "lucide-react";

import { type CapabilityKind } from "@/data/capabilities";
import {
  CapabilityRowItem,
  CapabilityDemoBody,
} from "@/components/capabilities/capability-row";
import { hasLazyDemo, type LazyDemoSlug } from "@/components/capabilities/lazy-demo";

/** Serializable row handed down from the server lane (a Surface's lucide `icon` cannot cross
 *  the RSC boundary, so we pass the slug + kind and resolve the surface client-side). */
export interface LaneRow {
  slug: string;
  kind: CapabilityKind;
}

export function CapabilityLaneGrid({
  rows,
  accent,
  groupId,
}: {
  rows: LaneRow[];
  accent: string;
  groupId: string;
}) {
  // At most one demo is expanded per lane; `mounted` accumulates every slug that has ever been
  // opened so its chunk is fetched at most once (collapse hides, never unmounts).
  const [openSlug, setOpenSlug] = React.useState<string | null>(null);
  const [mounted, setMounted] = React.useState<ReadonlySet<string>>(
    () => new Set(),
  );

  const toggle = React.useCallback((slug: string) => {
    setMounted((prev) =>
      prev.has(slug) ? prev : new Set(prev).add(slug),
    );
    setOpenSlug((cur) => (cur === slug ? null : slug));
  }, []);

  // The lane's demo slugs that actually have a registered lazy demo (mirrors the row guard in
  // CapabilityRowItem, so a data-drift "demo" surface without a demo never mounts a body).
  const demoSlugs = React.useMemo(
    () =>
      rows
        .filter((r) => r.kind === "demo")
        .map((r) => r.slug)
        .filter((s): s is LazyDemoSlug => hasLazyDemo(s)),
    [rows],
  );

  return (
    <>
      <div className="grid gap-3 sm:grid-cols-2">
        {rows.map((r) => (
          <CapabilityRowItem
            key={r.slug}
            slug={r.slug}
            kind={r.kind}
            accent={accent}
            open={openSlug === r.slug}
            onToggle={() => toggle(r.slug)}
          />
        ))}

        {/* Privacy lane: the explicit research-grade caveat strip (made loud, not a footnote).
            LIVE privacy-claims honesty — the copy stays qualified (not externally audited). */}
        {groupId === "privacy" && <PrivacyCaveatStrip />}
      </div>

      {/* Full-width demo bodies. Only opened slugs are mounted; a collapsed one is display:none
          (so it neither shows nor reserves a grid row). Each is a direct grid child of the
          section via this fragment, so `lg:col-span-2` gives it the full lane width. */}
      {demoSlugs
        .filter((slug) => mounted.has(slug))
        .map((slug) => (
          <CapabilityDemoBody
            key={slug}
            slug={slug}
            open={openSlug === slug}
            onCollapse={() => toggle(slug)}
          />
        ))}
    </>
  );
}

function PrivacyCaveatStrip() {
  return (
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
  );
}
