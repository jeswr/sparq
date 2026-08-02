// [OPUS-5] sq-ixc3.17 — unit tests for the MPC tool's pure simulation core.
//
// Covers: the additive-sharing invariants (shares reconstruct the secret; every proper subset is
// unconstrained), the end-to-end secure-threshold flow, the live-store adapter (which rows become
// parties and why the rest are dropped), and the field-capacity refusal that keeps the disclosed
// verdict honest on arbitrary user data.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults } from "@sparq/client";

import {
  FIELD_P,
  describeInputProblem,
  partiesFromResults,
  reconstruct,
  runSecureThreshold,
  splitShares,
  type Party,
} from "./mpc-sim.js";

/** A deterministic [0,1) generator so a run is reproducible (a mulberry32 step). */
function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Build a SELECT results doc over `?party ?value`. */
function results(
  rows: Array<{ party?: string; value?: string; partyType?: "uri" | "literal" }>,
  vars: string[] = ["party", "value"],
): SparqlResults {
  return {
    head: { vars },
    results: {
      bindings: rows.map((r) => ({
        ...(r.party === undefined
          ? {}
          : { party: { type: r.partyType ?? "literal", value: r.party } }),
        ...(r.value === undefined ? {} : { value: { type: "literal", value: r.value } }),
      })),
    },
  } as SparqlResults;
}

// ---------------------------------------------------------------------------
// splitShares / reconstruct
// ---------------------------------------------------------------------------

test("splitShares produces n shares that reconstruct the secret", () => {
  const rand = seeded(1);
  for (const secret of [0, 1, 42, 100_000, FIELD_P - 1]) {
    const shares = splitShares(secret, 4, rand);
    assert.equal(shares.length, 4);
    assert.equal(reconstruct(shares), secret);
  }
});

test("splitShares keeps every share inside the field", () => {
  const shares = splitShares(123_456, 5, seeded(7));
  for (const s of shares) {
    assert.ok(Number.isSafeInteger(s), `${s} is not a safe integer`);
    assert.ok(s >= 0 && s < FIELD_P, `${s} escaped [0, ${FIELD_P})`);
  }
});

test("splitShares rejects fewer than two parties", () => {
  assert.throws(() => splitShares(10, 1, seeded(1)), /at least 2 parties/);
});

test("a proper subset of shares does not pin the secret", () => {
  // Dropping one share leaves the remaining sum unconstrained: for ANY target secret there is a
  // closing share in the field. Asserted structurally (the closing share always exists and is a
  // field element), which is the confidentiality property the panel visualises.
  const shares = splitShares(777, 4, seeded(3));
  const partial = reconstruct(shares.slice(0, 3));
  for (const target of [0, 777, FIELD_P - 1]) {
    const closing = (((target - partial) % FIELD_P) + FIELD_P) % FIELD_P;
    assert.ok(closing >= 0 && closing < FIELD_P);
    assert.equal(reconstruct([...shares.slice(0, 3), closing]), target);
  }
});

// ---------------------------------------------------------------------------
// runSecureThreshold
// ---------------------------------------------------------------------------

const PARTIES: Party[] = [
  { name: "Alice", value: 30 },
  { name: "Bob", value: 25 },
  { name: "Carol", value: 41 },
  { name: "Dan", value: 19 },
];

test("runSecureThreshold discloses the correct threshold bit", () => {
  const total = PARTIES.reduce((a, p) => a + p.value, 0); // 115
  const above = runSecureThreshold(PARTIES, 100, seeded(11));
  assert.equal(above.verdict, true);
  assert.equal(above.totalRedacted, total);

  const below = runSecureThreshold(PARTIES, 200, seeded(11));
  assert.equal(below.verdict, false);
  assert.equal(below.totalRedacted, total);
});

test("runSecureThreshold's local sums combine to the total", () => {
  const r = runSecureThreshold(PARTIES, 100, seeded(5));
  assert.equal(r.localSums.length, PARTIES.length);
  assert.equal(reconstruct(r.localSums), r.totalRedacted);
});

