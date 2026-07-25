// [FABLE-5] sq-ymr2e.9 — BEHAVIORAL accessibility: the checks axe CANNOT see.
//
// Design of record: research/web-gui-test-program.md §3.2 (the behavioral half of the WCAG
// gate) + §1 (the determinism doctrine, inherited verbatim from the shared `test` — frozen
// clock, seeded random, hermetic network, web-first waiters, no timing assertions). The axe
// half lives in a11y.spec.ts; this file is deliberately FILE-DISJOINT from it.
//
// [OPUS-4.8] sq-4hiqe — the /try SPARQL playground was REMOVED, so its keyboard contracts (the
// complete keyboard-only run journey, the execution-target switch, and the connect-drawer
// disclosure) are gone with it. What remains are the /try-INDEPENDENT APG contracts:
//   (b) The WAI-ARIA TABS pattern on the home hero Query/Data tab/panel widget:
//       tablist/tab/tabpanel roles, exactly one aria-selected tab, ROVING TABINDEX (only the
//       selected tab in the page Tab-order), and ArrowRight/ArrowLeft focus movement with
//       selection-follows-focus (automatic activation). This is the regression class of the real
//       ARIA tab/panel bug found by hand.
//   (c) The ⌘K/Ctrl-K COMMAND PALETTE (a GLOBAL app-shell affordance, driven here from the home
//       page): focus lands in the combobox input on open with exactly one active option; from the
//       first arrow key on, the active option is communicated via aria-activedescendant while DOM
//       focus never leaves the input (the APG combobox/listbox mechanism cmdk implements —
//       upstream cmdk 1.1.1 emits no aria-activedescendant for the mount-time initial selection,
//       bead sq-pa15k); Tab cannot escape the dialog while open; ESC closes it AND RESTORES focus
//       to the invoker.
//
// NON-VACUITY. a11y suites rot silently: assertions that keep passing on broken markup are
// worse than none. The last test strips the load-bearing ARIA attributes (aria-selected, the
// roving tabindex) off a live tablist via DOM mutation and asserts the tabs-contract helper
// REJECTS — a green run therefore proves the assertions are live, mirroring the injected
// unlabelled-button probe in a11y.spec.ts.
//
// Every test here is wasm-FREE and runs on the light site-e2e CI lane. Locally:
//   npx playwright install chromium \
//     && npx playwright test a11y-keyboard.spec.ts --repeat-each=5 --retries=0
import { type Locator, type Page } from "@playwright/test";
import { test, expect, BasePage } from "./support";

// ── Shared contract helpers ──────────────────────────────────────────────────────────────────

/**
 * The STATIC half of the APG tabs contract on a `role="tablist"` container:
 *   * at least two `role="tab"` children;
 *   * EXACTLY ONE tab with aria-selected="true";
 *   * a roving tabindex — the selected tab (and only it) is in the page Tab-order.
 * `timeout` is threaded so the non-vacuity probe can assert a FAST rejection instead of
 * burning the suite-default expect timeout on markup it deliberately broke.
 */
async function expectApgTablistStatic(
  tablist: Locator,
  opts: { timeout?: number } = {},
): Promise<void> {
  const { timeout } = opts;
  await expect(tablist).toBeVisible({ timeout });
  const tabs = tablist.locator('[role="tab"]');
  expect(
    await tabs.count(),
    "an APG tablist has at least two tabs — fewer means the widget is mis-roled",
  ).toBeGreaterThanOrEqual(2);
  await expect(
    tablist.locator('[role="tab"][aria-selected="true"]'),
    "exactly ONE tab must carry aria-selected='true' (APG tabs: single selection)",
  ).toHaveCount(1, { timeout });
  await expect(
    tablist.locator('[role="tab"][aria-selected="true"]'),
    "the selected tab must be IN the page Tab-order (tabindex 0 — roving tabindex)",
  ).toHaveAttribute("tabindex", "0", { timeout });
  const unselected = tablist.locator('[role="tab"]:not([aria-selected="true"])');
  const n = await unselected.count();
  for (let i = 0; i < n; i++) {
    await expect(
      unselected.nth(i),
      "every unselected tab must be OUT of the page Tab-order (tabindex -1 — roving tabindex)",
    ).toHaveAttribute("tabindex", "-1", { timeout });
  }
}

