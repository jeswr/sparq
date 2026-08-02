// [OPUS-5] sq-ixc3.17 — the in-tab UltraHonk prover the ZK tool drives, ported from the site's
// `/showcase/zk-car-hire` demo (site/src/lib/zk-prover.ts).
//
// This is NOT a simulation: it runs the genuine sparq ZK circuit `zk/compose/filter_int_d2`
// (compiled with `nargo 1.0.0-beta.21` to the ACIR JSON synced to `public/zk/`) and proves it
// with `@aztec/bb.js` (UltraHonk, the same prover the native crate drives over its `bb`
// subprocess) entirely in the tab:
//
//   private witness: the store value's decimal digits
//   public inputs:   challenge, operand_enc (the committed term anchor), op, bound, expected
//                    — the value itself is NEVER public.
//
// What differs from the site page: the operand is a value the user's own SPARQL SELECT returned
// from the LIVE WORKSPACE STORE, and `op`/`bound` are the user's choice (both are public circuit
// inputs). The circuit asserts the claimed verdict equals the one the hidden value actually
// satisfies, so a false claim is unsatisfiable — the witness solve fails and no proof exists.
//
// HONESTY: the cryptographic mechanics here are real and run in the browser. The broader sparq ZK
// estate is research-grade, internally re-audited only, and external accredited-cryptographer
// sign-off is pending (bead sq-qhy4) — see SECURITY.md and the panel's honesty strip. Do not
// treat a proof produced here as a production guarantee.
//
// bb.js + noir_js are lazy-loaded via dynamic import (client-only) so the ~MB WASM never enters
// the workbench's shared chunks, mirroring the lean-wasm engine loader.

import { basePath } from "@/lib/base-path";
import {
  CIRCUIT_MEMBER,
  digitBytes,
  evaluateOp,
  termAnchor,
  type OpCode,
} from "@/lib/zk-filter";

/** Where `gui/app/scripts/sync-wasm.mjs` places the synced ACIR artifact. */
const CIRCUIT_URL = () => `${basePath()}/zk/${CIRCUIT_MEMBER}.json`;

/**
 * The message shown when the ACIR artifact is not present — e.g. a build whose asset sync did not
 * run. The tool degrades honestly to this rather than fabricating a proof.
 */
export const CIRCUIT_MISSING_MESSAGE =
  `The ${CIRCUIT_MEMBER} circuit artifact is not in this build. It is synced into public/zk/ ` +
  "from site/public/zk/ by `npm run sync-wasm` (the predev/prebuild hook); re-run the build to " +
  "enable in-tab proving.";

interface NoirModule {
  Noir: new (circuit: unknown) => {
    execute(inputs: Record<string, unknown>): Promise<{ witness: Uint8Array }>;
  };
}

/**
 * Subset of `@aztec/bb.js`'s `UltraHonkBackendOptions` we use. We pass `verifierTarget: 'evm'`
 * (the keccak-oracle flavour) to BOTH prove and verify — it leaves `disableZk` at its default
 * `false`, so the proof stays fully zero-knowledge (value-hiding) while shrinking the wire proof.
 * We NEVER pass any `*-no-zk` flavour: those set `disableZk: true` and strip the masking, which
 * would make the private value recoverable in principle.
 */
type VerifierTarget = "evm" | "noir-recursive" | "starknet";
interface BbBackendOptions {
  verifierTarget?: VerifierTarget;
}
interface BbModule {
  Barretenberg: { new: (opts: { threads: number }) => Promise<unknown> };
  UltraHonkBackend: new (
    bytecode: string,
    api: unknown,
  ) => {
    generateProof(
      witness: Uint8Array,
      options?: BbBackendOptions,
    ): Promise<{ proof: Uint8Array; publicInputs: string[] }>;
    verifyProof(
      proof: { proof: Uint8Array; publicInputs: string[] },
      options?: BbBackendOptions,
    ): Promise<boolean>;
  };
}

/**
 * Prove and verify MUST be passed the SAME options or the in-tab self-verify rejects a valid
 * proof.
 *
 * ⚠️ `@aztec/bb.js` is pinned to a NIGHTLY (see package.json). The `verifierTarget` option
 * surface can shift between nightlies — RE-TEST this flavour (`verified === true`, proofs still
 * randomised, the operand still absent from the public inputs) on ANY bb.js bump before trusting
 * it. Never substitute a `*-no-zk` flavour.
 */
const PROOF_OPTIONS: BbBackendOptions = { verifierTarget: "evm" };

interface ProverContext {
  Noir: NoirModule["Noir"];
  backend: InstanceType<BbModule["UltraHonkBackend"]>;
  circuit: { bytecode: string };
  threads: number;
}

let proverPromise: Promise<ProverContext> | null = null;

/**
 * Cross-origin isolation gates bb.js multithreading (SharedArrayBuffer). Neither the hosted
 * GitHub-Pages target nor the Tauri webview sets COOP/COEP, so this is normally 1 — reported so
 * the panel can label the run honestly rather than implying a threaded prover.
 */
