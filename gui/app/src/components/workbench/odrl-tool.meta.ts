// [FABLE-5] sq-qgkwy.2 — the ODRL tool's honesty-metadata override, split out of the
// panel file so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty
// read path needs it at first paint) while the panel component itself stays behind a lazy
// dynamic import() and out of the first-load bundle. Still a per-tool file — a tool bead
// flips its tier/copy/group HERE, never in the shared data/tools.ts (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * Honesty-metadata override — flips this tool to a working panel (sq-ixc3.15).
 * The tier stays `live-native`: live on the desktop's native engine, honestly
 * unavailable in the hosted web build. Never edit data/tools.ts.
 */
export const ODRL_TOOL_OVERRIDE: ToolOverride = {
  built: true,
  group: "working",
};
