"use client";

// [OPUS-4.8] sq-ixc3.9 — the LEFT RAIL (w-56, denser than the site's w-64;
// research/gui-design.md §A.2):
//   (1) workspace switcher — ●saved/○unsaved (the persistent-workspace model is sq-atb0; this
//       foundation shows the default workspace; the picker is a later-phase stub);
//   (2) datasets tree of the LIVE store — default + named graphs with per-graph counts, plus an
//       Imports subgroup placeholder + "+ Import" entry point (the real import drawer is
//       sq-ixc3.13);
//   (3) TOOLS list — each a VERB opened as a tab with an honesty-tier dot (NOT a page).

import * as React from "react";
import { ChevronRight, Plus, Database, FolderTree, FileUp, Link2 } from "lucide-react";

import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { useWorkspace } from "@/lib/workspace-context";
import { useImportDrawer } from "@/components/workbench/import-drawer";
import { ExportDataMenu } from "@/components/workbench/store-export";
import { TIER_META, type ToolDef } from "@/data/tools";
import { resolveTool } from "@/components/workbench/tool-panel";
// [SONNET-4.6] sq-lmlch — the real workspace switcher UI (replaces the dead stub).
import { WorkspaceSwitcher } from "@/components/workbench/workspace-switcher";

interface LeftRailProps {
  tools: ToolDef[];
  activeId: string;
  onOpenTool: (toolId: string) => void;
}

