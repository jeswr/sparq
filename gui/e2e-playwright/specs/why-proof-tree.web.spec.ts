// [FABLE-5] sq-ixc3.20 — click-to-explain inferred facts: the why() proof-tree panel
// (browser / web persona). The RDFox-console explain-an-inferred-fact parity journey:
//   (a) TABLE view: with RDFS inference on over a MULTI-STEP fixture (a subClassOf chain —
//       closed by the RECURSIVE transitivity rule — plus an instance typing), the inferred
//       rows carry a "why?" affordance and the ASSERTED rows carry NONE; clicking one opens
//       the proof panel showing a multi-step derivation whose internal nodes name the fired
//       rules and whose leaves are `asserted` source facts; a leaf click reveals the full
//       triple; ESC closes.
//   (b) GRAPH view: a CONSTRUCT of the same closure renders the inferred edges dashed with
//       the legend, and clicking an inferred edge opens the same panel. Turning inference
//       Off removes every affordance (asserted-only results carry nothing to explain).
//   (c) [GPT-5.6] The standalone Graph tool, raw N-Triples result lines, and the Inference
//       tool's entailed-facts browser expose the same exact click-to-explain contract.
//
// NON-VACUOUS by design: the gating Playwright lane (gui.yml) BUILDS the W-reason bundle
// (npm run build:reason-wasm, `--features explain`) before exporting the app, so `why()` is
// present and these tests REQUIRE the proof panel to render a real derivation. A missing
// bundle fails this spec loudly — that is a build/sync regression, not a tolerated state.
//
// Stable selectors (sq-ixc3.20 data-* contract):
//   [data-explain-trigger]        — the per-row "why?" button (inferred TABLE rows only)
//   [data-inferred-row]           — a table row whose s/p/o the closure ADDED
//   [data-entailed-edge]          — an inferred (dashed, clickable) GRAPH edge
//   [data-graph-inferred-legend]  — the graph view's inferred-edge legend strip
//   [data-inferred-line] / [data-ntriples-explain] — raw inferred result line + why?
//   [data-entailed-facts-browser] / [data-entailed-fact] — closure-additions browser
//   [data-proof-panel] / [data-proof-node] / [data-proof-rule=…] / [data-proof-note]
//   [data-proof-triple] / [data-proof-full-triple] / [data-proof-close]
//
// SCOPING NOTE: as in n3-inference.web.spec.ts, InferenceControl renders in both the query
// toolbar and the Inference tool panel — mode/status selectors are scoped through
// `[data-tab="inference"]`. Determinism: NO waitForTimeout; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

type Page = import("@playwright/test").Page;
type Locator = import("@playwright/test").Locator;

/** Multi-step RDFS fixture: a two-edge subClassOf CHAIN (its closure needs the recursive
 *  transitivity rule) + one instance typing. Asserted: 3 triples. Entailed (among others):
 *  rex a Mammal, rex a Animal (multi-step), Dog subClassOf Animal (transitive). */
const CHAIN_TTL = `@prefix ex: <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
ex:Dog rdfs:subClassOf ex:Mammal .
ex:Mammal rdfs:subClassOf ex:Animal .
ex:rex a ex:Dog .`;

const SELECT_ALL = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
const CONSTRUCT_REX =
  "PREFIX ex: <http://example.org/> CONSTRUCT { ex:rex ?p ?o } WHERE { ex:rex ?p ?o }";

// ---------------------------------------------------------------------------
// Helpers (mirroring the n3-inference spec's house patterns).
// ---------------------------------------------------------------------------

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

/** Paste-import a Turtle document in REPLACE mode and wait for the ok feedback. */
async function importPasteReplace(page: Page, turtle: string): Promise<void> {
  await page.locator('[data-import-trigger="rail"]').click();
  const drawer = page.locator("[data-import-drawer]");
  await expect(drawer).toBeVisible();
  await drawer.locator('[data-import-tab="paste"]').click();
  await drawer.locator("textarea").first().fill(turtle);
  await drawer.getByRole("button", { name: "Replace store" }).click();
  await drawer.getByRole("button", { name: /Import \(replace store\)/ }).click();
  await expect(drawer.locator('[data-import-feedback="ok"]')).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
}

/** Switch the workspace inference regime via the Inference tool; waits for READY. */
async function selectRdfsReady(page: Page): Promise<Locator> {
  await page.locator('[data-tool="inference"]').click();
  const panel = page.locator('[data-tab="inference"]');
  const btn = panel.locator('[data-inference-mode="rdfs"]');
  await expect(btn).toBeVisible();
  await btn.click();
  await expect(btn).toHaveAttribute("aria-pressed", "true");
  await expect(panel.locator('[data-inference-status="ready"]')).toBeVisible({
    timeout: 60_000,
  });
  return panel;
}

