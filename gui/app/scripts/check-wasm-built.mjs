// [OPUS-4.8] sq-ixc3.8 — hard-fail the GUI frontend build if the wasm bundle is absent.
//
// The operational workbench runs the in-tab WASM engine, so the build embeds the lean
// `--features shacl,jsonld,…,forms` bundle (the same one the site syncs). That bundle is git-ignored
// (`js/wasm/` is a build artifact), so a fresh checkout must build it FIRST:
//
//     (cd ../../js && npm ci && npm run build:wasm)
//
// This pre-sync check makes the failure mode explicit and actionable rather than letting the
// engine 404 silently at runtime.
import { access } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const wasm = join(here, "..", "..", "..", "js", "wasm", "sparq_wasm_bg.wasm");

try {
  await access(wasm);
} catch {
  console.error(
    `[gui-app] WASM bundle not found at ${wasm}.\n` +
      `The GUI frontend embeds the in-tab engine, so build the bundle first:\n` +
      `    (cd ../../js && npm ci && npm run build:wasm)\n`,
  );
  process.exit(1);
}
