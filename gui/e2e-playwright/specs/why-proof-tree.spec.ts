// [FABLE-5] sq-ixc3.20 — click-to-explain inferred facts under the DESKTOP persona
// (mocked Tauri IPC): the why() path is the SAME in-tab W-reason wasm surface on both
// targets (reasoning has no native IPC — see engine-context's sq-tp1m design), so this spec
// is the compact desktop-parity check of the full journey the web spec
// (why-proof-tree.web.spec.ts) covers exhaustively: inferred row → why? → a real proof
// bottoming out in asserted facts; asserted rows carry no affordance.

import { test, expect } from "../support/index.ts";

type Page = import("@playwright/test").Page;

// Seeded via SPARQL UPDATE (the in-tab wasm engine in BOTH personas) rather than the Import
// drawer: the desktop persona's mocked `load_text` IPC returns a canned single-triple
// fixture, so a paste import cannot carry this document there.
const INSERT_CHAIN = `PREFIX ex: <http://example.org/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
INSERT DATA {
  ex:Dog rdfs:subClassOf ex:Mammal .
  ex:Mammal rdfs:subClassOf ex:Animal .
  ex:rex a ex:Dog .
}`;

/** Set a React-controlled textarea via the native setter + input event. */
async function fillReactTextarea(page: Page, selector: string, value: string): Promise<void> {
  await page.evaluate(
    ([sel, text]) => {
      const el = document.querySelector<HTMLTextAreaElement>(sel);
      if (!el) throw new Error(`Textarea not found: ${sel}`);
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLTextAreaElement.prototype,
        "value",
      )?.set;
      if (!setter) throw new Error("Could not get native value setter");
      setter.call(el, text);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    },
    [selector, value] as [string, string],
  );
}

test("desktop persona: an inferred row explains; asserted rows carry no affordance", async ({
  page,
}) => {
  // Seed the multi-step fixture through a SPARQL UPDATE (in-tab wasm engine, no IPC).
  await page.locator('[data-tool="query"]').click();
  await fillReactTextarea(page, "#repl-query", INSERT_CHAIN);
  await page.getByRole("button", { name: "Run query" }).click();
  await expect(page.locator('[data-result-kind="update"]')).toBeVisible({ timeout: 60_000 });

  // RDFS regime on, closure ready (scoped through the inference tab panel — SCOPING NOTE
  // in n3-inference.web.spec.ts).
  await page.locator('[data-tool="inference"]').click();
  const panel = page.locator('[data-tab="inference"]');
  await panel.locator('[data-inference-mode="rdfs"]').click();
  await expect(panel.locator('[data-inference-status="ready"]')).toBeVisible({
    timeout: 60_000,
  });

  // Run SELECT * — the multi-step-entailed typing (rex a Animal) carries why?, the
  // asserted typing (rex a Dog) does not.
  await page.locator('[data-tool="query"]').click();
  await fillReactTextarea(page, "#repl-query", "SELECT ?s ?p ?o WHERE { ?s ?p ?o }");
  await page.getByRole("button", { name: "Run query" }).click();
  const table = page.locator('[data-result-view="table"]');
  await expect(table).toBeVisible({ timeout: 60_000 });

  const entailedRow = table
    .locator("tbody tr")
    .filter({ hasText: "rex" })
    .filter({ hasText: "Animal" });
  await expect(entailedRow).toHaveCount(1);
  const assertedRow = table
    .locator("tbody tr")
    .filter({ hasText: "rex" })
    .filter({ hasText: "Dog" });
  await expect(assertedRow.locator("[data-explain-trigger]")).toHaveCount(0);

  // The proof panel renders a real derivation bottoming out in asserted facts.
  await entailedRow.locator("[data-explain-trigger]").click();
  const proof = page.locator("[data-proof-panel]");
  await expect(proof).toBeVisible();
  await expect(
    proof.locator('[data-proof-node][data-proof-rule="asserted"]').first(),
  ).toBeVisible({ timeout: 30_000 });
  await page.locator("[data-proof-close]").click();
  await expect(proof).toBeHidden();
});
