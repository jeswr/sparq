// [GPT-5.6] #1046 — the binary-to-RDF boundary shared by file uploads and URL imports.
// Read compressed sources as bytes, decode their container before UTF-8, then select the
// RDF syntax from the inner name. Keeping this framework-free makes both invocation paths
// unit-testable without mounting the former /try workbench.

import {
  archiveCodecFromContentType,
  archiveCodecFromName,
  decompressArchive,
  sniffArchive,
  UNSUPPORTED_ARCHIVE_ERROR,
} from "./dataset-archive";
import { formatFromContentType, guessFormat } from "./repl-dataset";

export interface BytesToRdfOptions {
  /** An explicit format picker value; `__auto__` keeps automatic detection enabled. */
  explicitFormat?: string;
  /** The response media type for URL imports, including an optional charset parameter. */
  contentType?: string | null;
}

export interface RdfSource {
  /** UTF-8 RDF source text, decoded only after any archive container is removed. */
  text: string;
  /** The engine format inferred from the inner RDF name/media type or explicitly selected. */
  format: string;
}

function isBzip2Source(
  bytes: Uint8Array,
  sourceName: string,
  contentType: string | null | undefined,
): boolean {
  const mime = contentType?.split(";")[0].trim().toLowerCase();
  const path = sourceName.split(/[?#]/)[0].toLowerCase();
  const named = /\.(?:bz2|bzip2|tbz|tbz2)$/.test(path);
  const served = mime === "application/x-bzip2" || mime === "application/bzip2";
  const magic =
    bytes.length >= 4 &&
    bytes[0] === 0x42 &&
    bytes[1] === 0x5a &&
    bytes[2] === 0x68 &&
    bytes[3] >= 0x31 &&
    bytes[3] <= 0x39;
  return named || served || magic;
}

/**
 * Turns uploaded/fetched bytes into RDF source text and its engine format. Archive detection
 * follows the existing URL contract (container Content-Type, source suffix, then magic bytes),
 * while plain RDF prefers its served media type over the source suffix. gzip/zip stay on the
 * browser-native archive path; zstd reaches the shared lazy decoder in `js/src/decompress.ts`.
 */
export async function bytesToRdf(
  bytes: Uint8Array,
  sourceName: string,
  options: BytesToRdfOptions = {},
): Promise<RdfSource> {
  const { explicitFormat, contentType } = options;
  const selectedFormat =
    explicitFormat && explicitFormat !== "__auto__"
      ? explicitFormat
      : undefined;
  const archiveCodec =
    archiveCodecFromContentType(contentType ?? null) ??
    archiveCodecFromName(sourceName) ??
    sniffArchive(bytes);

  // bzip2 is deliberately not a browser import codec. Detect its established signals so a URL
  // response cannot fall through to UTF-8 and reach the RDF parser as corrupted text; preserve
  // the archive helper's existing unsupported-format error instead of adding a decoder.
  if (!archiveCodec && isBzip2Source(bytes, sourceName, contentType)) {
    throw new Error(UNSUPPORTED_ARCHIVE_ERROR);
  }

  if (archiveCodec) {
    const { text, innerName } = await decompressArchive(
      bytes,
      sourceName,
      archiveCodec,
    );
    return {
      text,
      format: selectedFormat ?? guessFormat(innerName ?? sourceName),
    };
  }

  return {
    text: new TextDecoder().decode(bytes),
    format:
      selectedFormat ??
      formatFromContentType(contentType ?? null) ??
      guessFormat(sourceName),
  };
}
