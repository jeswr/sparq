<!-- [OPUS-4.8] sq-ymr2e.12 — SPARQ agent. Operational gating/probation policy for the
     deterministic site + GUI Playwright lanes. Design of record:
     research/web-gui-test-program.md §6.3 (advisory-first, promotion earned). -->

# Web + GUI E2E — gating, probation & flake-quarantine policy

> 🤖 SPARQ agent policy record. This is the **operational runbook** for promoting the
> deterministic site (`site/e2e/`) and GUI (`gui/e2e-playwright/`) Playwright lanes from
> **advisory** to **required**. The *design* rationale lives in
> [`research/web-gui-test-program.md` §6.3](../research/web-gui-test-program.md); this file
> is the checked-in policy the workflows link to, and the **probation-evidence ledger**.

## 1. The one rule everything follows

`ci-summary / gate` (`.github/workflows/ci-summary.yml`) is the **single** required
branch-protection context. It aggregates every other check-run on the head commit and
**excludes any check whose name matches the whole word `advisory` or `informational`**
(`\b(advisory|informational)\b`). So a lane's **gating status is decided by its job NAME**:

- name contains `advisory` → the aggregator ignores it → **non-gating** (in probation).
- name has no such token → the aggregator waits on it → **gating** (promoted).

Branch protection is therefore **never edited directly** to promote a lane. Promotion is a
one-line change to a workflow file (§4). Adding a raw required context is forbidden.

## 2. Scope — the promotion-set lanes

| Lane (what it proves) | Workflow · job | Today |
|---|---|---|
| Site functional E2E + a11y (foundation smoke, axe WCAG 2.1 AA) | `site-e2e-foundation.yml` · `determinism gate + foundation smoke (advisory)` | **advisory** |
| Home hero-runner journeys (5 P1 journeys + stress) | `site-e2e-hero.yml` · `home hero-runner journeys (advisory)` | **advisory** |
| GUI mocked-IPC desktop journeys (headless Chromium, `retries=0`) | `gui.yml` · `gui-mock-ipc` | **gating** — see §6 |
| Visual key-layouts (container-pinned snapshots) | `site-visual.yml` · `visual key layouts (container-pinned, advisory)` | **advisory** (promotes **last**, §5) |

**Never promotable** (documented so nobody wires them by accident):

- `gui.yml` · `tauri-e2e` (`GUI tauri-driver e2e (Linux, advisory)`) — WebKitWebDriver +
  `xvfb` + a real native webview. Environment-coupled by nature → **advisory forever**
  (design §5.3). Its stabilisation/flake-probe is tracked by `sq-ymr2e.6`, not here.
- `gui.yml` · `tauri-e2e-probe` — a `workflow_dispatch` flake probe; never a PR check.
- The nightly full-sweep lane (`sq-ymr2e.11`: cross-browser, full-surface axe/visual,
  the per-platform tauri matrix) — **nightlies never gate** `ci-summary`.

> The per-platform Tauri **build + clippy** matrix (`gui.yml` · `tauri-build`) is a separate
> question governed by its own bead (`sq-var9`): it is a compile lane, not an e2e lane, and
> may earn gating once the macOS/Windows rows are proven — out of scope for *this* policy.

## 3. The probation bar (identical for every promotable lane)

A lane may be promoted only once it has, **on `main`**, accumulated:

> **50 consecutive green runs spanning ≥ 10 distinct PRs, OR two weeks — whichever is
> LONGER**, with **zero** quarantine events (§7) inside that window.

"Whichever is LONGER" is deliberate: 50 greens in three days is not enough soak, and two
green weeks with only two PRs is not enough surface coverage. Both floors must clear. The
evidence (a link to the run history / the ledger row) goes in the **promotion PR body**.

## 4. Promotion = a one-line flip (the runbook)

To promote a lane, edit **its job in the workflow file** and drop the `advisory` marker so
the aggregator starts gating it. Concretely, per lane:

