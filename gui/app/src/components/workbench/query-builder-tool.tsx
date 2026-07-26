"use client";

// [SONNET-4.6] sq-ixc3.24 (#2700) — the VISUAL QUERY BUILDER tool: a diagram-to-SPARQL canvas.
//
// Competitor parity (research/competitive-feature-analysis-2026-07.md line 115): AllegroGraph
// Gruff is the reference for "draw the pattern, get the query"; Stardog Explorer contributes the
// model-driven half — walk relationship paths, attribute filters, AND-NOT, GROUP BY. The gap
// slice this closes is emitting HONEST SPARQL into the existing editor.
//
// Two-way by construction: the generated SPARQL is ALWAYS visible and ALWAYS editable in the pane
// below the canvas. Hand-editing it does not silently vanish on the next canvas change — the pane
// switches to "hand-edited" and stops regenerating until the user asks it to (Regenerate). The
// builder never round-trips text back into the diagram: parsing arbitrary SPARQL into a canvas is
// not something this v1 does, and pretending to would be the dishonest option.
//
// SUGGESTIONS ARE EARNED, NOT INVENTED. The predicate pickers are driven by real introspection of
// the LIVE store — characteristic sets (`?s a ?class ; ?p ?o` grouped) — merged with SHACL
// property shapes when a shapes graph is present in the store. Each offer shows its provenance
// (shape / data / both) and its observed use count; with an empty store and no shapes there are
// no suggestions, and the picker says exactly that.
//
// The tool RUNS nothing itself: "Open in Query" hands the text to the Query tool's editor
// (lib/query-handoff.ts), where the user reads, edits and runs it like any other query. No hidden
// dialect, no rewriting behind the user's back.
//
// Stable hooks for a later e2e lane: [data-tool-panel="query-builder"], [data-builder-canvas],
// [data-builder-node], [data-builder-edge], [data-builder-warning], the <textarea
// id="query-builder-sparql">, and [data-builder-open-in-query].

import * as React from "react";
import {
  Boxes,
  Plus,
  RefreshCw,
  Loader2,
  Trash2,
  ArrowRightLeft,
  Copy,
  Wand2,
  AlertTriangle,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useEngine, type EngineContextValue } from "@/lib/engine-context";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import { useWorkbench } from "@/components/workbench/workbench-context";
import { useRegisterPaletteCommands } from "@/components/workbench/command-palette";
import { requestQueryHandoff } from "@/lib/query-handoff";
import type { PaletteCommand } from "@/lib/palette-commands";
import type { SparqlResults } from "@sparq/client";
import {
  CHARACTERISTIC_SETS_QUERY,
  CLASSES_QUERY,
  SHAPES_QUERY,
  UNSCOPED_PREDICATES_QUERY,
  addAttribute,
  addNode,
  addRelationship,
  aggregatableVariables,
  buildSparql,
  buildSuggestionIndex,
  connectNodes,
  emptyModel,
  emptySuggestionIndex,
  localName,
  modelVariables,
  nextId,
  partitionSuggestions,
  removeNode,
  setAggregateFn,
  suggestionsFor,
  uniqueVariable,
  type AggregateFn,
  type BuilderEdge,
  type BuilderModel,
  type BuilderNode,
  type FilterOp,
  type OperandKind,
  type PredicateSuggestion,
  type SuggestionIndex,
} from "@/lib/query-builder";

// [SONNET-4.6] sq-ixc3.24 — the override lives in the sibling `.meta.ts` (eagerly bundled for the
// rail/tab honesty read path) so THIS panel module can stay behind a lazy dynamic import().
export { QUERY_BUILDER_TOOL_OVERRIDE } from "@/components/workbench/query-builder-tool.meta";

/** Node card geometry — the edge layer anchors to these, so they must match the CSS width. */
const NODE_WIDTH = 176;
const ANCHOR_Y = 18;

const CONTROL =
  "h-6 w-full rounded border bg-background px-1.5 text-[11px] text-foreground disabled:opacity-50";

const FILTER_OPS: { id: FilterOp; label: string }[] = [
  { id: "eq", label: "=" },
  { id: "ne", label: "≠" },
  { id: "lt", label: "<" },
  { id: "le", label: "≤" },
  { id: "gt", label: ">" },
  { id: "ge", label: "≥" },
  { id: "contains", label: "contains" },
  { id: "starts", label: "starts with" },
  { id: "ends", label: "ends with" },
  { id: "regex", label: "matches (regex)" },
];

const OPERAND_KINDS: OperandKind[] = ["literal", "number", "iri"];

const AGGREGATE_FNS: AggregateFn[] = [
  "COUNT",
  "SUM",
  "AVG",
  "MIN",
  "MAX",
  "SAMPLE",
  "GROUP_CONCAT",
];

type Selection = { kind: "node"; id: string } | { kind: "edge"; id: string } | null;

interface Probe {
  results: SparqlResults | null;
  error: string | null;
}

/** Run one introspection SELECT, surfacing an engine error instead of silently degrading. */
async function probe(run: EngineContextValue["run"], query: string): Promise<Probe> {
  try {
    const { outcome } = await run(query);
    if (outcome.kind === "select") return { results: outcome.results, error: null };
    if (outcome.kind === "error") return { results: null, error: outcome.message };
    return { results: null, error: null };
  } catch (error) {
    return { results: null, error: error instanceof Error ? error.message : String(error) };
  }
}

