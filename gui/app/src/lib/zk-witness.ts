// [OPUS-5] sq-ixc3.17 — pure view/selection logic for the ZK tool (no React, no DOM, no
// bb.js), unit-tested with the gui/app node test runner. The panel
// (components/workbench/zk-tool.tsx) is a thin renderer over these.
//
// The ZK tool's job is to turn a SELECT over the LIVE workspace store into a list of
// candidate PRIVATE WITNESSES — integer literals the operator could prove a FILTER over
// without disclosing the value. This module answers, per row, the only question that
// matters honestly: *can this value actually be proven in your tab today, and if not, why
// not?* It NEVER guesses; a row that cannot be proven carries the exact reason.
//
// The ceiling is real and is stated, not hidden. The in-tab prover ships ONE compiled
// circuit member, `filter_int_d2` (see lib/zk-prover.ts), and its `operand_enc` public
// input — the blake3 term commitment binding the hidden value to the committed graph — is
// produced by the NATIVE encoder (`sparq_zk_compose::build::encode_int_literal`), which is
// not in the GUI's wasm bundle. So only values with a COMMITTED fixture can be proven
// in-tab; everything else is declined with that reason rather than silently dropped.

import type { SparqlResults, SparqlTerm } from "@sparq/client";

const XSD_INTEGER = "http://www.w3.org/2001/XMLSchema#integer";

/**
 * Is this term a canonical `xsd:integer` literal?
 *
 * Deliberately EXACT: the circuit rebuilds the canonical `"<digits>"^^xsd:integer`
 * N-Triples token in-circuit and hashes it, so a value typed `xsd:int` / `xsd:long` /
 * `xsd:nonNegativeInteger` would hash to a DIFFERENT token and would not match any
 * committed `operand_enc`. Accepting those here would produce rows that look provable and
 * then fail the witness solve — so they are not accepted.
 */
export function isIntegerLiteral(term: SparqlTerm | undefined): term is SparqlTerm {
  if (!term || term.type !== "literal" || term.datatype !== XSD_INTEGER) return false;
  return /^-?[0-9]+$/.test(term.value);
}

/** One candidate row: a store value the operator may select as the private witness. */
export interface WitnessCandidate {
  /** Index into the SELECT's kept rows — the stable selection key. */
  row: number;
  /** The label variable's binding for this row (display only; never a proof input). */
  label: string;
  /** The literal's lexical form exactly as the store holds it. */
  lexical: string;
  /** The parsed value — the PRIVATE witness a proof would hide. */
  value: number;
  /** True iff an in-tab proof over this value can be produced today. */
  provable: boolean;
  /** Why an in-tab proof is impossible for this row, or `null` when it is possible. */
  decline: string | null;
}

/** The result of scanning a SELECT for provable witnesses. */
export interface WitnessScan {
  /** The variable used as the row label, or `null` when the SELECT projects only one var. */
  labelVar: string | null;
  /** The variable holding the integer witness, or `null` when no row bound one. */
  valueVar: string | null;
  candidates: WitnessCandidate[];
  /** Rows dropped because the witness variable was unbound or not an `xsd:integer`. */
  skipped: number;
}

/**
 * The first projected variable that binds an `xsd:integer` literal in at least one row —
 * the witness column. Returns `null` when the result has no integer column at all (the
 * panel then says so instead of rendering an empty table with no explanation).
 */
export function pickValueVar(results: SparqlResults): string | null {
  for (const v of results.head.vars) {
    if (results.results.bindings.some((b) => isIntegerLiteral(b[v]))) return v;
  }
  return null;
}

/** The first projected variable other than the witness column that binds anything — the
 *  row label. `null` when the SELECT projects the witness column only. */
export function pickLabelVar(results: SparqlResults, valueVar: string | null): string | null {
  for (const v of results.head.vars) {
    if (v === valueVar) continue;
    if (results.results.bindings.some((b) => b[v] !== undefined)) return v;
  }
  return null;
}

/**
 * Why `value` cannot be proven in-tab, or `null` when it can.
 *
 * Three distinct honest reasons, in narrowing order — the panel shows them verbatim so an
 * operator can tell "the shipped circuit does not cover this shape" apart from "this value
 * has no committed term commitment yet".
 */
export function declineReason(value: number, committed: ReadonlySet<number>): string | null {
  if (!Number.isSafeInteger(value)) {
    return "not a finite integer — the filter_int circuit member takes an integer witness.";
  }
  if (value < 10 || value > 99) {
    return "outside the shipped filter_int_d2 member's two-digit domain (10–99); a wider member is compiled natively but is not in this build.";
  }
  if (!committed.has(value)) {
    return "no committed operand_enc term commitment for this value — the blake3 term encoder is native-only (not in the in-tab wasm bundle), so an arbitrary store value cannot yet be bound to the circuit in your tab.";
  }
  return null;
}

