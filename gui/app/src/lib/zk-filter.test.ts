// [OPUS-5] sq-ixc3.17 — unit tests for the ZK tool's pure core.
//
// Covers: the shipped circuit member's envelope (which live-store literals it can witness), the
// committed term anchors, the verdict the circuit will be asked to assert, and the per-row
// classification that makes an unprovable store value say WHY rather than disappear.
//
// The bb.js half (zk-prover.ts) is deliberately not unit-tested here — it needs the WASM prover;
// its behaviour is exercised in the tab.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults } from "@sparq/client";

import {
  CIRCUIT_DIGITS,
  COMMITTED_TERM_ANCHORS,
  ZK_OPS,
  candidatesFromResults,
  canonicalDigits,
  digitBytes,
  evaluateOp,
  termAnchor,
  type OpCode,
} from "./zk-filter.js";

/** Build a SELECT results doc over `?subject ?value`. */
function results(rows: Array<{ subject?: string; value?: string }>): SparqlResults {
  return {
    head: { vars: ["subject", "value"] },
    results: {
      bindings: rows.map((r) => ({
        ...(r.subject === undefined ? {} : { subject: { type: "uri", value: r.subject } }),
        ...(r.value === undefined ? {} : { value: { type: "literal", value: r.value } }),
      })),
    },
  } as SparqlResults;
}

// ---------------------------------------------------------------------------
// canonicalDigits — the filter_int_d2 envelope
// ---------------------------------------------------------------------------

test("canonicalDigits accepts exactly a canonical two-digit integer", () => {
  assert.equal(canonicalDigits("30"), "30");
  assert.equal(canonicalDigits(" 42 "), "42");
  assert.equal(canonicalDigits("10"), "10");
  assert.equal(canonicalDigits("99"), "99");
});

test("canonicalDigits rejects everything outside the shipped member", () => {
  for (const bad of ["9", "100", "07", "0", "-30", "3.0", "", "abc", "3 0"]) {
    assert.equal(canonicalDigits(bad), null, `${JSON.stringify(bad)} must not be witnessable`);
  }
});

test("the circuit member takes two digits", () => {
  assert.equal(CIRCUIT_DIGITS, 2);
  for (const digits of Object.keys(COMMITTED_TERM_ANCHORS)) {
    assert.equal(canonicalDigits(digits), digits, `anchor key ${digits} must be canonical`);
  }
});

// ---------------------------------------------------------------------------
// Committed term anchors
// ---------------------------------------------------------------------------

test("every committed term anchor is a 32-byte field element in hex", () => {
  const entries = Object.entries(COMMITTED_TERM_ANCHORS);
  assert.ok(entries.length > 0);
  for (const [digits, enc] of entries) {
    assert.match(enc, /^0x[0-9a-f]{64}$/, `anchor for ${digits}`);
    assert.equal(termAnchor(digits), enc);
  }
});

test("termAnchor is undefined for a value with no committed anchor", () => {
  assert.equal(termAnchor("41"), undefined);
  assert.equal(termAnchor("19"), undefined);
});

// ---------------------------------------------------------------------------
// evaluateOp — the verdict the circuit asserts
// ---------------------------------------------------------------------------

test("evaluateOp matches the circuit's OP_* semantics", () => {
  const cases: Array<[OpCode, number, number, boolean]> = [
    [0, 30, 25, false], // <
    [0, 20, 25, true],
    [1, 25, 25, true], // ≤
    [1, 26, 25, false],
    [2, 30, 25, true], // >
    [2, 25, 25, false],
    [3, 25, 25, true], // ≥
    [3, 24, 25, false],
    [4, 25, 25, true], // =
    [4, 24, 25, false],
    [5, 24, 25, true], // ≠
    [5, 25, 25, false],
  ];
  for (const [op, value, bound, expected] of cases) {
    assert.equal(evaluateOp(value, op, bound), expected, `op ${op}: ${value} vs ${bound}`);
  }
});

test("ZK_OPS covers every circuit operator exactly once, in code order", () => {
  assert.deepEqual(
    ZK_OPS.map((o) => o.code),
    [0, 1, 2, 3, 4, 5],
  );
  assert.deepEqual(
    ZK_OPS.map((o) => o.circuitName),
    ["OP_LT", "OP_LE", "OP_GT", "OP_GE", "OP_EQ", "OP_NE"],
  );
});

// ---------------------------------------------------------------------------
// digitBytes — the private witness
// ---------------------------------------------------------------------------

test("digitBytes emits the ASCII code of each digit", () => {
  assert.deepEqual(digitBytes("30"), ["51", "48"]);
  assert.deepEqual(digitBytes("42"), ["52", "50"]);
});

// ---------------------------------------------------------------------------
// candidatesFromResults — classification over the live store
// ---------------------------------------------------------------------------

test("candidatesFromResults marks anchored two-digit values provable", () => {
  const { candidates } = candidatesFromResults(
    results([{ subject: "http://example.org/alice", value: "30" }]),
  );
  assert.equal(candidates.length, 1);
  const c = candidates[0];
  assert.equal(c.provable, true);
  assert.equal(c.digits, "30");
  assert.equal(c.value, 30);
  assert.equal(c.anchor, COMMITTED_TERM_ANCHORS["30"]);
  assert.equal(c.reason, "");
  assert.equal(c.subject, "http://example.org/alice");
});

test("candidatesFromResults keeps unprovable rows and says why", () => {
  const { candidates } = candidatesFromResults(
    results([
      { subject: "ex:carol", value: "41" }, // in envelope, no committed anchor
      { subject: "ex:dan", value: "9" }, // outside the two-digit envelope
      { subject: "ex:erin" }, // unbound
    ]),
  );
  assert.equal(candidates.length, 3);
  assert.equal(candidates.every((c) => !c.provable), true);
  assert.match(candidates[0].reason, /no committed term anchor/);
  assert.equal(candidates[0].digits, "41");
  assert.match(candidates[1].reason, /exactly 2 digits/);
  assert.equal(candidates[1].digits, null);
  assert.match(candidates[2].reason, /\?value is unbound/);
});

test("candidatesFromResults never filters a row away", () => {
  const rows = [
    { subject: "ex:a", value: "30" },
    { subject: "ex:b", value: "41" },
    { subject: "ex:c", value: "25" },
    { subject: "ex:d", value: "19" },
  ];
  const { candidates } = candidatesFromResults(results(rows));
  assert.equal(candidates.length, rows.length);
  assert.deepEqual(
    candidates.map((c) => c.provable),
    [true, false, true, false],
  );
  assert.deepEqual(
    candidates.map((c) => c.row),
    [0, 1, 2, 3],
  );
});

test("candidatesFromResults falls back to the first two projected variables", () => {
  const sel = candidatesFromResults({
    head: { vars: ["who", "amount"] },
    results: {
      bindings: [
        { who: { type: "uri", value: "ex:a" }, amount: { type: "literal", value: "42" } },
      ],
    },
  } as SparqlResults);
  assert.equal(sel.subjectVar, "who");
  assert.equal(sel.valueVar, "amount");
  assert.equal(sel.candidates[0].provable, true);
});

test("candidatesFromResults tolerates an empty result", () => {
  assert.deepEqual(candidatesFromResults(results([])).candidates, []);
});
