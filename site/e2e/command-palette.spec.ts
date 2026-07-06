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
