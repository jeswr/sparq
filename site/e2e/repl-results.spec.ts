// [OPUS-4.8] sq-x0kp — headless browser smoke test for the /try REPL results panel.
//
// WHAT IT GUARDS. The results panel (src/components/repl.tsx `ResultPanel` / `SelectResult`)
// renders a SELECT result as a TYPED TABLE and offers a raw-SPARQL-JSON view toggle plus
// CSV/TSV/JSON exports. This test drives a REAL headless Chromium against the live REPL: it
// runs the default SELECT example on the wasm engine, asserts the bindings table renders with
// the projected columns, flips the view to raw JSON and asserts a `head`/`results` document
// shows, and asserts ZERO console errors across the whole interaction. The DOM anchors are
// the panel's stable `data-result-kind` / `data-result-view` attributes — no copy-scraping.
//
// It is a CORRECTNESS smoke test, not a benchmark: it asserts the panel's DOM, never a
// wall-clock threshold (timings on a work-box / CI runner are non-canonical).
//
// WASM PREREQ. The REPL loads the lean wasm bundle from `public/wasm/` (a gitignored build
// artifact synced from `js/`'s `build:wasm`). The site-e2e CI lane is deliberately LIGHT (no
// Rust toolchain), so it does not build that bundle; when the bundle is absent this spec
// SKIPS (the same "optional wasm — warn and skip" posture sync-wasm.mjs already takes for the
// reason/rsp/text bundles), keeping the light lane green and fast. Locally — and on any lane
// that builds the bundle first — it runs in full. Run after `npm run sync-wasm`:
//   npx playwright install chromium && npm run test:e2e
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath.
const ROUTE = "try/";

// The lean wasm bundle the REPL loads at runtime. Synced into public/wasm/ by `sync-wasm`
// from the `js/` build; gitignored, so its presence gates this spec.
const WASM_BUNDLE = fileURLToPath(
  new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url),
);

// Skip the whole file when the wasm bundle has not been built/synced — the light CI lane
// does not produce it, and a REPL that cannot load the engine is not the thing under test.
test.skip(
  !existsSync(WASM_BUNDLE),
  "lean wasm bundle absent (public/wasm/sparq_wasm_bg.wasm) — run `npm run sync-wasm` after `(cd ../js && npm run build:wasm)`",
);

/** Collect every `console.error` the page emits, so the test can assert there were none. */
function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

/** Navigate to /try and wait for the wasm engine readiness pill to settle to "ready". */
async function gotoReady(page: Page): Promise<void> {
  await page.goto(ROUTE, { waitUntil: "domcontentloaded" });
  // The engine pre-warms on mount; the pill flips to "Engine ready" when the wasm is loaded.
  await expect(page.getByText("Engine ready")).toBeVisible({ timeout: 90_000 });
}

