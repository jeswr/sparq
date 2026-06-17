"use client";

import * as React from "react";
import { Play, Loader2, Database, Zap, CheckCircle2 } from "lucide-react";
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
  prewarmSparq,
  storeToNQuads,
  datasetSize,
  formatTerm,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
import { EXAMPLE_QUERIES, BUILTIN_DATASETS } from "@/data/sample-graph";
import {
  DatasetControls,
  DatasetViewer,
  type ActiveDataset,
} from "@/components/repl-datasets";

type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "select"; results: SparqlResults; ms: number }
  | { kind: "boolean"; value: boolean; ms: number }
  | { kind: "error"; message: string };

// [OPUS-4.8] Engine warm-up lifecycle, surfaced as a subtle indicator. The wasm fetch +
// instantiate is kicked off on mount (prewarmSparq) so the first "Run query" is instant.
type EngineState = "cold" | "warming" | "ready" | "error";

const DEFAULT_DATASET = BUILTIN_DATASETS[0];

export function Repl() {
  const [sparql, setSparql] = React.useState(EXAMPLE_QUERIES[0].sparql);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });
  const [size, setSize] = React.useState<number | null>(null);
  const [engine, setEngine] = React.useState<EngineState>("cold");
  const [viewerOpen, setViewerOpen] = React.useState(false);
  const [active, setActive] = React.useState<ActiveDataset>({
    label: DEFAULT_DATASET.label,
    description: DEFAULT_DATASET.description,
  });
  const [activeBuiltinId, setActiveBuiltinId] = React.useState<string | null>(
    DEFAULT_DATASET.id,
  );
  const storeRef = React.useRef<WasmStore | null>(null);

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
      return store;
    },
    [],
  );

  // Pre-warm the engine AND parse the default dataset eagerly on mount, off the render
  // path. The first "Run query" then runs against an already-built store with no cold
  // start. A failure resets the indicator so a later run can retry via ensureStore.
  React.useEffect(() => {
    let cancelled = false;
    setEngine("warming");
    prewarmSparq()
      .then(async () => {
        if (cancelled || storeRef.current) return;
        await buildStore(DEFAULT_DATASET.text, DEFAULT_DATASET.format);
      })
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
    return () => {
      cancelled = true;
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

  const run = React.useCallback(async () => {
    try {
      const store = await ensureStore();
      setState({ kind: "running" });
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
      toast.error("Query failed", { description: message });
    }
  }, [ensureStore, sparql]);

  // Switch to a built-in dataset: reload the store, reset the count + active descriptor.
  const selectBuiltin = React.useCallback(
    async (id: string) => {
      const ds = BUILTIN_DATASETS.find((d) => d.id === id);
      if (!ds) return;
      try {
        await buildStore(ds.text, ds.format);
        setActiveBuiltinId(ds.id);
        setActive({ label: ds.label, description: ds.description });
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
          setActive({
            label,
            description: `Custom ${format} dataset loaded in your tab.`,
          });
        }
        setActiveBuiltinId(null);
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

  const busy = state.kind === "running";
  const controlsDisabled = engine === "warming" || engine === "cold";

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <Zap className="size-4 text-primary" />
          Live SPARQL REPL
        </CardTitle>
        <div className="flex items-center gap-2">
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
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <DatasetControls
          activeBuiltinId={activeBuiltinId}
          onSelectBuiltin={selectBuiltin}
          onLoadText={loadText}
          disabled={controlsDisabled}
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
        <textarea
          id="repl-query"
          value={sparql}
          spellCheck={false}
          onChange={(e) => setSparql(e.target.value)}
          rows={9}
          className="w-full resize-y rounded-lg border bg-muted/40 p-3 font-mono text-[13px] leading-relaxed outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
        />

        <div className="flex items-center gap-3">
          <Button onClick={run} disabled={busy}>
            {busy ? (
              <Loader2 className="size-4 animate-spin" />
            ) : (
              <Play className="size-4" />
            )}
            Run query
          </Button>
          <p aria-live="polite" className="text-xs text-muted-foreground">
            {state.kind === "select" &&
              `${state.results.results.bindings.length} rows · ${state.ms.toFixed(1)} ms`}
            {state.kind === "boolean" && `${state.ms.toFixed(1)} ms`}
            {state.kind === "running" && "Running on the wasm engine…"}
            {state.kind === "idle" &&
              engine === "warming" &&
              "Pre-warming the wasm engine…"}
          </p>
        </div>

        <ResultPanel state={state} />
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
  if (state.kind !== "select") return null;

  const { vars } = state.results.head;
  const rows = state.results.results.bindings;
  if (rows.length === 0) {
    return (
      <p className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
        No solutions.
      </p>
    );
  }
  return (
    <div className="overflow-x-auto rounded-lg border">
      <table className="w-full text-left text-sm">
        <thead className="bg-muted/50">
          <tr>
            {vars.map((v) => (
              <th key={v} className="px-3 py-2 font-medium">
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => (
            <tr key={i} className="border-t">
              {vars.map((v) => (
                <td key={v} className="px-3 py-1.5 font-mono text-[12.5px]">
                  {formatTerm(row[v])}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
