// [OPUS-4.8] sq-ymr2e.1 — the FOUNDATION SMOKE + the anti-flake ACCEPTANCE demo.
//
// This is the runner-state-helper demo the bead's acceptance calls out: it must pass
// `--repeat-each=5 --retries=0` (npm run test:e2e:stress). It exercises the whole foundation on a
// WASM-FREE, deterministic surface (the home hero runner's server-rendered idle-preview state), so
// it runs on the light site-e2e CI lane with no wasm bundle and is inherently repeatable:
//   * the runner-state waiter reaches "idle-preview" via a stable, role/text anchor (no sleep);
//   * the hermetic layer recorded ZERO external requests (the zero-network invariant, in miniature);
//   * the frozen clock pins Date across a reload; the seeded PRNG is reproducible across a reload.
// Design of record: research/web-gui-test-program.md §1.
import { test, expect, BasePage, FROZEN_TIME } from "./index";

test.describe("E2E foundation (determinism harness + hermetic network)", () => {
  test("home reaches the idle-preview runner state with zero external network", async ({
    page,
    hermetic,
  }) => {
    const home = new BasePage(page);
    await home.goto(""); // the /sparq/ home route

    // The runner-state waiter reaches idle-preview deterministically (server-rendered, wasm-free).
    await home.expectRunnerState("home", "idle-preview", { timeout: 15_000 });

    // Hermetic invariant: nothing left the localhost origin during load (page-originated requests).
    expect(
      hermetic.externalRequests,
      `unexpected external requests:\n${hermetic.externalRequests.join("\n")}`,
    ).toEqual([]);
  });

  test("the frozen clock pins Date across a reload", async ({ page }) => {
    const home = new BasePage(page);
    await home.goto("");

    const before = await page.evaluate(() => Date.now());
    expect(before).toBe(FROZEN_TIME.getTime());

    await page.reload({ waitUntil: "domcontentloaded" });
    const after = await page.evaluate(() => Date.now());
    expect(after).toBe(FROZEN_TIME.getTime());
  });

  test("Math.random is deterministically seeded (stable across reloads)", async ({ page }) => {
    // Capture the FIRST three draws immediately after the harness seeds Math.random, before any
    // app code runs — this init script is registered after the harness's seed init script, so it
    // observes the fresh generator state. That makes the assertion independent of how much
    // randomness the app later consumes during hydration (which varies run-to-run); comparing raw
    // draws across two full page loads would be flaky for exactly that reason.
    await page.addInitScript(
      "window.__sparqSeedProbe = [Math.random(), Math.random(), Math.random()];",
    );

    const home = new BasePage(page);
    await home.goto("");
    const first = (await page.evaluate("window.__sparqSeedProbe")) as number[];

    await page.reload({ waitUntil: "domcontentloaded" });
    const second = (await page.evaluate("window.__sparqSeedProbe")) as number[];

    // Same seed every navigation ⇒ identical fresh draws (deterministic, not asserting magic values).
    expect(second).toEqual(first);
    // And it is actually the PRNG, not a constant: the three draws differ.
    expect(new Set(first).size).toBe(3);
  });
});
