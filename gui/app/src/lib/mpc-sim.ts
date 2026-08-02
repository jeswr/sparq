// [OPUS-5] sq-ixc3.17 — the MPC tool's simulation core, ported from the site's
// `/showcase/mpc-100k` demo (site/src/lib/mpc-sim.ts) with the marketing chrome CUT and the
// party list rebound to the LIVE WORKSPACE STORE.
//
// Translation rule (research/gui-design.md §A.4/§A.5): the site page runs the additive-sharing
// illustration over four hard-coded salaries. Here the parties come from a SPARQL SELECT the
// user runs against the actual imported store, so the shares, the local sums and the disclosed
// verdict are all derived from the user's own data — never a fixture.
//
// HONESTY (load-bearing, unchanged from the site's framing):
//   * This is an ILLUSTRATION of the protocol SHAPE, not the hardened `sparq-mpc` crate and NOT
//     live MPC. There is no network, no peer, and no cryptographic guarantee here.
//   * It uses plain additive (n-out-of-n) sharing over a small prime field — the simplest sharing
//     that already makes "no proper subset of shares reveals anything" and "addition is free over
//     shares" visible. The native crate uses honest-majority *Shamir* t-of-n sharing (degree-t
//     polynomials) and a bit-decomposition secure comparison.
//   * The native crate itself is honest-majority *semi-honest* only, makes no production security
//     claim, and its collaborative zero-knowledge proof-of-correctness layer is a stub that
//     returns `NotYetImplemented`. External accredited-cryptographer sign-off is pending
//     (bead sq-qhy4). See compliance/cryptoreview/README.md and SECURITY.md.
//
// This module is PURE (no React, no `@/` aliases) so `npm run test:unit` can exercise it under
// `node --test`.

import type { SparqlResults } from "@sparq/client";

/**
 * A small prime field, mirroring the crate's F_p (p = 2^61 - 1) in spirit. A JS-safe prime well
 * under 2^53 keeps every intermediate exact in IEEE-754 doubles (no BigInt), which matters
 * because the values now come from arbitrary user data rather than a curated fixture.
 */
export const FIELD_P = 2_147_483_647; // 2^31 - 1 (a Mersenne prime), exact in f64

function mod(x: number): number {
  const r = x % FIELD_P;
  return r < 0 ? r + FIELD_P : r;
}

/**
 * Split `secret` into `n` additive shares over F_p.
 *
 * Shares `s_0 … s_{n-2}` are uniform-random field elements; the last share is chosen so that
 * `Σ s_i ≡ secret (mod p)`. Therefore ANY proper subset of the shares is uniformly random and
 * independent of the secret — that is the confidentiality property the panel visualises (no
 * party, seeing only the shares it holds, learns anything about another party's value).
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
  // last share closes the sum to `secret`
  shares.push(mod(mod(secret) - acc));
  return shares;
}

/** Reconstruct a secret from a FULL set of additive shares: Σ shares (mod p). */
export function reconstruct(shares: number[]): number {
  return shares.reduce((a, s) => mod(a + s), 0);
}

export interface Party {
  /** Display name — the `?party` binding from the live-store query. */
  name: string;
  /** The party's contribution, kept out of every disclosed output below. */
  value: number;
  /** The store subject/term the value was read from (shown so a row is traceable to the data). */
  source?: string;
}

export interface ShareMatrixCell {
  /** Share value party `from` computed for party `to`. */
  value: number;
  /** True iff this share stays with the originating party (the diagonal). */
  kept: boolean;
}

export interface MpcResult {
  /** parties[i] = the input row this run was given. */
  parties: Party[];
  /**
   * matrix[i][j] = the share party i produced for party j.
   * Off-diagonal cells (i ≠ j) are SENT to peer j (would cross the wire in the real protocol).
   * Diagonal cells (i === j) are KEPT by party i.
   */
  matrix: ShareMatrixCell[][];
  /**
   * received[j] = the column party j ends up holding (one share from every party, including its
   * own) — the per-party "local view". Summing this column reveals no single party's value.
   */
  received: number[][];
  /** Per-party local partial sum over the shares it received (free, zero-round addition). */
  localSums: number[];
  /**
   * The reconstructed total — Σ localSums = Σ all inputs. In the real protocol this is NEVER
   * opened; it is surfaced only so the panel can show what is REDACTED.
   */
  totalRedacted: number;
  /** The public threshold. */
  threshold: number;
  /** The ONLY value the run discloses: total ≥ threshold. */
  verdict: boolean;
}

/**
 * Run the additive-secret-sharing illustration of the secure sum + threshold. Mirrors the
 * crate's flow (ShamirBackend::share_private_input → run_secure → disclose_threshold_verdict):
 *   1. each party secret-shares its value (splitShares),
 *   2. shares are distributed (the N×N matrix; off-diagonal = sent),
 *   3. each party locally sums the shares it received (free, zero-round),
 *   4. the per-party local sums are combined → the secret-shared total,
 *   5. ONLY the boolean `total ≥ threshold` is disclosed; the exact total is never an output.
 *
 * `rand` is injectable so a run can be made deterministic; the default is `Math.random` (NOT a
 * CSPRNG — this is illustration, not security).
 *
 * Throws when the inputs would make the illustration DISHONEST — see {@link describeInputProblem}.
 * Callers should surface that message rather than render a wrapped-around verdict.
 */
