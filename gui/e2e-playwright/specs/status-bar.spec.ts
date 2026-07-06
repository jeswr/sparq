// [SONNET-4.6] sq-ymr2e.5 — status-bar journey: latency/rows presence + mocked disk gauge.
//
// Exercises StatusBar (gui/app/src/components/workbench/status-bar.tsx):
//
//   1. After a query run: the status bar shows "X.Y ms" (latency) and "N rows" (row count).
//      Presence-checked only — never assert exact values (wall-clock is non-canonical).
//
//   2. Disk gauge: the mocked disk_usage IPC returns { bytes: 12345678, exists: true } on mount,
//      so the status bar should show [data-disk-source="os"] (OS-reported figure) and the label
//      "disk" (not "≈disk").  The PRESENCE of the gauge is the assertion, not the byte count.
//
//   3. Backend label: the backend chip shows "backend:" followed by a resolved state.
//
// Stable selectors:
//   footer.sq-statusbar          — the status bar footer element
//   [data-status-disk]           — the disk gauge span
//   [data-disk-source="os"]      — attribute set when diskBytes comes from the native probe
//   [data-status-ingest]         — the ingest meter (only while an import is in progress)
//
// Determinism rules: NO waitForTimeout; presence-only assertions (never specific ms/byte values).

import { test, expect } from "../support/index.ts";

test.describe("status-bar", () => {
  test("status bar is visible on load", async ({ page }) => {
    // The status bar is always present (mounted in the workbench shell).
    const statusBar = page.locator("footer.sq-statusbar");
    await expect(statusBar).toBeVisible();
  });

  test("after query run, latency and row count are shown in the status bar", async ({ page }) => {
    // Run the default query.
    const runBtn = page.getByRole("button", { name: "Run query" });
    await runBtn.click();

    // Wait for the result.
    await expect(page.locator('[data-result-kind="select"]')).toBeVisible();

    const statusBar = page.locator("footer.sq-statusbar");

    // The status bar shows "X.Y ms" — presence-check the "ms" unit, never the numeric value.
    await expect(statusBar.getByText(/\d+(\.\d+)?\s*ms/)).toBeVisible();

    // The row count shows "N rows" — presence only.
    await expect(statusBar.getByText(/\d[\d,]*\s*rows/)).toBeVisible();
  });

  test("disk gauge is present and OS-reported (mocked disk_usage returns bytes)", async ({
    page,
  }) => {
    // The engine-context calls nativeDiskUsage() on mount (disk_usage IPC).
    // The mock returns { bytes: 12345678, exists: true } → osReported = true.
    // StatusBar sets data-disk-source="os" and shows "disk" (not "≈disk").
    const diskGauge = page.locator("[data-status-disk]");
    await expect(diskGauge).toBeVisible();

    // The OS-reported attribute must be set (not "estimate").
    await expect(page.locator('[data-disk-source="os"]')).toBeVisible();

    // The gauge label is "disk" (without the ≈ prefix used for estimates).
    await expect(diskGauge).toContainText("disk");
  });

  test("backend label is visible in the status bar", async ({ page }) => {
    const statusBar = page.locator("footer.sq-statusbar");
    // The backend span shows "backend: <state>" — presence-check the "backend:" text.
    await expect(statusBar.getByText(/backend:/)).toBeVisible();
  });
});
