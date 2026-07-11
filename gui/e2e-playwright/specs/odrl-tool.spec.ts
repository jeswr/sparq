// [FABLE-5] sq-ixc3.15 — ODRL policy tool journey (mocked-desktop persona).
//
// Exercises the tool end-to-end through the real frontend: open the tab → run the prefilled
// sample (policy + two-named-graph dataset + graph-scoped query) → the two-pane preview where
// the PROHIBITION visibly flips a previously visible named graph to hidden in alice's pane
// while bob's and the ungated pane keep it → the malformed-policy DENY-EVERYTHING banner with
// the visible reason. The IPC layer is the tauri-mock, which mirrors the Rust command's
// fail-closed contract (gui/src-tauri/src/odrl.rs); the REAL evaluator + PodStore gating are
// covered by that module's native-lane tests.
//
// Stable selectors (declared E2E hooks):
//   [data-tool="odrl"]                — the rail entry
//   [data-odrl-policy]                — the policy editor textarea
//   [data-odrl-run]                   — the run trigger
//   [data-odrl-decision-a|b]          — the per-requester decision badge (allow|deny)
//   [data-odrl-pane-a|b]              — the per-requester gated pane
//   [data-odrl-hidden-a]              — the hidden-vs-ungated graph diff line
//   [data-odrl-pane-ungated]          — the ungated pane
//   [data-odrl-policy-error]          — the malformed-policy fail-closed banner
//
// Determinism rules: NO waitForTimeout; web-first assertions only.

import { test, expect, readIpcLog } from "../support/index.ts";

/** Set a React-controlled textarea via the native setter + input event. */
async function setTextareaValue(
  page: import("@playwright/test").Page,
  selector: string,
  value: string,
): Promise<void> {
  await page.evaluate(
    ({ selector, text }) => {
      const el = document.querySelector<HTMLTextAreaElement>(selector);
      if (!el) throw new Error(`Textarea ${selector} not found`);
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLTextAreaElement.prototype,
        "value",
      )?.set;
      if (!setter) throw new Error("Could not access native value setter");
      setter.call(el, text);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    },
    { selector, text: value },
  );
}

test.describe("odrl tool", () => {
  // The tauriMock auto-fixture navigates to "/" and waits for engine ready before each test.

  test("a prohibition flips a previously visible named graph to hidden in the preview pane", async ({
    page,
  }) => {
    await page.locator('[data-tool="odrl"]').click();

    // The prefilled sample carries the prohibition scoped to alice on the secret graph.
    await page.locator("[data-odrl-run]").click();

    // Policy validated: permissions + the prohibition counted.
    await expect(page.locator("[data-odrl-policy-ok]")).toBeVisible();
    await expect(page.locator("[data-odrl-policy-ok]")).toContainText(/prohibition/i);

    // Alice: DENIED, her pane HIDES the secret graph (previously visible ungated) but keeps
    // the still-permitted public one — the visible flip the bead's acceptance test names.
    await expect(page.locator('[data-odrl-decision-a="deny"]')).toBeVisible();
    const paneA = page.locator("[data-odrl-pane-a]");
    await expect(paneA).toContainText("Public report");
    await expect(paneA).not.toContainText("Secret memo");
    await expect(page.locator("[data-odrl-hidden-a]")).toContainText(
      "http://example.org/secret",
    );

    // Bob: ALLOWED, his pane keeps both graphs.
    await expect(page.locator('[data-odrl-decision-b="allow"]')).toBeVisible();
    await expect(page.locator("[data-odrl-pane-b]")).toContainText("Secret memo");

    // The ungated pane always shows the raw data — the honest baseline of the diff.
    await expect(page.locator("[data-odrl-pane-ungated]")).toContainText("Secret memo");

    // IPC contract: odrl_preview got the policy, both requesters, and the query.
    const log = await readIpcLog(page);
    const call = log.find((e) => e.cmd === "odrl_preview");
    expect(call, "odrl_preview must be invoked").toBeTruthy();
    const args = (call?.args ?? {}) as {
      policy?: string;
      requesters?: string[];
      query?: string;
    };
    expect(args.policy ?? "").toContain("odrl:prohibition");
    expect(args.requesters).toHaveLength(2);
    expect(args.query ?? "").toContain("GRAPH");
  });

  test("a malformed policy denies everything (fail-closed) with a visible reason", async ({
    page,
  }) => {
    await page.locator('[data-tool="odrl"]').click();

    await setTextareaValue(page, "[data-odrl-policy]", "this is @@ not turtle ;;");
    await page.locator("[data-odrl-run]").click();

    // The deny-everything banner: consequence first, parser reason verbatim.
    const banner = page.locator("[data-odrl-policy-error]");
    await expect(banner).toBeVisible();
    await expect(banner).toContainText(/fail-closed/i);
    await expect(banner).toContainText(/denying everything/i);
    await expect(banner).toContainText(/parse error/i);

    // Both gated panes are empty (nothing materialized), while the ungated pane still
    // proves the data exists — deny-everything is a GATING outcome, not missing data.
    await expect(page.locator("[data-odrl-pane-a]")).toContainText("(0 rows)");
    await expect(page.locator("[data-odrl-pane-b]")).toContainText("(0 rows)");
    await expect(page.locator("[data-odrl-pane-ungated]")).toContainText("Secret memo");
  });
});
