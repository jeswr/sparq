# sparq GUI — operational workbench (Tauri 2 desktop + hosted web)

A cross-platform **operational workbench** for the sparq RDF + SPARQL engine, per the design
record [`research/gui-design.md`](../research/gui-design.md) §A (beads `sq-ixc3.8` / `sq-ixc3.9`,
epic `sq-ixc3`). It is a **DISTINCT app, not the marketing site in a window**: a dense IDE-style
workbench (left rail · thin top bar · IDE tab strip · status bar) over a live engine, where the
showcase capabilities are TOOLS you run, not pages that describe them.

> **Foundation shell.** This PR stands up the distinct frontend + app shell with a working Query
> tool over the live in-tab engine; the other TOOLS open as honest stubs the later phases fill
> (Cmd-K palette `sq-ixc3.10`, the multi-view query workbench `sq-ixc3.12`, the import drawer
> `sq-ixc3.13`, surfaces-as-tools `sq-ixc3.11`). The full per-platform desktop `cargo tauri build`
> needs the webview **system libraries** (webkit2gtk on Linux, WebView2 on Windows, WKWebView on
> macOS) validated in CI ([`.github/workflows/gui.yml`](../.github/workflows/gui.yml)), not
> necessarily on every dev box.

## Two builds, one frontend

The operational frontend (`gui/app/`) static-exports (`output: export`, backend-free) for BOTH
targets, selected by the single `NEXT_PUBLIC_BASE_PATH` switch (the same switch the site + the
`@sparq/client` wasm loader key off, kept in lockstep):

| Target | Command (`gui/app`) | `basePath` | Where it runs |
|---|---|---|---|
| **Tauri desktop** | `npm run build:tauri` | `""` (root-relative) | embedded in the desktop webview (serves from the `tauri://` root) |
| **Hosted "Try the GUI live" web** | `npm run build:web` | `/sparq/gui` (default; overridable) | a static page so the site's "Try the GUI" link has a real target (bead `sq-wt4s3`; the hosting/Pages-root slot is `sq-svtt`) |

The desktop bundle is wired via `gui/src-tauri/tauri.conf.json` — `frontendDist: "../app/out"`
and `beforeBuildCommand: cd ../app && npm run build:tauri`. **`frontendDist` is no longer
`site/out`** (the core of `sq-ixc3.8`): the GUI never embeds the marketing site.

## Why Tauri 2

The design weighed Tauri 2 vs egui/iced vs Electron/PWA and chose **Tauri 2** because it (1)
reuses the existing React/Tailwind stack + the WASM engine client, (2) unlocks the **full native
engine** on desktop (rayon, mmap, `save`/`open`, no ~2 GiB tab ceiling) via a direct Rust link,
(3) closes the static-site live-backend gap, and (4) is the only credible mobile path here. See
`research/gui-design.md` §1 for the full comparison and honest caveats.

## Layout

```
gui/
  README.md
  app/                  # NEW: the DISTINCT operational frontend (Next.js, static-exported)
    next.config.ts      #   env-switched basePath: build:tauri (root) / build:web (sub-path)
    src/
      app/              #   single route — the workbench shell (no /about, /benchmarks, …)
      components/
        workbench/      #   the shell: left-rail · top-bar · tab-strip · status-bar +
                        #     query-workbench (the working Query tool) + tool-stub (honest stubs)
        ui/             #   reused design-system primitives (button, badge) — sibling of the site
      data/tools.ts     #   the TOOLS taxonomy (each a VERB with an honesty-tier dot)
      lib/engine-context.tsx  # the ONE live wasm store + measured-latency query path
  e2e/                  # tauri-driver (WebDriver) e2e — launches the shell, runs a query, asserts
  src-tauri/
    Cargo.toml          # standalone crate root (own [workspace]); links sparq-engine + sparq-core
    tauri.conf.json     # window + bundle + frontendDist (= ../app/out)
    build.rs            # tauri_build::build()
    capabilities/       # least-privilege window capability (core + dialog:open)
    src/
      main.rs           # desktop binary -> sparq_gui_lib::run()
      lib.rs            # Tauri builder: manages EngineState, registers the command handlers
      engine.rs         # the DIRECT native engine link: load/query/queryQuads/update/explain/count/ask
```

## The app shell (sq-ixc3.9)

`gui/app/src/components/workbench/` is the operational shell from `gui-design.md` §A.2:

- **Left rail** (`w-56`): a workspace switcher (the persistent model is `sq-atb0`; foundation =
  default), a **datasets tree of the live store** (default + named graphs with per-graph counts)
  with an `+ Import` entry point, and the **TOOLS** list — each a VERB opened as a tab with an
  honesty-tier dot, never a page describing the feature.
- **Top bar** (`h-10`): a LOCAL⇄ENDPOINT target switch, the store size, a ⌘K hint, a theme toggle,
  and an engine status LED.
- **IDE tab strip** + a full-bleed work area (default = Query).
- **Status bar** (`h-6`): the **measured `performance.now()` latency** of the last run, the row
  count, the target, and the persistence backend.

## The direct native engine link (desktop)

`src-tauri/src/engine.rs` is the concrete proof of the design's embedding decision: a native
`sparq_core::Graph` behind a `Mutex`, with Tauri commands that mirror the wasm `Store` surface
(`load` / `query` / `query_quads` / `update_in_place` / `explain` / `explain_analyze` / `count` /
`ask` / `store_size`) but backed by the full native store. Its `#[cfg(test)]` tests exercise the
engine calls directly. The foundation frontend runs the in-tab WASM engine in BOTH targets (the
honest, working-today path); swapping the desktop target onto this IPC command layer is a later
phase (`sq-ixc3.6`).

## Shared TS client

The frontend consumes the framework-agnostic `@sparq/client`
([`packages/sparq-client`](../packages/sparq-client)) for the wasm `Store` type surface, the
loaders, and the result-shaping helpers — the **same package the site uses** — so the GUI is a
**zero-new-copy** consumer, not a third hand-redeclaration of the engine's TS surface.

## Honesty

No performance number is baked in — the status bar shows the **measured** latency of the query you
just ran, labelled as such (this work box / CI runner is non-canonical). The ZK/MPC tools are
**research-grade and not externally audited** (the v1 ZK verifier is internally re-audited only,
external accredited-cryptographer sign-off is pending; `sparq-mpc` is honest-majority semi-honest
only, and the site's MPC demo is an in-tab JS simulation, not live MPC) — the tool stubs carry
those caveats verbatim and never present either as a settled cryptographic guarantee. A
`walkthrough`/`soon` tool is never silently dressed up as `live`.
