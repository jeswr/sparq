# @sparq/client

Framework-agnostic TypeScript client for the **sparq** WASM engine. This is the **single
shared declaration** of the `WasmStore` surface (the `crates/sparq-wasm` `Store` exposed over
wasm-bindgen) plus the loaders and query helpers around it.

It exists to remove the drift liability flagged in `research/gui-design.md` (§0 / §4): the
WASM `Store` TS interface was **hand-redeclared** inside `site/src/lib/sparq-wasm.ts`, so any
future GUI would have become a **third** hand-copy kept in sync by hand. The site now
re-exports the surface from here; the (proposed) Tauri 2 GUI consumes the same package.

## What's in it

- The type surface: `WasmStore`, `WasmSolutionCursor`, `WasmModule` / `WasmStoreCtor`, the
  SPARQL 1.1 JSON shapes (`SparqlResults`, `SparqlTerm`, `SparqlBinding`), and the SHACL
  report shapes (`ShaclReport`, `ShaclResult`).
- The runtime loader: `loadSparq(opts?)` / `prewarmSparq(opts?)` — single cold-start, with a
  configurable `basePath` (defaulted from `NEXT_PUBLIC_BASE_PATH` ?? `"/sparq"` so the site is
  unchanged; a desktop GUI passes its own `tauri://` / `file://` origin).
- The framework-agnostic query helpers: `matchQuads`, `countQuads`, `streamQueryRows`,
  `sparqShaclValidate`, `formatTerm`.
- **Endpoint mode** (`src/endpoint.ts`, `sq-2mke`): the SPARQL 1.1 Protocol HTTP client —
  the companion to the in-tab `WasmStore`. Run the SAME editor against any running
  `sparq-server` (or any conformant endpoint) over `fetch`. `runEndpointQuery(config, sparql)`
  classifies the form (`classifyEndpointForm`), builds the request (`buildSparqlRequest`: a
  direct `application/sparql-query` POST for reads with per-form `Accept`, a direct
  `application/sparql-update` POST for writes, bearer-auth in the `Authorization` header only),
  and parses the response per form (SELECT/ASK JSON, CONSTRUCT/DESCRIBE N-Triples, a `204`
  update ack). `connectionSafetyWarnings(config)` is the pure, honest connection-safety
  classifier that drives the Connect-panel UX; `wsSubprotocols(token)` derives the
  `bearer.<token>` subprotocol the server's WS handshake accepts (browsers cannot set an
  `Authorization` header on a WS upgrade). This client **consumes** the existing server API and
  **never bypasses a server gate** — it claims no security the server does not provide.

## Endpoint-mode safety posture (honest, never an overclaim)

`connectionSafetyWarnings` mirrors the real `sparq-server` posture (`crates/sparq-server/README.md`)
and the browser's transport rules, as classified `SafetyWarning`s (each with a stable `code`):

- `invalid-url` / `mixed-content` (**error**, hard block): not a valid http(s) URL; or an HTTPS
  page cannot `fetch` a plaintext `http:` endpoint (the browser blocks it before any request).
- `token-over-plaintext` / `non-loopback-no-tls` (**warning**): a bearer token — or the query and
  results — sent over plaintext `http:` to a **non-loopback** host travel in cleartext. (A token
  over **loopback** `http:` stays on the machine, so it is only an `info` note.) The server's
  Bearer gate is one shared secret with no per-user identity.
- `service-allowlist` / `cors-required` (**info**): a `SERVICE` clause is refused before any
  socket unless the operator set the egress allowlist (off / default-DENY); and the server emits
  **no CORS headers** by default, so a cross-origin browser fetch is blocked until an origin is
  opted in (`--cors-allow-origin`).

## What's NOT in it (deliberately)

- **Dataset-format coupling.** The site's `loadIntoStore` / `storeToNQuads` / `datasetSize`
  depend on `site/src/lib/repl-dataset.ts` (which named-graph formats route through
  `loadDataset`, the all-quads serialisation query). Those stay in the site; this package
  takes no opinion on dataset formats.
- **Any framework.** No React, no Next.js. The only browser globals touched are the two the
  wasm-pack glue needs at load time (`window.location`, dynamic `import()`), both guarded so
  the module is importable under `tsc` / Node type-checking.

## Consumption today (and the §4 generated-surface step)

The site imports this package via a **TypeScript path alias** (`@sparq/client` →
`../packages/sparq-client/src`), not an npm dependency — there is no repo-root `package.json`
or workspaces field today, and adopting npm/pnpm workspaces is a separate reviewable change to
the JS build topology (`research/gui-design.md` §3 "Tooling caveat"). The alias keeps the
static export building with **no new install and no lockfile change**.

### Generated-surface conformance guard — `sq-06gq`

The design's §4 end-state — re-exporting the **wasm-pack-generated** `sparq_wasm.d.ts`
directly so the `Store` surface is generated, not hand-mirrored — has a hard prerequisite: the
generated d.ts lives in the **git-ignored** build-artifact tree `js/wasm/` (see
`js/.gitignore`), so it does not exist until `cd js && npm run build:wasm` runs. A literal
`import` of it in `src/index.ts` would break the bare-package `tsc` (the `gui.yml`
`shared-client` job typechecks this package with **no** wasm build).

The CI-safe step taken now is a **compile-time conformance guard** (`src/conformance.ts` +
`tsconfig.conformance.json`): it imports the generated `Store` type via the
`#sparq-wasm-generated` path alias and asserts the hand-written surface stays a faithful
**subset** of the generated `Store`, so any future drift (a renamed/retyped/dropped method)
becomes a type error. Run it where the artifact exists:

```bash
(cd ../../js && npm run build:wasm)   # produces js/wasm/sparq_wasm.d.ts
npm run typecheck:conformance         # asserts WasmStore stays a subset of generated Store
```

It is wired into the `gui.yml` `site-with-shared-client` job (which builds the bundle) and is
**excluded** from the bare `npm run typecheck`, so the artifact-free `shared-client` CI job is
unaffected. The two deliberate divergences are asserted explicitly: `validate` is optional here
(the lean bundle may omit the `shacl` binding) and `queryCursor` returns the narrowed
`WasmSolutionCursor` rather than the generated `SolutionCursor`.

This package collapsed today's **two** hand-copies into **one** (consumed by the site and the
GUI), and the surface is now drift-checked against the generated d.ts. **Eliminating** the last
hand-copy — re-exporting the generated d.ts as the literal source so the hand mirror can be
deleted — needs repo-root npm/pnpm workspaces (so the package resolves a *tracked*,
always-present generated d.ts) and is tracked as a follow-up bead.

## Honesty

No performance number is asserted anywhere in this package (this repo's work box is
non-canonical). A caller may time a single query with `performance.now()` and label it as a
measured per-query latency. The ZK and MPC surfaces this client can drive elsewhere in the
repo are **research-grade and not externally audited** — that framing lives with those
surfaces and is unchanged by this package.
