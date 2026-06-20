// [OPUS-4.8] sq-atb0 (epic sq-ixc3) — the React hook that owns the persistent workspace
// lifecycle for the REPL: it resolves the best persistence backend once, lists/loads/saves/
// deletes workspaces through the framework-agnostic `WorkspaceStore`, and tracks the last-
// opened id for startup re-hydration. The wasm-Store snapshotting is NOT here — the hook deals
// only in the data model; the REPL composes it with `snapshotStore` / `restoreStoreFromSnapshot`
// so this hook stays free of engine coupling.

"use client";

import * as React from "react";
import {
  type Workspace,
  type WorkspaceBackend,
  type WorkspaceStore,
  type WorkspaceSummary,
  createWorkspaceStore,
} from "@sparq/client";
import { loadTauriFs } from "@/lib/tauri-fs";

/** The hook's public surface — what the REPL drives the workspace panel with. */
export interface UseWorkspaces {
  /** Whether the persistence backend has resolved yet (false during the initial async pick). */
  ready: boolean;
  /** Which concrete storage resolved: durable on Tauri/web, session-only in memory. */
  backend: WorkspaceBackend | null;
  /** Every stored workspace, most-recently-updated first. */
  list: WorkspaceSummary[];
  /** The id of the workspace last restored on startup, if any (for the REPL's initial open). */
  initialId: string | null;
  /** Load one workspace's full record by id. */
  load(id: string): Promise<Workspace | null>;
  /** Persist a workspace (create or overwrite); refreshes the list + records it as last-opened. */
  save(ws: Workspace): Promise<void>;
  /** Delete a workspace; refreshes the list. */
  remove(id: string): Promise<void>;
  /** Record which workspace is open so the next launch re-hydrates it. */
  setLastOpened(id: string | null): Promise<void>;
}

/**
 * Resolve + own the workspace persistence backend, exposing the CRUD surface the REPL needs.
 * The backend is picked ONCE on mount (`createWorkspaceStore`, feature-detecting Tauri fs →
 * localStorage → memory). All mutating calls go through the store and then refresh the cached
 * list. `initialId` is read once on mount so the REPL can restore the last session.
 */
export function useWorkspaces(): UseWorkspaces {
  const storeRef = React.useRef<WorkspaceStore | null>(null);
  const [ready, setReady] = React.useState(false);
  const [backend, setBackend] = React.useState<WorkspaceBackend | null>(null);
  const [list, setList] = React.useState<WorkspaceSummary[]>([]);
  const [initialId, setInitialId] = React.useState<string | null>(null);

  // Resolve the backend once, then load the list + the last-opened pointer.
  React.useEffect(() => {
    let cancelled = false;
    void (async () => {
      // The Tauri fs loader is passed unconditionally; `createWorkspaceStore` only calls it
      // INSIDE a Tauri webview (else it picks localStorage / memory), so the static export
      // never imports the Tauri plugin.
      const store = await createWorkspaceStore({ loadTauriFs });
      if (cancelled) return;
      storeRef.current = store;
      setBackend(store.backend);
      const [summaries, last] = await Promise.all([
        store.list(),
        store.lastOpenedId(),
      ]);
      if (cancelled) return;
      setList(summaries);
      setInitialId(last);
      setReady(true);
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const refresh = React.useCallback(async () => {
    const store = storeRef.current;
    if (!store) return;
    setList(await store.list());
  }, []);

  const load = React.useCallback(async (id: string) => {
    return (await storeRef.current?.load(id)) ?? null;
  }, []);

  const save = React.useCallback(
    async (ws: Workspace) => {
      const store = storeRef.current;
      if (!store) return;
      await store.save(ws);
      await store.setLastOpenedId(ws.id);
      await refresh();
    },
    [refresh],
  );

  const remove = React.useCallback(
    async (id: string) => {
      const store = storeRef.current;
      if (!store) return;
      await store.remove(id);
      await refresh();
    },
    [refresh],
  );

  const setLastOpened = React.useCallback(async (id: string | null) => {
    await storeRef.current?.setLastOpenedId(id);
  }, []);

  return {
    ready,
    backend,
    list,
    initialId,
    load,
    save,
    remove,
    setLastOpened,
  };
}
