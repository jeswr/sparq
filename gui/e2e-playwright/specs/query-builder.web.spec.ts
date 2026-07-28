// [OPUS-5] sq-ixc3.24 — the visual query builder, web persona: proves the diagram-to-SPARQL
// canvas WORKS with NO Tauri backend (pure browser — the deployed /app code path).
//
// Runs under the chromium-web project (support/web-fixtures.ts): window.__TAURI__ /
// __TAURI_INTERNALS__ are never installed, so the panel takes the pure-browser paths. The
// webPersona auto-fixture guarantees before each test body: hermetic network, Tauri globals
// genuinely absent, and the in-tab WASM engine warm.
//
// The assertions are non-vacuous against the seeded sample graph (data/sample-graph.ts): four
// foaf:Person subjects with foaf:name / foaf:age / ex:city literals and foaf:knows links. The
// class picker, the characteristic-set suggestions and the preview rows are all produced by REAL
// SPARQL introspection over that store — nothing in this panel is canned.
//
// Stable selectors (the contract query-builder-tool.tsx documents):
//   [data-tool="query-builder"]        — the left-rail button
//   [data-tool-panel="query-builder"]  — the panel root
//   [data-qb-canvas]                   — the SVG canvas
//   [data-qb-node]                     — a node group
//   [data-qb-suggestion="<iri>"]       — one predicate suggestion
//   #query-builder-sparql              — the generated-SPARQL textarea
//   [data-qb-open-in-query]            — hand off to the Query tool
//   #repl-query                        — the Query tool's editor (the handoff target)
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

const FOAF = "http://xmlns.com/foaf/0.1/";

// Set a React-controlled <textarea> the way the rest of this suite does (workbench-query.spec.ts):
// via the native value setter + an `input` event, so React's synthetic event system sees it.
async function setEditorValue(
  page: import("@playwright/test").Page,
  selector: string,
  value: string,
): Promise<void> {
  await page.evaluate(
    ([sel, text]) => {
      const el = document.querySelector<HTMLTextAreaElement>(sel);
      if (!el) throw new Error(`Editor textarea ${sel} not found`);
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLTextAreaElement.prototype,
        "value",
      )?.set;
      if (!setter) throw new Error("Could not access native value setter");
      setter.call(el, text);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    },
    [selector, value],
  );
}

test.describe("query-builder (web persona)", () => {
  test("drawing a pattern generates editable SPARQL and hands it to the Query tool", async ({
    page,
  }) => {
    await page.locator('[data-tool="query-builder"]').click();

    const panel = page.locator('[data-tool-panel="query-builder"]');
    await expect(panel).toBeVisible();
    await expect(panel.locator("[data-qb-canvas]")).toBeVisible();

    // Introspection of the live store finds foaf:Person — this button IS the query result, so
    // its presence proves classesQuery() ran against the real engine.
    const personClass = panel.getByRole("button", { name: /^Person/ });
    await expect(personClass).toBeVisible();
    await personClass.click();

    // A node appears on the canvas, and the generated SPARQL already constrains its class.
    await expect(panel.locator("[data-qb-node]")).toHaveCount(1);
    const sparql = page.locator("#query-builder-sparql");
    await expect(sparql).toHaveValue(/\?person a foaf:Person/);
    await expect(sparql).toHaveValue(/SELECT \?person/);

    // The characteristic set of foaf:Person, from the data: a literal attribute and an IRI link.
    const nameSuggestion = panel.locator(`[data-qb-suggestion="${FOAF}name"]`);
    const knowsSuggestion = panel.locator(`[data-qb-suggestion="${FOAF}knows"]`);
    await expect(nameSuggestion).toBeVisible();
    await expect(knowsSuggestion).toBeVisible();
    // With no SHACL shapes in the store the provenance must say so honestly — not "shape".
    await expect(nameSuggestion).toContainText("in the data");

    // An all-literal predicate is offered as an attribute filter; an all-IRI one as a link.
    await nameSuggestion.getByRole("button", { name: "filter" }).click();
    await expect(sparql).toHaveValue(/foaf:name \?name/);

    await knowsSuggestion.getByRole("button", { name: "link" }).click();
    await expect(panel.locator("[data-qb-node]")).toHaveCount(2);
    await expect(sparql).toHaveValue(/foaf:knows/);

    // Preview runs the generated text on the in-tab engine and shows REAL rows.
    await panel.getByRole("button", { name: "Preview" }).click();
    const preview = panel.locator("[data-qb-preview]");
    await expect(preview).toBeVisible();
    await expect(preview.locator("table tbody tr").first()).toBeVisible();

    // The handoff: the same text lands in the Query tool's editor, unmodified.
    const generated = await sparql.inputValue();
    await panel.locator("[data-qb-open-in-query]").click();
    const queryEditor = page.locator("#repl-query");
    await expect(queryEditor).toBeVisible();
    await expect(queryEditor).toHaveValue(generated);
  });

  test("an and-not link lowers to FILTER NOT EXISTS and warns about the out-of-scope variable", async ({
    page,
  }) => {
    await page.locator('[data-tool="query-builder"]').click();
    const panel = page.locator('[data-tool-panel="query-builder"]');
    await panel.getByRole("button", { name: /^Person/ }).click();
    await panel.locator(`[data-qb-suggestion="${FOAF}knows"]`).getByRole("button", { name: "link" }).click();

    // Select the link and flip it to the AND-NOT join.
    await panel.locator("[data-qb-edge]").first().click();
    await panel.getByLabel("Join").selectOption("not");

    const sparql = page.locator("#query-builder-sparql");
    await expect(sparql).toHaveValue(/FILTER NOT EXISTS/);
    // The negated leaf cannot be projected — the panel says so rather than emitting a broken query.
    await expect(panel.locator("[data-qb-warning]").first()).toContainText("AND-NOT");
  });

  test("editing the generated SPARQL detaches it from the canvas until Regenerate", async ({
    page,
  }) => {
    await page.locator('[data-tool="query-builder"]').click();
    const panel = page.locator('[data-tool-panel="query-builder"]');
    await panel.getByRole("button", { name: /^Person/ }).click();

    const sparql = page.locator("#query-builder-sparql");
    await expect(sparql).toHaveValue(/foaf:Person/);
    await setEditorValue(page, "#query-builder-sparql", "SELECT * WHERE { ?s ?p ?o } LIMIT 1");

    // The pane admits the text is no longer generated, and a canvas change does NOT clobber it.
    await expect(panel).toContainText("manually edited");
    await panel.getByRole("button", { name: "Add node" }).click();
    await expect(sparql).toHaveValue("SELECT * WHERE { ?s ?p ?o } LIMIT 1");

    // Regenerate is the only thing that discards the manual edit.
    await panel.getByRole("button", { name: "Regenerate" }).click();
    await expect(sparql).toHaveValue(/foaf:Person/);
  });
});
