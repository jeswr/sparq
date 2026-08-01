// [OPUS-5] sq-ixc3.17 — the workbench MPC tool's protocol logic, ported from the site's
// £100k secure-threshold demo (`site/src/lib/mpc-sim.ts`, sq-3hrc) with the loan-application
// narrative cut and a store-driven party source added. No React, no DOM — unit-tested with
// the gui/app node test runner; components/workbench/mpc-tool.tsx is a thin renderer.
//
// HONESTY (load-bearing): this is an ILLUSTRATION of the protocol SHAPE, not the hardened
// crate and NOT live MPC. It uses plain additive (n-out-of-n) secret sharing over a prime
// field — the simplest sharing that already makes "no single share reveals anything" and
// "addition is free over shares" visible. The native `sparq-mpc` crate uses honest-majority
// *Shamir* t-of-n sharing (degree-t polynomials) and a bit-decomposition secure comparison;
// the additive variant here gives the same intuition without the polynomial machinery. NO
// real MPC, NO network, NO cryptographic guarantee is provided by this file — the panel says
// so. The native crate itself is honest-majority *semi-honest* only (no malicious security;
// the collaborative ZK proof-of-correctness layer is a stub) — see
// compliance/cryptoreview/README.md.

import type { SparqlResults } from "@sparq/client";

/** A small prime field, mirroring the crate's F_p (p = 2^61 − 1) in spirit. We use a
 *  JS-safe prime well under 2^53 so all arithmetic stays exact in IEEE-754 doubles. */
export const FIELD_P = 2_147_483_647; // 2^31 − 1 (a Mersenne prime), exact in f64

/**
 * The largest AGGREGATE the additive sharing can carry EXACTLY, and the invariant that makes
 * the revealed bit MEAN what the panel says it means.
 *
 * Reconstruction is a sum *modulo p*, so it recovers `Σ contributions` only while that sum
 * stays below `p`; past the prime it wraps to a small residue. Bounding each contribution
 * individually is NOT enough — two parties contributing `p − 1` each reconstruct to `p − 2`,
 * and a threshold comparison against that residue would confidently answer `false` for a
 * total that is in fact nearly twice the prime. {@link runSecureThreshold} therefore enforces
 * the AGGREGATE bound and refuses an input set that would wrap, rather than returning a wrong
 * verdict — the same "decline, never silently wrap" rule the per-row scan already applies.
 */
export const MAX_TOTAL = FIELD_P - 1;

function mod(x: number): number {
  const r = x % FIELD_P;
  return r < 0 ? r + FIELD_P : r;
}

/**
 * Split `secret` into `n` additive shares over F_p.
 *
 * Shares `s_0 … s_{n−2}` are uniform-random field elements; the last share is chosen so
 * that `Σ s_i ≡ secret (mod p)`. Therefore ANY proper subset of the shares is uniformly
 * random and independent of the secret — the confidentiality property the panel visualises.
 */
export function splitShares(secret: number, n: number, rand: () => number): number[] {
  if (n < 2) throw new Error("need at least 2 parties");
  const shares: number[] = [];
  let acc = 0;
  for (let i = 0; i < n - 1; i++) {
    const r = Math.floor(rand() * FIELD_P);
    shares.push(r);
    acc = mod(acc + r);
  }
  shares.push(mod(mod(secret) - acc)); // last share closes the sum to `secret`
  return shares;
}

/** Reconstruct a secret from a full set of additive shares: Σ shares (mod p). */
export function reconstruct(shares: number[]): number {
  return shares.reduce((a, s) => mod(a + s), 0);
}

export interface Party {
  /** Display name — the row's label binding from the live store. */
  name: string;
  /** The party's PRIVATE contribution (never leaves the party in the protocol). */
  value: number;
}

export interface ShareMatrixCell {
  /** Share value party `from` computed for party `to`. */
  value: number;
  /** True iff this share stays with the originating party (the diagonal). */
  kept: boolean;
}

