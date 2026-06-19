import path from "node:path";
import type { NextConfig } from "next";

// [OPUS-4.8] sq-8thu — static-export config for GitHub Pages.
// Pages serves this project site under https://jeswr.github.io/sparq/, so every
// asset + route must be prefixed with `/sparq`. `output: "export"` writes a fully
// static `out/` tree (no Node server) that the Pages deploy workflow uploads.
//
// [OPUS-4.8] sq-9vw5 — env-switch the base path so the SAME export tree serves TWO hosts:
//
//   * GitHub Pages (the default): served under `/sparq/`, so basePath/assetPrefix must be
//     `/sparq`. This is what `npm run build` produces with no env set.
//   * The Tauri 2 desktop webview: serves the frontend from the `tauri://` root (a local
//     `frontendDist`), so EVERY asset must be ROOT-relative — a `/sparq` prefix 404s there.
//     The GUI's `gui/src-tauri/tauri.conf.json` `beforeBuildCommand` builds the site with
//     `NEXT_PUBLIC_BASE_PATH=''`, so this config reads that env var and emits a root-relative
//     export (`basePath: ''`, no asset prefix).
//
// `NEXT_PUBLIC_BASE_PATH` is the single switch — and the `@sparq/client` wasm loader already
// keys its RUNTIME asset URLs off the SAME env var, so the build-time route prefix and the
// runtime wasm-fetch prefix stay in lockstep. Two build modes (also in site/README.md):
//   * Pages :                      `npm run build`     -> basePath '/sparq'
//   * Tauri : `NEXT_PUBLIC_BASE_PATH='' npm run build` -> basePath '' (root-relative)
//
// An UNSET var keeps the historical `/sparq` default so the Pages build and every existing
// caller are unchanged; an explicit empty string selects the root-relative (Tauri) export.
// Next requires basePath to be empty or start with `/` (no trailing slash), so we honour only
// `''` or a `/`-leading value and otherwise fall back to the Pages default.
const rawBasePath = process.env.NEXT_PUBLIC_BASE_PATH;
const basePath =
  rawBasePath === undefined
    ? "/sparq" // unset -> Pages default (historical behaviour, no caller change)
    : rawBasePath === "" || rawBasePath.startsWith("/")
      ? rawBasePath // '' (Tauri root-relative) or an explicit '/prefix'
      : "/sparq"; // a malformed value falls back to the Pages default

const nextConfig: NextConfig = {
  output: "export",
  // [OPUS-4.8] sq-jpki — the repo now has repo-root npm workspaces (a root package.json with
  // a `workspaces` field + a root package-lock.json). Next would otherwise INFER the workspace
  // root from the multiple lockfiles it sees (root + site/package-lock.json) and emit a warning;
  // pin the file-tracing root to the monorepo root explicitly so the static export's tracing is
  // deterministic and the warning is gone. `__dirname` is site/, so the repo root is one up.
  outputFileTracingRoot: path.resolve(__dirname, ".."),
  // Only emit basePath/assetPrefix when there IS a prefix. An empty basePath must not be
  // set as `assetPrefix: ""` either — leaving both unset is exactly the root-relative
  // behaviour the Tauri webview needs.
  ...(basePath ? { basePath, assetPrefix: basePath } : {}),
  trailingSlash: true,
  // Static export cannot run the Next.js image optimiser.
  images: { unoptimized: true },
  // The @jeswr/sparq wrapper ships ESM with `.js` import specifiers that resolve
  // to `.ts`/`.tsx` sources in dev; mirror solid-pod-manager's webpack alias so the
  // bundler follows them.
  webpack: (config) => {
    config.resolve.extensionAlias = {
      ".js": [".ts", ".tsx", ".js", ".jsx"],
    };
    // [OPUS-4.8] sq-2e93 / sq-jpki — resolve the shared framework-agnostic client
    // (`packages/sparq-client`) to its TS SOURCE. The repo now has repo-root npm workspaces
    // (so `node_modules/@sparq/client` symlinks the package), but the package's `main`/`types`
    // point at `src/index.ts` and the site bundles from source; this alias (matching the
    // tsconfig `paths` entry) keeps the bundler following the import to the TS source the same
    // way in both the workspace and a bare `site/`-only install.
    config.resolve.alias = {
      ...config.resolve.alias,
      "@sparq/client": path.resolve(
        __dirname,
        "../packages/sparq-client/src/index.ts",
      ),
    };
    return config;
  },
};

export default nextConfig;
