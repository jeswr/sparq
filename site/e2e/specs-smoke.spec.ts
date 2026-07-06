// [OPUS-4.8] sq-rvgr2.1 — CRITICAL-FLOW smoke test for the /specs section (the ReSpec-style
// Unofficial Proposal Drafts). Mirrors home-smoke: it asserts the section BOOTS and renders
// its load-bearing pieces, and (the regression net) that the route produces zero console
// errors.
//
// NO WASM NEEDED. The /specs index and each per-spec page are fully server-rendered static
// content (the ReSpec HTML fragment is injected at build time — no engine, no client JS), so
// this spec runs on EVERY lane, including the light site-e2e CI lane that builds no wasm
// bundle. It is a CORRECTNESS smoke test (DOM + console), never a wall-clock threshold.
//
// EMPIRICAL HONESTY. The load-bearing assertion is that the per-spec page states plainly it is
// an Unofficial Proposal Draft with NO W3C standing — the honesty invariant this section MUST
// preserve.
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";

const BENIGN_RESOURCE_404 = /Failed to load resource:.*\b404\b/i;

function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() !== "error") return;
    const text = msg.text();
    if (BENIGN_RESOURCE_404.test(text)) return; // optional wasm bundle absent on the light lane
    errors.push(text);
  });
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

// The coi-serviceworker shim reloads the tab ONCE on a fresh visit; wait until the SW is
// controlling the page before asserting (the deterministic "reload already fired" signal).
async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
  await page
    .waitForFunction(() => navigator.serviceWorker?.controller != null, undefined, {
      timeout: 10_000,
    })
    .catch(() => {});
}

test("the /specs index renders the section header and the template draft card", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoSettled(page, "specs/");

  await expect(page.getByRole("heading", { name: "Specs", level: 1 })).toBeVisible();
  // The template draft card is present (its title is a card <div>, not a heading), with a
  // "Read draft" link into the per-spec page.
  await expect(
    page.getByText(/Proposed Specifications — Template/i).first(),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: /Read draft/i }).first()).toBeVisible();

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});

test("a per-spec page renders ReSpec conventions and states it has NO W3C standing", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoSettled(page, "specs/proposed-specifications-template/");

  // The ReSpec-style document header (title + status line).
  await expect(
    page.getByRole("heading", { name: /Proposed Specifications — Template/i, level: 1 }),
  ).toBeVisible();

  // The Status-of-This-Document box makes the honest "no standing" claim (empirical-honesty).
  await expect(page.getByText(/Unofficial Proposal Draft/i).first()).toBeVisible();
  await expect(page.getByText(/not a W3C standard/i).first()).toBeVisible();
  await expect(page.getByText(/no official standing/i).first()).toBeVisible();

  // The generated Table of Contents links to a numbered section anchor.
  const toc = page.locator("nav#toc");
  await expect(toc).toBeVisible();
  await expect(toc.getByRole("link", { name: /Introduction/i })).toHaveAttribute(
    "href",
    /#introduction$/,
  );

  // A numbered section heading rendered from the ToC target.
  await expect(page.locator("#introduction")).toBeVisible();

  // The PDF download affordance is present and points at the basePath-prefixed asset.
  await expect(
    page.getByRole("link", { name: /Download PDF/i }),
  ).toHaveAttribute("href", /\/specs\/proposed-specifications-template\.pdf$/);

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
