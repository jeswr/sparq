// [review #3601] The CHEAP eligibility predicate behind the home hero's Table | Graph toggle.
//
// Split out from result-graph.ts (deriveGraph) on purpose. The home hero-runner chunk only needs to
// decide WHETHER to offer the Graph toggle; it must NOT statically pull in the full node-link
// derivation (result-graph.ts) or the SVG renderer (repl-graph-view.tsx) — those are a rarely-used,
// net-new view that must arrive on the invocation path (site bundle policy). This module imports
// nothing heavy, so the hero chunk carries only this allocation-free scan; deriveGraph + the SVG
// renderer load lazily (React.lazy on ResultGraphView) the first time a visitor switches to Graph.
//
// SINGLE SOURCE OF TRUTH: deriveGraph uses this as its precondition gate, so the toggle's
// "is this graph-shaped?" answer and the derivation's own decline can never drift. (deriveGraph may
// still return null AFTER this passes only in the pathological >MAX_GRAPH_NODES cap-boundary case,
// where every edge/resource sits past the node cap — the renderer handles that as a defensive null.)

import type { SparqlResults, SparqlTerm } from "./sparq-wasm";

/**
 * Whether two terms are the SAME graph node — identity over (type, value, and, for literals,
 * datatype + language). Mirrors `termKey` in result-graph.ts: an adjacent column pair bound to the
 * same term is a self-loop, which draws no edge.
 */
function sameTerm(a: SparqlTerm, b: SparqlTerm): boolean {
  if (a.type !== b.type || a.value !== b.value) return false;
  if (a.type !== "literal") return true;
  return (a.datatype ?? "") === (b.datatype ?? "") && (a["xml:lang"] ?? "") === (b["xml:lang"] ?? "");
}

/**
 * Cheap eligibility check: does this result have the entity-relationship shape the Graph view draws
 * — ≥2 columns, ≥1 row, at least one edge (two adjacent BOUND columns bound to DIFFERENT terms), and
 * at least one RESOURCE (uri/bnode) node? Mirrors deriveGraph's decline conditions WITHOUT building
 * the node/edge maps, so the hero can gate the toggle without eagerly running the full derivation.
 * Early-exits as soon as both an edge and a resource have been seen.
 */
export function isGraphShaped(results: SparqlResults): boolean {
  const vars = results.head?.vars ?? [];
  const rows = results.results?.bindings ?? [];
  if (vars.length < 2 || rows.length === 0) return false;

  let hasEdge = false;
  let hasResource = false;
  for (const row of rows) {
    let prev: SparqlTerm | undefined;
    for (const v of vars) {
      const t = row[v];
      if (!t) continue; // unbound cell: skip so an OPTIONAL gap never breaks the adjacency chain
      if (!hasResource && t.type !== "literal") hasResource = true;
      if (!hasEdge && prev && !sameTerm(prev, t)) hasEdge = true;
      prev = t;
    }
    if (hasEdge && hasResource) return true;
  }
  return false;
}
