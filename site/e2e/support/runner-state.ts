// [OPUS-4.8] sq-ymr2e.1 — WEB-FIRST RUNNER-STATE WAITERS.
//
// Design of record: research/web-gui-test-program.md §1.1 — assert on the WASM runner's EXPLICIT
// UI states, never a `waitForTimeout`. This maps each surface's runner states to a STABLE locator
// (getByRole / getByText / data-testid / data-* — never a Tailwind class, §1.3) and web-first
// asserts it visible with one generous timeout for WASM instantiate (never a timing ASSERTION —
// §1.7: the timeout is a bound, not a measured value).
//
// The maps are keyed on the anchors that already ship today (the ZK prover `data-prover` pill, the
// SHACL "Engine ready" pill, the home hero's preview pill + `data-hero-results-table`). Where a
// component does not yet expose a state, a sibling journey bead adds a `data-testid`/`data-*` hook
// + a map entry here — that is the ONE shared extension point this foundation deliberately leaves
// open (test hooks, not behaviour changes; per the design §8).
// [OPUS-4.8] sq-4hiqe — the "try" surface (the removed /try REPL) was dropped from the map.
import { expect, type Locator, type Page } from "@playwright/test";

/** A surface that hosts a WASM runner with observable UI states. */
export type RunnerSurface = "home" | "zk" | "shacl";

/** The canonical runner lifecycle states (superset across surfaces; each map is partial). */
export type RunnerState =
  | "idle-preview"
  | "starting-engine"
  | "engine-ready"
  | "running"
  | "results"
  | "error";

type StateResolver = (page: Page) => Locator;

/** Per-surface state → stable-locator resolvers. */
const SURFACE_MAPS: Record<RunnerSurface, Partial<Record<RunnerState, StateResolver>>> = {
  // Home hero runner (src/components/home/hero-runner.tsx). idle-preview is server-rendered and
  // WASM-FREE — the deterministic anchor the foundation smoke drives on the light lane.
  home: {
    "idle-preview": (p) => p.getByText(/press Run to compute it live/i),
    "starting-engine": (p) => p.getByText(/Starting engine/i),
    running: (p) => p.getByText(/^Running…/),
    results: (p) => p.locator("[data-hero-results-table]"),
    error: (p) => p.getByRole("alert"),
  },
  // ZK car-hire prover (src/components/zk-car-hire.tsx) — explicit data-* states already ship.
  zk: {
    "starting-engine": (p) => p.locator('[data-prover="warming"]'),
    "engine-ready": (p) => p.locator('[data-prover="ready"]'),
    error: (p) => p.locator('[data-prover="error"]'),
    results: (p) => p.locator("[data-proof-result]"),
  },
  // SHACL playground (src/components/shacl-playground.tsx).
  shacl: {
    "engine-ready": (p) => p.getByText("Engine ready"),
    results: (p) => p.locator('[data-testid="shacl-report"]'),
  },
};

/** Generous bound for WASM instantiate; a BOUND, never a measured/asserted value (§1.7). */
export const RUNNER_STATE_TIMEOUT = 90_000;

export type RunnerStateOptions = { timeout?: number };

/** The stable locator for a surface's runner state (throws if the surface has no such anchor). */
export function runnerStateLocator(
  page: Page,
  surface: RunnerSurface,
  state: RunnerState,
): Locator {
  const resolver = SURFACE_MAPS[surface][state];
  if (!resolver) {
    throw new Error(
      `No runner-state anchor for surface="${surface}" state="${state}". Add a getByRole/` +
        `data-testid hook to the component + a map entry in e2e/support/runner-state.ts ` +
        `(see e2e/support/README.md).`,
    );
  }
  return resolver(page);
}

/** Web-first assert the surface has reached `state`. Returns the locator for chained assertions. */
export async function expectRunnerState(
  page: Page,
  surface: RunnerSurface,
  state: RunnerState,
  opts: RunnerStateOptions = {},
): Promise<Locator> {
  const locator = runnerStateLocator(page, surface, state).first();
  await expect(locator).toBeVisible({ timeout: opts.timeout ?? RUNNER_STATE_TIMEOUT });
  return locator;
}
