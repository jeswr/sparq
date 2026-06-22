import assert from 'node:assert/strict';
import { test } from 'node:test';
import { decompress as fzstdDecompress } from 'fzstd';
import {
  dictIdOf,
  parseZstdDictId,
  SparqDictionaryClient,
  SPARQ_DICTIONARY_CURRENT_HEADER,
  SPARQ_DICTIONARY_HEADER,
  verifyDictId,
} from '../dist/index.js';
// zstd fixtures are synthesised in pure JS (Raw_Block frames, dictId 0) rather
// than via node:zlib's native zstdCompressSync, which is absent on Node 20 and
// experimental/version-coupled thereafter (bead sq-fz8s). The protocol logic is
// driven by the Sparq-Dictionary response header, not the frame's own bytes, so
// raw frames exercise both the plain and dictionary-decode paths identically.
import { zstdRawFrame } from './helpers/zstd-fixtures.mjs';

const enc = new TextEncoder();
const dec = new TextDecoder();

// A "vocabulary" raw-content dictionary and a typical small SPARQL-JSON body.
const DICT = enc.encode(
  '{"head":{"vars":["s","p","o"]},"results":{"bindings":[{"type":"uri","value":"http://example.org/vocab#"}',
);
const BODY = '{"head":{"vars":["o"]},"results":{"bindings":[{"o":{"type":"uri","value":"http://example.org/vocab#x"}}]}}';

/**
 * A dict-capable decoder stand-in (what zstd-wasm provides in browsers). The
 * client routes a response to this hook purely because the `Sparq-Dictionary`
 * header named a dictionary; the `dictionary` argument is therefore threaded
 * through (and asserted present) even though our Raw_Block fixtures reference
 * none — matching the real contract, in which a dict-capable backend is what
 * makes the dictionary path engage at all.
 */
const dictAwareDecoder = (body, dictionary) => {
  assert.ok(dictionary instanceof Uint8Array && dictionary.length > 0);
  return fzstdDecompress(body);
};

/**
 * A mock origin implementing the server side of the protocol: dictionary
 * compression ONLY when the request proves it holds the current dictionary,
 * `Sparq-Dictionary` echoing the dictionary used, `Sparq-Dictionary-Current`
 * always advertising the newest one, and `GET /dictionary/{id}` serving the
 * raw bytes. Truncated (22-char) ids, as the design allows.
 */
async function mockOrigin({ corruptDictionary = false } = {}) {
  const dictId = (await dictIdOf(DICT)).slice(0, 22);
  const log = [];
  const fetchImpl = async (url, init) => {
    const u = new URL(url, 'https://sparq.example');
    if (u.pathname === `/dictionary/${encodeURIComponent(dictId)}`) {
      log.push('dict-fetch');
      const bytes = corruptDictionary ? new Uint8Array([1, 2, 3]) : DICT;
      return new Response(bytes, { headers: { 'Cache-Control': 'public, max-age=31536000, immutable' } });
    }
    const held = new Headers(init?.headers).get(SPARQ_DICTIONARY_HEADER) ?? '';
    const headers = { [SPARQ_DICTIONARY_CURRENT_HEADER]: dictId };
    if (held.split(/,\s*/).includes(dictId)) {
      log.push('dict-compressed');
      headers[SPARQ_DICTIONARY_HEADER] = dictId;
      // The Sparq-Dictionary header — not the frame bytes — drives the client
      // onto the dict-capable decode hook, so a plain Raw_Block fixture suffices.
      return new Response(zstdRawFrame(enc.encode(BODY)), { headers });
    }
    log.push('plain');
    return new Response(zstdRawFrame(enc.encode(BODY)), { headers });
  };
  return { dictId, log, fetchImpl };
}

test('content addressing: dictIdOf / truncated verifyDictId', async () => {
  const id = await dictIdOf(DICT);
  assert.match(id, /^[A-Za-z0-9_-]{43}$/); // base64url SHA-256, unpadded
  assert.ok(await verifyDictId(DICT, id));
  assert.ok(await verifyDictId(DICT, id.slice(0, 22))); // server-truncated form
  assert.ok(!(await verifyDictId(DICT, 'bogus')));
  assert.ok(!(await verifyDictId(new Uint8Array([9]), id)));
});