/** The provenance pill on a suggestion — never claims data support a shape-only offer has. */
function sourceLabel(suggestion: PredicateSuggestion): string {
  const uses = suggestion.count === null ? "" : ` · ${suggestion.count.toLocaleString()}`;
  if (suggestion.source === "shape") return "shape";
  if (suggestion.source === "both") return `shape + data${uses}`;
  return `data${uses}`;
}

function suggestionOptionLabel(suggestion: PredicateSuggestion): string {
  const name = suggestion.label ?? localName(suggestion.predicate);
  const required = suggestion.required ? " *" : "";
  return `${name}${required} — ${sourceLabel(suggestion)}`;
}

export function QueryBuilderTool() {
  const { run, status, storeEpoch, storeSize } = useEngine();
  const workbench = useWorkbench();

  const [model, setModel] = React.useState<BuilderModel>(emptyModel);
  const [index, setIndex] = React.useState<SuggestionIndex>(emptySuggestionIndex);
  const [selection, setSelection] = React.useState<Selection>(null);
  const [introspecting, setIntrospecting] = React.useState(false);
  const [introspectError, setIntrospectError] = React.useState<string | null>(null);
  /** The epoch the current schema snapshot was taken at — null until it has ever run. */
  const [schemaEpoch, setSchemaEpoch] = React.useState<number | null>(null);
  /** Non-null once the user hand-edits the SPARQL; the canvas then stops overwriting it. */
  const [draft, setDraft] = React.useState<string | null>(null);
  const [copied, setCopied] = React.useState(false);

  const canvasRef = React.useRef<HTMLDivElement>(null);
  const dragRef = React.useRef<{ id: string; dx: number; dy: number } | null>(null);

  const ready = status.kind === "ready";
  const built = React.useMemo(() => buildSparql(model), [model]);
  const sparql = draft ?? built.sparql;

  const introspect = React.useCallback(async () => {
    setIntrospecting(true);
    setIntrospectError(null);
    try {
      const [classes, characteristicSets, unscoped, shapes] = await Promise.all([
        probe(run, CLASSES_QUERY),
        probe(run, CHARACTERISTIC_SETS_QUERY),
        probe(run, UNSCOPED_PREDICATES_QUERY),
        probe(run, SHAPES_QUERY),
      ]);
      const failure = [classes, characteristicSets, unscoped, shapes].find((p) => p.error !== null);
      setIntrospectError(failure ? failure.error : null);
      setIndex(
        buildSuggestionIndex({
          classes: classes.results,
          characteristicSets: characteristicSets.results,
          unscoped: unscoped.results,
          shapes: shapes.results,
        }),
      );
      setSchemaEpoch(storeEpoch);
    } finally {
      setIntrospecting(false);
    }
  }, [run, storeEpoch]);

  // Introspect ONCE when the engine first becomes ready. Later store changes do not silently
  // re-run it (an import can be large) — the strip says the snapshot is stale and offers Refresh.
  React.useEffect(() => {
    if (!ready || schemaEpoch !== null) return;
    void introspect();
  }, [ready, schemaEpoch, introspect]);

  const stale = schemaEpoch !== null && schemaEpoch !== storeEpoch;

  const selectedNode =
    selection?.kind === "node" ? (model.nodes.find((n) => n.id === selection.id) ?? null) : null;
  const selectedEdge =
    selection?.kind === "edge" ? (model.edges.find((e) => e.id === selection.id) ?? null) : null;

  const updateNode = React.useCallback((nodeId: string, patch: Partial<BuilderNode>) => {
    setModel((current) => ({
      ...current,
      nodes: current.nodes.map((node) => (node.id === nodeId ? { ...node, ...patch } : node)),
    }));
  }, []);

  const updateEdge = React.useCallback((edgeId: string, patch: Partial<BuilderEdge>) => {
    setModel((current) => ({
      ...current,
      edges: current.edges.map((edge) => (edge.id === edgeId ? { ...edge, ...patch } : edge)),
    }));
  }, []);

  // These two select what they just created, so they read the current model directly rather than
  // running a side effect inside a state updater (updaters must stay pure).
  const onAddNode = React.useCallback(
    (classIri: string | null) => {
      const added = addNode(model, { classIri });
      setModel(added.model);
      setSelection({ kind: "node", id: added.nodeId });
    },
    [model],
  );

  const onWalk = React.useCallback(
    (fromNodeId: string, suggestion: PredicateSuggestion) => {
      const walked = addRelationship(model, fromNodeId, suggestion.predicate, {
        targetClass: suggestion.objectClass,
      });
      setModel(walked.model);
      setSelection({ kind: "node", id: walked.nodeId });
    },
    [model],
  );

  const onConnect = React.useCallback(
    (fromNodeId: string, toNodeId: string, predicate: string) => {
      const connected = connectNodes(model, fromNodeId, toNodeId, predicate);
      setModel(connected.model);
      setSelection({ kind: "edge", id: connected.edgeId });
    },
    [model],
  );

  const onAddAttribute = React.useCallback((nodeId: string, predicate: string) => {
    setModel((current) => addAttribute(current, nodeId, predicate).model);
  }, []);

  const onRemoveNode = React.useCallback((nodeId: string) => {
    setModel((current) => removeNode(current, nodeId));
    setSelection(null);
  }, []);

  const onRemoveEdge = React.useCallback((edgeId: string) => {
    setModel((current) => ({ ...current, edges: current.edges.filter((e) => e.id !== edgeId) }));
    setSelection(null);
  }, []);

  // ── canvas dragging (pointer events; no drag library) ───────────────────────────────────────
  const onNodePointerDown = React.useCallback(
    (event: React.PointerEvent<HTMLDivElement>, node: BuilderNode) => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      dragRef.current = {
        id: node.id,
        dx: event.clientX - rect.left + canvas.scrollLeft - node.x,
        dy: event.clientY - rect.top + canvas.scrollTop - node.y,
      };
      event.currentTarget.setPointerCapture(event.pointerId);
      setSelection({ kind: "node", id: node.id });
    },
    [],
  );

  const onNodePointerMove = React.useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      const drag = dragRef.current;
      const canvas = canvasRef.current;
      if (!drag || !canvas) return;
      const rect = canvas.getBoundingClientRect();
      updateNode(drag.id, {
        x: Math.max(0, event.clientX - rect.left + canvas.scrollLeft - drag.dx),
        y: Math.max(0, event.clientY - rect.top + canvas.scrollTop - drag.dy),
      });
    },
    [updateNode],
  );

  const onNodePointerUp = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (dragRef.current) event.currentTarget.releasePointerCapture(event.pointerId);
    dragRef.current = null;
  }, []);

  // ── handoff into the Query tool's editor ───────────────────────────────────────────────────
  const onOpenInQuery = React.useCallback(() => {
    requestQueryHandoff(sparql);
    workbench?.openTool("query");
  }, [sparql, workbench]);

  const onCopy = React.useCallback(() => {
    void navigator.clipboard?.writeText(sparql).then(
      () => setCopied(true),
      () => setCopied(false),
    );
  }, [sparql]);

  React.useEffect(() => {
    if (!copied) return;
    const handle = setTimeout(() => setCopied(false), 1500);
    return () => clearTimeout(handle);
  }, [copied]);

  const paletteCommands = React.useMemo<PaletteCommand[]>(
    () => [
      {
        id: "query-builder.open-in-query",
        group: "Actions",
        title: "Send the built query to the Query editor",
        blurb: "Hand the canvas's generated SPARQL to the Query tool",
        keywords: ["builder", "sparql", "diagram"],
        icon: ArrowRightLeft,
        run: onOpenInQuery,
      },
      {
        id: "query-builder.refresh-schema",
        group: "Actions",
        title: "Refresh the query builder's schema suggestions",
        blurb: "Re-introspect classes, characteristic sets and SHACL shapes",
        keywords: ["builder", "schema", "shapes"],
        icon: RefreshCw,
        run: () => void introspect(),
        disabled: !ready || introspecting,
      },
    ],
    [onOpenInQuery, introspect, ready, introspecting],
  );
  useRegisterPaletteCommands("query-builder", paletteCommands);

  const extent = React.useMemo(() => {
    let width = 640;
    let height = 360;
    for (const node of model.nodes) {
      width = Math.max(width, node.x + NODE_WIDTH + 80);
      height = Math.max(height, node.y + 200);
    }
    return { width, height };
  }, [model.nodes]);

  return (
    <div className="flex h-full flex-col" data-tool-panel="query-builder">
      {/* Action row */}
      <div className="flex flex-wrap items-center gap-2 border-b bg-card px-3 py-1.5">
        <Boxes className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="text-xs font-medium text-muted-foreground">Query builder</span>
        <Badge variant="outline" className="h-5 text-[10px]" title="Where the suggestions come from">
          LOCAL · in-tab WASM
        </Badge>
        <AddNodeControl index={index} onAdd={onAddNode} />
        <div className="ml-auto flex items-center gap-2 text-[11px] text-muted-foreground">
          <span data-builder-schema-summary>
            {schemaEpoch === null
              ? "schema not introspected yet"
              : `${index.classes.length.toLocaleString()} classes · ${
                  index.shapesPresent ? "SHACL shapes found" : "no SHACL shapes in the store"
                }`}
          </span>
          {stale && (
            <Badge variant="warning" className="h-5 text-[10px]">
              store changed since this snapshot
            </Badge>
          )}
          <Button
            size="sm"
            variant="outline"
            onClick={() => void introspect()}
            disabled={!ready || introspecting}
            title="Re-introspect classes, characteristic sets and SHACL shapes from the live store"
            data-builder-refresh
          >
            {introspecting ? (
              <Loader2 className="size-3.5 animate-spin" />
            ) : (
              <RefreshCw className="size-3.5" />
            )}
            Refresh schema
          </Button>
        </div>
      </div>

      {introspectError && (
        <p
          className="flex items-start gap-1.5 border-b bg-destructive/10 px-3 py-1.5 text-[11px] text-destructive"
          data-builder-schema-error
        >
          <AlertTriangle className="mt-0.5 size-3 shrink-0" />
          <span>
            Schema introspection failed — the pickers show only what did load. {introspectError}
          </span>
        </p>
      )}

      <div className="flex min-h-0 flex-1">
        {/* Canvas */}
        <div
          ref={canvasRef}
          className="relative min-w-0 flex-1 overflow-auto bg-[radial-gradient(circle,var(--border)_1px,transparent_1px)] [background-size:16px_16px]"
          data-builder-canvas
        >
          <div
            className="relative"
            style={{ width: extent.width, height: extent.height }}
            onClick={(event) => {
              // A click on the empty canvas (not on a node/edge) clears the selection.
              if (event.target === event.currentTarget) setSelection(null);
            }}
          >
            <svg
              className="pointer-events-none absolute left-0 top-0"
              width={extent.width}
              height={extent.height}
              aria-hidden
            >
              {model.edges.map((edge) => {
                const from = model.nodes.find((n) => n.id === edge.from);
                const to = model.nodes.find((n) => n.id === edge.to);
                if (!from || !to) return null;
                return (
                  <line
                    key={edge.id}
                    x1={from.x + NODE_WIDTH / 2}
                    y1={from.y + ANCHOR_Y}
                    x2={to.x + NODE_WIDTH / 2}
                    y2={to.y + ANCHOR_Y}
                    stroke="currentColor"
                    className={cn(
                      "text-muted-foreground",
                      edge.negated && "text-destructive",
                      selection?.kind === "edge" && selection.id === edge.id && "text-primary",
                    )}
                    strokeWidth={selection?.kind === "edge" && selection.id === edge.id ? 2 : 1.5}
                    strokeDasharray={edge.optional || edge.negated ? "4 3" : undefined}
                  />
                );
              })}
            </svg>

            {model.edges.map((edge) => {
              const from = model.nodes.find((n) => n.id === edge.from);
              const to = model.nodes.find((n) => n.id === edge.to);
              if (!from || !to) return null;
              const midX = (from.x + to.x) / 2 + NODE_WIDTH / 2;
              const midY = (from.y + to.y) / 2 + ANCHOR_Y;
              return (
                <button
                  key={edge.id}
                  type="button"
                  className={cn(
                    "absolute -translate-x-1/2 -translate-y-1/2 rounded border bg-card px-1.5 py-0.5 font-mono text-[10px] shadow-sm hover:bg-accent",
                    selection?.kind === "edge" && selection.id === edge.id && "ring-1 ring-primary",
                    edge.negated && "text-destructive",
                  )}
                  style={{ left: midX, top: midY }}
                  onClick={() => setSelection({ kind: "edge", id: edge.id })}
                  data-builder-edge={edge.id}
                  title={edge.predicate}
                >
                  {edge.negated ? "NOT " : ""}
                  {edge.optional && !edge.negated ? "OPTIONAL " : ""}
                  {localName(edge.predicate)}
                  {edge.alternates.length > 0 ? ` +${edge.alternates.length}` : ""}
                </button>
              );
            })}

            {model.nodes.map((node) => (
              <NodeCard
                key={node.id}
                node={node}
                selected={selection?.kind === "node" && selection.id === node.id}
                onPointerDown={onNodePointerDown}
                onPointerMove={onNodePointerMove}
                onPointerUp={onNodePointerUp}
                onSelect={() => setSelection({ kind: "node", id: node.id })}
              />
            ))}

            {model.nodes.length === 0 && (
              <p className="pointer-events-none absolute left-1/2 top-16 w-72 -translate-x-1/2 text-center text-xs text-muted-foreground">
                Draw a pattern: add a node for a class, walk its relationships, and filter its
                attributes. The SPARQL below is generated as you go — and stays editable.
              </p>
            )}
          </div>
        </div>

        {/* Inspector */}
        <aside className="flex w-72 shrink-0 flex-col overflow-y-auto border-l bg-card/40">
          {selectedNode ? (
            <NodeInspector
              node={selectedNode}
              nodes={model.nodes}
              index={index}
              storeSize={storeSize}
              introspected={schemaEpoch !== null}
              onChange={(patch) => updateNode(selectedNode.id, patch)}
              onWalk={(suggestion) => onWalk(selectedNode.id, suggestion)}
              onConnect={(toNodeId, predicate) => onConnect(selectedNode.id, toNodeId, predicate)}
              onAddAttribute={(predicate) => onAddAttribute(selectedNode.id, predicate)}
              onRemove={() => onRemoveNode(selectedNode.id)}
            />
          ) : selectedEdge ? (
            <EdgeInspector
              edge={selectedEdge}
              onChange={(patch) => updateEdge(selectedEdge.id, patch)}
              onRemove={() => onRemoveEdge(selectedEdge.id)}
            />
          ) : (
            <p className="p-3 text-[11px] text-muted-foreground">
              Select a node or a relationship on the canvas to edit it.
            </p>
          )}
          <AggregatePanel model={model} onChange={setModel} />
        </aside>
      </div>

      {/* Generated SPARQL — always visible, always editable. */}
      <div className="flex h-64 shrink-0 flex-col border-t">
        <div className="flex items-center gap-2 border-b bg-card px-3 py-1">
          <span className="text-[11px] font-medium text-muted-foreground">Generated SPARQL</span>
          {draft !== null && (
            <Badge variant="warning" className="h-5 text-[10px]" data-builder-hand-edited>
              hand-edited — the canvas is no longer regenerating it
            </Badge>
          )}
          <div className="ml-auto flex items-center gap-1.5">
            {draft !== null && (
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setDraft(null)}
                title="Discard the hand edits and regenerate from the canvas"
                data-builder-regenerate
              >
                <Wand2 className="size-3.5" />
                Regenerate
              </Button>
            )}
            <Button size="sm" variant="outline" onClick={onCopy} title="Copy the query">
              <Copy className="size-3.5" />
              {copied ? "Copied" : "Copy"}
            </Button>
            <Button
              size="sm"
              onClick={onOpenInQuery}
              title="Load this query into the Query tool's editor"
              data-builder-open-in-query
            >
              <ArrowRightLeft className="size-3.5" />
              Open in Query
            </Button>
          </div>
        </div>

        {built.warnings.length > 0 && (
          <ul className="max-h-20 shrink-0 overflow-y-auto border-b bg-muted/40 px-3 py-1 text-[11px] text-muted-foreground">
            {built.warnings.map((warning) => (
              <li key={warning} className="flex items-start gap-1.5" data-builder-warning>
                <AlertTriangle className="mt-0.5 size-3 shrink-0 text-[var(--warning)]" />
                <span>{warning}</span>
              </li>
            ))}
          </ul>
        )}

        <div className="flex min-h-0 flex-1 flex-col">
          <WorkbenchSparqlEditor
            id="query-builder-sparql"
            value={sparql}
            onChange={setDraft}
            ariaLabel="Generated SPARQL query"
          />
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Canvas pieces.
// ---------------------------------------------------------------------------

