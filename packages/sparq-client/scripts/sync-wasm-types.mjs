#!/usr/bin/env node
// [OPUS-4.8] sq-jpki — re-sync the TRACKED wasm `Store` d.ts from a fresh build.
//
// Copies the just-built `js/wasm/sparq_wasm.d.ts` (the site's `--features shacl,jsonld`
// bundle output — run `cd js && npm run build:wasm` first) into the tracked copy that
// `@sparq/client` re-exports as the single source of truth for the `Store` surface. Run this
// after any change to the `crates/sparq-wasm` binding, then commit the updated file;
// `npm run check:wasm-types` will then pass again.

import { copyFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, "..");
const repoRoot = resolve(pkgRoot, "..", "..");

const built = resolve(repoRoot, "js", "wasm", "sparq_wasm.d.ts");
const tracked = resolve(pkgRoot, "src", "generated", "sparq_wasm.d.ts");

if (!existsSync(built)) {
  console.error(
    `[sync-wasm-types] FAIL: ${built} is missing.\n` +
      "Build the bundle first: `cd js && npm run build:wasm`.",
  );
  process.exit(1);
}

copyFileSync(built, tracked);
console.log(
  `[sync-wasm-types] OK: copied\n  ${built}\n-> ${tracked}\n` +
    "Commit the updated src/generated/sparq_wasm.d.ts.",
);
