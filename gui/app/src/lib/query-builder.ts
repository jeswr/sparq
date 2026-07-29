// [OPUS-5] sq-ixc3.24 — the visual query builder's PURE core: the diagram model, its
// validation, and the SPARQL serializer (no React, no DOM), plus the schema-introspection
// queries + row parsers that drive the shape-aware pickers.
//
// The competitor reference is AllegroGraph Gruff ("draw the pattern, get the query") and
// Stardog Explorer's model-driven query builder (walk relationship paths, attribute filters,
// AND-NOT, GROUP BY). The differentiator this file makes possible is SHAPE-AWARE suggestion:
// when the loaded store contains SHACL shapes, the predicate pickers are driven by the
// DECLARED shape properties; otherwise they fall back to characteristic sets OBSERVED in the
// data. Both are labelled at the point of use — see {@link SuggestionSource}.
//
// HONESTY INVARIANTS (the reason this file exists separately from the panel):
//   * **No hidden dialect.** Everything emitted is standard SPARQL 1.1 — no sparq-only syntax,
//     no magic predicates, no rewriting behind the user's back. What the canvas shows is what
//     the engine is asked. The generated text is always visible and always editable.
//   * **Nothing is fabricated.** Suggestions come from a query over the LIVE store. If the
//     store holds no shapes, there are no shape suggestions — the builder says so rather than
//     inventing a vocabulary.
//   * **Invalid models still serialize.** {@link buildQuery} returns best-effort SPARQL
//     ALONGSIDE its issues, so the user can always read (and fix) the text. The panel refuses
//     to RUN on an error; it never hides the query.
//   * **Stated v1 limits.** UNION is offered only as alternative predicates on one edge, and a
//     NOT-EXISTS edge must be a leaf. Anything beyond that is done by editing the SPARQL — the
//     builder does not pretend to cover the whole language.

import { COMMON_PREFIXES, type SparqlResults, type SparqlTerm } from "@sparq/client";

// ---------------------------------------------------------------------------
// The diagram model.
// ---------------------------------------------------------------------------

/** How a filter compares an attribute variable to a value. */
export type FilterOp =
  | "="
  | "!="
  | "<"
  | "<="
  | ">"
  | ">="
  | "contains"
  | "starts"
  | "ends"
  | "regex";

/** The ops implemented with a SPARQL string function (and so requiring a string value). */
const STRING_OPS: ReadonlySet<FilterOp> = new Set(["contains", "starts", "ends", "regex"]);

/** How a filter's value text is turned into a SPARQL term. */
export type ValueKind = "string" | "number" | "boolean" | "date" | "iri";

/** One attribute filter — `?var OP value`, or a string function over `STR(?var)`. */
export interface AttributeFilter {
  op: FilterOp;
  /** The raw value text as typed. Interpreted per {@link kind}. */
  value: string;
  kind: ValueKind;
  /** `regex` only — adds the `"i"` flag. Ignored by every other op. */
  caseInsensitive?: boolean;
}

/** A literal-or-node attribute hanging off a node: `?node predicate ?var`. */
export interface BuilderAttribute {
  id: string;
  /** A full IRI, a prefixed name, or the literal `a` (rdf:type). */
  predicate: string;
  /** The bound variable name, WITHOUT the leading `?`. */
  variable: string;
  /** Project this variable in the SELECT clause. */
  projected: boolean;
  /** Wrap the triple (and its filter) in `OPTIONAL { … }`. */
  optional: boolean;
  filter: AttributeFilter | null;
}

/** A node on the canvas — one subject/object variable and everything asserted directly on it. */
export interface BuilderNode {
  id: string;
  /** The bound variable name, WITHOUT the leading `?`. */
  variable: string;
  /** `?var a <classIri>` when set; `null` leaves the node untyped. */
  classIri: string | null;
  attributes: BuilderAttribute[];
  /** Project this node's variable in the SELECT clause. */
  projected: boolean;
  /** Canvas position (view state — it never affects the generated SPARQL). */
  x: number;
  y: number;
}

/**
 * How an edge participates in the pattern:
 *   * `required`   — a plain triple pattern;
 *   * `optional`   — wrapped in `OPTIONAL { … }` (the target node's own patterns come with it);
 *   * `not-exists` — wrapped in `FILTER NOT EXISTS { … }` (Stardog's AND-NOT). The target node
 *     is then bound only INSIDE the negation, so it cannot be projected — v1 additionally
 *     requires it to be a leaf (see {@link validateModel}).
 */
export type EdgeMode = "required" | "optional" | "not-exists";

/** A directed predicate edge between two canvas nodes. */
export interface BuilderEdge {
  id: string;
  /** Source node id. */
  from: string;
  /** Target node id. */
  to: string;
  /** A full IRI, a prefixed name, or `a`. */
  predicate: string;
  /**
   * The v1 UNION affordance: extra predicates that are acceptable ALTERNATIVES to
   * {@link predicate}. Non-empty ⇒ the edge emits `{ … } UNION { … }` over each alternative.
   * Only meaningful for a `required` edge (see {@link validateModel}).
   */
  alternatives: string[];
  mode: EdgeMode;
}

