# sparq feature-showcase site

The live-interactive feature-demonstration website for sparq, served at
**https://sparq.jeswr.org/**. A Next.js static export (`output: "export"`)
styled as a sibling of [`jeswr/solid-pod-manager`](https://github.com/jeswr/solid-pod-manager):
Tailwind v4 theme-in-CSS, the privacy-first teal OKLCH palette, shadcn (`radix-nova`),
Inter, `--radius 0.7rem`. Design record: `research/feature-showcase-site-design.md`.

## What's in this foundation

- **App shell** — `w-64` sidebar (feature surfaces grouped by the IA), sticky `h-16`
  backdrop-blur header, `max-w-6xl` main, mobile `Sheet` drawer, light/dark theme.
- **Landing / overview** — what sparq is, the live REPL, the three flagship cards, and
  the full surface grid (each card links to its page; unbuilt pages render an honest
  "coming soon" placeholder that states the planned execution tier).
- **Live SPARQL REPL** (`/try` and the landing marquee) — runs **real SPARQL** against a
  sample graph via the actual Rust engine compiled to WebAssembly. Not a fixture; nothing is
  sent to a server. The component is code-split and the wasm engine loads lazily on browser
  idle, so it never blocks the initial page load (see *Lazy wasm loading* below — `sq-4296`).
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
wasm-pack `--target web` output from `js/wasm/`, then `scripts/bundle-wasm-esm.mjs`
([`esbuild`](https://esbuild.github.io/)) bundles the `@jeswr/sparq` RDF/JS surface
(`js/src`) into a single self-hosted `public/wasm/sparq.js` (see below). The `prebuild`
script runs both automatically before `next build`.

### Lazy wasm loading — keeping the engine off the critical path — `sq-4296` (#935 / #981)

The sparq engine wasm is heavy, so it is kept **out of the initial page load** in three
layers — the page paints and becomes interactive before any of it has finished loading:

1. **The wasm binary is never bundled into JS.** `@sparq/client`'s `loadSparq()`
   dynamic-imports the wasm-pack glue with a `/* webpackIgnore: true */` hint, so the glue
   (`public/wasm/sparq_wasm.js`) is fetched as a plain ESM module at runtime and the
   `sparq_wasm_bg.wasm` binary is a static asset — neither enters a webpack chunk. (Verify:
   no `_next` JS chunk embeds the `.wasm`; the loader chunk only holds the *URL string*.)
2. **The REPL component is code-split.** `site/src/components/repl-lazy.tsx` wraps the heavy
   `Repl` in `next/dynamic(..., { ssr: false })` with a skeleton fallback, so the home page
   (`/`) and `/try` render their shell first and stream the REPL chunk in afterwards. The
   server pages render `<ReplLazy />` rather than `<Repl />`.
3. **The engine warm-up is deferred to browser idle.** Components call
   `prewarmSparqWhenIdle()` (not the eager `prewarmSparq()`), which schedules the
   fetch+instantiate via `requestIdleCallback` (with a `setTimeout` fallback) so the wasm
   loads *after* first paint. The first interaction (`Run query` / `Validate`) still
   `await`s the **memoised** `loadSparq()`, so it joins the in-flight load rather than
   re-paying a cold start — and never calls into an uninitialised wasm.

**Self-hosted ESM `<script type="module">` import — named `Dataset` (#981, `sq-55w5a`).**
`scripts/bundle-wasm-esm.mjs` bundles the `@jeswr/sparq` RDF/JS surface into a single
self-contained `public/wasm/sparq.js`, published into the static export — so the named
`Dataset` entry can be imported directly from this project's **own** GitHub Pages origin, with
**no third-party CDN**. The bundle keeps the ~MB engine `.wasm` **external** (it re-exports the
sibling wasm-pack glue), so it is fetched lazily by the first `await Dataset.…`, never by the
import line — the #981 lazy-load posture:

```html
<script type="module">
  import { Dataset, DataFactory as DF } from "https://sparq.jeswr.org/wasm/sparq.js";
  const ds = await Dataset.fromString('<a> <b> "x" .', "ntriples"); // wasm lazy-fetched HERE
  ds.add(DF.quad(DF.namedNode("a"), DF.namedNode("b"), DF.literal("y")));
  console.log(ds.size, ds.store.queryBoolean("ASK { ?s ?p ?o }"));
</script>
```

The same named entry is also available from an ESM CDN (the published `@jeswr/sparq` npm
package): `import { Dataset } from "https://esm.sh/@jeswr/sparq"`.

**Low-level glue (`Store`).** The wasm-pack `--target web` glue is itself a real ESM module, so
the engine `Store` class can be imported directly — instantiate it (its default `init`) before
use; the ~MB `.wasm` is fetched lazily by that init, not by the script tag:

```html
<script type="module">
  // basePath-aware: '/sparq/...' on GitHub Pages, '/...' under the Tauri webview root.
  import init, { Store } from "https://sparq.jeswr.org/wasm/sparq_wasm.js";
  await init(); // lazily fetches + instantiates sparq_wasm_bg.wasm (off the critical path)
  const store = Store.load("<a> <b> <c> .", "ntriples");
  console.log(store.query("SELECT * WHERE { ?s ?p ?o }"));
</script>
```

In an app, prefer `@sparq/client`'s `loadSparq()` (memoised, basePath-defaulted) over a
hand-rolled `init()` so the cold start happens at most once across the whole page.

### Build modes — `basePath` (Pages vs Tauri) — `sq-9vw5`

`next.config.ts` env-switches `basePath`/`assetPrefix` off `NEXT_PUBLIC_BASE_PATH` so the
**same** `out/` export serves two hosts. The `@sparq/client` wasm loader keys its runtime
asset URLs off the *same* env var, so the build-time route prefix and the runtime wasm-fetch
prefix stay in lockstep.

| Host | Command | `basePath` | When |
|---|---|---|---|
| **GitHub Pages** @ custom-domain root (production) | `cross-env NEXT_PUBLIC_BASE_PATH= npm run build` (the Pages workflow sets `NEXT_PUBLIC_BASE_PATH=''`) | `''` (root-relative) | served at `https://sparq.jeswr.org/` (org-migration cutover, `sq-uj38w`) — every asset/route is root-relative (`/_next/…`) |
| **Tauri 2 webview** | `npm run build:tauri` (= `cross-env NEXT_PUBLIC_BASE_PATH= npm run build`) | `''` (root-relative) | the desktop GUI serves the export from the `tauri://` root, where a `/sparq` prefix would 404 |
| **Legacy sub-path** (fallback) | `npm run build` (no env) | `/sparq` | the historical `jeswr.github.io/sparq/` sub-path; kept only as the unset-env fallback, not used in production |

The env var is read once in `next.config.ts`: **unset** keeps the historical `/sparq` sub-path as a
LEGACY fallback (no test/caller change); an explicit **empty string** selects the root-relative
export used by BOTH production Pages (custom domain) and Tauri; a malformed value falls back to the
legacy default. The `build:tauri` script uses [`cross-env`](https://www.npmjs.com/package/cross-env)
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

Coverage spans the critical site flows. **Critical-flow smoke** (bead sq-jp7ry, issue #835):
`home-smoke.spec.ts` asserts the home hero + primary nav boot with zero console errors;
`try-query-smoke.spec.ts` runs the trivial `SELECT * WHERE { ?s ?p ?o }` on the bundled
sample in the `/try` REPL and asserts a non-empty results table; `capabilities-smoke.spec.ts`
asserts the `/capabilities` showcase renders its hero, flagship band and all five theme
sections. **Regression guards:** `shacl-rerun-regression.spec.ts` (sq-jp7ry) drives the
`/surface/shacl` validator three times in one session and asserts no wasm object-lifecycle
fault (`__wbg_ptr` / "null pointer passed to rust" / "recursive use of an object") and a
fresh report each run — guarding the issue-#835 SHACL `__wbg_ptr` class; `repl-results.spec.ts`
and `shacl-validator.spec.ts` cover the REPL result panel and single-validate report. The
**ZK car-hire prover pre-warm** (`zk-prewarm.spec.ts`, sq-5q63) loads `/showcase/zk-car-hire`,
waits for the *Prover ready* pill, and asserts the first **Generate ZK proof** click pays no
cold start (observed via a test-only `window.__zkProverColdStarts` counter — pure observability).
The wasm-engine specs (`try-query`, `shacl-*`, `repl-results`) `test.skip` when the wasm bundle
is absent, so the **light** CI lane (no Rust toolchain) stays green; they run in full once
`npm run sync-wasm` has synced a `build:wasm` bundle. CI runs this lane on site-touching PRs
(`.github/workflows/site-e2e.yml`); Playwright outputs (`test-results/`, `playwright-report/`,
the browser cache) are git-ignored.

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
