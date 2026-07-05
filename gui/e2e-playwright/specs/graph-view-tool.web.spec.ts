// (sq-lxomy) [SONNET-4.6] — graph-view-tool web-persona variant: proves the Graph View tab WORKS
// with NO Tauri backend (pure browser — the deployed /app code path).
//
// Runs under the chromium-web project (support/web-fixtures.ts): window.__TAURI__ /
// __TAURI_INTERNALS__ are never installed, so isTauriRuntime() === false and the page takes the
// pure-browser paths. The webPersona auto-fixture guarantees before the test body runs:
//   * hermetic network (only 127.0.0.1 / localhost / blob: / data:),
//   * Tauri globals genuinely absent,
//   * the in-tab WASM engine reached "Engine ready" (browser-only boot).
//
// Stable selectors (same contract as graph-view-tool.spec.ts):
//   [data-tool="graph-view"]      — the left-rail button
//   [data-tool-panel="graph-view"] — the panel root
//   #graph-view-query             — the CONSTRUCT/DESCRIBE query textarea
//   button "Run"                  — the run trigger
//   [data-result-view="graph"]    — result container from GraphView
//   [data-result-view="graph"] circle — IRI resource nodes
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

test.describe("graph-view-tool (web persona)", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  // (sq-plqfs) [SONNET-4.6] — WorkbenchSparqlEditor renders highlighted tokens in the
  // aria-hidden <pre> layer.  The default CONSTRUCT query contains "CONSTRUCT" and "WHERE"
  // which are tokenised as SPARQL keywords, producing <span class="sq-tok-keyword"> elements.
  test("graph-view editor renders SPARQL keyword highlighting tokens (web persona)", async ({
    page,
  }) => {
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    // WorkbenchSparqlEditor renders highlighted tokens in the aria-hidden <pre> layer.
    // The default CONSTRUCT query contains "CONSTRUCT" and "WHERE" — both are SPARQL keywords.
    const kwSpan = page.locator(".sq-tok-keyword").first();
    await expect(kwSpan).toBeAttached();
    const count = await page.locator(".sq-tok-keyword").count();
    expect(count).toBeGreaterThan(0);
  });

  test("Graph View tab runs CONSTRUCT and renders SVG nodes in the browser persona", async ({
    page,
  }) => {
    // Open the Graph View tool from the left rail — works identically in the browser persona.
    await page.locator('[data-tool="graph-view"]').click();

    const panel = page.locator('[data-tool-panel="graph-view"]');
    await expect(panel).toBeVisible();

    // The query textarea is present with the default CONSTRUCT.
    const textarea = page.locator("#graph-view-query");
    await expect(textarea).toBeVisible();
    await expect(textarea).toHaveValue(/CONSTRUCT/i);

    // Run the default CONSTRUCT over the in-tab WASM engine (no native backend — this is the
    // core browser-persona assertion: the engine runs and returns a graph result).
    const runBtn = page.getByRole("button", { name: "Run" });
    await expect(runBtn).toBeEnabled();
    await runBtn.click();

    // Wait for the GraphView result container.
    const graphResult = page.locator('[data-result-view="graph"]');
    await expect(graphResult).toBeVisible();

    // Non-vacuous: the SVG renders with real IRI resource nodes (circles). The sample graph
    // has ex:alice/bob/carol/dan — all IRI subjects — so at least one circle MUST be present.
    const svg = graphResult.locator("svg");
    await expect(svg).toBeVisible();

    const circles = graphResult.locator("circle");
    await expect(circles.first()).toBeVisible();
  });
});
