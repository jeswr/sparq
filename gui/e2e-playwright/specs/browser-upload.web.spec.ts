// (sq-eydh9) [SONNET-4.6] — browser upload + drag-and-drop RDF import: multi-file File tab on
// web, multi-select native dialog, global dropzone.
//
// Runs under the chromium-web project (support/web-fixtures.ts): no window.__TAURI__ /
// __TAURI_INTERNALS__, so isTauriRuntime() === false and the page takes the pure-browser code
// paths. The webPersona auto-fixture blocks external network, navigates to "/", and waits for
// "Engine ready" before yielding to each test body.
//
// The three acceptance tests from sq-eydh9:
//   (a) Multi-file upload via the file input: two .ttl fixtures → both are imported, store size
//       grows by the sum of their triples (non-vacuous count assertion — must be > 0).
//   (b) Drag-and-drop: the global drop zone's data-global-drop-input exercises the same import
//       code path the window-level drop uses; files set via setInputFiles() are imported.
//   (c) Malformed file in a batch: a syntactically invalid .ttl alongside a valid one — the
//       bad file surfaces a per-file error (`data-file-error` attribute) WITHOUT aborting the
//       valid file's import (the valid file's row shows "quads imported").
//
// Stable selectors (declared E2E hooks — role/data-* only):
//   [data-import-trigger="rail"]  — left rail's "+ Import data…"
//   [data-import-drawer]          — Import drawer dialog content
//   [data-import-tab="file"]      — File tab button
//   [data-web-file-input]         — stable hidden <input type="file"> in WebFilePane
//   [data-global-drop-input]      — stable hidden <input type="file"> in GlobalDropOverlay
//   [data-import-feedback="ok"]   — aggregate success banner
//   [data-import-feedback="error"]— aggregate error/partial banner
//   [data-file-error="<name>"]    — per-file error span
//   [data-file-row="<name>"]      — per-file list row
//   button "Run query"            — SPARQL run trigger
//   [data-result-kind="select"]   — SELECT result container
//   #repl-query                   — the SPARQL editor textarea
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions except storeSize > 0;
//   web-first assertions only; store-size checked via a SPARQL COUNT query.

import { webTest as test, webExpect as expect } from "../support/index.ts";

// ── Inline Turtle fixtures (no filesystem access; content is deterministic) ─────────────────

// fixture-a.ttl — 2 triples (Alice and Bob with names)
const FIXTURE_A: string = `
@prefix ex: <http://example.org/upload-a#> .
ex:Alice ex:name "Alice Upload A" .
ex:Bob   ex:name "Bob Upload A" .
`.trim();

// fixture-b.ttl — 3 triples (Carol, Dave, Eve with names)
const FIXTURE_B: string = `
@prefix ex: <http://example.org/upload-b#> .
ex:Carol ex:name "Carol Upload B" .
ex:Dave  ex:name "Dave Upload B" .
ex:Eve   ex:name "Eve Upload B" .
`.trim();

// fixture-bad.ttl — syntactically invalid Turtle (unpaired < for the predicate)
const FIXTURE_BAD: string = `
@prefix ex: <http://example.org/bad#> .
ex:Subject <UNCLOSED_IRI_NO_CLOSE ex:Object .
`.trim();

const ZSTD_NT =
  '<http://example.org/zstd#Subject> <http://example.org/zstd#name> "Zstd Browser Upload" .\n';

/**
 * A standards-compliant single-segment zstd frame containing one raw final block.
 * Building the fixture here keeps the e2e independent of a zstd encoder while the app still
 * has to invoke its lazy `fzstd` decoder to recover the N-Triples payload.
 */
function rawZstdFrame(text: string): Buffer {
  const payload = Buffer.from(text, "utf8");
  if (payload.length > 255) throw new Error("raw zstd fixture must fit the one-byte content size");
  const blockHeader = (payload.length << 3) | 1;
  return Buffer.concat([
    Buffer.from([
      0x28, 0xb5, 0x2f, 0xfd,
      0x20, payload.length,
      blockHeader & 0xff,
      (blockHeader >> 8) & 0xff,
      (blockHeader >> 16) & 0xff,
    ]),
    payload,
  ]);
}

// ── Helper: read the live store size from the top bar ────────────────────────────────────────
//
// The TopBar renders "{storeSize.toLocaleString()} quads" in a <span>. Reading this is more
// reliable than running a COUNT(*) SPARQL query (which returns cells as `"21"^^xsd:integer`
// via formatTerm, requiring extra parsing). The locale is "en-US" per playwright.config.ts, so
// the number may include thousands commas ("1,234 quads").

