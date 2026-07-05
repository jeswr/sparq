// [OPUS-4.8] sq-jp7ry (issue #835) — CRITICAL-FLOW smoke test for the HOME route.
//
// WHAT IT GUARDS. The landing page (src/app/page.tsx) is the site's front door: a server-
// rendered atmospheric HERO (`src/components/home/hero.tsx`) with the gradient display headline
// and the primary nav (the AppShell "Primary" navigation landmark). This is the #1 "did the app
// even boot" smoke: it asserts the hero headline paints AND every primary-nav destination is
// present, and — load-bearing for catching a quiet runtime fault — that the route produces ZERO
// console errors. A broken client component, a bad import, or a hydration throw all surface as a
// console error here even when the page still paints, so the zero-error assertion is the real
// regression net (issue #835's standing ask: "unexpected bugs get introduced").
//
// NO WASM NEEDED. The hero + nav are server-rendered shell — they paint without the wasm engine,
// so this spec runs on EVERY lane (including the light site-e2e CI lane that builds no wasm
// bundle). The in-tab hero runner below the fold is its own (wasm-gated) concern, covered by
// home-runner.spec.ts; here we only assert the shell boots clean.
//
// It is a CORRECTNESS smoke test, not a benchmark: it asserts the DOM + console, never a
// wall-clock threshold (timings on a work-box / CI runner are non-canonical).
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";

// Empty relative route → resolves to the baseURL's `/sparq/` basePath (the home page).
const ROUTE = "";

// The home page eagerly mounts the lazy in-tab hero runner (`HeroQueryRunner`), which pre-warms
// the wasm engine by FETCHING `public/wasm/sparq_wasm_bg.wasm`. On the LIGHT site-e2e CI lane that
// bundle is intentionally NOT built (no Rust toolchain), so the fetch 404s and the browser logs a
// bare "Failed to load resource: the server responded with a status of 404 (Not Found)" console
// error. That is the repo's documented "optional wasm — warn and skip" posture (sync-wasm.mjs),
// NOT a product bug — the runner surfaces a clear in-page state and the SHELL we are smoke-testing
// (hero + nav) is unaffected. So we IGNORE that benign resource-load
// 404 while still catching every real client-side error (a JS throw / bad import / hydration
// fault arrives as a `pageerror` or a NON-404 `console.error`, both of which still fail the test).
const BENIGN_RESOURCE_404 = /Failed to load resource:.*\b404\b/i;

/**
 * Collect every REAL `console.error` / `pageerror` the page emits — minus the benign
 * optional-wasm-bundle 404 that the light CI lane produces by design — so the test can assert
 * the shell booted clean.
 */
function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() !== "error") return;
    const text = msg.text();
    if (BENIGN_RESOURCE_404.test(text)) return; // optional wasm bundle absent on the light lane
    errors.push(text);
  });
  // A real JS throw (never a network 404) — always load-bearing, never filtered.
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

// The site ships the coi-serviceworker shim which reloads the tab ONCE on a fresh visit. That
// one-time reload tears down a pending render, so wait until the SW is CONTROLLING the page (the
// deterministic signal the reload has already fired and won't fire again) before asserting.
//
// We deliberately do NOT wait for `networkidle` here (unlike the lighter site-nav specs): the
// home route eagerly mounts the in-tab REPL, whose wasm pre-warm fetch RETRIES when the bundle is
// absent (the light CI lane), so the network never goes idle and a `networkidle` wait would hang
// to the test timeout. The deterministic SW-controller signal plus the explicit element waits in
// the test body are sufficient — and `.catch` keeps it working if the SW never registers.
async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page
    .waitForFunction(() => navigator.serviceWorker?.controller != null, undefined, {
      timeout: 10_000,
    })
    .catch(() => {});
}

test("the home route renders the hero headline and the primary nav with no console errors", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoSettled(page, ROUTE);

  // The hero <h1> (server-rendered, paints with the shell). Matched on a stable substring of the
  // headline copy rather than the whole gradient-split markup ("A full SPARQL engine. In this
  // tab.").
  await expect(
    page.getByRole("heading", { name: /full SPARQL engine/i, level: 1 }),
  ).toBeVisible();

  // The slim top bar (the AppShell "Primary" navigation landmark) carries the six content
  // destinations. Asserting the landmark + its core links proves the nav booted, not just copy.
  // [SONNET-4.6] sq-4hiqe — "Try" was removed from the nav (the /try playground is gone).
  // [OPUS-4.8] sq-1scgk — "Papers" was promoted INTO the bar (maintainer 2026-07-04 item 9b:
  // make the paper-factory output prominently findable); the bar is now
  // Home · Capabilities · App · Benchmarks · Papers · Download (no Try item).
  const primary = page.getByRole("navigation", { name: "Primary" }).first();
  await expect(primary).toBeVisible();
  for (const label of ["Home", "Capabilities", "App", "Benchmarks", "Papers", "Download"]) {
    await expect(primary.getByRole("link", { name: label, exact: true })).toBeVisible();
  }

  // The whole boot produced zero console errors — the regression net for a quiet client-side
  // throw / bad import / hydration fault that still paints (issue #835's "unexpected bugs").
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
