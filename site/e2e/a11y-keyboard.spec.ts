// [FABLE-5] sq-ymr2e.9 — BEHAVIORAL accessibility: the checks axe CANNOT see.
//
// Design of record: research/web-gui-test-program.md §3.2 (the behavioral half of the WCAG
// gate) + §1 (the determinism doctrine, inherited verbatim from the shared `test` — frozen
// clock, seeded random, hermetic network, web-first waiters, no timing assertions). The axe
// half lives in a11y.spec.ts; this file is deliberately FILE-DISJOINT from it and from the
// /try journey suites.
//
// WHAT IT GUARDS — four contracts straight from the ARIA Authoring Practices, asserted as
// CONTRACTS (roles/state/keyboard behavior), never incidental DOM:
//   (a) A COMPLETE KEYBOARD-ONLY JOURNEY on /try: Tab to the editor → type a query →
//       Ctrl/Cmd+Enter → reach and read the results by keyboard, with :focus-visible ring
//       assertions along the path — and a machine check that NO trusted pointer event fired.
//   (b) The WAI-ARIA TABS pattern on the tab/panel widget family (home Query/Data, the /try
//       result-view + execution-target selectors): tablist/tab(/tabpanel) roles, exactly one
//       aria-selected tab, ROVING TABINDEX (only the selected tab in the page Tab-order), and
//       ArrowRight/ArrowLeft focus movement with selection-follows-focus (automatic
//       activation). This is the regression class of the real ARIA tab/panel bug already
//       found on /try by hand.
//   (c) The ⌘K/Ctrl-K COMMAND PALETTE: focus lands in the combobox input on open with exactly
//       one active option; from the first arrow key on, the active option is communicated via
//       aria-activedescendant while DOM focus never leaves the input (the APG combobox/listbox
//       mechanism cmdk implements — upstream cmdk 1.1.1 emits no aria-activedescendant for the
//       mount-time initial selection, bead sq-pa15k); Tab cannot escape the dialog while open;
//       ESC closes it AND RESTORES focus to the invoker.
//   (d) The /try CONNECT DRAWER disclosure: keyboard-toggleable with a programmatically
//       exposed expanded/collapsed state (a native <details>/<summary>, so the state is the
//       host-language semantic ARIA's aria-expanded mirrors), and the endpoint form fields
//       inside carry PROGRAMMATICALLY-ASSOCIATED labels (resolvable by role + accessible
//       name, not by proximity).
//
// NON-VACUITY. a11y suites rot silently: assertions that keep passing on broken markup are
// worse than none. The last test strips the load-bearing ARIA attributes (aria-selected, the
// roving tabindex) off a live tablist via DOM mutation and asserts the tabs-contract helper
// REJECTS — a green run therefore proves the assertions are live, mirroring the injected
// unlabelled-button probe in a11y.spec.ts.
//
// WASM PREREQ. Only the journey test needs the in-tab engine (it runs a real query); it
// SKIPS when the lean wasm bundle is absent, exactly like try-journeys.spec.ts. Every other
// test here is wasm-FREE and runs on the light site-e2e CI lane. Full set locally:
//   (cd ../js && npm run build:wasm) && npm run sync-wasm && npx playwright install chromium \
//     && npx playwright test a11y-keyboard.spec.ts --repeat-each=5 --retries=0
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { type Locator, type Page } from "@playwright/test";
import { test, expect, BasePage } from "./support";

// The lean wasm bundle the /try REPL loads at runtime (gitignored build artifact synced into
// public/wasm/ by `sync-wasm`). Its presence gates ONLY the run-a-real-query journey test.
const WASM_BUNDLE = fileURLToPath(new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url));
const WASM_PRESENT = existsSync(WASM_BUNDLE);

// The keyboard "run" chord the app advertises (repl.tsx accepts metaKey OR ctrlKey; we press
// the HOST-OS convention, same as try-journeys.spec.ts).
const RUN_CHORD = process.platform === "darwin" ? "Meta+Enter" : "Control+Enter";

