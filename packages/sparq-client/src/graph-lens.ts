// [OPUS-5] sq-ixc3.22 (epic sq-ixc3) — PROGRAMMABLE GRAPH-VIZ LENSES + RDF 1.2 reifier
// annotation folding: the framework-agnostic core behind the GUI graph view's "viz as a
// programmable lens" model.
//
// WHAT A LENS IS. A named configuration of up to FIVE SPARQL queries that together define how a
// graph is explored and drawn — the user, not the app, decides what a node is, what expanding it
// means, and how it is styled:
//
//   1. `start`      SELECT ?node            — the entry points the view opens on.
//   2. `expand`     CONSTRUCT { … }         — the neighbourhood of ONE focus node (`?node` is
//                                             bound to the clicked node before the run).
//   3. `nodeStyle`  SELECT ?node ?type ?label ?rank
//                                           — node basics: colour (from ?type), caption (?label)
//                                             and relative size (?rank).
//   4. `edgeStyle`  SELECT ?s ?p ?o ?label  — edge basics: an alternative caption per edge.
//   5. `nodeDetail` SELECT ?key ?value      — the side panel for the focused node (`?node` bound).
//
// A lens is DATA (it persists in the workspace record and serialises to JSON), so it is saveable
// and shareable. Everything in this module is PURE — no DOM, no React, no engine handle — so the
// same code serves the Next.js host and the Tauri webview, and is unit-testable under
// `node --test` (see `test/graph-lens.test.mjs`).
//
// WHY THE `?node` BINDING IS TEXTUAL. The wasm `Store.query` surface takes a query STRING and has
// no initial-bindings parameter, so `bindFocusNode` substitutes the `?node` / `$node` variable
// token with the focus term's exact N-Triples spelling. The substitution is escape-aware: it
// skips string literals, IRIs and `#` comments, so a `?node` inside `"…"` or `<…>` is never
// rewritten. It is a variable BINDING, not a query builder — the slot's query must be written so
// that `?node` occurring in a pattern position is the only use (that is the documented contract
// for the `expand` / `nodeDetail` slots).
//
// RDF 1.2 ANNOTATION FOLDING (`foldReifiedAnnotations`) is deliberately lens-INDEPENDENT: any
// CONSTRUCT/DESCRIBE result that carries reifying triples (`?r rdf:reifies <<( s p o )>>`) plus
// the reifier's own properties gets those properties folded ONTO the edge they describe instead
// of drawn as anonymous plumbing nodes. sparq's engine already leads on RDF 1.2; this is what
// lets the view actually SHOW it.
//
// NO PERFORMANCE CLAIM is made anywhere here.

import type { SparqlResults, SparqlTerm } from "./index.js";
import type { RdfStatement, RdfTerm } from "./pretty-turtle.js";

/** The RDF 1.2 reification predicate linking a reifier to a triple term. */
export const RDF_REIFIES = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

// ---------------------------------------------------------------------------
// The lens model.
// ---------------------------------------------------------------------------

/** The five query slots a lens can fill. Every slot is optional; an empty slot is inert. */
export type GraphLensSlot = "start" | "expand" | "nodeStyle" | "edgeStyle" | "nodeDetail";

/** Static description of one slot — what it is for, what form it takes, what it must project. */
export interface GraphLensSlotSpec {
  slot: GraphLensSlot;
  /** Short UI label. */
  label: string;
  /** The SPARQL query form the slot expects. */
  form: "SELECT" | "CONSTRUCT";
  /** The variables the slot reads from the result (a projection the lens author must provide). */
  projects: readonly string[];
  /** Whether `?node` is bound to the focus node before the query runs. */
  bindsFocus: boolean;
  /** One-line authoring hint shown next to the editor. */
  hint: string;
}

/**
 * The five slots in authoring order. This is the single source of truth the lens editor renders
 * from — adding a slot is a change here plus a consumer of the new result shape, never a new
 * hard-coded form in the UI.
 */
