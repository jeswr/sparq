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
import { PanelsTopLeft, X, Layers, Upload, Download } from "lucide-react";

import { LeftRail } from "@/components/workbench/left-rail";
import { TitleBar } from "@/components/workbench/title-bar";
import { TopBar } from "@/components/workbench/top-bar";
import { TabStrip } from "@/components/workbench/tab-strip";
import { StatusBar } from "@/components/workbench/status-bar";
import { ToolPanel } from "@/components/workbench/tool-panel";
import { TOOLS, toolById } from "@/data/tools";
import { useEngine } from "@/lib/engine-context";
// [OPUS-4.8] sq-ixc3.10 — the keyboard-first spine: the Cmd-K palette provider + the shell's own
// contributed commands (open every tool, switch / close open tabs). Tools register their OWN verbs
// (the Query tool: run / EXPLAIN / recent queries) from inside their panels.
import {
  CommandPaletteProvider,
  useRegisterPaletteCommands,
} from "@/components/workbench/command-palette";
import { WorkbenchProvider, type WorkbenchActions } from "@/components/workbench/workbench-context";
import { graphLabel, type PaletteCommand } from "@/lib/palette-commands";
// [OPUS-4.8] sq-ixc3.13 — the Import drawer (real disk/URL/paste ingest via the native loader)
// is mounted once here and opened from the rail's "+ Import", the top bar, and Cmd-K.
import {
  ImportDrawerProvider,
  useImportDrawer,
} from "@/components/workbench/import-drawer";
// [OPUS-4.8] sq-xvj9 — the Cmd-K counterpart to the rail's "Export data…": serialise + download the
// whole store as pretty Turtle / TriG / JSON-LD from the keyboard-first spine.
import { downloadText } from "@/lib/download";
import { EXPORT_FORMATS, exportFilename } from "@/lib/rdf-format";

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

  const actions = React.useMemo<WorkbenchActions>(() => ({ openTool }), [openTool]);

  return (
    // [OPUS-4.8] sq-ixc3.10 — the whole workbench is wrapped in the Cmd-K palette provider (the
    // keyboard-first spine) + the shell-actions provider, both mounted ONCE here.
    <CommandPaletteProvider>
      <WorkbenchProvider actions={actions}>
        <ImportDrawerProvider>
        <ShellPaletteCommands
          tabs={tabs}
          activeId={activeId}
          onOpenTool={openTool}
          onSelectTab={setActiveId}
          onCloseTab={closeTab}
        />
        {/* [OPUS-4.8] sq-vw3ax (#820 redesign) — a native-feeling dark workspace. The ambient teal
            aura (a faint radial wash behind the chrome) makes the brand the lead, not garnish; the
            title bar above the top bar signals a real desktop window. */}
        <div className="relative flex h-screen flex-col overflow-hidden bg-background text-foreground">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-x-0 top-0 z-0 h-56 opacity-60"
            style={{
              background:
                "radial-gradient(900px 220px at 22% -10%, var(--teal-glow), transparent 60%), radial-gradient(700px 200px at 92% 0%, color-mix(in oklch, var(--primary) 18%, transparent), transparent 62%)",
            }}
          />
          <div className="relative z-10 flex min-h-0 flex-1 flex-col">
          <TitleBar />
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
        </div>
        </ImportDrawerProvider>
      </WorkbenchProvider>
    </CommandPaletteProvider>
  );
}

