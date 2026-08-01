// [OPUS-5] sq-xqchl.3 — the entry `scripts/build-iife.mjs` bundles into the classic-`<script>`
// artifact `dist/eyereasoner-compat.iife.js`.
//
// This file is BUILD TOOLING, not package source: it deliberately lives outside `src/` so
// `tsc` never emits it into `dist/` (an ESM `dist/iife.js` alongside the IIFE would be a
// confusing second entrypoint). It is bundled from source by esbuild, which handles the
// `.ts` imports below directly.
//
// Its ONLY job is to pick the engine's `.wasm` URL before anything else runs. A classic
// script has no `import.meta.url`, so the wasm-bindgen `--target web` glue cannot self-resolve
// its sibling `_bg.wasm` the way the ESM entry (`dist/index.js`) does. `document.currentScript`
// is the classic-script equivalent: it is readable only while the script is executing
// SYNCHRONOUSLY — i.e. exactly now, at IIFE evaluation time — and is correct for `<script src>`
// including `async`/`defer`. It is `null` for a module script (which should import the ESM
// entry instead) and empty-stringed for an inline script; both fall through to leaving the
// loader's default in place, and the caller can always name the engine explicitly with
// `eyereasoner.configureWasm(...)` before the first `n3reasoner(...)` call.
import { configureWasm } from '../src/index.ts';

export * from '../src/index.ts';

// The artifact sits at `<pkg>/dist/eyereasoner-compat.iife.js` and the engine at
// `<pkg>/wasm/sparq_reason_wasm_bg.wasm` — a layout the `files: ["dist", "wasm"]` allowlist
// guarantees inside the tarball and every npm CDN serves verbatim.
const WASM_RELATIVE = '../wasm/sparq_reason_wasm_bg.wasm';

const scriptSrc = typeof document !== 'undefined' && document.currentScript
  ? document.currentScript.src
  : '';

if (scriptSrc) configureWasm(new URL(WASM_RELATIVE, scriptSrc));
