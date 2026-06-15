# sparq feature-showcase site

The live-interactive feature-demonstration website for sparq, served at
**https://jeswr.github.io/sparq/**. A Next.js static export (`output: "export"`)
styled as a sibling of [`jeswr/solid-pod-manager`](https://github.com/jeswr/solid-pod-manager):
Tailwind v4 theme-in-CSS, the privacy-first teal OKLCH palette, shadcn (`radix-nova`),
Inter, `--radius 0.7rem`. Design record: `research/feature-showcase-site-design.md`.

## What's in this foundation

- **App shell** — `w-64` sidebar (feature surfaces grouped by the IA), sticky `h-16`
  backdrop-blur header, `max-w-6xl` main, mobile `Sheet` drawer, light/dark theme.
- **Landing / overview** — what sparq is, the live REPL, the three flagship cards, and
  the full surface grid (each card links to its page; unbuilt pages render an honest
  "coming soon" placeholder that states the planned execution tier).
- **Live SPARQL REPL** (`/try` and the landing marquee) — loads the lean sparq wasm
  bundle and runs **real SPARQL** against a sample graph via the actual Rust engine
  compiled to WebAssembly. Not a fixture; nothing is sent to a server.
- **/about** — the honest "what runs where" matrix.

The per-surface interactive demos and the three flagship demos (ZK car-hire, MPC £100k,
Solid pairs) are tracked as child beads of the showcase epic (`sq-4r4b`).

## Develop

```bash
# The REPL needs the wasm bundle. Build it once from the workspace root:
(cd ../js && npm ci && npm run build:wasm)   # → js/wasm/

cd site
npm ci
npm run dev        # sync-wasm → public/wasm, then next dev
npm run build      # → out/  (static export)
npm run lint
```

`public/wasm/` is generated (git-ignored): `scripts/sync-wasm.mjs` copies the
wasm-pack `--target web` output from `js/wasm/`. The `prebuild` script runs it
automatically before `next build`.

## Deploy

`.github/workflows/pages.yml` (on push to `main`) builds the wasm bundle + the static
export, **overlays the existing benchmark dashboard** (`dev/` from the `benchmark-data`
branch) into `out/dev/`, and publishes one Pages artifact via `actions/deploy-pages`.
The dashboard at `/sparq/dev/bench/` is preserved verbatim; the workflow never writes
the `benchmark-data` branch (bench.yml owns it). Publishing requires the repo's Pages
source to be **"GitHub Actions"** (Settings → Pages) — a one-time switch from the legacy
branch source.
