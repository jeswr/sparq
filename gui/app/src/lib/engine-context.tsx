"use client";

// [OPUS-4.8] sq-ixc3.9 — the operational engine context: the ONE live wasm store the whole
// workbench shares, plus warm status, the measured-latency query path, and the dataset
// summary the left-rail datasets tree renders.
//
// HONESTY: no performance number is baked in. The bottom status bar shows the latency of the
// query the user JUST ran, measured with `performance.now()` and labelled as such — never a
// benchmark claim. On the desktop Tauri target the design's end state is the DIRECT native
// engine link (gui/src-tauri/src/engine.rs); this foundation shell runs the same in-tab WASM
// engine in both targets (the honest, working-today path) and the IPC swap is a later phase.

import * as React from "react";
import {
  loadSparq,
  prewarmSparq,
  formatSparqlJson,
  extractTable,
  isAskResult,
  askValue,
  type SparqlResults,
  type WasmStore,
  type WasmStoreCtor,
} from "@sparq/client";

import { basePath } from "@/lib/base-path";
import { SAMPLE_TURTLE, SAMPLE_FORMAT } from "@/data/sample-graph";

/** The engine warm lifecycle. `error` carries a load/parse failure message. */
export type EngineStatus =
  | { kind: "cold" }
  | { kind: "warming" }
  | { kind: "ready" }
  | { kind: "error"; message: string };

/** A per-named-graph row for the datasets tree (default graph + named graphs). */
export interface GraphSummary {
  /** The graph IRI, or null for the default graph. */
  graph: string | null;
  /** Triple/quad count in this graph. */
  count: number;
}

/** What a run produced — discriminated by SPARQL form so the results panel can branch. */
export type QueryOutcome =
  | { kind: "select"; results: SparqlResults; rawJson: string; rowCount: number }
  | { kind: "ask"; value: boolean; rawJson: string }
  | { kind: "graph"; ntriples: string; tripleCount: number }
  | { kind: "update"; sizeAfter: number }
  | { kind: "error"; message: string };

/** A completed run + its MEASURED latency (performance.now delta, ms). */
export interface RunResult {
  outcome: QueryOutcome;
  /** Wall-clock latency of THIS run, measured with performance.now() (ms). Labelled, not a benchmark. */
  latencyMs: number;
}

export interface EngineContextValue {
  status: EngineStatus;
  /** Total quads in the live store (default + all named graphs). */
  storeSize: number;
  /** Per-graph counts for the datasets tree. */
  graphs: GraphSummary[];
  /** The latency (ms) of the most recent run, or null before any run. */
  lastLatencyMs: number | null;
  /** The row count of the most recent SELECT run, or null. */
  lastRowCount: number | null;
  /** Run a query/update against the live store; resolves with the outcome + measured latency. */
  run: (query: string) => Promise<RunResult>;
  /**
   * [OPUS-4.8] sq-ixc3.10 — render the planner's EXPLAIN (or EXPLAIN ANALYZE) plan text for a
   * query WITHOUT executing it (ANALYZE also executes). This is an in-tab planner introspection
   * the Cmd-K spine drives ("Run EXPLAIN"); it returns the plan text or throws on a parse error
   * (e.g. an Update form, which the planner rejects). Separate from `run` because it produces
   * plan text, not a result set.
   */
  explain: (query: string, analyze?: boolean) => string;
  /**
   * [OPUS-4.8] sq-ixc3.11 — serialise the LIVE store to TriG so an operational tool (e.g. the
   * SHACL validator) can run over the actual imported store rather than a fixture. TriG (not
   * N-Triples — the serialise binding does not emit N-Triples) preserves every named graph as
   * well as the default graph. Returns `null` before the engine is ready or if the loaded
   * bundle lacks the serialise binding. The serialise-rdf binding is in the GUI's wasm bundle.
   */
  serializeStore: () => string | null;
}

const EngineContext = React.createContext<EngineContextValue | null>(null);

