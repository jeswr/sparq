// [SONNET-4.6] sq-b66fc — shared content-hash utility for the wasm sync scripts.
// Both site/scripts/sync-wasm.mjs and gui/app/scripts/sync-wasm.mjs import this to
// produce content-addressed filenames + the wasm-manifest.json that the @sparq/client
// resolveWasmAsset() loader uses to defeat GitHub Pages (~10 min max-age) stale-glue
// / new-wasm mismatches after a redeploy.
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

/**
 * Returns the first 12 hex chars of SHA-256(fileBytes).
 * @param {string} filePath  absolute or cwd-relative path to the file
 * @returns {Promise<string>}
 */
export async function hashFile(filePath) {
  const bytes = await readFile(filePath);
  return createHash("sha256").update(bytes).digest("hex").slice(0, 12);
}

/**
 * Given a filename like "sparq_wasm_bg.wasm", returns "sparq_wasm_bg-{hash}.wasm".
 * If the filename has no dot extension the hash suffix is appended directly.
 * @param {string} filename  bare filename (no directory component)
 * @param {string} hash      12-hex-char SHA-256 prefix
 * @returns {string}
 */
export function hashedName(filename, hash) {
  const dot = filename.lastIndexOf(".");
  return dot === -1
    ? `${filename}-${hash}`
    : `${filename.slice(0, dot)}-${hash}${filename.slice(dot)}`;
}
