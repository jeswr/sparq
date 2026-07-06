// [OPUS-4.8] sq-jp7ry (issue #835) — REGRESSION GUARD for the SHACL-validator wasm
// OBJECT-LIFECYCLE bug (the `__wbg_ptr` family of faults), exercised by RE-RUNNING the
// validator twice in ONE browser session.
//
// THE BUG CLASS THIS GUARDS. Issue #835's reported symptom was the live SHACL validator
// throwing `Cannot read properties of undefined (reading '__wbg_ptr')` — a wasm-bindgen
// object-lifecycle misuse. The first incarnation (sq-800o, fixed) was a LOST RECEIVER: the
// validate method was extracted detached and invoked bare, so `this.__wbg_ptr` was undefined.
// The whole `__wbg_ptr` / "null pointer passed to rust" / "recursive use of an object" family
// is what surfaces when a wasm `Store` is reused after free, double-borrowed, or invoked on a
// detached receiver. shacl-validator.spec.ts covers a SINGLE validate per page load; the bug
// most likely to slip back in is a SECOND-RUN one — e.g. a future change that caches/reuses the
// ephemeral wasm `Store` across `sparqShaclValidate` calls and frees it on the first run, so the
// SECOND `run()` invokes a dangling pointer. This spec drives the validator surface TWICE in one
// session (validate → validate again, including a content change between runs) and asserts:
//   (a) NO `__wbg_ptr` / "null pointer passed to rust" / "recursive use of an object" console
//       error is ever emitted (the lifecycle-fault net — these are the exact wasm-bindgen
//       panic/throw strings for the bug class); and
//   (b) the SECOND run STILL renders a fresh report (the validator is not left wedged after the
//       first run consumes/frees a shared object).
//
// The SHACL surface lives at /surface/shacl (the same route shacl-validator.spec.ts drives — the
// validator UI is not a distinct "SHACL validator" route, it is the /surface/shacl playground's
// Validate path, which calls `sparqShaclValidate` → the wasm `Store.validate` binding).
//
// WASM PREREQ. The validator loads the shacl-enabled wasm bundle from `public/wasm/` (a gitignored
// build artifact synced from `js`'s `build:wasm`, which builds `--features shacl,…` so `validate`
// exists). The light site-e2e CI lane has no Rust toolchain to build it, so this whole spec SKIPS
// when the bundle is absent and runs in full when present. Run after `npm run sync-wasm`:
//   npx playwright install chromium && npm run test:e2e
// [OPUS-4.8] sq-ymr2e.1 — runs under the shared hermetic + deterministic `test` (e2e/support):
// the SHACL validator loads a same-origin wasm bundle and makes no external request, so the
// hermetic network block is a no-op here and simply proves this spec is compatible with the
// foundation fixture (research/web-gui-test-program.md §1 acceptance).
import { test, expect } from "./support";
import { type Page, type ConsoleMessage } from "@playwright/test";
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

// The exact wasm object-lifecycle fault strings the regression must never re-introduce. Matched
// case-insensitively as substrings against every console error + pageerror the session emits.
const LIFECYCLE_FAULTS = [
  "__wbg_ptr",
  "null pointer passed to rust",
  "recursive use of an object",
];

/** Collect every `console.error` (and pageerror) the page emits across the whole session. */
function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

/** Assert none of the collected errors is a wasm object-lifecycle fault. */
function assertNoLifecycleFault(errors: string[]): void {
  const hits = errors.filter((e) => {
    const lower = e.toLowerCase();
    return LIFECYCLE_FAULTS.some((f) => lower.includes(f));
  });
  expect(
    hits,
    `wasm object-lifecycle fault(s) detected (the __wbg_ptr regression class):\n${hits.join("\n")}`,
  ).toEqual([]);
}

/** Navigate to /surface/shacl and wait for the wasm engine readiness pill to settle. */
async function gotoReady(page: Page): Promise<void> {
  await page.goto(ROUTE, { waitUntil: "domcontentloaded" });
  await expect(page.getByText("Engine ready")).toBeVisible({ timeout: 90_000 });
}

test("re-running the validator twice in one session never throws a __wbg_ptr lifecycle fault and still reports", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoReady(page);

  const report = page.locator('[data-testid="shacl-report"]');
  const validate = page.getByRole("button", { name: "Validate" });

  // ── FIRST RUN — the default example (`ex:age "thirty"` datatype violation). ───────────────
  await validate.click();
  await expect(report).toBeVisible({ timeout: 30_000 });
  await expect(report.locator("[data-conforms]")).toHaveAttribute("data-conforms", "false");
  // No lifecycle fault after the first run (this also re-covers the original sq-800o lost-receiver).
  assertNoLifecycleFault(consoleErrors);

  // ── SECOND RUN, SAME SESSION — switch to the CONFORMING example, then validate AGAIN. ─────
  // Changing the data between runs forces a genuinely fresh validate on the SAME live page (a
  // shared/freed wasm Store reused from the first run would now fault), and gives the second
  // report a DIFFERENT, deterministic outcome (conforms = true) so we know it is a fresh render
  // and not a stale first-run panel left on screen.
  await page.getByRole("button", { name: "Conforming data" }).click();
  // Selecting an example resets the report to idle (component `selectExample`), so the prior
  // panel detaches before the second run — the re-run starts from a clean slate.
  await expect(report).toHaveCount(0);

  await validate.click();
  // The SECOND run still produces a report — the validator is not wedged after the first run.
  await expect(report).toBeVisible({ timeout: 30_000 });
  await expect(report.locator("[data-conforms]")).toHaveAttribute("data-conforms", "true");
  // A conforming report carries no per-violation cards — proof the second validate really ran
  // against the new data, not a cached first-run result.
  await expect(report.locator('[data-testid="shacl-violation"]')).toHaveCount(0);

  // ── THIRD RUN — a DIFFERENT non-conforming example ("Missing required property"), a THIRD
  // validate in the same session. Three validates stresses the object lifecycle past the first
  // re-use; a pointer freed on run 1 or 2 and reused here would fault deterministically.
  await page.getByRole("button", { name: "Missing required property" }).click();
  await expect(report).toHaveCount(0);
  await validate.click();
  await expect(report).toBeVisible({ timeout: 30_000 });
  // The third run is genuinely non-conforming (a missing required property), so it renders a
  // report with at least one violation — proof it really executed, not a stale panel.
  await expect(report.locator("[data-conforms]")).toHaveAttribute("data-conforms", "false");
  expect(await report.locator('[data-testid="shacl-violation"]').count()).toBeGreaterThan(0);

  // Across all THREE validate runs in this single session, NO wasm object-lifecycle fault was
  // ever emitted — the `__wbg_ptr` regression class stays fixed.
  assertNoLifecycleFault(consoleErrors);
});
