// [SONNET-4.6] sq-ymr2e.5 — export-contract journey: dataset export format menu + download.
//
// Exercises ExportDataMenu (gui/app/src/components/workbench/store-export.tsx):
//   * The rail export trigger is enabled once the engine is ready with a non-empty store.
//   * Clicking it opens a dropdown with Turtle / TriG / JSON-LD format options.
//   * Selecting a format triggers a browser download (downloadText creates a Blob URL anchor).
//
// Stable selectors:
//   [data-export-trigger="rail"]   — the "Export data…" button in the left rail
//   [data-export-format="turtle"]  — Turtle format item in the dropdown
//   [data-export-format="trig"]    — TriG format item
//   [data-export-format="jsonld"]  — JSON-LD format item
//   [data-export-menu]             — the open dropdown content
//
// The download is detected by Playwright's `page.waitForEvent('download')`, which fires when
// downloadText's synthetic anchor click starts a blob: URL download.
//
// Determinism rules: NO waitForTimeout; web-first assertions only; never assert file content size.

import { test, expect } from "../support/index.ts";

test.describe("export-contract", () => {
  test("export trigger is present and enabled after engine ready with non-empty store", async ({
    page,
  }) => {
    // The engine-context loads the sample graph on mount; by the time engine ready fires, the
    // store has 4 persons (21 quads).  ExportDataMenu disables the trigger when storeSize === 0.
    const exportTrigger = page.locator('[data-export-trigger="rail"]');
    await expect(exportTrigger).toBeVisible();
    // The trigger is enabled because the store is non-empty.
    await expect(exportTrigger).toBeEnabled();
  });

  test("export menu shows format options", async ({ page }) => {
    const exportTrigger = page.locator('[data-export-trigger="rail"]');
    await expect(exportTrigger).toBeEnabled();

    // Open the dropdown.
    await exportTrigger.click();

    // All three export formats must be present in the open menu.
    await expect(page.locator('[data-export-format="turtle"]')).toBeVisible();
    await expect(page.locator('[data-export-format="trig"]')).toBeVisible();
    await expect(page.locator('[data-export-format="jsonld"]')).toBeVisible();
  });

  test("selecting Turtle triggers a browser download", async ({ page }) => {
    const exportTrigger = page.locator('[data-export-trigger="rail"]');
    await expect(exportTrigger).toBeEnabled();
    await exportTrigger.click();

    // Set up the download listener BEFORE clicking the format item.
    const downloadPromise = page.waitForEvent("download");
    await page.locator('[data-export-format="turtle"]').click();

    // Playwright detects the blob: URL anchor click that downloadText triggers.
    const download = await downloadPromise;

    // The filename is "dataset.ttl" (exportFilename("turtle") from rdf-format.ts).
    expect(download.suggestedFilename()).toBe("dataset.ttl");
  });
});
