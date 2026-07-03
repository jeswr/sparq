# Web + GUI test program — E2E, accessibility, and UX-regression (sq-ymr2e)

> 🤖 SPARQ agent design record (architect stage, Claude Fable 5). Decomposition of epic
> **sq-ymr2e** ("appropriate e2e/a11y/UX-regression coverage for the interface") into a
> prioritized, **disjoint** implementation plan (beads sq-ymr2e.1–.12). DESIGN + DECOMPOSE
> only — no tests are implemented here. Prose-only design pass: reasoned from the design
> records (`research/site-redesign-home-try-app-download.md`, `research/gui-design.md`,
> `research/website-redesign.md`, the `frontend-design` skill) **without** opening component
> source or workflow YAML; file paths below are quoted from those records — implementers
> verify against the tree. Assumptions flagged in §8.

## 0. Ground truth + the problem

Two frontends, deliberately different products (see the `frontend-design` skill): the
**Next.js site** (`site/`, static export to GitHub Pages, ~16 surfaces — home with a live
WASM hero runner, `/try` the flagship in-browser SPARQL workbench, `/app` a hard-nav to the
hosted GUI overlay, `/download`, `/papers`, `/specs`, benchmarks, per-surface showcases) and
the **Tauri 2 desktop GUI** (query workbench over the native engine: status bar wired to
real engine state, inference toggles, dataset export). Both consume the sparq WASM/engine.

Current coverage is a handful of smokes: a `site-nav` spec, a SHACL `__wbg_ptr` regression
guard, `zk-prewarm`, and an **advisory, notoriously flaky** `GUI tauri-driver e2e (Linux)`
check. There is **no systematic a11y testing and no visual/UX-regression detection** — a
real ARIA tab/panel bug already shipped on `/try` and was found by hand.

Honesty framing:

- **E2E green ≠ correct engine.** These suites test the *interface contract* (UI states,
  journeys, honesty claims rendered as UI). SPARQL correctness is owned by the conformance
  + differential lanes; a journey asserts *specific result values* only as evidence the
  UI→engine→UI plumbing is intact, not as a semantics oracle.
- **A flaky gate is worse than no gate** — it trains contributors to re-run and erodes
  every other gate's authority (the tauri-driver lesson). Determinism is therefore a
  first-class design constraint (§1), not a cleanup pass, and **nothing gates until it has
  earned it** (§6.3).
- **Automated a11y is a floor, not a certificate.** axe catches roughly the mechanical
  half of WCAG; the keyboard/focus behaviors it cannot see get explicit Playwright
  assertions (§3.2). We claim "zero known serious/critical automated violations at WCAG
  2.1 AA", never "WCAG-compliant".

## 1. Determinism doctrine (goes verbatim into every bead brief)

1. **No time-based waits.** Zero `waitForTimeout`/sleeps (grep-gated). Web-first
   assertions on the runner's *explicit UI states* (idle-preview / starting-engine /
   running / results / error — the states the site design already specifies) with one
   generous per-assertion timeout for WASM instantiation. If a state is not observable,
   the fix is a `data-state` attribute in the component, not a poll loop in the test.
2. **Hermetic network.** Block all non-localhost requests by default; external APIs
   (the `/download` GitHub releases call) are fixture-routed. A test that touches the
   real network is a bug.
3. **Stable selectors.** `getByRole`/`data-testid` only — never Tailwind classes (they
   churn with every restyle; that brittleness *is* the UX-regression false-positive).
4. **Pinned renderers.** Browsers pinned by the Playwright version lock; the visual
   project additionally runs only inside the digest-pinned Playwright container (§4).
5. **Anti-flake acceptance bar.** Every new spec must pass `--repeat-each=5 --retries=0`
   locally/pre-merge. CI keeps `retries=1` with trace-on-first-retry **as telemetry**: a
   pass-on-retry is a defect to fix, not a success (quarantine policy, §6.3).
6. **Parallel by default.** Serialize only what genuinely shares state (the
   `sessionStorage` handoff test); everything else runs fully parallel workers.
7. **Never assert a timing value.** Latency/size/version strings are presence-checked or
   masked — the no-hard-coded-perf-numbers rule applies to test expectations too.

## 2. E2E — critical user journeys (Playwright)

