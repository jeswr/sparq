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
// SCOPE OF THE CLAIM: the proof binds the hidden digits to ONE committed RDF term and to the
// comparison verdict — NOT to the row, subject, store or credential the operand was read from
// (no such commitment is among the public inputs). The SELECT supplies a local witness; it does
// not make the proof a statement about the live store. See `buildCircuitInputs` below.
//
// HONESTY: the in-tab proving mechanics are real, but the sparq ZK estate is research-grade —
// the v1 verifier is internally re-audited only and external accredited-cryptographer sign-off
// is pending (bead sq-qhy4). A proof produced here is NOT a production cryptographic guarantee.
// See SECURITY.md and compliance/cryptoreview/README.md.
//
// This module is PURE (no bb.js, no React, no `@/` aliases) so `npm run test:unit` can exercise
// it under `node --test`; the wasm-loading half lives in `zk-prover.ts`.

import type { SparqlResults, SparqlTerm } from "@sparq/client";

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
 * The ONE datatype the shipped circuit is bound to. `filter_int` rebuilds the token
 * `"<digits>"^^<http://www.w3.org/2001/XMLSchema#integer>` byte-for-byte before hashing it, so a
 * literal with any other datatype — or a language tag, or a non-literal term — is a DIFFERENT RDF
 * term with a different encoding, however its lexical form reads.
 */
export const XSD_INTEGER = "http://www.w3.org/2001/XMLSchema#integer";

/**
 * Field encodings of the RDF term `"<value>"^^xsd:integer`, precomputed natively by
 * `sparq_zk_compose::build::encode_int_literal` (the SAME encoder the scan proof uses for the
 * operand column). Each is the `operand_enc` public input: the circuit rebuilds that exact
 * N-Triples token from the private digits, hashes it with the blake3 blackbox, and asserts the
 * encoding matches.
 *
 * WHAT AN ANCHOR BINDS: the hidden digits to ONE RDF term, and nothing else. It does NOT bind
 * them to a credential, a dataset, a query result, or the row the SELECT returned — tying an
 * `operand_enc` to a committed triple slot is the SCAN proof's job
 * (`sparq_zk_compose_core::scan`), which this tool does not ship. See {@link buildCircuitInputs}.
 *
 * DISCLOSURE: this table is published in this build and `operand_enc` is a PUBLIC input, so a
 * verifier holding the table can map a proof's anchor straight back to the value it was made
 * about. The value is not itself a public input, but across four committed anchors it is not
 * hidden from anyone who reads this file.
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
 * The canonical decimal digit string the circuit witnesses, or `null` when this LEXICAL FORM is
 * outside the shipped member's envelope. `filter_int` requires ASCII decimal digits, no leading
 * zero, and exactly {@link CIRCUIT_DIGITS} of them.
 *
 * The lexical form must be canonical EXACTLY: the circuit hashes the rebuilt token byte-for-byte,
 * so `" 42 "` — a legal but non-canonical `xsd:integer` lexical form under the XSD whitespace-
 * collapse facet — encodes to a different term than `"42"` and must not be matched to the `"42"`
 * anchor. Hence no trimming here.
 *
 * This checks the lexical form ONLY; {@link termRejection} checks the term kind and datatype that
 * make that lexical form an `xsd:integer` in the first place. Both must pass.
 */
export function canonicalDigits(lexical: string): string | null {
  if (!/^[1-9][0-9]*$/.test(lexical)) return null;
  if (lexical.length !== CIRCUIT_DIGITS) return null;
  return lexical;
}

/**
 * Why this RDF term cannot be the circuit's operand, or `null` when its kind and datatype are
 * exactly what {@link XSD_INTEGER} requires.
 *
 * Checking the lexical form alone is NOT enough: `"30"`, `"30"^^xsd:string`, `"30"@en` and
 * `<30>` all read as `30` in a SPARQL-JSON `value` field but are four distinct RDF terms with
 * four distinct encodings, only one of which the committed anchor for `30` is the encoding of.
 * Classifying any of the others as provable would make the panel's claim about a different term
 * than the one the live store returned.
 */
export function termRejection(term: SparqlTerm): string | null {
  if (term.type !== "literal") {
    return `the operand must be a literal; this row bound a ${term.type} term`;
  }
  const lang = term["xml:lang"];
  if (lang) {
    return `a language-tagged literal (…@${lang}) is a different RDF term from the "…"^^xsd:integer the circuit rebuilds`;
  }
  if (term.datatype !== XSD_INTEGER) {
    const found = term.datatype ?? "a plain literal (xsd:string)";
    return `the operand must be an xsd:integer literal; this row bound ${found}`;
  }
  return null;
}

