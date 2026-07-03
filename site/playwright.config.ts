// [OPUS-4.8] sq-5q63 — Playwright config for the site's headless browser smoke tests.
//
// Scope: the ZK car-hire prover pre-warm smoke test (e2e/zk-prewarm.spec.ts). The test
// drives a REAL browser against a REAL Next.js server so the "first proof pays no cold
// start" claim is verified by an automated run, not just by code reasoning.
//
// [OPUS-4.8] sq-ymr2e.1 — this config is now also the base for the shared E2E foundation
// (e2e/support/**): a determinism harness (frozen clock, seeded random, animations-off,
// pinned viewport/dark/reduced-motion/UTC/en-US) and a hermetic-network fixture. The
// determinism CONTEXT options live in `use` below; the per-page bits are auto-fixtures in
// e2e/support/fixtures.ts. Design of record: research/web-gui-test-program.md §1.
//
// The server is `next dev` (NOT the static export): the ZK route is fully functional in
// dev — it dynamic-imports @noir-lang/noir_js + @aztec/bb.js (real deps) and fetches the
// committed circuit from public/zk/, none of which need the wasm-REPL/Typst prebuild
// artifacts. Driving dev directly keeps the smoke test self-contained and fast.
//
// Run locally:  npx playwright install chromium && npm run test:e2e
import { defineConfig, devices } from "@playwright/test";

const PORT = Number(process.env.PORT ?? 3210);
// basePath "/sparq" (next.config.ts) prefixes every route, so the demo lives under it.
// Keep the TRAILING SLASH: Playwright resolves a relative `page.goto("showcase/…")`
// against this baseURL, and only a trailing slash makes the basePath a path *prefix*
// (without it, the relative segment would replace "/sparq" instead of nesting under it).
const BASE_URL = `http://127.0.0.1:${PORT}/sparq/`;

export default defineConfig({
  testDir: "./e2e",
  // Real ZK proving in a single-thread browser is the slow step; give it room. The
  // assertion is correctness (cold-start count), never a timing threshold — wall-clock
  // here is non-canonical (work-box / CI-runner), so we never gate on a duration.
  timeout: 120_000,
  expect: { timeout: 90_000 },
  // A flaky network/instantiate cold start is the prover's documented self-reset path,
  // not a product bug; one retry absorbs that without masking a real regression (a true
  // second cold start fails deterministically on every attempt).
  retries: process.env.CI ? 1 : 0,
  // Proving is CPU-bound and single-threaded; serial keeps the box quiet + the run honest.
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI ? [["github"], ["list"]] : [["list"]],
  use: {
    baseURL: BASE_URL,
    // trace on the first retry is CI telemetry: a pass-on-retry is a defect to fix, not a
    // success (determinism doctrine §1.5).
    trace: "on-first-retry",
    // [OPUS-4.8] sq-ymr2e.1 — DETERMINISM defaults that are CONTEXT options, so they apply to
    // EVERY spec (migrated or not) with no per-spec change (research/web-gui-test-program.md §1):
    // a pinned viewport, the site's dark-first colour scheme, reduced motion, and a fixed
    // timezone/locale. Frozen clock + seeded random + animations-off are applied per-page by the
    // support/ auto-fixtures (they need page.clock / addInitScript) — see e2e/support/determinism.ts.
    viewport: { width: 1280, height: 720 },
    colorScheme: "dark",
    timezoneId: "UTC",
    locale: "en-US",
    // reducedMotion is a browser-context option in the test runner (not a top-level `use` key).
    contextOptions: { reducedMotion: "reduce" },
  },
  projects: [
    {
      name: "chromium",
      // Keep the pinned viewport at the project level too (device presets carry their own).
      use: { ...devices["Desktop Chrome"], viewport: { width: 1280, height: 720 } },
    },
  ],
  webServer: {
    // The local Next 15 binary serves the dev route under /sparq; bypass the `dev` npm
    // script (which runs sync-wasm/Typst prebuild steps the ZK route does not need).
    command: `npx next dev -p ${PORT}`,
    url: BASE_URL + "showcase/zk-car-hire/",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
