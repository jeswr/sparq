// [OPUS-4.8] sq-spita — unit tests for the client-side dataset-archive helpers that let
// the live SPARQL REPL (and the GUI, when it reuses the controls) load a gzip/zip RDF
// dataset by upload OR URL, decompressing entirely in the browser tab. These cover the
// pure, framework-free detection + decompression logic. Fixtures are REAL archives built
// here from the native CompressionStream / a hand-assembled ZIP — no canned blobs and no
// extra dependency. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  sniffArchive,
  archiveCodecFromName,
  archiveCodecFromContentType,
  innerNameForArchive,
  decompressArchive,
} from "../src/lib/dataset-archive.ts";

const SAMPLE_NT =
  '<http://example.org/s> <http://example.org/p> "object value" .\n' +
  '<http://example.org/s> <http://example.org/q> <http://example.org/o2> .\n';

// --- helpers to build real archive fixtures ---------------------------------

async function collect(stream) {
  const chunks = [];
  const reader = stream.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    if (value) chunks.push(value);
  }
  let total = 0;
  for (const c of chunks) total += c.length;
  const out = new Uint8Array(total);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.length;
  }
  return out;
}

/** Real gzip bytes for a string via the native CompressionStream. */
async function gzipBytes(text) {
  const cs = new CompressionStream("gzip");
  const stream = new Blob([text]).stream().pipeThrough(cs);
  return collect(stream);
}

/** Real raw-deflate bytes (ZIP method 8) for a string. */
async function deflateRawBytes(text) {
  const cs = new CompressionStream("deflate-raw");
  const stream = new Blob([text]).stream().pipeThrough(cs);
  return collect(stream);
}

// Minimal single-member ZIP writer (DEFLATE or STORED). Enough to exercise the reader;
// the real-world archives we load are produced by `zip`/`7z`, which this mirrors.
import { deflateRawSync } from "node:zlib";

function crc32(buf) {
  // Standard CRC-32 (poly 0xEDB88320), no node crc32 dependency for portability.
  let crc = ~0;
  for (let i = 0; i < buf.length; i++) {
    crc ^= buf[i];
    for (let k = 0; k < 8; k++) crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
  }
  return (~crc) >>> 0;
}

function makeZip(members) {
  // members: [{ name, text, method }]  method 0 = stored, 8 = deflate
  const enc = new TextEncoder();
  const localParts = [];
  const central = [];
  let offset = 0;
  for (const m of members) {
    const nameBytes = enc.encode(m.name);
    const raw = enc.encode(m.text);
    const compressed =
      m.method === 8 ? new Uint8Array(deflateRawSync(raw)) : raw;
    const crc = crc32(raw);
    const lfh = new DataView(new ArrayBuffer(30));
    lfh.setUint32(0, 0x04034b50, true);
    lfh.setUint16(4, 20, true); // version needed
    lfh.setUint16(6, 0, true); // flags
    lfh.setUint16(8, m.method, true);
    lfh.setUint16(10, 0, true); // time
    lfh.setUint16(12, 0, true); // date
    lfh.setUint32(14, crc, true);
    lfh.setUint32(18, compressed.length, true);
    lfh.setUint32(22, raw.length, true);
    lfh.setUint16(26, nameBytes.length, true);
    lfh.setUint16(28, 0, true); // extra len
    const localHeaderOffset = offset;
    localParts.push(new Uint8Array(lfh.buffer), nameBytes, compressed);
    offset += 30 + nameBytes.length + compressed.length;

    const cdh = new DataView(new ArrayBuffer(46));
    cdh.setUint32(0, 0x02014b50, true);
    cdh.setUint16(4, 20, true);
    cdh.setUint16(6, 20, true);
    cdh.setUint16(8, 0, true);
    cdh.setUint16(10, m.method, true);
    cdh.setUint16(12, 0, true);
    cdh.setUint16(14, 0, true);
    cdh.setUint32(16, crc, true);
    cdh.setUint32(20, compressed.length, true);
    cdh.setUint32(24, raw.length, true);
    cdh.setUint16(28, nameBytes.length, true);
    cdh.setUint16(30, 0, true);
    cdh.setUint16(32, 0, true);
    cdh.setUint16(34, 0, true);
    cdh.setUint16(36, 0, true);
    cdh.setUint32(38, 0, true);
    cdh.setUint32(42, localHeaderOffset, true);
    central.push({ header: new Uint8Array(cdh.buffer), name: nameBytes });
  }
  const localBytes = concat(localParts);
  const cdParts = [];
  for (const c of central) cdParts.push(c.header, c.name);
  const cdBytes = concat(cdParts);
  const eocd = new DataView(new ArrayBuffer(22));
  eocd.setUint32(0, 0x06054b50, true);
  eocd.setUint16(8, members.length, true);
  eocd.setUint16(10, members.length, true);
  eocd.setUint32(12, cdBytes.length, true);
  eocd.setUint32(16, localBytes.length, true);
  eocd.setUint16(20, 0, true);
  return concat([localBytes, cdBytes, new Uint8Array(eocd.buffer)]);
}

function concat(arrs) {
  let total = 0;
  for (const a of arrs) total += a.length;
  const out = new Uint8Array(total);
  let off = 0;
  for (const a of arrs) {
    out.set(a, off);
    off += a.length;
  }
  return out;
}

// --- detection tests --------------------------------------------------------

