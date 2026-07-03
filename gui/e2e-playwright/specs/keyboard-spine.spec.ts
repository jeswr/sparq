// [SONNET-4.6] sq-ymr2e.5 — keyboard-spine journey: Ctrl+Enter runs the query; Ctrl-K opens
// the command palette.
//
// Exercises two keyboard paths wired in query-workbench.tsx + command-palette.tsx:
//
//   Ctrl+Enter (⌘↵) — the onKeyDown handler on the SPARQL editor textarea; same path as the
//                      Run query button.  Fires onRun("run") when metaKey || ctrlKey + Enter.
//
//   Ctrl-K / ⌘K    — the global keydown listener in CommandPaletteProvider; toggles the
//                     command palette dialog open/closed.
//
// Stable selectors:
//   #repl-query                        — the SPARQL editor textarea (id in sparql-editor.tsx)
//   button "Run query"                 — the run button (text match)
//   [data-result-kind="select"]        — a successful SELECT result
//   [data-result-kind="error"]         — an engine-rejected query
//   dialog / [role="dialog"]           — the command palette dialog (Radix UI DialogContent)
//   DialogPrimitive.Title "Command palette" (sr-only) — the accessible dialog title
//
// Determinism rules: NO waitForTimeout; web-first assertions only.

import { test, expect } from "../support/index.ts";

test.describe("keyboard-spine", () => {
  test("Ctrl+Enter runs the current query (same effect as clicking Run query)", async ({
    page,
  }) => {
    // Focus the SPARQL editor by clicking it.
    const editor = page.locator("#repl-query");
    await expect(editor).toBeVisible();
    await editor.click();

    // Press Ctrl+Enter — the onKeyDown handler in WorkbenchSparqlEditor calls e.preventDefault()
    // then onRun("run").
    await page.keyboard.press("Control+Enter");

    // The query should run and show a result (either select or error depending on parse success).
    await expect(
      page.locator('[data-result-kind="select"], [data-result-kind="error"]'),
    ).toBeVisible();
  });

  test("Ctrl-K opens the command palette dialog", async ({ page }) => {
    // The global keydown listener on `document` intercepts Ctrl-K and toggles the palette.
    await page.keyboard.press("Control+k");

    // Radix UI DialogContent renders a [role="dialog"] element.
    // The accessible title "Command palette" is rendered as an sr-only element inside the dialog.
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // The accessible title text is present (even though it is visually hidden).
    await expect(page.getByText("Command palette", { exact: true })).toBeAttached();

    // Pressing Escape closes the palette (Radix UI handles this natively).
    await page.keyboard.press("Escape");
    await expect(dialog).not.toBeVisible();
  });
});
