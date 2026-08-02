"use client";

// [OPUS-5] sq-ixc3.22 (epic sq-ixc3) — the LENS editor: the UI over the framework-agnostic lens
// model in `@sparq/client` (`graph-lens.ts`). A lens is a named set of up to five SPARQL queries
// that define how the graph view explores and draws the store — start / expand / node basics /
// edge basics / node panel. This panel is the whole authoring surface: pick a lens, edit its
// slots, save it to the workspace, and copy or paste it as JSON to share.
//
// NO ENGINE ACCESS HERE. The panel edits DATA; the graph tool is what runs the queries. That
// split is deliberate — a lens must be inspectable and shareable without a live store, and it
// keeps this file free of the run/abort lifecycle.
//
// BUNDLE. This module is only ever imported by `graph-view-tool.tsx`, which the tool registry
// already loads through a lazy dynamic `import()` (the `.meta.ts` split), so nothing here
// enters the initial bundle — the frontend optional-code policy is satisfied by the existing
// tool-level split, with no second dynamic boundary needed.

import * as React from "react";
import { Check, Copy, Plus, Trash2, Undo2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useWorkspace } from "@/lib/workspace-context";
import {
  DEFAULT_LENS_ID,
  GRAPH_LENS_SLOTS,
  importGraphLens,
  newGraphLens,
  serializeGraphLens,
  type GraphLens,
  type GraphLensSlot,
} from "@sparq/client";

/** The transient status line under the action row (never an error dialog — this is an editor). */
type PanelStatus = { kind: "ok" | "error"; message: string } | null;

