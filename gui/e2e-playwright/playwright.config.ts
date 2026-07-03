// [SONNET-4.6] sq-ymr2e.5 — Playwright config for the GUI mocked-IPC deterministic lane.
//
// Drives gui/app/out (the Next.js static export) in headless Chromium.  window.__TAURI__ +
// window.__TAURI_INTERNALS__ are stubbed by the auto-fixture in support/fixtures.ts via
// page.addInitScript (the withGlobalTauri injection pattern from research/web-gui-test-program.md
// §5). The in-tab WASM engine (queries / inference) is NOT mocked — it runs for real.
//
// Design rules (§1 determinism doctrine):
//   - NO waitForTimeout — web-first assertions only
//   - 0 retries — flakes are fixed not hidden
//   - 1 worker — serial execution for determinism
//   - External network blocked per test (hermetic)
//   - Stable selectors: role/data-* only, never CSS classes
//   - Timing values (ms, row counts) never asserted exactly — only presence checks

import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./specs",

  // WASM instantiation can be slow; give each assertion generous room.
  timeout: 120_000,
  expect: { timeout: 90_000 },

  // Determinism doctrine §1.1 — 0 retries: a flaky test is a broken test.
  retries: 0,
  // Serial execution keeps results deterministic across the shared WASM tab.
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI ? [["github"], ["list"]] : [["list"]],

  use: {
    baseURL: "http://127.0.0.1:3007",
    trace: "on-first-retry",
    viewport: { width: 1280, height: 720 },
    colorScheme: "dark",
    timezoneId: "UTC",
    locale: "en-US",
    contextOptions: { reducedMotion: "reduce" },
  },

  projects: [
    {
      name: "chromium-mock-ipc",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
      },
    },
  ],

  // Serves gui/app/out (the Tauri-target static export).  -s = SPA mode so every route
  // resolves to index.html (required for Next.js static export with client-side routing).
  // [SONNET-4.6] serve is a pinned devDependency (14.2.6) so npm ci installs it offline;
  // the local binary is used directly to avoid any npx network fetch in CI.
  webServer: {
    command: "./node_modules/.bin/serve -l 3007 -s ../app/out",
    url: "http://127.0.0.1:3007",
    reuseExistingServer: !process.env.CI,
    timeout: 15_000,
  },
});
