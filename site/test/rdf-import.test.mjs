// [GPT-5.6] #1046 — import-boundary coverage for the same .nt.zst fixture through the
// Upload and From URL call shapes. The helper returns parser-ready RDF text + syntax; the
// wasm parser is covered separately, so these tests stay fast and framework-free.
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import assert from "node:assert/strict";

import { bytesToRdf } from "../src/lib/rdf-import.ts";

const SAMPLE_NT =
  '<http://example.org/s> <http://example.org/p> "object value" .\n' +
  "<http://example.org/s> <http://example.org/q> <http://example.org/o2> .\n";

// Reference-codec fixture generated once from SAMPLE_NT with zstd. Node 20 has no zstd
// compressor, so retaining the real RFC 8878 frame as base64 keeps the test portable.
const NT_ZST_FIXTURE = Uint8Array.from(
  Buffer.from(
    "KLUv/SSHDQIAJAM8aHR0cDovL2V4YW1wbGUub3JnL3M+IHA+ICJvYmplY3QgdmFsdWUiIC4KcW8yPiAuCgQAOooIuAaWolmlZxOc62uy",
    "base64",
  ),
);

test("Upload imports a .nt.zst fixture as parser-ready N-Triples", async () => {
  const result = await bytesToRdf(NT_ZST_FIXTURE, "fixture.nt.zst");
  assert.deepEqual(result, { text: SAMPLE_NT, format: "ntriples" });
});

test("From URL imports the same .nt.zst fixture from binary response metadata", async () => {
  const result = await bytesToRdf(
    NT_ZST_FIXTURE,
    "https://example.org/fixture.nt.zst?download=1",
    { explicitFormat: "__auto__", contentType: "application/zstd" },
  );
  assert.deepEqual(result, { text: SAMPLE_NT, format: "ntriples" });
});

test("bzip2 stays rejected with the existing unsupported archive error", async () => {
  const bzip2Magic = new Uint8Array([0x42, 0x5a, 0x68, 0x39, 0, 0, 0, 0]);
  await assert.rejects(
    bytesToRdf(bzip2Magic, "fixture.nt.bz2"),
    /Unrecognised compressed payload: expected a gzip.*zip.*zstd/,
  );
});

test("site zstd import delegates to js/decompress and never imports fzstd directly", async () => {
  const archiveSource = await readFile(
    new URL("../src/lib/dataset-archive.ts", import.meta.url),
    "utf8",
  );
  const jsSource = await readFile(
    new URL("../../js/src/decompress.ts", import.meta.url),
    "utf8",
  );

  assert.match(archiveSource, /import\([\s\S]*\.\.\/\.\.\/\.\.\/js\/src\/decompress\.js/);
  assert.doesNotMatch(archiveSource, /from ["']fzstd["']|import\(["']fzstd["']\)/);
  assert.match(jsSource, /await import\(["']fzstd["']\)/);
  assert.doesNotMatch(jsSource, /^import .* from ["']fzstd["'];?$/m);
});
