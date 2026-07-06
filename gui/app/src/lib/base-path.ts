// [OPUS-4.8] sq-ixc3.8 — the ONE GUI-frontend base-path resolver.
//
// The operational GUI frontend is served under TWO targets (see gui/app/next.config.ts):
//   * the Tauri 2 desktop webview — from the `tauri://` root, where a `/prefix` 404s (basePath '');
//   * the hosted "Try the GUI live" web target — under a sub-path (default `/app`).
//
// `next.config.ts` env-switches `basePath`/`assetPrefix` off `NEXT_PUBLIC_BASE_PATH`, so Next
// rewrites `_next/*` chunk URLs and `<Link href>` routes automatically. A HARDCODED absolute
// asset path written by hand (a favicon, a static link) is NOT rewritten by Next, so it must
// read the same env var. This is also the value passed to the @sparq/client wasm loader so the
// runtime engine-asset fetch prefix stays in lockstep with the build-time route prefix.

/**
 * The configured base path, derived from `NEXT_PUBLIC_BASE_PATH` (the same switch
 * `next.config.ts` uses):
 *
 *   * **unset** → `"/app"` (the hosted-web default);
 *   * **`""`** → `""` (the Tauri root-relative export);
 *   * a leading-`/` value → that value (an explicit deploy prefix);
 *   * any malformed value → falls back to `"/app"`.
 *
 * Returns `""` (not `"/"`) for the root, so {@link withBasePath} composes a clean
 * `${basePath}/foo` without a double slash.
 */
export function basePath(): string {
  const raw = process.env.NEXT_PUBLIC_BASE_PATH;
  if (raw === undefined) return "/app";
  if (raw === "") return "";
  if (raw.startsWith("/")) return raw.replace(/\/$/, ""); // strip any trailing slash
  return "/app";
}

/** Prefix a root-absolute asset path with the configured {@link basePath}. */
export function withBasePath(path: string): string {
  return `${basePath()}${path}`;
}
