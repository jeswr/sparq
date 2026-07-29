"use client";

// [OPUS-5] sq-ixc3.24 — the VISUAL QUERY BUILDER: a diagram-to-SPARQL canvas over the live store.
//
// Competitor reference: AllegroGraph Gruff ("draw the pattern, get the query") and Stardog
// Explorer's model-driven query builder (walk relationship paths, attribute filters, AND-NOT,
// GROUP BY). What this out-flanks them on is SHAPE-AWARENESS: when the loaded store contains
// SHACL shapes, the predicate pickers are driven by the DECLARED shape properties; otherwise
// they fall back to characteristic sets OBSERVED in the data. Every suggestion is labelled with
// which of the two it is, because they mean different things (see `SuggestionSource`).
//
// The honesty spine of this tool — all of it enforced in `@/lib/query-builder`, which is where
// the pure model + serializer + validation live (and where the unit tests are):
//   * the generated SPARQL is STANDARD SPARQL 1.1, always VISIBLE and always EDITABLE — there is
//     no hidden dialect and no rewriting behind the user's back;
//   * suggestions come from queries over the LIVE store. No store, no suggestions — the panel
//     says so rather than offering an invented vocabulary;
//   * a model whose SPARQL would be invalid still RENDERS its SPARQL (so the user can read and
//     fix it) but cannot be RUN — the issues list says exactly why;
//   * editing the SPARQL by hand DETACHES it from the canvas, and the panel says so. The canvas
//     cannot parse SPARQL back into a diagram, so it never pretends the two are still in sync.

import * as React from "react";
import {
  AlertTriangle,
  Copy,
  Info,
  Loader2,
  Play,
  Plus,
  RefreshCw,
  Share2,
  Workflow,
} from "lucide-react";
import { extractTable, type SparqlResults } from "@sparq/client";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useEngine, type EngineContextValue, type QueryOutcome } from "@/lib/engine-context";
import { useWorkbench } from "@/components/workbench/workbench-context";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import { useRegisterPaletteCommands } from "@/components/workbench/command-palette";
import type { PaletteCommand } from "@/lib/palette-commands";
import {
  ANY_PREDICATE_QUERY,
  CHARACTERISTIC_SET_QUERY,
  CLASS_INDEX_QUERY,
  SHAPE_PROPERTY_QUERY,
  buildQuery,
  buildSchemaIndex,
  emptyModel,
  modelVariables,
  negatedNodeIds,
  uniqueVariable,
  variableStem,
  type BuilderAggregate,
  type BuilderAttribute,
  type BuilderEdge,
  type BuilderIssue,
  type BuilderModel,
  type BuilderNode,
  type SchemaIndex,
} from "@/lib/query-builder";
import {
  QueryBuilderCanvas,
  type CanvasSelection,
} from "@/components/workbench/query-builder-canvas";
import {
  EdgeInspector,
  NodeInspector,
  ResultShapePanel,
} from "@/components/workbench/query-builder-inspectors";

/**
 * Above this store size the four introspection queries are NOT run automatically on open — a
 * characteristic-set scan groups over every triple, and silently spending that on tab-open would
 * be rude. The panel offers the button instead and says why. This is a UX threshold, not a
 * benchmark or a claim about engine speed.
 */
const AUTO_INTROSPECT_MAX_TRIPLES = 200_000;

/** Rows kept for the inline preview table. The full result belongs in the Query tool. */
const PREVIEW_ROWS = 20;

/** Canvas placement grid (px) for newly-added node cards. */
const COL_GAP = 280;
const ROW_GAP = 190;

// ---------------------------------------------------------------------------
// Schema-introspection state.
// ---------------------------------------------------------------------------

type SchemaState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; index: SchemaIndex; partialErrors: string[]; stale: boolean }
  | { kind: "error"; message: string };

/** Run one introspection SELECT, returning its results or the engine's own error message. */
async function selectResults(
  run: EngineContextValue["run"],
  query: string,
): Promise<{ results: SparqlResults | null; error: string | null }> {
  try {
    const { outcome } = await run(query);
    if (outcome.kind === "select") return { results: outcome.results, error: null };
    if (outcome.kind === "error") return { results: null, error: outcome.message };
    return { results: null, error: null };
  } catch (err) {
    return { results: null, error: err instanceof Error ? err.message : String(err) };
  }
}

