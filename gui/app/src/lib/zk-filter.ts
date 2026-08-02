// [OPUS-5] sq-ixc3.17 — the ZK tool's PURE core: which live-store values the shipped circuit
// member can prove about, which comparison is being claimed, and the honest reason a row cannot
// be proven in-tab.
//
// Ported from the site's `/showcase/zk-car-hire` demo (site/src/lib/zk-prover.ts) with the
// marketing chrome CUT and the operand rebound to the LIVE WORKSPACE STORE: instead of a fixed
// "age ≥ 25" over a hard-coded holder, the user runs a SPARQL SELECT over their own imported data
// and proves a comparison about one of the returned integers. The circuit's `op` and `bound` are
// PUBLIC inputs (zk/compose/filter_int_d2/src/main.nr), so the operator and the constant are the
// user's to choose here — only the hidden operand is constrained (see below).
//
// HONESTY: the in-tab proving mechanics are real, but the sparq ZK estate is research-grade —
// the v1 verifier is internally re-audited only and external accredited-cryptographer sign-off
// is pending (bead sq-qhy4). A proof produced here is NOT a production cryptographic guarantee.
// See SECURITY.md and compliance/cryptoreview/README.md.
//
// This module is PURE (no bb.js, no React, no `@/` aliases) so `npm run test:unit` can exercise
// it under `node --test`; the wasm-loading half lives in `zk-prover.ts`.

import type { SparqlResults } from "@sparq/client";

/**
 * The circuit-family member whose ACIR the workbench ships: `filter_int_d2`, i.e. a hidden
 * `xsd:integer` operand of exactly D = 2 canonical decimal digits (10…99). The digit count is a
 * comptime family parameter (blake3's blackbox needs a fixed token length), so a value with a
 * different digit count needs a different member's ACIR shipped — it is not a bug, it is the
 * documented family envelope. Choosing a member leaks ceil(log10(value)) of the hidden operand.
 */
export const CIRCUIT_MEMBER = "filter_int_d2";
export const CIRCUIT_DIGITS = 2;

/**
 * Field encodings of `"<value>"^^xsd:integer`, precomputed natively by
 * `sparq_zk_compose::build::encode_int_literal` (the SAME encoder the scan proof uses for the
 * operand column). Each is the `operand_enc` public input that binds the hidden value to the
 * committed credential; the circuit rebuilds the canonical N-Triples token in-circuit, hashes it
 * with the blake3 blackbox, and asserts the encoding matches.
 *
 * These cannot be derived in JS — they come from the native encoder — so a store value is
 * provable in-tab only if its anchor was committed here. That is the honest limit this tool
 * surfaces per row rather than hiding.
 *
 * DRIFT GUARD: these are the same four constants the site's `AGE_OPERAND_ENC` carries, pinned
 * against the native encoder by `crates/sparq-zk-compose/tests/site_age_enc_drift.rs`. If a
 * circuit/encoder bump changes the term encoding, that test fails RED and prints the regenerated
 * hex — update BOTH copies. A stale anchor makes the witness solve fail with "operand encoding
 * mismatch", which this tool surfaces verbatim rather than degrading to a fabricated result.
 */
export const COMMITTED_TERM_ANCHORS: Record<string, string> = {
  "24": "0x1c8a81ea95b253e105b99209deff1a4908be9568e588fbd89afea9f49f5f20cf",
  "25": "0x2b5caeb2bbd290ab32434a9109030784c7faebadee7a9908d24dccb847910d1d",
  "30": "0x132fa587351bf3f12fd3cbed64d5526f28791099d1d40870f94595873c78fa72",
  "42": "0x1a4aa7fd962d0004ac2294cc98471ea1ebfdad74a8f702e89fedf83f92d0f97b",
};

/** The comparison selector, matching `sparq_zk_compose_core::filter_int`'s OP_* globals. */
export type OpCode = 0 | 1 | 2 | 3 | 4 | 5;

export interface OpDef {
  code: OpCode;
  /** The SPARQL/maths symbol shown in the UI. */
  symbol: string;
  /** The circuit's global name, so the panel can name what it is actually asserting. */
  circuitName: string;
}

export const ZK_OPS: readonly OpDef[] = [
  { code: 0, symbol: "<", circuitName: "OP_LT" },
  { code: 1, symbol: "≤", circuitName: "OP_LE" },
  { code: 2, symbol: ">", circuitName: "OP_GT" },
  { code: 3, symbol: "≥", circuitName: "OP_GE" },
  { code: 4, symbol: "=", circuitName: "OP_EQ" },
  { code: 5, symbol: "≠", circuitName: "OP_NE" },
] as const;

