// [FABLE-5] sq-zv37m — post-export strip of the legacy noModule polyfills chunk, Tauri target
// ONLY (wired into `build:tauri`; `build:web` is untouched).
//
// Next.js static export unconditionally emits `_next/static/chunks/polyfills-*.js` (~110 KB raw,
// ~39 KB gz) referenced solely via `<script … noModule>` tags. A `noModule` classic script is
// never even FETCHED by an ES-module-capable browser, and the Tauri 2 webview (WebView2 /
// WKWebView / WebKitGTK) is always module-capable — so for the desktop bundle the chunk is pure
// dead installer weight. Upstream config seam is being proposed to vercel/next.js (see
// research/nextjs-nomodule-polyfills-upstream.md); this is the interim sparq-side mitigation.
//
// Behaviour (deterministic, loud):
//   1. delete out/_next/static/chunks/polyfills-*.js — HARD ERROR if none exist, so when a Next
//      upgrade stops emitting the chunk (e.g. the upstream seam lands) this step fails visibly
//      and gets removed rather than rotting;
//   2. remove every `<script … noModule …>` tag referencing a polyfills- chunk from the exported
//      *.html (belt-and-braces: modern engines skip the fetch anyway, but no dangling markup);
//   3. re-scan the WHOLE export tree for any remaining `polyfills-` reference — HARD ERROR if
//      one survives (guards against a future Next referencing the chunk from somewhere new,
//      e.g. a manifest the strip does not know about).
import { readdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const POLYFILL_RE = /polyfills-/;
// Next emits the polyfill reference as `<script src="…/polyfills-<hash>.js" noModule=""></script>`.
const TAG_RE = /<script\b[^>]*\bnomodule\b[^>]*><\/script>/gi;

async function* walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(p);
    else if (entry.isFile()) yield p;
  }
}

/**
 * Strip the legacy noModule polyfills chunk from a static-export tree.
 * Returns { removedFiles, bytesSaved, htmlEdited }; throws on the two drift conditions above.
 */
export async function stripLegacyPolyfills(outDir) {
  const chunksDir = join(outDir, "_next", "static", "chunks");
  const chunks = (await readdir(chunksDir)).filter(
    (f) => f.startsWith("polyfills-") && f.endsWith(".js"),
  );
  if (chunks.length === 0) {
    throw new Error(
      `[strip-legacy-polyfills] no polyfills-*.js chunk under ${chunksDir} — ` +
        `Next.js may have stopped emitting it (upstream seam landed?); ` +
        `remove this build step (bead sq-zv37m) instead of skipping silently.`,
    );
  }

  let bytesSaved = 0;
  const removedFiles = [];
  for (const f of chunks) {
    const p = join(chunksDir, f);
    bytesSaved += (await stat(p)).size;
    await rm(p);
    removedFiles.push(relative(outDir, p));
  }

  const htmlEdited = [];
  const leftover = [];
  for await (const p of walk(outDir)) {
    if (p.endsWith(".html")) {
      const html = await readFile(p, "utf8");
      const stripped = html.replaceAll(TAG_RE, (tag) =>
        POLYFILL_RE.test(tag) ? "" : tag,
      );
      if (stripped !== html) {
        await writeFile(p, stripped);
        htmlEdited.push(relative(outDir, p));
      }
      if (POLYFILL_RE.test(stripped)) leftover.push(relative(outDir, p));
    } else if (POLYFILL_RE.test(await readFile(p, "latin1"))) {
      leftover.push(relative(outDir, p));
    }
  }
  if (leftover.length > 0) {
    throw new Error(
      `[strip-legacy-polyfills] 'polyfills-' still referenced after strip: ` +
        `${leftover.join(", ")} — Next.js now references the chunk from somewhere ` +
        `this script does not handle; fix the strip or drop it (bead sq-zv37m).`,
    );
  }
  return { removedFiles, bytesSaved, htmlEdited };
}

// CLI entry (what `build:tauri` runs).
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const here = dirname(fileURLToPath(import.meta.url));
  const { removedFiles, bytesSaved, htmlEdited } = await stripLegacyPolyfills(
    join(here, "..", "out"),
  );
  console.log(
    `[strip-legacy-polyfills] removed ${removedFiles.join(", ")} ` +
      `(${bytesSaved} B) and its noModule tag from ${htmlEdited.length} html file(s).`,
  );
}
