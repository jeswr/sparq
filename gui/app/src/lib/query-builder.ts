// [OPUS-5] sq-ixc3.24 — the visual query builder's PURE core: the canvas MODEL, its lowering to
// HONEST SPARQL 1.1, and the live-store introspection (characteristic sets + SHACL shapes) that
// drives the predicate pickers.
//
// Competitor parity (bead sq-ixc3.24, epic sq-lsp7k): AllegroGraph **Gruff** is the industry
// reference for "draw the pattern, get the query"; Stardog **Explorer**'s model-driven builder
// adds walking relationship paths, attribute filters, AND-NOT and GROUP BY. This module is the
// half of that which must be *correct*, so it is deliberately React-free / DOM-free / wasm-free
// and covered by `node --test` (see `query-builder.test.ts`); the canvas component
// (`components/workbench/query-builder-tool.tsx`) only draws and edits the model.
//
// HONESTY RULES this module holds to:
//   * **No hidden dialect.** Everything emitted is standard SPARQL 1.1 that the same engine the
//     Query tool uses can run — no builder-only extensions, no rewriting at run time. The
//     generated text IS the query; the tool always shows it and lets you edit it.
//   * **Never silently drop.** Anything the lowering cannot express (a projected variable that
//     the pattern never binds, an edge to a deleted node, a non-numeric "number" filter) comes
//     back in `warnings`, which the panel renders. A wrong query is better surfaced than hidden.
//   * **Suggestions are labelled by provenance.** A predicate offered by the picker records
//     whether it came from a SHACL shape, from the live data, or from both — a shape-declared
//     property that no instance uses yet is still offered, but is not dressed up as observed.

import { COMMON_PREFIXES, type PrefixBinding, type SparqlResults } from "@sparq/client";

// ---------------------------------------------------------------------------
// The canvas model
// ---------------------------------------------------------------------------

/** Comparison applied to an attribute (a predicate whose object is kept as a value). */
export type FilterOp =
  | "any" // bind the value, constrain nothing
  | "eq"
  | "ne"
  | "lt"
  | "le"
  | "gt"
  | "ge"
  | "contains"
  | "starts"
  | "ends"
  | "regex"
  | "absent"; // FILTER NOT EXISTS — the attribute must be missing

/** How a filter's right-hand value is written into the query. */
export type ValueKind = "text" | "number" | "iri";

/** The aggregate applied to a projected item (drives the GROUP BY panel). */
export type Aggregate = "count" | "count-distinct" | "sum" | "avg" | "min" | "max";

/** How an edge is joined: plain BGP, OPTIONAL, or the AND-NOT (negated) form. */
export type EdgeMode = "required" | "optional" | "not";

/** One attribute constraint hanging off a node (Stardog Explorer's "attribute filters"). */
export interface AttributeFilter {
  id: string;
  /** The predicate whose object is the attribute value. */
  predicateIri: string;
  /** The variable the value binds to (sanitised + uniquified at lowering time). */
  variable: string;
  op: FilterOp;
  /** Right-hand side, as typed by the user. Ignored for `any` / `absent`. */
  value: string;
  valueKind: ValueKind;
  /** Include the value in the SELECT projection. */
  project: boolean;
  /** Aggregate to apply when projected (null = a plain grouping key). */
  aggregate: Aggregate | null;
}

/** A node on the canvas — one subject/object variable, optionally typed by a class. */
export interface BuilderNode {
  id: string;
  variable: string;
  /** `?v a <classIri>` when set; an untyped node contributes no rdf:type pattern. */
  classIri: string | null;
  x: number;
  y: number;
  project: boolean;
  aggregate: Aggregate | null;
  filters: AttributeFilter[];
}

/** A directed edge — one triple pattern between two node variables. */
export interface BuilderEdge {
  id: string;
  from: string;
  to: string;
  predicateIri: string;
  mode: EdgeMode;
}

/** Result-shaping controls (the aggregate / GROUP BY panel). */
export interface BuilderModel {
  nodes: BuilderNode[];
  edges: BuilderEdge[];
  distinct: boolean;
  /** Output variable to order by (an alias for an aggregated item), or null. */
  orderBy: { variable: string; desc: boolean } | null;
  /** `LIMIT n` when a positive integer; null emits no LIMIT. */
  limit: number | null;
}

/** The lowering's output: the query text, what it projects, and everything it could not express. */
export interface BuiltQuery {
  sparql: string;
  warnings: string[];
  /** Output variable names (without `?`), in projection order. `[]` means `SELECT *`. */
  projected: string[];
  /** Grouping keys (without `?`); empty when the query does not aggregate. */
  groupBy: string[];
}