export const GRAPH_LENS_SLOTS: readonly GraphLensSlotSpec[] = [
  {
    slot: "start",
    label: "Start",
    form: "SELECT",
    projects: ["node"],
    bindsFocus: false,
    hint: "Entry points the view opens on. Project ?node.",
  },
  {
    slot: "expand",
    label: "Expand",
    form: "CONSTRUCT",
    projects: [],
    bindsFocus: true,
    hint: "The neighbourhood of one node. ?node is bound to the focus before each run.",
  },
  {
    slot: "nodeStyle",
    label: "Node basics",
    form: "SELECT",
    projects: ["node", "type", "label", "rank"],
    bindsFocus: false,
    hint: "?type picks the colour, ?label the caption, ?rank the relative size.",
  },
  {
    slot: "edgeStyle",
    label: "Edge basics",
    form: "SELECT",
    projects: ["s", "p", "o", "label"],
    bindsFocus: false,
    hint: "Per-edge caption override for the edge ?s ?p ?o.",
  },
  {
    slot: "nodeDetail",
    label: "Node panel",
    form: "SELECT",
    projects: ["key", "value"],
    bindsFocus: true,
    hint: "Side-panel rows for the focused node. ?node is bound before the run.",
  },
] as const;

/** A named, saveable, shareable lens: up to five SPARQL queries plus its identity. */
export interface GraphLens {
  /** Stable opaque id (the persistence + selection key). */
  id: string;
  /** User-facing name. */
  name: string;
  /** The slot queries. An absent or blank slot is inert. */
  queries: Partial<Record<GraphLensSlot, string>>;
  /** Epoch ms of the last edit (display + ordering only). */
  updatedAt: number;
  /** True for the built-in lens, which is offered but never persisted or deleted. */
  builtin?: boolean;
}

/** The id of the built-in default lens. Reserved: a user lens can never take this id. */
export const DEFAULT_LENS_ID = "lens:default";

/**
 * The built-in default lens.
 *
 * HONEST NOTE ON RANKING. `?rank` here is **out-degree**, computed by a plain SPARQL `COUNT` over
 * the data itself. sparq's full-text index has a prefix-token ("autocomplete") search path, but
 * it exposes no per-term ranking through the in-tab wasm bundle the GUI drives, so there is no
 * autocomplete-rank signal for this lens to read. Out-degree is a real, computed ordering — not a
 * stand-in pretending to be something else — and any lens can bind `?rank` to whatever ranking
 * predicate its own data carries.
 */
export const DEFAULT_LENS: GraphLens = {
  id: DEFAULT_LENS_ID,
  name: "Default (out-degree)",
  builtin: true,
  updatedAt: 0,
  queries: {
    start: `SELECT ?node (COUNT(?o) AS ?rank) WHERE {
  ?node ?p ?o .
  FILTER(!isLiteral(?node))
}
GROUP BY ?node
ORDER BY DESC(?rank)
LIMIT 10`,
    expand: `CONSTRUCT { ?node ?p ?o . ?in ?ip ?node }
WHERE {
  { ?node ?p ?o }
  UNION
  { ?in ?ip ?node }
}
LIMIT 100`,
    nodeStyle: `SELECT ?node ?type ?label (COUNT(?o) AS ?rank) WHERE {
  ?node ?p ?o .
  OPTIONAL { ?node a ?type }
  OPTIONAL { ?node <http://www.w3.org/2000/01/rdf-schema#label> ?label }
}
GROUP BY ?node ?type ?label`,
    nodeDetail: `SELECT ?key ?value WHERE {
  ?node ?key ?value .
  FILTER(isLiteral(?value))
}
LIMIT 50`,
  },
};

/** A short, collision-resistant lens id (mirrors `workspaceId()`'s fallback discipline). */
export function graphLensId(): string {
  const c = (globalThis as { crypto?: { randomUUID?: () => string } }).crypto;
  if (c?.randomUUID) return `lens_${c.randomUUID()}`;
  return `lens_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 10)}`;
}

/** Create a new user lens seeded from `from` (the built-in default unless given). */
export function newGraphLens(name: string, from: GraphLens = DEFAULT_LENS): GraphLens {
  return {
    id: graphLensId(),
    name,
    queries: { ...from.queries },
    updatedAt: Date.now(),
  };
}

/**
 * Validate + normalise an arbitrary value into a {@link GraphLens}, or return `null` when it is
 * not a recognisable lens. Defensive about every field: a hand-edited workspace file, a pasted
 * "shared lens" JSON blob and a record written by a future version all flow through here, and a
 * malformed one must be DROPPED rather than crash the view. Unknown slot keys are discarded.
 */
