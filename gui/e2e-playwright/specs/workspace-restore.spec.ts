// [OPUS-4.8] sq-lcd6e — workspace RESTORE journey: imported data + editor text survive a reload.
//
// This is the regression test for the epic's data-loss bug (sq-2ucrz bug 1): the workspace model
// persisted a `dataSnapshot` on every import, but the GUI NEVER restored it — the engine warm path
// unconditionally seeded the sample graph, silently replacing a user's imported data on every
// relaunch. This spec paste-imports a distinctive triple in REPLACE mode, reloads the page, and
// asserts (a) the imported triple is still queryable and (b) the sample-graph-only data is gone.
//
// Persistence backend: on the served static export `loadTauriFs()` cannot import the (absent)
// @tauri-apps/plugin-fs, so `createWorkspaceStore` resolves the WEB (localStorage) backend. The
// Tauri IPC mock is still injected (isTauriRuntime() → true), so the paste import routes through
// the mocked `load_text` command, which returns a deterministic imported triple.
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { test, expect, waitForEngineReady } from "../support/index.ts";

// The distinctive triple the mocked `load_text` command returns for ANY pasted document
// (support/tauri-mock.ts → LOAD_FIXTURE). Its subject is the marker we query for after reload.
const IMPORTED_SUBJECT = "http://example.org/imported";

/**
 * Set the SPARQL editor value via the native setter + input event (React-controlled textarea).
 * Mirrors the helper in workbench-query.spec.ts.
 */
async function setEditorValue(
  page: import("@playwright/test").Page,
  value: string,
): Promise<void> {
  await page.evaluate((text) => {
    const el = document.querySelector<HTMLTextAreaElement>("#repl-query");
    if (!el) throw new Error("Editor textarea #repl-query not found");
    const setter = Object.getOwnPropertyDescriptor(
      window.HTMLTextAreaElement.prototype,
      "value",
    )?.set;
    if (!setter) throw new Error("Could not access native value setter");
    setter.call(el, text);
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }, value);
}

/** Run the current editor query and wait for a result of the given kind to render. */
async function runQuery(
  page: import("@playwright/test").Page,
  kind: "ask" | "select" | "error",
): Promise<void> {
  await page.getByRole("button", { name: "Run query" }).click();
  await expect(page.locator(`[data-result-kind="${kind}"]`)).toBeVisible();
}

test.describe("workspace-restore", () => {
  test("imported data + editor text survive a page reload (no data loss)", async ({ page }) => {
    // ── Pre-condition: the fresh default workspace seeded the sample graph (Alice present). ──────
    await setEditorValue(page, 'PREFIX foaf: <http://xmlns.com/foaf/0.1/> ASK { ?s foaf:name "Alice" }');
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    // ── Paste-import a distinctive triple in REPLACE mode. ───────────────────────────────────────
    await page.locator('[data-import-trigger="topbar"]').click();
    const drawer = page.locator("[data-import-drawer]");
    await expect(drawer).toBeVisible();

    await drawer.locator('[data-import-tab="paste"]').click();
    await drawer
      .locator("textarea")
      .fill(`<${IMPORTED_SUBJECT}> <http://example.org/p> <http://example.org/o> .`);

    // Replace the store (so the sample graph is gone) and import.
    await drawer.getByRole("button", { name: "Replace store" }).click();
    await drawer.getByRole("button", { name: /Import \(replace store\)/ }).click();

    // The import succeeded (best-effort snapshot persisted to localStorage).
    await expect(drawer.locator('[data-import-feedback="ok"]')).toBeVisible();

    // Persist the editor text too: change the query, so the round-trip is exercised on reload.
    await page.keyboard.press("Escape"); // close the drawer
    await expect(drawer).toBeHidden();
    const restoredQuery = `ASK { <${IMPORTED_SUBJECT}> ?p ?o }`;
    await setEditorValue(page, restoredQuery);
    // Deterministic flush: poll the persisted localStorage workspace record until it contains the
    // new query text — guarantees the debounced editor write-back has committed before we reload.
    // (Running a query alone is non-deterministic: the 400 ms debounce may not have flushed yet.)
    await expect
      .poll(
        () =>
          page.evaluate((q: string) => {
            const prefix = "sparq.workspace.v1.";
            for (let i = 0; i < localStorage.length; i++) {
              const k = localStorage.key(i);
              if (!k || !k.startsWith(prefix)) continue;
              if (k.endsWith("__index__") || k.endsWith("__last__")) continue;
              const val = localStorage.getItem(k);
              if (!val) continue;
              try {
                const ws = JSON.parse(val) as { editor?: { query?: string } };
                if (ws?.editor?.query === q) return true;
              } catch {
                /* skip corrupt entries */
              }
            }
            return false;
          }, restoredQuery),
        { timeout: 5_000 },
      )
      .toBe(true);

    // ── RELOAD — the moment the old code silently replaced the imported data with the sample. ────
    await page.reload();
    await waitForEngineReady(page, { timeout: 90_000 });

    // ── Editor text survived the reload: assert the restored value BEFORE any overwrite. ─────────
    await expect(page.locator("#repl-query")).toHaveValue(restoredQuery);

    // ── The imported triple is STILL there (restored from the snapshot, not the sample graph). ──
    await setEditorValue(page, `ASK { <${IMPORTED_SUBJECT}> ?p ?o }`);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    // ── The sample-graph-only data is GONE (the store was replaced, not merged over the sample). ─
    await setEditorValue(page, 'PREFIX foaf: <http://xmlns.com/foaf/0.1/> ASK { ?s foaf:name "Alice" }');
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");
  });

  test("a fresh workspace with no prior data still seeds the sample graph", async ({ page }) => {
    // Sanity: the seed-only-a-genuinely-fresh-workspace branch still gives a first-run user data.
    await setEditorValue(page, 'PREFIX foaf: <http://xmlns.com/foaf/0.1/> ASK { ?s foaf:name "Alice" }');
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");
  });
});
