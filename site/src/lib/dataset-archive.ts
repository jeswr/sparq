// [OPUS-4.8] sq-spita — client-side decompression of compressed RDF datasets so a
// browser surface (today the /surface/data-formats compressed-ingest demo; the web
// GUI's import drawer is the follow-up consumer) can load a compressed resource by
// file upload OR by URL — e.g. a 10M-triple UMBC `.nt.gz` redirect or a TIB watdiv
// `.zip` — entirely in the browser tab, no server.
//
// These helpers are pure + framework-free (no React, no DOM beyond the standard
// `DecompressionStream` Web API, no wasm), so they unit-test directly under
// `node --test` (Node ≥18 ships `DecompressionStream`). The actual RDF parse still
// happens in the wasm Store — this layer only turns archive BYTES into RDF TEXT.
//
// Codec choice (native-first; anything else is LAZY-loaded):
//   • gzip (`.gz`, magic 1f 8b) — decoded by the native `DecompressionStream("gzip")`.
//     Single-member only: browsers silently truncate multi-member gzip to the first
//     member (measured in research/custom-parsers-D4 §3). Real-world RDF `.gz` dumps
//     (DBpedia/UMBC/LOD) are single-member, so this is the right trade vs. shipping a
//     JS gzip codec.
//   • zip (`.zip`, magic 50 4b) — a minimal central-directory reader that inflates the
//     first RDF-looking member with the native `DecompressionStream("deflate-raw")`
//     (DEFLATE, ZIP method 8) or copies a STORED member (method 0). No JSZip / pako
//     dependency (would add ~95 kB / ~45 kB to the bundle for a feature served by a
//     browser-native API). Encrypted, ZIP64, and other compression methods are
//     rejected with a clear message rather than silently mis-decoding.
//   • zstd (`.zst`, magic 28 b5 2f fd) — [GPT-5.6] #1046: delegated to the existing
//     `js/src/decompress.ts` implementation. That shared path dynamically imports `fzstd`
//     only when zstd is selected, so the decoder remains outside every route's first-load
//     bundle. It handles the multi-frame streams RFC 8878 allows, but not dictionary frames.
//
// bzip2 stays out: there is no native browser codec and no small pure-JS decoder — a
// wasm-blob decoder would be a heavy add for a rarely-shipped format (deferred in
// sq-4ssz1 until someone actually asks).

/** A recognised dataset-archive container codec. */
export type ArchiveCodec = "gzip" | "zip" | "zstd";

/** The result of decompressing an archive: the decoded RDF text plus the inner
 *  member name (used to guess the RDF format — `data.nt.gz` → `data.nt`). */
export interface DecompressedArchive {
  /** The decoded RDF document text. */
  text: string;
  /** The inner file name (archive suffix stripped / zip member name), or `null`
   *  when it cannot be determined — the caller then falls back to its own guess. */
  innerName: string | null;
}

/** Stable error used when bytes do not name one of the browser import codecs. */
export const UNSUPPORTED_ARCHIVE_ERROR =
  "Unrecognised compressed payload: expected a gzip (.gz), zip (.zip), or zstd (.zst) magic number.";

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

// zstd frame magics (RFC 8878 §3.1): a data frame is 28 b5 2f fd little-endian
// (0xFD2FB528); a skippable frame is 0x184D2A50..0x184D2A5F (low nibble free).
const ZSTD_MAGIC = 0xfd2fb528;
const ZSTD_SKIPPABLE_MAGIC = 0x184d2a50;
const ZSTD_SKIPPABLE_MASK = 0xfffffff0;

