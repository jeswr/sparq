// [OPUS-4.8] sq-vw3ax.1 — headless browser smoke test for the Cmd-K command palette.
//
// WHAT IT GUARDS. The redesign collapses a tripled navigation into one slim top bar, which is
// only safe because the Cmd-K palette (src/components/command-palette.tsx) is the 0-click fast
// path to every surface (research/website-redesign.md §2, §7). This test drives a REAL headless
// Chromium and exercises the load-bearing flow end-to-end: OPEN (⌘/Ctrl-K and the header
// trigger), SEARCH (fuzzy-filter to one surface), and NAVIGATE (Enter jumps to its route under
// the /sparq basePath). It also asserts ESC closes the dialog.
//
// The palette is pure UI wired to the single GROUPS source — it needs no wasm bundle — so this
// runs on the light site-e2e lane unconditionally (unlike the runner/ZK specs that gate on wasm).
//
// DOM anchors: the cmdk dialog exposes its accessible name "Command palette"; items are real
// listbox options reachable by role + accessible name. No copy-scraping of layout text.
import { test, expect, type Page } from "@playwright/test";
// [OPUS-4.8] sq-ymr2e.1 — shared deterministic navigation barrier: waits for the SW-controller
// signal AND the app-shell hydration (the "Primary" nav landmark), which is the deterministic
// proxy for "the global ⌘K listener is armed", replacing the fixed 500ms sleep
// (research/web-gui-test-program.md §1.1; no-timeout grep gate).
import { waitForAppReady } from "./support/app-ready";

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath — a
// leading slash would target the origin root and miss the basePath entirely. /papers is a
// stable, wasm-free route, so the palette is available with no engine prerequisites.
const ROUTE = "papers/";

// On macOS the shortcut is ⌘K; everywhere else Ctrl-K. The headless Chromium reports a Linux
// platform, so Control is the correct modifier here; the component binds both (metaKey||ctrlKey).
const MOD = "Control";

/** The open palette dialog, located by its accessible name (the sr-only DialogTitle). */
function palette(page: Page) {
  return page.getByRole("dialog", { name: "Command palette" });
}

// The site ships the coi-serviceworker shim (public/coi-serviceworker.js, registered in
// app/layout.tsx): on a FRESH visit it registers and reloads the tab ONCE to take effect.
// That reload tears down any open dialog and resets React state, so a test that opens the
// palette before the reload settles would race it. Wait for the network to go idle (the
// post-reload document) before interacting — the same one-time-reload absorption the
// zk-prewarm spec performs via its readiness pill.
async function gotoSettled(page: Page): Promise<void> {
  await page.goto(ROUTE, { waitUntil: "domcontentloaded" });
  await waitForAppReady(page);
}

test.beforeEach(async ({ page }) => {
  await gotoSettled(page);
});

test("opens with the keyboard shortcut, filters, and navigates on Enter", async ({ page }) => {
  // The palette is not mounted/visible until opened.
  await expect(palette(page)).toHaveCount(0);

  // OPEN via the global keyboard shortcut.
  await page.keyboard.press(`${MOD}+KeyK`);
  const dialog = palette(page);
  await expect(dialog).toBeVisible();

  // The search input is focused on open.
  const input = dialog.getByPlaceholder("Search surfaces, pages, actions…");
  await expect(input).toBeFocused();

  // SEARCH — fuzzy-filter down to the SHACL surface. cmdk filters every Item by its `value`,
  // which includes the theme label + title + blurb, so "shacl" narrows to that row.
  await input.fill("shacl");
  const shaclOption = dialog.getByRole("option", { name: /SHACL/i });
  await expect(shaclOption).toBeVisible();

  // NAVIGATE — the highlighted result is the first match; Enter selects it and routes there.
  await page.keyboard.press("Enter");
  await page.waitForURL("**/surface/shacl/**", { timeout: 15_000 });
  expect(new URL(page.url()).pathname).toContain("/surface/shacl");

  // The palette closed on navigation.
  await expect(palette(page)).toHaveCount(0);
});

test("the header trigger opens the palette and ESC closes it", async ({ page }) => {
  // OPEN via the visible header trigger button (the discoverable affordance).
  await page.getByRole("button", { name: /Search \(Command-K\)/i }).click();
  await expect(palette(page)).toBeVisible();

  // ESC closes it — a11y requirement; the route does not change.
  await page.keyboard.press("Escape");
  await expect(palette(page)).toHaveCount(0);
  expect(new URL(page.url()).pathname).toContain("/papers");
});

