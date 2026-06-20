import type { Metadata } from "next";

import { DownloadClient } from "./download-client";

// [OPUS-4.8] sq-gl3cf / sq-ixc3 — the /download route.
//
// Server component: owns the route metadata; the interactive body (client-side OS
// detection for progressive enhancement) lives in ./download-client.tsx. The page
// links the GitHub Releases assets — the desktop GUI installers (.dmg/.msi/.AppImage)
// plus the CLI/server binaries — and is honest that the desktop bundles are UNSIGNED
// developer builds (signing/notarization is the separate needs:user bead sq-v286.8).
export const metadata: Metadata = {
  title: "Download",
  description:
    "Download the sparq desktop GUI (macOS/Windows/Linux) and the CLI/server binaries from the latest GitHub Release. Desktop bundles are unsigned developer builds; the web playground at /try needs no install.",
};

export default function DownloadPage() {
  return <DownloadClient />;
}
