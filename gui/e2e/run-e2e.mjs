// [OPUS-4.8] sq-pnnl — REAL tauri-driver (WebDriver) e2e for the sparq Tauri GUI shell.
//
// Replaces the no-op "harness scaffold OK" smoke (sq-bu69) with a real assertion:
//   1. spawn tauri-driver (the WebDriver intermediary; on Linux it proxies WebKitWebDriver),
//   2. connect a WebDriver client (webdriverio) and launch the BUILT Tauri shell binary,
//   3. wait for the DISTINCT operational GUI frontend (the in-tab WASM workbench) to mount +
//      the engine to warm,
//   4. ENTER a known SPARQL SELECT into the editor (the React-controlled `#repl-query`
//      textarea — set via the native value setter + an `input` event so React's state
//      updates, the standard controlled-input driving technique),
//   5. click "Run query", and
//   6. ASSERT the SELECT result table renders with the expected binding ("Alice", from the
//      seeded sample graph).
//
// This proves the desktop shell actually loads, the WASM engine runs a query IN THE TAB, and
// the result renders — i.e. the shell is a working app, not just a crate that compiles.
//
// HOW IT FINDS THINGS (kept faithful to the DISTINCT operational GUI frontend, gui/app, so it
// cannot silently drift — sq-ixc3.8 repointed the shell off the marketing site onto gui/app):
//   * The Tauri window loads the GUI frontend's static-export INDEX (gui/app/out/index.html),
//     which renders the workbench shell with the Query tool open by default — so no in-webview
//     route navigation is needed.
//   * The editor is a controlled <textarea id="repl-query">
//     (gui/app/src/components/workbench/query-workbench.tsx).
//   * The Run button carries the visible text "Run query" (same file).
//   * The top bar shows "Engine ready" once the in-tab WASM engine warms
//     (gui/app/src/components/workbench/top-bar.tsx).
//   * A SELECT result renders a container with data-result-kind="select" and, in table view,
//     data-result-view="table" wrapping a <table> (query-workbench.tsx). We assert on those
//     stable data-* hooks + the literal "Alice" (from the seeded sample graph).
//
// CONFIG via env (CI sets APP_BINARY): APP_BINARY is the absolute path to the built shell
// binary tauri-driver launches (the gui crate's package name is `sparq-gui`, so the debug
// binary is gui/src-tauri/target/debug/sparq-gui). tauri-driver listens on 127.0.0.1:4444.

import { spawn } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { remote } from "webdriverio";

const TAURI_DRIVER_PORT = 4444;
const TAURI_DRIVER_HOST = "127.0.0.1";

// The built shell binary tauri-driver launches. CI passes an absolute APP_BINARY; the default
// is the debug binary cargo produces for the `sparq-gui` package.
const APP_BINARY =
  process.env.APP_BINARY ||
  path.resolve(
    process.cwd(),
    "..",
    "src-tauri",
    "target",
    "debug",
    "sparq-gui",
  );

// tauri-driver is installed by `cargo install`, so it lives in ~/.cargo/bin on the runner.
const TAURI_DRIVER_BIN =
  process.env.TAURI_DRIVER_BIN ||
  path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver");

// The query we ENTER + run. A deterministic SELECT over the built-in sample graph — every
// person's foaf:name, ordered — so "Alice" is guaranteed in the result table.
const TEST_QUERY = [
  "PREFIX foaf: <http://xmlns.com/foaf/0.1/>",
  "SELECT ?name WHERE { ?s foaf:name ?name } ORDER BY ?name",
].join("\n");

const EXPECTED_CELL = "Alice";

function log(...args) {
  // Prefix so the CI log clearly attributes the harness output.
  console.log("[tauri-e2e]", ...args);
}

let tauriDriver;

// Spawn tauri-driver and wait until its port accepts a TCP connection (it boots fast, but
// racing the WebDriver session against an unbound port is the classic flake source).
async function startTauriDriver() {
  log(`spawning tauri-driver: ${TAURI_DRIVER_BIN}`);
  tauriDriver = spawn(TAURI_DRIVER_BIN, ["--port", String(TAURI_DRIVER_PORT)], {
    stdio: ["ignore", "inherit", "inherit"],
  });
  tauriDriver.on("error", (err) => {
    log("tauri-driver process error:", err.message);
  });

  const net = await import("node:net");
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    const up = await new Promise((resolve) => {
      const sock = net
        .createConnection(TAURI_DRIVER_PORT, TAURI_DRIVER_HOST)
        .on("connect", () => {
          sock.destroy();
          resolve(true);
        })
        .on("error", () => resolve(false));
    });
    if (up) {
      log(`tauri-driver is listening on ${TAURI_DRIVER_HOST}:${TAURI_DRIVER_PORT}`);
      return;
    }
    await sleep(250);
  }
  throw new Error("tauri-driver did not start listening within 20s");
}

