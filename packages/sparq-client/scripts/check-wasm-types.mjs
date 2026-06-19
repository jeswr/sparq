#!/usr/bin/env node
// [OPUS-4.8] sq-jpki — single-source-of-truth guard for the tracked wasm `Store` d.ts.
//
// `@sparq/client` re-exports the wasm-pack-GENERATED `Store` surface from the TRACKED copy
// `src/generated/sparq_wasm.d.ts` (the hand-redeclared `WasmStore` mirror is DELETED —
// research/gui-design.md §0/§4). The live build output lives in the git-ignored
// `js/wasm/sparq_wasm.d.ts`, so the tracked copy is the only type source the artifact-free
// `tsc` can see — which means it CAN silently drift from the actual binding. This guard
// closes that hole: when a freshly built `js/wasm/sparq_wasm.d.ts` is present it asserts the
// tracked copy is BYTE-IDENTICAL to it, so a binding change that was not re-synced here
// (`npm run sync:wasm-types`) fails CI rather than rotting. The mirror cannot reappear,
// because the export IS the generated type and this check proves it matches the engine.
//
// CI-safe: where the wasm bundle has NOT been built (the bare `shared-client` typecheck
// job), `js/wasm/sparq_wasm.d.ts` is absent — the guard then SKIPS with a clear message and
// exit 0, exactly as the old conformance guard ran only in the bundle-building job. The
// `site-with-shared-client` job (which runs `js/npm run build:wasm`) runs it for real.

import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, "..");
// packages/sparq-client/scripts -> repo root is three up.
const repoRoot = resolve(pkgRoot, "..", "..");

const tracked = resolve(pkgRoot, "src", "generated", "sparq_wasm.d.ts");
const built = resolve(repoRoot, "js", "wasm", "sparq_wasm.d.ts");

if (!existsSync(tracked)) {
  console.error(
    `[check-wasm-types] FAIL: tracked generated d.ts is missing: ${tracked}\n` +
      "It must be committed so the bare-package typecheck can resolve the Store surface.",
  );
  process.exit(1);
}

if (!existsSync(built)) {
  console.log(
    "[check-wasm-types] SKIP: no freshly built js/wasm/sparq_wasm.d.ts present " +
      "(run `cd js && npm run build:wasm` to enable the byte-identity check). " +
      "The tracked copy is the type source for the artifact-free typecheck.",
  );
  process.exit(0);
}

const trackedText = readFileSync(tracked, "utf8");
const builtText = readFileSync(built, "utf8");

if (trackedText !== builtText) {
  console.error(
    "[check-wasm-types] FAIL: the tracked wasm d.ts has DRIFTED from the freshly built one.\n" +
      `  tracked: ${tracked}\n` +
      `  built:   ${built}\n` +
      "The wasm `Store` binding changed without re-syncing the tracked copy. " +
      "Re-sync with `npm run sync:wasm-types` and commit the updated " +
      "src/generated/sparq_wasm.d.ts (do NOT re-introduce a hand-written mirror).",
  );
  process.exit(1);
}

console.log(
  "[check-wasm-types] OK: src/generated/sparq_wasm.d.ts is byte-identical to the freshly " +
    "built js/wasm/sparq_wasm.d.ts — the generated Store surface is the single source of truth.",
);
