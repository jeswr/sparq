// [OPUS-5] sq-ixc3.21 — pure derivation for the dataset OVERVIEW tool: the aggregate SPARQL the
// panel runs over the live store, and the pure models its three views render — the
// class-hierarchy bubble pack, the class-relationship chord, and the domain–range table.
//
// HONESTY: every model here is derived from the REAL bindings the engine returned for the
// queries below. Nothing is seeded, sampled or fabricated — an empty store yields empty models
// and the panel says so. Every cap (top-N classes, per-query row limits) is explicit, carried
// back in the model (`hidden…` fields) and surfaced in the UI, so a truncated view is never
// presented as a complete one.
//
// React-free + dependency-free (like lib/select-result-graph.ts) so `npm run test:unit` covers
// the parsing, the packing geometry and the chord angles directly, with no DOM.

import { COMMON_PREFIXES, type SparqlResults, type SparqlTerm } from "@sparq/client";

const RDF_PREFIX = "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>";
const RDFS_PREFIX = "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>";

/**
 * Per-query row limits. These bound what the OVERVIEW asks the engine for; a store with more
 * classes / class-pair predicates than this renders the top rows and the panel says how many
 * were left out. Not a benchmark bound — a view bound.
 */
export const CLASS_ROW_LIMIT = 500;
export const EDGE_ROW_LIMIT = 1000;

/** Q1 — the class bubble sizes: distinct instances per `rdf:type` class. */
export const CLASS_COUNT_QUERY = `SELECT ?class (COUNT(DISTINCT ?s) AS ?instances)
WHERE { ?s a ?class }
GROUP BY ?class
ORDER BY DESC(?instances)
LIMIT ${CLASS_ROW_LIMIT}`;

/** Q2 — the bubble NESTING: asserted `rdfs:subClassOf` axioms between classes. */
export const SUBCLASS_QUERY = `${RDFS_PREFIX}
SELECT ?sub ?super
WHERE { ?sub rdfs:subClassOf ?super }
LIMIT ${EDGE_ROW_LIMIT}`;

/** Q3 — the chord ribbons + the object half of domain–range: class → predicate → class counts. */
export const CLASS_RELATION_QUERY = `${RDF_PREFIX}
SELECT ?source ?predicate ?target (COUNT(*) AS ?statements)
WHERE {
  ?s a ?source .
  ?s ?predicate ?o .
  ?o a ?target .
  FILTER(?predicate != rdf:type)
}
GROUP BY ?source ?predicate ?target
ORDER BY DESC(?statements)
LIMIT ${EDGE_ROW_LIMIT}`;

/** Q4 — the literal half of domain–range: class → predicate → datatype counts. */
export const LITERAL_RANGE_QUERY = `SELECT ?source ?predicate ?datatype (COUNT(*) AS ?statements)
WHERE {
  ?s a ?source .
  ?s ?predicate ?o .
  FILTER(isLiteral(?o))
  BIND(DATATYPE(?o) AS ?datatype)
}
GROUP BY ?source ?predicate ?datatype
ORDER BY DESC(?statements)
LIMIT ${EDGE_ROW_LIMIT}`;

/**
 * Abbreviate an IRI with a well-known prefix (`foaf:Person`), else shorten to its fragment /
 * last path segment. Local to this module: the overview labels IRI *strings* pulled out of
 * SPARQL-JSON, whereas graph-view.tsx labels RDF *terms* and select-result-graph.ts keeps its
 * own private copy — there is no exported shared helper to reuse yet.
 */
export function abbreviateIri(iri: string): string {
  for (const { prefix, iri: ns } of COMMON_PREFIXES) {
    if (iri.startsWith(ns)) return `${prefix}:${iri.slice(ns.length)}`;
  }
  const cut = Math.max(iri.lastIndexOf("#"), iri.lastIndexOf("/"));
  return cut >= 0 && cut < iri.length - 1 ? iri.slice(cut + 1) : iri;
}

