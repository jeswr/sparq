// [OPUS-4.8] sq-ymr2e.4 — AUTOMATED accessibility harness: axe-core WCAG 2.1 AA on the P1
// interactive surfaces × their key UI states.
//
// WHAT IT GUARDS. Accessibility bugs live in the states a pristine page-load never shows — the
// OPEN command palette, the EXPANDED connect drawer, the ERROR strip (a real ARIA tab/panel bug
// already shipped on /try and was found by hand). This spec scans each of those states with axe
// pinned to the WCAG 2.1 A+AA rule tags and enforces the two-tier gate in e2e/support/a11y.ts:
//   * ZERO tier      — serious + critical = HARD FAIL AT ZERO (unconditional).
//   * KNOWN-GAP tier — a surface with a STRUCTURAL, beaded serious violation (a design-token
//                      color-contrast fix that wants the visual-regression net) is not dropped: it
//                      is guarded against any NEW serious rule while its beaded set may only shrink.
// Both tiers ratchet moderate + minor counts in bench/a11y-baseline.json (a ceiling that may only
// fall). Non-vacuity: the last test injects an unlabelled icon-button and asserts axe CATCHES it,
// so a green run proves the scanner is live, not silently disabled.
//
// The harness itself found + this PR FIXED three serious findings (WCAG 1.4.1 inline-link underline
// on home + /download → both /download states are ZERO-gated; keyboard access to the /try error
// <pre>; and the Sonner rich-colours error-toast color-contrast — #e60000 on #fff0f0 = 4.34:1,
// darkened to ~5.8:1 in globals.css, so the "Engine failed to load"/"Query failed" toast the /try
// REPL raises clears AA and the wasm-free /try states stay ZERO-gated even when the engine can't
// load). The remaining structural color-contrast gaps are beaded (sq-ymr2e.13 home hero table +
// button; sq-ymr2e.14 command-palette --success badge) and their surfaces run in the KNOWN-GAP tier.
//
// Design of record: research/web-gui-test-program.md §3.1. Determinism doctrine §1 (frozen clock,
// seeded random, hermetic network, web-first waiters — inherited from the shared `test`).
//
// WASM PREREQ. Two states need the in-tab engine (home RESULTS, /try ERROR): they SKIP when the
// lean wasm bundle is absent (the light site-e2e CI lane builds no Rust toolchain), exactly like
// try-query-smoke.spec.ts. The wasm-FREE states (home idle, /try default/palette/drawer, /download
// release+no-release) run on every lane. Run the full set after `npm run sync-wasm`.
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { Page } from "@playwright/test";
import { test, expect, BasePage } from "./support";
import { assertA11y, writeSeedBaseline, WCAG_TAGS } from "./support/a11y";

// The lean wasm bundle the home runner + /try REPL load at runtime (gitignored build artifact,
// synced into public/wasm/ by `sync-wasm`). Its presence gates the two engine-dependent states.
const WASM_BUNDLE = fileURLToPath(new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url));
const WASM_PRESENT = existsSync(WASM_BUNDLE);

// [SONNET-4.6] sq-ymr2e.13 — home-hero contrast gap fixed: the home hero surfaces are now ZERO-tier.
// The three failing pairs were: (a) sq-tok-string / sq-tok-number / text-primary th on the
// opacity-45 PreviewTable (removed opacity — axe scans aria-hidden visual content; at α=0.45
// even black text only reaches ~3.3:1 on white, so no token colour could clear 4.5:1 AA);
// (b) chart-3 (number token, oklch(0.62 0.14 95) → ~3.6:1) darkened to oklch(0.54 0.14 95)
// → ~5.0:1 and chart-4 (string token, oklch(0.58 0.16 30) → borderline) to oklch(0.55 0.16 30)
// → ~5.2:1 in globals.css :root; (c) the gradient Run button: tailwind-merge drops bg-primary
// when bg-[var(--hero-grad)] is appended, leaving background-color: transparent, so axe sees
// near-white text on the near-white muted/15 parent (≈1.1:1) — fixed by style.backgroundColor:
// var(--primary) on the Button, giving axe a computable 5.3:1 teal-on-near-white ratio.
const GAP_PALETTE = {
  bead: "sq-ymr2e.14",
  reason: "command-palette --success badge is ~4.09:1 (need 4.5); a global token nudge",
};

// In A11Y_SEED mode, rewrite the ratchet baseline from the measured counts (no-op otherwise).
test.afterAll(() => writeSeedBaseline());

// ── Home — the hero runner is the product claim (§2.1 / §3.1). ZERO-tier after sq-ymr2e.13. ──
test.describe("a11y · home", () => {
  test("idle-preview state — WCAG 2.1 AA", async ({ page }) => {
    const home = new BasePage(page);
    await home.goto("");
    await home.expectRunnerState("home", "idle-preview");
    await assertA11y(page, "home:idle"); // ZERO tier: sq-ymr2e.13 fixed.
  });

  test("results state — WCAG 2.1 AA", async ({ page }) => {
    test.skip(!WASM_PRESENT, "wasm bundle absent — the home Run→results state needs the in-tab engine");
    const home = new BasePage(page);
    await home.goto("");
    await home.expectRunnerState("home", "idle-preview");
    // Run the shipped sample; the results table is the wasm-computed state a11y must also cover.
    await page.getByRole("button", { name: /Run/ }).first().click();
    await home.expectRunnerState("home", "results");
    await assertA11y(page, "home:results"); // ZERO tier: sq-ymr2e.13 fixed.
  });
});

