// [OPUS-5] sq-ixc3.17 — unit tests for the MPC tool's pure protocol + live-store logic.
//
// Covers: the additive-sharing invariants the panel's claims rest on (shares sum back to the
// secret; every share but the last is independent of it), the secure-threshold flow (local
// sums combine to the total, only the verdict bit is derived), and the SELECT → parties scan
// (which column is the contribution, and every unusable row surfaced with a reason).
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import type { SparqlResults } from "@sparq/client";

import {
  DEFAULT_THRESHOLD,
  FIELD_P,
  MAX_TOTAL,
  partiesFromResults,
  reconstruct,
  runSecureThreshold,
  splitShares,
  type Party,
} from "./mpc-sim.js";

const XSD = "http://www.w3.org/2001/XMLSchema#";

/** A deterministic stand-in for Math.random so a run is reproducible in tests. */
function seeded(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    // xorshift32 → [0, 1)
    s ^= s << 13;
    s >>>= 0;
    s ^= s >>> 17;
    s ^= s << 5;
    s >>>= 0;
    return s / 4_294_967_296;
  };
}

/** Build a SELECT results doc over `?party ?contribution`. */
function results(
  rows: Array<{ party?: string; contribution?: string; datatype?: string }>,
  vars = ["party", "contribution"],
): SparqlResults {
  return {
    head: { vars },
    results: {
      bindings: rows.map((r) => ({
        ...(r.party ? { party: { type: "literal" as const, value: r.party } } : {}),
        ...(r.contribution !== undefined
          ? {
              contribution: {
                type: "literal" as const,
                value: r.contribution,
                datatype: r.datatype ?? `${XSD}integer`,
              },
            }
          : {}),
      })),
    },
  } as SparqlResults;
}

// ---------------------------------------------------------------------------
// Additive sharing
// ---------------------------------------------------------------------------

test("splitShares produces n shares that reconstruct the secret", () => {
  const rand = seeded(1);
  for (const secret of [0, 1, 30_000, FIELD_P - 1]) {
    for (const n of [2, 3, 5]) {
      const shares = splitShares(secret, n, rand);
      assert.equal(shares.length, n);
      assert.equal(reconstruct(shares), secret % FIELD_P);
    }
  }
});

test("splitShares keeps every share inside the field", () => {
  const shares = splitShares(42, 4, seeded(7));
  for (const s of shares) {
    assert.ok(Number.isInteger(s) && s >= 0 && s < FIELD_P, `share out of field: ${s}`);
  }
});

test("splitShares rejects a single-party run", () => {
  assert.throws(() => splitShares(1, 1, seeded(3)), /at least 2 parties/);
});

test("all but the last share are independent of the secret (same randomness, same prefix)", () => {
  // The confidentiality claim the panel makes: a party's received view is uniform random.
  // Concretely, re-running with the SAME randomness but a DIFFERENT secret must leave every
  // share except the closing one byte-identical.
  const a = splitShares(11, 4, seeded(99));
  const b = splitShares(77, 4, seeded(99));
  assert.deepEqual(a.slice(0, -1), b.slice(0, -1));
  assert.notEqual(a[a.length - 1], b[b.length - 1]);
});

// ---------------------------------------------------------------------------
// The secure-threshold flow
// ---------------------------------------------------------------------------

const PARTIES: Party[] = [
  { name: "Alice", value: 30 },
  { name: "Bob", value: 25 },
  { name: "Carol", value: 41 },
  { name: "Dan", value: 19 },
];

test("runSecureThreshold combines local sums into the total and reveals only the verdict", () => {
  const r = runSecureThreshold(PARTIES, DEFAULT_THRESHOLD, seeded(5));
  const total = PARTIES.reduce((a, p) => a + p.value, 0);

  assert.equal(r.matrix.length, 4);
  assert.equal(r.matrix[0].length, 4);
  // Exactly the diagonal is kept; every other cell crosses the wire.
  for (let i = 0; i < 4; i++) {
    for (let j = 0; j < 4; j++) assert.equal(r.matrix[i][j].kept, i === j);
  }
  // received[j] is column j — one share from every party.
  for (let j = 0; j < 4; j++) {
    assert.deepEqual(
      r.received[j],
      r.matrix.map((row) => row[j].value),
    );
  }
  assert.equal(reconstruct(r.localSums), total);
  assert.equal(r.totalRedacted, total);
  assert.equal(r.verdict, total >= DEFAULT_THRESHOLD);
});

test("runSecureThreshold's verdict is the only threshold-derived output", () => {
  const below = runSecureThreshold(PARTIES, 1_000, seeded(5));
  assert.equal(below.verdict, false);
  // The exact total is still computed — surfaced ONLY to show what is withheld.
  assert.equal(below.totalRedacted, 115);
});

test("runSecureThreshold rejects a single-party run", () => {
  assert.throws(() => runSecureThreshold(PARTIES.slice(0, 1), 10, seeded(1)), /at least 2 parties/);
});