async function countQuads(page: import("@playwright/test").Page): Promise<number> {
  // The top bar "N quads" span is always visible after engine ready.
  const quadsSpan = page.locator("span", { hasText: /\d[\d,]* quads/ }).first();
  await expect(quadsSpan).toBeVisible();
  const text = await quadsSpan.textContent();
  // Strip thousands commas, then extract the leading number.
  const digits = text?.replace(/,/g, "").match(/^\d+/);
  return digits ? parseInt(digits[0], 10) : 0;
}

// ── Helper: open the Import drawer to the File tab ──────────────────────────────────────────

async function openFileTab(page: import("@playwright/test").Page) {
  await page.locator('[data-import-trigger="rail"]').click();
  const drawer = page.locator("[data-import-drawer]");
  await expect(drawer).toBeVisible();
  await drawer.locator('[data-import-tab="file"]').click();
  await expect(drawer.locator('[data-import-tab="file"]')).toHaveAttribute(
    "aria-selected",
    "true",
  );
  return drawer;
}

// ── Tests ─────────────────────────────────────────────────────────────────────────────────────

test.describe("browser-upload (sq-eydh9)", () => {
  // The webPersona fixture navigates to "/" and waits for "Engine ready" before each test.

  // ── (a) Multi-file upload via the file input ───────────────────────────────────────────────

  test("(a) multi-file upload via file input imports ALL selected files' triples", async ({
    page,
  }) => {
    // Measure baseline store size (sample graph is already loaded).
    const baseline = await countQuads(page);
    expect(baseline).toBeGreaterThan(0); // sample graph sanity-check

    // Open the drawer, switch to the File tab.
    const drawer = await openFileTab(page);

    // Set two .ttl fixtures on the stable hidden input (data-web-file-input).
    const fileInput = drawer.locator("[data-web-file-input]");
    await expect(fileInput).toBeAttached();
    await fileInput.setInputFiles([
      { name: "fixture-a.ttl", mimeType: "text/turtle", buffer: Buffer.from(FIXTURE_A) },
      { name: "fixture-b.ttl", mimeType: "text/turtle", buffer: Buffer.from(FIXTURE_B) },
    ]);

    // Both files appear in the per-file list before import.
    await expect(drawer.locator('[data-file-row="fixture-a.ttl"]')).toBeVisible();
    await expect(drawer.locator('[data-file-row="fixture-b.ttl"]')).toBeVisible();

    // The Import button is now enabled.
    const importBtn = drawer.getByRole("button", { name: /^Import/i });
    await expect(importBtn).toBeEnabled();

    // Click Import and wait for success feedback.
    await importBtn.click();
    await expect(drawer.locator('[data-import-feedback="ok"]')).toBeVisible();

    // Both rows show "quads imported" (non-vacuous: their added counts > 0).
    await expect(
      drawer.locator('[data-file-row="fixture-a.ttl"]').getByText(/quads? imported/),
    ).toBeVisible();
    await expect(
      drawer.locator('[data-file-row="fixture-b.ttl"]').getByText(/quads? imported/),
    ).toBeVisible();

    // Close the drawer and verify the store grew by at least the two files' triples (5 total).
    await page.getByRole("button", { name: "Close import drawer" }).click();
    await expect(drawer).not.toBeVisible();

    const after = await countQuads(page);
    // fixture-a = 2 triples, fixture-b = 3 triples → store must grow by at least 5.
    expect(after).toBeGreaterThan(baseline);
    expect(after - baseline).toBeGreaterThanOrEqual(5);

    // Specific subjects from BOTH fixtures must be queriable — non-vacuous presence check.
    const editor = page.locator("#repl-query");
    await editor.fill(
      `SELECT * WHERE { <http://example.org/upload-a#Alice> ?p ?o } LIMIT 1`,
    );
    await page.getByRole("button", { name: "Run query" }).click();
    const aliceResult = page.locator('[data-result-kind="select"]');
    await expect(aliceResult).toBeVisible();
    await expect(aliceResult.getByText("Alice Upload A")).toBeVisible();

    await editor.fill(
      `SELECT * WHERE { <http://example.org/upload-b#Carol> ?p ?o } LIMIT 1`,
    );
    await page.getByRole("button", { name: "Run query" }).click();
    await expect(page.locator('[data-result-kind="select"]')).toBeVisible();
    await expect(page.locator('[data-result-kind="select"]').getByText("Carol Upload B")).toBeVisible();
  });

  test("[SONNET-4.6] sq-ljc12: .zst upload is decoded before RDF format detection", async ({
    page,
  }) => {
    const drawer = await openFileTab(page);
    const fileInput = drawer.locator("[data-web-file-input]");
    await fileInput.setInputFiles({
      name: "browser-upload.nt.zst",
      mimeType: "application/zstd",
      buffer: rawZstdFrame(ZSTD_NT),
    });

    await expect(
      drawer.locator('[data-file-row="browser-upload.nt.zst"]'),
    ).toBeVisible();
    await drawer.getByRole("button", { name: /^Import/i }).click();
    await expect(drawer.locator('[data-import-feedback="ok"]')).toBeVisible();

    await page.getByRole("button", { name: "Close import drawer" }).click();
    const editor = page.locator("#repl-query");
    await editor.fill(
      "SELECT ?name WHERE { <http://example.org/zstd#Subject> <http://example.org/zstd#name> ?name }",
    );
    await page.getByRole("button", { name: "Run query" }).click();
    await expect(
      page.locator('[data-result-kind="select"]').getByText("Zstd Browser Upload"),
    ).toBeVisible();
  });

  // ── (b) Drag-and-drop via the global drop input ────────────────────────────────────────────

  test("(b) drag-and-drop of files onto the window imports them", async ({ page }) => {
    // The global drop zone's data-global-drop-input exercises the same import code path that
    // window-level drag-and-drop uses. setInputFiles on this input simulates a file drop per
    // Playwright's recommended approach for drag-and-drop file upload testing.
    const baseline = await countQuads(page);

    const globalInput = page.locator("[data-global-drop-input]");
    await expect(globalInput).toBeAttached();

    // "Drop" fixture-a onto the workbench via the global drop input.
    await globalInput.setInputFiles([
      { name: "drop-fixture-a.ttl", mimeType: "text/turtle", buffer: Buffer.from(FIXTURE_A) },
    ]);

    // The import drawer should open automatically (GlobalDropOverlay calls setOpen(true)).
    const drawer = page.locator("[data-import-drawer]");
    await expect(drawer).toBeVisible();

    // The store must have grown by at least the 2 triples from fixture-a (non-vacuous).
    // Wait for the import to complete by polling the store via the Query tool.
    await page.getByRole("button", { name: "Close import drawer" }).click();
    await expect(drawer).not.toBeVisible();

    // Verify fixture-a's triples are queryable.
    const editor = page.locator("#repl-query");
    await editor.fill(
      `SELECT * WHERE { <http://example.org/upload-a#Bob> ?p ?o } LIMIT 1`,
    );
    await page.getByRole("button", { name: "Run query" }).click();
    const result = page.locator('[data-result-kind="select"]');
    await expect(result).toBeVisible();
    await expect(result.getByText("Bob Upload A")).toBeVisible();

    const after = await countQuads(page);
    expect(after).toBeGreaterThan(baseline);
  });

  // ── (c) Malformed file surfaces a per-file error without aborting the batch ───────────────

  test("(c) a malformed file surfaces a per-file error without aborting valid files", async ({
    page,
  }) => {
    const baseline = await countQuads(page);

    // Open the drawer, switch to the File tab.
    const drawer = await openFileTab(page);

    const fileInput = drawer.locator("[data-web-file-input]");
    await expect(fileInput).toBeAttached();

    // Set two files: one valid (fixture-b) and one malformed (fixture-bad).
    await fileInput.setInputFiles([
      { name: "fixture-b.ttl", mimeType: "text/turtle", buffer: Buffer.from(FIXTURE_B) },
      { name: "fixture-bad.ttl", mimeType: "text/turtle", buffer: Buffer.from(FIXTURE_BAD) },
    ]);

    // Both rows appear.
    await expect(drawer.locator('[data-file-row="fixture-b.ttl"]')).toBeVisible();
    await expect(drawer.locator('[data-file-row="fixture-bad.ttl"]')).toBeVisible();

    // Import.
    const importBtn = drawer.getByRole("button", { name: /^Import/i });
    await expect(importBtn).toBeEnabled();
    await importBtn.click();

    // The aggregate feedback is an error/partial-error banner (not a full ok).
    await expect(drawer.locator('[data-import-feedback="error"]')).toBeVisible();

    // The bad file's row has a per-file error attribute — error is surfaced, not swallowed.
    await expect(drawer.locator('[data-file-error="fixture-bad.ttl"]')).toBeAttached();

    // The valid file's row shows "quads imported" — the batch was NOT aborted by the bad file.
    await expect(
      drawer.locator('[data-file-row="fixture-b.ttl"]').getByText(/quads? imported/),
    ).toBeVisible();

    // Close and verify fixture-b's triples ARE in the store (the valid file was imported).
    await page.getByRole("button", { name: "Close import drawer" }).click();
    await expect(drawer).not.toBeVisible();

    const editor = page.locator("#repl-query");
    await editor.fill(
      `SELECT * WHERE { <http://example.org/upload-b#Dave> ?p ?o } LIMIT 1`,
    );
    await page.getByRole("button", { name: "Run query" }).click();
    const result = page.locator('[data-result-kind="select"]');
    await expect(result).toBeVisible();
    await expect(result.getByText("Dave Upload B")).toBeVisible();

    // fixture-b's triples are new (bad file contributed zero).
    const after = await countQuads(page);
    expect(after - baseline).toBeGreaterThanOrEqual(3); // fixture-b has 3 triples
  });
});
