import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax.7 / sq-rclb8 — /gui is the OLD placeholder path. Option-B (the maintainer's
// decision after #1004 opened) renames the live operational GUI destination to /app; the in-tab
// REPL keeps /try. Static export cannot 301 (research/website-redesign.md §7), so this is a
// CLIENT REDIRECT STUB that keeps the old /gui path alive — inbound links land on /app.
//
// [OPUS-4.8] sq-vw3ax.11 — /app is served by a SEPARATE Next.js app (gui/app, overlaid at
// /app/ by pages.yml, sq-vnd0i), so this must be a HARD (full-page) redirect: a router
// soft `replace` across two distinct Next builds fetches the foreign RSC payload
// (/app/index.txt) and lands on a raw .txt instead of the GUI. `hard` forces window.location.
import { RedirectStub } from "@/components/redirect-stub";

export const metadata: Metadata = {
  title: "Try the GUI — moved to App",
  description: "The sparq GUI destination has moved to /app.",
};

export default function GuiRedirectPage() {
  return <RedirectStub to="/app" label="App (the sparq GUI)" hard />;
}