/** A SPARQL 1.1 set function offered by the aggregate panel. */
export type AggregateFn = "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "SAMPLE" | "GROUP_CONCAT";

/** One `(FN(?target) AS ?alias)` projection. */
export interface BuilderAggregate {
  id: string;
  fn: AggregateFn;
  /** The aggregated variable name (without `?`), or `*` — `*` is COUNT-only. */
  target: string;
  distinct: boolean;
  /** The result variable name, without `?`. */
  alias: string;
  /** `GROUP_CONCAT` only — the `SEPARATOR` string. Empty/absent ⇒ the SPARQL default. */
  separator?: string;
}

/** One ORDER BY term — a variable name (without `?`), ascending unless `desc`. */
export interface BuilderOrder {
  variable: string;
  desc: boolean;
}

/** The whole canvas: what the user drew plus the projection/aggregation controls. */
export interface BuilderModel {
  nodes: BuilderNode[];
  edges: BuilderEdge[];
  distinct: boolean;
  aggregates: BuilderAggregate[];
  /** GROUP BY variable names, without `?`. */
  groupBy: string[];
  orderBy: BuilderOrder[];
  /** `null` ⇒ no LIMIT clause. */
  limit: number | null;
}

/** An empty canvas — the starting model. */
export function emptyModel(): BuilderModel {
  return {
    nodes: [],
    edges: [],
    distinct: false,
    aggregates: [],
    groupBy: [],
    orderBy: [],
    limit: 100,
  };
}

// ---------------------------------------------------------------------------
// Term rendering.
// ---------------------------------------------------------------------------

/** A syntactically valid SPARQL variable name (the conservative subset the builder accepts). */
const VARIABLE_RE = /^[A-Za-z_][A-Za-z0-9_]*$/;