export function GraphLensPanel() {
  const { lenses, activeLens, setActiveLensId, saveLens, deleteLens } = useWorkspace();
  // The DRAFT is local: edits are not persisted until Save, so an experiment never clobbers a
  // working lens. Re-seeded whenever the selected lens changes (including after a save).
  const [draft, setDraft] = React.useState<GraphLens>(activeLens);
  const [status, setStatus] = React.useState<PanelStatus>(null);
  const [importText, setImportText] = React.useState("");

  React.useEffect(() => {
    setDraft(activeLens);
    setStatus(null);
  }, [activeLens]);

  const isBuiltin = draft.id === DEFAULT_LENS_ID;
  const dirty = React.useMemo(
    () => serializeGraphLens(draft) !== serializeGraphLens(activeLens),
    [draft, activeLens],
  );

  const setSlot = React.useCallback((slot: GraphLensSlot, text: string) => {
    setDraft((d) => ({ ...d, queries: { ...d.queries, [slot]: text } }));
  }, []);

  const onSave = React.useCallback(async () => {
    // The built-in lens is a stable reference point: saving an edit to it forks a copy instead
    // of overwriting it, so "Default" always means the same five queries. A name the user has
    // already changed is kept as-is; an unchanged one is suffixed so the two are tellable apart.
    const forkName =
      draft.name === activeLens.name ? `${draft.name} (copy)` : draft.name;
    const target = isBuiltin
      ? { ...newGraphLens(forkName), queries: draft.queries }
      : draft;
    await saveLens(target);
    setStatus({ kind: "ok", message: isBuiltin ? "Saved as a new lens." : "Lens saved." });
  }, [activeLens.name, draft, isBuiltin, saveLens]);

  const onNew = React.useCallback(async () => {
    await saveLens(newGraphLens("New lens", draft));
    setStatus({ kind: "ok", message: "New lens created from the current one." });
  }, [draft, saveLens]);

  const onDelete = React.useCallback(async () => {
    await deleteLens(draft.id);
    setStatus({ kind: "ok", message: "Lens deleted." });
  }, [draft.id, deleteLens]);

  const onCopy = React.useCallback(async () => {
    const json = serializeGraphLens(draft);
    try {
      await navigator.clipboard.writeText(json);
      setStatus({ kind: "ok", message: "Lens JSON copied to the clipboard." });
    } catch {
      // A webview without clipboard permission is a normal outcome, not a failure to hide:
      // drop the JSON into the import box so it can still be selected and copied by hand.
      setImportText(json);
      setStatus({
        kind: "error",
        message: "Clipboard unavailable — the JSON is in the box below to copy by hand.",
      });
    }
  }, [draft]);

  const onImport = React.useCallback(async () => {
    const lens = importGraphLens(importText);
    if (!lens) {
      setStatus({ kind: "error", message: "That is not a valid lens JSON document." });
      return;
    }
    await saveLens(lens);
    setImportText("");
    setStatus({ kind: "ok", message: `Imported “${lens.name}” as a new lens.` });
  }, [importText, saveLens]);

  return (
    <div className="flex h-full flex-col overflow-auto" data-graph-lens-panel>
      {/* Lens selection + lifecycle. */}
      <div className="flex flex-wrap items-center gap-1.5 border-b bg-card px-3 py-1.5">
        <label className="text-[11px] font-medium text-muted-foreground" htmlFor="graph-lens-pick">
          Lens
        </label>
        <select
          id="graph-lens-pick"
          className="h-7 rounded-md border bg-background px-1.5 text-xs"
          value={activeLens.id}
          onChange={(e) => void setActiveLensId(e.target.value)}
          title="The lens the graph view runs with"
        >
          {lenses.map((l) => (
            <option key={l.id} value={l.id}>
              {l.name}
            </option>
          ))}
        </select>
        <Button size="sm" variant="outline" onClick={() => void onNew()} title="Fork a new lens">
          <Plus className="size-3.5" /> New
        </Button>
        <Button size="sm" onClick={() => void onSave()} disabled={!dirty && !isBuiltin}>
          <Check className="size-3.5" /> {isBuiltin ? "Save as new" : "Save"}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => setDraft(activeLens)}
          disabled={!dirty}
          title="Discard unsaved edits"
        >
          <Undo2 className="size-3.5" /> Revert
        </Button>
        <Button size="sm" variant="ghost" onClick={() => void onCopy()} title="Copy this lens as JSON">
          <Copy className="size-3.5" /> Copy JSON
        </Button>
        <Button
          size="sm"
          variant="destructive"
          onClick={() => void onDelete()}
          disabled={isBuiltin}
          title={isBuiltin ? "The built-in lens cannot be deleted" : "Delete this lens"}
        >
          <Trash2 className="size-3.5" /> Delete
        </Button>
      </div>

      {status && (
        <p
          className={`border-b px-3 py-1 text-[11px] ${
            status.kind === "error" ? "text-destructive" : "text-muted-foreground"
          }`}
          data-graph-lens-status={status.kind}
        >
          {status.message}
        </p>
      )}

      <div className="flex flex-col gap-3 p-3">
        <div className="flex items-center gap-2">
          <label className="w-24 text-[11px] font-medium text-muted-foreground" htmlFor="graph-lens-name">
            Name
          </label>
          <input
            id="graph-lens-name"
            className="h-7 flex-1 rounded-md border bg-background px-2 text-xs"
            value={draft.name}
            onChange={(e) => setDraft((d) => ({ ...d, name: e.target.value }))}
          />
        </div>

        {isBuiltin && (
          <p className="text-[11px] text-muted-foreground">
            This is the built-in lens. Its <code className="rounded bg-muted px-1">?rank</code> is
            out-degree, computed by a SPARQL <code className="rounded bg-muted px-1">COUNT</code>{" "}
            over your data — the in-tab engine exposes no separate ranking index for it to read.
            Editing it saves a copy; the built-in stays as it is.
          </p>
        )}

        {GRAPH_LENS_SLOTS.map((spec) => (
          <div key={spec.slot} className="flex flex-col gap-1">
            <div className="flex items-baseline gap-2">
              <label
                className="text-[11px] font-medium text-foreground"
                htmlFor={`graph-lens-${spec.slot}`}
              >
                {spec.label}
              </label>
              <span className="text-[10px] uppercase text-muted-foreground">{spec.form}</span>
              <span className="text-[10px] text-muted-foreground">{spec.hint}</span>
            </div>
            <textarea
              id={`graph-lens-${spec.slot}`}
              className="min-h-16 w-full resize-y rounded-md border bg-background p-2 font-mono text-[11px] leading-relaxed"
              spellCheck={false}
              value={draft.queries[spec.slot] ?? ""}
              placeholder={`${spec.form} … (leave empty to disable this slot)`}
              onChange={(e) => setSlot(spec.slot, e.target.value)}
            />
          </div>
        ))}

        <div className="flex flex-col gap-1 border-t pt-3">
          <label className="text-[11px] font-medium text-foreground" htmlFor="graph-lens-import">
            Import a shared lens
          </label>
          <p className="text-[10px] text-muted-foreground">
            Paste a lens JSON document. It is always imported under a fresh id, so it can never
            overwrite one of your own lenses.
          </p>
          <textarea
            id="graph-lens-import"
            className="min-h-16 w-full resize-y rounded-md border bg-background p-2 font-mono text-[11px]"
            spellCheck={false}
            value={importText}
            onChange={(e) => setImportText(e.target.value)}
            placeholder='{ "id": "…", "name": "…", "queries": { … } }'
          />
          <div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => void onImport()}
              disabled={importText.trim() === ""}
            >
              Import
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
