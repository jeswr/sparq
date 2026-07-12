// [FABLE-5] sq-zv37m — unit tests for the Tauri-target legacy-polyfill strip, against a
// fixture export tree (no Next build needed).
import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { stripLegacyPolyfills } from "./strip-legacy-polyfills.mjs";

const POLYFILL = "polyfills-42372ed130431b0a.js";
const MODERN_TAG = `<script src="/_next/static/chunks/main-app-abc.js" async=""></script>`;
const POLYFILL_TAG = `<script src="/_next/static/chunks/${POLYFILL}" noModule=""></script>`;

/** Build a minimal static-export fixture mirroring what `next build` (output: export) emits. */
async function makeFixture() {
  const out = await mkdtemp(join(tmpdir(), "strip-polyfills-"));
  const chunks = join(out, "_next", "static", "chunks");
  await mkdir(chunks, { recursive: true });
  await writeFile(join(chunks, POLYFILL), "x".repeat(1234));
  await writeFile(join(chunks, "main-app-abc.js"), "console.log(1)");
  await mkdir(join(out, "404"), { recursive: true });
  const html = `<!DOCTYPE html><html><head>${POLYFILL_TAG}${MODERN_TAG}</head><body></body></html>`;
  await writeFile(join(out, "index.html"), html);
  await writeFile(join(out, "404", "index.html"), html);
  return out;
}

test("removes the polyfills chunk + its noModule tag, keeps modern scripts", async (t) => {
  const out = await makeFixture();
  t.after(() => rm(out, { recursive: true, force: true }));

  const res = await stripLegacyPolyfills(out);

  assert.deepEqual(res.removedFiles, [join("_next", "static", "chunks", POLYFILL)]);
  assert.equal(res.bytesSaved, 1234);
  assert.equal(res.htmlEdited.length, 2);
  const html = await readFile(join(out, "index.html"), "utf8");
  assert.ok(!html.includes("polyfills-"), "polyfill tag stripped");
  assert.ok(html.includes(MODERN_TAG), "modern script tag untouched");
  await assert.rejects(readFile(join(out, "_next", "static", "chunks", POLYFILL)));
});

test("hard-errors when no polyfills chunk exists (upstream fixed → remove the step)", async (t) => {
  const out = await makeFixture();
  t.after(() => rm(out, { recursive: true, force: true }));
  await rm(join(out, "_next", "static", "chunks", POLYFILL));

  await assert.rejects(() => stripLegacyPolyfills(out), /no polyfills-\*\.js chunk/);
});

test("hard-errors when a non-html file still references the chunk after the strip", async (t) => {
  const out = await makeFixture();
  t.after(() => rm(out, { recursive: true, force: true }));
  // Simulate a future Next referencing the chunk from a manifest the strip does not know about.
  await writeFile(
    join(out, "_next", "static", "chunks", "webpack-runtime.js"),
    `self.__FILES__=["static/chunks/${POLYFILL}"]`,
  );

  await assert.rejects(() => stripLegacyPolyfills(out), /still referenced after strip/);
});
