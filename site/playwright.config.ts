// [OPUS-4.8] sq-5q63 — Playwright config for the site's headless browser smoke tests.
//
// Scope: the ZK car-hire prover pre-warm smoke test (e2e/zk-prewarm.spec.ts). The test
// drives a REAL browser against a REAL Next.js server so the "first proof pays no cold
// start" claim is verified by an automated run, not just by code reasoning.
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
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
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
