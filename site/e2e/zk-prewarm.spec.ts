// [OPUS-4.8] sq-5q63 — headless browser smoke test for the ZK car-hire prover pre-warm.
//
// WHAT IT GUARDS. The car-hire age-gate prover pre-warms on route mount
// (`prewarmProver()` in src/lib/zk-prover.ts, kicked off from a useEffect in
// src/components/zk-car-hire.tsx), so the FIRST "Generate ZK proof" click should pay no
// cold start: the dynamic-import of @noir-lang/noir_js + @aztec/bb.js and the dominant
// `Barretenberg.new` WASM instantiate must already be done. Previously that was argued
// only by code reasoning (no browser driver). This test verifies it with a real run.
//
// HOW IT IS OBSERVABLE. `zk-prover.ts` counts every REAL cold start — each time the
// expensive `loadProver` body actually executes — and mirrors the count onto
// `window.__zkProverColdStarts` (pure test observability; it changes nothing about what
// is proved or any security property). A correctly pre-warmed prover leaves the counter
// at 1 across the first proof; a regression that re-paid the cold start on the click
// would bump it to 2. The pre-warm pill (`[data-prover="ready"]`) and the proof-result
// card (`[data-proof-result]`) are the DOM anchors we await — no copy-scraping.
//
// THE COI RELOAD. The page ships the coi-serviceworker shim (public/coi-serviceworker.js,
// registered in app/layout.tsx) so `crossOriginIsolated` becomes true on plain static
// hosting. On a fresh visit that SW registers and reloads the tab ONCE; the cold-start
// counter is per-document, so it is the POST-RELOAD load that pre-warms. We therefore
// wait for the readiness pill to settle (which only happens after the reload) before
// reading the counter — `gotoSettled` encapsulates that.
//
// This is a CORRECTNESS smoke test, not a benchmark: it asserts the cold-start COUNT,
// never a wall-clock threshold (timings on a work-box / CI runner are non-canonical).
// [OPUS-4.8] sq-ymr2e.1 — runs under the shared foundation `test` (e2e/support) for the
// DETERMINISM harness (frozen clock via page.clock.setFixedTime — timers stay LIVE so the
// prewarm/prove flow is not deadlocked — seeded random that leaves WebCrypto untouched, and
// animations-off). It opts OUT of the hermetic-network block (`hermeticNetwork: false`): the ZK
// prover's bb.js SharedArrayBuffer multithreading requires the coi-serviceworker's cross-origin
// isolation (COOP/COEP header injection), which blanket `context.route` interception disrupts, so
// the prover would never reach ready under it. That is safe to skip here — the prover's deps are
// bundled and the circuit is fetched same-origin from public/zk (zero-network by construction), so
// the block would only ever be a no-op. See e2e/support/fixtures.ts (the hermeticNetwork option).
import { test, expect } from "./support";
import { type Page } from "@playwright/test";

test.use({ hermeticNetwork: false });

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath —
// a leading slash would target the origin root and miss the basePath entirely.
const ROUTE = "showcase/zk-car-hire/";

/** Read the test-only cold-start counter the prover mirrors onto `window`. */
function coldStarts(page: Page): Promise<number> {
  return page.evaluate(() => window.__zkProverColdStarts ?? 0);
}

/**
 * Navigate to the ZK route and wait for the prover to settle to "ready". This absorbs
 * the one-time coi-serviceworker reload: we wait on the readiness pill (which only the
 * post-reload document ever shows), so the subsequent counter read is on the live tab.
 */
async function gotoReady(page: Page): Promise<void> {
  await page.goto(ROUTE, { waitUntil: "domcontentloaded" });
  // The post-reload document pre-warms and flips the pill to ready. If pre-warm failed,
  // the pill would read [data-prover="error"] instead — fail loudly in that case.
  await expect(page.locator('[data-prover="error"]')).toHaveCount(0);
  await page.locator('[data-prover="ready"]').waitFor({ state: "visible", timeout: 90_000 });
}

