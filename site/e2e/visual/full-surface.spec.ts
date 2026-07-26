// [SONNET-4.6] sq-ledny — NIGHTLY full-surface visual-regression specs beyond key layouts.
//
// WHAT IT GUARDS. The per-PR key-layouts spec (e2e/visual/key-layouts.spec.ts, sq-ymr2e.10)
// covers the home first fold (dark + light), /download cards, the mpc-100k showcase, and
// nav + palette-open overlay. This nightly companion extends visual regression to the rest
// of the site surface:
//   • Top-level content routes: /capabilities, /app, /examples
//   • Papers: /papers index + one representative per-paper page
//   • Specs: /specs index + one representative per-spec page
//   • Benchmarks: /benchmarks index (the per-family chart pages are excluded pending
//     data-vr-mask on chart containers — adding that mask is a follow-up bead)
//   • Surface deep pages: /surface/sparql, data-formats, javascript-wasm, shacl, inference
//   • Showcase pages: /showcase/solid-pairs, verifiable-credentials (mpc-100k is per-PR;
//     zk-car-hire is excluded because its ZK prover auto-starts on mount, making the
//     pre-interaction snapshot non-deterministic without a state waiter — follow-up bead)
//
// ANTI-FLAKE DISCIPLINE. Every capture inherits the full determinism harness from the shared
// `test` (frozen clock, seeded random, animations-off, reducedMotion, hermetic network,
// pinned viewport, dark theme, UTC, en-US). On top of that each capture:
//   * waits for `document.fonts.ready` (font loading is async);
//   * waits for the app-shell hydration barrier (`data-app-ready`) via gotoAppReady;
//   * masks every `[data-vr-mask]` region (dynamic content: timing footers, release
//     version/size/sha strings) via the vrMasks() helper — "mask, don't chase".
//
// CONTAINER RULE. Same as key-layouts.spec.ts: these specs run ONLY inside the
// digest-pinned official Playwright container (SPARQ_VR=1, scripts/vr.sh). The visual-*
// projects are container-only by design; see e2e/visual/README.md.
//
// BASELINE GENERATION. The baselines for this spec ARE committed under baselines/<project>/.
// They can only be minted inside the pinned container: `npm run vr:update` (from site/, via
// scripts/vr.sh), or — with no local docker — the nightly's `refresh_visual_baselines` +
// `mode: full` dispatch, which re-mints in CI and uploads them as an artifact to review and
// commit. See e2e/visual/README.md § "No docker?" for the full baseline-update workflow.
//
// [SONNET-4.6] sq-hfd82 — DRIFT NOTE. This spec runs ONLY in the nightly (playwright.config.ts
// gates it behind SPARQ_NIGHTLY_VR), so a PR that restyles the app shell, adds a /capabilities
// tile, or edits a /surface page does not see its own pixel drift. The baselines therefore go
// stale between nightlies as a matter of course, and a red sweep here is far more often
// accumulated intentional change than a regression — read the uploaded diffs before assuming
// either. Data-driven regions must carry data-vr-mask instead ("mask, don't chase").
//
// Design of record: research/web-gui-test-program.md §4 + §6.2.
import type { Page } from "@playwright/test";

import { test, expect, gotoAppReady } from "../support";
import { PAPERS } from "../../src/data/papers";
import { SPECS } from "../../src/data/specs";

/** Every dynamic region opted into the mask convention (mirrors key-layouts.spec.ts). */
function vrMasks(page: Page) {
  return [page.locator("[data-vr-mask]")];
}

