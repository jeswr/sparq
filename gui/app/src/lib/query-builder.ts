// [SONNET-4.6] sq-ixc3.24 (#2700) — the VISUAL QUERY BUILDER's pure core: the diagram model,
// its honest SPARQL serialisation, and the shape-/data-aware predicate suggestion index.
//
// Competitor framing (research/competitive-feature-analysis-2026-07.md line 115): AllegroGraph
// Gruff is the industry reference for "draw the pattern, get the query"; Stardog Explorer adds
// model-driven relationship walking, attribute filters, AND-NOT and GROUP BY. The gap slice this
// closes is emitting **honest SPARQL 1.1 into the existing editor** — no private dialect, no
// hidden rewriting, nothing the user cannot read, edit and run themselves.
//
// HONESTY RULES this module keeps:
//   * every construct it emits is standard SPARQL 1.1 the engine already accepts;
//   * it never silently "fixes" a pattern — a cartesian product, an unconstrained node, an
//     unbound projected variable and a NOT-EXISTS-only variable each surface as a WARNING
//     alongside the query, so the user sees what the diagram actually asks for;
//   * a suggestion always carries its PROVENANCE (`shape` = a SHACL property shape declares it,
//     `data` = characteristic-set introspection observed it in the live store, `both`). A
//     suggestion is never invented — with no shapes and an empty store there are none.
//
// This file is PURE (no React, no engine handle) so it is unit-testable under `node --test`
// (src/lib/query-builder.test.ts); the panel (components/workbench/query-builder-tool.tsx) owns
// the canvas, the engine calls and the two-way editor handoff.

import type { SparqlResults, SparqlTerm } from "@sparq/client";

// ---------------------------------------------------------------------------
// The diagram model.
// ---------------------------------------------------------------------------

/** What a predicate's object looks like — `unknown` when nothing observed or declared says. */
export type ObjectKind = "iri" | "literal" | "unknown";

/** The attribute-filter comparisons the v1 builder offers (all plain SPARQL 1.1). */
export type FilterOp =
  | "eq"
  | "ne"
  | "lt"
  | "le"
  | "gt"
  | "ge"
  | "contains"
  | "starts"
  | "ends"
  | "regex";

/** How the right-hand side of a filter is serialised. */
export type OperandKind = "literal" | "number" | "iri";

export interface AttributeFilter {
  op: FilterOp;
  /** The right-hand side exactly as the user typed it. */
  value: string;
  kind: OperandKind;
}

/** A literal/leaf property hanging off a node (Stardog Explorer's "attribute filters"). */
export interface BuilderAttribute {
  id: string;
  /** Full predicate IRI (no angle brackets). */
  predicate: string;
  /** Result variable name, WITHOUT the leading `?`. */
  variable: string;
  projected: boolean;
  /** Wrap the attribute (and its filter) in OPTIONAL — "if present, it must match". */
  optional: boolean;
  filter: AttributeFilter | null;
}

/** A node drawn on the canvas — one SPARQL variable, optionally typed by a class. */
export interface BuilderNode {
  id: string;
  /** Result variable name, WITHOUT the leading `?`. */
  variable: string;
  /** `rdf:type` constraint (full IRI), or null for an untyped node. */
  classIri: string | null;
  /** Canvas position. Layout only — it never changes the generated SPARQL. */
  x: number;
  y: number;
  projected: boolean;
  attributes: BuilderAttribute[];
}

/** A drawn relationship between two nodes. */
export interface BuilderEdge {
  id: string;
  /** Source node id. */
  from: string;
  /** Target node id. */
  to: string;
  /** Full predicate IRI (no angle brackets). */
  predicate: string;
  /** Further predicates OR-ed with `predicate` — emitted as a UNION of one-triple groups. */
  alternates: string[];
  optional: boolean;
  /** AND-NOT (Stardog Explorer): emit `FILTER NOT EXISTS { … }` instead of a matching pattern. */
  negated: boolean;
}

export type AggregateFn = "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "SAMPLE" | "GROUP_CONCAT";

export interface BuilderAggregate {
  id: string;
  fn: AggregateFn;
  /** Variable to aggregate, WITHOUT `?`. `COUNT` also accepts `*`. */
  variable: string;
  distinct: boolean;
  /** Result variable name, WITHOUT `?`. */
  alias: string;
}

export interface BuilderModel {
  /** Extra prefix → namespace bindings on top of {@link DEFAULT_PREFIXES}. */
  prefixes: Record<string, string>;
  nodes: BuilderNode[];
  edges: BuilderEdge[];
  aggregates: BuilderAggregate[];
  /** Variables (WITHOUT `?`) to GROUP BY. Only meaningful with at least one aggregate. */
  groupBy: string[];
  distinct: boolean;
  /** `null` = no LIMIT clause. */
  limit: number | null;
}

/** The generated query plus the honest notes about what the diagram actually asks for. */
export interface BuiltQuery {
  sparql: string;
  /** Non-blocking notes (cartesian product, unconstrained node, unbound projection, …). */
  warnings: string[];
  /** Projected variables in order, WITHOUT `?`. Empty when the query projects `*`. */
  projection: string[];
}

