import assert from 'node:assert/strict';
import { test } from 'node:test';
import { gzipSync, zstdCompressSync } from 'node:zlib';
import { decompress, decompressToString, sniffCodec, SparqStore } from '../dist/index.js';

const NT = [
  '<http://ex/a> <http://ex/p> "one" .\n',
  '<http://ex/b> <http://ex/p> "two" .\n',
  '<http://ex/c> <http://ex/p> "three" .\n',
];
const FULL = NT.join('');
const bytes = s => new Uint8Array(Buffer.from(s));

test('sniffCodec recognises zstd, skippable-frame zstd, gzip, and rejects junk', () => {
  assert.equal(sniffCodec(new Uint8Array(zstdCompressSync(bytes(FULL)))), 'zstd');
  assert.equal(sniffCodec(new Uint8Array(gzipSync(bytes(FULL)))), 'gzip');
  // a skippable frame (magic 0x184D2A50, LE on the wire) may legally lead a stream
  assert.equal(sniffCodec(new Uint8Array([0x50, 0x2a, 0x4d, 0x18, 0, 0, 0, 0])), 'zstd');
  assert.equal(sniffCodec(bytes('<http://ex/a> …')), undefined);
});

test('.nt.zst ingest: single zstd frame', async () => {
  const store = await SparqStore.fromCompressed(new Uint8Array(zstdCompressSync(bytes(FULL))), 'ntriples');
  assert.equal(store.size, 3);
  assert.equal(store.queryBoolean('ASK { <http://ex/b> ?p "two" }'), true);
});

test('multi-frame zstd (the CompressedSink wire shape) decodes fully', async () => {
  // Independently compressed chunks concatenated in order — RFC 8878 multi-frame.
  const frames = Buffer.concat(NT.map(line => zstdCompressSync(bytes(line))));
  assert.equal(await decompressToString(new Uint8Array(frames)), FULL);
  const store = await SparqStore.fromCompressed(new Uint8Array(frames), 'ntriples');
  assert.equal(store.size, 3); // NOT first-frame-only (Node's own zstd truncates here)
});

test('.gz ingest (multi-member gzip decodes fully in Node)', async () => {
  const single = new Uint8Array(gzipSync(bytes(FULL)));
  const store = await SparqStore.fromCompressed(single, 'ntriples');
  assert.equal(store.size, 3);
  // RFC 1952 multi-member stream: node:zlib loops members (browsers would not —
  // documented; servers emit single-member gzip for browser consumers).
  const multi = new Uint8Array(Buffer.concat(NT.map(line => gzipSync(bytes(line)))));
  assert.equal(await decompressToString(multi), FULL);
});

test('explicit codec, dataset pass-through, and clear junk error', async () => {
  const nq = '<http://ex/s> <http://ex/p> "in-g" <http://ex/g> .\n';
  const store = await SparqStore.fromCompressed(new Uint8Array(zstdCompressSync(bytes(nq))), 'nquads', {
    codec: 'zstd',
    dataset: true,
  });
  assert.equal(store.queryBoolean('ASK { GRAPH <http://ex/g> { ?s ?p "in-g" } }'), true);

  await assert.rejects(decompress(bytes('plain text, no magic')), /zstd or gzip magic/);
  // an explicit wrong codec also fails loudly, not silently
  await assert.rejects(decompress(new Uint8Array(gzipSync(bytes(FULL))), 'zstd'), /./);
});
