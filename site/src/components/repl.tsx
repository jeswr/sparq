"use client";

import * as React from "react";
import {
  Play,
  Loader2,
  Database,
  Zap,
  CheckCircle2,
  Table2,
  Braces,
  Download,
  PlugZap,
  Telescope,
  Gauge,
  History,
  Layers,
  FolderOpen,
  FilePlus2,
  Save,
} from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  loadSparq,
  loadIntoStore,
  prewarmSparqWhenIdle,
  storeToNQuads,
  datasetSize,
  extractTable,
  resultsToCsv,
  resultsToTsv,
  formatSparqlJson,
  serializeGraphAsJsonLd,
  type JsonLdMode,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
import { downloadText } from "@/lib/download";
import {
  classifyQueryForm,
  isGraphForm,
  modeSupportsForm,
  NAMED_GRAPH_STATS_QUERY,
  DEFAULT_GRAPH_COUNT_QUERY,
  parseGraphStats,
  type GraphStat,
  type QueryForm,
  type RunMode,
} from "@/lib/repl-dataset";
import { EXAMPLE_QUERIES, BUILTIN_DATASETS } from "@/data/sample-graph";
import {
  DatasetControls,
  DatasetViewer,
  type ActiveDataset,
} from "@/components/repl-datasets";
// [OPUS-4.8] sq-daru — the dataset panel: named-graph list with per-graph triple counts.
import { DatasetPanel } from "@/components/repl-dataset-panel";
// [OPUS-4.8] sq-n5aw — the syntax-highlighting SPARQL editor replaces the plain <textarea>.
import { SparqlEditor } from "@/components/sparql-editor";
// [OPUS-4.8] sq-8uew — Turtle/N-Triples syntax highlighting for the CONSTRUCT/DESCRIBE graph.
// [OPUS-4.8] sq-gb4o (#805) — pretty/indented Turtle (with a raw N-Triples toggle) for the graph.
import { TurtleResult } from "@/components/rdf-highlight";
// [OPUS-4.8] sq-oy1f.7 — JSON-LD output mode: the read-only JSON-LD syntax-highlight renderer
// for the CONSTRUCT/DESCRIBE graph serialised via the wasm engine's JSON-LD writer.
import { JsonLdHighlight } from "@/components/jsonld-editor";
// [OPUS-4.8] sq-2mke — endpoint mode: the Connect panel + the SPARQL 1.1 Protocol client.
import { ConnectPanel } from "@/components/connect-panel";
// [OPUS-4.8] sq-9ij6 — endpoint mode: the live subscriptions view (SSE result deltas).
import { SubscriptionsView } from "@/components/subscriptions-view";
// [OPUS-4.8] sq-he72 — endpoint mode: the server health / capabilities panel (metrics + VoID/SD).
import { ServerHealthPanel } from "@/components/server-health-panel";
import {
  type EndpointConfig,
  type Workspace,
  type WorkspaceSourceMeta,
  newWorkspace,
  runEndpointQuery,
} from "@sparq/client";
// [OPUS-4.8] sq-atb0 — the persistent cross-session workspace panel + its hook + the
// site-side wasm-Store ⇄ snapshot bridge.
import { WorkspacePanel } from "@/components/workspace-panel";
import { useWorkspaces } from "@/lib/use-workspaces";
import {
  snapshotStore,
  restoreStoreFromSnapshot,
} from "@/lib/workspace-snapshot";
// [OPUS-4.8] sq-ixc3.10 — the keyboard-first spine. The REPL is the workbench, so it
// CONTRIBUTES its operational verbs (run / EXPLAIN / connect / export / import / switch-
// workspace) plus the live named graphs + recent queries to the Cmd-K command palette via the
// shell-mounted registry. The pure command-model helpers (recent-query ring, graph/query
// labels) live in the unit-tested `@/lib/palette-commands`.
import { useRegisterPaletteCommands } from "@/components/palette-commands";
import {
  pushRecentQuery,
  previewQuery,
  graphLabel,
  type PaletteCommand,
  type RecentQuery,
} from "@/lib/palette-commands";

// [OPUS-4.8] sq-vfbm — the REPL now dispatches across the whole lean-bundle query
// surface, so the result state carries one variant per shape of answer: a solution
// table (SELECT), a boolean (ASK), a constructed N-Triples graph (CONSTRUCT/DESCRIBE),
// an Update acknowledgement (mutated the in-tab store), and the EXPLAIN plan text.
type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "select"; results: SparqlResults; ms: number }
  | { kind: "boolean"; value: boolean; ms: number }
  // [OPUS-4.8] sq-gb4o (#805) — `query` carries the SPARQL text the graph came from so the
  // pretty-Turtle view can abbreviate IRIs using the query's own PREFIX declarations.
  | { kind: "graph"; ntriples: string; triples: number; ms: number; query: string }
  // [OPUS-4.8] sq-2mke — `endpoint` marks an update applied to a REMOTE server (no
  // before/after in-tab count), so the result copy stays honest about which store mutated.
  | {
      kind: "update";
      sizeBefore: number;
      sizeAfter: number;
      ms: number;
      endpoint?: boolean;
    }
  | { kind: "explain"; plan: string; analyze: boolean; ms: number }
  | { kind: "error"; message: string };

// Run mode: execute the query, or render the planner's EXPLAIN / EXPLAIN ANALYZE
// plan without (EXPLAIN) or with (ANALYZE) executing it. The RunMode union now lives
// alongside the form classifier in repl-dataset so the per-form mode-support rule
// (modeSupportsForm) is a single pure, unit-tested source of truth.

// [OPUS-4.8] Engine warm-up lifecycle, surfaced as a subtle indicator. The wasm fetch +
// instantiate is kicked off on mount (prewarmSparq) so the first "Run query" is instant.
type EngineState = "cold" | "warming" | "ready" | "error";

const DEFAULT_DATASET = BUILTIN_DATASETS[0];

