// [SONNET-4.6] sq-1y04h — browser web-upload decompression shim for the GUI import drawer.
//
// Reads a browser `File` as binary, detects whether it is a compressed dataset archive
// (.gz / .gzip, .zip, .zst / .zstd) by magic bytes first and file
// extension second, then routes through `@sparq/client`'s `decompressDatasetBytes` before
// UTF-8 decoding. Uncompressed files fall through unchanged.
//
// Why binary-first: `File.text()` round-trips bytes through a TextDecoder, which corrupts
// binary payloads (compressed streams are NOT valid UTF-8). Reading as `arrayBuffer` first
// and decoding only AFTER decompression is the only correct path for compressed inputs.
//
// Used by the import-drawer.tsx `WebFilePane` and URL tab so the web persona can import
// `.gz`, `.zip`, and `.zst` datasets without the desktop app (native loader).
// The native Tauri path (gui/src-tauri open_reader) is unchanged — leave it.

import { decompressDatasetBytes, type DatasetCompressionCodec } from "@sparq/client";

/** The decoded result of a (possibly compressed) browser-uploaded file. */
export interface MaybeDecompressedFile {
  /** UTF-8 text content, decompressed if the input was a compressed archive. */
  text: string;
  /**
   * The effective inner filename for format detection.
   * For compressed archives: the suffix-stripped name (e.g. `"data.ttl"` from `"data.ttl.gz"`).
   * For uncompressed files: the original filename unchanged.
   */
  effectiveName: string;
  /** True when decompression was applied. */
  wasDecompressed: boolean;
  /** The codec selected, when decompression was applied. */
  codec?: DatasetCompressionCodec;
}

const decoder = new TextDecoder("utf-8", { fatal: false });

// [GPT-5.6] sq-n18o5 — the response subset required by binary URL import. Keeping this
// structural makes the fetch/decompress path unit-testable without a browser Response global.
type BinaryFetchResponse = Pick<
  Response,
  "ok" | "status" | "statusText" | "headers" | "arrayBuffer"
>;

type BinaryFetcher = (url: string) => Promise<BinaryFetchResponse>;

/** A fetched RDF document, decoded only after optional archive decompression. */
export interface FetchedRdfDocument extends MaybeDecompressedFile {
  /** The server's response media type, retained for RDF format detection. */
  contentType: string | null;
}

/**
 * [GPT-5.6] sq-n18o5 — fetch a URL as binary and decompress a supported dataset archive before
 * UTF-8 decoding. The effective inner name lets the caller detect RDF syntax from `data.nt`
 * rather than the outer `data.nt.gz` / `data.zip` container name.
 *
 * Binary-first is load-bearing: calling `Response.text()` would irreversibly corrupt compressed
 * bytes before the codec sees them.
 */
export async function fetchRdfDocument(
  url: string,
  fetcher: BinaryFetcher = (target) => fetch(target),
): Promise<FetchedRdfDocument> {
  const response = await fetcher(url);
  if (!response.ok) {
    throw new Error(
      `Fetch failed: HTTP ${response.status} ${response.statusText}`,
    );
  }

  const bytes = new Uint8Array(await response.arrayBuffer());
  return {
    ...(await maybeDecompressBytes(bytes, url)),
    contentType: response.headers.get("content-type"),
  };
}

/**
 * [GPT-5.6] sq-n18o5 — decode source bytes after decompressing any supported archive. Shared by
 * browser Files and URL responses so both import tabs use identical codec detection and errors.
 */
export async function maybeDecompressBytes(
  bytes: Uint8Array,
  sourceName: string,
): Promise<MaybeDecompressedFile> {
  const lowerName = sourceName.split(/[?#]/)[0].toLowerCase();
  const isBzip2 =
    (bytes.length >= 3 &&
      bytes[0] === 0x42 &&
      bytes[1] === 0x5a &&
      bytes[2] === 0x68) ||
    [".bz2", ".bzip2", ".tbz", ".tbz2"].some((suffix) =>
      lowerName.endsWith(suffix),
    );
  if (isBzip2) {
    throw new Error(
      "Bzip2 archives are supported only by the desktop app.",
    );
  }

  // Try decompress — decompressDatasetBytes throws when it sees no recognised magic/extension.
  // Catch the "Unrecognised compressed payload" error to fall through to the uncompressed path.
  let result: Awaited<ReturnType<typeof decompressDatasetBytes>> | null = null;
  try {
    result = await decompressDatasetBytes(bytes, sourceName);
  } catch (err) {
    // "Unrecognised compressed payload" means the source is not compressed — fall through.
    // Any other error (e.g. corrupt archive) is re-thrown so the import drawer surfaces it.
    if (
      !(err instanceof Error) ||
      !err.message.startsWith("Unrecognised compressed payload")
    ) {
      throw err;
    }
  }

  if (result !== null) {
    return {
      text: decoder.decode(result.bytes),
      effectiveName: result.innerName ?? sourceName,
      wasDecompressed: true,
      codec: result.codec,
    };
  }

  return {
    text: decoder.decode(bytes),
    effectiveName: sourceName,
    wasDecompressed: false,
  };
}

/**
 * [SONNET-4.6] sq-1y04h — read a browser `File` and decompress it if it is a recognised
 * compressed dataset archive (.gz, .zip, .zst). Uncompressed files are returned as-is
 * (their bytes decoded to UTF-8 text). Codec selection is by magic bytes first (the
 * `decompressDatasetBytes` payload probe), filename extension second — the same strategy
 * the `@sparq/client` util uses. The `effectiveName` is the suffix-stripped inner name so
 * `guessFormat` in `rdf-format.ts` sees `"data.ttl"` rather than `"data.ttl.gz"`.
 *
 * NEVER falls through to `File.text()` for compressed inputs: binary payloads decoded as
 * UTF-8 without decompression produce garbled text that the RDF parser rejects.
 */
export async function maybeDecompressFile(file: File): Promise<MaybeDecompressedFile> {
  // Read binary first in all cases: we need the magic bytes to probe the codec even for
  // files that turn out to be uncompressed. The extra arraybuffer copy is negligible for
  // typical RDF documents and is the only correct approach for compressed inputs.
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  return maybeDecompressBytes(bytes, file.name);
}
