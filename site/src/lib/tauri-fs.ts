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
// HOW IT RESOLVES AT RUNTIME (sq-ixc3.6). The desktop shell registers `tauri_plugin_fs` and
// sets `app.withGlobalTauri = true` (gui/src-tauri), so the plugin is injected onto
// `window.__TAURI__.fs`. We resolve the fs API from THAT runtime global FIRST — the robust
// path for a pre-bundled static export that never imports a Tauri package — and only fall back
// to a (webpack-ignored, unbundled) bare `import()` for a hypothetical bundler-based host. On
// the GitHub Pages build neither the global nor the import resolves, so the loader returns
// `null` and the GUI cleanly degrades to the web (localStorage) backend — exactly as on a
// plain browser.
//
// CAPABILITY SCOPE (sq-ixc3.6). The fs grant is LEAST-PRIVILEGE: the shell's capability
// (gui/src-tauri/capabilities/default.json) scopes every fs permission to
// `$APPLOCALDATA/workspaces` ONLY — read/write text file, remove, mkdir, exists, read dir, and
// NO arbitrary FS. A call outside that scope is denied by Tauri and rejects, which this loader
// treats the same as an absent plugin (returns `null`).

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
 * Read the fs plugin off the Tauri runtime global, when present. The desktop shell sets
 * `app.withGlobalTauri = true` and registers `tauri_plugin_fs`, so Tauri injects the plugin at
 * `window.__TAURI__.fs` — the resolution path that works for a PRE-BUNDLED static export that
 * never imports a Tauri npm package. Returns `undefined` in a plain browser (no `__TAURI__`),
 * where the caller falls through to the web/localStorage backend. No import, so nothing
 * Tauri-shaped is ever bundled.
 */
function readGlobalTauriFs(): TauriFsModule | undefined {
  if (typeof window === "undefined") return undefined;
  const fs = (window as unknown as { __TAURI__?: { fs?: unknown } }).__TAURI__?.fs;
  if (fs && typeof (fs as { writeTextFile?: unknown }).writeTextFile === "function") {
    return fs as TauriFsModule;
  }
  return undefined;
}

/**
 * Dynamically import `@tauri-apps/plugin-fs` at runtime. Isolated in its own one-line helper so
 * the single `@ts-expect-error` for the intentionally-absent module specifier lands precisely
 * on the import expression (it is NOT a site dependency — the static build must not bundle a
 * Tauri package). `webpackIgnore` keeps it out of the bundle graph. This is only the SECONDARY
 * path (a hypothetical bundler-based Tauri host); the primary path is the runtime global above,
 * which is what the current pre-bundled static-export shell uses.
 */
function importTauriFsPlugin(): Promise<unknown> {
  // @ts-expect-error — optional Tauri-only module, intentionally absent from the static build.
  return import(/* webpackIgnore: true */ "@tauri-apps/plugin-fs");
}

/**
 * Resolve the Tauri filesystem API scoped to the app's local-data dir, or `null` if the plugin
 * is not present / the fs capability was not granted. Passed to
 * `createWorkspaceStore({ loadTauriFs })`, which only calls it inside a Tauri webview.
 *
 * Resolution order: (1) the `window.__TAURI__.fs` runtime global injected by the desktop shell
 * (`withGlobalTauri` + registered `tauri_plugin_fs`) — the path the pre-bundled static export
 * uses; (2) a webpack-ignored bare `import()` for a hypothetical bundler-based host. Either way,
 * a missing plugin / permission error / unusable shape resolves to `null` so the store factory
 * falls back to the web backend rather than throwing.
 */
export async function loadTauriFs(): Promise<TauriFsApi | null> {
  // (1) Prefer the runtime global injected by the desktop shell. No import, so the static build
  // never bundles a Tauri package; on a plain browser `__TAURI__` is absent and this is skipped.
  const global = readGlobalTauriFs();
  if (global) return adaptFsModule(global);

  // (2) Fallback: a bundler-based host that can resolve the bare specifier at runtime. On the
  // GitHub Pages build the specifier is unresolvable, the import rejects, and we return `null`.
  try {
    // webpackIgnore keeps the Tauri plugin out of the static bundle entirely. `@tauri-apps/
    // plugin-fs` is deliberately NOT a site dependency, so `tsc` cannot resolve the specifier —
    // the import is a pure runtime concern. The expect-error in `importTauriFsPlugin` documents
    // that the missing-module diagnostic is intentional; the cast pins the runtime shape.
    const mod = (await importTauriFsPlugin()) as unknown as TauriFsModule;
    if (typeof mod?.writeTextFile !== "function") return null;
    return adaptFsModule(mod);
  } catch {
    // No plugin / capability not granted — the web (localStorage) backend takes over.
    return null;
  }
}

/** Project a resolved `@tauri-apps/plugin-fs` module onto the `TauriFsApi` the store consumes. */
function adaptFsModule(mod: TauriFsModule): TauriFsApi {
  return {
    baseDir: mod.BaseDirectory.AppLocalData,
    readTextFile: mod.readTextFile,
    writeTextFile: mod.writeTextFile,
    remove: mod.remove,
    exists: mod.exists,
    mkdir: mod.mkdir,
    readDir: mod.readDir,
  };
}
