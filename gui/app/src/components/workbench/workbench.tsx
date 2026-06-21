"use client";

// [OPUS-4.8] sq-ixc3.9 — the operational app SHELL (research/gui-design.md §A.2):
//
//   ┌──────────────────────────────────────────────────────────┐
//   │ TOP BAR (h-10): target switch · store size · ⌘K · theme · LED │
//   ├──────────┬───────────────────────────────────────────────┤
//   │ LEFT     │ IDE TAB STRIP                                   │
//   │ RAIL     ├───────────────────────────────────────────────┤
//   │ (w-56)   │ WORK AREA (full-bleed, default = Query)         │
//   │ workspace│                                                 │
//   │ datasets ├───────────────────────────────────────────────┤
//   │ TOOLS    │ STATUS BAR (h-6): latency · rows · target · backend │
//   └──────────┴───────────────────────────────────────────────┘
//
// This is the workbench, NOT a route tree: no Showcase/Benchmarks/About marketing chrome. The
// Query tab is a working tool over the live store; the other TOOLS open as honest stub tabs the
// later phases (sq-ixc3.11/.12/.13) fill. A single Help link opens the marketing website in the
// system browser (the design's rule: the GUI never renders the site, it links to it).

import * as React from "react";

import { LeftRail } from "@/components/workbench/left-rail";
import { TopBar } from "@/components/workbench/top-bar";
import { TabStrip } from "@/components/workbench/tab-strip";
import { StatusBar } from "@/components/workbench/status-bar";
import { ToolPanel } from "@/components/workbench/tool-panel";
import { TOOLS, toolById } from "@/data/tools";

/** An open tab in the IDE tab strip — keyed by tool id (a tool opens at most once). */
export interface OpenTab {
  id: string;
  label: string;
}

const DEFAULT_TAB: OpenTab = { id: "query", label: "Query" };

export function Workbench() {
  // The Query tool is open by default (the design's default work area).
  const [tabs, setTabs] = React.useState<OpenTab[]>([DEFAULT_TAB]);
  const [activeId, setActiveId] = React.useState<string>(DEFAULT_TAB.id);

  /** Open a tool as a tab (focus it if already open). */
  const openTool = React.useCallback((toolId: string) => {
    const tool = toolById(toolId);
    if (!tool) return;
    setTabs((prev) =>
      prev.some((t) => t.id === toolId)
        ? prev
        : [...prev, { id: tool.id, label: tool.label }],
    );
    setActiveId(toolId);
  }, []);

  /** Close a tab; if it was active, focus the previous (never close the last one). */
  const closeTab = React.useCallback(
    (toolId: string) => {
      setTabs((prev) => {
        if (prev.length <= 1) return prev; // never empty the work area
        const idx = prev.findIndex((t) => t.id === toolId);
        const next = prev.filter((t) => t.id !== toolId);
        if (toolId === activeId) {
          const fallback = next[Math.max(0, idx - 1)] ?? next[0];
          setActiveId(fallback.id);
        }
        return next;
      });
    },
    [activeId],
  );

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <TopBar />
      <div className="flex min-h-0 flex-1">
        <LeftRail tools={TOOLS} activeId={activeId} onOpenTool={openTool} />
        <main className="flex min-w-0 flex-1 flex-col">
          <TabStrip
            tabs={tabs}
            activeId={activeId}
            onSelect={setActiveId}
            onClose={closeTab}
          />
          <div className="min-h-0 flex-1 overflow-hidden">
            {tabs.map((tab) => (
              <div
                key={tab.id}
                hidden={tab.id !== activeId}
                className="h-full"
                data-tab={tab.id}
              >
                <ToolPanel toolId={tab.id} />
              </div>
            ))}
          </div>
          <StatusBar />
        </main>
      </div>
    </div>
  );
}
