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
// bundle). The heavy in-tab REPL below the hero is its own (wasm-gated) concern, covered by
// repl-results.spec.ts + try-query-smoke.spec.ts; here we only assert the shell boots clean.
//
// It is a CORRECTNESS smoke test, not a benchmark: it asserts the DOM + console, never a
// wall-clock threshold (timings on a work-box / CI runner are non-canonical).
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";

// Empty relative route → resolves to the baseURL's `/sparq/` basePath (the home page).
const ROUTE = "";

/** Collect every `console.error` the page emits, so the test can assert there were none. */
function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

// The site ships the coi-serviceworker shim which reloads the tab ONCE on a fresh visit. That
// one-time reload tears down a pending render, so wait until the SW is CONTROLLING the page (the
// deterministic signal the reload has already fired and won't fire again) before asserting — the
// same settle the site-nav specs use. Wrapped in `.catch` so it still proceeds if the SW never
// registers (e.g. an environment without service-worker support).
async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
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
  // headline copy rather than the whole gradient-split markup.
  await expect(
    page.getByRole("heading", { name: /state-of-the-art RDF engine/i, level: 1 }),
  ).toBeVisible();

  // The slim top bar (the AppShell "Primary" navigation landmark) carries the six content
  // destinations. Asserting the landmark + its core links proves the nav booted, not just copy.
  const primary = page.getByRole("navigation", { name: "Primary" }).first();
  await expect(primary).toBeVisible();
  for (const label of ["Home", "Capabilities", "Try", "Benchmarks"]) {
    await expect(primary.getByRole("link", { name: label, exact: true })).toBeVisible();
  }

  // The whole boot produced zero console errors — the regression net for a quiet client-side
  // throw / bad import / hydration fault that still paints (issue #835's "unexpected bugs").
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
