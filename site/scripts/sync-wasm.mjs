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

// [OPUS-4.8] sq-0po6 — also sync the tier-b "W-reason" bundle that drives
// /surface/inference (crates/sparq-reason-wasm, built with `--features explain` by
// `js/`'s `build:reason-wasm`). Its wasm-pack output stays in the crate's own pkg/
// (it is NOT part of the published @jeswr/sparq package), so we copy it from there
// into public/wasm/reason/. This bundle is OPTIONAL: if it has not been built (e.g. a
// quick `next dev` that skips it), we WARN and skip rather than fail — the inference
// page surfaces a clear "bundle failed to load" state at runtime in that case.
const reasonSrc = join(here, "..", "..", "crates", "sparq-reason-wasm", "pkg");
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
    `[sync-wasm] W-reason bundle not found at ${reasonSrc}; /surface/inference will not\n` +
      `            run live. Build it:  (cd ../js && npm run build:reason-wasm)`,
  );
}
