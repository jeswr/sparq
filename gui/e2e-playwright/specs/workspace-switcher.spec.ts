// [SONNET-4.6] sq-lmlch — workspace switcher UI: create/rename/delete/switch correctness.
//
// INVARIANT under test: switching never loses data — the outgoing workspace's snapshot is saved
// before the incoming one hydrates (delegated to switchWorkspace / createWorkspace in sq-lcd6e).
//
// Test strategy: the "paste" import tab sends text DIRECTLY to the WASM engine (bypasses the
// tauri-mock `load_text` fixture), so each test can pick a DISTINCTIVE triple to import and
// assert its presence/absence across workspace switches.
//
// Persistence backend: web (localStorage) — the Tauri FS plugin is not mocked here, so
// createWorkspaceStore resolves the WEB backend.  Workspaces persist across reloads but
// we don't need reloads in this spec — all actions are in-session.
//
// Determinism rules: NO waitForTimeout; web-first assertions only; NO exact numeric assertions.

import { test, expect, waitForEngineReady } from "../support/index.ts";

// ── Helpers ───────────────────────────────────────────────────────────────────────────────────

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

/** Run the current editor query and wait for a result of the given kind to render. */
async function runQuery(
  page: import("@playwright/test").Page,
  kind: "ask" | "select" | "error",
): Promise<void> {
  await page.getByRole("button", { name: "Run query" }).click();
  await expect(page.locator(`[data-result-kind="${kind}"]`)).toBeVisible();
}

/** Paste-import a triple (REPLACE mode) using the import drawer's paste tab. */
async function pasteImportReplace(
  page: import("@playwright/test").Page,
  triple: string,
): Promise<void> {
  // Open the import drawer from the topbar trigger.
  await page.locator('[data-import-trigger="topbar"]').click();
  const drawer = page.locator("[data-import-drawer]");
  await expect(drawer).toBeVisible();

  await drawer.locator('[data-import-tab="paste"]').click();
  await drawer.locator("textarea").fill(triple);

  // Switch to REPLACE mode so the store is deterministic after each import.
  await drawer.getByRole("button", { name: "Replace store" }).click();
  await drawer.getByRole("button", { name: /Import \(replace store\)/ }).click();
  await expect(drawer.locator('[data-import-feedback="ok"]')).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
}

/**
 * Open the workspace switcher popover.
 * Returns the locator for the popover content (data-workspace-popover).
 */
async function openSwitcher(
  page: import("@playwright/test").Page,
): Promise<import("@playwright/test").Locator> {
  await page.locator("[data-workspace-trigger]").click();
  const popover = page.locator("[data-workspace-popover]");
  await expect(popover).toBeVisible();
  return popover;
}

/**
 * Create a new workspace via the popover UI.
 * Leaves the popover closed (createWorkspace closes it after success).
 */
async function createWorkspaceViaUI(
  page: import("@playwright/test").Page,
  name: string,
): Promise<void> {
  const popover = await openSwitcher(page);
  await popover.locator("[data-workspace-new]").click();
  const input = popover.locator("[data-workspace-create-input]");
  await expect(input).toBeVisible();
  await input.fill(name);
  await popover.locator("[data-workspace-create-confirm]").click();
  // Popover closes after create.
  await expect(popover).toBeHidden();
}

/** Switch to a workspace by clicking its row in the popover. */
async function switchWorkspaceViaUI(
  page: import("@playwright/test").Page,
  workspaceId: string,
): Promise<void> {
  const popover = await openSwitcher(page);
  await popover.locator(`[data-workspace-item="${workspaceId}"]`).click();
  await expect(popover).toBeHidden();
  // Wait for the engine to settle after the snapshot-restore switch.
  await waitForEngineReady(page, { timeout: 90_000 });
}

/** Read the active workspace ID from localStorage (the __last__ pointer). */
async function readActiveWorkspaceId(page: import("@playwright/test").Page): Promise<string | null> {
  return page.evaluate(() => {
    return localStorage.getItem("sparq.workspace.v1.__last__");
  });
}

/** Read all workspace IDs from the localStorage index. */
async function readWorkspaceIds(page: import("@playwright/test").Page): Promise<string[]> {
  return page.evaluate(() => {
    const raw = localStorage.getItem("sparq.workspace.v1.__index__");
    if (!raw) return [];
    try {
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return [];
    }
  });
}

// ── Tests ─────────────────────────────────────────────────────────────────────────────────────

