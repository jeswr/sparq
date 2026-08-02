"use client";

// [OPUS-4.8] sq-ixc3.12 — the GRAPH result view: a dependency-free SVG node-link visualisation
// of a CONSTRUCT/DESCRIBE result graph (the engine answers those as flat N-Triples). This is
// exactly what research/gui-design.md §A.4 names as something the GUI adds that the static site
// "fundamentally cannot": a graph visualisation of results. The site's REPL only ever serialises
// the graph to Turtle/N-Triples; here it renders.
//
// DEPENDENCY-FREE on purpose: no d3 / cytoscape / vis-network. A force simulation or a heavy
// graph lib would be a large SSR-incompatible dependency (cytoscape is ~350 KB min, d3-force
// pulls a tree of modules) for a view that, for a CONSTRUCT/DESCRIBE result, is small. Instead we
// use the `parseNTriples` reshaper already in `@sparq/client` and a deterministic radial layout
// (no animation, no physics, no RNG → stable + diffable). Nodes are subjects/objects; directed
// edges are predicates. IRIs are abbreviated with the common prefixes for legibility. For very
// large graphs we cap the rendered triples and say so (the full result still lives in the
// N-Triples / Turtle views + export).

import * as React from "react";

import {
  parseNTriples,
  COMMON_PREFIXES,
  LENS_TYPE_COLORS,
  edgeKeyOfStatement,
  foldReifiedAnnotations,
  nodeKeyOfRdfTerm,
  rankRadius,
  typeColorIndex,
  type GraphEdgeAnnotation,
  type GraphEdgeStyle,
  type GraphNodeStyle,
  type RdfStatement,
  type RdfTerm,
} from "@sparq/client";

// [FABLE-5] sq-ixc3.20 — canonical triple identity (the inferred-edge membership test) +
// the clicked-triple shape the proof panel consumes.
import { tripleKeyOfTerms } from "@/lib/inferred-facts";
import type { ExplainTarget } from "@/components/workbench/proof-panel";

/**
 * [FABLE-5] sq-ixc3.20 — the optional INFERRED-fact affordance: when the active inference
 * regime has a materialised closure, `keys` holds the canonical keys of every triple the
 * closure ADDED (exact set difference, engine-context `entailedTripleKeys`), and an edge
 * whose statement is in that set renders dashed in the primary colour and becomes a
 * click-to-explain target (`onExplain` opens the why() proof panel). Asserted edges are
 * untouched — no affordance, exactly as the honesty contract requires.
 */
export interface InferredAffordance {
  keys: ReadonlySet<string>;
  onExplain: (t: ExplainTarget) => void;
}

/**
 * [OPUS-5] sq-ixc3.22 — the resolved styling a graph-viz LENS contributes to this render:
 * `nodes` maps a node key (`nodeKeyOfRdfTerm`) to the node basics the lens's `nodeStyle` slot
 * bound (`?type` → colour, `?label` → caption, `?rank` → relative size), `edges` maps an edge
 * key (`edgeKeyOfStatement`) to the caption its `edgeStyle` slot bound. Both are optional and
 * both degrade to the plain view when a lens leaves that slot empty — the graph never depends
 * on a lens being configured.
 */
export interface GraphLensStyling {
  nodes: ReadonlyMap<string, GraphNodeStyle>;
  edges: ReadonlyMap<string, GraphEdgeStyle>;
}

/** The max triples we lay out before capping (a render bound, labelled — not a result bound). */
const MAX_TRIPLES = 200;

/** Node radius range the lens's `?rank` interpolates over (the un-ranked default is the mid). */
const NODE_R_MIN = 4;
const NODE_R_MAX = 11;

/**
 * The colour for a lens `?type` bucket: a fixed hue step over the deterministic palette slot, at
 * a mid lightness that stays legible in both themes. Deterministic in the type IRI, so a shared
 * lens paints the same picture for everyone who opens it.
 */
function typeColor(type: string): string {
  return `hsl(${(typeColorIndex(type) * 360) / LENS_TYPE_COLORS} 62% 48%)`;
}

