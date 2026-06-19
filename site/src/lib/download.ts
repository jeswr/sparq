// [OPUS-4.8] sq-x0kp — a tiny DOM helper to download an in-memory string as a file.
//
// This is the ONE place in the site that turns a string (a CSV/TSV export the
// framework-agnostic `@sparq/client` produced) into a browser file download. It is
// deliberately DOM-coupled, so it stays in the site (NOT in `@sparq/client`, which carries no
// DOM beyond the wasm loader's two globals): the pure "what bytes to export" lives in the
// shared package, this is only "hand those bytes to the browser as a file". An object URL is
// created, a hidden anchor clicked, then the URL revoked so it is not leaked.

/**
 * Trigger a client-side download of `text` as a file named `filename` with the given MIME
 * `type`. No network — a `Blob` object URL is created, a synthetic anchor click starts the
 * save, and the URL is revoked on the next tick. Safe to call from a button handler.
 */
export function downloadText(filename: string, text: string, type: string): void {
  const blob = new Blob([text], { type });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  // Appending is not strictly required in modern browsers, but keeps the click reliable
  // across them; remove it immediately after.
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Revoke after the click has been dispatched so the navigation has read the URL.
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