/**
 * Scan a SELECT result for candidate private witnesses.
 *
 * `committed` is the set of values with a shipped `operand_enc` fixture (lib/zk-prover.ts's
 * {@link COMMITTED_VALUES}). Rows whose witness variable is unbound or not an `xsd:integer`
 * are COUNTED as `skipped`, not silently discarded — the panel reports the count so the
 * operator knows the table is a subset of the query's rows.
 */
export function scanWitnesses(
  results: SparqlResults,
  committed: ReadonlySet<number>,
): WitnessScan {
  const valueVar = pickValueVar(results);
  const labelVar = pickLabelVar(results, valueVar);
  const candidates: WitnessCandidate[] = [];
  let skipped = 0;

  results.results.bindings.forEach((binding, row) => {
    const term = valueVar ? binding[valueVar] : undefined;
    if (!isIntegerLiteral(term)) {
      skipped += 1;
      return;
    }
    const value = Number.parseInt(term.value, 10);
    const decline = declineReason(value, committed);
    candidates.push({
      row,
      label: (labelVar ? binding[labelVar]?.value : undefined) ?? `row ${row + 1}`,
      lexical: term.value,
      value,
      provable: decline === null,
      decline,
    });
  });

  return { labelVar, valueVar, candidates, skipped };
}

/**
 * The label the shipped circuit gives the ONE constraint a false verdict claim trips:
 * `assert_eq(result, expected, "filter verdict mismatch")` in
 * `sparq_zk_compose_core::filter_int::filter_int_check` (zk/compose/compose_core/src/filter_int.nr),
 * pinned circuit-side by `filter_int_rejects_lying_verdict` in
 * zk/compose/compose_core/src/tests.nr (`#[test(should_fail_with = "filter verdict mismatch")]`).
 * Recognising it depends on noir_js surfacing the failing assertion's label in the thrown
 * error's message — see the fail-safe direction noted on {@link isUnsatisfiable}.
 *
 * Every OTHER assertion in that circuit member — `operand encoding mismatch`, `not a decimal
 * digit`, `non-canonical leading zero`, `unknown comparison operator` — indicates a broken
 * fixture or a bad input encoding on OUR side, not a refused claim, so none of them are
 * listed here.
 */
const VERDICT_CONSTRAINT = "filter verdict mismatch";

/**
 * Does a prover error mean the circuit REFUSED THE CLAIM — i.e. the verdict constraint was
 * unsatisfiable — rather than something breaking?
 *
 * The distinction is load-bearing for honesty: a refused claim is the SUCCESS case of the
 * tool's "try to forge the opposite verdict" affordance, and the panel reports it as the
 * constraint system declining to certify a verdict the hidden value does not satisfy. That
 * is cryptographic evidence, so it must be earned by the SPECIFIC constraint, not inferred
 * from the shape of an error.
 *
 * Hence the deliberately narrow rule: only the circuit's own {@link VERDICT_CONSTRAINT}
 * label counts. A bare `execution failed`, a generic `Assertion failed`, an `unsatisfiable`
 * with no attribution, or any `constraint` wording is treated as a PROVER ERROR and shown
 * with its raw message — those also arise from malformed ACIR, a bb.js / noir_js nightly
 * whose API has shifted, a witness-encoding bug, or a stale `operand_enc` fixture, and
 * presenting any of those as "the real circuit refused a false claim" would dress a broken
 * build up as soundness evidence.
 *
 * The failure direction is chosen on purpose. If a prover bump changes how the assertion
 * label is surfaced, a genuine refusal degrades to an honest "proving failed" with the raw
 * message — never the reverse. Pinned against `@aztec/bb.js` 5.0.0-nightly.20260324 +
 * `@noir-lang/noir_js` 1.0.0-beta.21 (gui/app/package.json); re-check on any bump.
 */
export function isUnsatisfiable(message: string): boolean {
  return message.toLowerCase().includes(VERDICT_CONSTRAINT);
}

/**
 * The default witness query the tool opens with — a plain SELECT over whatever the operator
 * already imported. It is NOT a fixture the tool secretly proves against: the rows it
 * returns come from the live store, and if the store holds no `foaf:age` the table is
 * honestly empty until the operator edits the query.
 */
export const DEFAULT_WITNESS_QUERY = `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
# Pick the private witness out of YOUR live store. Any SELECT works: the first
# projected xsd:integer column is the hidden value, the first other column labels the row.
SELECT ?person ?age WHERE {
  ?person foaf:age ?age .
}
ORDER BY ?person`;
