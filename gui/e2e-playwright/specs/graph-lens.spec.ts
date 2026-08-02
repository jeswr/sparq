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
//   #graph-lens-import                 — the "import a shared lens" textarea
//   [data-graph-lens-status]           — the lens editor's status line (ok | error)
//   [data-graph-lens-run]              — the "Draw lens" button (present iff start+expand filled)
//   [data-graph-lens-run-status]       — the lens RUNTIME status line (info | error)
//   [data-result-view="graph"]         — result container from GraphView
//   [data-graph-node]                  — a focusable node group (resource AND literal)
//   [data-graph-node-panel]            — the focused node's side panel
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

  // A lens is DATA — this is the "someone shared a lens with me" path, end to end, with a
  // store-wiping UPDATE parked in the start slot. The runtime must refuse it BEFORE the engine
  // sees it: the assertion that matters is the last one, that the sample data is still there.
  // Delete the form guard in graph-view-tool.tsx's `runSlot` and this test goes red on it —
  // `DELETE WHERE { ?s ?p ?o }` empties the store and the CONSTRUCT then draws nothing.
  test("an imported lens whose slot is a SPARQL UPDATE is refused, and the store is untouched", async ({
    page,
  }) => {
    await page.locator('[data-tool="graph-view"]').click();
    const panel = page.locator('[data-tool-panel="graph-view"]');
    await expect(panel).toBeVisible();

    await page.locator("[data-graph-lens-toggle]").click();
    await expect(page.locator("[data-graph-lens-panel]")).toBeVisible();

    // The hostile "shared lens": the UI calls the start slot a SELECT, but a lens is just JSON,
    // so nothing stops the blob from carrying an UPDATE there.
    const hostile = JSON.stringify({
      id: "lens:hostile",
      name: "Hostile lens",
      updatedAt: 0,
      queries: {
        start: "DELETE WHERE { ?s ?p ?o }",
        expand: "CONSTRUCT { ?node ?p ?o } WHERE { ?node ?p ?o }",
      },
    });
    await page.locator("#graph-lens-import").fill(hostile);
    await page
      .locator("[data-graph-lens-panel]")
      .getByRole("button", { name: "Import", exact: true })
      .click();
    await expect(page.locator('[data-graph-lens-status="ok"]')).toBeVisible();
    // Importing activates it, so "Draw lens" now runs the hostile start slot.
    await expect(page.locator("[data-graph-active-lens]")).toContainText("Hostile lens");

    await page.locator("[data-graph-lens-run]").click();

    // Refused, and said so — not silently swallowed.
    const runStatus = page.locator('[data-graph-lens-run-status="error"]');
    await expect(runStatus).toBeVisible();
    await expect(runStatus).toContainText(/UPDATE/);
    await expect(runStatus).toContainText(/not run/);

    // The data is still there: run the editor's CONSTRUCT and the sample graph still draws.
    // Had the UPDATE executed, the store would be empty and GraphView would say so instead.
    await page.locator("[data-graph-lens-toggle]").click();
    await page.getByRole("button", { name: "Run" }).click();
    const graphResult = page.locator('[data-result-view="graph"]');
    await expect(graphResult).toBeVisible();
    await expect(graphResult.locator("circle").first()).toBeVisible();
    await expect(graphResult).not.toContainText("Empty graph");
  });

  // Literal nodes are laid out like any other node and are valid focus terms (an expand slot
  // binding ?node in object position expands one), so they carry the same click/keyboard path.
  test("literal nodes are focusable, not just resource nodes", async ({ page }) => {
    await page.locator('[data-tool="graph-view"]').click();
    await expect(page.locator('[data-tool-panel="graph-view"]')).toBeVisible();

    await page.getByRole("button", { name: "Run" }).click();
    const graphResult = page.locator('[data-result-view="graph"]');
    await expect(graphResult).toBeVisible();

    // A literal node is the group drawn as a rounded rect; a resource node is drawn as a circle.
    // The literal one must carry the focus affordance too.
    const literalNode = graphResult.locator("[data-graph-node]:has(rect)").first();
    await expect(literalNode).toBeAttached();
    await expect(literalNode).toHaveAttribute("tabindex", "0");

    // And clicking it actually focuses that node — the side panel opens for the clicked term.
    await literalNode.click();
    await expect(page.locator("[data-graph-node-panel]")).toBeVisible();
  });
});
