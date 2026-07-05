// [FABLE-5] sq-u0my5 — web-persona smoke: the workbench boots and a tool works with NO Tauri.
//
// Runs under the chromium-web project (support/web-fixtures.ts): no window.__TAURI__ /
// __TAURI_INTERNALS__ is ever installed, so isTauriRuntime() === false and the page takes the
// pure-browser code paths of the deployed /app — the persona the mock-IPC lane structurally
// cannot see. The webPersona auto-fixture already guarantees, before the test body runs:
//   * hermetic network (only 127.0.0.1 / localhost / blob: / data:),
//   * the Tauri globals are genuinely absent,
//   * the in-tab WASM engine reached "Engine ready" (browser-only boot).
//
// This spec then proves:
//   1. the Query tool renders AND executes the default query against the in-tab engine
//      (a tool demonstrably *works*, not merely mounts, without a native backend);
//   2. the Import drawer is honest about the browser persona: it defaults to the Paste tab and
//      the File tab shows the "Desktop app only" notice INSTEAD of the native file picker —
//      today's honest browser state. (Sibling bead sq-eydh9 adds real browser upload; its spec
//      will supersede assertion 2 with a positive one.)
//
// Stable selectors (all declared E2E hooks — role / data-* only):
//   #repl-query                 — the SPARQL editor textarea (query-workbench.tsx contract)
//   button "Run query"          — the run trigger
//   [data-result-kind="select"] — SELECT result container
//   [data-import-trigger="rail"]— the left rail's "+ Import data…" entry point
//   [data-import-drawer]        — the Import drawer dialog content
//   [data-import-tab="file"] / ["paste"] — the drawer's tab buttons (role="tab", aria-selected)
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

test.describe("web-persona", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  test("workbench boots and the Query tool runs against the in-tab engine", async ({ page }) => {
    // The workbench shell is up: the SPARQL editor and its run trigger render.
    const editor = page.locator("#repl-query");
    const runBtn = page.getByRole("button", { name: "Run query" });
    await expect(editor).toBeVisible();
    await expect(runBtn).toBeEnabled();

    // The default query executes against the in-tab WASM engine — no native backend exists in
    // this persona, so a SELECT result rendering proves the browser code path works end-to-end.
    await runBtn.click();
    const selectResult = page.locator('[data-result-kind="select"]');
    await expect(selectResult).toBeVisible();

    // The seeded sample graph is loaded (Alice is its first ORDER BY ?name row); presence-only,
    // no row-count / timing assertion.
    await expect(selectResult.getByText("Alice").first()).toBeVisible();
  });

  test("import drawer offers no native disk path in the browser persona", async ({ page }) => {
    // Open the Import drawer from the left rail.
    await page.locator('[data-import-trigger="rail"]').click();
    const drawer = page.locator("[data-import-drawer]");
    await expect(drawer).toBeVisible();

    // Browser persona default: the drawer opens on Paste (the File tab is the desktop default —
    // import-drawer.tsx picks the initial tab off isTauriRuntime()).
    await expect(drawer.locator('[data-import-tab="paste"]')).toHaveAttribute(
      "aria-selected",
      "true",
    );

    // The File tab exists but is honest: it shows the "Desktop app only" notice and does NOT
    // offer the native file picker.
    await drawer.locator('[data-import-tab="file"]').click();
    await expect(drawer.getByText("Desktop app only")).toBeVisible();
    await expect(drawer.getByRole("button", { name: /Choose a file/ })).not.toBeAttached();

    // Close the drawer to leave the page in its initial state.
    await page.getByRole("button", { name: "Close import drawer" }).click();
    await expect(drawer).not.toBeVisible();
  });
});
