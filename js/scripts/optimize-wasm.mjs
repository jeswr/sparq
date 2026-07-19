// [SONNET-4.6] Keep the optimizer choice visible instead of inheriting wasm-pack's default.
import { renameSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';

const [wasmPath, level] = process.argv.slice(2);
const supportedLevels = new Set(['-O', '-O2', '-O3', '-Os', '-Oz']);

if (!wasmPath || !supportedLevels.has(level)) {
  console.error('usage: node scripts/optimize-wasm.mjs <module.wasm> <-O|-O2|-O3|-Os|-Oz>');
  process.exit(2);
}

const outputPath = `${wasmPath}.optimized`;
const result = spawnSync(
  'wasm-opt',
  [level, '--strip-debug', '--strip-producers', wasmPath, '-o', outputPath],
  { stdio: 'inherit' },
);

if (result.error?.code === 'ENOENT') {
  console.error('wasm-opt was not found on PATH; install Binaryen before building the wasm bundle');
  process.exit(1);
}
if (result.error || result.status !== 0) {
  rmSync(outputPath, { force: true });
  process.exit(result.status ?? 1);
}

renameSync(outputPath, wasmPath);
