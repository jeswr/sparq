// [OPUS-4.8] sq-2e93 — the sparq Tauri 2 desktop binary.
//
// Thin entry point: delegate to the library `run()` (the lib/bin split keeps the door open
// for the later mobile track). `windows_subsystem = "windows"` suppresses the console window
// on a Windows release build (standard Tauri scaffold).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sparq_gui_lib::run();
}
