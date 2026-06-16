// [OPUS-4.8] sq-13rg — REAL in-browser zero-knowledge proving for the car-hire
// age-gate, via Noir + Barretenberg UltraHonk compiled to WASM.
//
// This is NOT a simulation. The age-gate sub-proof of the car-hire flagship runs
// the genuine sparq ZK circuit `zk/compose/filter_int_d2` (compiled with
// `nargo 1.0.0-beta.21` to the ACIR JSON shipped at `/zk/filter_int_d2.json`) and
// proves it with `@aztec/bb.js` (UltraHonk, the same prover the native crate drives
// over its `bb` subprocess) entirely in your tab:
//
//   private witness: the holder's exact age (the decimal digits)
//   public inputs:   challenge, operand_enc (the committed term anchor), op (≥),
//                    bound (25), expected (true)  — the age itself is NEVER public.
//
// The circuit (`sparq_zk_compose_core::filter_int`) rebuilds the canonical
// `"<digits>"^^xsd:integer` N-Triples token in-circuit, hashes it (blake3 blackbox),
// and asserts `h2(LITERAL, hs) == operand_enc` — binding the hidden value to the
// committed credential — then asserts the comparison verdict. A false claim
// (e.g. age 24 proving "≥ 25 = true") is UNSATISFIABLE: the witness solve fails, so
// no proof can be produced. That is the soundness the demo demonstrates live.
//
// HONESTY: the *cryptographic mechanics* here are real and run in your browser. The
// broader sparq ZK estate is research-grade, internally reviewed only, and NOT
// externally audited — see the page's honesty panel and SECURITY.md. Do not treat a
// proof produced here as production-trustworthy.
//
// bb.js + noir_js are lazy-loaded via dynamic import (client-only) so the ~MB WASM
// never enters the main page bundle, mirroring the lean-wasm REPL loader.

/** Field encodings of `"<age>"^^xsd:integer`, precomputed natively via
 *  `sparq_zk_compose::build::encode_int_literal` (the SAME encoder the scan proof
 *  uses for the operand column). Each is the `operand_enc` public input that binds
 *  the hidden age to the committed credential. */
const AGE_OPERAND_ENC: Record<number, string> = {
  24: "0x1c8a81ea95b253e105b99209deff1a4908be9568e588fbd89afea9f49f5f20cf",
  25: "0x2b5caeb2bbd290ab32434a9109030784c7faebadee7a9908d24dccb847910d1d",
  30: "0x132fa587351bf3f12fd3cbed64d5526f28791099d1d40870f94595873c78fa72",
  42: "0x1a4aa7fd962d0004ac2294cc98471ea1ebfdad74a8f702e89fedf83f92d0f97b",
};

/** The ages we can prove against (those for which we shipped a committed
 *  `operand_enc`). All are 2-digit, matching the `filter_int_d2` circuit member. */
export const PROVABLE_AGES = Object.keys(AGE_OPERAND_ENC)
  .map(Number)
  .sort((a, b) => a - b);

export const AGE_THRESHOLD = 25;

export interface ProofResult {
  /** The disclosed eligibility verdict (the only age-derived bit revealed). */
  eligible: boolean;
  /** The proof bytes — opaque to the verifier; reveals nothing about the age. */
  proofByteLength: number;
  /** The public inputs the verifier sees (NOT the age). */
  publicInputs: string[];
  /** Independent in-tab verification of the proof against the same circuit. */
  verified: boolean;
  proveMs: number;
  verifyMs: number;
  /** bb.js thread count actually used (1 on a non-cross-origin-isolated host). */
  threads: number;
}

interface NoirModule {
  Noir: new (circuit: unknown) => {
    execute(inputs: Record<string, unknown>): Promise<{ witness: Uint8Array }>;
  };
}
interface BbModule {
  Barretenberg: { new: (opts: { threads: number }) => Promise<unknown> };
  UltraHonkBackend: new (
    bytecode: string,
    api: unknown,
  ) => {
    generateProof(
      witness: Uint8Array,
    ): Promise<{ proof: Uint8Array; publicInputs: string[] }>;
    verifyProof(proof: {
      proof: Uint8Array;
      publicInputs: string[];
    }): Promise<boolean>;
  };
}

let circuitPromise: Promise<{ bytecode: string }> | null = null;

function basePath(): string {
  return process.env.NEXT_PUBLIC_BASE_PATH ?? "/sparq";
}

/** Fetches the committed ACIR artifact once. */
async function loadCircuit(): Promise<{ bytecode: string }> {
  if (!circuitPromise) {
    circuitPromise = fetch(`${basePath()}/zk/filter_int_d2.json`).then((r) => {
      if (!r.ok) throw new Error(`circuit fetch failed: ${r.status}`);
      return r.json();
    });
    circuitPromise.catch(() => {
      circuitPromise = null;
    });
  }
  return circuitPromise;
}

/** Cross-origin isolation gates bb.js multithreading (SharedArrayBuffer). GitHub
 *  Pages cannot set COOP/COEP, so this is normally false → single-threaded. */
function maxThreads(): number {
  if (typeof window === "undefined") return 1;
  if (!window.crossOriginIsolated) return 1;
  return Math.min(navigator.hardwareConcurrency || 1, 8);
}

/** Two ASCII decimal digit-bytes of a 2-digit age (the private witness). */
function ageDigits(age: number): string[] {
  const s = String(age).padStart(2, "0");
  return [String(s.charCodeAt(0)), String(s.charCodeAt(1))];
}

/**
 * Generate AND verify a real UltraHonk proof that the holder's hidden age
 * satisfies `age ≥ 25`, entirely in the browser. Throws if `age` is not one of
 * {@link PROVABLE_AGES}. For an under-age value the circuit is unsatisfiable, so
 * `execute` throws — which is the honest "you cannot forge eligibility" outcome.
 */
export async function proveAgeEligibility(age: number): Promise<ProofResult> {
  const operandEnc = AGE_OPERAND_ENC[age];
  if (!operandEnc) {
    throw new Error(`no committed operand_enc fixture for age ${age}`);
  }
  const eligible = age >= AGE_THRESHOLD;

  const [{ Noir }, bb, circuit] = await Promise.all([
    import(/* webpackChunkName: "noir-js" */ "@noir-lang/noir_js") as Promise<NoirModule>,
    import(/* webpackChunkName: "bb-js" */ "@aztec/bb.js") as Promise<BbModule>,
    loadCircuit(),
  ]);

  const inputs = {
    challenge: "0x1", // per-presentation verifier nonce (fixed here for the demo)
    operand_enc: operandEnc,
    op: "3", // OP_GE
    bound: String(AGE_THRESHOLD),
    expected: eligible,
    digits: ageDigits(age), // PRIVATE — the exact age, never disclosed
  };

  const noir = new Noir(circuit);
  // For an under-age claim the verdict assertion fails here: no proof possible.
  const { witness } = await noir.execute(inputs);

  const threads = maxThreads();
  const api = await bb.Barretenberg.new({ threads });
  const backend = new bb.UltraHonkBackend(circuit.bytecode, api);

  const t0 = performance.now();
  const proof = await backend.generateProof(witness);
  const proveMs = performance.now() - t0;

  const t1 = performance.now();
  const verified = await backend.verifyProof(proof);
  const verifyMs = performance.now() - t1;

  return {
    eligible,
    proofByteLength: proof.proof.length,
    publicInputs: proof.publicInputs,
    verified,
    proveMs,
    verifyMs,
    threads,
  };
}
