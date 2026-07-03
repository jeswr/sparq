// [SONNET-4.6] sq-ymr2e.5 — the Tauri IPC mock injected by page.addInitScript.
//
// Stubs window.__TAURI__ + window.__TAURI_INTERNALS__ so isTauriRuntime() returns true
// (gui/app/src/lib/tauri-ipc.ts checks for either global) and every IPC invoke() call
// returns a deterministic fixture instead of reaching the native Rust backend.
//
// The LOG array is exposed as window.__TAURI_IPC_LOG__ so tests can verify the IPC contract
// (assert which commands were fired and with which args) via page.evaluate.
//
// Commands mocked:
//   disk_usage       → { path: "/fake/workspaces", bytes: 12345678, exists: true }
//   load_text        → { nquads: "<...> .", count: 1, format: "turtle" }
//   load_path        → same as load_text
//   plugin:dialog|open → "/tmp/test.ttl"
//   (any other cmd)  → throws Error (catches unexpected IPC surface drift)

/** The script string injected via page.addInitScript before the page's own scripts run. */
export const tauriMockScript: string = `
(function () {
  "use strict";
  // Satisfy isTauriRuntime() — checks __TAURI_INTERNALS__ || __TAURI__ in window.
  window.__TAURI_INTERNALS__ = { metadata: { currentWebviewLabel: "main" } };

  var DISK_FIXTURE = { path: "/fake/workspaces", bytes: 12345678, exists: true };
  var LOAD_FIXTURE = {
    nquads: "<http://example.org/imported> <http://example.org/p> <http://example.org/o> .",
    count: 1,
    format: "turtle"
  };
  // The LOG accumulates every invoke call so tests can assert the IPC contract.
  var LOG = [];

  window.__TAURI__ = {
    core: {
      invoke: function (cmd, args) {
        LOG.push({ cmd: cmd, args: args });
        if (cmd === "disk_usage")        return Promise.resolve(DISK_FIXTURE);
        if (cmd === "load_text")         return Promise.resolve(LOAD_FIXTURE);
        if (cmd === "load_path")         return Promise.resolve(LOAD_FIXTURE);
        if (cmd === "plugin:dialog|open") return Promise.resolve("/tmp/test.ttl");
        return Promise.reject(new Error("[tauri-mock] Unexpected IPC invoke: " + cmd));
      }
    }
  };

  // Expose the log for test assertions: page.evaluate(() => window.__TAURI_IPC_LOG__)
  window.__TAURI_IPC_LOG__ = LOG;
})();
`;