/** A prefixed name `foo:bar` (or `:bar`) — the tolerant shape the builder passes through. */
const PREFIXED_NAME_RE = /^([A-Za-z][\w.-]*)?:[^\s<>"{}|^`\\]*$/;

/** A local part safe to emit after a prefix (no slashes, no trailing dot). */
const SAFE_LOCAL_RE = /^[A-Za-z0-9_][A-Za-z0-9_.\-]*$/;

/** Is this a variable name the builder will emit as `?name`? */
export function isValidVariable(name: string): boolean {
  return VARIABLE_RE.test(name);
}

/** `foo` → `?foo` (the model stores bare names; only the serializer adds the sigil). */
export function varRef(name: string): string {
  return `?${name}`;
}

/**
 * Abbreviate an absolute IRI against {@link COMMON_PREFIXES} when the local part is safe to
 * emit as a prefixed name, else `null`. Shared by the serializer and the picker labels, so the
 * text the canvas SHOWS is the text the query CARRIES.
 */
export function abbreviateIri(iri: string): string | null {
  for (const { prefix, iri: ns } of COMMON_PREFIXES) {
    if (!iri.startsWith(ns)) continue;
    const local = iri.slice(ns.length);
    if (local.length > 0 && SAFE_LOCAL_RE.test(local) && !local.endsWith(".")) {
      return `${prefix}:${local}`;
    }
  }
  return null;
}

/** Does this text look like an absolute IRI (rather than a prefixed name)? */
function looksAbsolute(value: string): boolean {
  if (value.includes("//")) return true;
  return COMMON_PREFIXES.some((p) => value.startsWith(p.iri));
}

/**
 * Render an IRI-position term exactly as it will appear in the query: `<…>` verbatim, an
 * absolute IRI abbreviated when a common prefix covers it (else angle-bracketed), and a
 * prefixed name passed through untouched. `a` is NOT special-cased here — see
 * {@link renderPredicate}.
 */
export function renderIri(value: string): string {
  const v = value.trim();
  if (v.startsWith("<") && v.endsWith(">")) return v;
  if (looksAbsolute(v)) return abbreviateIri(v) ?? `<${v}>`;
  if (PREFIXED_NAME_RE.test(v)) return v;
  return `<${v}>`;
}

/** As {@link renderIri}, but `a` (and `rdf:type`'s keyword form) stays the `a` keyword. */
export function renderPredicate(value: string): string {
  const v = value.trim();
  if (v === "a") return "a";
  return renderIri(v);
}

/** Escape a string literal's lexical form for a `"…"` SPARQL literal. */
export function escapeLiteral(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
}

/** Render a filter's value text as the SPARQL term its {@link ValueKind} calls for. */
export function renderValue(value: string, kind: ValueKind): string {
  const v = value.trim();
  switch (kind) {
    case "number":
      return v;
    case "boolean":
      return v.toLowerCase() === "true" ? "true" : "false";
    case "date":
      return `"${escapeLiteral(v)}"^^xsd:date`;
    case "iri":
      return renderIri(v);
    case "string":
    default:
      return `"${escapeLiteral(v)}"`;
  }
}

/** Build the FILTER expression (without the `FILTER(…)` wrapper) for one attribute filter. */
export function renderFilterExpr(variable: string, filter: AttributeFilter): string {
  const ref = varRef(variable);
  const lex = `"${escapeLiteral(filter.value.trim())}"`;
  switch (filter.op) {
    case "contains":
      return `CONTAINS(STR(${ref}), ${lex})`;
    case "starts":
      return `STRSTARTS(STR(${ref}), ${lex})`;
    case "ends":
      return `STRENDS(STR(${ref}), ${lex})`;
    case "regex":
      return filter.caseInsensitive
        ? `REGEX(STR(${ref}), ${lex}, "i")`
        : `REGEX(STR(${ref}), ${lex})`;
    default:
      return `${ref} ${filter.op} ${renderValue(filter.value, filter.kind)}`;
  }
}

// ---------------------------------------------------------------------------
// Validation.
// ---------------------------------------------------------------------------

/**
 * One problem with the drawn model. `error` blocks running (the query would be invalid or
 * would not mean what the canvas shows); `warning` is a heads-up the user may accept — most
 * often an undeclared prefix, which the panel cannot resolve for them without guessing.
 */
export interface BuilderIssue {
  severity: "error" | "warning";
  message: string;
  /** The node/edge/aggregate id the issue attaches to, when it has one. */
  subject?: string;
}

/** The prefix labels the serializer can declare automatically. */
const KNOWN_PREFIX_LABELS: ReadonlySet<string> = new Set(COMMON_PREFIXES.map((p) => p.prefix));

/** Collect an undeclared-prefix warning for one IRI-position term, if it needs one. */
function prefixWarning(value: string, where: string, subject: string): BuilderIssue | null {
  const v = value.trim();
  if (v === "a" || v.startsWith("<") || looksAbsolute(v)) return null;
  const m = PREFIXED_NAME_RE.exec(v);
  if (!m) return null;
  const label = m[1] ?? "";
  if (KNOWN_PREFIX_LABELS.has(label)) return null;
  return {
    severity: "warning",
    subject,
    message:
      `${where} uses the prefix \`${label}:\`, which is not one the builder can declare. ` +
      `Add a PREFIX line to the generated SPARQL, or use the full IRI.`,
  };
}

/** Every variable name the model binds (nodes, attributes, aggregate aliases). */
export function modelVariables(model: BuilderModel): string[] {
  const out: string[] = [];
  for (const n of model.nodes) {
    out.push(n.variable);
    for (const a of n.attributes) out.push(a.variable);
  }
  for (const g of model.aggregates) out.push(g.alias);
  return out;
}

/** Node ids bound only inside a `not-exists` branch — their variables are not visible outside. */
export function negatedNodeIds(model: BuilderModel): Set<string> {
  return new Set(model.edges.filter((e) => e.mode === "not-exists").map((e) => e.to));
}

/**
 * Check the drawn model against the things that would make the generated SPARQL invalid, or
 * make it mean something other than what the canvas shows. Deterministic order: structural
 * issues first, then projection/aggregation, then prefix warnings.
 */
export function validateModel(model: BuilderModel): BuilderIssue[] {
  const issues: BuilderIssue[] = [];
  const nodeIds = new Set(model.nodes.map((n) => n.id));
  const negated = negatedNodeIds(model);

  if (model.nodes.length === 0) {
    issues.push({ severity: "error", message: "The canvas is empty — add a node to start." });
  }

  // --- variables ---------------------------------------------------------
  const seen = new Map<string, number>();
  for (const name of modelVariables(model)) seen.set(name, (seen.get(name) ?? 0) + 1);
  for (const [name, count] of seen) {
    if (!isValidVariable(name)) {
      issues.push({
        severity: "error",
        message: `\`?${name}\` is not a valid variable name (letters, digits and _ only).`,
      });
    } else if (count > 1) {
      issues.push({
        severity: "error",
        message:
          `\`?${name}\` is bound ${count} times by different parts of the canvas. ` +
          `Rename one — two patterns sharing a variable would join, which is not what the ` +
          `diagram shows.`,
      });
    }
  }

  // --- nodes -------------------------------------------------------------
  const incident = new Map<string, number>();
  for (const e of model.edges) {
    if (nodeIds.has(e.from)) incident.set(e.from, (incident.get(e.from) ?? 0) + 1);
    if (nodeIds.has(e.to)) incident.set(e.to, (incident.get(e.to) ?? 0) + 1);
  }
  for (const n of model.nodes) {
    const contributes =
      n.classIri !== null || n.attributes.length > 0 || (incident.get(n.id) ?? 0) > 0;
    if (!contributes) {
      issues.push({
        severity: "error",
        subject: n.id,
        message: `\`?${n.variable}\` has no class, no attribute and no edge — it matches nothing.`,
      });
    }
    for (const a of n.attributes) {
      if (a.predicate.trim() === "") {
        issues.push({
          severity: "error",
          subject: n.id,
          message: `An attribute on \`?${n.variable}\` has no predicate.`,
        });
      }
      if (a.filter && a.filter.value.trim() === "") {
        issues.push({
          severity: "error",
          subject: n.id,
          message: `The filter on \`?${a.variable}\` has no value.`,
        });
      }
      if (a.filter && !STRING_OPS.has(a.filter.op) && a.filter.kind === "number") {
        if (!Number.isFinite(Number(a.filter.value.trim()))) {
          issues.push({
            severity: "error",
            subject: n.id,
            message: `The numeric filter on \`?${a.variable}\` is not a number.`,
          });
        }
      }
      if (a.filter && STRING_OPS.has(a.filter.op) && a.filter.kind !== "string") {
        issues.push({
          severity: "error",
          subject: n.id,
          message:
            `\`${a.filter.op}\` on \`?${a.variable}\` is a string function — set the value ` +
            `kind to text.`,
        });
      }
    }
  }

  // --- edges -------------------------------------------------------------
  for (const e of model.edges) {
    if (!nodeIds.has(e.from) || !nodeIds.has(e.to)) {
      issues.push({
        severity: "error",
        subject: e.id,
        message: "An edge points at a node that is no longer on the canvas.",
      });
      continue;
    }
    if (e.predicate.trim() === "") {
      issues.push({ severity: "error", subject: e.id, message: "An edge has no predicate." });
    }
    if (e.mode === "not-exists") {
      const otherEdges = model.edges.filter((o) => o.id !== e.id && (o.from === e.to || o.to === e.to));
      if (otherEdges.length > 0) {
        issues.push({
          severity: "error",
          subject: e.id,
          message:
            "A NOT-EXISTS branch must be a leaf in v1: its target node cannot carry further " +
            "edges. Model the rest by editing the generated SPARQL.",
        });
      }
      if (e.alternatives.length > 0) {
        issues.push({
          severity: "error",
          subject: e.id,
          message: "Alternative predicates (UNION) are only available on a required edge.",
        });
      }
    }
    if (e.mode === "optional" && e.alternatives.length > 0) {
      issues.push({
        severity: "error",
        subject: e.id,
        message: "Alternative predicates (UNION) are only available on a required edge.",
      });
    }
  }

  // --- projection + aggregation -----------------------------------------
  const projected = projectedVariables(model);
  for (const name of projected) {
    const owner = model.nodes.find((n) => n.variable === name);
    if (owner && negated.has(owner.id)) {
      issues.push({
        severity: "error",
        subject: owner.id,
        message:
          `\`?${name}\` is bound only inside a NOT-EXISTS branch, so it is not visible to the ` +
          `SELECT clause. Clear its projection.`,
      });
    }
  }
  for (const g of model.aggregates) {
    if (g.target !== "*" && !isValidVariable(g.target)) {
      issues.push({
        severity: "error",
        subject: g.id,
        message: `\`${g.fn}\` has no variable to aggregate.`,
      });
    }
    if (g.target === "*" && g.fn !== "COUNT") {
      issues.push({
        severity: "error",
        subject: g.id,
        message: `\`*\` can only be aggregated by COUNT.`,
      });
    }
    if (!isValidVariable(g.alias)) {
      issues.push({
        severity: "error",
        subject: g.id,
        message: `\`${g.fn}\` needs a result variable name.`,
      });
    }
  }
  if (model.aggregates.length > 0) {
    const grouped = new Set(model.groupBy);
    for (const name of projected) {
      if (!grouped.has(name)) {
        issues.push({
          severity: "error",
          message:
            `\`?${name}\` is projected alongside an aggregate but is not in GROUP BY — SPARQL ` +
            `rejects that. Add it to GROUP BY, or stop projecting it.`,
        });
      }
    }
  } else if (model.groupBy.length > 0) {
    issues.push({
      severity: "warning",
      message: "GROUP BY without an aggregate only de-duplicates — add an aggregate or DISTINCT.",
    });
  }

  const orderable = new Set([...projected, ...model.aggregates.map((g) => g.alias)]);
  for (const o of model.orderBy) {
    if (!orderable.has(o.variable)) {
      issues.push({
        severity: "warning",
        message:
          `ORDER BY \`?${o.variable}\` is not a projected variable; with an aggregate or ` +
          `DISTINCT in play the engine may reject it.`,
      });
    }
  }

  if (model.limit !== null && (!Number.isInteger(model.limit) || model.limit < 1)) {
    issues.push({ severity: "error", message: "LIMIT must be a positive whole number." });
  }

  // --- prefixes ----------------------------------------------------------
  for (const n of model.nodes) {
    if (n.classIri) {
      const w = prefixWarning(n.classIri, `The class of \`?${n.variable}\``, n.id);
      if (w) issues.push(w);
    }
    for (const a of n.attributes) {
      const w = prefixWarning(a.predicate, `The attribute \`?${a.variable}\``, n.id);
      if (w) issues.push(w);
    }
  }
  for (const e of model.edges) {
    for (const p of [e.predicate, ...e.alternatives]) {
      const w = prefixWarning(p, "An edge predicate", e.id);
      if (w) issues.push(w);
    }
  }

  return issues;
}

/** The variable names the SELECT clause will project, in canvas order. */
export function projectedVariables(model: BuilderModel): string[] {
  const out: string[] = [];
  for (const n of model.nodes) {
    if (n.projected) out.push(n.variable);
    for (const a of n.attributes) if (a.projected) out.push(a.variable);
  }
  return out;
}

// ---------------------------------------------------------------------------
// Serialization.
// ---------------------------------------------------------------------------

const INDENT = "  ";

/** Render one node's own patterns (class + attributes + attribute filters). */
function nodePatterns(node: BuilderNode, indent: string): string[] {
  const lines: string[] = [];
  const subject = varRef(node.variable);
  if (node.classIri) lines.push(`${indent}${subject} a ${renderIri(node.classIri)} .`);
  for (const attr of node.attributes) {
    const triple = `${subject} ${renderPredicate(attr.predicate)} ${varRef(attr.variable)} .`;
    // An OPTIONAL attribute's filter goes INSIDE the OPTIONAL: filtering outside would drop the
    // whole row when the attribute is absent, which is not what "optional + filter" shows.
    if (attr.optional) {
      if (attr.filter) {
        lines.push(`${indent}OPTIONAL {`);
        lines.push(`${indent}${INDENT}${triple}`);
        lines.push(`${indent}${INDENT}FILTER(${renderFilterExpr(attr.variable, attr.filter)})`);
        lines.push(`${indent}}`);
      } else {
        lines.push(`${indent}OPTIONAL { ${triple} }`);
      }
    } else {
      lines.push(`${indent}${triple}`);
      if (attr.filter) {
        lines.push(`${indent}FILTER(${renderFilterExpr(attr.variable, attr.filter)})`);
      }
    }
  }
  return lines;
}

/** Render the aggregate's projection expression, e.g. `(COUNT(DISTINCT ?x) AS ?n)`. */
export function renderAggregate(agg: BuilderAggregate): string {
  const arg = agg.target === "*" ? "*" : varRef(agg.target);
  const distinct = agg.distinct && agg.target !== "*" ? "DISTINCT " : "";
  const sep =
    agg.fn === "GROUP_CONCAT" && agg.separator
      ? `; SEPARATOR="${escapeLiteral(agg.separator)}"`
      : "";
  return `(${agg.fn}(${distinct}${arg}${sep}) AS ${varRef(agg.alias)})`;
}

/**
 * Serialize the drawn model to standard SPARQL 1.1, WITHOUT the PREFIX header (see
 * {@link buildQuery}, which prepends exactly the prefixes the body uses).
 *
 * Emission order is deterministic — nodes in canvas order (each with its own patterns), then
 * edges in canvas order — so the text only changes when the diagram does.
 */
export function serializeModel(model: BuilderModel): string {
  const byId = new Map(model.nodes.map((n) => [n.id, n]));
  const negated = negatedNodeIds(model);
  const lines: string[] = [];

  // 1. Node patterns — but a node bound only inside a negation is emitted THERE, not here.
  for (const node of model.nodes) {
    if (negated.has(node.id)) continue;
    lines.push(...nodePatterns(node, INDENT));
  }

  // 2. Edges.
  for (const edge of model.edges) {
    const from = byId.get(edge.from);
    const to = byId.get(edge.to);
    if (!from || !to) continue;
    const s = varRef(from.variable);
    const o = varRef(to.variable);
    const triple = (p: string) => `${s} ${renderPredicate(p)} ${o} .`;

    if (edge.mode === "not-exists") {
      // The AND-NOT branch: the edge plus everything asserted on its (leaf) target node.
      const inner = nodePatterns(to, INDENT + INDENT);
      lines.push(`${INDENT}FILTER NOT EXISTS {`);
      lines.push(`${INDENT}${INDENT}${triple(edge.predicate)}`);
      lines.push(...inner);
      lines.push(`${INDENT}}`);
      continue;
    }

    const alts = [edge.predicate, ...edge.alternatives].filter((p) => p.trim() !== "");
    const body =
      alts.length > 1
        ? alts.map((p) => `{ ${triple(p).replace(/ \.$/, "")} }`).join(" UNION ")
        : triple(edge.predicate);

    lines.push(edge.mode === "optional" ? `${INDENT}OPTIONAL { ${body} }` : `${INDENT}${body}`);
  }

  const projected = projectedVariables(model);
  const projection = [
    ...projected.map(varRef),
    ...model.aggregates.map(renderAggregate),
  ];
  // An empty projection is `SELECT *` — honest (it is what SPARQL means), never a silent guess.
  const selectList = projection.length > 0 ? projection.join(" ") : "*";
  const distinct = model.distinct ? "DISTINCT " : "";

  const out: string[] = [`SELECT ${distinct}${selectList}`, "WHERE {"];
  out.push(...lines);
  out.push("}");
  if (model.groupBy.length > 0) out.push(`GROUP BY ${model.groupBy.map(varRef).join(" ")}`);
  if (model.orderBy.length > 0) {
    out.push(
      `ORDER BY ${model.orderBy
        .map((o) => (o.desc ? `DESC(${varRef(o.variable)})` : varRef(o.variable)))
        .join(" ")}`,
    );
  }
  if (model.limit !== null) out.push(`LIMIT ${model.limit}`);
  return out.join("\n");
}

/**
 * The PREFIX lines the body actually needs — derived from the rendered text, so a prefix is
 * declared iff it is used (no speculative header, no missing declaration).
 */
export function requiredPrefixLines(body: string): string {
  // Blank out `<…>` IRIs and `"…"` literals FIRST: a namespace inside an angle-bracket IRI is
  // already absolute (it needs no PREFIX), and a colon inside a filter's string value is not a
  // prefixed name at all. Scanning the raw text would over-declare on both.
  const scan = body.replace(/<[^>]*>/g, " ").replace(/"(?:[^"\\]|\\.)*"/g, '""');
  const used = new Set<string>();
  for (const m of scan.matchAll(/(?:^|[^\w.:-])([A-Za-z][\w.-]*):[A-Za-z_]/g)) used.add(m[1]);
  const lines = COMMON_PREFIXES.filter((p) => used.has(p.prefix)).map(
    (p) => `PREFIX ${p.prefix}: <${p.iri}>`,
  );
  return lines.join("\n");
}

