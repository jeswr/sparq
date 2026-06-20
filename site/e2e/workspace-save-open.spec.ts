// [OPUS-4.8] sq-800o (epic sq-ixc3) — headless browser smoke test for the /try REPL
// workspace SAVE / OPEN flow (the persistence model from sq-atb0 / #965).
//
// WHAT IT GUARDS. The workspace panel (src/components/workspace-panel.tsx, wired into
// src/components/repl.tsx) lets a user save the current REPL state (dataset snapshot + editor
// query) under a name and re-open it later. On GitHub-Pages / static hosting the backend is
// localStorage ("Saved in this browser"). This test drives a REAL headless Chromium: it types a
// DISTINCTIVE query into the editor, Save-as a named workspace, switches the picker to a fresh
// scratch session (which resets the editor to the default example), then re-opens the saved
// workspace from the picker and asserts the editor query is restored verbatim. It also asserts
// ZERO console errors across the interaction.
//
// It is a CORRECTNESS smoke test, not a benchmark — it asserts restored DOM/editor state, never
// a wall-clock threshold (timings on a work-box / CI runner are non-canonical).
//
// DOM ANCHORS. Reuses the component's own stable controls (no new test-ids needed): the
// "SPARQL query" textbox (the controlled editor `<textarea>`), the `#repl-workspace` workspace
// picker `<select>`, and the Save-as dialog's "Save as" button / "Workspace name" input /
// "Save workspace" confirm — all by accessible role/label.
//
// DETERMINISM. localStorage is per-origin and persists across tests in the Playwright context,
// so we `localStorage.clear()` right after the first navigation to start from a known-empty
// store. The save/open path does not need the wasm engine for the EDITOR-state restore we
// assert (the dataset snapshot is empty on a fresh session), but the REPL pre-warms the engine
// on mount and the panel's controls enable once the persistence backend resolves; we wait on
// the readiness pill so the run is stable. The spec still skips when the wasm bundle is absent
// (the light CI lane), matching the sibling specs, so the whole suite has one consistent gate.
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath.
const ROUTE = "try/";

const WASM_BUNDLE = fileURLToPath(
  new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url),
);

test.skip(
  !existsSync(WASM_BUNDLE),
  "wasm bundle absent (public/wasm/sparq_wasm_bg.wasm) — run `npm run sync-wasm` after `(cd ../js && npm run build:wasm)`",
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

/** Navigate to /try, clear localStorage for determinism, and wait for the engine pill. */
async function gotoReadyClean(page: Page): Promise<void> {
  await page.goto(ROUTE, { waitUntil: "domcontentloaded" });
  // Start from an empty per-origin workspace store so the picker has no stale saved entries.
  await page.evaluate(() => localStorage.clear());
  await page.reload({ waitUntil: "domcontentloaded" });
  await expect(page.getByText("Engine ready")).toBeVisible({ timeout: 90_000 });
  // The picker enables once the persistence backend resolves (web/localStorage here).
  await expect(page.locator("#repl-workspace")).toBeEnabled({ timeout: 30_000 });
}

test("a saved workspace restores the editor query when re-opened", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReadyClean(page);

  const editor = page.getByRole("textbox", { name: "SPARQL query" });
  const picker = page.locator("#repl-workspace");

  // A DISTINCTIVE query so the restore is unambiguous (and differs from the default example).
  const distinctive = "SELECT ?marker WHERE { BIND(42 AS ?marker) }";
  await editor.fill(distinctive);
  await expect(editor).toHaveValue(distinctive);

  // Save it as a new named workspace via the Save-as dialog.
  await page.getByRole("button", { name: "Save as" }).click();
  const nameInput = page.getByRole("textbox", { name: "Workspace name" });
  await expect(nameInput).toBeVisible();
  const wsName = "E2E restore probe";
  await nameInput.fill(wsName);
  await page.getByRole("button", { name: "Save workspace" }).click();

  // The save makes it the OPEN workspace: the picker now lists the new option (its label is the
  // name plus a ` · N triples` suffix, so we key off the stable option VALUE = the workspace id).
  const savedOption = picker.locator("option", { hasText: wsName });
  await expect(savedOption).toHaveCount(1);
  const wsValue = (await savedOption.getAttribute("value")) ?? "";
  expect(wsValue).not.toBe("");
  await expect(picker).toHaveValue(wsValue);

  // Switch to a fresh scratch session — this resets the editor to the default example query,
  // so a successful re-open below cannot be a no-op (the editor genuinely changed away first).
  await picker.selectOption("__scratch__");
  await expect(editor).not.toHaveValue(distinctive);

  // Re-open the saved workspace from the picker (by id) and assert the editor query is restored.
  await picker.selectOption(wsValue);
  await expect(editor).toHaveValue(distinctive, { timeout: 30_000 });

  // The whole save/open round-trip produced zero console errors.
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("a saved workspace survives a full page reload (localStorage persistence)", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReadyClean(page);

  const editor = page.getByRole("textbox", { name: "SPARQL query" });
  const picker = page.locator("#repl-workspace");

  const distinctive = "ASK { ?persisted ?across ?reload }";
  await editor.fill(distinctive);

  await page.getByRole("button", { name: "Save as" }).click();
  const wsName = "E2E persistence probe";
  await page.getByRole("textbox", { name: "Workspace name" }).fill(wsName);
  await page.getByRole("button", { name: "Save workspace" }).click();
  await expect(picker.locator("option", { hasText: wsName })).toHaveCount(1);

  // Reload WITHOUT clearing localStorage — the saved workspace must still be listed (web
  // backend persists per-origin), and re-opening it restores the editor query.
  await page.reload({ waitUntil: "domcontentloaded" });
  await expect(page.getByText("Engine ready")).toBeVisible({ timeout: 90_000 });
  await expect(picker).toBeEnabled({ timeout: 30_000 });

  const reloadedOption = picker.locator("option", { hasText: wsName });
  await expect(reloadedOption).toHaveCount(1);
  await picker.selectOption((await reloadedOption.getAttribute("value")) ?? "");
  await expect(editor).toHaveValue(distinctive, { timeout: 30_000 });

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