export function parseGraphLens(value: unknown): GraphLens | null {
  if (typeof value !== "object" || value === null) return null;
  const v = value as Record<string, unknown>;
  if (typeof v.id !== "string" || v.id === "") return null;
  if (typeof v.name !== "string" || v.name.trim() === "") return null;
  const rawQueries =
    typeof v.queries === "object" && v.queries !== null
      ? (v.queries as Record<string, unknown>)
      : {};
  const queries: Partial<Record<GraphLensSlot, string>> = {};
  for (const { slot } of GRAPH_LENS_SLOTS) {
    const q = rawQueries[slot];
    if (typeof q === "string" && q.trim() !== "") queries[slot] = q;
  }
  return {
    id: v.id,
    name: v.name.trim(),
    queries,
    updatedAt: typeof v.updatedAt === "number" ? v.updatedAt : 0,
  };
}

/**
 * Validate a persisted lens LIST: keep the parseable entries, drop malformed ones, and drop any
 * entry colliding with the built-in id or with an earlier entry's id (ids are the selection key,
 * so a duplicate would make selection ambiguous).
 */
export function parseGraphLenses(value: unknown): GraphLens[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>([DEFAULT_LENS_ID]);
  return value.flatMap((entry) => {
    const lens = parseGraphLens(entry);
    if (!lens || seen.has(lens.id)) return [];
    seen.add(lens.id);
    return [lens];
  });
}

/** Serialise a lens to the shareable JSON blob (the `builtin` flag is never carried over). */
export function serializeGraphLens(lens: GraphLens): string {
  return JSON.stringify(
    { id: lens.id, name: lens.name, queries: lens.queries, updatedAt: lens.updatedAt },
    null,
    2,
  );
}

/**
 * Parse a shared-lens JSON blob into a lens with a FRESH id, so importing a lens someone else
 * exported can never silently overwrite a local lens that happens to share its id.
 */
export function importGraphLens(json: string): GraphLens | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch {
    return null;
  }
  const lens = parseGraphLens(parsed);
  if (!lens) return null;
  return { ...lens, id: graphLensId(), updatedAt: Date.now() };
}

// ---------------------------------------------------------------------------
// Node identity — one key space over both term representations.
// ---------------------------------------------------------------------------

/** Escape a literal's lexical form back into its N-Triples spelling. */
function escapeLiteral(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
}

/**
 * The identity key for a node, in a space SHARED by the two term representations the view joins:
 * the `RdfTerm`s parsed out of a CONSTRUCT result and the `SparqlTerm`s a styling SELECT binds.
 * IRIs and blank nodes key on their value (never on the N-Triples spelling) so an IRI written
 * with an `\uXXXX` escape on one side still matches the same IRI on the other.
 */
export function nodeKeyOfRdfTerm(t: RdfTerm): string {
  switch (t.kind) {
    case "iri":
      return `i:${t.value}`;
    case "bnode":
      return `b:${t.label}`;
    case "literal":
      return `l:${t.nt}`;
    case "triple":
      return `t:${t.nt}`;
  }
}

/** The {@link nodeKeyOfRdfTerm} key for a SPARQL-JSON term binding. */
export function nodeKeyOfSparqlTerm(t: SparqlTerm | undefined): string | null {
  if (!t) return null;
  if (t.type === "uri") return `i:${t.value}`;
  if (t.type === "bnode") return `b:${t.value}`;
  const lex = `"${escapeLiteral(t.value)}"`;
  const lang = t["xml:lang"];
  if (lang) return `l:${lex}@${lang}`;
  if (t.datatype && t.datatype !== XSD_STRING) return `l:${lex}^^<${t.datatype}>`;
  return `l:${lex}`;
}

/**
 * The exact N-Triples spelling of a SPARQL-JSON term binding — what {@link bindFocusNode}
 * substitutes for `?node`. Returns `null` for an unbound variable (there is nothing to bind).
 */
export function termToNTriples(t: SparqlTerm | undefined): string | null {
  if (!t) return null;
  if (t.type === "uri") return `<${t.value}>`;
  if (t.type === "bnode") return `_:${t.value}`;
  const lex = `"${escapeLiteral(t.value)}"`;
  const lang = t["xml:lang"];
  if (lang) return `${lex}@${lang}`;
  if (t.datatype && t.datatype !== XSD_STRING) return `${lex}^^<${t.datatype}>`;
  return lex;
}

/** The identity key for one directed edge, over the shared node key space. */
export function edgeKeyOf(s: string, predicate: string, o: string): string {
  return `${s} ${predicate} ${o}`;
}

