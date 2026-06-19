import * as React from "react";

import { cn } from "@/lib/utils";

/*
 * [OPUS-4.8] sq-jnh9 — shield + lightning logo CONCEPTS for issue #207.
 *
 * The user rejected every option in #207 and asked for a fresh "Claude Design"
 * pass on a shield + lightning-bolt motif (security + speed: the two pillars of
 * sparq — verifiable/ZK proofs and a fast SPARQL engine). These are CONCEPTS to
 * choose from; they do NOT replace the live favicon/header `Logo` yet.
 *
 * Palette: reused from the real site theme (src/app/globals.css) — the teal/cyan
 * privacy-first brand. Marks lean on `currentColor` so they inherit the
 * surrounding text colour and adapt to the class-based light/dark toggle; the
 * brand teal is pinned via the `--primary` token (and a sRGB hex fallback for a
 * standalone export, which has no theme context). The lightning bolt carries the
 * warm `--warning` accent (the existing spark colour, #f5a623) so the marks stay
 * in the established sparq accent language.
 *
 * Each mark uses viewBox="0 0 64 64" so it reads cleanly at favicon size (16/32px)
 * and scales up as a header mark. Geometry is hand-tuned for balanced negative
 * space; no editor cruft. The shield outline uses currentColor; the bolt is the
 * accent; the teal fills come from CSS custom properties.
 */

// Brand tokens, reused from globals.css. Hex fallbacks let an exported standalone
// SVG (favicon, social card) render correctly with no CSS-variable context.
const TEAL = "var(--primary, #2a8c95)";
const SPARK = "#f5a623"; // the established sparq accent (warning token / spark)

type MarkProps = React.ComponentProps<"svg">;

/**
 * Concept A — "Aegis Bolt".
 * A solid teal shield silhouette with the lightning bolt carved out as negative
 * space (the bolt is the absence of fill — a clean knockout that reads instantly
 * at 16px because it relies on the strongest possible contrast, fill vs. void).
 * The single most legible direction and the one given a full lockup below.
 */
export function MarkAegis({ className, ...props }: MarkProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      role="img"
      aria-label="sparq shield-bolt mark, concept Aegis"
      className={cn("text-foreground", className)}
      {...props}
    >
      {/* Shield body. evenodd lets the inner bolt subpath knock a hole. */}
      <path
        fill={TEAL}
        fillRule="evenodd"
        d="M32 3 L57 12 V32 C57 47 46 57 32 62 C18 57 7 47 7 32 V12 Z
           M36 16 L20 36 H30 L28 48 L44 28 H34 Z"
      />
    </svg>
  );
}

/**
 * Concept B — "Crest Strike".
 * An outlined shield (stroke = currentColor, so it's a clean line mark that works
 * on any background) with a bold filled lightning bolt in the warm accent sitting
 * inside it. Lighter, more "badge"-like; the outline keeps it airy at header size
 * while the solid bolt survives down to 16px.
 */
export function MarkCrest({ className, ...props }: MarkProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      role="img"
      aria-label="sparq shield-bolt mark, concept Crest"
      className={cn("text-foreground", className)}
      {...props}
    >
      <path
        fill="none"
        stroke="currentColor"
        strokeWidth={5}
        strokeLinejoin="round"
        d="M32 5 L55 13 V31 C55 45 45 55 32 60 C19 55 9 45 9 31 V13 Z"
      />
      <path fill={SPARK} d="M37 17 L22 37 H31 L29 49 L43 28 H34 Z" />
    </svg>
  );
}

/**
 * Concept C — "Split Shield".
 * The shield is split down a diagonal cleave that doubles as the lightning bolt:
 * the left half is brand teal, the right half is a lighter teal, and the jagged
 * seam between them IS the bolt, picked out with the warm accent. A single
 * geometric idea (one cut serves as both heraldic division and bolt) — distinct,
 * modern, no separate bolt object.
 */
export function MarkSplit({ className, ...props }: MarkProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      role="img"
      aria-label="sparq shield-bolt mark, concept Split"
      className={cn("text-foreground", className)}
      {...props}
    >
      {/* clip the two halves to the shield silhouette */}
      <defs>
        <clipPath id="sparq-split-shield">
          <path d="M32 3 L57 12 V32 C57 47 46 57 32 62 C18 57 7 47 7 32 V12 Z" />
        </clipPath>
      </defs>
      <g clipPath="url(#sparq-split-shield)">
        {/* whole shield in teal first */}
        <rect x="0" y="0" width="64" height="64" fill={TEAL} />
        {/* right half painted with a lighter teal up to the bolt seam */}
        <path
          fill="var(--accent-foreground, #1f6f78)"
          d="M40 0 L27 30 H37 L31 64 H64 V0 Z"
        />
        {/* the bolt seam itself, drawn in the warm accent for a crisp spark line */}
        <path fill={SPARK} d="M42 0 L29 30 H39 L33 64 H37 L45 28 H35 L48 0 Z" />
      </g>
      <path
        fill="none"
        stroke="currentColor"
        strokeOpacity={0.18}
        strokeWidth={2}
        d="M32 3 L57 12 V32 C57 47 46 57 32 62 C18 57 7 47 7 32 V12 Z"
      />
    </svg>
  );
}

