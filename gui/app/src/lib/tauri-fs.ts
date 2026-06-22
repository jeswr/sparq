"use client";

// [OPUS-4.8] sq-ixc3.13 (epic sq-ixc3) — the OPTIONAL Tauri filesystem loader for workspace
// persistence in the operational GUI. A sibling of site/src/lib/tauri-fs.ts (same discipline):
// it bridges the framework-agnostic `@sparq/client` workspace store (sq-atb0) to the Tauri 2
// `@tauri-apps/plugin-fs` plugin WITHOUT the static (web) build ever depending on a Tauri
// package.
//
// HOW IT STAYS STATIC-EXPORT-SAFE. `@tauri-apps/plugin-fs` is imported with a `webpackIgnore`
// dynamic `import()` evaluated only inside a Tauri webview (feature-detected by `isTauriRuntime`).
// On the hosted "Try the GUI live" web build the import is never reached, never bundled.
//
// HONEST LIMITATION. The on-disk path activates only when the Tauri shell grants the `fs`
// capability scoped to AppLocalData (follow-up bead sq-ixc3.6). Until then this loader returns
// `null` and the GUI cleanly degrades to the browser localStorage backend — even inside Tauri.

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

/** Dynamically import the Tauri fs plugin (webpackIgnore keeps it out of the web bundle). */
function importTauriFsPlugin(): Promise<unknown> {
  // @ts-expect-error — optional Tauri-only module, intentionally absent from the static build.
  return import(/* webpackIgnore: true */ "@tauri-apps/plugin-fs");
}

/**
 * Resolve the Tauri filesystem API scoped to the app's local-data dir, or `null` if the plugin
 * is not importable / the fs capability was not granted. Passed to
 * `createWorkspaceStore({ loadTauriFs })`, which only calls it inside a Tauri webview.
 */
export async function loadTauriFs(): Promise<TauriFsApi | null> {
  try {
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
