"use client";

// [OPUS-4.8] sq-ixc3.9 / sq-ixc3.11 — dispatches a tab id to its tool panel. The Query and
// SHACL tools are WORKING tabs over the live store (the surface-to-tool translation of
// research/gui-design.md §A.4/§A.5, marketing chrome cut); every tool whose capability is not
// genuinely in the GUI's in-tab wasm bundle stays an honest STUB (ToolStub) that the later
// phases fill (sq-ixc3.12 the query workbench multi-view, sq-ixc3.13 the import drawer, sq-zeai
// the native-only vector/geo wasm spike). A stub never fabricates a result — it states the verb
// + tier.

import { QueryWorkbench } from "@/components/workbench/query-workbench";
import { ShaclTool } from "@/components/workbench/shacl-tool";
import { InferenceTool } from "@/components/workbench/inference-tool";
import { ToolStub } from "@/components/workbench/tool-stub";

export function ToolPanel({ toolId }: { toolId: string }) {
  if (toolId === "query") return <QueryWorkbench />;
  if (toolId === "shacl") return <ShaclTool />;
  // [OPUS-4.8] sq-tp1m — the Inference tool is a real, working panel (per-workspace RDFS / OWL 2
  // RL entailment wired to the engine's forward-chaining reasoner), no longer an honest stub.
  if (toolId === "inference") return <InferenceTool />;
  return <ToolStub toolId={toolId} />;
}
