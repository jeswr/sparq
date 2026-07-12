// [FABLE-5] sq-ixc3.15 — ODRL policy tool spec (web persona).
//
// Runs under the chromium-web project (*.web.spec.ts pattern): NO Tauri globals, pure browser
// persona. The ODRL stack (sparq-policy evaluator + sparq-solid enforcement store) is NOT in
// the in-tab wasm bundle, so in the browser the tool must degrade HONESTLY: the native-only
// message is shown and the run trigger is disabled — never a fabricated decision, never a
// silent no-op. The tier chip carries the `live-native` framing.
//
// Determinism rules: NO waitForTimeout; web-first assertions only; stable selectors.

import { webTest as test, webExpect as expect } from "../support/index.ts";

test.describe("odrl-tool (web persona)", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  test("degrades honestly in the browser: native-only note shown, run disabled", async ({
    page,
  }) => {
    await page.locator('[data-tool="odrl"]').click();

    // The honest degradation note names the native-only reality.
    const note = page.locator("[data-odrl-web-note]");
    await expect(note).toBeVisible();
    await expect(note).toContainText(/native-only/i);
    await expect(note).toContainText(/desktop/i);

    // The run trigger is disabled — the browser cannot produce a real decision.
    await expect(page.locator("[data-odrl-run]")).toBeDisabled();

    // The honesty tier chip carries the live-native framing (dot + label).
    await expect(page.locator("[data-odrl-tier]")).toContainText(/native/i);

    // The authoring surfaces still render (author in the browser, run on desktop).
    await expect(page.locator("[data-odrl-policy]")).toBeVisible();
  });
});