test("the share matrix keeps exactly the diagonal and columns are the received view", () => {
  const r = runSecureThreshold(PARTIES, 100, seeded(9));
  const n = PARTIES.length;
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      assert.equal(r.matrix[i][j].kept, i === j, `cell ${i},${j} kept flag`);
    }
    // Row i is party i's own sharing, so it reconstructs party i's value.
    assert.equal(
      reconstruct(r.matrix[i].map((c) => c.value)),
      PARTIES[i].value,
      `row ${i} reconstructs its owner's value`,
    );
    // Column j is what party j holds.
    assert.deepEqual(
      r.received[j],
      r.matrix.map((row) => row[j].value),
      `column ${j} is the received view`,
    );
  }
});

test("runSecureThreshold refuses a field overflow instead of wrapping the verdict", () => {
  const huge: Party[] = [
    { name: "A", value: FIELD_P - 1 },
    { name: "B", value: FIELD_P - 1 },
  ];
  // Σ = 2p - 2, which reduces to p - 2 — below any large threshold, so a wrapped run would
  // silently disclose the WRONG bit. It must refuse instead.
  assert.throws(() => runSecureThreshold(huge, FIELD_P, seeded(2)), /field size/);
});

// ---------------------------------------------------------------------------
// describeInputProblem
// ---------------------------------------------------------------------------

test("describeInputProblem passes a well-formed party list", () => {
  assert.equal(describeInputProblem(PARTIES, 100), null);
});

test("describeInputProblem rejects a single party, a negative threshold and a negative value", () => {
  assert.match(describeInputProblem([PARTIES[0]], 10) ?? "", /at least 2 parties/);
  assert.match(describeInputProblem(PARTIES, -1) ?? "", /non-negative integer/);
  assert.match(
    describeInputProblem([{ name: "X", value: -5 }, PARTIES[0]], 10) ?? "",
    /outside the illustration's field/,
  );
});

// ---------------------------------------------------------------------------
// partiesFromResults — the live-store adapter
// ---------------------------------------------------------------------------

test("partiesFromResults reads ?party / ?value from a live-store SELECT", () => {
  const sel = partiesFromResults(
    results([
      { party: "Alice", value: "30" },
      { party: "Bob", value: "25" },
    ]),
  );
  assert.equal(sel.partyVar, "party");
  assert.equal(sel.valueVar, "value");
  assert.deepEqual(
    sel.parties.map((p) => [p.name, p.value]),
    [
      ["Alice", 30],
      ["Bob", 25],
    ],
  );
  assert.equal(sel.skipped.length, 0);
});

test("partiesFromResults falls back to the first two projected variables", () => {
  const sel = partiesFromResults(
    results([{ party: "Alice", value: "30" }], ["party", "value"]),
  );
  assert.equal(sel.partyVar, "party");
  const positional = partiesFromResults({
    head: { vars: ["who", "amount"] },
    results: {
      bindings: [
        { who: { type: "literal", value: "Alice" }, amount: { type: "literal", value: "30" } },
      ],
    },
  } as SparqlResults);
  assert.equal(positional.partyVar, "who");
  assert.equal(positional.valueVar, "amount");
  assert.deepEqual(positional.parties, [{ name: "Alice", value: 30, source: undefined }]);
});

test("partiesFromResults drops non-integer, negative and unbound rows with a stated reason", () => {
  const sel = partiesFromResults(
    results([
      { party: "Alice", value: "30" },
      { party: "Bob", value: "not-a-number" },
      { party: "Carol", value: "-3" },
      { party: "Dan" },
      { party: "Erin", value: "1.5" },
    ]),
  );
  assert.deepEqual(
    sel.parties.map((p) => p.name),
    ["Alice"],
  );
  assert.deepEqual(
    sel.skipped.map((s) => [s.party, s.reason]),
    [
      ["Bob", '"not-a-number" is not an integer'],
      ["Carol", "-3 is negative"],
      ["Dan", "?value is unbound"],
      ["Erin", '"1.5" is not an integer'],
    ],
  );
});

test("partiesFromResults records a URI party as its own source", () => {
  const sel = partiesFromResults(
    results([{ party: "http://example.org/alice", value: "30", partyType: "uri" }]),
  );
  assert.equal(sel.parties[0].source, "http://example.org/alice");
});

test("partiesFromResults tolerates an empty result", () => {
  const sel = partiesFromResults(results([]));
  assert.deepEqual(sel.parties, []);
  assert.deepEqual(sel.skipped, []);
});
