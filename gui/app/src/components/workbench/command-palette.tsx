"use client";

// [OPUS-4.8] sq-ixc3.10 (epic sq-ixc3) — the operational GUI's keyboard-first SPINE: the Cmd-K /
// Ctrl-K command palette (research/gui-design.md §A.3).
//
// This is the GUI's answer to surface count: any TOOL, every NAMED GRAPH, RECENT QUERIES, and the
// verbs run / run-EXPLAIN / browse-dataset / switch-tab are one fuzzy keystroke away, so the left
// rail need not show everything. It PARALLELS the website Cmd-K (site/src/components/command-
// palette.tsx) — same pattern, different shell: the website palette is navigational (jump to a
// surface/page); this one is OPERATIONAL (run a verb over the live store).
//
// CONTRIBUTOR REGISTRY. The palette is mounted once (in the Workbench). The shell + each tool
// contribute their commands via `useRegisterPaletteCommands` while mounted; the palette renders the
// flattened, grouped result. A ref-backed registry + a useSyncExternalStore read keeps a tool that
// re-registers every render (its callbacks change identity with state) from thrashing provider
// state. Built on the same `cmdk` the website palette uses (already vetted in this repo) for the
// fuzzy filter + keyboard nav + a11y; the dialog is the shared `radix-ui` primitive.

import * as React from "react";
import { Command } from "cmdk";
import { Dialog as DialogPrimitive } from "radix-ui";
import { Search, CornerDownLeft } from "lucide-react";

import { cn } from "@/lib/utils";
import {
  groupPaletteCommands,
  type PaletteCommand,
} from "@/lib/palette-commands";

// ── The contributor registry ─────────────────────────────────────────────────────────────────

interface Registry {
  register(sourceKey: string, commands: PaletteCommand[]): () => void;
  snapshot(): PaletteCommand[];
  subscribe(onChange: () => void): () => void;
}

const RegistryContext = React.createContext<Registry | null>(null);

// A context so the TopBar trigger (and any other chrome) can open the palette without prop-drilling.
const OpenContext = React.createContext<{
  open: boolean;
  setOpen: (open: boolean) => void;
} | null>(null);

/** Open / toggle the palette from anywhere inside <CommandPaletteProvider>. */
export function useCommandPalette() {
  const ctx = React.useContext(OpenContext);
  if (!ctx) {
    throw new Error("useCommandPalette must be used within <CommandPaletteProvider>");
  }
  return ctx;
}

const EMPTY: PaletteCommand[] = [];

/**
 * Contribute commands for the lifetime of the calling component. Pass a STABLE `sourceKey` (one per
 * logical contributor, e.g. "shell" / "query"). The list is re-registered whenever it changes
 * identity and removed on unmount. A no-op when no provider is mounted.
 */
export function useRegisterPaletteCommands(
  sourceKey: string,
  commands: PaletteCommand[],
): void {
  const registry = React.useContext(RegistryContext);
  React.useEffect(() => {
    if (!registry) return;
    return registry.register(sourceKey, commands);
  }, [registry, sourceKey, commands]);
}

/** The palette's read side: the live, flattened command list. */
function usePaletteCommands(): PaletteCommand[] {
  const registry = React.useContext(RegistryContext);
  const subscribe = React.useCallback(
    (onChange: () => void) => registry?.subscribe(onChange) ?? (() => {}),
    [registry],
  );
  const getSnapshot = React.useCallback(
    () => registry?.snapshot() ?? EMPTY,
    [registry],
  );
  return React.useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

// ── The fuzzy scorer (mirrors the website palette's substring scorer) ─────────────────────────

// cmdk's default fuzzy-subsequence scorer surfaces near-everything for a short query and lets DOM
// order break the ties. This substring scorer returns > 0 only on a real hit, weighting a TITLE
// match far above a keyword match, so the obvious row wins. Case-insensitive; whitespace-trimmed.
function scoreItem(value: string, search: string, keywords?: string[]): number {
  const q = search.trim().toLowerCase();
  if (!q) return 1;
  const title = value.toLowerCase();
  if (title === q) return 1;
  if (title.startsWith(q)) return 0.9;
  if (title.includes(q)) return 0.7;
  const hay = (keywords ?? []).join(" ").toLowerCase();
  if (hay.includes(q)) return 0.4;
  return 0;
}

// ── The provider + palette ────────────────────────────────────────────────────────────────────

/**
 * Mount once in the workbench. Owns the registry + the open state + the global ⌘K binding, and
 * renders the palette dialog.
 */
export function CommandPaletteProvider({ children }: { children: React.ReactNode }) {
  const sourcesRef = React.useRef<Map<string, PaletteCommand[]>>(new Map());
  const listenersRef = React.useRef<Set<() => void>>(new Set());
  const snapshotRef = React.useRef<PaletteCommand[]>([]);

  const registry = React.useMemo<Registry>(() => {
    const rebuild = () => {
      const flat: PaletteCommand[] = [];
      for (const list of sourcesRef.current.values()) flat.push(...list);
      snapshotRef.current = flat;
    };
    const notify = () => {
      for (const l of listenersRef.current) l();
    };
    return {
      register(sourceKey, commands) {
        sourcesRef.current.set(sourceKey, commands);
        rebuild();
        notify();
        return () => {
          // Only delete if this exact list is still registered — guards a late cleanup from a
          // remounted source clobbering a newer registration.
          if (sourcesRef.current.get(sourceKey) === commands) {
            sourcesRef.current.delete(sourceKey);
            rebuild();
            notify();
          }
        };
      },
      snapshot() {
        return snapshotRef.current;
      },
      subscribe(onChange) {
        listenersRef.current.add(onChange);
        return () => listenersRef.current.delete(onChange);
      },
    };
  }, []);

  const [open, setOpen] = React.useState(false);
  const openCtx = React.useMemo(() => ({ open, setOpen }), [open]);

  // Global ⌘K (macOS) / Ctrl-K (Windows/Linux) toggles the palette.
  React.useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((v) => !v);
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <RegistryContext.Provider value={registry}>
      <OpenContext.Provider value={openCtx}>
        {children}
        <CommandPaletteDialog open={open} onOpenChange={setOpen} />
      </OpenContext.Provider>
    </RegistryContext.Provider>
  );
}

function CommandPaletteDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const commands = usePaletteCommands();
  const groups = React.useMemo(() => groupPaletteCommands(commands), [commands]);

  // Run a command, then close. The command's own handler does the work.
  const runCommand = React.useCallback(
    (cmd: PaletteCommand) => {
      onOpenChange(false);
      cmd.run();
    },
    [onOpenChange],
  );

  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay
          className={cn(
            "fixed inset-0 z-50 bg-black/40 backdrop-blur-sm",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
          )}
        />
        <DialogPrimitive.Content
          className={cn(
            "bg-card text-card-foreground fixed left-1/2 top-[12vh] z-50 w-[min(38rem,calc(100vw-2rem))] -translate-x-1/2",
            "overflow-hidden rounded-xl p-0 shadow-2xl ring-1 ring-foreground/10",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
          )}
        >
          <DialogPrimitive.Title className="sr-only">Command palette</DialogPrimitive.Title>
          <DialogPrimitive.Description className="sr-only">
            Type to fuzzy-search every tool, action, recent query, and named graph, then press
            Enter to run it.
          </DialogPrimitive.Description>

          <Command label="Search tools, actions, recent queries and graphs" className="flex flex-col" filter={scoreItem}>
            <div className="flex items-center gap-2 border-b px-3">
              <Search className="size-4 shrink-0 text-muted-foreground" aria-hidden />
              <Command.Input
                autoFocus
                placeholder="Run a tool, action, recent query, or jump to a graph…"
                className="h-12 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
              />
              <kbd className="hidden rounded border px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground sm:inline-block">
                ESC
              </kbd>
            </div>

            <Command.List className="max-h-[min(60vh,28rem)] overflow-y-auto p-2">
              <Command.Empty className="px-3 py-6 text-center text-sm text-muted-foreground">
                No matches. Try a tool name, &ldquo;run&rdquo;, or &ldquo;explain&rdquo;.
              </Command.Empty>

              {groups.map(({ group, commands: rows }) => (
                <Command.Group
                  key={group}
                  heading={group}
                  className="px-1 pb-1 text-xs font-medium text-muted-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5"
                >
                  {rows.map((c) => (
                    <PaletteItem key={c.id} command={c} onRun={() => runCommand(c)} />
                  ))}
                </Command.Group>
              ))}
            </Command.List>

            <div className="flex items-center justify-between gap-2 border-t px-3 py-2 text-[11px] text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <CornerDownLeft className="size-3" aria-hidden /> to run
              </span>
              <span>
                Open with <kbd className="rounded border px-1 py-0.5 font-medium">⌘K</kbd> /{" "}
                <kbd className="rounded border px-1 py-0.5 font-medium">Ctrl K</kbd>
              </span>
            </div>
          </Command>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}

function PaletteItem({
  command,
  onRun,
}: {
  command: PaletteCommand;
  onRun: () => void;
}) {
  const Icon = command.icon;
  return (
    <Command.Item
      // Lead with the title (so the scorer matches it), append the id as an invisible
      // disambiguator so two rows with the same title do not collide in cmdk's value map.
      value={`${command.title} ${command.id}`}
      keywords={command.keywords}
      disabled={command.disabled}
      onSelect={onRun}
      className={cn(
        "flex items-center gap-3 rounded-md px-3 py-2 text-sm",
        command.disabled
          ? "cursor-not-allowed opacity-50"
          : "cursor-pointer text-foreground/90 data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground",
      )}
    >
      <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
        <Icon className="size-4" />
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{command.title}</span>
        {command.blurb && (
          <span className="truncate text-xs text-muted-foreground">{command.blurb}</span>
        )}
      </span>
    </Command.Item>
  );
}
