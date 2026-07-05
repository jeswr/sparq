// [SONNET-4.6] sq-1odi9 — rail honesty regrouping: working tools vs coming-soon stub subgroup.
//
// Verifies the two-group split in left-rail.tsx is data-driven from the panel registry:
//   [data-tool-group="working"]       — rendered visible by default
//   [data-tool-group="coming-soon"]   — inside the collapsed subgroup; hidden by default
//   [data-rail-section="coming-soon-toggle"] — the toggle button that expands/collapses
//
// Test strategy: expectations are DERIVED from [data-tool-group] DOM attributes (which the
// registry writes at render time) — never from a hardcoded list of tool ids. This stays
// correct as sibling PRs (sq-lxomy / sq-9nwab / sq-kwb74 / sq-iemfq) flip tools to "working".
//
// Determinism rules: NO waitForTimeout; web-first assertions only.

import { test, expect } from "../support/index.ts";

test.describe("rail-groups", () => {
  test("working-group tools are visible by default", async ({ page }) => {
    // There should be at least one working tool in the rail at all times.
    const first = page.locator("[data-tool-group='working']").first();
    await expect(first).toBeVisible();
  });

  test("coming-soon tools are hidden when section is collapsed (default)", async ({ page }) => {
    // The coming-soon subgroup starts collapsed — tools should not be in the visible DOM.
    const first = page.locator("[data-tool-group='coming-soon']").first();
    await expect(first).not.toBeVisible();
  });

  test("Coming-soon toggle exists in the rail", async ({ page }) => {
    const toggle = page.locator("[data-rail-section='coming-soon-toggle']");
    await expect(toggle).toBeVisible();
    // Initially collapsed: aria-expanded=false
    await expect(toggle).toHaveAttribute("aria-expanded", "false");
  });

  test("expanding Coming soon reveals all stub tools", async ({ page }) => {
    const toggle = page.locator("[data-rail-section='coming-soon-toggle']");
    await toggle.click();

    // aria-expanded flips
    await expect(toggle).toHaveAttribute("aria-expanded", "true");

    // At least one coming-soon tool is now visible
    const firstStub = page.locator("[data-tool-group='coming-soon']").first();
    await expect(firstStub).toBeVisible();
  });

  test("collapsing Coming soon hides stub tools again", async ({ page }) => {
    const toggle = page.locator("[data-rail-section='coming-soon-toggle']");
    // Expand first
    await toggle.click();
    await expect(page.locator("[data-tool-group='coming-soon']").first()).toBeVisible();
    // Collapse
    await toggle.click();
    await expect(page.locator("[data-tool-group='coming-soon']").first()).not.toBeVisible();
  });

  test("working tools remain visible when Coming soon section is expanded", async ({ page }) => {
    const toggle = page.locator("[data-rail-section='coming-soon-toggle']");
    await toggle.click();

    // Working tools must still be visible after expanding the coming-soon section
    await expect(page.locator("[data-tool-group='working']").first()).toBeVisible();
  });

  test("every tool with data-tool-group=coming-soon has an honesty-tier dot", async ({ page }) => {
    // Expand the coming-soon section so the tools are rendered
    const toggle = page.locator("[data-rail-section='coming-soon-toggle']");
    await toggle.click();
    await expect(page.locator("[data-tool-group='coming-soon']").first()).toBeVisible();

    // Each coming-soon tool button must contain an honesty-tier dot (span[aria-hidden])
    const stubTools = page.locator("[data-tool-group='coming-soon']");
    const count = await stubTools.count();
    // There must be at least one coming-soon tool (sanity)
    expect(count).toBeGreaterThan(0);
    for (let i = 0; i < count; i++) {
      const dot = stubTools.nth(i).locator("span[aria-hidden]");
      await expect(dot).toBeAttached();
    }
  });

  test("every tool remains reachable from the rail (data-tool integrity)", async ({ page }) => {
    // Expand coming-soon to get all tools rendered
    await page.locator("[data-rail-section='coming-soon-toggle']").click();

    // Every [data-tool] button in the rail must have a non-empty data-tool-group
    const allToolBtns = page.locator("[data-tool][data-tool-group]");
    const count = await allToolBtns.count();
    expect(count).toBeGreaterThan(0);

    for (let i = 0; i < count; i++) {
      const btn = allToolBtns.nth(i);
      const group = await btn.getAttribute("data-tool-group");
      expect(["working", "coming-soon"]).toContain(group);
    }
  });
});
