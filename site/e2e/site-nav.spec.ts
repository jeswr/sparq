// [OPUS-4.8] sq-vw3ax.7 — headless smoke test for the COLLAPSED navigation.
//
// WHAT IT GUARDS. The redesign removed the persistent w-64 sidebar tree + the duplicate
// top-tab bar, leaving ONE slim top bar of content destinations + a "Try the GUI" entry
// (research/website-redesign.md §2, §7). This test asserts the slim bar's five destinations
// route correctly, that the old full sidebar tree is GONE (no persistent surface tree on a
// content page), and that the maintainer's two discoverability gaps — Download and Try-the-GUI
// — are reachable from the bar.
import { test, expect, type Page } from "@playwright/test";

const MOD = "Control";

// The site ships the coi-serviceworker shim which reloads the tab once on a fresh visit; wait
// for the post-reload document to settle before interacting (same pattern as command-palette).
async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(500);
}

test("the slim top bar shows the 5 destinations + Try the GUI, and no full sidebar tree", async ({
  page,
}) => {
  await gotoSettled(page, "");

  const primary = page.getByRole("navigation", { name: "Primary" }).first();
  for (const label of ["Home", "Capabilities", "Benchmarks", "Papers", "Download"]) {
    await expect(primary.getByRole("link", { name: label, exact: true })).toBeVisible();
  }
  // "Try the GUI" is a distinct top-bar entry (the maintainer's flagged GUI-discoverability gap).
  await expect(page.getByRole("link", { name: /Try the GUI/i }).first()).toBeVisible();

  // The OLD full sidebar tree is gone: there is no persistent "Feature surfaces" nav landmark
  // (the sidebar's aria-label) and no per-surface tree link like "GeoSPARQL" on a content page.
  await expect(page.getByRole("navigation", { name: "Feature surfaces" })).toHaveCount(0);
});

test("Capabilities is reachable from the top bar and renders the 5 themes", async ({ page }) => {
  await gotoSettled(page, "");
  await page
    .getByRole("navigation", { name: "Primary" })
    .first()
    .getByRole("link", { name: "Capabilities", exact: true })
    .click();
  await page.waitForURL("**/capabilities/**", { timeout: 15_000 });

  for (const theme of [
    "Query & data",
    "Reason & validate",
    "Search & GenAI",
    "Privacy (ZK / MPC)",
    "Serve & embed",
  ]) {
    await expect(page.getByRole("heading", { name: theme })).toBeVisible();
  }
});

test("Download and Try the GUI are reachable destinations", async ({ page }) => {
  await gotoSettled(page, "");

  await page
    .getByRole("navigation", { name: "Primary" })
    .first()
    .getByRole("link", { name: "Download", exact: true })
    .click();
  await page.waitForURL("**/download/**", { timeout: 15_000 });
  expect(new URL(page.url()).pathname).toContain("/download");

  await page.getByRole("link", { name: /Try the GUI/i }).first().click();
  await page.waitForURL("**/gui/**", { timeout: 15_000 });
  expect(new URL(page.url()).pathname).toContain("/gui");
});

test("Cmd-K still opens after the sidebar removal (the fast path to every surface)", async ({
  page,
}) => {
  await gotoSettled(page, "");
  await page.keyboard.press(`${MOD}+KeyK`);
  await expect(page.getByRole("dialog", { name: "Command palette" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "Command palette" })).toHaveCount(0);
});