/**
 * The KEYBOARD half of the APG tabs contract: with focus on the selected tab, ArrowRight
 * moves focus to the next enabled tab and selection FOLLOWS focus (automatic activation —
 * every panel here is cheap local state); ArrowLeft returns. The roving tabindex is
 * re-checked after each move so the Tab-stop travels with the selection.
 */
async function expectApgTablistArrows(page: Page, tablist: Locator): Promise<void> {
  const selected = () =>
    tablist.locator('[role="tab"][aria-selected="true"]');
  const initialLabel = ((await selected().textContent()) ?? "").trim();

  // Programmatic focus is not a pointer event; it parks focus exactly where a keyboard user
  // Tab-arrives (the roving tabindex guarantees the selected tab is that Tab-stop).
  await selected().focus();
  await expect(selected()).toBeFocused();

  await page.keyboard.press("ArrowRight");
  // Focus moved AND selection followed it — the selected tab is a *different* one and holds focus.
  await expect(
    selected(),
    "ArrowRight must move focus to the next enabled tab with selection following focus",
  ).toBeFocused();
  await expect(selected()).not.toHaveText(initialLabel);
  await expectApgTablistStatic(tablist); // the Tab-stop (tabindex 0) travelled with the selection

  await page.keyboard.press("ArrowLeft");
  await expect(
    selected(),
    "ArrowLeft must return focus + selection to the previous tab",
  ).toBeFocused();
  await expect(selected()).toHaveText(initialLabel);
  await expectApgTablistStatic(tablist);
}

// ── (b) The WAI-ARIA tabs contract on the tab/panel widget family. ──────────────────────────────
test.describe("WAI-ARIA tabs contract", () => {
  test("home hero Query/Data: full tabs pattern — roles, panels, roving focus", async ({
    page,
  }) => {
    const home = new BasePage(page);
    await home.goto("");
    await home.expectRunnerState("home", "idle-preview");

    const tablist = page.getByRole("tablist", { name: "Runner editor" });
    await expectApgTablistStatic(tablist);
    await expectApgTablistArrows(page, tablist);

    // The hero is a FULL tabs widget (it has real panels), so the panel half of the pattern
    // is asserted too: each tab controls a role=tabpanel labelled by it, and exactly the
    // selected tab's panel is visible.
    const queryTab = tablist.getByRole("tab", { name: "Query" });
    const dataTab = tablist.getByRole("tab", { name: "Data" });
    await expect(queryTab).toHaveAttribute("aria-controls", "hero-panel-query");
    await expect(dataTab).toHaveAttribute("aria-controls", "hero-panel-data");
    const queryPanel = page.locator("#hero-panel-query");
    const dataPanel = page.locator("#hero-panel-data");
    await expect(queryPanel).toHaveAttribute("role", "tabpanel");
    await expect(dataPanel).toHaveAttribute("role", "tabpanel");
    await expect(queryPanel).toHaveAttribute("aria-labelledby", "hero-tab-query");
    await expect(dataPanel).toHaveAttribute("aria-labelledby", "hero-tab-data");

    // Switch to Data BY KEYBOARD and assert the panels actually swap…
    await queryTab.focus();
    await page.keyboard.press("ArrowRight");
    await expect(dataTab).toHaveAttribute("aria-selected", "true");
    await expect(dataPanel).toBeVisible();
    await expect(queryPanel).toBeHidden();

    // …and the roving-tabindex payoff: Tab from the tablist lands INSIDE the visible panel
    // (the Data editor), never on another tab (APG: the tablist is one Tab-stop).
    await page.keyboard.press("Tab");
    await expect(page.getByRole("textbox", { name: "Sample data (Turtle)" })).toBeFocused();
  });
});

