// [OPUS-4.8] sq-9vw5 — the ONE site-side base-path resolver.
//
// The site is served under TWO hosts (see site/next.config.ts):
//   * GitHub Pages — under `/sparq/`, so static asset hrefs must be `/sparq`-prefixed;
//   * the Tauri 2 desktop webview — from the `tauri://` root, where a `/sparq` prefix 404s.
//
// `next.config.ts` already env-switches `basePath`/`assetPrefix` off `NEXT_PUBLIC_BASE_PATH`,
// so Next rewrites `_next/*` chunk URLs and `<Link href>` routes automatically. But a HARDCODED
// absolute asset path written by hand into a component (a `<Script src>`, a favicon, a PDF
// link) is NOT rewritten by Next — it ships verbatim. Those must read the same env var so they
// move with the build mode. This module is that single read, mirroring the resolver the wasm
// loaders already use (`?? "/sparq"`), kept in one place so the rule has one source of truth
// and one unit test.

/**
 * The configured base path for hand-written absolute asset URLs, derived from
 * `NEXT_PUBLIC_BASE_PATH` (the same switch `next.config.ts` and the `@sparq/client` wasm
 * loader use):
 *
 *   * **unset** → `"/sparq"` (the GitHub Pages default; historical behaviour, no change);
 *   * **`""`** → `""` (the Tauri root-relative export);
 *   * a leading-`/` value → that value (an explicit deploy prefix);
 *   * any malformed value → falls back to `"/sparq"`.
 *
 * Returns `""` (not `"/"`) for the root, so {@link withBasePath} composes a clean
 * `${basePath}/foo` without a double slash.
 */
export function basePath(): string {
  const raw = process.env.NEXT_PUBLIC_BASE_PATH;
  if (raw === undefined) return "/sparq";
  if (raw === "") return "";
  if (raw.startsWith("/")) return raw.replace(/\/$/, ""); // strip any trailing slash
  return "/sparq";
}

/**
 * Prefix a root-absolute asset path with the configured {@link basePath}. `path` MUST start
 * with `/` (it is a root-absolute public asset, e.g. `/logo.svg`). On Pages this yields
 * `/sparq/logo.svg`; in the Tauri root-relative build it yields `/logo.svg` unchanged.
 */
export function withBasePath(path: string): string {
  return `${basePath()}${path}`;
}
