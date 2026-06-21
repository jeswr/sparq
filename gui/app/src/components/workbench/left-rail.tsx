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
import { ChevronDown, Plus, Circle, Database, FolderTree } from "lucide-react";

import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { TIER_META, type ToolDef } from "@/data/tools";

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
      {/* Imports subgroup placeholder — the real WorkspaceSourceMeta list is sq-ixc3.13. */}
      <button
        className="mt-1 flex w-full cursor-not-allowed items-center gap-1.5 rounded px-2 py-1 text-xs text-muted-foreground"
        title="Import RDF from disk / URL / paste — the import drawer is a later phase"
        disabled
      >
        <Plus className="size-3" /> Import
      </button>
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
  return (
    <nav className="flex w-56 shrink-0 flex-col overflow-y-auto border-r bg-sidebar text-sidebar-foreground">
      {/* (1) Workspace switcher — the persistent model is sq-atb0; foundation = default. */}
      <div className="border-b px-2 py-2">
        <button
          className="flex w-full items-center gap-1.5 rounded-md border bg-background px-2 py-1.5 text-left text-xs hover:bg-sidebar-accent/40"
          title="Workspace switcher — persistent workspaces are a later phase (sq-atb0)"
        >
          <Circle className="size-2.5 fill-[var(--success)] text-[var(--success)]" />
          <span className="flex-1 truncate font-medium">default workspace</span>
          <ChevronDown className="size-3 text-muted-foreground" />
        </button>
      </div>

      {/* (2) Datasets tree of the LIVE store. */}
      <Section label="Datasets" icon={<FolderTree className="size-3" />}>
        <DatasetsTree />
      </Section>

      {/* (3) TOOLS — each a VERB opened as a tab with its honesty-tier dot. */}
      <Section label="Tools">
        <ul className="px-1 pb-2">
          {tools.map((tool) => {
            const Icon = tool.icon;
            const tier = TIER_META[tool.tier];
            const isActive = tool.id === activeId;
            return (
              <li key={tool.id}>
                <button
                  onClick={() => onOpenTool(tool.id)}
                  className={cn(
                    "group flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-xs hover:bg-sidebar-accent/50",
                    isActive && "bg-sidebar-accent/70 font-medium",
                  )}
                  title={`${tool.blurb} — ${tier.label}`}
                >
                  <Icon className="size-3.5 shrink-0 text-muted-foreground" />
                  <span className="min-w-0 flex-1 truncate">{tool.label}</span>
                  <span
                    className={cn("size-2 shrink-0 rounded-full", tier.dot)}
                    aria-hidden
                  />
                </button>
              </li>
            );
          })}
        </ul>
      </Section>
    </nav>
  );
}