function NodeCard({
  node,
  selected,
  onPointerDown,
  onPointerMove,
  onPointerUp,
  onSelect,
}: {
  node: BuilderNode;
  selected: boolean;
  onPointerDown: (event: React.PointerEvent<HTMLDivElement>, node: BuilderNode) => void;
  onPointerMove: (event: React.PointerEvent<HTMLDivElement>) => void;
  onPointerUp: (event: React.PointerEvent<HTMLDivElement>) => void;
  onSelect: () => void;
}) {
  return (
    <div
      className={cn(
        "absolute rounded-md border bg-card shadow-sm",
        selected ? "ring-1 ring-primary" : "hover:border-primary/40",
      )}
      style={{ left: node.x, top: node.y, width: NODE_WIDTH }}
      data-builder-node={node.id}
      onClick={onSelect}
    >
      <div
        className="flex cursor-grab items-center gap-1 rounded-t-md border-b bg-muted/50 px-2 py-1 active:cursor-grabbing"
        onPointerDown={(event) => onPointerDown(event, node)}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        <span className="truncate font-mono text-[11px] text-primary">?{node.variable}</span>
        {node.projected && (
          <span className="ml-auto text-[9px] uppercase text-muted-foreground">in result</span>
        )}
      </div>
      <div className="px-2 py-1 text-[10px]">
        <p className="truncate text-muted-foreground" title={node.classIri ?? undefined}>
          {node.classIri ? localName(node.classIri) : "any node (untyped)"}
        </p>
        {node.attributes.map((attribute) => (
          <p key={attribute.id} className="truncate font-mono" title={attribute.predicate}>
            {attribute.optional ? "? " : ""}
            {localName(attribute.predicate)}
            {attribute.filter ? ` ${filterGlyph(attribute.filter.op)} ${attribute.filter.value}` : ""}
          </p>
        ))}
      </div>
    </div>
  );
}

