// [OPUS-4.8] sq-ixc3.10 (epic sq-ixc3) — the keyboard-first spine's COMMAND MODEL + pure
// builders.
//
// WHAT THIS IS. The GUI design (research/gui-design.md §A.3) makes the command palette the
// operational backbone: any tool, every named graph, recent queries, and the verbs
// import / connect / export / run / run-EXPLAIN / switch-workspace are one fuzzy keystroke
// away. The website Cmd-K (sq-vw3ax.1) is purely navigational — it jumps to surfaces/pages.
// This module is the extra OPERATIONAL layer: a typed command descriptor plus the PURE
// functions that turn live REPL state (recent queries, the dataset's graphs, the stored
// workspaces) into those descriptors.
//
// WHY A PURE MODULE. The React provider/registration wiring lives in the `.tsx` host
// (components/palette-commands.tsx); the *logic* — capping + de-duplicating the recent-query
// ring, formatting a graph row, labelling the workspace switcher — is plain data-in/data-out
// so it is unit-tested node-side (test/palette-commands.test.mjs) without a DOM, exactly like
// the other REPL helpers (repl-dataset.ts, results.ts). Icons are passed in by the caller so
// this stays free of the lucide-react import (keeps it framework-light and trivially testable).

import type { LucideIcon } from "lucide-react";

/**
 * One selectable operational command in the palette. `id` is stable (so cmdk keys + the
 * registry de-dupe on it); `run` performs the action then the palette closes. `keywords`
 * feed the fuzzy match alongside the title. `group` buckets it under a heading.
 */
export interface PaletteCommand {
  /** Stable identity — registry key + cmdk `value` disambiguator. */
  id: string;
  /** The heading this command renders under (e.g. "Actions", "Named graphs"). */
  group: PaletteCommandGroup;
  /** The primary, fuzzy-matched label. */
  title: string;
  /** Optional secondary line (a hint / the query text / the graph IRI). */
  blurb?: string;
  /** Extra fuzzy-match terms (verbs, synonyms) that should not dilute the title score. */
  keywords?: string[];
  /** The icon rendered in the row's leading slot. */
  icon: LucideIcon;
  /** Run the command. The palette closes first, then this fires. */
  run: () => void;
  /** True when the command is currently unavailable (rendered disabled, not hidden). */
  disabled?: boolean;
}

/**
 * The operational command headings, in render order. They sit ABOVE the navigational
 * groups (flagships / themes / pages) because in a workbench the live verbs are the spine.
 */
export type PaletteCommandGroup =
  | "Actions"
  | "Recent queries"
  | "Named graphs"
  | "Workspaces";

/** Fixed render order of the operational groups (matches `PaletteCommandGroup`). */
export const PALETTE_COMMAND_GROUP_ORDER: PaletteCommandGroup[] = [
  "Actions",
  "Recent queries",
  "Named graphs",
  "Workspaces",
];

/** How many distinct recent queries the spine remembers / offers. */
export const RECENT_QUERY_LIMIT = 8;

/** A single remembered query (the SPARQL text + when it last ran). */
export interface RecentQuery {
  /** The exact SPARQL text. */
  query: string;
  /** Epoch-ms of the most recent run (newest-first ordering). */
  ranAt: number;
}

/**
 * Push a just-run query onto the recent ring: most-recent first, de-duplicated by the
 * TRIMMED text (re-running the same query bumps it to the front rather than duplicating it),
 * empty/whitespace queries ignored, capped at `RECENT_QUERY_LIMIT`. Pure — returns a NEW
 * array, never mutates the input (so it is safe as a React state updater).
 */
export function pushRecentQuery(
  prev: readonly RecentQuery[],
  query: string,
  ranAt: number,
  limit: number = RECENT_QUERY_LIMIT,
): RecentQuery[] {
  const trimmed = query.trim();
  if (trimmed === "") return [...prev];
  const without = prev.filter((r) => r.query.trim() !== trimmed);
  return [{ query: trimmed, ranAt }, ...without].slice(0, Math.max(0, limit));
}

/**
 * A one-line preview of a (possibly multi-line) SPARQL query for a palette row: collapse
 * runs of whitespace, then truncate with an ellipsis. Keeps the row height fixed.
 */
export function previewQuery(query: string, max = 64): string {
  const oneLine = query.replace(/\s+/g, " ").trim();
  if (oneLine.length <= max) return oneLine;
  // Reserve one char for the ellipsis so the visible width stays <= `max`.
  return `${oneLine.slice(0, Math.max(0, max - 1))}…`;
}

/**
 * A short human label for a named graph in a palette row: the local name after the last
 * `#` or `/`, falling back to the full IRI when there is no separator. The default graph is
 * labelled explicitly by the caller (it has no IRI), so this only handles named graphs.
 */
export function graphLabel(iri: string): string {
  const hash = iri.lastIndexOf("#");
  const slash = iri.lastIndexOf("/");
  const cut = Math.max(hash, slash);
  if (cut >= 0 && cut < iri.length - 1) return iri.slice(cut + 1);
  return iri;
}