// ── (c) The ⌘K/Ctrl-K command palette: combobox focus contract + trap + restore. ────────────────
test.describe("command palette focus contract", () => {
  test("aria-activedescendant drives the active option; Tab is trapped; ESC restores the invoker", async ({
    page,
  }) => {
    // [OPUS-4.8] sq-4hiqe — the command palette is a GLOBAL app-shell affordance; with /try gone
    // we drive it from the home page (which also exposes a "SPARQL query" textbox — the home hero
    // Query editor — as the identifiable focus invoker for the restore assertion).
    const p = new BasePage(page);
    await p.goto("");
    await p.expectRunnerState("home", "idle-preview");

    // Park focus on a real, identifiable invoker (the home hero query editor) so the restore
    // assertion is meaningful — the palette's document-level shortcut works from any focus target.
    const editor = page.getByRole("textbox", { name: "SPARQL query" });
    await editor.focus();
    await expect(editor).toBeFocused();

    const dialog = await p.openCommandPalette();

    // FOCUS lands in the search input, which cmdk exposes as an APG combobox, and exactly
    // ONE option is marked active (aria-selected="true") on open. KNOWN UPSTREAM GAP: cmdk
    // 1.1.1 does not emit aria-activedescendant until the first arrow key (its store only
    // writes selectedItemId on a value CHANGE, never for the mount-time initial selection) —
    // bead sq-pa15k tracks it — so the activedescendant contract is asserted on the ARROW
    // interaction below, where the app satisfies it today.
    const input = dialog.getByRole("combobox");
    await expect(input).toBeFocused();
    await expect(dialog.locator('[role="option"][aria-selected="true"]')).toHaveCount(1);

    // ArrowDown: the active option is communicated via aria-activedescendant — it must
    // reference a REAL element that is the selected option (a dangling id would pass any
    // attribute-presence check but be silent to AT) — while DOM focus STAYS on the input
    // (the APG combobox/listbox mechanism cmdk implements).
    await page.keyboard.press("ArrowDown");
    await expect(input).toHaveAttribute("aria-activedescendant", /.+/);
    const firstActive = await input.getAttribute("aria-activedescendant");
    await expect(page.locator(`[id="${firstActive}"][role="option"]`)).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(input).toBeFocused();

    // A second ArrowDown MOVES the active option: aria-activedescendant changes to another
    // real selected option and DOM focus still never leaves the input.
    await page.keyboard.press("ArrowDown");
    await expect(input).not.toHaveAttribute("aria-activedescendant", firstActive ?? "");
    const nextActive = await input.getAttribute("aria-activedescendant");
    await expect(page.locator(`[id="${nextActive}"][role="option"]`)).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(input).toBeFocused();

    // FOCUS TRAP: Tab cycles inside the dialog; focus may never escape to the page below.
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((d) => d.contains(document.activeElement)),
        `Tab press ${i + 1} let focus ESCAPE the open dialog`,
      ).toBe(true);
    }

    // ESC closes the palette AND RESTORES focus to the invoker.
    await p.closeCommandPalette();
    await expect(editor).toBeFocused();
  });
});

// ── Non-vacuity: stripping the ARIA contract off live markup must FAIL the helper. ──────────────
// If the tabs-contract helper silently matched nothing (a renamed widget, a lazy locator), every
// green above would be vacuous. Strip the two load-bearing attributes off the REAL home tablist
// and assert the helper REJECTS each time — proving the assertions bite on broken markup.
test.describe("harness non-vacuity", () => {
  test("stripping aria-selected or the roving tabindex makes the tabs contract FAIL", async ({
    page,
  }) => {
    const home = new BasePage(page);
    await home.goto("");
    await home.expectRunnerState("home", "idle-preview");
    const tablist = page.getByRole("tablist", { name: "Runner editor" });

    // Sanity: the intact widget passes (so the rejections below are caused by the strip).
    await expectApgTablistStatic(tablist, { timeout: 5_000 });

    // Strip 1 — remove aria-selected from the selected tab (no tab selected): must FAIL fast.
    await tablist.evaluate((el) => {
      el.querySelector('[role="tab"][aria-selected="true"]')?.removeAttribute("aria-selected");
    });
    await expect(
      expectApgTablistStatic(tablist, { timeout: 1_500 }),
      "the contract helper PASSED on a tablist with no selected tab — the a11y assertions are vacuous",
    ).rejects.toThrow();

    // Restore, then Strip 2 — flatten the roving tabindex (every tab tabbable): must FAIL fast.
    await tablist.evaluate((el) => {
      const tabs = el.querySelectorAll<HTMLElement>('[role="tab"]');
      tabs.forEach((t, i) => t.setAttribute("aria-selected", i === 0 ? "true" : "false"));
    });
    await expectApgTablistStatic(tablist, { timeout: 5_000 });
    await tablist.evaluate((el) => {
      el.querySelectorAll<HTMLElement>('[role="tab"]').forEach((t) =>
        t.setAttribute("tabindex", "0"),
      );
    });
    await expect(
      expectApgTablistStatic(tablist, { timeout: 1_500 }),
      "the contract helper PASSED on a flattened tabindex — the roving assertion is vacuous",
    ).rejects.toThrow();
  });
});
