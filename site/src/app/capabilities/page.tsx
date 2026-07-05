import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax — /capabilities: the BOLD redesign on the merged dark-first foundation,
// faithful to the approved mockup (sparq-design-system/proposals/web-capabilities.html).
//
// The page is an opinionated, scannable showcase of the full surface set, built entirely from
// the single GROUPS / FLAGSHIPS source (data/surfaces.ts) — every name, blurb, tier badge, and
// route is REAL; every hero figure is DERIVED from that data (hero-stats.ts), never hardcoded.
// Structure (mockup order):
//   1. HERO — display heading + brand gradient + derived stat strip + theme jump-rail + the
//      REAL prefilled query (→ run it live in the /app workbench; sq-4hiqe: /try was removed).
//   2. FLAGSHIP band — the three flagships promoted to "start here" depth cards.
//   3. LEGEND — the tier honesty contract, first-class.
//   4. five LANES — each a sticky numbered spine + a per-theme accent + a tile grid; the
//      tiles reuse the lazy-mount machinery (the #1 risk, unchanged); the privacy lane carries
//      the explicit research-grade caveat strip.
// The atmospheric .bg-atmos/.bg-grid foundation utilities are mounted once (OPT-IN, pure CSS).
import { capabilityThemes } from "@/data/capabilities";
import { CapabilityHero } from "@/components/capabilities/capability-hero";
import { FlagshipBand } from "@/components/capabilities/flagship-band";
import { TierLegend } from "@/components/capabilities/tier-legend";
import { CapabilityLane } from "@/components/capabilities/capability-lane";

export const metadata: Metadata = {
  title: "Capabilities",
  description:
    "Every sparq capability in one bold, scannable showcase — grouped into five themes. Open a live deep page, expand a demo in place, or open a flagship end-to-end. Each surface runs the real engine and is honestly tier-labelled by how it runs.",
};

export default function CapabilitiesPage() {
  const themes = capabilityThemes();

  return (
    <>
      {/* Atmospheric backdrop — fixed, inert, pure CSS (foundation utilities). */}
      <div className="bg-atmos" aria-hidden />
      <div className="bg-grid" aria-hidden />

      <div className="space-y-16">
        <CapabilityHero />

        <FlagshipBand />

        <div className="space-y-2">
          <TierLegend />
          <div className="mt-2">
            {themes.map((theme, i) => (
              <CapabilityLane
                key={theme.group.id}
                theme={theme}
                index={i}
                total={themes.length}
              />
            ))}
          </div>
        </div>
      </div>
    </>
  );
}
