// [SONNET-4.6] sq-ymr2e.5 — extended Playwright `test` for the GUI mocked-IPC lane.
//
// Every spec imports `{ test, expect }` from here (via support/index.ts) rather than from
// @playwright/test directly.  The extension adds ONE auto fixture — `tauriMock` — that wires
// the four invariants required by the determinism doctrine (research/web-gui-test-program.md §1):
//
//   1. IPC mock    — injects window.__TAURI__ + __TAURI_INTERNALS__ before the page scripts
//                    run (page.addInitScript).  isTauriRuntime() returns true; every invoke()
//                    call resolves from the fixture table, never from the native backend.
//
//   2. Hermetic    — blocks all external network requests at the context level.  The in-tab
//                    WASM engine + all Next.js static assets load from 127.0.0.1:3007 (the
//                    serve process); nothing else is allowed through.
//
//   3. Navigate    — goes to "/" after the mock + block are installed.
//
//   4. Engine ready — waits for the literal "Engine ready" copy in the top bar before yielding
//                    control to the test body. Tests start with a warm engine + loaded sample.
//
// All four steps happen before `await use()`, so the test body never races the engine.
//
// [OPUS-5] Step 2 is now an OPTION (`hermeticNetwork`, default `true`) rather than an
// unconditional install, mirroring the reviewed site harness (site/e2e/support/fixtures.ts).
// Every existing spec keeps the block and behaves exactly as before; the opt-out exists for the
// one runtime this repo has already found to be incompatible with blanket `context.route`
// interception — the in-tab bb.js UltraHonk prover, whose spec on the site side
// (site/e2e/zk-prewarm.spec.ts) has carried `test.use({ hermeticNetwork: false })` since the
// e2e foundation landed (PR #1405). The GUI's ZK tool is the same prover, so
// specs/zk-tool.spec.ts needs the same seam.

import { test as base, expect, type Page } from "@playwright/test";
import { tauriMockScript } from "./tauri-mock.ts";
import { waitForEngineReady } from "./gui-ready.ts";

/** The IPC log entry shape exposed on window.__TAURI_IPC_LOG__. */
export interface IpcLogEntry {
  cmd: string;
  args: unknown;
}

/** Options + fixtures added by this extension. */
type GuiMockFixtures = {
  /**
   * Install the blanket hermetic-network `context.route` block at all. Default: `true` — every
   * spec is hermetic unless it says otherwise, and only specs/zk-tool.spec.ts does.
   *
   * Set `false` ONLY for a spec whose runtime is incompatible with blanket `context.route`
   * interception. Today that is exactly one thing: the in-tab `@aztec/bb.js` UltraHonk prover,
   * which the site harness already has to exempt for the same reason
   * (site/e2e/support/fixtures.ts + site/e2e/zk-prewarm.spec.ts, PR #1405). Turning it off is a
   * REAL reduction in that spec's hermeticity, so the spec must say why in its header — it is
   * not a knob to reach for when a locator is flaky.
   */
  hermeticNetwork: boolean;
  /** Auto-fixture: injects the Tauri mock, blocks external network, navigates + waits for engine. */
  tauriMock: void;
};

export const test = base.extend<GuiMockFixtures>({
  hermeticNetwork: [true, { option: true }],

  tauriMock: [
    async ({ page, context, hermeticNetwork }, use) => {
      // ── 1. Hermetic network block (context-level, covers all pages) ──────────────────────────
      // Allow only localhost / 127.0.0.1 (the static file server + blob: URLs the export uses).
      if (hermeticNetwork) {
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
      }

      // ── 2. Inject Tauri IPC mock before any page script runs ─────────────────────────────────
      await page.addInitScript(tauriMockScript);

      // ── 3. Navigate ──────────────────────────────────────────────────────────────────────────
      await page.goto("/");

      // ── 4. Wait for engine ready (WASM instantiation + sample graph load) ───────────────────
      // The TopBar's StatusLed renders "Engine ready" when status.kind === "ready".
      // 90 s is the per-assertion timeout; WASM cold-start on CI can be slow.
      await waitForEngineReady(page, { timeout: 90_000 });

      await use();
    },
    { auto: true },
  ],
});

export { expect };

/**
 * Read the accumulated IPC log from the page.  Call after the action under test to assert
 * which commands the app fired (e.g. assert `disk_usage` was invoked on mount).
 */
export async function readIpcLog(page: Page): Promise<IpcLogEntry[]> {
  return page.evaluate(
    () => (window as unknown as { __TAURI_IPC_LOG__: IpcLogEntry[] }).__TAURI_IPC_LOG__,
  );
}