1. Remove ` (advisory)` from the job `name:` (that is the load-bearing change — it takes the
   job out of the aggregator's exclusion set).
2. Remove the `continue-on-error: true` on the browser/exec step(s) so a real failure turns
   the job red (the determinism grep-gates are already hard; leave them).
3. Record the evidence in the §8 ledger and cite it in the PR body.

Nothing else changes — no branch-protection edit, no new required context. The rename alone
moves the lane from "reported but ignored" to "waited on by `ci-summary / gate`". To **demote**
(if a promoted lane starts flaking), do the reverse: it is equally a one-line flip, so an
unstable gate is never left blocking the train while it is fixed.

## 5. Promotion sequence

- **Group A — functional E2E + a11y + the GUI mocked-IPC lane** promote **first**, together,
  each on its own §3 evidence. These assert *behavioural contracts* (states, values, hrefs,
  ARIA, invoke shapes) via roles/testids that survive restyling — the non-brittle half.
- **Visual key-layouts promote SEPARATELY and LAST.** Pixel snapshots are the most
  brittleness-prone lane (font rasterising, antialiasing, masked dynamic regions), so the
  visual subset must earn its own §3 window *after* the functional lanes are stable, never
  bundled with them.

## 6. Current anomaly — `gui-mock-ipc` gates today (maintainer decision pending)

The `gui-mock-ipc` job carries **no `advisory` token and no `continue-on-error`**, so it
**already gates** `ci-summary`. It was promoted at creation (`sq-ymr2e.5`, PR #1431) on the
rationale that it is a fully deterministic headless-Chromium lane (`retries=0`, mocked IPC).

That **pre-dates this governance** and does not sit on recorded §3 probation evidence, which
the architect's plan of record (design §6.3 "everything lands advisory"; the `sq-ymr2e.12`
note classifying `gui-mock-ipc` as *in the promotion set*) says it should. This policy does
**not** unilaterally demote a green, deterministic gate — that would reduce real enforcement
on an actively-developed surface. Instead the discrepancy is **flagged for the maintainer**
(tracking issue **#1656**) to decide:

- **Ratify** — bless the early promotion, keep it gating, and backfill its ledger row from
  the run history to date; or
- **Reset** — apply the §4 flip in reverse (add ` (advisory)` + `continue-on-error: true` on
  the `Run Playwright mocked-IPC tests` step) so it re-earns gating uniformly under §3.

Until then it is recorded truthfully in §8 as *gating (early promotion, unratified)*.

## 7. Flake-quarantine policy (codified beside the suites)

A gate that is tolerated when flaky trains contributors to "re-run until green" and erodes
every other gate's authority. So:

- **A test that passes-on-retry twice within a 7-day window is QUARANTINED the same day**
  (`test.fixme(...)` / `test.skip(...)` with a comment linking the bead) and a **P2 fix bead
  is filed same-day**. A quarantined test cannot gate (it does not run in the gating set) and
  must be fixed or deleted, never left skipped indefinitely.
- **CI retry regime is diagnostic, not a safety net:**
  - Site lanes (`site/playwright.config.ts`): `retries: 1` in CI with `trace: on-first-retry`.
    A pass-on-retry is a **defect to fix**, not a success — the trace is the evidence.
  - GUI mocked-IPC lane (`gui/e2e-playwright/playwright.config.ts`): `retries: 0` with
    `trace: retain-on-failure` — **stricter**: a flake is an immediate hard failure, so the
    quarantine trigger there is "fails then passes on a re-push", handled the same way.
  Do not raise `retries` to hide a flake; that inverts the policy.
- **Anti-flake acceptance bar (pre-merge, before a spec is even advisory):** every new spec
  passes `--repeat-each=5 --retries=0` locally (`npm run test:e2e:stress` for the site suite).

## 8. Probation-evidence ledger

Evidence accumulates on `main`. Update the row on each green run once collection begins; a
lane is promotable only when its row clears **both** §3 floors with zero §7 quarantine events.

| Lane | Window opened | Consecutive green (on main) | Distinct PRs | Quarantine events | Promotable? | Evidence |
|---|---|---|---|---|---|---|
| site functional E2E + a11y (`site-e2e-foundation`) | not yet opened | 0 — accumulating | 0 | 0 | **No** | — |
| home hero-runner (`site-e2e-hero`) | not yet opened | 0 — accumulating | 0 | 0 | **No** | — |
| GUI mocked-IPC (`gui-mock-ipc`) | n/a (gating early, §6) | not tracked pre-governance | — | 0 known | **Gating — unratified** | pending maintainer (§6, issue #1656) |
| visual key-layouts (`site-visual`) | not yet opened | 0 — accumulating | 0 | 0 | **No** (promotes last, §5) | — |

> The window is "opened" the first green run after the lane's spec set is considered stable;
> record the date + first run URL then. Counts reset to 0 on any red run or quarantine event
> inside the window (the "consecutive" and "zero quarantine" requirements are strict).

## 9. See also

- Design of record: [`research/web-gui-test-program.md`](../research/web-gui-test-program.md) §6.3.
- Determinism doctrine + the shared harness: [`site/e2e/support/README.md`](../site/e2e/support/README.md).
- The GUI mocked-IPC suite: [`gui/e2e-playwright/README.md`](../gui/e2e-playwright/README.md).
- The aggregator semantics: [`.github/workflows/ci-summary.yml`](workflows/ci-summary.yml)
  (header) + `scripts/ci_summary_gate.py`.
