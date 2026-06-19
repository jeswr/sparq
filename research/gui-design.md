# sparq GUI — design record (design-for-review)

Status: **design-only, for maintainer review.** Nothing in this document is built yet. It
is the deliverable of the open epic `sq-ixc3` ("EPIC: cross-platform GUI to interact with
the sparq engine — developed + maintained"). Its sibling epic is `sq-v286` (the
release-CI epic; deliverable `research/release-ci-design.md`, also not yet written). Both
are **open** in `.beads/issues.jsonl` and explicitly design-phase.

This record (1) recommends a concrete framework with justification, (2) defines an MVP
feature set plus later phases, (3) gives a maintenance / architecture plan (component
sharing with the site, CI, tests, releases, versioning), and (4) is explicit about scope:
every claim below is tagged **feasible-now** (the infrastructure or code already exists in
this repo) or **proposed** (design-for-review, not built).

## Honesty preamble (load-bearing, non-negotiable)

These constraints are inherited from the repo's standing honesty discipline and must
survive into any GUI that ships:

- **No performance numbers.** This work box is non-canonical (see `AGENTS.md` and the
  MEMORY hygiene rule). The only quantitative figure cited in this document is a static
  on-disk artifact size (`js/wasm/sparq_wasm_bg.wasm` = 2,583,006 bytes / ~2.58 MB),
  which is a build-output size, not a benchmark. GUI copy may show a live in-tab
  `performance.now()` latency for the query the user just ran (measured, labelled), but
  must never bake in a benchmark claim.
- **ZK and MPC are NOT externally audited.** The in-browser ZK proving path is real
  (`site/src/lib/zk-prover.ts` drives `@aztec/bb.js` UltraHonk), but the file itself
  records that the broader ZK estate is "research-grade, internally reviewed only, and NOT
  externally audited" (`site/src/lib/zk-prover.ts` header). The MPC surface on the site is
  a **faithful in-tab JS simulation**, explicitly "an ILLUSTRATION of the protocol SHAPE,
  not the hardened crate and NOT live MPC" (`site/src/lib/mpc-sim.ts` header), using
  `Math.random` rather than a CSPRNG. The native `sparq-mpc` crate is honest-majority
  semi-honest with a stubbed correctness-proof layer. The GUI must keep these caveats
  exactly as the site does today. The CI gate `scripts/check-privacy-claims.sh` enforces
  this on all user-facing copy.
- **The honesty-tier system is preserved verbatim.** Every surface in the site already
  declares which tier it runs in (`Tier` in `site/src/data/surfaces.ts:27-34`). The GUI
  inherits this taxonomy unchanged; a `walkthrough` surface must never be silently
  dressed up as `live`.

## 0. Ground truth — what exists today

A word-boundary grep for `tauri|electron|egui|iced` across `*.rs / *.toml / *.md / *.ts /
*.tsx` (minus `node_modules`) returns nothing, and there is no `uniffi` / `cargo-ndk` /
`xcframework` anywhere in the tree. So the GUI is **greenfield**: every framework and
architecture choice below is **proposed**, built on **feasible-now** primitives that
already ship.

### The reuse surface that already exists

The "site" (`site/`) is a real, maintained Next.js application — this is the largest asset
the GUI shares from:

- **Stack** (`site/package.json`, `site/next.config.ts`): Next.js with
  `output: "export"` (a fully static, backend-free `out/` tree for GitHub Pages), React 19,
  Tailwind v4, radix/shadcn UI primitives. A hardcoded `basePath: "/sparq"` +
  `assetPrefix: "/sparq"` + `images: { unoptimized: true }` (`site/next.config.ts:8-13`).
- **Components**: ~40 React components under `site/src/components/`, including the flagship
  `repl.tsx` (564 lines — a full SPARQL playground), the per-feature playgrounds
  (`shacl-playground.tsx`, `inference-playground.tsx`, `text-playground.tsx`,
  `rsp-playground.tsx`), the flagship demos (`zk-car-hire.tsx`, `mpc-demo.tsx`,
  `solid-pairs-demo.tsx`), and the captured walkthroughs (`http-server-walkthrough.tsx`
  and the vector/genai/geosparql walkthroughs), plus shared `components/ui/` primitives.
