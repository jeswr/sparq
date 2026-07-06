// [OPUS-4.8] sq-800o (epic sq-ixc3) — headless browser smoke test for the /surface/shacl
// live SHACL validator. THIS IS THE REGRESSION GUARD.
//
// WHAT IT GUARDS. The validator (src/components/shacl-playground.tsx `run()`) calls
// `sparqShaclValidate` (re-exported from `@sparq/client`) on the shacl-enabled wasm bundle and
// renders a W3C validation report. The fixed bug (sq-800o): `sparqShaclValidate` extracted the
// wasm `Store.validate` as a DETACHED method reference and invoked it bare — wasm-bindgen's
// `validate` body does `wasm.store_validate(this.__wbg_ptr, …)`, so with `this === undefined`
// it threw `Cannot read properties of undefined (reading '__wbg_ptr')`. The report never
// rendered and the run landed in the error/toast path. The fix invokes `store.validate(…)` on
// the live receiver. This spec drives a REAL headless Chromium: it runs the default example
// (`ex:age "thirty"` — a datatype violation), asserts the report renders with
// `sh:conforms = false` and at least one DatatypeConstraintComponent violation, and asserts
// ZERO console errors across the interaction. Pre-fix, the `__wbg_ptr` throw both surfaces a
// console error AND leaves the report panel absent — so this test fails red on the regression.
//
// DOM ANCHORS. Stable `data-testid` / `data-*` attributes added to the component (no copy
// scraping): `[data-testid="shacl-report"]` (the result panel), `[data-conforms]` (the
// conformance banner's boolean), `[data-testid="shacl-violation"]` + `[data-component]` (each
// per-violation card and its constraint component). The "Compact syntax" path reuses the
// pre-existing `[data-testid="scs-output"]` anchor.
//
// It is a CORRECTNESS smoke test, not a benchmark: it asserts the report's DOM, never a
// wall-clock threshold (timings on a work-box / CI runner are non-canonical).
//
// WASM PREREQ. Like home-runner.spec.ts, the validator loads the wasm bundle from
// `public/wasm/` (a gitignored build artifact synced from `js`'s `build:wasm`, which builds
// `--features shacl,jsonld,…` so `validate` exists). The light site-e2e CI lane has no Rust
// toolchain to build it, so this whole spec SKIPS when the bundle is absent and runs in full
// when present. Run after `npm run sync-wasm`:
//   npx playwright install chromium && npm run test:e2e
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath.
const ROUTE = "surface/shacl/";

// The shacl-enabled wasm bundle the validator loads at runtime. Synced into public/wasm/ by
// `sync-wasm` from the `js` build; gitignored, so its presence gates this spec.
const WASM_BUNDLE = fileURLToPath(
  new URL("../public/wasm/sparq_wasm_bg.wasm", import.meta.url),
);

// Skip the whole file when the wasm bundle has not been built/synced — the light CI lane does
// not produce it, and a validator that cannot load the engine is not the thing under test.
test.skip(
  !existsSync(WASM_BUNDLE),
  "wasm bundle absent (public/wasm/sparq_wasm_bg.wasm) — run `npm run sync-wasm` after `(cd ../js && npm run build:wasm)`",
);

/** Collect every `console.error` the page emits, so the test can assert there were none. */
function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

/** Navigate to /surface/shacl and wait for the wasm engine readiness pill to settle. */
async function gotoReady(page: Page): Promise<void> {
  await page.goto(ROUTE, { waitUntil: "domcontentloaded" });
  // The playground pre-warms the engine on mount; the badge flips to "Engine ready".
  await expect(page.getByText("Engine ready")).toBeVisible({ timeout: 90_000 });
}

