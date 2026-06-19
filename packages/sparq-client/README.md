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

## What's NOT in it (deliberately)

- **Dataset-format coupling.** The site's `loadIntoStore` / `storeToNQuads` / `datasetSize`
  depend on `site/src/lib/repl-dataset.ts` (which named-graph formats route through
  `loadDataset`, the all-quads serialisation query). Those stay in the site; this package
  takes no opinion on dataset formats.
- **Any framework.** No React, no Next.js. The only browser globals touched are the two the
  wasm-pack glue needs at load time (`window.location`, dynamic `import()`), both guarded so
  the module is importable under `tsc` / Node type-checking.

## Consumption today (and the deferred §4 step)

The site imports this package via a **TypeScript path alias** (`@sparq/client` →
`../packages/sparq-client/src`), not an npm dependency — there is no repo-root `package.json`
or workspaces field today, and adopting npm/pnpm workspaces is a separate reviewable change to
the JS build topology (`research/gui-design.md` §3 "Tooling caveat"). The alias keeps the
static export building with **no new install and no lockfile change**.

The design's §4 end-state — re-exporting the **wasm-pack-generated** `sparq_wasm.d.ts`
directly so the `Store` surface is generated, not hand-mirrored — is **deferred**: it depends
on the workspaces adoption above. This package collapses today's **two** hand-copies into
**one**; eliminating the last hand-copy is tracked as follow-up work.

## Honesty

No performance number is asserted anywhere in this package (this repo's work box is
non-canonical). A caller may time a single query with `performance.now()` and label it as a
measured per-query latency. The ZK and MPC surfaces this client can drive elsewhere in the
repo are **research-grade and not externally audited** — that framing lives with those
surfaces and is unchanged by this package.
