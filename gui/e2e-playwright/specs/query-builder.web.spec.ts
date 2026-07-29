// [OPUS-5] sq-ixc3.24 — visual query builder spec (web persona).
//
// Drives the Builder tool (gui/app/src/components/workbench/query-builder-tool.tsx) against the
// REAL in-tab WASM engine over the seeded sample graph — no mocks, no canned suggestions. The
// pure model/serializer/validation logic is unit-tested in gui/app/src/lib/query-builder.test.ts;
// this spec covers what only the rendered tool can prove:
//
//   * schema introspection actually queries the live store (the class picker fills from data);
//   * drawing a node produces the corresponding STANDARD SPARQL in a visible, editable editor;
//   * the honesty labels are the real state — "no shapes in the store" for a store that holds
//     none, and a `data` provenance badge on an observed suggestion;
//   * hand-editing the SPARQL DETACHES it from the canvas and says so;
//   * "Open in Query" hands the exact text to the Query tool's editor.
//
// Stable selectors (the panel's E2E contract):
//   [data-tool="builder"]              — left-rail nav button
//   [data-tool-panel="builder"]        — panel root
//   [data-builder-schema]              — idle | loading-less ready | error (introspection state)
//   [data-builder-add-node]            — toolbar "Node"
//   [data-builder-node="<var>"]        — a node card on the canvas
//   [data-builder-node-inspector]      — the selected node's inspector
//   [data-builder-suggestions="attribute"] — the shape/data-driven predicate picker
//   #builder-sparql                    — the generated (editable) SPARQL textarea
//   [data-builder-issues]              — none | warning | error
//   [data-builder-regenerate]          — "Regenerate from canvas" (only when detached)
//   [data-builder-run] / [data-builder-preview] — run + inline preview
//   [data-builder-send]                — "Open in Query"
//   #repl-query                        — the Query tool's editor (hand-off target)
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

/** Open the Builder tab and wait for its (lazily-chunked) panel to mount. */
async function openBuilder(page: import("@playwright/test").Page) {
  await page.locator('[data-tool="builder"]').click();
  await expect(page.locator('[data-tool-panel="builder"]')).toBeVisible();
}

/** Set a React-controlled textarea's value through the native setter. */
async function setTextarea(
  page: import("@playwright/test").Page,
  selector: string,
  value: string,
): Promise<void> {
  await page.locator(selector).waitFor();
  await page.evaluate(
    ({ sel, text }) => {
      const el = document.querySelector<HTMLTextAreaElement>(sel);
      if (!el) throw new Error(`Textarea ${sel} not found`);
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLTextAreaElement.prototype,
        "value",
      )?.set;
      if (!setter) throw new Error("Could not access native value setter");
      setter.call(el, text);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    },
    { sel: selector, text: value },
  );
}