export interface MpcResult {
  /** parties[i] = the original input row. */
  parties: Party[];
  /** matrix[i][j] = the share party i produced for party j. Off-diagonal cells (i ≠ j) are
   *  SENT to peer j (they cross the wire); diagonal cells are KEPT by party i. */
  matrix: ShareMatrixCell[][];
  /** received[j] = the column party j ends up holding (one share from every party,
   *  including its own) — the per-party "local view". */
  received: number[][];
  /** Per-party local partial sum over the shares it received (free, zero-round addition). */
  localSums: number[];
  /** The reconstructed total. Because {@link runSecureThreshold} enforces
   *  `Σ contributions ≤ MAX_TOTAL`, this residue IS the exact mathematical sum — not a
   *  wrapped one. In the real protocol it is NEVER opened; it is surfaced only to show what
   *  is REDACTED. */
  totalRedacted: number;
  /** The public threshold. */
  threshold: number;
  /** The ONLY value revealed: total ≥ threshold. */
  verdict: boolean;
}

/**
 * Run the additive-secret-sharing illustration of the secure sum + threshold, mirroring the
 * crate's flow:
 *   1. each party secret-shares its private value (splitShares),
 *   2. shares are distributed (the N×N matrix; off-diagonal = sent),
 *   3. each party locally sums the shares it received (free, zero-round),
 *   4. the per-party local sums are combined → the secret-shared total,
 *   5. ONLY the boolean `total ≥ threshold` is revealed; the exact total is never an
 *      output (here retained as `totalRedacted` purely to show what is withheld).
 *
 * DOMAIN (checked, not assumed). Every contribution must be a non-negative integer below
 * `FIELD_P`, the threshold must be a non-negative integer, and — the aggregate invariant that
 * step 5's claim rests on — `Σ contributions` must not exceed {@link MAX_TOTAL}. Reconstruction
 * is a sum mod p, so past the prime the verdict would be about a wrapped residue rather than
 * the sum; an input set that would wrap is REFUSED with that reason instead of answered.
 *
 * `rand` is injectable so the panel (and the tests) can be deterministic; the default is
 * `Math.random` (NOT a CSPRNG — this is illustration, not security).
 */
export function runSecureThreshold(
  parties: Party[],
  threshold: number,
  rand: () => number = Math.random,
): MpcResult {
  const n = parties.length;
  if (n < 2) throw new Error("need at least 2 parties");

  if (!Number.isInteger(threshold) || threshold < 0) {
    throw new Error(
      `the public threshold must be a non-negative integer, got ${String(threshold)}`,
    );
  }

  // The aggregate range check. Each contribution is re-validated here (the exported entry
  // point must not trust its caller) and the running total is checked against MAX_TOTAL as it
  // accumulates — bailing on the party that crosses the prime keeps `total` itself exact in
  // f64, and names the row an operator has to change.
  let total = 0;
  for (const p of parties) {
    if (!Number.isInteger(p.value) || p.value < 0 || p.value >= FIELD_P) {
      throw new Error(
        `party "${p.name}" contributes ${String(p.value)}, which is not a non-negative integer below the field prime ${FIELD_P}`,
      );
    }
    total += p.value;
    if (total > MAX_TOTAL) {
      throw new Error(
        `the contributions sum past the field prime (running total ${total} exceeds ${MAX_TOTAL} at party "${p.name}"): the additive sharing reconstructs modulo p, so a threshold comparison here would answer about the wrapped residue, not about the sum. Narrow the query or scale the contribution column down.`,
      );
    }
  }

  // 1 + 2: every party splits its value into n shares (row i of the matrix).
  const matrix: ShareMatrixCell[][] = parties.map((p) => {
    const row = splitShares(p.value, n, rand);
    return row.map((value): ShareMatrixCell => ({ value, kept: false }));
  });
  for (let i = 0; i < n; i++) matrix[i][i].kept = true;

  // received[j] = column j across all rows = the shares party j now holds.
  const received: number[][] = [];
  for (let j = 0; j < n; j++) {
    received.push(matrix.map((row) => row[j].value));
  }

  // 3: each party sums the shares it holds (a local share of the global sum).
  const localSums = received.map((col) => reconstruct(col));

  // 4 + 5: combining the local sums reconstructs Σ inputs. The real protocol never opens
  // this; it is computed only to display the REDACTED value and derive the one revealed bit.
  const totalRedacted = reconstruct(localSums);

  // The aggregate check above guarantees the residue IS the exact sum, so `verdict` below is
  // the mathematical predicate `Σ contributions ≥ threshold` the panel displays — not a
  // statement about a wrapped field element. Assert the equality rather than assume it.
  if (totalRedacted !== total) {
    throw new Error(
      `internal invariant broken: reconstruction gave ${totalRedacted} but the contributions total ${total}`,
    );
  }

  return {
    parties,
    matrix,
    received,
    localSums,
    totalRedacted,
    threshold,
    verdict: totalRedacted >= threshold,
  };
}