// A deterministic, dataset-INDEPENDENT query with exact engine-COMPUTED answers: ?tripled is
// computed (?n * 3) and ORDER BY DESC proves the engine sorted the VALUES — a static echo of
// the typed text could not produce these cells (determinism doctrine: concrete values, never
// row counts or timings).
const JOURNEY_QUERY =
  "SELECT ?n ?tripled WHERE { VALUES ?n { 1 4 7 } BIND(?n * 3 AS ?tripled) } ORDER BY DESC(?n)";
const JOURNEY_ROWS = [
  ["7", "21"],
  ["4", "12"],
  ["1", "3"],
];

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

/** Assert `loc` holds focus AND paints a visible :focus-visible indicator (the ring). */
async function expectFocusVisibleRing(loc: Locator): Promise<void> {
  await expect(loc).toBeFocused();
  const probe = await loc.evaluate((el) => {
    const cs = getComputedStyle(el);
    return {
      focusVisible: el.matches(":focus-visible"),
      boxShadow: cs.boxShadow,
      outlineStyle: cs.outlineStyle,
    };
  });
  expect(
    probe.focusVisible,
    ":focus-visible must match after KEYBOARD focus — otherwise the ring styles never apply",
  ).toBe(true);
  expect(
    probe.boxShadow !== "none" || probe.outlineStyle !== "none",
    `the focused element must paint a visible ring (box-shadow or outline); got box-shadow=${probe.boxShadow}, outline-style=${probe.outlineStyle}`,
  ).toBe(true);
}

/**
 * Press Tab until `target` holds focus (bounded — the DOM is deterministic, so the bound is
 * a tripwire, never a wait). Returns how many presses it took. Every intermediate stop is
 * checked too: keyboard focus must NEVER land inside aria-hidden content (the Chromium
 * keyboard-focusable-scroller class — e.g. an `overflow-auto` aria-hidden highlight layer —
 * which axe's aria-hidden-focus rule misses because the focusability is UA-derived, not a
 * tabindex). That real bug existed on the editors' highlight <pre> until sq-ymr2e.9.
 */
async function tabUntilFocused(page: Page, target: Locator, maxTabs = 120): Promise<number> {
  for (let i = 1; i <= maxTabs; i++) {
    await page.keyboard.press("Tab");
    const stop = await page.evaluate(() => {
      const el = document.activeElement;
      return {
        inAriaHidden: !!el?.closest('[aria-hidden="true"]'),
        desc: el ? `${el.tagName} aria-label=${el.getAttribute("aria-label")}` : "none",
      };
    });
    expect(
      stop.inAriaHidden,
      `Tab press ${i} put keyboard focus INSIDE aria-hidden content (${stop.desc}) — invisible to AT`,
    ).toBe(false);
    const focused = await target
      .evaluate((el) => el === document.activeElement)
      .catch(() => false);
    if (focused) return i;
  }
  throw new Error(
    `keyboard-only journey broken: the target never received focus within ${maxTabs} Tab presses`,
  );
}

/**
 * Machine check that a journey used NO pointing device: counts TRUSTED pointer/mouse
 * button events (a keyboard Enter/Space on a button synthesizes only `click`, never
 * pointerdown/mousedown, so a keyboard-only run records zero). Install BEFORE goto.
 */
async function installPointerGuard(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const w = window as unknown as { __pointerEvents: string[] };
    w.__pointerEvents = [];
    for (const type of ["pointerdown", "pointerup", "mousedown", "mouseup"]) {
      window.addEventListener(
        type,
        (e) => {
          if (e.isTrusted) w.__pointerEvents.push(type);
        },
        true,
      );
    }
  });
}

async function pointerEvents(page: Page): Promise<string[]> {
  return page.evaluate(
    () => (window as unknown as { __pointerEvents?: string[] }).__pointerEvents ?? [],
  );
}