function filterGlyph(op: FilterOp): string {
  return FILTER_OPS.find((entry) => entry.id === op)?.label ?? op;
}

function AddNodeControl({
  index,
  onAdd,
}: {
  index: SuggestionIndex;
  onAdd: (classIri: string | null) => void;
}) {
  const [value, setValue] = React.useState("");
  return (
    <div className="flex items-center gap-1">
      <select
        className="h-7 max-w-48 rounded border bg-background px-1.5 text-[11px]"
        value={value}
        onChange={(event) => setValue(event.currentTarget.value)}
        aria-label="Class for the next node"
        data-builder-class-picker
      >
        <option value="">Any node (untyped)</option>
        {index.classes.map((entry) => (
          <option key={entry.iri} value={entry.iri}>
            {localName(entry.iri)}
            {entry.instances === null ? " (shape only)" : ` (${entry.instances.toLocaleString()})`}
          </option>
        ))}
      </select>
      <Button
        size="sm"
        variant="outline"
        onClick={() => onAdd(value === "" ? null : value)}
        title="Add a node to the canvas"
        data-builder-add-node
      >
        <Plus className="size-3.5" />
        Add node
      </Button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Inspectors.
// ---------------------------------------------------------------------------

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="border-b px-3 py-2">
      <h3 className="mb-1.5 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
        {title}
      </h3>
      {children}
    </section>
  );
}

