<!-- [OPUS-4.8] sq-pnnl -->

# sparq GUI — tauri-driver (WebDriver) e2e

A **real** end-to-end test for the Tauri desktop shell: it launches the built shell binary
through [`tauri-driver`](https://v2.tauri.app/develop/tests/webdriver/) (WebDriver), drives the
reused Next.js frontend in the native webview, **runs a SPARQL query in the in-tab WASM REPL**,
and asserts the result table renders. It replaces the earlier no-op harness smoke (`sq-bu69`),
which only confirmed that `tauri-driver` / `WebKitWebDriver` / `xvfb` were on `PATH`.

## What it does (`run-e2e.mjs`)

1. Spawns `tauri-driver` on `127.0.0.1:4444` (on Linux it proxies `WebKitWebDriver`).
2. Connects a [`webdriverio`](https://webdriver.io) `remote()` session and launches the built
   shell binary via the `tauri:options.application` capability.
3. Waits for the SPARQL editor (`#repl-query`) to mount and the WASM engine to report
   **Engine ready**.
4. **Enters** a deterministic `SELECT ?name` over the built-in sample graph (set through the
   native `<textarea>` value setter + a bubbling `input` event, so the React-controlled editor
   updates).
5. Clicks **Run query**.
6. **Asserts** a `data-result-kind="select"` table renders containing the binding `"Alice"`
   and at least one data row.

The Tauri window loads the site's static-export **index** (`site/out/index.html`), which embeds
`<Repl />`, so the REPL is present on load with no in-webview route navigation.

## Run it locally (Linux)

Requires the webview system libraries (`libwebkit2gtk-4.1-dev`, …), `webkit2gtk-driver`
(provides `WebKitWebDriver`), and `xvfb`. Then:

```bash
# 1. Build the frontend the webview embeds (root-relative, as the GUI consumes it).
cd js && npm install && npm run build:wasm:lean
cd ../site && npm install && npm run build:tauri   # = cross-env NEXT_PUBLIC_BASE_PATH= npm run build (root-relative)

# 2. Build the shell binary (embeds site/out at compile time).
cd ../gui/src-tauri && cargo build

# 3. Install the e2e deps + the driver, then run under a virtual display.
cd ../e2e && npm ci
cargo install tauri-driver --locked
xvfb-run --auto-servernum npm run test:e2e
```

`APP_BINARY` (absolute path to the built shell binary) and `TAURI_DRIVER_BIN` can override the
defaults; CI sets `APP_BINARY` explicitly.

## CI + honesty

Wired as the `tauri-e2e` job in [`.github/workflows/gui.yml`](../../.github/workflows/gui.yml).
The job keeps the **`(Linux, advisory)`** name token and `continue-on-error: true`, so it is
**excluded from the `ci-summary / gate`** aggregator and cannot break the merge gate for all
PRs while the headless `tauri-driver` + `WebKitWebDriver` + `xvfb` stack is being proven
reliably green on `ubuntu-latest`. The frontend-build + shell-build prerequisites are reproduced
locally; the headless WebDriver session itself is **documented-untested locally** (it needs
webkit2gtk system libraries) — it validates on the first CI run. Promotion to a hard gate (drop
the `advisory` token + `continue-on-error`, add to branch-protection required contexts) is the
separate maintainer step tracked by `sq-var9`.