/** One class with its measured direct-instance count. */
export interface ClassRow {
  iri: string;
  label: string;
  /** Distinct subjects with this `rdf:type`. Direct instances only — no entailment applied. */
  instances: number;
}

/** One asserted `rdfs:subClassOf` axiom (IRI → IRI). */
export interface SubClassEdge {
  sub: string;
  super: string;
}

/** Statements from instances of `source` to instances of `target` via `predicate`. */
export interface RelationRow {
  source: string;
  predicate: string;
  target: string;
  count: number;
}

/** Literal-valued statements on instances of `source` via `predicate`, by datatype. */
export interface LiteralRangeRow {
  source: string;
  predicate: string;
  /** The `DATATYPE(?o)` IRI, or null when the engine left it unbound. */
  datatype: string | null;
  count: number;
}

function rows(results: SparqlResults): Array<Record<string, SparqlTerm>> {
  return results.results?.bindings ?? [];
}

/** The IRI of an `uri`-typed binding, or null for a missing / bnode / literal binding. */
function iriOf(row: Record<string, SparqlTerm>, name: string): string | null {
  const term = row[name];
  return term && term.type === "uri" && term.value.length > 0 ? term.value : null;
}

/** A non-negative integer count, or null when the binding is missing or not a number. */
function countOf(row: Record<string, SparqlTerm>, name: string): number | null {
  const term = row[name];
  if (!term) return null;
  const n = Number(term.value);
  return Number.isFinite(n) && n >= 0 ? Math.trunc(n) : null;
}

/**
 * Parse {@link CLASS_COUNT_QUERY} bindings. Non-IRI classes (a bnode class expression such as an
 * OWL restriction) are skipped — the bubble view names classes, and a bnode has no name to show.
 * Rows are returned sorted by instance count descending, then IRI, so the view is deterministic
 * whatever order the engine produced.
 */
export function parseClassRows(results: SparqlResults): ClassRow[] {
  const out: ClassRow[] = [];
  for (const row of rows(results)) {
    const iri = iriOf(row, "class");
    const instances = countOf(row, "instances");
    if (iri === null || instances === null) continue;
    out.push({ iri, label: abbreviateIri(iri), instances });
  }
  out.sort((a, b) => b.instances - a.instances || (a.iri < b.iri ? -1 : a.iri > b.iri ? 1 : 0));
  return out;
}

/** Parse {@link SUBCLASS_QUERY} bindings; self-edges and non-IRI ends are dropped. */
export function parseSubClassEdges(results: SparqlResults): SubClassEdge[] {
  const out: SubClassEdge[] = [];
  const seen = new Set<string>();
  for (const row of rows(results)) {
    const sub = iriOf(row, "sub");
    const sup = iriOf(row, "super");
    if (sub === null || sup === null || sub === sup) continue;
    const key = `${sub} ${sup}`;
    if (seen.has(key)) continue;
    seen.add(key);
    out.push({ sub, super: sup });
  }
  return out;
}

/** Parse {@link CLASS_RELATION_QUERY} bindings. */
export function parseRelationRows(results: SparqlResults): RelationRow[] {
  const out: RelationRow[] = [];
  for (const row of rows(results)) {
    const source = iriOf(row, "source");
    const predicate = iriOf(row, "predicate");
    const target = iriOf(row, "target");
    const count = countOf(row, "statements");
    if (source === null || predicate === null || target === null || count === null) continue;
    if (count === 0) continue;
    out.push({ source, predicate, target, count });
  }
  return out;
}

/** Parse {@link LITERAL_RANGE_QUERY} bindings. */
export function parseLiteralRangeRows(results: SparqlResults): LiteralRangeRow[] {
  const out: LiteralRangeRow[] = [];
  for (const row of rows(results)) {
    const source = iriOf(row, "source");
    const predicate = iriOf(row, "predicate");
    const count = countOf(row, "statements");
    if (source === null || predicate === null || count === null || count === 0) continue;
    out.push({ source, predicate, datatype: iriOf(row, "datatype"), count });
  }
  return out;
}

