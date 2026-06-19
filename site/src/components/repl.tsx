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
  prewarmSparq,
  storeToNQuads,
  datasetSize,
  extractTable,
  resultsToCsv,
  resultsToTsv,
  formatSparqlJson,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
import { downloadText } from "@/lib/download";
import {
  classifyQueryForm,
  isGraphForm,
  modeSupportsForm,
  type QueryForm,
  type RunMode,
} from "@/lib/repl-dataset";
import { EXAMPLE_QUERIES, BUILTIN_DATASETS } from "@/data/sample-graph";
import {
  DatasetControls,
  DatasetViewer,
  type ActiveDataset,
} from "@/components/repl-datasets";
// [OPUS-4.8] sq-n5aw — the syntax-highlighting SPARQL editor replaces the plain <textarea>.
import { SparqlEditor } from "@/components/sparql-editor";

// [OPUS-4.8] sq-vfbm — the REPL now dispatches across the whole lean-bundle query
// surface, so the result state carries one variant per shape of answer: a solution
// table (SELECT), a boolean (ASK), a constructed N-Triples graph (CONSTRUCT/DESCRIBE),
// an Update acknowledgement (mutated the in-tab store), and the EXPLAIN plan text.
type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "select"; results: SparqlResults; ms: number }
  | { kind: "boolean"; value: boolean; ms: number }
  | { kind: "graph"; ntriples: string; triples: number; ms: number }
  | { kind: "update"; sizeBefore: number; sizeAfter: number; ms: number }
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

  // [OPUS-4.8] sq-vfbm — dispatch across the whole lean-bundle query surface. EXPLAIN /
  // ANALYZE modes render the planner's plan text (every form for EXPLAIN; SELECT/ASK for
  // ANALYZE). Otherwise we classify the SPARQL form and route to the matching wasm export:
  // SELECT/ASK -> query (JSON), CONSTRUCT/DESCRIBE -> queryQuads (N-Triples), and an Update
  // keyword -> updateInPlace (mutates the in-tab store; we re-count the dataset after).
  const run = React.useCallback(async () => {
    try {
      const store = await ensureStore();
      const form = classifyQueryForm(sparql);
      setState({ kind: "running" });

      if (mode === "explain" || mode === "analyze") {
        const analyze = mode === "analyze";
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
        setState({ kind: "update", sizeBefore, sizeAfter, ms });
        return;
      }

      if (isGraphForm(form)) {
        const t0 = performance.now();
        const ntriples = store.queryQuads(sparql);
        const ms = performance.now() - t0;
        const trimmed = ntriples.trim();
        const triples = trimmed === "" ? 0 : trimmed.split("\n").length;
        setState({ kind: "graph", ntriples: trimmed, triples, ms });
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
      toast.error(mode === "run" ? "Query failed" : "EXPLAIN failed", {
        description: message,
      });
    }
  }, [ensureStore, sparql, mode]);

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
        <SparqlEditor
          id="repl-query"
          ariaLabel="SPARQL query"
          value={sparql}
          onChange={setSparql}
          rows={9}
        />

        <div className="flex flex-wrap items-center gap-3">
          <Button onClick={run} disabled={busy}>
            {busy ? (
              <Loader2 className="size-4 animate-spin" />
            ) : (
              <Play className="size-4" />
            )}
            {mode === "run" ? "Run query" : "Run EXPLAIN"}
          </Button>
          <ModeTabs mode={mode} onChange={setMode} disabled={busy} form={form} />
          <p aria-live="polite" className="text-xs text-muted-foreground">
            {state.kind === "select" &&
              `${state.results.results.bindings.length} rows · ${state.ms.toFixed(1)} ms`}
            {state.kind === "boolean" && `${state.ms.toFixed(1)} ms`}
            {state.kind === "graph" &&
              `${state.triples} triples · ${state.ms.toFixed(1)} ms`}
            {state.kind === "update" &&
              `store ${state.sizeBefore} → ${state.sizeAfter} triples · ${state.ms.toFixed(1)} ms`}
            {state.kind === "explain" &&
              `plan ${state.analyze ? "+ trace " : ""}· ${state.ms.toFixed(1)} ms`}
            {state.kind === "running" &&
              (mode === "run"
                ? "Running on the wasm engine…"
                : "Planning on the wasm engine…")}
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
}: {
  mode: RunMode;
  onChange: (m: RunMode) => void;
  disabled: boolean;
  form: QueryForm;
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
        // mode that would parse-error.
        const unsupported = !modeSupportsForm(t.value, form);
        const tabDisabled = disabled || unsupported;
        return (
          <button
            key={t.value}
            type="button"
            role="tab"
            aria-selected={mode === t.value}
            aria-disabled={unsupported}
            title={
              unsupported
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
      <pre className="max-h-96 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
        {state.ntriples}
      </pre>
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
