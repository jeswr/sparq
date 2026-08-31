// [GPT-5.6] sq-3eukz — desktop Forms journey over the active workspace + mocked Tauri adapter.
import { test, expect, readIpcLog } from "../support/index.ts";

const EX = "http://example.org/";

test.describe("forms-tool (desktop persona)", () => {
  test.beforeEach(async ({ page }) => {
    await page.locator('[data-tool="forms"]').click();
    await expect(page.locator('[data-tool-panel="forms"]')).toBeVisible();
    await expect(page.locator('[data-form-field="Workspace name"]')).toBeVisible();
  });

  test("focus, mode, and shape changes re-invoke Tauri with the complete workspace request", async ({
    page,
  }) => {
    const panel = page.locator('[data-tool-panel="forms"]');
    const picker = panel.locator("[data-form-focus-picker]");
    // The placeholder + one option per subject in the seeded sample graph: alice, bob, carol,
    // dan, and the RDF 1.2 reifier `ex:knowsClaim` (data/sample-graph.ts). Kept as an EXACT
    // count so a subject appearing or vanishing from the fixture fails here rather than silently.
    await expect(picker.locator("option")).toHaveCount(6);

    await picker.selectOption(JSON.stringify(["iri", `${EX}bob`, null, null]));
    await expect(panel.locator("[data-form-focus]")).toContainText(`${EX}bob`);
    await expect(panel.getByRole("textbox", { name: "Workspace name" })).toHaveValue("Bob");

    let calls = (await readIpcLog(page)).filter((entry) => entry.cmd === "derive_form");
    let args = calls.at(-1)?.args as Record<string, unknown>;
    expect(args.focus).toBe(`${EX}bob`);
    expect(args.dataset).toBe(args.shapes);
    expect(args.format).toBe("nquads");
    expect(args.mode).toBe("edit");
    expect(args.shape).toBeUndefined();

    await panel.locator('[data-form-mode-choice="view"]').click();
    await expect(panel.locator("[data-form-renderer]")).toHaveAttribute("data-form-mode", "view");
    calls = (await readIpcLog(page)).filter((entry) => entry.cmd === "derive_form");
    args = calls.at(-1)?.args as Record<string, unknown>;
    expect(args.focus).toBe(`${EX}bob`);
    expect(args.mode).toBe("view");
    expect(args.shape).toBe(`${EX}PersonShape`);

    await panel.locator("[data-form-shape-switcher]").selectOption({ label: "Auditable" });
    await expect
      .poll(async () => {
        const log = (await readIpcLog(page)).filter((entry) => entry.cmd === "derive_form");
        return (log.at(-1)?.args as Record<string, unknown>)?.shape;
      })
      .toBe(`${EX}AuditableShape`);
  });

  test("desktop host fixture renders the shared FormDescription content", async ({ page }) => {
    const panel = page.locator('[data-tool-panel="forms"]');
    await expect(panel.locator('[data-form-derive-source]')).toContainText("Tauri");
    await expect(panel.locator('[data-form-group="declared"]')).toContainText("Workspace profile");
    await expect(panel.getByRole("textbox", { name: "Workspace name" })).toHaveValue("Alice");
  });

  test("manual JSON remains available and malformed input does not replace the host form", async ({
    page,
  }) => {
    const panel = page.locator('[data-tool-panel="forms"]');
    await panel.locator("[data-form-json-source]").evaluate((details: HTMLDetailsElement) => {
      details.open = true;
    });
    await panel
      .getByRole("textbox", { name: "FormDescription JSON" })
      .fill('{"mode":"edit","groups":[],"shapes":[]}');
    await panel.locator("[data-load-form-json]").click();
    await expect(panel.getByRole("alert")).toContainText("focus");
    await expect(panel.locator("[data-form-focus]")).toContainText(`${EX}alice`);
  });
});
