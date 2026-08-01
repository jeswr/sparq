/**
 * Compressed-ingest helpers: decode `.zst` (zstd, including the multi-frame
 * streams sparq's `CompressedSink` emits — RFC 8878 allows concatenated
 * frames) and `.gz` (gzip) payloads in both Node and the browser, so
 * compressed RDF can be ingested without native zstd support.
 *
 * - zstd decodes through `fzstd` (pure JS, ~8 kB, dynamically imported so it
 *   never lands in bundles that don't ingest zstd). Verified against
 *   multi-frame streams; zstd *dictionary* frames are NOT supported by fzstd —
 *   use a dict-capable decoder via the dictionary-fetch client's hook.
 * - gzip decodes through `node:zlib` in Node (which loops multi-member
 *   streams) and `DecompressionStream('gzip')` in the browser. NOTE (measured
 *   in research/custom-parsers-D4 §3): browsers silently truncate MULTI-member
 *   gzip to the first member — only serve browsers single-member gzip.
 */

export type CompressionCodec = 'zstd' | 'gzip';

const ZSTD_MAGIC = 0xfd2fb528; // RFC 8878 §3.1.1 frame magic (LE)
const ZSTD_SKIPPABLE_MASK = 0xfffffff0;
const ZSTD_SKIPPABLE_MAGIC = 0x184d2a50; // §3.1.2: 0x184D2A50..5F

/** The compression codec a payload's magic number announces, if recognised. */
export function sniffCodec(bytes: Uint8Array): CompressionCodec | undefined {
  if (bytes.length >= 2 && bytes[0] === 0x1f && bytes[1] === 0x8b) return 'gzip';
  if (bytes.length >= 4) {
    const magic = (bytes[0]! | (bytes[1]! << 8) | (bytes[2]! << 16) | (bytes[3]! << 24)) >>> 0;
    if (magic === ZSTD_MAGIC || (magic & ZSTD_SKIPPABLE_MASK) === ZSTD_SKIPPABLE_MAGIC) return 'zstd';
  }
  return undefined;
}

declare const process: { versions?: { node?: string } } | undefined;

const isNode = (): boolean => typeof process !== 'undefined' && Boolean(process?.versions?.node);

async function gunzip(bytes: Uint8Array): Promise<Uint8Array> {
  if (isNode()) {
    // node:zlib loops gzip members, so multi-member streams decode fully.
    // [GPT-5.6] #1046 — browser bundles reuse this module for its zstd branch. Keep the
    // runtime-guarded Node builtin external so webpack does not try to compile `node:zlib`
    // into that browser-only lazy chunk; native Node import() semantics are unchanged.
    const { gunzipSync } = await import(/* webpackIgnore: true */ 'node:zlib');
    return new Uint8Array(gunzipSync(bytes));
  }
  // Browser: DecompressionStream (single-member gzip only — see module doc).
  const stream = new Blob([bytes as BlobPart]).stream().pipeThrough(new DecompressionStream('gzip'));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

/**
 * Decompresses a payload. `codec` defaults to sniffing the magic number;
 * pass it explicitly for payloads from unlabelled sources you already trust.
 */
export async function decompress(bytes: Uint8Array, codec?: CompressionCodec): Promise<Uint8Array> {
  const resolved = codec ?? sniffCodec(bytes);
  switch (resolved) {
    case 'gzip':
      return gunzip(bytes);
    case 'zstd': {
      const { decompress: fzstdDecompress } = await import('fzstd');
      return fzstdDecompress(bytes);
    }
    default:
      throw new Error('unrecognised compressed payload: expected a zstd or gzip magic number (pass codec explicitly?)');
  }
}

const utf8 = /* @__PURE__ */ new TextDecoder();

/** [`decompress`] straight to a UTF-8 string (the shape RDF parsers ingest). */
export async function decompressToString(bytes: Uint8Array, codec?: CompressionCodec): Promise<string> {
  return utf8.decode(await decompress(bytes, codec));
}
