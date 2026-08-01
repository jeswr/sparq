// [SONNET-4.6] PR #3588 review round 1 — CONTRACT test pinning the captured car-hire
// manifest fixture (site/public/zk/car-hire-manifest.json) against the schema the real
// consumer (src/lib/zk-prover.ts `CapturedManifest` / `verifyCapturedManifest`) reads
// AND the schema the native crate seam (`sparq_zk_compose::capture::CapturedCarHireManifest`)
// serializes. The prior Rust unit test only substring-checked emitted keys, so it could not
// catch the singular-vs-plural incompatibility this test now guards. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const fixturePath = join(here, "..", "public", "zk", "car-hire-manifest.json");
const AGE_GATE_MEMBER = "filter_int_d2"; // must match zk-prover.ts

function loadFixture() {
  return JSON.parse(readFileSync(fixturePath, "utf8"));
}

test("fixture has the plural `subProofs` schema the consumer + crate agree on", () => {
  const m = loadFixture();
  // The exact top-level key set `sparq_zk_compose::capture::CapturedCarHireManifest`
  // serializes and `CapturedManifest` in zk-prover.ts declares.
  assert.deepEqual(
    Object.keys(m).sort(),
    ["capturedAt", "note", "subProofs", "type"],
    "top-level keys must be exactly {type,note,capturedAt,subProofs}",
  );
  // Guard against a regression back to the OLD singular shape the consumer can't read.
  assert.equal("subProof" in m, false, "legacy singular `subProof` key must be gone");
  assert.equal("circuit" in m, false, "legacy `circuit` key must be gone");

  assert.equal(typeof m.type, "string");
  assert.equal(typeof m.note, "string");
  assert.equal(typeof m.capturedAt, "string");
  assert.ok(Array.isArray(m.subProofs), "subProofs is an array");
  assert.ok(m.subProofs.length >= 1, "at least one bundled sub-proof");
});

test("each sub-proof matches the CapturedSubProof field contract", () => {
  const m = loadFixture();
  for (const sp of m.subProofs) {
    assert.deepEqual(
      Object.keys(sp).sort(),
      ["member", "proof", "publicInputs", "relation"],
      "sub-proof keys must be exactly {member,relation,proof,publicInputs}",
    );
    assert.equal(typeof sp.member, "string");
    assert.equal(typeof sp.relation, "string");
    assert.ok(Array.isArray(sp.proof) && sp.proof.length > 0, "proof is a non-empty number[]");
    assert.ok(
      sp.proof.every((b) => Number.isInteger(b) && b >= 0 && b <= 255),
      "proof bytes are u8",
    );
    assert.ok(Array.isArray(sp.publicInputs) && sp.publicInputs.length > 0, "publicInputs present");
    assert.ok(
      sp.publicInputs.every((h) => /^0x[0-9a-f]{64}$/.test(h)),
      "each public input is 0x + 64 lowercase nibbles (one 32-byte field word)",
    );
  }
});

test("the consumer's age-gate selection resolves a sub-proof (fixture is verifiable)", () => {
  const m = loadFixture();
  // Mirror verifyCapturedManifest's selection: age-gate member, else the first.
  const sp = m.subProofs.find((p) => p.member === AGE_GATE_MEMBER) ?? m.subProofs[0];
  assert.ok(sp, "selection must resolve a sub-proof the in-tab verifier can run");
  assert.equal(sp.member, AGE_GATE_MEMBER, "the shipped fixture's verifiable member is the age-gate");
});
