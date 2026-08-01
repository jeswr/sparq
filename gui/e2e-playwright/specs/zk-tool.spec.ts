// [OPUS-5] sq-ixc3.17 — the ZK tool's LOAD-BEARING browser gate.
//
// WHY THIS SPEC EXISTS. `lib/zk-prover-parity.test.ts` pins the GUI prover's *configuration*
// to the site's browser-exercised one (same ACIR, same pins, same public inputs, same
// UltraHonk flavour) — but a configuration match is an equivalence ARGUMENT, not a proof
// run. It cannot see any of the GUI-specific machinery the panel actually depends on: the
// lazy `import()` of `@noir-lang/noir_js` + `@aztec/bb.js` surviving the Next STATIC EXPORT
// (the site's own ZK lane runs against `next dev`, so the exported-bundle path is exercised
// nowhere else), the `withBasePath("/zk/filter_int_d2.json")` fetch of the synced artifact,
// `Barretenberg.new`, the Noir input wiring, the witness solve, generateProof/verifyProof,
// the public-input rendering, and the panel's refusal branch. This spec drives every one of
// them through the real UI, so the tool's "working" tier is earned by a gate rather than by
// code review.
//
// WHAT IT ASSERTS, end to end in headless Chromium against `gui/app/out`:
//   1. the prover cold start reaches `ready` — i.e. the WASM glue loaded and the committed
//      ACIR was fetched (a missing/404 artifact or an export-broken bundle lands on `error`);
//   2. a SELECT over the LIVE seeded store yields candidate witnesses, with an uncommitted
//      value DECLINED and its reason shown rather than silently dropped;
//   3. an honest claim over a real store value produces a proof that VERIFIES in-tab;
//   4. the store value is NOT among the proof's public inputs — the privacy property, read
//      off the vector the verifier actually receives;
//   5. the OPPOSITE claim is refused through the narrowly-classified refusal branch
//      (`lib/zk-witness.ts::isUnsatisfiable`, keyed on the circuit's own `filter verdict
//      mismatch` label) and NOT through the generic prover-error branch.
//
// The store fixture is `data/sample-graph.ts` (ages 30, 25, 41, 19), seeded into the live
// wasm Store on engine warm. 30 has a committed `operand_enc`; 41 does not.
//
// HONESTY. A green run here is evidence that the panel's cryptographic machinery genuinely
// runs — it is NOT a soundness result. sparq's ZK estate is research-track and has had no
// external accredited-cryptographer review (bead sq-qhy4).
//
// Stable selectors (declared E2E hooks, all in components/workbench/zk-tool.tsx):
//   [data-tool="zk"]            — the rail entry
//   [data-zk-prover]            — the cold-start pill (warming|ready|error)
//   [data-result-kind="zk"]     — the candidates + proof pane
//   [data-zk-candidate="<n>"]   — a candidate row, keyed by its integer value
//   [data-zk-provable]          — whether that row can be proven in-tab
//   [data-zk-proof-verified]    — a finished proof + its in-tab verify result
//   [data-zk-public-inputs]     — the public-input list (attribute value = its length)
//   [data-zk-public-input]      — one public input, verbatim
//   [data-zk-refused]           — the circuit's refusal of a false claim
//   [data-result-kind="error"]  — the generic prover-error <pre>
//
// Determinism rules: NO waitForTimeout; web-first assertions only. No wall-clock threshold is
// asserted — only outcomes (the timeouts below are generous CEILINGS, not measurements).
//
// WHY THIS SPEC OPTS OUT OF THE HERMETIC NETWORK BLOCK (support/fixtures.ts). Every other spec
// in this lane runs under a blanket `context.route("**/*")` that aborts anything off
// 127.0.0.1. The in-tab bb.js prover is the ONE runtime in this repo already known to be
// incompatible with that interception: the site's own bb.js spec has carried
// `test.use({ hermeticNetwork: false })` since the e2e foundation landed, on the finding that
// the prover does not come up under a blanket block (site/e2e/zk-prewarm.spec.ts,
// site/e2e/support/fixtures.ts, PR #1405). The GUI panel drives the SAME prover, and this lane
// is the first place in the repo where it runs under an interception — so it inherits the same
// exemption rather than re-deriving the finding.
//
// What that costs, stated plainly: this spec is NOT network-hermetic. Everything the GUI itself
// fetches is same-origin (the static export off `serve`, the synced ACIR at /zk/, the wasm
// bundles), but bb.js's own load/prove path is third-party code and this spec no longer proves
// it stays off the network. Nothing else in the lane changes: the option defaults to `true`, and
// the browser-persona specs block unconditionally in support/web-fixtures.ts.

import { test, expect } from "../support/index.ts";

// See the note above: scoped to this file only.
test.use({ hermeticNetwork: false });

/** The seeded store value we prove over: two digits, `>= FILTER_BOUND` (25), and one of the
 *  values with a committed `operand_enc` fixture in `lib/zk-prover.ts`. */