/** The {@link edgeKeyOf} key for a parsed statement. */
export function edgeKeyOfStatement(st: RdfStatement): string {
  return edgeKeyOf(nodeKeyOfRdfTerm(st.s), nodeKeyOfRdfTerm(st.p), nodeKeyOfRdfTerm(st.o));
}

// ---------------------------------------------------------------------------
// Binding the focus node into a slot query.
// ---------------------------------------------------------------------------

// A character that can CONTINUE a SPARQL variable name. Deliberately generous (every non-ASCII
// code point counts): erring toward "this is a longer variable, leave it alone" can only ever
// decline a substitution, never corrupt one.
const NAME_CHAR = /[A-Za-z0-9_·À-￿]/;

/**
 * Bind the focus node into a slot query by substituting the `?node` / `$node` variable token with
 * `termNt` — the term's exact N-Triples spelling (`<iri>` / `_:label` / `"lit"…`).
 *
 * The scan is escape-aware: it skips over `#` comments, `<…>` IRIs and `"…"` / `'…'` string
 * literals (including the triple-quoted forms), so a `?node` appearing INSIDE any of those is
 * left untouched. Only a `?node` / `$node` whose next character cannot continue a variable name
 * is replaced, so `?nodes` and `?node_id` are not clobbered.
 *
 * Returns the query unchanged when `?node` does not occur — a lens slot that ignores the focus is
 * a legitimate (if unusual) lens, not an error.
 */
export function bindFocusNode(query: string, termNt: string): string {
  let out = "";
  let i = 0;
  while (i < query.length) {
    const c = query[i];
    if (c === "#") {
      const end = query.indexOf("\n", i);
      const stop = end === -1 ? query.length : end;
      out += query.slice(i, stop);
      i = stop;
      continue;
    }
    if (c === "<") {
      // An IRI runs to the first unescaped '>'. A stray '<' (the less-than operator) has no
      // closing '>' on the line; treat it as a plain character so we do not swallow the query.
      const end = closingIri(query, i);
      if (end === -1) {
        out += c;
        i++;
        continue;
      }
      out += query.slice(i, end + 1);
      i = end + 1;
      continue;
    }
    if (c === '"' || c === "'") {
      const end = closingString(query, i);
      out += query.slice(i, end);
      i = end;
      continue;
    }
    if ((c === "?" || c === "$") && query.startsWith("node", i + 1)) {
      const after = query[i + 5];
      if (after === undefined || !NAME_CHAR.test(after)) {
        out += termNt;
        i += 5;
        continue;
      }
    }
    out += c;
    i++;
  }
  return out;
}

/** Index of the '>' closing the IRI starting at `start`, or -1 when it is not an IRI. */
function closingIri(s: string, start: number): number {
  for (let i = start + 1; i < s.length; i++) {
    const c = s[i];
    if (c === "\\") {
      i++;
      continue;
    }
    if (c === ">") return i;
    // An IRI reference cannot span a line or contain whitespace / '<'.
    if (c === "\n" || c === " " || c === "\t" || c === "<") return -1;
  }
  return -1;
}

/** Index just PAST the string literal starting at `start` (its quote char is `s[start]`). */
function closingString(s: string, start: number): number {
  const q = s[start];
  const triple = s[start + 1] === q && s[start + 2] === q;
  const delim = triple ? q + q + q : q;
  let i = start + delim.length;
  while (i < s.length) {
    if (s[i] === "\\") {
      i += 2;
      continue;
    }
    if (s.startsWith(delim, i)) return i + delim.length;
    // A single-quoted SPARQL string cannot contain a raw newline; stop there rather than
    // swallowing the rest of the query when the literal is unterminated.
    if (!triple && s[i] === "\n") return i;
    i++;
  }
  return s.length;
}

// ---------------------------------------------------------------------------
// Reading the styling slots' results.
// ---------------------------------------------------------------------------

/** Node basics a lens's `nodeStyle` slot resolved for one node. */
export interface GraphNodeStyle {
  /** The `?type` term's value — the colour bucket (deterministic, see {@link typeColorIndex}). */
  type?: string;
  /** The `?label` caption override. */
  label?: string;
  /** The `?rank` numeric weight driving relative node size (higher = larger). */
  rank?: number;
}

/** Edge basics a lens's `edgeStyle` slot resolved for one edge. */
export interface GraphEdgeStyle {
  label?: string;
}

