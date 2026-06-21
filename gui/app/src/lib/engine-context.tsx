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
  isAskResult,
  askValue,
  streamQueryRows,
  type SparqlBinding,
  type SparqlResults,
  type WasmStore,
  type WasmStoreCtor,
} from "@sparq/client";

import { basePath } from "@/lib/base-path";
import { SAMPLE_TURTLE, SAMPLE_FORMAT } from "@/data/sample-graph";

/**
 * Internal sentinel thrown out of the streaming loop when the caller's {@link AbortSignal} fires,
 * so a Stop is distinguishable from a real engine error in the `catch` below. Named so the
 * instanceof check is robust to minification.
 */
class AbortError extends Error {
  constructor() {
    super("aborted");
    this.name = "AbortError";
  }
}

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

/**
 * The SELECT result the workbench renders. A streamed SELECT keeps at most {@link RunOptions.rowCap}
 * rows in JS (so a large result cannot blow the tab's memory); `truncated` records whether the
 * full result exceeded that cap. `rawJson` is a SPARQL-1.1-JSON document over the KEPT rows.
 */
export interface SelectOutcome {
  kind: "select";
  /** The SPARQL-JSON results doc over the kept rows (for the Table / Raw JSON views + export). */
  results: SparqlResults;
  rawJson: string;
  /** Rows kept in JS (≤ rowCap). */
  rowCount: number;
  /** Total rows the engine produced (may exceed `rowCount` when streamed + capped). */
  totalRows: number;
  /** True when the engine produced more rows than were kept in JS. */
  truncated: boolean;
}

/** What a run produced — discriminated by SPARQL form so the results panel can branch. */
export type QueryOutcome =
  | SelectOutcome
  | { kind: "ask"; value: boolean; rawJson: string }
  | { kind: "graph"; ntriples: string; tripleCount: number }
  | { kind: "update"; sizeAfter: number }
  | { kind: "explain"; mode: "explain" | "analyze"; plan: string }
  | { kind: "cancelled" }
  | { kind: "error"; message: string };

/** A completed run + its MEASURED latency (performance.now delta, ms). */
export interface RunResult {
  outcome: QueryOutcome;
  /** Wall-clock latency of THIS run, measured with performance.now() (ms). Labelled, not a benchmark. */
  latencyMs: number;
}

/** How to run a query — plain execution, EXPLAIN (plan only), or EXPLAIN ANALYZE (plan + run). */
export type RunMode = "run" | "explain" | "analyze";

/** Optional per-run controls. */
export interface RunOptions {
  /** plain run / EXPLAIN / EXPLAIN ANALYZE. Defaults to `"run"`. */
  mode?: RunMode;
  /**
   * Max SELECT rows to keep in JS when streaming (the cap that bounds peak memory). Rows beyond
   * this are counted but dropped; the outcome is marked `truncated`. Defaults to {@link DEFAULT_ROW_CAP}.
   */
  rowCap?: number;
  /** A cooperative cancel signal — checked between streamed batches (the Stop button). */
  signal?: AbortSignal;
}

/**
 * The default cap on SELECT rows kept in JS for the table/JSON views (streaming bounds peak
 * memory at one batch + this many displayed rows; exports re-stream the WHOLE result). This is a
 * UI display bound, not a result bound — it is labelled in the results panel, not a benchmark.
 */
export const DEFAULT_ROW_CAP = 5_000;