/** Heuristic SPARQL form classifier (the WASM Store has separate verbs per form). */
function classifyQuery(q: string): "select" | "ask" | "construct" | "describe" | "update" {
  // Strip comments + leading PREFIX/BASE declarations to find the first significant keyword.
  const body = q
    .replace(/(^|\s)#[^\n]*/g, " ")
    .replace(/\b(PREFIX\s+\S+\s+<[^>]*>|BASE\s+<[^>]*>)/gi, " ")
    .trim();
  const m = body.match(/\b(SELECT|ASK|CONSTRUCT|DESCRIBE|INSERT|DELETE|LOAD|CLEAR|CREATE|DROP|COPY|MOVE|ADD)\b/i);
  const kw = m ? m[1].toUpperCase() : "SELECT";
  if (kw === "ASK") return "ask";
  if (kw === "CONSTRUCT") return "construct";
  if (kw === "DESCRIBE") return "describe";
  if (["INSERT", "DELETE", "LOAD", "CLEAR", "CREATE", "DROP", "COPY", "MOVE", "ADD"].includes(kw))
    return "update";
  return "select";
}

/** Summarise the live store into per-graph counts via a single grouped query. */
function summariseGraphs(store: WasmStore): { size: number; graphs: GraphSummary[] } {
  const graphs: GraphSummary[] = [];
  let size = 0;
  // Default graph.
  try {
    const def = store.count("SELECT * WHERE { ?s ?p ?o }");
    if (def > 0) {
      graphs.push({ graph: null, count: def });
      size += def;
    }
  } catch {
    /* count over an empty store can be zero; ignore */
  }
  // Named graphs (group by graph).
  try {
    const json = store.query(
      "SELECT ?g (COUNT(*) AS ?c) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g ORDER BY ?g",
    );
    const parsed = JSON.parse(json) as SparqlResults;
    for (const b of parsed.results?.bindings ?? []) {
      const g = b["g"]?.value ?? null;
      const c = Number.parseInt(b["c"]?.value ?? "0", 10) || 0;
      if (g) {
        graphs.push({ graph: g, count: c });
        size += c;
      }
    }
  } catch {
    /* a store with no named graphs yields no rows; ignore */
  }
  return { size, graphs };
}

export function EngineProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = React.useState<EngineStatus>({ kind: "cold" });
  const [storeSize, setStoreSize] = React.useState(0);
  const [graphs, setGraphs] = React.useState<GraphSummary[]>([]);
  const [lastLatencyMs, setLastLatencyMs] = React.useState<number | null>(null);
  const [lastRowCount, setLastRowCount] = React.useState<number | null>(null);

  const storeRef = React.useRef<WasmStore | null>(null);
  const ctorRef = React.useRef<WasmStoreCtor | null>(null);

  // Warm the engine once on mount: load wasm, seed the sample graph, compute the summary.
  React.useEffect(() => {
    let cancelled = false;
    const opts = { basePath: basePath() };
    setStatus({ kind: "warming" });
    prewarmSparq(opts)
      .then(() => loadSparq(opts))
      .then((Store) => {
        if (cancelled) return;
        ctorRef.current = Store;
        const store = Store.load(SAMPLE_TURTLE, SAMPLE_FORMAT);
        storeRef.current = store;
        const { size, graphs: gs } = summariseGraphs(store);
        setStoreSize(size);
        setGraphs(gs);
        setStatus({ kind: "ready" });
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setStatus({
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const refreshSummary = React.useCallback(() => {
    const store = storeRef.current;
    if (!store) return;
    const { size, graphs: gs } = summariseGraphs(store);
    setStoreSize(size);
    setGraphs(gs);
  }, []);

  const run = React.useCallback(
    async (query: string): Promise<RunResult> => {
      const store = storeRef.current;
      if (!store) {
        const outcome: QueryOutcome = {
          kind: "error",
          message: "The engine is not ready yet — wait for the store to warm.",
        };
        return { outcome, latencyMs: 0 };
      }
      const form = classifyQuery(query);
      const t0 = performance.now();
      let outcome: QueryOutcome;
      try {
        if (form === "ask") {
          const json = store.query(query);
          const parsed = JSON.parse(json) as SparqlResults;
          outcome = isAskResult(parsed)
            ? { kind: "ask", value: askValue(parsed) ?? false, rawJson: formatSparqlJson(parsed) }
            : { kind: "error", message: "ASK query did not return a boolean result." };
        } else if (form === "construct" || form === "describe") {
          const ntriples = store.queryQuads(query);
          const tripleCount = ntriples
            .split("\n")
            .filter((l) => l.trim().length > 0).length;
          outcome = { kind: "graph", ntriples, tripleCount };
        } else if (form === "update") {
          store.updateInPlace(query);
          // `size` reports the default graph only; recompute the full per-graph total below.
          const { size } = summariseGraphs(store);
          outcome = { kind: "update", sizeAfter: size };
        } else {
          const json = store.query(query);
          const parsed = JSON.parse(json) as SparqlResults;
          const table = extractTable(parsed);
          outcome = {
            kind: "select",
            results: parsed,
            rawJson: formatSparqlJson(parsed),
            rowCount: table.rows.length,
          };
        }
      } catch (err) {
        outcome = {
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        };
      }
      const latencyMs = performance.now() - t0;
      setLastLatencyMs(latencyMs);
      setLastRowCount(
        outcome.kind === "select"
          ? outcome.rowCount
          : outcome.kind === "graph"
            ? outcome.tripleCount
            : null,
      );
      // An UPDATE mutated the store; refresh the datasets tree.
      if (outcome.kind === "update") refreshSummary();
      return { outcome, latencyMs };
    },
    [refreshSummary],
  );

  // [OPUS-4.8] sq-ixc3.10 — the EXPLAIN / ANALYZE planner introspection the Cmd-K spine drives.
  // Throws if the store is cold or the planner rejects the form (e.g. an Update), so the caller
  // surfaces a clear message rather than silently no-op'ing.
  const explain = React.useCallback((query: string, analyze = false): string => {
    const store = storeRef.current;
    if (!store) throw new Error("The engine is not ready yet — wait for the store to warm.");
    return analyze ? store.explainAnalyze(query) : store.explain(query);
  }, []);

  // [OPUS-4.8] sq-ixc3.11 — TriG dump of the live store, the input an operational tool (SHACL
  // validate-the-active-store) consumes. TriG (the serialise binding does NOT accept
  // "ntriples" — only turtle/trig/jsonld) preserves the default graph AND every named graph,
  // and is unabbreviated (`abbreviate=false`) so no caller-supplied prefix map can disagree.
  // `null` until the store warms or if a lean bundle lacks the binding.
  const serializeStore = React.useCallback((): string | null => {
    const store = storeRef.current;
    if (!store) return null;
    // The GUI bundle is built with `serialize-rdf`, but the runtime-loaded bundle decides
    // whether the binding exists; keep a defensive view so a lean bundle yields a clear empty
    // result rather than a `serialize is not a function` crash.
    const serialize = (store as { serialize?: WasmStore["serialize"] }).serialize;
    if (typeof serialize !== "function") return null;
    return store.serialize("trig", false, null, false, null);
  }, []);

  const value = React.useMemo<EngineContextValue>(
    () => ({ status, storeSize, graphs, lastLatencyMs, lastRowCount, run, explain, serializeStore }),
    [status, storeSize, graphs, lastLatencyMs, lastRowCount, run, explain, serializeStore],
  );

  return <EngineContext.Provider value={value}>{children}</EngineContext.Provider>;
}

export function useEngine(): EngineContextValue {
  const ctx = React.useContext(EngineContext);
  if (!ctx) throw new Error("useEngine must be used within an <EngineProvider>");
  return ctx;
}
