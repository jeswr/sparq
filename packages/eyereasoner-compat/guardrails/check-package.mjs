#!/usr/bin/env node
// [FABLE-5] sq-ohnj1 — publish guardrail for @sparq-org/eyereasoner-compat.
//
// Refuses to publish (or git-pin) a package that would ship without its wasm engine, that drops
// wasm/ or dist/ from the `files` allowlist, or that lost the `prepare` git-pin build hook.
// Inspects the tree `npm pack --dry-run --json` WOULD ship (no network, no Rust toolchain).
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const pkgDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const EXPECTED_NAME = '@sparq-org/eyereasoner-compat';
const failures = [];
const fail = (m) => failures.push(m);

const pkg = JSON.parse(readFileSync(resolve(pkgDir, 'package.json'), 'utf8'));

if (pkg.name !== EXPECTED_NAME) fail(`package name is "${pkg.name}", expected "${EXPECTED_NAME}"`);
if (!pkg.scripts?.prepare) fail('missing `prepare` script — git-pin installs would ship no engine');
for (const f of ['dist', 'wasm']) {
  if (!(pkg.files ?? []).includes(f)) fail(`\`files\` allowlist is missing "${f}"`);
}
if (pkg.publishConfig?.access !== 'public') fail('publishConfig.access must be "public" (scoped package)');

let packed;
try {
  // --ignore-scripts: inspect the CURRENT built tree (dist/ + wasm/) without re-triggering the
  // prepack wasm build; the build step runs before this guard in CI and locally.
  packed = JSON.parse(execFileSync('npm', ['pack', '--dry-run', '--json', '--ignore-scripts'], { cwd: pkgDir, encoding: 'utf8' }));
} catch (e) {
  fail(`npm pack --dry-run failed: ${e.message}`);
}
if (packed) {
  const entries = (packed[0]?.files ?? []).map((f) => f.path);
  const has = (re) => entries.some((p) => re.test(p));
  if (!has(/^dist\/index\.js$/)) fail('packed tarball is missing dist/index.js (the main entrypoint)');
  if (!has(/^dist\/index\.d\.ts$/)) fail('packed tarball is missing dist/index.d.ts (the type entrypoint)');
  // [OPUS-5] sq-xqchl.3 — the classic-`<script>` entrypoint, produced by `build:iife`. It is
  // easy to drop from the `build` chain without any ESM consumer noticing, so pin it here.
  if (!has(/^dist\/eyereasoner-compat\.iife\.js$/)) {
    fail('packed tarball is missing dist/eyereasoner-compat.iife.js (the classic <script> bundle)');
  }
  if (!has(/^wasm\/.*_bg\.wasm$/)) fail('packed tarball is missing the wasm engine (wasm/*_bg.wasm)');
  if (!has(/^wasm\/sparq_reason_wasm\.js$/)) fail('packed tarball is missing the wasm-bindgen glue');
}

if (failures.length) {
  console.error(`check:package FAILED for ${EXPECTED_NAME}:\n` + failures.map((m) => `  - ${m}`).join('\n'));
  process.exit(1);
}
console.log(`check:package OK — ${EXPECTED_NAME} ships dist/ + wasm engine + prepare hook.`);
