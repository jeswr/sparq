"use client";

// [OPUS-4.8] sq-atb0 (epic sq-ixc3) — the WORKSPACE panel for the live SPARQL REPL.
//
// This is the React host for the persistent cross-session workspace model. It draws the
// workspace switcher (a named-workspace picker), the Save / New / Rename / Delete actions, and
// an HONEST backend label so the user knows whether their work is saved to disk (desktop),
// saved in this browser (web), or only kept for this session (the in-memory fallback). All the
// model + persistence logic lives in the framework-agnostic `@sparq/client` workspace module
// and the `useWorkspaces` hook; this component is just the controls + honesty copy.
//
// The save/open mechanism is a SNAPSHOT CACHE (maintainer decision on sq-atb0): saving captures
// the whole loaded dataset as an N-Quads snapshot plus the editor state plus source metadata;
// opening restores that exact dataset (no re-ingest from the original sources). For URL sources
// the metadata is kept so the source CAN be re-fetched; for local-file imports the original
// file cannot be silently re-read across sessions — the snapshot is the durable copy. The panel
// states this limitation plainly rather than implying a local file round-trips.

import * as React from "react";
import {
  FolderOpen,
  Save,
  Plus,
  Trash2,
  HardDrive,
  Database,
  Globe,
  Pencil,
} from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  type WorkspaceBackend,
  type WorkspaceSummary,
} from "@sparq/client";

export interface WorkspacePanelProps {
  /** Whether the persistence backend has resolved (controls are disabled until then). */
  ready: boolean;
  /** Which storage resolved — drives the honest "saved to …" label. */
  backend: WorkspaceBackend | null;
  /** Every stored workspace, most-recent first. */
  list: WorkspaceSummary[];
  /** The currently-open workspace id (or null when on an unsaved scratch session). */
  currentId: string | null;
  /** Open a stored workspace by id (restores its dataset + editor state). */
  onOpen: (id: string) => void;
  /** Save the current REPL state into the open workspace (overwrites it). */
  onSave: () => void;
  /** Save the current REPL state into a NEW workspace with the given name. */
  onSaveAs: (name: string) => void;
  /** Start a fresh, empty scratch session (does not delete anything). */
  onNew: () => void;
  /** Rename the open workspace. */
  onRename: (name: string) => void;
  /** Delete a stored workspace by id. */
  onDelete: (id: string) => void;
  /** True while a save/open is in flight (disables the action buttons). */
  busy?: boolean;
}

/**
 * The workspace controls row. A picker to switch workspaces, Save / Save-as / New / Rename /
 * Delete, and the backend-honesty badge. Mirrors the `DatasetControls` bar styling so the two
 * read as one family.
 */
export function WorkspacePanel({
  ready,
  backend,
  list,
  currentId,
  onOpen,
  onSave,
  onSaveAs,
  onNew,
  onRename,
  onDelete,
  busy,
}: WorkspacePanelProps) {
  const [saveAsOpen, setSaveAsOpen] = React.useState(false);
  const [renameOpen, setRenameOpen] = React.useState(false);
  const disabled = !ready || busy;
  const current = list.find((w) => w.id === currentId) ?? null;

  return (
    <div className="space-y-1.5">
      <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/30 p-2">
        <span className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
          <FolderOpen className="size-3.5" /> Workspace
        </span>

        <label htmlFor="repl-workspace" className="sr-only">
          Open workspace
        </label>
        <select
          id="repl-workspace"
          value={currentId ?? "__scratch__"}
          disabled={disabled}
          onChange={(e) => {
            const v = e.target.value;
            if (v === "__scratch__") onNew();
            else onOpen(v);
          }}
          className="h-7 min-w-40 rounded-md border bg-background px-2 text-xs outline-none focus-visible:ring-3 focus-visible:ring-ring/40 disabled:opacity-50"
        >
          <option value="__scratch__">
            {currentId === null ? "Unsaved scratch session" : "New scratch session…"}
          </option>
          {list.length > 0 && (
            <optgroup label="Saved workspaces">
              {list.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name}
                  {w.approxTriples > 0 ? ` · ${w.approxTriples} triples` : ""}
                </option>
              ))}
            </optgroup>
          )}
        </select>

        <span className="ml-auto flex flex-wrap items-center gap-1.5">
          <BackendBadge ready={ready} backend={backend} />

          <Button
            variant="outline"
            size="sm"
            disabled={disabled || currentId === null}
            title="Overwrite the open workspace with the current dataset + editor state"
            onClick={onSave}
          >
            <Save className="size-3.5" /> Save
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={disabled}
            title="Save the current dataset + editor state as a new named workspace"
            onClick={() => setSaveAsOpen(true)}
          >
            <Plus className="size-3.5" /> Save as
          </Button>
          {currentId !== null && (
            <>
              <Button
                variant="outline"
                size="sm"
                disabled={disabled}
                title="Rename the open workspace"
                onClick={() => setRenameOpen(true)}
              >
                <Pencil className="size-3.5" /> Rename
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={disabled}
                title="Delete the open workspace"
                onClick={() => {
                  onDelete(currentId);
                }}
              >
                <Trash2 className="size-3.5" /> Delete
              </Button>
            </>
          )}
        </span>
      </div>

      {/* Honest, persistent note about what the snapshot cache does and does NOT round-trip. */}
      <PersistenceNote backend={backend} sourceCount={current?.sourceCount ?? 0} />

      <NameDialog
        open={saveAsOpen}
        onOpenChange={setSaveAsOpen}
        title="Save as a new workspace"
        description="Captures the loaded dataset (as a snapshot), the imported-source list, and the SPARQL editor state under a name. Re-opening restores that exact dataset — no re-ingest."
        confirmLabel="Save workspace"
        defaultValue={current ? `${current.name} copy` : "My workspace"}
        onConfirm={(name) => {
          onSaveAs(name);
          setSaveAsOpen(false);
        }}
      />
      <NameDialog
        open={renameOpen}
        onOpenChange={setRenameOpen}
        title="Rename workspace"
        description="Give this workspace a new name."
        confirmLabel="Rename"
        defaultValue={current?.name ?? ""}
        onConfirm={(name) => {
          onRename(name);
          setRenameOpen(false);
        }}
      />
    </div>
  );
}