function maxThreads(): number {
  if (typeof window === "undefined") return 1;
  if (!window.crossOriginIsolated) return 1;
  return Math.min(navigator.hardwareConcurrency || 1, 8);
}

/**
 * The cold-start cost of the in-tab prover, paid once and shared: the dynamic-import of the
 * noir_js + bb.js chunks, the ACIR fetch, and the dominant `Barretenberg.new` WASM instantiate.
 * None of it depends on the chosen value, so {@link prewarmProver} kicks it off on panel mount
 * and {@link proveFilter} awaits the SAME promise — a click that lands early simply waits.
 */
async function loadProver(): Promise<ProverContext> {
  if (!proverPromise) {
    proverPromise = (async () => {
      const [{ Noir }, bb, circuit] = await Promise.all([
        import(/* webpackChunkName: "noir-js" */ "@noir-lang/noir_js") as Promise<NoirModule>,
        import(/* webpackChunkName: "bb-js" */ "@aztec/bb.js") as Promise<BbModule>,
        fetch(CIRCUIT_URL()).then((r) => {
          if (!r.ok) throw new Error(CIRCUIT_MISSING_MESSAGE);
          return r.json() as Promise<{ bytecode: string }>;
        }),
      ]);

      const threads = maxThreads();
      const api = await bb.Barretenberg.new({ threads });
      const backend = new bb.UltraHonkBackend(circuit.bytecode, api);
      return { Noir, backend, circuit, threads };
    })();
    proverPromise.catch(() => {
      proverPromise = null; // allow retry on a transient failure (fetch / instantiate)
    });
  }
  return proverPromise;
}

/**
 * Eagerly pre-warm the prover without blocking render. Safe to call on mount and repeatedly — it
 * shares the single {@link loadProver} promise, so the cold start happens at most once. Purely a
 * UX measure: it changes nothing about what is proved.
 */
export function prewarmProver(): Promise<unknown> {
  return loadProver();
}

export interface ProofRequest {
  /** The canonical digit string of the hidden store value (the private witness). */
  digits: string;
  op: OpCode;
  /** The FILTER's public constant. */
  bound: number;
}

export interface ProofResult {
  /** The disclosed verdict — the only value-derived bit revealed. */
  verdict: boolean;
  /** Proof size in bytes — opaque to the verifier; reveals nothing about the hidden value. */
  proofByteLength: number;
  /** The public inputs the verifier sees (the hidden value is NOT among them). */
  publicInputs: string[];
  /** Independent in-tab verification of the proof against the same circuit. */
  verified: boolean;
  proveMs: number;
  verifyMs: number;
  /** bb.js thread count actually used (1 on a non-cross-origin-isolated host). */
  threads: number;
}

/**
 * Generate AND verify a real UltraHonk proof that the hidden store value satisfies
 * `value <op> bound`, entirely in the tab.
 *
 * Throws when no committed term anchor ships for the value (the encoding comes from the native
 * encoder and cannot be derived in-tab), and propagates the witness-solve failure verbatim when
 * the circuit refuses the claim — both are honest outcomes, never a fabricated result.
 */
export async function proveFilter({ digits, op, bound }: ProofRequest): Promise<ProofResult> {
  const operandEnc = termAnchor(digits);
  if (!operandEnc) {
    throw new Error(`no committed term anchor ships for the value ${digits}`);
  }
  if (!Number.isSafeInteger(bound) || bound < 0) {
    throw new Error("the bound must be a non-negative integer (the circuit takes a u64)");
  }
  // The circuit asserts `result == expected`, so we publish the verdict the hidden value actually
  // satisfies. Claiming the other one makes the witness solve fail — that is the property the
  // panel demonstrates, not a bug to work around.
  const verdict = evaluateOp(Number(digits), op, bound);

  const { Noir, backend, circuit, threads } = await loadProver();

  const inputs = {
    challenge: "0x1", // per-presentation verifier nonce (fixed here; no replay context in-tab)
    operand_enc: operandEnc,
    op: String(op),
    bound: String(bound),
    expected: verdict,
    digits: digitBytes(digits), // PRIVATE — the exact value, never disclosed
  };

  const noir = new Noir(circuit);
  const { witness } = await noir.execute(inputs);

  const t0 = performance.now();
  const proof = await backend.generateProof(witness, PROOF_OPTIONS);
  const proveMs = performance.now() - t0;

  const t1 = performance.now();
  // Verify with the SAME options as prove (the keccak-oracle flavour) — a mismatch would make the
  // in-tab self-verify reject a perfectly valid proof.
  const verified = await backend.verifyProof(proof, PROOF_OPTIONS);
  const verifyMs = performance.now() - t1;

  return {
    verdict,
    proofByteLength: proof.proof.length,
    publicInputs: proof.publicInputs,
    verified,
    proveMs,
    verifyMs,
    threads,
  };
}
