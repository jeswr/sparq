"use client";

import * as React from "react";
import { Play, Loader2, Database, Zap } from "lucide-react";
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
  formatTerm,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
import { SAMPLE_TURTLE, EXAMPLE_QUERIES } from "@/data/sample-graph";

type RunState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "running" }
  | { kind: "select"; results: SparqlResults; ms: number }
  | { kind: "boolean"; value: boolean; ms: number }
  | { kind: "error"; message: string };

export function Repl() {
  const [sparql, setSparql] = React.useState(EXAMPLE_QUERIES[0].sparql);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });
  const [size, setSize] = React.useState<number | null>(null);
  const storeRef = React.useRef<WasmStore | null>(null);

  const ensureStore = React.useCallback(async (): Promise<WasmStore> => {
    if (storeRef.current) return storeRef.current;
    setState({ kind: "loading" });
    const Store = await loadSparq();
    const store = Store.load(SAMPLE_TURTLE, "turtle");
    storeRef.current = store;
    setSize(store.size);
    return store;
  }, []);

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

  const busy = state.kind === "loading" || state.kind === "running";

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <Zap className="size-4 text-primary" />
          Live SPARQL REPL
        </CardTitle>
        <div className="flex items-center gap-2">
          <Badge variant="success">Live in your tab</Badge>
          {size !== null && (
            <Badge variant="muted" className="tabular">
              <Database className="size-3" /> {size} triples
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
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
            {state.kind === "loading" ? "Loading engine…" : "Run query"}
          </Button>
          <p
            aria-live="polite"
            className="text-xs text-muted-foreground"
          >
            {state.kind === "select" &&
              `${state.results.results.bindings.length} rows · ${state.ms.toFixed(1)} ms`}
            {state.kind === "boolean" && `${state.ms.toFixed(1)} ms`}
            {state.kind === "loading" && "Fetching the wasm bundle…"}
            {state.kind === "running" && "Running on the wasm engine…"}
          </p>
        </div>

        <ResultPanel state={state} />
      </CardContent>
    </Card>
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
