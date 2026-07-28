"use client";

// [OPUS-5] sq-ixc3.24 — the VISUAL QUERY BUILDER: a diagram-to-SPARQL canvas.
//
// Competitor parity (epic sq-lsp7k): AllegroGraph **Gruff** is the industry reference for "draw
// the pattern, get the query"; Stardog **Explorer**'s model-driven builder adds walking
// relationship paths, attribute filters, AND-NOT and GROUP BY. This tool does both, and adds the
// thing neither has: **shape-aware** suggestions — when SHACL shapes are in the store, the
// predicate picker is driven by what the shapes DECLARE, not only by what the data happens to
// contain, and each suggestion says which of the two it came from.
//
// Honesty contract (the whole point of the tool):
//   * **No hidden dialect.** The generated SPARQL is always visible and always editable in the
//     pane below the canvas. It is plain SPARQL 1.1 — the exact text is what runs, and
//     "Open in Query" hands that same text to the real Query tool, which owns execution.
//   * **Manual edits are never silently discarded.** Editing the SPARQL detaches it from the
//     canvas; the pane says so and offers an explicit Regenerate. Nothing rewrites your text.
//   * **Nothing is fabricated.** Class + predicate pickers are populated by REAL SPARQL
//     introspection over the live store (see lib/query-builder.ts); an empty store yields an
//     empty picker and says so, and everything the lowering cannot express is surfaced as a
//     warning rather than dropped.
//
// The lowering, the introspection queries and the suggestion merge are pure and unit-tested
// (lib/query-builder.ts + its .test.ts). This file only draws + edits the model.
//
// Stable E2E hooks (gui/e2e-playwright/specs/query-builder.web.spec.ts):
//   [data-tool-panel="query-builder"]  — the panel root
//   [data-qb-canvas]                   — the SVG canvas
//   [data-qb-add-node]                 — "Add node"
//   [data-qb-node]                     — a node group (with data-qb-node-id)
//   #query-builder-sparql              — the generated-SPARQL textarea
//   [data-qb-warning]                  — one honest warning row
//   [data-qb-open-in-query]            — hand off to the Query tool

import * as React from "react";
import {
  ArrowRight,
  Copy,
  Link2,
  Loader2,
  Play,
  Plus,
  RefreshCw,
  Shapes,
  Trash2,
  Workflow,
  X,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { useWorkbench } from "@/components/workbench/workbench-context";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import { publishQuery } from "@/lib/query-handoff";
import {
  buildSparql,
  classesQuery,
  emptyModel,
  linkTargetsQuery,
  localName,
  mergeSuggestions,
  parseClassRows,
  parsePredicateRows,
  parseShapeRows,
  predicatesQuery,
  sanitizeVariable,
  shapesQuery,
  type Aggregate,
  type AttributeFilter,
  type BuilderEdge,
  type BuilderModel,
  type BuilderNode,
  type ClassStat,
  type EdgeMode,
  type FilterOp,
  type PredicateSuggestion,
  type ShapeProperty,
  type ValueKind,
} from "@/lib/query-builder";
import type { SparqlResults } from "@sparq/client";

export { QUERY_BUILDER_TOOL_OVERRIDE } from "@/components/workbench/query-builder-tool.meta";

// ---------------------------------------------------------------------------
// canvas geometry
// ---------------------------------------------------------------------------

const NODE_W = 172;
const NODE_H = 48;
/** Drag slop, in px: a pointer that moves less than this is a click (select), not a drag. */
const CLICK_SLOP = 4;

/** Where the segment towards (dx, dy) leaves a node's box — so arrows stop at the border. */
function boxExit(dx: number, dy: number): { x: number; y: number } {
  const hw = NODE_W / 2;
  const hh = NODE_H / 2;
  if (dx === 0 && dy === 0) return { x: 0, y: 0 };
  const scale = Math.min(hw / Math.abs(dx || Number.EPSILON), hh / Math.abs(dy || Number.EPSILON));
  return { x: dx * scale, y: dy * scale };
}

// ---------------------------------------------------------------------------
// small option tables (kept next to the UI that renders them)
// ---------------------------------------------------------------------------

const FILTER_OPS: { value: FilterOp; label: string }[] = [
  { value: "any", label: "has any value" },
  { value: "eq", label: "=" },
  { value: "ne", label: "≠" },
  { value: "lt", label: "<" },
  { value: "le", label: "≤" },
  { value: "gt", label: ">" },
  { value: "ge", label: "≥" },
  { value: "contains", label: "contains" },
  { value: "starts", label: "starts with" },
  { value: "ends", label: "ends with" },
  { value: "regex", label: "matches regex" },
  { value: "absent", label: "is absent" },
];

const AGGREGATES: { value: Aggregate | ""; label: string }[] = [
  { value: "", label: "group by (no aggregate)" },
  { value: "count", label: "COUNT" },
  { value: "count-distinct", label: "COUNT DISTINCT" },
  { value: "sum", label: "SUM" },
  { value: "avg", label: "AVG" },
  { value: "min", label: "MIN" },
  { value: "max", label: "MAX" },
];

const EDGE_MODES: { value: EdgeMode; label: string; hint: string }[] = [
  { value: "required", label: "required", hint: "A plain triple pattern — the link must exist." },
  { value: "optional", label: "optional", hint: "OPTIONAL { … } — keep rows where the link is missing." },
  { value: "not", label: "and-not", hint: "FILTER NOT EXISTS { … } — keep rows where the link does NOT exist." },
];

/** Suggestion → the flavour of affordance the picker offers. */
const isLinkish = (s: PredicateSuggestion) => s.kind === "link" || s.kind === "mixed";
const isAttributish = (s: PredicateSuggestion) =>
  s.kind === "attribute" || s.kind === "mixed" || s.kind === "unknown";

type Selection = { kind: "node"; id: string } | { kind: "edge"; id: string } | null;

interface SchemaState {
  kind: "idle" | "loading" | "ready" | "error";
  message?: string;
}

/** The in-panel preview of the generated query. Never a canned result — always a real run. */
type PreviewState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "rows"; results: SparqlResults; rowCount: number; latencyMs: number }
  | { kind: "message"; text: string; error: boolean };

