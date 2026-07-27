// [OPUS-4.8] sq-8dx2 — capture a REAL ProofManifest fixture for the ZK car-hire
// flagship's fallback "captured manifest + live in-tab verify" path.
//
// This is a BUILD HELPER, not a runtime asset and not part of the static export.
// It runs the SAME prover the browser runs (`@noir-lang/noir_js` + `@aztec/bb.js`,
// UltraHonk, `verifierTarget: 'evm'`) against the SAME committed circuit the page
// ships (`public/zk/filter_int_d2.json`), produces a genuine UltraHonk proof of the
// age-gate sub-relation (`age >= 25` over a hidden age, age never disclosed), and
// writes it as `public/zk/car-hire-manifest.json`.
//
// WHY a captured manifest at all: full live in-tab PROVING of the whole
// cross-credential family (2x scan + 2x hidden_issuer + revoke_unset + join_eq +
// holder_pok) needs native witness generation (Poseidon commitments, Schnorr
// signatures, Merkle roots) that is not feasible in the browser. So the flagship's
// fallback ships ONE genuinely captured, genuinely verifiable sub-proof — the
// age-gate — and the browser does a REAL bb.js `verifyProof` over it in-tab. The
// other family members are described honestly as composed-natively-but-not-bundled,
// never as live in-browser proofs. Nothing here is fabricated: the proof bytes are a
// real UltraHonk transcript, and a tamper of any byte makes the in-tab verify reject.
//
// HONESTY: the captured proof is research-grade. The sparq ZK estate is NOT
// externally audited (audit obligation: bead sq-qhy4 / gap CR-G1). A "verified"
// result is a research demonstration, not a production cryptographic guarantee.
//
// Re-run after any circuit / bb.js / noir_js bump:  node scripts/capture-zk-manifest.mjs
import { readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const circuitPath = join(here, "..", "public", "zk", "filter_int_d2.json");
const outPath = join(here, "..", "public", "zk", "car-hire-manifest.json");

// MUST mirror src/lib/zk-prover.ts exactly (same circuit, inputs, proof flavour) so
// the captured proof verifies under the SAME bb.js path the live page uses.
const PROOF_OPTIONS = { verifierTarget: "evm" };
const CAPTURED_AGE = 30;
const AGE_THRESHOLD = 25;
// `operand_enc` for `"30"^^xsd:integer`, the SAME committed anchor the page ships in
// AGE_OPERAND_ENC[30] (precomputed natively via encode_int_literal). It binds the
// hidden age digits to the committed credential. DRIFT GUARD (sq-1s2.4): pinned
// against the native encoder by crates/sparq-zk-compose/tests/site_age_enc_drift.rs —
// on a circuit/encoder bump that test fails RED with the regenerated value to paste.
const OPERAND_ENC =
  "0x132fa587351bf3f12fd3cbed64d5526f28791099d1d40870f94595873c78fa72";

function ageDigits(age) {
  const s = String(age).padStart(2, "0");
  return [String(s.charCodeAt(0)), String(s.charCodeAt(1))];
}

async function main() {
  const circuit = JSON.parse(await readFile(circuitPath, "utf8"));
  const { Noir } = await import("@noir-lang/noir_js");
  const bb = await import("@aztec/bb.js");

  const api = await bb.Barretenberg.new({ threads: 1 });
  const backend = new bb.UltraHonkBackend(circuit.bytecode, api);

  const inputs = {
    challenge: "0x1",
    operand_enc: OPERAND_ENC,
    op: "3", // OP_GE
    bound: String(AGE_THRESHOLD),
    expected: CAPTURED_AGE >= AGE_THRESHOLD,
    digits: ageDigits(CAPTURED_AGE), // PRIVATE — the exact age, never disclosed
  };

  const noir = new Noir(circuit);
  const { witness } = await noir.execute(inputs);
  const { proof, publicInputs } = await backend.generateProof(witness, PROOF_OPTIONS);
  const verified = await backend.verifyProof({ proof, publicInputs }, PROOF_OPTIONS);
  if (!verified) {
    throw new Error("captured proof did not self-verify — refusing to write a bad fixture");
  }
  // Sanity: the hidden age must NOT appear in the public inputs.
  const leaked = publicInputs.some((pi) => BigInt(pi).toString() === String(CAPTURED_AGE));
  if (leaked) {
    throw new Error("captured public inputs appear to leak the hidden age — aborting");
  }

  const manifest = {
    // The captured-manifest fallback for /showcase/zk-car-hire. The bundled sub-proof
    // is a REAL UltraHonk proof of the age-gate; the browser re-verifies it live in-tab.
    // Other family members are composed natively, NOT bundled as proofs here. The shape
    // MUST mirror `sparq_zk_compose::capture::CapturedCarHireManifest` (a `subProofs`
    // ARRAY) so the native crate seam and this script emit an identical schema.
    type: "urn:sparq:zk:ProofManifest",
    note:
      "Research-grade, NOT externally audited (bead sq-qhy4). One genuinely captured + " +
      "in-tab-verifiable sub-proof: the age-gate FILTER (age >= 25). The hidden age is " +
      "never in the public inputs. Captured by site/scripts/capture-zk-manifest.mjs.",
    capturedAt: new Date().toISOString().slice(0, 10),
    subProofs: [
      {
        member: "filter_int_d2",
        relation: "age >= 25 over a hidden integer age",
        // proof bytes as a plain array of u8 (JSON-portable); rehydrated to Uint8Array
        // in the browser before bb.js verifyProof.
        proof: Array.from(proof),
        publicInputs,
      },
    ],
  };

  // Pretty-print the manifest, but keep the (long) proof byte array on a single line so
  // the fixture stays a few KB of readable JSON rather than 8k+ one-int-per-line rows.
  const pretty = JSON.stringify(manifest, null, 2).replace(
    /"proof": \[[\s\S]*?\]/,
    `"proof": [${manifest.subProofs[0].proof.join(",")}]`,
  );
  await writeFile(outPath, `${pretty}\n`, "utf8");
  console.log(
    `[capture-zk-manifest] wrote ${outPath} — proof ${proof.length} B, ` +
      `${publicInputs.length} public inputs, self-verify ${verified}.`,
  );
  // bb keeps a wasm worker pool alive; force exit so the helper terminates cleanly.
  process.exit(0);
}

main().catch((e) => {
  console.error("[capture-zk-manifest] FAILED:", e);
  process.exit(1);
});
