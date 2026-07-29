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
import { PanelsTopLeft, X, Layers, Upload, Download, AlertTriangle, FileUp } from "lucide-react";

import { LeftRail } from "@/components/workbench/left-rail";
import { TitleBar } from "@/components/workbench/title-bar";
import { TopBar } from "@/components/workbench/top-bar";
import { TabStrip } from "@/components/workbench/tab-strip";
import { StatusBar } from "@/components/workbench/status-bar";
import { ToolPanel } from "@/components/workbench/tool-panel";
import { TOOLS, toolById } from "@/data/tools";
import { useEngine } from "@/lib/engine-context";
// (sq-eydh9) [SONNET-4.6] — global workbench drop zone: dropping RDF files ANYWHERE on the
// workbench opens the import drawer and runs the multi-file import (both personas — Tauri
// webview drops also deliver File objects, so the browser path handles them too).
import { useFileDrop } from "@/components/workbench/dropzone";
import { guessFormat } from "@/lib/rdf-format";
// [OPUS-4.8] sq-ixc3.10 — the keyboard-first spine: the Cmd-K palette provider + the shell's own
// contributed commands (open every tool, switch / close open tabs). Tools register their OWN verbs
// (the Query tool: run / EXPLAIN / recent queries) from inside their panels.
import {
  CommandPaletteProvider,
  useRegisterPaletteCommands,
} from "@/components/workbench/command-palette";
import {
  WorkbenchProvider,
  type QueryHandoff,
  type WorkbenchActions,
} from "@/components/workbench/workbench-context";
import { graphLabel, type PaletteCommand } from "@/lib/palette-commands";
// [OPUS-4.8] sq-ixc3.13 — the Import drawer (real disk/URL/paste ingest via the native loader)
// is mounted once here and opened from the rail's "+ Import", the top bar, and Cmd-K.
import {
  ImportDrawerProvider,
  useImportDrawer,
} from "@/components/workbench/import-drawer";
// [OPUS-4.8] sq-tp1m (#757) — keeps the engine's inference regime in lockstep with the active
// workspace's persisted choice (restores it on load), mounted once regardless of the active tab.
// [sq-glo5r] — N3RulesBridge keeps the engine's N3 rules cache in lockstep with the workspace.
import { InferenceModeBridge, N3RulesBridge } from "@/components/workbench/inference-control";
// [FABLE-5] sq-ixc3.14 — keeps the engine's federation egress allowlist in lockstep with the
// active workspace's persisted setting (fail-closed until pushed), mounted once like the bridges above.
import { FederationBridge } from "@/components/workbench/federation-control";
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

  // [OPUS-5] sq-ixc3.24 — the visual query builder's hand-off slot: the SPARQL it emits, plus a
  // monotonic sequence so sending the same text twice still re-loads the Query editor.
  const [queryHandoff, setQueryHandoff] = React.useState<QueryHandoff | null>(null);
  const sendToQueryEditor = React.useCallback(
    (sparql: string) => {
      setQueryHandoff((prev) => ({ sparql, seq: (prev?.seq ?? 0) + 1 }));
      openTool("query");
    },
    [openTool],
  );

  const actions = React.useMemo<WorkbenchActions>(
    () => ({ openTool, sendToQueryEditor, queryHandoff }),
    [openTool, sendToQueryEditor, queryHandoff],
  );

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
        {/* [OPUS-4.8] sq-tp1m — sync the persisted per-workspace inference regime into the engine. */}
        <InferenceModeBridge />
        {/* [sq-glo5r] — sync the per-workspace N3 rules into the engine's closure cache. */}
        <N3RulesBridge />
        {/* [FABLE-5] sq-ixc3.14 — sync the per-workspace federation egress allowlist. */}
        <FederationBridge />
        {/* (sq-eydh9) [SONNET-4.6] — global RDF file drop overlay, mounted once here. */}
        <GlobalDropOverlay />
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
  // [OPUS-4.8] sq-xvj9 — the Cmd-K export path must not fail silently: when `exportStore` returns
  // null (a lean bundle without the serialise binding) surface the same message the rail shows,
  // as an ephemeral aria-live alert (the palette has already closed, so an inline note is not
  // visible from here).
  const [exportError, setExportError] = React.useState<string | null>(null);
  React.useEffect(() => {
    if (!exportError) return;
    const id = window.setTimeout(() => setExportError(null), 6000);
    return () => window.clearTimeout(id);
  }, [exportError]);

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
          if (text === null) {
            setExportError("The engine is not ready, or this build cannot serialise RDF.");
            return;
          }
          setExportError(null);
          downloadText(exportFilename(f.value), text, f.mime);
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

  if (!exportError) return null;
  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-8 z-[60] flex justify-center px-4">
      <div
        role="alert"
        aria-live="assertive"
        data-export-feedback="error"
        className="pointer-events-auto flex max-w-md items-start gap-2 rounded-md border border-destructive/40 bg-popover px-3 py-2 text-xs text-destructive shadow-lg"
      >
        <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
        <span className="min-w-0">{exportError}</span>
        <button
          type="button"
          onClick={() => setExportError(null)}
          aria-label="Dismiss export error"
          className="ml-1 shrink-0 rounded-sm text-muted-foreground hover:text-foreground"
        >
          <X className="size-3.5" />
        </button>
      </div>
    </div>
  );
}

