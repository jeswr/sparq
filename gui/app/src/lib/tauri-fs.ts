"use client";

// [FABLE-5] sq-w3dmj (was [OPUS-4.8] sq-ixc3.13, epic sq-ixc3) — the OPTIONAL Tauri filesystem
// loader for workspace persistence in the operational GUI. A sibling of the historical
// site/src/lib/tauri-fs.ts (same discipline): it bridges the framework-agnostic `@sparq/client`
// workspace store (sq-atb0) to the Tauri 2 `@tauri-apps/plugin-fs` plugin WITHOUT the static
// (web) build ever depending on a Tauri package.
//
// HOW IT RESOLVES AT RUNTIME (sq-w3dmj). The desktop shell registers `tauri_plugin_fs` and sets
// `app.withGlobalTauri = true` (gui/src-tauri/tauri.conf.json), so Tauri injects the plugin onto
// `window.__TAURI__.fs`. We resolve the fs API from THAT runtime global FIRST — the ONLY path
// that can work in a pre-bundled static-export webview, which has no import map and therefore
// can never resolve a bare module specifier. The (webpack-ignored, unbundled) bare `import()` is
// kept strictly as a fallback for a hypothetical bundler-based Tauri host. The previous version
// of this file dropped the global-first step and relied on the bare import alone, so the import
// always rejected inside the desktop shell and persistence silently degraded to localStorage —
// that regression is what sq-w3dmj fixes (restoring the historical site loader's behaviour).
//
// HOW IT STAYS STATIC-EXPORT-SAFE. `@tauri-apps/plugin-fs` is NOT a dependency
// (gui/app/package.json); the fallback imports it with a `webpackIgnore` dynamic `import()`
// evaluated only inside a Tauri webview (feature-detected by `isTauriRuntime` in
// `createWorkspaceStore`). On the hosted "Try the GUI live" web build neither the global nor
// the import resolves, so the loader returns `null` and the GUI cleanly degrades to the
// browser localStorage backend.
//
// CAPABILITY SCOPE (sq-ixc3.6). The fs grant is LEAST-PRIVILEGE: the shell's capability
// (gui/src-tauri/capabilities/default.json) scopes every fs permission to
// `$APPLOCALDATA/workspaces` ONLY. A call outside that scope is denied by Tauri and rejects,
// which the workspace store treats the same as an absent plugin.

import { type TauriFsApi } from "@sparq/client";

/**
 * The shape of `@tauri-apps/plugin-fs` we read at runtime. Declared locally (not imported from
 * the intentionally-absent package) so the web build typechecks without the Tauri package.
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
 * never imports a Tauri npm package (mirrors how tauri-ipc.ts reads
 * `window.__TAURI__.core.invoke`). Returns `undefined` in a plain browser (no `__TAURI__`),
 * where the caller falls through to the bare-import fallback and ultimately the web backend.
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
 * Dynamically import `@tauri-apps/plugin-fs` at runtime — the SECONDARY path, for a
 * hypothetical bundler-based Tauri host that can resolve the bare specifier. Isolated in its
 * own helper so the single `@ts-expect-error` for the intentionally-absent module lands
 * precisely on the import expression (`webpackIgnore` keeps it out of the bundle graph).
 */
function importTauriFsPlugin(): Promise<unknown> {
  // @ts-expect-error — optional Tauri-only module, intentionally absent from the static build.
  return import(/* webpackIgnore: true */ "@tauri-apps/plugin-fs");
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

/**
 * Resolve the Tauri filesystem API scoped to the app's local-data dir, or `null` if the plugin
 * is not present / the fs capability was not granted. Passed to
 * `createWorkspaceStore({ loadTauriFs })`, which only calls it inside a Tauri webview — and
 * whose RESOLVED `store.backend` (not a Tauri-presence guess) is what the status bar / switcher
 * "saved on device" labels render, so the label is honest by construction.
 *
 * Resolution order (sq-w3dmj): (1) the `window.__TAURI__.fs` runtime global injected by the
 * desktop shell — the path the pre-bundled static export REQUIRES; (2) a webpack-ignored bare
 * `import()` for a hypothetical bundler-based host. Either way, a missing plugin / permission
 * error / unusable shape resolves to `null` so the store factory falls back to the web backend
 * rather than throwing.
 */
export async function loadTauriFs(): Promise<TauriFsApi | null> {
  // (1) Prefer the runtime global injected by the desktop shell. No import, so the static build
  // never bundles a Tauri package; on a plain browser `__TAURI__` is absent and this is skipped.
  const global = readGlobalTauriFs();
  if (global) return adaptFsModule(global);

  // (2) Fallback: a bundler-based host that can resolve the bare specifier at runtime. In the
  // static-export webview and on the hosted web build the specifier is unresolvable, the import
  // rejects, and we return `null`.
  try {
    const mod = (await importTauriFsPlugin()) as unknown as TauriFsModule;
    if (typeof mod?.writeTextFile !== "function") return null;
    return adaptFsModule(mod);
  } catch {
    // No plugin / capability not granted — the web (localStorage) backend takes over.
    return null;
  }
}
