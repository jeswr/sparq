// [SONNET-4.6] sq-iemfq — server-tool Playwright spec (web persona).
//
// Runs under the chromium-web project (*.web.spec.ts pattern): NO Tauri globals, pure
// browser persona. The webPersona auto-fixture (support/web-fixtures.ts) has already:
//   1. Blocked external network (only 127.0.0.1 / localhost allowed through).
//   2. Navigated to "/" with no Tauri init script.
//   3. Waited for "Engine ready" in the top bar.
//
// This spec drives the Server tool panel — the SPARQL 1.1 Protocol endpoint client —
// using page.route() to mock a hermetic sparq-server at http://127.0.0.1:7777/sparql.
//
// Determinism rules: NO waitForTimeout; web-first assertions only; stable selectors
// (data-* / role only); no exact numeric assertions on timing or row counts.

import { webTest as test, webExpect as expect } from "../support/index.ts";

// Mock endpoint — loopback, allowed through the hermetic context-level route.
const MOCK_URL = "http://127.0.0.1:7777/sparql";

// Canned SPARQL-JSON SELECT result for the default query.
const CANNED_SELECT = JSON.stringify({
  head: { vars: ["subject", "predicate", "object"] },
  results: {
    bindings: [
      {
        subject: { type: "uri", value: "http://example.org/alice" },
        predicate: { type: "uri", value: "http://xmlns.com/foaf/0.1/name" },
        object: { type: "literal", value: "Alice" },
      },
    ],
  },
});

test.describe("server-tool (web persona)", () => {
  // The webPersona auto-fixture navigates to "/" and waits for engine ready before each test.

  test("connect to mock endpoint, run query, assert bindings table", async ({ page }) => {
    // ── 1. Install page.route mocks before navigating to the server tab ──────────────────────
    // The context-level hermetic block allows all http://127.0.0.1 regardless of port.
    // Page-level routes intercept BEFORE the context route, so these fire first.

    // /health → "ok" (plaintext)
    await page.route("http://127.0.0.1:7777/health", (route) => {
      void route.fulfill({ status: 200, body: "ok", contentType: "text/plain" });
    });

    // /metrics → 404 (feature off — not-exposed)
    await page.route("http://127.0.0.1:7777/metrics", (route) => {
      void route.fulfill({ status: 404, body: "not found", contentType: "text/plain" });
    });

    // /.well-known/void → 404 (opt-in feature off — not-exposed)
    await page.route("http://127.0.0.1:7777/.well-known/void", (route) => {
      void route.fulfill({ status: 404, body: "not found", contentType: "text/plain" });
    });

    // /sparql → GET = 400 (SD feature off / no `query` param — treated as not-exposed);
    //           POST = canned SELECT result
    await page.route("http://127.0.0.1:7777/sparql", (route) => {
      if (route.request().method() === "POST") {
        void route.fulfill({
          status: 200,
          body: CANNED_SELECT,
          contentType: "application/sparql-results+json",
        });
      } else {
        // GET — Service Description not exposed (400 treated as not-exposed by fetchServerHealth)
        void route.fulfill({ status: 400, body: "missing query", contentType: "text/plain" });
      }
    });

    // ── 2. Navigate to the Server tool tab ────────────────────────────────────────────────────
    await page.locator('[data-tool="server"]').click();

    // ── 3. Fill the endpoint URL ──────────────────────────────────────────────────────────────
    await page.locator('[data-server-url]').fill(MOCK_URL);

    // ── 4. Click Connect ──────────────────────────────────────────────────────────────────────
    await page.locator('[data-server-connect]').click();

    // ── 5. Assert health shows "Live" ─────────────────────────────────────────────────────────
    // The health badge appears after the async fetchServerHealth resolves.
    await expect(page.locator('[data-server-health-status]')).toContainText("Live");

    // ── 6. Run the default query ──────────────────────────────────────────────────────────────
    await page.locator('[data-server-run-query]').click();

    // ── 7. Assert the bindings table shows Alice ──────────────────────────────────────────────
    const table = page.locator('[data-server-result-table]');
    await expect(table).toBeVisible();
    await expect(table.getByText("Alice").first()).toBeVisible();
  });

  test("connect error is shown plainly when endpoint unreachable", async ({ page }) => {
    // No page.route mock for port 7778 — requests will reach the network stack
    // and fail with connection refused (the port has no listener).
    // The hermetic context route allows 127.0.0.1 regardless of port, so the
    // fetch is attempted and fails with a transport error caught by runEndpointQuery.
    const DEAD_URL = "http://127.0.0.1:7778/sparql";

    // Navigate to Server tool.
    await page.locator('[data-tool="server"]').click();

    // Fill the unreachable endpoint URL.
    await page.locator('[data-server-url]').fill(DEAD_URL);

    // Connect (health fetch will fail, but the UI transitions to connected state immediately).
    await page.locator('[data-server-connect]').click();

    // The query panel is visible — we can attempt a query right away.
    await expect(page.locator('[data-server-run-query]')).toBeVisible();

    // Run the query against the unreachable endpoint.
    await page.locator('[data-server-run-query]').click();

    // An honest error is shown in the results pane — no fabricated result.
    await expect(page.locator('[data-server-result-error]')).toBeVisible();
  });
});
