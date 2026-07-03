// [OPUS-4.8] sq-ymr2e.2 — the /try P1 JOURNEY suite (edit→run→values, share-link round-trip,
// own-data load, error states) on the WG1 determinism foundation (e2e/support/**, sq-ymr2e.1).
//
// Design of record: research/web-gui-test-program.md §2.2 (the /try journeys) + §1 (the seven-rule
// determinism doctrine — esp. rule 1 no time-based waits and rule 7 never assert a timing value).
// Each journey asserts CONCRETE result VALUES a static site could not credibly fake (ordered
// numerics the engine computed, custom triples the visitor supplied), never a row count or a
// wall-clock number.
//
// WHY hermetic ON (the shared foundation default, NOT the zk-prewarm opt-out). The /try engine is
// the lean sparq wasm bundle, which is SINGLE-THREADED: nothing under src/ references
// SharedArrayBuffer / crossOriginIsolated for it (only the ZK prover's bb.js does, hence its
// `hermeticNetwork: false`). So the coi-serviceworker cross-origin-isolation the hermetic
// `context.route` interception disrupts is irrelevant here — the engine loads either way — and we
// KEEP the real zero-network invariant (`hermetic.externalRequests` stays empty), which is the
// honest machine-check of /try's "nothing leaves this tab" promise. The foundation even pre-wired
// the `try` → "engine-ready" runner-state anchor (runner-state.ts) for exactly this suite.
//
// WASM PREREQ. The REPL loads the lean wasm bundle from `public/wasm/` (a gitignored build
// artifact synced from js/'s `build:wasm`). The light site-e2e CI lanes have no Rust toolchain to
// build it, so this whole file SKIPS when the bundle is absent — the same posture as
// try-query-smoke.spec.ts / repl-results.spec.ts — and runs in full wherever the bundle is present
// (locally after `npm run sync-wasm`, or any wasm-syncing lane). Run:
//   (cd ../js && npm run build:wasm) && npm run sync-wasm && npx playwright install chromium \
//     && npm run test:e2e -- try-journeys.spec.ts
import { test, expect, BasePage, rdf } from "./support";
import { encodeQueryHash } from "../src/lib/try-handoff";
import { type Locator, type Page } from "@playwright/test";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

// The lean wasm bundle the REPL loads at runtime — gitignored, so its presence gates this file.
const WASM_BUNDLE = fileURLToPath(
  new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url),
);
test.skip(
  !existsSync(WASM_BUNDLE),
  "lean wasm bundle absent (public/wasm/sparq_wasm_bg.wasm) — run `npm run sync-wasm` after `(cd ../js && npm run build:wasm)`",
);

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath — a leading
// slash would target the origin root and miss the basePath entirely.
const ROUTE = "try/";

// The keyboard "run" chord the app advertises as Ctrl/Cmd+Enter. The wrapping editor keydown
// handler (repl.tsx) accepts metaKey OR ctrlKey, so either works, but we press the HOST-OS
// convention (⌘ on macOS, Ctrl elsewhere) to drive the real spine faithfully.
const RUN_CHORD = process.platform === "darwin" ? "Meta+Enter" : "Control+Enter";

// ── Deterministic, dataset-independent queries with EXACT, engine-COMPUTED answers ──────────────
// `?doubled` is `?n * 2` computed in the engine, so a static echo of the query text could not
// produce it; ORDER BY proves the engine sorted the VALUES (input order 2,3,5). Distinct ASC/DESC
// variants let the keyboard-run journey prove the second run actually re-executed the edited text.
const Q_DESC =
  "SELECT ?n ?doubled WHERE { VALUES ?n { 2 3 5 } BIND(?n * 2 AS ?doubled) } ORDER BY DESC(?n)";
const Q_ASC =
  "SELECT ?n ?doubled WHERE { VALUES ?n { 2 3 5 } BIND(?n * 2 AS ?doubled) } ORDER BY ASC(?n)";
const ROWS_DESC = [
  ["5", "10"],
  ["3", "6"],
  ["2", "4"],
];
const ROWS_ASC = [
  ["2", "4"],
  ["3", "6"],
  ["5", "10"],
];

// Ages ordered DESC over the shared SAMPLE_TURTLE fixture — a concrete, order-sensitive projection
// over the visitor's OWN uploaded triples (the "load your own data" proof for the own-data journey).
const AGES_DESC_ROWS = [
  ["Carol", "41"],
  ["Alice", "30"],
  ["Bob", "24"],
];

/**
 * The /try workbench page object (the foundation's base-page comment names this "TryWorkbenchPage
 * for sq-ymr2e.2"). Kept LOCAL to this one spec file per the bead scope ("new file … ONLY"): the
 * shared e2e/support/ dir stays read-only. Centralises the stable, role/data-* selectors (never a
 * Tailwind class, doctrine §1.3).
 */