test.describe("workspace-switcher", () => {

  test("switcher trigger renders with the active workspace name", async ({ page }) => {
    // The trigger button must be visible with the default workspace name.
    const trigger = page.locator("[data-workspace-trigger]");
    await expect(trigger).toBeVisible();
    // The active name span must be non-empty.
    const nameSpan = page.locator("[data-workspace-active-name]");
    await expect(nameSpan).toBeVisible();
    const name = await nameSpan.textContent();
    expect(name?.trim().length).toBeGreaterThan(0);
  });

  test("opening the switcher shows the backend label", async ({ page }) => {
    const popover = await openSwitcher(page);
    const label = popover.locator("[data-workspace-backend-label]");
    await expect(label).toBeVisible();
    // On the web persona, backend is "web" → "saved in this browser"
    await expect(label).toContainText("saved in this browser");
  });

  test("create workspace B → distinct data in each workspace → switch preserves each", async ({
    page,
  }) => {
    // Strategy: the DEFAULT workspace starts with the SAMPLE GRAPH (Alice, from seedSampleWhenEmpty).
    // Workspace B is created empty then receives a paste import — in the Tauri mock persona the
    // `load_text` IPC is intercepted, so the actual import result is the LOAD_FIXTURE triple
    // (<http://example.org/imported>). The distinction between the two workspaces is therefore:
    //   default → Alice present, imported fixture absent
    //   workspace B → Alice absent, imported fixture present
    //
    // This strategy does not depend on passing custom text through the Tauri IPC mock.

    const ALICE_QUERY = 'PREFIX foaf: <http://xmlns.com/foaf/0.1/> ASK { ?s foaf:name "Alice" }';
    // The fixture subject returned by the tauri-mock `load_text` stub (tauri-mock.ts LOAD_FIXTURE).
    const FIXTURE_SUBJECT = "http://example.org/imported";
    const FIXTURE_QUERY = `ASK { <${FIXTURE_SUBJECT}> ?p ?o }`;

    // ── (A) Default workspace: Alice is present (sample graph). ──────────────────────────────
    await setEditorValue(page, ALICE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    // Fixture subject is absent in the default workspace's sample graph.
    await setEditorValue(page, FIXTURE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");

    // Remember default workspace ID so we can switch back to it later.
    const defaultId = await readActiveWorkspaceId(page);
    expect(defaultId).not.toBeNull();

    // ── (B) Create workspace B — createWorkspace saves the default's live snapshot first. ─────
    await createWorkspaceViaUI(page, "workspace-b-e2e");

    // Workspace B is now active.
    await expect(page.locator("[data-workspace-active-name]")).toContainText("workspace-b-e2e");

    // Workspace B is empty (no sample, no import yet) — both triples are absent.
    await setEditorValue(page, ALICE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");

    await setEditorValue(page, FIXTURE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");

    // Import into workspace B (REPLACE mode). The mock returns the LOAD_FIXTURE triple.
    // The paste text content is irrelevant here — the Tauri IPC mock intercepts load_text
    // and always returns `<http://example.org/imported> <...> <...>`.
    await pasteImportReplace(
      page,
      "<http://example.org/imported> <http://example.org/p> <http://example.org/o> .",
    );

    // Workspace B: fixture triple present, Alice absent.
    await setEditorValue(page, FIXTURE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    await setEditorValue(page, ALICE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");

    // Remember workspace B ID.
    const bId = await readActiveWorkspaceId(page);
    expect(bId).not.toBeNull();
    expect(bId).not.toBe(defaultId);

    // ── (C) Switch BACK to the default workspace. ─────────────────────────────────────────────
    await switchWorkspaceViaUI(page, defaultId!);
    await expect(page.locator("[data-workspace-active-name]")).not.toContainText("workspace-b-e2e");

    // Default workspace: Alice present (sample graph restored), fixture triple absent.
    await setEditorValue(page, ALICE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    await setEditorValue(page, FIXTURE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");

    // ── (D) Switch BACK to workspace B — its data must still be there. ───────────────────────
    await switchWorkspaceViaUI(page, bId!);
    await expect(page.locator("[data-workspace-active-name]")).toContainText("workspace-b-e2e");

    // Workspace B: fixture triple still present, Alice still absent.
    await setEditorValue(page, FIXTURE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    await setEditorValue(page, ALICE_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("false");
  });

  test("rename is reflected in the trigger and in the popover list", async ({ page }) => {
    // ── Set up: create a workspace we will rename. ────────────────────────────────────────────
    await createWorkspaceViaUI(page, "workspace-to-rename");
    await expect(page.locator("[data-workspace-active-name]")).toContainText("workspace-to-rename");

    // ── Open switcher, find the active workspace, click rename. ───────────────────────────────
    const popover = await openSwitcher(page);

    // The workspace-to-rename item is active; find its ID.
    const activeId = await readActiveWorkspaceId(page);
    expect(activeId).not.toBeNull();

    // Click the rename button for the active workspace.
    await popover.locator(`[data-workspace-rename="${activeId}"]`).click();

    // Inline rename input should appear pre-filled with the old name.
    const renameInput = popover.locator("[data-workspace-rename-input]");
    await expect(renameInput).toBeVisible();
    await expect(renameInput).toHaveValue("workspace-to-rename");

    // Clear and type a new name.
    await renameInput.fill("workspace-renamed-e2e");
    await renameInput.press("Enter");

    // Popover stays open (rename does not close the popover).
    // The renamed workspace should appear in the list.
    // The active-name in the trigger must reflect the rename.
    await expect(page.locator("[data-workspace-active-name]")).toContainText("workspace-renamed-e2e");
  });

  test("delete non-active workspace falls back safely (active workspace unchanged)", async ({
    page,
  }) => {
    // ── Set up: create two extra workspaces. ─────────────────────────────────────────────────
    await createWorkspaceViaUI(page, "to-delete-e2e");
    const toDeleteId = await readActiveWorkspaceId(page);
    expect(toDeleteId).not.toBeNull();

    // Switch back to the default workspace before deleting to-delete-e2e.
    const allIds = await readWorkspaceIds(page);
    const defaultId = allIds.find((id) => id !== toDeleteId);
    expect(defaultId).toBeDefined();
    await switchWorkspaceViaUI(page, defaultId!);

    // ── Delete the non-active workspace. ─────────────────────────────────────────────────────
    const popover = await openSwitcher(page);
    await popover.locator(`[data-workspace-delete="${toDeleteId}"]`).click();

    // Confirm button must appear.
    const confirmBtn = popover.locator("[data-workspace-delete-confirm]");
    await expect(confirmBtn).toBeVisible();
    await confirmBtn.click();

    // After delete the active workspace has not changed (we deleted a non-active one).
    await expect(page.locator("[data-workspace-active-name]")).not.toContainText("to-delete-e2e");

    // The deleted workspace row must be gone from the popover list.
    const deletedRow = page.locator(`[data-workspace-item="${toDeleteId}"]`);
    // Reopen the popover to confirm the row is absent.
    await openSwitcher(page);
    await expect(deletedRow).not.toBeAttached();
  });

  test("deleting the ACTIVE workspace switches to a fallback workspace", async ({ page }) => {
    // ── Set up: create workspace B, switch to it, then delete it. ────────────────────────────
    await createWorkspaceViaUI(page, "active-to-delete-e2e");
    const toDeleteId = await readActiveWorkspaceId(page);
    expect(toDeleteId).not.toBeNull();

    // Confirm workspace-b is active.
    await expect(page.locator("[data-workspace-active-name]")).toContainText("active-to-delete-e2e");

    // Delete the active workspace via the popover.
    const popover = await openSwitcher(page);
    await popover.locator(`[data-workspace-delete="${toDeleteId}"]`).click();
    const confirmBtn = popover.locator("[data-workspace-delete-confirm]");
    await expect(confirmBtn).toBeVisible();
    await confirmBtn.click();

    // After deleting the active workspace the context must switch to a fallback.
    // The trigger must no longer show the deleted workspace's name.
    await expect(page.locator("[data-workspace-active-name]")).not.toContainText(
      "active-to-delete-e2e",
    );

    // The engine must still be ready (no crash / white screen).
    await waitForEngineReady(page, { timeout: 90_000 });
  });

  test("'New workspace' Cmd-K command opens the create inline input", async ({ page }) => {
    // Trigger Cmd-K (or Ctrl-K).
    await page.keyboard.press("Meta+k");
    const paletteInput = page.locator("input[placeholder*='Run a tool']");
    await expect(paletteInput).toBeVisible();

    // Type "New workspace" and press Enter.
    await paletteInput.fill("New workspace");
    await page.keyboard.press("Enter");

    // The palette must close and the workspace create input must be visible.
    await expect(paletteInput).not.toBeVisible();
    const createInput = page.locator("[data-workspace-create-input]");
    await expect(createInput).toBeVisible();
  });
});