// [OPUS-4.8] sq-atb0 — the backend-honesty badge: tells the user exactly where their work is
// stored, so "persistent" is never overclaimed for the session-only memory fallback.
function BackendBadge({
  ready,
  backend,
}: {
  ready: boolean;
  backend: WorkspaceBackend | null;
}) {
  if (!ready || backend === null) {
    return (
      <Badge variant="muted" aria-live="polite">
        Resolving storage…
      </Badge>
    );
  }
  if (backend === "tauri") {
    return (
      <Badge variant="success" title="Saved to this device's local app-data on disk">
        <HardDrive className="size-3" /> Saved on device
      </Badge>
    );
  }
  if (backend === "web") {
    return (
      <Badge
        variant="default"
        title="Saved in this browser's localStorage — persists on this browser/profile only, and can be cleared by browser data-clearing"
      >
        <Globe className="size-3" /> Saved in this browser
      </Badge>
    );
  }
  return (
    <Badge
      variant="warning"
      title="No persistent storage available — workspaces are kept for this session only"
    >
      <Database className="size-3" /> This session only
    </Badge>
  );
}

// [OPUS-4.8] sq-atb0 — the standing honesty note. States what the snapshot cache restores and,
// crucially, the local-file limitation (a previously chosen local file cannot be silently
// re-read across sessions — the snapshot is the durable copy).
function PersistenceNote({
  backend,
  sourceCount,
}: {
  backend: WorkspaceBackend | null;
  sourceCount: number;
}) {
  const where =
    backend === "tauri"
      ? "to this device's local storage"
      : backend === "web"
        ? "in this browser"
        : "for this session only";
  return (
    <p className="rounded-lg border bg-muted/20 px-2.5 py-1.5 text-[11px] leading-relaxed text-muted-foreground">
      A workspace saves a <span className="font-medium">snapshot</span> of the loaded dataset,
      the imported-source list{sourceCount > 0 ? ` (${sourceCount})` : ""}, and the SPARQL
      editor state {where}. Re-opening restores that exact dataset — it does not re-ingest from
      the original files. URL sources keep their address so they can be re-fetched;{" "}
      <span className="font-medium">local file imports cannot be silently re-read</span> across
      sessions (the browser has no persistent handle), so the snapshot is their durable copy.
      Bearer tokens are never saved.
    </p>
  );
}

// [OPUS-4.8] sq-atb0 — a tiny name-entry dialog reused by Save-as + Rename. Reuses the shared
// Dialog primitive + the editor's input styling.
function NameDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel,
  defaultValue,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  title: string;
  description: string;
  confirmLabel: string;
  defaultValue: string;
  onConfirm: (name: string) => void;
}) {
  const [name, setName] = React.useState(defaultValue);
  // Reset to the default whenever the dialog (re)opens, so a stale edit never leaks across opens.
  React.useEffect(() => {
    if (open) setName(defaultValue);
  }, [open, defaultValue]);

  const submit = React.useCallback(() => {
    const trimmed = name.trim();
    if (!trimmed) {
      toast.error("Enter a workspace name");
      return;
    }
    onConfirm(trimmed);
  }, [name, onConfirm]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(28rem,calc(100vw-2rem))]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FolderOpen className="size-4 text-primary" /> {title}
          </DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <div className="space-y-3">
          <div className="space-y-1.5">
            <label htmlFor="workspace-name" className="text-xs font-medium">
              Workspace name
            </label>
            <input
              id="workspace-name"
              type="text"
              value={name}
              autoFocus
              spellCheck={false}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") submit();
              }}
              className="w-full rounded-lg border bg-muted/40 px-3 py-2 text-[13px] outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" disabled={!name.trim()} onClick={submit}>
              {confirmLabel}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
