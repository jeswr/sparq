"use client";

// [OPUS-4.8] sq-ixc3.13 (epic sq-ixc3) — the OPTIONAL Tauri IPC bridge for the Import drawer's
// NATIVE loader.
//
// The desktop target links the engine directly (gui/src-tauri/src/engine.rs) and exposes the
// native loader over Tauri IPC: `load_path` (decode a disk file — incl. compressed + native-only
// HDT) and `load_text` (decode a pasted / fetched document), both returning the whole dataset as
// N-Quads for the in-tab store to merge. This module is the thin frontend bridge to those
// commands plus the native file-open dialog.
//
// HOW IT STAYS STATIC-EXPORT-SAFE + DEPENDENCY-FREE. The Tauri shell sets `withGlobalTauri: true`
// (gui/src-tauri/tauri.conf.json), which injects `window.__TAURI__` with `core.invoke` and the
// registered plugins' globals (`dialog.open`). We read those off the runtime window — so the GUI
// needs NO `@tauri-apps/*` npm dependency at all, and the hosted web build (where the global is
// absent) simply sees `null` and uses the in-tab WASM loader instead. Feature-detected via
// `isTauriRuntime()` from `@sparq/client`.
//
// NO performance claim is made here. The native loader's advantage (threads, no ~2 GiB wasm-tab
// ceiling, native-only HDT) is a capability statement, not a benchmarked number.

import { isTauriRuntime } from "@sparq/client";

/**
 * What the native loader (`load_text` / `load_path`) hands back — byte-for-byte the Rust
 * `LoadedDocument` (gui/src-tauri/src/engine.rs): the whole dataset as N-Quads (the named-graph-
 * preserving merge wire format), the whole-dataset quad count, and the format it parsed as.
 */
export interface LoadedDocument {
  /** The whole dataset serialised to N-Quads (default graph + every named graph). */
  nquads: string;
  /** Total triples/quads parsed (default graph + every named graph). */
  count: number;
  /** The RDF serialisation the native loader actually parsed the document as. */
  format: string;
}

/**
 * [OPUS-4.8] sq-cno90 (#820 follow-up) — what the native disk-usage probe (`disk_usage`) hands
 * back, byte-for-byte the Rust `DiskUsage` (gui/src-tauri/src/disk.rs): the resolved
 * `$APPLOCALDATA/workspaces` path, its REAL on-disk byte total (a recursive `stat()` sum of every
 * file), and whether the dir exists yet. This is the OS-reported store footprint the desktop
 * status bar PREFERS over the snapshot-bytes estimate.
 */
export interface DiskUsage {
  /** The resolved absolute path of the `$APPLOCALDATA/workspaces` tree the figure covers. */
  path: string;
  /** The summed real byte size of every regular file under `path` (apparent size, `du --bytes`). */
  bytes: number;
  /** Whether the workspaces dir exists yet (false on a fresh install → `bytes` is 0). */
  exists: boolean;
}

/** Re-export the runtime detector so callers need only one import. */
export { isTauriRuntime };

/** The IPC invoker shape `window.__TAURI__.core.invoke` exposes. */
type TauriInvoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

/** The subset of the injected `window.__TAURI__` global this module reads. */
interface TauriGlobal {
  core?: { invoke?: TauriInvoke };
}

/** Read the injected Tauri global (present only inside the desktop webview), or `null`. */
function tauriGlobal(): TauriGlobal | null {
  if (!isTauriRuntime() || typeof window === "undefined") return null;
  const g = (window as unknown as { __TAURI__?: TauriGlobal }).__TAURI__;
  return g ?? null;
}

/** Resolve the Tauri `invoke`, or `null` outside the desktop webview / if it is absent. */
function tauriInvoke(): TauriInvoke | null {
  const g = tauriGlobal();
  return typeof g?.core?.invoke === "function" ? g.core.invoke : null;
}

/**
 * The RDF file extensions the native open dialog offers as a filter group. Includes the
 * compression suffixes the native loader streams (`.gz` / `.bz2` / `.zst`) and the native-only
 * HDT archive extensions — capabilities the web target fundamentally lacks.
 */
