// [GPT-5] sq-ixc3.17 — invocation-only UltraHonk prover extracted from the site's live demo.
// The proof mechanics are real, but the research-track ZK estate is not externally audited.
import { basePath } from "@/lib/base-path";
const OPERAND_ENCODING: Record<number, string> = {
  24: "0x1c8a81ea95b253e105b99209deff1a4908be9568e588fbd89afea9f49f5f20cf",
  25: "0x2b5caeb2bbd290ab32434a9109030784c7faebadee7a9908d24dccb847910d1d",
  30: "0x132fa587351bf3f12fd3cbed64d5526f28791099d1d40870f94595873c78fa72",
  42: "0x1a4aa7fd962d0004ac2294cc98471ea1ebfdad74a8f702e89fedf83f92d0f97b",
};

export const PROVABLE_AGES = Object.keys(OPERAND_ENCODING).map(Number);
export const AGE_THRESHOLD = 25;

export interface ProofResult {
  eligible: boolean;
  verified: boolean;
  proofBytes: number;
  publicInputs: string[];
  proveMs: number;
  verifyMs: number;
}

interface NoirModule {
  Noir: new (circuit: unknown) => {
    execute(inputs: Record<string, unknown>): Promise<{ witness: Uint8Array }>;
  };
}

interface BbModule {
  Barretenberg: { new: (options: { threads: number }) => Promise<unknown> };
  UltraHonkBackend: new (
    bytecode: string,
    api: unknown,
  ) => {
    generateProof(
      witness: Uint8Array,
      options: { verifierTarget: "evm" },
    ): Promise<{ proof: Uint8Array; publicInputs: string[] }>;
    verifyProof(
      proof: { proof: Uint8Array; publicInputs: string[] },
      options: { verifierTarget: "evm" },
    ): Promise<boolean>;
  };
}

export async function proveAge(age: number): Promise<ProofResult> {
  const operandEnc = OPERAND_ENCODING[age];
  if (!operandEnc) throw new Error(`No committed circuit input is available for age ${age}`);

  // Heavy optional dependencies cross this literal import boundary only after the user clicks.
  const [{ Noir }, bb, circuit] = await Promise.all([
    import("@noir-lang/noir_js") as Promise<NoirModule>,
    import("@aztec/bb.js") as Promise<BbModule>,
    fetch(`${basePath()}/zk/filter_int_d2.json`).then((response) => {
      if (!response.ok) throw new Error(`Circuit fetch failed (${response.status})`);
      return response.json() as Promise<{ bytecode: string }>;
    }),
  ]);
  const api = await bb.Barretenberg.new({ threads: 1 });
  const backend = new bb.UltraHonkBackend(circuit.bytecode, api);
  const eligible = age >= AGE_THRESHOLD;
  const digits = String(age).padStart(2, "0").split("").map((digit) => String(digit.charCodeAt(0)));
  const noir = new Noir(circuit);
  const { witness } = await noir.execute({
    challenge: "0x1",
    operand_enc: operandEnc,
    op: "3",
    bound: String(AGE_THRESHOLD),
    expected: eligible,
    digits,
  });
  const options = { verifierTarget: "evm" as const };
  const proveStart = performance.now();
  const proof = await backend.generateProof(witness, options);
  const proveMs = performance.now() - proveStart;
  const verifyStart = performance.now();
  const verified = await backend.verifyProof(proof, options);
  return {
    eligible,
    verified,
    proofBytes: proof.proof.length,
    publicInputs: proof.publicInputs,
    proveMs,
    verifyMs: performance.now() - verifyStart,
  };
}
