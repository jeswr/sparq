// (sq-7gdfp) [SONNET-4.6] — UPDATE persistence: SPARQL INSERT/DELETE data survives a page reload.
//
// Regression for bead sq-7gdfp (found by #1518 escalated review): workspace snapshots were
// never taken after a SPARQL UPDATE, so INSERT/DELETE-applied data was silently lost on reload.
// The fix (workspace-context.tsx recordUpdateSnapshot) snapshots after each successful UPDATE.
//
// Browser persona (*.web.spec.ts): no Tauri mock, no Tauri IPC. INSERT DATA runs entirely in
// the in-tab WASM engine; the workspace persists to localStorage (web backend).
//
// Determinism rules: NO waitForTimeout; web-first assertions only; assert the SPECIFIC triple.

import { webTest as test, webExpect as expect, waitForEngineReady } from "../support/index.ts";

// The specific triple we INSERT and then assert survives reload.
const SUBJECT   = "http://example.org/update-persist-subject";
const PREDICATE = "http://example.org/update-persist-predicate";
const OBJECT    = "http://example.org/update-persist-object";

const INSERT_QUERY =
  `INSERT DATA { <${SUBJECT}> <${PREDICATE}> <${OBJECT}> . }`;
const ASK_QUERY =
  `ASK { <${SUBJECT}> <${PREDICATE}> <${OBJECT}> }`;

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

/** Click Run query and wait for an outcome of the given kind. */
async function runQuery(
  page: import("@playwright/test").Page,
  kind: "ask" | "select" | "update" | "error",
): Promise<void> {
  await page.getByRole("button", { name: "Run query" }).click();
  await expect(page.locator(`[data-result-kind="${kind}"]`)).toBeVisible();
}

/** Poll localStorage until the active workspace's dataSnapshot contains the given string. */
async function waitForSnapshotToContain(
  page: import("@playwright/test").Page,
  needle: string,
): Promise<void> {
  await expect
    .poll(
      () =>
        page.evaluate((n: string) => {
          const prefix = "sparq.workspace.v1.";
          for (let i = 0; i < localStorage.length; i++) {
            const k = localStorage.key(i);
            if (!k || !k.startsWith(prefix)) continue;
            if (k.endsWith("__index__") || k.endsWith("__last__")) continue;
            const val = localStorage.getItem(k);
            if (!val) continue;
            try {
              const ws = JSON.parse(val) as { dataSnapshot?: string };
              if (ws?.dataSnapshot?.includes(n)) return true;
            } catch {
              /* skip corrupt entries */
            }
          }
          return false;
        }, needle),
      { timeout: 5_000 },
    )
    .toBe(true);
}

test.describe("update-persist", () => {
  test("INSERT DATA triple survives a page reload", async ({ page }) => {
    // ── Apply INSERT DATA. ────────────────────────────────────────────────────────────────────
    await setEditorValue(page, INSERT_QUERY);
    await runQuery(page, "update");

    // ── Assert the triple is present in the live store (non-vacuous). ─────────────────────────
    await setEditorValue(page, ASK_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    // ── Wait for the snapshot to be persisted to localStorage. ────────────────────────────────
    // The snapshot is taken synchronously in recordUpdateSnapshot; poll to confirm the write.
    await waitForSnapshotToContain(page, SUBJECT);

    // ── Reload — the moment INSERT DATA used to be silently lost. ─────────────────────────────
    await page.reload();
    await waitForEngineReady(page, { timeout: 90_000 });

    // ── The inserted triple is STILL queryable after reload. ──────────────────────────────────
    await setEditorValue(page, ASK_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");
  });

  test("a FAILED UPDATE (bad syntax) does NOT corrupt a prior snapshot", async ({ page }) => {
    // ── First, do a successful INSERT to establish a clean baseline snapshot. ─────────────────
    await setEditorValue(page, INSERT_QUERY);
    await runQuery(page, "update");

    // Confirm the triple is present.
    await setEditorValue(page, ASK_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");

    // Wait for the snapshot to be persisted.
    await waitForSnapshotToContain(page, SUBJECT);

    // ── Now run a syntactically invalid UPDATE. ────────────────────────────────────────────────
    const BAD_UPDATE = "INSERT DATA { BAD SYNTAX TRIPLE . }";
    await setEditorValue(page, BAD_UPDATE);
    await runQuery(page, "error");

    // ── Reload — the corrupted-snapshot path. ─────────────────────────────────────────────────
    await page.reload();
    await waitForEngineReady(page, { timeout: 90_000 });

    // ── The originally-inserted triple must STILL be present (snapshot not corrupted). ─────────
    await setEditorValue(page, ASK_QUERY);
    await runQuery(page, "ask");
    await expect(page.locator('[data-result-kind="ask"]')).toContainText("true");
  });
});
