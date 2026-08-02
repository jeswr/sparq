"use client";

// (sq-lxomy) [SONNET-4.6] — Graph-view TOOL tab: working CONSTRUCT/DESCRIBE panel over the live
// store, using the already-shipped GraphView component (graph-view.tsx, sq-ixc3.12).
//
// The tool accepts any CONSTRUCT or DESCRIBE query typed in the compact textarea; the default is
// CONSTRUCT WHERE { ?s ?p ?o } LIMIT 100, which renders the live store (whatever the user has
// imported) as a node-link graph immediately on first Run. The result is fed directly to the
// existing <GraphView> component.
//
// [OPUS-5] sq-ixc3.22 — the tool is also the LENS RUNTIME. A lens (the framework-agnostic model
// in `@sparq/client`, edited in `graph-lens-panel.tsx`) makes the view programmable: its `start`
// slot picks the entry nodes, `expand` draws one node's neighbourhood, and the `nodeStyle` /
// `edgeStyle` slots colour, size and caption what is drawn. Clicking a node runs `expand` for it
// and MERGES the result into the picture, so exploration accumulates instead of replacing. Every
// slot is a real SPARQL query against the live store — a lens is configuration, never a second
// rendering path with its own idea of the data. With no start+expand pair the tool behaves
// exactly as it did before: Run executes the query in the editor.
//
// A lens READS, it never WRITES. Lens text is untrusted data (a pasted "shared lens", a
// hand-edited workspace record), so every slot run goes through `runSlot`, which classifies the
// exact string with the SAME `classifyQuery` the engine dispatches on and refuses anything that
// is not the slot's documented form — an UPDATE in any slot is reported, never executed.
//
// INVARIANT: renders REAL query results from the live store (never a canned figure). The
// MAX_TRIPLES render cap is GraphView's own (and is labelled there — no perf claim here).
//
// OVERRIDE: flip tier/built/group via GRAPH_VIEW_TOOL_OVERRIDE — never by editing data/tools.ts.

import * as React from "react";
import { Play, Loader2, Network, SlidersHorizontal } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { classifyQuery, useEngine, type QueryOutcome } from "@/lib/engine-context";
import { useWorkspace } from "@/lib/workspace-context";
import {
  GraphView,
  type GraphLensStyling,
  type InferredAffordance,
} from "@/components/workbench/graph-view";
import { GraphLensPanel } from "@/components/workbench/graph-lens-panel";
import { ProofPanel, type ExplainTarget } from "@/components/workbench/proof-panel";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import {
  bindFocusNode,
  edgeStyleIndex,
  graphLensSlotFormError,
  mergeNTriples,
  nodeDetailRows,
  nodeKeyOfRdfTerm,
  nodeStyleIndex,
  termToNTriples,
  type GraphEdgeStyle,
  type GraphLens,
  type GraphLensSlot,
  type GraphNodeDetailRow,
  type GraphNodeStyle,
  type RdfTerm,
} from "@sparq/client";

// [FABLE-5] sq-qgkwy.2 — the override lives in the sibling `.meta.ts` (eagerly bundled for the
// rail/tab honesty read path) so THIS panel module can stay behind a lazy dynamic import().
// Re-exported here so the panel file remains the tool's single import surface.
export { GRAPH_VIEW_TOOL_OVERRIDE } from "@/components/workbench/graph-view-tool.meta";

/** The default CONSTRUCT query the tool opens with — broad enough to show the sample graph. */
const DEFAULT_CONSTRUCT = `CONSTRUCT WHERE { ?s ?p ?o } LIMIT 100`;

/**
 * [OPUS-5] sq-ixc3.22 — how many of the lens `start` slot's nodes are expanded on Run. A start
 * query is free to return many entry points, and expanding all of them would fire one engine run
 * per row. This bound keeps Run to a predictable handful — the user expands further by clicking.
 * It is a UI bound, not a result bound, and the panel SAYS when it applied.
 */
const LENS_START_FANOUT = 5;

/** What the lens runtime is reporting about the last run (surfaced inline, never thrown). */
type LensStatus = { kind: "info" | "error"; message: string } | null;

/** Triples in an N-Triples document, counted the same cheap way `workspaceSummary` does. */
function countLines(ntriples: string): number {
  const trimmed = ntriples.trim();
  return trimmed === "" ? 0 : trimmed.split("\n").length;
}

