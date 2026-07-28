// [OPUS-5] sq-ixc3.24 — the Query builder's honesty-metadata override, split out of the panel
// file so the tool-panel registry can read it EAGERLY (the rail/tab/palette honesty read path
// needs it at first paint) while the panel component stays behind a lazy dynamic import().
// Per the sq-5lyme seam, a tool bead flips its tier/copy/group HERE, never in data/tools.ts.

import type { ToolOverride } from "@/data/tools";

/**
 * The builder is `live`: the canvas, the SPARQL lowering and the schema/shape introspection all
 * run in-tab against the same wasm engine the Query tool uses. There is no native-only half and
 * nothing is simulated — an empty store yields empty pickers and says so.
 */
export const QUERY_BUILDER_TOOL_OVERRIDE: ToolOverride | undefined = {
  built: true,
  group: "working" as const,
};
