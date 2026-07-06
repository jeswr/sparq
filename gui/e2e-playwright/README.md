<!-- [OPUS-4.8] sq-ymr2e.12 -->

# sparq GUI — Playwright mocked-IPC lane (`gui/e2e-playwright/`)

The deterministic desktop lane (bead `sq-ymr2e.5`). Drives the operational GUI frontend
(`gui/app/out`, the Tauri-target static export) in **headless Chromium** with
`window.__TAURI__` + `window.__TAURI_INTERNALS__` stubbed via `page.addInitScript`
(`support/fixtures.ts`). No Tauri binary, no display server, no real IPC — fully hermetic.
The in-tab WASM engine (queries / inference / SHACL / full-text / RSP) runs for real.

Two personas over the same served export (`playwright.config.ts`): `chromium-mock-ipc`
(desktop, Tauri globals injected) and `chromium-web` (browser, no Tauri globals — the
deployed `/app` code paths). Split by filename: `*.web.spec.ts` → web persona.

For the real-IPC (WebKitWebDriver + `xvfb`) native smoke, see `gui/e2e/` — that lane is
environment-coupled and **advisory forever** (it never gates).

## Run it

```bash
# From the repo root — build the engine + the frontend the export serves, then test.
cd js && npm install && npm run build:wasm && npm run build:reason-wasm
cd ../gui/app && npm install && npm run build:tauri     # → gui/app/out
cd ../e2e-playwright && npx playwright install --with-deps chromium && npm test
```

Determinism gate (no `waitForTimeout`): `bash support/no-timeout-gate.sh` (hard step in CI).

## CI, gating & flake-quarantine

Wired as the `gui-mock-ipc` job in [`.github/workflows/gui.yml`](../../.github/workflows/gui.yml).

Promotion of the deterministic site + GUI lanes to a **required** check is governed by the
checked-in policy **[`.github/E2E-GATING-POLICY.md`](../../.github/E2E-GATING-POLICY.md)**
(design of record: `research/web-gui-test-program.md` §6.3):

- **Probation bar:** a lane earns gating only after **50 consecutive green runs on `main`
  spanning ≥ 10 distinct PRs, OR two weeks — whichever is LONGER**, zero quarantine events in the
  window; promotion is a one-line flip (drop the `advisory` token from the job name).
- **Current status of this lane:** `gui-mock-ipc` **gates today** — it was promoted at creation on
  the deterministic-lane rationale (`retries: 0`, mocked IPC), which pre-dates this governance and
  is flagged for maintainer ratification (policy §6). It is not being unilaterally demoted here.
- **Flake-quarantine:** this lane runs `retries: 0` with `trace: retain-on-failure` — **stricter**
  than the site lanes' `retries: 1`: a flake is an immediate hard failure. A test that flakes
  (fails then passes on a re-push) is quarantined the same day (`test.fixme`) with a P2 fix bead;
  quarantined tests cannot gate. Do not raise `retries` to hide a flake — that inverts the policy.
