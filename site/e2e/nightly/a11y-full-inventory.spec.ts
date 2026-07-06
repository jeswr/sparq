// [SONNET-4.6] sq-izpab — NIGHTLY full-inventory AXE sweep: axe on EVERY site route in its
// default load state (advisory, nightly lane only, never per-PR).
//
// WHAT IT GUARDS. The P1 interactive-surface axe harness (e2e/a11y.spec.ts) covers key UI
// states of the home hero + /download surfaces in their per-PR scan. This nightly companion
// extends coverage to EVERY route in the full site inventory — content pages (papers, specs,
// benchmarks, surfaces, showcases) that the per-PR scan intentionally omits for speed. Each
// route is loaded in its DEFAULT render state (no interaction required) and scanned with
// @axe-core/playwright pinned to the WCAG 2.1 A+AA rule tags.
//
// GATE SHAPE. Two-tier, matching a11y.spec.ts §3.1:
//   * ZERO tier — serious + critical = HARD FAIL AT ZERO (unconditional, same as P1 spec).
//   * MODERATE/MINOR counts are LOGGED per route (ratchet is a planned follow-up bead; first
//     nightly run calibrates the ceiling — seed with A11Y_NIGHTLY_SEED=1, see below).
//
// NIGHTLY LANE. Lives in e2e/nightly/ — only the `nightly-a11y` Playwright project (defined
// only when SPARQ_NIGHTLY_A11Y=1, set by .github/workflows/nightly-full-sweep.yml) picks it
// up. The per-PR `chromium`/`firefox`/`webkit` projects all ignore e2e/nightly/, so the
// per-PR path is byte-unchanged.
//
// ENGINE-GATED STATE. The home:results state (WASM-computed) requires the lean wasm bundle.
// All other routes render in their default state without WASM and are unconditionally scanned.
// The nightly lane builds the bundle (site-visual.yml recipe), but the WASM_PRESENT guard lets
// this spec run in smoke mode (--list) and in any environment without the bundle.
//
// NON-VACUITY. A dedicated test injects an unlabelled icon-button and asserts axe catches it
// — if axe were silently disabled, every ZERO-tier check above would pass vacuously.
//
// ROUTE INVENTORY. Sourced from the same data modules as e2e/surface-sweep.spec.ts (PAPERS /
// SPECS); the filesystem guard tests in surface-sweep validate the BENCHMARK_FAMILY_KEYS /
// DEEP_SURFACE_SLUGS / SHOWCASE_SLUGS constants — those guard tests are NOT duplicated here.
// When a new route is added to surface-sweep.spec.ts, add it to the inventory below too.
// [SONNET-4.6] sq-yk2ho — /assurance, /assurance/served-conformance, /dogfooding added.
//
// NEXT STEPS (follow-up bead). Add moderate/minor ratchet: run once with A11Y_NIGHTLY_SEED=1
// to populate bench/a11y-nightly-baseline.json, review the seeded counts, commit, then wire
// the ceiling check alongside the existing ZERO tier.
//
// Design of record: research/web-gui-test-program.md §3.1 + §6.2.
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { AxeBuilder } from "@axe-core/playwright";
import { test, expect, gotoAppReady } from "../support";
import { WCAG_TAGS, formatViolations, countByImpact } from "../support/a11y";
import { PAPERS } from "../../src/data/papers";
import { SPECS } from "../../src/data/specs";
import type { Page } from "@playwright/test";

// ── WASM presence check (same as a11y.spec.ts) ───────────────────────────────────────────
const WASM_BUNDLE = fileURLToPath(new URL("../../public/wasm/sparq_wasm_bg.wasm", import.meta.url));
const WASM_PRESENT = existsSync(WASM_BUNDLE);

// ── Route inventory ───────────────────────────────────────────────────────────────────────
// Each entry: path relative to the /sparq/ baseURL (include trailing slash), human name.
// NOTE: keep in sync with surface-sweep.spec.ts when routes are added/removed (guard tests
// for the inventory constants live in surface-sweep.spec.ts, NOT duplicated here).

interface RouteEntry {
  path: string;
  name: string;
}

// Benchmark family keys — must stay in sync with BENCHMARK_FAMILY_KEYS in surface-sweep.spec.ts.
const BENCHMARK_FAMILY_KEYS = [
  "sparql",
  "shacl",
  "geo",
  "fts",
  "vector",
  "reasoning",
  "core",
  "zk",
  "solid",
  "hdt",
  "rsp",
  "genai",
  "gpu",
] as const;

// Surface deep-page slugs — must stay in sync with DEEP_SURFACE_SLUGS in surface-sweep.spec.ts.
const DEEP_SURFACE_SLUGS = [
  "sparql",
  "data-formats",
  "javascript-wasm",
  "shacl",
  "inference",
] as const;

// Showcase slugs — must stay in sync with SHOWCASE_SLUGS in surface-sweep.spec.ts.
const SHOWCASE_SLUGS = [
  "mpc-100k",
  "solid-pairs",
  "verifiable-credentials",
  "zk-car-hire",
] as const;

// ── Static top-level routes ───────────────────────────────────────────────────────────────
const STATIC_ROUTES: RouteEntry[] = [
  { path: "", name: "/ (home, idle)" },
  { path: "capabilities/", name: "/capabilities" },
  { path: "app/", name: "/app" },
  { path: "download/", name: "/download" },
  { path: "examples/", name: "/examples" },
  // [SONNET-4.6] sq-yk2ho — previously-unlinked content routes now in inventory.
  { path: "assurance/", name: "/assurance" },
  { path: "assurance/served-conformance/", name: "/assurance/served-conformance" },
  { path: "dogfooding/", name: "/dogfooding" },
];

