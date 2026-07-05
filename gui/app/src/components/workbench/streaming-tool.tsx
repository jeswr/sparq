"use client";

// [FABLE-5] sq-5lyme — the Streaming tool's OWN panel file (tool-panel registry seam).
// Today it renders the exact honest stub it always did; sq-kwb74 fills it with the RSP-QL
// tick view over the sparq-rsp-wasm bundle. When that lands, flip the honesty metadata via
// STREAMING_TOOL_OVERRIDE below — never by editing the shared data/tools.ts — so parallel
// tool beads stay file-disjoint.

import { ToolStub } from "@/components/workbench/tool-stub";
import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. `undefined` = base metadata
 * unchanged. Omit fields you do not override.
 */
export const STREAMING_TOOL_OVERRIDE: ToolOverride | undefined = undefined;

export function StreamingTool() {
  return <ToolStub toolId="streaming" override={STREAMING_TOOL_OVERRIDE} />;
}