Per-PR journeys run **headless chromium only**; firefox/webkit run nightly (§6.2). The
positions below name the P1 (core-value: if this breaks, the product pitch is broken) vs
P2 (important, not existential) cut per surface.

### 2.1 Home — the hero runner is the product claim (bead sq-ymr2e.3, P1)

- **P1** Idle→results: the first fold shows the labelled preview table (never a skeleton)
  → Run → results state renders the *computed* aggregate answer of the shipped sample
  (exact values, DESC order — an answer a static site could not credibly fake) + the
  proof-line footer.
- **P1** **Zero-network invariant**: record every request between Run-click and
  results-render and assert **zero** — this converts the site's load-bearing honesty copy
  ("runs in your browser · 0 network requests", explicitly scoped to query execution) into
  a tested invariant. Pre-Run wasm/JS chunk loads are excluded by scoping the listener.
- **P1** Query/Data tab switch: edit the data → run → changed results (the "sample is
  hackable" promise).
- **P1** Error state: invalid query → error strip between editor and results; previous
  content dims, never blanks; editing clears it.
- **P1** Handoff: "Open in workbench" → `/try` holds the handed-off query+data
  (`sessionStorage` consumed-and-cleared).
- **P2** Section presence / CTA budget (≤3 in the fold) — cheap structural guards.

### 2.2 /try — the flagship interactive surface (bead sq-ymr2e.2, P1)

- **P1** Edit→run→results via the Run button **and** Ctrl/Cmd+Enter; assert concrete
  result values.
- **P1** Share-link round-trip: `/try#q=<base64url>` prepopulates and runs; a link
  produced in one browser context restores in a *fresh* context (the actual share
  scenario, not a same-page reload).
- **P1** Own-data load: paste Turtle into the dataset panel → query returns it (the "load
  your own data" promise).
- **P1** Error states: parse error surfaces the message (line:col where available)
  without blanking prior results.
- **P2** Split-button `Run ▾` → EXPLAIN renders the plan tree; Advanced connect drawer
  collapsed by default with **no endpoint input visible** while disconnected + the
  in-browser status chip (the design's biggest honesty/declutter invariant); Cmd-K
  palette opens/filters/inserts an example. Connected-state remote execution is **out of
  scope** for per-PR e2e (needs a live endpoint → nightly candidate at most).

### 2.3 /download + /app — the conversion surfaces (bead sq-ymr2e.7, P2)

- Release fixture: detected-platform primary button `href` equals the
  `releases/latest/download/<version-agnostic alias>` URL per platform (parametrized
  platform/UA); version+size render from the fixture; checksum details when the fixture
  publishes `SHA256SUMS`. **Assert hrefs — never perform real downloads.**
- 404 fixture: the "no release yet" disabled state with only the releases link.
- API-error fixture: buttons still carry working `latest/download` hrefs (the designed
  graceful degradation).
- `/app`: the App nav slot is a plain **hard** `<a href>` (base-path-prefixed, trailing
  slash) and not client-intercepted — assert a full document navigation is initiated.
  This is the regression guard for the shipped cross-app RSC `.txt` soft-nav bug; the
  deployed overlay itself is untestable locally, so we guard the *mechanism*.
  P2 not P1 only because the journeys are static-DOM assertions (low flake, low cost)
  over a lower-traffic surface; the hard-nav guard is the highest-value single assert.

### 2.4 Navigation + shell (existing `site-nav` spec, extended by sq-ymr2e.8)

Top-bar navigation across destinations, `aria-current` on the active item, 404/redirect
stubs for collapsed routes. Keep the existing spec's assertions (PR #1352 already leans
on its App-slot assert) — extend, don't rewrite.

### 2.5 Content surfaces — one smoke each (bead sq-ymr2e.8, P2)

A parametrized sweep over the route inventory **sourced from the IA source of truth**
(the surfaces data module — so a new surface is swept automatically): every route loads
with zero `pageerror`/`console.error`, an `h1` + `main` landmark, and one per-surface key
artifact (a paper list item on `/papers`; a rendered ReSpec body on `/specs`; the
benchmarks dashboard container — presence only, never a number). The SHACL `__wbg_ptr`
guard stays. This sweep is the cheap early-warning net under everything the journey specs
don't reach.

## 3. Accessibility — WCAG 2.1 AA, two complementary halves

**The bar:** WCAG 2.1 AA, enforced as "zero serious/critical automated violations" plus
behavioral keyboard/focus assertions. All *interactive* surfaces are P1 scope (home
runner, `/try` in its major UI states, `/download`, the palette, drawers, every
tab/panel widget); the full route inventory is swept nightly.

### 3.1 Automated axe half (bead sq-ymr2e.4, P1)

`@axe-core/playwright` pinned to the WCAG 2.1 A+AA rule tags, run against surfaces **in
their key states** (a11y bugs live in the *open* palette, the *expanded* drawer, the
*error* strip — not the pristine page load): home idle+results; `/try` default, palette
open, connect drawer open, error; `/download` release + no-release fixture states.

**Gate shape (firm):** serious+critical = **hard fail at zero from day one** — fix the
findings in the harness PR where small, file per-surface fix beads where structural.
Moderate/minor counts go into `bench/a11y-baseline.json`, a ratchet that may only fall
(the coverage-floor mechanics this repo already trusts). Color-contrast and form-label
rules explicitly enabled; any rule exclusion is documented in the baseline with a reason.
Non-vacuity check: an injected unlabeled icon-button in a fixture must be caught.

### 3.2 Behavioral half — what axe cannot see (bead sq-ymr2e.9, P2)

- Keyboard-only journey on `/try`: Tab to the editor → type → Ctrl/Cmd+Enter → reach
  results by keyboard, with focus-visible assertions along the path.
- **WAI-ARIA tabs pattern on every tab/panel widget** (home Query/Data, the `/try`
  rails/result tabs): `tablist`/`tab`/`tabpanel` roles, `aria-selected`, arrow-key roving
  focus — the exact class of the real bug already found on `/try`, now a regression class.
- Cmd-K palette: focus trap while open; ESC closes **and restores focus to the invoker**.
- Connect drawer: `aria-expanded` disclosure semantics; programmatically-associated
  labels on the endpoint form fields.

## 4. UX-regression detection

Two mechanisms, deliberately separated because their failure economics differ:

**Interaction regression = the journey suite.** "A redesign broke a working flow" is
precisely what §2's P1 journeys detect, because they assert *behavioral contracts*
(states, values, hrefs, keyboard bindings) via roles/testids that survive restyling. This
is the non-brittle half and it gates first.

**Visual regression = a scoped snapshot rig (bead sq-ymr2e.10, P2).** Playwright
`toHaveScreenshot` with brittleness engineered out up front:

- Runs **only inside the digest-pinned official Playwright container** → font rasterizing
  and antialiasing are stable; **baselines are Linux-only by policy** (no per-OS baseline
  zoo; devs run the container locally via the `vr:*` scripts).
- Pinned viewports 1280×720 + 390×844; **dark theme default** (the site is dark-first)
  plus one light-theme home shot; `reducedMotion` + animations disabled; capture waits on
  `document.fonts.ready` **and** the runner's idle/results UI state.
- **Mask, don't chase:** a `data-vr-mask` convention on every dynamic region (timing
  footers, version/size strings, benchmark numbers, paper dates) + Playwright `mask`.
- **Scope is the flake-control:** per-PR shoots only the key layouts — home first fold
  (idle state), `/try` workbench shell, `/download` cards (fixture state), one showcase
  page, nav + palette-open overlay. The full-surface sweep is nightly (sq-ymr2e.11).
- Baseline update is a *reviewed artifact*: intentional visual changes refresh baselines
  **in the same PR** via `vr:update`; failures upload readable diff artifacts. The lane
  starts advisory and must earn gating separately (§6.3) — visual is the most
  brittleness-prone lane, so it gates last.

## 5. The Tauri GUI — invert the pyramid

**Position:** the flaky tauri-driver lane is unfixable *as the primary carrier* of GUI
coverage, because it couples every UI assertion to WebKitWebDriver + a real WebKit webview
+ xvfb + native-engine startup on a shared runner. The fix is architectural, not more
retries: move UI-logic coverage onto a deterministic substrate and shrink the real-IPC
surface to a smoke.

Three layers:

1. **Rust command layer — native unit tests** (exists; per `research/gui-design.md` the
   engine command layer is unit-tested natively). Unchanged.
2. **GUI frontend journeys — Playwright + mocked Tauri IPC (bead sq-ymr2e.5, P1).**
   Drive the built GUI frontend in headless chromium with `@tauri-apps/api/mocks`
   (`mockIPC`/`mockWindows`). Desktop journeys: workbench query (type→run→fixture rows +
   pagination); **inference toggles** (toggle → assert the invoke contract + status chip +
   re-run shows inferred fixture rows); **dataset export** (the export action fires the
   expected `invoke` with format+path args — assert the IPC *contract*, not the file);
   **status-bar state transitions** (store size / backend / last-run latency *present*,
   values masked); keyboard spine (Cmd/Ctrl-Enter, palette). No Tauri binary, no
   WebKitWebDriver, no display server → this is the lane that can become **required**.
3. **tauri-driver — a ≤3-assertion true-integration smoke (bead sq-ymr2e.6, P1).**
   What only it can prove: the app launches; one real query round-trips through real IPC
   to the native engine; the status bar reflects real store state. Stabilization: pin
   `tauri-driver` + the WebKitWebDriver version matched to the runner's webkit2gtk;
   `xvfb-run`; `workers=1`; explicit element polling instead of implicit/sleep waits;
   job-level retry 1 with driver logs + screenshots on failure; a flake probe measuring
   consecutive-green. **Decision rule (pre-authorized):** if the shrunk smoke cannot hit
   20 consecutive green runs, the same PR demotes it to nightly-only. Either way it stays
   **advisory forever** — an environment-coupled native lane never gates the merge queue.

The mock/real seam is the standard risk of layer 2 (the mock can drift from the real
command surface). Mitigation: layer 3's smoke exercises the highest-traffic command
end-to-end, and layer 2's invoke-contract assertions are written against the command
names/arg shapes the native unit tests also pin — drift breaks one side visibly.

## 6. CI wiring — fast, honest, and earned gating

### 6.1 Per-PR (path-scoped to `site/**`, `packages/**`, `gui/**` as relevant)

- **site functional E2E + a11y**: one job, chromium-only, hermetic, parallel workers —
  budget ≈5 min wall-clock. Same workflow as today's `site-e2e` lane (path-scoped,
  `cancel-in-progress`, not merge-queue — the shape `gui.yml` already copied).
