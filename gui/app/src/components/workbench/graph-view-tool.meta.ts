// [FABLE-5] sq-qgkwy.2 — the Graph view tool's honesty-metadata override, split out of the
// panel file so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty
// read path needs it at first paint) while the panel component itself stays behind a lazy
// dynamic import() and out of the first-load bundle. Still a per-tool file — a tool bead
// flips its tier/copy/group HERE, never in the shared data/tools.ts (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. Now that the working panel has
 * landed, flip tier/built/group to reflect reality — never by editing the shared data/tools.ts.
 */
export const GRAPH_VIEW_TOOL_OVERRIDE: ToolOverride = {
  tier: "live",
  built: true,
  group: "working",
  blurb: "Node-link visualisation of CONSTRUCT/DESCRIBE results over the live store.",
};