// ── /try — the flagship interactive surface, in its major UI states (§2.2 / §3.1). ──────────────
test.describe("a11y · /try", () => {
  /** Navigate to /try and wait for the REPL editor (renders without the engine; wasm-free). */
  async function gotoTry(page: Page): Promise<BasePage> {
    const p = new BasePage(page);
    await p.goto("try/");
    await expect(page.getByRole("textbox", { name: "SPARQL query" })).toBeVisible();
    return p;
  }

  test("default state — WCAG 2.1 AA", async ({ page }) => {
    await gotoTry(page);
    await assertA11y(page, "try:default"); // ZERO tier.
  });

  test("command palette open — WCAG 2.1 AA", async ({ page }) => {
    const p = await gotoTry(page);
    await p.openCommandPalette();
    await assertA11y(page, "try:palette-open", { knownGap: GAP_PALETTE });
  });

  test("connect drawer expanded — WCAG 2.1 AA", async ({ page }) => {
    await gotoTry(page);
    // The endpoint config is a collapsed <details> disclosure; expand it to scan the form fields
    // (endpoint URL + token inputs) that only exist inside the open drawer.
    await page.getByText(/Advanced · Connect to a sparq-server/).click();
    await expect(page.locator("#endpoint-url")).toBeVisible();
    await assertA11y(page, "try:connect-drawer-open"); // ZERO tier.
  });

  test("error state — WCAG 2.1 AA", async ({ page }) => {
    test.skip(!WASM_PRESENT, "wasm bundle absent — the /try error state needs the in-tab engine to reject a query");
    await gotoTry(page);
    await expect(page.getByText("Engine ready")).toBeVisible({ timeout: 90_000 });
    // A deliberately malformed query drives the error strip between editor and results.
    await page.getByRole("textbox", { name: "SPARQL query" }).fill("SELECT ?s WHERE { ?s ?p");
    await page.getByRole("button", { name: "Run query" }).click();
    await expect(page.locator('[data-result-kind="error"]')).toBeVisible({ timeout: 30_000 });
    // ZERO tier: the rich-colours error toast this state raises had a 4.34:1 color-contrast gap
    // (the former GAP_ERROR / sq-ymr2e.14); it is now fixed in globals.css, so this surface is clean.
    await assertA11y(page, "try:error");
  });
});

// ── /download — the conversion surface, both release fixture states (§2.3 / §3.1). ZERO tier. ───
// The releases API is fixture-routed by the hermetic layer; blocking the coi-serviceworker seals
// its fetch-reissue bypass so the fixture deterministically drives the render (download-page.spec
// pattern). Wasm-free — these run on every lane.
test.describe("a11y · /download", () => {
  test.use({ serviceWorkers: "block" });

  test("release-ready state — WCAG 2.1 AA", async ({ page }) => {
    // Default github fixture = "release" (200): the buttons become live direct-download links.
    await page.goto("download/", { waitUntil: "domcontentloaded" });
    await expect(page.getByText("v0.4.2-e2e-fixture").first()).toBeVisible({ timeout: 30_000 });
    await assertA11y(page, "download:release");
  });

  test.describe("no-release state", () => {
    test.use({ github: "notFound" }); // 404 → the degraded "No release yet" state.

    test("no-release state — WCAG 2.1 AA", async ({ page }) => {
      await page.goto("download/", { waitUntil: "domcontentloaded" });
      await expect(
        page.getByRole("link", { name: /No release yet — watch releases/i }).first(),
      ).toBeVisible({ timeout: 30_000 });
      await assertA11y(page, "download:no-release");
    });
  });
});

// ── Non-vacuity: the scanner must CATCH a deliberately unlabelled icon-button (§3.1). ────────────
// If axe were silently disabled/misconfigured, every gate above would pass vacuously. This injects
// a real WCAG-A "button-name" failure (a button whose only child is an aria-hidden icon → no
// accessible name) and asserts axe reports it as a serious/critical violation.
test.describe("a11y · harness non-vacuity", () => {
  test("an injected unlabelled icon-button is caught as a serious/critical violation", async ({
    page,
  }) => {
    const home = new BasePage(page);
    await home.goto("");
    await home.expectRunnerState("home", "idle-preview");

    // Inject an unlabelled icon-button (button-name is a WCAG 2.1 A rule, impact = critical).
    await page.evaluate(() => {
      const probe = document.createElement("div");
      probe.id = "a11y-nonvacuity-probe";
      probe.innerHTML =
        '<button type="button"><svg aria-hidden="true" width="16" height="16"></svg></button>';
      document.body.appendChild(probe);
    });

    // Use the raw AxeBuilder (not the gate) so we assert the RULE fires, scoped to the probe.
    const { AxeBuilder } = await import("@axe-core/playwright");
    const { violations } = await new AxeBuilder({ page })
      .withTags([...WCAG_TAGS])
      .include("#a11y-nonvacuity-probe")
      .analyze();

    const buttonName = violations.find((v) => v.id === "button-name");
    expect(
      buttonName,
      "axe did not catch the injected unlabelled icon-button — the harness is VACUOUS/misconfigured",
    ).toBeTruthy();
    expect(buttonName && (buttonName.impact === "serious" || buttonName.impact === "critical")).toBe(
      true,
    );
  });
});
