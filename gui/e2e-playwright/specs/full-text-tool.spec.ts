// [SONNET-4.6] sq-9nwab — full-text BM25 search panel journey (desktop / mock-IPC persona).
//
// Runs under chromium-mock-ipc (Tauri IPC mock). The auto-fixture `tauriMock` navigates to "/"
// and waits for engine ready before the test body runs.
//
// The W-text wasm bundle may or may not be present in the test build. The spec distinguishes:
//   * Bundle UNAVAILABLE → the mount-time probe sets bundleError; [data-text-unavailable] appears
//     automatically (no Search click needed). Asserts the honest rebuild hint.
//   * Bundle AVAILABLE   → the search input is enabled; asserts a real BM25 query returns at
//     least one "Alice" hit from the sample graph (non-vacuous, no blanket pass).
//
// Stable selectors (all declared in full-text-tool.tsx):
//   data-tool="full-text"        — left-rail nav button for the Full-text tool
//   data-text-search-input       — the search term input (always present; disabled when unavailable)
//   data-text-search-btn         — the Search button
//   data-text-results            — the results container div
//   data-text-unavailable        — the honest unavailable error block (mount-time probe failure)
//   data-text-hit                — each individual hit row (for counting hits)
//
// Determinism rules: NO waitForTimeout; web-first assertions only.

import { test, expect } from "../support/index.ts";

test.describe("full-text-tool", () => {
  test("full-text tool shows BM25 results or honest unavailable state", async ({ page }) => {
    // Navigate to the Full-text tab via the left rail.
    await page.locator('[data-tool="full-text"]').click();

    const unavailableEl = page.locator("[data-text-unavailable]");
    const searchInput = page.locator("[data-text-search-input]");

    // The search input is ALWAYS rendered (visible when the tab opens). Wait for it — this is
    // a fast assertion that just confirms the FullTextTool panel mounted.
    await expect(searchInput).toBeVisible({ timeout: 10_000 });

    // The mount-time probe fires and resolves quickly (a network fetch of the wasm JS glue).
    // Wait for EITHER the unavailable block to appear (bundle missing) OR the input to become
    // enabled (bundle loaded). A disabled input means the probe is still in flight or failed;
    // we poll both outcomes so the test settles as soon as either resolves.
    //
    // Strategy: wait up to 30s for the unavailable block; if it doesn't appear, the bundle
    // loaded successfully and the input should be enabled.
    const unavailableOrEnabled = unavailableEl.or(
      page.locator("[data-text-search-input]:not([disabled])"),
    );
    await expect(unavailableOrEnabled).toBeVisible({ timeout: 30_000 });

    if (await unavailableEl.isVisible()) {
      // Bundle unavailable — assert the honest state with the rebuild hint.
      await expect(unavailableEl).toContainText("build:text-wasm");
    } else {
      // Bundle available — assert a real BM25 search works end-to-end.
      await expect(searchInput).not.toBeDisabled();
      await searchInput.fill("Alice");
      await page.locator("[data-text-search-btn]").click();

      const results = page.locator("[data-text-results]");
      await expect(results).toBeVisible();

      // Must get at least one hit — non-vacuous (real BM25 result from the sample graph).
      await expect(results.locator("[data-text-hit]").first()).toBeVisible({ timeout: 30_000 });

      // The hit must contain "Alice" — content-level non-vacuous assertion.
      await expect(results.locator("[data-text-hit]").first()).toContainText("Alice");
    }
  });
});
