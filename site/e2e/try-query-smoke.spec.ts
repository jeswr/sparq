// [OPUS-4.8] sq-jp7ry (issue #835) — CRITICAL-FLOW smoke test for the /try SPARQL playground.
//
// WHAT IT GUARDS. The /try REPL (src/components/repl.tsx) is the site's killer artifact: edit a
// query, hit Run, get answers from the real Rust engine compiled to wasm, in-tab. This is the
// most basic end-to-end the playground must always satisfy — type the trivial all-triples query
// `SELECT * WHERE { ?s ?p ?o }`, run it against the bundled sample graph, and get a NON-EMPTY
// results table back. It complements repl-results.spec.ts (which drives the built-in default
// example + the view toggles): this one types the query the brief calls out verbatim, so it
// guards the raw "editor → run → table" path the way a first-time visitor exercises it, and
// asserts ZERO console errors across the interaction (the wasm-fault net for issue #835).
//
// WASM PREREQ. The REPL loads the lean wasm bundle from `public/wasm/` (a gitignored build
// artifact synced from `js`'s `build:wasm`). The light site-e2e CI lane has no Rust toolchain
// to build it, so this whole spec SKIPS when the bundle is absent (the same posture as
// repl-results.spec.ts / shacl-validator.spec.ts) and runs in full when present. Run after
// `npm run sync-wasm`:  npx playwright install chromium && npm run test:e2e
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath.
const ROUTE = "try/";

// The lean wasm bundle the REPL loads at runtime. Synced into public/wasm/ by `sync-wasm` from
// the `js` build; gitignored, so its presence gates this spec.
const WASM_BUNDLE = fileURLToPath(
  new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url),
);

// Skip the whole file when the wasm bundle has not been built/synced — the light CI lane does
// not produce it, and a REPL that cannot load the engine is not the thing under test.
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

test("the trivial all-triples SELECT runs on the bundled sample and renders a non-empty table", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Type the brief's trivial query verbatim, replacing the editor's default. The editor (sq-n5aw)
  // is a highlight overlay over a real <textarea> exposed as the accessible "SPARQL query"
  // textbox, so `fill` replaces its whole value.
  await page
    .getByRole("textbox", { name: "SPARQL query" })
    .fill("SELECT * WHERE { ?s ?p ?o }");

  // Run it. The button label is "Run query" in run mode.
  await page.getByRole("button", { name: "Run query" }).click();

  // The SELECT result panel appears with the typed table view (the bundled sample graph is
  // non-empty, so `?s ?p ?o` binds rows for every triple).
  const panel = page.locator('[data-result-kind="select"]');
  await expect(panel).toBeVisible({ timeout: 30_000 });

  const table = panel.locator('[data-result-view="table"] table');
  await expect(table).toBeVisible();
  // Header columns for the projected variables (?s ?p ?o) and at least one body row.
  await expect(table.locator("thead th").first()).toBeVisible();
  expect(await table.locator("tbody tr").count()).toBeGreaterThan(0);

  // The whole interaction produced zero console errors (no wasm/render/runtime faults).
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
