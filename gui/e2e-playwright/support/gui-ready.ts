// [SONNET-4.6] sq-ymr2e.5 — shared "wait for engine ready" helper used by the auto-fixture
// and by any spec that needs to assert against the initial ready state independently.
//
// The TopBar renders the literal text "Engine ready" inside a <span> when the WASM engine's
// status is { kind: "ready" } (gui/app/src/components/workbench/top-bar.tsx: StatusLed).
// That text is the stable, declared E2E hook — the tauri-driver harness (gui/e2e/run-e2e.mjs)
// uses the same anchor, so it cannot drift silently.

import type { Page } from "@playwright/test";

/** The literal string the TopBar's StatusLed renders when the engine is warm. */
const ENGINE_READY_TEXT = "Engine ready";

/**
 * Wait for the in-tab WASM engine to finish warming.  The engine loads the sample graph and
 * reaches { kind: "ready" } asynchronously; the top bar's StatusLed then renders the literal
 * "Engine ready" copy.  WASM instantiation can be slow, so a generous timeout is used.
 *
 * NEVER uses waitForTimeout — this is a web-first assertion on a visible UI state.
 */
export async function waitForEngineReady(
  page: Page,
  options: { timeout?: number } = {},
): Promise<void> {
  const timeout = options.timeout ?? 90_000;
  await page
    .getByText(ENGINE_READY_TEXT, { exact: true })
    .waitFor({ state: "visible", timeout });
}