/** The committed `operand_enc` anchor for a canonical digit string, if one shipped. */
export function termAnchor(digits: string): string | undefined {
  return COMMITTED_TERM_ANCHORS[digits];
}

/** One row of the live-store SELECT, classified for provability. */
export interface ZkCandidate {
  /** 0-based row index in the SELECT result. */
  row: number;
  /**
   * The `?subject` term (or "" when unbound). LOCAL provenance for the operator's own eye only —
   * no proof this tool produces attests it (see {@link buildCircuitInputs}).
   */
  subject: string;
  /**
   * The bound term's `value` field exactly as the store returned it — a literal's lexical form,
   * or a URI term's IRI. Carries no datatype or language tag; {@link reason} names those when
   * they are why the row cannot be proven.
   */
  literal: string;
  /** The canonical digit string, when the term is inside the member's envelope. */
  digits: string | null;
  /** The numeric value, when this really is an in-envelope integer term. Never a public input. */
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
    const rejection = termRejection(term);
    if (rejection) {
      return {
        row: i,
        subject,
        literal: term.value,
        digits: null,
        value: null,
        provable: false,
        reason: rejection,
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

// ---------------------------------------------------------------------------
// The circuit's input contract
// ---------------------------------------------------------------------------

/**
 * The circuit's PUBLIC inputs, in `zk/compose/filter_int_d2/src/main.nr` declaration order. Every
 * other key of {@link CircuitInputs} is private witness.
 */
export const PUBLIC_INPUT_KEYS = ["challenge", "operand_enc", "op", "bound", "expected"] as const;

/** The one private-witness key of {@link CircuitInputs}. */
export const PRIVATE_INPUT_KEY = "digits";

/**
 * The Noir input map for `filter_int_d2`. A type ALIAS rather than an interface so it keeps the
 * implicit index signature `Noir.execute(Record<string, unknown>)` needs.
 */
export type CircuitInputs = {
  /** Per-presentation verifier nonce. */
  challenge: string;
  /** The committed term anchor — see {@link COMMITTED_TERM_ANCHORS}. */
  operand_enc: string;
  op: string;
  bound: string;
  /** The disclosed verdict, DERIVED from the hidden value (never taken from the caller). */
  expected: boolean;
  /** PRIVATE witness — the value's ASCII digit bytes. */
  digits: string[];
};

export interface ProofRequest {
  /** The canonical digit string of the hidden store value (the private witness). */
  digits: string;
  op: OpCode;
  /** The FILTER's public constant. */
  bound: number;
}

/**
 * Assemble the circuit's inputs for a request. Split out of `zk-prover.ts`'s `proveFilter` so the
 * load-bearing encoding decisions — which keys exist, which are public, and that the disclosed
 * verdict is DERIVED rather than caller-supplied — are pinned by `npm run test:unit` without the
 * WASM prover.
 *
 * WHAT THE RESULTING PROOF ESTABLISHES, exactly: that the prover knew {@link CIRCUIT_DIGITS}
 * canonical decimal digits whose `"<digits>"^^xsd:integer` term encoding equals `operand_enc`,
 * and that those digits satisfy `value <op> bound` with verdict `expected`. Nothing more.
 *
 * WHAT IT DOES NOT ESTABLISH: there is no dataset commitment, query-result commitment, subject
 * identifier or credential root among the public inputs, so the proof does NOT attest that the
 * value came from the selected SPARQL row, from the workspace store, or from any issued
 * credential. The SELECT only supplies a LOCAL witness and picks which published anchor to claim
 * about; binding an `operand_enc` to a committed triple slot is the scan proof's job
 * (`sparq_zk_compose_core::scan`), which this tool does not ship.
 *
 * Throws when no committed anchor ships for the value, or when the bound is not a `u64`.
 */
export function buildCircuitInputs({ digits, op, bound }: ProofRequest): CircuitInputs {
  const operandEnc = termAnchor(digits);
  if (!operandEnc) {
    throw new Error(`no committed term anchor ships for the value ${digits}`);
  }
  if (!Number.isSafeInteger(bound) || bound < 0) {
    throw new Error("the bound must be a non-negative integer (the circuit takes a u64)");
  }
  return {
    challenge: "0x1", // fixed here; the in-tab panel has no replay context to bind
    operand_enc: operandEnc,
    op: String(op),
    bound: String(bound),
    // The circuit asserts `result == expected`, so we publish the verdict the hidden value
    // actually satisfies. Claiming the other one makes the witness solve fail — that is the
    // property the panel demonstrates, not a bug to work around.
    expected: evaluateOp(Number(digits), op, bound),
    // PRIVATE witness — not a public input (the published anchor still identifies it; see
    // COMMITTED_TERM_ANCHORS).
    digits: digitBytes(digits),
  };
}
