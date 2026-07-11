// [OPUS-4.8] sq-ixc3.8 — copy the lean sparq wasm bundle into public/wasm/ for the in-tab
// engine the operational workbench runs. Source of truth is the wasm-pack `--target web`
// output `js/`'s `build:wasm` produces (crates/sparq-wasm → js/wasm). Run after that build.
//
// The workbench loads these as plain static assets from /public at runtime (via the
// @sparq/client loader, which keys its asset URLs off NEXT_PUBLIC_BASE_PATH), so they must live
// in public/. The core SPARQL + SHACL engine is REQUIRED; the tier-b W-reason bundle (below) is
// OPTIONAL (the Inference tool degrades honestly without it).
import { mkdir, copyFile, access, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "..", "..", "..", "js", "wasm");
const dest = join(here, "..", "public", "wasm");
// [FABLE-5] sq-qgkwy.2 — RUNTIME files only (glue + wasm). The wasm-pack `.d.ts` files are
// build-time TypeScript declarations nothing fetches at runtime; everything under public/ is
// copied verbatim into out/ and then into the Tauri bundle + the hosted web deploy, so copying
// them here shipped ~46 KiB of dead bytes per build. Type-checking reads js/wasm/ directly.
const files = ["sparq_wasm.js", "sparq_wasm_bg.wasm"];

/** Remove a stale non-runtime file an OLDER sync may have left in public/wasm/. */
async function rmStale(...paths) {
  for (const p of paths) {
    await rm(p, { force: true });
  }
}

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
await rmStale(join(dest, "sparq_wasm.d.ts"));
console.log(`[sync-wasm] copied ${files.length} files → public/wasm/`);

// [OPUS-4.8] sq-tp1m (#757) — also sync the tier-b "W-reason" bundle that powers the Inference
// tool (crates/sparq-reason-wasm, built with `--features explain` by `js/`'s `build:reason-wasm`,
// exactly as the marketing site syncs it). Its wasm-pack output stays in the crate's own pkg/
// (it is NOT part of the published @jeswr/sparq package), so we copy it from there into
// public/wasm/reason/. This bundle is OPTIONAL: if it has not been built (e.g. a `next dev` that
// skips it, or the gated CI build), we WARN and skip rather than fail — the Inference tool then
// surfaces a clear "reasoner unavailable" state at runtime, and (rather than silently running
// un-reasoned) queries hard-fail while inference is on until it is rebuilt or inference is off.
const reasonSrc = join(here, "..", "..", "..", "crates", "sparq-reason-wasm", "pkg");
const reasonDest = join(dest, "reason");
const reasonFiles = ["sparq_reason_wasm.js", "sparq_reason_wasm_bg.wasm"];

try {
  await access(join(reasonSrc, "sparq_reason_wasm_bg.wasm"));
  await mkdir(reasonDest, { recursive: true });
  for (const f of reasonFiles) {
    await copyFile(join(reasonSrc, f), join(reasonDest, f));
  }
  await rmStale(join(reasonDest, "sparq_reason_wasm.d.ts"));
  console.log(
    `[sync-wasm] copied ${reasonFiles.length} files → public/wasm/reason/ (W-reason)`,
  );
} catch {
  console.warn(
    `[sync-wasm] W-reason bundle not found at ${reasonSrc}; the Inference tool will not run\n` +
      `            live. Build it:  (cd ../../js && npm run build:reason-wasm)`,
  );
}

// [HAIKU-4.5] sq-9ifq4 — also sync the tier-b "W-text" bundle that powers the Full-text tool
// (crates/sparq-text-wasm, built with `npm run build:text-wasm` by js/'s wasm build, exactly
// as the marketing site syncs it). Its wasm-pack output stays in the crate's own pkg/, so we copy
// it from there into public/wasm/text/. This bundle is OPTIONAL: if it has not been built (e.g. a
// `next dev` that skips it), we WARN and skip rather than fail — the Full-text tool then
// surfaces a clear "bundle unavailable" state at runtime and degrades gracefully.
const textSrc = join(here, "..", "..", "..", "crates", "sparq-text-wasm", "pkg");
const textDest = join(dest, "text");
const textFiles = ["sparq_text_wasm.js", "sparq_text_wasm_bg.wasm"];

try {
  await access(join(textSrc, "sparq_text_wasm_bg.wasm"));
  await mkdir(textDest, { recursive: true });
  for (const f of textFiles) {
    await copyFile(join(textSrc, f), join(textDest, f));
  }
  await rmStale(join(textDest, "sparq_text_wasm.d.ts"));
  console.log(
    `[sync-wasm] copied ${textFiles.length} files → public/wasm/text/ (W-text)`,
  );
} catch {
  console.warn(
    `[sync-wasm] W-text bundle not found at ${textSrc}; the Full-text tool will not run\n` +
      `            live. Build it:  (cd ../../js && npm run build:text-wasm)`,
  );
}

// [HAIKU-4.5] sq-9ifq4 — also sync the tier-b "W-rsp" bundle that powers the Streaming tool
// (crates/sparq-rsp-wasm, built with `npm run build:rsp-wasm` by js/'s wasm build, exactly
// as the marketing site syncs it). Its wasm-pack output stays in the crate's own pkg/, so we copy
// it from there into public/wasm/rsp/. This bundle is OPTIONAL: if it has not been built (e.g. a
// `next dev` that skips it), we WARN and skip rather than fail — the Streaming tool then
// surfaces a clear "bundle unavailable" state at runtime and degrades gracefully.
const rspSrc = join(here, "..", "..", "..", "crates", "sparq-rsp-wasm", "pkg");
const rspDest = join(dest, "rsp");
const rspFiles = ["sparq_rsp_wasm.js", "sparq_rsp_wasm_bg.wasm"];

try {
  await access(join(rspSrc, "sparq_rsp_wasm_bg.wasm"));
  await mkdir(rspDest, { recursive: true });
  for (const f of rspFiles) {
    await copyFile(join(rspSrc, f), join(rspDest, f));
  }
  await rmStale(join(rspDest, "sparq_rsp_wasm.d.ts"));
  console.log(
    `[sync-wasm] copied ${rspFiles.length} files → public/wasm/rsp/ (W-rsp)`,
  );
} catch {
  console.warn(
    `[sync-wasm] W-rsp bundle not found at ${rspSrc}; the Streaming tool will not run\n` +
      `            live. Build it:  (cd ../../js && npm run build:rsp-wasm)`,
  );
}
