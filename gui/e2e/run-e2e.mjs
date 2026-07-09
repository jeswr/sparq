// [SONNET-4.6] sq-ymr2e.6 — SHRUNK tauri-driver (WebDriver) smoke: max-3-assertion real-IPC test.
//
// Only tauri-driver can prove these three things:
//   [1/3] LAUNCH   — the real binary launched and the React frontend mounted in the native webview.
//   [2/3] IPC LIVE — the native engine came up and signalled "Engine ready" over real IPC.
//   [3/3] ROUND-TRIP — a SPARQL SELECT executed through real IPC, returned rows containing
//                      the expected binding ("Alice"), and the status bar reflects the row count.
//
// Everything beyond these three is covered by the deterministic Playwright mock-IPC lane
// (gui/e2e-playwright, sq-ymr2e.5): EXPLAIN, SHACL validation, Import drawer, keyboard spine.
// Keeping this smoke tiny is the whole point — small surface = small flake budget.
//
// DETERMINISM (§1 doctrine): zero arbitrary sleeps. All waits use webdriverio's event-driven
// polling APIs (waitForExist / waitUntil) with explicit conditions. The tauri-driver startup
// wait uses a callback-based retry (net.createConnection + setTimeout-in-callback) — not an
// awaited sleep — so it does not trigger the no-sleep gate (support/no-sleep-gate.sh).
//
// FAILURE ARTIFACTS: on any assertion failure, a screenshot is saved to artifacts/ and the
// accumulated tauri-driver logs are flushed to artifacts/tauri-driver.log before exit so the
// CI `upload-artifact` step can surface them.
//
// CONFIG via env:
//   APP_BINARY        — absolute path to the built shell binary (CI sets this; default: debug path)
//   TAURI_DRIVER_BIN  — path to the tauri-driver binary (default: ~/.cargo/bin/tauri-driver)
//   ARTIFACTS_DIR     — directory for failure artifacts (default: ./artifacts)

import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { remote } from "webdriverio";

// ESM __dirname shim (__dirname requires Node 20.11+; this works from Node 18).
const __dirname = path.dirname(fileURLToPath(import.meta.url));

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const TAURI_DRIVER_PORT = 4444;
const TAURI_DRIVER_HOST = "127.0.0.1";

const APP_BINARY =
  process.env.APP_BINARY ||
  path.resolve(
    __dirname,
    "..",
    "src-tauri",
    "target",
    "debug",
    "sparq-gui",
  );

const TAURI_DRIVER_BIN =
  process.env.TAURI_DRIVER_BIN ||
  path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver");

const ARTIFACTS_DIR =
  process.env.ARTIFACTS_DIR ||
  path.resolve(__dirname, "artifacts");

// The deterministic SELECT — every foaf:name in the seeded sample graph, ordered. "Alice" is
// the expected first binding and MUST appear in the result table for the assertion to pass.
const TEST_QUERY = [
  "PREFIX foaf: <http://xmlns.com/foaf/0.1/>",
  "SELECT ?name WHERE { ?s foaf:name ?name } ORDER BY ?name",
].join("\n");

const EXPECTED_BINDING = "Alice";

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

function log(...args) {
  console.log("[tauri-e2e]", ...args);
}

// Ensure the artifacts output directory exists.
function ensureArtifactsDir() {
  fs.mkdirSync(ARTIFACTS_DIR, { recursive: true });
}

// ---------------------------------------------------------------------------
// tauri-driver lifecycle
// ---------------------------------------------------------------------------

let tauriDriver = null;
const driverLogLines = [];