// [OPUS-4.8] sq-ixc3.10 — the SHELL's contributed Cmd-K commands: open any tool as a tab, switch
// to / close an open tab, and jump to a named graph (read from the live engine). The per-tool verbs
// (run / EXPLAIN / recent queries for the Query tool) are registered by the tool panels themselves.
// Rendered as a sibling (returns null) so it can call the registration hook inside the provider.
function ShellPaletteCommands({
  tabs,
  activeId,
  onOpenTool,
  onSelectTab,
  onCloseTab,
}: {
  tabs: OpenTab[];
  activeId: string;
  onOpenTool: (toolId: string) => void;
  onSelectTab: (toolId: string) => void;
  onCloseTab: (toolId: string) => void;
}) {
  const { graphs, status, storeSize, exportStore } = useEngine();
  // [OPUS-4.8] sq-ixc3.13 — the Cmd-K entry point for the Import drawer.
  const { setOpen: setImportOpen } = useImportDrawer();
  // [OPUS-4.8] sq-xvj9 — export is only meaningful once the engine is warm with a non-empty store.
  const exportDisabled = status.kind !== "ready" || storeSize === 0;

  const commands = React.useMemo<PaletteCommand[]>(() => {
    const cmds: PaletteCommand[] = [];

    // [OPUS-4.8] sq-ixc3.13 — the lead ACTION: open the Import drawer (real disk/URL/paste ingest).
    cmds.push({
      id: "action.import",
      group: "Actions",
      title: "Import data…",
      blurb: "Load RDF from a file (compressed / HDT), a URL, or pasted text into the store.",
      keywords: ["import", "load", "data", "file", "url", "paste", "rdf", "hdt", "open dataset"],
      icon: Upload,
      run: () => setImportOpen(true),
    });

    // [OPUS-4.8] sq-xvj9 — export the whole store as pretty Turtle / TriG / JSON-LD (the rail's
    // "Export data…" from the keyboard). Disabled until the engine is warm with a non-empty store.
    for (const f of EXPORT_FORMATS) {
      cmds.push({
        id: `action.export.${f.value}`,
        group: "Actions",
        title: `Export dataset as ${f.label}`,
        blurb: `Download the ${f.scope} as ${f.label}, prefix-abbreviated.`,
        keywords: [
          "export",
          "download",
          "save",
          "serialize",
          "serialise",
          "dataset",
          f.label,
          f.value,
          "turtle",
          "trig",
          "json-ld",
        ],
        icon: Download,
        disabled: exportDisabled,
        run: () => {
          const text = exportStore(f.value);
          if (text !== null) downloadText(exportFilename(f.value), text, f.mime);
        },
      });
    }

    // Every TOOL as an "open …" command. A `built` tool opens its working tab; a stub still opens
    // (the panel states its tier + what it will do honestly — no fabricated result).
    for (const tool of TOOLS) {
      cmds.push({
        id: `tool.${tool.id}`,
        group: "Tools",
        title: `Open ${tool.label}`,
        blurb: tool.blurb,
        keywords: ["open", "tool", "tab", tool.label, tool.blurb],
        icon: tool.icon,
        run: () => onOpenTool(tool.id),
      });
    }

    // Open tabs: switch to any non-active open tab, or close the active one.
    for (const tab of tabs) {
      if (tab.id === activeId) continue;
      cmds.push({
        id: `tab.switch.${tab.id}`,
        group: "Open tabs",
        title: `Switch to ${tab.label}`,
        keywords: ["switch", "tab", "focus", tab.label],
        icon: PanelsTopLeft,
        run: () => onSelectTab(tab.id),
      });
    }
    cmds.push({
      id: "tab.close-active",
      group: "Open tabs",
      title: "Close the active tab",
      blurb: tabs.length <= 1 ? "Cannot close the last tab" : undefined,
      keywords: ["close", "tab"],
      icon: X,
      disabled: tabs.length <= 1,
      run: () => onCloseTab(activeId),
    });

    // Named graphs (live, from the engine summary): focus the Query tool so the user can scope a
    // query to the graph. The default graph (graph === null) is not listed (it has no IRI).
    for (const g of graphs) {
      if (g.graph === null) continue;
      const iri = g.graph;
      cmds.push({
        id: `graph.${iri}`,
        group: "Named graphs",
        title: graphLabel(iri),
        blurb: `${iri} · ${g.count.toLocaleString()} quad${g.count === 1 ? "" : "s"}`,
        keywords: ["graph", "named graph", iri],
        icon: Layers,
        run: () => onOpenTool("query"),
      });
    }

    return cmds;
  }, [
    tabs,
    activeId,
    graphs,
    onOpenTool,
    onSelectTab,
    onCloseTab,
    setImportOpen,
    exportDisabled,
    exportStore,
  ]);

  useRegisterPaletteCommands("shell", commands);
  return null;
}