test("the default example validates to a non-conformant report with a datatype violation", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Run the default example (the `ex:age "thirty"` datatype violation) via the Validate button.
  await page.getByRole("button", { name: "Validate" }).click();

  // The report panel renders. Pre-fix this never appears: the detached `validate(…)` threw
  // `Cannot read properties of undefined (reading '__wbg_ptr')` and the run hit the error path.
  const report = page.locator('[data-testid="shacl-report"]');
  await expect(report).toBeVisible({ timeout: 30_000 });

  // The conformance banner reports sh:conforms = false (a string datatype where the shape
  // requires xsd:integer). Asserted via the stable data-conforms attr, not the banner copy.
  await expect(report.locator("[data-conforms]")).toHaveAttribute(
    "data-conforms",
    "false",
  );

  // At least one violation, and the datatype one is present (DatatypeConstraintComponent).
  const violations = report.locator('[data-testid="shacl-violation"]');
  expect(await violations.count()).toBeGreaterThan(0);
  await expect(
    report.locator('[data-component="DatatypeConstraintComponent"]'),
  ).toHaveCount(1);

  // The whole interaction produced zero console errors — this is what catches the __wbg_ptr
  // regression (pre-fix the detached call logged the TypeError to the console).
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("the conforming example validates to sh:conforms = true with no violations", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Switch to the "Conforming data" example (ex:age is a well-typed integer), then validate.
  await page.getByRole("button", { name: "Conforming data" }).click();
  await page.getByRole("button", { name: "Validate" }).click();

  const report = page.locator('[data-testid="shacl-report"]');
  await expect(report).toBeVisible({ timeout: 30_000 });
  await expect(report.locator("[data-conforms]")).toHaveAttribute(
    "data-conforms",
    "true",
  );
  // A conforming report carries no per-violation cards.
  await expect(report.locator('[data-testid="shacl-violation"]')).toHaveCount(0);

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("the Compact syntax button renders the shapes graph in SHACL Compact Syntax", async ({
  page,
}) => {
  // A SEPARATE wasm path (loadIntoStore + matchQuads, not validate) — worth covering since the
  // scs-output anchor already exists. Guards that the shapes graph round-trips to compact form.
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  await page.getByRole("button", { name: "Compact syntax" }).click();

  const scs = page.locator('[data-testid="scs-output"]');
  await expect(scs).toBeVisible({ timeout: 30_000 });
  // The default PersonShape carries an ex:age property constraint; the compact rendering names
  // the node shape. Assert a stable token of the compact grammar rather than the full text.
  await expect(scs).toContainText("PersonShape");

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("the SCS-input example parses compact shapes and validates the data against them", async ({
  page,
}) => {
  // [OPUS-4.8] sq-pyn7 (#796) — the SCS *input* path: the shapes editor switches to Compact
  // mode and its SHACL Compact Syntax text is parsed to a Turtle shapes graph via the wasm
  // `Store.parseShaclCompact` binding BEFORE validation. This guards that the compact-authored
  // Person shape flags `ex:carol`'s string age exactly as the equivalent Turtle shape would.
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  // Select the "Compact-syntax shapes (SCS input)" example — it flips the shapes-input toggle
  // to Compact and loads SCS into the shapes editor.
  await page.getByRole("button", { name: "Compact-syntax shapes (SCS input)" }).click();
  // The shapes-input radiogroup now reports Compact as the checked option.
  await expect(
    page.getByRole("radio", { name: "Compact" }),
  ).toHaveAttribute("aria-checked", "true");

  await page.getByRole("button", { name: "Validate" }).click();

  // The SCS is parsed to a shapes graph and the data validated against it: a non-conformant
  // report with the datatype violation (a string ex:age where the SCS requires xsd:integer).
  const report = page.locator('[data-testid="shacl-report"]');
  await expect(report).toBeVisible({ timeout: 30_000 });
  await expect(report.locator("[data-conforms]")).toHaveAttribute(
    "data-conforms",
    "false",
  );
  await expect(
    report.locator('[data-component="DatatypeConstraintComponent"]'),
  ).toHaveCount(1);

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
