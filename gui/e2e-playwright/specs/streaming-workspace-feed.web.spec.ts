// [FABLE-5] sq-ixc3.16 — Streaming tool WORKSPACE FEED: a tumbling-window RSP-QL query over
// data brought into the live workspace store, closed purely by workspace updates.
//
// Browser persona (*.web.spec.ts): no Tauri mock; the in-tab WASM engine carries everything.
//
// NON-VACUOUS by construction (same probe discipline as streaming-tool.web.spec.ts): the spec
// first PROBES whether this build ships the W-rsp bundle and then REQUIRES the matching panel
// state — bundle served → the feed MUST stream and the window value MUST be the engine's real
// aggregate; bundle absent → the honest unavailable state.
//
// Happy-path physics (logical time only — one tick per workspace-update batch):
// tumbling window range=step=2 over SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }.
//   tick 0 — INSERT DATA { 10, 20 }   (one update = one batch = one tick; both readings @0)
//   tick 1 — INSERT DATA { 30 }       (inside [0,2); nothing closes: max_delay=0, watermark 1)
//   tick 2 — INSERT DATA { 50 }       (arrival @2 closes [0,2) → AVG(10,20,30) = 20, "20.0")
// The exact aggregate is asserted: it proves the number came out of the live engine fed by the
// WORKSPACE (the Streaming tab is hidden, not unmounted, while the Query tab runs the INSERTs).
//
// New stable selectors (declared in streaming-tool.tsx):
//   [data-rsp-config-range] / [data-rsp-config-step]  — window spec inputs
//   [data-rsp-apply]        — Register (re-register the continuous query)
//   [data-rsp-applied]      — the applied-config echo line
//   [data-rsp-feed-toggle]  — workspace feed on/off
//   [data-rsp-feed-status]  — live feed accounting (tick count, triples streamed)
//
// Determinism rules: NO waitForTimeout; web-first assertions; logical timestamps only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

/** Set the SPARQL editor value via the native setter + input event (React-controlled textarea). */
async function setEditorValue(
  page: import("@playwright/test").Page,
  value: string,
): Promise<void> {
  await page.evaluate((text) => {
    const el = document.querySelector<HTMLTextAreaElement>("#repl-query");
    if (!el) throw new Error("Editor textarea #repl-query not found");
    const setter = Object.getOwnPropertyDescriptor(
      window.HTMLTextAreaElement.prototype,
      "value",
    )?.set;
    if (!setter) throw new Error("Could not access native value setter");
    setter.call(el, text);
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }, value);
}

/** Run one INSERT DATA batch from the Query tab (one update = one feed tick). */
async function insertReadings(
  page: import("@playwright/test").Page,
  triples: string,
): Promise<void> {
  await page.locator('[data-tool="query"]').click();
  await setEditorValue(page, `INSERT DATA { ${triples} }`);
  await page.getByRole("button", { name: "Run query" }).click();
  await expect(page.locator('[data-result-kind="update"]')).toBeVisible();
}

test.describe("streaming-tool workspace feed", () => {
  test("tumbling-window RSP query closes over workspace updates (or honest unavailable)", async ({
    page,
  }) => {
    // Probe the served export for the W-rsp bundle — decides the REQUIRED panel state.
    const probe = await page.request.get("/wasm/rsp/sparq_rsp_wasm.js");
    const bundleServed = probe.ok();

    await page.locator('[data-tool="streaming"]').click();
    const statusEl = page.locator("[data-rsp-status]");
    await expect(statusEl).toBeVisible();

    if (!bundleServed) {
      // INVARIANT: without the bundle the tool degrades honestly — never a canned replay.
      await expect(page.locator('[data-rsp-status="unavailable"]')).toBeVisible();
      await expect(page.locator("[data-rsp-window-item]")).toHaveCount(0);
      return;
    }

    // Bundle served ⇒ the panel MUST come up live.
    await expect(page.locator('[data-rsp-status="ready"]')).toBeVisible();

    // ── Register a tumbling range=step=2 window (default AVG-of-readings query) ───────────
    await page.locator("[data-rsp-config-range]").fill("2");
    await page.locator("[data-rsp-config-step]").fill("2");
    await page.locator("[data-rsp-apply]").click();
    await expect(page.locator("[data-rsp-applied]")).toContainText(
      "Tumbling window — range=2, step=2",
    );

    // ── Enable the workspace feed (baseline = current store; only NEW triples stream) ─────
    await page.locator("[data-rsp-feed-toggle]").click();
    await expect(page.locator("[data-rsp-feed-status]")).toContainText("tick 0");

    // ── tick 0: one update batch carrying TWO readings ─────────────────────────────────────
    await insertReadings(
      page,
      "<http://ex/feed-a> <http://ex/reading> 10 . <http://ex/feed-b> <http://ex/reading> 20 .",
    );
    // The hidden (not unmounted) Streaming panel consumed the update as one tick.
    await page.locator('[data-tool="streaming"]').click();
    await expect(page.locator("[data-rsp-feed-status]")).toContainText("tick 1");
    await expect(page.locator("[data-rsp-feed-status]")).toContainText("2 triples streamed");
    // Both arrivals are inside the still-open [0,2): nothing has closed.
    await expect(page.locator("[data-rsp-window-item]")).toHaveCount(0);

    // ── tick 1: still inside [0,2) (max_delay=0 ⇒ watermark 1 < 2) ────────────────────────
    await insertReadings(page, "<http://ex/feed-c> <http://ex/reading> 30 .");

    // ── tick 2: the arrival @2 closes [0,2) — REAL engine output AVG(10,20,30)=20 ─────────
    await insertReadings(page, "<http://ex/feed-d> <http://ex/reading> 50 .");

    await page.locator('[data-tool="streaming"]').click();
    const windowItem = page.locator("[data-rsp-window-item]");
    await expect(windowItem).toHaveCount(1);
    await expect(windowItem).toContainText("Window [0, 2)");
    // Exact aggregate (xsd:decimal → "20.0"): proves the value came from the live engine fed
    // by workspace updates, not a stub. The reading @2 (50) is in the OPEN [2,4) window and
    // must NOT contaminate the closed one.
    await expect(windowItem.locator("td")).toHaveText(/^20(\.0)?$/);
    await expect(page.locator("[data-rsp-feed-status]")).toContainText("4 triples streamed");
  });
});