- **Engine-binding TypeScript** (`site/src/lib/`): `sparq-wasm.ts` is a production-grade,
  documented client wrapper around the WASM `Store` — `loadSparq()` / `prewarmSparq()`
  single-init (`site/src/lib/sparq-wasm.ts:243-271`), `loadIntoStore`, `storeToNQuads` /
  `datasetSize`, RDF/JS-style `matchQuads` / `countQuads`, `streamQueryRows` batched
  cursor consumer (`:201`), `sparqShaclValidate`, and SPARQL-JSON render types +
  `formatTerm` (`:343`). Siblings: `reason-wasm.ts`, `sparq-rsp-wasm.ts`,
  `sparq-text-wasm.ts`, `zk-prover.ts`, `mpc-sim.ts`, `http-server.ts`.

### The engine — two real embeddings that already ship

- **Native Rust**: `sparq-cli` (binary) and `sparq-server` (binary + rlib; axum/hyper/tokio
  HTTP) link the engine library crates (`sparq-core`, `sparq-engine`, …) directly, with
  `rayon` threads, `mmap`, and `save`/`open` persistence on natively.
- **WASM**: `crates/sparq-wasm` (`crate-type = ["cdylib","rlib"]`, `wasm-bindgen 0.2`),
  built by `js/`'s `wasm-pack build --target web` into the npm package `@jeswr/sparq`.
  The site default bundle is built `--features shacl,jsonld` (`js/package.json:37`). Four
  satellite WASM bundles exist (`crates/sparq-{reason,rsp,text,shacl}-wasm`) and prove the
  "one feature, one bundle" pattern.

### The constraints the WASM path imposes

Documented in `crates/sparq-wasm/src/lib.rs`: the wasm `Store` is **single-threaded** (no
rayon), has **no filesystem** (`save`/`open`/`mmap` unavailable), grows **append-only**
until rebuild, and lives under the browser tab's **~4 GiB / practically <2 GiB**
linear-memory ceiling. The doc-comment frames the wasm store as "read-replica shaped," not
a full-power primary store. The built artifact is **2,583,006 bytes (~2.58 MB)** for the
`shacl,jsonld` site build (the "~1.2 MB" note in `site/src/lib/sparq-wasm.ts:6` is the
lean, feature-less baseline, not what the site ships).

### The two structural gaps a static site cannot close

- **No live backend.** `/surface/http-server` is, by its own header in
  `site/src/lib/http-server.ts:3-8`, a captured curl/SSE transcript fallback — "the static
  GitHub-Pages site has no backend to talk to." The vector / genai / geosparql surfaces are
  `walkthrough` for the same reason, and MPC is `live-sim`. A desktop app with a real local
  engine is exactly what closes these.
- **No shared type source.** The WASM `Store` surface is declared once in `js/` (the
  `wasm-pack`-generated `sparq_wasm.d.ts`, copied by `js/package.json:36`) and
  **re-declared by hand** as the `WasmStore` interface in `site/src/lib/sparq-wasm.ts:36`.
  The site does not depend on `@jeswr/sparq` as a package — it runs its own
  `scripts/sync-wasm.mjs` (the `prebuild` hook, `site/package.json:12`) to copy
  `js/wasm/` → `site/public/wasm/`. So there are two copies of the engine's TS surface kept
  in sync manually. This is the single biggest long-term-maintenance liability today, and a
  GUI would become a third hand-copy unless it is fixed (see §4).

## 1. Framework recommendation

**Recommendation (proposed): Tauri 2.** On desktop, embed the engine as a **direct native
Rust link** (the app *is* a Rust binary, so it depends on `sparq-engine` / `sparq-core`
and the `sparq-server` rlib directly — not WASM, not HTTP). Reuse the existing Next.js /
React / Tailwind / radix frontend for the webview. Treat mobile (Android / iOS) and a thin
WASM-only web fallback as **separate, spike-gated, later tracks**.

This matches the lead candidate named in `sq-ixc3`'s own description.

### The three options, weighed