// ---------------------------------------------------------------------------
// Live-store party source — the operational half of the tool (sq-ixc3.17).
// ---------------------------------------------------------------------------

/** A row the scan could NOT turn into a party, with the exact reason. */
export interface SkippedRow {
  label: string;
  reason: string;
}

/** What a SELECT over the live store yielded as protocol inputs. */
export interface PartyScan {
  /** The variable used as the party name, or `null` when the SELECT projects one var. */
  nameVar: string | null;
  /** The variable holding each party's private contribution, or `null` when none bound. */
  valueVar: string | null;
  parties: Party[];
  /** Rows the scan could not use — surfaced, never silently dropped. */
  skipped: SkippedRow[];
}

/** Parse a literal's lexical form as a field-representable contribution, or `null`. */
function asContribution(lexical: string): number | null {
  const n = Number(lexical);
  return Number.isFinite(n) ? n : null;
}

/** The first projected variable binding a numeric literal in at least one row. */
function pickValueVar(results: SparqlResults): string | null {
  for (const v of results.head.vars) {
    const hit = results.results.bindings.some((b) => {
      const t = b[v];
      return t?.type === "literal" && asContribution(t.value) !== null;
    });
    if (hit) return v;
  }
  return null;
}

/**
 * Turn a SELECT over the live store into the protocol's private inputs: the first numeric
 * column is each party's contribution, the first other column names the party.
 *
 * Rows that cannot be used are reported in {@link PartyScan.skipped} with the reason — an
 * unbound cell, a non-numeric literal, or a value outside the field's exact range. The
 * additive sharing is defined over F_p on non-negative integers, so a negative or
 * fractional contribution is DECLINED rather than silently wrapped mod p (which would
 * produce a confident, wrong verdict).
 *
 * This is a PER-ROW filter only: the rows it keeps can still sum past the prime, which would
 * wrap the aggregate. That bound belongs to the whole input set, so it is enforced (and the
 * run refused with the reason) by {@link runSecureThreshold} — dropping high rows here would
 * quietly answer about a different set of parties than the one the panel lists.
 */
export function partiesFromResults(results: SparqlResults): PartyScan {
  const valueVar = pickValueVar(results);
  const nameVar =
    results.head.vars.find(
      (v) => v !== valueVar && results.results.bindings.some((b) => b[v] !== undefined),
    ) ?? null;

  const parties: Party[] = [];
  const skipped: SkippedRow[] = [];

  results.results.bindings.forEach((binding, row) => {
    const label = (nameVar ? binding[nameVar]?.value : undefined) ?? `row ${row + 1}`;
    const term = valueVar ? binding[valueVar] : undefined;
    if (!term || term.type !== "literal") {
      skipped.push({ label, reason: "no numeric literal bound in the contribution column" });
      return;
    }
    const value = asContribution(term.value);
    if (value === null) {
      skipped.push({ label, reason: `"${term.value}" is not a number` });
      return;
    }
    if (!Number.isInteger(value) || value < 0 || value >= FIELD_P) {
      skipped.push({
        label,
        reason: `${term.value} is not a non-negative integer below the field prime, so it cannot be shared exactly`,
      });
      return;
    }
    parties.push({ name: label, value });
  });

  return { nameVar, valueVar, parties, skipped };
}

/**
 * The default contribution query the tool opens with — a plain SELECT over whatever the
 * operator already imported. It is NOT a fixture the tool secretly computes over: the
 * parties come from the live store, and an empty store yields an honestly empty run.
 */
export const DEFAULT_PARTIES_QUERY = `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
# Each row is one party's PRIVATE contribution. The first numeric column is the value;
# the first other column names the party. Point this at YOUR live store's columns.
SELECT ?party ?contribution WHERE {
  ?p foaf:name ?party ;
     foaf:age  ?contribution .
}
ORDER BY ?party`;

/** The threshold the tool opens with — a public constant, chosen so the seeded sample
 *  store produces a meaningful verdict; the operator edits it freely. */
export const DEFAULT_THRESHOLD = 100;
