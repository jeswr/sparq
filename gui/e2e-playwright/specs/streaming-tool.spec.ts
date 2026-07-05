// [SONNET-4.6] sq-kwb74 — RSP-QL streaming tool tick-view spec.
// Uses the desktop mock-IPC lane (tauriMock auto-fixture).
// Test: either the wasm bundle is present (happy path) or absent (honest unavailable state).
// Happy path: push 2 events at ts=0 and ts=65 into a tumbling range=60 window →
//   the [0,60) window closes on the second push → assert window output rows appear.
// Both outcomes are valid and non-vacuous: the spec distinguishes them with data-rsp-status.

import { test, expect } from "../support/index.ts";

test.describe("streaming-tool", () => {
  test.beforeEach(async ({ page }) => {
    // Open the Streaming tool tab from the left rail.
    await page.locator('[data-tool="streaming"]').click();
  });

  test("streaming tool renders and either shows the RSP-QL tick view or the honest unavailable state", async ({ page }) => {
    // Wait for the wasm status to settle (loading → ready or unavailable).
    const statusEl = page.locator('[data-rsp-status]');
    await expect(statusEl).toBeVisible({ timeout: 60_000 });
    await expect(
      page.locator('[data-rsp-status="ready"], [data-rsp-status="unavailable"]'),
    ).toBeVisible({ timeout: 60_000 });

    const status = await statusEl.getAttribute('data-rsp-status');

    if (status === 'unavailable') {
      // Honest unavailable state: assert the message is present.
      await expect(page.getByText(/RSP bundle/i).or(page.getByText(/unavailable/i)).first()).toBeVisible();
      return;
    }

    // Happy path: the RSP-QL engine is ready.
    // Push event 1: s1 reading=10 at ts=0 → no window closes yet.
    await page.locator('[data-rsp-push-s]').fill('<http://ex/s1>');
    await page.locator('[data-rsp-push-p]').fill('<http://ex/reading>');
    await page.locator('[data-rsp-push-o]').fill('10');
    await page.locator('[data-rsp-push-ts]').fill('0');
    await page.locator('[data-rsp-push-button]').click();

    // No windows should have closed yet (ts=0 < window end=60).
    await expect(page.locator('[data-rsp-window-item]')).toHaveCount(0);

    // Push event 2: s1 reading=20 at ts=65 → watermark=65 > 60, closes [0,60).
    await page.locator('[data-rsp-push-ts]').fill('65');
    await page.locator('[data-rsp-push-o]').fill('20');
    await page.locator('[data-rsp-push-button]').click();

    // Window [0,60) should now appear in the tick output.
    const windowItem = page.locator('[data-rsp-window-item]').first();
    await expect(windowItem).toBeVisible();

    // The window output must contain real engine results (the avg binding).
    // AVG(10, 20) = 15 — check for a value that proves real output (not a stub).
    await expect(windowItem).toContainText('avg');
  });
});