/** The generated query plus everything the panel needs to be honest about it. */
export interface BuiltQuery {
  /** The full SPARQL text (PREFIX header + query). Always produced, even when invalid. */
  sparql: string;
  issues: BuilderIssue[];
  /** True when no issue has severity `error` — i.e. the panel may offer to RUN it. */
  runnable: boolean;
}

/**
 * Build the SPARQL for a drawn model. Best-effort by design: the text is produced even when
 * the model has errors, so the user can always read and fix it — `runnable` is what gates the
 * Run button, never the visibility of the query.
 */
export function buildQuery(model: BuilderModel): BuiltQuery {
  const issues = validateModel(model);
  const body = serializeModel(model);
  const header = requiredPrefixLines(body);
  return {
    sparql: header ? `${header}\n\n${body}` : body,
    issues,
    runnable: !issues.some((i) => i.severity === "error"),
  };
}

// ---------------------------------------------------------------------------
// Schema introspection — the shape-aware pickers.
// ---------------------------------------------------------------------------

/**
 * Where a suggestion came from. This is surfaced in the picker, because the two mean
 * genuinely different things:
 *   * `shape` — DECLARED by a SHACL shape present in the loaded store (`sh:targetClass` +
 *     `sh:property`/`sh:path`). A statement of intent: the property is expected, whether or
 *     not any instance uses it.
 *   * `data`  — OBSERVED in the loaded store (a characteristic set: the predicates that
 *     co-occur on subjects of a class). A statement of fact about THIS data — not a schema,
 *     and not a guarantee about data you have not loaded.
 */
