// [OPUS-4.8] sq-vw3ax.5 — the per-surface honesty-tier DOT used inline in the homepage
// capability cards (the mockup's `.tdot`). The colour is DERIVED from the same TIER_VARIANT
// map that drives every honesty badge on the site, so the dot can never disagree with the
// badge a surface's own page renders. Colour is NEVER the only signal — every dot carries an
// aria-label and sits beside the surface's text label + (in the legend) the written tier name.

import { TIER_LABEL, TIER_VARIANT, type Tier } from "@/data/surfaces";

// Map the four badge variants to their token colour. success/warning/muted/default are the
// only variants TIER_VARIANT yields, so this is total over the tier space.
const VARIANT_COLOR: Record<
  (typeof TIER_VARIANT)[Tier],
  string
> = {
  success: "var(--success)",
  warning: "var(--warning)",
  muted: "var(--muted-foreground)",
  default: "var(--primary)",
};

export function tierDotColor(tier: Tier): string {
  return VARIANT_COLOR[TIER_VARIANT[tier]];
}

export function TierDot({ tier }: { tier: Tier }) {
  return (
    <span
      className="inline-block size-1.5 shrink-0 rounded-full align-middle"
      style={{ backgroundColor: tierDotColor(tier) }}
      role="img"
      aria-label={`Honesty tier: ${TIER_LABEL[tier]}`}
    />
  );
}