export function Repl() {
  const [sparql, setSparql] = React.useState(EXAMPLE_QUERIES[0].sparql);
  const [mode, setMode] = React.useState<RunMode>("run");
  const [state, setState] = React.useState<RunState>({ kind: "idle" });
  const [size, setSize] = React.useState<number | null>(null);
  const [engine, setEngine] = React.useState<EngineState>("cold");
  const [viewerOpen, setViewerOpen] = React.useState(false);
  // [OPUS-4.8] sq-daru — monotonic version bumped whenever the in-tab dataset's CONTENT
  // changes (a new dataset loaded, a merge, or an in-tab Update). The dataset panel keys
  // its per-graph-count re-read off it, so an Update that leaves the total size unchanged
  // but moves triples between graphs still refreshes the panel.
  const [datasetVersion, setDatasetVersion] = React.useState(0);
  const [active, setActive] = React.useState<ActiveDataset>({
    label: DEFAULT_DATASET.label,
    description: DEFAULT_DATASET.description,
  });
  const [activeBuiltinId, setActiveBuiltinId] = React.useState<string | null>(
    DEFAULT_DATASET.id,
  );
  const storeRef = React.useRef<WasmStore | null>(null);

  // [OPUS-4.8] sq-ixc3.10 — the keyboard-first spine's live inputs.
  //   • recentQueries: a session-only ring of the most-recently-RUN distinct queries, newest
  //     first (pure ring in `@/lib/palette-commands`). Re-selecting one drops it back into the
  //     editor. Session-only by design: it is workbench scratch, not persisted state.
  //   • namedGraphs: the dataset's NAMED graph IRIs, re-read from the live store off the same
  //     `datasetVersion` the dataset panel uses, so the palette's "Named graphs" group tracks
  //     the loaded data. The palette command opens the dataset viewer focused on the data.
  const [recentQueries, setRecentQueries] = React.useState<RecentQuery[]>([]);
  const [namedGraphs, setNamedGraphs] = React.useState<GraphStat[]>([]);

  // [OPUS-4.8] sq-2mke — endpoint mode. When `endpointActive`, queries route to a running
  // SPARQL 1.1 Protocol endpoint over the shared `@sparq/client` HTTP client instead of the
  // in-tab WASM store. The config (URL + optional bearer token) is lifted here so `run()`
  // can dispatch on it; the Connect panel owns the form + safety UX.
  const [endpointActive, setEndpointActive] = React.useState(false);
  const [endpointConfig, setEndpointConfig] = React.useState<EndpointConfig>({
    url: "http://127.0.0.1:3030/sparql",
    token: "",
  });

  // [OPUS-4.8] sq-atb0 — the persistent cross-session workspace state. `workspaces` owns the
  // resolved persistence backend (Tauri disk / browser localStorage / in-memory) and the CRUD
  // surface; `currentWorkspaceId` is the open workspace (null = an unsaved scratch session);
  // `sources` accumulates the import metadata (local + url) of every source loaded this session
  // so a saved workspace can re-draw its source list and re-fetch its URL sources. The dataset
  // itself is persisted as an N-Quads SNAPSHOT (the save/open cache), not re-ingested on open.
  const workspaces = useWorkspaces();
  const [currentWorkspaceId, setCurrentWorkspaceId] = React.useState<string | null>(
    null,
  );
  const [sources, setSources] = React.useState<WorkspaceSourceMeta[]>([]);
  const [workspaceBusy, setWorkspaceBusy] = React.useState(false);
  // Guards the one-shot startup re-hydration so it runs at most once per mount.
  const rehydratedRef = React.useRef(false);

  // Build (or rebuild) the store from RDF text + format. Centralises error handling so
  // every load path (default, picker, upload, URL) reports failures the same way.
  const buildStore = React.useCallback(
    async (text: string, format: string): Promise<WasmStore> => {
      const Store = await loadSparq();
      // [OPUS-4.8] sq-17nw — route quad formats through loadDataset so uploaded
      // N-Quads / TriG keep their named graphs (GRAPH ?g) instead of being folded
      // into the default graph. The badge counts the WHOLE dataset (the wasm
      // `size` getter counts the default graph only).
      const store = loadIntoStore(Store, text, format);
      storeRef.current = store;
      setSize(datasetSize(store));
      // [OPUS-4.8] sq-daru — new store content: refresh the dataset panel's per-graph counts.
      setDatasetVersion((v) => v + 1);
      return store;
    },
    [],
  );

  // [OPUS-4.8] sq-4296 (#935 / #981) — pre-warm the engine AND parse the default dataset on
  // the next browser-IDLE slot, not synchronously during mount. The REPL renders on the home
  // page and /try; firing the ~2.8 MB engine wasm fetch while the component mounts would
  // compete with the initial paint / first-input readiness. `prewarmSparqWhenIdle` defers the
  // fetch via `requestIdleCallback` (with a `setTimeout` fallback) so the page is interactive
  // FIRST, then the wasm loads in the background. The first "Run query" still awaits the
  // memoised `loadSparq()` via `ensureStore`, so an interaction before the idle warm-up
  // completes joins the in-flight load — it never calls into an uninitialised wasm.
  React.useEffect(() => {
    let cancelled = false;
    setEngine("warming");
    const handle = prewarmSparqWhenIdle({
      onReady: () => {
        if (cancelled || storeRef.current) {
          if (!cancelled) setEngine("ready");
          return;
        }
        // The engine is ready; build the default dataset off the render path, then mark ready.
        buildStore(DEFAULT_DATASET.text, DEFAULT_DATASET.format)
          .then(() => {
            if (!cancelled) setEngine("ready");
          })
          .catch((e) => {
            if (cancelled) return;
            setEngine("error");
            toast.error("Engine failed to load", {
              description: e instanceof Error ? e.message : String(e),
            });
          });
      },
      onError: (e) => {
        if (cancelled) return;
        setEngine("error");
        toast.error("Engine failed to load", {
          description: e instanceof Error ? e.message : String(e),
        });
      },
    });
    return () => {
      cancelled = true;
      handle.cancel();
    };
  }, [buildStore]);

  // Guarantees a store exists before a query runs — the safety net if pre-warm hasn't
  // finished (or failed): never lets "Run query" no-op or throw on a cold engine.
  const ensureStore = React.useCallback(async (): Promise<WasmStore> => {
    if (storeRef.current) return storeRef.current;
    setEngine("warming");
    const store = await buildStore(
      DEFAULT_DATASET.text,
      DEFAULT_DATASET.format,
    );
    setEngine("ready");
    return store;
  }, [buildStore]);

  // [OPUS-4.8] sq-2mke — endpoint-mode execution path. Routes the SAME editor text to the
  // configured SPARQL 1.1 Protocol endpoint over the shared `@sparq/client` HTTP client.
  // The form is classified to the four wire shapes (SELECT / ASK / CONSTRUCT-DESCRIBE /
  // Update) and the response parsed accordingly. EXPLAIN / ANALYZE are NOT routed here —
  // they are an in-tab planner introspection, so those modes stay WASM-only (the UI grays
  // them in endpoint mode). The bearer token is sent only in the Authorization header by
  // the client; it is never logged or echoed by the REPL.
  const runEndpoint = React.useCallback(async () => {
    setState({ kind: "running" });
    const t0 = performance.now();
    const result = await runEndpointQuery(endpointConfig, sparql);
    const ms = performance.now() - t0;
    switch (result.kind) {
      case "select":
        setState({ kind: "select", results: result.results, ms });
        return;
      case "boolean":
        setState({ kind: "boolean", value: result.value, ms });
        return;
      case "graph": {
        const trimmed = result.ntriples.trim();
        const triples = trimmed === "" ? 0 : trimmed.split("\n").length;
        setState({ kind: "graph", ntriples: trimmed, triples, ms, query: sparql });
        return;
      }
      case "update":
        // The endpoint owns the dataset; we cannot read a before/after size cheaply, so
        // surface the server's 204 acknowledgement honestly without inventing a count.
        setState({
          kind: "update",
          sizeBefore: 0,
          sizeAfter: 0,
          ms,
          endpoint: true,
        });
        return;
    }
  }, [endpointConfig, sparql]);

  // [OPUS-4.8] sq-ixc3.10 — remember a just-run query for the command palette's "Recent
  // queries" group. De-duplicated + capped + session-only by the pure ring helper.
  const recordRecent = React.useCallback((query: string) => {
    setRecentQueries((prev) => pushRecentQuery(prev, query, Date.now()));
  }, []);

  // [OPUS-4.8] sq-vfbm — dispatch across the whole lean-bundle query surface. EXPLAIN /
  // ANALYZE modes render the planner's plan text (every form for EXPLAIN; SELECT/ASK for
  // ANALYZE). Otherwise we classify the SPARQL form and route to the matching wasm export:
  // SELECT/ASK -> query (JSON), CONSTRUCT/DESCRIBE -> queryQuads (N-Triples), and an Update
  // keyword -> updateInPlace (mutates the in-tab store; we re-count the dataset after).
  const run = React.useCallback(async (overrideMode?: RunMode) => {
    // [OPUS-4.8] sq-ixc3.10 — the command palette runs a SPECIFIC mode (Run / EXPLAIN /
    // ANALYZE) without first flipping the mode-tab state and racing this read; an explicit
    // `overrideMode` wins over the current tab. The button path calls `run()` with no
    // argument, so it keeps using the selected tab exactly as before.
    const runMode = overrideMode ?? mode;
    // [OPUS-4.8] sq-ixc3.10 — record the query text in the recent ring for the spine. Done
    // up front (covers run / EXPLAIN / ANALYZE / endpoint) so the palette always offers what
    // you actually ran, even if the run then errors.
    recordRecent(sparql);
    // [OPUS-4.8] sq-2mke — when endpoint mode is active, the query runs on the remote
    // server, not the in-tab store. EXPLAIN / ANALYZE remain a WASM-only planner view, so
    // a "run" in endpoint mode always executes the query (the mode tabs are grayed).
    if (endpointActive) {
      try {
        await runEndpoint();
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        setState({ kind: "error", message });
        toast.error("Endpoint query failed", { description: message });
      }
      return;
    }
    try {
      const store = await ensureStore();
      const form = classifyQueryForm(sparql);
      setState({ kind: "running" });

      if (runMode === "explain" || runMode === "analyze") {
        const analyze = runMode === "analyze";
        const t0 = performance.now();
        const plan = analyze ? store.explainAnalyze(sparql) : store.explain(sparql);
        const ms = performance.now() - t0;
        setState({ kind: "explain", plan, analyze, ms });
        return;
      }

      if (form === "update") {
        const sizeBefore = datasetSize(store);
        const t0 = performance.now();
        store.updateInPlace(sparql);
        const ms = performance.now() - t0;
        const sizeAfter = datasetSize(store);
        setSize(sizeAfter);
        // [OPUS-4.8] sq-daru — an in-tab Update mutated the store (and may have moved
        // triples between graphs even if the total is unchanged): refresh the panel.
        setDatasetVersion((v) => v + 1);
        setState({ kind: "update", sizeBefore, sizeAfter, ms });
        return;
      }

      if (isGraphForm(form)) {
        const t0 = performance.now();
        const ntriples = store.queryQuads(sparql);
        const ms = performance.now() - t0;
        const trimmed = ntriples.trim();
        const triples = trimmed === "" ? 0 : trimmed.split("\n").length;
        setState({ kind: "graph", ntriples: trimmed, triples, ms, query: sparql });
        return;
      }

      const t0 = performance.now();
      const json = store.query(sparql);
      const ms = performance.now() - t0;
      const parsed = JSON.parse(json) as SparqlResults;
      if (typeof parsed.boolean === "boolean") {
        setState({ kind: "boolean", value: parsed.boolean, ms });
      } else {
        setState({ kind: "select", results: parsed, ms });
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setState({ kind: "error", message });
      toast.error(runMode === "run" ? "Query failed" : "EXPLAIN failed", {
        description: message,
      });
    }
  }, [ensureStore, sparql, mode, endpointActive, runEndpoint, recordRecent]);

  // Switch to a built-in dataset: reload the store, reset the count + active descriptor.
  const selectBuiltin = React.useCallback(
    async (id: string) => {
      const ds = BUILTIN_DATASETS.find((d) => d.id === id);
      if (!ds) return;
      try {
        await buildStore(ds.text, ds.format);
        setActiveBuiltinId(ds.id);
        setActive({ label: ds.label, description: ds.description });
        // [OPUS-4.8] sq-atb0 — a built-in dataset REPLACES the store, so it resets the imported-
        // source list: there are no user imports to re-list (the data lives in the snapshot).
        setSources([]);
        setState({ kind: "idle" });
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        toast.error("Could not load dataset", { description: message });
      }
    },
    [buildStore],
  );

  // Load a custom RDF document (upload / URL). "replace" swaps the store; "add" merges
  // by concatenating both graphs' N-Triples and re-parsing — format-agnostic and correct.
  const loadText = React.useCallback(
    async (
      text: string,
      format: string,
      label: string,
      mode: "replace" | "add",
      origin?: { kind: "local" | "url"; url?: string },
    ) => {
      try {
        const Store = await loadSparq();
        // Parse the incoming doc first so a parse error aborts BEFORE mutating state.
        // [OPUS-4.8] sq-17nw — loadIntoStore keeps named graphs for quad formats.
        const incoming = loadIntoStore(Store, text, format);
        if (mode === "add" && storeRef.current) {
          // Merge as N-Quads (default graph + every named graph) and re-load with
          // loadDataset, so the named graphs of BOTH stores survive the merge.
          const merged =
            storeToNQuads(storeRef.current) + "\n" + storeToNQuads(incoming);
          await buildStore(merged, "nquads");
          setActive((a) => ({
            label: `${a.label} + ${label}`,
            description: `Merged dataset (${size ?? 0} + new triples).`,
          }));
        } else {
          storeRef.current = incoming;
          setSize(datasetSize(incoming));
          // [OPUS-4.8] sq-daru — replaced the store directly (not via buildStore): refresh.
          setDatasetVersion((v) => v + 1);
          setActive({
            label,
            description: `Custom ${format} dataset loaded in your tab.`,
          });
        }
        setActiveBuiltinId(null);
        // [OPUS-4.8] sq-atb0 — record this import as workspace source metadata. A "replace"
        // load starts a fresh source list; an "add" appends. `origin` defaults to a local file
        // when the caller did not say (the only callers that omit it are programmatic loads).
        const meta: WorkspaceSourceMeta = {
          kind: origin?.kind ?? "local",
          label,
          url: origin?.kind === "url" ? origin.url : undefined,
          format,
          bytes:
            typeof TextEncoder !== "undefined"
              ? new TextEncoder().encode(text).length
              : text.length,
          importedAt: Date.now(),
        };
        setSources((prev) => (mode === "add" ? [...prev, meta] : [meta]));
        setState({ kind: "idle" });
        toast.success("Dataset loaded", {
          // Count the WHOLE dataset (default + named graphs), not just the default.
          description: `${label} — ${datasetSize(storeRef.current)} triples`,
        });
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        toast.error("Could not parse dataset", { description: message });
        throw e; // let the URL dialog surface it inline too
      }
    },
    [buildStore, size],
  );

  // [OPUS-4.8] sq-atb0 — apply a loaded workspace to the live REPL: re-hydrate the wasm store
  // from the workspace's N-Quads SNAPSHOT (the save/open cache — no re-ingest from source),
  // restore the editor state (query + run mode + endpoint view, never a bearer token), and
  // re-draw the imported-source list. Becomes the open workspace.
  const applyWorkspace = React.useCallback(
    async (ws: Workspace) => {
      const store = await restoreStoreFromSnapshot(ws.dataSnapshot);
      storeRef.current = store;
      setSize(datasetSize(store));
      setDatasetVersion((v) => v + 1);
      setActiveBuiltinId(null);
      setActive({
        label: ws.name,
        description: `Restored from workspace "${ws.name}" (${ws.sources.length} source${
          ws.sources.length === 1 ? "" : "s"
        }).`,
      });
      setSources(ws.sources);
      setSparql(ws.editor.query);
      setMode(ws.editor.mode);
      setEndpointActive(ws.editor.endpointActive);
      if (ws.editor.endpointUrl) {
        // Restore the endpoint URL but NEVER a token — a restored endpoint session re-prompts.
        setEndpointConfig((c) => ({ url: ws.editor.endpointUrl ?? c.url, token: "" }));
      }
      setCurrentWorkspaceId(ws.id);
      setState({ kind: "idle" });
    },
    [],
  );

  // [OPUS-4.8] sq-atb0 — assemble a Workspace record from the current REPL state: the
  // whole-dataset N-Quads snapshot, the imported-source metadata, and the editor state (no
  // token). Reused by both Save (overwrite) and Save-as (new id).
  const buildWorkspaceRecord = React.useCallback(
    (base: Workspace): Workspace => {
      const store = storeRef.current;
      return {
        ...base,
        sources,
        dataSnapshot: store ? snapshotStore(store) : undefined,
        editor: {
          query: sparql,
          mode,
          endpointActive,
          endpointUrl: endpointActive ? endpointConfig.url : undefined,
        },
      };
    },
    [sources, sparql, mode, endpointActive, endpointConfig.url],
  );

  // Open a stored workspace by id (from the switcher).
  const openWorkspace = React.useCallback(
    async (id: string) => {
      setWorkspaceBusy(true);
      try {
        const ws = await workspaces.load(id);
        if (!ws) {
          toast.error("Workspace not found", {
            description: "It may have been deleted in another tab.",
          });
          return;
        }
        await applyWorkspace(ws);
        await workspaces.setLastOpened(ws.id);
        toast.success("Workspace opened", { description: ws.name });
      } catch (e) {
        toast.error("Could not open workspace", {
          description: e instanceof Error ? e.message : String(e),
        });
      } finally {
        setWorkspaceBusy(false);
      }
    },
    [workspaces, applyWorkspace],
  );

  // Overwrite the open workspace with the current state.
  const saveWorkspace = React.useCallback(async () => {
    if (currentWorkspaceId === null) return;
    setWorkspaceBusy(true);
    try {
      const existing = await workspaces.load(currentWorkspaceId);
      const base = existing ?? newWorkspace("Workspace", sparql);
      await workspaces.save(buildWorkspaceRecord({ ...base, id: currentWorkspaceId }));
      toast.success("Workspace saved");
    } catch (e) {
      toast.error("Could not save workspace", {
        description: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setWorkspaceBusy(false);
    }
  }, [currentWorkspaceId, workspaces, buildWorkspaceRecord, sparql]);

  // Save the current state as a NEW named workspace (becomes the open one).
  const saveWorkspaceAs = React.useCallback(
    async (name: string) => {
      setWorkspaceBusy(true);
      try {
        const record = buildWorkspaceRecord(newWorkspace(name, sparql));
        await workspaces.save(record);
        setCurrentWorkspaceId(record.id);
        toast.success("Workspace created", { description: name });
      } catch (e) {
        toast.error("Could not save workspace", {
          description: e instanceof Error ? e.message : String(e),
        });
      } finally {
        setWorkspaceBusy(false);
      }
    },
    [buildWorkspaceRecord, workspaces, sparql],
  );

  // Rename the open workspace.
  const renameWorkspace = React.useCallback(
    async (name: string) => {
      if (currentWorkspaceId === null) return;
      setWorkspaceBusy(true);
      try {
        const existing = await workspaces.load(currentWorkspaceId);
        if (!existing) return;
        await workspaces.save({ ...existing, name });
        toast.success("Workspace renamed", { description: name });
      } catch (e) {
        toast.error("Could not rename workspace", {
          description: e instanceof Error ? e.message : String(e),
        });
      } finally {
        setWorkspaceBusy(false);
      }
    },
    [currentWorkspaceId, workspaces],
  );

  // Delete a stored workspace; if it was the open one, drop back to a scratch session.
  const deleteWorkspace = React.useCallback(
    async (id: string) => {
      setWorkspaceBusy(true);
      try {
        await workspaces.remove(id);
        if (id === currentWorkspaceId) {
          setCurrentWorkspaceId(null);
          await workspaces.setLastOpened(null);
        }
        toast.success("Workspace deleted");
      } catch (e) {
        toast.error("Could not delete workspace", {
          description: e instanceof Error ? e.message : String(e),
        });
      } finally {
        setWorkspaceBusy(false);
      }
    },
    [workspaces, currentWorkspaceId],
  );

  // Start a fresh scratch session: reload the default dataset, reset the editor + sources, and
  // detach from any open workspace (nothing is deleted).
  const newScratchSession = React.useCallback(async () => {
    setCurrentWorkspaceId(null);
    await workspaces.setLastOpened(null);
    setSources([]);
    setSparql(EXAMPLE_QUERIES[0].sparql);
    setMode("run");
    setActiveBuiltinId(DEFAULT_DATASET.id);
    setActive({
      label: DEFAULT_DATASET.label,
      description: DEFAULT_DATASET.description,
    });
    try {
      await buildStore(DEFAULT_DATASET.text, DEFAULT_DATASET.format);
    } catch {
      // The pre-warm effect / ensureStore will rebuild the default on the next run.
    }
    setState({ kind: "idle" });
  }, [workspaces, buildStore]);

  // [OPUS-4.8] sq-atb0 — startup re-hydration: once the persistence backend has resolved, if a
  // last-opened workspace exists, restore it (its snapshot + editor state). Runs at most once
  // per mount (the ref guard), and only after the engine pre-warm has built the default store,
  // so applying the snapshot replaces a known-good store. A failure is non-fatal — the user
  // keeps the default scratch session.
  React.useEffect(() => {
    if (rehydratedRef.current) return;
    if (!workspaces.ready || workspaces.initialId === null) return;
    if (engine !== "ready") return;
    rehydratedRef.current = true;
    void (async () => {
      try {
        const ws = await workspaces.load(workspaces.initialId as string);
        if (ws) await applyWorkspace(ws);
      } catch {
        // Keep the scratch session if re-hydration fails.
      }
    })();
  }, [workspaces.ready, workspaces.initialId, engine, workspaces, applyWorkspace]);

  const busy = state.kind === "running";
  const controlsDisabled = engine === "warming" || engine === "cold";

  // [OPUS-4.8] sq-xe4f — classify the current editor text so the mode toggle can gray
  // out EXPLAIN / ANALYZE for SPARQL Update forms (the query planner those modes drive
  // rejects Update). If the user had EXPLAIN/ANALYZE selected and then loads an Update
  // example, snap the mode back to "run" so the next click runs the Update instead of
  // hitting an engine parse error.
  const form: QueryForm = React.useMemo(
    () => classifyQueryForm(sparql),
    [sparql],
  );
  React.useEffect(() => {
    if (!modeSupportsForm(mode, form)) setMode("run");
  }, [mode, form]);

  // [OPUS-4.8] sq-2mke — EXPLAIN / ANALYZE are an in-tab planner introspection (they call
  // the WASM `explain`/`explainAnalyze`), so in endpoint mode they are unavailable; snap
  // back to "run" so the next click executes the query against the endpoint.
  React.useEffect(() => {
    if (endpointActive && mode !== "run") setMode("run");
  }, [endpointActive, mode]);

  // [OPUS-4.8] sq-ixc3.10 — keep the palette's "Named graphs" group in sync with the live
  // store. We re-read the NAMED graph IRIs (+ counts) off the same `datasetVersion` the dataset
  // panel keys off, using the engine's own queries (no new Store API, no baked figure). In
  // endpoint mode the remote server owns its data, so there are no in-tab named graphs to list.
  React.useEffect(() => {
    const store = storeRef.current;
    if (!store || endpointActive) {
      setNamedGraphs([]);
      return;
    }
    try {
      const namedJson = JSON.parse(store.query(NAMED_GRAPH_STATS_QUERY)) as SparqlResults;
      const defaultJson = JSON.parse(store.query(DEFAULT_GRAPH_COUNT_QUERY)) as SparqlResults;
      // parseGraphStats returns the default graph first, then named graphs; the palette only
      // lists the NAMED ones (the default graph is opened via the "Browse dataset" action).
      const stats = parseGraphStats(defaultJson, namedJson).filter((s) => !s.isDefault);
      setNamedGraphs(stats);
    } catch {
      setNamedGraphs([]);
    }
  }, [datasetVersion, endpointActive]);

  // [OPUS-4.8] sq-ixc3.10 — assemble the OPERATIONAL command set the REPL contributes to the
  // Cmd-K spine and register it for as long as the REPL is mounted. The list is memoised off the
  // live state it reads, so the palette always reflects the current workbench (mode availability,
  // open workspace, loaded graphs, recent queries) without prop-drilling. `runQuery` jumps to a
  // mode then runs; the editor/import/connect/export verbs reuse the SAME callbacks the panels do.
  const runWithMode = React.useCallback(
    (m: RunMode) => {
      // Reflect the chosen mode in the tab strip AND run it immediately — `run` takes the mode
      // explicitly, so there is no state-then-run race.
      setMode(m);
      void run(m);
    },
    [run],
  );

  const paletteCommands = React.useMemo<PaletteCommand[]>(() => {
    const cmds: PaletteCommand[] = [];

    // ── Actions: the verbs (run / EXPLAIN / connect / export / import / workspace) ──────────
    cmds.push({
      id: "repl.run",
      group: "Actions",
      title: "Run query",
      blurb: endpointActive ? "Execute on the connected endpoint" : "Execute on the in-tab engine",
      keywords: ["run", "execute", "query", "go"],
      icon: Play,
      disabled: busy,
      run: () => runWithMode("run"),
    });
    // EXPLAIN / ANALYZE drive the in-tab planner — unavailable in endpoint mode or for Updates.
    const planUnsupported = endpointActive || !modeSupportsForm("explain", form);
    cmds.push({
      id: "repl.explain",
      group: "Actions",
      title: "Run EXPLAIN",
      blurb: "Show the query plan without executing it",
      keywords: ["explain", "plan", "planner", "optimize"],
      icon: Telescope,
      disabled: busy || planUnsupported,
      run: () => runWithMode("explain"),
    });
    cmds.push({
      id: "repl.analyze",
      group: "Actions",
      title: "Run EXPLAIN ANALYZE",
      blurb: "Plan + execute with a per-operator trace (SELECT/ASK)",
      keywords: ["analyze", "analyse", "trace", "profile", "explain"],
      icon: Gauge,
      disabled: busy || endpointActive || !modeSupportsForm("analyze", form),
      run: () => runWithMode("analyze"),
    });
    cmds.push({
      id: "repl.connect",
      group: "Actions",
      title: endpointActive ? "Disconnect from endpoint" : "Connect to a SPARQL endpoint",
      blurb: endpointActive
        ? "Switch back to the in-tab WASM engine"
        : "Run against a running sparq-server (SPARQL 1.1 Protocol)",
      keywords: ["connect", "disconnect", "endpoint", "server", "remote", "http"],
      icon: PlugZap,
      run: () => setEndpointActive((v) => !v),
    });
    // Export the current SELECT result set (only when one is on screen).
    const haveSelect = state.kind === "select";
    cmds.push({
      id: "repl.export.csv",
      group: "Actions",
      title: "Export results as CSV",
      blurb: haveSelect ? undefined : "Run a SELECT first to export its rows",
      keywords: ["export", "download", "csv", "results", "save"],
      icon: Download,
      disabled: !haveSelect,
      run: () => {
        if (state.kind === "select")
          downloadText("sparql-results.csv", resultsToCsv(state.results), "text/csv");
      },
    });
    cmds.push({
      id: "repl.export.tsv",
      group: "Actions",
      title: "Export results as TSV",
      blurb: haveSelect ? undefined : "Run a SELECT first to export its rows",
      keywords: ["export", "download", "tsv", "results", "save"],
      icon: Download,
      disabled: !haveSelect,
      run: () => {
        if (state.kind === "select")
          downloadText(
            "sparql-results.tsv",
            resultsToTsv(state.results),
            "text/tab-separated-values",
          );
      },
    });
    cmds.push({
      id: "repl.export.json",
      group: "Actions",
      title: "Export results as JSON",
      blurb: haveSelect ? undefined : "Run a SELECT first to export its rows",
      keywords: ["export", "download", "json", "results", "save", "sparql-results"],
      icon: Download,
      disabled: !haveSelect,
      run: () => {
        if (state.kind === "select")
          downloadText(
            "sparql-results.json",
            formatSparqlJson(state.results),
            "application/sparql-results+json",
          );
      },
    });
    cmds.push({
      id: "repl.browse",
      group: "Actions",
      title: "Browse the loaded dataset",
      blurb: "Open the dataset viewer (default + named graphs)",
      keywords: ["browse", "view", "dataset", "data", "triples", "import"],
      icon: Database,
      disabled: endpointActive || size === null,
      run: () => setViewerOpen(true),
    });
    cmds.push({
      id: "repl.new",
      group: "Actions",
      title: "New scratch session",
      blurb: "Reset to the default dataset + example query",
      keywords: ["new", "scratch", "reset", "fresh", "clear"],
      icon: FilePlus2,
      run: () => void newScratchSession(),
    });
    cmds.push({
      id: "repl.save",
      group: "Actions",
      title: "Save workspace",
      blurb:
        currentWorkspaceId === null
          ? "No open workspace — use Save as instead"
          : "Overwrite the open workspace",
      keywords: ["save", "workspace", "persist", "store"],
      icon: Save,
      disabled: !workspaces.ready || workspaceBusy || currentWorkspaceId === null,
      run: () => void saveWorkspace(),
    });

    // ── Recent queries: re-load a recently-run query into the editor ───────────────────────
    for (const r of recentQueries) {
      const preview = previewQuery(r.query);
      cmds.push({
        id: `repl.recent.${r.ranAt}`,
        group: "Recent queries",
        title: preview,
        keywords: ["recent", "history", "query", preview],
        icon: History,
        run: () => setSparql(r.query),
      });
    }

    // ── Named graphs: open the dataset viewer focused on the loaded data ────────────────────
    for (const g of namedGraphs) {
      if (g.iri === null) continue; // named graphs always carry an IRI (default filtered out)
      const iri = g.iri;
      cmds.push({
        id: `repl.graph.${iri}`,
        group: "Named graphs",
        title: graphLabel(iri),
        blurb: `${iri} · ${g.count} triple${g.count === 1 ? "" : "s"}`,
        keywords: ["graph", "named graph", iri],
        icon: Layers,
        run: () => setViewerOpen(true),
      });
    }

    // ── Workspaces: switch to any stored workspace by name ─────────────────────────────────
    for (const w of workspaces.list) {
      if (w.id === currentWorkspaceId) continue; // already open
      cmds.push({
        id: `repl.workspace.${w.id}`,
        group: "Workspaces",
        title: `Switch to “${w.name}”`,
        blurb: w.approxTriples > 0 ? `${w.approxTriples} triples` : undefined,
        keywords: ["workspace", "switch", "open", w.name],
        icon: FolderOpen,
        disabled: !workspaces.ready || workspaceBusy,
        run: () => void openWorkspace(w.id),
      });
    }

    return cmds;
  }, [
    endpointActive,
    busy,
    form,
    state,
    size,
    currentWorkspaceId,
    workspaceBusy,
    workspaces.ready,
    workspaces.list,
    recentQueries,
    namedGraphs,
    runWithMode,
    newScratchSession,
    saveWorkspace,
    openWorkspace,
  ]);

  useRegisterPaletteCommands("repl", paletteCommands);

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <Zap className="size-4 text-primary" />
          Live SPARQL REPL
        </CardTitle>
        <div className="flex items-center gap-2">
          {endpointActive ? (
            // [OPUS-4.8] sq-2mke — in endpoint mode the in-tab WASM engine state + triple
            // count are irrelevant; show that queries route to the remote endpoint.
            <Badge variant="default" aria-live="polite">
              <PlugZap className="size-3" /> Endpoint mode
            </Badge>
          ) : (
            <>
              <EngineIndicator engine={engine} />
              {size !== null && (
                <button
                  type="button"
                  onClick={() => setViewerOpen(true)}
                  aria-label={`View the ${size} triples in the loaded dataset`}
                  className="rounded-4xl outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
                >
                  <Badge
                    variant="muted"
                    className="tabular cursor-pointer transition-colors hover:bg-muted-foreground/20"
                  >
                    <Database className="size-3" /> {size} triples
                  </Badge>
                </button>
              )}
            </>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {/* [OPUS-4.8] sq-atb0 — the persistent cross-session workspace panel: save / open the
            loaded dataset (as a snapshot) + the imported-source list + the SPARQL editor state,
            so a session survives an app/browser restart. Backend resolves to Tauri disk on the
            desktop app, browser localStorage on GitHub Pages, or an in-memory session fallback. */}
        <WorkspacePanel
          ready={workspaces.ready}
          backend={workspaces.backend}
          list={workspaces.list}
          currentId={currentWorkspaceId}
          onOpen={(id) => void openWorkspace(id)}
          onSave={() => void saveWorkspace()}
          onSaveAs={(name) => void saveWorkspaceAs(name)}
          onNew={() => void newScratchSession()}
          onRename={(name) => void renameWorkspace(name)}
          onDelete={(id) => void deleteWorkspace(id)}
          busy={workspaceBusy}
        />

        <ConnectPanel
          config={endpointConfig}
          onConfigChange={setEndpointConfig}
          active={endpointActive}
          onActiveChange={setEndpointActive}
        />

        {/* The in-tab dataset controls manage the WASM store; in endpoint mode the
            endpoint owns the data, so they are disabled with an honest note. */}
        {endpointActive ? (
          <p className="rounded-lg border bg-muted/30 p-2.5 text-xs text-muted-foreground">
            Endpoint mode is active — queries run against the configured server, which owns
            its own dataset. The in-tab dataset picker below is for the WASM engine; switch
            back to <span className="font-medium">In-tab WASM</span> to use it.
          </p>
        ) : null}
        <DatasetControls
          activeBuiltinId={activeBuiltinId}
          onSelectBuiltin={selectBuiltin}
          onLoadText={loadText}
          disabled={controlsDisabled || endpointActive}
        />

        {/* [OPUS-4.8] sq-daru — the dataset panel: the loaded dataset's graphs (default +
            named) with per-graph triple counts, refreshed on every content change. Hidden
            in endpoint mode (the remote server owns its own dataset). */}
        <DatasetPanel
          store={storeRef.current}
          refreshKey={datasetVersion}
          hidden={endpointActive}
        />

        <div className="flex flex-wrap gap-1.5">
          {EXAMPLE_QUERIES.map((q) => (
            <Button
              key={q.label}
              variant="outline"
              size="sm"
              onClick={() => setSparql(q.sparql)}
            >
              {q.label}
            </Button>
          ))}
        </div>

        <label htmlFor="repl-query" className="sr-only">
          SPARQL query
        </label>
        <SparqlEditor
          id="repl-query"
          ariaLabel="SPARQL query"
          value={sparql}
          onChange={setSparql}
          rows={9}
        />

        <div className="flex flex-wrap items-center gap-3">
          <Button onClick={() => void run()} disabled={busy}>
            {busy ? (
              <Loader2 className="size-4 animate-spin" />
            ) : (
              <Play className="size-4" />
            )}
            {mode === "run" ? "Run query" : "Run EXPLAIN"}
          </Button>
          <ModeTabs
            mode={mode}
            onChange={setMode}
            disabled={busy}
            form={form}
            endpointMode={endpointActive}
          />
          <p aria-live="polite" className="text-xs text-muted-foreground">
            {state.kind === "select" &&
              `${state.results.results.bindings.length} rows · ${state.ms.toFixed(1)} ms`}
            {state.kind === "boolean" && `${state.ms.toFixed(1)} ms`}
            {state.kind === "graph" &&
              `${state.triples} triples · ${state.ms.toFixed(1)} ms`}
            {state.kind === "update" &&
              (state.endpoint
                ? `endpoint write acknowledged · ${state.ms.toFixed(1)} ms`
                : `store ${state.sizeBefore} → ${state.sizeAfter} triples · ${state.ms.toFixed(1)} ms`)}
            {state.kind === "explain" &&
              `plan ${state.analyze ? "+ trace " : ""}· ${state.ms.toFixed(1)} ms`}
            {state.kind === "running" &&
              (endpointActive
                ? "Running on the endpoint…"
                : mode === "run"
                  ? "Running on the wasm engine…"
                  : "Planning on the wasm engine…")}
            {state.kind === "idle" &&
              !endpointActive &&
              engine === "warming" &&
              "Pre-warming the wasm engine…"}
          </p>
        </div>

        <ResultPanel state={state} />

        {/* [OPUS-4.8] sq-9ij6 — the live subscriptions view. Only meaningful in endpoint
            mode (it streams from a real, mutating server's /subscriptions/sse), so it
            renders an honest "switch on endpoint mode" hint otherwise. It reuses the SAME
            endpoint config + bearer/connection-safety posture the Connect panel established. */}
        <SubscriptionsView config={endpointConfig} active={endpointActive} />

        {/* [OPUS-4.8] sq-he72 — the server health / capabilities panel. Reads the connected
            server's /health, Prometheus /metrics, and the opt-in VoID / SPARQL Service
            Description, rendering "not exposed" honestly when the operator left a feature off.
            Like the subscriptions view, it only runs in endpoint mode and reuses the SAME
            endpoint config + bearer/connection-safety posture. */}
        <ServerHealthPanel config={endpointConfig} active={endpointActive} />
      </CardContent>

      <DatasetViewer
        open={viewerOpen}
        onOpenChange={setViewerOpen}
        store={storeRef.current}
        size={size}
        active={active}
      />
    </Card>
  );
}

// [OPUS-4.8] Subtle engine-readiness pill. Reuses the badge tokens; never blocks the UI.
function EngineIndicator({ engine }: { engine: EngineState }) {
  if (engine === "ready") {
    return (
      <Badge variant="success" aria-live="polite">
        <CheckCircle2 className="size-3" /> Engine ready
      </Badge>
    );
  }
  if (engine === "error") {
    return (
      <Badge variant="warning" aria-live="polite">
        Engine failed — retries on run
      </Badge>
    );
  }
  return (
    <Badge variant="muted" aria-live="polite">
      <Loader2 className="size-3 animate-spin" /> Engine loading…
    </Badge>
  );
}

// [OPUS-4.8] sq-vfbm — Run / EXPLAIN / EXPLAIN ANALYZE selector. EXPLAIN is a
// planning-only dry run (every query form); ANALYZE also executes (SELECT/ASK).
// [OPUS-4.8] sq-xe4f — EXPLAIN / ANALYZE drive the query planner, which rejects SPARQL
// Update forms, so those tabs are grayed (disabled + aria-disabled) when the editor
// holds an Update example. The decision is the pure `modeSupportsForm` predicate.
function ModeTabs({
  mode,
  onChange,
  disabled,
  form,
  endpointMode,
}: {
  mode: RunMode;
  onChange: (m: RunMode) => void;
  disabled: boolean;
  form: QueryForm;
  // [OPUS-4.8] sq-2mke — in endpoint mode EXPLAIN / ANALYZE are unavailable (they drive
  // the in-tab WASM planner, not the remote server), so those tabs are grayed.
  endpointMode: boolean;
}) {
  const tabs: { value: RunMode; label: string; title: string }[] = [
    { value: "run", label: "Run", title: "Execute the query / update" },
    {
      value: "explain",
      label: "EXPLAIN",
      title: "Show the query plan without executing it",
    },
    {
      value: "analyze",
      label: "ANALYZE",
      title: "Show the plan and execute it with a per-operator trace (SELECT/ASK)",
    },
  ];
  return (
    <div
      role="tablist"
      aria-label="Run mode"
      className="inline-flex rounded-lg border bg-muted/40 p-0.5"
    >
      {tabs.map((t) => {
        // Gray out a mode the current query form cannot use (EXPLAIN/ANALYZE on an
        // Update). The engine still validates — this only stops the user picking a
        // mode that would parse-error. EXPLAIN/ANALYZE are also unavailable in endpoint
        // mode (an in-tab planner introspection, not a protocol operation).
        const planMode = t.value !== "run";
        const unsupported = !modeSupportsForm(t.value, form) || (endpointMode && planMode);
        const tabDisabled = disabled || unsupported;
        return (
          <button
            key={t.value}
            type="button"
            role="tab"
            aria-selected={mode === t.value}
            aria-disabled={unsupported}
            title={
              endpointMode && planMode
                ? `${t.label} runs the in-tab WASM planner — switch to In-tab WASM to use it`
                : unsupported
                  ? `${t.label} is unavailable for SPARQL Update — it plans a query`
                  : t.title
            }
            disabled={tabDisabled}
            onClick={() => onChange(t.value)}
            className={cn(
              "rounded-md px-2.5 py-1 text-xs font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40 disabled:opacity-50 disabled:cursor-not-allowed",
              mode === t.value
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {t.label}
          </button>
        );
      })}
    </div>
  );
}

function ResultPanel({ state }: { state: RunState }) {
  if (state.kind === "error") {
    return (
      <pre className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
        {state.message}
      </pre>
    );
  }
  if (state.kind === "boolean") {
    return (
      <div
        data-result-kind="boolean"
        data-ask-value={state.value ? "true" : "false"}
        className={cn(
          "rounded-lg p-3 text-sm font-medium",
          state.value
            ? "bg-[color-mix(in_oklch,var(--success)_15%,transparent)] text-[var(--success)]"
            : "bg-muted text-muted-foreground",
        )}
      >
        ASK → {state.value ? "true" : "false"}
      </div>
    );
  }
  if (state.kind === "update") {
    // [OPUS-4.8] sq-2mke — endpoint updates mutate the REMOTE server (the protocol acks a
    // 204 with no body), so we cannot show a before/after count without inventing one.
    if (state.endpoint) {
      return (
        <div className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
          <span className="font-medium text-foreground">Update applied</span> on the
          endpoint — the server acknowledged the write (HTTP 204, no body). Run a SELECT
          against the same endpoint to see the change.
        </div>
      );
    }
    const delta = state.sizeAfter - state.sizeBefore;
    const sign = delta > 0 ? "+" : "";
    return (
      <div className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
        <span className="font-medium text-foreground">Update applied</span> to the
        in-tab store — {state.sizeBefore} → {state.sizeAfter} triples ({sign}
        {delta}). Switch the example to a SELECT and re-run to see the change.
      </div>
    );
  }
  if (state.kind === "graph") {
    if (state.triples === 0) {
      return (
        <p className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
          Empty graph — the template produced no triples.
        </p>
      );
    }
    return (
      <GraphResult
        ntriples={state.ntriples}
        query={state.query}
        triples={state.triples}
      />
    );
  }
  if (state.kind === "explain") {
    return (
      <pre className="max-h-96 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
        {state.plan}
      </pre>
    );
  }
  if (state.kind !== "select") return null;

  return <SelectResult results={state.results} />;
}

// [OPUS-4.8] sq-oy1f.3 / sq-oy1f.7 — the CONSTRUCT / DESCRIBE result-graph view with an
// OUTPUT-FORMAT selector. The default is the existing pretty-Turtle / raw-N-Triples view
// (`TurtleResult`). The JSON-LD modes serialise the SAME result triples through the wasm
// engine's OWN JSON-LD writer (never a TS reshaper): "Expanded" / "Flattened" / "Compacted
// (prefixes)" drive `Store.serialize`'s document forms (#900/#923); "Compaction (@context)"
// drives `Store.serializeCompact` — the full W3C JSON-LD 1.1 Compaction Algorithm against a
// user-supplied `@context` (sq-oy1f.5, #957). Both bindings are in the site's `serialize-rdf`
// bundle. The serialise runs lazily on format switch (off the query path) and is memoised, so
// re-selecting a format never re-serialises; a serialise error surfaces inline (the
// `serializeCompact` binding rejects a non-object `@context` with a clear message).
type GraphFormat =
  | "turtle"
  | JsonLdMode; // "expanded" | "flattened" | "compacted" | "compact"

const GRAPH_FORMAT_TABS: { value: GraphFormat; label: string; title: string }[] = [
  { value: "turtle", label: "Turtle", title: "Pretty Turtle / raw N-Triples (the engine's graph result)" },
  { value: "expanded", label: "JSON-LD (expanded)", title: "W3C JSON-LD 1.1 expanded document form" },
  { value: "flattened", label: "JSON-LD (flattened)", title: "W3C JSON-LD 1.1 flattened document form" },
  { value: "compacted", label: "JSON-LD (prefixes)", title: "JSON-LD with a prefix-only @context (CURIE abbreviation)" },
  { value: "compact", label: "JSON-LD (Compaction)", title: "Full W3C JSON-LD 1.1 Compaction against your @context" },
];

// A sensible starting @context for the full-Compaction mode: the well-known vocabularies the
// built-in datasets use. The user edits it freely; an empty `{}` yields an expanded-shaped
// document with no abbreviation (the wasm binding still runs the algorithm, losslessly).
const DEFAULT_COMPACTION_CONTEXT = `{
  "@vocab": "http://xmlns.com/foaf/0.1/",
  "ex": "http://example.org/",
  "knows": { "@id": "http://xmlns.com/foaf/0.1/knows", "@type": "@id" }
}`;

type JsonLdRender =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "ready"; doc: string }
  | { kind: "error"; message: string };

function GraphResult({
  ntriples,
  query,
  triples,
}: {
  ntriples: string;
  query: string;
  triples: number;
}) {
  const [format, setFormat] = React.useState<GraphFormat>("turtle");
  const [context, setContext] = React.useState(DEFAULT_COMPACTION_CONTEXT);
  const [render, setRender] = React.useState<JsonLdRender>({ kind: "idle" });

  // A fresh result graph (a re-run yields new `ntriples`): snap back to the Turtle view and
  // drop any cached JSON-LD render, so a new result never shows the previous query's output.
  React.useEffect(() => {
    setFormat("turtle");
    setRender({ kind: "idle" });
  }, [ntriples]);

  // Serialise on demand whenever a JSON-LD format is active (or the @context changes for the
  // full-Compaction mode). The serialise is async (it spins up an ephemeral wasm store), so a
  // stale-result guard (`cancelled`) prevents an out-of-order render landing after a re-select.
  React.useEffect(() => {
    if (format === "turtle") return;
    let cancelled = false;
    setRender({ kind: "running" });
    serializeGraphAsJsonLd(ntriples, format, context)
      .then((doc) => {
        if (!cancelled) setRender({ kind: "ready", doc });
      })
      .catch((e) => {
        if (cancelled) return;
        setRender({
          kind: "error",
          message: e instanceof Error ? e.message : String(e),
        });
      });
    return () => {
      cancelled = true;
    };
    // `context` only feeds the "compact" mode, but keying off it for every JSON-LD format is
    // harmless (the other modes ignore it) and keeps the dependency list honest.
  }, [ntriples, format, context]);

  return (
    <div className="space-y-2" data-result-kind="graph">
      <GraphFormatTabs format={format} onChange={setFormat} />

      {/* The full-Compaction mode is driven by a caller-supplied @context; the others are not. */}
      {format === "compact" ? (
        <div className="space-y-1">
          <label
            htmlFor="repl-jsonld-context"
            className="text-xs font-medium text-muted-foreground"
          >
            JSON-LD <code className="font-mono">@context</code> (full W3C 1.1 Compaction)
          </label>
          <textarea
            id="repl-jsonld-context"
            value={context}
            onChange={(e) => setContext(e.target.value)}
            spellCheck={false}
            rows={6}
            aria-label="JSON-LD @context for full Compaction"
            className="w-full resize-y rounded-lg border bg-muted/40 p-2.5 font-mono text-[12.5px] leading-relaxed outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
          />
          <p className="text-[11px] text-muted-foreground">
            The engine applies the full W3C JSON-LD 1.1 Compaction Algorithm against this{" "}
            <code className="font-mono">@context</code> — term definitions,{" "}
            <code className="font-mono">@vocab</code>, type / language /{" "}
            <code className="font-mono">@container</code> coercion,{" "}
            <code className="font-mono">@reverse</code>, and{" "}
            <code className="font-mono">@id</code>/<code className="font-mono">@type</code>{" "}
            aliasing. It must be a JSON object; an empty{" "}
            <code className="font-mono">{"{}"}</code> yields an expanded-shaped document.
          </p>
        </div>
      ) : null}

      {format === "turtle" ? (
        <TurtleResult
          text={ntriples}
          query={query}
          className="max-h-96 overflow-auto rounded-lg border bg-muted/40 p-3 text-[12.5px] leading-relaxed"
        />
      ) : render.kind === "error" ? (
        <pre
          data-graph-view="error"
          className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive"
        >
          {render.message}
        </pre>
      ) : render.kind === "ready" ? (
        <JsonLdHighlight
          text={render.doc}
          data-graph-view="jsonld"
          className="max-h-96 overflow-auto rounded-lg border bg-muted/40 p-3 text-[12.5px] leading-relaxed"
        />
      ) : (
        <p
          data-graph-view="running"
          className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground"
        >
          Serialising {triples} triple{triples === 1 ? "" : "s"} as JSON-LD on the wasm engine…
        </p>
      )}
    </div>
  );
}

// [OPUS-4.8] sq-oy1f.7 — the graph output-format selector. Same `role="tablist"` design-token
// pattern as the Run/EXPLAIN ModeTabs + the SELECT ResultViewTabs, so every selector in the
// REPL reads as one family.
function GraphFormatTabs({
  format,
  onChange,
}: {
  format: GraphFormat;
  onChange: (f: GraphFormat) => void;
}) {
  return (
    <div
      role="tablist"
      aria-label="Graph output format"
      className="inline-flex flex-wrap gap-0.5 rounded-lg border bg-muted/40 p-0.5"
    >
      {GRAPH_FORMAT_TABS.map((t) => (
        <button
          key={t.value}
          type="button"
          role="tab"
          aria-selected={format === t.value}
          title={t.title}
          onClick={() => onChange(t.value)}
          className={cn(
            "rounded-md px-2.5 py-1 text-xs font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40",
            format === t.value
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground",
          )}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}

// [OPUS-4.8] sq-x0kp — the structured SELECT results view (GUI MVP item 2). It carries a
// TABLE ⇄ raw-SPARQL-JSON view toggle and CSV/TSV/JSON exports. The pure shaping — the typed
// table, the CSV/TSV documents, the pretty-printed JSON — comes from the framework-agnostic
// `@sparq/client` helpers, so this component is just the React host that draws them (and the
// Tauri webview draws the SAME cells from the SAME helpers). The view-mode is local to one
// result render; switching the view never re-runs the query.
type ResultView = "table" | "json";

function SelectResult({ results }: { results: SparqlResults }) {
  const [view, setView] = React.useState<ResultView>("table");
  // Re-default to the table whenever a NEW result arrives (a re-run yields a fresh `results`
  // object), so a fresh query always lands on the typed table, not whatever view was last
  // toggled. `results` identity changes per run.
  React.useEffect(() => setView("table"), [results]);

  const table = React.useMemo(() => extractTable(results), [results]);
  const hasRows = table.rows.length > 0;

  return (
    <div className="space-y-2" data-result-kind="select">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <ResultViewTabs view={view} onChange={setView} />
        {/* Exports operate on the WHOLE result (every solution), independent of the view. */}
        <div className="flex items-center gap-1.5">
          <ExportButton
            label="CSV"
            onClick={() =>
              downloadText("sparql-results.csv", resultsToCsv(results), "text/csv")
            }
          />
          <ExportButton
            label="TSV"
            onClick={() =>
              downloadText(
                "sparql-results.tsv",
                resultsToTsv(results),
                "text/tab-separated-values",
              )
            }
          />
          <ExportButton
            label="JSON"
            onClick={() =>
              downloadText(
                "sparql-results.json",
                formatSparqlJson(results),
                "application/sparql-results+json",
              )
            }
          />
        </div>
      </div>

      {view === "json" ? (
        <pre
          data-result-view="json"
          className="max-h-96 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed"
        >
          {formatSparqlJson(results)}
        </pre>
      ) : !hasRows ? (
        <p
          data-result-view="table"
          className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground"
        >
          No solutions.
        </p>
      ) : (
        <div
          data-result-view="table"
          className="max-h-96 overflow-auto rounded-lg border"
        >
          <table className="w-full text-left text-sm">
            <thead className="sticky top-0 bg-muted/80 backdrop-blur">
              <tr>
                {table.vars.map((v) => (
                  <th key={v} className="px-3 py-2 font-medium">
                    ?{v}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {table.rows.map((row, i) => (
                <tr key={i} className="border-t">
                  {row.map((cell, j) => (
                    <td
                      key={table.vars[j]}
                      className="px-3 py-1.5 font-mono text-[12.5px]"
                    >
                      {cell}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// [OPUS-4.8] sq-x0kp — the TABLE ⇄ JSON view toggle for a SELECT result. Same `role="tablist"`
// design-token pattern as the Run/EXPLAIN ModeTabs above, so the two selectors read as one
// family.
function ResultViewTabs({
  view,
  onChange,
}: {
  view: ResultView;
  onChange: (v: ResultView) => void;
}) {
  const tabs: { value: ResultView; label: string; Icon: typeof Table2 }[] = [
    { value: "table", label: "Table", Icon: Table2 },
    { value: "json", label: "JSON", Icon: Braces },
  ];
  return (
    <div
      role="tablist"
      aria-label="Result view"
      className="inline-flex rounded-lg border bg-muted/40 p-0.5"
    >
      {tabs.map((t) => (
        <button
          key={t.value}
          type="button"
          role="tab"
          aria-selected={view === t.value}
          title={
            t.value === "table"
              ? "Show the bindings as a typed table"
              : "Show the raw SPARQL 1.1 JSON results document"
          }
          onClick={() => onChange(t.value)}
          className={cn(
            "inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40",
            view === t.value
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground",
          )}
        >
          <t.Icon className="size-3.5" />
          {t.label}
        </button>
      ))}
    </div>
  );
}

// [OPUS-4.8] sq-x0kp — one export button (CSV / TSV / JSON). Reuses the `outline` Button
// tokens; the download itself is the site-local `downloadText` DOM helper (the bytes come
// from the framework-agnostic exporters).
function ExportButton({
  label,
  onClick,
}: {
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      variant="outline"
      size="sm"
      onClick={onClick}
      title={`Download the result as ${label}`}
    >
      <Download className="size-3.5" />
      {label}
    </Button>
  );
}