function stopTauriDriver() {
  if (tauriDriver && !tauriDriver.killed) {
    log("stopping tauri-driver");
    tauriDriver.kill();
  }
}

async function main() {
  log(`app binary: ${APP_BINARY}`);
  await startTauriDriver();

  // webdriverio `remote()` session against tauri-driver. The `tauri:options.application`
  // capability is what tells tauri-driver which native binary to launch.
  const browser = await remote({
    hostname: TAURI_DRIVER_HOST,
    port: TAURI_DRIVER_PORT,
    // Keep webdriverio's own logging quiet; the harness logs the meaningful steps.
    logLevel: "error",
    capabilities: {
      "tauri:options": { application: APP_BINARY },
      // wry/WebKitWebGTK identifies as a generic webkit webview; tauri-driver fills the rest.
      browserName: "wry",
    },
  });

  try {
    // 1. The editing textarea proves the frontend (and React) mounted.
    log("waiting for the SPARQL editor to mount…");
    const editor = await browser.$("#repl-query");
    await editor.waitForExist({ timeout: 30_000 });

    // 2. Wait for the engine to be ready. The REPL shows an "Engine ready" badge once the
    //    WASM bundle + default dataset are warm. `ensureStore` is a safety net (a click
    //    before ready still works), but waiting removes the only real timing race.
    log("waiting for the WASM engine to warm…");
    await browser.waitUntil(
      async () => {
        const body = await browser.$("body");
        const text = await body.getText();
        return /Engine ready/i.test(text);
      },
      {
        timeout: 60_000,
        timeoutMsg: "engine never reported ready",
        interval: 500,
      },
    );

    // 3. ENTER the query. The textarea is React-controlled, so a plain setValue/sendKeys
    //    would not update React state (React owns `value`). Set the value through the native
    //    HTMLTextAreaElement value setter and dispatch a bubbling `input` event — the
    //    canonical way to drive a controlled React input from outside React.
    log("entering the test query into the editor…");
    await browser.execute((value) => {
      const el = document.querySelector("#repl-query");
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLTextAreaElement.prototype,
        "value",
      ).set;
      setter.call(el, value);
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }, TEST_QUERY);

    // Sanity-check the editor actually holds our query before running.
    const editorValue = await editor.getValue();
    if (!editorValue.includes("SELECT ?name")) {
      throw new Error(
        `editor did not accept the query (got: ${JSON.stringify(editorValue).slice(0, 120)}…)`,
      );
    }

    // 4. Click "Run query". Locate the button by its visible label (stable copy).
    log('clicking "Run query"…');
    const runButton = await browser.$("button=Run query");
    await runButton.waitForClickable({ timeout: 10_000 });
    await runButton.click();

    // 5. ASSERT the SELECT result table renders. Wait for the select-result container, then
    //    the table view, then assert the expected binding cell is present.
    log("waiting for the SELECT result to render…");
    const selectResult = await browser.$('[data-result-kind="select"]');
    await selectResult.waitForExist({ timeout: 30_000 });

    const tableView = await browser.$('[data-result-view="table"]');
    await tableView.waitForExist({ timeout: 10_000 });

    const tableText = await tableView.getText();
    if (!tableText.includes(EXPECTED_CELL)) {
      throw new Error(
        `result table rendered but did not contain "${EXPECTED_CELL}" (text: ${JSON.stringify(
          tableText,
        ).slice(0, 200)}…)`,
      );
    }

    // Assert at least one data row exists (header is a <th>; rows are <td>). In webdriverio
    // v9 `$$` returns a ChainablePromiseArray; awaiting it yields an ElementArray with a
    // numeric `.length`.
    const rows = await browser.$$('[data-result-view="table"] tbody tr');
    const rowCount = rows.length;
    if (rowCount < 1) {
      throw new Error("result table has no data rows");
    }

    log(
      `PASS — SELECT ran in the in-tab WASM engine and rendered ${rowCount} row(s) including "${EXPECTED_CELL}".`,
    );

    await browser.deleteSession();
  } catch (err) {
    // Best-effort session teardown; surface the real failure.
    try {
      await browser.deleteSession();
    } catch {
      /* ignore */
    }
    throw err;
  }
}

main()
  .then(() => {
    stopTauriDriver();
    log("e2e succeeded");
    process.exit(0);
  })
  .catch((err) => {
    stopTauriDriver();
    console.error("[tauri-e2e] FAILED:", err && err.stack ? err.stack : err);
    process.exit(1);
  });