// ── Class hierarchy ────────────────────────────────────────────────────────────────────────────

/** A class in the asserted `rdfs:subClassOf` forest built by {@link buildClassHierarchy}. */
export interface HierarchyNode {
  iri: string;
  label: string;
  /** Direct instances of THIS class (0 for a superclass that only appears in an axiom). */
  instances: number;
  children: HierarchyNode[];
  /**
   * Superclasses beyond the one this node is nested under. A class may have several parents;
   * the tree can only show one, so the rest are recorded here and named in the UI title rather
   * than silently dropped.
   */
  otherParents: string[];
}

/**
 * Build the `rdfs:subClassOf` forest over the classes the store actually uses. A class named
 * only as a superclass is included with `instances: 0` (honest: it has no DIRECT instances).
 * A parent edge is used only when it does not create a cycle, so a cyclic `subClassOf` set
 * (`A ⊑ B ⊑ A`) still terminates — the edge that would close the cycle is reported in
 * `otherParents` instead. Children are ordered by instance count descending, then IRI.
 */
export function buildClassHierarchy(
  classes: readonly ClassRow[],
  edges: readonly SubClassEdge[],
): HierarchyNode[] {
  const nodes = new Map<string, HierarchyNode>();
  const node = (iri: string, instances: number): HierarchyNode => {
    const existing = nodes.get(iri);
    if (existing) return existing;
    const created: HierarchyNode = {
      iri,
      label: abbreviateIri(iri),
      instances,
      children: [],
      otherParents: [],
    };
    nodes.set(iri, created);
    return created;
  };

  for (const c of classes) node(c.iri, c.instances);

  // Candidate parents per class, deterministic order (sorted), restricted to classes we know.
  const parents = new Map<string, string[]>();
  for (const e of edges) {
    if (!nodes.has(e.sub)) continue; // only nest classes that are actually used as a type
    node(e.super, 0); // a superclass with no direct instances is still a real node
    const list = parents.get(e.sub);
    if (list) list.push(e.super);
    else parents.set(e.sub, [e.super]);
  }
  for (const list of parents.values()) list.sort();

  // Attach each class to its first non-cycle-forming parent; the rest become `otherParents`.
  const parentOf = new Map<string, string>();
  const wouldCycle = (child: string, parent: string): boolean => {
    let cursor: string | undefined = parent;
    const guard = new Set<string>();
    while (cursor !== undefined) {
      if (cursor === child) return true;
      if (guard.has(cursor)) return true;
      guard.add(cursor);
      cursor = parentOf.get(cursor);
    }
    return false;
  };
  for (const iri of [...parents.keys()].sort()) {
    const self = nodes.get(iri);
    if (!self) continue;
    for (const parent of parents.get(iri) ?? []) {
      if (parentOf.has(iri) || wouldCycle(iri, parent)) {
        self.otherParents.push(parent);
        continue;
      }
      parentOf.set(iri, parent);
    }
  }

  const roots: HierarchyNode[] = [];
  for (const [iri, self] of nodes) {
    const parent = parentOf.get(iri);
    const parentNode = parent === undefined ? undefined : nodes.get(parent);
    if (parentNode) parentNode.children.push(self);
    else roots.push(self);
  }

  const byWeight = (a: HierarchyNode, b: HierarchyNode) =>
    b.instances - a.instances || (a.iri < b.iri ? -1 : a.iri > b.iri ? 1 : 0);
  const sortDeep = (list: HierarchyNode[]): void => {
    list.sort(byWeight);
    for (const n of list) sortDeep(n.children);
  };
  sortDeep(roots);
  return roots;
}

/** Total instances in a subtree (the parent bubble's tooltip figure). */
export function subtreeInstances(node: HierarchyNode): number {
  return node.children.reduce((sum, c) => sum + subtreeInstances(c), node.instances);
}

// ── Bubble pack ────────────────────────────────────────────────────────────────────────────────

