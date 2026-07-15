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
mod explain;
mod federation;
mod forms;
mod odrl;

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
            // [FABLE-5] sq-ixc3.14 — the ONE deliberate native QUERY command (the reviewed
            // exception to the removed pre-wired surface above): evaluate a SERVICE-bearing
            // SELECT/ASK over a snapshot of the live workspace store, because the in-tab WASM
            // engine cannot dial remote SPARQL endpoints. Every call installs the engine's
            // STRICT fail-closed egress allowlist (the per-workspace federation setting); the
            // capability is gated behind the non-default `federation` cargo feature and a lean
            // build registers a stub that fails loudly with a rebuild hint (federation.rs).
            federation::query_service,
            // [FABLE-5] sq-ixc3.19 — the plan explorer's ONE native command (the second
            // reviewed exception, after query_service): structured EXPLAIN / EXPLAIN
            // ANALYZE over a native snapshot of the live workspace store, because only
            // the native engine can measure REAL per-operator wall time (the in-tab wasm
            // engine reads 0 nanos — no monotonic clock on wasm32). Pure local
            // computation (no FS, no egress); under the `federation` build every call
            // pins the STRICT EMPTY egress allowlist so an ANALYZE of a SERVICE-bearing
            // query is refused pre-HTTP, never dialed (explain.rs).
            explain::explain_native,
            // [GPT-5.6] sq-1cnox — derive the Forms tool's complete serde FormDescription
            // from serialized snapshots of the active workspace. The sparq-forms stack is
            // default-OFF behind the `forms` feature; lean builds register the same command
            // name but return an actionable unavailable error instead of demo content.
            forms::derive_form,
            // [FABLE-5] sq-ixc3.15 — the ODRL policy tool's ONE native command: parse/validate
            // a Turtle ODRL policy, evaluate the (party, action, target) request, materialize
            // the policy through the odrl-bridge, and run the SAME query per requester through
            // PodStore's fail-closed per-session named-graph gating next to the ungated run.
            // Gated behind the non-default `odrl` cargo feature; a lean build registers a stub
            // that fails loudly with a rebuild hint (odrl.rs). Malformed policy ⇒ nothing
            // materializes ⇒ deny-everything with the parse error as the visible reason.
            odrl::odrl_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the sparq Tauri application");
}
