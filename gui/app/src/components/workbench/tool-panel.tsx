"use client";

// [FABLE-5] sq-5lyme — the static tool-panel REGISTRY (was an if-chain): one panel file per
// tool, so each later tool bead lands in ITS OWN file with zero shared-file conflict (epic
// sq-2ucrz, research/gui-consolidation-fix-plan.md). The Query / SHACL / Inference tools are
// WORKING tabs over the live store; the four feasible-now tools (graph-view / full-text /
// streaming / server) now have their own panel files that still render the honest stub until
// sq-lxomy / sq-9nwab / sq-kwb74 / sq-iemfq fill them. Every tool whose capability is not
// genuinely available stays an honest STUB (ToolStub) — a stub never fabricates a result.
//
// A registry entry may carry the optional honesty-metadata override its panel file exports
// (ToolOverride in data/tools.ts); `resolveTool` merges it over the base ToolDef. That is the
// read path rail/tab consumers adopt (sq-1odi9 regroups the rail on it), so a tool bead flips
// its tier/copy/group INSIDE its own panel file — never in the shared data/tools.ts.

import type { ComponentType } from "react";
import { QueryWorkbench, QUERY_TOOL_OVERRIDE } from "@/components/workbench/query-workbench";
import { ShaclTool } from "@/components/workbench/shacl-tool";
import { InferenceTool } from "@/components/workbench/inference-tool";
import {
  GraphViewTool,
  GRAPH_VIEW_TOOL_OVERRIDE,
} from "@/components/workbench/graph-view-tool";
import { FullTextTool, FULL_TEXT_TOOL_OVERRIDE } from "@/components/workbench/full-text-tool";
import { StreamingTool, STREAMING_TOOL_OVERRIDE } from "@/components/workbench/streaming-tool";
import { ServerTool, SERVER_TOOL_OVERRIDE } from "@/components/workbench/server-tool";
import { ToolStub } from "@/components/workbench/tool-stub";
import { applyToolOverride, toolById, type ToolDef, type ToolOverride } from "@/data/tools";

interface ToolPanelEntry {
  Component: ComponentType;
  /** The optional honesty-metadata override the panel file owns (undefined = none yet). */
  override?: ToolOverride;
}

/**
 * Static id → panel registry. Tools with no entry (vector / geosparql / zk / mpc — genuinely
 * deferred behind the sq-zeai portability spike and the ZK/MPC audit posture) fall back to the
 * shared honest ToolStub, exactly as before.
 */
const TOOL_PANELS: Record<string, ToolPanelEntry> = {
  // [FABLE-5] sq-ixc3.14 — blurb override: the Query tool now also runs federated SERVICE
  // (native engine on desktop, allowlist-gated fail-closed; honestly native-only in the browser).
  query: { Component: QueryWorkbench, override: QUERY_TOOL_OVERRIDE },
  shacl: { Component: ShaclTool },
  // [OPUS-4.8] sq-tp1m — the Inference tool is a real, working panel (per-workspace RDFS /
  // OWL 2 RL entailment wired to the engine's forward-chaining reasoner), not an honest stub.
  inference: { Component: InferenceTool },
  "graph-view": { Component: GraphViewTool, override: GRAPH_VIEW_TOOL_OVERRIDE },
  "full-text": { Component: FullTextTool, override: FULL_TEXT_TOOL_OVERRIDE },
  streaming: { Component: StreamingTool, override: STREAMING_TOOL_OVERRIDE },
  server: { Component: ServerTool, override: SERVER_TOOL_OVERRIDE },
};

/**
 * Resolved (base ⊕ panel-file override) tool metadata — the honest read path for rail / tab /
 * palette consumers, so honesty flips land with the panel that earns them (sq-5lyme seam).
 */
export function resolveTool(id: string): ToolDef | undefined {
  const base = toolById(id);
  return base ? applyToolOverride(base, TOOL_PANELS[id]?.override) : undefined;
}

export function ToolPanel({ toolId }: { toolId: string }) {
  const entry = TOOL_PANELS[toolId];
  if (entry) return <entry.Component />;
  return <ToolStub toolId={toolId} />;
}
