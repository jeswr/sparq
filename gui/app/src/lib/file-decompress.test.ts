// [SONNET-4.6] sq-1y04h — unit tests for the browser web-upload decompression shim.
//
// Load-bearing property under test: importing a COMPRESSED fixture via `maybeDecompressFile`
// yields the SAME decoded text as the UNCOMPRESSED original. The mutation check (passing
// compressed bytes through without decompression) produces DIFFERENT text — confirming that
// the decompress step is non-vacuous.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";
import { gzipSync } from "node:zlib";

import { fetchRdfDocument, maybeDecompressFile } from "./file-decompress.js";
import { formatFromContentType, guessFormat } from "./rdf-format.js";

// ── Sample RDF content ────────────────────────────────────────────────────────────────────────

const SAMPLE_NT =
  "<http://example.org/s> <http://example.org/p> <http://example.org/o> .\n" +
  '<http://example.org/s> <http://example.org/q> "literal value" .\n';

// ── Minimal browser `File` shim (Node has no DOM File) ───────────────────────────────────────
//
// `maybeDecompressFile` only calls `file.arrayBuffer()` and reads `file.name` — the shim
// only needs those two surfaces to be correct. The real browser `File` class's `File.text()`
// is NOT called by the function under test (the whole point of this PR is to bypass it).

class FakeFile {
  readonly name: string;
  readonly size: number;
  private readonly _bytes: Uint8Array;
  constructor(name: string, bytes: Uint8Array) {
    this.name = name;
    this.size = bytes.byteLength;
    this._bytes = bytes;
  }
  arrayBuffer(): Promise<ArrayBuffer> {
    return Promise.resolve(this._bytes.buffer.slice(this._bytes.byteOffset, this._bytes.byteOffset + this._bytes.byteLength) as ArrayBuffer);
  }
  // text() is intentionally NOT on this shim — it should never be called for compressed files.
  text(): Promise<string> {
    throw new Error("FakeFile.text() called — this proves the decompression path was bypassed (mutation check).");
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────────────────────

test("[SONNET-4.6] sq-1y04h: gzip-compressed .nt.gz file is decompressed to original text", async () => {
  const compressed = gzipSync(Buffer.from(SAMPLE_NT, "utf8"));
  const file = new FakeFile("dataset.nt.gz", new Uint8Array(compressed));

  // [SONNET-4.6] sq-1y04h — DecompressionStream is available in Node 18+.
  const result = await maybeDecompressFile(file as unknown as File);

  assert.strictEqual(result.wasDecompressed, true, "wasDecompressed must be true for a .gz file");
  assert.strictEqual(result.codec, "gzip", "codec must be 'gzip'");
  assert.strictEqual(result.text, SAMPLE_NT, "decompressed text must exactly match the uncompressed original");
  // effectiveName strips the .gz suffix so guessFormat() sees the inner RDF extension.
  assert.strictEqual(result.effectiveName, "dataset.nt", "effectiveName must be the suffix-stripped inner name");
});

test("[SONNET-4.6] sq-1y04h: non-vacuity — bypassing decompression yields DIFFERENT text (mutation check)", () => {
  // If we naively decode the gzip bytes as UTF-8 (what File.text() does), we get garbled output.
  const compressed = gzipSync(Buffer.from(SAMPLE_NT, "utf8"));
  const decoder = new TextDecoder("utf-8", { fatal: false });
  const garbled = decoder.decode(compressed);

  // The garbled text must NOT equal the original N-Triples. If this assertion fails the
  // test would be vacuous: passing compressed bytes as text would still "work", meaning
  // the decompression step would provide no actual value.
  assert.notStrictEqual(
    garbled,
    SAMPLE_NT,
    "Compressed bytes decoded as UTF-8 (without decompression) must NOT equal the original — " +
    "this proves the decompression step is load-bearing, not a no-op",
  );
});

test("[SONNET-4.6] sq-1y04h: uncompressed .nt file passes through unchanged (regression guard)", async () => {
  const bytes = new TextEncoder().encode(SAMPLE_NT);
  const file = new FakeFile("dataset.nt", bytes);

  const result = await maybeDecompressFile(file as unknown as File);

  assert.strictEqual(result.wasDecompressed, false, "wasDecompressed must be false for a plain text file");
  assert.strictEqual(result.codec, undefined, "codec must be undefined for an uncompressed file");
  assert.strictEqual(result.text, SAMPLE_NT, "uncompressed text must be returned unchanged");
  assert.strictEqual(result.effectiveName, "dataset.nt", "effectiveName must equal the original filename");
});

test("[SONNET-4.6] sq-1y04h: gzip by magic bytes (file named .nt but gzip-compressed)", async () => {
  // decompressDatasetBytes probes magic bytes first; even a misnamed file is decompressed.
  const compressed = gzipSync(Buffer.from(SAMPLE_NT, "utf8"));
  const file = new FakeFile("dataset.nt", new Uint8Array(compressed));

  const result = await maybeDecompressFile(file as unknown as File);

  assert.strictEqual(result.wasDecompressed, true, "magic-bytes probe must detect gzip regardless of extension");
  assert.strictEqual(result.text, SAMPLE_NT, "decompressed text must match original");
});

test("[SONNET-4.6] sq-ljc12: bzip2 remains desktop-only in browser ingest", async () => {
  const file = new FakeFile(
    "dataset.nt.bz2",
    new Uint8Array([0x42, 0x5a, 0x68, 0x39]),
  );

  await assert.rejects(
    maybeDecompressFile(file as unknown as File),
    /Bzip2 archives are supported only by the desktop app/,
  );
});

test("[GPT-5.6] sq-n18o5: compressed URL is fetched as binary and decompressed before format detection", async () => {
  const compressed = gzipSync(Buffer.from(SAMPLE_NT, "utf8"));
  let arrayBufferCalls = 0;

  // [GPT-5.6] The response deliberately has no text() method: a regression to text-first fetch
  // fails instead of silently feeding corrupted compressed bytes to the RDF parser.
  const document = await fetchRdfDocument(
    "https://example.org/dataset.nt.gz?download=1",
    async (url) => {
      assert.strictEqual(url, "https://example.org/dataset.nt.gz?download=1");
      return {
        ok: true,
        status: 200,
        statusText: "OK",
        headers: new Headers({ "content-type": "application/gzip" }),
        arrayBuffer: async () => {
          arrayBufferCalls += 1;
          return compressed.buffer.slice(
            compressed.byteOffset,
            compressed.byteOffset + compressed.byteLength,
          ) as ArrayBuffer;
        },
      };
    },
  );

  assert.strictEqual(arrayBufferCalls, 1, "the URL response must be consumed exactly once as binary");
  assert.strictEqual(document.wasDecompressed, true);
  assert.strictEqual(document.codec, "gzip");
  assert.strictEqual(document.text, SAMPLE_NT, "URL decompression must recover the exact RDF text");
  assert.notStrictEqual(
    new TextDecoder("utf-8", { fatal: false }).decode(compressed),
    document.text,
    "mutation check: decoding the fetched archive without decompression must produce different text",
  );
  assert.strictEqual(
    formatFromContentType(document.contentType) ?? guessFormat(document.effectiveName),
    "ntriples",
    "the decompressed inner .nt name must select N-Triples before RDF parse",
  );
});
