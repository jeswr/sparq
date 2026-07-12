// [GPT-5.6] sq-epbw4 — shared browser decompression for the site and GUI.
//
// gzip and ZIP use browser-native DecompressionStream support. zstd and bzip2 have no
// native browser codec, so their dependencies are imported only when that codec is selected.
// Keeping the import() expressions in the invocation path is load-bearing: neither decoder
// belongs in a surface's initial JavaScript bundle.

/** A compression/container format supported by {@link decompressDatasetBytes}. */
export type DatasetCompressionCodec = "gzip" | "zip" | "zstd" | "bzip2";

/** A decoded dataset and the best available name for its inner RDF document. */
export interface DecompressedDatasetBytes {
  /** The decompressed RDF document bytes. */
  bytes: Uint8Array;
  /** The codec selected from the payload magic, filename, or explicit argument. */
  codec: DatasetCompressionCodec;
  /** The archive member or suffix-stripped source name, when one is available. */
  innerName: string | null;
}

const ZSTD_MAGIC = 0xfd2fb528;
const ZSTD_SKIPPABLE_MAGIC = 0x184d2a50;
const ZSTD_SKIPPABLE_MASK = 0xfffffff0;

const CODEC_EXTENSIONS: Readonly<
  Record<DatasetCompressionCodec, readonly string[]>
> = {
  gzip: ["gz", "gzip", "tgz"],
  zip: ["zip"],
  zstd: ["zst", "zstd"],
  bzip2: ["bz2", "bzip2", "tbz", "tbz2"],
};

