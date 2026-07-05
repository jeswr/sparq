// (sq-lxomy) [SONNET-4.6] — graph-view-tool journey: open the tool, run the default CONSTRUCT
// over the sample graph, assert SVG nodes/edges appear (non-vacuous: real rendered output).
//
// Tests run under the chromium-mock-ipc project (Tauri IPC mocked, desktop persona). A parallel
// web-persona variant lives in graph-view-tool.web.spec.ts (chromium-web project, no Tauri mock).
//
// E2E contract (stable selectors — role/data-* only, never CSS classes):
//   [data-tool="graph-view"]    — the left-rail button that opens the tool tab
//   [data-tool-panel="graph-view"] — the panel root (graph-view-tool.tsx)
//   #graph-view-query           — the CONSTRUCT/DESCRIBE query textarea
//   button with text "Run"      — the run trigger (getByRole)
//   [data-result-view="graph"]  — result container from GraphView (graph-view.tsx)
//   [data-result-view="graph"] svg — the rendered SVG (present whenever triples > 0)
//   [data-result-view="graph"] circle — IRI resource nodes
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { test, expect } from "../support/index.ts";

test.describe("graph-view-tool", () => {
  // The tauriMock auto-fixture navigates to "/" and waits for engine ready before each test.

  test("tool tab opens and shows the query textarea", async ({ page }) => {
    // Open the Graph View tool from the left rail.
    await page.locator('[data-tool="graph-view"]').click();

    // The panel root renders.
    const panel = page.locator('[data-tool-panel="graph-view"]');
    await expect(panel).toBeVisible();

    // The query textarea is present and contains the default CONSTRUCT.
    const textarea = page.locator("#graph-view-query");
    await expect(textarea).toBeVisible();
    await expect(textarea).toHaveValue(/CONSTRUCT/i);
  });

  test("default CONSTRUCT over sample graph renders SVG nodes", async ({ page }) => {
    // Open the Graph View tool.
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    // Run the default CONSTRUCT WHERE { ?s ?p ?o } LIMIT 100 over the seeded sample graph.
    const runBtn = page.getByRole("button", { name: "Run" });
    await expect(runBtn).toBeEnabled();
    await runBtn.click();

    // Wait for the GraphView result container to appear (data-result-view="graph" from graph-view.tsx).
    const graphResult = page.locator('[data-result-view="graph"]');
    await expect(graphResult).toBeVisible();

    // Non-vacuous: the SVG renders — the sample graph has IRI resource nodes (rendered as
    // <circle> elements) and literal value nodes (rendered as <rect> elements). Assert that at
    // least one circle is present (Alice/Bob/Carol/Dan are all IRI subjects).
    const svg = graphResult.locator("svg");
    await expect(svg).toBeVisible();

    // At least one IRI node (circle) must appear — the sample graph has ex:alice/bob/carol/dan.
    const circles = graphResult.locator("circle");
    await expect(circles.first()).toBeVisible();
  });

  // (sq-plqfs) [SONNET-4.6] — WorkbenchSparqlEditor renders highlighted tokens in the
  // aria-hidden <pre> layer.  The default CONSTRUCT query contains "CONSTRUCT" and "WHERE"
  // which are tokenised as SPARQL keywords, producing <span class="sq-tok-keyword"> elements.
  test("graph-view editor renders SPARQL keyword highlighting tokens", async ({ page }) => {
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    // WorkbenchSparqlEditor renders highlighted tokens in the aria-hidden <pre> layer.
    // The default CONSTRUCT query contains "CONSTRUCT" and "WHERE" — both are SPARQL keywords.
    const kwSpan = page.locator(".sq-tok-keyword").first();
    await expect(kwSpan).toBeAttached();
    const count = await page.locator(".sq-tok-keyword").count();
    expect(count).toBeGreaterThan(0);
  });

  test("error result on invalid SPARQL", async ({ page }) => {
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    // Replace the query with invalid SPARQL via the native setter (React-controlled textarea).
    await page.evaluate(() => {
      const el = document.querySelector<HTMLTextAreaElement>("#graph-view-query");
      if (!el) throw new Error("graph-view-query textarea not found");
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLTextAreaElement.prototype,
        "value",
      )?.set;
      if (!setter) throw new Error("Could not access native value setter");
      setter.call(el, "NOT VALID SPARQL AT ALL");
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });

    const runBtn = page.getByRole("button", { name: "Run" });
    await runBtn.click();

    // The engine rejects the query and an error is shown.
    const errorEl = page.locator('[data-result-kind="error"]');
    await expect(errorEl).toBeVisible();
  });
});