/**
 * Concept D — "Spark Sigil".
 * A rounded-square "app tile" shield (the softened-square silhouette favicons read
 * best) with a teal field and a knockout bolt, plus two warm "speed lines" trailing
 * the bolt to read as velocity (the speed pillar). The most app-icon-native of the
 * set.
 */
export function MarkSigil({ className, ...props }: MarkProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      role="img"
      aria-label="sparq shield-bolt mark, concept Sigil"
      className={cn("text-foreground", className)}
      {...props}
    >
      {/* Tile + knockout bolt via evenodd. */}
      <path
        fill={TEAL}
        fillRule="evenodd"
        d="M16 6 H48 C54 6 58 10 58 16 V40 C58 52 46 58 32 60 C18 58 6 52 6 40 V16 C6 10 10 6 16 6 Z
           M39 16 L23 37 H33 L31 49 L45 28 H35 Z"
      />
      {/* speed lines, the accent spark trailing the bolt = velocity gesture */}
      <g fill={SPARK}>
        <rect x="9" y="22" width="11" height="3.6" rx="1.8" />
        <rect x="9" y="31" width="8" height="3.6" rx="1.8" />
      </g>
    </svg>
  );
}

/**
 * Full lockup for the strongest concept (Aegis): mark + monospace "sparq" wordmark,
 * matching the existing header typography (JetBrains Mono, weight 700). Wordmark is
 * currentColor so it follows the theme; the mark keeps its teal + carved bolt.
 */
export function LockupAegis({ className, ...props }: MarkProps) {
  return (
    <svg
      viewBox="0 0 232 64"
      role="img"
      aria-label="sparq"
      className={cn("text-foreground", className)}
      {...props}
    >
      <path
        fill={TEAL}
        fillRule="evenodd"
        d="M32 3 L57 12 V32 C57 47 46 57 32 62 C18 57 7 47 7 32 V12 Z
           M36 16 L20 36 H30 L28 48 L44 28 H34 Z"
      />
      <text
        x="74"
        y="44"
        fill="currentColor"
        fontFamily="ui-monospace,'JetBrains Mono','SF Mono',Menlo,monospace"
        fontSize="40"
        fontWeight={700}
        letterSpacing="-1"
      >
        sparq
      </text>
    </svg>
  );
}

/**
 * Full lockup for Crest (the line-mark direction), for an alternative comparison —
 * outlined shield + bolt with the wordmark beside it.
 */
export function LockupCrest({ className, ...props }: MarkProps) {
  return (
    <svg
      viewBox="0 0 232 64"
      role="img"
      aria-label="sparq"
      className={cn("text-foreground", className)}
      {...props}
    >
      <path
        fill="none"
        stroke="currentColor"
        strokeWidth={5}
        strokeLinejoin="round"
        d="M32 5 L55 13 V31 C55 45 45 55 32 60 C19 55 9 45 9 31 V13 Z"
      />
      <path fill={SPARK} d="M37 17 L22 37 H31 L29 49 L43 28 H34 Z" />
      <text
        x="74"
        y="44"
        fill="currentColor"
        fontFamily="ui-monospace,'JetBrains Mono','SF Mono',Menlo,monospace"
        fontSize="40"
        fontWeight={700}
        letterSpacing="-1"
      >
        sparq
      </text>
    </svg>
  );
}

export type Concept = {
  id: string;
  name: string;
  tagline: string;
  idea: string;
  Mark: React.ComponentType<MarkProps>;
  Lockup?: React.ComponentType<MarkProps>;
};

export const CONCEPTS: Concept[] = [
  {
    id: "aegis",
    name: "Aegis Bolt",
    tagline: "Solid shield, carved bolt",
    idea: "A solid teal shield with the lightning bolt knocked out as negative space — the bolt is pure void, so contrast is maximal and the mark stays legible at 16px. The strongest, most favicon-ready direction; shown with a full wordmark lockup.",
    Mark: MarkAegis,
    Lockup: LockupAegis,
  },
  {
    id: "crest",
    name: "Crest Strike",
    tagline: "Outlined badge, filled spark",
    idea: "An airy outlined shield (a clean currentColor line mark that drops onto any surface) cradling a bold accent-coloured bolt. More heraldic/badge-like; the solid bolt keeps it readable when the thin outline starts to thin out at very small sizes.",
    Mark: MarkCrest,
    Lockup: LockupCrest,
  },
  {
    id: "split",
    name: "Split Shield",
    tagline: "One cut, both meanings",
    idea: "The shield is cleaved by a single diagonal that IS the bolt — heraldic division and lightning in one stroke. Two teal tones meet at a warm spark seam; one geometric idea doing double duty, no separate bolt object.",
    Mark: MarkSplit,
  },
  {
    id: "sigil",
    name: "Spark Sigil",
    tagline: "App-tile shield + speed arc",
    idea: 'A softened-square "app tile" shield (the silhouette that reads best as an OS/PWA icon) in teal with a carved bolt, plus a faint accent arc sweeping out of the tile to evoke query speed. The most app-icon-native of the set.',
    Mark: MarkSigil,
  },
];
