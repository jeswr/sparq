#!/usr/bin/env node
// [FABLE-5] sq-ohnj1 — copy the wasm-pack output into the package's `wasm/` dir so the asset
// ships INSIDE the npm tarball (never a site-hosted URL). Mirrors js/'s `_copy:wasm` step.
import { cpSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const pkgDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const src = resolve(pkgDir, '..', '..', 'crates', 'sparq-reason-wasm', 'pkg');
const dst = resolve(pkgDir, 'wasm');

const FILES = [
  'sparq_reason_wasm.js',
  'sparq_reason_wasm.d.ts',
  'sparq_reason_wasm_bg.wasm',
  'sparq_reason_wasm_bg.wasm.d.ts',
];

rmSync(dst, { recursive: true, force: true });
mkdirSync(dst, { recursive: true });
for (const f of FILES) {
  cpSync(resolve(src, f), resolve(dst, f));
}
console.log(`copied ${FILES.length} wasm-pack artifacts into ${dst}`);