/** A laid-out bubble. `x`/`y` are ABSOLUTE coordinates inside the returned `width`×`height` box. */
export interface PackedBubble {
  iri: string;
  label: string;
  instances: number;
  /** Instances in this bubble's whole subtree (equals `instances` for a leaf). */
  totalInstances: number;
  x: number;
  y: number;
  r: number;
  /** 0 for a root bubble, 1 for its children, … — drives the fill shade. */
  depth: number;
  otherParents: string[];
}

export interface BubblePack {
  bubbles: PackedBubble[];
  width: number;
  height: number;
}

/** Leaf radius bounds (SVG user units, before the viewBox scales them to the panel). */
export const BUBBLE_MIN_RADIUS = 14;
export const BUBBLE_MAX_RADIUS = 56;
const BUBBLE_PADDING = 6;

/**
 * Leaf radius, scaled by √instances against the largest class so bubble AREA tracks instance
 * count. A class with no direct instances still gets {@link BUBBLE_MIN_RADIUS} so it stays
 * visible and clickable.
 */
export function bubbleRadius(instances: number, maxInstances: number): number {
  if (instances <= 0 || maxInstances <= 0) return BUBBLE_MIN_RADIUS;
  const scale = Math.sqrt(Math.min(instances, maxInstances) / maxInstances);
  return BUBBLE_MIN_RADIUS + (BUBBLE_MAX_RADIUS - BUBBLE_MIN_RADIUS) * scale;
}

interface Placed {
  x: number;
  y: number;
  r: number;
}

/**
 * Deterministic greedy placement: circles (largest first) are placed at the first candidate
 * position — scanned outwards from the origin, ring by ring — that overlaps nothing already
 * placed. No physics, no RNG, so the same data always lays out the same way.
 */
function placeAroundOrigin(circles: Placed[], padding: number): void {
  const done: Placed[] = [];
  const fits = (x: number, y: number, r: number): boolean =>
    done.every((p) => Math.hypot(p.x - x, p.y - y) >= p.r + r + padding);

  for (const c of circles) {
    if (done.length === 0) {
      c.x = 0;
      c.y = 0;
      done.push(c);
      continue;
    }
    const step = Math.max(1, c.r / 3);
    let placed = false;
    for (let d = step; d < 4000 && !placed; d += step) {
      const slots = Math.max(8, Math.round((2 * Math.PI * d) / step));
      for (let i = 0; i < slots; i += 1) {
        const angle = (2 * Math.PI * i) / slots;
        const x = d * Math.cos(angle);
        const y = d * Math.sin(angle);
        if (fits(x, y, c.r)) {
          c.x = x;
          c.y = y;
          placed = true;
          break;
        }
      }
    }
    if (!placed) {
      // Unreachable for any realistic view cap; keep it total rather than dropping a bubble.
      c.x = 0;
      c.y = 0;
    }
    done.push(c);
  }
}

interface SizedNode extends Placed {
  node: HierarchyNode;
  children: SizedNode[];
}

function sizeNode(node: HierarchyNode, maxInstances: number): SizedNode {
  const children = node.children.map((c) => sizeNode(c, maxInstances));
  const own = bubbleRadius(node.instances, maxInstances);
  if (children.length === 0) return { node, children, x: 0, y: 0, r: own };
  placeAroundOrigin(children, BUBBLE_PADDING);
  const enclosing = children.reduce((m, c) => Math.max(m, Math.hypot(c.x, c.y) + c.r), 0);
  return { node, children, x: 0, y: 0, r: Math.max(own, enclosing + BUBBLE_PADDING) };
}

/**
 * Pack a class hierarchy into nested circles: a subclass bubble sits INSIDE its superclass
 * bubble, and every bubble's area tracks its direct instance count. Returns absolute
 * coordinates in a tight `width`×`height` box the caller renders as the SVG viewBox.
 */
