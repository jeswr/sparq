// [GPT-5] sq-ixc3.17 — additive-sharing protocol illustration used by the operational MPC tool.
// This is deliberately a simulation, not the native sparq-mpc protocol.
export const FIELD_P = 2_147_483_647;

function mod(value: number): number {
  const result = value % FIELD_P;
  return result < 0 ? result + FIELD_P : result;
}

export interface MpcParty {
  name: string;
  value: number;
}

export interface MpcSimulation {
  shares: number[][];
  localSums: number[];
  verdict: boolean;
}

export function simulateThreshold(
  parties: MpcParty[],
  threshold: number,
  random: () => number = Math.random,
): MpcSimulation {
  if (parties.length < 2) throw new Error("At least two parties are required");
  const shares = parties.map(({ value }) => {
    const row: number[] = [];
    let sum = 0;
    for (let index = 0; index < parties.length - 1; index += 1) {
      const share = Math.floor(random() * FIELD_P);
      row.push(share);
      sum = mod(sum + share);
    }
    row.push(mod(value - sum));
    return row;
  });
  const localSums = parties.map((_, receiver) =>
    shares.reduce((sum, row) => mod(sum + row[receiver]), 0),
  );
  const total = localSums.reduce((sum, value) => mod(sum + value), 0);
  return { shares, localSums, verdict: total >= threshold };
}

/** Extract non-negative integer literals from the active workspace snapshot. */
export function workspaceIntegers(snapshot: string): number[] {
  const values = new Set<number>();
  const pattern = /"(\d+)"\^\^<http:\/\/www\.w3\.org\/2001\/XMLSchema#integer>/g;
  for (const match of snapshot.matchAll(pattern)) {
    const value = Number(match[1]);
    if (Number.isSafeInteger(value)) values.add(value);
  }
  return [...values];
}
