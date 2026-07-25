// [GPT-5.6] sq-3eukz — hosted-web Forms journeys: explicit absence and structural wasm adapter.
import {
  webTest as test,
  webExpect as expect,
  waitForEngineReady,
} from "../support/index.ts";
import { wasmFormsMockScript } from "../support/forms-mock.ts";

const EX = "http://example.org/";

test("an absent wasm bridge is explicit, keeps manual JSON, and never renders demo data", async ({
  page,
}) => {
  await page.locator('[data-tool="forms"]').click();
  const panel = page.locator('[data-tool-panel="forms"]');
  await expect(panel.locator("[data-form-unavailable]")).toContainText(
    "Form derivation is unavailable",
  );
  await expect(panel.locator("[data-form-renderer]")).toHaveCount(0);

  await panel.locator("[data-form-json-source]").evaluate((details: HTMLDetailsElement) => {
    details.open = true;
  });
  await expect(panel.getByRole("textbox", { name: "FormDescription JSON" })).toHaveValue("");
  await expect(panel.getByText("order42", { exact: false })).toHaveCount(0);
});

test("mocked sparq-wasm derives the same content and receives focus/mode/shape requests", async ({
  page,
}) => {
  await page.addInitScript(wasmFormsMockScript);
  await page.reload();
  await waitForEngineReady(page);
  await page.locator('[data-tool="forms"]').click();

  const panel = page.locator('[data-tool-panel="forms"]');
  await expect(panel.getByRole("textbox", { name: "Workspace name" })).toHaveValue("Alice");
  await expect(panel.locator('[data-form-derive-source]')).toContainText("WASM");
  await expect(panel.locator('[data-form-group="declared"]')).toContainText("Workspace profile");

  await panel
    .locator("[data-form-focus-picker]")
    .selectOption(JSON.stringify(["iri", `${EX}carol`, null, null]));
  await expect(panel.locator("[data-form-focus]")).toContainText(`${EX}carol`);
  await expect(panel.getByRole("textbox", { name: "Workspace name" })).toHaveValue("Carol");
  await panel.locator('[data-form-mode-choice="view"]').click();
  await expect(panel.locator("[data-form-renderer]")).toHaveAttribute("data-form-mode", "view");
  await panel.locator("[data-form-shape-switcher]").selectOption({ label: "Auditable" });

  await expect
    .poll(() =>
      page.evaluate(() => {
        const log = (
          window as unknown as {
            __SPARQ_FORMS_WASM_LOG__: Array<{
              data: string;
              shapes: string;
              focus: string;
              format: string;
              optionsJson: string;
            }>;
          }
        ).__SPARQ_FORMS_WASM_LOG__;
        return log.at(-1);
      }),
    )
    .toMatchObject({
      focus: `${EX}carol`,
      format: "nquads",
      optionsJson: JSON.stringify({ mode: "view", shape: `${EX}AuditableShape` }),
    });

  const last = await page.evaluate(() => {
    const log = (
      window as unknown as {
        __SPARQ_FORMS_WASM_LOG__: Array<{ data: string; shapes: string }>;
      }
    ).__SPARQ_FORMS_WASM_LOG__;
    return log.at(-1);
  });
  expect(last?.data).toBe(last?.shapes);
});
