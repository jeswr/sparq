"use client";

// [FABLE-5] sq-ixc3.19 — the Plan explorer tool: a navigable EXPLAIN / EXPLAIN ANALYZE
// operator tree with per-operator time / cardinality / q-error heat, plus the live query
// monitor (this workbench session's in-flight queries, each with Kill).
//
// Three plan sources, one schema (the sq-jbqh4 `PlanNode` contract), rendered by the shared
// `PlanTree`:
//   * IN-TAB — the live workspace store's wasm engine (`run(query, {mode})`, the same canonical
//     EXPLAIN path as the Query tool). Exact rows + q-error; structured ANALYZE wall times use
//     performance.now() at browser host-timer resolution.
//   * NATIVE (desktop only) — the `explain_native` Tauri command over an N-Quads snapshot of the
//     same store, with per-operator wall nanos from the native monotonic clock.
//   * ENDPOINT — a remote sparq-server (`Accept: application/x-sparq-explain+json`), degrading
//     HONESTLY to the text plan on a lean/older server (rendered as text, never a faked tree).
//
// The MONITOR is honest about its scope (queries issued from THIS workbench) and about what
// Kill can do per target: an endpoint Kill aborts the HTTP request (the server bounds its own
// side with the cooperative budget/timeout); an in-tab/native explain is a single engine call
// with no cancellation point mid-flight, so its Kill is shown disabled with the reason.
// A server-side ALL-clients monitor/kill needs net-new sparq-server machinery (tracked as
// follow-up work) — this panel never pretends to be one.

import * as React from "react";
import { Activity, Network, OctagonX, Telescope } from "lucide-react";
import {
  type PlanNode,
  type SafetyWarning,
  EndpointError,
  connectionSafetyWarnings,
  hasBlockingWarning,
  runEndpointExplain,
} from "@sparq/client";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { isTauriRuntime } from "@/lib/tauri-ipc";
import {
  getRunning,
  killQuery,
  subscribeRunning,
  trackQuery,
  type RunningQuery,
} from "@/lib/query-monitor";
import { PlanTree } from "@/components/workbench/plan-tree";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import { formatNanos, formatQError, planSummary } from "@/lib/plan-view";

const DEFAULT_QUERY = "SELECT * WHERE { ?s ?p ?o } LIMIT 10";

type PlanSource = "wasm" | "native" | "endpoint";

type PlanState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "tree"; tree: PlanNode; analyzed: boolean; source: PlanSource }
  | { kind: "text"; plan: string; analyzed: boolean; source: PlanSource }
  | { kind: "error"; message: string };

