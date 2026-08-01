// [OPUS-5] sq-ixc3.17 — unit tests for the ZK tool's pure witness-selection logic.
//
// Covers: the EXACT `xsd:integer` acceptance rule (a near-miss datatype must NOT be offered
// as provable, or the witness solve fails at prove time), witness/label column pick, the
// three distinct decline reasons, the skipped-row accounting, and the refused-vs-broken error
// classification.
//
// SCOPE, stated so it is not mistaken for more: nothing in THIS file runs bb.js, and opening
// the tool by hand is not a test. The load-bearing prove/verify path is covered elsewhere —
// see zk-prover-parity.test.ts for what is gated, and what is still only argued.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults } from "@sparq/client";

import {
  declineReason,
  isIntegerLiteral,
  isUnsatisfiable,
  pickLabelVar,
  pickValueVar,
  scanWitnesses,
} from "./zk-witness.js";

const XSD = "http://www.w3.org/2001/XMLSchema#";
const COMMITTED = new Set([24, 25, 30, 42]);

/** Build a SELECT results doc over `?person ?age`, `age` optionally non-integer-typed. */
function results(
  rows: Array<{ person?: string; age?: string; datatype?: string }>,
  vars = ["person", "age"],
): SparqlResults {
  return {
    head: { vars },
    results: {
      bindings: rows.map((r) => ({
        ...(r.person ? { person: { type: "uri" as const, value: r.person } } : {}),
        ...(r.age !== undefined
          ? {
              age: {
                type: "literal" as const,
                value: r.age,
                datatype: r.datatype ?? `${XSD}integer`,
              },
            }
          : {}),
      })),
    },
  } as SparqlResults;
}

// ---------------------------------------------------------------------------
// isIntegerLiteral — deliberately exact
// ---------------------------------------------------------------------------

test("isIntegerLiteral accepts only canonical xsd:integer literals", () => {
  assert.equal(isIntegerLiteral({ type: "literal", value: "30", datatype: `${XSD}integer` }), true);
  assert.equal(isIntegerLiteral({ type: "literal", value: "-7", datatype: `${XSD}integer` }), true);
  // Near-miss datatypes hash to a DIFFERENT canonical token, so they are not witnesses.
  assert.equal(isIntegerLiteral({ type: "literal", value: "30", datatype: `${XSD}int` }), false);
  assert.equal(isIntegerLiteral({ type: "literal", value: "30", datatype: `${XSD}decimal` }), false);
  assert.equal(isIntegerLiteral({ type: "literal", value: "30" }), false);
  assert.equal(isIntegerLiteral({ type: "uri", value: "http://example.org/30" }), false);
  assert.equal(isIntegerLiteral(undefined), false);
  // Right datatype, wrong lexical form.
  assert.equal(isIntegerLiteral({ type: "literal", value: "3.0", datatype: `${XSD}integer` }), false);
});

// ---------------------------------------------------------------------------
// Column picking
// ---------------------------------------------------------------------------

test("pickValueVar finds the first integer column, pickLabelVar the first other one", () => {
  const r = results([{ person: "http://example.org/alice", age: "30" }]);
  assert.equal(pickValueVar(r), "age");
  assert.equal(pickLabelVar(r, "age"), "person");
});

test("pickValueVar returns null when no row binds an integer literal", () => {
  const r = results([{ person: "http://example.org/alice", age: "30", datatype: `${XSD}int` }]);
  assert.equal(pickValueVar(r), null);
  // With no witness column the label falls back to the first bound variable.
  assert.equal(pickLabelVar(r, null), "person");
});

test("pickLabelVar returns null when the SELECT projects only the witness column", () => {
  const r = results([{ age: "30" }], ["age"]);
  assert.equal(pickValueVar(r), "age");
  assert.equal(pickLabelVar(r, "age"), null);
});

// ---------------------------------------------------------------------------
// declineReason — three distinct, non-fabricated reasons
// ---------------------------------------------------------------------------

test("declineReason returns null exactly for committed two-digit values", () => {
  for (const v of COMMITTED) assert.equal(declineReason(v, COMMITTED), null);
});

test("declineReason names the two-digit domain for out-of-domain values", () => {
  assert.match(String(declineReason(9, COMMITTED)), /two-digit domain/);
  assert.match(String(declineReason(100, COMMITTED)), /two-digit domain/);
});

test("declineReason names the native-only term encoder for uncommitted in-domain values", () => {
  const why = String(declineReason(31, COMMITTED));
  assert.match(why, /operand_enc/);
  assert.match(why, /native-only/);
});

test("declineReason rejects a non-integer before anything else", () => {
  assert.match(String(declineReason(Number.NaN, COMMITTED)), /not a finite integer/);
});

// ---------------------------------------------------------------------------
// scanWitnesses
// ---------------------------------------------------------------------------

