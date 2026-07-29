"use client";

// [OPUS-5] sq-ixc3.24 — the visual query builder's CANVAS: node cards, predicate edges, drag and
// click-to-link. Split out of query-builder-tool.tsx the way the graph view and plan tree are
// (graph-view.tsx / plan-tree.tsx), so the panel file stays reviewable.
//
// Pure presentation over the `BuilderModel`: it renders the diagram and reports intent upward.
// It holds NO model state and does no SPARQL — the serializer lives in `@/lib/query-builder`.

import * as React from "react";
import { Plus, Spline, Trash2, Workflow } from "lucide-react";

import { Button } from "@/components/ui/button";
import { iriLabel, negatedNodeIds, type BuilderEdge, type BuilderModel, type BuilderNode } from "@/lib/query-builder";

/** Node-card geometry (px). Edges are drawn between card centres. */
const NODE_W = 216;
const NODE_H = 96;

/** What the canvas has selected — the inspector column follows this. */
export type CanvasSelection = { kind: "node" | "edge"; id: string } | null;

export function QueryBuilderCanvas({
  model,
  selected,
  linkFrom,
  onSelect,
  onMove,
  onStartLink,
  onCompleteLink,
  onDelete,
  onAddNode,
}: {
  model: BuilderModel;
  selected: CanvasSelection;
  linkFrom: string | null;
  onSelect: (s: CanvasSelection) => void;
  onMove: (id: string, x: number, y: number) => void;
  onStartLink: (id: string | null) => void;
  onCompleteLink: (id: string) => void;
  onDelete: (id: string) => void;
  onAddNode: () => void;
}) {
  const byId = React.useMemo(() => new Map(model.nodes.map((n) => [n.id, n])), [model.nodes]);
  const negated = React.useMemo(() => negatedNodeIds(model), [model]);
  const dragRef = React.useRef<{ id: string; dx: number; dy: number } | null>(null);

  const onPointerDown = (e: React.PointerEvent, node: BuilderNode) => {
    if (linkFrom) return; // in link mode a press is a link target, not a drag
    dragRef.current = { id: node.id, dx: e.clientX - node.x, dy: e.clientY - node.y };
    e.currentTarget.setPointerCapture(e.pointerId);
  };
  const onPointerMove = (e: React.PointerEvent) => {
    const drag = dragRef.current;
    if (!drag) return;
    onMove(drag.id, Math.max(0, e.clientX - drag.dx), Math.max(0, e.clientY - drag.dy));
  };
  const onPointerUp = () => {
    dragRef.current = null;
  };

  return (
    <div
      className="relative min-w-0 flex-1 overflow-auto bg-[color-mix(in_oklch,var(--muted)_35%,transparent)]"
      data-builder-canvas
      onClick={() => {
        onSelect(null);
        onStartLink(null);
      }}
    >
      {model.nodes.length === 0 ? (
        <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
          <Workflow className="size-8 text-muted-foreground" aria-hidden />
          <p className="max-w-sm text-sm text-muted-foreground">
            Draw a pattern: add a node, give it a class, hang attributes off it, and link nodes with
            predicates. The SPARQL below is generated as you go — and stays editable.
          </p>
          <Button size="sm" onClick={(e) => { e.stopPropagation(); onAddNode(); }} data-builder-empty-add>
            <Plus className="size-3.5" /> Add a node
          </Button>
        </div>
      ) : null}

      {/* Edge layer. Straight lines between card centres — a diagram, not a claim about layout. */}
      <svg className="pointer-events-none absolute inset-0 size-full" aria-hidden>
        {model.edges.map((edge) => {
          const from = byId.get(edge.from);
          const to = byId.get(edge.to);
          if (!from || !to) return null;
          const x1 = from.x + NODE_W / 2;
          const y1 = from.y + NODE_H / 2;
          const x2 = to.x + NODE_W / 2;
          const y2 = to.y + NODE_H / 2;
          const active = selected?.kind === "edge" && selected.id === edge.id;
          return (
            // The caption is the clickable hit-target below, not a second SVG label.
            <line
              key={edge.id}
              x1={x1}
              y1={y1}
              x2={x2}
              y2={y2}
              stroke={active ? "var(--primary)" : "var(--muted-foreground)"}
              strokeWidth={active ? 2 : 1.25}
              strokeDasharray={edge.mode === "required" ? undefined : "5 4"}
            />
          );
        })}
      </svg>

      {/* Clickable edge hit-targets, on top of the SVG (which is pointer-events-none). */}
      {model.edges.map((edge) => {
        const from = byId.get(edge.from);
        const to = byId.get(edge.to);
        if (!from || !to) return null;
        return (
          <button
            key={`hit-${edge.id}`}
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onSelect({ kind: "edge", id: edge.id });
            }}
            className="absolute -translate-x-1/2 -translate-y-1/2 rounded-sm border bg-background/90 px-1 text-[10px] text-muted-foreground hover:text-foreground"
            style={{
              left: (from.x + to.x) / 2 + NODE_W / 2,
              top: (from.y + to.y) / 2 + NODE_H / 2 + 10,
            }}
            data-builder-edge={edge.id}
          >
            {edgeCaption(edge)}
          </button>
        );
      })}

      {model.nodes.map((node) => {
        const isSelected = selected?.kind === "node" && selected.id === node.id;
        const inNegation = negated.has(node.id);
        return (
          <div
            key={node.id}
            className={`absolute rounded-md border bg-card p-2 shadow-sm ${
              isSelected ? "border-primary ring-1 ring-primary" : "border-border"
            } ${linkFrom && linkFrom !== node.id ? "cursor-crosshair" : "cursor-move"}`}
            style={{ left: node.x, top: node.y, width: NODE_W, minHeight: NODE_H }}
            data-builder-node={node.variable}
            onPointerDown={(e) => onPointerDown(e, node)}
            onPointerMove={onPointerMove}
            onPointerUp={onPointerUp}
            onClick={(e) => {
              e.stopPropagation();
              if (linkFrom) onCompleteLink(node.id);
              else onSelect({ kind: "node", id: node.id });
            }}
          >
            <div className="flex items-center gap-1">
              <span className="min-w-0 flex-1 truncate font-mono text-xs font-medium">
                ?{node.variable}
              </span>
              <button
                type="button"
                title="Link this node to another with a predicate"
                aria-label={`Link ?${node.variable} to another node`}
                onClick={(e) => {
                  e.stopPropagation();
                  onStartLink(linkFrom === node.id ? null : node.id);
                }}
                className={`rounded-sm p-0.5 ${linkFrom === node.id ? "text-primary" : "text-muted-foreground hover:text-foreground"}`}
                data-builder-link={node.variable}
              >
                <Spline className="size-3.5" />
              </button>
              <button
                type="button"
                title="Remove this node"
                aria-label={`Remove ?${node.variable}`}
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(node.id);
                }}
                className="rounded-sm p-0.5 text-muted-foreground hover:text-destructive"
              >
                <Trash2 className="size-3.5" />
              </button>
            </div>
            <p className="truncate text-[11px] text-muted-foreground" title={node.classIri ?? undefined}>
              {node.classIri ? iriLabel(node.classIri) : "any type"}
            </p>
            {inNegation && (
              <p className="text-[10px] text-[var(--warning)]">inside NOT EXISTS — not projectable</p>
            )}
            <ul className="mt-1 space-y-0.5">
              {node.attributes.slice(0, 4).map((a) => (
                <li key={a.id} className="truncate font-mono text-[10px] text-muted-foreground">
                  {a.optional ? "?" : ""}
                  {iriLabel(a.predicate)} → ?{a.variable}
                  {a.filter ? " ⧗" : ""}
                </li>
              ))}
              {node.attributes.length > 4 && (
                <li className="text-[10px] text-muted-foreground">
                  +{node.attributes.length - 4} more
                </li>
              )}
            </ul>
          </div>
        );
      })}

      {linkFrom && (
        <div className="pointer-events-none absolute inset-x-0 top-2 flex justify-center">
          <span className="rounded-full border bg-popover px-3 py-1 text-[11px] shadow">
            Click another node to link it — or press the link button again to cancel.
          </span>
        </div>
      )}
    </div>
  );
}

/** The short caption an edge shows on the canvas. */
function edgeCaption(edge: BuilderEdge): string {
  const base = edge.predicate.trim() === "" ? "(no predicate)" : iriLabel(edge.predicate);
  const alt = edge.alternatives.length > 0 ? ` +${edge.alternatives.length}` : "";
  const mode = edge.mode === "optional" ? " (optional)" : edge.mode === "not-exists" ? " (not exists)" : "";
  return `${base}${alt}${mode}`;
}