// ── Papers routes ─────────────────────────────────────────────────────────────────────────
const PAPERS_ROUTES: RouteEntry[] = [
  { path: "papers/", name: "/papers (index)" },
  ...PAPERS.map((p) => ({ path: `papers/${p.slug}/`, name: `/papers/${p.slug}` })),
];

// ── Specs routes ──────────────────────────────────────────────────────────────────────────
const SPECS_ROUTES: RouteEntry[] = [
  { path: "specs/", name: "/specs (index)" },
  ...SPECS.map((s) => ({ path: `specs/${s.slug}/`, name: `/specs/${s.slug}` })),
];

// ── Benchmarks routes ─────────────────────────────────────────────────────────────────────
const BENCHMARKS_ROUTES: RouteEntry[] = [
  { path: "benchmarks/", name: "/benchmarks (index)" },
  ...BENCHMARK_FAMILY_KEYS.map((key) => ({
    path: `benchmarks/${key}/`,
    name: `/benchmarks/${key}`,
  })),
];

// ── Surface deep-page routes ──────────────────────────────────────────────────────────────
const SURFACE_ROUTES: RouteEntry[] = DEEP_SURFACE_SLUGS.map((slug) => ({
  path: `surface/${slug}/`,
  name: `/surface/${slug}`,
}));

// ── Showcase routes ───────────────────────────────────────────────────────────────────────
const SHOWCASE_ROUTES: RouteEntry[] = SHOWCASE_SLUGS.map((slug) => ({
  path: `showcase/${slug}/`,
  name: `/showcase/${slug}`,
}));

// ── Complete inventory ────────────────────────────────────────────────────────────────────
const ALL_ROUTES: RouteEntry[] = [
  ...STATIC_ROUTES,
  ...PAPERS_ROUTES,
  ...SPECS_ROUTES,
  ...BENCHMARKS_ROUTES,
  ...SURFACE_ROUTES,
  ...SHOWCASE_ROUTES,
];

// ── Axe helper: ZERO-tier check + count logging ───────────────────────────────────────────

/** Run the WCAG 2.1 A+AA axe scan and HARD-FAIL on any serious/critical violation. */
async function assertNoSeriousCritical(page: Page, routeName: string): Promise<void> {
  const { violations } = await new AxeBuilder({ page })
    .withTags([...WCAG_TAGS])
    .analyze();

  const blocking = violations.filter((v) => v.impact === "serious" || v.impact === "critical");
  const counts = countByImpact(violations);

  // Log moderate/minor for informational tracking (no ratchet in this first version).
  if (counts.moderate > 0 || counts.minor > 0) {
    // eslint-disable-next-line no-console
    console.log(
      `[a11y-inventory] ${routeName}: moderate=${counts.moderate} minor=${counts.minor} ` +
        `(zero-tier green; moderate/minor ratchet is a follow-up bead)`,
    );
  }

  expect(
    blocking,
    `\n[a11y-inventory ${routeName}] serious/critical WCAG 2.1 AA violation(s) — must be ZERO:\n` +
      `${formatViolations(blocking)}\n`,
  ).toHaveLength(0);
}

// ── Parametrized full-inventory ZERO-tier sweep ───────────────────────────────────────────

for (const entry of ALL_ROUTES) {
  test(`[a11y-inventory] ${entry.name} — ZERO serious/critical WCAG 2.1 AA`, async ({ page }) => {
    await gotoAppReady(page, entry.path);
    await assertNoSeriousCritical(page, entry.name);
  });
}

// ── Engine-gated state: home:results (WASM-computed) ─────────────────────────────────────
// The per-PR a11y spec already covers home:results when WASM is present. This nightly entry
// adds it to the full-inventory scan. Skip when the lean wasm bundle is absent.
test("[a11y-inventory] / (home, WASM results state) — ZERO serious/critical WCAG 2.1 AA", async ({
  page,
}) => {
  test.skip(!WASM_PRESENT, "wasm bundle absent — home:results needs the in-tab engine (build with npm run sync-wasm)");
  await gotoAppReady(page, "");
  await page.getByRole("button", { name: /Run/ }).first().click();
  // Wait for the WASM-computed results state before scanning.
  await expect(
    page.locator("[data-runner-surface='home'][data-runner-state='results']"),
  ).toBeVisible({ timeout: 60_000 });
  await assertNoSeriousCritical(page, "/ (home, WASM results)");
});

// ── Non-vacuity: the scanner must CATCH a deliberately unlabelled icon-button ─────────────
// Mirrors the non-vacuity check in e2e/a11y.spec.ts. If axe were silently disabled, every
// ZERO-tier test above would pass vacuously.
test("[a11y-inventory] harness non-vacuity — injected unlabelled icon-button is caught", async ({
  page,
}) => {
  await gotoAppReady(page, "");

  await page.evaluate(() => {
    const probe = document.createElement("div");
    probe.id = "a11y-nightly-nonvacuity-probe";
    probe.innerHTML =
      '<button type="button"><svg aria-hidden="true" width="16" height="16"></svg></button>';
    document.body.appendChild(probe);
  });

  const { violations } = await new AxeBuilder({ page })
    .withTags([...WCAG_TAGS])
    .include("#a11y-nightly-nonvacuity-probe")
    .analyze();

  const buttonName = violations.find((v) => v.id === "button-name");
  expect(
    buttonName,
    "axe did not catch the injected unlabelled icon-button — the nightly a11y harness is VACUOUS/misconfigured",
  ).toBeTruthy();
  expect(
    buttonName && (buttonName.impact === "serious" || buttonName.impact === "critical"),
  ).toBe(true);
});
