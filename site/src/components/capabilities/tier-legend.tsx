// [OPUS-4.8] sq-vw3ax — bold /capabilities redesign tier LEGEND (server component).
//
// Faithful to the approved mockup (web-capabilities.html §LEGEND): the honesty contract made
// first-class — a legend of the tier badges with a one-line gloss each. DRIVEN BY REAL DATA:
// it lists exactly the tiers that ACTUALLY appear across the surface set (data/surfaces.ts), in
// the canonical Tier order, so it can never advertise a badge no surface uses or omit one that
// does. The gloss explains what each badge promises about how that surface runs in-tab.

import { ALL_SURFACES, TIER_LABEL, TIER_VARIANT, type Tier } from "@/data/surfaces";
import { Badge } from "@/components/ui/badge";

/** Canonical tier order (matches the Tier union) for a stable legend. */
const TIER_ORDER: Tier[] = [
  "live",
  "live-new-wasm",
  "live-bbjs",
  "live-sim",
  "hosted",
  "native",
  "walkthrough",
  "soon",
];

/** One-line honest gloss per tier — what the badge promises about execution. */
const TIER_GLOSS: Record<Tier, string> = {
  live: "shipped wasm, running in your tab now",
  "live-new-wasm": "an additional wasm bundle, lazy-loaded on expand",
  "live-bbjs": "in-tab proving via 3rd-party bb.js UltraHonk",
  "live-sim": "a faithful in-tab JS simulation of the native protocol",
  hosted: "a sparq-server endpoint",
  native: "a built, opt-in Rust crate with code and docs linked from this static site",
  walkthrough: "captured, verbatim engine output replayed (no backend on a static site)",
  soon: "not built yet — an honest placeholder",
};

export function TierLegend() {
  const present = new Set(ALL_SURFACES.map((s) => s.tier));
  const tiers = TIER_ORDER.filter((t) => present.has(t));

  return (
    <section aria-labelledby="surfaces-heading" className="space-y-4">
      <div className="space-y-2">
        <p className="kicker">The full surface set</p>
        <h2 id="surfaces-heading" className="display-2 font-semibold">
          Every capability, five themes
        </h2>
        <p className="measure text-[15px] text-muted-foreground">
          Each tile is a real surface. The badge is the honest contract for how that
          surface runs on this static site.
        </p>
      </div>

      <ul className="flex flex-wrap gap-x-5 gap-y-2.5 rounded-[var(--radius-lg)] border bg-card/60 px-4 py-3.5">
        {tiers.map((t) => (
          <li
            key={t}
            className="inline-flex items-center gap-2 text-[12.5px] text-muted-foreground"
          >
            <Badge
              variant={TIER_VARIANT[t]}
              className="h-5 px-2 text-[11px]"
            >
              {TIER_LABEL[t]}
            </Badge>
            {TIER_GLOSS[t]}
          </li>
        ))}
      </ul>
    </section>
  );
}
