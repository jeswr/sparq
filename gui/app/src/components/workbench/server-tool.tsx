"use client";

// [FABLE-5] sq-5lyme — the Server tool's OWN panel file (tool-panel registry seam).
// Today it renders the exact honest stub it always did; sq-iemfq fills it with the SPARQL 1.1
// Protocol endpoint client over the existing @sparq/client core. When that lands, flip the
// honesty metadata via SERVER_TOOL_OVERRIDE below — never by editing the shared
// data/tools.ts — so parallel tool beads stay file-disjoint.

import { ToolStub } from "@/components/workbench/tool-stub";
import type { ToolOverride } from "@/data/tools";

/**
 * Optional honesty-metadata override merged over the base `ToolDef` (data/tools.ts) by the
 * tool-panel registry's `resolveTool` and by the stub itself. `undefined` = base metadata
 * unchanged. Omit fields you do not override.
 */
export const SERVER_TOOL_OVERRIDE: ToolOverride | undefined = undefined;

export function ServerTool() {
  return <ToolStub toolId="server" override={SERVER_TOOL_OVERRIDE} />;
}