/**
 * The verdict the circuit will assert for `value <op> bound`. Computed here so the panel can
 * publish the SAME `expected` public input the prover passes — the circuit asserts the two agree,
 * so a claim that disagrees with the hidden value is unsatisfiable and no proof can be produced.
 */
export function evaluateOp(value: number, op: OpCode, bound: number): boolean {
  switch (op) {
    case 0:
      return value < bound;
    case 1:
      return value <= bound;
    case 2:
      return value > bound;
    case 3:
      return value >= bound;
    case 4:
      return value === bound;
    default:
      return value !== bound;
  }
}

/**
 * The canonical decimal digit string the circuit witnesses, or `null` when this literal is
 * outside the shipped member's envelope. `filter_int` requires ASCII decimal digits, no leading
 * zero, and exactly {@link CIRCUIT_DIGITS} of them.
 */
export function canonicalDigits(literal: string): string | null {
  const s = literal.trim();
  if (!/^[1-9][0-9]*$/.test(s)) return null;
  if (s.length !== CIRCUIT_DIGITS) return null;
  return s;
}

/** The committed `operand_enc` anchor for a canonical digit string, if one shipped. */
export function termAnchor(digits: string): string | undefined {
  return COMMITTED_TERM_ANCHORS[digits];
}

/** One row of the live-store SELECT, classified for provability. */
export interface ZkCandidate {
  /** 0-based row index in the SELECT result. */
  row: number;
  /** The `?subject` term (or "" when unbound) — so a candidate is traceable back to the data. */
  subject: string;
  /** The value term exactly as the store returned it. */
  literal: string;
  /** The canonical digit string, when the literal is inside the member's envelope. */
  digits: string | null;
  /** The numeric value, when parseable. Never a public input to the proof. */
  value: number | null;
  /** The committed term anchor, when one shipped for this value. */
  anchor?: string;
  /** True iff this row can be proven with the ACIR + anchors this build ships. */
  provable: boolean;
  /** Why not, when `provable` is false. Empty when it is. */
  reason: string;
}

export interface CandidateSelection {
  candidates: ZkCandidate[];
  subjectVar: string;
  valueVar: string;
}

/**
 * The default SPARQL the ZK tool opens with. It binds over the workbench's seeded sample graph
 * (four `foaf:Person`s with `foaf:age`) but is a plain SELECT — any query projecting a subject
 * and an integer works the same way.
 */
export const DEFAULT_ZK_QUERY = `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?subject ?value WHERE {
  ?subject foaf:age ?value .
}
ORDER BY ?subject`;

/**
 * Classify every row of a SELECT over the live store: which values this build can prove a
 * comparison about, and — for the rest — the specific reason it cannot. Non-provable rows are
 * KEPT and labelled rather than filtered away, so the tool never implies coverage it lacks.
 */
export function candidatesFromResults(results: SparqlResults): CandidateSelection {
  const vars = results.head?.vars ?? [];
  const subjectVar = vars.includes("subject") ? "subject" : (vars[0] ?? "");
  const valueVar = vars.includes("value")
    ? "value"
    : (vars.find((v) => v !== subjectVar) ?? "");

  const bindings = results.results?.bindings ?? [];
  const candidates = bindings.map((row, i): ZkCandidate => {
    const subject = (subjectVar ? row[subjectVar]?.value : undefined) ?? "";
    const term = valueVar ? row[valueVar] : undefined;
    if (!term) {
      return {
        row: i,
        subject,
        literal: "",
        digits: null,
        value: null,
        provable: false,
        reason: `?${valueVar || "value"} is unbound`,
      };
    }
    const digits = canonicalDigits(term.value);
    if (!digits) {
      return {
        row: i,
        subject,
        literal: term.value,
        digits: null,
        value: Number.isInteger(Number(term.value)) ? Number(term.value) : null,
        provable: false,
        reason: `outside the ${CIRCUIT_MEMBER} envelope — it takes a canonical xsd:integer of exactly ${CIRCUIT_DIGITS} digits`,
      };
    }
    const anchor = termAnchor(digits);
    if (!anchor) {
      return {
        row: i,
        subject,
        literal: term.value,
        digits,
        value: Number(digits),
        provable: false,
        reason:
          "no committed term anchor ships for this value — the operand_enc encoding comes from the native encoder and cannot be derived in-tab",
      };
    }
    return {
      row: i,
      subject,
      literal: term.value,
      digits,
      value: Number(digits),
      anchor,
      provable: true,
      reason: "",
    };
  });

  return { candidates, subjectVar, valueVar };
}

/** The two ASCII digit-bytes of a canonical digit string — the circuit's private witness. */
export function digitBytes(digits: string): string[] {
  return Array.from(digits, (c) => String(c.charCodeAt(0)));
}