export type SuggestionSource = "shape" | "data";

/** What the object position of a predicate holds, as observed from a sampled object term. */
export type ObjectKind = "iri" | "literal" | "unknown";

/** One predicate offered by the picker for a given class. */
export interface PredicateSuggestion {
  /** The predicate IRI (full, as it came out of the store). */
  predicate: string;
  source: SuggestionSource;
  /** Triples observed with this predicate on this class; `null` for a shape-only suggestion. */
  uses: number | null;
  /**
   * The object position's kind. For `data` this is read from a SAMPLED object term — it
   * describes the sample, not every triple. For `shape` it is inferred from `sh:datatype` /
   * `sh:nodeKind` / `sh:class` when the shape declares one.
   */
  objectKind: ObjectKind;
  /** `sh:datatype` or `sh:class` when a shape declared one, else `null`. */
  valueType: string | null;
}

/** One class offered by the node-type picker. */
export interface ClassSuggestion {
  classIri: string;
  /** Instances observed in the loaded store; `null` when the class is only shape-declared. */
  instances: number | null;
  source: SuggestionSource;
}

/** The picker index: classes, and the predicates suggested for each. */
export interface SchemaIndex {
  classes: ClassSuggestion[];
  /** class IRI → its suggestions (shape-declared first, then observed, deduped by predicate). */
  byClass: Map<string, PredicateSuggestion[]>;
  /** Predicates observed anywhere, for an untyped node. */
  anyClass: PredicateSuggestion[];
  /** True when at least one suggestion came from a SHACL shape in the store. */
  hasShapes: boolean;
}

