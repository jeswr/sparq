// [FABLE-5] sq-qgkwy.2 — the Server tool's honesty-metadata override, split out of the
// panel file so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty
// read path needs it at first paint) while the panel component itself stays behind a lazy
// dynamic import() and out of the first-load bundle. Still a per-tool file — a tool bead
// flips its tier/copy/group HERE, never in the shared data/tools.ts (the sq-5lyme seam).

import type { ToolOverride } from "@/data/tools";

/**
 * Honesty-metadata override — flips this tool from "walkthrough / not built" to
 * "live / working" once this panel lands (sq-iemfq). Never edit data/tools.ts.
 */
export const SERVER_TOOL_OVERRIDE: ToolOverride = {
  tier: "live",
  built: true,
  group: "working",
  blurb:
    "Connect to a running SPARQL 1.1 Protocol endpoint — query + bindings table, health status.",
};