export function GraphViewTool() {
  const { run, status, entailedTripleKeys, inferenceMode, inferenceStatus } = useEngine();
  const { activeLens } = useWorkspace();
  const [query, setQuery] = React.useState(DEFAULT_CONSTRUCT);
  const [running, setRunning] = React.useState(false);
  const [outcome, setOutcome] = React.useState<QueryOutcome | null>(null);
  // [OPUS-5] sq-ixc3.22 — the ACCUMULATED graph currently drawn, held apart from `outcome` so a
  // lens expansion can merge into it (click a node, its neighbourhood joins the picture) while
  // `outcome` still carries the non-graph shapes (error / cancelled / wrong query form).
  const [graphDoc, setGraphDoc] = React.useState<string>("");
  const [lensStyling, setLensStyling] = React.useState<GraphLensStyling | null>(null);
  const [focus, setFocus] = React.useState<RdfTerm | null>(null);
  const [detail, setDetail] = React.useState<GraphNodeDetailRow[] | null>(null);
  const [lensStatus, setLensStatus] = React.useState<LensStatus>(null);
  const [showLens, setShowLens] = React.useState(false);
  // [GPT-5.6] The standalone Graph tool owns its proof overlay just as QueryWorkbench does.
  const [explainTarget, setExplainTarget] = React.useState<ExplainTarget | null>(null);
  const abortRef = React.useRef<AbortController | null>(null);

  // [GPT-5.6] Exact closure-minus-base membership; an asserted edge never gains why().
  const inferred = React.useMemo<InferredAffordance | null>(() => {
    if (inferenceMode === "off" || inferenceStatus.kind !== "ready") return null;
    const keys = entailedTripleKeys();
    if (!keys || keys.size === 0) return null;
    return { keys, onExplain: setExplainTarget };
  }, [inferenceMode, inferenceStatus, entailedTripleKeys]);

  React.useEffect(() => {
    setExplainTarget(null);
  }, [inferenceMode]);

  // Switching lens invalidates everything the previous lens RESOLVED. The drawn graph stays —
  // it is the user's exploration, not the lens's output — but its styling and side panel do not.
  React.useEffect(() => {
    setLensStyling(null);
    setDetail(null);
    setLensStatus(null);
  }, [activeLens.id]);

  /** Run one query over the live store and hand back its outcome. */
  const runOne = React.useCallback(
    async (text: string, signal: AbortSignal): Promise<QueryOutcome> => {
      const result = await run(text, { signal });
      return result.outcome;
    },
    [run],
  );

  /**
   * Run ONE lens slot — the single path by which lens text reaches the engine.
   *
   * FAIL-CLOSED ON QUERY FORM. A lens is DATA: imported from a pasted JSON blob and restored from
   * a hand-editable workspace record, so its slot text is untrusted regardless of which slot it
   * sits in. `run` dispatches on `classifyQuery`, so a slot carrying an UPDATE would MUTATE the
   * live store, and the "that outcome was not a solution set / not a graph" checks below would
   * notice only after the damage. Classifying here with the SAME function `run` dispatches on and
   * refusing BEFORE the call closes that: what is refused is exactly what would have executed as
   * a mutation. The refusal comes back as an ordinary error outcome, so every call site reports
   * it through the status line it already has.
   *
   * The guard is at RUN time, not at import time, on purpose — a persisted or hand-edited lens
   * record never passes through the import path, so run time is the only chokepoint that sees
   * every slot. `text` is the FINAL string (focus already bound), so what is classified is
   * character-for-character what would be executed.
   */
  const runSlot = React.useCallback(
    async (slot: GraphLensSlot, text: string, signal: AbortSignal): Promise<QueryOutcome> => {
      const refusal = graphLensSlotFormError(slot, classifyQuery(text));
      if (refusal) return { kind: "error", message: refusal };
      return runOne(text, signal);
    },
    [runOne],
  );

  /**
   * Re-resolve the lens's styling slots over the live store. Both are plain SELECTs with no focus
   * binding, so they run once per graph change rather than once per node. A slot that errors is
   * reported and skipped — a broken `edgeStyle` must not blank a working `nodeStyle`.
   */
  const refreshStyling = React.useCallback(
    async (lens: GraphLens, signal: AbortSignal): Promise<void> => {
      let nodes: ReadonlyMap<string, GraphNodeStyle> = new Map();
      let edges: ReadonlyMap<string, GraphEdgeStyle> = new Map();
      let resolved = false;
      if (lens.queries.nodeStyle) {
        const res = await runSlot("nodeStyle", lens.queries.nodeStyle, signal);
        if (res.kind === "select") {
          nodes = nodeStyleIndex(res.results);
          resolved = true;
        } else if (res.kind === "error") {
          setLensStatus({ kind: "error", message: `Node-basics slot: ${res.message}` });
        }
      }
      if (lens.queries.edgeStyle) {
        const res = await runSlot("edgeStyle", lens.queries.edgeStyle, signal);
        if (res.kind === "select") {
          edges = edgeStyleIndex(res.results);
          resolved = true;
        } else if (res.kind === "error") {
          setLensStatus({ kind: "error", message: `Edge-basics slot: ${res.message}` });
        }
      }
      setLensStyling(resolved ? { nodes, edges } : null);
    },
    [runSlot],
  );

  /** Commit a merged N-Triples document as the drawn graph. */
  const showGraph = React.useCallback((ntriples: string) => {
    setGraphDoc(ntriples);
    setOutcome({ kind: "graph", ntriples, tripleCount: countLines(ntriples) });
  }, []);

  /**
   * Run the query in the editor — the tool's original behaviour, deliberately UNCHANGED by the
   * lens work. The active lens still styles what comes back (its `nodeStyle` / `edgeStyle` slots
   * describe the store, not this query), so a plain CONSTRUCT gains lens colours for free.
   */
  const onRun = React.useCallback(async () => {
    // A run already in flight: ignore (no parallel runs for this tool).
    if (abortRef.current) return;
    const controller = new AbortController();
    abortRef.current = controller;
    setRunning(true);
    setLensStatus(null);
    setFocus(null);
    setDetail(null);
    try {
      const result = await runOne(query, controller.signal);
      setOutcome(result);
      setGraphDoc(result.kind === "graph" ? result.ntriples : "");
      if (result.kind === "graph") await refreshStyling(activeLens, controller.signal);
    } finally {
      abortRef.current = null;
      setRunning(false);
    }
  }, [activeLens, query, refreshStyling, runOne]);

  /**
   * Run the LENS: its `start` slot picks the entry nodes and its `expand` slot draws each one's
   * neighbourhood, unioned into a single graph. Offered only when the lens fills both slots —
   * with either empty there is nothing programmable to run.
   */
  const onRunLens = React.useCallback(async () => {
    if (abortRef.current) return;
    const lens = activeLens;
    const start = lens.queries.start;
    const expand = lens.queries.expand;
    if (!start || !expand) return;
    const controller = new AbortController();
    abortRef.current = controller;
    setRunning(true);
    setLensStatus(null);
    setFocus(null);
    setDetail(null);
    try {
      const started = await runSlot("start", start, controller.signal);
      if (started.kind !== "select") {
        // The start slot answered something other than a solution set. Show it honestly rather
        // than pretending the lens ran: a CONSTRUCT still draws, everything else explains itself.
        setOutcome(started);
        setGraphDoc(started.kind === "graph" ? started.ntriples : "");
        if (started.kind === "error") {
          setLensStatus({ kind: "error", message: `Start slot: ${started.message}` });
        } else if (started.kind !== "cancelled") {
          setLensStatus({
            kind: "error",
            message: "The lens start slot must be a SELECT projecting ?node.",
          });
        }
        return;
      }
      const seeds = started.results.results.bindings
        .map((row) => termToNTriples(row.node))
        .filter((nt): nt is string => nt !== null);
      if (seeds.length === 0) {
        showGraph("");
        setLensStatus({ kind: "info", message: "The lens start slot bound no ?node." });
        return;
      }
      const expandedSeeds = seeds.slice(0, LENS_START_FANOUT);
      const docs: string[] = [];
      for (const nt of expandedSeeds) {
        const res = await runSlot("expand", bindFocusNode(expand, nt), controller.signal);
        if (res.kind === "graph") docs.push(res.ntriples);
        else if (res.kind === "error") {
          setLensStatus({ kind: "error", message: `Expand slot: ${res.message}` });
          break;
        } else if (res.kind === "cancelled") break;
      }
      showGraph(mergeNTriples(...docs));
      if (seeds.length > expandedSeeds.length) {
        setLensStatus({
          kind: "info",
          message: `Expanded ${expandedSeeds.length} of ${seeds.length} start nodes — click a node to expand further.`,
        });
      }
      await refreshStyling(lens, controller.signal);
    } finally {
      abortRef.current = null;
      setRunning(false);
    }
  }, [activeLens, refreshStyling, runSlot, showGraph]);

  /**
   * Click a node: run the lens's `expand` slot with `?node` bound to it and MERGE the result into
   * the drawn graph, then refresh the styling and the side panel. With no `expand` slot the click
   * still selects the node (and fills the side panel if the lens has a `nodeDetail` slot).
   */
  const onFocusNode = React.useCallback(
    async (term: RdfTerm) => {
      if (abortRef.current) return;
      const lens = activeLens;
      setFocus(term);
      setDetail(null);
      const expand = lens.queries.expand;
      const nodeDetail = lens.queries.nodeDetail;
      if (!expand && !nodeDetail) {
        setLensStatus({
          kind: "info",
          message: "This lens has no expand or node-panel slot — nothing to run for that node.",
        });
        return;
      }
      const controller = new AbortController();
      abortRef.current = controller;
      setRunning(true);
      setLensStatus(null);
      try {
        if (expand) {
          const res = await runSlot("expand", bindFocusNode(expand, term.nt), controller.signal);
          if (res.kind === "graph") {
            showGraph(mergeNTriples(graphDoc, res.ntriples));
            await refreshStyling(lens, controller.signal);
          } else if (res.kind === "error") {
            setLensStatus({ kind: "error", message: `Expand slot: ${res.message}` });
          }
        }
        if (nodeDetail) {
          const bound = bindFocusNode(nodeDetail, term.nt);
          const res = await runSlot("nodeDetail", bound, controller.signal);
          if (res.kind === "select") setDetail(nodeDetailRows(res.results));
          else if (res.kind === "error") {
            setLensStatus({ kind: "error", message: `Node-panel slot: ${res.message}` });
          }
        }
      } finally {
        abortRef.current = null;
        setRunning(false);
      }
    },
    [activeLens, graphDoc, refreshStyling, runSlot, showGraph],
  );

  // (sq-plqfs) [SONNET-4.6] ⌘/Ctrl-Enter fires Run from within the editor.
  const onKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
        e.preventDefault();
        void onRun();
      }
    },
    [onRun],
  );

  const ready = status.kind === "ready";
  const lensDriven = Boolean(activeLens.queries.start && activeLens.queries.expand);

  return (
    <div className="flex h-full flex-col" data-tool-panel="graph-view">
      {/* Action row */}
      <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
        <Network className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="text-xs font-medium text-muted-foreground">Graph view</span>
        <Badge variant="outline" className="h-5 gap-1 text-[10px]" title="Where this query runs">
          LOCAL · in-tab WASM
        </Badge>
        <Badge
          variant="outline"
          className="h-5 gap-1 text-[10px]"
          title={
            lensDriven
              ? "This lens can drive the view from its start + expand slots"
              : "This lens has no start/expand pair — it only styles what Run draws"
          }
          data-graph-active-lens
        >
          Lens · {activeLens.name}
        </Badge>
        <div className="ml-auto flex items-center gap-1.5">
          <Button
            size="sm"
            variant={showLens ? "secondary" : "ghost"}
            onClick={() => setShowLens((v) => !v)}
            title="Configure the graph lens (up to five SPARQL queries)"
            data-graph-lens-toggle
          >
            <SlidersHorizontal className="size-3.5" />
            Lens
          </Button>
          {/* [OPUS-5] sq-ixc3.22 — driving the view from the LENS is its own explicit action.
              `Run` keeps executing the query in the editor, unchanged, so the tool never
              silently swaps out what the button the user already knows does. The label
              deliberately avoids the word "Run" so it cannot collide with that button's
              accessible name in the e2e journeys. */}
          {lensDriven && (
            <Button
              size="sm"
              variant="outline"
              onClick={() => void onRunLens()}
              disabled={!ready || running}
              title="Draw with the active lens: its start slot picks the entry nodes, its expand slot draws them"
              data-graph-lens-run
            >
              <Network className="size-3.5" />
              Draw lens
            </Button>
          )}
          <Button
            size="sm"
            onClick={() => void onRun()}
            disabled={!ready || running}
            title="Run the CONSTRUCT or DESCRIBE over the live store"
            data-graph-view-run
          >
            {running ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />}
            Run
          </Button>
        </div>
      </div>

      {/* (sq-plqfs) Syntax-highlighting SPARQL editor — replaces the plain <textarea>.
          The id="graph-view-query" hook is preserved on the inner <textarea>
          (WorkbenchSparqlEditor passes it through). Fixed-height wrapper keeps the editor
          compact so the result graph fills most of the panel.
          [OPUS-5] sq-ixc3.22 — the Lens toggle swaps this row for the lens editor. */}
      {showLens ? (
        <div className="flex max-h-[60%] min-h-0 flex-col border-b">
          <GraphLensPanel />
        </div>
      ) : (
        <div className="flex h-28 flex-col border-b">
          <WorkbenchSparqlEditor
            id="graph-view-query"
            value={query}
            onChange={setQuery}
            onKeyDown={onKeyDown}
            ariaLabel="CONSTRUCT or DESCRIBE query"
          />
        </div>
      )}

      {lensStatus && (
        <p
          className={`border-b px-3 py-1 text-[11px] ${
            lensStatus.kind === "error" ? "text-destructive" : "text-muted-foreground"
          }`}
          data-graph-lens-run-status={lensStatus.kind}
        >
          {lensStatus.message}
        </p>
      )}

      {/* Result area */}
      <div className="relative flex min-h-0 flex-1 overflow-hidden" data-graph-view-result>
        <div className="min-h-0 flex-1 overflow-auto">
          {outcome === null ? (
            <p className="p-4 text-sm text-muted-foreground">
              {!ready
                ? "Waiting for the engine to warm…"
                : lensDriven
                  ? `Run a CONSTRUCT or DESCRIBE query to see it as a node-link graph, or use “Draw lens” to let the “${activeLens.name}” lens pick and expand the entry nodes for you.`
                  : "Run a CONSTRUCT or DESCRIBE query to see the result as a node-link graph."}
            </p>
          ) : outcome.kind === "graph" ? (
            <GraphView
              ntriples={graphDoc}
              inferred={inferred}
              lens={lensStyling}
              focusKey={focus ? nodeKeyOfRdfTerm(focus) : null}
              onFocusNode={(term) => void onFocusNode(term)}
            />
          ) : outcome.kind === "error" ? (
            <pre
              className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs text-destructive"
              data-result-kind="error"
            >
              {outcome.message}
            </pre>
          ) : outcome.kind === "cancelled" ? (
            <p className="p-4 text-sm text-muted-foreground">Run stopped.</p>
          ) : (
            <p className="p-4 text-sm text-muted-foreground">
              Use a{" "}
              <code className="rounded bg-muted px-1 py-0.5">CONSTRUCT</code> or{" "}
              <code className="rounded bg-muted px-1 py-0.5">DESCRIBE</code> query to generate a
              graph result that can be visualised here.
            </p>
          )}
        </div>

        {/* [OPUS-5] sq-ixc3.22 — the lens's node-panel slot for the focused node. */}
        {focus && (
          <aside
            className="w-64 shrink-0 overflow-auto border-l bg-card p-3 text-xs"
            data-graph-node-panel
          >
            <p className="mb-2 break-all font-mono text-[11px] text-muted-foreground">{focus.nt}</p>
            {detail === null ? (
              <p className="text-[11px] text-muted-foreground">
                {activeLens.queries.nodeDetail
                  ? "Running the node-panel slot…"
                  : "This lens has no node-panel slot."}
              </p>
            ) : detail.length === 0 ? (
              <p className="text-[11px] text-muted-foreground">
                The node-panel slot returned no rows for this node.
              </p>
            ) : (
              <dl className="flex flex-col gap-1">
                {detail.map((row, i) => (
                  <div key={i} className="flex flex-col">
                    <dt className="break-all text-[10px] text-muted-foreground">{row.key}</dt>
                    <dd className="break-words">{row.value}</dd>
                  </div>
                ))}
              </dl>
            )}
          </aside>
        )}

        {/* [GPT-5.6] why() overlay for an inferred edge clicked in this standalone tool. */}
        {explainTarget && (
          <ProofPanel target={explainTarget} onClose={() => setExplainTarget(null)} />
        )}
      </div>
    </div>
  );
}
