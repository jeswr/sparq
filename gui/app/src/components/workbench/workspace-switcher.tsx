"use client";

// [SONNET-4.6] sq-lmlch — workspace switcher UI in the left rail.
//
// Renders a trigger button (active workspace name + ChevronDown) that opens a Popover listing
// ALL stored WorkspaceSummary rows (name, approxTriples, sourceCount, updatedAt). From the
// popover the user can:
//   • SWITCH to any workspace (save-current-snapshot invariant is owned by switchWorkspace)
//   • CREATE a new empty workspace (name prompt inline; uses createWorkspace which saves outgoing)
//   • RENAME a workspace (inline edit; uses renameWorkspace)
//   • DELETE a workspace (inline confirm; deleteWorkspace handles active-deletion fallback)
//   • see the honest backend label ("saved on device" / "in this browser" / "this session only")
//
// INVARIANT: the UI NEVER calls hydrateFromSnapshot directly — all lifecycle goes through the
// WorkspaceContext APIs (sq-lcd6e). Creating and switching both save the outgoing snapshot first.
//
// Cmd-K commands registered here: "New workspace" + "Switch workspace…" (Actions group) +
// per-workspace "Switch to …" rows for every non-active workspace.

import * as React from "react";
import { Popover as PopoverPrimitive } from "radix-ui";
import {
  ChevronDown,
  Check,
  Plus,
  Pencil,
  Trash2,
  Database,
  X,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { useWorkspace } from "@/lib/workspace-context";
import { useRegisterPaletteCommands } from "@/components/workbench/command-palette";
import type { PaletteCommand } from "@/lib/palette-commands";
import type { WorkspaceBackend } from "@sparq/client";

// ── Helpers ───────────────────────────────────────────────────────────────────

function backendLabel(backend: WorkspaceBackend | null): string {
  if (backend === "tauri") return "saved on device";
  if (backend === "web") return "saved in this browser";
  return "this session only";
}

function relativeTime(ts: number): string {
  const diff = Date.now() - ts;
  const s = Math.floor(diff / 1_000);
  if (s < 60) return "just now";
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  return `${d}d ago`;
}

// ── Inline-mode union — drives the popover's editing state ───────────────────

type InlineMode =
  | { kind: "idle" }
  | { kind: "creating" }
  | { kind: "renaming"; id: string }
  | { kind: "confirming-delete"; id: string; name: string };

// ── Component ─────────────────────────────────────────────────────────────────

export function WorkspaceSwitcher() {
  const {
    workspace,
    workspaces,
    backend,
    createWorkspace,
    renameWorkspace,
    deleteWorkspace,
    switchWorkspace,
  } = useWorkspace();

  const [open, setOpen] = React.useState(false);
  const [mode, setMode] = React.useState<InlineMode>({ kind: "idle" });
  const [inputValue, setInputValue] = React.useState("");
  const inputRef = React.useRef<HTMLInputElement>(null);

  // Focus the inline input whenever an editing mode activates.
  React.useEffect(() => {
    if (mode.kind === "creating" || mode.kind === "renaming") {
      // A 0-ms tick is enough to let the DOM update before focusing.
      const id = window.setTimeout(() => inputRef.current?.focus(), 0);
      return () => window.clearTimeout(id);
    }
  }, [mode.kind]);

  const resetMode = React.useCallback(() => {
    setMode({ kind: "idle" });
    setInputValue("");
  }, []);

  const handleCreate = React.useCallback(async () => {
    const name = inputValue.trim();
    if (!name) return;
    resetMode();
    await createWorkspace(name);
    setOpen(false);
  }, [inputValue, resetMode, createWorkspace]);

  const handleRename = React.useCallback(
    async (id: string) => {
      const name = inputValue.trim();
      if (!name) {
        resetMode();
        return;
      }
      resetMode();
      await renameWorkspace(id, name);
    },
    [inputValue, resetMode, renameWorkspace],
  );

  const handleDelete = React.useCallback(
    async (id: string) => {
      resetMode();
      // deleteWorkspace handles the active-deletion fallback: switches to the next most-recently-
      // updated workspace, or creates a fresh default if none remain. The UI does not need to
      // guard against "deleting the last workspace" separately — the context creates a new default.
      await deleteWorkspace(id);
    },
    [resetMode, deleteWorkspace],
  );

  const handleSwitch = React.useCallback(
    async (id: string) => {
      setOpen(false);
      if (id === workspace?.id) return;
      await switchWorkspace(id);
    },
    [workspace?.id, switchWorkspace],
  );

  // ── Palette commands ───────────────────────────────────────────────────────

  const paletteCommands = React.useMemo<PaletteCommand[]>(() => {
    const cmds: PaletteCommand[] = [
      {
        id: "workspace.new",
        group: "Actions",
        title: "New workspace",
        blurb: "Create a fresh, empty workspace",
        keywords: ["workspace", "new", "create", "add", "fresh"],
        icon: Plus,
        run: () => {
          setOpen(true);
          setMode({ kind: "creating" });
          setInputValue("");
        },
      },
      {
        id: "workspace.switch",
        group: "Actions",
        title: "Switch workspace…",
        blurb:
          workspaces.length > 1
            ? `${workspaces.length} workspaces available`
            : "Only one workspace",
        keywords: ["workspace", "switch", "open", "change"],
        icon: Database,
        disabled: workspaces.length <= 1,
        run: () => setOpen(true),
      },
    ];
    // Per-workspace "Switch to …" entries for every non-active workspace.
    for (const ws of workspaces) {
      if (ws.id === workspace?.id) continue;
      cmds.push({
        id: `workspace.switch.${ws.id}`,
        group: "Actions",
        title: `Switch to "${ws.name}"`,
        blurb: `${ws.approxTriples.toLocaleString()} triple${ws.approxTriples !== 1 ? "s" : ""} · ${ws.sourceCount} source${ws.sourceCount !== 1 ? "s" : ""}`,
        keywords: ["workspace", "switch", ws.name],
        icon: Database,
        run: () => {
          void switchWorkspace(ws.id);
        },
      });
    }
    return cmds;
  }, [workspaces, workspace?.id, switchWorkspace]);

  useRegisterPaletteCommands("workspace", paletteCommands);

  // ── Render ─────────────────────────────────────────────────────────────────

  const activeWorkspaceName = workspace?.name ?? "loading…";

  return (
    <PopoverPrimitive.Root
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (!o) resetMode();
      }}
    >
      <PopoverPrimitive.Trigger asChild>
        <button
          data-workspace-trigger
          className="flex w-full items-center gap-1.5 rounded-md border bg-background px-2 py-1.5 text-left text-xs hover:bg-sidebar-accent/40"
          title={`Active workspace: ${activeWorkspaceName} · ${backendLabel(backend)}`}
        >
          {/* Saved-status LED: green = workspace loaded + context has a backend */}
          <span
            className="size-2.5 shrink-0 rounded-full bg-[var(--success)] drop-shadow-[0_0_5px_var(--success)]"
            aria-hidden
          />
          <span
            className="flex-1 truncate font-medium"
            data-workspace-active-name
          >
            {activeWorkspaceName}
          </span>
          <ChevronDown className="size-3 shrink-0 text-muted-foreground" aria-hidden />
        </button>
      </PopoverPrimitive.Trigger>

      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          data-workspace-popover
          align="start"
          sideOffset={4}
          className={cn(
            "z-50 w-56 overflow-hidden rounded-lg border bg-popover p-1 text-popover-foreground shadow-md",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            "data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
          )}
        >
          {/* Honest persistence label */}
          <p className="px-2 pb-1 pt-0.5 text-[10px] text-muted-foreground" data-workspace-backend-label>
            {backendLabel(backend)}
          </p>

          {/* Workspace list */}
          <ul role="listbox" aria-label="Workspaces" className="space-y-0.5">
            {workspaces.map((ws) => {
              const isActive = ws.id === workspace?.id;
              const isRenaming = mode.kind === "renaming" && mode.id === ws.id;
              const isConfirming =
                mode.kind === "confirming-delete" && mode.id === ws.id;

              return (
                <li key={ws.id} role="option" aria-selected={isActive}>
                  {isRenaming ? (
                    // ── Inline rename ──────────────────────────────────────
                    <div className="flex items-center gap-1 px-1">
                      <input
                        ref={inputRef}
                        data-workspace-rename-input
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") void handleRename(ws.id);
                          if (e.key === "Escape") resetMode();
                        }}
                        className="min-w-0 flex-1 rounded border bg-background px-1.5 py-0.5 text-xs outline-none focus:ring-1 focus:ring-ring/50"
                        placeholder="Workspace name"
                        aria-label="Rename workspace"
                      />
                      <button
                        onClick={() => void handleRename(ws.id)}
                        className="rounded p-0.5 text-primary hover:bg-accent"
                        aria-label="Save rename"
                      >
                        <Check className="size-3" />
                      </button>
                      <button
                        onClick={resetMode}
                        className="rounded p-0.5 text-muted-foreground hover:bg-accent"
                        aria-label="Cancel rename"
                      >
                        <X className="size-3" />
                      </button>
                    </div>
                  ) : isConfirming ? (
                    // ── Delete confirm ──────────────────────────────────────
                    <div className="flex items-center gap-1 rounded px-2 py-1 text-xs">
                      <span className="min-w-0 flex-1 truncate text-destructive">
                        Delete &ldquo;{ws.name}&rdquo;?
                      </span>
                      <button
                        data-workspace-delete-confirm
                        onClick={() => void handleDelete(ws.id)}
                        className="rounded bg-destructive/10 px-1.5 py-0.5 text-[10px] font-medium text-destructive hover:bg-destructive/20"
                        aria-label={`Confirm delete workspace ${ws.name}`}
                      >
                        Delete
                      </button>
                      <button
                        onClick={resetMode}
                        className="rounded p-0.5 text-muted-foreground hover:bg-accent"
                        aria-label="Cancel delete"
                      >
                        <X className="size-3" />
                      </button>
                    </div>
                  ) : (
                    // ── Normal row ──────────────────────────────────────────
                    <div className="group flex items-center gap-1.5 rounded px-2 py-1 hover:bg-sidebar-accent/40">
                      <button
                        data-workspace-item={ws.id}
                        onClick={() => void handleSwitch(ws.id)}
                        className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
                        aria-label={`Switch to workspace ${ws.name}`}
                      >
                        {isActive ? (
                          <Check className="size-3 shrink-0 text-primary" />
                        ) : (
                          <span className="size-3 shrink-0" aria-hidden />
                        )}
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-xs font-medium">
                            {ws.name}
                          </span>
                          <span className="block truncate text-[10px] text-muted-foreground">
                            {ws.approxTriples.toLocaleString()}{" "}
                            {ws.approxTriples === 1 ? "triple" : "triples"} ·{" "}
                            {ws.sourceCount} src · {relativeTime(ws.updatedAt)}
                          </span>
                        </span>
                      </button>

                      {/* Rename + delete — shown on hover via group-hover */}
                      <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
                        <button
                          data-workspace-rename={ws.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            setMode({ kind: "renaming", id: ws.id });
                            setInputValue(ws.name);
                          }}
                          className="rounded p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                          aria-label={`Rename workspace ${ws.name}`}
                          title="Rename"
                        >
                          <Pencil className="size-3" />
                        </button>
                        <button
                          data-workspace-delete={ws.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            setMode({
                              kind: "confirming-delete",
                              id: ws.id,
                              name: ws.name,
                            });
                          }}
                          className="rounded p-0.5 text-muted-foreground hover:bg-destructive/20 hover:text-destructive"
                          aria-label={`Delete workspace ${ws.name}`}
                          title="Delete"
                        >
                          <Trash2 className="size-3" />
                        </button>
                      </div>
                    </div>
                  )}
                </li>
              );
            })}
          </ul>

          {/* Divider + New workspace */}
          <div className="my-1 border-t" />

          {mode.kind === "creating" ? (
            // ── Inline create ───────────────────────────────────────────────
            <div className="flex items-center gap-1 px-1 pb-0.5">
              <input
                ref={inputRef}
                data-workspace-create-input
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void handleCreate();
                  if (e.key === "Escape") resetMode();
                }}
                className="min-w-0 flex-1 rounded border bg-background px-1.5 py-0.5 text-xs outline-none focus:ring-1 focus:ring-ring/50"
                placeholder="New workspace name"
                aria-label="New workspace name"
              />
              <button
                data-workspace-create-confirm
                onClick={() => void handleCreate()}
                className="rounded p-0.5 text-primary hover:bg-accent"
                aria-label="Create workspace"
              >
                <Check className="size-3" />
              </button>
              <button
                onClick={resetMode}
                className="rounded p-0.5 text-muted-foreground hover:bg-accent"
                aria-label="Cancel create"
              >
                <X className="size-3" />
              </button>
            </div>
          ) : (
            <button
              data-workspace-new
              onClick={() => {
                setMode({ kind: "creating" });
                setInputValue("");
              }}
              className="flex w-full items-center gap-1.5 rounded px-2 py-1 text-xs text-muted-foreground hover:bg-sidebar-accent/40 hover:text-foreground"
            >
              <Plus className="size-3" />
              New workspace
            </button>
          )}
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