- **visual key-layouts**: a second, container-image job in the same workflow (different
  runner image than the functional job) — budget ≈4 min.
- **GUI mockIPC lane**: joins the `gui.yml` family, path-scoped on `gui/**` +
  `packages/**` — headless, no native build needed for the frontend journeys.
- **tauri-driver smoke**: stays in `gui.yml`, advisory, per §5.3 (or nightly if demoted).

### 6.2 Nightly (bead sq-ymr2e.11)

Full route-inventory axe sweep + full-surface visual sweep + the P1 functional journeys
on **firefox + webkit** + the per-platform tauri-driver smoke matrix. Failure routing
opens/refreshes **one consolidated issue/bead per failing category** — never one per test
(alert fatigue is how nightlies die). Nightly never gates `ci-summary`.

### 6.3 Gating discipline — advisory first, promotion earned (bead sq-ymr2e.12)

Everything lands **advisory**. Promotion to required (via the `ci-summary` aggregator,
never raw branch-protection edits) requires a probation bar: **50 consecutive green runs
spanning ≥10 distinct PRs, or two weeks — whichever is longer**, evidence linked in the
promotion PR. Functional E2E + a11y + the mockIPC lane promote first; the visual subset
promotes separately on the same bar; tauri-driver and nightlies never promote.