test('protocol round-trip: plain first request, warmed-up dictionary on the second', async () => {
  const { dictId, log, fetchImpl } = await mockOrigin();
  const client = new SparqDictionaryClient({ fetch: fetchImpl, decodeWithDictionary: dictAwareDecoder });

  // First request: nothing held -> plain zstd, decoded by the bundled fzstd.
  const first = await client.fetch('https://sparq.example/sparql?query=...');
  assert.equal(dec.decode(first.body), BODY);
  assert.equal(first.dictionary, undefined);

  // The advertised current dictionary was warmed up in the background.
  await client.idle();
  assert.deepEqual(client.dictionaryIds, [dictId]);

  // Second request: the held id is advertised, the body comes back
  // dictionary-compressed and decodes through the hook.
  const second = await client.fetch('https://sparq.example/sparql?query=...');
  assert.equal(dec.decode(second.body), BODY);
  assert.equal(second.dictionary, dictId);
  assert.deepEqual(log, ['plain', 'dict-fetch', 'dict-compressed']);
});

test('without a dict-capable decoder the client never advertises (stays plain)', async () => {
  const { log, fetchImpl } = await mockOrigin();
  const client = new SparqDictionaryClient({ fetch: fetchImpl }); // no hook
  await client.fetch('https://sparq.example/sparql');
  await client.idle();
  const again = await client.fetch('https://sparq.example/sparql');
  assert.equal(dec.decode(again.body), BODY); // still decodes (plain fzstd path)
  assert.equal(again.dictionary, undefined);
  assert.deepEqual(log.filter(e => e === 'dict-compressed'), []);
});

test('a corrupted dictionary fails content verification and is never trusted', async () => {
  const { fetchImpl } = await mockOrigin({ corruptDictionary: true });
  const client = new SparqDictionaryClient({ fetch: fetchImpl, decodeWithDictionary: dictAwareDecoder });
  await client.fetch('https://sparq.example/sparql');
  await client.idle();
  assert.deepEqual(client.dictionaryIds, []); // rejected, not cached
  // and the next request therefore stays plain
  const next = await client.fetch('https://sparq.example/sparql');
  assert.equal(next.dictionary, undefined);
});

test('addDictionary seeds a persisted dictionary (verified against the claimed id)', async () => {
  const { dictId, log, fetchImpl } = await mockOrigin();
  const client = new SparqDictionaryClient({ fetch: fetchImpl, decodeWithDictionary: dictAwareDecoder });
  await client.addDictionary(DICT, dictId);
  const res = await client.fetch('https://sparq.example/sparql');
  assert.equal(res.dictionary, dictId); // dictionary path engaged on the FIRST request
  assert.deepEqual(log, ['dict-compressed']);
  await assert.rejects(client.addDictionary(new Uint8Array([1]), dictId), /hash/);
});

test('non-compressed and platform-decoded bodies pass through untouched', async () => {
  const fetchImpl = async () => new Response('plain text body', { headers: {} });
  const client = new SparqDictionaryClient({ fetch: fetchImpl });
  const res = await client.fetch('https://sparq.example/anything');
  assert.equal(dec.decode(res.body), 'plain text body');
});

test('parseZstdDictId reads the frame-header Dictionary_ID', () => {
  // A frame that names no dictionary reads as 0 (= "names none"), like the
  // raw-content dictionaries this protocol uses.
  assert.equal(parseZstdDictId(zstdRawFrame(enc.encode(BODY))), 0);
  // A frame whose header DOES carry a Dictionary_ID reads it back (our helper
  // emits a single-segment 2-byte DID field for 0xBEEF here).
  assert.equal(parseZstdDictId(zstdRawFrame(enc.encode(BODY), { dictId: 0xbeef })), 0xbeef);
  // Hand-crafted RFC 8878 header: magic, FHD with DID_Field_Size=2 (0x02),
  // not single-segment -> 1 Window_Descriptor byte, then DID 0xBEEF LE.
  assert.equal(parseZstdDictId(new Uint8Array([0x28, 0xb5, 0x2f, 0xfd, 0x02, 0x00, 0xef, 0xbe])), 0xbeef);
  // Single-segment variant: no Window_Descriptor byte, 1-byte DID.
  assert.equal(parseZstdDictId(new Uint8Array([0x28, 0xb5, 0x2f, 0xfd, 0x21, 0x07])), 7);
  assert.equal(parseZstdDictId(new Uint8Array([1, 2, 3, 4, 5])), null); // not zstd
});