/** The dotted canvas grid, as a CSS background (kept out of Tailwind's arbitrary-value parser). */
const GRID_BACKGROUND =
  "repeating-linear-gradient(0deg, transparent, transparent 23px, var(--border) 23px, var(--border) 24px)," +
  "repeating-linear-gradient(90deg, transparent, transparent 23px, var(--border) 23px, var(--border) 24px)";

// ---------------------------------------------------------------------------
// panel
// ---------------------------------------------------------------------------

export function QueryBuilderTool() {
  const { run, status } = useEngine();
  const workbench = useWorkbench();
  const ready = status.kind === "ready";

  const [model, setModel] = React.useState<BuilderModel>(emptyModel);
  const [selection, setSelection] = React.useState<Selection>(null);
  const [linkFrom, setLinkFrom] = React.useState<string | null>(null);

  // Live-store introspection. `classes` + `shapes` are dataset-wide; `suggestions` is memoised
  // per class IRI ("*" = the untyped-node profile) so re-selecting a node costs no queries.
  const [classes, setClasses] = React.useState<ClassStat[]>([]);
  const [shapes, setShapes] = React.useState<ShapeProperty[]>([]);
  const [suggestions, setSuggestions] = React.useState<Record<string, PredicateSuggestion[]>>({});
  const [schema, setSchema] = React.useState<SchemaState>({ kind: "idle" });

  // The SPARQL pane. `edited` is non-null once the user types into it — from then on the pane
  // shows THEIR text (never silently regenerated) until they press Regenerate.
  const [edited, setEdited] = React.useState<string | null>(null);
  const [preview, setPreview] = React.useState<PreviewState>({ kind: "idle" });

  const idRef = React.useRef(0);
  const nextId = React.useCallback((prefix: string) => `${prefix}${++idRef.current}`, []);

  const built = React.useMemo(() => buildSparql(model), [model]);
  const sparql = edited ?? built.sparql;

  // The SVG grows to hold whatever has been dragged where, so the canvas scrolls instead of
  // clipping a node the user pushed off the right edge.
  const canvasSize = React.useMemo(
    () => ({
      width: Math.max(640, ...model.nodes.map((n) => n.x + NODE_W + 64)),
      height: Math.max(320, ...model.nodes.map((n) => n.y + NODE_H + 64)),
    }),
    [model.nodes],
  );

  // Esc leaves link mode (the canvas hint promises this).
  React.useEffect(() => {
    if (!linkFrom) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setLinkFrom(null);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [linkFrom]);

  // --- engine helpers -----------------------------------------------------------------------

  /**
   * Run an introspection SELECT and hand back its bindings. An engine error is THROWN rather
   * than swallowed, so a failed introspection surfaces as "introspection failed: …" instead of
   * an empty picker that looks like an empty store.
   */
  const selectRows = React.useCallback(
    async (query: string): Promise<SparqlResults | null> => {
      const { outcome } = await run(query, { rowCap: 1_000 });
      if (outcome.kind === "error") throw new Error(outcome.message);
      return outcome.kind === "select" ? outcome.results : null;
    },
    [run],
  );

  const loadSchema = React.useCallback(async () => {
    setSchema({ kind: "loading" });
    try {
      const classRows = await selectRows(classesQuery());
      const shapeRows = await selectRows(shapesQuery());
      setClasses(classRows ? parseClassRows(classRows) : []);
      setShapes(shapeRows ? parseShapeRows(shapeRows) : []);
      setSuggestions({});
      setSchema({ kind: "ready" });
    } catch (err) {
      setSchema({ kind: "error", message: err instanceof Error ? err.message : String(err) });
    }
  }, [selectRows]);

  // Introspect once the engine is warm. A later import is picked up by the Refresh button (the
  // store has no change signal here, and silently re-querying on every render would be worse).
  const introspectedRef = React.useRef(false);
  React.useEffect(() => {
    if (!ready || introspectedRef.current) return;
    introspectedRef.current = true;
    void loadSchema();
  }, [ready, loadSchema]);

  const selectedNode =
    selection?.kind === "node" ? (model.nodes.find((n) => n.id === selection.id) ?? null) : null;
  const selectedEdge =
    selection?.kind === "edge" ? (model.edges.find((e) => e.id === selection.id) ?? null) : null;

  // Fetch the selected node's characteristic set + shape properties on demand.
  const suggestionKey = selectedNode ? (selectedNode.classIri ?? "*") : null;
  React.useEffect(() => {
    if (!ready || suggestionKey === null || suggestions[suggestionKey]) return;
    let cancelled = false;
    const classIri = suggestionKey === "*" ? null : suggestionKey;
    void (async () => {
      try {
        const predRows = await selectRows(predicatesQuery(classIri));
        const linkRows = await selectRows(linkTargetsQuery(classIri));
        if (cancelled) return;
        const stats = predRows ? parsePredicateRows(predRows, linkRows ?? undefined) : [];
        const declared = classIri ? shapes.filter((s) => s.targetClass === classIri) : [];
        setSuggestions((prev) => ({ ...prev, [suggestionKey]: mergeSuggestions(stats, declared) }));
      } catch (err) {
        if (cancelled) return;
        setSchema({ kind: "error", message: err instanceof Error ? err.message : String(err) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [ready, suggestionKey, suggestions, selectRows, shapes]);

  // --- model edits --------------------------------------------------------------------------

  const patchNode = React.useCallback((id: string, patch: Partial<BuilderNode>) => {
    setModel((m) => ({ ...m, nodes: m.nodes.map((n) => (n.id === id ? { ...n, ...patch } : n)) }));
  }, []);

  const patchEdge = React.useCallback((id: string, patch: Partial<BuilderEdge>) => {
    setModel((m) => ({ ...m, edges: m.edges.map((e) => (e.id === id ? { ...e, ...patch } : e)) }));
  }, []);

  /**
   * A variable name derived from a class (or "node"), made unique against the names already in
   * use. The lowering uniquifies again as a backstop, but doing it here keeps the name the user
   * sees on the canvas and the one in the query the same.
   */
  const freshVariable = React.useCallback((base: string, taken: readonly string[]): string => {
    const seed = sanitizeVariable(base.charAt(0).toLowerCase() + base.slice(1), "node");
    if (!taken.includes(seed)) return seed;
    let n = 2;
    while (taken.includes(`${seed}${n}`)) n++;
    return `${seed}${n}`;
  }, []);

  const addNode = React.useCallback(
    (classIri: string | null, at?: { x: number; y: number }) => {
      const id = nextId("n");
      setModel((m) => {
        const taken = m.nodes.map((n) => n.variable);
        const node: BuilderNode = {
          id,
          variable: freshVariable(classIri ? localName(classIri) : "node", taken),
          classIri,
          x: at?.x ?? 48 + (m.nodes.length % 3) * (NODE_W + 56),
          y: at?.y ?? 40 + Math.floor(m.nodes.length / 3) * (NODE_H + 64),
          project: true,
          aggregate: null,
          filters: [],
        };
        return { ...m, nodes: [...m.nodes, node] };
      });
      setSelection({ kind: "node", id });
      return id;
    },
    [freshVariable, nextId],
  );

  const removeNode = React.useCallback((id: string) => {
    setModel((m) => ({
      ...m,
      nodes: m.nodes.filter((n) => n.id !== id),
      edges: m.edges.filter((e) => e.from !== id && e.to !== id),
    }));
    setSelection(null);
  }, []);

  const addEdge = React.useCallback(
    (from: string, to: string, predicateIri: string, mode: EdgeMode = "required") => {
      const id = nextId("e");
      setModel((m) => ({ ...m, edges: [...m.edges, { id, from, to, predicateIri, mode }] }));
      return id;
    },
    [nextId],
  );

  /** Walk a relationship path: create the target node AND the link in one gesture. */
  const walkLink = React.useCallback(
    (fromId: string, suggestion: PredicateSuggestion) => {
      const source = model.nodes.find((n) => n.id === fromId);
      const targetClass = suggestion.objectClasses[0] ?? null;
      const id = addNode(targetClass, {
        x: (source?.x ?? 0) + NODE_W + 72,
        y: (source?.y ?? 0) + 24,
      });
      addEdge(fromId, id, suggestion.iri);
    },
    [addEdge, addNode, model.nodes],
  );

  const addFilter = React.useCallback(
    (nodeId: string, suggestion: PredicateSuggestion) => {
      const id = nextId("f");
      setModel((m) => ({
        ...m,
        nodes: m.nodes.map((n) => {
          if (n.id !== nodeId) return n;
          const taken = [...m.nodes.map((x) => x.variable), ...m.nodes.flatMap((x) => x.filters.map((f) => f.variable))];
          const filter: AttributeFilter = {
            id,
            predicateIri: suggestion.iri,
            variable: freshVariable(localName(suggestion.iri), taken),
            op: "any",
            value: "",
            valueKind: suggestion.datatype && /int|decimal|double|float|long|short|byte/i.test(suggestion.datatype)
              ? "number"
              : "text",
            project: true,
            aggregate: null,
          };
          return { ...n, filters: [...n.filters, filter] };
        }),
      }));
    },
    [freshVariable, nextId],
  );

  const patchFilter = React.useCallback((nodeId: string, filterId: string, patch: Partial<AttributeFilter>) => {
    setModel((m) => ({
      ...m,
      nodes: m.nodes.map((n) =>
        n.id === nodeId
          ? { ...n, filters: n.filters.map((f) => (f.id === filterId ? { ...f, ...patch } : f)) }
          : n,
      ),
    }));
  }, []);

  const removeFilter = React.useCallback((nodeId: string, filterId: string) => {
    setModel((m) => ({
      ...m,
      nodes: m.nodes.map((n) => (n.id === nodeId ? { ...n, filters: n.filters.filter((f) => f.id !== filterId) } : n)),
    }));
  }, []);

  // --- canvas dragging ----------------------------------------------------------------------

  const dragRef = React.useRef<{ id: string; dx: number; dy: number; moved: boolean } | null>(null);

  const onNodePointerDown = React.useCallback(
    (event: React.PointerEvent<SVGGElement>, node: BuilderNode) => {
      event.stopPropagation();
      event.currentTarget.setPointerCapture(event.pointerId);
      // Node coordinates live in the SVG's own space, which is 1:1 with CSS pixels (no viewBox),
      // so the SVG's client rect is all that is needed to convert — and it tracks the scroll.
      const box = event.currentTarget.ownerSVGElement?.getBoundingClientRect();
      dragRef.current = {
        id: node.id,
        dx: event.clientX - (box?.left ?? 0) - node.x,
        dy: event.clientY - (box?.top ?? 0) - node.y,
        moved: false,
      };
    },
    [],
  );

  const onNodePointerMove = React.useCallback(
    (event: React.PointerEvent<SVGGElement>) => {
      const drag = dragRef.current;
      if (!drag) return;
      const box = event.currentTarget.ownerSVGElement?.getBoundingClientRect();
      const x = event.clientX - (box?.left ?? 0) - drag.dx;
      const y = event.clientY - (box?.top ?? 0) - drag.dy;
      const node = model.nodes.find((n) => n.id === drag.id);
      if (!node) return;
      if (Math.abs(x - node.x) + Math.abs(y - node.y) > CLICK_SLOP) drag.moved = true;
      patchNode(drag.id, { x: Math.max(0, x), y: Math.max(0, y) });
    },
    [model.nodes, patchNode],
  );

  const onNodePointerUp = React.useCallback(
    (event: React.PointerEvent<SVGGElement>, node: BuilderNode) => {
      const drag = dragRef.current;
      dragRef.current = null;
      if (event.currentTarget.hasPointerCapture(event.pointerId)) {
        event.currentTarget.releasePointerCapture(event.pointerId);
      }
      if (drag?.moved) return; // a drag, not a click
      if (linkFrom && linkFrom !== node.id) {
        // Complete a manual link. Its predicate starts EMPTY, which the lowering refuses to
        // guess at: the edge is skipped with a warning and the inspector asks for a predicate,
        // rather than emitting `?a <> ?b` and pretending that meant something.
        const edgeId = addEdge(linkFrom, node.id, "");
        setLinkFrom(null);
        // Select the new LINK, not the node: its predicate is the one thing still missing.
        setSelection({ kind: "edge", id: edgeId });
        return;
      }
      setLinkFrom(null);
      setSelection({ kind: "node", id: node.id });
    },
    [addEdge, linkFrom],
  );

  // --- SPARQL pane actions ------------------------------------------------------------------

  const onCopy = React.useCallback(() => {
    void navigator.clipboard?.writeText(sparql);
  }, [sparql]);

  const onOpenInQuery = React.useCallback(() => {
    publishQuery(sparql);
    workbench?.openTool("query");
  }, [sparql, workbench]);

  const onPreview = React.useCallback(async () => {
    setPreview({ kind: "running" });
    const result = await run(sparql, { rowCap: 200 });
    const { outcome } = result;
    if (outcome.kind === "select") {
      setPreview({
        kind: "rows",
        results: outcome.results,
        rowCount: outcome.rowCount,
        latencyMs: result.latencyMs,
      });
    } else if (outcome.kind === "error") {
      setPreview({ kind: "message", text: outcome.message, error: true });
    } else if (outcome.kind === "cancelled") {
      setPreview({ kind: "message", text: "Cancelled.", error: false });
    } else {
      setPreview({
        kind: "message",
        text: `The builder generates SELECT queries; this text produced a ${outcome.kind} result. Run it in the Query tool for the full result views.`,
        error: false,
      });
    }
  }, [run, sparql]);

  // --- render -------------------------------------------------------------------------------

  const activeSuggestions = suggestionKey ? (suggestions[suggestionKey] ?? null) : null;

  return (
    <div className="flex h-full flex-col" data-tool-panel="query-builder">
      {/* Action row */}
      <div className="flex flex-wrap items-center gap-2 border-b bg-card px-3 py-1.5">
        <Workflow className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="text-xs font-medium text-muted-foreground">Query builder</span>
        <Badge variant="outline" className="h-5 gap-1 text-[10px]" title="Where introspection + preview run">
          LOCAL · in-tab WASM
        </Badge>
        {shapes.length > 0 ? (
          <Badge variant="outline" className="h-5 gap-1 text-[10px]" title="SHACL shapes found in the live store drive the pickers">
            <Shapes className="size-3" />
            shape-aware
          </Badge>
        ) : null}
        <div className="ml-auto flex items-center gap-1.5">
          <Button
            size="sm"
            variant="outline"
            onClick={() => void loadSchema()}
            disabled={!ready || schema.kind === "loading"}
            title="Re-introspect the live store (run this after an import)"
          >
            <RefreshCw className={cn("size-3.5", schema.kind === "loading" && "animate-spin")} />
            Refresh schema
          </Button>
          <Button size="sm" onClick={() => addNode(null)} disabled={!ready} data-qb-add-node>
            <Plus className="size-3.5" />
            Add node
          </Button>
        </div>
      </div>

      {/* Canvas + inspector */}
      <div className="flex min-h-0 flex-1">
        <div
          className="relative min-w-0 flex-1 overflow-auto"
          style={{ backgroundImage: GRID_BACKGROUND }}
          data-qb-canvas
          onPointerDown={() => {
            setSelection(null);
            setLinkFrom(null);
          }}
        >
          <svg
            className="block"
            role="presentation"
            width={canvasSize.width}
            height={canvasSize.height}
            style={{ minWidth: "100%", minHeight: "100%" }}
          >
            <defs>
              <marker id="qb-arrow" markerWidth="9" markerHeight="9" refX="8" refY="4.5" orient="auto">
                <path d="M0,0 L9,4.5 L0,9 z" fill="var(--muted-foreground)" />
              </marker>
            </defs>
            {model.edges.map((edge) => (
              <EdgeShape
                key={edge.id}
                edge={edge}
                nodes={model.nodes}
                selected={selection?.kind === "edge" && selection.id === edge.id}
                onSelect={() => setSelection({ kind: "edge", id: edge.id })}
              />
            ))}
            {model.nodes.map((node) => (
              <NodeShape
                key={node.id}
                node={node}
                selected={selection?.kind === "node" && selection.id === node.id}
                linkSource={linkFrom === node.id}
                linkTargetable={linkFrom !== null && linkFrom !== node.id}
                onPointerDown={(e) => onNodePointerDown(e, node)}
                onPointerMove={onNodePointerMove}
                onPointerUp={(e) => onNodePointerUp(e, node)}
              />
            ))}
          </svg>

          {model.nodes.length === 0 ? (
            <div className="pointer-events-none absolute inset-0 flex items-center justify-center p-6 text-center">
              <p className="max-w-md text-sm text-muted-foreground">
                {ready
                  ? "Add a node to start drawing a pattern. Pick its class, then walk its links and attributes from the picker — the SPARQL below is generated as you go, and stays editable."
                  : "Waiting for the engine to warm…"}
              </p>
            </div>
          ) : null}
          {linkFrom ? (
            <div className="pointer-events-none absolute inset-x-0 top-2 flex justify-center">
              <span className="rounded-md border bg-card px-2 py-1 text-[11px] text-muted-foreground shadow-sm">
                Click another node to link to it — Esc or a click on empty canvas cancels.
              </span>
            </div>
          ) : null}
        </div>

        <aside className="flex w-80 shrink-0 flex-col overflow-auto border-l bg-card/40" data-qb-inspector>
          {selectedNode ? (
            <NodeInspector
              node={selectedNode}
              classes={classes}
              suggestions={activeSuggestions}
              schema={schema}
              linking={linkFrom === selectedNode.id}
              onToggleLink={() => setLinkFrom((prev) => (prev === selectedNode.id ? null : selectedNode.id))}
              onPatch={(patch) => patchNode(selectedNode.id, patch)}
              onRemove={() => removeNode(selectedNode.id)}
              onWalk={(s) => walkLink(selectedNode.id, s)}
              onAddFilter={(s) => addFilter(selectedNode.id, s)}
              onPatchFilter={(fid, patch) => patchFilter(selectedNode.id, fid, patch)}
              onRemoveFilter={(fid) => removeFilter(selectedNode.id, fid)}
            />
          ) : selectedEdge ? (
            <EdgeInspector
              edge={selectedEdge}
              nodes={model.nodes}
              onPatch={(patch) => patchEdge(selectedEdge.id, patch)}
              onRemove={() => {
                setModel((m) => ({ ...m, edges: m.edges.filter((e) => e.id !== selectedEdge.id) }));
                setSelection(null);
              }}
            />
          ) : (
            <ResultShapePanel
              model={model}
              outputs={built.projected}
              classes={classes}
              shapes={shapes}
              schema={schema}
              onPatch={(patch) => setModel((m) => ({ ...m, ...patch }))}
              onAddNodeOfClass={(iri) => addNode(iri)}
            />
          )}
        </aside>
      </div>

      {/* Generated SPARQL — always visible, always editable */}
      <div className="flex h-64 shrink-0 flex-col border-t">
        <div className="flex flex-wrap items-center gap-1.5 border-b bg-card px-3 py-1.5">
          <span className="text-[11px] font-medium text-muted-foreground">
            Generated SPARQL
            {edited !== null ? <span className="ml-1 text-[var(--warning)]">· manually edited</span> : null}
          </span>
          {edited !== null ? (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setEdited(null)}
              title="Discard your edits and regenerate from the canvas"
            >
              <RefreshCw className="size-3.5" />
              Regenerate
            </Button>
          ) : null}
          <div className="ml-auto flex items-center gap-1.5">
            <Button size="sm" variant="ghost" onClick={onCopy} title="Copy the query text">
              <Copy className="size-3.5" />
              Copy
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => void onPreview()}
              disabled={!ready || preview.kind === "running"}
              title="Run this query over the live store and preview the rows"
            >
              {preview.kind === "running" ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />}
              Preview
            </Button>
            <Button size="sm" onClick={onOpenInQuery} data-qb-open-in-query title="Send this exact text to the Query tool">
              <ArrowRight className="size-3.5" />
              Open in Query
            </Button>
          </div>
        </div>

        {edited === null && built.warnings.length > 0 ? (
          <ul className="max-h-20 shrink-0 overflow-auto border-b bg-[var(--warning)]/5 px-3 py-1 text-[11px] text-muted-foreground">
            {built.warnings.map((warning, i) => (
              <li key={i} data-qb-warning>
                ⚠ {warning}
              </li>
            ))}
          </ul>
        ) : null}

        <div className="flex min-h-0 flex-1">
          <div className="flex min-w-0 flex-1 flex-col">
            <WorkbenchSparqlEditor
              id="query-builder-sparql"
              value={sparql}
              onChange={(value) => setEdited(value)}
              ariaLabel="Generated SPARQL (editable)"
            />
          </div>
          {preview.kind !== "idle" ? (
            <div className="w-96 shrink-0 overflow-auto border-l" data-qb-preview>
              <PreviewPanel preview={preview} onClose={() => setPreview({ kind: "idle" })} />
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// canvas pieces
// ---------------------------------------------------------------------------

function NodeShape({
  node,
  selected,
  linkSource,
  linkTargetable,
  onPointerDown,
  onPointerMove,
  onPointerUp,
}: {
  node: BuilderNode;
  selected: boolean;
  linkSource: boolean;
  linkTargetable: boolean;
  onPointerDown: (e: React.PointerEvent<SVGGElement>) => void;
  onPointerMove: (e: React.PointerEvent<SVGGElement>) => void;
  onPointerUp: (e: React.PointerEvent<SVGGElement>) => void;
}) {
  const stroke = selected || linkSource ? "var(--primary)" : linkTargetable ? "var(--success)" : "var(--border)";
  return (
    <g
      transform={`translate(${node.x},${node.y})`}
      className="cursor-grab active:cursor-grabbing"
      data-qb-node
      data-qb-node-id={node.id}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    >
      <rect
        width={NODE_W}
        height={NODE_H}
        rx={8}
        fill="var(--card)"
        stroke={stroke}
        strokeWidth={selected || linkSource ? 2 : 1.25}
      />
      <text x={10} y={19} fontSize={12} fontWeight={600} fill="var(--foreground)">
        ?{node.variable}
      </text>
      <text x={10} y={35} fontSize={10.5} fill="var(--muted-foreground)">
        {node.classIri ? localName(node.classIri) : "any node"}
        {node.filters.length > 0 ? ` · ${node.filters.length} attr` : ""}
      </text>
      {node.project ? (
        <circle cx={NODE_W - 11} cy={12} r={3.5} fill="var(--success)">
          <title>Projected — appears in the SELECT</title>
        </circle>
      ) : null}
    </g>
  );
}

function EdgeShape({
  edge,
  nodes,
  selected,
  onSelect,
}: {
  edge: BuilderEdge;
  nodes: BuilderNode[];
  selected: boolean;
  onSelect: () => void;
}) {
  const from = nodes.find((n) => n.id === edge.from);
  const to = nodes.find((n) => n.id === edge.to);
  if (!from || !to) return null;
  const fx = from.x + NODE_W / 2;
  const fy = from.y + NODE_H / 2;
  const tx = to.x + NODE_W / 2;
  const ty = to.y + NODE_H / 2;
  const exit = boxExit(tx - fx, ty - fy);
  const entry = boxExit(fx - tx, fy - ty);
  const x1 = fx + exit.x;
  const y1 = fy + exit.y;
  const x2 = tx + entry.x;
  const y2 = ty + entry.y;
  const label = edge.predicateIri ? localName(edge.predicateIri) : "choose a predicate";
  const stroke = selected
    ? "var(--primary)"
    : edge.mode === "not"
      ? "var(--destructive)"
      : "var(--muted-foreground)";

  return (
    <g className="cursor-pointer" onPointerDown={(e) => e.stopPropagation()} onClick={onSelect} data-qb-edge>
      <line
        x1={x1}
        y1={y1}
        x2={x2}
        y2={y2}
        stroke={stroke}
        strokeWidth={selected ? 2 : 1.25}
        strokeDasharray={edge.mode === "required" ? undefined : "5 3"}
        markerEnd="url(#qb-arrow)"
      />
      {/* A wide invisible hit line so a 1px edge is still clickable. */}
      <line x1={x1} y1={y1} x2={x2} y2={y2} stroke="transparent" strokeWidth={12} />
      <rect
        x={(x1 + x2) / 2 - (label.length * 3.1 + 8)}
        y={(y1 + y2) / 2 - 9}
        width={label.length * 6.2 + 16}
        height={16}
        rx={4}
        fill="var(--background)"
        stroke={selected ? "var(--primary)" : "var(--border)"}
        strokeWidth={0.75}
      />
      <text
        x={(x1 + x2) / 2}
        y={(y1 + y2) / 2 + 2.5}
        textAnchor="middle"
        fontSize={10}
        fill={edge.predicateIri ? "var(--foreground)" : "var(--warning)"}
      >
        {edge.mode === "optional" ? "? " : edge.mode === "not" ? "¬ " : ""}
        {label}
      </text>
    </g>
  );
}

// ---------------------------------------------------------------------------
// inspectors
// ---------------------------------------------------------------------------

const fieldClass =
  "w-full rounded-md border bg-background px-2 py-1 text-xs outline-none focus:ring-1 focus:ring-primary";
const sectionClass = "border-b px-3 py-2";
const labelClass = "mb-1 block text-[10px] font-medium uppercase tracking-wide text-muted-foreground";

function NodeInspector({
  node,
  classes,
  suggestions,
  schema,
  linking,
  onToggleLink,
  onPatch,
  onRemove,
  onWalk,
  onAddFilter,
  onPatchFilter,
  onRemoveFilter,
}: {
  node: BuilderNode;
  classes: ClassStat[];
  suggestions: PredicateSuggestion[] | null;
  schema: SchemaState;
  linking: boolean;
  onToggleLink: () => void;
  onPatch: (patch: Partial<BuilderNode>) => void;
  onRemove: () => void;
  onWalk: (s: PredicateSuggestion) => void;
  onAddFilter: (s: PredicateSuggestion) => void;
  onPatchFilter: (filterId: string, patch: Partial<AttributeFilter>) => void;
  onRemoveFilter: (filterId: string) => void;
}) {
  return (
    <>
      <div className={cn(sectionClass, "flex items-center gap-2")}>
        <span className="truncate text-xs font-medium">Node ?{node.variable}</span>
        <div className="ml-auto flex gap-1">
          <Button
            size="icon-sm"
            variant={linking ? "default" : "ghost"}
            onClick={onToggleLink}
            title="Link this node to another existing node"
          >
            <Link2 className="size-3.5" />
          </Button>
          <Button size="icon-sm" variant="ghost" onClick={onRemove} title="Delete this node">
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      </div>

      <div className={sectionClass}>
        <label className={labelClass} htmlFor={`qb-var-${node.id}`}>
          Variable
        </label>
        <input
          id={`qb-var-${node.id}`}
          className={fieldClass}
          value={node.variable}
          onChange={(e) => onPatch({ variable: e.target.value })}
        />
        <label className={cn(labelClass, "mt-2")} htmlFor={`qb-class-${node.id}`}>
          Class
        </label>
        <select
          id={`qb-class-${node.id}`}
          className={fieldClass}
          value={node.classIri ?? ""}
          onChange={(e) => onPatch({ classIri: e.target.value || null })}
        >
          <option value="">(any — no rdf:type constraint)</option>
          {classes.map((c) => (
            <option key={c.iri} value={c.iri}>
              {localName(c.iri)} · {c.instances.toLocaleString()}
            </option>
          ))}
          {/* A class chosen from a shape may have no instances yet — keep it selectable. */}
          {node.classIri && !classes.some((c) => c.iri === node.classIri) ? (
            <option value={node.classIri}>{localName(node.classIri)} (not in the data)</option>
          ) : null}
        </select>
        <label className="mt-2 flex items-center gap-1.5 text-xs">
          <input type="checkbox" checked={node.project} onChange={(e) => onPatch({ project: e.target.checked })} />
          Select this node
        </label>
        {node.project ? (
          <select
            className={cn(fieldClass, "mt-1.5")}
            value={node.aggregate ?? ""}
            onChange={(e) => onPatch({ aggregate: (e.target.value || null) as Aggregate | null })}
            aria-label="Aggregate for this node"
          >
            {AGGREGATES.map((a) => (
              <option key={a.value} value={a.value}>
                {a.label}
              </option>
            ))}
          </select>
        ) : null}
      </div>

      {node.filters.length > 0 ? (
        <div className={sectionClass}>
          <span className={labelClass}>Attribute filters</span>
          <ul className="flex flex-col gap-2">
            {node.filters.map((filter) => (
              <li key={filter.id} className="rounded-md border bg-background p-2">
                <div className="flex items-center gap-1">
                  <span className="truncate text-[11px] font-medium" title={filter.predicateIri}>
                    {localName(filter.predicateIri)}
                  </span>
                  <Button
                    size="icon-sm"
                    variant="ghost"
                    className="ml-auto"
                    onClick={() => onRemoveFilter(filter.id)}
                    title="Remove this filter"
                  >
                    <X className="size-3" />
                  </Button>
                </div>
                <input
                  className={cn(fieldClass, "mt-1")}
                  value={filter.variable}
                  onChange={(e) => onPatchFilter(filter.id, { variable: e.target.value })}
                  aria-label="Value variable"
                />
                <div className="mt-1 flex gap-1">
                  <select
                    className={fieldClass}
                    value={filter.op}
                    onChange={(e) => onPatchFilter(filter.id, { op: e.target.value as FilterOp })}
                    aria-label="Comparison"
                  >
                    {FILTER_OPS.map((op) => (
                      <option key={op.value} value={op.value}>
                        {op.label}
                      </option>
                    ))}
                  </select>
                  {filter.op !== "any" && filter.op !== "absent" ? (
                    <select
                      className={cn(fieldClass, "w-24 shrink-0")}
                      value={filter.valueKind}
                      onChange={(e) => onPatchFilter(filter.id, { valueKind: e.target.value as ValueKind })}
                      aria-label="Value kind"
                    >
                      <option value="text">text</option>
                      <option value="number">number</option>
                      <option value="iri">IRI</option>
                    </select>
                  ) : null}
                </div>
                {filter.op !== "any" && filter.op !== "absent" ? (
                  <input
                    className={cn(fieldClass, "mt-1")}
                    value={filter.value}
                    placeholder="value"
                    onChange={(e) => onPatchFilter(filter.id, { value: e.target.value })}
                    aria-label="Value"
                  />
                ) : null}
                <label className="mt-1 flex items-center gap-1.5 text-[11px]">
                  <input
                    type="checkbox"
                    checked={filter.project}
                    disabled={filter.op === "absent"}
                    onChange={(e) => onPatchFilter(filter.id, { project: e.target.checked })}
                  />
                  Select the value
                </label>
                {filter.project && filter.op !== "absent" ? (
                  <select
                    className={cn(fieldClass, "mt-1")}
                    value={filter.aggregate ?? ""}
                    onChange={(e) =>
                      onPatchFilter(filter.id, { aggregate: (e.target.value || null) as Aggregate | null })
                    }
                    aria-label="Aggregate"
                  >
                    {AGGREGATES.map((a) => (
                      <option key={a.value} value={a.value}>
                        {a.label}
                      </option>
                    ))}
                  </select>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      <div className="min-h-0 flex-1 px-3 py-2">
        <span className={labelClass}>
          Suggestions{node.classIri ? ` for ${localName(node.classIri)}` : ""}
        </span>
        {schema.kind === "error" ? (
          <p className="text-[11px] text-destructive">Introspection failed: {schema.message}</p>
        ) : suggestions === null ? (
          <p className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
            <Loader2 className="size-3 animate-spin" /> Reading the store…
          </p>
        ) : suggestions.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">
            No predicates found for this node in the live store, and no SHACL shape in the store
            declares any. Import data or shapes, then Refresh schema.
          </p>
        ) : (
          <ul className="flex flex-col gap-1" data-qb-suggestions>
            {suggestions.map((suggestion) => (
              <li
                key={suggestion.iri}
                className="rounded-md border bg-background p-1.5"
                data-qb-suggestion={suggestion.iri}
              >
                <div className="flex items-center gap-1">
                  <span className="truncate text-[11px] font-medium" title={suggestion.iri}>
                    {suggestion.label}
                  </span>
                  {suggestion.required ? (
                    <span className="shrink-0 text-[9px] text-[var(--warning)]" title="sh:minCount ≥ 1">
                      required
                    </span>
                  ) : null}
                </div>
                <div className="mt-0.5 flex items-center gap-1 text-[10px] text-muted-foreground">
                  <span title={SOURCE_HINT[suggestion.source]}>{SOURCE_LABEL[suggestion.source]}</span>
                  <span>·</span>
                  <span>{suggestion.uses === null ? "not used yet" : `${suggestion.uses.toLocaleString()} uses`}</span>
                  <div className="ml-auto flex gap-1">
                    {isLinkish(suggestion) ? (
                      <Button size="sm" variant="ghost" className="h-5 px-1" onClick={() => onWalk(suggestion)}>
                        link
                      </Button>
                    ) : null}
                    {isAttributish(suggestion) ? (
                      <Button size="sm" variant="ghost" className="h-5 px-1" onClick={() => onAddFilter(suggestion)}>
                        filter
                      </Button>
                    ) : null}
                  </div>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </>
  );
}

/** How a suggestion's provenance is shown — never implying data that is not there. */
const SOURCE_LABEL: Record<PredicateSuggestion["source"], string> = {
  shape: "SHACL shape",
  data: "in the data",
  both: "shape + data",
};
const SOURCE_HINT: Record<PredicateSuggestion["source"], string> = {
  shape: "Declared by a SHACL shape in the store; no instance uses it yet.",
  data: "Observed on instances of this class in the live store.",
  both: "Declared by a SHACL shape AND observed in the live store.",
};

function EdgeInspector({
  edge,
  nodes,
  onPatch,
  onRemove,
}: {
  edge: BuilderEdge;
  nodes: BuilderNode[];
  onPatch: (patch: Partial<BuilderEdge>) => void;
  onRemove: () => void;
}) {
  const from = nodes.find((n) => n.id === edge.from);
  const to = nodes.find((n) => n.id === edge.to);
  return (
    <>
      <div className={cn(sectionClass, "flex items-center gap-2")}>
        <span className="truncate text-xs font-medium">
          ?{from?.variable} → ?{to?.variable}
        </span>
        <Button size="icon-sm" variant="ghost" className="ml-auto" onClick={onRemove} title="Delete this link">
          <Trash2 className="size-3.5" />
        </Button>
      </div>
      <div className={sectionClass}>
        <label className={labelClass} htmlFor={`qb-pred-${edge.id}`}>
          Predicate IRI
        </label>
        <input
          id={`qb-pred-${edge.id}`}
          className={fieldClass}
          value={edge.predicateIri}
          placeholder="http://…"
          onChange={(e) => onPatch({ predicateIri: e.target.value })}
        />
        {edge.predicateIri.trim().length === 0 ? (
          <p className="mt-1 text-[10px] text-[var(--warning)]">
            This link has no predicate yet — pick one, or delete the link.
          </p>
        ) : null}
        <label className={cn(labelClass, "mt-2")} htmlFor={`qb-mode-${edge.id}`}>
          Join
        </label>
        <select
          id={`qb-mode-${edge.id}`}
          className={fieldClass}
          value={edge.mode}
          onChange={(e) => onPatch({ mode: e.target.value as EdgeMode })}
        >
          {EDGE_MODES.map((m) => (
            <option key={m.value} value={m.value}>
              {m.label}
            </option>
          ))}
        </select>
        <p className="mt-1 text-[10px] text-muted-foreground">
          {EDGE_MODES.find((m) => m.value === edge.mode)?.hint}
        </p>
        <Button
          size="sm"
          variant="outline"
          className="mt-2"
          onClick={() => onPatch({ from: edge.to, to: edge.from })}
        >
          Reverse direction
        </Button>
      </div>
    </>
  );
}

function ResultShapePanel({
  model,
  outputs,
  classes,
  shapes,
  schema,
  onPatch,
  onAddNodeOfClass,
}: {
  model: BuilderModel;
  outputs: string[];
  classes: ClassStat[];
  shapes: ShapeProperty[];
  schema: SchemaState;
  onPatch: (patch: Partial<BuilderModel>) => void;
  onAddNodeOfClass: (iri: string) => void;
}) {
  return (
    <>
      <div className={cn(sectionClass, "text-xs font-medium")}>Result shape</div>
      <div className={sectionClass}>
        <label className="flex items-center gap-1.5 text-xs">
          <input
            type="checkbox"
            checked={model.distinct}
            onChange={(e) => onPatch({ distinct: e.target.checked })}
          />
          DISTINCT
        </label>
        <label className={cn(labelClass, "mt-2")} htmlFor="qb-order">
          Order by
        </label>
        <div className="flex gap-1">
          <select
            id="qb-order"
            className={fieldClass}
            value={model.orderBy?.variable ?? ""}
            onChange={(e) =>
              onPatch({ orderBy: e.target.value ? { variable: e.target.value, desc: model.orderBy?.desc ?? true } : null })
            }
          >
            <option value="">(none)</option>
            {outputs.map((output) => (
              <option key={output} value={output}>
                ?{output}
              </option>
            ))}
          </select>
          <select
            className={cn(fieldClass, "w-20 shrink-0")}
            value={model.orderBy?.desc ? "desc" : "asc"}
            disabled={!model.orderBy}
            onChange={(e) =>
              onPatch(
                model.orderBy ? { orderBy: { ...model.orderBy, desc: e.target.value === "desc" } } : {},
              )
            }
            aria-label="Order direction"
          >
            <option value="asc">asc</option>
            <option value="desc">desc</option>
          </select>
        </div>
        <label className={cn(labelClass, "mt-2")} htmlFor="qb-limit">
          Limit
        </label>
        <input
          id="qb-limit"
          className={fieldClass}
          type="number"
          min={1}
          value={model.limit ?? ""}
          placeholder="(no limit)"
          onChange={(e) => onPatch({ limit: e.target.value ? Number(e.target.value) : null })}
        />
      </div>
      <div className="min-h-0 flex-1 px-3 py-2">
        <span className={labelClass}>Classes in the live store</span>
        {schema.kind === "loading" ? (
          <p className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
            <Loader2 className="size-3 animate-spin" /> Introspecting…
          </p>
        ) : schema.kind === "error" ? (
          <p className="text-[11px] text-destructive">Introspection failed: {schema.message}</p>
        ) : classes.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">
            No <code>rdf:type</code> statements in the store yet. Import data (or add an untyped
            node and type the predicates yourself), then Refresh schema.
          </p>
        ) : (
          <ul className="flex flex-col gap-1">
            {classes.map((c) => (
              <li key={c.iri}>
                <button
                  type="button"
                  className="flex w-full items-center gap-1 rounded-md border bg-background px-2 py-1 text-left text-[11px] hover:bg-accent"
                  onClick={() => onAddNodeOfClass(c.iri)}
                  title={c.iri}
                >
                  <span className="truncate">{localName(c.iri)}</span>
                  <span className="ml-auto shrink-0 text-muted-foreground">
                    {c.instances.toLocaleString()}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <p className="mt-3 text-[10px] leading-relaxed text-muted-foreground">
          {shapes.length > 0
            ? `${shapes.length} SHACL property constraint${shapes.length === 1 ? "" : "s"} found in the store — they rank and label the predicate suggestions.`
            : "No SHACL shapes in the store, so suggestions come from the data alone. Shapes pasted into the SHACL tool are that tool's session state; import them to make them drive suggestions."}
        </p>
      </div>
    </>
  );
}

function PreviewPanel({ preview, onClose }: { preview: PreviewState; onClose: () => void }) {
  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b bg-card px-2 py-1 text-[11px] text-muted-foreground">
        <span>Preview</span>
        {preview.kind === "rows" ? (
          <span className="tabular">
            {preview.rowCount.toLocaleString()} row{preview.rowCount === 1 ? "" : "s"} ·{" "}
            {preview.latencyMs.toFixed(1)} ms measured
          </span>
        ) : null}
        <Button size="icon-sm" variant="ghost" className="ml-auto" onClick={onClose} title="Close the preview">
          <X className="size-3" />
        </Button>
      </div>
      <div className="min-h-0 flex-1 overflow-auto">
        {preview.kind === "running" ? (
          <p className="p-2 text-[11px] text-muted-foreground">Running…</p>
        ) : preview.kind === "message" ? (
          <pre
            className={cn(
              "whitespace-pre-wrap p-2 font-mono text-[11px]",
              preview.error ? "text-destructive" : "text-muted-foreground",
            )}
          >
            {preview.text}
          </pre>
        ) : preview.kind === "rows" ? (
          <PreviewTable results={preview.results} />
        ) : null}
      </div>
    </div>
  );
}

/** The kept rows of a preview run, exactly as the engine returned them. */
function PreviewTable({ results }: { results: SparqlResults }) {
  const vars = results.head.vars;
  return (
    <table className="w-full border-collapse text-[11px]">
      <thead className="sticky top-0 bg-card">
        <tr>
          {vars.map((v) => (
            <th key={v} className="border-b px-2 py-1 text-left font-medium">
              ?{v}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {results.results.bindings.map((row, i) => (
          <tr key={i} className="odd:bg-muted/30">
            {vars.map((v) => (
              <td key={v} className="max-w-40 truncate border-b px-2 py-0.5" title={row[v]?.value ?? ""}>
                {row[v]?.value ?? ""}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
