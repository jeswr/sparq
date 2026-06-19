# sparq GUI (Tauri 2 — MVP scaffold)

A cross-platform desktop **scaffold** for the sparq RDF + SPARQL workbench, per the design
record [`research/gui-design.md`](../research/gui-design.md) (bead `sq-2e93`). It follows the
design's framework recommendation: **Tauri 2**, reusing the existing Next.js/React frontend
(`site/`) in a native webview, with the engine linked **directly as native Rust**.

> **This is a scaffold, not a shipped app.** It is structurally valid (`tauri.conf.json`
> parses, `cargo metadata` resolves the full dependency graph) and the engine command layer
> is unit-tested natively, but a full `cargo build` / `cargo tauri build` needs the webview
> **system libraries** (webkit2gtk on Linux, WebView2 on Windows, WKWebView on macOS) that
> are not present on every dev box. The full build/lint/typecheck is validated in the
> path-scoped CI lane [`.github/workflows/gui.yml`](../.github/workflows/gui.yml), not
> necessarily locally. Until that lane is green on a runner, treat this as **CI-validated /
> locally-scaffolded**, not a working app.

## Why Tauri 2

The design weighed Tauri 2 vs egui/iced vs Electron/PWA and chose **Tauri 2** because it (1)
maximises reuse of the existing ~40-component React frontend, (2) unlocks the **full native
engine** (rayon, mmap, `save`/`open`, no ~2 GiB tab ceiling — escaping every WASM limit), (3)
closes the static-site live-backend gap, and (4) is the only credible mobile path here. See
`research/gui-design.md` §1 for the full comparison and honest caveats.

## Layout

```
gui/
  README.md
  src-tauri/
    Cargo.toml          # standalone crate root (own [workspace]); links sparq-engine + sparq-core
    tauri.conf.json     # window + bundle + frontendDist (= site/out)
    build.rs            # tauri_build::build()
    capabilities/       # least-privilege window capability (core + dialog:open)
    icons/              # placeholder brand icons (PNG)
    src/
      main.rs           # desktop binary -> sparq_gui_lib::run()
      lib.rs            # Tauri builder: manages EngineState, registers the command handlers
      engine.rs         # the DIRECT native engine link: load/query/queryQuads/update/explain/count/ask
```

## The direct native engine link

`src/engine.rs` is the concrete proof of the design's embedding decision: a native
`sparq_core::Graph` behind a `Mutex`, with Tauri commands that mirror the wasm `Store`
surface exactly (`crates/sparq-wasm/src/lib.rs`) — `load`, `query`, `query_quads`,
`update_in_place`, `explain`, `explain_analyze`, `count`, `ask`, `store_size` — but backed by
the full native store rather than the WASM read-replica. Its `#[cfg(test)]` tests exercise the
engine calls **directly** (no Tauri runtime), so they are the locally-runnable proof that the
command layer is wired to the real engine, even when the full Tauri build is CI-only.

## Frontend reuse + the basePath switch (`sq-9vw5`)

The webview loads the site's static export (`frontendDist: "../../site/out"`). The site
defaults to `basePath: "/sparq"` for GitHub Pages, but a Tauri webview serves from the
`tauri://` root, so the export the GUI consumes **must** be built root-relative
(`basePath: ""`). `site/next.config.ts` now **env-switches** `basePath`/`assetPrefix` off
`NEXT_PUBLIC_BASE_PATH`: the `beforeBuildCommand` passes `NEXT_PUBLIC_BASE_PATH=''`, so the
GUI's export is root-relative with no extra step. Hand-written absolute asset hrefs (the
favicon, the COOP/COEP service-worker `<Script src>`, the paper PDF links) read the **same**
env var via `site/src/lib/base-path.ts`, so they move with the build mode too. See the
"Build modes" table in [`site/README.md`](../site/README.md).

## Shared TS client

The frontend consumes the framework-agnostic `@sparq/client`
([`packages/sparq-client`](../packages/sparq-client)) for the `WasmStore` type surface and
loaders — the same package the site uses — so the GUI is a **zero-new-copy** consumer rather
than a third hand-redeclaration of the engine's TS surface (the drift liability the design
flags). On desktop the GUI prefers the IPC command layer above over WASM, but the shared
types remain the single source of truth for the result shapes both render.

## Honesty

No performance number is asserted. The ZK/MPC surfaces the reused frontend can drive are
**research-grade and not externally audited**; that framing is inherited unchanged from the
site. Mobile (Android/iOS), the native `save`/`open` persistence path, endpoint mode, and the
editor uplift are later phases in the design, not in this scaffold.
