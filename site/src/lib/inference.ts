// [OPUS-4.8] sq-0po6 — pure, framework-free helpers for the live /surface/inference page:
// parse the bundle's N-Triples output into clickable triples, parse the stats JSON, and
// shape a `why()` proof tree for rendering. Kept separate from the React component so they
// can be unit-tested under node:test without a DOM (test/inference.test.mjs).

import type {
  MaterializeStats,
  ProofNode,
  ProofTree,
  ReasoningProfile,
} from "./reason-wasm";

/** A single parsed N-Triples triple: the three term strings exactly as the bundle emits
 *  them (so they can be fed straight back into `Reasoner.why(s, p, o)`). */
export interface Triple {
  subject: string;
  predicate: string;
  object: string;
}

/**
 * Splits ONE N-Triples line into its three term strings, or `null` if the line is blank, a
 * comment, or not a well-formed `S P O .` triple. Terms are returned VERBATIM (no
 * unescaping) so they round-trip back into `Reasoner.why`. Whitespace-tolerant: a term is an
 * IRI (`<…>`), a blank node (`_:…`), or a literal (`"…"` with optional `^^<dt>` / `@lang`,
 * honouring `\"` and `\\` escapes inside the quotes).
 */
export function parseNTriplesLine(line: string): Triple | null {
  const trimmed = line.trim();
  if (trimmed.length === 0 || trimmed.startsWith("#")) return null;
  // Drop the trailing ` .` statement terminator.
  if (!trimmed.endsWith(".")) return null;
  const body = trimmed.slice(0, -1).trim();

  const terms: string[] = [];
  let i = 0;
  while (i < body.length && terms.length < 3) {
    // Skip inter-term whitespace.
    while (i < body.length && /\s/.test(body[i])) i++;
    if (i >= body.length) break;
    const start = i;
    const c = body[i];
    if (c === "<") {
      // IRI: up to the next '>'.
      const end = body.indexOf(">", i);
      if (end < 0) return null;
      i = end + 1;
    } else if (c === '"') {
      // Literal: consume the quoted body (honouring escapes), then an optional
      // datatype (`^^<…>`) or language tag (`@…`).
      i++;
      while (i < body.length && body[i] !== '"') {
        if (body[i] === "\\") i++; // skip the escaped char
        i++;
      }
      if (i >= body.length) return null; // unterminated literal
      i++; // closing quote
      if (body.startsWith("^^", i)) {
        const end = body.indexOf(">", i);
        if (end < 0) return null;
        i = end + 1;
      } else if (body[i] === "@") {
        i++;
        while (i < body.length && /[\w-]/.test(body[i])) i++;
      }
    } else if (c === "_" && body[i + 1] === ":") {
      // Blank node: up to whitespace.
      while (i < body.length && !/\s/.test(body[i])) i++;
    } else {
      return null; // unexpected token
    }
    terms.push(body.slice(start, i));
  }
  // After three terms only trailing whitespace may remain.
  while (i < body.length && /\s/.test(body[i])) i++;
  if (terms.length !== 3 || i !== body.length) return null;
  return { subject: terms[0], predicate: terms[1], object: terms[2] };
}

/**
 * Parses a whole N-Triples document (the bundle's `materialize` / `entailed` output) into
 * its triples, skipping blank/comment lines. Malformed lines are dropped (the bundle emits
 * canonical N-Triples, so this is robust to a stray blank line, not a lenient parser).
 */
export function parseNTriples(nt: string): Triple[] {
  const out: Triple[] = [];
  for (const line of nt.split("\n")) {
    const t = parseNTriplesLine(line);
    if (t) out.push(t);
  }
  return out;
}

/**
 * Parses the `Reasoner.materializeStats` JSON, validating the shape. Throws a clear error
 * if the document is not the expected `{profile, baseTriples, closureTriples, entailed}`.
 */
export function parseStats(json: string): MaterializeStats {
  const v = JSON.parse(json) as unknown;
  if (
    typeof v !== "object" ||
    v === null ||
    typeof (v as MaterializeStats).baseTriples !== "number" ||
    typeof (v as MaterializeStats).closureTriples !== "number" ||
    typeof (v as MaterializeStats).entailed !== "number"
  ) {
    throw new Error(`unexpected materializeStats JSON: ${json}`);
  }
  return v as MaterializeStats;
}

/**
 * Parses the `Reasoner.why` result: a {@link ProofTree} or `null` when the triple is not
 * entailed (the bundle returns the JSON literal `null`). Throws if the JSON is malformed.
 */
export function parseProof(json: string): ProofTree | null {
  const v = JSON.parse(json) as unknown;
  if (v === null) return null;
  if (
    typeof v !== "object" ||
    typeof (v as ProofTree).root !== "number" ||
    !Array.isArray((v as ProofTree).nodes)
  ) {
    throw new Error(`unexpected why() JSON: ${json}`);
  }
  return v as ProofTree;
}

// ---- term display ----

const PREFIXES: [string, string][] = [
  ["http://www.w3.org/1999/02/22-rdf-syntax-ns#", "rdf:"],
  ["http://www.w3.org/2000/01/rdf-schema#", "rdfs:"],
  ["http://www.w3.org/2002/07/owl#", "owl:"],
  ["http://www.w3.org/2001/XMLSchema#", "xsd:"],
  ["http://ex/", "ex:"],
  ["http://example.org/", "ex:"],
];

/**
 * Compacts an N-Triples term for display: an IRI (`<…>`) to a CURIE using the well-known
 * RDF/RDFS/OWL/XSD/example prefixes (anything else keeps its `<…>`); literals and blank
 * nodes pass through unchanged.
 */
export function shortenTerm(term: string): string {
  if (term.startsWith("<") && term.endsWith(">")) {
    const iri = term.slice(1, -1);
    for (const [ns, prefix] of PREFIXES) {
      if (iri.startsWith(ns)) return prefix + iri.slice(ns.length);
    }
    return term;
  }
  return term;
}

/** A one-line, CURIE-compacted rendering of a triple (`s p o`). */
export function formatTriple(t: Triple): string {
  return `${shortenTerm(t.subject)} ${shortenTerm(t.predicate)} ${shortenTerm(t.object)}`;
}

// ---- proof tree layout ----

/** One row of the rendered proof tree: a node plus its indentation depth (root = 0). */
export interface ProofRow {
  node: ProofNode;
  depth: number;
  /** True the SECOND+ time a shared sub-proof node is reached (rendered as a back-ref). */
  repeated: boolean;
}

/**
 * Flattens a {@link ProofTree} into an indented, root-first list of rows for rendering —
 * mirroring `sparq-reason`'s `ProofTree::to_text` traversal: each node prints once with its
 * premises nested beneath it; a node reached again (a shared sub-proof) is emitted as a
 * `repeated` back-reference row WITHOUT re-expanding its premises. Robust against a
 * malformed premise index (out-of-range premises are skipped).
 */
export function proofRows(proof: ProofTree): ProofRow[] {
  const rows: ProofRow[] = [];
  const expanded = new Set<number>();
  const walk = (id: number, depth: number): void => {
    const node = proof.nodes[id];
    if (!node) return;
    const repeated = expanded.has(id);
    rows.push({ node, depth, repeated });
    if (repeated) return;
    expanded.add(id);
    for (const p of node.premises) walk(p, depth + 1);
  };
  walk(proof.root, 0);
  return rows;
}

/** A human label for a proof-rule identifier (leaf vs derived). */
export function ruleLabel(rule: string): string {
  if (rule === "asserted") return "asserted";
  return rule;
}

export type { MaterializeStats, ProofNode, ProofTree, ReasoningProfile };
