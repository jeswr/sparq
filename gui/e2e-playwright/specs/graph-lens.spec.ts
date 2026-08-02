// [OPUS-5] sq-ixc3.22 — graph LENS journey: the lens editor opens with its five SPARQL slots,
// and the seeded sample graph's RDF 1.2 reifier renders as an EDGE ANNOTATION badge rather than
// as plumbing nodes.
//
// Runs under the chromium-mock-ipc project (Tauri IPC mocked, desktop persona), alongside
// graph-view-tool.spec.ts which owns the plain CONSTRUCT journey.
//
// E2E contract (stable selectors — role/data-* only, never CSS classes):
//   [data-tool="graph-view"]           — the left-rail button that opens the tool tab
//   [data-tool-panel="graph-view"]     — the panel root (graph-view-tool.tsx)
//   [data-graph-lens-toggle]           — the Lens button that swaps in the editor
//   [data-graph-lens-panel]            — the lens editor root (graph-lens-panel.tsx)
//   #graph-lens-pick                   — the lens <select>
//   #graph-lens-start … #graph-lens-nodeDetail — the five slot textareas
//   [data-graph-active-lens]           — the action-row badge naming the active lens
//   [data-result-view="graph"]         — result container from GraphView
//   [data-graph-annotation-legend]     — the RDF 1.2 annotation legend (present iff ≥1 annotated edge)
//   [data-annotated-edge]              — an edge carrying folded reifier annotations
//
// Why the badge is expected: data/sample-graph.ts seeds `ex:knowsClaim rdf:reifies
// <<( ex:alice foaf:knows ex:bob )>>` with two properties, and `ex:alice foaf:knows ex:bob` is
// itself asserted — so the default CONSTRUCT returns both, and GraphView folds the reifier onto
// that edge.
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { test, expect } from "../support/index.ts";

const SLOT_IDS = [
  "#graph-lens-start",
  "#graph-lens-expand",
  "#graph-lens-nodeStyle",
  "#graph-lens-edgeStyle",
  "#graph-lens-nodeDetail",
];

test.describe("graph lens", () => {
  test("the lens editor opens with all five SPARQL slots", async ({ page }) => {
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    // The action row names the lens that is active before anything is configured.
    await expect(page.locator("[data-graph-active-lens]")).toBeVisible();

    await page.locator("[data-graph-lens-toggle]").click();
    await expect(page.locator("[data-graph-lens-panel]")).toBeVisible();

    // The picker offers at least the built-in lens.
    await expect(page.locator("#graph-lens-pick")).toBeVisible();

    // All five slots are editable, and the built-in lens fills the expand slot with a CONSTRUCT
    // mentioning ?node (the focus variable the runtime binds) — non-vacuous.
    for (const id of SLOT_IDS) await expect(page.locator(id)).toBeVisible();
    await expect(page.locator("#graph-lens-expand")).toHaveValue(/CONSTRUCT/i);
    await expect(page.locator("#graph-lens-expand")).toHaveValue(/\?node/);
  });

  test("the sample graph's RDF 1.2 reifier renders as an edge annotation badge", async ({
    page,
  }) => {
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    // Run the default CONSTRUCT WHERE { ?s ?p ?o } over the seeded sample graph.
    const runBtn = page.getByRole("button", { name: "Run" });
    await expect(runBtn).toBeEnabled();
    await runBtn.click();

    const graphResult = page.locator('[data-result-view="graph"]');
    await expect(graphResult).toBeVisible();

    // The legend appears only when at least one edge carries an annotation, and the annotated
    // edge itself is in the SVG — the reifier was folded, not drawn as its own node.
    await expect(page.locator("[data-graph-annotation-legend]")).toBeVisible();
    await expect(graphResult.locator("[data-annotated-edge]").first()).toBeAttached();
  });
});