test("the results panel renders a SELECT table and toggles to raw JSON", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Run the default example (a SELECT). The button label is "Run query" in run mode.
  await page.getByRole("button", { name: "Run query" }).click();

  // The SELECT result panel appears with the typed table view.
  const panel = page.locator('[data-result-kind="select"]');
  await expect(panel).toBeVisible({ timeout: 30_000 });

  const tableView = panel.locator('[data-result-view="table"]');
  await expect(tableView).toBeVisible();
  // It is a real <table> with a header row and at least one body row (the sample graph
  // binds rows for the default query).
  const table = tableView.locator("table");
  await expect(table).toBeVisible();
  await expect(table.locator("thead th").first()).toBeVisible();
  expect(await table.locator("tbody tr").count()).toBeGreaterThan(0);

  // Flip to the raw SPARQL-JSON view via the "JSON" tab.
  await panel.getByRole("tab", { name: "JSON" }).click();
  const jsonView = panel.locator('[data-result-view="json"]');
  await expect(jsonView).toBeVisible();
  // The raw view shows the SPARQL 1.1 JSON document (head + results), pretty-printed.
  await expect(jsonView).toContainText('"head"');
  await expect(jsonView).toContainText('"bindings"');
  // The table view is no longer mounted once JSON is selected.
  await expect(tableView).toHaveCount(0);

  // Flip back to the table — the toggle is non-destructive.
  await panel.getByRole("tab", { name: "Table" }).click();
  await expect(panel.locator('[data-result-view="table"] table')).toBeVisible();

  // The whole interaction produced zero console errors (no wasm/render/runtime faults).
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("an ASK query renders the boolean result, not a table", async ({ page }) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Replace the editor with an ASK query, then run it. The editor (sq-n5aw) is a highlight
  // overlay over a real <textarea> exposed as the accessible "SPARQL query" textbox, so
  // `fill` replaces its whole value.
  await page.getByRole("textbox", { name: "SPARQL query" }).fill("ASK { ?s ?p ?o }");

  await page.getByRole("button", { name: "Run query" }).click();

  const boolPanel = page.locator('[data-result-kind="boolean"]');
  await expect(boolPanel).toBeVisible({ timeout: 30_000 });
  // The sample graph is non-empty, so `ASK { ?s ?p ?o }` is true.
  await expect(boolPanel).toHaveAttribute("data-ask-value", "true");
  // It is NOT rendered as a SELECT table.
  await expect(page.locator('[data-result-kind="select"]')).toHaveCount(0);

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

// [OPUS-4.8] sq-oy1f.7 — the CONSTRUCT result graph's JSON-LD OUTPUT MODES. The graph result
// (`GraphResult` in repl.tsx) defaults to the pretty-Turtle view but offers a format selector
// whose JSON-LD modes serialise the SAME result triples through the wasm engine's own JSON-LD
// writer: "Expanded"/"Flattened"/"Compacted (prefixes)" via `Store.serialize`, and
// "Compaction (@context)" via `Store.serializeCompact` (the full W3C JSON-LD 1.1 Compaction
// against a user-supplied @context, sq-oy1f.5). This drives a CONSTRUCT, switches to the
// expanded JSON-LD form, then to the full-Compaction form with the default @context, asserting
// each renders a real JSON-LD document with zero console errors. Anchored on the stable
// `data-result-kind="graph"` / `data-graph-view="jsonld"` attributes — no copy-scraping.
test("a CONSTRUCT result graph serialises to JSON-LD via the output-format selector", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Pick the built-in CONSTRUCT example (derives a symmetric :friendOf graph), then run it.
  await page.getByRole("button", { name: "CONSTRUCT friend-of graph" }).click();
  await page.getByRole("button", { name: "Run query" }).click();

  // The graph result panel appears; it defaults to the pretty-Turtle view.
  const panel = page.locator('[data-result-kind="graph"]');
  await expect(panel).toBeVisible({ timeout: 30_000 });
  await expect(panel.locator('[data-turtle-view="pretty"]')).toBeVisible();

  // Switch to the expanded JSON-LD output form. The serialise runs on the wasm engine.
  await panel.getByRole("tab", { name: "JSON-LD (expanded)" }).click();
  const jsonldView = panel.locator('[data-graph-view="jsonld"]');
  await expect(jsonldView).toBeVisible({ timeout: 30_000 });
  // The expanded form is an array of node objects keyed by full-IRI predicates with `@id`s.
  await expect(jsonldView).toContainText("@id");
  await expect(jsonldView).toContainText("friendOf");
  // The Turtle view is no longer the active rendering.
  await expect(panel.locator('[data-turtle-view]')).toHaveCount(0);

  // Switch to the full W3C JSON-LD 1.1 Compaction mode — the @context textarea appears, and
  // the document is recompacted against the default @context (which maps `ex:`/foaf terms).
  await panel.getByRole("tab", { name: "JSON-LD (Compaction)" }).click();
  await expect(
    page.getByLabel("JSON-LD @context for full Compaction"),
  ).toBeVisible();
  const compacted = panel.locator('[data-graph-view="jsonld"]');
  await expect(compacted).toBeVisible({ timeout: 30_000 });
  // The compaction collapses the friend-of predicate to a `@context` term and emits `@context`.
  await expect(compacted).toContainText("@context");

  // The whole interaction (two engine serialises) produced zero console errors.
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
