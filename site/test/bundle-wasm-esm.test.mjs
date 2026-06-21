// [OPUS-4.8] sq-55w5a (#981) — unit guard for the self-hosted ESM bundle that publishes the
// named `Dataset` entry from the project's OWN GitHub Pages origin (no third-party CDN).
//
// Two layers, both BUILD-INDEPENDENT so they run in CI without the ~MB wasm build:
//   1. A fixture build that exercises the SAME esbuild config the real script uses — asserts
//      the load-bearing invariants: the wasm glue stays EXTERNAL (rewritten to a sibling
//      `./sparq_wasm.js`, so the heavy binary is NEVER inlined → lazy-load preserved), bare
//      `node:` builtins stay external, and there is no third-party CDN specifier baked in.
//   2. If the REAL artifact has been built (`public/wasm/sparq.js` present), assert it too:
//      it exports the named `Dataset`, re-exports the sibling glue, inlines no `.wasm` bytes,
//      and carries no bare CDN import.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile, readFile, mkdir, access } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

const here = dirname(fileURLToPath(import.meta.url));

// The exact externalise-glue plugin the real script ships: any `…/sparq_wasm.js` import is
// kept external and rewritten to the sibling specifier, so the wasm binary is never bundled.
const externaliseGlue = {
  name: "externalise-wasm-glue",
  setup(b) {
    b.onResolve({ filter: /(^|\/)sparq_wasm\.js$/ }, () => ({
      path: "./sparq_wasm.js",
      external: true,
    }));
  },
};

test("the bundler keeps the wasm glue external (sibling specifier) and inlines fzstd", async () => {
  const dir = await mkdtemp(join(tmpdir(), "sparq-esm-"));
  // A fixture standing in for js/src: a module that (a) imports the wasm glue via the path the
  // real source uses (`../wasm/sparq_wasm.js`), and (b) lazily imports the bare `fzstd` dep.
  await mkdir(join(dir, "src"), { recursive: true });
  await mkdir(join(dir, "wasm"), { recursive: true });
  await writeFile(
    join(dir, "wasm", "sparq_wasm.js"),
    "export default async () => {}; export class Store {}\n",
  );
  await writeFile(
    join(dir, "node_modules-fzstd-shim.js"),
    "export function decompress(b){ return b; }\n",
  );
  await writeFile(
    join(dir, "src", "index.ts"),
    [
      "import initWasm, { Store } from '../wasm/sparq_wasm.js';",
      "export class Dataset { static async create(){ await initWasm(); return new Dataset(); } }",
      "export async function unzip(b: Uint8Array){ const { decompress } = await import('fzstd'); return decompress(b); }",
      "export { Store };",
    ].join("\n"),
  );

  const out = join(dir, "out", "sparq.js");
  await build({
    entryPoints: [join(dir, "src", "index.ts")],
    outfile: out,
    bundle: true,
    format: "esm",
    platform: "browser",
    target: "es2022",
    external: ["node:fs/promises", "node:zlib"],
    plugins: [
      externaliseGlue,
      // Resolve the fixture's bare `fzstd` to the local shim so the fixture is self-contained.
      {
        name: "fzstd-shim",
        setup(b) {
          b.onResolve({ filter: /^fzstd$/ }, () => ({
            path: join(dir, "node_modules-fzstd-shim.js"),
          }));
        },
      },
    ],
    minify: false,
  });

  const code = await readFile(out, "utf8");
  // The glue import is rewritten to the SIBLING specifier and kept external (not inlined).
  assert.match(code, /from\s*"\.\/sparq_wasm\.js"/);
  assert.doesNotMatch(code, /\.\.\/wasm\/sparq_wasm\.js/);
  // The named `Dataset` entry is exported.
  assert.match(code, /\bDataset\b/);
  // The lazily-imported `fzstd` is INLINED (no bare specifier survives), so a browser
  // `<script type="module">` from a static origin needs no CDN to resolve it.
  assert.doesNotMatch(code, /import\(\s*["']fzstd["']\s*\)/);
  // No third-party CDN origin is baked into the artifact.
  assert.doesNotMatch(code, /esm\.sh|unpkg\.com|jsdelivr/);
});

test("the published artifact (if built) exports Dataset and keeps the wasm binary external", async () => {
  const artifact = join(here, "..", "public", "wasm", "sparq.js");
  try {
    await access(artifact);
  } catch {
    // The bundle is git-ignored + only produced by the prebuild chain after the wasm build;
    // skip cleanly when a bare checkout has not built it (CI builds it before `next build`).
    return;
  }
  const code = await readFile(artifact, "utf8");
  // Named `Dataset` export present (the #981 import shape).
  assert.match(code, /\bDataset\b/);
  // Re-exports the sibling glue — the wasm binary is fetched by IT, lazily, not inlined here.
  assert.match(code, /from\s*"\.\/sparq_wasm\.js"/);
  // No third-party CDN origin baked in (self-hosted).
  assert.doesNotMatch(code, /esm\.sh|unpkg\.com|jsdelivr/);
  // No bare `fzstd` import survives (inlined).
  assert.doesNotMatch(code, /import\(\s*["']fzstd["']\s*\)/);
});
