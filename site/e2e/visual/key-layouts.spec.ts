// [FABLE-5] sq-ymr2e.10 — VISUAL-REGRESSION key layouts (per-PR scope).
//
// Design of record: research/web-gui-test-program.md §4. These specs run ONLY in the
// `visual-desktop` / `visual-mobile` projects, which exist ONLY under SPARQ_VR=1 — i.e. inside
// the digest-pinned official Playwright container that scripts/vr.sh launches (font rasterizing
// and antialiasing are container-stable; committed baselines are Linux-only by policy). See
// e2e/visual/README.md for the baseline-update workflow (`npm run vr:update`).
//
// Determinism inherited from the shared harness (support/): frozen clock, seeded Math.random,
// animations-off stylesheet, reducedMotion, pinned viewport/dark/UTC/en-US, hermetic network
// (the /download GitHub releases call is served from the committed "release" fixture). On top
// of that, every capture:
//   * waits for `document.fonts.ready` AND the surface's deterministic UI state (§4) — never a
//     timeout;
//   * masks every `[data-vr-mask]` region — the convention for dynamic content (measured
//     timings, release version/size/sha strings) so the baseline survives content churn
//     ("mask, don't chase");
//   * relies on toHaveScreenshot's own stabilisation (it captures until two consecutive frames
//     are identical) plus the config-level `animations: "disabled"` / `caret: "hide"`.
//
// SCOPE is the flake-control (§4): per-PR shoots ONLY the key layouts — home first fold (idle
// state, dark + one light), the /download cards (release-fixture state), one showcase page
// (mpc-100k: the only flagship with no auto-start effect, so its pre-interaction render is
// static), and the nav + palette-open overlay. The full-surface sweep is the nightly lane
// (sq-ymr2e.11).
// [OPUS-4.8] sq-4hiqe — the /try workbench-shell shot was removed with the /try playground.
import type { Page } from "@playwright/test";

import { test, expect, gotoAppReady, runnerStateLocator } from "../support";

/** Every dynamic region opted into the mask convention (one locator masks all matches). */
function vrMasks(page: Page) {
  return [page.locator("[data-vr-mask]")];
}

/** Deterministic capture barrier: all declared web fonts are loaded and applied. */
async function fontsReady(page: Page): Promise<void> {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

test("home first fold — idle hero runner (dark)", async ({ page }) => {
  await gotoAppReady(page, ".");
  // The hero runner's idle-preview state is server-rendered and WASM-free (§2.1) — the
  // deterministic anchor for the first-fold capture.
  await expect(runnerStateLocator(page, "home", "idle-preview")).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("home-first-fold.png", { mask: vrMasks(page) });
});

test("home first fold — the one light-theme shot", async ({ page }, testInfo) => {
  // ONE light shot per §4 (the site is dark-first; light is the toggle path) — desktop only,
  // to keep the per-PR baseline set tight.
  test.skip(testInfo.project.name !== "visual-desktop", "one light-theme shot, desktop only (§4)");
  // next-themes persists the toggle in localStorage ("theme"); seeding it before the first
  // document renders the light palette exactly as a returning light-theme visitor sees it.
  await page.addInitScript(() => window.localStorage.setItem("theme", "light"));
  await gotoAppReady(page, ".");
  await expect(runnerStateLocator(page, "home", "idle-preview")).toBeVisible();
  await expect(page.locator("html")).toHaveClass(/light/);
  await fontsReady(page);
  await expect(page).toHaveScreenshot("home-first-fold-light.png", { mask: vrMasks(page) });
});

test("download — cards in the release-fixture state", async ({ page }) => {
  // The hermetic layer serves the committed "release" fixture for the releases/latest call, so
  // the enriched (version/size/sha) card state is deterministic; the strings themselves are
  // additionally data-vr-mask'ed so a fixture refresh cannot invalidate the layout baseline.
  await gotoAppReady(page, "download/");
  // The version badge appears only once the fixture fetch has resolved to `ready`.
  await expect(page.getByText("Desktop app")).toBeVisible();
  await expect(page.locator("[data-vr-mask]").first()).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("download-cards.png", { mask: vrMasks(page) });
});

test("showcase — MPC £100k secure threshold (pre-interaction)", async ({ page }) => {
  // The one per-PR showcase layout (§4). mpc-100k is the flagship whose demo has NO auto-start
  // effect — its pre-interaction render (header badges, demo controls, caption) is static.
  await gotoAppReady(page, "showcase/mpc-100k/");
  await expect(
    page.getByRole("heading", { name: /MPC £100k secure threshold/i }),
  ).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("showcase-mpc-100k.png", { mask: vrMasks(page) });
});

test("nav + command-palette-open overlay", async ({ page }, testInfo) => {
  await gotoAppReady(page, ".");
  await expect(runnerStateLocator(page, "home", "idle-preview")).toBeVisible();
  // DESKTOP: open the palette via the header search button (role-based, deterministic). The
  // trigger is `hidden lg:inline-flex` (app-shell.tsx), so at the 390px MOBILE viewport it does
  // not exist — there the global Control+K binding (registered before the app-ready barrier
  // resolves) is the only open path, same as the functional palette spec. The keybinding
  // BEHAVIOUR is owned by e2e/command-palette.spec.ts; this test only needs the OPEN overlay's
  // pixels. Focus is on <body> straight after load, so the keypress cannot land in an editor.
  if (testInfo.project.name === "visual-mobile") {
    await page.keyboard.press("Control+KeyK");
  } else {
    await page.getByRole("button", { name: /Search \(Command-K\)/ }).click();
  }
  await expect(page.getByRole("dialog", { name: "Command palette" })).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("nav-palette-open.png", { mask: vrMasks(page) });
});
