// (sq-w3dmj) [FABLE-5] — unit tests for the GLOBAL-FIRST Tauri fs plugin resolution in
// tauri-fs.ts.
//
// The load-bearing property under test: `loadTauriFs()` must resolve the plugin from the
// `window.__TAURI__.fs` runtime global BEFORE attempting the bare dynamic
// `import("@tauri-apps/plugin-fs")`. Under Node (as in the static-export webview) that bare
// import can NEVER resolve — the package is intentionally absent — so a non-null result here
// PROVES the global-first path was taken. The dropped global-first step is exactly the sq-w3dmj
// regression: with it missing, the desktop TauriWorkspaceStore was unreachable and persistence
// silently degraded to localStorage.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import { loadTauriFs } from "./tauri-fs.js";

/** A well-formed fake of the `@tauri-apps/plugin-fs` module surface. */
function fakeFsModule() {
  return {
    BaseDirectory: { AppLocalData: 11 },
    readTextFile: async () => "",
    writeTextFile: async () => {},
    remove: async () => {},
    exists: async () => false,
    mkdir: async () => {},
    readDir: async () => [],
  };
}

/** Install / clear a fake `window` global for one test (Node has none by default). */
function setWindow(value: unknown): void {
  (globalThis as { window?: unknown }).window = value;
}
function clearWindow(): void {
  delete (globalThis as { window?: unknown }).window;
}

test("loadTauriFs resolves window.__TAURI__.fs FIRST — selected over the always-failing bare import", async (t) => {
  t.after(clearWindow);
  const mod = fakeFsModule();
  setWindow({ __TAURI__: { fs: mod } });

  const api = await loadTauriFs();

  // Under Node the bare import("@tauri-apps/plugin-fs") always rejects (the package is
  // intentionally absent), so a resolved API proves the runtime global won the resolution.
  assert.ok(api, "expected the injected window.__TAURI__.fs global to resolve");
  assert.equal(api.baseDir, 11); // BaseDirectory.AppLocalData projected onto TauriFsApi.baseDir
  // Every member is the global module's own function, projected 1:1 (no wrapping, no loss).
  assert.equal(api.readTextFile, mod.readTextFile);
  assert.equal(api.writeTextFile, mod.writeTextFile);
  assert.equal(api.remove, mod.remove);
  assert.equal(api.exists, mod.exists);
  assert.equal(api.mkdir, mod.mkdir);
  assert.equal(api.readDir, mod.readDir);
});

test("loadTauriFs returns null with no window at all (SSR / Node)", async () => {
  clearWindow();
  assert.equal(await loadTauriFs(), null);
});

test("loadTauriFs returns null in a plain browser (window without __TAURI__)", async (t) => {
  t.after(clearWindow);
  setWindow({});
  assert.equal(await loadTauriFs(), null);
});

test("loadTauriFs ignores a malformed global (writeTextFile not a function) and falls through to null", async (t) => {
  t.after(clearWindow);
  setWindow({ __TAURI__: { fs: { writeTextFile: "not-a-function" } } });
  // The malformed global is rejected, the bare-import fallback rejects under Node → null,
  // so the store factory degrades to the web backend instead of crashing.
  assert.equal(await loadTauriFs(), null);
});
