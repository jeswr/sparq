// [OPUS-5] sq-ixc3.17 — unit tests for the ZK tool's pure core.
//
// Covers: the shipped circuit member's envelope (which live-store TERMS it can witness — kind,
// datatype and lexical form, not just the lexical form), the committed term anchors, the verdict
// the circuit will be asked to assert, the per-row classification that makes an unprovable store
// value say WHY rather than disappear, and the circuit's input contract (which keys are public,
// and that the disclosed verdict is derived rather than caller-supplied).
//
// The WASM half — a real proof generated and verified through `proveFilter` — is exercised by
// the sibling `zk-prover.integration.test.ts`, an OPT-IN target (`npm run test:zk-proof`) because
// it instantiates the bb.js prover. This file stays WASM-free so it can run in the build-free
// `test:unit` lane.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults, SparqlTerm } from "@sparq/client";

import {
  CIRCUIT_DIGITS,
  COMMITTED_TERM_ANCHORS,
  FIXED_CHALLENGE,
  PRIVATE_INPUT_KEY,
  PUBLIC_INPUT_KEYS,
  XSD_INTEGER,
  ZK_OPS,
  buildCircuitInputs,
  candidatesFromResults,
  canonicalDigits,
  digitBytes,
  evaluateOp,
  termAnchor,
  termRejection,
  type OpCode,
} from "./zk-filter.js";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

interface Row {
  subject?: string;
  value?: string;
  /** Defaults to `xsd:integer`; pass `null` for a plain literal (i.e. `xsd:string`). */
  datatype?: string | null;
  lang?: string;
  type?: SparqlTerm["type"];
}

/**
 * Build a SELECT results doc over `?subject ?value`. `value` defaults to the SHAPE the live store
 * actually returns for a Turtle integer — a literal tagged `^^xsd:integer` (see
 * `crates/sparq-wasm`'s SPARQL-JSON serialisation) — because that datatype is precisely what the
 * circuit's in-circuit token rebuild binds. Pass `datatype`/`lang`/`type` to build one of the
 * look-alike terms whose lexical form reads the same but which must NOT be provable.
 */
function results(rows: Row[]): SparqlResults {
  return {
    head: { vars: ["subject", "value"] },
    results: {
      bindings: rows.map((r) => ({
        ...(r.subject === undefined ? {} : { subject: { type: "uri", value: r.subject } }),
        ...(r.value === undefined ? {} : { value: term(r) }),
      })),
    },
  } as SparqlResults;
}

function term(r: Row): SparqlTerm {
  const type = r.type ?? "literal";
  if (type !== "literal") return { type, value: r.value as string };
  if (r.lang) return { type, value: r.value as string, "xml:lang": r.lang };
  const datatype = r.datatype === undefined ? XSD_INTEGER : r.datatype;
  return datatype === null
    ? { type, value: r.value as string }
    : { type, value: r.value as string, datatype };
}

// ---------------------------------------------------------------------------
// canonicalDigits — the filter_int_d2 envelope
// ---------------------------------------------------------------------------

test("canonicalDigits accepts exactly a canonical two-digit integer", () => {
  assert.equal(canonicalDigits("30"), "30");
  assert.equal(canonicalDigits("42"), "42");
  assert.equal(canonicalDigits("10"), "10");
  assert.equal(canonicalDigits("99"), "99");
});

test("canonicalDigits rejects everything outside the shipped member", () => {
  for (const bad of ["9", "100", "07", "0", "-30", "3.0", "", "abc", "3 0"]) {
    assert.equal(canonicalDigits(bad), null, `${JSON.stringify(bad)} must not be witnessable`);
  }
});

