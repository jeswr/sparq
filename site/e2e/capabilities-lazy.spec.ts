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
// [OPUS-4.8] sq-ymr2e.1 — the shared deterministic navigation barrier (SW-reload absorption +
// app-shell hydration), replacing the fixed 500ms sleeps with web-first signals
// (research/web-gui-test-program.md §1.1; no-timeout grep gate).
import { gotoAppReady } from "./support/app-ready";

async function gotoSettled(page: Page, route: string): Promise<void> {
  await gotoAppReady(page, route);
}

// [OPUS-4.8] sq-ymr2e.1 — resolve once no NEW tracked request has arrived for one poll interval.
// A deterministic "the interaction's async loads have all landed" settle (the first expand fires a
// code-split chunk AND the demo's async text-search wasm load, which can land late) — used before
// snapshotting so a straggler never pollutes the re-expand delta. Bounded; NOT a fixed sleep.
async function waitForRequestsSettled(count: () => number): Promise<void> {
  let previous = -1;
  await expect
    .poll(
      () => {
        const now = count();
        const settled = now === previous;
        previous = now;
        return settled;
      },
      { timeout: 8_000, intervals: [250] },
    )
    .toBe(true);
}

test("no demo body or demo chunk loads on /capabilities entry", async ({ page }) => {
  // Record every .js chunk URL the browser requests across the test lifetime.
  const jsRequests: string[] = [];
  page.on("request", (req) => {
    const url = req.url();
    // [OPUS-4.8] sq-ymr2e.1 — count Next.js code-split CHUNKS only. Exclude two classes of
    // dev-runtime `.js` noise that are NOT the demo's code-split chunk and would false-positive the
    // "no chunk re-fetch on re-expand" invariant: (1) `/wasm/` loader assets (the full-text demo
    // lazily loads its text-search wasm on show — a runtime asset, surfaces only when the wasm
    // bundle is present locally), and (2) `next dev` HMR `hot-update` chunks (the ±1 Fast-Refresh
    // drift the test comment below warns about). The real demo chunk is `_next/static/chunks/*.js`.
    if (
      (url.endsWith(".js") || url.includes(".js?")) &&
      !url.includes("/wasm/") &&
      !url.includes("hot-update")
    ) {
      jsRequests.push(url);
    }
  });

  await gotoSettled(page, "capabilities/");

  // The page rendered the themes, but NO demo body is mounted yet (lazy: present only on expand).
  await expect(
    page.getByRole("heading", { name: "Privacy (ZK / MPC / E2EE)" }),
  ).toBeVisible();
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
    // [OPUS-4.8] sq-ymr2e.1 — count Next.js code-split CHUNKS only. Exclude two classes of
    // dev-runtime `.js` noise that are NOT the demo's code-split chunk and would false-positive the
    // "no chunk re-fetch on re-expand" invariant: (1) `/wasm/` loader assets (the full-text demo
    // lazily loads its text-search wasm on show — a runtime asset, surfaces only when the wasm
    // bundle is present locally), and (2) `next dev` HMR `hot-update` chunks (the ±1 Fast-Refresh
    // drift the test comment below warns about). The real demo chunk is `_next/static/chunks/*.js`.
    if (
      (url.endsWith(".js") || url.includes(".js?")) &&
      !url.includes("/wasm/") &&
      !url.includes("hot-update")
    ) {
      jsRequests.push(url);
    }
  });
  await gotoSettled(page, "capabilities/");

  const toggle = page.getByRole("button", { name: /Full-text/i });
  await toggle.click();
  await expect(page.locator('[data-demo-body="full-text"]')).toBeVisible();
  // Wait until the first expand's async loads (its code-split chunk + the demo's text-search wasm)
  // have all landed before snapshotting — deterministic request-settle, not a fixed sleep.
  await waitForRequestsSettled(() => jsRequests.length);
  // [OPUS-4.8] sq-rclb8 — snapshot the chunk set AFTER the first expand, then assert that the
  // re-collapse/re-expand cycle fires NO new .js request. We compare the request set DELTA, not
  // the total count: under `next dev` a stray Next.js <Link> prefetch / HMR chunk can land at any
  // moment (more so now the slim bar has more visible links to prefetch), which drifts the total
  // count by ±1 without ever being a demo re-fetch. The load-bearing invariant is that NOTHING
  // new is fetched on re-expand (the body stays mounted, CSS-hidden) — exactly the delta below.
  const afterFirstExpand = jsRequests.length;

  // Collapse, then re-expand. The body stays mounted (CSS-hidden) so no NEW chunk is fetched.
  await toggle.click();
  await expect(page.locator('[data-demo-body="full-text"]')).toBeHidden();
  await toggle.click();
  await expect(page.locator('[data-demo-body="full-text"]')).toBeVisible();
  // Deterministic settle so a late request would be observed — not a fixed sleep.
  await waitForRequestsSettled(() => jsRequests.length);
  // No request fired during the re-collapse/re-expand cycle — the demo chunk was not re-fetched.
  expect(jsRequests.slice(afterFirstExpand)).toEqual([]);
});
