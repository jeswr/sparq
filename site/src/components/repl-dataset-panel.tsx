"use client";

// [OPUS-4.8] sq-daru — GUI MVP item 3: the dataset panel.
//
// Lists the loaded dataset's graphs — the DEFAULT graph plus every NAMED graph — each
// with its triple count, so a user can see at a glance how the data is partitioned. This
// is the live SPARQL REPL's dataset overview; it sits under the dataset-source controls
// and refreshes whenever the store changes (a new dataset is loaded, or an in-tab Update
// mutates it — the parent bumps `refreshKey`).
//
// HONESTY: every number here is the engine's OWN count, read live from the in-tab WASM
// store via two queries it already supports (no new Store API, no baked-in figure):
//   • NAMED_GRAPH_STATS_QUERY  — `GROUP BY ?g` over `GRAPH ?g { ?s ?p ?o }`
//   • DEFAULT_GRAPH_COUNT_QUERY — `SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }`
// The pure `parseGraphStats` helper (in `@/lib/repl-dataset`) turns the two SPARQL-JSON
// documents into the typed rows rendered below, so this component is just the React host.
// On desktop the Tauri webview renders the SAME component against the SAME query results
// (its native engine answers `query` identically — see gui/src-tauri/src/engine.rs).

import * as React from "react";
import { Globe, FileBox, Loader2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { WasmStore, SparqlResults } from "@/lib/sparq-wasm";
import {
  NAMED_GRAPH_STATS_QUERY,
  DEFAULT_GRAPH_COUNT_QUERY,
  parseGraphStats,
  type GraphStat,
} from "@/lib/repl-dataset";

export interface DatasetPanelProps {
  /** The live in-tab WASM store, or `null` while the engine is still cold. */
  store: WasmStore | null;
  /** Bumped by the parent whenever the dataset changes (load / merge / Update), so the
   *  panel re-reads the per-graph counts. */
  refreshKey: number;
  /** Hide the panel entirely (e.g. endpoint mode owns its own dataset remotely). */
  hidden?: boolean;
}

type PanelState =
  | { kind: "loading" }
  | { kind: "ready"; stats: GraphStat[] }
  | { kind: "error"; message: string };

/**
 * The dataset panel: a labelled list of the loaded dataset's graphs with per-graph
 * triple counts. The default graph is always shown (even when empty); named graphs
 * follow in IRI order. Re-queries whenever the store or `refreshKey` changes.
 */
export function DatasetPanel({ store, refreshKey, hidden }: DatasetPanelProps) {
  const [state, setState] = React.useState<PanelState>({ kind: "loading" });

  React.useEffect(() => {
    if (!store) {
      setState({ kind: "loading" });
      return;
    }
    try {
      const namedJson = JSON.parse(
        store.query(NAMED_GRAPH_STATS_QUERY),
      ) as SparqlResults;
      const defaultJson = JSON.parse(
        store.query(DEFAULT_GRAPH_COUNT_QUERY),
      ) as SparqlResults;
      setState({ kind: "ready", stats: parseGraphStats(defaultJson, namedJson) });
    } catch (e) {
      setState({ kind: "error", message: e instanceof Error ? e.message : String(e) });
    }
  }, [store, refreshKey]);

  if (hidden) return null;

  const namedCount =
    state.kind === "ready" ? state.stats.filter((s) => !s.isDefault).length : 0;

  return (
    // [OPUS-4.8] sq-vw3ax — the panel chrome (header label + border) is now owned by the
    // enclosing rail ReplPanel; this renders just the per-graph meta + rows. The
    // `data-testid` anchor is preserved.
    <section aria-label="Dataset graphs" data-testid="dataset-panel" className="space-y-1.5">
      {state.kind === "ready" && (
        <p className="text-[11px] text-muted-foreground">
          {/* The default graph is always present; show how many NAMED graphs alongside. */}
          {namedCount === 0
            ? "Default graph only"
            : `${namedCount} named ${namedCount === 1 ? "graph" : "graphs"} + default`}
        </p>
      )}

      {state.kind === "loading" ? (
        <p className="flex items-center gap-1.5 px-1 py-1 text-xs text-muted-foreground">
          <Loader2 className="size-3.5 animate-spin" aria-hidden="true" /> Reading graphs…
        </p>
      ) : state.kind === "error" ? (
        <p className="px-1 py-1 text-xs text-destructive">{state.message}</p>
      ) : (
        <ul className="space-y-1">
          {state.stats.map((g) => (
            <GraphRow key={g.isDefault ? "__default__" : (g.iri as string)} stat={g} />
          ))}
        </ul>
      )}
    </section>
  );
}

/** One graph row: an icon + label (the default graph labelled clearly, or the named
 *  graph's IRI) and a triple-count badge. The IRI truncates with a full-text title so a
 *  long IRI never breaks the layout but stays inspectable on hover. */
function GraphRow({ stat }: { stat: GraphStat }) {
  const label = stat.isDefault ? "Default graph" : stat.iri;
  const Icon = stat.isDefault ? Globe : FileBox;
  return (
    <li className="flex items-center gap-2 rounded-md bg-background/60 px-2 py-1">
      <Icon
        className={cn(
          "size-3.5 shrink-0",
          stat.isDefault ? "text-primary" : "text-muted-foreground",
        )}
        aria-hidden="true"
      />
      <span
        className={cn(
          "min-w-0 flex-1 truncate text-xs",
          stat.isDefault ? "font-medium" : "font-mono text-[11.5px]",
        )}
        title={stat.isDefault ? "The default (unnamed) graph" : (stat.iri as string)}
      >
        {label}
      </span>
      <Badge variant="muted" className="tabular shrink-0 text-[11px]">
        {stat.count} {stat.count === 1 ? "triple" : "triples"}
      </Badge>
    </li>
  );
}
