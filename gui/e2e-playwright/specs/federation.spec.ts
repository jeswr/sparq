// [FABLE-5] sq-ixc3.14 — federated SERVICE execution journey (mocked-desktop persona).
//
// Exercises the Query tool's FEDERATED path end-to-end through the real frontend: SERVICE
// detection → the honest run-location badge → the per-workspace egress-allowlist control →
// the query_service IPC contract (dataset snapshot + query + allowlist) → the joined rows in
// the existing multi-view result panel — and the FAIL-CLOSED refusal when the endpoint is not
// allowlisted. The IPC layer is the tauri-mock, which mirrors the Rust command's contract
// (strict allowlist, the stable "SERVICE egress refused" marker); the REAL engine + a LIVE
// loopback SPARQL endpoint are covered by gui/src-tauri/src/federation.rs's native-lane tests.
//
// Stable selectors (declared E2E hooks):
//   #repl-query                        — the SPARQL editor textarea
//   button "Run query"                 — the run trigger
//   [data-run-location="…"]            — the honest run-location badge state
//   [data-federation-control]          — the allowlist popover trigger
//   [data-federation-input] / [data-federation-add] / [data-federation-entry] — the editor
//   [data-result-kind="select"|"error"] — result panel outcomes
//
// Determinism rules: NO waitForTimeout; NO exact numeric assertions; web-first assertions only.

import { test, expect, readIpcLog } from "../support/index.ts";

/** A SELECT that joins the seeded sample graph with a remote SERVICE endpoint. */
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

test.describe("federation", () => {
  // The tauriMock auto-fixture navigates to "/" and waits for engine ready before each test.

  test("plain query shows the LOCAL badge; SERVICE flips it to the native federated badge", async ({
    page,
  }) => {
    // The default (plain) query runs in-tab: the badge says so.
    await expect(page.locator('[data-run-location="local-wasm"]')).toBeVisible();

    // A SERVICE-bearing query flips the badge to the native federated location (desktop persona).
    await setEditorValue(page, SERVICE_QUERY);
    await expect(page.locator('[data-run-location="native-federated"]')).toBeVisible();
  });

  test("SERVICE endpoint off the allowlist is refused fail-closed (clean error, no hang)", async ({
    page,
  }) => {
    // Fresh workspace: the allowlist is EMPTY, so the run must surface the actionable
    // fail-closed refusal — a clean error outcome, not a hang and not an empty result.
    await setEditorValue(page, SERVICE_QUERY);
    await page.getByRole("button", { name: "Run query" }).click();

    const error = page.locator('[data-result-kind="error"]');
    await expect(error).toBeVisible();
    await expect(error).toContainText(/fail-closed/i);
    await expect(error).toContainText(/allowlist/i);
  });

  test("allowlisting the endpoint runs the SERVICE join natively and renders joined rows", async ({
    page,
  }) => {
    // 1. Allowlist the fixture endpoint host through the Federation control.
    await page.locator("[data-federation-control]").click();
    const input = page.locator("[data-federation-input]");
    await expect(input).toBeVisible();
    await input.fill("fed.example.org");
    await page.locator("[data-federation-add]").click();
    await expect(page.locator('[data-federation-entry="fed.example.org"]')).toBeVisible();
    // Close the popover so it does not overlap the run button.
    await page.keyboard.press("Escape");

    // 2. Run the SERVICE query.
    await setEditorValue(page, SERVICE_QUERY);
    await page.getByRole("button", { name: "Run query" }).click();

    // 3. The JOINED row renders in the existing multi-view panel: the LOCAL binding (Alice,
    //    from the sample graph) alongside the REMOTE-only binding (remote-captain).
    await expect(page.locator('[data-result-kind="select"]')).toBeVisible();
    await expect(page.locator('[data-result-view="table"]')).toBeVisible();
    await expect(page.getByRole("cell", { name: "Alice" }).first()).toBeVisible();
    await expect(page.getByRole("cell", { name: "remote-captain" })).toBeVisible();

    // 4. IPC contract: query_service was invoked with the workspace allowlist AND a non-empty
    //    dataset snapshot (the live store handed to the native engine).
    const log = await readIpcLog(page);
    const call = log.find((e) => e.cmd === "query_service");
    expect(call, "query_service must be invoked for a SERVICE-bearing run").toBeTruthy();
    const args = (call?.args ?? {}) as { dataset?: string; query?: string; allow?: string[] };
    expect(args.allow).toContain("fed.example.org");
    expect(typeof args.dataset).toBe("string");
    expect((args.dataset ?? "").length).toBeGreaterThan(0);
    expect(args.query ?? "").toContain("SERVICE");
  });
});