**Flake-quarantine policy** (codified next to the suites): a test that passes-on-retry
twice within 7 days is quarantined (`test.fixme`) with a P2 fix bead filed same-day;
quarantined tests cannot gate; CI retries stay at 1 with trace-on-first-retry as
diagnostics. The lane's job is to *stay believed* — one tolerated flake and the whole
program regresses to "re-run until green".

Throughput: the per-PR additions are two path-scoped jobs that run only on frontend
changes and never enter the merge queue for Rust-only PRs — net-zero cost to the
workspace's main throughput path.

## 7. The beads (created under sq-ymr2e; disjointness by construction)

Each bead owns **its own new spec file(s) or workflow file** — the only shared surface is
sq-ymr2e.1's support/fixtures directory, which every later spec consumes read-only.
sq-ymr2e.5/.6 (GUI) touch no `site/` files and run fully parallel to the site wave.

| # | bead | scope (one line) | P | deps | tier |
|---|---|---|---|---|---|
| 1 | sq-ymr2e.1 | E2E foundation: determinism harness, hermetic network, runner-state waiters, GitHub-API fixtures, stress script | P1 | — | sonnet |
| 2 | sq-ymr2e.2 | `/try` P1 journeys (edit→run→values, share-link round-trip, own-data, errors) | P1 | .1 | sonnet |
| 3 | sq-ymr2e.3 | Home hero-runner journeys + the zero-network-during-Run invariant + handoff | P1 | .1 | sonnet |
| 4 | sq-ymr2e.4 | a11y harness: axe WCAG 2.1 AA, zero serious/critical + moderate ratchet, P1 surfaces×states | P1 | .1 | sonnet |
| 5 | sq-ymr2e.5 | GUI frontend journeys under Playwright + mocked Tauri IPC (query, inference toggles, export contract, status bar) | P1 | — | sonnet |
| 6 | sq-ymr2e.6 | tauri-driver: shrink to a ≤3-assertion real-IPC smoke + pin/stabilize; 20-green probe or demote to nightly | P1 | — | sonnet |
| 7 | sq-ymr2e.7 | `/download` journeys vs mocked releases API + `/app` hard-nav regression guard | P2 | .1 | sonnet |
| 8 | sq-ymr2e.8 | All-surface render sweep (zero console errors + key artifact; papers/specs/benchmarks smokes) | P2 | .1 | haiku |
| 9 | sq-ymr2e.9 | Behavioral a11y: keyboard-only journey, ARIA tabs pattern, palette/drawer focus management | P2 | .1 | sonnet |
| 10 | sq-ymr2e.10 | Visual-regression rig: pinned container, masks, key-layout baselines, update workflow | P2 | .1 | sonnet |
| 11 | sq-ymr2e.11 | Nightly full-sweep lane (all-surface a11y+visual, cross-browser, tauri matrix, consolidated routing) | P2 | .4,.10 | sonnet |
| 12 | sq-ymr2e.12 | Advisory→required promotion + probation criteria + flake-quarantine policy | P2 | .2,.3,.4,.5 | sonnet |

