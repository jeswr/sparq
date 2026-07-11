"use client";

// [OPUS-4.8] sq-ixc3.13 (epic sq-ixc3) — the OPTIONAL Tauri IPC bridge for the Import drawer's
// NATIVE loader.
//
// The desktop target links the engine directly (gui/src-tauri/src/engine.rs) and exposes the
// native loader over Tauri IPC: `pick_rdf_files` (open the OS file dialog SERVER-SIDE + approve the
// chosen paths), `load_path` (decode an approved disk file — incl. compressed + native-only HDT),
// and `load_text` (decode a pasted / fetched document). The loaders return the whole dataset as
// N-Quads for the in-tab store to merge. This module is the thin frontend bridge to those commands.
//
// [OPUS-4.8] sq-w2sod — the file dialog is now driven from Rust (`pick_rdf_files`), not from the
// webview via `plugin:dialog|open`. The chosen paths are the ONLY ones `load_path` will load
// (approved-path capability), so a webview script cannot forge a path to read; and every load
// failure returns a single generic error (no OS/path detail) to close the FS-enumeration oracle.
//
// HOW IT STAYS STATIC-EXPORT-SAFE + DEPENDENCY-FREE. The Tauri shell sets `withGlobalTauri: true`
// (gui/src-tauri/tauri.conf.json), which injects `window.__TAURI__` with `core.invoke`. We read
// that off the runtime window — so the GUI needs NO `@tauri-apps/*` npm dependency at all, and the
// hosted web build (where the global is absent) simply sees `null` and uses the in-tab WASM loader
// instead. Feature-detected via `isTauriRuntime()` from `@sparq/client`.
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
 * Open the native file picker for an RDF document, returning the chosen absolute path(s) or an
 * empty array (cancelled / not in a Tauri webview).
 *
 * [OPUS-4.8] sq-w2sod — this now invokes the app-defined `pick_rdf_files` command, which opens the
 * OS file dialog SERVER-SIDE (in Rust) rather than from the webview. That is the security-relevant
 * change: `pick_rdf_files` canonicalises each chosen path and records it in the native engine's
 * APPROVED-PATH set, which is the ONLY thing the gated `load_path` command will subsequently read
 * — so a webview script cannot forge a path to load; the user must physically pick the file. The
 * returned strings are the canonical paths the caller threads back into `nativeLoadPath`.
 *
 * (sq-eydh9) Returns an array of selected paths (multi-select). An empty array means the user
 * cancelled (or we are not in a Tauri webview).
 */
export async function pickRdfFile(): Promise<string[]> {
  const invoke = tauriInvoke();
  if (!invoke) return [];
  try {
    // `pick_rdf_files` returns the canonicalised, now-approved paths (an empty array on cancel).
    // The filter/title live Rust-side so the webview no longer needs the `dialog:allow-open`
    // capability nor an `@tauri-apps/plugin-dialog` npm package.
    const selected = await invoke<string[] | null>("pick_rdf_files");
    return selected ?? [];
  } catch {
    return [];
  }
}

/**
 * Decode a disk file through the NATIVE loader (`load_path`): compressed streams + native-only
 * HDT + named-graph-preserving, off the main thread in the Rust engine. Returns the whole
 * dataset as N-Quads for the in-tab store to merge, or `null` if not in a Tauri webview.
 *
 * [OPUS-4.8] sq-w2sod — `path` MUST be one returned by {@link pickRdfFile} (i.e. a path the user
 * picked through the native dialog, which the Rust side recorded as approved). Any other path — and
 * any open/read/parse failure — throws the SAME generic error string (no OS detail, no path echo),
 * so this cannot be used as a filesystem existence/permission oracle.
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
 * [FABLE-5] sq-ixc3.14 — whether the native federated-query IPC path is available (we are
 * inside a Tauri webview). Whether the desktop build actually COMPILED the `federation`
 * feature is only knowable at invoke time: a lean build's `query_service` stub returns a loud
 * rebuild-hint error, which surfaces through the result panel like any other run error.
 */
export function hasNativeFederation(): boolean {
  return isTauriRuntime();
}

/**
 * [FABLE-5] sq-ixc3.14 — evaluate a SERVICE-bearing SELECT/ASK over the NATIVE engine
 * (`query_service`, gui/src-tauri/src/federation.rs): `dataset` is the live workspace store's
 * whole-dataset N-Quads snapshot, `allow` the per-workspace federation egress allowlist the
 * engine enforces STRICTLY (fail-closed: an endpoint off the list is refused pre-HTTP with the
 * stable `SERVICE egress refused` marker — see lib/federation.ts). Returns the SPARQL 1.1 JSON
 * results document, or `null` outside the desktop shell (the web target degrades honestly in
 * the Query tool instead). Throws the native error string on refusal / parse / HTTP failure.
 */
export async function nativeServiceQuery(
  dataset: string,
  query: string,
  allow: string[],
): Promise<string | null> {
  const invoke = tauriInvoke();
  if (!invoke) return null;
  return invoke<string>("query_service", { dataset, query, allow });
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