export function packHierarchy(roots: readonly HierarchyNode[]): BubblePack {
  if (roots.length === 0) return { bubbles: [], width: 0, height: 0 };

  let maxInstances = 0;
  const scan = (list: readonly HierarchyNode[]): void => {
    for (const n of list) {
      maxInstances = Math.max(maxInstances, n.instances);
      scan(n.children);
    }
  };
  scan(roots);

  const sized = roots.map((r) => sizeNode(r, maxInstances));
  placeAroundOrigin(sized, BUBBLE_PADDING);

  const bubbles: PackedBubble[] = [];
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  const emit = (s: SizedNode, ox: number, oy: number, depth: number): void => {
    const x = ox + s.x;
    const y = oy + s.y;
    bubbles.push({
      iri: s.node.iri,
      label: s.node.label,
      instances: s.node.instances,
      totalInstances: subtreeInstances(s.node),
      x,
      y,
      r: s.r,
      depth,
      otherParents: s.node.otherParents,
    });
    minX = Math.min(minX, x - s.r);
    minY = Math.min(minY, y - s.r);
    maxX = Math.max(maxX, x + s.r);
    maxY = Math.max(maxY, y + s.r);
    for (const child of s.children) emit(child, x, y, depth + 1);
  };
  for (const s of sized) emit(s, 0, 0, 0);

  const pad = BUBBLE_PADDING;
  for (const b of bubbles) {
    b.x += pad - minX;
    b.y += pad - minY;
  }
  return {
    bubbles,
    width: maxX - minX + 2 * pad,
    height: maxY - minY + 2 * pad,
  };
}

// ── Class-relationship chord ───────────────────────────────────────────────────────────────────

/** One class arc around the chord circle. Angles are radians, clockwise from 12 o'clock. */
export interface ChordArc {
  iri: string;
  label: string;
  /** Statements on this arc: outgoing + incoming (a self-relationship counts on both sides). */
  value: number;
  startAngle: number;
  endAngle: number;
}

/** One ribbon: all statements from `source` instances to `target` instances, any predicate. */
export interface ChordRibbon {
  source: string;
  target: string;
  count: number;
  sourceStartAngle: number;
  sourceEndAngle: number;
  targetStartAngle: number;
  targetEndAngle: number;
}

export interface ChordModel {
  arcs: ChordArc[];
  ribbons: ChordRibbon[];
  /** Statements represented by the rendered ribbons. */
  shownStatements: number;
  /** Classes dropped by the top-N cap, and the statements that went with them. */
  hiddenClasses: number;
  hiddenStatements: number;
}

/** How many classes the chord shows before it caps (the rest are reported, not silently cut). */
export const CHORD_MAX_CLASSES = 12;

/** Radians of blank space between adjacent class arcs. */
const CHORD_GAP = 0.035;

/** All statements between one ordered class pair, summed over every predicate. */
interface ClassPair {
  source: string;
  target: string;
  count: number;
}

/** The angular span a pair occupies on each of its two arcs, filled in endpoint by endpoint. */
interface PairSpans {
  sourceStart?: number;
  sourceEnd?: number;
  targetStart?: number;
  targetEnd?: number;
}

/**
 * Build the chord model from the class-relationship rows: predicates between the same class
 * pair are summed into ONE ribbon (click it to see the per-predicate breakdown via
 * {@link predicatesBetween}). Classes are ranked by total statements and capped at
 * `maxClasses`; ribbons touching a dropped class are excluded and counted in
 * `hiddenStatements`.
 */