export function PlanExplorer() {
  const { run, status } = useEngine();
  const [sparql, setSparql] = React.useState(DEFAULT_QUERY);
  const [state, setState] = React.useState<PlanState>({ kind: "idle" });

  // Endpoint mode — URL + token live in React state only (the Server tool's posture:
  // the token is never persisted and is sent only in the Authorization header).
  const [endpointUrl, setEndpointUrl] = React.useState("");
  const [endpointToken, setEndpointToken] = React.useState("");
  const endpointWarnings = endpointUrl ? connectionSafetyWarnings({ url: endpointUrl, token: endpointToken }) : [];
  const endpointBlocked = endpointUrl !== "" && hasBlockingWarning(endpointWarnings);

  const native = isTauriRuntime();
  const engineReady = status.kind === "ready";

  /** IN-TAB / NATIVE explain via the engine context's canonical run() path. */
  const runLocal = React.useCallback(
    async (mode: "explain" | "analyze", useNative: boolean) => {
      setState({ kind: "running" });
      const t = trackQuery(
        `Plan · ${useNative ? "native " : ""}${mode === "analyze" ? "ANALYZE" : "EXPLAIN"}`,
        sparql,
        useNative ? "native" : "local",
      );
      try {
        const res = await run(sparql, { mode, native: useNative, signal: t.signal });
        const o = res.outcome;
        if (o.kind === "explain") {
          const source: PlanSource = o.source === "native" ? "native" : "wasm";
          if (o.tree) {
            setState({ kind: "tree", tree: o.tree, analyzed: o.mode === "analyze", source });
          } else {
            setState({ kind: "text", plan: o.plan, analyzed: o.mode === "analyze", source });
          }
        } else if (o.kind === "cancelled") {
          setState({ kind: "idle" });
        } else if (o.kind === "error") {
          setState({ kind: "error", message: o.message });
        } else {
          setState({ kind: "error", message: "Unexpected outcome from the explain run." });
        }
      } catch (e) {
        setState({ kind: "error", message: e instanceof Error ? e.message : String(e) });
      } finally {
        t.finish();
      }
    },
    [run, sparql],
  );

  /** ENDPOINT explain — a killable fetch (the monitor aborts the tracked signal). */
  const runEndpoint = React.useCallback(
    async (mode: "plan" | "analyze") => {
      setState({ kind: "running" });
      const t = trackQuery(
        `Endpoint · ${mode === "analyze" ? "ANALYZE" : "EXPLAIN"}`,
        sparql,
        "endpoint",
      );
      const abortableFetch: typeof fetch = (input, init) =>
        fetch(input, { ...init, signal: t.signal });
      try {
        const r = await runEndpointExplain(
          { url: endpointUrl, token: endpointToken.trim() || undefined },
          sparql,
          mode,
          abortableFetch,
        );
        if (r.kind === "tree") {
          setState({ kind: "tree", tree: r.plan, analyzed: mode === "analyze", source: "endpoint" });
        } else {
          setState({ kind: "text", plan: r.plan, analyzed: mode === "analyze", source: "endpoint" });
        }
      } catch (e) {
        if (t.signal.aborted) {
          setState({ kind: "idle" });
        } else {
          const message =
            e instanceof EndpointError ? e.message : e instanceof Error ? e.message : String(e);
          setState({ kind: "error", message });
        }
      } finally {
        t.finish();
      }
    },
    [endpointUrl, endpointToken, sparql],
  );

  // ⌘/Ctrl-Enter runs the in-tab ANALYZE (the keyboard-first spine, matching the Query tool).
  const onKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
        e.preventDefault();
        void runLocal("analyze", false);
      }
    },
    [runLocal],
  );

  return (
    <div className="flex h-full flex-col" data-tool-panel="plan">
      {/* Editor + source actions */}
      <div className="flex min-h-0 flex-[2] flex-col border-b border-border">
        <WorkbenchSparqlEditor
          value={sparql}
          onChange={setSparql}
          onKeyDown={onKeyDown}
          ariaLabel="Plan explorer SPARQL editor"
          id="plan-query"
        />
        <div className="flex flex-wrap items-center gap-1.5 border-t border-border px-2 py-1.5">
          <Button
            size="sm"
            variant="outline"
            disabled={!engineReady || state.kind === "running"}
            onClick={() => void runLocal("explain", false)}
            title="Planning-only dry run over the live in-tab store"
            data-plan-explain
          >
            <Telescope className="size-3.5" aria-hidden />
            EXPLAIN
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={!engineReady || state.kind === "running"}
            onClick={() => void runLocal("analyze", false)}
            title="Execute over the live in-tab store and trace per-operator rows, q-error, and wall time at browser timer resolution"
            data-plan-analyze
          >
            <Activity className="size-3.5" aria-hidden />
            ANALYZE
          </Button>
          {native && (
            <Button
              size="sm"
              variant="outline"
              disabled={!engineReady || state.kind === "running"}
              onClick={() => void runLocal("analyze", true)}
              title="Execute over the desktop's NATIVE engine (a snapshot of the same store) — measures REAL per-operator wall time"
              data-plan-analyze-native
            >
              <Activity className="size-3.5" aria-hidden />
              Native ANALYZE
            </Button>
          )}
        </div>
      </div>

      {/* Result region */}
      <div className="min-h-0 flex-[3] overflow-auto" data-plan-result={state.kind}>
        <PlanResult state={state} />
      </div>

      {/* Endpoint strip + monitor */}
      <div className="shrink-0 border-t border-border">
        <EndpointStrip
          url={endpointUrl}
          setUrl={setEndpointUrl}
          token={endpointToken}
          setToken={setEndpointToken}
          warnings={endpointWarnings}
          blocked={endpointBlocked}
          busy={state.kind === "running"}
          onExplain={() => void runEndpoint("plan")}
          onAnalyze={() => void runEndpoint("analyze")}
        />
        <QueryMonitor />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Result rendering.
// ---------------------------------------------------------------------------

function PlanResult({ state }: { state: PlanState }) {
  if (state.kind === "idle") {
    return (
      <p className="p-3 text-sm text-muted-foreground">
        Run EXPLAIN (plan only) or ANALYZE (execute + per-operator rows, q-error and — on the
        desktop native / endpoint sources — real wall time) to explore the engine&apos;s plan.
      </p>
    );
  }
  if (state.kind === "running") {
    return <p className="p-3 text-sm text-muted-foreground">Running…</p>;
  }
  if (state.kind === "error") {
    return (
      <pre className="whitespace-pre-wrap p-3 font-mono text-xs text-destructive" data-plan-error>
        {state.message}
      </pre>
    );
  }
  if (state.kind === "text") {
    return (
      <div>
        <p className="border-b border-border px-3 py-1.5 text-[11px] text-muted-foreground">
          This source produced the TEXT plan only (a lean build without the structured
          `explain-json` surface) — shown verbatim, not synthesised into a tree.
        </p>
        <pre className="overflow-auto whitespace-pre p-3 font-mono text-xs" data-plan-text>
          {state.plan || "(no plan)"}
        </pre>
      </div>
    );
  }
  const summary = planSummary(state.tree);
  return (
    <div>
      <p
        className="flex flex-wrap gap-x-4 border-b border-border px-3 py-1.5 text-[11px] text-muted-foreground"
        data-plan-summary
      >
        <span>{summary.operators} operators</span>
        <span>
          rows{" "}
          {summary.rootRows === null ? "— (not executed)" : summary.rootRows.toLocaleString("en-US")}
        </span>
        <span>total {formatNanos(summary.rootNanos)}</span>
        <span>worst q-error {formatQError(summary.worstQError)}</span>
        <span className="uppercase">{state.source}</span>
      </p>
      <PlanTree tree={state.tree} analyzed={state.analyzed} source={state.source} />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Endpoint strip — explain against a remote sparq-server.
// ---------------------------------------------------------------------------

function EndpointStrip({
  url,
  setUrl,
  token,
  setToken,
  warnings,
  blocked,
  busy,
  onExplain,
  onAnalyze,
}: {
  url: string;
  setUrl: (v: string) => void;
  token: string;
  setToken: (v: string) => void;
  warnings: SafetyWarning[];
  blocked: boolean;
  busy: boolean;
  onExplain: () => void;
  onAnalyze: () => void;
}) {
  const ready = url.trim() !== "" && !blocked;
  return (
    <div className="border-b border-border px-3 py-2">
      <div className="flex flex-wrap items-center gap-1.5">
        <Network className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
        <input
          type="url"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="Endpoint /sparql URL (optional — explain on a running sparq-server)"
          aria-label="Endpoint URL"
          className="min-w-0 flex-1 rounded-md border bg-background px-2 py-1 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
          spellCheck={false}
          data-plan-endpoint-url
        />
        <input
          type="password"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          placeholder="Bearer token (optional)"
          aria-label="Endpoint bearer token (never persisted)"
          className="w-40 rounded-md border bg-background px-2 py-1 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
          data-plan-endpoint-token
        />
        <Button size="sm" variant="outline" disabled={!ready || busy} onClick={onExplain} data-plan-endpoint-explain>
          EXPLAIN
        </Button>
        <Button size="sm" variant="outline" disabled={!ready || busy} onClick={onAnalyze} data-plan-endpoint-analyze>
          ANALYZE
        </Button>
      </div>
      {url !== "" && warnings.length > 0 && (
        <ul className="mt-1 space-y-0.5">
          {warnings.map((w) => (
            <li
              key={w.code}
              className={cn(
                "text-[11px]",
                w.level === "error"
                  ? "text-destructive"
                  : w.level === "warning"
                    ? "text-[var(--warning)]"
                    : "text-muted-foreground",
              )}
            >
              {w.message}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// The live query monitor.
// ---------------------------------------------------------------------------

function QueryMonitor() {
  const running = React.useSyncExternalStore(subscribeRunning, getRunning, getRunning);
  // Re-render each second while anything is in flight, so the elapsed readout is live.
  const [, forceTick] = React.useReducer((n: number) => n + 1, 0);
  React.useEffect(() => {
    if (running.length === 0) return;
    const t = setInterval(forceTick, 1000);
    return () => clearInterval(t);
  }, [running.length]);

  return (
    <div className="px-3 py-2" data-plan-monitor>
      <h3 className="text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
        Running queries — this workbench
      </h3>
      {running.length === 0 ? (
        <p className="py-1 text-xs text-muted-foreground" data-plan-monitor-empty>
          No queries in flight. (Server-side monitoring of OTHER clients needs a sparq-server
          running-queries surface — tracked as follow-up work.)
        </p>
      ) : (
        <ul className="divide-y divide-border">
          {running.map((q) => (
            <MonitorRow key={q.id} q={q} />
          ))}
        </ul>
      )}
    </div>
  );
}

function MonitorRow({ q }: { q: RunningQuery }) {
  const elapsedMs = typeof performance !== "undefined" ? performance.now() - q.startedAt : 0;
  // Only an endpoint request has a genuine mid-flight cancellation point (aborting the
  // fetch); an in-tab/native explain is one engine call — Kill would be a lie there.
  const killable = q.target === "endpoint";
  return (
    <li className="flex items-center gap-2 py-1 text-xs" data-plan-monitor-row>
      <span className="shrink-0 rounded border border-border px-1 text-[10px] uppercase text-muted-foreground">
        {q.target}
      </span>
      <span className="shrink-0 text-muted-foreground">{q.label}</span>
      <span className="min-w-0 flex-1 truncate font-mono text-muted-foreground" title={q.sparql}>
        {q.sparql}
      </span>
      <span className="shrink-0 font-mono tabular-nums text-muted-foreground">
        {(elapsedMs / 1000).toFixed(1)}s
      </span>
      <Button
        size="sm"
        variant="ghost"
        disabled={!killable}
        onClick={() => killQuery(q.id)}
        title={
          killable
            ? "Abort the HTTP request (the server bounds its own side with its query budget/timeout)"
            : "An in-tab/native explain is a single engine call with no mid-flight cancellation point"
        }
        data-plan-kill
      >
        <OctagonX className="size-3.5" aria-hidden />
        Kill
      </Button>
    </li>
  );
}