// ── (sq-eydh9) GlobalDropOverlay — drop RDF files ANYWHERE to import them ───────────────────
//
// Mounts a window-level drag/drop surface (via `useFileDrop`) so the user never has to open the
// Import drawer just to drop a file. On drop: reads each file in-tab (File.text()), imports it
// sequentially into the live store, then opens the drawer to show the aggregate result. Both the
// Tauri webview and a plain browser deliver File objects on drop, so this path works in both
// personas.
//
// The hidden `data-global-drop-input` is a stable <input type="file"> for Playwright
// `setInputFiles()` — it exercises the same import code path the window-level drop uses.

const GLOBAL_RDF_ACCEPT = [".ttl", ".nt", ".nq", ".trig", ".jsonld", ".json"];

function GlobalDropOverlay() {
  const { importRdf } = useEngine();
  const { setOpen } = useImportDrawer();
  const inputRef = React.useRef<HTMLInputElement>(null);

  // Import a batch of already-read files: sequential, non-aborting per-file errors.
  const runImportBatch = React.useCallback(
    async (files: Array<{ name: string; text: string }>) => {
      for (const file of files) {
        try {
          await importRdf({
            kind: "paste",
            mode: "add",
            preserveGraphs: true,
            label: file.name,
            format: guessFormat(file.name),
            text: file.text,
          });
        } catch {
          // Per-file failure must not abort the rest — invariant from sq-eydh9.
        }
      }
    },
    [importRdf],
  );

  // Drop handler: extract File objects synchronously (before first await), then read + import.
  const handleFiles = React.useCallback(
    (result: import("@/lib/file-ingest").IngestResult) => {
      if (result.accepted.length === 0) return;
      // Open the drawer so the user can see the result after import.
      setOpen(true);
      void runImportBatch(result.accepted);
    },
    [runImportBatch, setOpen],
  );

  const { dragActive, targetProps } = useFileDrop({
    onFiles: handleFiles,
    accept: GLOBAL_RDF_ACCEPT,
  });

  // Hidden input: lets Playwright setInputFiles() exercise the global-drop code path.
  const handleInputChange = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const fileArr = Array.from(e.target.files ?? []);
      if (fileArr.length === 0) return;
      if (inputRef.current) inputRef.current.value = "";
      const accepted: Array<{ name: string; text: string }> = [];
      for (const file of fileArr) {
        try {
          const text = await file.text();
          accepted.push({ name: file.name, text });
        } catch {
          // Unreadable files are silently skipped at the global-drop level; the drawer's own
          // per-file upload shows explicit rejection reasons for user-initiated picks.
        }
      }
      if (accepted.length > 0) {
        setOpen(true);
        void runImportBatch(accepted);
      }
    },
    [runImportBatch, setOpen],
  );

  return (
    // Full-screen transparent overlay: spread the drop handlers onto a fixed inset-0 div so the
    // ENTIRE workbench surface is a drop target, not just a specific panel.
    <div
      data-global-drop
      className="pointer-events-none fixed inset-0 z-40"
      {...targetProps}
      style={{ pointerEvents: dragActive ? "auto" : "none" }}
    >
      {/* Visible feedback ring only while a files drag is active over the window. */}
      {dragActive && (
        <div
          aria-hidden
          className="absolute inset-2 flex items-center justify-center rounded-xl border-2 border-dashed border-primary bg-background/70 backdrop-blur-sm"
        >
          <div className="flex items-center gap-3 text-sm font-medium text-foreground">
            <FileUp className="size-6 text-primary" aria-hidden />
            Drop RDF files to import
          </div>
        </div>
      )}
      <span className="sr-only" role="status">
        {dragActive ? "Release to drop the RDF files and import them" : ""}
      </span>
      {/* Stable input for Playwright setInputFiles() — not visible; does not intercept pointer events. */}
      <input
        ref={inputRef}
        type="file"
        multiple
        accept={GLOBAL_RDF_ACCEPT.join(",")}
        data-global-drop-input
        aria-label="Drop files or use this input to import RDF"
        className="sr-only"
        style={{ pointerEvents: "auto" }}
        onChange={handleInputChange}
      />
    </div>
  );
}