test("first ZK proof pays no cold start after pre-warm", async ({ page }) => {
  await gotoReady(page);

  // The route-mount pre-warm ran the ONE cold start (dynamic import + circuit fetch +
  // Barretenberg.new) and resolved the readiness pill.
  expect(await coldStarts(page)).toBe(1);

  // Click "Generate ZK proof" and await the completed-proof card. Because pre-warm
  // finished, `proveAgeEligibility` awaits the SAME shared prover promise — it must NOT
  // trigger a second `loadProver` body.
  await page.getByRole("button", { name: "Generate ZK proof" }).click();

  const resultCard = page.locator("[data-proof-result]");
  await expect(resultCard).toBeVisible({ timeout: 90_000 });

  // THE ASSERTION: the first proof did not re-pay the cold start. Still exactly one.
  expect(await coldStarts(page)).toBe(1);

  // The default age (30) is provable and the proof self-verifies in-tab — a sanity check
  // that we measured a real proof, not an error path that happens to leave the count at 1.
  await expect(resultCard).toHaveAttribute("data-proof-verified", "true");
});

test("the cold start happens at most once even across a second proof", async ({ page }) => {
  // A second proof on the same tab must also reuse the warmed prover — the memoised
  // `proverPromise` means the cold-start body never runs again for the document's life.
  await gotoReady(page);

  const generate = page.getByRole("button", { name: "Generate ZK proof" });
  await generate.click();
  await expect(page.locator("[data-proof-result]")).toBeVisible({ timeout: 90_000 });
  expect(await coldStarts(page)).toBe(1);

  // Re-running proves the warm path holds; the count stays pinned at one.
  await generate.click();
  await expect(page.locator('[data-proof-verified="true"]')).toBeVisible({ timeout: 90_000 });
  expect(await coldStarts(page)).toBe(1);
});

// [OPUS-4.8] sq-8dx2 — the FL1 composition reaches an ELIGIBLE verdict once the one live
// relation (the age-gate) is proven+verified in-tab, and the per-circuit progress list
// shows the age-gate verified while every OTHER member stays "composed" (never a fake
// verified tick). This guards the load-bearing honesty: a green row == a real in-tab
// check, which is only ever the age-gate.
test("the composition reaches ELIGIBLE with only the age-gate verified live", async ({ page }) => {
  await gotoReady(page);

  await page.getByRole("button", { name: "Generate ZK proof" }).click();
  await expect(page.locator('[data-proof-verified="true"]')).toBeVisible({ timeout: 90_000 });

  // The composed verdict flips to ELIGIBLE.
  await expect(page.locator('[data-zk-verdict="eligible"]')).toBeVisible();

  // The age-gate row is verified; every other row remains "composed" (NOT verified).
  await expect(page.locator('[data-zk-step="filter-age"]')).toHaveAttribute(
    "data-zk-step-status",
    "verified",
  );
  const composedRows = page.locator('[data-zk-step]:not([data-zk-step="filter-age"])');
  const n = await composedRows.count();
  expect(n).toBeGreaterThan(0);
  for (let i = 0; i < n; i++) {
    await expect(composedRows.nth(i)).toHaveAttribute("data-zk-step-status", "composed");
  }
});

// [OPUS-4.8] sq-8dx2 — the FALLBACK: switch to "Captured + verify" and run a REAL in-tab
// bb.js verifyProof over the captured ProofManifest's bundled age-gate sub-proof. The
// bundled bytes are a genuine UltraHonk transcript; this asserts the live verify passes.
test("captured-manifest fallback verifies a real proof in-tab", async ({ page }) => {
  await gotoReady(page);

  await page.getByRole("button", { name: "Captured + verify" }).click();
  await page.getByRole("button", { name: "Verify captured proof" }).click();

  const verifyCard = page.locator("[data-verify-result]");
  await expect(verifyCard).toBeVisible({ timeout: 90_000 });
  await expect(verifyCard).toHaveAttribute("data-verify-ok", "true");

  // The composed verdict also reaches ELIGIBLE on the fallback path.
  await expect(page.locator('[data-zk-verdict="eligible"]')).toBeVisible();
});
