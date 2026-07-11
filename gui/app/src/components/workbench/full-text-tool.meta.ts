// [FABLE-5] sq-qgkwy.2 — the Full-text tool's honesty-metadata override, split out of the
// panel file so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty
// read path needs it at first paint) while the panel component itself stays behind a lazy
// dynamic import() and out of the first-load bundle. Still a per-tool file — a tool bead
// flips its tier/copy/group HERE, never in the shared data/tools.ts (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. `undefined` = base metadata
 * unchanged. Omit fields you do not override.
 */
export const FULL_TEXT_TOOL_OVERRIDE: ToolOverride | undefined = {
  built: true,
  group: "working" as const,
  // [FABLE-5] sq-ixc3.16 — the tool now also surfaces the live index footprint.
  blurb: "BM25 full-text search via text: magic predicates, with live index stats, over the workspace store.",
};