/** Abbreviate an IRI with a common prefix (`foaf:name`), else shorten to its last path segment. */
function abbreviateIri(iri: string): string {
  for (const { prefix, iri: ns } of COMMON_PREFIXES) {
    if (iri.startsWith(ns)) return `${prefix}:${iri.slice(ns.length)}`;
  }
  // Fall back to the fragment / last path segment so a bare IRI is still legible.
  const hash = iri.lastIndexOf("#");
  const slash = iri.lastIndexOf("/");
  const cut = Math.max(hash, slash);
  return cut >= 0 && cut < iri.length - 1 ? iri.slice(cut + 1) : iri;
}

/** A short, human label for a term (the node/edge caption). */
function termLabel(t: RdfTerm): string {
  switch (t.kind) {
    case "iri":
      return abbreviateIri(t.value);
    case "bnode":
      return `_:${t.label}`;
    case "literal":
      return t.value.length > 24 ? `"${t.value.slice(0, 23)}…"` : `"${t.value}"`;
    case "triple":
      return "« triple »";
  }
}

/** A stable identity key for a term (so the same node merges across triples). */
function termKey(t: RdfTerm): string {
  return t.nt;
}

interface LaidOutNode {
  key: string;
  label: string;
  isLiteral: boolean;
  x: number;
  y: number;
  /** [OPUS-5] sq-ixc3.22 — the term itself, so a click can report the focus node to the lens. */
  term: RdfTerm;
}

interface LaidOutEdge {
  from: string;
  to: string;
  label: string;
}

/** Deterministic radial layout: collect distinct nodes, place them on a circle in first-seen
 *  order, and connect them with the predicate-labelled directed edges. No physics, no RNG. */
function layout(statements: RdfStatement[], size: number): {
  nodes: LaidOutNode[];
  edges: LaidOutEdge[];
} {
  const order: string[] = [];
  const meta = new Map<string, { label: string; isLiteral: boolean; term: RdfTerm }>();
  const note = (t: RdfTerm) => {
    const k = termKey(t);
    if (!meta.has(k)) {
      order.push(k);
      meta.set(k, { label: termLabel(t), isLiteral: t.kind === "literal", term: t });
    }
  };
  const edges: LaidOutEdge[] = [];
  for (const st of statements) {
    note(st.s);
    note(st.o);
    edges.push({ from: termKey(st.s), to: termKey(st.o), label: termLabel(st.p) });
  }

  const cx = size / 2;
  const cy = size / 2;
  const radius = size / 2 - 64;
  const n = order.length;
  const nodes: LaidOutNode[] = order.map((k, i) => {
    // A single node sits at the centre; otherwise spread evenly around the circle starting at
    // the top (−90°) so the layout reads clockwise.
    const angle = n <= 1 ? 0 : (i / n) * 2 * Math.PI - Math.PI / 2;
    const x = n <= 1 ? cx : cx + radius * Math.cos(angle);
    const y = n <= 1 ? cy : cy + radius * Math.sin(angle);
    const m = meta.get(k)!;
    return { key: k, label: m.label, isLiteral: m.isLiteral, term: m.term, x, y };
  });
  return { nodes, edges };
}

/**
 * [OPUS-5] sq-ixc3.22 — the honest note for reifier annotations whose described triple is NOT in
 * this result. RDF 1.2 reification does not assert the reified triple, so this is a normal
 * outcome; the view reports the count rather than inventing an edge to hang them on.
 */
function UnattachedNote({ count }: { count: number }) {
  return (
    <p className="text-[11px] text-muted-foreground" data-graph-unattached-annotations={count}>
      {count.toLocaleString()} annotation{count === 1 ? "" : "s"} describe a triple that is not in
      this result, so {count === 1 ? "it has" : "they have"} no edge to sit on. RDF 1.2
      reification does not assert the triple it describes — widen the query to draw them.
    </p>
  );
}

/** The `<title>` tooltip listing a folded reifier's properties, one per line. */
function annotationTooltip(anns: readonly GraphEdgeAnnotation[]): string {
  const head =
    anns.length === 1
      ? "1 RDF 1.2 annotation on this triple:"
      : `${anns.length} RDF 1.2 annotations on this triple:`;
  return [head, ...anns.map((a) => `  ${termLabel(a.p)} → ${termLabel(a.o)}`)].join("\n");
}