function NodeInspector({
  node,
  nodes,
  index,
  storeSize,
  introspected,
  onChange,
  onWalk,
  onConnect,
  onAddAttribute,
  onRemove,
}: {
  node: BuilderNode;
  nodes: BuilderNode[];
  index: SuggestionIndex;
  storeSize: number;
  introspected: boolean;
  onChange: (patch: Partial<BuilderNode>) => void;
  onWalk: (suggestion: PredicateSuggestion) => void;
  onConnect: (toNodeId: string, predicate: string) => void;
  onAddAttribute: (predicate: string) => void;
  onRemove: () => void;
}) {
  const offer = suggestionsFor(index, node.classIri);
  const { relationships, attributes } = partitionSuggestions(offer);
  const emptyOffer = offer.length === 0;

  const updateAttribute = (attributeId: string, patch: Partial<BuilderNode["attributes"][number]>) =>
    onChange({
      attributes: node.attributes.map((attribute) =>
        attribute.id === attributeId ? { ...attribute, ...patch } : attribute,
      ),
    });

  return (
    <>
      <Section title="Node">
        <label className="mb-1.5 block text-[10px] text-muted-foreground">
          Variable
          <input
            className={cn(CONTROL, "mt-0.5 font-mono")}
            value={node.variable}
            onChange={(event) => onChange({ variable: event.currentTarget.value })}
            aria-label="Node variable name"
            data-builder-node-variable
          />
        </label>
        <label className="mb-1.5 flex items-center gap-1.5 text-[11px]">
          <input
            type="checkbox"
            checked={node.projected}
            onChange={(event) => onChange({ projected: event.currentTarget.checked })}
          />
          Include in the result
        </label>
        <p className="truncate text-[10px] text-muted-foreground" title={node.classIri ?? undefined}>
          {node.classIri ? node.classIri : "Untyped — matches any term."}
        </p>
        <Button
          size="sm"
          variant="destructive"
          className="mt-1.5 w-full"
          onClick={onRemove}
          data-builder-remove-node
        >
          <Trash2 className="size-3.5" />
          Remove node
        </Button>
      </Section>

      <Section title="Walk a relationship">
        {emptyOffer ? (
          <p className="text-[11px] text-muted-foreground">
            {introspected
              ? storeSize === 0
                ? "The store is empty — load data (or a SHACL shapes graph) to get suggestions."
                : "No predicate was observed for this class and no shape declares one."
              : "Waiting for the schema snapshot…"}
          </p>
        ) : relationships.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">
            Every suggestion for this class has literal objects — see Attributes below.
          </p>
        ) : (
          <RelationshipPicker
            suggestions={relationships}
            targets={nodes.filter((candidate) => candidate.id !== node.id)}
            onWalk={onWalk}
            onConnect={onConnect}
          />
        )}
      </Section>

      <Section title="Attributes">
        {attributes.length > 0 && (
          <SuggestionPicker
            suggestions={attributes}
            label="Attribute to add"
            action="Add"
            onPick={(suggestion) => onAddAttribute(suggestion.predicate)}
            testId="builder-add-attribute"
          />
        )}
        {node.attributes.length === 0 ? (
          <p className="mt-1.5 text-[11px] text-muted-foreground">No attributes yet.</p>
        ) : (
          <ul className="mt-1.5 space-y-2">
            {node.attributes.map((attribute) => (
              <li key={attribute.id} className="rounded border p-1.5" data-builder-attribute>
                <div className="flex items-center gap-1">
                  <span
                    className="truncate font-mono text-[11px]"
                    title={attribute.predicate}
                  >
                    {localName(attribute.predicate)}
                  </span>
                  <button
                    type="button"
                    className="ml-auto rounded p-0.5 text-muted-foreground hover:bg-accent"
                    onClick={() =>
                      onChange({
                        attributes: node.attributes.filter((a) => a.id !== attribute.id),
                      })
                    }
                    aria-label={`Remove ${localName(attribute.predicate)}`}
                  >
                    <Trash2 className="size-3" />
                  </button>
                </div>
                <div className="mt-1 grid grid-cols-2 gap-1">
                  <select
                    className={CONTROL}
                    value={attribute.filter?.op ?? ""}
                    onChange={(event) => {
                      const op = event.currentTarget.value;
                      updateAttribute(attribute.id, {
                        filter:
                          op === ""
                            ? null
                            : {
                                op: op as FilterOp,
                                value: attribute.filter?.value ?? "",
                                kind: attribute.filter?.kind ?? "literal",
                              },
                      });
                    }}
                    aria-label={`Filter comparison for ${localName(attribute.predicate)}`}
                  >
                    <option value="">no filter</option>
                    {FILTER_OPS.map((entry) => (
                      <option key={entry.id} value={entry.id}>
                        {entry.label}
                      </option>
                    ))}
                  </select>
                  <input
                    className={CONTROL}
                    value={attribute.filter?.value ?? ""}
                    disabled={!attribute.filter}
                    placeholder="value"
                    onChange={(event) =>
                      attribute.filter &&
                      updateAttribute(attribute.id, {
                        filter: { ...attribute.filter, value: event.currentTarget.value },
                      })
                    }
                    aria-label={`Filter value for ${localName(attribute.predicate)}`}
                  />
                </div>
                {attribute.filter && (
                  <select
                    className={cn(CONTROL, "mt-1")}
                    value={attribute.filter.kind}
                    onChange={(event) =>
                      attribute.filter &&
                      updateAttribute(attribute.id, {
                        filter: {
                          ...attribute.filter,
                          kind: event.currentTarget.value as OperandKind,
                        },
                      })
                    }
                    aria-label={`Value kind for ${localName(attribute.predicate)}`}
                  >
                    {OPERAND_KINDS.map((kind) => (
                      <option key={kind} value={kind}>
                        as {kind}
                      </option>
                    ))}
                  </select>
                )}
                <div className="mt-1 flex flex-wrap gap-2 text-[10px]">
                  <label className="flex items-center gap-1">
                    <input
                      type="checkbox"
                      checked={attribute.projected}
                      onChange={(event) =>
                        updateAttribute(attribute.id, { projected: event.currentTarget.checked })
                      }
                    />
                    in result
                  </label>
                  <label className="flex items-center gap-1">
                    <input
                      type="checkbox"
                      checked={attribute.optional}
                      onChange={(event) =>
                        updateAttribute(attribute.id, { optional: event.currentTarget.checked })
                      }
                    />
                    optional
                  </label>
                </div>
              </li>
            ))}
          </ul>
        )}
      </Section>
    </>
  );
}

