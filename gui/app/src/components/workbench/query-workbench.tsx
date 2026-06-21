"use client";

// [OPUS-4.8] sq-ixc3.9 — the WORKING Query tool: a full-height editor over the live store + a
// co-resident results panel (Table | Raw JSON | N-Triples). This is the foundation slice of the
// query workbench (sq-ixc3.12 adds the Graph view + streaming + the richer multi-view layout);
// it runs SELECT/ASK/CONSTRUCT/DESCRIBE/UPDATE live in-tab and measures each run's latency.
//
// E2E CONTRACT (gui/e2e/run-e2e.mjs asserts these stable hooks, kept faithful so the harness
// cannot silently drift): the editing <textarea id="repl-query">, a "Run query" button, the
// "Engine ready" status copy (in the top bar), and the result container's
// data-result-kind="select" + data-result-view="table" wrapping a <table> with the binding.

import * as React from "react";
import { Play, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useEngine, type QueryOutcome } from "@/lib/engine-context";
import { extractTable, type SparqlResults } from "@sparq/client";
import { DEFAULT_QUERY } from "@/data/sample-graph";

type ResultView = "table" | "json" | "ntriples";

function SelectTable({ results }: { results: SparqlResults }) {
  const table = extractTable(results);
  if (table.vars.length === 0) {
    return <p className="p-3 text-sm text-muted-foreground">No projected variables.</p>;
  }
  return (
    <div className="overflow-auto" data-result-view="table">
      <table className="w-full border-collapse text-sm">
        <thead className="sticky top-0 bg-card">
          <tr>
            {table.vars.map((v) => (
              <th
                key={v}
                className="border-b px-3 py-1.5 text-left font-mono text-xs font-semibold"
              >
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.length === 0 ? (
            <tr>
              <td
                colSpan={table.vars.length}
                className="px-3 py-3 text-center text-muted-foreground"
              >
                0 rows
              </td>
            </tr>
          ) : (
            table.rows.map((row, i) => (
              <tr key={i} className="odd:bg-muted/30">
                {row.map((cell, j) => (
                  <td key={j} className="border-b px-3 py-1 font-mono text-xs">
                    {cell}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

function ResultBody({
  outcome,
  view,
}: {
  outcome: QueryOutcome;
  view: ResultView;
}) {
  if (outcome.kind === "error") {
    return (
      <pre
        className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs text-destructive"
        data-result-kind="error"
      >
        {outcome.message}
      </pre>
    );
  }
  if (outcome.kind === "ask") {
    return (
      <div className="p-3 text-sm" data-result-kind="ask">
        {view === "json" ? (
          <pre className="overflow-auto font-mono text-xs">{outcome.rawJson}</pre>
        ) : (
          <span className="font-mono">
            ASK → <strong>{String(outcome.value)}</strong>
          </span>
        )}
      </div>
    );
  }
  if (outcome.kind === "graph") {
    return (
      <pre
        className="overflow-auto whitespace-pre p-3 font-mono text-xs"
        data-result-kind="graph"
      >
        {outcome.ntriples || "(empty graph)"}
      </pre>
    );
  }
  if (outcome.kind === "update") {
    return (
      <div className="p-3 text-sm" data-result-kind="update">
        Update applied. Store now holds{" "}
        <span className="tabular font-medium">{outcome.sizeAfter.toLocaleString()}</span> quads.
      </div>
    );
  }
  // select
  return (
    <div className="h-full" data-result-kind="select">
      {view === "table" ? (
        <SelectTable results={outcome.results} />
      ) : view === "ntriples" ? (
        <p className="p-3 text-sm text-muted-foreground">
          N-Triples view applies to CONSTRUCT/DESCRIBE results.
        </p>
      ) : (
        <pre className="overflow-auto p-3 font-mono text-xs" data-result-view="json">
          {outcome.rawJson}
        </pre>
      )}
    </div>
  );
}

const VIEWS: { id: ResultView; label: string }[] = [
  { id: "table", label: "Table" },
  { id: "json", label: "Raw JSON" },
  { id: "ntriples", label: "N-Triples" },
];

export function QueryWorkbench() {
  const { run, status } = useEngine();
  const [query, setQuery] = React.useState(DEFAULT_QUERY);
  const [outcome, setOutcome] = React.useState<QueryOutcome | null>(null);
  const [view, setView] = React.useState<ResultView>("table");
  const [running, setRunning] = React.useState(false);

  const onRun = React.useCallback(async () => {
    setRunning(true);
    try {
      const result = await run(query);
      setOutcome(result.outcome);
      // A CONSTRUCT/DESCRIBE answer is graph-shaped → default its view to N-Triples.
      if (result.outcome.kind === "graph") setView("ntriples");
      else if (result.outcome.kind === "select" && view === "ntriples") setView("table");
    } finally {
      setRunning(false);
    }
  }, [run, query, view]);

  // ⌘/Ctrl-Enter runs the query (the keyboard-first spine; the full Cmd-K palette is sq-ixc3.10).
  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void onRun();
    }
  };

  const ready = status.kind === "ready";

  return (
    <div className="flex h-full flex-col">
      {/* Editor pane. */}
      <div className="flex min-h-0 flex-[3] flex-col border-b">
        <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
          <span className="text-xs font-medium text-muted-foreground">SPARQL</span>
          <Button
            size="sm"
            onClick={() => void onRun()}
            disabled={!ready || running}
            className="ml-auto"
          >
            {running ? <Loader2 className="animate-spin" /> : <Play />}
            Run query
          </Button>
          <span className="text-[11px] text-muted-foreground">⌘↵</span>
        </div>
        <textarea
          id="repl-query"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={onKeyDown}
          spellCheck={false}
          className="min-h-0 flex-1 resize-none bg-background p-3 font-mono text-sm outline-none"
          placeholder="Write a SPARQL query…"
          aria-label="SPARQL query editor"
        />
      </div>

      {/* Results pane. */}
      <div className="flex min-h-0 flex-[2] flex-col">
        <div className="flex items-center gap-1 border-b bg-card px-3 py-1">
          {VIEWS.map((v) => (
            <button
              key={v.id}
              onClick={() => setView(v.id)}
              className={cn(
                "rounded px-2 py-0.5 text-xs",
                view === v.id
                  ? "bg-primary/10 font-medium text-primary"
                  : "text-muted-foreground hover:bg-accent/40",
              )}
            >
              {v.label}
            </button>
          ))}
        </div>
        <div className="min-h-0 flex-1 overflow-auto">
          {outcome === null ? (
            <p className="p-3 text-sm text-muted-foreground">
              {ready
                ? "Run a query to see results."
                : "Waiting for the engine to warm…"}
            </p>
          ) : (
            <ResultBody outcome={outcome} view={view} />
          )}
        </div>
      </div>
    </div>
  );
}
