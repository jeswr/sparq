// [OPUS-4.8] sq-vw3ax.7 / sq-rclb8 — headless smoke test for the COLLAPSED navigation (Option-B).
//
// WHAT IT GUARDS. The redesign removed the persistent w-64 sidebar tree + the duplicate top-tab
// bar, leaving ONE slim top bar of content destinations (research/website-redesign.md §2, §7).
// Option-B (the maintainer's decision after #1004 opened) gives the bar TWO distinct destinations
// instead of one "Try the GUI": "Try" → /try (the lightweight in-browser SPARQL REPL playground,
// kept unchanged) and "App" → /app (the live operational GUI). The old single "Try the GUI" → /gui
// entry is dropped; /gui now client-redirects to /app. This test asserts the slim bar's six
// destinations route correctly, that the old full sidebar tree is GONE, that Try (/try) and App
// (/app) are both reachable, and that the legacy /gui path redirects to /app.
import { test, expect, type Page } from "@playwright/test";

const MOD = "Control";

// The site ships the coi-serviceworker shim which reloads the tab ONCE on a fresh visit to take
// effect. That reload tears down any pending client-side navigation, so a click-then-navigate
// test can race it (the click fires, then the SW reload bounces the tab back to where it was).
// Wait until the SW is CONTROLLING the page — the deterministic signal that the one-time reload
// has already happened and won't fire again — before interacting. This is the reliable form of
// the "absorb the one-time reload" pattern the /try specs do via an explicit page.reload().
async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
  // The coi-serviceworker reloads once, after which navigator.serviceWorker.controller is set.
  // (Wrapped in a try so the test still proceeds if the SW never registers, e.g. unsupported.)
  await page
    .waitForFunction(() => navigator.serviceWorker?.controller != null, undefined, {
      timeout: 10_000,
    })
    .catch(() => {});
  await page.waitForTimeout(500);
}

test("the slim top bar shows the 6 destinations (Try + App distinct) and no full sidebar tree", async ({
  page,
}) => {
  await gotoSettled(page, "");

  const primary = page.getByRole("navigation", { name: "Primary" }).first();
  for (const label of [
    "Home",
    "Capabilities",
    "Try",
    "App",
    "Benchmarks",
    "Download",
  ]) {
    await expect(primary.getByRole("link", { name: label, exact: true })).toBeVisible();
  }
  // The OLD single "Try the GUI" entry is gone — Try (REPL) and App (GUI) are separate now.
  await expect(page.getByRole("link", { name: /Try the GUI/i })).toHaveCount(0);

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

test("the top bar points Try at the REPL (/try) and App at the GUI (/app)", async ({ page }) => {
  await gotoSettled(page, "");
  const primary = page.getByRole("navigation", { name: "Primary" }).first();
  // The two Option-B destinations are DISTINCT and point at the right routes (assert the href
  // directly — deterministic, and not subject to the dev-server's one-time coi-serviceworker
  // reload racing a click-then-navigate).
  await expect(primary.getByRole("link", { name: "Try", exact: true })).toHaveAttribute(
    "href",
    /\/try\/?$/,
  );
  await expect(primary.getByRole("link", { name: "App", exact: true })).toHaveAttribute(
    "href",
    /\/app\/?$/,
  );
});

test("the Try destination (/try) is the lightweight live REPL playground", async ({ page }) => {
  // The Try nav link's href is asserted above; here we confirm the destination it points at IS
  // the REPL. We goto /try DIRECTLY (the same way every other /try spec reaches it) rather than
  // click-navigating into it: under `next dev` the heavy lazy REPL route compiles on demand and
  // can trigger a Fast Refresh full reload that interrupts a client-side click-navigation — the
  // click mechanism itself is already covered by the App + Download click tests on the same bar.
  await gotoSettled(page, "try/");
  expect(new URL(page.url()).pathname).toContain("/try");
  // [OPUS-4.8] sq-vw3ax — the /try redesign moved the page-identity heading out of the heavy,
  // lazily-loaded REPL card (the old `<h2>Live SPARQL REPL</h2>`) into a server-rendered hero
  // `<h1>` that paints with the route shell. Assert that hero heading (matched by a stable
  // substring) — it confirms /try IS the SPARQL playground destination without waiting on the
  // wasm REPL chunk to stream in.
  await expect(
    page.getByRole("heading", { name: /SPARQL playground/i }),
  ).toBeVisible();
});

test("App navigates to the live operational GUI destination", async ({ page }) => {
  await gotoSettled(page, "");
  await page
    .getByRole("navigation", { name: "Primary" })
    .first()
    .getByRole("link", { name: "App", exact: true })
    .click();
  await page.waitForURL("**/app/**", { timeout: 30_000 });
  expect(new URL(page.url()).pathname).toContain("/app");
  await expect(page.getByRole("heading", { name: "App", level: 1 })).toBeVisible();
});

test("Download is a reachable destination", async ({ page }) => {
  await gotoSettled(page, "");
  await page
    .getByRole("navigation", { name: "Primary" })
    .first()
    .getByRole("link", { name: "Download", exact: true })
    .click();
  await page.waitForURL("**/download/**", { timeout: 30_000 });
  expect(new URL(page.url()).pathname).toContain("/download");
});

test("the legacy /gui path client-redirects to /app", async ({ page }) => {
  await gotoSettled(page, "gui/");
  await page.waitForURL("**/app/**", { timeout: 15_000 });
  expect(new URL(page.url()).pathname).toContain("/app");
  await expect(page.getByRole("heading", { name: "App", level: 1 })).toBeVisible();
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