/** One side-panel row from the `nodeDetail` slot. */
export interface GraphNodeDetailRow {
  key: string;
  value: string;
}

function rows(results: SparqlResults): Array<Record<string, SparqlTerm | undefined>> {
  return (results.results?.bindings ?? []) as Array<Record<string, SparqlTerm | undefined>>;
}

/**
 * Index a `nodeStyle` result by node key. The FIRST row for a node wins for `?type` / `?label`
 * (so an `ORDER BY` in the lens is the author's tiebreak), and `?rank` takes the maximum seen —
 * a grouped-count projection can emit one row per `?type`, and the node's weight should not
 * depend on which of those rows arrived first.
 */
export function nodeStyleIndex(results: SparqlResults): Map<string, GraphNodeStyle> {
  const index = new Map<string, GraphNodeStyle>();
  for (const row of rows(results)) {
    const key = nodeKeyOfSparqlTerm(row.node);
    if (key === null) continue;
    const current = index.get(key) ?? {};
    if (current.type === undefined && row.type) current.type = row.type.value;
    if (current.label === undefined && row.label) current.label = row.label.value;
    if (row.rank) {
      const n = Number(row.rank.value);
      if (Number.isFinite(n)) current.rank = current.rank === undefined ? n : Math.max(current.rank, n);
    }
    index.set(key, current);
  }
  return index;
}

/** Index an `edgeStyle` result by edge key (first row per edge wins). */
export function edgeStyleIndex(results: SparqlResults): Map<string, GraphEdgeStyle> {
  const index = new Map<string, GraphEdgeStyle>();
  for (const row of rows(results)) {
    const s = nodeKeyOfSparqlTerm(row.s);
    const p = nodeKeyOfSparqlTerm(row.p);
    const o = nodeKeyOfSparqlTerm(row.o);
    if (s === null || p === null || o === null) continue;
    const key = edgeKeyOf(s, p, o);
    if (index.has(key)) continue;
    index.set(key, { label: row.label?.value });
  }
  return index;
}

/** Shape a `nodeDetail` result into the side panel's key/value rows (in result order). */
export function nodeDetailRows(results: SparqlResults): GraphNodeDetailRow[] {
  return rows(results).flatMap((row) => {
    if (!row.key || !row.value) return [];
    return [{ key: row.key.value, value: row.value.value }];
  });
}

/** How many distinct colours the type palette cycles through. */
export const LENS_TYPE_COLORS = 8;

/**
 * A deterministic palette slot (0 … {@link LENS_TYPE_COLORS} − 1) for a `?type` value. A stable
 * string hash, so the same type is the same colour across runs, reloads and hosts — a lens is
 * shareable, and two people looking at one must see the same picture.
 */
