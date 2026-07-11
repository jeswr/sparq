// [FABLE-5] sq-ixc3.14 — federated SERVICE HONEST DEGRADATION (browser persona, no Tauri).
//
// Runs under the chromium-web project (support/web-fixtures.ts): the Tauri globals are
// genuinely absent, so isTauriRuntime() === false — exactly the deployed /app overlay. A
// SERVICE-bearing query must degrade HONESTLY there: the run-location badge labels the
// capability native-only (the tools.ts tier taxonomy's framing), the run returns a clear
// native-only error (the browser cannot dial cross-origin SPARQL endpoints — CORS), and the
// Federation control says the setting only applies in the desktop app. Never a hang, never a
// silently-dropped SERVICE pattern.
//
// Stable selectors: #repl-query, button "Run query", [data-run-location="…"],
// [data-federation-control], [data-federation-web-note], [data-result-kind="error"].
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { webTest as test, webExpect as expect } from "../support/index.ts";

const SERVICE_QUERY = `SELECT ?name ?fedRole WHERE {
  ?s <http://xmlns.com/foaf/0.1/name> ?name .
  SERVICE <http://fed.example.org/sparql> { ?s <http://example.org/fedRole> ?fedRole }
}`;

/** Set the React-controlled editor via the native setter + input event (see workbench-query). */
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

test.describe("federation-web", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  test("SERVICE query is labelled native-only and errs honestly instead of hanging", async ({
    page,
  }) => {
    // The badge flips to the native-only label as soon as SERVICE is detected.
    await setEditorValue(page, SERVICE_QUERY);
    await expect(page.locator('[data-run-location="service-native-only"]')).toBeVisible();

    // Running it surfaces the honest degradation message — a clean error outcome (CORS /
    // native-only), NOT a hang and NOT a silently un-federated answer.
    await page.getByRole("button", { name: "Run query" }).click();
    const error = page.locator('[data-result-kind="error"]');
    await expect(error).toBeVisible();
    await expect(error).toContainText(/native-only/i);
    await expect(error).toContainText(/desktop/i);
  });

  test("the Federation control is honest about being a desktop capability", async ({ page }) => {
    await page.locator("[data-federation-control]").click();
    // The web note says the setting persists but SERVICE execution is native-only here.
    const note = page.locator("[data-federation-web-note]");
    await expect(note).toBeVisible();
    await expect(note).toContainText(/native-only/i);
  });
});
