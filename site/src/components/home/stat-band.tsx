// [OPUS-4.8] sq-vw3ax.11 — the slim structural stat band that sits UNDER the split hero.
//
// The four-cell stat strip used to live inside the hero; the hero is now a split "text + live
// runner" layout, so the strip moves down here as a slim band. Same content, same single IA
// source — no number is a performance/timing claim.
//
// REAL DATA / HONESTY: every figure is a STRUCTURAL fact derived from the canonical IA source
// (SPARQL query forms + RDF text formats the engine ships, the surface/theme counts from
// data/surfaces.ts, and "0 servers" — the engine runs in-tab). Nothing is fabricated.

import { ALL_SURFACES, GROUPS } from "@/data/surfaces";
import { RDF_TEXT_FORMATS } from "@/data/home-facts";

// Derived from the single IA source so the counts can never drift from the nav / gallery / Cmd-K.
const SURFACE_COUNT = ALL_SURFACES.length;
const THEME_COUNT = GROUPS.length;
const FORMAT_COUNT = RDF_TEXT_FORMATS.length;

interface Stat {
  /** The leading number/value (mono, the `--primary` accent unit lives in `unit`). */
  value: string;
  unit?: string;
  /** Whether the value should render after the unit (e.g. "4 formats"). */
  unitLeads?: boolean;
  label: string;
}

const STATS: Stat[] = [
  {
    value: "SPARQL",
    unit: "1.1/1.2",
    label: "SELECT · ASK · CONSTRUCT · DESCRIBE · UPDATE",
  },
  {
    value: "formats",
    unit: String(FORMAT_COUNT),
    unitLeads: true,
    label: RDF_TEXT_FORMATS.join(" · "),
  },
  {
    value: "surfaces",
    unit: String(SURFACE_COUNT),
    unitLeads: true,
    label: `across ${THEME_COUNT} capability themes`,
  },
  {
    value: "servers",
    unit: "0",
    unitLeads: true,
    label: "the engine runs entirely in your tab",
  },
];

export function StatBand() {
  return (
    <dl
      className="grid grid-cols-2 gap-px overflow-hidden rounded-xl bg-border ring-1 ring-foreground/10 md:grid-cols-4"
      aria-label="What the engine answers in-tab"
    >
      {STATS.map((stat) => (
        <div
          key={stat.label}
          className="bg-[color-mix(in_oklch,var(--card)_80%,var(--background))] px-5 py-4"
        >
          <dt className="block font-mono text-2xl font-semibold tracking-tight">
            {stat.unitLeads ? (
              <>
                <span className="text-primary">{stat.unit}</span> {stat.value}
              </>
            ) : (
              <>
                {stat.value}{" "}
                {stat.unit ? <span className="text-primary">{stat.unit}</span> : null}
              </>
            )}
          </dt>
          <dd className="mt-1 text-xs text-muted-foreground">{stat.label}</dd>
        </div>
      ))}
    </dl>
  );
}