// ---------------------------------------------------------------------------
// Model helpers (id minting + immutable updates).
// ---------------------------------------------------------------------------

let idCounter = 0;
const mintId = (kind: string) => `${kind}-${++idCounter}`;

/** Replace one node in the model. */
function withNode(model: BuilderModel, id: string, fn: (n: BuilderNode) => BuilderNode): BuilderModel {
  return { ...model, nodes: model.nodes.map((n) => (n.id === id ? fn(n) : n)) };
}

/** Replace one attribute of one node. */
function withAttribute(
  model: BuilderModel,
  nodeId: string,
  attrId: string,
  fn: (a: BuilderAttribute) => BuilderAttribute,
): BuilderModel {
  return withNode(model, nodeId, (n) => ({
    ...n,
    attributes: n.attributes.map((a) => (a.id === attrId ? fn(a) : a)),
  }));
}

/** Replace one edge. */
function withEdge(model: BuilderModel, id: string, fn: (e: BuilderEdge) => BuilderEdge): BuilderModel {
  return { ...model, edges: model.edges.map((e) => (e.id === id ? fn(e) : e)) };
}

/** Drop a node and every edge that touched it (an edge to nowhere is not a diagram). */
function dropNode(model: BuilderModel, id: string): BuilderModel {
  return {
    ...model,
    nodes: model.nodes.filter((n) => n.id !== id),
    edges: model.edges.filter((e) => e.from !== id && e.to !== id),
  };
}

/** A free-ish canvas slot for the next node, laid out in columns. */
function nextPosition(count: number): { x: number; y: number } {
  return { x: 32 + (count % 3) * COL_GAP, y: 24 + Math.floor(count / 3) * ROW_GAP };
}

// ---------------------------------------------------------------------------
// The panel.
// ---------------------------------------------------------------------------

