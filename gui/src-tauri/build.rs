// [OPUS-4.8] sq-2e93 — Tauri build script: generates the context (parses tauri.conf.json,
// embeds the frontend dist + capabilities). Standard Tauri 2 scaffold.
//
// FRONTEND-DIST CAVEAT (recorded here, not in tauri.conf.json — the `build` table is
// deserialized with deny_unknown_fields, so a JSON "//" comment key fails the build): the
// GUI reuses the site's Next.js frontend. `beforeBuildCommand` builds the static export and
// `frontendDist` (../../site/out) is that export tree. The site hardcodes basePath '/sparq'
// for GitHub Pages, but a Tauri webview serves from the `tauri://` root, so the export
// consumed here MUST be built with NEXT_PUBLIC_BASE_PATH='' / basePath '' (env-switched).
// Wiring that env-switch into the site config is a follow-up bead.
fn main() {
    tauri_build::build();
}