/** Navigate to /try and wait for the REPL editor (renders without the engine; wasm-free). */
async function gotoTry(page: Page): Promise<BasePage> {
  const p = new BasePage(page);
  await p.goto("try/");
  await expect(page.getByRole("textbox", { name: "SPARQL query" })).toBeVisible();
  return p;
}

// ── (a) The keyboard-only /try journey: reach the editor, query, run, read results. ─────────────
test.describe("keyboard-only /try journey", () => {
  test.skip(
    !WASM_PRESENT,
    "lean wasm bundle absent (public/wasm/sparq_wasm_bg.wasm) — run `npm run sync-wasm` after `(cd ../js && npm run build:wasm)`",
  );

  test("Tab → editor → type → Ctrl/Cmd+Enter → read results, with zero pointer events", async ({
    page,
    hermetic,
  }) => {
    await installPointerGuard(page);
    const p = new BasePage(page);
    await p.goto("try/");
    await p.expectRunnerState("try", "engine-ready");

    // REACH THE EDITOR BY KEYBOARD. Tab from the document start; the bound is a tripwire on
    // a deterministic DOM, not a wait. The editor must paint its :focus-visible ring.
    const editor = page.getByRole("textbox", { name: "SPARQL query" });
    const tabsToEditor = await tabUntilFocused(page, editor);
    expect(tabsToEditor, "the editor must be keyboard-reachable").toBeGreaterThan(0);
    await expectFocusVisibleRing(editor);

    // TYPE THE QUERY — real key events (select-all then type), never fill/paste.
    await page.keyboard.press("ControlOrMeta+a");
    await page.keyboard.type(JOURNEY_QUERY);
    await expect(editor).toHaveValue(JOURNEY_QUERY);

    // RUN via the advertised keyboard chord.
    await page.keyboard.press(RUN_CHORD);

    // READ THE RESULTS: exact engine-computed cells, in engine-sorted order.
    const table = page
      .locator('[data-result-kind="select"]')
      .locator('[data-result-view="table"] table');
    await expect(table).toBeVisible();
    const rows = table.locator("tbody tr");
    await expect(rows).toHaveCount(JOURNEY_ROWS.length);
    for (let r = 0; r < JOURNEY_ROWS.length; r++) {
      for (let c = 0; c < JOURNEY_ROWS[r].length; c++) {
        await expect(rows.nth(r).locator("td").nth(c)).toContainText(JOURNEY_ROWS[r][c]);
      }
    }

    // REACH THE RESULTS REGION BY KEYBOARD: thanks to the roving tabindex the whole
    // result-view tablist is ONE Tab-stop (the selected "Table" tab). Ring asserted there too.
    const resultTabs = page.getByRole("tablist", { name: "Result view" });
    const tableTab = resultTabs.locator('[role="tab"][aria-selected="true"]');
    await tabUntilFocused(page, tableTab);
    await expectFocusVisibleRing(tableTab);

    // The /try result-view tablist honours the full APG tabs contract in situ — and the
    // arrow-key view switch really re-renders the SAME result: the raw SPARQL-JSON view must
    // contain the engine-computed 21 (7*3), then ArrowLeft restores the typed table.
    await expectApgTablistStatic(resultTabs);
    await page.keyboard.press("ArrowRight");
    await expect(resultTabs.locator('[role="tab"][aria-selected="true"]')).toHaveText(/JSON/);
    const jsonView = page.locator('[data-result-view="json"]');
    await expect(jsonView).toBeVisible();
    await expect(jsonView).toContainText('"21"');
    await page.keyboard.press("ArrowLeft");
    await expect(table).toBeVisible();

    // The journey used NO pointing device and NOTHING left the tab (the honest machine
    // checks of "keyboard-only" and of /try's zero-network promise).
    expect(await pointerEvents(page), "trusted pointer/mouse events fired").toEqual([]);
    expect(
      hermetic.externalRequests,
      `unexpected external requests:\n${hermetic.externalRequests.join("\n")}`,
    ).toEqual([]);
  });
});

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

  test("/try execution-target switch (connect drawer): tabs roles + roving tabindex", async ({
    page,
  }) => {
    await gotoTry(page);
    // Open the drawer BY KEYBOARD (its own contract is test (d); here it is just the door).
    const summary = page.locator("summary", { hasText: "Advanced · Connect to a sparq-server" });
    await summary.focus();
    await page.keyboard.press("Enter");
    await expect(page.locator("#endpoint-url")).toBeVisible();

    // The In-tab-WASM ⇄ Endpoint switch is exposed as a tablist: static contract only.
    // (Arrow-key activation is exercised on the two sibling widgets above/in the journey —
    // activating THIS one flips real execution modes, which mounts endpoint-mode panels that
    // poll the configured server: a network side effect the hermetic harness would have to
    // absorb for no additional contract coverage, since all three share one roving-tablist
    // implementation.)
    const tablist = page.getByRole("tablist", { name: "Execution target" });
    await expectApgTablistStatic(tablist);
    // Both execution targets are enabled with the default (valid, loopback) endpoint URL, so
    // the roving Tab-stop is exactly the selected "In-tab WASM" tab.
    await expect(tablist.getByRole("tab", { name: "In-tab WASM" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(tablist.getByRole("tab", { name: "Endpoint" })).toBeEnabled();
  });
});

// ── (c) The ⌘K/Ctrl-K command palette: combobox focus contract + trap + restore. ────────────────
test.describe("command palette focus contract", () => {
  test("aria-activedescendant drives the active option; Tab is trapped; ESC restores the invoker", async ({
    page,
  }) => {
    const p = await gotoTry(page);

    // Park focus on a real, identifiable invoker (the editor) so the restore assertion is
    // meaningful — the palette's document-level shortcut works from any focus target.
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

// ── (d) The /try connect drawer: disclosure semantics + programmatic labels. ────────────────────
test.describe("connect drawer disclosure", () => {
  test("keyboard toggle with exposed expanded state; endpoint fields have programmatic labels", async ({
    page,
  }) => {
    await gotoTry(page);

    const details = page.locator("details", { hasText: "Advanced · Connect to a sparq-server" });
    const summary = details.locator("summary");

    // Collapsed by default, with the state PROGRAMMATICALLY exposed. The drawer is a native
    // <details>/<summary>, so the determinable expanded/collapsed state is the host-language
    // `open` semantic that aria-expanded mirrors on custom disclosures (HTML-AAM maps the
    // open summary to an expanded disclosure button) — we assert the reflected DOM state, not
    // a hand-authored (and redundant) aria-expanded attribute.
    await expect(details).toHaveJSProperty("open", false);
    await expect(page.locator("#endpoint-url")).toBeHidden();

    // Keyboard-operable: the summary is focusable, paints a focus ring, and Enter expands.
    await summary.focus();
    await expectFocusVisibleRing(summary);
    await page.keyboard.press("Enter");
    await expect(details).toHaveJSProperty("open", true);

    // The endpoint form fields are labelled PROGRAMMATICALLY: each is resolvable by role +
    // accessible name (the <label htmlFor> association), not by visual proximity.
    const urlField = page.getByRole("textbox", { name: "Endpoint URL" });
    await expect(urlField).toBeVisible();
    await expect(urlField).toHaveAttribute("id", "endpoint-url");
    const tokenField = page.getByLabel(/Bearer token/);
    await expect(tokenField).toBeVisible();
    await expect(tokenField).toHaveAttribute("id", "endpoint-token");

    // Keyboard-collapsible again (Enter toggles), and the fields leave the page.
    await summary.focus();
    await page.keyboard.press("Enter");
    await expect(details).toHaveJSProperty("open", false);
    await expect(page.locator("#endpoint-url")).toBeHidden();
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