test("clicking a result row navigates to its surface", async ({ page }) => {
  await page.keyboard.press(`${MOD}+KeyK`);
  const dialog = palette(page);
  await expect(dialog).toBeVisible();

  // Search a flagship by name and click its row — the mouse path, not the keyboard path.
  await dialog.getByPlaceholder("Search surfaces, pages, actions…").fill("car-hire");
  const flagship = dialog.getByRole("option", { name: /car-hire/i });
  await expect(flagship).toBeVisible();
  await flagship.click();

  await page.waitForURL("**/showcase/zk-car-hire/**", { timeout: 15_000 });
  expect(new URL(page.url()).pathname).toContain("/showcase/zk-car-hire");
});

// ── palette → /app hard-navigation guard (regression) ──────────────────────────────────────────
//
// [FABLE-5] sq-vw3ax.11.1 — /app is served in production by a SEPARATE Next.js build (gui/app)
// overlaid at /app/ (sq-vnd0i), so EVERY user-reachable navigation to /app from the site UI must
// be a HARD full-page navigation. The Cmd-K palette's "App" row was the last live instance of the
// txt-redirect bug class: its go() handler did a next/router soft push, which across the two
// distinct Next builds fetches the foreign RSC Flight payload /app/index.txt and lands on a raw
// .txt instead of the GUI. This guard mirrors the hard-nav assertion in e2e/download-app.spec.ts
// §6: mark the document, select the row, then assert the sentinel is GONE (the document was
// replaced by a full page load), and contrast it with a same-build page (Benchmarks) whose soft
// push KEEPS the same document + client-side router. In `next dev` the overlay is absent, so /app
// resolves to the site's own /app fallback page — a real HTML document — which is fine: the guard
// asserts NAVIGATION SEMANTICS (hard vs soft), not GUI content.
//
// The test MUST fail if go()'s external branch is reverted to a plain router.push (verified
// non-vacuous by that mutation).

/** Open the palette (keyboard), filter to a single row by its title, return the visible option. */
async function openAndFilter(page: Page, query: string, name: RegExp) {
  await page.keyboard.press(`${MOD}+KeyK`);
  const dialog = palette(page);
  await expect(dialog).toBeVisible();
  await dialog.getByPlaceholder("Search surfaces, pages, actions…").fill(query);
  const option = dialog.getByRole("option", { name });
  await expect(option).toBeVisible();
  return option;
}

/** Stamp a sentinel on the live document; it survives a soft SPA nav but not a hard page load. */
async function markDocument(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as unknown as Record<string, unknown>).__sparq_e2e_doc_mark = "palette-hard-nav-guard";
  });
}

/** Whether the sentinel stamped by {@link markDocument} is still on the current document. */
function sentinelPresent(page: Page): Promise<boolean> {
  return page.evaluate(
    () =>
      (window as unknown as Record<string, unknown>).__sparq_e2e_doc_mark ===
      "palette-hard-nav-guard",
  );
}

test("selecting App HARD-navigates (document replaced) — the /app txt-redirect guard", async ({
  page,
}) => {
  // The "App" row is the only external TOP_PAGES entry; match it by its unique blurb (the option's
  // accessible name is the title + blurb, so an anchored /^App$/ would not match).
  const appRow = await openAndFilter(page, "App", /live operational GUI/i);

  await markDocument(page);

  // Set the navigation waiter BEFORE the click so we never miss the event.
  const navPromise = page.waitForURL("**/app/**", { waitUntil: "domcontentloaded" });
  await appRow.click();
  await navPromise;

  // A hard navigation replaces the document, so the sentinel is GONE. If go() had done a soft
  // router.push (the bug), the same document would survive and the sentinel would persist.
  expect(
    await sentinelPresent(page),
    "Expected a HARD full-page navigation to /app (document replaced) but the sentinel survived: " +
      "the palette soft-navigated. go() must window.location.assign for the external /app entry, " +
      "else a next/router push fetches the foreign RSC payload /app/index.txt instead of the GUI.",
  ).toBe(false);

  // Landed under /app (the separate build's overlay in prod; the site's /app fallback in dev).
  expect(new URL(page.url()).pathname).toContain("/app");
});

test("selecting a same-build page (Benchmarks) SOFT-navigates (document survives)", async ({
  page,
}) => {
  // Benchmarks is a same-build TOP_PAGES entry with no `external` flag, so go() keeps the soft
  // router.push — the contrast that proves the hard-nav is scoped to the external /app entry, not
  // a blanket change to every palette row.
  const benchRow = await openAndFilter(page, "Benchmarks", /Per-commit, same-box benchmark/i);

  await markDocument(page);

  const navPromise = page.waitForURL("**/benchmarks/**", { waitUntil: "domcontentloaded" });
  await benchRow.click();
  await navPromise;

  // A soft SPA transition keeps the SAME document, so the sentinel SURVIVES.
  expect(
    await sentinelPresent(page),
    "Expected a SOFT client-side navigation to /benchmarks (same document) but the sentinel was " +
      "lost: a same-build page must keep the router.push soft nav, not hard-reload the document.",
  ).toBe(true);

  expect(new URL(page.url()).pathname).toContain("/benchmarks");
});
