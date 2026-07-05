// (sq-txrui) [SONNET-4.6] — SHACL tool upgrade: Turtle syntax highlighting + bulk multi-file
// shapes upload with per-source enable/disable toggle.
//
// Runs under the chromium-web project (support/web-fixtures.ts): NO window.__TAURI__ / Tauri
// globals — the pure-browser code path.  The webPersona auto-fixture boots the app + waits for
// "Engine ready" before each test.
//
// Three obligations (sq-txrui acceptance):
//   (a) HIGHLIGHTING — assert `.sq-tok-keyword` spans render in the shapes editor (the STARTER_
//       SHAPES @prefix lines are tokenised as keyword tokens by WorkbenchTurtleEditor).
//   (b) BULK UPLOAD — upload TWO shape files via setInputFiles on the persistent hidden input;
//       assert both appear in the sources list as enabled.
//   (c) NON-VACUOUS VALIDATION — disable the second source, validate, assert the report shows
//       exactly the violations produced by the FIRST (enabled) source only; no result from the
//       disabled source appears.
//
// File ingest approach: ShaclSources exposes a persistent hidden <input type="file" multiple>
// with data-sources-file-input. Playwright's setInputFiles() dispatches a native 'change' event
// on it — more reliable than waitForEvent('filechooser') + showOpenFilePicker in headless CI.
//
// Shape files (inlined as buffers):
//   shape-knows.ttl  — sh:minCount 1 on foaf:knows for foaf:Person
//                      → violates for ex:dan (no foaf:knows in the sample graph)
//   shape-age-max.ttl — sh:maxInclusive 25 on foaf:age for foaf:Person
//                      → violates for ex:alice (30) and ex:carol (41)
//
// With only shape-knows.ttl ENABLED:
//   • 1 violation expected  (ex:dan missing foaf:knows)
//   • ex:alice / ex:carol age violations must NOT appear
//
// Sample graph (gui/app/src/data/sample-graph.ts):
//   ex:alice  foaf:age 30, foaf:knows ex:bob + ex:carol
//   ex:bob    foaf:age 25, foaf:knows ex:alice
//   ex:carol  foaf:age 41, foaf:knows ex:alice + ex:dan
//   ex:dan    foaf:age 19, NO foaf:knows
//
// Stable selectors (all declared in shacl-tool.tsx / shacl-sources.tsx):
//   [data-tool="shacl"]           — rail entry to open the SHACL tool
//   #shacl-shapes                 — the TEXTAREA inside WorkbenchTurtleEditor
//   .sq-tok-keyword               — CSS class on highlighted keyword tokens in the <pre> layer
//   [data-sources-list]           — the <ul> of source rows (present iff sources > 0)
//   [data-source-name]            — file name span per source row
//   [data-source-toggle]          — enable/disable toggle button (aria-pressed="true/false")
//   [data-sources-file-input]     — hidden <input type="file"> — setInputFiles() target
//   [data-result-kind="shacl"]    — the report pane container
//
// Determinism rules: NO waitForTimeout; web-first assertions; never assert exact timing values.

import { webTest as test, webExpect as expect } from "../support/index.ts";

// ── Inline shape-file content ─────────────────────────────────────────────────────────────────

/** Requires foaf:Person to have foaf:knows ≥1 — violates ex:dan (no foaf:knows). */
const SHAPE_KNOWS_TTL = `@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/shapes/KnowsShape> a sh:NodeShape ;
  sh:targetClass foaf:Person ;
  sh:property [
    sh:path foaf:knows ;
    sh:minCount 1
  ] .
`;

/** Requires foaf:Person foaf:age ≤25 — violates ex:alice (30) and ex:carol (41). */
const SHAPE_AGE_MAX_TTL = `@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

<http://example.org/shapes/AgeMaxShape> a sh:NodeShape ;
  sh:targetClass foaf:Person ;
  sh:property [
    sh:path foaf:age ;
    sh:maxInclusive 25
  ] .
`;

