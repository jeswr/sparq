"use client";

// [OPUS-4.8] sq-ixc3.10 (epic sq-ixc3) — the OPERATIONAL command registry for the Cmd-K spine.
//
// The website Cmd-K (command-palette.tsx, sq-vw3ax.1) is navigational: it indexes the static
// IA (surfaces / pages / a couple of fixed actions). This registry adds the *live* operational
// layer the GUI design wants (research/gui-design.md §A.3): the workbench (the REPL) registers
// run / EXPLAIN / connect / export / import / switch-workspace / named-graph / recent-query
// commands while it is mounted, and the palette renders them ABOVE the navigation. When no
// workbench is mounted (most marketing pages) the registry is empty and the palette degrades to
// pure navigation — exactly how an operational palette should behave off the workbench.
//
// DESIGN. A ref-backed registry (not state) holds the current command set, and a version
// counter notifies subscribers — so a component re-registering its commands every render (the
// REPL's callbacks change identity as state changes) does NOT thrash React state on the
// provider. `useRegisterPaletteCommands` installs a command list for the lifetime of the
// calling component and replaces it whenever the list changes; `usePaletteCommands` is the
// palette's read side. This is the standard "command contributor" pattern for a cmdk palette
// driven by a live host, kept dependency-free + static-export-safe.

import * as React from "react";

import type { PaletteCommand } from "@/lib/palette-commands";

export type { PaletteCommand } from "@/lib/palette-commands";

interface Registry {
  /** Register a source's command list under a stable source key; returns an unregister fn. */
  register(sourceKey: string, commands: PaletteCommand[]): () => void;
  /** Snapshot every registered command, in source-registration order. */
  snapshot(): PaletteCommand[];
  /** Subscribe to registry changes (React 18 useSyncExternalStore contract). */
  subscribe(onChange: () => void): () => void;
}

const PaletteCommandsContext = React.createContext<Registry | null>(null);

/**
 * Provider mounted ONCE in the shell (inside the CommandPalette). Owns the live registry.
 * Children register operational commands via `useRegisterPaletteCommands`; the palette reads
 * them via `usePaletteCommands`.
 */
export function PaletteCommandsProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  // A ref-backed map keyed by source. We bump `versionRef` + notify on every change so the
  // external-store snapshot identity changes exactly when the command set does.
  const sourcesRef = React.useRef<Map<string, PaletteCommand[]>>(new Map());
  const listenersRef = React.useRef<Set<() => void>>(new Set());
  // Cache the flattened snapshot so useSyncExternalStore gets a STABLE reference between
  // changes (it compares by identity; a fresh array every read would loop-render).
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
          // Only delete if this exact list is still the registered one — guards against a
          // late cleanup clobbering a newer registration from the same remounted source.
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

  return (
    <PaletteCommandsContext.Provider value={registry}>
      {children}
    </PaletteCommandsContext.Provider>
  );
}

/**
 * The palette's read side: the live, flattened operational command list. Safe to call when
 * no provider is mounted (returns an empty list) so the palette never crashes off the shell.
 */
export function usePaletteCommands(): PaletteCommand[] {
  const registry = React.useContext(PaletteCommandsContext);
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

const EMPTY: PaletteCommand[] = [];

/**
 * Contribute a list of operational commands to the palette for the lifetime of the calling
 * component. Pass a STABLE `sourceKey` (one per logical contributor, e.g. "repl"). The list is
 * re-registered whenever `commands` changes identity, and removed on unmount. A no-op when no
 * provider is mounted (e.g. the component renders outside the shell in a test).
 */
export function useRegisterPaletteCommands(
  sourceKey: string,
  commands: PaletteCommand[],
): void {
  const registry = React.useContext(PaletteCommandsContext);
  React.useEffect(() => {
    if (!registry) return;
    return registry.register(sourceKey, commands);
  }, [registry, sourceKey, commands]);
}