class TryWorkbench extends BasePage {
  /** Navigate to /try and web-first wait for the wasm engine to reach the "Engine ready" state. */
  async gotoReady(route: string = ROUTE): Promise<void> {
    await this.goto(route);
    // Never a sleep: the runner-state waiter asserts the "Engine ready" pill (foundation map),
    // with the generous WASM-instantiate BOUND (never a measured/asserted timing, §1.7).
    await this.expectRunnerState("try", "engine-ready");
  }

  /** The accessible SPARQL editor textbox (a real <textarea> exposed as "SPARQL query"). */
  editor(): Locator {
    return this.page.getByRole("textbox", { name: "SPARQL query" });
  }

  /** Replace the whole editor value with `query` (fill targets the textarea's value). */
  async setQuery(query: string): Promise<void> {
    await this.editor().fill(query);
  }

  /** Run the current query via the primary Run button. */
  async clickRun(): Promise<void> {
    await this.page.getByRole("button", { name: "Run query" }).click();
  }

  /** Run the current query via the Ctrl/Cmd+Enter keyboard chord (focuses the editor, then presses). */
  async keyboardRun(): Promise<void> {
    await this.editor().press(RUN_CHORD);
  }

  /** The SELECT result panel (typed table), by its stable data-result-kind anchor. */
  selectPanel(): Locator {
    return this.page.locator('[data-result-kind="select"]');
  }

  /** The engine error panel (a <pre>), by its stable data-result-kind anchor. */
  errorPanel(): Locator {
    return this.page.locator('[data-result-kind="error"]');
  }

  /** The built-in dataset <select> (id `repl-dataset`); reads "__custom__" once own/shared data loads. */
  datasetSelect(): Locator {
    return this.page.locator("#repl-dataset");
  }

  /** The hidden dataset Upload file input (the "load your own data" affordance). */
  fileInput(): Locator {
    return this.page.locator('input[type="file"]');
  }

  /**
   * Web-first assert the SELECT table renders EXACTLY these rows of cell values, in order. Uses
   * per-cell substring match so it is robust to typed-cell decoration (a string literal renders
   * quoted as `"Alice"`, a numeric literal as plain `10`) while still asserting the concrete VALUE.
   */
  async expectSelectRows(expected: string[][]): Promise<void> {
    const table = this.selectPanel().locator('[data-result-view="table"] table');
    await expect(table).toBeVisible();
    const rows = table.locator("tbody tr");
    await expect(rows).toHaveCount(expected.length);
    for (let r = 0; r < expected.length; r++) {
      const cells = rows.nth(r).locator("td");
      for (let c = 0; c < expected[r].length; c++) {
        await expect(cells.nth(c)).toContainText(expected[r][c]);
      }
    }
  }
}

/** Collect uncaught page errors so a journey can assert the app never CRASHED (a friendly error
 *  is a `data-result-kind="error"` panel, not an unhandled exception). */
function trackPageErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

// ── (a) P1 — edit → run → results, via the Run BUTTON and via Ctrl/Cmd+Enter; EXACT values. ─────
test("edit→run renders exact computed values — Run button and Ctrl/Cmd+Enter", async ({
  page,
  hermetic,
}) => {
  const wb = new TryWorkbench(page);
  await wb.gotoReady();

  // Run via the primary button: DESC ordering of the engine-computed ?doubled column.
  await wb.setQuery(Q_DESC);
  await wb.clickRun();
  await expect(wb.selectPanel()).toBeVisible();
  await wb.expectSelectRows(ROWS_DESC);

  // Edit the query (flip to ASC) and run it via the KEYBOARD spine. The reordered rows prove the
  // second run re-executed the edited text — not a stale render of the first result.
  await wb.setQuery(Q_ASC);
  await wb.keyboardRun();
  await wb.expectSelectRows(ROWS_ASC);

  // The whole interaction stayed in-tab: nothing left the localhost origin (the honest machine
  // check of /try's "nothing leaves this tab" promise). The one product call the site can make —
  // github releases/latest — is fixture-ROUTED by the foundation, so it never counts as external.
  expect(
    hermetic.externalRequests,
    `unexpected external requests:\n${hermetic.externalRequests.join("\n")}`,
  ).toEqual([]);
});

