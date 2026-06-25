// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign: the per-theme accent map.
//
// The approved mockup (sparq-design-system/proposals/web-capabilities.html) gives each of the
// five capability themes a distinct accent drawn from the existing OKLCH chart palette
// (--chart-1…5) so the lanes read as five distinct PLACES, scannable without reading every
// blurb. This is a single source of truth keyed by the theme's stable anchor id (data/surfaces.ts
// `GROUPS[].id`). NO new hue is invented — every value is an existing `--chart-*` token, which
// already carries light + dark variants, so the accents follow the theme automatically.

/** The `--chart-*` CSS variable each theme's lane is accented with (mockup order). */
export const LANE_ACCENT: Record<string, string> = {
  "query-data": "var(--chart-1)",
  "reason-validate": "var(--chart-2)",
  "search-genai": "var(--chart-3)",
  privacy: "var(--chart-5)",
  "serve-embed": "var(--chart-4)",
};

/** Fallback accent for any theme id not in the map (keeps the brand teal). */
export const DEFAULT_ACCENT = "var(--primary)";

export function laneAccent(themeId: string): string {
  return LANE_ACCENT[themeId] ?? DEFAULT_ACCENT;
}
