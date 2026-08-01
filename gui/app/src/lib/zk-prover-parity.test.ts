// [OPUS-5] sq-ixc3.17 — the drift gate on the GUI's in-tab prover configuration.
//
// WHY THIS FILE EXISTS. `lib/zk-prover.ts` makes load-bearing claims: that a real UltraHonk
// proof is produced in the tab over the committed `filter_int_d2` ACIR, that the honest
// verdict verifies, that the opposite claim is refused by the circuit, and that the store
// value never appears among the public inputs. None of that is established by the pure
// witness-selection unit tests, and re-proving it here is not possible: bb.js proving needs
// the WASM prover and belongs in a browser lane, not in `node --test`.
//
// WHAT IS ACTUALLY GATED. The GUI's prover is not an independent implementation — it is the
// SAME configuration as the site's car-hire prover, which IS proven and verified end to end
// in a real headless-Chromium lane (`site/e2e/zk-prewarm.spec.ts`, run by the `site-e2e`
// workflow; it asserts `data-proof-verified="true"` on a freshly generated proof). Same ACIR
// artifact — `gui/app/scripts/sync-wasm.mjs` COPIES the one committed
// `site/public/zk/filter_int_d2.json`; same prover versions — one repo-root lockfile pins
// `@aztec/bb.js` / `@noir-lang/noir_js` for both packages; same public inputs, same private
// digit encoding, same UltraHonk flavour. This file pins every one of those to the site's,
// so the moment the GUI's copy drifts from the configuration that lane actually exercises,
// CI goes red instead of the tool quietly becoming unprovable.
//
// Separately, the circuit-level half of the refusal claim is gated natively:
// `zk/compose/compose_core/src/tests.nr::filter_int_rejects_lying_verdict` is a
// `#[test(should_fail_with = "filter verdict mismatch")]` — which is also the exact assertion
// label `lib/zk-witness.ts::isUnsatisfiable` keys on.
//
// WHAT IS NOT GATED — say it plainly. This is an EQUIVALENCE argument, not a proof run. No
// GUI-native browser test yet drives the tool's own Prove button through bb.js, so the
// wiring BETWEEN the panel and the prover (the two prove buttons, the refusal branch, the
// public-inputs rendering) rests on code review, not on a gate. A Playwright spec in the
// `gui-mock-ipc` lane is the missing piece; it is tracked as follow-up work.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** Repo root: this file sits at `gui/app/src/lib/`, so four levels up. */
const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "..");
const read = (rel: string): string => readFileSync(join(REPO_ROOT, rel), "utf8");

const GUI_PROVER = read("gui/app/src/lib/zk-prover.ts");
const SITE_PROVER = read("site/src/lib/zk-prover.ts");

/**
 * The `operand_enc` fixture table out of a prover module. These hex field elements ARE the
 * binding between the hidden value and the committed graph: a single stale digit makes the
 * witness solve fail and the tool unprovable, so they must not diverge between the two copies.
 */
function operandEncFixtures(source: string, recordName: string): Record<string, string> {
  const start = source.indexOf(`const ${recordName}: Record<number, string> = {`);
  assert.notEqual(start, -1, `no ${recordName} fixture record found — did the shape change?`);
  const body = source.slice(start, source.indexOf("};", start));
  const fixtures: Record<string, string> = {};
  for (const [, value, hex] of body.matchAll(/(\d+):\s*"(0x[0-9a-f]+)"/g)) fixtures[value] = hex;
  assert.ok(Object.keys(fixtures).length > 0, `${recordName} parsed as empty`);
  return fixtures;
}

/** The single value captured by `pattern` in `source`, asserted to be present exactly once. */
function only(source: string, pattern: RegExp, what: string): string {
  const hits = [...source.matchAll(new RegExp(pattern, "g"))];
  assert.equal(hits.length, 1, `expected exactly one ${what}, found ${hits.length}`);
  return hits[0][1];
}

