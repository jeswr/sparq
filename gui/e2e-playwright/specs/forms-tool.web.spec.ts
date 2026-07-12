// [GPT-5.6] sq-lsp7k.1.2 — hosted-web persona proves the same F2 chunk without Tauri globals.
import { webTest as test, webExpect as expect } from "../support/index.ts";

test("shared Forms renderer works in the hosted web persona", async ({ page }) => {
  await page.locator('[data-tool="forms"]').click();
  const panel = page.locator('[data-tool-panel="forms"]');
  await expect(panel).toBeVisible();
  await expect(panel.locator('[data-form-field="Status"] [data-field-editor="enum"]')).toBeVisible();
  await expect(panel.locator('[data-form-field="Shipping address"] [data-nested-form]')).toBeVisible();
  await panel.locator('[data-form-mode-choice="view"]').click();
  await expect(panel.locator('[data-form-field="Customer"] [data-field-viewer="label"]')).toContainText("acme");
});
