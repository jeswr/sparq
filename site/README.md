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
- **Persistent cross-session workspaces** (`sq-atb0`) — the REPL's workspace panel
  saves a named workspace: a **snapshot** of the loaded dataset (the whole default+named-graph
  content as N-Quads — a save/open cache, not a re-ingest-from-source), the imported-source
  list (local + URL, with the URL kept so a remote source can be re-fetched), and the SPARQL
  editor state (query text + run mode + endpoint URL — never a bearer token). The persistence
  is **one abstraction, three runtime-selected backends** (`@sparq/client` `workspace.ts`):
  Tauri local-disk on the desktop app (when the shell grants the `fs` capability), browser
  `localStorage` on GitHub Pages (the static-export path), and an in-memory session fallback —
  feature-detected, so the static export never depends on a Tauri API. The last workspace
  re-hydrates on startup. **Honest limitation**: a previously chosen local file cannot be
  silently re-read across sessions (the browser keeps no persistent handle), so the snapshot is
  a local import's durable copy; the panel says so plainly.
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
npm run test:unit  # node:test — pure helper logic (REPL dataset, …)
npm run test:e2e   # Playwright — headless browser smoke tests (see below)
```

`public/wasm/` is generated (git-ignored): `scripts/sync-wasm.mjs` copies the
wasm-pack `--target web` output from `js/wasm/`. The `prebuild` script runs it
automatically before `next build`.

### Build modes — `basePath` (Pages vs Tauri) — `sq-9vw5`

`next.config.ts` env-switches `basePath`/`assetPrefix` off `NEXT_PUBLIC_BASE_PATH` so the
**same** `out/` export serves two hosts. The `@sparq/client` wasm loader keys its runtime
asset URLs off the *same* env var, so the build-time route prefix and the runtime wasm-fetch
prefix stay in lockstep.

| Host | Command | `basePath` | When |
|---|---|---|---|
| **GitHub Pages** (default) | `npm run build` | `/sparq` | served under `https://jeswr.github.io/sparq/` — every asset/route is `/sparq`-prefixed |
| **Tauri 2 webview** | `npm run build:tauri` (= `cross-env NEXT_PUBLIC_BASE_PATH= npm run build`) | `''` (root-relative) | the desktop GUI serves the export from the `tauri://` root, where a `/sparq` prefix would 404 |

The env var is read once in `next.config.ts`: **unset** keeps the historical `/sparq` default
(no caller change); an explicit **empty string** selects the root-relative export; a malformed
value falls back to the Pages default. The `build:tauri` script uses [`cross-env`](https://www.npmjs.com/package/cross-env)
(`cross-env NEXT_PUBLIC_BASE_PATH= npm run build`) so the env var is set to an **empty string**
identically on Linux, macOS, and Windows `cmd.exe` — the bare `NEXT_PUBLIC_BASE_PATH='' npm run build`
inline-prefix form is a Unix-shell idiom that `cmd.exe` cannot parse. The GUI's
`gui/src-tauri/tauri.conf.json` `beforeBuildCommand` runs this script, so a Tauri build gets the
right export with no extra step.

### Browser smoke tests (Playwright)

`e2e/` holds headless-browser smoke tests driven by Playwright against a real `next dev`
server (config: `playwright.config.ts`). The first time, install the browser:

```bash
npx playwright install chromium
npm run test:e2e
```

The current coverage is the **ZK car-hire prover pre-warm** (`e2e/zk-prewarm.spec.ts`,
bead sq-5q63): it loads `/showcase/zk-car-hire`, waits for the *Prover ready* pill, and
asserts the first **Generate ZK proof** click pays no cold start — i.e. it does not
re-pay the lazy `@noir-lang/noir_js` + `@aztec/bb.js` dynamic-import or the
`Barretenberg.new` WASM instantiate that `prewarmProver()` already did on route mount.
That is observed via a test-only cold-start counter the prover mirrors onto
`window.__zkProverColdStarts` (pure observability; it changes nothing the prover proves).
CI runs this lane on site-touching PRs (`.github/workflows/site-e2e.yml`); Playwright
outputs (`test-results/`, `playwright-report/`, the browser cache) are git-ignored.

## Papers (the academic paper factory)

The `/papers` route is generated by the paper factory (epic **sq-gum8**; design record
`research/paper-factory-design.md`, process `skills/academic-paper/SKILL.md`).

- **Sources** — `papers/*.typ` (single-source [Typst](https://typst.app/)); shared helpers
  in `papers/_lib/bench.typ`. Numbers are **never hard-coded** — each paper reads them from
  the paper-bound evidence file (`src/data/paper-evidence.json`) via `--input data=...`, so
  the PDF and the in-site HTML cannot disagree and a paper auto-updates as evidence refreshes.
- **Build** — `scripts/build-papers.mjs` (run by `prebuild` + `dev`, also `npm run
  build-papers`) compiles each registered paper to a **PDF** (`public/papers/<slug>.pdf`, the
  download) and a **semantic HTML fragment** (`src/generated/papers/<slug>.html`, the in-site
  render). Both `public/papers/` and `src/generated/` are git-ignored build outputs.
- **In-site render** — Typst's native HTML export (`typst compile --format html --features
  html`), rendered as a static asset (no WASM compiler shipped to the browser). This was
  chosen over `typst.ts` for static-export compatibility + a far smaller client payload; the
  trade-off is that page-layout-only constructs (centring, rules) are dropped in the HTML
  view but preserved in the PDF.
- **Honesty gate** — `papers/_lib/bench.typ`'s `headline(key)` accessor **fails the build** if
  a paper cites an `environment: "indicative"` (non-canonical work-box) number as a headline
  result; only `environment: "canonical"` (deterministic, machine-independent) numbers may
  back a claim. `build-papers.mjs` also schema-checks the evidence file first.
- **Toolchain** — needs the **Typst CLI 0.15+**. Install the release binary
  (`https://github.com/typst/typst/releases`) on `PATH` (or `~/.local/bin` / `~/.cargo/bin`,
  or set `TYPST_BIN`). Without it, `build-papers.mjs` degrades to placeholders + a warning so
  local dev still runs; **CI installs Typst** (pinned + SHA-256-verified in `pages.yml`) so
  the real artifacts are produced. To register a new paper: add an entry to
  `src/data/papers.ts` + a `papers/<slug>.typ`.

## Deploy

`.github/workflows/pages.yml` (on push to `main`) builds the wasm bundle + the static
export, **overlays the existing benchmark dashboard** (`dev/` from the `benchmark-data`
branch) into `out/dev/`, and publishes one Pages artifact via `actions/deploy-pages`.
The dashboard at `/sparq/dev/bench/` is preserved verbatim; the workflow never writes
the `benchmark-data` branch (bench.yml owns it). Publishing requires the repo's Pages
source to be **"GitHub Actions"** (Settings → Pages) — a one-time switch from the legacy
branch source.
