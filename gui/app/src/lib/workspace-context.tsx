"use client";

// [OPUS-4.8] sq-ixc3.13 (epic sq-ixc3) — the active-WORKSPACE context for the operational GUI.
//
// The persistent-workspace MODEL + persistence abstraction is the framework-agnostic
// `@sparq/client` `workspace.ts` (sq-atb0): one `WorkspaceStore` interface with three runtime-
// selected backends (Tauri on-disk when the fs capability is granted, browser localStorage on
// the web target, in-memory fallback). This context is the GUI's thin host glue around it for
// the IMPORT path: it holds the active workspace's imported-source list (drives the left rail's
// Imports subgroup) and persists a workspace SNAPSHOT (the save/open cache) after each import.
//
// SCOPE (this bead). A SINGLE auto-created/restored workspace — the full switcher/create/delete
// UI is a separate phase (sq-atb0 model exists; the rail switcher button is still a stub). What
// this bead needs and lights up: record `WorkspaceSourceMeta` for each import + write the
// dataset snapshot so a re-open restores the imported store. NO secret is ever persisted.

import * as React from "react";
import {
  createWorkspaceStore,
  newWorkspace,
  type Workspace,
  type WorkspaceBackend,
  type WorkspaceInferenceMode,
  type WorkspaceSourceMeta,
  type WorkspaceStore,
} from "@sparq/client";

import { loadTauriFs } from "@/lib/tauri-fs";

/** The starting editor query a freshly-created workspace carries (kept in lockstep with the Query tool default). */
const STARTER_QUERY = "SELECT * WHERE { ?s ?p ?o } LIMIT 25";

export interface WorkspaceContextValue {
  /** The active workspace, or `null` until the store + workspace are restored on mount. */
  workspace: Workspace | null;
  /** Which concrete persistence backend resolved (for an honest "saved on device / in browser" label). */
  backend: WorkspaceBackend | null;
  /** The imported-source metadata list for the active workspace (drives the rail's Imports subgroup). */
  sources: WorkspaceSourceMeta[];
  /**
   * Record an imported source + persist a fresh dataset SNAPSHOT of the live store. Called by the
   * Import drawer on a successful ingest. `snapshot` is the live store's whole-dataset N-Quads
   * (the engine context's `snapshotStore()`), the save/open cache. Best-effort persistence: a
   * write failure does not throw (the import itself already succeeded in-memory).
   */
  recordImport: (source: WorkspaceSourceMeta, snapshot: string | null) => Promise<void>;
  /**
   * [OPUS-4.8] sq-tp1m (#757) — the active workspace's persisted INFERENCE regime (query-time
   * RDFS / OWL 2 RL entailment), defaulting to `"off"`. The inference-mode bridge pushes this
   * into the engine so the two stay in lockstep.
   */
  inference: WorkspaceInferenceMode;
  /**
   * [OPUS-4.8] sq-tp1m — persist a new inference regime for the active workspace (best-effort,
   * mirroring {@link recordImport}: a write failure never breaks the in-memory selection).
   */
  setInference: (mode: WorkspaceInferenceMode) => Promise<void>;
}

const WorkspaceContext = React.createContext<WorkspaceContextValue | null>(null);

export function WorkspaceProvider({ children }: { children: React.ReactNode }) {
  const [workspace, setWorkspace] = React.useState<Workspace | null>(null);
  const [backend, setBackend] = React.useState<WorkspaceBackend | null>(null);
  const storeRef = React.useRef<WorkspaceStore | null>(null);

  // Resolve the persistence backend + restore (or create) the active workspace once on mount.
  React.useEffect(() => {
    let cancelled = false;
    (async () => {
      const store = await createWorkspaceStore({ loadTauriFs });
      if (cancelled) return;
      storeRef.current = store;
      setBackend(store.backend);
      // Restore the last-opened workspace if there is one, else create a fresh default.
      let ws: Workspace | null = null;
      try {
        const lastId = await store.lastOpenedId();
        if (lastId) ws = await store.load(lastId);
      } catch {
        /* unreadable index — fall through to a fresh workspace */
      }
      if (!ws) {
        ws = newWorkspace("default workspace", STARTER_QUERY);
        try {
          await store.save(ws);
          await store.setLastOpenedId(ws.id);
        } catch {
          /* in-memory / locked-down backend — keep the workspace for this session anyway */
        }
      }
      if (!cancelled) setWorkspace(ws);
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const recordImport = React.useCallback(
    async (source: WorkspaceSourceMeta, snapshot: string | null): Promise<void> => {
      setWorkspace((prev) => {
        const base = prev ?? newWorkspace("default workspace", STARTER_QUERY);
        const next: Workspace = {
          ...base,
          sources: [...base.sources, source],
          dataSnapshot: snapshot ?? base.dataSnapshot,
          updatedAt: Date.now(),
        };
        // Persist the updated record (best-effort; the import already succeeded in-memory).
        const store = storeRef.current;
        if (store) {
          store
            .save(next)
            .then(() => store.setLastOpenedId(next.id))
            .catch(() => {
              /* a write failure must not break the in-memory import */
            });
        }
        return next;
      });
    },
    [],
  );

  // [OPUS-4.8] sq-tp1m (#757) — persist a new inference regime for the active workspace. Same
  // best-effort discipline as recordImport: update the in-memory workspace immediately and write
  // through to the backend without letting a persistence failure surface to the caller.
  const setInference = React.useCallback(
    async (mode: WorkspaceInferenceMode): Promise<void> => {
      setWorkspace((prev) => {
        const base = prev ?? newWorkspace("default workspace", STARTER_QUERY);
        if (base.inference === mode) return base;
        const next: Workspace = { ...base, inference: mode, updatedAt: Date.now() };
        const store = storeRef.current;
        if (store) {
          store
            .save(next)
            .then(() => store.setLastOpenedId(next.id))
            .catch(() => {
              /* a write failure must not break the in-memory selection */
            });
        }
        return next;
      });
    },
    [],
  );

  const value = React.useMemo<WorkspaceContextValue>(
    () => ({
      workspace,
      backend,
      sources: workspace?.sources ?? [],
      recordImport,
      inference: workspace?.inference ?? "off",
      setInference,
    }),
    [workspace, backend, recordImport, setInference],
  );

  return <WorkspaceContext.Provider value={value}>{children}</WorkspaceContext.Provider>;
}

export function useWorkspace(): WorkspaceContextValue {
  const ctx = React.useContext(WorkspaceContext);
  if (!ctx) throw new Error("useWorkspace must be used within a <WorkspaceProvider>");
  return ctx;
}
