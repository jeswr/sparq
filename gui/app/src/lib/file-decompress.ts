// [SONNET-4.6] sq-1y04h — browser web-upload decompression shim for the GUI import drawer.
//
// Reads a browser `File` as binary, detects whether it is a compressed dataset archive
// (.gz / .gzip / .tgz, .zip, .zst / .zstd, .bz2 / .bzip2) by magic bytes first and file
// extension second, then routes through `@sparq/client`'s `decompressDatasetBytes` before
// UTF-8 decoding. Uncompressed files fall through unchanged.
//
// Why binary-first: `File.text()` round-trips bytes through a TextDecoder, which corrupts
// binary payloads (compressed streams are NOT valid UTF-8). Reading as `arrayBuffer` first
// and decoding only AFTER decompression is the only correct path for compressed inputs.
//
// Used by the import-drawer.tsx `WebFilePane` so the web persona can import
// `.gz`, `.zip`, `.zst`, and `.bz2` datasets without the desktop app (native loader).
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

/**
 * [SONNET-4.6] sq-1y04h — read a browser `File` and decompress it if it is a recognised
 * compressed dataset archive (.gz, .zip, .zst, .bz2). Uncompressed files are returned as-is
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

  // Try decompress — decompressDatasetBytes throws when it sees no recognised magic/extension.
  // Catch the "Unrecognised compressed payload" error to fall through to the uncompressed path.
  let result: Awaited<ReturnType<typeof decompressDatasetBytes>> | null = null;
  try {
    result = await decompressDatasetBytes(bytes, file.name);
  } catch (err) {
    // "Unrecognised compressed payload" means the file is not compressed — fall through.
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
      effectiveName: result.innerName ?? file.name,
      wasDecompressed: true,
      codec: result.codec,
    };
  }

  // Uncompressed: decode directly.
  return {
    text: decoder.decode(bytes),
    effectiveName: file.name,
    wasDecompressed: false,
  };
}
