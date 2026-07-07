// [OPUS-4.8] sq-j50v / sq-ixc3.1 — unit tests for the /surface/data-formats demo helpers: the
// sample catalogue (four text formats + JSON-LD) and the in-tab gzip round-trip. These cover
// the pure, framework-free logic (no React, no wasm); the engine-level parse of each sample is
// covered by the Rust tests + js/test/store.test.mjs. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  FORMAT_SAMPLES,
  isDatasetFormat,
  sampleFor,
  gzipString,
  gunzipToString,
  formatBytes,
  formatFootprintBytes,
} from "../src/lib/data-formats.ts";

test("the catalogue covers the four text formats plus JSON-LD", () => {
  assert.deepEqual(
    FORMAT_SAMPLES.map((f) => f.format),
    ["turtle", "ntriples", "nquads", "trig", "jsonld"],
  );
});

test("every sample is non-empty and carries the right named-graph syntax", () => {
  for (const f of FORMAT_SAMPLES) {
    assert.ok(f.sample.trim().length > 0, `${f.format} sample is empty`);
  }
  // The quad-bearing formats include the named graph the bead asks for.
  assert.match(sampleFor("nquads").sample, /<http:\/\/example\.org\/social>/);
  assert.match(sampleFor("trig").sample, /ex:social\s*\{/);
  // The triple-only formats do NOT (no 4th term / no graph block).
  assert.doesNotMatch(sampleFor("turtle").sample, /\{/);
  // The JSON-LD sample is real JSON-LD: it carries a `@context` and a `@graph`.
  assert.match(sampleFor("jsonld").sample, /"@context"/);
  assert.match(sampleFor("jsonld").sample, /"@graph"/);
});

test("isDatasetFormat is true exactly for the quad-bearing formats", () => {
  assert.equal(isDatasetFormat("nquads"), true);
  assert.equal(isDatasetFormat("trig"), true);
  // JSON-LD can carry named graphs (`@graph` under an outer `@id`), so it routes through
  // loadDataset too.
  assert.equal(isDatasetFormat("jsonld"), true);
  assert.equal(isDatasetFormat("turtle"), false);
  assert.equal(isDatasetFormat("ntriples"), false);
});

test("sampleFor returns undefined for an unknown format", () => {
  // @ts-expect-error — deliberately probing the not-a-format branch.
  assert.equal(sampleFor("hdt"), undefined);
});

test("gzip round-trips every sample byte-for-byte through the native codec", async () => {
  // The live compressed-ingest path is gunzip(gzip(text)) === text — proving the demo's
  // decode side is a REAL gzip stream (CompressionStream -> DecompressionStream), not a
  // canned blob. gzip is the one codec the browser decodes natively.
  for (const f of FORMAT_SAMPLES) {
    const gz = await gzipString(f.sample);
    assert.ok(gz instanceof Uint8Array && gz.byteLength > 0);
    const back = await gunzipToString(gz);
    assert.equal(back, f.sample, `${f.format} did not gzip round-trip`);
  }
});

test("gunzipToString rejects on non-gzip bytes", async () => {
  await assert.rejects(() => gunzipToString(new Uint8Array([1, 2, 3, 4])));
});

test("formatBytes renders B under 1 KiB and KB above", () => {
  assert.equal(formatBytes(0), "0 B");
  assert.equal(formatBytes(512), "512 B");
  assert.equal(formatBytes(2048), "2.0 KB");
});

// [SONNET-4.6] sq-su1oe (#820) — the unintrusive store-stats line formats the
// in-memory footprint + ingest source size in B / KB / MB.
test("formatFootprintBytes carries B / KB / MB magnitudes", () => {
  assert.equal(formatFootprintBytes(0), "0 B");
  assert.equal(formatFootprintBytes(512), "512 B");
  assert.equal(formatFootprintBytes(2048), "2.0 KB");
  assert.equal(formatFootprintBytes(5 * 1024 * 1024), "5.0 MB");
  // The 1 KiB / 1 MiB boundaries round up into the next magnitude rather than showing "1024 B".
  assert.equal(formatFootprintBytes(1024), "1.0 KB");
  assert.equal(formatFootprintBytes(1024 * 1024), "1.0 MB");
});

test("formatFootprintBytes clamps a bad reading to 0 B (never a fabricated figure)", () => {
  assert.equal(formatFootprintBytes(-1), "0 B");
  assert.equal(formatFootprintBytes(Number.NaN), "0 B");
  assert.equal(formatFootprintBytes(Number.POSITIVE_INFINITY), "0 B");
});