No security-sensitive surface → no opus routing needed anywhere in this plan. After .1
lands, .2/.3/.4/.7/.8/.9/.10 are mutually file-disjoint; .5/.6 are dispatchable
immediately.

## 8. Assumptions (prose-only design pass — implementers verify)

- The runner/workbench expose (or can trivially expose) their UI states as observable
  attributes; if not, adding `data-state`/`data-testid` hooks is in-scope for sq-ymr2e.1–.3
  (test hooks, not behavior changes).
- `@tauri-apps/api/mocks` covers the GUI's invoke surface (Tauri 2 ships `mockIPC`/
  `mockWindows`); the GUI frontend can be built and served standalone for headless
  chromium. If the frontend hard-requires the Tauri runtime beyond IPC, sq-ymr2e.5 adds a
  thin runtime shim — escalate to the architect if that grows beyond a shim.
- The `gui/e2e` npm workspace can host a second (Playwright) project alongside the
  tauri-driver harness without breaking `gui.yml`; otherwise a sibling package is fine.
- Exact spec/workflow file names above are suggestions; the *disjointness* contract (one
  bead = its own files) is the requirement.
- The share-link (`#q=`) and handoff (`sessionStorage`) mechanisms are as specified in
  `research/site-redesign-home-try-app-download.md` §1.6/§2.4; if implementation diverged,
  test the shipped mechanism and note the divergence.

## 9. Success criteria (measurable, no vanity)

1. Every P1 journey in §2 has a green, deterministic spec (`--repeat-each=5 --retries=0`
   evidence in each PR); zero `waitForTimeout` under the e2e trees (grep-gated).
2. Zero serious/critical axe violations on the interactive surfaces×states, with the
   moderate ratchet seeded and only falling.
3. The `GUI tauri-driver e2e (Linux)` check is either ≥20-consecutive-green as a smoke or
   demoted to nightly — it stops being ambient red noise either way.
4. Site functional E2E + a11y + the GUI mockIPC lane are **required** in `ci-summary`
   after their probation bar, with the quarantine policy in force.
5. A deliberate visual change to a key layout fails the visual lane with a readable diff,
   and an intentional redesign updates baselines in its own PR.

Beads carry ids `sq-ymr2e.*`; this record is the plan of record for the wave.
