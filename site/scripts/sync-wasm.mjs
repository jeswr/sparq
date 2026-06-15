// [OPUS-4.8] sq-8thu — copy the lean sparq wasm bundle into public/wasm/ for the
// live REPL. Source of truth is the wasm-pack `--target web` output that `js/`'s
// `build:wasm` produces (crates/sparq-wasm → js/wasm). Run after that build.
//
// Kept out of the bundler: the REPL loads these as plain static assets from
// /public at runtime (see src/lib/sparq-wasm.ts), so they must live in public/.
import { mkdir, copyFile, access } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "..", "..", "js", "wasm");
const dest = join(here, "..", "public", "wasm");
const files = ["sparq_wasm.js", "sparq_wasm_bg.wasm", "sparq_wasm.d.ts"];

try {
  await access(join(src, "sparq_wasm_bg.wasm"));
} catch {
  console.error(
    `[sync-wasm] ${src}/sparq_wasm_bg.wasm not found.\n` +
      `Build it first:  (cd ../js && npm ci && npm run build:wasm)`,
  );
  process.exit(1);
}

await mkdir(dest, { recursive: true });
for (const f of files) {
  await copyFile(join(src, f), join(dest, f));
}
console.log(`[sync-wasm] copied ${files.length} files → public/wasm/`);
