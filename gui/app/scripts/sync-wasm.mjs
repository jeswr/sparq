// [OPUS-4.8] sq-ixc3.8 — copy the lean sparq wasm bundle into public/wasm/ for the in-tab
// engine the operational workbench runs. Source of truth is the wasm-pack `--target web`
// output `js/`'s `build:wasm` produces (crates/sparq-wasm → js/wasm). Run after that build.
//
// The workbench loads these as plain static assets from /public at runtime (via the
// @sparq/client loader, which keys its asset URLs off NEXT_PUBLIC_BASE_PATH), so they must live
// in public/. The full satellite bundles (reason/rsp/text) the marketing site syncs are NOT
// copied here — the foundation shell ships only the core SPARQL + SHACL engine; later GUI
// phases (sq-ixc3.11/.12) add the tool tabs that would consume them.
import { mkdir, copyFile, access } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "..", "..", "..", "js", "wasm");
const dest = join(here, "..", "public", "wasm");
const files = ["sparq_wasm.js", "sparq_wasm_bg.wasm", "sparq_wasm.d.ts"];

try {
  await access(join(src, "sparq_wasm_bg.wasm"));
} catch {
  console.error(
    `[sync-wasm] ${src}/sparq_wasm_bg.wasm not found.\n` +
      `Build it first:  (cd ../../js && npm ci && npm run build:wasm)`,
  );
  process.exit(1);
}

await mkdir(dest, { recursive: true });
for (const f of files) {
  await copyFile(join(src, f), join(dest, f));
}
console.log(`[sync-wasm] copied ${files.length} files → public/wasm/`);
