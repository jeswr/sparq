// [OPUS-4.8] sq-vw3ax.11 — headless smoke test for the /download DIRECT-download redesign.
//
// WHAT IT GUARDS. /download turns every button into a one-click DIRECT download of the right
// per-platform file via GitHub's version-stable `releases/latest/download/<alias>` endpoint, and
// ENRICHES each card (version / size / copyable sha256) from an unauthenticated
// `api.github.com/repos/jeswr/sparq/releases/latest` fetch. Two states must both be correct:
//
//   * NO RELEASE YET (API 404 — the current live reality): every download control degrades to a
//     subdued "No release yet — watch releases" link to the Releases page, and the unsigned-build
//     banner stays front-and-centre. That Releases link is the ONLY GitHub-page link retained.
//   * READY (API 200): the buttons are live direct-download links whose href is the STATIC
//     `…/releases/latest/download/<alias>` (needs no per-release rebuild), the version tag renders,
//     and the collapsed details expose a copyable sha256.
//
// The GitHub API is MOCKED via page.route so the test is hermetic — it never depends on live
// GitHub state or network, and it deterministically exercises BOTH states. It is a CORRECTNESS
// smoke test (DOM + hrefs + console), never a timing threshold.
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";

// Block the coi-serviceworker for this spec. The shim re-issues EVERY fetch (incl. cross-origin)
// from inside the service worker, which bypasses page.route — so the api.github.com mock below
// would never intercept. The /download page needs no cross-origin isolation (it only does a plain
// CORS fetch + renders static links), so blocking the SW is safe here and makes the mock
// deterministic. (Other specs that DO need the SW keep the config default.)
test.use({ serviceWorkers: "block" });

const API_GLOB = "**/api.github.com/repos/jeswr/sparq/releases/latest";
const RELEASES_URL = "https://github.com/jeswr/sparq/releases";
const LATEST_DL = "https://github.com/jeswr/sparq/releases/latest/download";

// The light site-e2e CI lane builds no wasm bundle, so any optional-wasm fetch 404s benignly;
// filter that one class of resource error while still catching every REAL client-side fault.
const BENIGN_RESOURCE_404 = /Failed to load resource:.*\b404\b/i;
// Blocking the service worker (test.use above) makes the coi-serviceworker registration script
// throw a benign "reading 'scope'" TypeError — a test-only artifact of the block, never a product
// fault (in a real browser the SW registers and returns a registration). Filter exactly that.
const BENIGN_SW_BLOCK = /reading 'scope'|ServiceWorker.*(register|controller)/i;

function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() !== "error") return;
    const text = msg.text();
    if (BENIGN_RESOURCE_404.test(text) || BENIGN_SW_BLOCK.test(text)) return;
    errors.push(text);
  });
  page.on("pageerror", (err) => {
    const text = String(err);
    if (BENIGN_SW_BLOCK.test(text)) return;
    errors.push(text);
  });
  return errors;
}

// With the coi-serviceworker blocked (test.use above) there is no one-time reload to absorb, so a
// plain domcontentloaded navigation is enough — the assertions below auto-wait for hydration.
async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
}

// A minimal `releases/latest` payload with the Windows GUI alias the Playwright Desktop-Chrome
// device's OS detection promotes (its device UA pins "Windows NT 10.0"), plus a version-stable
// digest.
const FAKE_SHA = "a".repeat(64);
const READY_BODY = JSON.stringify({
  tag_name: "v9.9.9-test",
  assets: [
    {
      name: "sparq-gui-win-x64.msi",
      size: 94371840, // 90.0 MB
      digest: `sha256:${FAKE_SHA}`,
    },
  ],
});

async function mockApi(page: Page, status: number, body: string): Promise<void> {
  await page.route(API_GLOB, async (route) => {
    await route.fulfill({
      status,
      // CORS + CORP headers so the fetch is allowed under the site's cross-origin isolation.
      headers: {
        "content-type": "application/json",
        "access-control-allow-origin": "*",
        "cross-origin-resource-policy": "cross-origin",
      },
      body,
    });
  });
}

test("no-release state: buttons degrade to a Releases link and the unsigned banner stays", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await mockApi(page, 404, JSON.stringify({ message: "Not Found" }));
  await gotoSettled(page, "download/");

  // The unsigned-build honesty banner is present in this state too.
  await expect(page.getByText("Developer builds — unsigned.")).toBeVisible();

  // Every download control is the "No release yet — watch releases" link to the Releases page.
  const watch = page.getByRole("link", { name: /No release yet — watch releases/i }).first();
  await expect(watch).toBeVisible();
  await expect(watch).toHaveAttribute("href", RELEASES_URL);

  // The no-install closer still offers a working no-install entry point — it now links to /app
  // ("Open the workbench") since /try was removed and /app is the single workbench (sq-4hiqe).
  // [SONNET-4.6] sq-4hiqe — text was "Open the playground"; the /try REPL removal renamed it.
  await expect(page.getByRole("link", { name: /Open the workbench/i })).toBeVisible();

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("ready state: the primary button is a static direct-download link with version + sha256", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await mockApi(page, 200, READY_BODY);
  await gotoSettled(page, "download/");

  // The Playwright Desktop-Chrome device (Windows UA) promotes the Windows platform to the primary row.
  await expect(page.getByText("For your Windows PC")).toBeVisible();

  // The primary download button is the STATIC latest/download alias URL (no per-release rebuild).
  // Locate it by href (unambiguous) rather than by its combined accessible name.
  const primary = page.locator(
    `a[href="${LATEST_DL}/sparq-gui-win-x64.msi"]`,
  );
  await expect(primary).toBeVisible();
  await expect(primary).toContainText("Download sparq-gui.msi");
  // The ready-state meta (version · size) is appended to the button in the ready state.
  await expect(primary).toContainText("v9.9.9-test");

  // The version tag also renders as the "Desktop app" section badge.
  await expect(page.getByText("v9.9.9-test").first()).toBeVisible();

  // The collapsed details expose the copyable sha256 from the asset digest.
  await page.getByText("First launch & verification").first().click();
  await expect(page.getByText(FAKE_SHA).first()).toBeVisible();

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