/** Deterministic capture barrier: all declared web fonts are loaded and applied. */
async function fontsReady(page: Page): Promise<void> {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

// ── Top-level content routes ──────────────────────────────────────────────────────────────

test("capabilities page — card grid", async ({ page }) => {
  await gotoAppReady(page, "capabilities/");
  await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("capabilities.png", { mask: vrMasks(page) });
});

test("app page — hosted GUI link", async ({ page }) => {
  await gotoAppReady(page, "app/");
  await expect(page.getByRole("heading", { level: 1, name: /App/i })).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("app.png", { mask: vrMasks(page) });
});

test("examples page — examples listing", async ({ page }) => {
  await gotoAppReady(page, "examples/");
  await expect(page.getByRole("heading", { level: 1, name: /Examples/i })).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("examples.png", { mask: vrMasks(page) });
});

// ── Papers ────────────────────────────────────────────────────────────────────────────────

test("papers index — card list", async ({ page }) => {
  await gotoAppReady(page, "papers/");
  // At least one "Read paper" link confirms the cards rendered.
  await expect(page.getByRole("link", { name: /Read paper/i }).first()).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("papers-index.png", { mask: vrMasks(page) });
});

// Representative per-paper page: first paper in the registry (deterministic, data-driven).
// The paper blurb is always server-rendered from the data module, so the card is visible
// even without the Typst-built HTML fragment (which the nightly lane builds).
test("papers — first paper page", async ({ page }, testInfo) => {
  // Skip if the paper registry is unexpectedly empty (guard against a broken state).
  if (PAPERS.length === 0) {
    testInfo.skip();
    return;
  }
  const paper = PAPERS[0];
  await gotoAppReady(page, `papers/${paper.slug}/`);
  // Wait for the blurb (always rendered, from the data module).
  await expect(
    page.getByText(paper.blurb.slice(0, 60), { exact: false }).first(),
  ).toBeVisible({ timeout: 20_000 });
  await fontsReady(page);
  await expect(page).toHaveScreenshot(`papers-${paper.slug}.png`, { mask: vrMasks(page) });
});

// ── Specs ─────────────────────────────────────────────────────────────────────────────────

test("specs index — card list", async ({ page }) => {
  await gotoAppReady(page, "specs/");
  await expect(page.getByRole("link", { name: /Read draft/i }).first()).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("specs-index.png", { mask: vrMasks(page) });
});

// Representative per-spec page: first spec in the registry.
test("specs — first spec page", async ({ page }, testInfo) => {
  if (SPECS.length === 0) {
    testInfo.skip();
    return;
  }
  const spec = SPECS[0];
  await gotoAppReady(page, `specs/${spec.slug}/`);
  await expect(
    page.getByText(spec.blurb.slice(0, 60), { exact: false }).first(),
  ).toBeVisible({ timeout: 20_000 });
  await fontsReady(page);
  await expect(page).toHaveScreenshot(`specs-${spec.slug}.png`, { mask: vrMasks(page) });
});

// ── Benchmarks (index only; per-family chart pages deferred pending data-vr-mask) ─────────
// The /benchmarks index page shows family cards (names, labels, counts) — fully static
// content from the committed benchmarks.generated.json. Per-family pages render SVG charts
// whose axis labels are derived from the benchmark data; those pages will be added once
// data-vr-mask is applied to the chart containers (follow-up bead sq-ledny.2).

test("benchmarks index — family cards", async ({ page }) => {
  await gotoAppReady(page, "benchmarks/");
  await expect(page.getByRole("heading", { level: 1, name: /Benchmarks/i })).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("benchmarks-index.png", { mask: vrMasks(page) });
});

// ── Surface deep pages ────────────────────────────────────────────────────────────────────

for (const slug of ["sparql", "data-formats", "javascript-wasm", "shacl", "inference"] as const) {
  test(`surface deep page — /surface/${slug}`, async ({ page }) => {
    await gotoAppReady(page, `surface/${slug}/`);
    await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
    await fontsReady(page);
    await expect(page).toHaveScreenshot(`surface-${slug}.png`, { mask: vrMasks(page) });
  });
}

// ── Showcase pages ────────────────────────────────────────────────────────────────────────
// mpc-100k is covered by key-layouts.spec.ts (per-PR). zk-car-hire is excluded from this
// spec because its ZK prover auto-starts on route mount (the prewarm mechanism), making
// the pre-interaction snapshot non-deterministic without a dedicated state waiter — add it
// in a follow-up bead once the prover-ready state is observable (data-state attribute).

test("showcase — solid-pairs (pre-interaction idle state)", async ({ page }) => {
  // The solid-pairs demo starts in idle state (no auto-run on mount). The pre-interaction
  // render (header, query editor, selector dropdowns, Run button) is static.
  await gotoAppReady(page, "showcase/solid-pairs/");
  await expect(page.getByRole("heading", { name: /Solid.*pair/i }).first()).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot("showcase-solid-pairs.png", { mask: vrMasks(page) });
});

test("showcase — verifiable-credentials (pre-interaction idle state)", async ({ page }) => {
  // The VC import-and-query workbench starts in idle state (awaiting drag/drop). The initial
  // render (header, import area, query panel) is static.
  await gotoAppReady(page, "showcase/verifiable-credentials/");
  await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
  await fontsReady(page);
  await expect(page).toHaveScreenshot(
    "showcase-verifiable-credentials.png",
    { mask: vrMasks(page) },
  );
});