/** An empty canvas. */
export function emptyModel(): BuilderModel {
  return { nodes: [], edges: [], distinct: false, orderBy: null, limit: 100 };
}

// ---------------------------------------------------------------------------
// Term rendering
// ---------------------------------------------------------------------------

/** The local part of an IRI (after the last `#` or `/`) — the picker's short label. */
export function localName(iri: string): string {
  const cut = Math.max(iri.lastIndexOf("#"), iri.lastIndexOf("/"));
  const tail = cut >= 0 ? iri.slice(cut + 1) : iri;
  return tail.length > 0 ? tail : iri;
}

/**
 * Make `raw` a legal SPARQL `VARNAME`. The grammar admits a leading digit (`?1st` is legal), so
 * only the character set needs fixing. Returns `fallback` when nothing usable survives, so the
 * lowering can never emit a syntactically broken `?`.
 */
export function sanitizeVariable(raw: string, fallback = "v"): string {
  const cleaned = raw.replace(/^\?+/, "").replace(/[^A-Za-z0-9_]/g, "_");
  return cleaned.length > 0 ? cleaned : fallback;
}

// Characters an IRIREF (`<...>`) may not contain, per the SPARQL grammar. They are
// percent-encoded rather than dropped, so a pasted IRI never yields an unparsable query.
const IRI_ILLEGAL = /[\u0000-\u0020<>"{}|^`\\]/g;

/** A legal `PN_LOCAL`-ish local part we are willing to abbreviate (conservative on purpose). */
const SAFE_LOCAL = /^[A-Za-z_][A-Za-z0-9_-]*$/;

/**
 * Render an IRI as a term. Abbreviates to `prefix:local` when the namespace is one of the
 * well-known {@link COMMON_PREFIXES} AND the local part is unambiguously safe; otherwise emits a
 * full `<IRI>`. Records the prefix it used in `used` so the header declares exactly what the
 * body references (never a prefix the query does not use).
 */
export function renderIri(iri: string, used?: Map<string, string>): string {
  for (const binding of COMMON_PREFIXES) {
    if (!iri.startsWith(binding.iri) || iri.length === binding.iri.length) continue;
    const local = iri.slice(binding.iri.length);
    if (!SAFE_LOCAL.test(local)) continue;
    // First registry hit wins, so a namespace with two labels (dc / dcterms) is stable.
    if (used && !used.has(binding.prefix)) used.set(binding.prefix, binding.iri);
    return `${binding.prefix}:${local}`;
  }
  return `<${iri.replace(IRI_ILLEGAL, (c) => encodeURIComponent(c))}>`;
}

/** Render a string as a quoted SPARQL literal, escaping the characters that would break it. */
export function renderLiteral(value: string): string {
  const escaped = value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
  return `"${escaped}"`;
}

/** A SPARQL numeric literal (integer / decimal / double), or null when `value` is not one. */
export function asNumericLiteral(value: string): string | null {
  const trimmed = value.trim();
  return /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(trimmed) ? trimmed : null;
}

// ---------------------------------------------------------------------------
// Lowering: model -> SPARQL
// ---------------------------------------------------------------------------

const AGG_FN: Record<Aggregate, { fn: string; distinct: boolean; suffix: string }> = {
  count: { fn: "COUNT", distinct: false, suffix: "count" },
  "count-distinct": { fn: "COUNT", distinct: true, suffix: "count" },
  sum: { fn: "SUM", distinct: false, suffix: "sum" },
  avg: { fn: "AVG", distinct: false, suffix: "avg" },
  min: { fn: "MIN", distinct: false, suffix: "min" },
  max: { fn: "MAX", distinct: false, suffix: "max" },
};

interface ProjectionItem {
  /** The variable the pattern binds. */
  source: string;
  /** The output variable (equal to `source` unless aggregated). */
  output: string;
  aggregate: Aggregate | null;
}

/**
 * Lower a canvas model to SPARQL 1.1.
 *
 * Structure of the emitted WHERE:
 *   * one block per node — its `a <Class>` triple, its attribute triples + FILTERs, then its
 *     outgoing REQUIRED edges (so the text reads in the same order the diagram walks);
 *   * one `OPTIONAL { … }` group per optional edge;
 *   * one `FILTER NOT EXISTS { … }` per negated (AND-NOT) edge. NOT EXISTS rather than MINUS
 *     because it is correct for this shape whether or not the inner group shares a variable —
 *     MINUS silently matches nothing when the two sides are disjoint.
 *
 * A soft (optional/negated) edge is lowered as a BRANCH, not as a lone triple: the whole
 * sub-graph whose only binding path runs through it — the target's class + attribute patterns,
 * the required links hanging off the target, and any further soft links below them (nested) —
 * is emitted INSIDE that group. Emitting any of it at the top level would quietly make the
 * branch mandatory, so `?a OPTIONAL→ ?b OPTIONAL→ ?c` never forces `?b` to exist.
 *
 * When several soft links converge on the SAME target, every one of those branches restates the
 * target's sub-graph: a branch that does not match leaves the target unbound, so a branch that
 * omitted the target's class/attribute patterns would be free to bind it to anything.
 */
export function buildSparql(model: BuilderModel): BuiltQuery {
  const warnings: string[] = [];
  const usedPrefixes = new Map<string, string>();
  const iri = (value: string) => renderIri(value, usedPrefixes);

  const byId = new Map<string, BuilderNode>(model.nodes.map((n) => [n.id, n]));

  // --- variable naming: one flat namespace, uniquified in a stable order ---------------------
  const taken = new Set<string>();
  const unique = (raw: string, fallback: string): string => {
    const base = sanitizeVariable(raw, fallback);
    let candidate = base;
    let n = 2;
    while (taken.has(candidate)) candidate = `${base}${n++}`;
    taken.add(candidate);
    return candidate;
  };
  const nodeVar = new Map<string, string>();
  const filterVar = new Map<string, string>();
  for (const node of model.nodes) {
    nodeVar.set(node.id, unique(node.variable, "node"));
    for (const f of node.filters) filterVar.set(f.id, unique(f.variable, "value"));
  }

  // --- edges: drop danglers + not-yet-chosen predicates LOUDLY -------------------------------
  const edges = model.edges.filter((e) => {
    if (!byId.has(e.from) || !byId.has(e.to)) {
      warnings.push(
        `Link "${localName(e.predicateIri)}" refers to a node that no longer exists — skipped.`,
      );
      return false;
    }
    if (e.predicateIri.trim().length === 0) {
      const from = byId.get(e.from)!;
      const to = byId.get(e.to)!;
      warnings.push(
        `The link ?${nodeVar.get(from.id)} → ?${nodeVar.get(to.id)} has no predicate yet — skipped.`,
      );
      return false;
    }
    return true;
  });

  // --- scope analysis: the mandatory body vs the soft branches -------------------------------
  // A soft (optional / not) edge is a BRANCH, not just one triple: every node whose only binding
  // path runs through it belongs INSIDE that group, together with the links hanging off it. A
  // REQUIRED edge binds both of its endpoints, so required edges fuse nodes into one component
  // that is always emitted as a unit — either wholly mandatory or wholly inside one branch.
  const requiredNeighbours = new Map<string, string[]>(model.nodes.map((n) => [n.id, []]));
  const inDegree = new Map<string, number>(model.nodes.map((n) => [n.id, 0]));
  for (const e of edges) {
    inDegree.set(e.to, (inDegree.get(e.to) ?? 0) + 1);
    if (e.mode !== "required") continue;
    requiredNeighbours.get(e.from)!.push(e.to);
    requiredNeighbours.get(e.to)!.push(e.from);
  }

  /** The required-connected component of `id`, in canvas order. */
  const componentOf = (id: string): BuilderNode[] => {
    const seen = new Set([id]);
    const stack = [id];
    while (stack.length > 0) {
      for (const other of requiredNeighbours.get(stack.pop()!) ?? []) {
        if (seen.has(other)) continue;
        seen.add(other);
        stack.push(other);
      }
    }
    return model.nodes.filter((n) => seen.has(n.id));
  };

  // The mandatory scope is seeded by the nodes nothing points at — the unconditional entry
  // points — and grown across required edges. Everything else is deferred into the soft branch
  // that reaches it, so a node reachable only through an optional/negated link stays inside it.
  const mandatory = new Set<string>();
  const anchor = (id: string) => {
    for (const n of componentOf(id)) mandatory.add(n.id);
  };
  for (const n of model.nodes) if (inDegree.get(n.id) === 0) anchor(n.id);

  // Every node still needs a home: a cycle of links (?a → ?b → ?a) has no unconditional entry
  // point, so anchor the first unreachable node rather than silently dropping its patterns.
  for (;;) {
    const reached = new Set(mandatory);
    for (;;) {
      const next = edges.find(
        (e) => e.mode !== "required" && reached.has(e.from) && !reached.has(e.to),
      );
      if (!next) break;
      for (const n of componentOf(next.to)) reached.add(n.id);
    }
    const orphan = model.nodes.find((n) => !reached.has(n.id));
    if (!orphan) break;
    anchor(orphan.id);
  }

  // --- pattern emission ---------------------------------------------------------------------
  /** The class + attribute lines for one node (no edges). */
  const nodeLines = (node: BuilderNode): string[] => {
    const out: string[] = [];
    const subject = `?${nodeVar.get(node.id)}`;
    if (node.classIri) out.push(`${subject} a ${iri(node.classIri)} .`);
    for (const f of node.filters) {
      const value = `?${filterVar.get(f.id)}`;
      if (f.predicateIri.trim().length === 0) {
        warnings.push(`The attribute bound to ${value} has no predicate yet — skipped.`);
        continue;
      }
      if (f.op === "absent") {
        out.push(`FILTER NOT EXISTS { ${subject} ${iri(f.predicateIri)} ${value} }`);
        continue;
      }
      out.push(`${subject} ${iri(f.predicateIri)} ${value} .`);
      const expr = filterExpression(f, value, iri, warnings);
      if (expr) out.push(`FILTER(${expr})`);
    }
    return out;
  };

  // Which node variables the emitted pattern actually BINDS. A `not` edge contributes nothing:
  // `FILTER NOT EXISTS { ?a p ?b }` constrains ?a but binds neither side.
  const bound = new Set<string>();
  const filterBinds = (f: AttributeFilter) => f.op !== "absent" && f.predicateIri.trim().length > 0;
  for (const n of model.nodes) {
    if (n.classIri || n.filters.some(filterBinds)) bound.add(n.id);
  }
  for (const e of edges) {
    if (e.mode === "not") continue;
    bound.add(e.from);
    bound.add(e.to);
  }

  const edgeLine = (e: BuilderEdge) =>
    `?${nodeVar.get(e.from)} ${iri(e.predicateIri)} ?${nodeVar.get(e.to)} .`;

  /** One node's own patterns plus the required links hanging off it. */
  const withRequired = (node: BuilderNode): string[] => {
    const out = nodeLines(node);
    for (const e of edges) if (e.mode === "required" && e.from === node.id) out.push(edgeLine(e));
    return out;
  };

  /**
   * Soft links out of `ids`, OPTIONALs first — both for readability and because a deferred
   * node's own patterns must land in an optional branch that can BIND it, never inside a
   * negation (where they would constrain the negated pattern instead).
   */
  const softOut = (ids: ReadonlySet<string>): BuilderEdge[] => [
    ...edges.filter((e) => e.mode === "optional" && ids.has(e.from)),
    ...edges.filter((e) => e.mode === "not" && ids.has(e.from)),
  ];

  /** Nodes whose patterns landed inside a FILTER NOT EXISTS. */
  const negatedScope = new Set<string>();
  /** Nodes whose patterns landed somewhere that BINDS them — the mandatory body or an OPTIONAL. */
  const bindingScope = new Set<string>();
  /** A node reached ONLY through negations: its variable is not in scope outside them. */
  const outOfScope = (id: string) => negatedScope.has(id) && !bindingScope.has(id);

  /**
   * One soft link as a group: its triple, the sub-graph only it reaches, then nested branches.
   *
   * `inScope` is the set of nodes whose patterns already hold ALONG THIS PATH — the mandatory
   * body plus every ENCLOSING branch — and those are not restated. A node emitted in a SIBLING
   * branch is deliberately NOT in it: sibling soft groups bind their target independently, and a
   * branch that does not match leaves the target unbound, so a later `?c OPTIONAL→ ?b` would then
   * bind ?b to anything unless it carries ?b's own class/attribute constraints too. Every branch
   * that can independently bind a target therefore restates that target's sub-graph. Carrying the
   * path down (rather than one global emitted-once bit) also stops a soft link cycle
   * ?b → ?c → ?b from recursing forever.
   */
  const softGroup = (e: BuilderEdge, negated: boolean, inScope: ReadonlySet<string>): string[] => {
    const inNegation = negated || e.mode === "not";
    const inner = [edgeLine(e)];
    const inside = new Set<string>();
    const below = new Set(inScope);
    if (!inScope.has(e.to)) {
      for (const node of componentOf(e.to)) {
        below.add(node.id);
        inside.add(node.id);
        (inNegation ? negatedScope : bindingScope).add(node.id);
        inner.push(...withRequired(node));
      }
    }
    for (const child of softOut(inside)) inner.push(...softGroup(child, inNegation, below));
    const keyword = e.mode === "optional" ? "OPTIONAL" : "FILTER NOT EXISTS";
    return [`${keyword} {`, ...inner.map((l) => `  ${l}`), `}`];
  };

  const body: string[] = [];
  for (const node of model.nodes) {
    if (!mandatory.has(node.id)) continue;
    bindingScope.add(node.id);
    body.push(...withRequired(node));
    if (node.project && !bound.has(node.id)) {
      warnings.push(
        `Node ?${nodeVar.get(node.id)} has no class, attribute or link, so nothing binds it.`,
      );
    }
  }
  // The mandatory body is the initial in-scope path: its patterns hold for every branch below it.
  for (const e of softOut(mandatory)) body.push(...softGroup(e, false, mandatory));

  // --- projection ---------------------------------------------------------------------------
  const projection: ProjectionItem[] = [];
  const addProjection = (source: string, aggregate: Aggregate | null) => {
    if (!aggregate) {
      projection.push({ source, output: source, aggregate: null });
      return;
    }
    const base = `${source}_${AGG_FN[aggregate].suffix}`;
    let output = base;
    let n = 2;
    while (taken.has(output)) output = `${base}${n++}`;
    taken.add(output);
    projection.push({ source, output, aggregate });
  };
  for (const node of model.nodes) {
    const variable = nodeVar.get(node.id)!;
    if (node.project) {
      if (outOfScope(node.id)) {
        warnings.push(
          `?${variable} is only reached through an AND-NOT link, so it is out of scope — not projected.`,
        );
      } else {
        addProjection(variable, node.aggregate);
      }
    }
    for (const f of node.filters) {
      // A blank predicate was already reported by the pattern pass; it binds nothing.
      if (!f.project || f.predicateIri.trim().length === 0) continue;
      const variable2 = filterVar.get(f.id)!;
      if (f.op === "absent") {
        warnings.push(
          `?${variable2} checks that "${localName(f.predicateIri)}" is ABSENT, so it binds no value — not projected.`,
        );
        continue;
      }
      if (outOfScope(node.id)) {
        warnings.push(`?${variable2} is inside an AND-NOT group, so it is out of scope — not projected.`);
        continue;
      }
      addProjection(variable2, f.aggregate);
    }
  }

  const aggregated = projection.filter((p) => p.aggregate !== null);
  const keys = projection.filter((p) => p.aggregate === null);
  const groupBy: string[] = aggregated.length > 0 ? keys.map((k) => k.output) : [];

  let selectClause: string;
  if (projection.length === 0) {
    if (model.nodes.length > 0) {
      warnings.push("Nothing is selected — the query returns every variable (SELECT *).");
    }
    selectClause = model.distinct ? "SELECT DISTINCT *" : "SELECT *";
  } else {
    const items = projection.map((p) => {
      if (!p.aggregate) return `?${p.output}`;
      const { fn, distinct } = AGG_FN[p.aggregate];
      return `(${fn}(${distinct ? "DISTINCT " : ""}?${p.source}) AS ?${p.output})`;
    });
    selectClause = `${model.distinct ? "SELECT DISTINCT" : "SELECT"} ${items.join(" ")}`;
  }

  // --- assemble ------------------------------------------------------------------------------
  const lines: string[] = [];
  const prefixBindings: PrefixBinding[] = COMMON_PREFIXES.filter((b) => usedPrefixes.has(b.prefix));
  for (const b of prefixBindings) lines.push(`PREFIX ${b.prefix}: <${b.iri}>`);
  if (prefixBindings.length > 0) lines.push("");

  lines.push(selectClause);
  lines.push("WHERE {");
  for (const line of body) lines.push(`  ${line}`);
  lines.push("}");
  if (groupBy.length > 0) lines.push(`GROUP BY ${groupBy.map((g) => `?${g}`).join(" ")}`);

  const outputs = new Set(projection.map((p) => p.output));
  if (model.orderBy) {
    if (projection.length > 0 && !outputs.has(model.orderBy.variable)) {
      warnings.push(`ORDER BY ?${model.orderBy.variable} is not in the result — dropped.`);
    } else {
      const v = `?${model.orderBy.variable}`;
      lines.push(`ORDER BY ${model.orderBy.desc ? `DESC(${v})` : v}`);
    }
  }
  if (model.limit !== null && Number.isFinite(model.limit) && model.limit > 0) {
    lines.push(`LIMIT ${Math.floor(model.limit)}`);
  }

  return {
    sparql: lines.join("\n"),
    warnings,
    projected: projection.map((p) => p.output),
    groupBy,
  };
}

/** The FILTER body for one attribute filter, or null when the op constrains nothing. */
function filterExpression(
  f: AttributeFilter,
  value: string,
  iri: (v: string) => string,
  warnings: string[],
): string | null {
  if (f.op === "any" || f.op === "absent") return null;
  const raw = f.value.trim();
  if (raw.length === 0) {
    warnings.push(`Filter on "${localName(f.predicateIri)}" has no value — the comparison is skipped.`);
    return null;
  }

  // String tests always compare the lexical form, case-insensitively — the behaviour a
  // "contains" box in a builder is expected to have, written out explicitly in the query.
  if (f.op === "contains" || f.op === "starts" || f.op === "ends") {
    const fn = f.op === "contains" ? "CONTAINS" : f.op === "starts" ? "STRSTARTS" : "STRENDS";
    return `${fn}(LCASE(STR(${value})), LCASE(${renderLiteral(raw)}))`;
  }
  if (f.op === "regex") return `REGEX(STR(${value}), ${renderLiteral(raw)}, "i")`;

  let rhs: string;
  if (f.valueKind === "iri") {
    rhs = iri(raw);
  } else if (f.valueKind === "number") {
    const numeric = asNumericLiteral(raw);
    if (numeric) {
      rhs = numeric;
    } else {
      warnings.push(`"${raw}" is not a number — compared as text instead.`);
      rhs = renderLiteral(raw);
    }
  } else {
    rhs = renderLiteral(raw);
  }
  const operator = COMPARISON_OPERATOR[f.op];
  // Unreachable for the six comparison ops; a future op reaching here constrains nothing rather
  // than emitting `?v undefined "x"`.
  if (!operator) return null;
  return `${value} ${operator} ${rhs}`;
}

/** The six ops that lower to a binary comparison. Anything else is handled before this table. */
const COMPARISON_OPERATOR: Partial<Record<FilterOp, string>> = {
  eq: "=",
  ne: "!=",
  lt: "<",
  le: "<=",
  gt: ">",
  ge: ">=",
};

// ---------------------------------------------------------------------------
// Live-store introspection — the queries behind the pickers
// ---------------------------------------------------------------------------

/** A class observed in the live data, with how many instances carry it. */
export interface ClassStat {
  iri: string;
  instances: number;
}

/** One predicate of a class's characteristic set, as observed in the live data. */
export interface PredicateStat {
  iri: string;
  uses: number;
  /** How many of those uses had a LITERAL object (the rest were IRIs/bnodes). */
  literalUses: number;
  /** Classes the object was observed to have (from the link-target probe). */
  objectClasses: string[];
}

/** One `sh:property` constraint of a `sh:targetClass` shape. */
export interface ShapeProperty {
  targetClass: string;
  path: string;
  objectClass: string | null;
  datatype: string | null;
  name: string | null;
  required: boolean;
}

const SH = "http://www.w3.org/ns/shacl#";
const RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/**
 * An IRI as a full `<IRIREF>`, never abbreviated. The introspection queries below are assembled
 * as standalone strings with no shared prefix header, so abbreviating here would emit a
 * prefixed name the query never declares.
 */
function iriRef(iri: string): string {
  return `<${iri.replace(IRI_ILLEGAL, (c) => encodeURIComponent(c))}>`;
}

/**
 * The classes present in the live store, most-instantiated first. Plain SPARQL 1.1 — it runs on
 * the same in-tab engine every other tool uses, so what the picker offers is what the data says.
 */
export function classesQuery(limit = 200): string {
  return [
    "SELECT ?class (COUNT(DISTINCT ?s) AS ?instances)",
    "WHERE { ?s a ?class }",
    "GROUP BY ?class",
    "ORDER BY DESC(?instances)",
    `LIMIT ${Math.max(1, Math.floor(limit))}`,
  ].join("\n");
}

/**
 * The **characteristic set** of a class: which predicates its instances actually use, how often,
 * and how many of those uses had a literal object (so the picker can tell an attribute from a
 * link). `classIri` null profiles every subject — the untyped-node case.
 */
export function predicatesQuery(classIri: string | null, limit = 200): string {
  const subject = classIri ? `  ?s a ${iriRef(classIri)} .\n` : "";
  return [
    "SELECT ?p (COUNT(*) AS ?uses) (SUM(IF(isLiteral(?o), 1, 0)) AS ?literals)",
    "WHERE {",
    subject + "  ?s ?p ?o .",
    `  FILTER(?p != ${iriRef(RDF_TYPE)})`,
    "}",
    "GROUP BY ?p",
    "ORDER BY DESC(?uses)",
    `LIMIT ${Math.max(1, Math.floor(limit))}`,
  ].join("\n");
}

/**
 * Where a class's outgoing links LAND — the object classes per predicate. Separate from
 * {@link predicatesQuery} on purpose: joining `?o a ?oc` into that aggregate would multiply the
 * `uses` counts for multi-typed objects and quietly inflate the numbers the picker shows.
 */
export function linkTargetsQuery(classIri: string | null, limit = 200): string {
  const subject = classIri ? `  ?s a ${iriRef(classIri)} .\n` : "";
  return [
    "SELECT ?p ?objectClass (COUNT(*) AS ?uses)",
    "WHERE {",
    subject + "  ?s ?p ?o .",
    "  ?o a ?objectClass .",
    `  FILTER(?p != ${iriRef(RDF_TYPE)})`,
    "}",
    "GROUP BY ?p ?objectClass",
    "ORDER BY DESC(?uses)",
    `LIMIT ${Math.max(1, Math.floor(limit))}`,
  ].join("\n");
}

/**
 * SHACL shapes **that are in the live store**. Only `sh:targetClass` shapes with an IRI
 * `sh:path` are read: implicit class targets and complex property paths (a blank-node path
 * expression) are genuinely out of scope for v1 rather than half-guessed. Shapes that were only
 * pasted into the SHACL tool's editor are session-only state of that tool and are not visible
 * here — import them to make them drive suggestions.
 */
export function shapesQuery(limit = 500): string {
  return [
    `PREFIX sh: <${SH}>`,
    "SELECT ?targetClass ?path ?objectClass ?datatype ?name ?minCount",
    "WHERE {",
    "  ?shape sh:targetClass ?targetClass .",
    "  ?shape sh:property ?ps .",
    "  ?ps sh:path ?path .",
    "  FILTER(isIRI(?path))",
    "  OPTIONAL { ?ps sh:class ?objectClass }",
    "  OPTIONAL { ?ps sh:datatype ?datatype }",
    "  OPTIONAL { ?ps sh:name ?name }",
    "  OPTIONAL { ?ps sh:minCount ?minCount }",
    "}",
    `LIMIT ${Math.max(1, Math.floor(limit))}`,
  ].join("\n");
}

const iriValue = (results: SparqlResults, row: number, name: string): string | null => {
  const term = results.results.bindings[row]?.[name];
  return term && term.type === "uri" ? term.value : null;
};

const numberValue = (results: SparqlResults, row: number, name: string): number => {
  const raw = results.results.bindings[row]?.[name]?.value;
  const parsed = raw === undefined ? Number.NaN : Number(raw);
  return Number.isFinite(parsed) ? parsed : 0;
};

/** Parse {@link classesQuery} results. Rows whose `?class` is not an IRI are ignored. */
export function parseClassRows(results: SparqlResults): ClassStat[] {
  const out: ClassStat[] = [];
  for (let i = 0; i < results.results.bindings.length; i++) {
    const value = iriValue(results, i, "class");
    if (value) out.push({ iri: value, instances: numberValue(results, i, "instances") });
  }
  return out;
}

/** Parse {@link predicatesQuery} + {@link linkTargetsQuery} results into one profile per predicate. */
export function parsePredicateRows(
  predicates: SparqlResults,
  linkTargets?: SparqlResults,
): PredicateStat[] {
  const out: PredicateStat[] = [];
  const index = new Map<string, PredicateStat>();
  for (let i = 0; i < predicates.results.bindings.length; i++) {
    const value = iriValue(predicates, i, "p");
    if (!value || index.has(value)) continue;
    const stat: PredicateStat = {
      iri: value,
      uses: numberValue(predicates, i, "uses"),
      literalUses: numberValue(predicates, i, "literals"),
      objectClasses: [],
    };
    index.set(value, stat);
    out.push(stat);
  }
  if (linkTargets) {
    for (let i = 0; i < linkTargets.results.bindings.length; i++) {
      const p = iriValue(linkTargets, i, "p");
      const objectClass = iriValue(linkTargets, i, "objectClass");
      if (!p || !objectClass) continue;
      const stat = index.get(p);
      if (!stat || stat.objectClasses.includes(objectClass)) continue;
      stat.objectClasses.push(objectClass);
    }
  }
  return out;
}

/** Parse {@link shapesQuery} results. */
export function parseShapeRows(results: SparqlResults): ShapeProperty[] {
  const out: ShapeProperty[] = [];
  const seen = new Set<string>();
  for (let i = 0; i < results.results.bindings.length; i++) {
    const targetClass = iriValue(results, i, "targetClass");
    const path = iriValue(results, i, "path");
    if (!targetClass || !path) continue;
    const key = `${targetClass} ${path}`;
    if (seen.has(key)) continue;
    seen.add(key);
    out.push({
      targetClass,
      path,
      objectClass: iriValue(results, i, "objectClass"),
      datatype: iriValue(results, i, "datatype"),
      name: results.results.bindings[i]?.name?.value ?? null,
      required: numberValue(results, i, "minCount") >= 1,
    });
  }
  return out;
}

// ---------------------------------------------------------------------------
// Suggestions — shapes ⊕ data
// ---------------------------------------------------------------------------

/** One entry in the class picker, carrying whether the data actually contains the class. */
export interface ClassOption {
  iri: string;
  /** Instances observed in the live store; null when only a `sh:targetClass` declares it. */
  instances: number | null;
}

/**
 * The classes the pickers offer: everything observed in the data (in the query's
 * most-instantiated-first order) followed by every `sh:targetClass` the store's shapes declare
 * but no instance carries yet, by local name.
 *
 * Without this second group a shape-declared class with zero instances would be unreachable from
 * the UI — and with it, every shape-only property suggestion for that class. The `instances: null`
 * marker is what lets the picker say "declared, not in the data" instead of implying a count.
 */
export function mergeClassOptions(
  data: readonly ClassStat[],
  shapes: readonly ShapeProperty[],
): ClassOption[] {
  const observed = new Set(data.map((c) => c.iri));
  const declared = [...new Set(shapes.map((s) => s.targetClass))]
    .filter((iri) => !observed.has(iri))
    .sort((a, b) => localName(a).localeCompare(localName(b)) || a.localeCompare(b));
  return [
    ...data.map((c) => ({ iri: c.iri, instances: c.instances })),
    ...declared.map((iri) => ({ iri, instances: null })),
  ];
}

/** Whether the picker should offer this predicate as a link (to a node) or an attribute (a value). */
export type SuggestionKind = "link" | "attribute" | "mixed" | "unknown";

/** One entry in the predicate picker, carrying WHERE the suggestion came from. */
export interface PredicateSuggestion {
  iri: string;
  label: string;
  /** `shape` = declared by a SHACL shape only; `data` = observed only; `both` = declared AND observed. */
  source: "shape" | "data" | "both";
  /** Observed uses in the live data; null when the predicate is shape-declared but unused. */
  uses: number | null;
  kind: SuggestionKind;
  /** Candidate classes for the object — shape `sh:class` first, then observed object classes. */
  objectClasses: string[];
  /** Shape-declared `sh:datatype`, when any. */
  datatype: string | null;
  /** Shape-declared `sh:minCount >= 1`. */
  required: boolean;
}

/**
 * Merge the shape-declared properties of a class with its observed characteristic set.
 *
 * Ordering is the picker's ranking: shape-REQUIRED first (a shape says you must have it), then
 * the rest of the shape-declared properties, then observed-only predicates by descending use.
 * Shape-declared-but-unused properties are still offered — with `uses: null`, so the UI can say
 * "declared, not yet used" rather than implying data that is not there.
 */
export function mergeSuggestions(
  data: readonly PredicateStat[],
  shapes: readonly ShapeProperty[],
): PredicateSuggestion[] {
  const dataByIri = new Map<string, PredicateStat>(data.map((d) => [d.iri, d]));
  const out: PredicateSuggestion[] = [];
  const emitted = new Set<string>();

  for (const shape of shapes) {
    if (emitted.has(shape.path)) continue;
    emitted.add(shape.path);
    const observed = dataByIri.get(shape.path);
    const objectClasses: string[] = shape.objectClass ? [shape.objectClass] : [];
    for (const c of observed?.objectClasses ?? []) {
      if (!objectClasses.includes(c)) objectClasses.push(c);
    }
    out.push({
      iri: shape.path,
      label: shape.name ?? localName(shape.path),
      source: observed ? "both" : "shape",
      uses: observed ? observed.uses : null,
      kind: shape.objectClass ? "link" : shape.datatype ? "attribute" : observedKind(observed),
      objectClasses,
      datatype: shape.datatype,
      required: shape.required,
    });
  }

  for (const stat of data) {
    if (emitted.has(stat.iri)) continue;
    emitted.add(stat.iri);
    out.push({
      iri: stat.iri,
      label: localName(stat.iri),
      source: "data",
      uses: stat.uses,
      kind: observedKind(stat),
      objectClasses: stat.objectClasses,
      datatype: null,
      required: false,
    });
  }

  const rank = (s: PredicateSuggestion) => (s.required ? 0 : s.source === "data" ? 2 : 1);
  return out.sort((a, b) => rank(a) - rank(b) || (b.uses ?? -1) - (a.uses ?? -1));
}

/** Classify an observed predicate by how its objects looked in the data. */
function observedKind(stat: PredicateStat | undefined): SuggestionKind {
  if (!stat || stat.uses <= 0) return "unknown";
  if (stat.literalUses <= 0) return "link";
  if (stat.literalUses >= stat.uses) return "attribute";
  return "mixed";
}
