// [OPUS-5] sq-ixc3.17 — REAL in-tab UltraHonk proving for the workbench ZK tool, ported
// from the site's flagship demo loader (`site/src/lib/zk-prover.ts`, sq-13rg / sq-8dx2) with
// the car-hire narrative cut. Same circuit, same prover, same flavour; the only differences
// are that the witness comes from the operator's LIVE workspace store (not a page fixture)
// and the marketing framing is gone.
//
// This is NOT a simulation. It runs the genuine sparq ZK circuit `zk/compose/filter_int_d2`
// (the ACIR JSON shipped at `/zk/filter_int_d2.json`, synced from `site/public/zk/` by
// `scripts/sync-wasm.mjs`) and proves it with `@aztec/bb.js` (UltraHonk — the same prover
// the native crate drives over its `bb` subprocess) entirely in the tab:
//
//   private witness: the selected store value's decimal digits
//   public inputs:   challenge, operand_enc (the committed term anchor), op (≥),
//                    bound, expected  — the value itself is NEVER public.
//
// The circuit (`sparq_zk_compose_core::filter_int`) rebuilds the canonical
// `"<digits>"^^xsd:integer` N-Triples token in-circuit, hashes it (blake3 blackbox), and
// asserts `h2(LITERAL, hs) == operand_enc` — binding the hidden value to the committed
// graph — then asserts the comparison verdict. A false claim is UNSATISFIABLE: the witness
// solve fails, so no proof can be produced. That is what the tool demonstrates live.
//
// HONESTY: the cryptographic mechanics here are real and run in the tab. The broader sparq
// ZK estate is research-track, internally reviewed only, and has NOT been externally
// audited — external accredited-cryptographer sign-off is pending (bead sq-qhy4). Do not
// treat a proof produced here as a production guarantee; the tool's honesty note says so.
//
// bb.js + noir_js are lazy-loaded via dynamic import (client-only) so the ~MB WASM never
// enters the workbench's first-load bundle, mirroring the lean-wasm engine loader.

import { withBasePath } from "@/lib/base-path";

/**
 * Field encodings of `"<value>"^^xsd:integer`, precomputed natively via
 * `sparq_zk_compose::build::encode_int_literal` (the SAME encoder the scan proof uses for
 * the operand column). Each is the `operand_enc` public input that binds the hidden value
 * to the committed graph.
 *
 * DRIFT GUARD: these are the same committed fixtures the site ships in
 * `site/src/lib/zk-prover.ts`, where they are pinned against the native encoder by
 * `crates/sparq-zk-compose/tests/site_age_enc_drift.rs`. That test does not yet cover THIS
 * copy — if a circuit / encoder bump changes the term encoding, it fails RED for the site
 * and the regenerated hex must be pasted into BOTH files, or the tool's in-tab proving
 * becomes unsatisfiable (the witness solve fails). Extending the drift test to this file is
 * tracked as a follow-up.
 */
const OPERAND_ENC: Record<number, string> = {
  24: "0x1c8a81ea95b253e105b99209deff1a4908be9568e588fbd89afea9f49f5f20cf",
  25: "0x2b5caeb2bbd290ab32434a9109030784c7faebadee7a9908d24dccb847910d1d",
  30: "0x132fa587351bf3f12fd3cbed64d5526f28791099d1d40870f94595873c78fa72",
  42: "0x1a4aa7fd962d0004ac2294cc98471ea1ebfdad74a8f702e89fedf83f92d0f97b",
};

/**
 * The store values that can be proven in-tab today: those for which a committed
 * `operand_enc` is shipped. All are two-digit, matching the `filter_int_d2` circuit member.
 * Everything else is declined with a reason (see lib/zk-witness.ts `declineReason`) rather
 * than attempted and silently failed.
 */
export const COMMITTED_VALUES: ReadonlySet<number> = new Set(
  Object.keys(OPERAND_ENC).map(Number),
);

/**
 * The comparison bound the shipped artifact is validated against. It IS a public input of
 * the circuit, so the member can prove other bounds — but only this one has been exercised
 * against the shipped ACIR, so the tool pins it rather than offering an unvalidated knob.
 */
export const FILTER_BOUND = 25;

/** The `filter_int` op code for `≥` (OP_GE) in `sparq_zk_compose_core`. */
const OP_GE = "3";