/**
 * The relationship affordance: pick a predicate, then either WALK it (creating the target node,
 * typed by the shape's `sh:class` when it declares one) or CONNECT it to a node already on the
 * canvas — the second is what lets a diagram express a cycle rather than only a tree.
 */
function RelationshipPicker({
  suggestions,
  targets,
  onWalk,
  onConnect,
}: {
  suggestions: PredicateSuggestion[];
  targets: BuilderNode[];
  onWalk: (suggestion: PredicateSuggestion) => void;
  onConnect: (toNodeId: string, predicate: string) => void;
}) {
  const [predicate, setPredicate] = React.useState("");
  const [target, setTarget] = React.useState("");
  const chosen = suggestions.find((s) => s.predicate === predicate) ?? suggestions[0];
  const existing = targets.find((candidate) => candidate.id === target) ?? null;

  return (
    <div className="space-y-1">
      <select
        className={CONTROL}
        value={chosen ? chosen.predicate : ""}
        onChange={(event) => setPredicate(event.currentTarget.value)}
        aria-label="Relationship to add"
        data-builder-relationship-select
      >
        {suggestions.map((suggestion) => (
          <option key={suggestion.predicate} value={suggestion.predicate}>
            {suggestionOptionLabel(suggestion)}
          </option>
        ))}
      </select>
      <div className="flex items-center gap-1">
        <select
          className={CONTROL}
          value={existing ? existing.id : ""}
          onChange={(event) => setTarget(event.currentTarget.value)}
          aria-label="Relationship target"
          data-builder-relationship-target
        >
          <option value="">to a new node</option>
          {targets.map((candidate) => (
            <option key={candidate.id} value={candidate.id}>
              to ?{candidate.variable}
            </option>
          ))}
        </select>
        <Button
          size="sm"
          variant="outline"
          className="shrink-0"
          disabled={!chosen}
          onClick={() => {
            if (!chosen) return;
            if (existing) onConnect(existing.id, chosen.predicate);
            else onWalk(chosen);
          }}
          data-builder-add-relationship
        >
          {existing ? "Connect" : "Walk"}
        </Button>
      </div>
    </div>
  );
}

