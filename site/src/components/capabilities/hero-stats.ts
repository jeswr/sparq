// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign: the hero stat strip, computed from REAL
// data (data/surfaces.ts), NEVER hardcoded. The approved mockup showed illustrative figures
// ("16 surfaces · 5 live · 3 flagships"); here every number is DERIVED from GROUPS / FLAGSHIPS /
// the tier set at module load, so a future surface/tier edit re-derives the strip and the page
// can never drift into a fabricated count.

import { ALL_SURFACES, FLAGSHIPS, GROUPS, type Tier } from "@/data/surfaces";

/** Tiers that actually EXECUTE the engine in the visitor's tab (wasm or 3rd-party wasm/JS),
 *  as opposed to a captured-output walkthrough / hosted / not-yet-built surface. This is the
 *  honest "runs in your tab" set — derived from the Tier union, not a magic number. */
const IN_TAB_TIERS: ReadonlySet<Tier> = new Set<Tier>([
  "live",
  "live-new-wasm",
  "live-bbjs",
  "live-sim",
]);

export interface HeroStat {
  /** The numeric value (rendered in mono). */
  value: string;
  /** A short unit suffix shown in the primary colour (optional). */
  unit?: string;
  /** The caption beneath the figure. */
  caption: string;
}

/** The number of surfaces that run the engine live in-tab (any in-tab tier). */
export const IN_TAB_SURFACE_COUNT = ALL_SURFACES.filter((s) =>
  IN_TAB_TIERS.has(s.tier),
).length;

export const SURFACE_COUNT = ALL_SURFACES.length;
export const THEME_COUNT = GROUPS.length;
export const FLAGSHIP_COUNT = FLAGSHIPS.length;

/** The hero stat strip — all four figures derived from the real data model. */
export const HERO_STATS: HeroStat[] = [
  {
    value: String(SURFACE_COUNT),
    unit: "surfaces",
    caption: `across ${THEME_COUNT} capability themes`,
  },
  {
    value: String(IN_TAB_SURFACE_COUNT),
    unit: "in-tab",
    caption: "run the real engine in your browser",
  },
  {
    value: String(FLAGSHIP_COUNT),
    unit: "flagships",
    caption: "ZK · MPC · Solid, end-to-end",
  },
  {
    value: "1.1 / 1.2",
    caption: "SPARQL + RDF-star, native Rust",
  },
];
