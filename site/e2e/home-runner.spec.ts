// [SONNET-4.6] sq-ymr2e.3 — the HOME hero-runner P1 journey suite.
//
// Design of record: research/web-gui-test-program.md §2.1 (home hero-runner journeys).
//
// Implements five P1 journeys that prove the home hero's in-browser WASM SPARQL runner
// actually computes results (a JOIN + SUM + ORDER BY that a static site cannot credibly fake),
// keeps ZERO network egress (the zero-network invariant, a machine-check of the "runs in your
// tab" product promise), and hands off to the live /app GUI.
// [OPUS-4.8] sq-4hiqe — the /try workbench was removed; the "Open in workbench" hand-off now does a
// hard full-page navigation to /app (window.location) instead of a sessionStorage handoff → /try.
//
// Doctrine (research/web-gui-test-program.md §1):
//   §1.1  No time-based waits — ONLY web-first waiters (expectRunnerState, web assertions).
//   §1.3  Stable selectors: getByRole / data-* / aria-label — never Tailwind classes.
//   §1.7  Never assert a timing value (the WASM instantiate timeout is a BOUND, not a result).
//
// WASM PREREQ. The hero runner loads the lean wasm bundle from `public/wasm/`. The light
// site-e2e CI lane has no Rust toolchain, so this whole file SKIPS when the bundle is absent
// (the shared wasm-gated posture — see a11y.spec.ts). Run locally:
//   (cd ../js && npm run build:wasm) && npm run sync-wasm \
//     && npx playwright install chromium && npm run test:e2e -- home-runner.spec.ts
import { test, expect, BasePage } from "./support";
import { HERO_DEFAULT_QUERY } from "../src/data/hero-sample";
import { type Locator } from "@playwright/test";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

// The lean wasm bundle the hero runner loads at runtime — gitignored, so its presence gates file.
const WASM_BUNDLE = fileURLToPath(
  new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url),
);
test.skip(
  !existsSync(WASM_BUNDLE),
  "lean wasm bundle absent — run `npm run sync-wasm` after `(cd ../js && npm run build:wasm)`",
);

// ── Custom Turtle for journey (c) ────────────────────────────────────────────────────────────
// Two contributors with distinct names and line-counts chosen to NOT overlap with the default
// sample (Ada/Lin/Grace/7300/6100/5300), so the result rows prove the DATA TAB was actually
// used — a query against the default graph could not produce Xena/999 or Yoko/111.
const CUSTOM_TURTLE = `@prefix :     <http://sparq.dev/demo/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

:xena foaf:name "Xena" ; :wrote :codeX .
:yoko foaf:name "Yoko" ; :wrote :codeY .

:codeX a :Component ; :linesOfCode 999 ; :lang "Rust" .
:codeY a :Component ; :linesOfCode 111 ; :lang "Rust" .
`;

// Expected DESC rows for CUSTOM_TURTLE with HERO_DEFAULT_QUERY (SUM per contributor, ORDER BY DESC).
const CUSTOM_ROWS: [string, string][] = [
  ["Xena", "999"],
  ["Yoko", "111"],
];

// Default expected rows (Ada 4200+3100=7300, Lin 6100, Grace 5300) in DESC order.
const DEFAULT_ROWS: [string, string][] = [
  ["Ada", "7300"],
  ["Lin", "6100"],
  ["Grace", "5300"],
];

// ── HomeRunnerPage page object ────────────────────────────────────────────────────────────────
// [SONNET-4.6] Kept LOCAL to this one spec file per the bead scope ("new file … ONLY"): the
// shared e2e/support/ dir stays read-only. Centralises the stable, role/data-* selectors (never
// a Tailwind class, §1.3 doctrine). Methods are web-first — they return Locators or click/fill
// and let Playwright's auto-wait handle readiness; no sleeps, no manual waits.
class HomeRunnerPage extends BasePage {
  /** Navigate to the home page (route "" resolves under the /sparq/ basePath) and await app-ready. */
  async gotoReady(): Promise<void> {
    await this.goto("");
  }

