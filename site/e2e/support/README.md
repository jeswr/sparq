<!-- [OPUS-4.8] sq-ymr2e.1 -->
# site E2E foundation (`e2e/support/`)

Shared harness for the site Playwright suite. Design of record:
`research/web-gui-test-program.md` (§1 determinism doctrine + the E2E-foundation section).
Sibling journey/a11y/visual suites (`sq-ymr2e.2`–`.12`) build on this; each owns its own
spec file, and this directory is the one shared surface they consume read-only.

## Use it

```ts
import { test, expect, BasePage } from "./support"; // adjust relative depth from your spec
// or the full barrel: import { test, expect, BasePage, rdf, expectRunnerState } from "./support";

test("…", async ({ page, hermetic }) => {
  const home = new BasePage(page);
  await home.goto(""); // route is relative to the /sparq/ baseURL; waits for app-ready
  await home.expectRunnerState("home", "idle-preview"); // web-first, never a sleep
  expect(hermetic.externalRequests).toEqual([]);        // zero-network invariant
});
```

Importing this `test` gives you, with no boilerplate:

- **Determinism** (`determinism.ts`): frozen `Date` (`page.clock.setFixedTime`, timers stay live so
  WASM prewarm is not deadlocked), seeded `Math.random` (mulberry32; WebCrypto is untouched so ZK
  proofs are unaffected), animations/transitions zeroed. Viewport 1280×720, dark colour-scheme,
  reduced-motion, UTC, `en-US` are pinned in `playwright.config.ts` (context options, so they apply
  to every spec).
- **Hermetic network** (`hermetic-net.ts`): all non-localhost requests are **blocked** and recorded
  on `hermetic.externalRequests`; the one external call the product makes —
  `api.github.com/repos/*/sparq/releases/latest` — is fixture-routed from
  `test-data/fixtures/*.json`. Pick the variant with `test.use({ github: "release" | "notFound" |
  "error" | "off" })`. Widen the allowlist only with justification: `test.use({ allowHosts: [...] })`.
  - **coi / WASM-threads seam:** a spec whose runtime needs `SharedArrayBuffer` multithreading (the
    ZK prover's bb.js) relies on the coi-serviceworker's cross-origin isolation (COOP/COEP header
    injection), which blanket `context.route` interception disrupts — the prover then never reaches
    ready. Such specs opt out with `test.use({ hermeticNetwork: false })` (they keep the determinism
    harness). Safe, because those surfaces are same-origin and zero-network by construction, so the
    block would only ever be a no-op. `zk-prewarm.spec.ts` is the reference.

## The doctrine (the seven rules — from the design record §1)

1. **No time-based waits.** Zero `waitForTimeout`/sleeps — **grep-gated** (`test:e2e:grep-gate`).
   Use `expectRunnerState(...)`, web-first `expect(...).toBeVisible()`, or `waitForAppReady`.
   If a state is not observable, add a `data-*`/role hook to the component + a `runner-state.ts`
   map entry — do not poll.
2. **Hermetic network.** A test that touches the real network is a bug. External APIs are
   fixture-routed. To also seal the coi-serviceworker bypass (needed for the `/download` github
   assertions and to OBSERVE every request for the zero-network invariant), add
   `test.use({ serviceWorkers: "block" })` — the `download-page.spec.ts` pattern.
3. **Stable selectors.** `getByRole` / `data-testid` / `data-*` only — never Tailwind classes.
4. **Pinned renderers.** Browsers are pinned by the Playwright version lock (see the visual bead).
5. **Anti-flake bar.** Every new spec must pass `npm run test:e2e:stress` (`--repeat-each=5
   --retries=0`). CI keeps `retries=1` + trace-on-first-retry as telemetry — a pass-on-retry is a
   defect to fix, not a success.
6. **Parallel by default.** `fullyParallel` is on; serialize only genuinely shared state.
7. **Never assert a timing value.** Latency/size/version strings are presence-checked or masked.

## What's here

| file | role |
|---|---|
| `fixtures.ts` | the extended `test`/`expect` (hermetic + determinism auto-fixtures + `github`/`allowHosts` options) |
| `determinism.ts` | frozen clock, seeded PRNG, animations-off; `applyDeterminism(page)` |
| `hermetic-net.ts` | request classifier + block/route policy + `HermeticController` |
| `app-ready.ts` | `waitForAppReady` / `gotoAppReady` (SW-reload + hydration barrier, no sleep) |
| `runner-state.ts` | `RunnerState` × surface → stable-locator map; `expectRunnerState` |
| `page-objects/base-page.ts` | `BasePage` (nav + ⌘K palette + runner-state); subclass per surface |
| `test-data/github.ts` + `fixtures/*.json` | the checked-in releases/latest fixtures + matcher |
| `test-data/rdf.ts` | small deterministic RDF/SPARQL fixtures for the home + runner journeys |
| `index.ts` | the barrel spec authors import |
| `foundation.smoke.spec.ts` | the runner-state demo + determinism/hermetic proof (the stress-bar spec) |

## CI, gating & flake-quarantine

`.github/workflows/site-e2e-foundation.yml` runs this foundation smoke (+ the axe a11y scan) as an
**advisory** (`continue-on-error`) lane — it never gates the merge queue. The site journey/visual
lanes (`site-e2e-hero.yml`, `site-visual.yml`) are likewise advisory.

Promotion of any of these deterministic lanes to a **required** check is governed by the
checked-in policy **[`.github/E2E-GATING-POLICY.md`](../../../.github/E2E-GATING-POLICY.md)**
(design of record: `research/web-gui-test-program.md` §6.3). In brief:

- **Probation bar:** a lane earns gating only after **50 consecutive green runs on `main`
  spanning ≥ 10 distinct PRs, OR two weeks — whichever is LONGER**, with zero quarantine events in
  the window; evidence is linked in the promotion PR + the policy's ledger. Promotion is a one-line
  flip (drop ` (advisory)` from the job name). The visual subset promotes separately and last.
- **Flake-quarantine:** a test that passes-on-retry twice within 7 days is quarantined the same day
  (`test.fixme`) with a P2 fix bead filed same-day; quarantined tests cannot gate. CI keeps
  `retries=1` + `trace: on-first-retry` as diagnostics — never to hide a flake. The lane must never
  train contributors to re-run.
