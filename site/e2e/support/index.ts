// [OPUS-4.8] sq-ymr2e.1 — the ONE import surface for the site Playwright suite.
//
// Spec authors write `import { test, expect, BasePage } from "./support"` (adjust the relative
// depth) and get the hermetic + deterministic `test`, the shared page object, the runner-state
// waiters, and the test-data helpers. Design of record: research/web-gui-test-program.md §1 +
// e2e/support/README.md.
export { test, expect } from "./fixtures";
export type { HermeticController } from "./fixtures";

export { waitForAppReady, gotoAppReady } from "./app-ready";

export {
  expectRunnerState,
  runnerStateLocator,
  RUNNER_STATE_TIMEOUT,
} from "./runner-state";
export type { RunnerState, RunnerSurface, RunnerStateOptions } from "./runner-state";

export { BasePage } from "./page-objects/base-page";

export { FROZEN_TIME, RANDOM_SEED } from "./determinism";

export type { HermeticPolicy, HermeticController as HermeticNetController } from "./hermetic-net";

export {
  GITHUB_RELEASE_GLOB,
  GITHUB_RELEASE_RE,
  githubFixtureResponse,
  isGithubReleaseUrl,
} from "./test-data/github";
export type { GithubFixtureName } from "./test-data/github";

export * as rdf from "./test-data/rdf";
