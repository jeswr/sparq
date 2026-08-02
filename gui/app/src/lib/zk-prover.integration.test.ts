// [SONNET-4.6] sq-ixc3.17 — the REGRESSION test for the ZK tool's load-bearing path: a real
// UltraHonk proof, generated and verified through `proveFilter` itself, against the SHIPPED ACIR
// and the pinned `@noir-lang/noir_js` + `@aztec/bb.js` versions.
//
// It exists because everything else about the tool is cheap to test and this is not: the pinned
// bb.js is a NIGHTLY, so an ACIR/package bump, a wrong Noir input encoding, a `verifierTarget`
// mismatch or an unexpected public-input disclosure would otherwise only ever surface by hand in
// a browser tab. What it pins:
//
//   * a proof over a store-shaped operand verifies (`verified === true`);
//   * the public-input contract — the committed anchor, the operator, the bound and the verdict
//     are disclosed, and the hidden value's field encoding is NOT;
//   * a bundle whose public claim has been altered is REJECTED, which is the property every
//     "the circuit refuses a false claim" line of the panel's copy rests on.
//
// OPT-IN TARGET (`npm run test:zk-proof`), NOT part of `npm run test:unit`: it instantiates the
// bb.js WASM prover, so it needs the installed `@aztec/bb.js`/`@noir-lang/noir_js` dependencies
// and takes seconds rather than milliseconds. The pure half lives in `zk-filter.test.ts`.
//
// The ACIR is read from the repo's ONE committed copy (site/public/zk/), the same artifact
// `scripts/sync-wasm.mjs` copies into `public/zk/` at build time — so this test does not depend
// on a build having run. `fetch` is stubbed to serve it, since `proveFilter` fetches the circuit
// as a static asset in the browser.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { CIRCUIT_MEMBER, PUBLIC_INPUT_KEYS, buildCircuitInputs } from "./zk-filter.js";
import { proveFilter, verifyProofBundle } from "./zk-prover.js";

const ACIR_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  `../../../../site/public/zk/${CIRCUIT_MEMBER}.json`,
);

const realFetch = globalThis.fetch;
globalThis.fetch = (async (input: Parameters<typeof realFetch>[0]) => {
  const url = String(input);
  if (!url.endsWith(`/zk/${CIRCUIT_MEMBER}.json`)) {
    throw new Error(`unexpected fetch in the prover integration test: ${url}`);
  }
  return new Response(await readFile(ACIR_PATH, "utf8"), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}) as typeof realFetch;

/** A public input's value, normalised past hex padding/casing. */
function field(n: number | bigint | string): bigint {
  return BigInt(n);
}

/** Whether some public input equals this field element (order/padding independent). */
function discloses(publicInputs: string[], value: bigint): boolean {
  return publicInputs.some((pi) => BigInt(pi) === value);
}

// The row the workbench's seeded sample graph returns for Alice: `"30"^^xsd:integer`, claimed
// against a bound it satisfies. 30 is deliberately different from every public input's value, so
// "the value is not disclosed" cannot pass by collision.
const REQUEST = { digits: "30", op: 3, bound: 25 } as const;

test("proveFilter produces a proof this build verifies", async () => {
  const result = await proveFilter(REQUEST);
  assert.equal(result.verdict, true, "30 ≥ 25");
  assert.equal(result.verified, true, "the in-tab self-verify must accept its own proof");
  assert.ok(result.bundle.proof.length > 0, "a proof with no bytes is not a proof");
  assert.ok(
    result.bundle.publicInputs.length >= PUBLIC_INPUT_KEYS.length,
    `expected at least ${PUBLIC_INPUT_KEYS.length} public inputs, got ${result.bundle.publicInputs.length}`,
  );
});

test("the public inputs disclose the claim and NOT the hidden value", async () => {
  const { bundle } = await proveFilter(REQUEST);
  const inputs = buildCircuitInputs(REQUEST);
  const pi = bundle.publicInputs;

  assert.ok(discloses(pi, field(inputs.operand_enc)), "operand_enc must be public");
  assert.ok(discloses(pi, field(REQUEST.op)), "the operator must be public");
  assert.ok(discloses(pi, field(REQUEST.bound)), "the bound must be public");
  // The verdict `true` and the fixed challenge `0x1` are the same field element, so they can only
  // be told apart by count: BOTH must be there.
  assert.ok(
    pi.filter((v) => field(v) === 1n).length >= 2,
    "the challenge and the disclosed verdict must both be public",
  );

  // The witness itself must never appear as a public input.
  assert.equal(
    discloses(pi, field(Number(REQUEST.digits))),
    false,
    "the hidden value leaked into the public inputs",
  );
});

test("a bundle whose public claim is altered is REJECTED", async () => {
  const { bundle } = await proveFilter(REQUEST);
  assert.equal(await verifyProofBundle(bundle), true, "the untouched bundle must verify");

  // Re-publish the same proof against a weaker bound. Nothing about the proof bytes changes, so
  // only the verifier's public-input binding can catch it — which is exactly what is under test.
  const weaker = `0x${(REQUEST.bound - 1).toString(16).padStart(64, "0")}`;
  const forged = bundle.publicInputs.map((v) => (field(v) === field(REQUEST.bound) ? weaker : v));
  assert.notDeepEqual(
    forged,
    bundle.publicInputs,
    "the bound was not found among the public inputs",
  );
  // A throw counts as a rejection: either way the verifier did not accept the altered claim. The
  // untouched bundle verifying above is what rules out a vacuous pass here.
  const accepted = await verifyProofBundle({
    proof: bundle.proof,
    publicInputs: forged,
  }).catch(() => false);
  assert.equal(accepted, false, "a mutated public claim must not verify");
});