export function buildChordModel(
  relations: readonly RelationRow[],
  maxClasses: number = CHORD_MAX_CLASSES,
): ChordModel {
  const pairs = new Map<string, ClassPair>();
  const totals = new Map<string, number>();
  const bump = (iri: string, n: number) => totals.set(iri, (totals.get(iri) ?? 0) + n);

  for (const r of relations) {
    const key = `${r.source} ${r.target}`;
    const existing = pairs.get(key);
    if (existing) existing.count += r.count;
    else pairs.set(key, { source: r.source, target: r.target, count: r.count });
    bump(r.source, r.count); // outgoing
    bump(r.target, r.count); // incoming (a self-pair therefore counts twice — both endpoints)
  }

  const ranked = [...totals.entries()].sort(
    (a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0),
  );
  const kept = ranked.slice(0, Math.max(0, maxClasses));
  const keptSet = new Set(kept.map(([iri]) => iri));

  const visible = [...pairs.values()].filter(
    (p) => keptSet.has(p.source) && keptSet.has(p.target),
  );
  const shownStatements = visible.reduce((s, p) => s + p.count, 0);
  const allStatements = [...pairs.values()].reduce((s, p) => s + p.count, 0);

  const arcValue = new Map<string, number>();
  for (const p of visible) {
    arcValue.set(p.source, (arcValue.get(p.source) ?? 0) + p.count);
    arcValue.set(p.target, (arcValue.get(p.target) ?? 0) + p.count);
  }
  const order = kept.map(([iri]) => iri).filter((iri) => (arcValue.get(iri) ?? 0) > 0);
  const total = order.reduce((s, iri) => s + (arcValue.get(iri) ?? 0), 0);
  if (total === 0 || order.length === 0) {
    return {
      arcs: [],
      ribbons: [],
      shownStatements: 0,
      hiddenClasses: ranked.length,
      hiddenStatements: allStatements,
    };
  }

  const span = 2 * Math.PI - CHORD_GAP * order.length;
  const arcs: ChordArc[] = [];
  // Per-class cursor: where the next ribbon endpoint on that arc starts.
  const cursor = new Map<string, number>();
  let angle = 0;
  for (const iri of order) {
    const value = arcValue.get(iri) ?? 0;
    const start = angle;
    const end = start + (value / total) * span;
    arcs.push({ iri, label: abbreviateIri(iri), value, startAngle: start, endAngle: end });
    cursor.set(iri, start);
    angle = end + CHORD_GAP;
  }

  const index = new Map<string, number>(order.map((iri, i) => [iri, i] as const));
  // Deterministic endpoint order within an arc: by partner arc index, outgoing before incoming.
  const endpoints = visible.flatMap((p) => [
    { arc: p.source, partner: index.get(p.target) ?? 0, out: true, pair: p },
    { arc: p.target, partner: index.get(p.source) ?? 0, out: false, pair: p },
  ]);
  endpoints.sort(
    (a, b) =>
      (index.get(a.arc) ?? 0) - (index.get(b.arc) ?? 0) ||
      a.partner - b.partner ||
      Number(b.out) - Number(a.out),
  );

  const ribbonSpans = new Map<ClassPair, PairSpans>();
  for (const e of endpoints) {
    const arcSpan = ((arcValue.get(e.arc) ?? 0) / total) * span;
    const width = (e.pair.count / (arcValue.get(e.arc) ?? 1)) * arcSpan;
    const start = cursor.get(e.arc) ?? 0;
    cursor.set(e.arc, start + width);
    const slot = ribbonSpans.get(e.pair) ?? {};
    if (e.out) {
      slot.sourceStart = start;
      slot.sourceEnd = start + width;
    } else {
      slot.targetStart = start;
      slot.targetEnd = start + width;
    }
    ribbonSpans.set(e.pair, slot);
  }

  const ribbons: ChordRibbon[] = visible
    .map((p) => {
      const s = ribbonSpans.get(p) ?? {};
      return {
        source: p.source,
        target: p.target,
        count: p.count,
        sourceStartAngle: s.sourceStart ?? 0,
        sourceEndAngle: s.sourceEnd ?? 0,
        targetStartAngle: s.targetStart ?? 0,
        targetEndAngle: s.targetEnd ?? 0,
      };
    })
    .sort((a, b) => b.count - a.count);

  return {
    arcs,
    ribbons,
    shownStatements,
    hiddenClasses: ranked.length - order.length,
    hiddenStatements: allStatements - shownStatements,
  };
}

/** The per-predicate breakdown behind one chord ribbon, most frequent first. */
export function predicatesBetween(
  relations: readonly RelationRow[],
  source: string,
  target: string,
): RelationRow[] {
  return relations
    .filter((r) => r.source === source && r.target === target)
    .sort(
      (a, b) =>
        b.count - a.count || (a.predicate < b.predicate ? -1 : a.predicate > b.predicate ? 1 : 0),
    );
}

