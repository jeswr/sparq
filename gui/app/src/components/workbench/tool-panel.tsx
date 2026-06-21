"use client";

// [OPUS-4.8] sq-ixc3.9 — dispatches a tab id to its tool panel. The Query tool is a WORKING
// tab over the live store; every other tool is an honest STUB (ToolStub) that the later phases
// fill (sq-ixc3.11 turns surfaces into live tools, sq-ixc3.12 the query workbench multi-view,
// sq-ixc3.13 the import drawer). A stub never fabricates a result — it states the verb + tier.

import { QueryWorkbench } from "@/components/workbench/query-workbench";
import { ToolStub } from "@/components/workbench/tool-stub";
import { toolById } from "@/data/tools";

export function ToolPanel({ toolId }: { toolId: string }) {
  if (toolId === "query") return <QueryWorkbench />;
  const tool = toolById(toolId);
  if (!tool) return <ToolStub toolId={toolId} />;
  return <ToolStub toolId={toolId} />;
}
