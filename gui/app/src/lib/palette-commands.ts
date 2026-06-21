// [OPUS-4.8] sq-ixc3.10 (epic sq-ixc3) — the keyboard-first SPINE's command model + pure helpers
// for the operational GUI workbench.
//
// research/gui-design.md §A.3 makes the command palette the workbench's backbone: any TOOL, every
// NAMED GRAPH, RECENT QUERIES, and the verbs run / run-EXPLAIN / open-tool / switch-tab / browse-
// dataset are one fuzzy keystroke away — so the left rail need not show everything. This module is
// the logic-bearing core: a typed command descriptor + the PURE functions (recent-query ring, a
// one-line query preview, a named-graph label) that the React palette renders. Icons are passed
// in by the caller so this file stays free of lucide-react and trivially testable.

import type { LucideIcon } from "lucide-react";

/**
 * One selectable command in the palette. `id` is stable (cmdk keys + the registry de-dupe on it);
 * `run` performs the action then the palette closes. `keywords` feed the fuzzy match alongside the
 * title. `group` buckets it under a heading; `disabled` greys it without hiding it.
 */
export interface PaletteCommand {
  id: string;
  group: PaletteCommandGroup;
  title: string;
  blurb?: string;
  keywords?: string[];
  icon: LucideIcon;
  run: () => void;
  disabled?: boolean;
}

/**
 * The command headings, in render order. Live verbs lead the spine, then the tools, the open tabs,
 * recent queries, and the loaded named graphs.
 */
export type PaletteCommandGroup =
  | "Actions"
  | "Tools"
  | "Open tabs"
  | "Recent queries"
  | "Named graphs";

/** Fixed render order of the groups (matches `PaletteCommandGroup`). */
export const PALETTE_COMMAND_GROUP_ORDER: PaletteCommandGroup[] = [
  "Actions",
  "Tools",
  "Open tabs",
  "Recent queries",
  "Named graphs",
];

/** How many distinct recent queries the spine remembers / offers. */
export const RECENT_QUERY_LIMIT = 8;

/** A single remembered query (the SPARQL text + when it last ran). */
export interface RecentQuery {
  query: string;
  ranAt: number;
}

/**
 * Push a just-run query onto the recent ring: most-recent first, de-duplicated by the TRIMMED text
 * (re-running the same query bumps it to the front, no duplicate), empty queries ignored, capped at
 * `RECENT_QUERY_LIMIT`. Pure — returns a NEW array, never mutates the input (safe as a state
 * updater).
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
 * A one-line preview of a (possibly multi-line) SPARQL query for a palette row: collapse runs of
 * whitespace, then truncate with an ellipsis so the row height stays fixed.
 */
export function previewQuery(query: string, max = 64): string {
  const oneLine = query.replace(/\s+/g, " ").trim();
  if (oneLine.length <= max) return oneLine;
  return `${oneLine.slice(0, Math.max(0, max - 1))}…`;
}

/**
 * A short human label for a named graph: the local name after the last `#` or `/`, falling back to
 * the full IRI when there is no usable separator. The default graph has no IRI (handled by the
 * caller), so this only labels named graphs.
 */
export function graphLabel(iri: string): string {
  const hash = iri.lastIndexOf("#");
  const slash = iri.lastIndexOf("/");
  const cut = Math.max(hash, slash);
  if (cut >= 0 && cut < iri.length - 1) return iri.slice(cut + 1);
  return iri;
}

/**
 * Bucket a flat command list into its fixed-order groups, dropping empties (so a heading never
 * renders without rows). Pure — the React palette maps over the result.
 */
export function groupPaletteCommands(
  commands: readonly PaletteCommand[],
): { group: PaletteCommandGroup; commands: PaletteCommand[] }[] {
  return PALETTE_COMMAND_GROUP_ORDER.map((group) => ({
    group,
    commands: commands.filter((c) => c.group === group),
  })).filter((g) => g.commands.length > 0);
}
