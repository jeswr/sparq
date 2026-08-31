// [OPUS-5] sq-ixc3.21 — dataset Overview tool, web persona: proves the three summary views are
// computed from REAL aggregate queries over the in-tab WASM store, with NO Tauri backend.
//
// Runs under the chromium-web project (support/web-fixtures.ts): the Tauri globals are never
// installed, so the page takes the pure-browser paths. The webPersona auto-fixture guarantees
// hermetic network, absent Tauri globals, and an engine that has reached "Engine ready" with the
// sample graph loaded before the test body runs.
//
// The sample graph (gui/app/src/data/sample-graph.ts) types four subjects as foaf:Person and
// links them with foaf:knows, so the overview MUST find that class, that self-relationship and
// the foaf:name / foaf:age literal signatures. Those are the non-vacuous anchors below.
//
// Stable selectors:
//   [data-tool="overview"]                    — the left-rail button
//   [data-tool-panel="overview"]              — the panel root
//   [data-overview-section="bubbles"|"chord"|"domain-range"]  — the three views
//   [data-overview-bubble="<label>"]          — one class bubble
//   [data-overview-ribbon="<src>-><tgt>"]     — one chord ribbon
//   [data-overview-instances]                 — the bubble drill-down
//   [data-overview-pair]                      — the ribbon drill-down
//   [data-overview-refresh]                   — the Refresh button
//
// Determinism rules: NO waitForTimeout; web-first assertions only; no exact timing assertions.

import { webTest as test, webExpect as expect } from "../support/index.ts";

test.describe("overview-tool (web persona)", () => {
  test("summarises the live store into bubble, chord and domain-range views", async ({ page }) => {
    await page.locator('[data-tool="overview"]').click();

    const panel = page.locator('[data-tool-panel="overview"]');
    await expect(panel).toBeVisible();

    // The panel computes on open — no click needed. The class bubble for the sample graph's
    // only class must appear, drawn from a REAL COUNT over the store.
    const bubbles = panel.locator('[data-overview-section="bubbles"]');
    await expect(bubbles).toBeVisible();
    const person = panel.locator('[data-overview-bubble="foaf:Person"]');
    await expect(person).toBeVisible();
    await expect(person.locator("circle")).toBeVisible();

    // foaf:knows links Person instances to Person instances → one self-ribbon on the chord.
    const chord = panel.locator('[data-overview-section="chord"]');
    await expect(chord).toBeVisible();
    await expect(
      panel.locator('[data-overview-ribbon="foaf:Person->foaf:Person"]'),
    ).toBeVisible();

    // The observed domain-range signatures include the object property and the literal ones.
    const domainRange = panel.locator('[data-overview-section="domain-range"]');
    await expect(domainRange).toBeVisible();
    await expect(domainRange.getByText("foaf:knows", { exact: true }).first()).toBeVisible();
    await expect(domainRange.getByText("foaf:age", { exact: true }).first()).toBeVisible();

    // Refresh re-runs the queries and the views survive.
    await panel.locator("[data-overview-refresh]").click();
    await expect(person).toBeVisible();
  });

  test("clicking a class bubble drills into its real instances", async ({ page }) => {
    await page.locator('[data-tool="overview"]').click();
    const panel = page.locator('[data-tool-panel="overview"]');
    await expect(panel).toBeVisible();

    await panel.locator('[data-overview-bubble="foaf:Person"]').click();

    const instances = panel.locator("[data-overview-instances]");
    await expect(instances).toBeVisible();
    // ex:alice is a foaf:Person in the sample graph — a real row from the drill-down query.
    await expect(instances.getByText("http://example.org/alice").first()).toBeVisible();
  });

  test("clicking a chord ribbon shows the predicates behind it", async ({ page }) => {
    await page.locator('[data-tool="overview"]').click();
    const panel = page.locator('[data-tool-panel="overview"]');
    await expect(panel).toBeVisible();

    await panel.locator('[data-overview-ribbon="foaf:Person->foaf:Person"]').click();

    const pair = panel.locator("[data-overview-pair]");
    await expect(pair).toBeVisible();
    await expect(pair.getByText("foaf:knows", { exact: true }).first()).toBeVisible();
  });
});
