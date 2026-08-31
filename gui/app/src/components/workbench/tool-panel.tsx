"use client";

// [FABLE-5] sq-5lyme — the static tool-panel REGISTRY (was an if-chain): one panel file per
// tool, so each later tool bead lands in ITS OWN file with zero shared-file conflict (epic
// sq-2ucrz, research/gui-consolidation-fix-plan.md). The Query / SHACL / Inference tools are
// WORKING tabs over the live store; every tool whose capability is not genuinely available
// stays an honest STUB (ToolStub) — a stub never fabricates a result.
//
// [FABLE-5] sq-qgkwy.2 — CODE-SPLIT: every panel except the default Query tab is loaded via a
// lazy `next/dynamic` import(), so a rarely-opened tool costs ZERO first-load bytes and its
// chunk is fetched only when its tab is first opened (then stays mounted, exactly as before).
// The Query panel stays a STATIC import — it is the default tab and must paint first render.
//
// A registry entry may carry the optional honesty-metadata override its panel owns (ToolOverride
// in data/tools.ts); `resolveTool` merges it over the base ToolDef. That read path (rail / tab /
// palette grouping) runs at FIRST PAINT, so overrides are imported EAGERLY from each tool's
// sibling `.meta.ts` file — still one per-tool file, so a tool bead flips its tier/copy/group
// inside its OWN meta file, never in the shared data/tools.ts (the sq-5lyme seam, unchanged).

import dynamic from "next/dynamic";
import { Loader2 } from "lucide-react";
import type { ComponentType } from "react";
import { QueryWorkbench, QUERY_TOOL_OVERRIDE } from "@/components/workbench/query-workbench";
import { GRAPH_VIEW_TOOL_OVERRIDE } from "@/components/workbench/graph-view-tool.meta";
import { FULL_TEXT_TOOL_OVERRIDE } from "@/components/workbench/full-text-tool.meta";
import { STREAMING_TOOL_OVERRIDE } from "@/components/workbench/streaming-tool.meta";
import { SERVER_TOOL_OVERRIDE } from "@/components/workbench/server-tool.meta";
import { ODRL_TOOL_OVERRIDE } from "@/components/workbench/odrl-tool.meta";
import { PLAN_TOOL_OVERRIDE } from "@/components/workbench/plan-explorer.meta";
import { ToolStub } from "@/components/workbench/tool-stub";
import { applyToolOverride, toolById, type ToolDef, type ToolOverride } from "@/data/tools";

/**
 * The placeholder a lazily-loaded panel shows while its chunk is fetched (local disk in the
 * Tauri webview — effectively instant; one small request on the hosted web target). Layout-only:
 * no tool copy, no fabricated state.
 */
function PanelLoading() {
  return (
    <div className="flex h-full items-center justify-center text-muted-foreground">
      <Loader2 className="size-4 animate-spin" aria-hidden />
      <span className="sr-only">Loading tool…</span>
    </div>
  );
}

const lazyPanel = (loader: () => Promise<{ default: ComponentType }>) =>
  dynamic(loader, { loading: PanelLoading });

// Lazily-imported panels — one webpack chunk each. `.then((m) => ({ default: m.X }))` selects
// the named export; the panel module (and everything only IT imports, e.g. the graph renderer)
// stays out of the shared first-load chunks.
const ShaclTool = lazyPanel(() =>
  import("@/components/workbench/shacl-tool").then((m) => ({ default: m.ShaclTool })),
);
const InferenceTool = lazyPanel(() =>
  import("@/components/workbench/inference-tool").then((m) => ({ default: m.InferenceTool })),
);
const FormsTool = lazyPanel(() =>
  import("@/components/workbench/forms-tool").then((m) => ({ default: m.FormsTool })),
);
const GraphViewTool = lazyPanel(() =>
  import("@/components/workbench/graph-view-tool").then((m) => ({ default: m.GraphViewTool })),
);
const FullTextTool = lazyPanel(() =>
  import("@/components/workbench/full-text-tool").then((m) => ({ default: m.FullTextTool })),
);
const StreamingTool = lazyPanel(() =>
  import("@/components/workbench/streaming-tool").then((m) => ({ default: m.StreamingTool })),
);
const ServerTool = lazyPanel(() =>
  import("@/components/workbench/server-tool").then((m) => ({ default: m.ServerTool })),
);
const OdrlTool = lazyPanel(() =>
  import("@/components/workbench/odrl-tool").then((m) => ({ default: m.OdrlTool })),
);
const PlanExplorer = lazyPanel(() =>
  import("@/components/workbench/plan-explorer").then((m) => ({ default: m.PlanExplorer })),
);
const OverviewTool = lazyPanel(() =>
  import("@/components/workbench/overview-tool").then((m) => ({ default: m.OverviewTool })),
);

interface ToolPanelEntry {
  Component: ComponentType;
  /** The optional honesty-metadata override the panel's `.meta.ts` owns (undefined = none yet). */
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
  // STATIC import: the default tab paints on first render, never behind a chunk fetch.
  query: { Component: QueryWorkbench, override: QUERY_TOOL_OVERRIDE },
  shacl: { Component: ShaclTool },
  // [GPT-5.6] sq-lsp7k.1.2 — invocation-only chunk shared by desktop and hosted web.
  forms: { Component: FormsTool },
  // [OPUS-4.8] sq-tp1m — the Inference tool is a real, working panel (per-workspace RDFS /
  // OWL 2 RL entailment wired to the engine's forward-chaining reasoner), not an honest stub.
  inference: { Component: InferenceTool },
  "graph-view": { Component: GraphViewTool, override: GRAPH_VIEW_TOOL_OVERRIDE },
  "full-text": { Component: FullTextTool, override: FULL_TEXT_TOOL_OVERRIDE },
  streaming: { Component: StreamingTool, override: STREAMING_TOOL_OVERRIDE },
  server: { Component: ServerTool, override: SERVER_TOOL_OVERRIDE },
  // [FABLE-5] sq-ixc3.15 — the ODRL policy tool: a working panel over the desktop's NATIVE
  // engine (evaluate + fail-closed gated preview); the hosted web build degrades honestly
  // inside the panel (the `live-native` tier's framing).
  odrl: { Component: OdrlTool, override: ODRL_TOOL_OVERRIDE },
  // [FABLE-5] sq-ixc3.19 — the visual query-plan explorer (EXPLAIN/ANALYZE operator tree +
  // q-error heat + the this-workbench query monitor with endpoint Kill).
  plan: { Component: PlanExplorer, override: PLAN_TOOL_OVERRIDE },
  // [OPUS-5] sq-ixc3.21 — the dataset overview: class-hierarchy bubble, class-relationship
  // chord and domain–range table, all derived from aggregate queries over the live store.
  overview: { Component: OverviewTool },
};

/**
 * Resolved (base ⊕ panel-meta override) tool metadata — the honest read path for rail / tab /
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
