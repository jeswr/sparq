// [OPUS-4.8] sq-2e93 — the sparq Tauri 2 GUI library entry point.
//
// The library/binary split is the Tauri 2 mobile-ready shape: `run()` lives here so the
// same entry point serves desktop (the `main.rs` binary) and, on a later spike-gated track,
// the Android/iOS shells (research/gui-design.md §1/§3). The desktop binary just calls
// `sparq_gui_lib::run()`.
//
// HONESTY: this is a SCAFFOLD. It registers the native engine command layer (the direct
// `sparq-engine`/`sparq-core` link) and a single main window that loads the reused Next.js
// frontend, per the design's MVP item 5. A full `cargo build` needs the webview system
// libraries (webkit2gtk / WebView2 / WKWebView) and is validated in CI (gui.yml), not
// necessarily locally. The engine command layer itself is unit-tested natively (engine.rs).

mod disk;
mod engine;

use engine::EngineState;

/// Build and run the Tauri application: register the native engine state + command handlers,
/// and let `tauri.conf.json` configure the main window (which loads the reused Next.js
/// frontend). The `#[cfg_attr(mobile, …)]` attribute is the Tauri 2 mobile entry hook;
/// harmless on desktop and ready if the mobile track is taken later.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // [OPUS-4.8] sq-ixc3.6 (epic sq-ixc3) — register the filesystem plugin so the webview's
        // workspace store (@sparq/client `TauriWorkspaceStore`) can persist each workspace to
        // `<app-local-data>/workspaces/*.json` and survive an app restart. The capability
        // (capabilities/default.json) scopes this to `$APPLOCALDATA/workspaces` ONLY — the
        // plugin grants no path the capability does not explicitly allow. With the plugin
        // unregistered the webview's runtime `fs` lookup fails and the GUI degrades to the
        // browser localStorage backend.
        .plugin(tauri_plugin_fs::init())
        // The single native store, shared across all command invocations.
        .manage(EngineState::new())
        .invoke_handler(tauri::generate_handler![
            // [OPUS-4.8] sq-ixc3.13 — the Import drawer's native loader: decode a pasted
            // document (`load_text`) or a disk file (`load_path`, incl. compressed + native-only
            // HDT) into N-Quads for the in-tab store to merge.
            //
            // [OPUS-4.8] sq-w2sod — the INVOCABLE SURFACE is deliberately minimal. `load_path` is
            // an app-defined command NOT bound by the fs-plugin capability allowlist, so any script
            // in the webview can invoke it; it is therefore GATED to paths the user picked through
            // `pick_rdf_files` (which opens the OS file dialog SERVER-SIDE and records the
            // canonicalised selection as the unforgeable approved-path capability). The nine
            // pre-wired query/update commands (`load` / `query` / `query_quads` / `update_in_place`
            // / `explain` / `explain_analyze` / `count` / `ask` / `store_size`) that the frontend
            // never invoked — the query/update path runs in the in-tab WASM engine — are
            // UNREGISTERED to shrink the attack surface.
            engine::pick_rdf_files,
            engine::load_text,
            engine::load_path,
            // [OPUS-4.8] sq-cno90 (#820 follow-up) — the precise native disk-usage probe: stat()s
            // the resolved $APPLOCALDATA/workspaces tree and returns its REAL byte total, so the
            // status bar can show the OS-reported store footprint instead of the snapshot estimate.
            disk::disk_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the sparq Tauri application");
}