function startTauriDriver() {
  log(`spawning tauri-driver: ${TAURI_DRIVER_BIN}`);
  tauriDriver = spawn(TAURI_DRIVER_BIN, ["--port", String(TAURI_DRIVER_PORT)], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  tauriDriver.stdout?.on("data", (chunk) => {
    const line = String(chunk);
    process.stdout.write("[tauri-driver stdout] " + line);
    driverLogLines.push("stdout: " + line);
  });
  tauriDriver.stderr?.on("data", (chunk) => {
    const line = String(chunk);
    process.stderr.write("[tauri-driver stderr] " + line);
    driverLogLines.push("stderr: " + line);
  });
  tauriDriver.on("error", (err) => {
    log("tauri-driver process error:", err.message);
    driverLogLines.push("process error: " + err.message);
  });
}

// Wait for the tauri-driver port to accept connections using event-driven retry.
// Uses callback-style setTimeout (NOT an awaited sleep) so the no-sleep gate passes.
function waitForDriverPort(port, host, timeoutMs) {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeoutMs;

    function attempt() {
      if (Date.now() >= deadline) {
        reject(new Error(`tauri-driver port ${port} not available after ${timeoutMs} ms`));
        return;
      }
      const sock = net
        .createConnection(port, host)
        .on("connect", () => {
          sock.destroy();
          log(`tauri-driver is listening on ${host}:${port}`);
          resolve();
        })
        .on("error", () => {
          sock.destroy();
          // Retry after 200 ms via callback (not an awaited sleep — no-sleep gate safe).
          setTimeout(attempt, 200);
        });
    }

    attempt();
  });
}

function stopTauriDriver() {
  if (tauriDriver && !tauriDriver.killed) {
    log("stopping tauri-driver");
    tauriDriver.kill();
  }
}

// Flush collected driver logs to a file (called on failure before exit).
function saveDriverLogs() {
  ensureArtifactsDir();
  const dest = path.join(ARTIFACTS_DIR, "tauri-driver.log");
  fs.writeFileSync(dest, driverLogLines.join(""), "utf8");
  log(`driver logs saved to ${dest}`);
}

// ---------------------------------------------------------------------------
// Main smoke — exactly 3 assertions
// ---------------------------------------------------------------------------

