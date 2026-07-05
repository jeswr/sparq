"use client";

// [FABLE-5] sq-5lyme — the Graph-view tool's OWN panel file (tool-panel registry seam).
// Today it renders the exact honest stub it always did; sq-lxomy fills it with the working
// CONSTRUCT/DESCRIBE tab over the already-shipped GraphView component. When that lands, flip
// the honesty metadata via GRAPH_VIEW_TOOL_OVERRIDE below — never by editing the shared
// data/tools.ts — so parallel tool beads stay file-disjoint.

import { ToolStub } from "@/components/workbench/tool-stub";
import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. `undefined` = base metadata
 * unchanged. Omit fields you do not override.
 */
export const GRAPH_VIEW_TOOL_OVERRIDE: ToolOverride | undefined = undefined;

export function GraphViewTool() {
  return <ToolStub toolId="graph-view" override={GRAPH_VIEW_TOOL_OVERRIDE} />;
}