/** The `<title>` tooltip for a node: its full term plus whatever the lens resolved for it. */
function nodeTooltip(
  node: LaidOutNode,
  style: GraphNodeStyle | undefined,
  interactive: boolean,
): string {
  const lines = [node.term.nt];
  if (style?.type) lines.push(`type: ${abbreviateIri(style.type)}`);
  if (style?.rank !== undefined) lines.push(`rank: ${style.rank}`);
  if (interactive) lines.push("Click to expand with the active lens");
  return lines.join("\n");
}

/**
 * Render a CONSTRUCT/DESCRIBE result's N-Triples document as an SVG node-link graph. Stateless +
 * deterministic; safe in a static export and in the Tauri webview.
 */
export function GraphView({
  ntriples,
  inferred,
  lens,
  focusKey,
  onFocusNode,
}: {
  ntriples: string;
  /** [FABLE-5] sq-ixc3.20 — mark + explain inferred edges (absent = no affordance). */
  inferred?: InferredAffordance | null;
  /** [OPUS-5] sq-ixc3.22 — node/edge basics the active lens resolved (absent = plain view). */
  lens?: GraphLensStyling | null;
  /** [OPUS-5] sq-ixc3.22 — the node key currently focused, drawn with a selection ring. */
  focusKey?: string | null;
  /** [OPUS-5] sq-ixc3.22 — click/Enter on a node (absent = nodes are not interactive). */
  onFocusNode?: (term: RdfTerm) => void;
}) {
  // [OPUS-5] sq-ixc3.22 — RDF 1.2 reifier annotations are folded ONTO their edges BEFORE the
  // render cap, so a reifier's properties become badges on the edge they describe instead of a
  // cloud of plumbing nodes — and so an annotation is never lost to the cap while its edge
  // survives it. Folding is unconditional: it needs no lens, only data that carries reification.
  const { statements, total, annotations, unattached } = React.useMemo(() => {
    // `parseNTriples` returns `{ statements, passthrough }`; we only graph the parsed triples.
    const { statements: all } = parseNTriples(ntriples);
    const folded = foldReifiedAnnotations(all);
    return {
      statements: folded.base.slice(0, MAX_TRIPLES),
      total: folded.base.length,
      annotations: folded.annotations,
      unattached: folded.unattached,
    };
  }, [ntriples]);

  const SIZE = 520;
  const { nodes, edges } = React.useMemo(() => layout(statements, SIZE), [statements]);

  // [FABLE-5] sq-ixc3.20 — per-edge inferredness. `layout` pushes exactly ONE edge per
  // statement in order, so edge i ↔ statement i; membership in the closure-added key set
  // is the affordance gate (an asserted edge can never test true).
  const inferredFlags = React.useMemo(() => {
    if (!inferred || inferred.keys.size === 0) return null;
    const flags = statements.map((st) => inferred.keys.has(tripleKeyOfTerms(st.s, st.p, st.o)));
    return flags.some(Boolean) ? flags : null;
  }, [statements, inferred]);
  const posByKey = React.useMemo(() => {
    const m = new Map<string, LaidOutNode>();
    for (const node of nodes) m.set(node.key, node);
    return m;
  }, [nodes]);

  // [OPUS-5] sq-ixc3.22 — the observed `?rank` range across the DRAWN nodes, so relative node
  // size is relative to what is on screen. Absent when the lens bound no rank at all.
  const rankRange = React.useMemo(() => {
    if (!lens) return null;
    let lo = Number.POSITIVE_INFINITY;
    let hi = Number.NEGATIVE_INFINITY;
    for (const node of nodes) {
      const rank = lens.nodes.get(nodeKeyOfRdfTerm(node.term))?.rank;
      if (rank === undefined || !Number.isFinite(rank)) continue;
      lo = Math.min(lo, rank);
      hi = Math.max(hi, rank);
    }
    return Number.isFinite(lo) && Number.isFinite(hi) ? { lo, hi } : null;
  }, [lens, nodes]);

  // [OPUS-5] sq-ixc3.22 — per-edge annotations, indexed the same way as `inferredFlags`
  // (edge i ↔ statement i, because `layout` pushes exactly one edge per statement in order).
  const edgeAnnotations = React.useMemo<Array<GraphEdgeAnnotation[] | undefined>>(
    () => statements.map((st) => annotations.get(edgeKeyOfStatement(st))),
    [statements, annotations],
  );
  const annotatedCount = React.useMemo(
    () => edgeAnnotations.filter((a) => a !== undefined).length,
    [edgeAnnotations],
  );

  if (total === 0) {
    return (
      <div className="p-3 text-sm text-muted-foreground" data-result-view="graph">
        <p>Empty graph — the template produced no triples.</p>
        {unattached > 0 && <UnattachedNote count={unattached} />}
      </div>
    );
  }

  const truncated = total > statements.length;

  return (
    <div className="flex h-full flex-col" data-result-view="graph">
      {/* [FABLE-5] sq-ixc3.20 — the inferred-edge legend, only when something IS inferred. */}
      {inferredFlags && (
        <p
          className="border-b bg-card px-3 py-1 text-[11px] text-muted-foreground"
          data-graph-inferred-legend
        >
          <span className="font-medium text-primary">Dashed edges are inferred</span> by the
          active inference regime (not asserted) — click one to see its derivation.
        </p>
      )}
      {/* [OPUS-5] sq-ixc3.22 — the RDF 1.2 annotation legend, only when something IS annotated. */}
      {annotatedCount > 0 && (
        <p
          className="border-b bg-card px-3 py-1 text-[11px] text-muted-foreground"
          data-graph-annotation-legend
        >
          <span className="font-medium text-foreground">
            {annotatedCount.toLocaleString()} edge{annotatedCount === 1 ? "" : "s"} carry an
            annotation badge
          </span>{" "}
          — RDF 1.2 reifiers describing that triple, folded onto it. Hover a badge for the
          reifier&rsquo;s properties.
        </p>
      )}
      {unattached > 0 && (
        <div className="border-b px-3 py-1">
          <UnattachedNote count={unattached} />
        </div>
      )}
      {truncated && (
        <p className="border-b bg-warning/10 px-3 py-1 text-[11px] text-muted-foreground">
          Showing the first {statements.length.toLocaleString()} of{" "}
          {total.toLocaleString()} triples (graph-view render cap). The full graph is in the
          N-Triples / Turtle views and the export.
        </p>
      )}
      <div className="min-h-0 flex-1 overflow-auto p-3">
        <svg
          viewBox={`0 0 ${SIZE} ${SIZE}`}
          className="mx-auto h-auto w-full max-w-[560px]"
          role="img"
          aria-label="Node-link graph of the result triples"
        >
          <defs>
            <marker
              id="sq-arrow"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--muted-foreground)" />
            </marker>
          </defs>

          {/* Edges first (under the nodes). */}
          {edges.map((e, i) => {
            const a = posByKey.get(e.from);
            const b = posByKey.get(e.to);
            if (!a || !b) return null;
            const mx = (a.x + b.x) / 2;
            const my = (a.y + b.y) / 2;
            // [FABLE-5] sq-ixc3.20 — an INFERRED edge (in the closure-added set) renders
            // dashed in the primary colour and is a click-to-explain target. Colour is not
            // the only signal: the dash pattern + the legend + the tooltip carry it too.
            const isInferred = inferredFlags?.[i] === true;
            const st = statements[i];
            const onExplain =
              isInferred && inferred
                ? () => inferred.onExplain({ s: st.s.nt, p: st.p.nt, o: st.o.nt })
                : undefined;
            // [OPUS-5] sq-ixc3.22 — the lens's `edgeStyle` caption wins over the raw predicate;
            // the RDF 1.2 reifier annotations fold onto this edge as a hoverable badge.
            const caption = lens?.edges.get(edgeKeyOfStatement(st))?.label ?? e.label;
            const anns = edgeAnnotations[i];
            return (
              <g
                key={i}
                {...(isInferred
                  ? {
                      role: "button",
                      tabIndex: 0,
                      className: "cursor-pointer focus:outline-1 focus:outline-primary",
                      onClick: onExplain,
                      onKeyDown: (ev: React.KeyboardEvent) => {
                        if (ev.key === "Enter" || ev.key === " ") {
                          ev.preventDefault();
                          onExplain?.();
                        }
                      },
                      "data-entailed-edge": true,
                      "aria-label": `Inferred: ${e.label} — press Enter to see why`,
                    }
                  : {})}
              >
                {isInferred && (
                  <title>Inferred (not asserted) — click to see the derivation</title>
                )}
                {/* A wide invisible hit line so a 1.25px inferred edge is actually clickable. */}
                {isInferred && (
                  <line
                    x1={a.x}
                    y1={a.y}
                    x2={b.x}
                    y2={b.y}
                    stroke="transparent"
                    strokeWidth={10}
                  />
                )}
                <line
                  x1={a.x}
                  y1={a.y}
                  x2={b.x}
                  y2={b.y}
                  stroke={isInferred ? "var(--primary)" : "var(--border)"}
                  strokeWidth={1.25}
                  strokeDasharray={isInferred ? "4 3" : undefined}
                  markerEnd="url(#sq-arrow)"
                />
                <text
                  x={mx}
                  y={my}
                  className={isInferred ? "fill-primary" : "fill-muted-foreground"}
                  fontSize={8}
                  textAnchor="middle"
                  dy={-2}
                >
                  {caption}
                </text>
                {/* [OPUS-5] sq-ixc3.22 — the RDF 1.2 annotation badge. The count is drawn (not
                    just a colour), and the reifier's properties are the <title> tooltip, so the
                    information is available without hover-only affordances. */}
                {anns && (
                  <g data-annotated-edge data-annotation-count={anns.length}>
                    <title>{annotationTooltip(anns)}</title>
                    <circle
                      cx={mx}
                      cy={my + 6}
                      r={5}
                      fill="var(--card)"
                      stroke="var(--muted-foreground)"
                      strokeWidth={0.75}
                    />
                    <text
                      x={mx}
                      y={my + 6}
                      className="fill-foreground"
                      fontSize={6}
                      textAnchor="middle"
                      dominantBaseline="central"
                    >
                      {anns.length}
                    </text>
                  </g>
                )}
              </g>
            );
          })}

          {/* Nodes. Literals render as rounded rects, resources as circles.
              [OPUS-5] sq-ixc3.22 — a lens's node basics restyle the resource circles: `?type`
              picks a deterministic colour, `?rank` the radius, `?label` the caption. Nothing
              here depends on a lens existing — an unconfigured view draws exactly as before. */}
          {nodes.map((node) => {
            const nodeKey = nodeKeyOfRdfTerm(node.term);
            const style = lens?.nodes.get(nodeKey);
            const caption = style?.label ?? node.label;
            const focused = focusKey === nodeKey;
            if (node.isLiteral) {
              return (
                <g key={node.key}>
                  <rect
                    x={node.x - 36}
                    y={node.y - 10}
                    width={72}
                    height={20}
                    rx={4}
                    fill="var(--accent)"
                    stroke="var(--border)"
                  />
                  <text
                    x={node.x}
                    y={node.y}
                    className="fill-accent-foreground"
                    fontSize={9}
                    textAnchor="middle"
                    dominantBaseline="central"
                  >
                    {caption}
                  </text>
                </g>
              );
            }
            const r = rankRange
              ? rankRadius(style?.rank, rankRange.lo, rankRange.hi, NODE_R_MIN, NODE_R_MAX)
              : 6;
            const focus = onFocusNode ? () => onFocusNode(node.term) : undefined;
            return (
              <g
                key={node.key}
                {...(focus
                  ? {
                      role: "button",
                      tabIndex: 0,
                      className: "cursor-pointer focus:outline-1 focus:outline-primary",
                      onClick: focus,
                      onKeyDown: (ev: React.KeyboardEvent) => {
                        if (ev.key === "Enter" || ev.key === " ") {
                          ev.preventDefault();
                          focus();
                        }
                      },
                      "data-graph-node": true,
                      "aria-label": `${caption} — press Enter to expand with the active lens`,
                    }
                  : {})}
              >
                <title>{nodeTooltip(node, style, Boolean(focus))}</title>
                {/* A wide transparent hit circle so a small node is still clickable. */}
                {focus && <circle cx={node.x} cy={node.y} r={12} fill="transparent" />}
                {focused && (
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r={r + 4}
                    fill="none"
                    stroke="var(--primary)"
                    strokeWidth={1.5}
                  />
                )}
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={r}
                  fill={style?.type ? typeColor(style.type) : "var(--primary)"}
                />
                <text
                  x={node.x}
                  y={node.y - r - 4}
                  className="fill-foreground"
                  fontSize={9}
                  textAnchor="middle"
                >
                  {caption}
                </text>
              </g>
            );
          })}
        </svg>
      </div>
    </div>
  );
}