/** The container codec a payload's magic number announces, or `undefined`. */
export function sniffArchive(bytes: Uint8Array): ArchiveCodec | undefined {
  // gzip: 1f 8b (RFC 1952 §2.3.1).
  if (bytes.length >= 2 && bytes[0] === 0x1f && bytes[1] === 0x8b) return "gzip";
  // zip: "PK\x03\x04" (local file header) or "PK\x05\x06" (empty-archive EOCD).
  if (
    bytes.length >= 4 &&
    bytes[0] === 0x50 &&
    bytes[1] === 0x4b &&
    (bytes[2] === 0x03 || bytes[2] === 0x05 || bytes[2] === 0x07) &&
    (bytes[3] === 0x04 || bytes[3] === 0x06 || bytes[3] === 0x08)
  ) {
    return "zip";
  }
  // zstd: a data frame or a leading skippable frame (RFC 8878 §3.1).
  if (bytes.length >= 4) {
    const magic =
      (bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)) >>> 0;
    if (
      magic === ZSTD_MAGIC ||
      (magic & ZSTD_SKIPPABLE_MASK) >>> 0 === ZSTD_SKIPPABLE_MAGIC
    ) {
      return "zstd";
    }
  }
  return undefined;
}

// Extensions / media types that name an archive container (vs. the inner RDF format).
const GZIP_EXTS = new Set(["gz", "gzip", "tgz"]);
const ZIP_EXTS = new Set(["zip"]);
const ZSTD_EXTS = new Set(["zst", "zstd"]);

const ARCHIVE_CONTENT_TYPES: Record<string, ArchiveCodec> = {
  "application/gzip": "gzip",
  "application/x-gzip": "gzip",
  "application/zip": "zip",
  "application/x-zip-compressed": "zip",
  "application/zstd": "zstd",
  "application/x-zstd": "zstd",
};

/** The container codec a filename/URL extension names, or `undefined`. */
export function archiveCodecFromName(name: string): ArchiveCodec | undefined {
  const ext = name.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "";
  if (GZIP_EXTS.has(ext)) return "gzip";
  if (ZIP_EXTS.has(ext)) return "zip";
  if (ZSTD_EXTS.has(ext)) return "zstd";
  return undefined;
}

/** The container codec a response `Content-Type` names, or `undefined`. */
export function archiveCodecFromContentType(
  contentType: string | null,
): ArchiveCodec | undefined {
  if (!contentType) return undefined;
  const mime = contentType.split(";")[0].trim().toLowerCase();
  return ARCHIVE_CONTENT_TYPES[mime];
}

/**
 * The inner file name for an archive name: strips a trailing
 * `.gz`/`.gzip`/`.zip`/`.zst`/`.zstd` so `dataset.nt.gz` → `dataset.nt` (a name the
 * format guesser understands). `.tgz` maps to `.tar`, which is not an RDF format, so
 * it returns `null` to force a fallback. Returns the input unchanged when it has no
 * recognised archive suffix.
 */
export function innerNameForArchive(name: string): string | null {
  const base = name.split(/[?#]/)[0];
  const lower = base.toLowerCase();
  if (lower.endsWith(".tgz")) return null; // tar inside — no single inner RDF name
  for (const suffix of [".gz", ".gzip", ".zip", ".zst", ".zstd"]) {
    if (lower.endsWith(suffix)) {
      const inner = base.slice(0, base.length - suffix.length);
      return inner.length > 0 ? inner : null;
    }
  }
  return base;
}

// ---------------------------------------------------------------------------
// Decompression
// ---------------------------------------------------------------------------

/** Collects a byte stream into one buffer. */
async function collectStream(
  stream: ReadableStream<Uint8Array>,
): Promise<Uint8Array> {
  const chunks: Uint8Array[] = [];
  let total = 0;
  const reader = stream.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    if (value) {
      chunks.push(value);
      total += value.length;
    }
  }
  const out = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    out.set(c, offset);
    offset += c.length;
  }
  return out;
}

/** Runs raw bytes through a native `DecompressionStream` of the given format. */
async function inflate(
  bytes: Uint8Array,
  format: "gzip" | "deflate-raw" | "deflate",
): Promise<Uint8Array> {
  const ds = new DecompressionStream(format);
  // `Blob` is available in both the browser and Node ≥18; using it (rather than a
  // hand-rolled ReadableStream) keeps this portable across both test + runtime.
  const stream = new Blob([bytes as BlobPart])
    .stream()
    .pipeThrough(ds) as unknown as ReadableStream<Uint8Array>;
  return collectStream(stream);
}