function SuggestionPicker({
  suggestions,
  label,
  action,
  onPick,
  testId,
}: {
  suggestions: PredicateSuggestion[];
  label: string;
  action: string;
  onPick: (suggestion: PredicateSuggestion) => void;
  testId: string;
}) {
  const [value, setValue] = React.useState("");
  const chosen = suggestions.find((s) => s.predicate === value) ?? suggestions[0];
  return (
    <div className="flex items-center gap-1">
      <select
        className={CONTROL}
        value={chosen ? chosen.predicate : ""}
        onChange={(event) => setValue(event.currentTarget.value)}
        aria-label={label}
        data-testid={`${testId}-select`}
      >
        {suggestions.map((suggestion) => (
          <option key={suggestion.predicate} value={suggestion.predicate}>
            {suggestionOptionLabel(suggestion)}
          </option>
        ))}
      </select>
      <Button
        size="sm"
        variant="outline"
        className="shrink-0"
        onClick={() => chosen && onPick(chosen)}
        disabled={!chosen}
        data-testid={`${testId}-button`}
      >
        {action}
      </Button>
    </div>
  );
}

function EdgeInspector({
  edge,
  onChange,
  onRemove,
}: {
  edge: BuilderEdge;
  onChange: (patch: Partial<BuilderEdge>) => void;
  onRemove: () => void;
}) {
  return (
    <Section title="Relationship">
      <p className="truncate font-mono text-[11px]" title={edge.predicate}>
        {localName(edge.predicate)}
      </p>
      <p className="mt-0.5 break-all text-[10px] text-muted-foreground">{edge.predicate}</p>
      <label className="mt-1.5 flex items-center gap-1.5 text-[11px]">
        <input
          type="checkbox"
          checked={edge.optional}
          onChange={(event) => onChange({ optional: event.currentTarget.checked })}
          data-builder-edge-optional
        />
        Optional (OPTIONAL)
      </label>
      <label className="mt-1 flex items-center gap-1.5 text-[11px]">
        <input
          type="checkbox"
          checked={edge.negated}
          onChange={(event) => onChange({ negated: event.currentTarget.checked })}
          data-builder-edge-negated
        />
        Exclude matches (FILTER NOT EXISTS)
      </label>
      <Button size="sm" variant="destructive" className="mt-2 w-full" onClick={onRemove}>
        <Trash2 className="size-3.5" />
        Remove relationship
      </Button>
    </Section>
  );
}

// ---------------------------------------------------------------------------
// The aggregate / GROUP BY panel.
// ---------------------------------------------------------------------------

