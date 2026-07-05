// [FABLE-5] sq-u0my5 — extended Playwright `test` for the GUI WEB-PERSONA lane (no Tauri mock).
//
// The mock-IPC lane (support/fixtures.ts) injects window.__TAURI__ + __TAURI_INTERNALS__ into
// EVERY spec, so isTauriRuntime() returns true and only the *desktop* persona is ever exercised.
// The pure-browser persona — the deployed /app, where the maintainer reports breakage — had zero
// CI coverage. This fixture is the browser-persona twin: identical hermetic doctrine, but it
// deliberately DOES NOT install the Tauri init script, so the page sees exactly what a real
// browser visitor sees (isTauriRuntime() === false; the in-tab WASM engine carries everything).
//
// The persona difference is the ABSENCE of the Tauri globals, not a different bundle: both
// projects drive the same served root-relative static export (gui/app/out).
//
// The auto fixture wires three of the mock lane's four invariants
// (research/web-gui-test-program.md §1) — the Tauri IPC mock is omitted BY DESIGN:
//
//   1. Hermetic    — blocks all external network requests at the context level. The in-tab
//                    WASM engine + all Next.js static assets load from 127.0.0.1:3007 (the
//                    serve process); nothing else is allowed through.
//
//   2. Navigate    — goes to "/" (no init script installed first).
//
//   3. Engine ready — waits for the literal "Engine ready" copy in the top bar before yielding
//                    control to the test body. The in-tab WASM engine warms + loads the sample
//                    graph with NO Tauri backend — that this succeeds is itself the core
//                    browser-persona boot assertion.
//
// A guard step then asserts the Tauri globals are genuinely absent, so a future harness change
// that leaks the mock into this project fails loudly instead of silently re-testing the desktop
// persona.

import { test as base, expect } from "@playwright/test";
import { waitForEngineReady } from "./gui-ready.ts";

/** Fixtures added by this extension. */
type GuiWebFixtures = {
  /** Auto-fixture: blocks external network, navigates + waits for engine — NO Tauri mock. */
  webPersona: void;
};

export const webTest = base.extend<GuiWebFixtures>({
  webPersona: [
    async ({ page, context }, use) => {
      // ── 1. Hermetic network block (context-level, covers all pages) ──────────────────────────
      // Allow only localhost / 127.0.0.1 (the static file server + blob:/data: URLs the export
      // uses). Same allowlist as the mock-IPC lane — determinism doctrine §1 (hermetic).
      await context.route("**/*", (route) => {
        const url = route.request().url();
        if (
          url.startsWith("http://127.0.0.1") ||
          url.startsWith("http://localhost") ||
          url.startsWith("blob:") ||
          url.startsWith("data:")
        ) {
          return route.continue();
        }
        return route.abort();
      });

      // ── 2. Navigate — deliberately NO page.addInitScript(tauriMockScript) ────────────────────
      await page.goto("/");

      // ── 3. Guard: the Tauri globals must be genuinely absent (browser persona) ──────────────
      const tauriGlobals = await page.evaluate(() => ({
        tauri: "__TAURI__" in window,
        internals: "__TAURI_INTERNALS__" in window,
      }));
      expect(tauriGlobals, "web persona must run with NO Tauri globals").toEqual({
        tauri: false,
        internals: false,
      });

      // ── 4. Wait for engine ready (WASM instantiation + sample graph load, browser-only) ─────
      await waitForEngineReady(page, { timeout: 90_000 });

      await use();
    },
    { auto: true },
  ],
});

export { expect as webExpect };
