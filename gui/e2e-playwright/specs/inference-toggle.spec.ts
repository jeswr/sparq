// [SONNET-4.6] sq-ymr2e.5 — inference-toggle journey: mode selector aria-pressed + status chip.
//
// Exercises the InferenceControl component (gui/app/src/components/workbench/inference-control.tsx):
//   * Off (default) → RDFS: aria-pressed changes, a status chip appears.
//   * Status chip settles to either "ready" (+N entailed) or "error" (reasoner unavailable).
//   * RDFS → Off: chip disappears (InferenceStatusPill returns null when mode is "off").
//
// Stable selectors:
//   [data-inference-mode="off"]    — the Off toggle button
//   [data-inference-mode="rdfs"]   — the RDFS toggle button
//   [data-inference-control]       — the toggle group container
//   [data-inference-status]        — the live status chip (loading / ready / error)
//   [data-inference-status="loading"] / ["ready"] / ["error"]
//
// The W-reason WASM bundle may or may not be present in the test build.  The test asserts the
// presence of a status chip and that it eventually settles — it does NOT assert the exact state
// (ready vs error).  Both outcomes are valid depending on the build.
//
// Determinism rules: NO waitForTimeout; web-first assertions only.

import { test, expect } from "../support/index.ts";

test.describe("inference-toggle", () => {
  test("toggle Off → RDFS changes aria-pressed and shows a status chip", async ({ page }) => {
    // Initial state: Off is active (aria-pressed="true"), RDFS is inactive.
    const offBtn = page.locator('[data-inference-mode="off"]');
    const rdfsBtn = page.locator('[data-inference-mode="rdfs"]');

    await expect(offBtn).toBeVisible();
    await expect(rdfsBtn).toBeVisible();

    await expect(offBtn).toHaveAttribute("aria-pressed", "true");
    await expect(rdfsBtn).toHaveAttribute("aria-pressed", "false");

    // ── Switch to RDFS ──────────────────────────────────────────────────────────────────────
    await rdfsBtn.click();

    // The toggle selection must flip immediately.
    await expect(rdfsBtn).toHaveAttribute("aria-pressed", "true");
    await expect(offBtn).toHaveAttribute("aria-pressed", "false");

    // A status chip appears (loading → then either ready or error).
    const statusChip = page.locator("[data-inference-status]");
    await expect(statusChip).toBeVisible();

    // Wait for the chip to settle out of "loading" (the reasoner either produces a closure
    // or reports unavailable — both are valid in a test build).
    // We wait for a non-loading state by polling for either ready or error.
    await expect(
      page.locator('[data-inference-status="ready"], [data-inference-status="error"]'),
    ).toBeVisible({ timeout: 60_000 });

    // ── Restore to Off ──────────────────────────────────────────────────────────────────────
    await offBtn.click();

    // Off is active again.
    await expect(offBtn).toHaveAttribute("aria-pressed", "true");
    await expect(rdfsBtn).toHaveAttribute("aria-pressed", "false");

    // The chip is removed from the DOM (InferenceStatusPill returns null when mode is "off").
    await expect(page.locator("[data-inference-status]")).not.toBeAttached({ timeout: 10_000 });
  });
});