  /** Click the primary Run button (the big teal CTA in the bottom bar of the hero panel). */
  async clickRun(): Promise<void> {
    await this.page.getByRole("button", { name: /Run/ }).click();
  }

  /** Click the Query tab (shows the SPARQL editor, hides the Data editor). */
  async clickQueryTab(): Promise<void> {
    await this.page.getByRole("tab", { name: "Query" }).click();
  }

  /** Click the Data tab (shows the Turtle editor, hides the SPARQL editor). */
  async clickDataTab(): Promise<void> {
    await this.page.getByRole("tab", { name: "Data" }).click();
  }

  /**
   * The SPARQL query textbox (accessible when the Query tab is active; hidden under Data tab).
   * Locator: the <textarea aria-label="SPARQL query"> rendered by SparqlEditor.
   */
  queryEditor(): Locator {
    return this.page.getByRole("textbox", { name: "SPARQL query" });
  }

  /**
   * The Turtle data textbox (accessible when the Data tab is active; hidden under Query tab).
   * Locator: the <textarea aria-label="Sample data (Turtle)"> rendered by RdfEditor.
   */
  dataEditor(): Locator {
    return this.page.getByRole("textbox", { name: "Sample data (Turtle)" });
  }

  /**
   * The typed results table; appears once the engine finishes a successful run.
   * Locator: <table data-hero-results-table> (the stable data-* anchor set by hero-runner.tsx).
   */
  resultsTable(): Locator {
    return this.page.locator("[data-hero-results-table]");
  }

  /**
   * The hero error strip (role="alert", data-testid="hero-error"). Only rendered in
   * phase === "error" && error is truthy. Using data-testid avoids the Next.js route
   * announcer (which also has role="alert" and is always-present in the DOM but empty).
   */
  heroErrorAlert(): Locator {
    return this.page.getByTestId("hero-error");
  }

  /**
   * The proof footer span that shows row count, timing, and "in-browser · 0 network requests".
   * Only rendered in phase === "done". Locator strategy: regex over the MEASURED text (never a
   * Tailwind class); the timing number is excluded from the regex so it is NOT asserted (§1.7).
   */
  proofFooter(): Locator {
    return this.page.getByText(/in-browser · 0 network requests/i).first();
  }

  /**
   * The "Open in workbench →" button in the proof footer; does a HARD full-page navigation to the
   * live /app GUI via window.location.assign (sq-4hiqe — the old sessionStorage handoff → /try was
   * removed). Only present in phase === "done".
   */
  openInWorkbenchButton(): Locator {
    return this.page.getByRole("button", { name: /Open in workbench/i });
  }

  /**
   * Web-first assert the hero results table renders exactly `rows` in order.
   * Each [col0, col1] pair is a [name, linesOfRust] value. Uses containsText so it is robust
   * to typed-cell decoration: string literals render as `"Ada"` (with quotes from ResultCell),
   * numeric literals render as `7300` (plain, no quotes). The assertion also checks the row COUNT
   * — if the engine returned a different number of rows, `toHaveCount` fails immediately.
   */
  async expectResultRows(rows: [string, string][]): Promise<void> {
    const table = this.resultsTable();
    await expect(table).toBeVisible();
    const trs = table.locator("tbody tr");
    await expect(trs).toHaveCount(rows.length);
    for (let r = 0; r < rows.length; r++) {
      const cells = trs.nth(r).locator("td");
      for (let c = 0; c < rows[r].length; c++) {
        await expect(cells.nth(c)).toContainText(rows[r][c]);
      }
    }
  }
}