test("the GUI's committed operand_enc fixtures are the site's browser-exercised set", () => {
  // Same term commitments → the same values are provable in-tab, against the same circuit.
  assert.deepEqual(
    operandEncFixtures(GUI_PROVER, "OPERAND_ENC"),
    operandEncFixtures(SITE_PROVER, "AGE_OPERAND_ENC"),
  );
});

test("the GUI's public inputs and private witness encoding match the site's", () => {
  // The comparison bound. The site proves `age >= 25`; the GUI proves `witness >= 25` over
  // the same circuit member, so the same public-input vector is exercised.
  assert.equal(
    only(GUI_PROVER, /export const FILTER_BOUND = (\d+);/, "GUI FILTER_BOUND"),
    only(SITE_PROVER, /export const AGE_THRESHOLD = (\d+);/, "site AGE_THRESHOLD"),
  );
  // The op selector: OP_GE == 3 in sparq_zk_compose_core::filter_int.
  assert.equal(only(GUI_PROVER, /const OP_GE = "(\d)";/, "GUI op code"), "3");
  assert.equal(only(SITE_PROVER, /op: "(\d)", \/\/ OP_GE/, "site op code"), "3");

  for (const [name, source] of [
    ["GUI", GUI_PROVER],
    ["site", SITE_PROVER],
  ] as const) {
    assert.ok(source.includes('challenge: "0x1"'), `${name}: challenge nonce changed`);
    // The PRIVATE witness encoding: two ASCII decimal digit-bytes, as the D=2 member wants.
    assert.ok(source.includes('padStart(2, "0")'), `${name}: digit padding changed`);
    assert.ok(
      source.includes("String(s.charCodeAt(0)), String(s.charCodeAt(1))"),
      `${name}: digit byte encoding changed`,
    );
  }
});

test("both provers use the same zero-knowledge UltraHonk flavour", () => {
  // `evm` selects the keccak transcript oracle and leaves `disableZk` false. Any `*-no-zk`
  // flavour would strip the masking and make the private value recoverable in principle —
  // asserting the exact string is what keeps that from being changed quietly on one side.
  const flavour = /const PROOF_OPTIONS: BbBackendOptions = \{ verifierTarget: "([\w-]+)" \};/;
  assert.equal(only(GUI_PROVER, flavour, "GUI proving flavour"), "evm");
  assert.equal(only(SITE_PROVER, flavour, "site proving flavour"), "evm");
});

test("both packages pin the same bb.js and noir_js versions", () => {
  // The equivalence argument only holds while the GUI runs the prover versions the site's
  // browser lane actually proved with; a one-sided nightly bump must fail here first.
  const pins = (pkg: string) => {
    const deps = (JSON.parse(read(pkg)) as { dependencies: Record<string, string> }).dependencies;
    return { bb: deps["@aztec/bb.js"], noir: deps["@noir-lang/noir_js"] };
  };
  const gui = pins("gui/app/package.json");
  assert.deepEqual(gui, pins("site/package.json"));
  assert.ok(gui.bb && gui.noir, "the GUI must pin both prover packages explicitly");
});

test("the GUI proves against the site's single committed ACIR artifact", () => {
  // The GUI ships no ACIR of its own (public/zk is gitignored); sync-wasm copies the one
  // committed artifact, so "the copied ACIR" is the same bytes the site lane proves against.
  const sync = read("gui/app/scripts/sync-wasm.mjs");
  assert.match(sync, /const zkFiles = \["filter_int_d2\.json"\];/);
  assert.match(sync, /const zkSrc = join\(here, "\.\.", "\.\.", "\.\.", "site", "public", "zk"\)/);
  assert.ok(GUI_PROVER.includes("/zk/filter_int_d2.json"), "the GUI fetches another circuit");
  // The committed artifact is present and non-trivial (a truncated copy must not pass).
  assert.ok(read("site/public/zk/filter_int_d2.json").length > 1000, "ACIR artifact missing");
});