function AggregatePanel({
  model,
  onChange,
}: {
  model: BuilderModel;
  onChange: React.Dispatch<React.SetStateAction<BuilderModel>>;
}) {
  const variables = aggregatableVariables(model);

  const addAggregate = () =>
    onChange((current) => ({
      ...current,
      aggregates: [
        ...current.aggregates,
        {
          id: nextId(
            "g",
            current.aggregates.map((a) => a.id),
          ),
          fn: "COUNT",
          variable: "*",
          distinct: false,
          alias: uniqueVariable("count", modelVariables(current)),
        },
      ],
    }));

  return (
    <Section title="Aggregate (GROUP BY)">
      {model.aggregates.length === 0 ? (
        <p className="mb-1.5 text-[11px] text-muted-foreground">
          No aggregate — the query projects rows.
        </p>
      ) : (
        <ul className="mb-1.5 space-y-2">
          {model.aggregates.map((aggregate) => (
            <li key={aggregate.id} className="rounded border p-1.5" data-builder-aggregate>
              <div className="grid grid-cols-2 gap-1">
                <select
                  className={CONTROL}
                  value={aggregate.fn}
                  onChange={(event) => {
                    const fn = event.currentTarget.value as AggregateFn;
                    // Only COUNT takes `*`, so the model edit re-points a wildcard on the way out.
                    onChange((current) => setAggregateFn(current, aggregate.id, fn));
                  }}
                  aria-label="Aggregate function"
                >
                  {AGGREGATE_FNS.map((fn) => (
                    <option key={fn} value={fn}>
                      {fn}
                    </option>
                  ))}
                </select>
                <select
                  className={CONTROL}
                  value={aggregate.variable}
                  onChange={(event) =>
                    onChange((current) => ({
                      ...current,
                      aggregates: current.aggregates.map((entry) =>
                        entry.id === aggregate.id
                          ? { ...entry, variable: event.currentTarget.value }
                          : entry,
                      ),
                    }))
                  }
                  aria-label="Aggregated variable"
                >
                  {aggregate.fn === "COUNT" ? (
                    <option value="*">*</option>
                  ) : aggregate.variable.trim() === "*" ? (
                    // No bound variable to move to — surfaced, not silently emitted as `SUM(*)`.
                    <option value="*">(pick a variable)</option>
                  ) : null}
                  {variables.map((name) => (
                    <option key={name} value={name}>
                      ?{name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="mt-1 flex items-center gap-1">
                <input
                  className={cn(CONTROL, "font-mono")}
                  value={aggregate.alias}
                  onChange={(event) =>
                    onChange((current) => ({
                      ...current,
                      aggregates: current.aggregates.map((entry) =>
                        entry.id === aggregate.id
                          ? { ...entry, alias: event.currentTarget.value }
                          : entry,
                      ),
                    }))
                  }
                  aria-label="Aggregate result variable"
                />
                <button
                  type="button"
                  className="rounded p-0.5 text-muted-foreground hover:bg-accent"
                  onClick={() =>
                    onChange((current) => ({
                      ...current,
                      aggregates: current.aggregates.filter((entry) => entry.id !== aggregate.id),
                    }))
                  }
                  aria-label="Remove aggregate"
                >
                  <Trash2 className="size-3" />
                </button>
              </div>
              <label className="mt-1 flex items-center gap-1 text-[10px]">
                <input
                  type="checkbox"
                  checked={aggregate.distinct}
                  onChange={(event) =>
                    onChange((current) => ({
                      ...current,
                      aggregates: current.aggregates.map((entry) =>
                        entry.id === aggregate.id
                          ? { ...entry, distinct: event.currentTarget.checked }
                          : entry,
                      ),
                    }))
                  }
                />
                DISTINCT
              </label>
            </li>
          ))}
        </ul>
      )}

      <Button size="sm" variant="outline" className="w-full" onClick={addAggregate}>
        <Plus className="size-3.5" />
        Add aggregate
      </Button>

      {model.aggregates.length > 0 && variables.length > 0 && (
        <div className="mt-2">
          <p className="mb-1 text-[10px] uppercase tracking-wide text-muted-foreground">Group by</p>
          <div className="flex flex-wrap gap-2">
            {variables.map((name) => (
              <label key={name} className="flex items-center gap-1 text-[10px]">
                <input
                  type="checkbox"
                  checked={model.groupBy.includes(name)}
                  onChange={(event) =>
                    onChange((current) => ({
                      ...current,
                      groupBy: event.currentTarget.checked
                        ? [...current.groupBy, name]
                        : current.groupBy.filter((entry) => entry !== name),
                    }))
                  }
                />
                ?{name}
              </label>
            ))}
          </div>
        </div>
      )}

      <div className="mt-2 flex items-center gap-3 text-[10px]">
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={model.distinct}
            onChange={(event) =>
              onChange((current) => ({ ...current, distinct: event.currentTarget.checked }))
            }
          />
          DISTINCT
        </label>
        <label className="flex items-center gap-1">
          LIMIT
          <input
            className="h-6 w-16 rounded border bg-background px-1 text-[10px]"
            type="number"
            min={0}
            value={model.limit ?? ""}
            onChange={(event) => {
              const raw = event.currentTarget.value;
              const parsed = Number.parseInt(raw, 10);
              onChange((current) => ({
                ...current,
                limit: raw === "" || !Number.isFinite(parsed) ? null : parsed,
              }));
            }}
            aria-label="Result limit"
          />
        </label>
      </div>
    </Section>
  );
}