export function runSecureThreshold(
  parties: Party[],
  threshold: number,
  rand: () => number = Math.random,
): MpcResult {
  const problem = describeInputProblem(parties, threshold);
  if (problem) throw new Error(problem);
  const n = parties.length;

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

  // 4 + 5: combining the local sums reconstructs Σ inputs. The real protocol never opens this;
  // it is computed only to display the REDACTED value and to derive the one disclosed bit.
  const totalRedacted = reconstruct(localSums);
  const verdict = totalRedacted >= threshold;

  return { parties, matrix, received, localSums, totalRedacted, threshold, verdict };
}

/**
 * Why this party list + threshold cannot be run honestly, or `null` when it can.
 *
 * The site demo could skip this: its four salaries were curated. Live-store values are arbitrary,
 * and the field is small — once `Σ values ≥ FIELD_P` the reconstruction wraps and the disclosed
 * verdict would be WRONG while still looking plausible. Refusing is the honest outcome.
 */
export function describeInputProblem(parties: Party[], threshold: number): string | null {
  if (parties.length < 2) {
    return `Secret sharing needs at least 2 parties; this query produced ${parties.length}.`;
  }
  if (!Number.isSafeInteger(threshold) || threshold < 0) {
    return "The threshold must be a non-negative integer.";
  }
  for (const p of parties) {
    if (!Number.isSafeInteger(p.value) || p.value < 0 || p.value >= FIELD_P) {
      return `Party "${p.name}" has value ${p.value}, which is outside the illustration's field [0, ${FIELD_P}).`;
    }
  }
  const total = parties.reduce((a, p) => a + p.value, 0);
  if (total >= FIELD_P) {
    return (
      `The values sum to ${total}, which is at or above the illustration's field size ` +
      `${FIELD_P}. Reconstruction would wrap and the disclosed verdict would be wrong, so the ` +
      `run is refused rather than shown. Narrow the query or scale the values down.`
    );
  }
  return null;
}

// ---------------------------------------------------------------------------
// Live-store adapter — the part the site demo has no equivalent of.
// ---------------------------------------------------------------------------

/** A query row that could NOT become a party, with the honest reason it was dropped. */
export interface SkippedRow {
  /** 0-based index of the row in the SELECT result. */
  row: number;
  /** The `?party` term as it appeared, or "" when unbound. */
  party: string;
  reason: string;
}

export interface PartySelection {
  parties: Party[];
  skipped: SkippedRow[];
  /** The variable read as the party label. */
  partyVar: string;
  /** The variable read as the contributed value. */
  valueVar: string;
}

/**
 * The default SPARQL the MPC tool opens with. It binds over the workbench's seeded sample graph
 * (four `foaf:Person`s with `foaf:age`) but is a plain SELECT — any query the user rewrites that
 * projects a label and a non-negative integer works the same way.
 */
export const DEFAULT_MPC_QUERY = `PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?party ?value WHERE {
  ?s foaf:name ?party ;
     foaf:age  ?value .
}
ORDER BY ?party`;

/**
 * Read the parties out of a SELECT result over the live store.
 *
 * The first projected variable is the party label and the second the contributed value, unless
 * the result binds `?party` / `?value` explicitly. Rows whose value is not a non-negative integer
 * are DROPPED with a stated reason rather than coerced — a silently-coerced row would change the
 * disclosed verdict.
 */
export function partiesFromResults(results: SparqlResults): PartySelection {
  const vars = results.head?.vars ?? [];
  const partyVar = vars.includes("party") ? "party" : (vars[0] ?? "");
  const valueVar = vars.includes("value")
    ? "value"
    : (vars.find((v) => v !== partyVar) ?? "");

  const parties: Party[] = [];
  const skipped: SkippedRow[] = [];
  const bindings = results.results?.bindings ?? [];

  bindings.forEach((row, i) => {
    const nameTerm = partyVar ? row[partyVar] : undefined;
    const valueTerm = valueVar ? row[valueVar] : undefined;
    const name = nameTerm?.value ?? "";
    if (!valueTerm) {
      skipped.push({ row: i, party: name, reason: `?${valueVar || "value"} is unbound` });
      return;
    }
    const parsed = Number(valueTerm.value);
    if (!Number.isFinite(parsed) || !Number.isInteger(parsed)) {
      skipped.push({
        row: i,
        party: name,
        reason: `"${valueTerm.value}" is not an integer`,
      });
      return;
    }
    if (parsed < 0) {
      skipped.push({ row: i, party: name, reason: `${parsed} is negative` });
      return;
    }
    parties.push({
      name: name || `party ${i + 1}`,
      value: parsed,
      source: nameTerm?.type === "uri" ? nameTerm.value : undefined,
    });
  });

  return { parties, skipped, partyVar, valueVar };
}
