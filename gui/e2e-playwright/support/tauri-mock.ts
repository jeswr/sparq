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
//   query_service    → [FABLE-5] sq-ixc3.14: fail-closed allowlist mirror of the Rust command —
//                      rejects with the "SERVICE egress refused" marker unless the caller's
//                      allow[] contains fed.example.org; else resolves a joined SELECT doc
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

  // [FABLE-5] sq-ixc3.14 — the native federated-query command (query_service). The mock
  // mirrors the Rust contract (gui/src-tauri/src/federation.rs): STRICT fail-closed
  // allowlist — the fixture endpoint host must be ON the caller's allowlist or the call
  // rejects with the engine's stable egress-refusal marker (a plain string, exactly like a
  // real Tauri command Err). When allowed, it resolves a SPARQL-Results-JSON doc whose row
  // JOINS a local sample-graph binding (Alice) with a remote-only binding (remote-captain).
  var FED_HOST = "fed.example.org";
  var SERVICE_FIXTURE = JSON.stringify({
    head: { vars: ["name", "fedRole"] },
    results: { bindings: [
      { name:    { type: "literal", value: "Alice" },
        fedRole: { type: "literal", value: "remote-captain" } }
    ] }
  });

  window.__TAURI__ = {
    core: {
      invoke: function (cmd, args) {
        LOG.push({ cmd: cmd, args: args });
        if (cmd === "disk_usage")        return Promise.resolve(DISK_FIXTURE);
        if (cmd === "load_text")         return Promise.resolve(LOAD_FIXTURE);
        if (cmd === "load_path")         return Promise.resolve(LOAD_FIXTURE);
        if (cmd === "plugin:dialog|open") return Promise.resolve("/tmp/test.ttl");
        if (cmd === "query_service") {
          var allow = (args && args.allow) || [];
          var permitted = false;
          for (var i = 0; i < allow.length; i++) {
            if (allow[i] === FED_HOST) permitted = true;
          }
          if (!permitted) {
            // The stable marker substring sparq_engine::SERVICE_EGRESS_REFUSED_MARKER carries.
            return Promise.reject(
              'SERVICE egress refused: endpoint host "' + FED_HOST + '" is not allowlisted'
            );
          }
          return Promise.resolve(SERVICE_FIXTURE);
        }
        return Promise.reject(new Error("[tauri-mock] Unexpected IPC invoke: " + cmd));
      }
    }
  };

  // Expose the log for test assertions: page.evaluate(() => window.__TAURI_IPC_LOG__)
  window.__TAURI_IPC_LOG__ = LOG;
})();
`;