function shortenGraph(iri: string | null): string {
  if (iri === null) return "default graph";
  // Show the last path/fragment segment for readability; full IRI on hover.
  const m = iri.match(/[#/]([^#/]+)\/?$/);
  return m ? m[1] : iri;
}

function DatasetsTree() {
  const { graphs, status } = useEngine();
  // [OPUS-4.8] sq-ixc3.13 — the live Imports subgroup (WorkspaceSourceMeta) + the working
  // "+ Import" entry point that opens the Import drawer.
  const { sources } = useWorkspace();
  const { setOpen } = useImportDrawer();
  return (
    <div className="px-1 pb-2">
      {status.kind !== "ready" ? (
        <p className="px-2 py-1 text-xs text-muted-foreground">
          {status.kind === "error" ? "engine error" : "loading store…"}
        </p>
      ) : graphs.length === 0 ? (
        <p className="px-2 py-1 text-xs text-muted-foreground">empty store</p>
      ) : (
        <ul className="space-y-0.5">
          {graphs.map((g) => (
            <li
              key={g.graph ?? "__default__"}
              className="flex items-center gap-1.5 rounded px-2 py-1 text-xs hover:bg-sidebar-accent/40"
              title={g.graph ?? "the default graph"}
            >
              <Database className="size-3 shrink-0 text-muted-foreground" />
              <span className="min-w-0 flex-1 truncate">{shortenGraph(g.graph)}</span>
              <span className="tabular text-[11px] text-muted-foreground">
                {g.count.toLocaleString()}
              </span>
            </li>
          ))}
        </ul>
      )}

      {/* [OPUS-4.8] sq-ixc3.13 — the Imports subgroup: the active workspace's WorkspaceSourceMeta
          list (local files + re-fetchable URL sources), most recent first. */}
      {sources.length > 0 && (
        <div className="mt-2">
          <h3 className="px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
            Imports
          </h3>
          <ul className="space-y-0.5">
            {[...sources]
              .sort((a, b) => b.importedAt - a.importedAt)
              .map((s) => (
                <li
                  key={`${s.kind}-${s.label}-${s.importedAt}`}
                  className="flex items-center gap-1.5 rounded px-2 py-1 text-xs hover:bg-sidebar-accent/40"
                  title={
                    s.kind === "url" && s.url
                      ? `${s.url} · ${s.format}`
                      : `${s.label} · ${s.format} (re-hydrated from the workspace snapshot)`
                  }
                >
                  {s.kind === "url" ? (
                    <Link2 className="size-3 shrink-0 text-muted-foreground" />
                  ) : (
                    <FileUp className="size-3 shrink-0 text-muted-foreground" />
                  )}
                  <span className="min-w-0 flex-1 truncate">{s.label}</span>
                  <span className="tabular text-[10px] uppercase text-muted-foreground">
                    {s.format}
                  </span>
                </li>
              ))}
          </ul>
        </div>
      )}

      {/* [OPUS-4.8] sq-ixc3.13 — the working "+ Import" entry point (opens the Import drawer). */}
      {/* [OPUS-4.8] sq-vw3ax (#820 redesign) — a dashed teal "drop zone" affordance for import. */}
      <button
        onClick={() => setOpen(true)}
        className="mt-2 flex w-full items-center gap-1.5 rounded-md border border-dashed border-teal-line px-2 py-1.5 text-xs text-muted-foreground hover:bg-teal-faint hover:text-foreground"
        title="Import RDF from a disk file (compressed / HDT), a URL, or pasted text"
        data-import-trigger="rail"
      >
        <Plus className="size-3" /> Import data…
      </button>

      {/* [OPUS-4.8] sq-xvj9 — the sibling EXPORT entry point: serialise the whole live store to a
          pretty Turtle / TriG / JSON-LD document and download it (disabled while the store is
          cold/empty). */}
      <ExportDataMenu />
    </div>
  );
}

function Section({
  label,
  icon,
  children,
}: {
  label: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="py-1">
      <h2 className="flex items-center gap-1.5 px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {icon}
        {label}
      </h2>
      {children}
    </section>
  );
}

export function LeftRail({ tools, activeId, onOpenTool }: LeftRailProps) {
  // [SONNET-4.6] sq-1odi9 — rail honesty regrouping: split working tools from coming-soon stubs.
  const [comingSoonOpen, setComingSoonOpen] = React.useState(false);

  // Grouping is DATA-DRIVEN off resolveTool(id).group so panel-file overrides (sq-lxomy /
  // sq-9nwab / sq-kwb74 / sq-iemfq) auto-move tools when they flip their panel override.
  const workingTools = tools.filter((t) => (resolveTool(t.id) ?? t).group === "working");
  const comingSoonTools = tools.filter((t) => (resolveTool(t.id) ?? t).group === "coming-soon");

  function renderToolButton(tool: ToolDef, toolGroup: "working" | "coming-soon") {
    const Icon = tool.icon;
    const resolved = resolveTool(tool.id) ?? tool;
    const tier = TIER_META[resolved.tier];
    const isActive = tool.id === activeId;
    return (
      <li key={tool.id}>
        {/* [OPUS-4.8] sq-ixc3.11 — `data-tool` is a stable e2e/test hook for selecting a
            tool in the rail (the tauri-driver smoke clicks data-tool="shacl"). */}
        <button
          data-tool={tool.id}
          data-tool-group={toolGroup}
          onClick={() => onOpenTool(tool.id)}
          className={cn(
            "group flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-xs hover:bg-sidebar-accent/50",
            // [OPUS-4.8] sq-vw3ax (#820 redesign) — the active tool gets the accent-lit
            // teal spine (a glowing left bar), not just a tint, so the brand leads.
            isActive && "sq-active-spine font-medium",
          )}
          title={`${resolved.blurb} — ${tier.label}`}
        >
          <Icon
            className={cn(
              "size-3.5 shrink-0",
              isActive ? "text-primary" : "text-muted-foreground",
            )}
          />
          <span className="min-w-0 flex-1 truncate">{tool.label}</span>
          <span
            className={cn("size-2 shrink-0 rounded-full", tier.dot)}
            aria-hidden
          />
        </button>
      </li>
    );
  }

  return (
    <nav className="flex w-56 shrink-0 flex-col overflow-y-auto border-r bg-sidebar text-sidebar-foreground">
      {/* (1) Workspace switcher — [SONNET-4.6] sq-lmlch: replaced stub with real switcher. */}
      <div className="border-b px-2 py-2">
        <WorkspaceSwitcher />
      </div>

      {/* (2) Datasets tree of the LIVE store. */}
      <Section label="Datasets" icon={<FolderTree className="size-3" />}>
        <DatasetsTree />
      </Section>

      {/* (3) TOOLS — each a VERB opened as a tab with its honesty-tier dot.
          [SONNET-4.6] sq-1odi9 — split into working (visible) + coming-soon (collapsed subgroup). */}
      <Section label="Tools">
        <ul className="px-1 pb-1">
          {workingTools.map((tool) => renderToolButton(tool, "working"))}
        </ul>

        {/* Coming soon subgroup — collapsed by default; expand to see honest stubs. */}
        <div className="px-1 pb-2">
          <button
            data-rail-section="coming-soon-toggle"
            aria-expanded={comingSoonOpen}
            onClick={() => setComingSoonOpen((o) => !o)}
            className="flex w-full items-center gap-1 px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground hover:text-foreground"
          >
            <ChevronRight
              className={cn("size-3 shrink-0 transition-transform", comingSoonOpen && "rotate-90")}
            />
            <span>Coming soon</span>
            <span className="ml-1 tabular-nums">({comingSoonTools.length})</span>
          </button>
          {comingSoonOpen && (
            <ul className="mt-0.5">
              {comingSoonTools.map((tool) => renderToolButton(tool, "coming-soon"))}
            </ul>
          )}
        </div>
      </Section>
    </nav>
  );
}