export function QueryBuilderTool() {
  const { run, status, storeSize, storeEpoch } = useEngine();
  const workbench = useWorkbench();

  const [model, setModel] = React.useState<BuilderModel>(() => emptyModel());
  const [selected, setSelected] = React.useState<CanvasSelection>(null);
  const [linkFrom, setLinkFrom] = React.useState<string | null>(null);
  const [schema, setSchema] = React.useState<SchemaState>({ kind: "idle" });
  /** Hand-edited SPARQL — non-null means the text has DETACHED from the canvas. */
  const [editedSparql, setEditedSparql] = React.useState<string | null>(null);
  const [preview, setPreview] = React.useState<QueryOutcome | null>(null);
  const [running, setRunning] = React.useState(false);
  const [bottomTab, setBottomTab] = React.useState<"issues" | "preview">("issues");
  const [copied, setCopied] = React.useState(false);

  const ready = status.kind === "ready";
  const built = React.useMemo(() => buildQuery(model), [model]);
  const sparql = editedSparql ?? built.sparql;
  const errorCount = built.issues.filter((i) => i.severity === "error").length;
  // A detached, hand-edited query is the user's own text: the canvas's validation no longer
  // describes it, so it is not used to block running.
  const canRun = ready && !running && (editedSparql !== null || built.runnable);

  const index = schema.kind === "ready" ? schema.index : null;

  // --- schema introspection ---------------------------------------------
  const loadSchema = React.useCallback(async () => {
    setSchema({ kind: "loading" });
    const classes = await selectResults(run, CLASS_INDEX_QUERY);
    if (classes.error) {
      setSchema({ kind: "error", message: classes.error });
      return;
    }
    const characteristics = await selectResults(run, CHARACTERISTIC_SET_QUERY);
    const anyPredicates = await selectResults(run, ANY_PREDICATE_QUERY);
    const shapes = await selectResults(run, SHAPE_PROPERTY_QUERY);
    const partialErrors = [characteristics.error, anyPredicates.error, shapes.error].filter(
      (e): e is string => e !== null,
    );
    setSchema({
      kind: "ready",
      index: buildSchemaIndex({
        classes: classes.results,
        characteristics: characteristics.results,
        anyPredicates: anyPredicates.results,
        shapes: shapes.results,
      }),
      partialErrors,
      stale: false,
    });
  }, [run]);

  // Introspect once on open, but only for a store small enough that doing it unasked is polite.
  const autoTriedRef = React.useRef(false);
  React.useEffect(() => {
    if (autoTriedRef.current || !ready) return;
    if (storeSize === 0 || storeSize > AUTO_INTROSPECT_MAX_TRIPLES) return;
    autoTriedRef.current = true;
    void loadSchema();
  }, [ready, storeSize, loadSchema]);

  // The store changed under a loaded index — mark it stale rather than silently showing
  // suggestions that describe data the store no longer holds.
  const epochRef = React.useRef(storeEpoch);
  React.useEffect(() => {
    if (epochRef.current === storeEpoch) return;
    epochRef.current = storeEpoch;
    setSchema((prev) => (prev.kind === "ready" ? { ...prev, stale: true } : prev));
  }, [storeEpoch]);

  // --- canvas actions ----------------------------------------------------
  // These three mint an id and move the selection, so they read `model` directly rather than
  // running inside a `setState` updater — an updater must stay pure (React may re-run it).
  const addNode = React.useCallback(
    (classIri: string | null) => {
      const stem = classIri ? variableStem(classIri) : "node";
      const node: BuilderNode = {
        id: mintId("node"),
        variable: uniqueVariable(stem, modelVariables(model)),
        classIri,
        attributes: [],
        projected: true,
        ...nextPosition(model.nodes.length),
      };
      setModel({ ...model, nodes: [...model.nodes, node] });
      setSelected({ kind: "node", id: node.id });
    },
    [model],
  );

  const addAttribute = React.useCallback(
    (nodeId: string, predicate: string) => {
      const owner = model.nodes.find((n) => n.id === nodeId);
      const attribute: BuilderAttribute = {
        id: mintId("attr"),
        predicate,
        variable: uniqueVariable(
          `${owner ? owner.variable : "node"}${capitalise(variableStem(predicate))}`,
          modelVariables(model),
        ),
        projected: true,
        optional: false,
        filter: null,
      };
      setModel(withNode(model, nodeId, (n) => ({ ...n, attributes: [...n.attributes, attribute] })));
    },
    [model],
  );

  const completeLink = React.useCallback(
    (toId: string) => {
      const fromId = linkFrom;
      setLinkFrom(null);
      if (!fromId || fromId === toId) return;
      const edge: BuilderEdge = {
        id: mintId("edge"),
        from: fromId,
        to: toId,
        predicate: "",
        alternatives: [],
        mode: "required",
      };
      setModel({ ...model, edges: [...model.edges, edge] });
      setSelected({ kind: "edge", id: edge.id });
    },
    [linkFrom, model],
  );

  const addAggregate = React.useCallback(() => {
    const aggregate: BuilderAggregate = {
      id: mintId("agg"),
      fn: "COUNT",
      target: "*",
      distinct: false,
      alias: uniqueVariable("count", modelVariables(model)),
    };
    setModel({ ...model, aggregates: [...model.aggregates, aggregate] });
  }, [model]);

  // Re-generating from the canvas is the only way back from a hand-edited query.
  const regenerate = React.useCallback(() => setEditedSparql(null), []);

  const onRun = React.useCallback(async () => {
    if (!canRun) return;
    setRunning(true);
    setBottomTab("preview");
    try {
      const { outcome } = await run(sparql, { rowCap: PREVIEW_ROWS * 5 });
      setPreview(outcome);
    } finally {
      setRunning(false);
    }
  }, [canRun, run, sparql]);

  const onSend = React.useCallback(() => {
    workbench?.sendToQueryEditor(sparql);
  }, [workbench, sparql]);

  const onCopy = React.useCallback(() => {
    void navigator.clipboard?.writeText(sparql).then(
      () => setCopied(true),
      () => setCopied(false),
    );
  }, [sparql]);
  React.useEffect(() => {
    if (!copied) return;
    const t = window.setTimeout(() => setCopied(false), 2000);
    return () => window.clearTimeout(t);
  }, [copied]);

  // --- Cmd-K verbs -------------------------------------------------------
  const paletteCommands = React.useMemo<PaletteCommand[]>(
    () => [
      {
        id: "builder.add-node",
        group: "Actions",
        title: "Builder: add a node",
        blurb: "Add a node to the visual query builder's canvas",
        keywords: ["builder", "node", "canvas", "diagram", "add"],
        icon: Plus,
        run: () => {
          workbench?.openTool("builder");
          addNode(index?.classes[0]?.classIri ?? null);
        },
      },
      {
        id: "builder.send",
        group: "Actions",
        title: "Builder: open the drawn query in the Query editor",
        blurb: "Send the builder's generated SPARQL to the Query tool",
        keywords: ["builder", "sparql", "query", "editor", "send", "open"],
        icon: Share2,
        run: onSend,
      },
    ],
    [workbench, addNode, index, onSend],
  );
  useRegisterPaletteCommands("builder", paletteCommands);

  const selectedNode =
    selected?.kind === "node" ? (model.nodes.find((n) => n.id === selected.id) ?? null) : null;
  const selectedEdge =
    selected?.kind === "edge" ? (model.edges.find((e) => e.id === selected.id) ?? null) : null;

  return (
    <div className="flex h-full flex-col" data-tool-panel="builder">
      {/* Action row */}
      <div className="flex flex-wrap items-center gap-2 border-b bg-card px-3 py-1.5">
        <Workflow className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
        <span className="text-xs font-medium text-muted-foreground">Query builder</span>
        <Badge variant="outline" className="h-5 gap-1 text-[10px]" title="Where this query runs">
          LOCAL · in-tab WASM
        </Badge>
        <SchemaBadge schema={schema} onLoad={() => void loadSchema()} storeSize={storeSize} />
        <div className="ml-auto flex items-center gap-1.5">
          <Button
            size="sm"
            variant="outline"
            onClick={() => addNode(index?.classes[0]?.classIri ?? null)}
            title="Add a node to the canvas"
            data-builder-add-node
          >
            <Plus className="size-3.5" /> Node
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={onCopy}
            title="Copy the generated SPARQL to the clipboard"
            data-builder-copy
          >
            <Copy className="size-3.5" /> {copied ? "Copied" : "Copy"}
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={onSend}
            title="Load this SPARQL into the Query tool's editor"
            data-builder-send
          >
            <Share2 className="size-3.5" /> Open in Query
          </Button>
          <Button
            size="sm"
            onClick={() => void onRun()}
            disabled={!canRun}
            title={
              built.runnable || editedSparql !== null
                ? "Run this SPARQL over the live store"
                : "Fix the issues below before running"
            }
            data-builder-run
          >
            {running ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />}
            Run
          </Button>
        </div>
      </div>

      {/* Canvas + inspector */}
      <div className="flex min-h-0 flex-1">
        <QueryBuilderCanvas
          model={model}
          selected={selected}
          linkFrom={linkFrom}
          onSelect={setSelected}
          onMove={(id, x, y) => setModel((prev) => withNode(prev, id, (n) => ({ ...n, x, y })))}
          onStartLink={setLinkFrom}
          onCompleteLink={completeLink}
          onDelete={(id) => {
            setModel((prev) => dropNode(prev, id));
            setSelected(null);
          }}
          onAddNode={() => addNode(index?.classes[0]?.classIri ?? null)}
        />
        <aside className="flex w-80 shrink-0 flex-col overflow-y-auto border-l" data-builder-inspector>
          {selectedNode ? (
            <NodeInspector
              // Remount per node: the inspector holds local input state (the free-text class /
              // predicate fields) that must not leak from one node to the next.
              key={selectedNode.id}
              node={selectedNode}
              index={index}
              negated={negatedNodeIds(model).has(selectedNode.id)}
              onChange={(fn) => setModel((prev) => withNode(prev, selectedNode.id, fn))}
              onChangeAttribute={(attrId, fn) =>
                setModel((prev) => withAttribute(prev, selectedNode.id, attrId, fn))
              }
              onAddAttribute={(predicate) => addAttribute(selectedNode.id, predicate)}
              onDropAttribute={(attrId) =>
                setModel((prev) =>
                  withNode(prev, selectedNode.id, (n) => ({
                    ...n,
                    attributes: n.attributes.filter((a) => a.id !== attrId),
                  })),
                )
              }
            />
          ) : selectedEdge ? (
            <EdgeInspector
              key={selectedEdge.id}
              edge={selectedEdge}
              model={model}
              index={index}
              onChange={(fn) => setModel((prev) => withEdge(prev, selectedEdge.id, fn))}
              onDelete={() => {
                setModel((prev) => ({ ...prev, edges: prev.edges.filter((e) => e.id !== selectedEdge.id) }));
                setSelected(null);
              }}
            />
          ) : (
            <p className="p-3 text-xs text-muted-foreground">
              Select a node or an edge to edit it. Everything you change is reflected in the SPARQL
              below — that text is the whole query, with nothing added behind it.
            </p>
          )}
          <ResultShapePanel model={model} onChange={setModel} onAddAggregate={addAggregate} />
        </aside>
      </div>

      {/* Generated SPARQL + issues/preview */}
      <div className="flex h-64 shrink-0 border-t">
        <div className="flex min-w-0 flex-1 flex-col">
          <div className="flex items-center gap-2 border-b px-3 py-1 text-[11px] text-muted-foreground">
            <span className="font-medium">Generated SPARQL</span>
            {editedSparql !== null ? (
              <>
                <Badge variant="outline" className="h-4 gap-1 text-[10px] text-[var(--warning)]">
                  edited by hand
                </Badge>
                <span className="min-w-0 truncate">
                  the canvas no longer drives this text (it cannot read SPARQL back)
                </span>
                <button
                  type="button"
                  onClick={regenerate}
                  className="ml-auto shrink-0 rounded-sm underline hover:text-foreground"
                  data-builder-regenerate
                >
                  Regenerate from canvas
                </button>
              </>
            ) : (
              <span className="min-w-0 truncate">
                standard SPARQL 1.1 — editable; editing detaches it from the canvas
              </span>
            )}
          </div>
          <div className="min-h-0 flex-1">
            <WorkbenchSparqlEditor
              id="builder-sparql"
              value={sparql}
              onChange={setEditedSparql}
              ariaLabel="Generated SPARQL query"
            />
          </div>
        </div>
        <div className="flex w-96 shrink-0 flex-col border-l">
          <div className="flex items-center gap-1 border-b px-2 py-1 text-[11px]">
            <TabButton active={bottomTab === "issues"} onClick={() => setBottomTab("issues")} testId="issues">
              Issues{built.issues.length > 0 ? ` (${built.issues.length})` : ""}
            </TabButton>
            <TabButton active={bottomTab === "preview"} onClick={() => setBottomTab("preview")} testId="preview">
              Preview
            </TabButton>
          </div>
          <div className="min-h-0 flex-1 overflow-auto">
            {bottomTab === "issues" ? (
              <IssueList issues={built.issues} detached={editedSparql !== null} errorCount={errorCount} />
            ) : (
              <PreviewPane outcome={preview} running={running} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function capitalise(s: string): string {
  return s.length === 0 ? s : s[0].toUpperCase() + s.slice(1);
}

// ---------------------------------------------------------------------------
// Toolbar bits.
// ---------------------------------------------------------------------------

function TabButton({
  active,
  onClick,
  testId,
  children,
}: {
  active: boolean;
  onClick: () => void;
  testId: string;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-sm px-2 py-0.5 ${active ? "bg-muted font-medium text-foreground" : "text-muted-foreground hover:text-foreground"}`}
      data-builder-tab={testId}
    >
      {children}
    </button>
  );
}

/**
 * The suggestion-provenance badge. It states, without softening: where the pickers' contents
 * came from, whether SHACL shapes were found IN THE STORE, and when the index is stale.
 */
function SchemaBadge({
  schema,
  onLoad,
  storeSize,
}: {
  schema: SchemaState;
  onLoad: () => void;
  storeSize: number;
}) {
  if (schema.kind === "loading") {
    return (
      <span className="flex items-center gap-1 text-[11px] text-muted-foreground">
        <Loader2 className="size-3 animate-spin" aria-hidden /> introspecting the store…
      </span>
    );
  }
  if (schema.kind === "error") {
    return (
      <span className="flex items-center gap-1 text-[11px] text-destructive" data-builder-schema="error">
        <AlertTriangle className="size-3" aria-hidden /> schema introspection failed — pickers are
        empty ({schema.message})
      </span>
    );
  }
  if (schema.kind === "ready") {
    return (
      <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground" data-builder-schema="ready">
        <span>
          {schema.index.classes.length} classes ·{" "}
          {schema.index.hasShapes ? "SHACL shapes in the store" : "no shapes in the store"}
        </span>
        {schema.stale && (
          <Badge variant="outline" className="h-4 text-[10px] text-[var(--warning)]">
            stale — the store changed
          </Badge>
        )}
        {schema.partialErrors.length > 0 && (
          <span className="text-[var(--warning)]" title={schema.partialErrors.join("\n")}>
            partial
          </span>
        )}
        <button type="button" onClick={onLoad} className="underline" data-builder-refresh-schema>
          <RefreshCw className="inline size-3" aria-hidden /> refresh
        </button>
      </span>
    );
  }
  return (
    <span className="flex items-center gap-1 text-[11px] text-muted-foreground" data-builder-schema="idle">
      {storeSize === 0
        ? "the store is empty — import data to get suggestions"
        : `${storeSize.toLocaleString()} triples — introspection is not run automatically at this size`}
      {storeSize > 0 && (
        <button type="button" onClick={onLoad} className="underline" data-builder-load-schema>
          introspect the store
        </button>
      )}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Issues + preview.
// ---------------------------------------------------------------------------

function IssueList({
  issues,
  detached,
  errorCount,
}: {
  issues: BuilderIssue[];
  detached: boolean;
  errorCount: number;
}) {
  if (detached) {
    return (
      <p className="p-3 text-xs text-muted-foreground">
        The SPARQL has been edited by hand, so these checks describe the CANVAS, not the text you
        are about to run.
        {errorCount > 0 && ` (The canvas currently has ${errorCount} error${errorCount === 1 ? "" : "s"}.)`}
      </p>
    );
  }
  if (issues.length === 0) {
    return (
      <p className="p-3 text-xs text-muted-foreground" data-builder-issues="none">
        No issues — the diagram maps cleanly to the SPARQL shown.
      </p>
    );
  }
  return (
    <ul className="divide-y" data-builder-issues={errorCount > 0 ? "error" : "warning"}>
      {issues.map((issue, i) => (
        <li key={i} className="flex gap-2 p-2 text-[11px]">
          {issue.severity === "error" ? (
            <AlertTriangle className="mt-0.5 size-3 shrink-0 text-destructive" aria-hidden />
          ) : (
            <Info className="mt-0.5 size-3 shrink-0 text-[var(--warning)]" aria-hidden />
          )}
          <span className={issue.severity === "error" ? "text-destructive" : "text-muted-foreground"}>
            {issue.message}
          </span>
        </li>
      ))}
    </ul>
  );
}

function PreviewPane({ outcome, running }: { outcome: QueryOutcome | null; running: boolean }) {
  if (running) {
    return (
      <p className="flex items-center gap-2 p-3 text-xs text-muted-foreground">
        <Loader2 className="size-3.5 animate-spin" aria-hidden /> Running…
      </p>
    );
  }
  if (!outcome) {
    return (
      <p className="p-3 text-xs text-muted-foreground">
        Run the query to preview its first rows here. The full result set, exports and EXPLAIN live
        in the Query tool — use “Open in Query”.
      </p>
    );
  }
  if (outcome.kind === "error") {
    return (
      <pre className="whitespace-pre-wrap p-3 font-mono text-[11px] text-destructive" data-builder-preview="error">
        {outcome.message}
      </pre>
    );
  }
  if (outcome.kind !== "select") {
    return (
      <p className="p-3 text-xs text-muted-foreground">
        The builder generates SELECT queries; this result is a {outcome.kind} result.
      </p>
    );
  }
  const table = extractTable(outcome.results);
  const shown = table.rows.slice(0, PREVIEW_ROWS);
  return (
    <div data-builder-preview="select">
      <p className="border-b px-2 py-1 text-[11px] text-muted-foreground">
        {outcome.totalRows.toLocaleString()} row{outcome.totalRows === 1 ? "" : "s"}
        {shown.length < table.rows.length || outcome.truncated
          ? ` · showing the first ${shown.length}`
          : ""}
      </p>
      <table className="w-full border-collapse text-[11px]">
        <thead>
          <tr>
            {table.vars.map((v) => (
              <th key={v} className="border-b px-2 py-1 text-left font-mono font-medium">
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {shown.map((row, i) => (
            <tr key={i}>
              {row.map((cell, j) => (
                <td key={j} className="max-w-40 truncate border-b px-2 py-0.5 font-mono" title={cell}>
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