test.describe("query builder (web persona)", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  test("introspects the live store and fills the class picker from real data", async ({ page }) => {
    await openBuilder(page);

    // The sample graph is small, so introspection runs on open. It must reach `ready` — the
    // suggestions come from queries over the store, so a failure is reported, never papered over.
    await expect(page.locator("[data-builder-schema]")).toHaveAttribute("data-builder-schema", "ready");

    // The sample graph carries NO SHACL shapes, and the badge must say exactly that rather than
    // implying the pickers are shape-driven.
    await expect(page.locator('[data-builder-schema="ready"]')).toContainText("no shapes in the store");

    // The class select is populated from `?s a ?class` over the live store — foaf:Person is in it.
    await page.locator("[data-builder-add-node]").click();
    await expect(page.locator("#builder-node-class")).toContainText("foaf:Person");
  });

  test("drawing a node emits standard SPARQL into the visible, editable editor", async ({ page }) => {
    await openBuilder(page);
    await expect(page.locator("[data-builder-schema]")).toHaveAttribute("data-builder-schema", "ready");

    await page.locator("[data-builder-add-node]").click();

    // The node lands on the canvas, typed with the most populous class from the live store.
    await expect(page.locator('[data-builder-node="person"]')).toBeVisible();

    // …and the generated SPARQL is plain SPARQL 1.1: a declared prefix, a SELECT, a type triple.
    const editor = page.locator("#builder-sparql");
    await expect(editor).toBeVisible();
    await expect(editor).toHaveValue(/PREFIX foaf: <http:\/\/xmlns\.com\/foaf\/0\.1\/>/);
    await expect(editor).toHaveValue(/SELECT \?person/);
    await expect(editor).toHaveValue(/\?person a foaf:Person \./);

    // A clean diagram reports no issues — and the editor is genuinely editable (not readonly).
    await expect(page.locator("[data-builder-issues]")).toHaveAttribute("data-builder-issues", "none");
    await expect(editor).toBeEditable();
  });

  test("an attribute picked from the observed schema is labelled `data`, not invented", async ({
    page,
  }) => {
    await openBuilder(page);
    await expect(page.locator("[data-builder-schema]")).toHaveAttribute("data-builder-schema", "ready");
    await page.locator("[data-builder-add-node]").click();

    // The picker is driven by a characteristic-set query over the live store, so every entry it
    // offers carries the provenance badge that says where it came from.
    const picker = page.locator('[data-builder-suggestions="attribute"]');
    await expect(picker).toBeVisible();
    const nameOption = picker.getByRole("button", { name: /foaf:name/ });
    await expect(nameOption).toContainText("data");

    await nameOption.click();
    await expect(page.locator("#builder-sparql")).toHaveValue(/\?person foaf:name \?person\w+ \./);
  });

  test("running the drawn query previews real rows from the live store", async ({ page }) => {
    await openBuilder(page);
    await expect(page.locator("[data-builder-schema]")).toHaveAttribute("data-builder-schema", "ready");
    await page.locator("[data-builder-add-node]").click();
    await page.locator('[data-builder-suggestions="attribute"]').getByRole("button", { name: /foaf:name/ }).click();

    await page.locator("[data-builder-run]").click();

    // A real SELECT over the seeded sample — the preview renders the engine's own rows.
    const preview = page.locator('[data-builder-preview="select"]');
    await expect(preview).toBeVisible();
    await expect(preview).toContainText("Alice");
  });

  test("hand-editing the SPARQL detaches it from the canvas, and the panel says so", async ({
    page,
  }) => {
    await openBuilder(page);
    await expect(page.locator("[data-builder-schema]")).toHaveAttribute("data-builder-schema", "ready");
    await page.locator("[data-builder-add-node]").click();

    // The canvas cannot parse SPARQL back into a diagram, so an edit must be declared, not hidden.
    await expect(page.locator("[data-builder-regenerate]")).toHaveCount(0);
    await setTextarea(page, "#builder-sparql", "SELECT * WHERE { ?s ?p ?o } LIMIT 1");
    // `exact` scopes this to the editor header's BADGE. A substring match also catches the
    // issues pane's "The SPARQL has been edited by hand, …" note — two elements, and Playwright's
    // strict mode (rightly) refuses to guess which one the assertion meant.
    await expect(page.getByText("edited by hand", { exact: true })).toBeVisible();

    const regenerate = page.locator("[data-builder-regenerate]");
    await expect(regenerate).toBeVisible();
    await regenerate.click();
    await expect(page.locator("#builder-sparql")).toHaveValue(/\?person a foaf:Person \./);
  });

  test("`Open in Query` hands the generated SPARQL to the Query tool's editor", async ({ page }) => {
    await openBuilder(page);
    await expect(page.locator("[data-builder-schema]")).toHaveAttribute("data-builder-schema", "ready");
    await page.locator("[data-builder-add-node]").click();

    const generated = await page.locator("#builder-sparql").inputValue();
    await page.locator("[data-builder-send]").click();

    // The Query tab is focused and carries the SAME text — the hand-off is verbatim.
    await expect(page.locator("#repl-query")).toBeVisible();
    await expect(page.locator("#repl-query")).toHaveValue(generated);
  });
});