export const RDF_FILE_EXTENSIONS = [
  "ttl",
  "nt",
  "nq",
  "trig",
  "jsonld",
  "json",
  "hdt",
  "gz",
  "bz2",
  "zst",
  "zstd",
] as const;

/**
 * Open the native file picker for an RDF document, returning the chosen absolute path or `null`
 * (cancelled / not in a Tauri webview). Implemented by invoking the dialog plugin's `open`
 * COMMAND directly (`plugin:dialog|open`) over IPC — so the GUI needs no `@tauri-apps/plugin-
 * dialog` npm package, only the Rust-side plugin (`tauri_plugin_dialog::init()`) + the granted
 * `dialog:allow-open` capability. The native loader (`load_path`) then decodes each chosen file.
 *
 * (sq-eydh9) Returns an array of selected paths (multi-select, `multiple: true`). An empty array
 * means the user cancelled. On old desktop builds that return a single string the result is
 * normalised to a one-element array so callers never need to branch on the return type.
 */
export async function pickRdfFile(): Promise<string[]> {
  const invoke = tauriInvoke();
  if (!invoke) return [];
  try {
    // The dialog plugin's `open` command returns the selected path(s) or null on cancel.
    // `multiple: true` → string[] on confirm; the API can still return a plain string when
    // a legacy Tauri build conflates single/multi, so we normalise to string[] either way.
    const selected = await invoke<string | string[] | null>("plugin:dialog|open", {
      options: {
        multiple: true,
        directory: false,
        title: "Import RDF files into the workspace",
        filters: [
          { name: "RDF (incl. compressed + HDT)", extensions: [...RDF_FILE_EXTENSIONS] },
          { name: "All files", extensions: ["*"] },
        ],
      },
    });
    if (selected === null) return [];
    if (typeof selected === "string") return [selected];
    return selected;
  } catch {
    return [];
  }
}

/**
 * Decode a disk file through the NATIVE loader (`load_path`): compressed streams + native-only
 * HDT + named-graph-preserving, off the main thread in the Rust engine. Returns the whole
 * dataset as N-Quads for the in-tab store to merge, or `null` if not in a Tauri webview. Throws
 * the native error string on a decode/parse failure (e.g. an HDT import in a lean build).
 */
export async function nativeLoadPath(
  path: string,
  format: string,
  preserveGraphs: boolean,
): Promise<LoadedDocument | null> {
  const invoke = tauriInvoke();
  if (!invoke) return null;
  return invoke<LoadedDocument>("load_path", { path, format, preserveGraphs });
}

/**
 * Decode an in-memory document (paste / a fetched URL body) through the NATIVE loader
 * (`load_text`) so its named graphs are preserved by the SAME engine path a disk file takes.
 * Returns the whole dataset as N-Quads, or `null` if not in a Tauri webview (the web target
 * parses paste/URL in the in-tab WASM engine instead). Throws the native error string on a
 * parse failure.
 */
export async function nativeLoadText(
  text: string,
  format: string,
  preserveGraphs: boolean,
): Promise<LoadedDocument | null> {
  const invoke = tauriInvoke();
  if (!invoke) return null;
  return invoke<LoadedDocument>("load_text", { text, format, preserveGraphs });
}

/** True when the native loader IPC path is available (we are inside a Tauri webview). */
export function hasNativeLoader(): boolean {
  return isTauriRuntime();
}

/**
 * [OPUS-4.8] sq-cno90 (#820 follow-up) — probe the REAL on-disk byte size of the
 * `$APPLOCALDATA/workspaces` tree through the native `disk_usage` command, or `null` outside the
 * desktop shell (the web target has no native FS — the status bar falls back to the snapshot-bytes
 * estimate there). This is the OS-reported store footprint: a recursive `stat()` sum the desktop
 * status bar PREFERS over the snapshot estimate. Never fabricated — a real value or `null`.
 */
export async function nativeDiskUsage(): Promise<DiskUsage | null> {
  const invoke = tauriInvoke();
  if (!invoke) return null;
  try {
    return await invoke<DiskUsage>("disk_usage");
  } catch {
    // A path-resolution failure (or an old desktop build without the command) must not break the
    // status bar — fall back to the snapshot estimate by reporting "no OS figure available".
    return null;
  }
}
