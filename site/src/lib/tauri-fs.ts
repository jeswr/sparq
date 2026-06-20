// [OPUS-4.8] sq-atb0 (epic sq-ixc3) — the OPTIONAL Tauri filesystem loader for workspace
// persistence. The site is a static export that ALSO runs inside the Tauri 2 webview; this
// module is the bridge that lets the desktop app persist workspaces to local disk WITHOUT the
// browser/static build ever depending on a Tauri package.
//
// HOW IT STAYS STATIC-EXPORT-SAFE. `@tauri-apps/plugin-fs` is NOT a site dependency (not in
// package.json). It is imported with a `webpackIgnore` dynamic `import()` of the plugin's
// runtime ESM module path, evaluated ONLY when we are actually inside a Tauri webview
// (feature-detected by `isTauriRuntime()` in `@sparq/client`). On the GitHub Pages build the
// import line is never reached, the module is never bundled, and nothing Tauri-shaped ships.
//
// HONEST LIMITATION. The Tauri shell's current capability set (`gui/src-tauri/capabilities/
// default.json`) grants only `core:default` + `dialog:allow-open` — it does NOT yet grant the
// `fs` capability. Until the desktop shell adds the `fs:` permissions for the app-local-data
// scope, this loader returns `null` at runtime and the GUI cleanly degrades to the web
// (localStorage) backend even inside the Tauri webview. Wiring the `fs` capability + the
// plugin into the Rust shell is tracked as follow-up bead sq-atb0-fs (a `gui/src-tauri`
// change, out of the site lane).

"use client";

import { type TauriFsApi } from "@sparq/client";

/**
 * The shape of `@tauri-apps/plugin-fs` we read at runtime. Declared locally (not imported as a
 * type from the absent package) so the site typechecks without the Tauri package installed.
 * `BaseDirectory.AppLocalData` is the per-app local-data dir Tauri reserves for durable app
 * state.
 */
interface TauriFsModule {
  BaseDirectory: { AppLocalData: number };
  readTextFile: TauriFsApi["readTextFile"];
  writeTextFile: TauriFsApi["writeTextFile"];
  remove: TauriFsApi["remove"];
  exists: TauriFsApi["exists"];
  mkdir: TauriFsApi["mkdir"];
  readDir: TauriFsApi["readDir"];
}

/**
 * Dynamically import `@tauri-apps/plugin-fs` at runtime. Isolated in its own one-line helper so
 * the single `@ts-expect-error` for the intentionally-absent module specifier lands precisely
 * on the import expression (it is NOT a site dependency — the static build must not bundle a
 * Tauri package). `webpackIgnore` keeps it out of the bundle graph; it is fetched only inside
 * the Tauri webview, never on the GitHub Pages build.
 */
function importTauriFsPlugin(): Promise<unknown> {
  // @ts-expect-error — optional Tauri-only module, intentionally absent from the static build.
  return import(/* webpackIgnore: true */ "@tauri-apps/plugin-fs");
}

/**
 * Resolve the Tauri filesystem API scoped to the app's local-data dir, or `null` if the plugin
 * is not importable / the fs capability was not granted. Passed to
 * `createWorkspaceStore({ loadTauriFs })`, which only calls it inside a Tauri webview. A failed
 * import (no plugin) or a permission error resolves to `null` so the store factory falls back
 * to the web backend rather than throwing.
 */
export async function loadTauriFs(): Promise<TauriFsApi | null> {
  try {
    // webpackIgnore keeps the Tauri plugin out of the static bundle entirely: it is fetched as
    // a plain ESM module from the Tauri runtime only when this code path actually runs (inside
    // the desktop webview). On the GitHub Pages build this line is never reached.
    //
    // `@tauri-apps/plugin-fs` is deliberately NOT a site dependency (the static build must not
    // depend on a Tauri package), so `tsc` cannot resolve the specifier — the import is a pure
    // runtime concern guarded by `isTauriRuntime()`. The expect-error documents that the
    // missing-module diagnostic is intentional; the cast pins the runtime shape we use.
    const mod = (await importTauriFsPlugin()) as unknown as TauriFsModule;
    if (typeof mod?.writeTextFile !== "function") return null;
    return {
      baseDir: mod.BaseDirectory.AppLocalData,
      readTextFile: mod.readTextFile,
      writeTextFile: mod.writeTextFile,
      remove: mod.remove,
      exists: mod.exists,
      mkdir: mod.mkdir,
      readDir: mod.readDir,
    };
  } catch {
    // No plugin / capability not granted — the web (localStorage) backend takes over.
    return null;
  }
}