/** The batch size {@link streamQueryRows} pulls per cursor step (one batch held at a time). */
const STREAM_BATCH_SIZE = 1_000;

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
  /**
   * Run a query/update against the live store; resolves with the outcome + measured latency.
   * [OPUS-4.8] sq-ixc3.10/.12 — this is the SINGLE EXPLAIN entry point: pass
   * {@link RunOptions.mode} `"explain"` / `"analyze"` to render the planner's plan (the canonical
   * EXPLAIN path the Cmd-K spine AND the workbench EXPLAIN/ANALYZE buttons both drive — there is
   * no separate `explain()` method). Pass {@link RunOptions.signal} to make the run cancellable
   * (Stop), and {@link RunOptions.rowCap} to bound the kept SELECT rows.
   */
  run: (query: string, opts?: RunOptions) => Promise<RunResult>;
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
    async (query: string, opts: RunOptions = {}): Promise<RunResult> => {
      const store = storeRef.current;
      if (!store) {
        const outcome: QueryOutcome = {
          kind: "error",
          message: "The engine is not ready yet — wait for the store to warm.",
        };
        return { outcome, latencyMs: 0 };
      }
      const mode = opts.mode ?? "run";
      const rowCap = opts.rowCap ?? DEFAULT_ROW_CAP;
      const signal = opts.signal;
      const form = classifyQuery(query);
      const t0 = performance.now();
      let outcome: QueryOutcome;
      try {
        if (mode === "explain" || mode === "analyze") {
          // EXPLAIN renders the planner's chosen plan; EXPLAIN ANALYZE also EXECUTES it and
          // traces the per-operator work (the wasm `explain` / `explainAnalyze` bindings, which
          // mirror `sparq_engine::explain[_analyze]` and the server's `explain=plan|analyze`).
          const plan = mode === "analyze" ? store.explainAnalyze(query) : store.explain(query);
          outcome = { kind: "explain", mode, plan };
        } else if (form === "ask") {
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
          // SELECT — STREAM the rows one batch at a time through the wasm cursor so a large
          // result never materialises whole in JS. We keep at most `rowCap` rows for the views;
          // rows beyond the cap are counted but dropped (the outcome records `truncated`). The
          // cooperative `signal` is checked between batches so Stop actually halts the pull.
          const kept: SparqlBinding[] = [];
          let total = 0;
          let cancelled = false;
          const meta = streamQueryRows(store, query, STREAM_BATCH_SIZE, (batch) => {
            if (signal?.aborted) {
              cancelled = true;
              // Throw to break streamQueryRows' loop; the cursor is freed in its `finally`.
              throw new AbortError();
            }
            total += batch.rows.length;
            for (const row of batch.rows) {
              if (kept.length < rowCap) kept.push(row);
            }
          });
          if (cancelled) {
            outcome = { kind: "cancelled" };
          } else {
            const parsed: SparqlResults = {
              head: { vars: meta.vars },
              results: { bindings: kept },
            };
            // `meta.rowCount` is the cursor's own total; prefer it (it covers an empty result's
            // single empty batch correctly), falling back to the counted total.
            const totalRows = meta.rowCount || total;
            outcome = {
              kind: "select",
              results: parsed,
              rawJson: formatSparqlJson(parsed),
              rowCount: kept.length,
              totalRows,
              truncated: totalRows > kept.length,
            };
          }
        }
      } catch (err) {
        outcome =
          err instanceof AbortError || signal?.aborted
            ? { kind: "cancelled" }
            : {
                kind: "error",
                message: err instanceof Error ? err.message : String(err),
              };
      }
      const latencyMs = performance.now() - t0;
      setLastLatencyMs(latencyMs);
      setLastRowCount(
        outcome.kind === "select"
          ? outcome.totalRows
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

  // [OPUS-4.8] sq-ixc3.10/.12 — EXPLAIN / EXPLAIN ANALYZE is NOT a separate context method: it is
  // run(query, { mode: "explain" | "analyze" }) above, which surfaces an { kind: "explain" }
  // outcome through the SAME RunResult + measured-latency pipeline every other run uses. The Cmd-K
  // "Run EXPLAIN" verb and the workbench EXPLAIN/ANALYZE buttons both drive that single path, so
  // there is one EXPLAIN contract (this consolidates the standalone explain() #1018 had shipped).

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
    () => ({ status, storeSize, graphs, lastLatencyMs, lastRowCount, run, serializeStore }),
    [status, storeSize, graphs, lastLatencyMs, lastRowCount, run, serializeStore],
  );

  return <EngineContext.Provider value={value}>{children}</EngineContext.Provider>;
}

export function useEngine(): EngineContextValue {
  const ctx = React.useContext(EngineContext);
  if (!ctx) throw new Error("useEngine must be used within an <EngineProvider>");
  return ctx;
}