/** What a completed in-tab prove + verify produced. */
export interface ProofResult {
  /** The proven claim about `value ≥ FILTER_BOUND` — the only value-derived bit revealed.
   *  A proof exists only when this is the TRUE verdict for the hidden value. */
  verdict: boolean;
  /** The proof bytes' length — opaque to the verifier; reveals nothing about the value. */
  proofByteLength: number;
  /** The public inputs the verifier sees (the hidden value is NOT among them). */
  publicInputs: string[];
  /** Independent in-tab verification of the proof against the same circuit. */
  verified: boolean;
  /** Measured wall-clock of THIS run (performance.now delta, ms) — non-canonical. */
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

/** Subset of `@aztec/bb.js`'s `UltraHonkBackendOptions` we use. We pass
 *  `verifierTarget: 'evm'` (the keccak-oracle flavour) to BOTH prove and verify — it leaves
 *  `disableZk` at its default `false`, so the proof stays FULLY zero-knowledge (the witness
 *  stays hidden) while shrinking the wire proof. We NEVER pass any `*-no-zk` flavour: those
 *  set `disableZk: true` and strip the masking, which would make the private value
 *  recoverable in principle. */
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
 * The UltraHonk proving/verifying flavour. `verifierTarget: 'evm'` selects the keccak
 * transcript oracle and yields a materially smaller proof at no measured time cost, while
 * keeping `disableZk` false — i.e. the proof is STILL zero-knowledge and the private value
 * is still hidden. Prove and verify MUST be passed the SAME options or the in-tab
 * self-verify rejects a perfectly valid proof.
 *
 * ⚠️ `@aztec/bb.js` is pinned to a NIGHTLY (see package.json). The `verifierTarget` option
 * surface can shift between nightlies — RE-TEST this flavour on ANY bb.js version bump
 * before trusting it. Never substitute a `*-no-zk` flavour.
 */
const PROOF_OPTIONS: BbBackendOptions = { verifierTarget: "evm" };

/**
 * The cold-start cost of the in-tab prover, paid once and shared. Three expensive things
 * must happen before the first proof, none of which depend on the chosen witness:
 *   1. dynamic-import the `@noir-lang/noir_js` + `@aztec/bb.js` chunks (the ~MB WASM glue,
 *      deliberately kept out of the workbench's first-load bundle);
 *   2. fetch + parse the committed ACIR artifact (`/zk/filter_int_d2.json`);
 *   3. instantiate the Barretenberg WASM backend (the dominant cold cost) and build the
 *      `UltraHonkBackend` over the circuit bytecode.
 *
 * {@link prewarmProver} kicks this off in the background when the tool mounts, so the first
 * "Prove" click pays none of it; {@link proveFilterGe} awaits the SAME promise as a safety
 * net, so a click that lands before pre-warm finishes simply waits.
 */
interface ProverContext {
  Noir: NoirModule["Noir"];
  backend: InstanceType<BbModule["UltraHonkBackend"]>;
  circuit: { bytecode: string };
  threads: number;
}

let proverPromise: Promise<ProverContext> | null = null;

async function loadProver(): Promise<ProverContext> {
  if (!proverPromise) {
    proverPromise = (async () => {
      const [{ Noir }, bb, circuit] = await Promise.all([
        import(/* webpackChunkName: "noir-js" */ "@noir-lang/noir_js") as Promise<NoirModule>,
        import(/* webpackChunkName: "bb-js" */ "@aztec/bb.js") as Promise<BbModule>,
        fetch(withBasePath("/zk/filter_int_d2.json")).then((r) => {
          if (!r.ok) throw new Error(`ZK circuit fetch failed: ${r.status}`);
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
 * Eagerly pre-warm the in-tab prover without blocking render. Safe to call on tool mount and
 * to call repeatedly — it shares the single {@link loadProver} promise, so the cold start
 * happens at most once. Resolves when the prover is ready, or rejects on a load failure
 * (which resets the cache so a later click retries). A UX measure only: it does not change
 * what is proved, nor any property of the proof.
 */
export function prewarmProver(): Promise<unknown> {
  return loadProver();
}

/** Cross-origin isolation gates bb.js multithreading (SharedArrayBuffer). Neither the
 *  hosted static export nor the Tauri webview sets COOP/COEP, so this is normally 1. */
function maxThreads(): number {
  if (typeof window === "undefined") return 1;
  if (!window.crossOriginIsolated) return 1;
  return Math.min(navigator.hardwareConcurrency || 1, 8);
}

/** The two ASCII decimal digit-bytes of a two-digit value (the private witness). */
function valueDigits(value: number): string[] {
  const s = String(value).padStart(2, "0");
  return [String(s.charCodeAt(0)), String(s.charCodeAt(1))];
}

/**
 * Generate AND verify a real UltraHonk proof that the hidden `value` satisfies
 * `(value ≥ FILTER_BOUND) == claim`, entirely in the tab. Throws if `value` has no
 * committed `operand_enc` (the panel only offers rows that do — see lib/zk-witness.ts).
 *
 * `claim` defaults to the TRUE verdict. Passing the opposite is the tool's "try to forge
 * it" affordance: the circuit is then unsatisfiable, `execute` throws, and no proof exists —
 * the honest "you cannot prove a verdict you do not have" outcome, demonstrated rather than
 * asserted.
 */
export async function proveFilterGe(
  value: number,
  claim: boolean = value >= FILTER_BOUND,
): Promise<ProofResult> {
  const operandEnc = OPERAND_ENC[value];
  if (!operandEnc) {
    throw new Error(`no committed operand_enc fixture for the value ${value}`);
  }
  const verdict = claim;

  // Reuse the shared, pre-warmed prover context. If pre-warm has not finished this awaits
  // the same in-flight promise — never a second cold start.
  const { Noir, backend, circuit, threads } = await loadProver();

  const inputs = {
    challenge: "0x1", // per-presentation verifier nonce (fixed here — one local tab)
    operand_enc: operandEnc,
    op: OP_GE,
    bound: String(FILTER_BOUND),
    expected: verdict,
    digits: valueDigits(value), // PRIVATE — the store value, never disclosed
  };

  const noir = new Noir(circuit);
  const { witness } = await noir.execute(inputs);

  const t0 = performance.now();
  const proof = await backend.generateProof(witness, PROOF_OPTIONS);
  const proveMs = performance.now() - t0;

  const t1 = performance.now();
  // Verify with the SAME options as prove (the keccak-oracle flavour) — a mismatch would
  // make the in-tab self-verify reject a perfectly valid proof.
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
