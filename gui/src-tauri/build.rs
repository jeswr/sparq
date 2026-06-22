// [OPUS-4.8] sq-2e93 / sq-ixc3.8 — Tauri build script: generates the context (parses
// tauri.conf.json, embeds the frontend dist + capabilities). Standard Tauri 2 scaffold.
//
// FRONTEND-DIST (recorded here, not in tauri.conf.json — the `build` table is deserialized with
// deny_unknown_fields, so a JSON "//" comment key fails the build): the GUI embeds the DISTINCT
// operational frontend at `gui/app/` — NOT the marketing site (sq-ixc3.8). `beforeBuildCommand`
// runs `gui/app`'s `build:tauri` (NEXT_PUBLIC_BASE_PATH='' → a root-relative static export,
// since a Tauri webview serves from the `tauri://` root where a `/prefix` 404s) and
// `frontendDist` (../app/out) is that export tree. The SAME frontend also builds as a hosted
// "Try the GUI live" web target via `gui/app`'s `build:web`.
fn main() {
    tauri_build::build();
}