test("scanWitnesses splits live-store rows into provable and declined, counting skips", () => {
  const scan = scanWitnesses(
    results([
      { person: "http://example.org/alice", age: "30" }, // committed → provable
      { person: "http://example.org/bob", age: "25" }, // committed → provable
      { person: "http://example.org/carol", age: "41" }, // in-domain, uncommitted
      { person: "http://example.org/dan", age: "19" }, // in-domain, uncommitted
      { person: "http://example.org/eve" }, // unbound witness → skipped
    ]),
    COMMITTED,
  );

  assert.equal(scan.valueVar, "age");
  assert.equal(scan.labelVar, "person");
  assert.equal(scan.skipped, 1);
  assert.equal(scan.candidates.length, 4);

  assert.deepEqual(
    scan.candidates.map((c) => c.provable),
    [true, true, false, false],
  );
  // A declined row ALWAYS carries a reason; a provable one never fabricates one.
  for (const c of scan.candidates) {
    assert.equal(c.decline === null, c.provable);
  }
  assert.equal(scan.candidates[0].value, 30);
  assert.equal(scan.candidates[0].lexical, "30");
  assert.equal(scan.candidates[0].label, "http://example.org/alice");
  // `row` indexes the ORIGINAL result rows, so a skipped row does not shift the keys.
  assert.deepEqual(
    scan.candidates.map((c) => c.row),
    [0, 1, 2, 3],
  );
});

test("scanWitnesses labels rows by index when the SELECT projects no label column", () => {
  const scan = scanWitnesses(results([{ age: "30" }], ["age"]), COMMITTED);
  assert.equal(scan.labelVar, null);
  assert.equal(scan.candidates[0].label, "row 1");
});

test("scanWitnesses skips every row when the store has no xsd:integer column", () => {
  const scan = scanWitnesses(
    results([{ person: "http://example.org/alice", age: "30", datatype: `${XSD}int` }]),
    COMMITTED,
  );
  assert.equal(scan.valueVar, null);
  assert.equal(scan.candidates.length, 0);
  assert.equal(scan.skipped, 1);
});

// ---------------------------------------------------------------------------
// isUnsatisfiable — the honesty split between "refused a false claim" and "broken"
// ---------------------------------------------------------------------------

test("isUnsatisfiable recognises the circuit's own verdict constraint", () => {
  // The only assertion a false claim can trip: `filter verdict mismatch` in
  // sparq_zk_compose_core::filter_int, surfaced by noir_js inside its execution error.
  assert.equal(
    isUnsatisfiable("Circuit execution failed: Assertion failed: 'filter verdict mismatch'"),
    true,
  );
  assert.equal(isUnsatisfiable("Error: filter verdict mismatch"), true);
  // Case-insensitively, since the wrapper text around the label is not contractual.
  assert.equal(isUnsatisfiable("ASSERTION FAILED: 'FILTER VERDICT MISMATCH'"), true);
});

test("isUnsatisfiable does not misreport a broken build as a refused claim", () => {
  assert.equal(isUnsatisfiable("ZK circuit fetch failed: 404"), false);
  assert.equal(isUnsatisfiable("WebAssembly.instantiate(): out of memory"), false);
});

test("isUnsatisfiable treats every UNATTRIBUTED execution failure as a prover error", () => {
  // These are the ambiguous middle the panel must not dress up as cryptographic refusal:
  // malformed ACIR, a shifted nightly API, or a witness-encoding bug all surface this way.
  assert.equal(isUnsatisfiable("Circuit execution failed"), false);
  assert.equal(isUnsatisfiable("Circuit execution failed: Assertion failed"), false);
  assert.equal(isUnsatisfiable("Error: constraint not satisfied"), false);
  assert.equal(isUnsatisfiable("the circuit is unsatisfiable"), false);
  assert.equal(isUnsatisfiable("could not deserialize circuit: invalid ACIR format"), false);
  assert.equal(isUnsatisfiable("TypeError: backend.generateProof is not a function"), false);
});

test("isUnsatisfiable does not count the circuit's OTHER assertions as a refused claim", () => {
  // A stale operand_enc fixture, a wrong digit encoding or a bad op code are OUR bugs — the
  // claim was never reached, so reporting them as "the circuit refused it" would be a lie.
  assert.equal(
    isUnsatisfiable("Circuit execution failed: Assertion failed: 'operand encoding mismatch'"),
    false,
  );
  assert.equal(
    isUnsatisfiable("Circuit execution failed: Assertion failed: 'not a decimal digit'"),
    false,
  );
  assert.equal(
    isUnsatisfiable("Circuit execution failed: Assertion failed: 'non-canonical leading zero'"),
    false,
  );
  assert.equal(
    isUnsatisfiable("Circuit execution failed: Assertion failed: 'unknown comparison operator'"),
    false,
  );
});