// ---------------------------------------------------------------------------
// The aggregate field bound — the invariant behind "Σ contributions ≥ threshold"
// ---------------------------------------------------------------------------

test("runSecureThreshold answers exactly at the largest aggregate the field carries", () => {
  // Sum lands precisely on MAX_TOTAL: reconstruction is still exact, so the displayed
  // predicate must be the mathematical one at both sides of the threshold.
  const edge: Party[] = [
    { name: "Alice", value: FIELD_P - 100 },
    { name: "Bob", value: 99 },
  ];
  const met = runSecureThreshold(edge, MAX_TOTAL, seeded(11));
  assert.equal(met.totalRedacted, MAX_TOTAL);
  assert.equal(met.verdict, true);

  const missed = runSecureThreshold(edge, MAX_TOTAL + 1, seeded(11));
  assert.equal(missed.totalRedacted, MAX_TOTAL);
  assert.equal(missed.verdict, false);
});

test("runSecureThreshold REFUSES an input set whose sum would wrap the field", () => {
  // The reviewer's case: two contributions of p − 1 reconstruct to p − 2, so comparing that
  // residue with a threshold of p − 1 used to answer `false` for a sum of nearly 2p. There
  // is no correct verdict to return from a wrapped residue, so the run is declined instead.
  const wrapping: Party[] = [
    { name: "Alice", value: FIELD_P - 1 },
    { name: "Bob", value: FIELD_P - 1 },
  ];
  assert.throws(() => runSecureThreshold(wrapping, MAX_TOTAL, seeded(13)), /sum past the field/);

  // The first total that is NOT representable is exactly p.
  const justOver: Party[] = [
    { name: "Alice", value: FIELD_P - 1 },
    { name: "Bob", value: 1 },
  ];
  assert.throws(() => runSecureThreshold(justOver, 10, seeded(13)), /sum past the field/);

  // The error names the party that crossed it, so the operator can see which row to change.
  assert.throws(() => runSecureThreshold(justOver, 10, seeded(13)), /"Bob"/);
});

test("runSecureThreshold rejects a contribution outside the field", () => {
  assert.throws(
    () => runSecureThreshold([{ name: "Alice", value: FIELD_P }, PARTIES[1]], 10, seeded(3)),
    /not a non-negative integer below the field prime/,
  );
  assert.throws(
    () => runSecureThreshold([{ name: "Alice", value: -1 }, PARTIES[1]], 10, seeded(3)),
    /not a non-negative integer below the field prime/,
  );
  assert.throws(
    () => runSecureThreshold([{ name: "Alice", value: 1.5 }, PARTIES[1]], 10, seeded(3)),
    /not a non-negative integer below the field prime/,
  );
});

test("runSecureThreshold rejects a threshold it cannot compare exactly", () => {
  assert.throws(() => runSecureThreshold(PARTIES, 1.5, seeded(3)), /non-negative integer/);
  assert.throws(() => runSecureThreshold(PARTIES, -1, seeded(3)), /non-negative integer/);
  assert.throws(() => runSecureThreshold(PARTIES, Number.NaN, seeded(3)), /non-negative integer/);
});

// ---------------------------------------------------------------------------
// SELECT → parties
// ---------------------------------------------------------------------------

test("partiesFromResults picks the numeric column as the contribution and the other as the name", () => {
  const scan = partiesFromResults(
    results([
      { party: "Alice", contribution: "30" },
      { party: "Bob", contribution: "25" },
    ]),
  );
  assert.equal(scan.valueVar, "contribution");
  assert.equal(scan.nameVar, "party");
  assert.deepEqual(scan.parties, [
    { name: "Alice", value: 30 },
    { name: "Bob", value: 25 },
  ]);
  assert.deepEqual(scan.skipped, []);
});

test("partiesFromResults surfaces every unusable row with a reason instead of dropping it", () => {
  const scan = partiesFromResults(
    results([
      { party: "Alice", contribution: "30" },
      { party: "Bob" }, // unbound contribution
      { party: "Carol", contribution: "many" }, // not a number
      { party: "Dan", contribution: "-4" }, // negative → not shareable in F_p
      { party: "Erin", contribution: "1.5", datatype: `${XSD}decimal` }, // fractional
    ]),
  );
  assert.deepEqual(scan.parties, [{ name: "Alice", value: 30 }]);
  assert.deepEqual(
    scan.skipped.map((s) => s.label),
    ["Bob", "Carol", "Dan", "Erin"],
  );
  assert.match(scan.skipped[0].reason, /no numeric literal/);
  assert.match(scan.skipped[1].reason, /not a number/);
  assert.match(scan.skipped[2].reason, /non-negative integer/);
  assert.match(scan.skipped[3].reason, /non-negative integer/);
});

test("partiesFromResults labels rows by index when the SELECT projects one column", () => {
  const scan = partiesFromResults(results([{ contribution: "30" }], ["contribution"]));
  assert.equal(scan.nameVar, null);
  assert.deepEqual(scan.parties, [{ name: "row 1", value: 30 }]);
});