// ── (a) P1 — idle → results: engine-computed rows in DESC order ─────────────────────────────
// [SONNET-4.6] The core "it actually runs" proof: the JOIN + SUM + ORDER BY result cannot be
// credibly produced by a static site — Ada's 7300 is the SUM of two components (4200+3100),
// which proves the GROUP BY and SUM are real. The exact DESC order proves the ORDER BY fired.
test("(a) idle→results: exact engine-computed rows in DESC order", async ({ page }) => {
  const hp = new HomeRunnerPage(page);
  await hp.gotoReady();

  // The idle-preview state is server-rendered (no WASM needed) — the deterministic anchor.
  await hp.expectRunnerState("home", "idle-preview");

  await hp.clickRun();

  // Web-first wait: the results table anchor appears only after the engine completes the query.
  await hp.expectRunnerState("home", "results");

  // Assert EXACT computed rows in DESC order — the load-bearing honesty check.
  // Ada's 7300 = SUM(4200+3100) (she wrote :parser AND :planner), NOT a passthrough.
  await hp.expectResultRows(DEFAULT_ROWS);

  // The proof footer appears only in phase === "done" (the real, measured line).
  await expect(hp.proofFooter()).toBeVisible();
  // "3 results" proves the full row count (not a truncation).
  await expect(hp.proofFooter()).toContainText("3 results");
  // "0 network requests" is the machine-verifiable side-channel check (see journey b).
  await expect(hp.proofFooter()).toContainText("0 network requests");
});

// ── (b) P1 — zero-network invariant ─────────────────────────────────────────────────────────
// [SONNET-4.6] The load-bearing honesty check: the hero's `run()` callback evaluates ENTIRELY
// in-tab via the WASM engine — no XHR, no fetch, no WebSocket. The hermetic layer intercepts
// every outbound request, records it in externalRequests, and aborts it. If this assertion
// ever fails, a URL will appear in the log showing exactly what called out.
//
// Non-vacuity guarantee (mutation spot-check): if you add
//   `await fetch('https://test.external.example.com/api')`
// inside the `run` callback in hero-runner.tsx, the hermetic layer would record that URL in
// `externalRequests`, and `expect(hermetic.externalRequests).toEqual([])` below would fail,
// printing the offending URL. The assertion is NOT vacuous — it would catch real network egress.
test("(b) zero-network invariant: no external requests during in-tab run", async ({
  page,
  hermetic,
}) => {
  const hp = new HomeRunnerPage(page);
  await hp.gotoReady();
  await hp.expectRunnerState("home", "idle-preview");

  // Clear the hermetic log AFTER page load (excludes the pre-run wasm/JS resource fetches, which
  // are same-origin localhost and not external — but reset() gives us a clean slate regardless).
  hermetic.reset();

  await hp.clickRun();
  await hp.expectRunnerState("home", "results");

  // Non-vacuity: if the runner called fetch('https://external.example.com'), the hermetic layer
  // would record it here and this assertion would fail.
  expect(
    hermetic.externalRequests,
    `unexpected external requests:\n${hermetic.externalRequests.join("\n")}`,
  ).toEqual([]);
});

// ── (c) P1 — data tab switch → custom Turtle → different results ─────────────────────────────
// [SONNET-4.6] Proves the "sample is hackable" product promise: a visitor who edits the Data tab
// and runs the same query gets DIFFERENT results — the engine queried THEIR data, not the default
// graph. The Xena/999 and Yoko/111 rows could only come from the custom Turtle (they do not
// appear anywhere in the default graph), so this is a concrete, order-sensitive proof.
test("(c) data tab switch → custom Turtle → changed results", async ({ page }) => {
  const hp = new HomeRunnerPage(page);
  await hp.gotoReady();
  await hp.expectRunnerState("home", "idle-preview");

  // Switch to the Data tab — makes the Turtle editor accessible (Query panel becomes hidden).
  await hp.clickDataTab();

  // Replace the data with the custom Turtle (triggers onChange → onDataChange in hero-runner).
  await hp.dataEditor().fill(CUSTOM_TURTLE);

  // The Run button lives in the bottom bar (always visible regardless of active tab).
  await hp.clickRun();
  await hp.expectRunnerState("home", "results");

  // The custom rows prove BOTH that the Data tab change was picked up AND that the engine ran.
  // These values cannot come from the default Ada/Lin/Grace graph.
  await hp.expectResultRows(CUSTOM_ROWS);
});

