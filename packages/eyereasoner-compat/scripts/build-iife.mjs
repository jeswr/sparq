#!/usr/bin/env node
// [OPUS-5] sq-xqchl.3 — build the non-module IIFE bundle for classic `<script>` use.
//
// The package's normal entrypoint (`dist/index.js`) is ESM: it only works from
// `<script type="module">`, a bundler, or Node. A plain `<script src="…">` in a page with no
// build step cannot load it. This produces the other half — ONE self-contained classic script
// that assigns the `eyereasoner` global (`eyereasoner.n3reasoner(…)`).
//
// WHY the whole graph is inlined here, unlike site/scripts/bundle-wasm-esm.mjs (which keeps the
// wasm glue EXTERNAL): a classic script cannot `import`, so every module in the graph — the
// compat layer AND the wasm-bindgen glue — must end up in the single file.
//
// WHY the wasm BINARY still stays out of the bundle: it is fetched lazily by the first
// `n3reasoner(…)` call from a URL derived in `iife-entry.mjs`, so loading the script stays cheap
// and the engine still streams/compiles as a real `.wasm` (no base64 inflation of the payload).
import { access } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { build } from 'esbuild';

const pkgDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const entry = join(pkgDir, 'scripts', 'iife-entry.mjs');
const glue = join(pkgDir, 'wasm', 'sparq_reason_wasm.js');
const outfile = join(pkgDir, 'dist', 'eyereasoner-compat.iife.js');

// `src/loader.ts` statically imports the glue, so the wasm build must have run first.
try {
  await access(glue);
} catch {
  console.error(
    `[build-iife] ${glue} not found.\n`
      + 'Run `npm run build:wasm` first (the `build` script chains them in order).',
  );
  process.exit(1);
}

// `src/loader.ts` reaches for `node:fs/promises` in its Node branch (Node's fetch cannot read
// `file://`). That branch is guarded by a `typeof process !== 'undefined'` check that is false
// in the browser this bundle targets, and a browser cannot resolve a `node:` specifier at all —
// so stub it rather than leave a bare external `import()` in a classic script.
const stubNodeFs = {
  name: 'stub-node-fs',
  setup(b) {
    b.onResolve({ filter: /^node:fs\/promises$/ }, () => ({ path: 'node-fs', namespace: 'iife-stub' }));
    b.onLoad({ filter: /.*/, namespace: 'iife-stub' }, () => ({
      contents: 'export const readFile = () => {'
        + ' throw new Error("[eyereasoner-compat] the classic <script> bundle is browser-only;'
        + ' use the ESM entry in Node."); };',
      loader: 'js',
    }));
  },
};

await build({
  entryPoints: [entry],
  outfile,
  bundle: true,
  format: 'iife',
  globalName: 'eyereasoner',
  platform: 'browser',
  target: 'es2022',
  plugins: [stubNodeFs],
  legalComments: 'none',
  minify: true,
  sourcemap: false,
  // `import.meta` has no meaning in a classic script, so esbuild substitutes `{}` and warns.
  // Both remaining uses are provably dead in this bundle: the glue's `new URL('…_bg.wasm',
  // import.meta.url)` default only runs when no `module_or_path` was given (iife-entry.mjs
  // gives one), and `src/loader.ts`'s is inside the Node branch stubbed out above. Silence the
  // expected warning so a REAL one in this build is not lost in the noise.
  logOverride: { 'empty-import-meta': 'silent' },
  // NOTE: no literal `</` + `script>` sequence anywhere in this banner — an HTML parser would
  // end the surrounding element on it if a page ever inlines this file into a script block.
  banner: {
    js:
      '/* @sparq-org/eyereasoner-compat — the eye-js `n3reasoner` API backed by sparq\'s native\n'
      + '   N3 reasoner (Rust->WASM). Classic-script build: assigns the `eyereasoner` global,\n'
      + '   so `await eyereasoner.n3reasoner(data, query)` works with no bundler and no modules.\n'
      + '   The engine wasm is fetched lazily from ../wasm/ (relative to THIS script) on first\n'
      + '   use; call eyereasoner.configureWasm(url) beforehand to point somewhere else.\n'
      + '   Docs: https://github.com/sparq-org/sparq/tree/main/packages/eyereasoner-compat */',
  },
});

console.log(`[build-iife] wrote classic <script> bundle -> ${outfile}`);