async function main() {
  log(`app binary: ${APP_BINARY}`);

  startTauriDriver();
  await waitForDriverPort(TAURI_DRIVER_PORT, TAURI_DRIVER_HOST, 20_000);

  // Open a WebDriver session through tauri-driver. `tauri:options.application` tells
  // tauri-driver which native binary to launch. connectionRetryCount / connectionRetryTimeout
  // cover transient HTTP hiccups — webdriverio's own retry policy, not arbitrary sleeps.
  //
  // CAPABILITIES — DO NOT re-add `browserName: "wry"` (bead sq-t6492 / issue #1740).
  // WebKit's matchCapabilities (Source/WebDriver/WebDriverService.cpp) rejects a session with
  // "Failed to match capabilities" whenever a NON-NULL `browserName` is supplied that does not
  // equal the driver's own reported name. Newer WebKitGTK (the runner's webkit2gtk 2.5x) reports
  // `browserName: "MiniBrowser"`, so the historic tauri#8828 workaround `browserName: "wry"` now
  // BACKFIRES: "wry" != "MiniBrowser" → hard reject before any session log (the 0-byte-log
  // signature). tauri-driver 2.0.6 forwards the client browserName verbatim on Linux (it only
  // rewrites it to "webview2" on Windows), so the fix is client-side: OMIT browserName entirely.
  // The guard only fires when a mismatching name is SUPPLIED, so omitting it matches on BOTH old
  // and new WebKitGTK. tauri:options.application is what actually selects the wry webview binary.
  const browser = await remote({
    hostname: TAURI_DRIVER_HOST,
    port: TAURI_DRIVER_PORT,
    logLevel: "error",
    connectionRetryTimeout: 30_000,
    connectionRetryCount: 5,
    capabilities: {
      "tauri:options": { application: APP_BINARY },
    },
  });

  try {
    // ------------------------------------------------------------------
    // ASSERTION 1/3: LAUNCH
    // The SPARQL editor (#repl-query) being present proves the real binary launched and
    // the React-based GUI frontend mounted inside the native WebKit webview.
    // ------------------------------------------------------------------
    log("[1/3] waiting for the SPARQL editor to mount (#repl-query)…");
    const editor = await browser.$("#repl-query");
    await editor.waitForExist({ timeout: 30_000 });
    log("PASS [1/3] LAUNCH — editor mounted in the native webview");

    // ------------------------------------------------------------------
    // ASSERTION 2/3: IPC LIVE
    // The top bar shows "Engine ready" once the native engine has responded over real IPC.
    // (top-bar.tsx: `status.kind === "ready"` renders the literal "Engine ready" copy —
    // a comment there marks it as the deterministic hook the e2e waits on.)
    // ------------------------------------------------------------------
    log("[2/3] waiting for the native engine to report ready over IPC…");
    await browser.waitUntil(
      async () => {
        const body = await browser.$("body");
        const text = await body.getText();
        return /Engine ready/i.test(text);
      },
      {
        timeout: 60_000,
        timeoutMsg: "native engine never reported 'Engine ready' — IPC may not be up",
        interval: 500,
      },
    );
    log("PASS [2/3] IPC LIVE — native engine reported 'Engine ready'");

    // ------------------------------------------------------------------
    // Plumbing: enter the test query and click Run.
    // The textarea is React-controlled — set value via the native setter + dispatch a
    // bubbling `input` event so React's state updates (standard controlled-input technique).
    // ------------------------------------------------------------------
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

    const editorValue = await editor.getValue();
    if (!editorValue.includes("SELECT ?name")) {
      throw new Error(
        `editor did not accept the query (got: ${JSON.stringify(editorValue).slice(0, 120)}…)`,
      );
    }

    log("clicking 'Run query'…");
    const runButton = await browser.$("button=Run query");
    await runButton.waitForClickable({ timeout: 10_000 });
    await runButton.click();

    // ------------------------------------------------------------------
    // ASSERTION 3/3: ROUND-TRIP
    // A SELECT result table renders containing the expected binding ("Alice"), proving the
    // query executed through real IPC against the native engine and a result was returned.
    // The status bar's row-count field also transitions from "— rows" to "N rows", confirming
    // the engine plumbed the result count back through IPC (status-bar.tsx: lastRowCount).
    // ------------------------------------------------------------------
    log("[3/3] waiting for the SELECT result to render…");

    const selectResult = await browser.$('[data-result-kind="select"]');
    await selectResult.waitForExist({ timeout: 30_000 });

    const tableView = await browser.$('[data-result-view="table"]');
    await tableView.waitForExist({ timeout: 10_000 });

    const tableText = await tableView.getText();
    if (!tableText.includes(EXPECTED_BINDING)) {
      throw new Error(
        `result table rendered but did not contain "${EXPECTED_BINDING}" ` +
          `(text: ${JSON.stringify(tableText).slice(0, 200)}…)`,
      );
    }

    // Status bar: "N rows" (not "— rows") proves the engine plumbed the count back over IPC.
    await browser.waitUntil(
      async () => {
        const body = await browser.$("body");
        const text = await body.getText();
        return /\d[\d,]* rows/.test(text);
      },
      {
        timeout: 10_000,
        timeoutMsg: "status bar never showed a numeric row count after the SELECT",
        interval: 200,
      },
    );

    log(
      `PASS [3/3] ROUND-TRIP — SELECT returned "${EXPECTED_BINDING}" and status bar shows row count`,
    );

    await browser.deleteSession();
  } catch (err) {
    // Save failure artifacts before tearing down — the CI `upload-artifact` step collects them.
    log("saving failure artifacts to", ARTIFACTS_DIR);
    ensureArtifactsDir();
    try {
      await browser.saveScreenshot(path.join(ARTIFACTS_DIR, "screenshot.png"));
      log("screenshot saved");
    } catch (screenshotErr) {
      log("could not save screenshot:", screenshotErr.message);
    }
    saveDriverLogs();
    try {
      await browser.deleteSession();
    } catch {
      /* ignore teardown errors — surface the real failure below */
    }
    throw err;
  }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

main()
  .then(() => {
    stopTauriDriver();
    log("smoke succeeded — all 3 assertions passed");
    process.exit(0);
  })
  .catch((err) => {
    stopTauriDriver();
    saveDriverLogs();
    console.error("[tauri-e2e] FAILED:", err?.stack ?? err);
    process.exit(1);
  });
