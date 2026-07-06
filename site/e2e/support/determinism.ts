// [OPUS-4.8] sq-ymr2e.1 — the DETERMINISM HARNESS for the site Playwright suite.
//
// Design of record: research/web-gui-test-program.md §1 (the determinism doctrine that goes
// verbatim into every child bead). This module makes a run reproducible so a failure is a real
// defect, never environment noise — the load-bearing property that lets these lanes eventually
// GATE (§6.3: "a flaky gate is worse than no gate").
//
// What it pins, and — deliberately — what it does NOT:
//   * Frozen Date via `page.clock.setFixedTime`: `Date.now()`/`new Date()` return a fixed instant,
//     so any date-dependent render (and any leaked wall-clock string) is stable. Crucially we use
//     setFixedTime, NOT `clock.install()`: install() also PAUSES setTimeout/rAF, which would
//     deadlock the WASM prewarm/runner flows that await real timers. setFixedTime keeps every timer
//     running on real time — safe for the wasm surfaces — while still fixing the calendar clock.
//   * Seeded Math.random (mulberry32): deterministic PRNG so any Math.random-derived value is
//     reproducible across reloads. The ZK/crypto paths use WebCrypto (crypto.getRandomValues),
//     which we do NOT touch — so seeding cannot weaken any proof, only make UI randomness stable.
//   * Animations/transitions disabled via an injected stylesheet, complementing the context-level
//     `reducedMotion: "reduce"` set in playwright.config.ts — belt-and-suspenders so a
//     JS/inline-driven animation can't race an assertion.
// Viewport / colour-scheme / timezone / locale are pinned at the CONFIG `use` level (they are
// context options), so they apply to every spec — see playwright.config.ts.
//
// Rule 7 ("never assert a timing value") is enforced by CONVENTION in the specs, not here; this
// module only removes the SOURCES of nondeterminism.
import type { Page } from "@playwright/test";

/** The single fixed instant every spec's clock reads. Arbitrary but stable. */
export const FROZEN_TIME = new Date("2025-01-02T03:04:05.678Z");

/** The fixed PRNG seed. Any 32-bit value; stable is all that matters. */
export const RANDOM_SEED = 0x5eed1234;

/** Runs IN THE PAGE (serialised by addInitScript): replace Math.random with a seeded mulberry32
 *  PRNG so the sequence is identical on every navigation. Standalone/self-contained by design. */
function installSeededRandom(seed: number): void {
  let a = seed >>> 0;
  Math.random = () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Runs IN THE PAGE: inject a stylesheet that zeroes every animation/transition. Idempotent, and
 *  re-applied if the <head> is replaced (SPA route change) so it never lapses mid-run. */
function installNoAnimations(): void {
  const STYLE_ID = "__sparq_e2e_no_anim";
  const CSS =
    "*,*::before,*::after{" +
    "animation-duration:0s!important;animation-delay:0s!important;" +
    "transition-duration:0s!important;transition-delay:0s!important;" +
    "scroll-behavior:auto!important;" +
    "}";
  const inject = () => {
    const head = document.head;
    if (!head || document.getElementById(STYLE_ID)) return;
    const style = document.createElement("style");
    style.id = STYLE_ID;
    style.textContent = CSS;
    head.appendChild(style);
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", inject, { once: true });
  } else {
    inject();
  }
}

/**
 * Apply the whole determinism harness to a page. Called once per test by the auto-fixture in
 * fixtures.ts BEFORE the test navigates, so it covers the first document and every later one.
 */
export async function applyDeterminism(page: Page): Promise<void> {
  // Fix the calendar clock but keep timers live (see the module header for why not install()).
  await page.clock.setFixedTime(FROZEN_TIME);
  // These run on every fresh document/frame (that is addInitScript's contract).
  await page.addInitScript(installSeededRandom, RANDOM_SEED);
  await page.addInitScript(installNoAnimations);
}
