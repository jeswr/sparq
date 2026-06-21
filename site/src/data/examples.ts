// [OPUS-4.8] sq-vw3ax.6 — the /examples gallery model: the curated "show me it working"
// set, derived from the single FLAGSHIPS source (data/surfaces.ts) plus the live REPL.
//
// /examples is the curated "show me it working" gallery (research/website-redesign.md §4):
// a 2-col grid of LARGE demo cards = the 3 flagships (zk-car-hire, mpc-100k, solid-pairs) +
// a "Live REPL" card. Each card carries a one-line OUTCOME (what you walk away having proven),
// a tier badge, and an [Open demo] CTA. The /showcase/<slug> pages are the detail targets
// (rebuilt demo-first); the REPL card targets /try.
//
// Keeping the three flagship items derived from FLAGSHIPS (rather than re-listed) means a
// re-word of a flagship's title/blurb/tier in surfaces.ts flows straight into this gallery, the
// Home "See it work" row, and Cmd-K together — no second source to drift.

import { PlayCircle, type LucideIcon } from "lucide-react";

import { FLAGSHIPS, type Tier } from "@/data/surfaces";

export interface ExampleCard {
  /** Stable key + the demo route the card opens. */
  href: string;
  title: string;
  /** One-line OUTCOME — what you walk away having seen proven (not a feature list). */
  outcome: string;
  tier: Tier;
  icon: LucideIcon;
  /** Flagships carry the ★ marker; the REPL card does not. */
  flagship: boolean;
}

// A per-flagship one-line OUTCOME (the "you walk away having proven X" line), keyed by slug.
// The flagship `blurb` in surfaces.ts is the longer descriptive sentence used elsewhere; the
// gallery wants the terse outcome, so it lives here, next to the gallery that consumes it.
const FLAGSHIP_OUTCOME: Record<string, string> = {
  "zk-car-hire": "Prove you may hire a car without revealing your documents.",
  "mpc-100k":
    "Learn only whether four incomes clear £100k — no salary, not even the total, revealed.",
  "solid-pairs":
    "One query, one Pod — a different result set per (user, app) pair, enforced live.",
};

const FLAGSHIP_CARDS: ExampleCard[] = FLAGSHIPS.map((f) => ({
  href: f.href,
  title: f.title,
  outcome: FLAGSHIP_OUTCOME[f.slug] ?? f.blurb,
  tier: f.tier,
  icon: f.icon,
  flagship: true,
}));

// The live REPL card — the single best proof artifact (the killer demo), opened as a large
// card alongside the flagships, targeting its own /try route.
const REPL_CARD: ExampleCard = {
  href: "/try",
  title: "Live SPARQL REPL",
  outcome:
    "Run real SPARQL against a sample graph — the Rust engine compiled to wasm, in your tab.",
  tier: "live",
  icon: PlayCircle,
  flagship: false,
};

/** The curated example gallery: the 3 flagships + the live REPL, in display order. */
export const EXAMPLE_CARDS: ExampleCard[] = [...FLAGSHIP_CARDS, REPL_CARD];