test("canonicalDigits rejects a legal but NON-canonical lexical form", () => {
  // `" 42 "^^xsd:integer` is a valid xsd:integer under the whitespace-collapse facet, but the
  // circuit hashes the rebuilt token byte-for-byte, so it encodes to a different term than "42"
  // and must not be matched to the "42" anchor.
  for (const bad of [" 42 ", " 42", "42 ", "\t42", "42\n"]) {
    assert.equal(canonicalDigits(bad), null, `${JSON.stringify(bad)} is not the canonical token`);
  }
});

test("the circuit member takes two digits", () => {
  assert.equal(CIRCUIT_DIGITS, 2);
  for (const digits of Object.keys(COMMITTED_TERM_ANCHORS)) {
    assert.equal(canonicalDigits(digits), digits, `anchor key ${digits} must be canonical`);
  }
});

// ---------------------------------------------------------------------------
// termRejection — the term KIND and DATATYPE the circuit is bound to
// ---------------------------------------------------------------------------

test("termRejection accepts only an xsd:integer literal", () => {
  assert.equal(termRejection({ type: "literal", value: "30", datatype: XSD_INTEGER }), null);
});

test("termRejection rejects the look-alike terms sharing a lexical form", () => {
  // All four read as `30` in a SPARQL-JSON `value` field; none is the term whose encoding the
  // committed anchor for 30 is, so none may be proven about with that anchor.
  const cases: Array<[SparqlTerm, RegExp]> = [
    [{ type: "literal", value: "30" }, /plain literal \(xsd:string\)/],
    [{ type: "literal", value: "30", datatype: XSD_STRING }, /xsd:integer/],
    [{ type: "literal", value: "30", datatype: XSD_INTEGER, "xml:lang": "en" }, /language-tagged/],
    [{ type: "uri", value: "30" }, /must be a literal/],
    [{ type: "bnode", value: "30" }, /must be a literal/],
    [
      { type: "literal", value: "30", datatype: "http://www.w3.org/2001/XMLSchema#decimal" },
      /xsd:integer/,
    ],
  ];
  for (const [t, expected] of cases) {
    const reason = termRejection(t);
    assert.ok(reason, `${JSON.stringify(t)} must be rejected`);
    assert.match(reason, expected);
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

test("candidatesFromResults will NOT prove about a term of the wrong kind or datatype", () => {
  // The regression the reviewer caught: classifying on `term.value` alone made every one of these
  // "provable" against the xsd:integer anchor for 30 — a claim about a different RDF term than
  // the row the live store returned.
  const rows: Row[] = [
    { subject: "ex:plain", value: "30", datatype: null }, // "30"          (xsd:string)
    { subject: "ex:string", value: "30", datatype: XSD_STRING }, // "30"^^xsd:string
    { subject: "ex:lang", value: "30", lang: "en" }, // "30"@en
    { subject: "ex:uri", value: "30", type: "uri" }, // <30>
  ];
  const { candidates } = candidatesFromResults(results(rows));
  assert.equal(candidates.length, rows.length);
  for (const c of candidates) {
    assert.equal(c.provable, false, `${c.subject} must not be provable`);
    assert.equal(c.digits, null);
    assert.equal(c.anchor, undefined);
    assert.notEqual(c.reason, "");
  }
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
        {
          who: { type: "uri", value: "ex:a" },
          amount: { type: "literal", value: "42", datatype: XSD_INTEGER },
        },
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

// ---------------------------------------------------------------------------
// buildCircuitInputs — the circuit's input contract
// ---------------------------------------------------------------------------

test("buildCircuitInputs emits exactly the circuit's public inputs plus the private digits", () => {
  const inputs = buildCircuitInputs({ digits: "30", op: 3, bound: 25 });
  assert.deepEqual(
    Object.keys(inputs).sort(),
    [...PUBLIC_INPUT_KEYS, PRIVATE_INPUT_KEY].sort(),
    "an unexpected key here is either a missing public input or an undisclosed disclosure",
  );
  assert.equal(inputs.operand_enc, COMMITTED_TERM_ANCHORS["30"]);
  assert.equal(inputs.op, "3");
  assert.equal(inputs.bound, "25");
  assert.deepEqual(inputs.digits, digitBytes("30"));
});

test("buildCircuitInputs DERIVES the disclosed verdict from the hidden value", () => {
  // The caller cannot publish a verdict of its own choosing: `expected` is computed here, which is
  // why the panel can say a false claim is unsatisfiable rather than merely unattempted.
  assert.equal(buildCircuitInputs({ digits: "30", op: 3, bound: 25 }).expected, true); // 30 ≥ 25
  assert.equal(buildCircuitInputs({ digits: "24", op: 3, bound: 25 }).expected, false); // 24 ≥ 25
  for (const [op, value, bound] of [
    [0, 24, 25],
    [4, 42, 42],
    [5, 30, 25],
  ] as Array<[OpCode, number, number]>) {
    assert.equal(
      buildCircuitInputs({ digits: String(value), op, bound }).expected,
      evaluateOp(value, op, bound),
    );
  }
});

test("the hidden value reaches the circuit only through the private digits", () => {
  // Two different values under the SAME public claim: every public input except the opaque
  // encoding must be identical, or the value is leaking into the claim itself.
  const a = buildCircuitInputs({ digits: "24", op: 0, bound: 99 });
  const b = buildCircuitInputs({ digits: "25", op: 0, bound: 99 });
  assert.equal(a.challenge, b.challenge);
  assert.equal(a.op, b.op);
  assert.equal(a.bound, b.bound);
  assert.equal(a.expected, b.expected); // both < 99
  assert.notEqual(a.operand_enc, b.operand_enc);
  assert.notDeepEqual(a.digits, b.digits);
});

test("challenge is a fixed constant, NOT a per-presentation nonce", () => {
  // The panel and CircuitInputs both state that `challenge` carries no freshness/replay binding.
  // Pinned so the docs cannot quietly start calling it a verifier nonce again: distinct
  // presentations — different value, operator and bound — must all publish the SAME constant.
  const inputs = [
    buildCircuitInputs({ digits: "24", op: 0, bound: 99 }),
    buildCircuitInputs({ digits: "30", op: 2, bound: 18 }),
    buildCircuitInputs({ digits: "42", op: 5, bound: 42 }),
  ];
  for (const i of inputs) {
    assert.equal(i.challenge, FIXED_CHALLENGE);
  }
  // The literal value, not just the constant: zk-prover.integration.test.ts tells the challenge
  // and the disclosed verdict apart by COUNTING public inputs equal to the field element 1, which
  // only works while the challenge is 0x1. Changing it must break here, in the cheap lane.
  assert.equal(BigInt(FIXED_CHALLENGE), 1n);
});

test("operand_enc identifies the value to anyone holding the published anchor table", () => {
  // NOT a leak the circuit can fix — the honest limit of shipping four committed anchors in the
  // client, documented on COMMITTED_TERM_ANCHORS and in the panel's public-inputs caption. Pinned
  // so the docs cannot quietly start claiming the value is hidden from the verifier.
  const inputs = buildCircuitInputs({ digits: "42", op: 2, bound: 25 });
  const recovered = Object.entries(COMMITTED_TERM_ANCHORS).find(
    ([, enc]) => enc === inputs.operand_enc,
  )?.[0];
  assert.equal(recovered, "42");
});

test("buildCircuitInputs refuses a value with no committed anchor", () => {
  assert.throws(
    () => buildCircuitInputs({ digits: "41", op: 3, bound: 25 }),
    /no committed term anchor/,
  );
});

test("buildCircuitInputs refuses a bound the circuit's u64 cannot take", () => {
  for (const bad of [-1, 1.5, Number.NaN, Number.MAX_SAFE_INTEGER + 2]) {
    assert.throws(
      () => buildCircuitInputs({ digits: "30", op: 3, bound: bad }),
      /non-negative integer/,
      `bound ${bad} must be refused`,
    );
  }
});