// ── Domain–range ───────────────────────────────────────────────────────────────────────────────

/** What a predicate's observed range is: another class, a datatype, or an untyped literal. */
export type RangeKind = "class" | "datatype" | "literal";

/** One observed domain→range signature of a predicate, with the statement count behind it. */
export interface DomainRangeRow {
  predicate: string;
  predicateLabel: string;
  domain: string;
  domainLabel: string;
  /** The range IRI (class or datatype), or null for an untyped literal. */
  range: string | null;
  rangeLabel: string;
  rangeKind: RangeKind;
  count: number;
}

/**
 * The domain–range panel rows: the OBSERVED signatures (what the data actually contains), not
 * the declared `rdfs:domain`/`rdfs:range` axioms — a distinction the panel states, because the
 * two routinely disagree in real datasets.
 */
export function buildDomainRangeRows(
  relations: readonly RelationRow[],
  literals: readonly LiteralRangeRow[],
): DomainRangeRow[] {
  const out: DomainRangeRow[] = relations.map((r) => ({
    predicate: r.predicate,
    predicateLabel: abbreviateIri(r.predicate),
    domain: r.source,
    domainLabel: abbreviateIri(r.source),
    range: r.target,
    rangeLabel: abbreviateIri(r.target),
    rangeKind: "class" as const,
    count: r.count,
  }));
  for (const l of literals) {
    out.push({
      predicate: l.predicate,
      predicateLabel: abbreviateIri(l.predicate),
      domain: l.source,
      domainLabel: abbreviateIri(l.source),
      range: l.datatype,
      rangeLabel: l.datatype === null ? "literal" : abbreviateIri(l.datatype),
      rangeKind: l.datatype === null ? "literal" : "datatype",
      count: l.count,
    });
  }
  out.sort(
    (a, b) =>
      b.count - a.count ||
      (a.predicateLabel < b.predicateLabel ? -1 : a.predicateLabel > b.predicateLabel ? 1 : 0) ||
      (a.domainLabel < b.domainLabel ? -1 : a.domainLabel > b.domainLabel ? 1 : 0) ||
      (a.rangeLabel < b.rangeLabel ? -1 : a.rangeLabel > b.rangeLabel ? 1 : 0),
  );
  return out;
}

// ── Drill-down ─────────────────────────────────────────────────────────────────────────────────

/** The non-control characters SPARQL's `IRIREF` production excludes. */
const IRIREF_FORBIDDEN = '<>"{}|^`\\';

/**
 * Can this IRI be written as a SPARQL `IRIREF`? An IRI containing a character the grammar
 * forbids inside `<…>` cannot be interpolated safely, so the caller declines to build a query
 * rather than emitting something the engine would mis-parse.
 */
export function isSafeIriRef(iri: string): boolean {
  if (iri.length === 0) return false;
  for (const ch of iri) {
    // The grammar excludes `#x00-#x20` outright, plus the literal characters above.
    if ((ch.codePointAt(0) ?? 0) <= 0x20) return false;
    if (IRIREF_FORBIDDEN.includes(ch)) return false;
  }
  return true;
}

/** The bubble drill-down: instances of a class. Returns null for an IRI we cannot safely embed. */
export function instanceListQuery(classIri: string, limit: number): string | null {
  if (!isSafeIriRef(classIri) || !Number.isInteger(limit) || limit <= 0) return null;
  return `SELECT ?instance WHERE { ?instance a <${classIri}> } LIMIT ${limit}`;
}

/** The instances the drill-down query returned (IRIs and bnodes, in engine order). */
export function parseInstanceRows(results: SparqlResults): Array<{ kind: string; value: string }> {
  const out: Array<{ kind: string; value: string }> = [];
  for (const row of rows(results)) {
    const term = row["instance"];
    if (!term) continue;
    out.push({ kind: term.type, value: term.value });
  }
  return out;
}
