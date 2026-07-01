// [OPUS-4.8] sq-ixc3.8 — copy the lean sparq wasm bundle into public/wasm/ for the in-tab
// engine the operational workbench runs. Source of truth is the wasm-pack `--target web`
// output `js/`'s `build:wasm` produces (crates/sparq-wasm → js/wasm). Run after that build.
//
// The workbench loads these as plain static assets from /public at runtime (via the
// @sparq/client loader, which keys its asset URLs off NEXT_PUBLIC_BASE_PATH), so they must live
// in public/. The core SPARQL + SHACL engine is REQUIRED; the tier-b W-reason bundle (below) is
// OPTIONAL (the Inference tool degrades honestly without it).
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

// [OPUS-4.8] sq-tp1m (#757) — also sync the tier-b "W-reason" bundle that powers the Inference
// tool (crates/sparq-reason-wasm, built with `--features explain` by `js/`'s `build:reason-wasm`,
// exactly as the marketing site syncs it). Its wasm-pack output stays in the crate's own pkg/
// (it is NOT part of the published @jeswr/sparq package), so we copy it from there into
// public/wasm/reason/. This bundle is OPTIONAL: if it has not been built (e.g. a `next dev` that
// skips it, or the gated CI build), we WARN and skip rather than fail — the Inference tool then
// surfaces a clear "reasoner unavailable" state at runtime and queries run without inference.
const reasonSrc = join(here, "..", "..", "..", "crates", "sparq-reason-wasm", "pkg");
const reasonDest = join(dest, "reason");
const reasonFiles = [
  "sparq_reason_wasm.js",
  "sparq_reason_wasm_bg.wasm",
  "sparq_reason_wasm.d.ts",
];

try {
  await access(join(reasonSrc, "sparq_reason_wasm_bg.wasm"));
  await mkdir(reasonDest, { recursive: true });
  for (const f of reasonFiles) {
    await copyFile(join(reasonSrc, f), join(reasonDest, f));
  }
  console.log(
    `[sync-wasm] copied ${reasonFiles.length} files → public/wasm/reason/ (W-reason)`,
  );
} catch {
  console.warn(
    `[sync-wasm] W-reason bundle not found at ${reasonSrc}; the Inference tool will not run\n` +
      `            live. Build it:  (cd ../../js && npm run build:reason-wasm)`,
  );
}
