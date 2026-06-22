/**
 * Portable zstd fixture helpers — version-independent test fixtures. [OPUS-4.8]
 *
 * `node:zlib`'s native `zstdCompressSync` / `zstdDecompressSync` (and the
 * `{ dictionary }` option) were only added in Node 22.15 / 23.8 and remain
 * STABILITY-1 EXPERIMENTAL — absent on Node 20 (where the import is
 * `undefined`, so every fixture call throws `TypeError`) and emitting a
 * different frame-header shape across Node versions (nodejs/node#58392: sync
 * frames carry no Content_Size, streaming frames do). Both made the
 * compressed/dictionary JS tests Node-version-coupled (bead sq-fz8s).
 *
 * Instead of compressing through that experimental native API, we synthesise
 * the zstd frames the tests need directly, using ONLY `Raw_Block`s (stored,
 * uncompressed payload) per RFC 8878 §3.1. Such frames:
 *   - are valid zstd that the bundled pure-JS `fzstd` decoder reads (the exact
 *     decode path the library ships), and that `node:zlib` would also accept;
 *   - let us embed an arbitrary `Dictionary_ID` in the frame header, exercising
 *     `parseZstdDictId`; and
 *   - need NO dictionary to decode (raw blocks reference none), so the
 *     dictionary-fetch protocol — which keys off the `Sparq-Dictionary`
 *     response header, not the frame bytes — is exercised identically.
 *
 * This keeps full coverage on EVERY supported Node (>=18) with no native zstd
 * and no extra dependency. gzip fixtures still use `node:zlib` (`gzipSync` is
 * stable across all supported versions and is not implicated in sq-fz8s).
 */

const ZSTD_MAGIC = [0x28, 0xb5, 0x2f, 0xfd]; // RFC 8878 §3.1.1 (LE on the wire)
const MAX_RAW_BLOCK = 128 * 1024; // §3.1.1.2.3 Block_Maximum_Size for our window

/** Encodes `dictId` (a non-negative int) into a header field + its flag (0..3). */
function dictionaryIdField(dictId) {
  if (dictId <= 0) return { flag: 0, bytes: [] };
  if (dictId < 0x100) return { flag: 1, bytes: [dictId & 0xff] };
  if (dictId < 0x10000) return { flag: 2, bytes: [dictId & 0xff, (dictId >> 8) & 0xff] };
  return {
    flag: 3,
    bytes: [dictId & 0xff, (dictId >> 8) & 0xff, (dictId >> 16) & 0xff, (dictId >>> 24) & 0xff],
  };
}

/** Frame_Content_Size field + its FCS_Field_flag (0 ⇒ 1 byte, 2 ⇒ 4 bytes). */
function contentSizeField(length) {
  if (length < 0x100) return { flag: 0, bytes: [length & 0xff] };
  return {
    flag: 2,
    bytes: [length & 0xff, (length >> 8) & 0xff, (length >> 16) & 0xff, (length >>> 24) & 0xff],
  };
}

/**
 * A single zstd frame carrying `payload` in `Raw_Block`s, optionally naming a
 * `dictId` in the frame header. Decodes through fzstd or any conformant zstd
 * reader without a dictionary.
 *
 * @param {Uint8Array | Buffer} payload
 * @param {{ dictId?: number }} [opts]
 * @returns {Uint8Array}
 */
export function zstdRawFrame(payload, { dictId = 0 } = {}) {
  const data = Uint8Array.from(payload);
  const did = dictionaryIdField(dictId);
  const fcs = contentSizeField(data.length);
  // Frame_Header_Descriptor: FCS_flag<<6 | Single_Segment<<5 | (unused 0) |
  // Content_Checksum 0 | Dictionary_ID_flag. Single_Segment=1 keeps the window
  // implicit and forces the Content_Size field to be present (no separate
  // Window_Descriptor byte then).
  const fhd = (fcs.flag << 6) | (1 << 5) | did.flag;

  // The frame is at least one block even for a zero-length payload (an empty
  // Raw_Block with Last_Block set). Each block contributes a 3-byte header plus
  // its raw bytes; size the output exactly so we can write it with .set()
  // rather than spreading chunk bytes as function args — a >=128 KiB chunk
  // spread into Array.push overflows the call stack (RangeError, bead sq-3lh7).
  const blockCount = Math.max(1, Math.ceil(data.length / MAX_RAW_BLOCK));
  const header = [...ZSTD_MAGIC, fhd, ...did.bytes, ...fcs.bytes];
  const frame = new Uint8Array(header.length + blockCount * 3 + data.length);
  frame.set(header, 0);

  let pos = header.length;
  let offset = 0;
  do {
    const end = Math.min(offset + MAX_RAW_BLOCK, data.length);
    const chunk = data.subarray(offset, end);
    const lastBlock = end >= data.length ? 1 : 0;
    // Block_Header (3 bytes LE): Block_Size<<3 | Block_Type<<1 | Last_Block.
    // Block_Type 0 = Raw_Block (stored uncompressed).
    const bh = (chunk.length << 3) | (0 << 1) | lastBlock;
    frame[pos] = bh & 0xff;
    frame[pos + 1] = (bh >> 8) & 0xff;
    frame[pos + 2] = (bh >> 16) & 0xff;
    frame.set(chunk, pos + 3);
    pos += 3 + chunk.length;
    offset = end;
  } while (offset < data.length);

  return frame;
}

/** Concatenated independent frames — the RFC 8878 multi-frame wire shape the
 *  `CompressedSink` emits (Node's own `zstdDecompressSync` decodes only the
 *  first; fzstd loops them all). */
export function zstdMultiFrame(payloads) {
  // Copy each frame in with .set() (no spread): frames can each exceed the
  // safe argument count, so flattening via spread would re-introduce sq-3lh7.
  const frames = payloads.map(p => zstdRawFrame(p));
  const total = frames.reduce((n, f) => n + f.length, 0);
  const out = new Uint8Array(total);
  let pos = 0;
  for (const f of frames) {
    out.set(f, pos);
    pos += f.length;
  }
  return out;
}
