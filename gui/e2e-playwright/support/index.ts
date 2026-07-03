// [SONNET-4.6] sq-ymr2e.5 — barrel: the one import every spec needs.
//
// Import from here, not from @playwright/test directly:
//   import { test, expect } from '../support/index.ts';
//
// Exports:
//   test           — extended test with the tauriMock auto-fixture
//   expect         — standard Playwright expect (re-exported for convenience)
//   readIpcLog     — helper to read the accumulated IPC log from the page
//   waitForEngineReady — wait for "Engine ready" in the top bar
//   tauriMockScript    — the raw init-script string (rarely needed directly)
//   IpcLogEntry    — type of entries in the IPC log

export { test, expect, readIpcLog, type IpcLogEntry } from "./fixtures.ts";
export { waitForEngineReady } from "./gui-ready.ts";
export { tauriMockScript } from "./tauri-mock.ts";