/** Prefixes the builder may use when compacting an IRI (emitted only when actually used). */
export const DEFAULT_PREFIXES: Record<string, string> = {
  rdf: "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
  rdfs: "http://www.w3.org/2000/01/rdf-schema#",
  owl: "http://www.w3.org/2002/07/owl#",
  xsd: "http://www.w3.org/2001/XMLSchema#",
  sh: "http://www.w3.org/ns/shacl#",
  foaf: "http://xmlns.com/foaf/0.1/",
  schema: "https://schema.org/",
  dcterms: "http://purl.org/dc/terms/",
};

/** The LIMIT a freshly-created diagram carries (a browsing default, not a result bound). */
export const DEFAULT_LIMIT = 100;

/** An empty diagram — what the tool opens with. */
export function emptyModel(): BuilderModel {
  return {
    prefixes: {},
    nodes: [],
    edges: [],
    aggregates: [],
    groupBy: [],
    distinct: false,
    limit: DEFAULT_LIMIT,
  };
}

// ---------------------------------------------------------------------------
// Names, ids and terms.
// ---------------------------------------------------------------------------

/** The local part of an IRI (after the last `#` or `/`), or the whole IRI when it has none. */
export function localName(iri: string): string {
  const trimmed = iri.replace(/[#/]+$/, "");
  const match = trimmed.match(/[#/]([^#/]+)$/);
  return match ? match[1] : trimmed;
}

/** Coerce arbitrary text into a legal SPARQL variable name (no `?`). */
export function sanitizeVariable(raw: string, fallback = "v"): string {
  const cleaned = raw.replace(/[^A-Za-z0-9_]/g, "");
  if (cleaned === "") return fallback;
  return /^[0-9]/.test(cleaned) ? `_${cleaned}` : cleaned;
}

/** `base`, or `base2`, `base3`, … — the first name not already `taken`. */
export function uniqueVariable(base: string, taken: Iterable<string>): string {
  const used = new Set(taken);
  const start = sanitizeVariable(base);
  if (!used.has(start)) return start;
  for (let n = 2; ; n += 1) {
    const candidate = `${start}${n}`;
    if (!used.has(candidate)) return candidate;
  }
}

/** Every variable name the model currently uses (nodes, attributes, aggregate aliases). */
export function modelVariables(model: BuilderModel): string[] {
  const names: string[] = [];
  for (const node of model.nodes) {
    names.push(node.variable);
    for (const attribute of node.attributes) names.push(attribute.variable);
  }
  for (const aggregate of model.aggregates) names.push(aggregate.alias);
  return names;
}

/** A deterministic next id (`n1`, `n2`, …) — no randomness, so renders are reproducible. */
export function nextId(prefix: string, existing: Iterable<string>): string {
  let max = 0;
  for (const id of existing) {
    if (!id.startsWith(prefix)) continue;
    const n = Number.parseInt(id.slice(prefix.length), 10);
    if (Number.isFinite(n) && n > max) max = n;
  }
  return `${prefix}${max + 1}`;
}

/** Escape a string for a SPARQL `"…"` literal. */
export function escapeLiteral(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
}

const NUMERIC_RE = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/;
/** Conservative PN_LOCAL: only names that are unambiguously safe get compacted. */
const SAFE_LOCAL_RE = /^[A-Za-z_][A-Za-z0-9_-]*$/;

/**
 * Writes IRIs as `prefix:local` where a known prefix matches (and the local part is safe),
 * otherwise as `<full-iri>`, recording which prefixes were actually used so the PREFIX header
 * declares those and only those.
 */
class IriWriter {
  private readonly entries: [string, string][];
  readonly used = new Set<string>();

  constructor(prefixes: Record<string, string>) {
    // Longest namespace first so the most specific prefix wins.
    this.entries = Object.entries(prefixes).sort((a, b) => b[1].length - a[1].length);
  }

  write(iri: string): string {
    for (const [prefix, namespace] of this.entries) {
      if (!iri.startsWith(namespace)) continue;
      const local = iri.slice(namespace.length);
      if (!SAFE_LOCAL_RE.test(local)) continue;
      this.used.add(prefix);
      return `${prefix}:${local}`;
    }
    return `<${iri}>`;
  }

  header(prefixes: Record<string, string>): string[] {
    return Object.entries(prefixes)
      .filter(([prefix]) => this.used.has(prefix))
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([prefix, namespace]) => `PREFIX ${prefix}: <${namespace}>`);
  }
}

// ---------------------------------------------------------------------------
// SPARQL serialisation.
// ---------------------------------------------------------------------------

interface EmitCtx {
  w: IriWriter;
  warn: (message: string) => void;
}

function indent(lines: string[], depth: number): string[] {
  const pad = "  ".repeat(depth);
  return lines.map((line) => (line === "" ? line : `${pad}${line}`));
}

function triple(subject: string, predicate: string, object: string): string {
  return `${subject} ${predicate} ${object} .`;
}

/** `KEYWORD { … }` — kept on one line for a single-pattern group. */
function wrapBlock(keyword: string, lines: string[]): string[] {
  if (lines.length === 1) return [`${keyword} { ${lines[0]} }`];
  return [`${keyword} {`, ...indent(lines, 1), "}"];
}

function operand(filter: AttributeFilter, ctx: EmitCtx): string {
  if (filter.kind === "iri") return ctx.w.write(filter.value);
  if (filter.kind === "number") {
    if (NUMERIC_RE.test(filter.value.trim())) return filter.value.trim();
    ctx.warn(`"${filter.value}" is not a number — compared as a string literal instead.`);
    return `"${escapeLiteral(filter.value)}"`;
  }
  return `"${escapeLiteral(filter.value)}"`;
}

const COMPARISON: Partial<Record<FilterOp, string>> = {
  eq: "=",
  ne: "!=",
  lt: "<",
  le: "<=",
  gt: ">",
  ge: ">=",
};

/** One FILTER line constraining `?variable`. */
function filterLine(variable: string, filter: AttributeFilter, ctx: EmitCtx): string {
  const ref = `?${variable}`;
  const comparison = COMPARISON[filter.op];
  if (comparison) return `FILTER(${ref} ${comparison} ${operand(filter, ctx)})`;
  // The string functions take a string right-hand side whatever the operand kind says.
  const text = `"${escapeLiteral(filter.value)}"`;
  switch (filter.op) {
    case "contains":
      return `FILTER(CONTAINS(STR(${ref}), ${text}))`;
    case "starts":
      return `FILTER(STRSTARTS(STR(${ref}), ${text}))`;
    case "ends":
      return `FILTER(STRENDS(STR(${ref}), ${text}))`;
    default:
      return `FILTER(REGEX(STR(${ref}), ${text}, "i"))`;
  }
}

function aggregateExpression(aggregate: BuilderAggregate): string {
  const alias = sanitizeVariable(aggregate.alias, "agg");
  const distinct = aggregate.distinct ? "DISTINCT " : "";
  const argument =
    aggregate.variable.trim() === "*" ? "*" : `?${sanitizeVariable(aggregate.variable)}`;
  return `(${aggregate.fn}(${distinct}${argument}) AS ?${alias})`;
}

/**
 * Disjoint-set roots over the nodes joined by non-negated edges (NOT EXISTS never joins). Nodes
 * a scoped group owns (`scopedOwner`) are not counted: they live inside that group rather than
 * standing alone in the top-level body, so they are never a disconnected group of their own.
 */
function connectedComponents(model: BuilderModel, scopedOwner: Map<string, BuilderEdge>): number {
  const parent = new Map<string, string>();
  const find = (id: string): string => {
    let root = id;
    while (parent.get(root) !== undefined && parent.get(root) !== root) {
      root = parent.get(root) as string;
    }
    return root;
  };
  for (const node of model.nodes) parent.set(node.id, node.id);
  for (const edge of model.edges) {
    if (edge.negated) continue;
    if (!parent.has(edge.from) || !parent.has(edge.to)) continue;
    const a = find(edge.from);
    const b = find(edge.to);
    if (a !== b) parent.set(a, b);
  }
  const roots = new Set<string>();
  for (const node of model.nodes) {
    if (!scopedOwner.has(node.id)) roots.add(find(node.id));
  }
  return roots.size;
}

const EMPTY_QUERY = `# The canvas is empty — add a node to start building a pattern.
SELECT * WHERE {
}`;

/**
 * Serialise the diagram to standard SPARQL 1.1, plus the honest warnings about what the pattern
 * will actually do. Deterministic: the same model always produces byte-identical output.
 */
export function buildSparql(model: BuilderModel): BuiltQuery {
  const warnings: string[] = [];
  const warn = (message: string) => {
    if (!warnings.includes(message)) warnings.push(message);
  };

  if (model.nodes.length === 0) {
    warn("The canvas is empty — nothing to query yet.");
    return { sparql: EMPTY_QUERY, warnings, projection: [] };
  }

  const prefixes = { ...DEFAULT_PREFIXES, ...model.prefixes };
  const ctx: EmitCtx = { w: new IriWriter(prefixes), warn };
  const body: string[] = [];

  const nodeById = new Map<string, BuilderNode>();
  for (const node of model.nodes) nodeById.set(node.id, node);
  const nodeVar = (node: BuilderNode) => `?${sanitizeVariable(node.variable, "s")}`;

  // Two elements sharing a variable name are joined — real, but rarely intended: say so.
  const seenVariables = new Set<string>();
  for (const name of modelVariables(model)) {
    const clean = sanitizeVariable(name);
    if (seenVariables.has(clean)) {
      warn(`?${clean} is used by more than one element — those elements are joined on it.`);
    }
    seenVariables.add(clean);
  }

  const incident = new Map<string, BuilderEdge[]>();
  for (const edge of model.edges) {
    for (const id of [edge.from, edge.to]) {
      const list = incident.get(id);
      if (list) list.push(edge);
      else incident.set(id, [edge]);
    }
  }

  // Which relationship group OWNS a walked-to node. A target reached SOLELY through one OPTIONAL
  // or excluded edge belongs INSIDE that edge's group: emitted at the top level, its class triple
  // and attributes would bind it independently of the group, which turns OPTIONAL into a cross
  // product and FILTER NOT EXISTS into a per-pair test rather than "has no such relationship".
  const scopedOwner = new Map<string, BuilderEdge>();
  for (const edge of model.edges) {
    if (!edge.optional && !edge.negated) continue;
    if (!nodeById.has(edge.from) || !nodeById.has(edge.to)) continue;
    // An edge that emits nothing (no predicate) can own nothing — the constraints would vanish.
    if (![edge.predicate, ...edge.alternates].some((p) => p.trim() !== "")) continue;
    // Anything the user constrained elsewhere too stays where they drew it. (A self-loop has the
    // node incident twice, so it never qualifies.)
    if ((incident.get(edge.to) ?? []).length !== 1) continue;
    scopedOwner.set(edge.to, edge);
  }

  const classLine = (node: BuilderNode): string[] =>
    node.classIri ? [triple(nodeVar(node), "a", ctx.w.write(node.classIri))] : [];

  /** A node's attribute patterns. A filter on an OPTIONAL attribute goes INSIDE the OPTIONAL
   *  block, so the row survives and the variable binds only when it matches. */
  const attributePatterns = (node: BuilderNode): string[] => {
    const out: string[] = [];
    for (const attribute of node.attributes) {
      if (attribute.predicate.trim() === "") {
        warn(`An attribute on ${nodeVar(node)} has no predicate — it was skipped.`);
        continue;
      }
      const variable = sanitizeVariable(attribute.variable, "value");
      const lines = [triple(nodeVar(node), ctx.w.write(attribute.predicate), `?${variable}`)];
      if (attribute.filter) lines.push(filterLine(variable, attribute.filter, ctx));
      out.push(...(attribute.optional ? wrapBlock("OPTIONAL", lines) : lines));
    }
    return out;
  };

  // 1. Class constraints (except those owned by a scoped relationship group).
  for (const node of model.nodes) {
    if (!scopedOwner.has(node.id)) body.push(...classLine(node));
  }

  // 2. Relationships — each scoped group carrying the complete pattern for the node it owns.
  for (const edge of model.edges) {
    const from = nodeById.get(edge.from);
    const to = nodeById.get(edge.to);
    if (!from || !to) {
      warn("A relationship references a node that is no longer on the canvas — it was skipped.");
      continue;
    }
    const predicates = [edge.predicate, ...edge.alternates].filter((p) => p.trim() !== "");
    if (predicates.length === 0) {
      warn(`The relationship ${nodeVar(from)} → ${nodeVar(to)} has no predicate — it was skipped.`);
      continue;
    }
    let core: string[];
    if (predicates.length === 1) {
      core = [triple(nodeVar(from), ctx.w.write(predicates[0]), nodeVar(to))];
    } else {
      core = predicates
        .map((p) => `{ ${triple(nodeVar(from), ctx.w.write(p), nodeVar(to))} }`)
        .flatMap((block, i) => (i === 0 ? [block] : ["UNION", block]));
    }
    const walked = scopedOwner.get(edge.to) === edge ? nodeById.get(edge.to) : undefined;
    if (walked) core.push(...classLine(walked), ...attributePatterns(walked));
    if (edge.negated) {
      if (edge.optional) {
        warn(
          `${nodeVar(from)} → ${nodeVar(to)} is both OPTIONAL and excluded — exclusion wins (FILTER NOT EXISTS).`,
        );
      }
      body.push(...wrapBlock("FILTER NOT EXISTS", core));
    } else if (edge.optional) {
      body.push(...wrapBlock("OPTIONAL", core));
    } else {
      body.push(...core);
    }
  }

  // 3. Attributes (and their filters) of every node a scoped group does not own.
  for (const node of model.nodes) {
    if (!scopedOwner.has(node.id)) body.push(...attributePatterns(node));
  }

  // 4. Honesty warnings about the SHAPE of the pattern.
  for (const node of model.nodes) {
    const edges = incident.get(node.id) ?? [];
    const bare = !node.classIri && node.attributes.length === 0;
    // A node scoped inside FILTER NOT EXISTS cannot escape it, whether or not it is otherwise bare.
    const insideNotExists = scopedOwner.get(node.id)?.negated === true;
    if (bare && edges.length === 0) {
      warn(`${nodeVar(node)} is unconstrained — it matches every term in the store.`);
    } else if (node.projected && (insideNotExists || (bare && edges.every((e) => e.negated)))) {
      warn(
        `${nodeVar(node)} only appears inside FILTER NOT EXISTS — it cannot be bound in the result.`,
      );
    }
  }
  const components = connectedComponents(model, scopedOwner);
  if (model.nodes.length > 1 && components > 1) {
    warn(`The canvas has ${components} disconnected groups — the result is their cross product.`);
  }

  // 5. Projection: an aggregate panel (GROUP BY) or the ticked variables.
  //    Only COUNT takes `*` in SPARQL 1.1, so any other wildcard aggregate is dropped with a
  //    note rather than emitted as the non-standard `SUM(*)` the grammar rejects.
  const aggregates = model.aggregates.filter((aggregate) => {
    if (aggregate.fn === "COUNT" || aggregate.variable.trim() !== "*") return true;
    warn(
      `${aggregate.fn}(*) is not legal SPARQL — only COUNT takes *. Pick a variable; the aggregate was skipped.`,
    );
    return false;
  });
  const distinct = model.distinct ? "DISTINCT " : "";
  let selectLine: string;
  let projection: string[];
  let groupByLine: string | null = null;

  if (aggregates.length > 0) {
    const groupVars: string[] = [];
    for (const name of model.groupBy) {
      const clean = sanitizeVariable(name);
      if (!groupVars.includes(clean)) groupVars.push(clean);
    }
    const aliases = aggregates.map((a) => sanitizeVariable(a.alias, "agg"));
    for (const aggregate of aggregates) {
      const argument = aggregate.variable.trim();
      if (argument !== "*" && !seenVariables.has(sanitizeVariable(argument))) {
        warn(`?${sanitizeVariable(argument)} is aggregated but never bound by the pattern.`);
      }
    }
    for (const node of model.nodes) {
      if (node.projected && !groupVars.includes(sanitizeVariable(node.variable, "s"))) {
        warn(
          `${nodeVar(node)} is ticked but not grouped — an aggregate query projects only GROUP BY variables and aggregates.`,
        );
      }
    }
    selectLine = `SELECT ${distinct}${[
      ...groupVars.map((v) => `?${v}`),
      ...aggregates.map(aggregateExpression),
    ].join(" ")}`;
    groupByLine = groupVars.length > 0 ? `GROUP BY ${groupVars.map((v) => `?${v}`).join(" ")}` : null;
    projection = [...groupVars, ...aliases];
  } else {
    projection = [];
    for (const node of model.nodes) {
      const clean = sanitizeVariable(node.variable, "s");
      if (node.projected && !projection.includes(clean)) projection.push(clean);
      for (const attribute of node.attributes) {
        const attributeVar = sanitizeVariable(attribute.variable, "value");
        if (attribute.projected && !projection.includes(attributeVar)) projection.push(attributeVar);
      }
    }
    if (projection.length === 0) {
      warn("Nothing is ticked for the result — projecting every variable (SELECT *).");
      selectLine = `SELECT ${distinct}*`;
    } else {
      selectLine = `SELECT ${distinct}${projection.map((v) => `?${v}`).join(" ")}`;
    }
  }

  const lines: string[] = [];
  const header = ctx.w.header(prefixes);
  if (header.length > 0) lines.push(...header, "");
  lines.push(selectLine, "WHERE {", ...indent(body, 1), "}");
  if (groupByLine) lines.push(groupByLine);
  if (model.limit !== null && model.limit > 0) lines.push(`LIMIT ${Math.floor(model.limit)}`);

  return { sparql: lines.join("\n"), warnings, projection };
}

// ---------------------------------------------------------------------------
// Model edits the canvas performs (kept here so they are unit-testable).
// ---------------------------------------------------------------------------

/** Add a node, optionally typed by a class. Returns the new model and the new node's id. */
export function addNode(
  model: BuilderModel,
  options: { classIri?: string | null; x?: number; y?: number } = {},
): { model: BuilderModel; nodeId: string } {
  const classIri = options.classIri ?? null;
  const nodeId = nextId("n", model.nodes.map((n) => n.id));
  const base = classIri ? localName(classIri).toLowerCase() : "node";
  const node: BuilderNode = {
    id: nodeId,
    variable: uniqueVariable(base, modelVariables(model)),
    classIri,
    x: options.x ?? 40 + model.nodes.length * 30,
    y: options.y ?? 40 + model.nodes.length * 30,
    projected: true,
    attributes: [],
  };
  return { model: { ...model, nodes: [...model.nodes, node] }, nodeId };
}

/**
 * Walk a relationship (the Stardog Explorer affordance): add a target node for `predicate` and
 * the edge reaching it, in one step. `targetClass` types the new node when the suggestion knows
 * it (a SHACL `sh:class`), which in turn drives the NEXT step's suggestions.
 */
export function addRelationship(
  model: BuilderModel,
  fromNodeId: string,
  predicate: string,
  options: { targetClass?: string | null; x?: number; y?: number } = {},
): { model: BuilderModel; nodeId: string; edgeId: string } {
  const targetClass = options.targetClass ?? null;
  const source = model.nodes.find((n) => n.id === fromNodeId);
  const added = addNode(model, {
    classIri: targetClass,
    x: options.x ?? (source ? source.x + 260 : undefined),
    y: options.y ?? (source ? source.y + 40 * countEdgesFrom(model, fromNodeId) : undefined),
  });
  // Untyped targets read better named after the predicate than "node".
  const nodes = added.model.nodes.map((node) =>
    node.id === added.nodeId && !targetClass
      ? {
          ...node,
          variable: uniqueVariable(localName(predicate).toLowerCase(), modelVariables(model)),
        }
      : node,
  );
  const edgeId = nextId("e", model.edges.map((e) => e.id));
  const edge: BuilderEdge = {
    id: edgeId,
    from: fromNodeId,
    to: added.nodeId,
    predicate,
    alternates: [],
    optional: false,
    negated: false,
  };
  return {
    model: { ...added.model, nodes, edges: [...added.model.edges, edge] },
    nodeId: added.nodeId,
    edgeId,
  };
}

/**
 * Connect two nodes that are ALREADY on the canvas (as opposed to {@link addRelationship}, which
 * walks to a fresh one) — the operation that lets a diagram express a cycle rather than a tree.
 */
export function connectNodes(
  model: BuilderModel,
  fromNodeId: string,
  toNodeId: string,
  predicate: string,
): { model: BuilderModel; edgeId: string } {
  const edgeId = nextId("e", model.edges.map((e) => e.id));
  const edge: BuilderEdge = {
    id: edgeId,
    from: fromNodeId,
    to: toNodeId,
    predicate,
    alternates: [],
    optional: false,
    negated: false,
  };
  return { model: { ...model, edges: [...model.edges, edge] }, edgeId };
}

function countEdgesFrom(model: BuilderModel, nodeId: string): number {
  return model.edges.filter((edge) => edge.from === nodeId).length;
}

/** Attach an attribute (a literal/leaf property) to a node. */
export function addAttribute(
  model: BuilderModel,
  nodeId: string,
  predicate: string,
): { model: BuilderModel; attributeId: string } {
  const existingIds = model.nodes.flatMap((node) => node.attributes.map((a) => a.id));
  const attributeId = nextId("a", existingIds);
  const attribute: BuilderAttribute = {
    id: attributeId,
    predicate,
    variable: uniqueVariable(localName(predicate).toLowerCase(), modelVariables(model)),
    projected: true,
    optional: false,
    filter: null,
  };
  return {
    model: {
      ...model,
      nodes: model.nodes.map((node) =>
        node.id === nodeId ? { ...node, attributes: [...node.attributes, attribute] } : node,
      ),
    },
    attributeId,
  };
}

/** The variables an aggregate may be taken over — every bound name that is not itself an alias. */
export function aggregatableVariables(model: BuilderModel): string[] {
  return modelVariables(model).filter(
    (name) => !model.aggregates.some((aggregate) => aggregate.alias === name),
  );
}

/**
 * Change an aggregate's function, keeping the pair legal: only `COUNT` accepts `*`, so moving
 * away from COUNT coerces a wildcard onto the first variable the pattern binds. With no bound
 * variable to move to, the wildcard is left in place and {@link buildSparql} reports it rather
 * than emitting the non-standard `SUM(*)`.
 */
export function setAggregateFn(
  model: BuilderModel,
  aggregateId: string,
  fn: AggregateFn,
): BuilderModel {
  const bound = aggregatableVariables(model);
  return {
    ...model,
    aggregates: model.aggregates.map((aggregate) => {
      if (aggregate.id !== aggregateId) return aggregate;
      if (fn === "COUNT" || aggregate.variable.trim() !== "*") return { ...aggregate, fn };
      return { ...aggregate, fn, variable: bound[0] ?? aggregate.variable };
    }),
  };
}

/** Remove a node, every edge touching it, and any GROUP BY entry that named its variable. */
export function removeNode(model: BuilderModel, nodeId: string): BuilderModel {
  const node = model.nodes.find((n) => n.id === nodeId);
  const dropped = node
    ? new Set([node.variable, ...node.attributes.map((a) => a.variable)])
    : new Set<string>();
  return {
    ...model,
    nodes: model.nodes.filter((n) => n.id !== nodeId),
    edges: model.edges.filter((e) => e.from !== nodeId && e.to !== nodeId),
    groupBy: model.groupBy.filter((v) => !dropped.has(v)),
  };
}

// ---------------------------------------------------------------------------
// Introspection → suggestions.
// ---------------------------------------------------------------------------

/** A class observed in the live store (`instances` is the measured count, never an estimate). */
export interface ClassSummary {
  iri: string;
  instances: number | null;
}

/** One ranked predicate offer, always carrying where it came from. */
export interface PredicateSuggestion {
  predicate: string;
  /** `shape` = declared by a SHACL property shape; `data` = observed in the store; `both`. */
  source: "shape" | "data" | "both";
  /** Uses observed by characteristic-set introspection; null when only a shape declares it. */
  count: number | null;
  objectKind: ObjectKind;
  /** `sh:class` — the class of the object, when a shape declares one. */
  objectClass: string | null;
  /** `sh:datatype`, when a shape declares one. */
  datatype: string | null;
  /** `sh:name`, when a shape declares one. */
  label: string | null;
  /** `sh:minCount >= 1`. */
  required: boolean;
}

export interface SuggestionIndex {
  classes: ClassSummary[];
  /** class IRI → ranked suggestions. */
  byClass: Record<string, PredicateSuggestion[]>;
  /** Predicates observed without class context — the offer for an untyped node. */
  unscoped: PredicateSuggestion[];
  /** True only when at least one SHACL property shape actually contributed. */
  shapesPresent: boolean;
}

/** An empty index — what the tool shows before introspection has run. */
export function emptySuggestionIndex(): SuggestionIndex {
  return { classes: [], byClass: {}, unscoped: [], shapesPresent: false };
}

const RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/** Classes in the live store, most-instantiated first. */
export const CLASSES_QUERY = `SELECT ?class (COUNT(?s) AS ?instances) WHERE {
  ?s a ?class .
}
GROUP BY ?class
ORDER BY DESC(?instances)
LIMIT 200`;

/**
 * CHARACTERISTIC SETS — for each class, the predicates its instances actually use, with a
 * sampled object so the picker can tell a relationship (IRI object) from an attribute (literal).
 */
export const CHARACTERISTIC_SETS_QUERY = `PREFIX rdf: <${RDF_TYPE.slice(0, -"type".length)}>
SELECT ?class ?p (COUNT(*) AS ?uses) (SAMPLE(?o) AS ?sample) WHERE {
  ?s a ?class ;
     ?p ?o .
  FILTER(?p != rdf:type)
}
GROUP BY ?class ?p
ORDER BY ?class DESC(?uses)
LIMIT 2000`;

/** Predicates without class context — the offer for an untyped node. */
export const UNSCOPED_PREDICATES_QUERY = `PREFIX rdf: <${RDF_TYPE.slice(0, -"type".length)}>
SELECT ?p (COUNT(*) AS ?uses) (SAMPLE(?o) AS ?sample) WHERE {
  ?s ?p ?o .
  FILTER(?p != rdf:type)
}
GROUP BY ?p
ORDER BY DESC(?uses)
LIMIT 200`;

/**
 * SHACL property shapes present IN THE STORE (a shapes graph the user imported). Only IRI
 * `sh:path`s are offered — a property-path shape is not a single predicate the canvas can draw.
 */
export const SHAPES_QUERY = `PREFIX sh: <http://www.w3.org/ns/shacl#>
SELECT ?targetClass ?path ?datatype ?objectClass ?nodeKind ?minCount ?name WHERE {
  ?shape sh:targetClass ?targetClass ;
         sh:property ?property .
  ?property sh:path ?path .
  FILTER(isIRI(?path))
  OPTIONAL { ?property sh:datatype ?datatype . }
  OPTIONAL { ?property sh:class ?objectClass . }
  OPTIONAL { ?property sh:nodeKind ?nodeKind . }
  OPTIONAL { ?property sh:minCount ?minCount . }
  OPTIONAL { ?property sh:name ?name . }
}
LIMIT 2000`;

function term(row: Record<string, SparqlTerm>, key: string): SparqlTerm | null {
  return row[key] ?? null;
}

function value(row: Record<string, SparqlTerm>, key: string): string | null {
  const found = row[key];
  return found ? found.value : null;
}

function rows(results: SparqlResults | null | undefined): Record<string, SparqlTerm>[] {
  return results?.results?.bindings ?? [];
}

function countOf(row: Record<string, SparqlTerm>, key: string): number | null {
  const raw = value(row, key);
  if (raw === null) return null;
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) ? parsed : null;
}

/** Classes from a {@link CLASSES_QUERY} result. */
export function parseClasses(results: SparqlResults | null): ClassSummary[] {
  const out: ClassSummary[] = [];
  const seen = new Set<string>();
  for (const row of rows(results)) {
    const iri = value(row, "class");
    if (iri === null || seen.has(iri)) continue;
    seen.add(iri);
    out.push({ iri, instances: countOf(row, "instances") });
  }
  return out;
}

function kindOfSample(sample: SparqlTerm | null): ObjectKind {
  if (!sample) return "unknown";
  if (sample.type === "literal") return "literal";
  if (sample.type === "uri" || sample.type === "bnode") return "iri";
  return "unknown";
}

/** A SHACL property shape flattened to the fields the picker uses. */
export interface ShapeProperty {
  targetClass: string;
  path: string;
  datatype: string | null;
  objectClass: string | null;
  nodeKind: string | null;
  minCount: number | null;
  name: string | null;
}

/** SHACL property shapes from a {@link SHAPES_QUERY} result. */
export function parseShapeProperties(results: SparqlResults | null): ShapeProperty[] {
  const out: ShapeProperty[] = [];
  for (const row of rows(results)) {
    const targetClass = value(row, "targetClass");
    const path = value(row, "path");
    if (targetClass === null || path === null) continue;
    out.push({
      targetClass,
      path,
      datatype: value(row, "datatype"),
      objectClass: value(row, "objectClass"),
      nodeKind: value(row, "nodeKind"),
      minCount: countOf(row, "minCount"),
      name: value(row, "name"),
    });
  }
  return out;
}

function shapeObjectKind(shape: ShapeProperty): ObjectKind {
  if (shape.datatype) return "literal";
  if (shape.objectClass) return "iri";
  if (shape.nodeKind) {
    if (shape.nodeKind.endsWith("Literal")) return "literal";
    if (shape.nodeKind.endsWith("IRI") || shape.nodeKind.endsWith("BlankNode")) return "iri";
  }
  return "unknown";
}

/** Merge one data-observed predicate into a per-class bucket. */
function mergeInto(
  bucket: Map<string, PredicateSuggestion>,
  suggestion: PredicateSuggestion,
): void {
  const existing = bucket.get(suggestion.predicate);
  if (!existing) {
    bucket.set(suggestion.predicate, suggestion);
    return;
  }
  bucket.set(suggestion.predicate, {
    predicate: existing.predicate,
    source: existing.source === suggestion.source ? existing.source : "both",
    count: suggestion.count ?? existing.count,
    objectKind: existing.objectKind !== "unknown" ? existing.objectKind : suggestion.objectKind,
    objectClass: existing.objectClass ?? suggestion.objectClass,
    datatype: existing.datatype ?? suggestion.datatype,
    label: existing.label ?? suggestion.label,
    required: existing.required || suggestion.required,
  });
}

/** Shape-declared first, then most-used; ties broken alphabetically so the order is stable. */
function rank(a: PredicateSuggestion, b: PredicateSuggestion): number {
  const weight = (s: PredicateSuggestion) =>
    (s.required ? 0 : 1) + (s.source === "both" ? 0 : s.source === "shape" ? 1 : 3);
  const byWeight = weight(a) - weight(b);
  if (byWeight !== 0) return byWeight;
  const byCount = (b.count ?? 0) - (a.count ?? 0);
  if (byCount !== 0) return byCount;
  return a.predicate.localeCompare(b.predicate);
}

/**
 * Build the predicate-suggestion index from the introspection results. Every input is optional:
 * with no shapes the index is data-only, with an empty store it is shape-only, and with neither
 * it is empty — the picker then says so rather than offering an invented vocabulary.
 */
export function buildSuggestionIndex(input: {
  classes?: SparqlResults | null;
  characteristicSets?: SparqlResults | null;
  unscoped?: SparqlResults | null;
  shapes?: SparqlResults | null;
}): SuggestionIndex {
  const classes = parseClasses(input.classes ?? null);
  const buckets = new Map<string, Map<string, PredicateSuggestion>>();
  const bucketFor = (classIri: string) => {
    const found = buckets.get(classIri);
    if (found) return found;
    const created = new Map<string, PredicateSuggestion>();
    buckets.set(classIri, created);
    return created;
  };

  // Data: the characteristic sets actually observed in the store.
  for (const row of rows(input.characteristicSets)) {
    const classIri = value(row, "class");
    const predicate = value(row, "p");
    if (classIri === null || predicate === null) continue;
    mergeInto(bucketFor(classIri), {
      predicate,
      source: "data",
      count: countOf(row, "uses"),
      objectKind: kindOfSample(term(row, "sample")),
      objectClass: null,
      datatype: null,
      label: null,
      required: false,
    });
  }

  // Shapes: what a SHACL property shape declares for the class (the shape-aware half).
  const shapeProperties = parseShapeProperties(input.shapes ?? null);
  for (const shape of shapeProperties) {
    mergeInto(bucketFor(shape.targetClass), {
      predicate: shape.path,
      source: "shape",
      count: null,
      objectKind: shapeObjectKind(shape),
      objectClass: shape.objectClass,
      datatype: shape.datatype,
      label: shape.name,
      required: (shape.minCount ?? 0) >= 1,
    });
    // A shape may target a class the store has no instances of yet — still offer the class.
    if (!classes.some((c) => c.iri === shape.targetClass)) {
      classes.push({ iri: shape.targetClass, instances: null });
    }
  }

  const unscopedBucket = new Map<string, PredicateSuggestion>();
  for (const row of rows(input.unscoped)) {
    const predicate = value(row, "p");
    if (predicate === null) continue;
    mergeInto(unscopedBucket, {
      predicate,
      source: "data",
      count: countOf(row, "uses"),
      objectKind: kindOfSample(term(row, "sample")),
      objectClass: null,
      datatype: null,
      label: null,
      required: false,
    });
  }

  const byClass: Record<string, PredicateSuggestion[]> = {};
  for (const [classIri, bucket] of buckets) {
    byClass[classIri] = [...bucket.values()].sort(rank);
  }

  return {
    classes,
    byClass,
    unscoped: [...unscopedBucket.values()].sort(rank),
    shapesPresent: shapeProperties.length > 0,
  };
}

/** The ranked offer for a node — its class's suggestions, or the unscoped ones when untyped. */
export function suggestionsFor(
  index: SuggestionIndex,
  classIri: string | null,
): PredicateSuggestion[] {
  if (classIri === null) return index.unscoped;
  return index.byClass[classIri] ?? index.unscoped;
}

/** Split an offer into relationship-shaped (IRI object) and attribute-shaped (literal) halves. */
export function partitionSuggestions(suggestions: PredicateSuggestion[]): {
  relationships: PredicateSuggestion[];
  attributes: PredicateSuggestion[];
} {
  return {
    relationships: suggestions.filter((s) => s.objectKind !== "literal"),
    attributes: suggestions.filter((s) => s.objectKind !== "iri"),
  };
}