/**
 * The class picker's source query — instance counts per `rdf:type`, most populous first.
 * Capped so introspecting a large store cannot stall the panel; the cap is stated in the UI.
 */
export const CLASS_INDEX_QUERY = `SELECT ?class (COUNT(?s) AS ?instances)
WHERE { ?s a ?class }
GROUP BY ?class
ORDER BY DESC(?instances)
LIMIT 200`;

/**
 * The characteristic-set query: for each class, the predicates its instances actually carry,
 * with a use count and one SAMPLED object (whose term type tells the picker whether the
 * predicate leads to a node or to a literal attribute).
 */
export const CHARACTERISTIC_SET_QUERY = `SELECT ?class ?p (COUNT(?o) AS ?uses) (SAMPLE(?o) AS ?sample)
WHERE {
  ?s a ?class .
  ?s ?p ?o .
}
GROUP BY ?class ?p
ORDER BY ?class DESC(?uses)
LIMIT 2000`;

/** Predicates observed anywhere — the fallback picker for an untyped node. */
export const ANY_PREDICATE_QUERY = `SELECT ?p (COUNT(?o) AS ?uses) (SAMPLE(?o) AS ?sample)
WHERE { ?s ?p ?o }
GROUP BY ?p
ORDER BY DESC(?uses)
LIMIT 300`;

