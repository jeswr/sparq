# GUI consolidation fix plan — diagnosis + decomposition for epic sq-2ucrz

> [FABLE] Design record for the maintainer's 2026-07-05 GUI directive (epic `sq-2ucrz`):
> *"fix (1) being able to switch workspaces (2) N3 inference not being available (3) most of
> the tabs not working — at least in the browser (4) there is no syntax highlighting for
> SHACL. There should also be a way of uploading many shape files into the GUI. Same with N3
> rules. In the browser I would like to be able to drag and drop or upload files from disk."*
> The site `/try` removal is dispatched separately (`sq-4hiqe`) and is **out of scope** here.

This record diagnoses each reported bug against the actual `gui/` code (every mechanism below
was verified by reading the source, not taken from the epic's framing), then decomposes the
fix into 15 disjoint child beads the implementation fleet can drain. Prior records:
`research/gui-design.md` (the workbench architecture this app implements) and
`research/web-gui-test-program.md` (the deterministic e2e doctrine the acceptance tests extend).

## 1. Diagnosis — the actual mechanism behind each report

### 1.1 Workspace switching (bug 1): model complete, restore + UI never built

- The framework-agnostic workspace model (`packages/sparq-client/src/workspace.ts`, sq-atb0)
  is **complete**: `WorkspaceStore.list/load/save/remove/lastOpenedId`, a
  `WorkspaceSummary` designed *for a switcher list*, and three runtime-selected backends
  (Tauri fs / `localStorage` / memory).
- The GUI never consumes it beyond a single auto-created workspace:
  - the left rail's workspace button (`left-rail.tsx`) is a **dead stub** (`title="…the
    picker is a later phase (sq-atb0)"`);
  - **worse, restore is broken**: imports persist a whole-dataset `dataSnapshot`
    (`workspace-context.tsx → recordImport`), but *nothing ever reads it back* —
    `engine-context.tsx` unconditionally seeds `SAMPLE_TURTLE` on warm
    (`Store.load(SAMPLE_TURTLE, …)`), and `grep dataSnapshot gui/app/src` finds **no
    consumer**. A user's imported data is silently replaced by the sample graph on every
    relaunch;
  - the persisted editor state (`workspace.editor.query`) is likewise never restored into
    the Query tool.
- **Verdict:** missing feature (switcher UI) **plus** a real data-loss restore bug. Fix =
  plumbing bead (`sq-lcd6e`, hydrate/switch APIs + snapshot/editor restore) then UI bead
  (`sq-lmlch`).

### 1.2 N3 inference (bug 2): the reasoner ships it; the GUI never calls it

- The tier-b W-reason wasm bundle **already exports** `reasonN3(n3) → entailed ground
  N-Triples` (`crates/sparq-reason-wasm/src/lib.rs`), and it is even *typed* in the GUI's
  own loader interface (`gui/app/src/lib/reason-wasm.ts → WasmReasoner.reasonN3`) — but no
  GUI code path invokes it.
- The native N3 engine (`crates/sparq-reason/src/n3/`) is full-featured: forward chaining
  with EYE/cwm parity across the W3C N3 community-group suite, `math:`/`string:`/`list:`/
  `time:`/`log:` builtins, backward rules. This is not a thin stub being dressed up.
- The Inference tool's UI caveat ("N3 … needs a store that can hold rule/formula terms") is
  half-right: the in-tab store is ground-triples-only, so N3 cannot be a *store* format —
  but it **can** be a query-time regime exactly like RDFS/OWL RL: serialise the base store
  to N-Triples (a valid N3 subset; named graphs folded as the other regimes already do),
  concatenate the workspace's rules documents, call `reasonN3`, and load base + derived
  ground triples into the closure store. Formula-/list-valued conclusions cannot be
  represented and are dropped with a visible count (honest scope note in the tool).
- **Verdict:** unexposed capability. Fix = `sq-glo5r` (an `"n3"` regime + per-workspace
  rules documents + bulk rules upload), sequenced after the workspace plumbing because both
  edit `engine-context.tsx`/`workspace-context.tsx`.

### 1.3 "Most of the tabs not working — at least in the browser" (bug 3): the tool matrix

All 11 tools were classified by reading `tools.ts`, `tool-panel.tsx`, and each panel:

| Tool | Panel today | Browser (/app) | Desktop (Tauri) | Classification |
| --- | --- | --- | --- | --- |
| Query | `QueryWorkbench` | live in-tab wasm | live in-tab wasm | **working** |
| SHACL | `ShaclTool` | live (bundle has `shacl`) | live | **working** (no highlighting; single pasted doc) |
| Inference | `InferenceTool` | live RDFS/OWL RL (reason bundle IS built + synced by `pages.yml`) | live | **working** (N3 unexposed) |
| Graph view | `ToolStub` | — | — | **stub**, but the viz **already shipped** as a Query result view (`graph-view.tsx`, sq-ixc3.12); `built:false` is metadata drift noted at sq-lyp8 close |
| Full-text | `ToolStub` | — | — | **stub, feasible now**: `crates/sparq-text-wasm` + `js` `build:text-wasm` exist, bundle never synced/wired |
| Streaming | `ToolStub` | — | — | **stub, feasible now**: `crates/sparq-rsp-wasm` + `build:rsp-wasm` exist, never synced/wired |
| Server | `ToolStub` | — | — | **stub, feasible now**: the SPARQL 1.1 Protocol client core already lives in `@sparq/client` (`endpoint.ts`, `server-health.ts`, `subscriptions.ts`, built for the site's `/try` endpoint mode — and `/try` is being removed by sq-4hiqe, so the GUI becomes this capability's home) |
| Vector | `ToolStub` | — | walkthrough | **stub, genuinely deferred** (native-only crate; portability spike sq-zeai) |
| GeoSPARQL | `ToolStub` | — | walkthrough | **stub, genuinely deferred** (same sq-zeai lane) |
| ZK proofs | `ToolStub` | — | — | **stub, deferred out of this epic** — the in-tab proving path is research-grade; the ZK estate's external accredited-cryptographer sign-off is still pending (sq-qhy4), and the stub's caveat text must stay |
| MPC | `ToolStub` | — | — | **stub, deferred out of this epic** — would be an in-tab illustration only; `sparq-mpc` itself is honest-majority **semi-honest only**, and the stub says so |

**Counts:** 3 working / 8 stubs, of which 4 are feasible-now wiring gaps (graph-view,
full-text, streaming, server) and 4 are genuinely deferred. So "most tabs not working" is
accurate as experienced — but the mechanism is *honest placeholders for capabilities whose
building blocks mostly already exist*, not browser-specific breakage.

There **is** a genuine browser-specific gap the maintainer's "at least in the browser"
points at, and CI structurally cannot see it: the Playwright lane's auto-fixture
(`gui/e2e-playwright/support/fixtures.ts`) injects `window.__TAURI__` into **every** spec,
so `isTauriRuntime()` returns `true` and only the *desktop* persona is ever tested. The
pure-browser persona (the deployed `/app`) has zero CI coverage. `inference-toggle.spec.ts`
is green (gui.yml is green on main) but deliberately accepts *either* `ready` *or* `error`
for the reasoner chip, and runs under the desktop mock. Fix = `sq-u0my5` (a no-mock
`chromium-web` project) so browser-parity claims become testable, plus the four tool beads.

### 1.4 SHACL highlighting (bug 4) + uploads: all building blocks exist

- `shacl-tool.tsx` uses a **plain `<textarea>`**; but the repo already has (a) the
  dependency-free overlay-highlight editor pattern (`sparql-editor.tsx`: transparent-text
  textarea over a highlighted `<pre>`) and (b) `tokenizeTurtle` exported from
  `@sparq/client` sharing the same token classes/CSS palette. SHACL shapes are Turtle, so
  highlighting is component reuse (`sq-5sjub` + `sq-txrui`), not new tokenizer work.
- **Uploads:** there is no drag-and-drop anywhere, no browser file input, and the native
  dialog is single-file (`tauri-ipc.ts → pickRdfFile … multiple: false`). The import
  drawer's File tab is hard-disabled in browsers ("Desktop app only") — over-restrictive: a
  browser cannot read an arbitrary *disk path*, but a user-picked/dropped `File` object is
  perfectly readable via `File.text()` into the in-tab wasm parse. Multi-file import needs
  **no engine change**: `importRdf` with `mode:"add"` already merges, so a loop over files
  suffices. Fix = one shared ingest lib (`sq-vnh1v`) consumed by RDF import (`sq-eydh9`),
  SHACL shapes (`sq-txrui`), and N3 rules (`sq-glo5r`).

## 2. Options considered

1. **Delete the stub tools instead of filling them.** Rejected: four of the eight stubs are
   wiring gaps over already-built engines/bundles, and the maintainer asked for working
   tabs. The four genuinely-deferred stubs stay (honest placeholders), but get regrouped
   into a collapsed "Coming soon" rail section (`sq-1odi9`) so they stop dominating.
2. **CodeMirror/Monaco for SHACL highlighting.** Rejected: the repo's editor doctrine is
   dependency-free overlay highlighting (no bundle-size hit, static-export-safe), and the
   Turtle tokenizer already exists. Adding an editor framework for one textarea is
   disproportionate.
3. **N3 as a first-class store regime (rule/formula terms in the store).** Rejected for
   this epic: the in-tab store is ground-triples-only. The query-time closure via
   `reasonN3` gives the maintainer working N3 inference now with an honest ground-triples
   scope note; a formula-capable store is a separate engine question.
4. **One big "fix the GUI" bead.** Rejected: it serialises a fleet that can run six beads
   wide, and the shared-file conflicts are avoidable with two small seam beads
   (tool-panel registry `sq-5lyme`; shared ingest lib `sq-vnh1v`).

## 3. Chosen decomposition — 15 disjoint child beads

File-level disjointness is the hard invariant: no two *parallel* beads touch the same file.
The two genuinely-coupled pairs are sequenced with `bd dep` edges instead (marked below).
Each bead carries `{files, model_tier, invariant, acceptance_test}` in its `bd` record.

### Phase 0 — seams and enablers (parallel)

| Bead | Tier | Files (exclusive) | What |
| --- | --- | --- | --- |
| `sq-5lyme` | sonnet | `tool-panel.tsx`, `tools.ts`, NEW `{graph-view,full-text,streaming,server}-tool.tsx` | Panel **registry** + one placeholder file per upcoming tool, so every later tool bead edits only its own file (incl. per-tool tier overrides + a `group` field) |
| `sq-vnh1v` | sonnet | NEW `lib/file-ingest.ts`, NEW `dropzone.tsx` | Shared multi-file picker (File System Access where available, `input[multiple]` fallback) + drag-and-drop reader; browser-parity floor |
| `sq-5sjub` | haiku | NEW `turtle-editor.tsx` | Overlay-highlight Turtle/N3 editor (mirror of `sparql-editor.tsx`, `tokenizeTurtle`) |
| `sq-u0my5` | sonnet | `playwright.config.ts`, NEW `support/web-fixtures.ts`, `support/index.ts`, NEW `specs/web-persona.web.spec.ts` | **No-Tauri-mock** Playwright project — makes the browser persona testable at all |
| `sq-9ifq4` | haiku | `scripts/sync-wasm.mjs`, `.github/workflows/gui.yml`, `.github/workflows/pages.yml` | Optionally sync the existing text + rsp wasm bundles (P3; only blocks the two P3 tools) |

### Phase 1 — the maintainer's four bugs (parallel after Phase 0)

| Bead | Tier | Files (exclusive) | What |
| --- | --- | --- | --- |
| `sq-lcd6e` | **opus** | `engine-context.tsx`, `workspace-context.tsx`, `query-workbench.tsx`, NEW `specs/workspace-restore.spec.ts` | **Restore + lifecycle plumbing** — fixes the silent data-loss-on-relaunch; hydrate/switch/create/delete APIs; watch the sq-tp1m closure-cache races |
| `sq-lmlch` | sonnet | `left-rail.tsx`, NEW `workspace-switcher.tsx`, NEW spec | Switcher UI (list/create/rename/delete/switch). **Dep:** `sq-lcd6e` |
| `sq-glo5r` | **opus** | `packages/sparq-client/src/workspace.ts`, `engine-context.tsx`, `workspace-context.tsx`, `reason-wasm.ts`, `inference-tool.tsx`, `inference-control.tsx`, `site/test/workspace.test.mjs`, NEW spec | **N3 regime** + per-workspace rules docs + bulk rules upload. **Deps:** `sq-lcd6e` (shared engine/workspace context files — sequenced, NOT parallel), `sq-vnh1v`, `sq-5sjub` |
| `sq-eydh9` | sonnet | `import-drawer.tsx`, `tauri-ipc.ts`, `workbench.tsx`, NEW `specs/browser-upload.web.spec.ts` | Browser multi-file upload + global drag-and-drop; native dialog goes multi-select. **Deps:** `sq-vnh1v`, `sq-u0my5` |
| `sq-txrui` | sonnet | `shacl-tool.tsx`, NEW `shacl-sources.tsx`, NEW spec | SHACL highlighting + bulk multi-file shapes with per-source toggles. **Deps:** `sq-vnh1v`, `sq-5sjub` |

### Phase 2 — the tab matrix (parallel after `sq-5lyme`)

| Bead | Tier | Files (exclusive) | What |
| --- | --- | --- | --- |
| `sq-lxomy` (P2) | sonnet | `graph-view-tool.tsx`, NEW spec | Wire the already-shipped `GraphView` as a working tab (fixes the sq-lyp8 metadata drift) |
| `sq-iemfq` (P2) | sonnet | `server-tool.tsx`, NEW spec | SPARQL 1.1 Protocol endpoint client tab over the existing `@sparq/client` core; token never persisted |
| `sq-9nwab` (P3) | sonnet | `full-text-tool.tsx`, NEW `lib/text-wasm.ts`, NEW spec | Full-text tab over `sparq-text-wasm` (ground the crate's real JS surface first; escalate if insufficient). **Deps:** `sq-5lyme`, `sq-9ifq4` |
| `sq-kwb74` (P3) | sonnet | `streaming-tool.tsx`, NEW `lib/rsp-wasm.ts`, NEW spec | RSP-QL tick-view tab over `sparq-rsp-wasm` (same ground-first rule). **Deps:** `sq-5lyme`, `sq-9ifq4` |
| `sq-1odi9` (P3) | haiku | `left-rail.tsx`, NEW spec | Rail regroup: working Tools vs collapsed "Coming soon". **Deps:** `sq-5lyme`, `sq-lmlch` (sequenced on the rail file) |

### Explicitly out of scope

- **ZK / MPC tool tabs.** Both stay honest stubs: any in-tab ZK proving is research-grade —
  the v1 verifier is internally re-audited but **external accredited-cryptographer sign-off
  is pending (`sq-qhy4`)** — and the MPC crate is honest-majority **semi-honest only**, so
  neither may be presented as a working "live" tab under this epic. Their stub caveat text
  is load-bearing and must not be weakened by the rail regroup.
- **Vector / GeoSPARQL tabs** — blocked on the `sq-zeai` wasm portability spike.
- **Site `/try` removal** — `sq-4hiqe` (in progress, separate). One coordination note: the
  Server-tool bead treats the GUI as the new home of the endpoint-client UX that `/try`
  hosted; only `@sparq/client` cores (not `site/` components) are reused, so the two epics
  share no files except `sq-glo5r`'s touch of `site/test/workspace.test.mjs` (a unit-test
  file `/try` removal has no reason to touch).

## 4. Judgment calls (proceed-and-document)

1. **N3 = query-time ground-closure regime** (not a store format) — the only design that
   ships working N3 inference on the existing store; scope note surfaced in the tool.
2. **Browser File tab enabled** — the "Desktop app only" restriction is narrowed to what is
   actually impossible in a browser (arbitrary disk paths, compressed/HDT streaming); plain
   text RDF files upload via `File.text()` into the in-tab engine.
3. **Stub tools regrouped, not deleted** — keeps the honesty register while answering the
   "most tabs" perception.
4. **Per-tool tier overrides move into the panel files** (`sq-5lyme`) so honesty metadata
   flips land with the panel that earns them, keeping `tools.ts` conflict-free.

A tracking issue for post-hoc maintainer steering accompanies the research PR.

## 5. Verification map

Every user-visible bead lands with its own `gui/e2e-playwright/specs/*.spec.ts` (new file
per bead — the harness auto-discovers specs, so no shared-file edits), run by the existing
gating `gui.yml` Playwright lane; browser-persona specs use the new `chromium-web` project
(`*.web.spec.ts`). Library beads gate on `npm run lint && npm run typecheck` plus their
consumers' specs. No perf number is asserted anywhere (work-box/CI timings are
non-canonical); latency lines in the UI remain measured-and-labelled only.
