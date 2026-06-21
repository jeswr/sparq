// [OPUS-4.8] sq-vw3ax.3 / .7 — headless smoke test for the CLIENT-SIDE redirect stubs.
//
// WHAT IT GUARDS. Static export cannot 301 (research/website-redesign.md §7 must_fix). The
// redesign removed the 8+ /surface/<slug> walkthrough routes (folded into /capabilities) and
// /about (folded into Home #how-it-runs); every old path ships a tiny stub that client-redirects
// to the new destination so inbound links don't 404. This test drives a real browser to a few
// removed paths and asserts the redirect lands on the right new home.
import { test, expect, type Page } from "@playwright/test";

async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(500);
}

// Each removed surface route → the /capabilities theme anchor it should land on.
const SURFACE_REDIRECTS: { from: string; theme: string }[] = [
  { from: "surface/zk/", theme: "privacy" },
  { from: "surface/mpc/", theme: "privacy" },
  { from: "surface/geosparql/", theme: "query-data" },
  { from: "surface/full-text/", theme: "search-genai" },
  { from: "surface/vector/", theme: "search-genai" },
  { from: "surface/genai/", theme: "search-genai" },
  { from: "surface/http-server/", theme: "serve-embed" },
  { from: "surface/streaming-rsp/", theme: "serve-embed" },
  { from: "surface/cli/", theme: "serve-embed" },
  { from: "surface/python/", theme: "serve-embed" },
];

for (const { from, theme } of SURFACE_REDIRECTS) {
  test(`removed /${from} client-redirects to /capabilities#${theme}`, async ({ page }: { page: Page }) => {
    await gotoSettled(page, from);
    await page.waitForURL("**/capabilities/**", { timeout: 15_000 });
    const url = new URL(page.url());
    expect(url.pathname).toContain("/capabilities");
    expect(url.hash).toBe(`#${theme}`);
    // The theme section it anchored to is on the page.
    await expect(page.locator(`#${theme}`)).toBeVisible();
  });
}

test("removed /about client-redirects to the Home #how-it-runs section", async ({ page }) => {
  await gotoSettled(page, "about/");
  await page.waitForURL((u) => new URL(u).pathname.replace(/\/$/, "") === "/sparq", {
    timeout: 15_000,
  });
  const url = new URL(page.url());
  expect(url.hash).toBe("#how-it-runs");
  await expect(page.locator("#how-it-runs")).toBeVisible();
});

test("the 5 retained deep pages are NOT redirected (they keep their own route)", async ({
  page,
}) => {
  for (const slug of ["sparql", "shacl", "inference", "data-formats", "javascript-wasm"]) {
    await gotoSettled(page, `surface/${slug}/`);
    // Still on the deep page — not bounced to /capabilities.
    expect(new URL(page.url()).pathname).toContain(`/surface/${slug}`);
    expect(new URL(page.url()).pathname).not.toContain("/capabilities");
  }
});