const WITNESS = 30;

/** A seeded value in the circuit's two-digit domain with NO committed term commitment — the
 *  honest ceiling the panel must SHOW rather than hide. */
const UNCOMMITTED = 41;

/** bb.js returns each public input as a field element; normalise for numeric comparison. */
function asField(hex: string): bigint {
  return BigInt(hex.startsWith("0x") ? hex : `0x${hex}`);
}

test.describe("zk tool", () => {
  // The tauriMock auto-fixture navigates to "/" and waits for engine ready before each test.

  // ONE test covers the whole journey on purpose: the bb.js cold start (dynamic import +
  // Barretenberg WASM instantiate) is the dominant cost and is paid per document, so splitting
  // the honest proof and the refusal into two tests would pay it twice for no extra coverage.
  // The raised ceiling accommodates a SINGLE-THREADED prover: this lane is not
  // cross-origin-isolated (no SharedArrayBuffer), so `maxThreads()` is 1. It is a CEILING, not
  // a measurement — nothing here asserts a wall-clock threshold.
  test("proves a live-store witness in-tab, keeps it out of the public inputs, and refuses the false verdict", {
    timeout: 300_000,
  }, async ({ page }) => {
    await page.locator('[data-tool="zk"]').click();

    // 1. The prover cold start. Asserting the ATTRIBUTE (not just the ready pill's presence)
    //    makes a failure report the state actually reached — `error` means the lazily
    //    imported bundle or the synced ACIR did not survive the static export.
    await expect(page.locator("[data-zk-prover]")).toHaveAttribute(
      "data-zk-prover",
      "ready",
      { timeout: 180_000 },
    );

    // 2. Scan the LIVE store for candidate witnesses.
    await page.getByRole("button", { name: "Find witnesses" }).click();
    const pane = page.locator('[data-result-kind="zk"]');

    const witnessRow = pane.locator(`[data-zk-candidate="${WITNESS}"]`);
    await expect(witnessRow).toHaveAttribute("data-zk-provable", "true");

    // No silent drops: the uncommitted value is listed WITH the reason it cannot be proven.
    const declinedRow = pane.locator(`[data-zk-candidate="${UNCOMMITTED}"]`);
    await expect(declinedRow).toHaveAttribute("data-zk-provable", "false");
    await expect(declinedRow).toContainText("no committed operand_enc");

    // 3. Select the witness explicitly (not relying on the panel's pre-selection) and prove
    //    the HONEST verdict: 30 >= 25 is true, so this claim is satisfiable.
    await witnessRow.getByRole("button").click();
    await page.getByRole("button", { name: "Prove verdict true" }).click();

    const proof = page.locator("[data-zk-proof-verified]");
    await expect(proof).toHaveAttribute("data-zk-proof-verified", "true", { timeout: 150_000 });

    // 4. THE PRIVACY ASSERTION, read off the vector the verifier receives. The circuit
    //    (`zk/compose/filter_int_d2/src/main.nr`) declares exactly five public inputs —
    //    challenge, operand_enc, op, bound, expected — and `digits` (the value) is private.
    const publicInputs = proof.locator("[data-zk-public-input]");
    await expect(proof.locator("[data-zk-public-inputs]")).toHaveAttribute(
      "data-zk-public-inputs",
      "5",
    );
    const fields: bigint[] = [];
    for (const input of await publicInputs.all()) {
      const hex = (await input.getAttribute("data-zk-public-input")) ?? "";
      expect(hex, "each public input renders its field element").toMatch(/^(0x)?[0-9a-fA-F]+$/);
      fields.push(asField(hex));
    }

    // Non-vacuous first: this really is the circuit's public-input vector, so "30 is absent"
    // below is a statement about a populated list rather than about an empty one.
    expect(fields).toContain(3n); // op — OP_GE
    expect(fields).toContain(25n); // bound — FILTER_BOUND
    // …and the store value never reaches the verifier.
    expect(fields).not.toContain(BigInt(WITNESS));

    // 5. The "try to forge it" affordance: ask the SAME witness for the opposite verdict. The
    //    circuit's `filter verdict mismatch` constraint is unsatisfiable, so the witness solve
    //    fails and NO proof exists. That must land on the narrow refusal branch — a generic
    //    prover error (broken ACIR, shifted nightly API, stale fixture) renders the error
    //    <pre> instead, and presenting that as a refusal would dress a broken build up as
    //    soundness evidence (see lib/zk-witness.ts::isUnsatisfiable).
    await page.getByRole("button", { name: "Try to prove false" }).click();
    await expect(pane.locator("[data-zk-refused]")).toBeVisible({ timeout: 150_000 });
    await expect(pane.locator('[data-result-kind="error"]')).toHaveCount(0);
    // The refused claim leaves no proof card behind.
    await expect(proof).toHaveCount(0);
  });
});