const utf8 = new TextDecoder();

/**
 * Decompresses a recognised dataset archive (`gzip`, `zip`, or `zstd`) to RDF text
 * plus the inner member name. `codec` defaults to sniffing the magic number; `name`
 * (the file/URL name) is used to break the single-stream codecs' "no inner name"
 * ambiguity (a zip carries its own member name). Throws a clear, actionable error for
 * an unrecognised / unsupported payload rather than mis-decoding. The zstd decoder is
 * lazy-loaded on first use by the shared JS package decompressor.
 */
export async function decompressArchive(
  bytes: Uint8Array,
  name: string,
  codec?: ArchiveCodec,
): Promise<DecompressedArchive> {
  const resolved = codec ?? sniffArchive(bytes);
  switch (resolved) {
    case "gzip": {
      const out = await inflate(bytes, "gzip");
      return { text: utf8.decode(out), innerName: innerNameForArchive(name) };
    }
    case "zip":
      return unzipFirstRdfMember(bytes);
    case "zstd": {
      // [GPT-5.6] #1046 — reuse the published JS surface's zstd path instead of
      // maintaining a second site-local `fzstd` import. Both this module and `fzstd`
      // remain literal dynamic split points and load only after a zstd import is invoked.
      const { decompress } = await import(
        /* webpackChunkName: "sparq-js-decompress" */ "../../../js/src/decompress.js"
      );
      const out = await decompress(bytes, "zstd");
      return { text: utf8.decode(out), innerName: innerNameForArchive(name) };
    }
    default:
      throw new Error(UNSUPPORTED_ARCHIVE_ERROR);
  }
}

// ---------------------------------------------------------------------------
// Minimal ZIP reader (central-directory based, DEFLATE + STORED only)
// ---------------------------------------------------------------------------

// ZIP signatures (PKZIP APPNOTE.TXT).
const EOCD_SIG = 0x06054b50; // End of central directory
const CDH_SIG = 0x02014b50; // Central directory file header
const LFH_SIG = 0x04034b50; // Local file header
const ZIP64_EOCD_LOCATOR_SIG = 0x07064b50;

// RDF file extensions we prefer when an archive holds several members. Ordered by how
// commonly compressed RDF dumps ship; the FIRST matching member wins.
const RDF_EXTS = [
  "nt",
  "ntriples",
  "ttl",
  "turtle",
  "nq",
  "nquads",
  "trig",
  "jsonld",
  "json-ld",
  "rdf",
  "n3",
];

interface ZipEntry {
  name: string;
  method: number; // 0 = stored, 8 = deflate
  flags: number;
  localHeaderOffset: number;
  compressedSize: number;
  uncompressedSize: number;
}

function extOf(name: string): string {
  return name.split(".").pop()?.toLowerCase() ?? "";
}

/** A member is a plausible RDF document (by extension), excluding directory entries. */
function isRdfMember(name: string): boolean {
  if (name.endsWith("/")) return false; // directory entry
  return RDF_EXTS.includes(extOf(name));
}

/** Finds the End Of Central Directory record by scanning backwards for its signature
 *  (it sits at the very end, after an optional ≤64 kB comment). */
function findEocd(view: DataView): number {
  // Minimum EOCD size is 22 bytes; comment can be up to 65535 bytes.
  const min = 22;
  const maxBack = Math.min(view.byteLength, min + 0xffff);
  for (let i = view.byteLength - min; i >= view.byteLength - maxBack; i--) {
    if (i < 0) break;
    if (view.getUint32(i, true) === EOCD_SIG) return i;
  }
  return -1;
}