/**
 * SHACL shapes PRESENT IN THE LOADED STORE — `sh:targetClass` + `sh:property`/`sh:path`. Only
 * IRI paths are read: a shape whose `sh:path` is a path expression (a blank-node sequence,
 * alternative or inverse) is skipped rather than mis-rendered as a single predicate.
 *
 * Shapes loaded into the SHACL TOOL's editor are not in the store and are invisible here — the
 * panel says so rather than implying the shapes graph was consulted.
 */
export const SHAPE_PROPERTY_QUERY = `PREFIX sh: <http://www.w3.org/ns/shacl#>
SELECT ?class ?path ?datatype ?nodeKind ?valueClass
WHERE {
  ?shape sh:targetClass ?class ;
         sh:property ?ps .
  ?ps sh:path ?path .
  FILTER(isIRI(?path))
  OPTIONAL { ?ps sh:datatype ?datatype }
  OPTIONAL { ?ps sh:nodeKind ?nodeKind }
  OPTIONAL { ?ps sh:class ?valueClass }
}
LIMIT 1000`;

/** Read one binding's term, or `undefined` when the variable is unbound in this row. */
function term(row: Record<string, SparqlTerm>, name: string): SparqlTerm | undefined {
  return row[name];
}

/** The IRI value of a binding, or `null` when absent / not an IRI. */
function iriValue(row: Record<string, SparqlTerm>, name: string): string | null {
  const t = term(row, name);
  return t && t.type === "uri" ? t.value : null;
}

/** A non-negative integer from a numeric-literal binding, or `null`. */
function countValue(row: Record<string, SparqlTerm>, name: string): number | null {
  const t = term(row, name);
  if (!t) return null;
  const n = Number(t.value);
  return Number.isFinite(n) ? n : null;
}

/** Rows of the SELECT result, or `[]` for anything that is not a SELECT result. */
function rows(results: SparqlResults | null): Record<string, SparqlTerm>[] {
  return results?.results?.bindings ?? [];
}

/** Parse {@link CLASS_INDEX_QUERY} rows into class suggestions (rows without an IRI dropped). */
export function parseClassRows(results: SparqlResults | null): ClassSuggestion[] {
  const out: ClassSuggestion[] = [];
  for (const row of rows(results)) {
    const classIri = iriValue(row, "class");
    if (!classIri) continue;
    out.push({ classIri, instances: countValue(row, "instances"), source: "data" });
  }
  return out;
}

/** The object kind a sampled term implies (a blank node is a node, like an IRI). */
function sampleKind(t: SparqlTerm | undefined): ObjectKind {
  if (!t) return "unknown";
  if (t.type === "uri" || t.type === "bnode") return "iri";
  if (t.type === "literal") return "literal";
  return "unknown";
}

/** Parse {@link CHARACTERISTIC_SET_QUERY} rows into per-class OBSERVED suggestions. */
export function parseCharacteristicRows(
  results: SparqlResults | null,
): Map<string, PredicateSuggestion[]> {
  const out = new Map<string, PredicateSuggestion[]>();
  for (const row of rows(results)) {
    const classIri = iriValue(row, "class");
    const predicate = iriValue(row, "p");
    if (!classIri || !predicate) continue;
    const list = out.get(classIri) ?? [];
    list.push({
      predicate,
      source: "data",
      uses: countValue(row, "uses"),
      objectKind: sampleKind(term(row, "sample")),
      valueType: null,
    });
    out.set(classIri, list);
  }
  return out;
}

/** Parse {@link ANY_PREDICATE_QUERY} rows into the untyped-node suggestions. */
export function parseAnyPredicateRows(results: SparqlResults | null): PredicateSuggestion[] {
  const out: PredicateSuggestion[] = [];
  for (const row of rows(results)) {
    const predicate = iriValue(row, "p");
    if (!predicate) continue;
    out.push({
      predicate,
      source: "data",
      uses: countValue(row, "uses"),
      objectKind: sampleKind(term(row, "sample")),
      valueType: null,
    });
  }
  return out;
}

const SH_IRI = "http://www.w3.org/ns/shacl#IRI";
const SH_BNODE = "http://www.w3.org/ns/shacl#BlankNode";
const SH_BNODE_OR_IRI = "http://www.w3.org/ns/shacl#BlankNodeOrIRI";
const SH_LITERAL = "http://www.w3.org/ns/shacl#Literal";