// ── (b) P1 — share-link round-trip: a /try#q=<base64url> link produced in one context restores
//     the exact query + data in a FRESH context (a new tab, separate sessionStorage — the actual
//     share scenario, not a same-page reload). ────────────────────────────────────────────────
test("share-link round-trip: a #q= link restores query + data in a fresh context", async ({
  page,
  context,
}) => {
  // Context 1: land the workbench so the share scenario starts from a real, ready session.
  const wb1 = new TryWorkbench(page);
  await wb1.gotoReady();

  // "Produce a link" through the PRODUCT'S OWN codec — the same base64url-JSON encoder the /try
  // mount decodes with (encodeQueryHash ⇄ decodeQueryHash), so this is a true round-trip, not a
  // re-implementation. The link carries BOTH the query and a custom dataset, so the restore is
  // fully self-contained (independent of whatever bundled sample ships).
  const hash = encodeQueryHash({
    query: rdf.SELECT_NAMES_ASC,
    data: rdf.SAMPLE_TURTLE,
    format: "turtle",
  });

  // A FRESH tab in the same (hermetic) browser context: its own sessionStorage, so the restore
  // MUST come from the URL hash — proving the shareable-link path, not the same-tab handoff.
  const fresh = await context.newPage();
  try {
    const wb2 = new TryWorkbench(fresh);
    await wb2.gotoReady(`${ROUTE}#q=${hash}`);

    // The editor is prepopulated with the shared query (the "prepopulates" half of the round-trip).
    await expect(wb2.editor()).toHaveValue(rdf.SELECT_NAMES_ASC);
    // The shared dataset replaced the default — the picker reflects the custom (non-built-in) load.
    await expect(wb2.datasetSelect()).toHaveValue("__custom__");

    // Run it: the answers are the shared graph's names in ASC order — proving the DATA travelled in
    // the link too (a query alone against a default graph could not yield exactly these names).
    await wb2.clickRun();
    await wb2.expectSelectRows(rdf.EXPECTED_NAMES_ASC.map((n) => [n]));
  } finally {
    await fresh.close();
  }
});

// ── (c) P1 — own-data load: bring your OWN Turtle into the dataset panel, then query it. The
//     shipped affordance is the dataset "Upload" file input (there is no paste textarea), so we
//     drive that real DOM (setInputFiles), which is the visitor's genuine "load your own data" path.
test("own-data load: upload custom Turtle, then query the uploaded triples", async ({ page }) => {
  const wb = new TryWorkbench(page);
  await wb.gotoReady();

  // Load the visitor's own dataset (mode defaults to "replace") via the real Upload input — an
  // in-memory Turtle file, parsed entirely in-tab through the wasm Store.
  await wb.fileInput().setInputFiles({
    name: "people.ttl",
    mimeType: "text/turtle",
    buffer: Buffer.from(rdf.SAMPLE_TURTLE, "utf-8"),
  });
  // The picker flips to the custom (non-built-in) state once the upload replaces the store.
  await expect(wb.datasetSelect()).toHaveValue("__custom__");

  // Query the UPLOADED triples: ages DESC returns the visitor's own people, engine-ordered.
  await wb.setQuery(rdf.SELECT_AGES_DESC);
  await wb.clickRun();
  await expect(wb.selectPanel()).toBeVisible();
  await wb.expectSelectRows(AGES_DESC_ROWS);
});

// ── (d) P1 — error states: a bad query surfaces a FRIENDLY error (not a crash), and the workbench
//     RECOVERS on the next valid run. NOTE (honest adaptation to the real DOM): the design record
//     §2.2 assumed an inline "strip" that leaves prior results intact and clears on edit; the
//     shipped REPL instead REPLACES the result panel with a `data-result-kind="error"` <pre> and
//     recovers on the next run (repl.tsx `ResultPanel`). We assert the shipped behaviour — the
//     load-bearing P1 intent, "bad query → friendly error, not a crash", holds either way.
test("error state: a parse error shows a friendly message and the workbench recovers", async ({
  page,
}) => {
  const pageErrors = trackPageErrors(page);
  const wb = new TryWorkbench(page);
  await wb.gotoReady();

  // A good run first, so we can prove the error path is entered from a live result — and later
  // that the workbench returns to a working state.
  await wb.setQuery(Q_ASC);
  await wb.clickRun();
  await wb.expectSelectRows(ROWS_ASC);

  // A syntactically invalid query (unterminated WHERE): the engine surfaces a non-empty, human
  // message in the error panel rather than throwing an unhandled exception (a crash).
  await wb.setQuery(rdf.INVALID_QUERY);
  await wb.clickRun();
  await expect(wb.errorPanel()).toBeVisible();
  await expect(wb.errorPanel()).not.toBeEmpty();
  // Friendly, not fatal: no SELECT table is shown for the failed query, and the editor + Run
  // control remain usable (the app shell survived the parse failure).
  await expect(wb.selectPanel()).toHaveCount(0);
  await expect(wb.editor()).toBeEditable();
  await expect(page.getByRole("button", { name: "Run query" })).toBeEnabled();

  // Recover: fix the query and re-run — the results render again (the error was transient, and
  // "editing then running" clears it, the shipped analogue of the design's "editing clears the strip").
  await wb.setQuery(Q_DESC);
  await wb.clickRun();
  await wb.expectSelectRows(ROWS_DESC);

  // The whole error→recover interaction produced ZERO uncaught page errors: the friendly error is
  // a rendered panel, never a crash (§2.2 "without a crash").
  expect(pageErrors, `uncaught page errors:\n${pageErrors.join("\n")}`).toEqual([]);
});