test("sniffArchive recognises gzip and zip magic numbers", async () => {
  const gz = await gzipBytes("x");
  assert.equal(sniffArchive(gz), "gzip");
  const zip = makeZip([{ name: "a.nt", text: "x", method: 0 }]);
  assert.equal(sniffArchive(zip), "zip");
  // Plain text is not an archive.
  assert.equal(sniffArchive(new TextEncoder().encode("plain")), undefined);
  assert.equal(sniffArchive(new Uint8Array([])), undefined);
});

test("archiveCodecFromName maps archive extensions (incl. query/hash strip)", () => {
  assert.equal(archiveCodecFromName("dataset.nt.gz"), "gzip");
  assert.equal(archiveCodecFromName("dump.zip"), "zip");
  assert.equal(archiveCodecFromName("dump.zip?x=1#frag"), "zip");
  assert.equal(archiveCodecFromName("data.ttl"), undefined);
  assert.equal(archiveCodecFromName("data"), undefined);
});

test("archiveCodecFromContentType handles common server media types", () => {
  assert.equal(archiveCodecFromContentType("application/gzip"), "gzip");
  assert.equal(archiveCodecFromContentType("application/x-gzip; charset=x"), "gzip");
  assert.equal(archiveCodecFromContentType("application/zip"), "zip");
  assert.equal(archiveCodecFromContentType("text/turtle"), undefined);
  assert.equal(archiveCodecFromContentType(null), undefined);
});

test("innerNameForArchive strips the archive suffix for format guessing", () => {
  assert.equal(innerNameForArchive("dataset.nt.gz"), "dataset.nt");
  assert.equal(innerNameForArchive("watdiv.ttl.GZ"), "watdiv.ttl");
  assert.equal(innerNameForArchive("dump.zip"), "dump");
  assert.equal(innerNameForArchive("dump.zip?download=1"), "dump");
  // A .tgz holds a tar with no single inner RDF name -> null (force fallback).
  assert.equal(innerNameForArchive("dump.tgz"), null);
  // No recognised archive suffix: returned unchanged.
  assert.equal(innerNameForArchive("data.ttl"), "data.ttl");
});

// --- decompression tests ----------------------------------------------------

test("decompressArchive round-trips a real gzip payload to text + inner name", async () => {
  const gz = await gzipBytes(SAMPLE_NT);
  const { text, innerName } = await decompressArchive(gz, "umbc.nt.gz", "gzip");
  assert.equal(text, SAMPLE_NT);
  assert.equal(innerName, "umbc.nt");
});

test("decompressArchive sniffs the codec when none is passed", async () => {
  const gz = await gzipBytes(SAMPLE_NT);
  const { text } = await decompressArchive(gz, "umbc.nt.gz");
  assert.equal(text, SAMPLE_NT);
});

test("decompressArchive inflates a DEFLATE zip member and reports its name", async () => {
  const zip = makeZip([{ name: "watdiv.nt", text: SAMPLE_NT, method: 8 }]);
  const { text, innerName } = await decompressArchive(zip, "watdiv.zip");
  assert.equal(text, SAMPLE_NT);
  assert.equal(innerName, "watdiv.nt");
});

test("decompressArchive copies a STORED (method 0) zip member", async () => {
  const zip = makeZip([{ name: "data.ttl", text: SAMPLE_NT, method: 0 }]);
  const { text, innerName } = await decompressArchive(zip, "data.zip");
  assert.equal(text, SAMPLE_NT);
  assert.equal(innerName, "data.ttl");
});

test("decompressArchive picks the first RDF member from a multi-file zip", async () => {
  const zip = makeZip([
    { name: "README.txt", text: "not rdf", method: 8 },
    { name: "graph.nt", text: SAMPLE_NT, method: 8 },
  ]);
  const { text, innerName } = await decompressArchive(zip, "bundle.zip");
  assert.equal(text, SAMPLE_NT);
  assert.equal(innerName, "graph.nt");
});

test("decompressArchive falls back to the first file when no member looks like RDF", async () => {
  const zip = makeZip([{ name: "dump.dat", text: SAMPLE_NT, method: 0 }]);
  const { text, innerName } = await decompressArchive(zip, "dump.zip");
  assert.equal(text, SAMPLE_NT);
  assert.equal(innerName, "dump.dat");
});

test("decompressArchive rejects an unrecognised payload with a clear error", async () => {
  await assert.rejects(
    () => decompressArchive(new TextEncoder().encode("not compressed"), "x.bin"),
    /Unrecognised compressed payload/,
  );
});

test("decompressArchive rejects an unsupported zip compression method", async () => {
  // method 12 = bzip2 (not supported by the native DecompressionStream path).
  const zip = makeZip([{ name: "a.nt", text: SAMPLE_NT, method: 0 }]);
  // Patch the central-directory + local method bytes to 12 to simulate bzip2.
  // Local header method @ offset 8; central header is later — patch both occurrences.
  const patched = zip.slice();
  const dv = new DataView(patched.buffer);
  dv.setUint16(8, 12, true); // local file header method
  // find central dir header signature and patch its method (offset +10)
  for (let i = 0; i + 4 <= patched.length; i++) {
    if (dv.getUint32(i, true) === 0x02014b50) {
      dv.setUint16(i + 10, 12, true);
      break;
    }
  }
  await assert.rejects(
    () => decompressArchive(patched, "a.zip"),
    /Unsupported zip compression method 12/,
  );
});