/** Parse {@link SHAPE_PROPERTY_QUERY} rows into per-class DECLARED suggestions. */
export function parseShapeRows(
  results: SparqlResults | null,
): Map<string, PredicateSuggestion[]> {
  const out = new Map<string, PredicateSuggestion[]>();
  for (const row of rows(results)) {
    const classIri = iriValue(row, "class");
    const predicate = iriValue(row, "path");
    if (!classIri || !predicate) continue;
    const datatype = iriValue(row, "datatype");
    const valueClass = iriValue(row, "valueClass");
    const nodeKind = iriValue(row, "nodeKind");
    let objectKind: ObjectKind = "unknown";
    if (datatype || nodeKind === SH_LITERAL) objectKind = "literal";
    else if (valueClass || nodeKind === SH_IRI || nodeKind === SH_BNODE || nodeKind === SH_BNODE_OR_IRI) {
      objectKind = "iri";
    }
    const list = out.get(classIri) ?? [];
    list.push({
      predicate,
      source: "shape",
      uses: null,
      objectKind,
      valueType: datatype ?? valueClass,
    });
    out.set(classIri, list);
  }
  return out;
}

/**
 * Merge shape-declared and data-observed suggestions for one class. Shape-declared entries
 * lead (they are the modelled intent); a predicate present in both keeps the shape's declared
 * value type AND gains the observed use count, so the picker can show both facts at once.
 */
export function mergeSuggestions(
  shape: readonly PredicateSuggestion[],
  data: readonly PredicateSuggestion[],
): PredicateSuggestion[] {
  const observed = new Map(data.map((d) => [d.predicate, d]));
  const out: PredicateSuggestion[] = [];
  const taken = new Set<string>();
  for (const s of shape) {
    if (taken.has(s.predicate)) continue;
    taken.add(s.predicate);
    const d = observed.get(s.predicate);
    out.push({
      ...s,
      uses: d?.uses ?? null,
      objectKind: s.objectKind !== "unknown" ? s.objectKind : (d?.objectKind ?? "unknown"),
    });
  }
  for (const d of data) {
    if (taken.has(d.predicate)) continue;
    taken.add(d.predicate);
    out.push(d);
  }
  return out;
}

/** Assemble the picker index from the four introspection results (any of which may be null). */
export function buildSchemaIndex(input: {
  classes: SparqlResults | null;
  characteristics: SparqlResults | null;
  anyPredicates: SparqlResults | null;
  shapes: SparqlResults | null;
}): SchemaIndex {
  const dataByClass = parseCharacteristicRows(input.characteristics);
  const shapeByClass = parseShapeRows(input.shapes);
  const classes = parseClassRows(input.classes);

  const byClass = new Map<string, PredicateSuggestion[]>();
  const classIris = new Set([...dataByClass.keys(), ...shapeByClass.keys()]);
  for (const iri of classIris) {
    byClass.set(iri, mergeSuggestions(shapeByClass.get(iri) ?? [], dataByClass.get(iri) ?? []));
  }

  // A class a shape targets but no instance carries is still worth offering — the shape says
  // it is expected. It is marked `shape` with a null instance count, never a fabricated one.
  const known = new Set(classes.map((c) => c.classIri));
  for (const iri of shapeByClass.keys()) {
    if (known.has(iri)) continue;
    known.add(iri);
    classes.push({ classIri: iri, instances: null, source: "shape" });
  }

  return {
    classes,
    byClass,
    anyClass: parseAnyPredicateRows(input.anyPredicates),
    hasShapes: shapeByClass.size > 0,
  };
}

/** The suggestions to offer for a node — its class's, or the store-wide list when untyped. */
export function suggestionsFor(index: SchemaIndex, classIri: string | null): PredicateSuggestion[] {
  if (classIri === null) return index.anyClass;
  return index.byClass.get(classIri) ?? index.anyClass;
}

// ---------------------------------------------------------------------------
// Naming helpers (used by the canvas when it creates nodes/attributes).
// ---------------------------------------------------------------------------

/** The last path/hash segment of an IRI, lower-camel-cased into a variable-safe stem. */
export function variableStem(iri: string): string {
  // Trailing separators carry no segment — `http://example.org/` should name itself after the
  // authority, not collapse the whole IRI into one run-together token.
  const trimmed = iri.replace(/[#/:]+$/, "");
  const cut = Math.max(trimmed.lastIndexOf("#"), trimmed.lastIndexOf("/"), trimmed.lastIndexOf(":"));
  const raw = cut >= 0 && cut < trimmed.length - 1 ? trimmed.slice(cut + 1) : trimmed;
  const cleaned = raw.replace(/[^A-Za-z0-9_]/g, "");
  if (cleaned === "") return "node";
  const stem = cleaned[0].toLowerCase() + cleaned.slice(1);
  return VARIABLE_RE.test(stem) ? stem : `n${stem}`;
}

/** `stem`, `stem2`, `stem3`, … — the first form not already taken. */
export function uniqueVariable(stem: string, taken: Iterable<string>): string {
  const used = new Set(taken);
  if (!used.has(stem)) return stem;
  for (let i = 2; ; i++) {
    const candidate = `${stem}${i}`;
    if (!used.has(candidate)) return candidate;
  }
}

/** A short label for an IRI in the canvas UI — prefixed form when possible, else last segment. */
export function iriLabel(iri: string): string {
  return abbreviateIri(iri) ?? variableStem(iri);
}