| Criterion | Tauri 2 (web UI + Rust backend) | egui / iced (pure-Rust native) | Packaged-web (Electron / PWA) |
|---|---|---|---|
| Reuse of the existing site | **High** — renders the same React 19 / Next.js / Tailwind / radix components in a system webview; the ~40 components + surface pages port largely as-is. | **None** — every screen (REPL editor, result tables, the surfaces, ZK/MPC demos) reimplemented in an immediate-mode/retained Rust UI. Largest rewrite. | High (Electron) to total (PWA = literally the static site). |
| Engine embedding | **Direct native Rust link** — full `rayon`, `mmap`, `save`/`open`, all opt-in crates; no WASM ceiling, no HTTP hop. | Direct native Rust link (same engine strength); embedding is trivial because the app is Rust — the cost is the UI. | Electron bundles the WASM engine (inherits the <2 GiB / no-mmap / single-thread limits) or shells out to a sidecar `sparq-server` over HTTP. PWA = WASM-only. |
| Bundle size | Small — no bundled Chromium (uses OS WebView2 / WKWebView / WebKitGTK). | Smallest (single Rust binary). | **Largest** (Electron ships Chromium). PWA ships nothing extra but is not a "real" app. |
| Native feel | Good — native window/menus/tray/FS dialogs; UI is HTML (matches the site). | **Best** raw native, but our UI vocabulary (code-editor, rich result grids, the Noir JS ZK tooling) is web-shaped; rebuilding it native is high-effort, lower-fidelity. | Electron good; PWA constrained by browser chrome. |
| Mobile (Android / iOS) | **Yes** — Tauri 2 added mobile targets; same webview UI; the engine on mobile is the existing native Rust library cross-compiled (NDK / xcframework). Proposed, not started. | iced/egui mobile is immature/experimental — weakest mobile story. | Electron: **none**. PWA: installable but WASM-bound, no native integration. |
| Maintenance burden | **Lowest marginal** — one React codebase serves web + desktop + mobile. Adds a Tauri shell + IPC command layer. | **Highest** — a second, parallel UI codebase forever diverging from the site (against the repo's anti-duplication discipline). | Electron: heavy Chromium-CVE update treadmill. PWA: low, but not the deliverable. |
| Ties into `sq-v286` | **Directly** — `release.yml` already builds per-tier native binaries on `macos-14` / `macos-15-intel` / `ubuntu-latest` / `ubuntu-24.04-arm` / `windows-latest` with SLSA provenance + SBOM/VEX. Tauri's bundler output is "native binary + signed assets," which is what that pipeline is shaped for. | Same native binaries would work, but you ship a worse UI. | Electron's Chromium bundle fights the "Rust native binary + provenance" model `release.yml` is built around. |

### Why Tauri 2, concretely

1. **It maximises the largest existing asset.** The site is a substantial, current React 19
   codebase (~40 components, the surface pages, the ZK tooling). Tauri renders that same
   frontend in a webview; egui/iced throws all of `site/src/components/**` away. The
   alternative is maintaining two UIs forever — which the repo's hygiene discipline is
   explicitly against.
2. **It unlocks the full engine, not the WASM-limited one.** Because a Tauri app's backend
   *is* a Rust binary, you link `sparq-engine` / `sparq-core` (and the `sparq-server` rlib)
   directly. That escapes every WASM constraint documented in
   `crates/sparq-wasm/src/lib.rs`: `rayon` parallel scan/parse, `mmap`-backed dictionaries,
   `save`/`open` persistence, and no ~2 GiB tab ceiling. The desktop app becomes the
   *primary* store the WASM tier admits it is not. Embedding preference, in order:
   **direct Rust link > WASM > HTTP-to-`sparq-server`**. (HTTP-to-sidecar remains a viable
   fallback if process isolation is wanted, but it adds a localhost server + serialization
   hop for no functional gain over a direct link.)
3. **It closes the real gap on the static site.** `/surface/http-server` is a recorded
   transcript today because Pages has no backend. A desktop app ships a real local engine,
   so those surfaces can become live.
4. **It is the only credible mobile path here.** `sq-v286` wants Android / iOS. Tauri 2
   supports mobile with the same UI; iced/egui mobile is experimental; Electron has none.
   The mobile engine artifact is the existing native Rust library cross-compiled
   (NDK / xcframework) — exactly the "library, not app" binding work `sq-v286` already
   scopes ("uniffi / cargo-ndk / xcframework"). **This is proposed/greenfield**: no
   `uniffi` / `cargo-ndk` exists in the repo yet, and `release.yml`'s matrix has no
   Android/iOS rows today.
5. **It rides the CI already built.** `release.yml` builds the desktop targets the GUI
   needs and attests them (SLSA provenance, CycloneDX SBOM + VEX, `cargo-auditable`,
   `SHA256SUMS`). A GUI bundle added to that matrix inherits all of it.

### Honest caveats on the framework choice

- **Static-export friction.** The site hardcodes `basePath: "/sparq"` for Pages. A Tauri
  build loads assets from `tauri://` / `file://`, so the `basePath` / `assetPrefix` must be
  made conditional (env-switched). Minor but real config work, not free.
- **Next.js in a webview.** `output: "export"` already produces a backend-free static
  bundle (good — Tauri needs no Node server). The current site is fully static, so any
  future feature relying on a Next server runtime must stay client-side.
- **Mobile is the riskiest leg.** Tauri 2 mobile is newer than its desktop story;
  cross-compiling the engine (with its `rayon` / native deps and the opt-in crates) to
  Android/iOS is unproven *in this repo* and may force a reduced feature set per target.
  Validate with a spike before committing.

## 2. Feature set — MVP and later phases

The GUI is **not greenfield in features**: the site already has a genuinely-live in-tab
SPARQL engine and ~20 showcase surfaces. The job is consolidation into a credible
*workbench* plus the editor uplift, not building a playground from zero.

### Engine capability → GUI feature → current tier

| Engine capability (real surface) | GUI feature | Current tier / state |
|---|---|---|
| SPARQL 1.1/1.2 SELECT/ASK/CONSTRUCT/DESCRIBE/UPDATE, paths, RDF-star, EXPLAIN/ANALYZE (lean wasm) | Query editor + results + plan view | **`live`** — `repl.tsx` dispatches all forms + EXPLAIN/ANALYZE via `sparq-wasm.ts` (`query`, `queryQuads`, `updateInPlace`, `explain`, `explainAnalyze`, `queryCursor`, `count`, `ask`, `applyDelta`) |
| Ingest: Turtle/N-Triples/N-Quads/TriG/JSON-LD (+ compressed) | Upload / paste / URL-load, named-graph-preserving | **`live`** — `repl-datasets.tsx` + `loadIntoStore`. HDT is native-only (`skills/hdt-format`), not in the wasm bundle |
| SHACL Core + SHACL-SPARQL → W3C report | Shapes editor + report with `sh:detail` | **`live`** — `validate` (gated `--features shacl`), `shacl-playground.tsx` |
| RDFS / OWL-RL / N3 closure + proof trees | Materialize + proof-tree view | **`live-new-wasm`** — `inference-playground.tsx`, `reason-wasm.ts` |
| BM25 full-text via magic predicates | Search box + ranked results | **`live-new-wasm`** — `text-playground.tsx`, `sparq-text-wasm.ts` |
| RSP-QL streaming windows | Window config + R/I/DSTREAM tick view | **`live-new-wasm`** — `rsp-playground.tsx`, `sparq-rsp-wasm.ts` |
| Vector kNN (`vec:nearest`) | Embedding viz + kNN results | **`walkthrough`** — `sparq-vectors` opt-in native, not in the lean wasm; captured output today |
| GeoSPARQL (`geof:`, R-tree, topology) | Map overlay + spatial-join results | **`walkthrough`** — `sparq-geo` native-only, server `geo` feature off by default |
| GenAI / NLQ (schema-card / VoID → NL→SPARQL) | Schema card + NL prompt → generated SPARQL | **`walkthrough`** — `sparq-nlq` / `sparq-introspect` native, needs a model backend |
| ZK query proofs (BGP+FILTER, attestation, revocation) | Commit → prove → verify, in-tab | **`live-bbjs`** — `zk-car-hire.tsx`, `zk-prover.ts` via `@noir-lang/noir_js` + `@aztec/bb.js` UltraHonk. **Not externally audited** |
| MPC federation (additive sharing) | Multi-party threshold demo | **`live-sim`** — `mpc-demo.tsx`, `mpc-sim.ts` — a faithful JS illustration, **not** the native `sparq-mpc` protocol, no CSPRNG, **not** live MPC |
| Access control (Solid WAC/ACP) FROM NAMED restriction | (user, app)-pair → different result sets | **`live`** (engine restriction) — `solid-pairs-demo.tsx`; ACP decision materialized at build time |
| HTTP server: SPARQL Protocol, GSP, `/metrics`, WS+SSE subscriptions, VoID/Service Description | Connect-to-endpoint mode + live subscription stream | **`walkthrough`** — static Pages has no backend; real endpoints exist in `crates/sparq-server` but aren't hosted |
| Federation client (TPF/brTPF/SPARQL, pushdown) | Multi-source federation plan + per-leaf fan-out | **`soon`** — `sparq-fedclient` native, feature-gated; no GUI surface yet |
| Usage-control policy (ODRL) conflict/containment | Policy editor + conflict/refinement view | **`soon`** — `sparq-policy` (`contains` / `detect_conflicts`); no GUI |
| PROV-O lineage of derived data | Lineage graph of a CONSTRUCT derivation | **`soon`** — `sparq-prov` (`derive_construct`); no GUI |

The honest pattern: everything in the **lean wasm bundle** (core SPARQL, data formats,
SHACL) is `live`. Everything that is an opt-in **native crate not compiled to wasm**
(vector, geo, nlq, fedclient, policy, prov, native MPC) is `walkthrough` / `live-sim` /
`soon`, because the GitHub Pages host has no backend and those crates aren't in `js/wasm`.

### MVP — a credible workbench around the existing live engine (mostly `live`, no backend)

The MVP is **consolidation + the editor uplift**, almost entirely `live` tier:

1. **Query editor uplift** (feasible-now; the single biggest real UX gap). The current
   editor is a plain `<textarea>` (`repl.tsx:327`). MVP: syntax highlighting, prefix
   awareness, keyword/example completion (a SPARQL mode on a client-side code editor such
   as CodeMirror 6). No engine change. Highest-leverage GUI improvement.
2. **Results: table + raw SPARQL-JSON + N-Triples, plus CSV/TSV export** (feasible-now,
   `live`). Table + N-Triples already render (`ResultPanel`, `repl.tsx:473`); add a raw
   SPARQL-JSON view (the wasm already returns that document) and a CSV/TSV export.
3. **Dataset load / manage panel** (feasible-now, `live`). Already present
   (`repl-datasets.tsx`: built-in picker, upload, URL, merge, dataset viewer). MVP polish:
   named-graph list with per-graph triple counts. HDT upload stays out of scope
   (native-only).
4. **Keep the showcase surfaces exactly as tiered** (the honesty discipline). SHACL /
   inference / full-text / RSP stay `live` / `live-new-wasm`; ZK stays `live-bbjs`; MPC /
   Solid stay `live-sim` / `live`. **Never silently upgrade a `walkthrough` to look live.**
5. **The Tauri desktop shell itself** (proposed, the gating MVP work). Wrap the existing
   frontend in a Tauri 2 webview with the conditional-`basePath` config change, and link
   the native engine for a first "real local store" command path (load file from disk via
   `save`/`open`, run a query) — proving the direct-Rust-link embedding end to end.

### Phase 2 — connect to a running `sparq-server` (closes the live-backend gap; proposed)

The server API already exists (`crates/sparq-server/src/http.rs:2046-2102`): `/sparql`
(GET + both POST forms), `/sparql/graph` + `/graphs/{*path}` (Graph Store read **and**
write), `/subscriptions` (WS) + `/subscriptions/sse` (SSE), `/health`, `/metrics`,
`/admin/compact`, and the opt-in `/.well-known/void`, `/tpf`, `/shacl/validate`. The GUI
work is a client + auth/CORS UX, not new engine work.

6. **Endpoint mode** — a "Connect" panel: endpoint URL, optional bearer token, run the
   *same* editor against any SPARQL 1.1 Protocol endpoint. The server already supports a
   constant-time Bearer **write gate** (`--auth-token`) and optional **read gate**
   (`--auth-token-read`); on a WebSocket handshake (where browsers can't set
   `Authorization`) it accepts `Sec-WebSocket-Protocol: bearer.<token>`. Unlocks persistent
   datasets, GSP graph management, and the server-only `geo` / `vec-predicate` features
   without a wasm port.
7. **Live subscriptions view** — consume `/subscriptions/sse` (or WS) and stream result
   deltas as the dataset mutates. This is the standout "live" demo a static site
   fundamentally cannot do.
8. **`/metrics` + Service Description panel** — render the Prometheus `/metrics` and the
   VoID / SPARQL Service Description as a "server health / capabilities" view.

The GUI must respect the server's security posture (`crates/sparq-server/README.md`): **no
auth by default** (anyone reaching the port reads and writes), loopback bind by default
(non-loopback refused unless the auth×bind matrix is satisfied, `--allow-remote`), and a
**strict SERVICE-egress allowlist** that refuses all federation before any network call
when empty (the default). The GUI surfaces these as connection-safety UX, never bypasses
them.

### Phase 3 — visualisation + heavier showcases going live (proposed)

9. **Graph/triple visualisation of results** (`live`). CONSTRUCT/DESCRIBE already returns
   N-Triples in-tab; render it as a node-link graph. Pure client-side over existing output.
10. **Vector + Geo go `live-new-wasm`**. Portability spikes to add `sparq-vectors` /
    `sparq-geo` to a wasm bundle (the surfaces' own tier comments name this as the blocker),
    upgrading kNN viz and the GeoSPARQL map overlay from `walkthrough` to live. Follows the
    proven `sparq-{shacl,reason,text,rsp}-wasm` pattern.
11. **Policy / PROV / Federation views** (`soon` today). An ODRL policy editor + conflict /
    containment view over `sparq-policy`; a PROV-O lineage graph over
    `sparq-prov::derive_construct`; a federation-plan visualiser over `sparq-fedclient`.
    Each needs either a wasm port or endpoint mode first.

### Explicitly out of scope / stays caveated

- **ZK and MPC are not externally audited** — the GUI keeps the existing caveats. ZK
  proving is real (`live-bbjs`, in-tab UltraHonk) but circuits/soundness are not
  third-party audited; MPC on the site is a **JS simulation** (`live-sim`), not the native
  protocol. Never present either as production-grade.
- **No hard-coded performance numbers** in any GUI copy. A live in-tab `performance.now()`
  latency for the query just run is fine (measured, labelled); benchmark *claims* may only
  come from the repo's existing generated benchmark data.

## 3. Architecture & maintenance plan

### Monorepo structure (proposed) — extend, don't fork

```text
packages/sparq-client/   # NEW shared TS pkg: the ONE WasmStore type + loaders
js/                      # @jeswr/sparq — npm wrapper; consumes packages/sparq-client
site/                    # Next.js site — consumes packages/sparq-client
gui/                     # NEW Tauri 2 app: frontend consumes packages/sparq-client;
                         #   gui/src-tauri/ is the Rust shell linking sparq-engine
```

The shared package's job is to **kill the hand-redeclared `WasmStore`** (§0): `js/`
generates the wasm `.d.ts`; `packages/sparq-client` re-exports it plus the loaders; both
`site/` and `gui/` import from it. This collapses today's 2-copy manual sync into one
source and makes the GUI a **zero-new-copy** consumer rather than a third hand-copy. The
npm `@jeswr/sparq` package already ships a handwritten `dist/index.d.ts`, so a shared TS
package is consistent with existing direction.

**Tooling caveat (real, not free).** `site/` and `js/` are independent npm roots today —
there is no root `package.json` and no workspaces field; `site/` has its own
`package-lock.json`. Introducing `packages/` means adopting npm (or pnpm) workspaces at a
new repo-root `package.json` — a reviewable change to the JS build topology, called out
here so the maintainer signs off.

### Engine embedding (proposed)

- **Desktop**: `gui/src-tauri/` is a Rust crate that depends on the engine library crates
  directly (and the `sparq-server` rlib if endpoint-mode wants an in-process server). A
  Tauri IPC command layer bridges the webview UI to the engine: load/query/update/explain
  map onto the same operations the WASM `Store` exposes, but backed by the full native
  store (threads, mmap, persistence). The GUI Rust crate should inherit the workspace's
  `forbid(unsafe_code)` posture (see the `unsafe-rust-attestation` skill) and the
  clippy-`-D warnings` discipline.
- **Mobile** (later track): the same native library cross-compiled via NDK (Android) and an
  xcframework (iOS) — net-new `uniffi` / `cargo-ndk` work that does not exist in the repo
  yet; spike-gated, possibly with a reduced per-target feature set.
- **Web fallback** (optional): the existing WASM bundle, inheriting the documented
  single-thread / no-mmap / <2 GiB ceilings — i.e. exactly the site as it is today.

### CI for the GUI (proposed, modelled on existing lanes)

The repo already has the exact patterns to clone. The GUI lane (`gui.yml`) should be
**path-scoped, per-platform, and NOT a merge-queue lane**, mirroring `site-e2e.yml` and
`js.yml`:

- **Path scoping + triggers**: copy `site-e2e.yml`'s shape —
  `on: pull_request: paths: ["gui/**", ".github/workflows/gui.yml"]` plus
  `push: branches:[main]` with the same paths. As the `site-e2e.yml:8-14` comment
  documents, a Rust-/docs-only PR then skips the lane and the required `ci-summary / gate`
  "simply never waits on it" (cross-workflow `needs:` is impossible). The GUI lane stays
  off `merge_group` exactly as `site-e2e` / `js` / `pages` already are.
- **Per-platform build matrix**: build / lint / typecheck / e2e on `macos`, `windows`,
  `ubuntu`. Reuse the exact runner labels already proven in `release.yml`
  (`macos-14`, `macos-15-intel`, `ubuntu-latest`, `ubuntu-24.04-arm`, `windows-latest`) so
  the GUI CI matrix and the release matrix can't drift.
- **Build + lint + typecheck**: `next lint` + `tsc` are already the site's gate
  (`site/package.json:15`); the GUI adds `cargo build` / `cargo clippy` for
  `gui/src-tauri/`, inheriting the repo's `-D warnings` bar.
- **E2E**: Playwright is already wired for the site (`@playwright/test`,
  `site/playwright.config.ts`, the real `site/e2e/zk-prewarm.spec.ts`). Tauri's WebView e2e
  uses `tauri-driver` (WebDriver), not Playwright's Chromium — so the GUI e2e lane is a
  **new** harness, not a copy. Per-platform e2e is expensive; scope it path-scoped with
  `cancel-in-progress`, like `site-e2e.yml`.
- **Least-privilege**: every existing JS/site lane sets `permissions: contents: read` for
  the Scorecard TokenPermissions check; the GUI lane must do the same.

**Required-gate wiring.** `ci-summary.yml` is "the SINGLE required status check for branch
protection on `main`" and explicitly does **not** require the site lanes by name (it can't
— cross-workflow `needs:` is impossible, per its header). So the GUI lane, like
`site-e2e` / `js` / `pages` today, will be **advisory** unless the maintainer adds it to the
branch-protection required-contexts list — a `needs:user` repo-settings action.

### Releases (proposed, riding `sq-v286`)

The release plumbing the GUI rides already exists and is sophisticated; the GUI extends the
matrix rather than inventing a pipeline:

- **Trigger**: `release.yml` fires on `push: tags: ["v*"]` (releasing is a deliberate act).
  `sq-v286` proposes automating the tag via conventional-commits / semantic-release; the
  manual `v*` tag → full release already works today.
- **Matrix to extend**: `release.yml`'s matrix already builds the **exact desktop targets**
  the GUI needs (arm64/x64 darwin, x64 baseline/v2/v3/v4 + arm64 linux, win-x64/arm64). The
  GUI's desktop bundles (`.dmg` / `.msi` / AppImage/`.deb` / etc.) are *additional*
  artifacts on those same rows. **Honest gap**: that matrix has **no Android/iOS rows**
  today — mobile GUI artifacts are net-new, consistent with `sq-v286` saying a mobile app
  artifact "requires the GUI epic."
- **Supply-chain evidence rides for free**: every released artifact already gets a SLSA
  build-provenance attestation (`actions/attest-build-provenance`), a per-release CycloneDX
  SBOM + VEX (`scripts/gen-sbom-vex.sh`), `cargo-auditable` embedding, and `SHA256SUMS`
  coverage. A GUI bundle added to the `subject-path` inherits all of it (the
  `rust-supply-chain-attestation` skill documents this estate).
- **Code-signing is the real blocker, and it is `needs:user`.** `release.yml` attests
  provenance but does **not** sign binaries. Apple notarization, Windows Authenticode, and
  the Android keystore all require user-provided credentials (per `sq-v286`). Unsigned CI
  builds are fine for testing; a shippable, un-quarantined GUI is gated on the maintainer
  supplying signing creds. This cannot be designed away.

### Versioning in lockstep with the engine (feasible-now foundation + proposed policy)

- **Foundation exists**: the workspace is single-version —
  `[workspace.package] version = "0.1.0"` (`Cargo.toml:18`), and `crates/sparq-wasm`
  uses `version.workspace = true`. `@jeswr/sparq` (`js/package.json`) and the site are also
  at `0.1.0`. Lockstep is the current de-facto model.
- **Proposed policy**: the `gui/src-tauri` crate inherits the workspace version via
  `version.workspace = true` (exactly like `sparq-wasm`), and the GUI's `package.json`
  version is stamped from the same source the release tag drives. `sq-v286`'s scope item —
  Rust-workspace versioning via `release-plz` / `cargo-release` — is the tool that would
  bump `workspace.package.version` and the JS `package.json`s together from
  conventional-commit history.
- **Honest tension**: with one workspace version, a breaking change in any crate bumps
  everything, including the GUI. That's the intended lockstep, but it means GUI releases are
  not independent — a deliberate trade for the maintainer to sign off in
  `research/release-ci-design.md`.

## 4. The Pages-root blocker that predates the GUI

`sq-svtt` is an **open, `needs:user`** product decision the GUI work must not trample:
GitHub Pages has one deploy slot, currently held by the showcase (`pages.yml`, live at
`/sparq/` + the bench dashboard at `/sparq/dev/`), and the mdBook-guide design (`sq-w9sr`)
also wants root. A GUI that wants its own web-hosted demo or download page is a *fourth*
claimant on that single slot. **Resolve `sq-svtt` first**; this design references it and
does not re-open it.

## 5. Bottom line (verdict)

- **Feasible-now (infra exists):** the per-platform CI patterns (`site-e2e.yml` / `js.yml`),
  the desktop build matrix + full supply-chain attestation (`release.yml`), single-workspace
  versioning (`Cargo.toml` + `version.workspace = true`), and a real reusable React/WASM
  frontend (`site/src/`) — the GUI rides all of these with extensions, not rewrites.
- **Proposed (design-for-review):** Tauri 2 with a direct native-Rust engine link on
  desktop; an npm-workspaces monorepo with a `packages/sparq-client` shared-TS package to
  eliminate the current hand-redeclared `WasmStore` drift (the single biggest long-term
  liability today); a path-scoped non-merge-queue `gui.yml` matrix lane; a `tauri-driver`
  e2e harness (new, not Playwright-reusable); GUI bundles added to `release.yml`'s
  `subject-path`; lockstep versioning via `release-plz` / `cargo-release`.
- **Hard `needs:user` blockers (cannot be designed away):** Apple notarization / Windows
  Authenticode / Android keystore credentials for a *distributable* GUI; the `sq-svtt`
  Pages-root decision; and branch-protection required-contexts if the GUI lane is to be a
  *hard* merge gate. Mobile (Android/iOS) GUI artifacts are net-new — no rows exist in the
  `release.yml` matrix today.
- **Caveats:** the ZK/MPC surfaces this GUI would expose (`zk-car-hire.tsx`, `mpc-demo.tsx`)
  are **not externally audited** — the GUI inherits the existing "what runs where" honesty
  framing, never presenting them as production-trust. No performance numbers are asserted
  (this work box is non-canonical).

## Key file citations

- Epics: `sq-ixc3`, `sq-v286`, `sq-svtt`, `sq-w9sr` (`.beads/issues.jsonl`).
- Site: `site/package.json`, `site/next.config.ts`, `site/src/components/repl.tsx`,
  `site/src/components/repl-datasets.tsx`,
  `site/src/lib/{sparq-wasm.ts,zk-prover.ts,mpc-sim.ts,http-server.ts}`,
  `site/src/data/surfaces.ts`, `site/README.md`, `research/feature-showcase-site-design.md`.
- Engine / WASM: `crates/sparq-wasm/{src/lib.rs,Cargo.toml}`,
  `crates/sparq-{reason,rsp,text,shacl}-wasm/src/lib.rs`, `js/package.json`,
  `js/wasm/sparq_wasm_bg.wasm`.
- Server: `crates/sparq-server/src/{http.rs,negotiate.rs,service_config.rs}`,
  `crates/sparq-server/README.md`.
- CI / release: `.github/workflows/{site-e2e.yml,js.yml,pages.yml,release.yml,ci-summary.yml}`,
  `scripts/gen-sbom-vex.sh`.
- Workspace: `Cargo.toml`.
- Opt-in crates: `crates/sparq-{vectors,geo,nlq,fedclient,policy,prov,mpc,introspect}/`.
