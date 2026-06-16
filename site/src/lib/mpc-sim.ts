// [OPUS-4.8] sq-3hrc — a FAITHFUL, in-tab JS illustration of the additive
// secret-sharing flow that the native `sparq-mpc` crate runs for the "is the
// combined income ≥ £100k?" secure-threshold example (crate path:
// ShamirBackend::share_private_input → run_secure → secure-threshold /
// disclose_threshold_verdict; SKILL.md recipe 2 / 2b).
//
// HONESTY (load-bearing): this is an ILLUSTRATION of the protocol SHAPE, not
// the hardened crate and NOT live MPC. We use plain additive (n-out-of-n)
// secret sharing over a prime field — the simplest sharing that already makes
// the "no single share reveals anything" and "addition is free over shares"
// properties visible. The native crate uses honest-majority *Shamir* t-of-n
// sharing (degree-t polynomials) and a bit-decomposition secure comparison;
// the additive variant here gives the same intuition without the polynomial
// machinery. NO real MPC, NO network, NO cryptographic guarantee is provided
// by this file — the page must say so. The native crate itself is honest-
// majority *semi-honest* only (no malicious security; the collaborative ZK
// proof-of-correctness layer is a stub) — see compliance/cryptoreview/README.md.

// A small prime field, mirroring the crate's F_p (p = 2^61 - 1) in spirit.
// We use a JS-safe prime well under 2^53 so all arithmetic stays exact in
// IEEE-754 doubles (no BigInt needed for the salary magnitudes in the demo).
export const FIELD_P = 2_147_483_647; // 2^31 - 1 (a Mersenne prime), exact in f64

function mod(x: number): number {
  const r = x % FIELD_P;
  return r < 0 ? r + FIELD_P : r;
}

/**
 * Split `secret` into `n` additive shares over F_p.
 *
 * Shares `s_0 … s_{n-2}` are uniform-random field elements; the last share is
 * chosen so that `Σ s_i ≡ secret (mod p)`. Therefore ANY proper subset of the
 * shares is uniformly random and independent of the secret — that is the
 * confidentiality property the demo visualises (no party, seeing only the
 * shares it holds, learns anything about another party's value).
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

/** Reconstruct a secret from a full set of additive shares: Σ shares (mod p). */
export function reconstruct(shares: number[]): number {
  return shares.reduce((a, s) => mod(a + s), 0);
}

export interface Party {
  /** Display name. */
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
  /** parties[i] = original input. */
  parties: Party[];
  /**
   * matrix[i][j] = the share party i produced for party j.
   * Off-diagonal cells (i ≠ j) are SENT to peer j (cross the wire).
   * Diagonal cells (i === j) are KEPT by party i.
   */
  matrix: ShareMatrixCell[][];
  /**
   * received[j] = the column party j ends up holding (one share from every
   * party, including its own). Summing this column would NOT reveal any single
   * party's value — it is the per-party "local view".
   */
  received: number[][];
  /** Per-party local partial sum over the shares it received (free addition). */
  localSums: number[];
  /** The reconstructed total — Σ localSums = Σ all inputs. In the real protocol
   *  this is NEVER opened; we surface it only to show what is REDACTED. */
  totalRedacted: number;
  /** The public threshold. */
  threshold: number;
  /** The ONLY value revealed: total ≥ threshold. */
  verdict: boolean;
}

/**
 * Run the faithful additive-secret-sharing illustration of the secure sum +
 * threshold. Mirrors the crate's flow:
 *   1. each party secret-shares its private value (splitShares),
 *   2. shares are distributed (the N×N matrix; off-diagonal = sent),
 *   3. each party locally sums the shares it received (free, zero-round),
 *   4. the per-party local sums are combined → the secret-shared total,
 *   5. ONLY the boolean `total ≥ threshold` is revealed; the exact total is
 *      never an output (here shown struck-through as `totalRedacted`).
 *
 * `rand` is injectable so the demo can be deterministic if desired; the default
 * is `Math.random` (NOT a CSPRNG — this is illustration, not security).
 */
export function runSecureThreshold(
  parties: Party[],
  threshold: number,
  rand: () => number = Math.random,
): MpcResult {
  const n = parties.length;
  if (n < 2) throw new Error("need at least 2 parties");

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

  // 4 + 5: combining the local sums reconstructs Σ inputs. The real protocol
  // never opens this; we compute it only to display the REDACTED value and to
  // derive the one revealed bit.
  const totalRedacted = reconstruct(localSums);
  const verdict = totalRedacted >= threshold;

  return {
    parties,
    matrix,
    received,
    localSums,
    totalRedacted,
    threshold,
    verdict,
  };
}
