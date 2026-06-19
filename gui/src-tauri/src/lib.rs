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
        // The single native store, shared across all command invocations.
        .manage(EngineState::new())
        .invoke_handler(tauri::generate_handler![
            engine::load,
            engine::query,
            engine::query_quads,
            engine::update_in_place,
            engine::explain,
            engine::explain_analyze,
            engine::count,
            engine::ask,
            engine::store_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the sparq Tauri application");
}