function codecFromMagic(
  bytes: Uint8Array,
): DatasetCompressionCodec | undefined {
  if (bytes.length >= 2 && bytes[0] === 0x1f && bytes[1] === 0x8b)
    return "gzip";
  if (
    bytes.length >= 4 &&
    bytes[0] === 0x50 &&
    bytes[1] === 0x4b &&
    (bytes[2] === 0x03 || bytes[2] === 0x05 || bytes[2] === 0x07) &&
    (bytes[3] === 0x04 || bytes[3] === 0x06 || bytes[3] === 0x08)
  ) {
    return "zip";
  }
  if (
    bytes.length >= 4 &&
    bytes[0] === 0x42 &&
    bytes[1] === 0x5a &&
    bytes[2] === 0x68 &&
    bytes[3] >= 0x31 &&
    bytes[3] <= 0x39
  ) {
    return "bzip2";
  }
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

function codecFromName(name: string): DatasetCompressionCodec | undefined {
  const extension = name.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "";
  for (const [codec, extensions] of Object.entries(CODEC_EXTENSIONS)) {
    if (extensions.includes(extension)) return codec as DatasetCompressionCodec;
  }
  return undefined;
}

function innerNameForSource(
  name: string,
  codec: DatasetCompressionCodec,
): string | null {
  const base = name.split(/[?#]/)[0];
  if (base.length === 0) return null;
  const extension = base.split(".").pop()?.toLowerCase() ?? "";
  if (codec === "gzip" && extension === "tgz") return null;
  if (codec === "bzip2" && (extension === "tbz" || extension === "tbz2"))
    return null;
  if (!CODEC_EXTENSIONS[codec].includes(extension)) return base;
  const suffixLength = extension.length + 1;
  const inner = base.slice(0, -suffixLength);
  return inner.length > 0 ? inner : null;
}

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
  const output = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    output.set(chunk, offset);
    offset += chunk.length;
  }
  return output;
}

async function inflate(
  bytes: Uint8Array,
  format: "gzip" | "deflate-raw",
): Promise<Uint8Array> {
  if (typeof DecompressionStream === "undefined") {
    throw new Error(
      `${format} decompression requires the browser DecompressionStream API.`,
    );
  }
  const stream = new Blob([bytes as BlobPart])
    .stream()
    .pipeThrough(
      new DecompressionStream(format),
    ) as unknown as ReadableStream<Uint8Array>;
  return collectStream(stream);
}

async function decodeZstd(bytes: Uint8Array): Promise<Uint8Array> {
  const { decompress } = await import(
    /* webpackChunkName: "codec-zstd" */ "fzstd"
  );
  return decompress(bytes);
}

async function decodeBzip2(bytes: Uint8Array): Promise<Uint8Array> {
  const [{ default: Bunzip }, { Buffer }] = await Promise.all([
    import(/* webpackChunkName: "codec-bzip2" */ "seek-bzip"),
    import(/* webpackChunkName: "codec-bzip2-buffer" */ "buffer"),
  ]);
  // seek-bzip is browser-safe apart from its Node-style Buffer global. Install the standard
  // browser shim only inside the bzip2 invocation path, after both lazy chunks have loaded.
  globalThis.Buffer ??= Buffer;
  return new Uint8Array(Bunzip.decode(bytes));
}

const EOCD_SIGNATURE = 0x06054b50;
const CENTRAL_HEADER_SIGNATURE = 0x02014b50;
const LOCAL_HEADER_SIGNATURE = 0x04034b50;
const ZIP64_EOCD_LOCATOR_SIGNATURE = 0x07064b50;

const RDF_EXTENSIONS = new Set([
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
]);

interface ZipEntry {
  name: string;
  method: number;
  flags: number;
  localHeaderOffset: number;
  compressedSize: number;
}

function findEndOfCentralDirectory(view: DataView): number {
  const minimumSize = 22;
  const maximumScan = Math.min(view.byteLength, minimumSize + 0xffff);
  for (
    let offset = view.byteLength - minimumSize;
    offset >= view.byteLength - maximumScan;
    offset--
  ) {
    if (offset < 0) break;
    if (view.getUint32(offset, true) === EOCD_SIGNATURE) return offset;
  }
  return -1;
}

function readCentralDirectory(bytes: Uint8Array): ZipEntry[] {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const eocd = findEndOfCentralDirectory(view);
  if (eocd < 0) {
    throw new Error("Invalid zip: no End Of Central Directory record found.");
  }
  if (
    eocd >= 20 &&
    view.getUint32(eocd - 20, true) === ZIP64_EOCD_LOCATOR_SIGNATURE
  ) {
    throw new Error("ZIP64 archives are not supported — re-zip without ZIP64.");
  }

  const entryCount = view.getUint16(eocd + 10, true);
  let offset = view.getUint32(eocd + 16, true);
  const entries: ZipEntry[] = [];
  const utf8 = new TextDecoder();
  for (let index = 0; index < entryCount; index++) {
    if (
      offset + 46 > bytes.byteLength ||
      view.getUint32(offset, true) !== CENTRAL_HEADER_SIGNATURE
    ) {
      break;
    }
    const nameLength = view.getUint16(offset + 28, true);
    const extraLength = view.getUint16(offset + 30, true);
    const commentLength = view.getUint16(offset + 32, true);
    const nameEnd = offset + 46 + nameLength;
    if (nameEnd > bytes.byteLength) break;
    entries.push({
      name: utf8.decode(bytes.subarray(offset + 46, nameEnd)),
      flags: view.getUint16(offset + 8, true),
      method: view.getUint16(offset + 10, true),
      compressedSize: view.getUint32(offset + 20, true),
      localHeaderOffset: view.getUint32(offset + 42, true),
    });
    offset = nameEnd + extraLength + commentLength;
  }
  if (entries.length === 0) {
    throw new Error(
      "Invalid or empty zip: no entries in the central directory.",
    );
  }
  return entries;
}

function isRdfMember(name: string): boolean {
  if (name.endsWith("/")) return false;
  const extension = name.split(".").pop()?.toLowerCase() ?? "";
  return RDF_EXTENSIONS.has(extension);
}

async function readZipEntry(
  bytes: Uint8Array,
  entry: ZipEntry,
): Promise<Uint8Array> {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const offset = entry.localHeaderOffset;
  if (
    offset + 30 > bytes.byteLength ||
    view.getUint32(offset, true) !== LOCAL_HEADER_SIGNATURE
  ) {
    throw new Error(`Invalid zip: bad local header for "${entry.name}".`);
  }
  if ((entry.flags & 0x0001) !== 0) {
    throw new Error(
      `Encrypted zip member "${entry.name}" is not supported — provide an unencrypted archive.`,
    );
  }
  const nameLength = view.getUint16(offset + 26, true);
  const extraLength = view.getUint16(offset + 28, true);
  const dataStart = offset + 30 + nameLength + extraLength;
  const dataEnd = dataStart + entry.compressedSize;
  if (dataEnd > bytes.byteLength) {
    throw new Error(`Invalid zip: member "${entry.name}" data is truncated.`);
  }
  const compressed = bytes.subarray(dataStart, dataEnd);
  if (entry.method === 0) return compressed.slice();
  if (entry.method === 8) return inflate(compressed, "deflate-raw");
  throw new Error(
    `Unsupported zip compression method ${entry.method} for "${entry.name}" — ` +
      "only STORED and DEFLATE are supported.",
  );
}

async function unzipFirstRdfMember(
  bytes: Uint8Array,
): Promise<{ bytes: Uint8Array; innerName: string }> {
  const files = readCentralDirectory(bytes).filter(
    (entry) => !entry.name.endsWith("/"),
  );
  if (files.length === 0)
    throw new Error("Zip contains only directories, no files.");
  const selected = files.find((entry) => isRdfMember(entry.name)) ?? files[0];
  return {
    bytes: await readZipEntry(bytes, selected),
    innerName: selected.name,
  };
}

/**
 * Decompresses browser dataset bytes, selecting the codec by payload magic first and the source
 * filename extension second. Pass `codec` only when neither signal is available. ZIP archives
 * yield their first RDF-looking member; other codecs report the suffix-stripped source name.
 *
 * gzip and ZIP use native browser APIs. zstd and bzip2 load `fzstd` and
 * `seek-bzip`, respectively, through lazy ESM `import()` calls on first use.
 */
export async function decompressDatasetBytes(
  bytes: Uint8Array,
  sourceName = "",
  codec?: DatasetCompressionCodec,
): Promise<DecompressedDatasetBytes> {
  const resolved = codec ?? codecFromMagic(bytes) ?? codecFromName(sourceName);
  switch (resolved) {
    case "gzip":
      return {
        bytes: await inflate(bytes, "gzip"),
        codec: resolved,
        innerName: innerNameForSource(sourceName, resolved),
      };
    case "zip": {
      const decoded = await unzipFirstRdfMember(bytes);
      return { ...decoded, codec: resolved };
    }
    case "zstd":
      return {
        bytes: await decodeZstd(bytes),
        codec: resolved,
        innerName: innerNameForSource(sourceName, resolved),
      };
    case "bzip2":
      return {
        bytes: await decodeBzip2(bytes),
        codec: resolved,
        innerName: innerNameForSource(sourceName, resolved),
      };
    default:
      throw new Error(
        "Unrecognised compressed payload: expected gzip (.gz), zip (.zip), " +
          "zstd (.zst), or bzip2 (.bz2) bytes.",
      );
  }
}