/** Run a query from the Query tool and wait for the given result container. */
async function runQuery(page: Page, query: string, resultSelector: string): Promise<void> {
  await page.locator('[data-tool="query"]').click();
  await fillReactTextarea(page, "#repl-query", query);
  await page.getByRole("button", { name: "Run query" }).click();
  await expect(page.locator(resultSelector)).toBeVisible({ timeout: 60_000 });
}

test.describe("why-proof-tree", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  test("(a) table view: inferred rows explain, asserted rows carry no affordance", async ({
    page,
  }) => {
    await importPasteReplace(page, CHAIN_TTL);
    await selectRdfsReady(page);
    await runQuery(page, SELECT_ALL, '[data-result-view="table"]');

    const table = page.locator('[data-result-view="table"]');

    // Inferred rows exist and carry the why? affordance (rex a Mammal, rex a Animal, the
    // transitive Dog ⊑ Animal, …) — and NOT every row has it (the 3 asserted facts don't).
    const triggers = table.locator("[data-explain-trigger]");
    await expect
      .poll(async () => await triggers.count(), { timeout: 10_000 })
      .toBeGreaterThanOrEqual(2);
    const inferredRows = await table.locator("[data-inferred-row]").count();
    const allRows = await table.locator("tbody tr").count();
    expect(inferredRows).toBeGreaterThanOrEqual(2);
    expect(allRows).toBeGreaterThan(inferredRows);

    // The ASSERTED typing (rex a Dog) has NO affordance; the ENTAILED typing (rex a Animal —
    // a MULTI-STEP derivation through the subclass chain) has one.
    const assertedRow = table.locator("tbody tr").filter({ hasText: "rex" }).filter({ hasText: "Dog" });
    await expect(assertedRow).toHaveCount(1);
    await expect(assertedRow.locator("[data-explain-trigger]")).toHaveCount(0);
    const entailedRow = table
      .locator("tbody tr")
      .filter({ hasText: "rex" })
      .filter({ hasText: "Animal" });
    await expect(entailedRow).toHaveCount(1);

    // Click why? → the proof panel opens with a real multi-step derivation.
    await entailedRow.locator("[data-explain-trigger]").click();
    const panel = page.locator("[data-proof-panel]");
    await expect(panel).toBeVisible();
    // The honesty note: one witness, leaves asserted.
    await expect(panel.locator("[data-proof-note]")).toContainText("witness");
    // Multi-step: at least 3 nodes, ≥1 real rule firing, ≥1 asserted leaf.
    const nodes = panel.locator("[data-proof-node]");
    await expect
      .poll(async () => await nodes.count(), { timeout: 30_000 })
      .toBeGreaterThanOrEqual(3);
    expect(
      await panel.locator('[data-proof-node][data-proof-rule="asserted"]').count(),
    ).toBeGreaterThanOrEqual(1);
    expect(
      await panel
        .locator("[data-proof-node]:not([data-proof-rule='asserted'])")
        .count(),
    ).toBeGreaterThanOrEqual(2);

    // "Each leaf clickable to the triple": clicking a step's caption reveals the FULL
    // N-Triples line of its concluded triple.
    await panel.locator("[data-proof-triple]").first().click();
    await expect(panel.locator("[data-proof-full-triple]")).toBeVisible();
    await expect(panel.locator("[data-proof-full-triple]")).toContainText("<http://");

    // ESC closes the panel.
    await page.keyboard.press("Escape");
    await expect(panel).toBeHidden();
  });

  test("(b) graph view: inferred edges are marked + explain; Off removes every affordance", async ({
    page,
  }) => {
    await importPasteReplace(page, CHAIN_TTL);
    const inferencePanel = await selectRdfsReady(page);
    await runQuery(page, CONSTRUCT_REX, '[data-result-view="graph"]');

    // The inferred edges (rex → Mammal, rex → Animal) render dashed with the legend.
    await expect(page.locator("[data-graph-inferred-legend]")).toBeVisible();
    const edges = page.locator("[data-entailed-edge]");
    await expect
      .poll(async () => await edges.count(), { timeout: 10_000 })
      .toBeGreaterThanOrEqual(2);

    // Clicking an inferred edge opens the proof panel, bottoming out in asserted facts.
    await edges.first().click();
    const panel = page.locator("[data-proof-panel]");
    await expect(panel).toBeVisible();
    await expect(
      panel.locator('[data-proof-node][data-proof-rule="asserted"]').first(),
    ).toBeVisible({ timeout: 30_000 });
    await page.locator("[data-proof-close]").click();
    await expect(panel).toBeHidden();

    // Inference OFF → the same CONSTRUCT is asserted-only: no legend, no inferred edges,
    // and (re-running the table query) no why? affordance anywhere.
    await page.locator('[data-tool="inference"]').click();
    await inferencePanel.locator('[data-inference-mode="off"]').click();
    await runQuery(page, CONSTRUCT_REX, '[data-result-view="graph"]');
    await expect(page.locator("[data-graph-inferred-legend]")).toHaveCount(0);
    await expect(page.locator("[data-entailed-edge]")).toHaveCount(0);
    await runQuery(page, SELECT_ALL, '[data-result-view="table"]');
    await expect(page.locator("[data-explain-trigger]")).toHaveCount(0);
  });

  test("(c) Graph tool, N-Triples lines, and entailed-facts browser explain exact additions", async ({
    page,
  }) => {
    // [GPT-5.6] One multi-step closure drives all three newly-covered surfaces.
    await importPasteReplace(page, CHAIN_TTL);
    const inferencePanel = await selectRdfsReady(page);

    // Inference tool: browse closure additions only; asserted rex a Dog is absent, while
    // inferred rex a Animal is present and opens a real proof.
    const browser = inferencePanel.locator("[data-entailed-facts-browser]");
    await expect(browser).toBeVisible();
    const entailedFacts = browser.locator("[data-entailed-fact]");
    await expect
      .poll(async () => await entailedFacts.count(), { timeout: 10_000 })
      .toBeGreaterThanOrEqual(2);
    await expect(
      entailedFacts.filter({ hasText: "rex" }).filter({ hasText: "Dog" }),
    ).toHaveCount(0);
    const browserAnimal = entailedFacts
      .filter({ hasText: "rex" })
      .filter({ hasText: "Animal" });
    await expect(browserAnimal).toHaveCount(1);
    await browserAnimal.locator("[data-entailed-fact-explain]").click();
    let proof = page.locator("[data-proof-panel]");
    await expect(
      proof.locator('[data-proof-node][data-proof-rule="asserted"]').first(),
    ).toBeVisible({ timeout: 30_000 });
    await proof.locator("[data-proof-close]").click();

    // Standalone Graph tool: its own run result receives the inferred-edge prop and hosts the
    // same proof overlay (the original follow-up's one-prop wiring gap).
    await page.locator('[data-tool="graph-view"]').click();
    const graphTool = page.locator('[data-tool-panel="graph-view"]');
    await expect(graphTool).toBeVisible();
    await fillReactTextarea(page, "#graph-view-query", CONSTRUCT_REX);
    await graphTool.getByRole("button", { name: "Run", exact: true }).click();
    await expect(graphTool.locator("[data-graph-inferred-legend]")).toBeVisible({
      timeout: 60_000,
    });
    const toolEdges = graphTool.locator("[data-entailed-edge]");
    await expect
      .poll(async () => await toolEdges.count(), { timeout: 10_000 })
      .toBeGreaterThanOrEqual(2);
    await toolEdges.first().click();
    proof = graphTool.locator("[data-proof-panel]");
    await expect(
      proof.locator('[data-proof-node][data-proof-rule="asserted"]').first(),
    ).toBeVisible({ timeout: 30_000 });
    await proof.locator("[data-proof-close]").click();

    // Query text view: raw N-Triples preserves the one-line-one-fact identity. Inferred lines
    // are marked and explainable; the asserted rex a Dog line has neither marker nor button.
    // The Query tool remembers its selected result tab; wait for the graph result itself rather
    // than a table-view-only wrapper before switching explicitly to the text view.
    await runQuery(page, CONSTRUCT_REX, '[data-tab="query"] [data-result-view="graph"]');
    await page.getByRole("button", { name: "N-Triples / Turtle", exact: true }).click();
    await page.getByRole("button", { name: "N-Triples", exact: true }).click();
    const raw = page.locator("[data-ntriples-lines]");
    await expect(raw).toBeVisible();
    const inferredLines = raw.locator("[data-inferred-line]");
    await expect
      .poll(async () => await inferredLines.count(), { timeout: 10_000 })
      .toBeGreaterThanOrEqual(2);
    const assertedLine = raw
      .locator(":scope > div")
      .filter({ hasText: "rex" })
      .filter({ hasText: "Dog" });
    await expect(assertedLine).toHaveCount(1);
    await expect(assertedLine.locator("[data-ntriples-explain]")).toHaveCount(0);
    const rawAnimal = inferredLines
      .filter({ hasText: "rex" })
      .filter({ hasText: "Animal" });
    await expect(rawAnimal).toHaveCount(1);
    await rawAnimal.locator("[data-ntriples-explain]").click();
    proof = page.locator('[data-tab="query"] [data-proof-panel]');
    await expect(
      proof.locator('[data-proof-node][data-proof-rule="asserted"]').first(),
    ).toBeVisible({ timeout: 30_000 });
  });
});
