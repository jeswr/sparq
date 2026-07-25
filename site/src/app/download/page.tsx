import type { Metadata } from "next";

import { DownloadClient } from "./download-client";

// [OPUS-4.8] sq-gl3cf / sq-ixc3 / sq-vw3ax.11 — the /download route.
//
// Server component: owns the route metadata; the interactive body lives in
// ./download-client.tsx (client-side OS detection + one-click DIRECT per-asset
// downloads via GitHub's version-stable `releases/latest/download/<alias>` endpoint,
// enriched with version/size/sha256 from an unauthenticated api.github.com fetch). It is
// honest that the desktop bundles are UNSIGNED developer builds (signing/notarization is
// the separate needs:user bead sq-v286.8).
export const metadata: Metadata = {
  title: "Download",
  description:
    "Download the sparq desktop GUI (macOS/Windows/Linux) and the CLI/server binaries — one click, straight to the right file for your platform. Desktop bundles are unsigned developer builds; the web workbench at /app needs no install.",
};

export default function DownloadPage() {
  return <DownloadClient />;
}