export function typeColorIndex(type: string): number {
  let h = 2166136261;
  for (let i = 0; i < type.length; i++) {
    h ^= type.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return Math.abs(h) % LENS_TYPE_COLORS;
}

/**
 * Map a node's `?rank` onto a radius in `[min, max]`, linearly over the observed rank range.
 * A missing rank, a degenerate range (every node equal) or a non-finite bound all fall back to
 * the midpoint, so an un-ranked graph draws uniformly instead of collapsing to dots.
 */
export function rankRadius(
  rank: number | undefined,
  lo: number,
  hi: number,
  min: number,
  max: number,
): number {
  const mid = (min + max) / 2;
  if (rank === undefined || !Number.isFinite(rank)) return mid;
  if (!Number.isFinite(lo) || !Number.isFinite(hi) || hi <= lo) return mid;
  const t = Math.min(1, Math.max(0, (rank - lo) / (hi - lo)));
  return min + t * (max - min);
}

// ---------------------------------------------------------------------------
// RDF 1.2 reifier annotations → edge badges.
// ---------------------------------------------------------------------------

/** One property of a reifier, folded onto the edge its triple term describes. */
export interface GraphEdgeAnnotation {
  /** The annotating predicate. */
  p: RdfTerm;
  /** The annotating value. */
  o: RdfTerm;
  /** The reifier that carried it (`null` for the direct triple-term-subject form). */
  reifier: RdfTerm | null;
}

/** The result of folding reifying triples out of a graph and onto its edges. */
export interface FoldedAnnotations {
  /** The graph to DRAW: every statement that is not reification plumbing. */
  base: RdfStatement[];
  /** Annotations per {@link edgeKeyOf} key. */
  annotations: Map<string, GraphEdgeAnnotation[]>;
  /**
   * Annotations whose described triple is NOT asserted in this result. RDF 1.2 reification does
   * not assert the reified triple, so this is a normal outcome — the view reports the count
   * rather than inventing an edge to hang them on.
   */
  unattached: number;
}

/**
 * Fold RDF 1.2 reifier annotations onto the edges they describe.
 *
 * Two surface forms are recognised:
 *
 *   * the spec form — `?r rdf:reifies <<( s p o )>>` names `?r` as the REIFIER of `s p o`, and
 *     every other statement with `?r` as its subject is an annotation on that edge;
 *   * the direct form — a statement whose SUBJECT is a triple term. RDF 1.2 puts triple terms in
 *     object position only, so this is not spec-shaped input, but the N-Triples reader accepts it
 *     and data in the wild carries it, so it folds the same way rather than drawing a node whose
 *     caption is « triple ».
 *
 * Reification plumbing (the `rdf:reifies` statement and the reifier's own properties) is removed
 * from `base`, which is the point: the drawn graph stays the data, and the annotations become
 * edge badges instead of a cloud of anonymous reifier nodes. Two passes, so a reifier may be
 * declared after the annotations that reference it.
 */
export function foldReifiedAnnotations(statements: readonly RdfStatement[]): FoldedAnnotations {
  // Pass 1 — reifier subject key → the edge key it describes.
  const reified = new Map<string, { edge: string; term: RdfTerm }>();
  for (const st of statements) {
    if (st.p.kind === "iri" && st.p.value === RDF_REIFIES && st.o.kind === "triple") {
      reified.set(nodeKeyOfRdfTerm(st.s), {
        edge: edgeKeyOf(
          nodeKeyOfRdfTerm(st.o.s),
          nodeKeyOfRdfTerm(st.o.p),
          nodeKeyOfRdfTerm(st.o.o),
        ),
        term: st.s,
      });
    }
  }

  // Pass 2 — split the document into the drawn base graph and the folded annotations.
  const base: RdfStatement[] = [];
  const pending: Array<{ edge: string; ann: GraphEdgeAnnotation }> = [];
  for (const st of statements) {
    if (st.p.kind === "iri" && st.p.value === RDF_REIFIES && st.o.kind === "triple") continue;
    if (st.s.kind === "triple") {
      pending.push({
        edge: edgeKeyOf(
          nodeKeyOfRdfTerm(st.s.s),
          nodeKeyOfRdfTerm(st.s.p),
          nodeKeyOfRdfTerm(st.s.o),
        ),
        ann: { p: st.p, o: st.o, reifier: null },
      });
      continue;
    }
    const target = reified.get(nodeKeyOfRdfTerm(st.s));
    if (target) {
      pending.push({ edge: target.edge, ann: { p: st.p, o: st.o, reifier: target.term } });
      continue;
    }
    base.push(st);
  }

  // Only attach an annotation to an edge the base graph actually asserts.
  const drawn = new Set(base.map(edgeKeyOfStatement));
  const annotations = new Map<string, GraphEdgeAnnotation[]>();
  let unattached = 0;
  for (const { edge, ann } of pending) {
    if (!drawn.has(edge)) {
      unattached++;
      continue;
    }
    const list = annotations.get(edge);
    if (list) list.push(ann);
    else annotations.set(edge, [ann]);
  }
  return { base, annotations, unattached };
}

// ---------------------------------------------------------------------------
// Merging expansion results.
// ---------------------------------------------------------------------------

/**
 * Union two N-Triples documents, preserving first-seen line order and dropping exact duplicates.
 * This is how an `expand` run MERGES into the graph already on screen (click a node, its
 * neighbourhood joins the picture) without re-running everything drawn so far.
 *
 * Deduplication is by exact line text. The engine emits canonical N-Triples, so two identical
 * triples serialise identically; a blank node keeps its label across one engine's output, so
 * `_:b0` from two runs over the same store is the same node.
 */
export function mergeNTriples(...documents: readonly string[]): string {
  const seen = new Set<string>();
  const lines: string[] = [];
  for (const doc of documents) {
    for (const raw of doc.split("\n")) {
      const line = raw.trim();
      if (line === "" || seen.has(line)) continue;
      seen.add(line);
      lines.push(line);
    }
  }
  return lines.length === 0 ? "" : `${lines.join("\n")}\n`;
}
