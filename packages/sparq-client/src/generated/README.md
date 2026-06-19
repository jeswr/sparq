<!-- [OPUS-4.8] sq-jpki -->

# `src/generated/` — the tracked wasm-pack type surface

`sparq_wasm.d.ts` in this directory is a **verbatim, machine-generated artifact**: it is the
`wasm-pack`-emitted TypeScript declaration for the site's WASM bundle
(`cd js && npm run build:wasm` → `crates/sparq-wasm` built with `--features shacl,jsonld`).
**Do not edit it by hand.**

## Why it is tracked here

The live build output lives in the git-ignored `js/wasm/` tree (`js/.gitignore`), so it does
not exist until someone runs the wasm build. `@sparq/client` re-exports the generated `Store`
class as the **single source of truth** for the WASM `Store` surface — replacing the
hand-redeclared `WasmStore` mirror that `research/gui-design.md` §0/§4 flagged as the drift
hazard ("kill the hand-redeclared `WasmStore` drift"). For the bare-package `tsc` (the
`gui.yml` `shared-client` job, which has **no** wasm build) to resolve those types, a copy of
the generated d.ts has to be checked in — hence this tracked file.

## Keeping it honest (no silent drift)

`npm run check:wasm-types` (`scripts/check-wasm-types.mjs`, wired into the `gui.yml`
`site-with-shared-client` job, which builds the bundle) rebuilds `js/wasm/sparq_wasm.d.ts` and
asserts this tracked copy is **byte-identical** to it. A binding change that is not
re-synced here therefore fails CI rather than rotting silently. The older
`typecheck:conformance` guard (which proved the *hand mirror* was a subset of the generated
`Store`) is obsolete now that the mirror is gone and the generated type IS the export.

## Regenerating after a binding change

```sh
cd js && npm run build:wasm      # rebuild the bundle (writes js/wasm/sparq_wasm.d.ts)
cd ../packages/sparq-client
npm run sync:wasm-types          # copy the fresh js/wasm d.ts into src/generated/
```

Commit the updated `sparq_wasm.d.ts`. `npm run check:wasm-types` then passes again.