// ── (d) P1 — error state ────────────────────────────────────────────────────────────────────
// [SONNET-4.6] Proves friendly error handling without a crash: a syntactically invalid query
// surfaces a non-empty role="alert" strip (not an unhandled exception), the PREVIOUS results
// stay intact (dimmed) while the error is shown, and editing the query clears the error strip —
// the natural "try again" gesture. The workbench recovers fully.
test("(d) error state: friendly error shown, previous results intact, edit clears error", async ({
  page,
}) => {
  const hp = new HomeRunnerPage(page);
  await hp.gotoReady();
  await hp.expectRunnerState("home", "idle-preview");

  // First: a VALID run to establish prior results (Ada/Lin/Grace). This puts the component
  // in phase === "done" with results in state — the foundation for the "results intact" check.
  await hp.clickRun();
  await hp.expectRunnerState("home", "results");
  await hp.expectResultRows(DEFAULT_ROWS);

  // Fill the query editor with invalid SPARQL (unterminated WHERE clause — always a parse error).
  // The query tab is the default active panel, so the editor is visible and accessible.
  await hp.queryEditor().fill("SELECT * WHERE { ?s ?p ?o");
  await hp.clickRun();

  // Web-first wait: the hero error strip appears — data-testid="hero-error" disambiguates it from
  // the Next.js route announcer which also carries role="alert" but is always-present and empty.
  await expect(hp.heroErrorAlert()).not.toBeEmpty();

  // The PREVIOUS results table stays in the DOM (dimmed) — not blanked. This is the "prior
  // results intact" guarantee: the component keeps `results` in state and renders ResultsTable
  // (with data-hero-results-table) even in the error phase, so the visitor can still see what
  // their last GOOD run produced.
  await expect(hp.resultsTable()).toBeVisible();

  // Edit the query (fill with valid SPARQL) — triggers onQueryChange which clears the error
  // (setError(null)) and reverts phase to "done". The error strip is removed from the DOM.
  await hp.queryEditor().fill(HERO_DEFAULT_QUERY);
  await expect(hp.heroErrorAlert()).toHaveCount(0);
});

// ── (e) P1 — hand off to the live /app GUI ────────────────────────────────────────────────────
// [OPUS-4.8] sq-4hiqe — the /try workbench (and its sessionStorage["sparq:handoff"] channel) was
// REMOVED. The "Open in workbench →" button no longer writes a handoff + router.push("/try"); it
// now does a HARD full-page navigation to the live operational GUI
// (window.location.assign(withBasePath("/app/"))). This asserts the button lands on /sparq/app/ —
// there is no query handoff anymore, just the navigation.
test("(e) hand off to /app: Open in workbench navigates to the live GUI", async ({ page }) => {
  const hp = new HomeRunnerPage(page);
  await hp.gotoReady();
  await hp.expectRunnerState("home", "idle-preview");

  // Run the default query so the proof footer (and "Open in workbench") appears.
  await hp.clickRun();
  await hp.expectRunnerState("home", "results");

  // The "Open in workbench" button only appears in phase === "done".
  await expect(hp.openInWorkbenchButton()).toBeVisible();

  // Click "Open in workbench": a HARD full-page navigation to the /app GUI (no handoff).
  await hp.openInWorkbenchButton().click();

  // Wait for the hard nav to land on /sparq/app/ — the live operational GUI destination.
  await page.waitForURL(/\/sparq\/app\//, { timeout: 30_000 });
  expect(new URL(page.url()).pathname).toContain("/app");
  await expect(page.getByRole("heading", { name: "App", level: 1 })).toBeVisible();
});