test.describe("shacl-shapes-upload (browser persona)", () => {
  // The webPersona auto-fixture navigates to "/" and waits for "Engine ready" before each test.

  test.beforeEach(async ({ page }) => {
    // Open the SHACL tool via the left-rail entry point.
    await page.locator('[data-tool="shacl"]').click();
    // The SHACL shapes editor textarea must be visible before we proceed.
    await expect(page.locator("#shacl-shapes")).toBeVisible();
  });

  // ── (a) Syntax highlighting ───────────────────────────────────────────────────────────────────

  test("(a) shapes editor renders Turtle keyword tokens", async ({ page }) => {
    // The STARTER_SHAPES includes `@prefix sh: …` etc. WorkbenchTurtleEditor tokenises those as
    // `keyword` and renders a `<span class="sq-tok-keyword">` in the aria-hidden highlight <pre>.
    // The span is not visible to AT (aria-hidden), but CSS selectors still find it.
    const kwSpan = page.locator(".sq-tok-keyword").first();
    await expect(kwSpan).toBeAttached();
    // The STARTER_SHAPES has `@prefix` tokens — at least one keyword span must be present.
    const count = await page.locator(".sq-tok-keyword").count();
    expect(count).toBeGreaterThan(0);
  });

  // ── (b) Bulk multi-file upload ────────────────────────────────────────────────────────────────

  test("(b) uploading two shape files adds both as enabled sources", async ({ page }) => {
    // Use the persistent hidden <input type="file" multiple> (data-sources-file-input).
    // Playwright's setInputFiles() dispatches a native 'change' event so the component's
    // onInputChange handler fires — more reliable than filechooser interception in headless CI.
    const fileInput = page.locator("[data-sources-file-input]");
    await expect(fileInput).toBeAttached();

    await fileInput.setInputFiles([
      { name: "shape-knows.ttl", mimeType: "text/plain", buffer: Buffer.from(SHAPE_KNOWS_TTL) },
      {
        name: "shape-age-max.ttl",
        mimeType: "text/plain",
        buffer: Buffer.from(SHAPE_AGE_MAX_TTL),
      },
    ]);

    // Both sources must appear in the [data-sources-list].
    const list = page.locator("[data-sources-list]");
    await expect(list).toBeVisible();

    const names = list.locator("[data-source-name]");
    await expect(names).toHaveCount(2);

    // Both sources are enabled by default (aria-pressed="true").
    const toggles = list.locator("[data-source-toggle]");
    await expect(toggles.nth(0)).toHaveAttribute("aria-pressed", "true");
    await expect(toggles.nth(1)).toHaveAttribute("aria-pressed", "true");

    // The correct file names appear.
    await expect(names.nth(0)).toContainText("shape-knows.ttl");
    await expect(names.nth(1)).toContainText("shape-age-max.ttl");
  });

  // ── (c) Non-vacuous validation with per-source toggle ─────────────────────────────────────────

  test("(c) disabling a source excludes its violations; validation is non-vacuous", async ({
    page,
  }) => {
    // Upload both shape files via the persistent file input.
    const fileInput = page.locator("[data-sources-file-input]");
    await expect(fileInput).toBeAttached();

    await fileInput.setInputFiles([
      { name: "shape-knows.ttl", mimeType: "text/plain", buffer: Buffer.from(SHAPE_KNOWS_TTL) },
      {
        name: "shape-age-max.ttl",
        mimeType: "text/plain",
        buffer: Buffer.from(SHAPE_AGE_MAX_TTL),
      },
    ]);

    // Wait for both sources to appear.
    const list = page.locator("[data-sources-list]");
    await expect(list).toBeVisible();
    await expect(list.locator("[data-source-name]")).toHaveCount(2);

    // Disable the SECOND source (shape-age-max.ttl): click its toggle.
    // After this, only shape-knows.ttl is enabled.
    const toggles = list.locator("[data-source-toggle]");
    await toggles.nth(1).click();
    await expect(toggles.nth(1)).toHaveAttribute("aria-pressed", "false");
    await expect(toggles.nth(0)).toHaveAttribute("aria-pressed", "true"); // unchanged

    // Clear the inline editor so the STARTER_SHAPES (which defines no constraints) does not
    // accidentally contribute additional shapes text.  The STARTER_SHAPES is comments only
    // (the example shape is commented out), so this is conservative; the key intent is that
    // only shape-knows.ttl contributes constraints.
    //
    // Use fill() — Playwright's standard method for clearing + filling text inputs.  It fires
    // the 'input' event which React's controlled textarea listens to, ensuring the shapes state
    // updates synchronously before we click Validate.
    const editor = page.locator("#shacl-shapes");
    await editor.fill("# no additional shapes\n");

    // Validate the live store.
    await page.getByRole("button", { name: "Validate live store" }).click();

    // Wait for the report to appear (non-vacuous: it must NOT be "Conforms").
    const reportPane = page.locator('[data-result-kind="shacl"]');
    // The report header shows "1 violation" (exactly one, from shape-knows.ttl → ex:dan).
    // Use first() because the violation ROW also contains "Violation" as a severity label.
    await expect(reportPane.getByText(/violation/i).first()).toBeVisible({ timeout: 30_000 });

    // shape-knows.ttl → ex:dan (no foaf:knows) → exactly 1 violation.
    // The violation entry shows the focus node using ex: prefix, so "dan" appears.
    await expect(reportPane.getByText(/dan/i).first()).toBeVisible();

    // Disabled shape-age-max.ttl violations MUST NOT appear: alice (age 30) and carol (age 41)
    // would show if that shape were active, but it is disabled.
    // The violation count in the report header must be 1 (only ex:dan).
    await expect(reportPane.getByText("1 violation")).toBeVisible();
  });
});