/** Parses the central directory into the list of stored entries. */
function readCentralDirectory(bytes: Uint8Array): ZipEntry[] {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const eocd = findEocd(view);
  if (eocd < 0) {
    throw new Error("Invalid zip: no End Of Central Directory record found.");
  }
  // ZIP64 archives put a real CD offset behind a locator; we don't support them.
  if (eocd >= 20 && view.getUint32(eocd - 20, true) === ZIP64_EOCD_LOCATOR_SIG) {
    throw new Error("ZIP64 archives are not supported — re-zip without ZIP64.");
  }
  const entryCount = view.getUint16(eocd + 10, true);
  let offset = view.getUint32(eocd + 16, true);
  const entries: ZipEntry[] = [];
  for (let i = 0; i < entryCount; i++) {
    if (
      offset + 46 > bytes.byteLength ||
      view.getUint32(offset, true) !== CDH_SIG
    ) {
      break; // truncated / malformed central directory — stop with what we have
    }
    const flags = view.getUint16(offset + 8, true);
    const method = view.getUint16(offset + 10, true);
    const compressedSize = view.getUint32(offset + 20, true);
    const uncompressedSize = view.getUint32(offset + 24, true);
    const nameLen = view.getUint16(offset + 28, true);
    const extraLen = view.getUint16(offset + 30, true);
    const commentLen = view.getUint16(offset + 32, true);
    const localHeaderOffset = view.getUint32(offset + 42, true);
    const nameBytes = bytes.subarray(offset + 46, offset + 46 + nameLen);
    // ZIP names are CP437 historically but UTF-8 in practice for modern tools; the
    // language-encoding flag (bit 11) marks UTF-8. We decode UTF-8 either way — RDF
    // dump member names are ASCII in practice, so this is safe.
    const name = utf8.decode(nameBytes);
    entries.push({
      name,
      method,
      flags,
      localHeaderOffset,
      compressedSize,
      uncompressedSize,
    });
    offset += 46 + nameLen + extraLen + commentLen;
  }
  if (entries.length === 0) {
    throw new Error("Invalid or empty zip: no entries in the central directory.");
  }
  return entries;
}

/** Reads + decompresses one entry's bytes via its local file header. */
async function readEntryBytes(
  bytes: Uint8Array,
  entry: ZipEntry,
): Promise<Uint8Array> {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const lfo = entry.localHeaderOffset;
  if (lfo + 30 > bytes.byteLength || view.getUint32(lfo, true) !== LFH_SIG) {
    throw new Error(`Invalid zip: bad local header for "${entry.name}".`);
  }
  // bit 0 of the general-purpose flag = encrypted.
  if ((entry.flags & 0x0001) !== 0) {
    throw new Error(
      `Encrypted zip member "${entry.name}" is not supported — provide an unencrypted archive.`,
    );
  }
  const nameLen = view.getUint16(lfo + 26, true);
  const extraLen = view.getUint16(lfo + 28, true);
  const dataStart = lfo + 30 + nameLen + extraLen;
  const dataEnd = dataStart + entry.compressedSize;
  if (dataEnd > bytes.byteLength) {
    throw new Error(`Invalid zip: member "${entry.name}" data is truncated.`);
  }
  const compressed = bytes.subarray(dataStart, dataEnd);
  switch (entry.method) {
    case 0: // STORED — copy as-is
      return compressed.slice();
    case 8: // DEFLATE — raw deflate stream (no zlib header)
      return inflate(compressed, "deflate-raw");
    default:
      throw new Error(
        `Unsupported zip compression method ${entry.method} for "${entry.name}" — ` +
          "only STORED and DEFLATE are supported.",
      );
  }
}

/** Decompresses the first RDF-looking member of a zip archive (or, failing that, the
 *  single member / first member) to RDF text + that member's name. */
async function unzipFirstRdfMember(
  bytes: Uint8Array,
): Promise<DecompressedArchive> {
  const entries = readCentralDirectory(bytes);
  const files = entries.filter((e) => !e.name.endsWith("/"));
  if (files.length === 0) {
    throw new Error("Zip contains only directories, no files.");
  }
  // Prefer the first RDF-typed member; otherwise fall back to the single/first file
  // (lets an extensionless dump still load, with the caller guessing the format).
  const chosen = files.find((e) => isRdfMember(e.name)) ?? files[0];
  const out = await readEntryBytes(bytes, chosen);
  return { text: utf8.decode(out), innerName: chosen.name };
}
