// [SONNET-4.6] sq-kwb74 — RSP-QL Streaming tool tick-view spec, BROWSER persona.
//
// Runs under the chromium-web project (support/web-fixtures.ts): no Tauri globals, the
// deployed-/app code paths — the persona the GUI-consolidation epic (sq-2ucrz) reports
// broken tabs in. The Streaming tool is pure in-tab wasm (no IPC), so the browser persona
// is the binding one to prove.
//
// NON-VACUOUS by construction: the spec first PROBES whether this build ships the W-rsp
// bundle (/wasm/rsp/sparq_rsp_wasm.js) and then REQUIRES the matching panel state —
//   bundle served  → the panel MUST reach data-rsp-status="ready" and produce REAL tick
//                    output (a broken loader showing "unavailable" fails the spec);
//   bundle absent  → the panel MUST show the honest unavailable state (the sq-kwb74
//                    INVARIANT: degrade honestly, never a canned replay dressed as live).
// CI's e2e lane builds the bundle (gui.yml `npm run build:rsp-wasm`), so the happy path is
// the one CI exercises.
//
// Happy-path physics (logical time only — ticks are driven EXPLICITLY, no wall clock):
// tumbling window range=step=60, max_delay=0 over
//   SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }
// push reading=10 @ ts=0 and reading=20 @ ts=30 — both inside [0,60), nothing closes;
// Flush is the explicit end-of-stream tick that closes [0,60) → one window whose real
// engine output is AVG(10,20) = 15 (an xsd:decimal, rendered "15.0"). The exact value is
// asserted: it proves the number came out of the live engine, not a stub.
//
// Stable selectors (declared E2E hooks in streaming-tool.tsx):
//   [data-tool="streaming"]   — left-rail entry that opens the tab
//   [data-rsp-status]         — "loading" | "ready" | "unavailable" | "error"
//   [data-rsp-push-s/-p/-o/-ts] — the push form inputs
//   [data-rsp-push-button] / [data-rsp-flush-button]
//   [data-rsp-window-item]    — one closed window in the tick output
//
// Determinism rules: NO waitForTimeout; web-first assertions; logical timestamps only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

test.describe("streaming-tool", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.
  test.beforeEach(async ({ page }) => {
    await page.locator('[data-tool="streaming"]').click();
  });

  test("tick view renders real RSP-QL window output (or is honestly unavailable without the bundle)", async ({
    page,
  }) => {
    // Probe the served export for the W-rsp bundle — this decides which panel state is the
    // REQUIRED one, so neither branch can pass vacuously.
    const probe = await page.request.get("/wasm/rsp/sparq_rsp_wasm.js");
    const bundleServed = probe.ok();

    const statusEl = page.locator("[data-rsp-status]");
    await expect(statusEl).toBeVisible();

    if (!bundleServed) {
      // INVARIANT: without the bundle the tool degrades honestly — never a canned replay.
      await expect(page.locator('[data-rsp-status="unavailable"]')).toBeVisible();
      await expect(statusEl).toContainText(/RSP bundle is unavailable/i);
      // No fabricated tick output exists.
      await expect(page.locator("[data-rsp-window-item]")).toHaveCount(0);
      return;
    }

    // Bundle served ⇒ the panel MUST come up live ("unavailable" here would be a loader bug).
    await expect(page.locator('[data-rsp-status="ready"]')).toBeVisible();

    // ── Feed 2 events inside the [0,60) window — no tick yet ─────────────────────────────
    await page.locator("[data-rsp-push-s]").fill("<http://ex/s1>");
    await page.locator("[data-rsp-push-p]").fill("<http://ex/reading>");
    await page.locator("[data-rsp-push-o]").fill("10");
    await page.locator("[data-rsp-push-ts]").fill("0");
    await page.locator("[data-rsp-push-button]").click();

    await page.locator("[data-rsp-push-o]").fill("20");
    await page.locator("[data-rsp-push-ts]").fill("30");
    await page.locator("[data-rsp-push-button]").click();

    // Both events are inside the still-open [0,60) window: nothing has closed.
    await expect(page.locator("[data-rsp-window-item]")).toHaveCount(0);

    // ── Explicit tick: Flush closes [0,60) ────────────────────────────────────────────────
    await page.locator("[data-rsp-flush-button]").click();

    const windowItem = page.locator("[data-rsp-window-item]");
    await expect(windowItem).toHaveCount(1);

    // The closed window is [0, 60) and its result is REAL engine output:
    // AVG(10,20) = 15 (xsd:decimal → "15.0"). Asserting the computed value (not just the
    // variable name) is what makes this non-vacuous.
    await expect(windowItem).toContainText("Window [0, 60)");
    await expect(windowItem.locator("td")).toHaveText(/^15(\.0)?$/);
  });
});
