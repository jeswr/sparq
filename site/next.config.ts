import type { NextConfig } from "next";

// [OPUS-4.8] sq-8thu — static-export config for GitHub Pages.
// Pages serves this project site under https://jeswr.github.io/sparq/, so every
// asset + route must be prefixed with `/sparq`. `output: "export"` writes a fully
// static `out/` tree (no Node server) that the Pages deploy workflow uploads.
const nextConfig: NextConfig = {
  output: "export",
  basePath: "/sparq",
  assetPrefix: "/sparq",
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
    return config;
  },
};

export default nextConfig;
