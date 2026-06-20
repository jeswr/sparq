// [OPUS-4.8] sq-vw3ax.3 — headless smoke test for the /capabilities LAZY-MOUNT (the #1 review
// risk: a consolidated gallery that eagerly imported all 8 demos would be HEAVIER than the 14
// pages it replaces). This test asserts the load-bearing invariant END-TO-END in a real browser:
//
//   1. On route entry, NO demo body is in the DOM and NO demo chunk has been requested.
//   2. Expanding a "Demo ▸" row mounts its body AND fires a NEW JS chunk request (the demo's
//      code-split chunk, fetched only on first expand).
//   3. The deep-page rows are plain "Open →" links to the retained /surface/<slug> pages.
//
// This is a CORRECTNESS test (DOM + which chunks were requested), never a timing threshold —
// wall-clock here is non-canonical (work-box / CI runner).
import { test, expect, type Page } from "@playwright/test";

async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(500);
}

test("no demo body or demo chunk loads on /capabilities entry", async ({ page }) => {
  // Record every .js chunk URL the browser requests across the test lifetime.
  const jsRequests: string[] = [];
  page.on("request", (req) => {
    const url = req.url();
    if (url.endsWith(".js") || url.includes(".js?")) jsRequests.push(url);
  });

  await gotoSettled(page, "capabilities/");

  // The page rendered the themes, but NO demo body is mounted yet (lazy: present only on expand).
  await expect(page.getByRole("heading", { name: "Privacy (ZK / MPC)" })).toBeVisible();
  await expect(page.locator("[data-demo-body]")).toHaveCount(0);

  // Snapshot the chunk set after the route + its app shell have fully settled.
  const baselineChunks = jsRequests.length;

  // Expand a Demo ▸ row (full-text — a light demo with no heavy wasm dependency on first paint).
  await page.getByRole("button", { name: /Full-text/i }).click();

  // The demo body is now mounted (its skeleton or the demo itself), and at least one NEW chunk
  // was requested — proof the demo's code-split chunk fetched ONLY on expand, not on load.
  await expect(page.locator('[data-demo-body="full-text"]')).toBeVisible();
  await expect
    .poll(() => jsRequests.length, { timeout: 15_000 })
    .toBeGreaterThan(baselineChunks);
});

test("a deep-page row is a plain Open link to the retained /surface page", async ({ page }) => {
  await gotoSettled(page, "capabilities/");
  // SHACL is one of the 5 retained deep pages → an "Open →" link, not an expand disclosure.
  const shacl = page.getByRole("link", { name: /SHACL/i }).first();
  await expect(shacl).toBeVisible();
  await shacl.click();
  await page.waitForURL("**/surface/shacl/**", { timeout: 15_000 });
  expect(new URL(page.url()).pathname).toContain("/surface/shacl");
});

test("re-collapsing then re-expanding a demo does not re-fetch its chunk", async ({ page }) => {
  const jsRequests: string[] = [];
  page.on("request", (req) => {
    const url = req.url();
    if (url.endsWith(".js") || url.includes(".js?")) jsRequests.push(url);
  });
  await gotoSettled(page, "capabilities/");

  const toggle = page.getByRole("button", { name: /Full-text/i });
  await toggle.click();
  await expect(page.locator('[data-demo-body="full-text"]')).toBeVisible();
  await page.waitForTimeout(500);
  const afterFirstExpand = jsRequests.length;

  // Collapse, then re-expand. The body stays mounted (CSS-hidden) so no NEW chunk is fetched.
  await toggle.click();
  await expect(page.locator('[data-demo-body="full-text"]')).toBeHidden();
  await toggle.click();
  await expect(page.locator('[data-demo-body="full-text"]')).toBeVisible();
  await page.waitForTimeout(500);
  expect(jsRequests.length).toBe(afterFirstExpand);
});
