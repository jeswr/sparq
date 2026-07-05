// (sq-rcuvq) [SONNET-4.6] — editor hotkeys journey: Ctrl+/ comment toggle, Tab indent,
// Shift+Tab dedent in the SPARQL editor (WorkbenchSparqlEditor, #repl-query).
//
// Exercises the handleEditorKeyDown handler wired into sparql-editor.tsx.  The same handler
// is wired into turtle-editor.tsx (WorkbenchTurtleEditor) but the SPARQL editor is the most
// accessible fixture in the mock-IPC lane.
//
// Stable selectors:
//   #repl-query   — the SPARQL editor textarea (id in sparql-editor.tsx)
//
// Determinism rules: NO waitForTimeout; web-first assertions only.
// Native value setter required: WorkbenchSparqlEditor is React-controlled — direct property
// assignment bypasses the synthetic event system, so we use the HTMLTextAreaElement native
// setter + an 'input' event, exactly as workbench-query.spec.ts does.

import { test, expect } from "../support/index.ts";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Set the SPARQL editor's value through React's controlled-input machinery. */
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

/** Read the current value of the SPARQL editor textarea. */
async function getEditorValue(
  page: import("@playwright/test").Page,
): Promise<string> {
  return page.evaluate(
    () =>
      document.querySelector<HTMLTextAreaElement>("#repl-query")?.value ?? "",
  );
}

/** Place the textarea's caret at `pos` (selectionStart = selectionEnd = pos). */
async function setCaretPos(
  page: import("@playwright/test").Page,
  pos: number,
): Promise<void> {
  await page.evaluate((p) => {
    const el = document.querySelector<HTMLTextAreaElement>("#repl-query");
    if (!el) throw new Error("#repl-query not found");
    el.focus();
    el.setSelectionRange(p, p);
  }, pos);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("editor-hotkeys – SPARQL editor", () => {
  test("Ctrl+/ toggles a line comment: adds '# ' prefix to an uncommented line", async ({
    page,
  }) => {
    const initial = "SELECT * WHERE { ?s ?p ?o }";
    await setEditorValue(page, initial);
    // Place caret at position 7 (somewhere in the middle of the line)
    await setCaretPos(page, 7);

    await page.keyboard.press("Control+/");

    const value = await getEditorValue(page);
    expect(value).toBe("# SELECT * WHERE { ?s ?p ?o }");
  });

  test("Ctrl+/ toggles back: removes '# ' from an already-commented line", async ({
    page,
  }) => {
    const commented = "# SELECT * WHERE { ?s ?p ?o }";
    await setEditorValue(page, commented);
    await setCaretPos(page, 5);

    await page.keyboard.press("Control+/");

    const value = await getEditorValue(page);
    expect(value).toBe("SELECT * WHERE { ?s ?p ?o }");
  });

  test("Tab (no selection) inserts 2 spaces at cursor", async ({ page }) => {
    await setEditorValue(page, "hello");
    await setCaretPos(page, 5); // caret at end

    await page.keyboard.press("Tab");

    const value = await getEditorValue(page);
    expect(value).toBe("hello  ");
  });

  test("Shift+Tab removes up to 2 leading spaces from the caret line", async ({
    page,
  }) => {
    await setEditorValue(page, "  hello");
    await setCaretPos(page, 3); // caret inside content

    await page.keyboard.press("Shift+Tab");

    const value = await getEditorValue(page);
    expect(value).toBe("hello");
  });
});
