// [GPT-5.6] sq-lsp7k.1.2 — desktop-persona F2 journey over the shared renderer chunk.
import { test, expect } from "../support/index.ts";

test.describe("forms-tool (desktop persona)", () => {
  test.beforeEach(async ({ page }) => {
    await page.locator('[data-tool="forms"]').click();
    await expect(page.locator('[data-tool-panel="forms"]')).toBeVisible();
  });

  test("renders grouped core DASH editors and nested blank-node details", async ({ page }) => {
    await expect(page.locator('[data-form-group="declared"]')).toHaveCount(3);
    for (const [label, kind] of [
      ["Title", "text"], ["Notes", "textarea"], ["Status", "enum"],
      ["Active", "boolean"], ["Due date", "date"], ["Reviewed at", "datetime"],
      ["Customer", "iri-ref"], ["Shipping address", "nested"],
      ["Related blank node", "blank-node"],
    ] as const) {
      await expect(page.locator(`[data-form-field="${label}"] [data-field-editor="${kind}"]`)).toBeVisible();
    }
    await expect(page.locator('[data-form-field="Shipping address"] [data-nested-form]')).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Street" })).toHaveValue("1 Coyote Way");
  });

  test("cardinality, widget switching and view/edit mode are functional", async ({ page }) => {
    const title = page.locator('[data-form-field="Title"]');
    const titleWidget = title.locator("[data-widget-switcher]");
    await titleWidget.selectOption({ label: "TextAreaEditor" });
    await title.getByRole("textbox", { name: "Title" }).fill("Edited with alternate widget");
    await expect(titleWidget).toHaveValue("http://datashapes.org/dash#TextAreaEditor");
    await expect(title.locator('[data-field-editor="textarea"]')).toBeVisible();
    await expect(title.getByRole("textbox", { name: "Title" })).toHaveValue("Edited with alternate widget");

    const notes = page.locator('[data-form-field="Notes"]');
    await expect(notes.locator("textarea")).toHaveCount(1);
    await notes.locator("[data-add-value]").click();
    await expect(notes.locator("textarea")).toHaveCount(2);
    await expect(page.locator('[data-form-field="Title"] [data-add-value]')).toHaveCount(0);

    await page.locator('[data-form-mode-choice="view"]').click();
    await expect(page.locator('[data-form-renderer]').first()).toHaveAttribute("data-form-mode", "view");
    await expect(page.locator('[data-form-field="Title"] [data-field-editor]')).toHaveCount(0);
    await page.locator('[data-form-field="Title"] [data-widget-switcher]').selectOption({ label: "ValueTableViewer" });
    await expect(page.locator('[data-form-field="Title"] [data-value-table]')).toBeVisible();

    await page.locator('[data-form-field="Customer"] [data-widget-switcher]').selectOption({ label: "HyperlinkViewer" });
    await expect(page.locator('[data-form-field="Customer"] a')).toHaveAttribute("href", "http://example.org/acme");
  });

  test("shape changes are surfaced to the host for F1 re-derivation", async ({ page }) => {
    await page.locator("[data-form-shape-switcher]").selectOption({ label: "Auditable item" });
    const request = page.locator('[data-tool-panel="forms"] p[role="status"]');
    await expect(request).toContainText("AuditableShape");
    await expect(request).toContainText("re-derive");
  });

  test("xsd:boolean numeric lexicals display and update canonically", async ({ page }) => {
    await page.locator("[data-form-json-source]").evaluate((details: HTMLDetailsElement) => {
      details.open = true;
    });
    const editor = page.getByRole("textbox", { name: "FormDescription JSON" });
    const description = JSON.parse(await editor.inputValue());
    const active = description.groups
      .flatMap((group: { fields: { label: string; values: { term: { value: string } }[] }[] }) => group.fields)
      .find((field: { label: string }) => field.label === "Active");
    active.values[0].term.value = "1";
    await editor.fill(JSON.stringify(description));
    await page.locator("[data-load-form-json]").click();

    const boolean = page.locator('[data-form-field="Active"] select[aria-label="Active"]');
    await expect(boolean).toHaveValue("true");
    await boolean.selectOption("false");
    await expect(boolean).toHaveValue("false");
  });

  test("malformed field JSON reports a path and leaves the current form mounted", async ({ page }) => {
    await page.locator("[data-form-json-source]").evaluate((details: HTMLDetailsElement) => {
      details.open = true;
    });
    const malformed = {
      focus: { kind: "iri", value: "urn:focus" },
      mode: "edit",
      shapes: [],
      groups: [{ kind: "default", fields: [{}] }],
    };
    await page.getByRole("textbox", { name: "FormDescription JSON" }).fill(JSON.stringify(malformed));
    await page.locator("[data-load-form-json]").click();
    await expect(page.locator('[data-tool-panel="forms"] p[role="alert"]')).toContainText("groups[0].fields[0].path");
    await expect(page.locator("[data-form-renderer]").first()).toBeVisible();
    await expect(page.locator("[data-form-focus]")).toContainText("order42");
  });
});
