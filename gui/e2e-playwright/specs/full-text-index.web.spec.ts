// [FABLE-5] sq-ixc3.16 — Full-text tool over IMPORTED workspace data + index management:
// a text: magic-predicate BM25 query must match a literal brought into the live store AFTER
// load (not only the seeded sample fixture), and the "Index stats" strip must report a real,
// non-zero index footprint over the workspace literals.
//
// Browser persona (*.web.spec.ts): no Tauri mock; the in-tab WASM engine carries everything.
//
// NON-VACUOUS by construction (same probe discipline as the streaming specs): the spec first
// PROBES whether this build ships the W-text bundle and then REQUIRES the matching panel
// state — bundle served → the imported literal MUST be found with a positive BM25 score and
// the stats MUST be non-zero; bundle absent → the honest unavailable state.
//
// New stable selectors (declared in full-text-tool.tsx):
//   [data-text-stats-btn]    — compute the index footprint over the current live store
//   [data-text-stats]        — the rendered stats strip
//   [data-text-stats-docs]   — indexed-document count (asserted non-zero)
//   [data-text-stats-tokens] — token count (asserted non-zero)
//
// Determinism rules: NO waitForTimeout; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

// A term that cannot occur in the seeded sample graph — proves the index covers data the
// user brought into the workspace, not a fixture.
const UNIQUE_TERM = "zephyrine";
const INSERT_QUERY =
  `INSERT DATA { <http://ex/imported-doc> <http://ex/note> ` +
  `"the ${UNIQUE_TERM} sensor array came online" . }`;

/** Set the SPARQL editor value via the native setter + input event (React-controlled textarea). */
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

test.describe("full-text tool over imported data + index stats", () => {
  test("BM25 finds a literal added to the workspace; index stats are real and non-zero (or honest unavailable)", async ({
    page,
  }) => {
    // Probe the served export for the W-text bundle — decides the REQUIRED panel state.
    const probe = await page.request.get("/wasm/text/sparq_text_wasm.js");
    const bundleServed = probe.ok();

    // Bring a uniquely-identifiable literal into the live workspace store first.
    await page.locator('[data-tool="query"]').click();
    await setEditorValue(page, INSERT_QUERY);
    await page.getByRole("button", { name: "Run query" }).click();
    await expect(page.locator('[data-result-kind="update"]')).toBeVisible();

    // Open the Full-text tool.
    await page.locator('[data-tool="full-text"]').click();
    const unavailableEl = page.locator("[data-text-unavailable]");
    const searchInput = page.locator("[data-text-search-input]");
    await expect(searchInput).toBeVisible({ timeout: 10_000 });

    if (!bundleServed) {
      // INVARIANT: without the bundle the tool degrades honestly (rebuild hint, no results).
      await expect(unavailableEl).toBeVisible({ timeout: 30_000 });
      await expect(unavailableEl).toContainText("build:text-wasm");
      return;
    }

    // Bundle served ⇒ search MUST come up live ("unavailable" here would be a loader bug).
    await expect(
      page.locator("[data-text-search-input]:not([disabled])"),
    ).toBeVisible({ timeout: 30_000 });

    // ── text: magic-predicate query over the IMPORTED literal ─────────────────────────────
    await searchInput.fill(UNIQUE_TERM);
    await page.locator("[data-text-search-btn]").click();
    const results = page.locator("[data-text-results]");
    await expect(results).toBeVisible();
    const hit = results.locator("[data-text-hit]").first();
    await expect(hit).toBeVisible({ timeout: 30_000 });
    await expect(hit).toContainText(UNIQUE_TERM);
    // A real BM25 score, not a placeholder: the score chip renders a positive number.
    await expect(hit).toContainText(/score \d/);

    // ── index management: the footprint over the live store is real and non-zero ──────────
    await page.locator("[data-text-stats-btn]").click();
    const stats = page.locator("[data-text-stats]");
    await expect(stats).toBeVisible({ timeout: 30_000 });
    // Non-zero doc + token counts (the store holds the sample graph + the imported literal).
    await expect(page.locator("[data-text-stats-docs]")).toHaveText(/^[1-9][\d,]*$/);
    await expect(page.locator("[data-text-stats-tokens]")).toHaveText(/^[1-9][\d,]*$/);
  });
});
