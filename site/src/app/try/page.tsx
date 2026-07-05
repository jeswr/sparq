import type { Metadata } from "next";

// [OPUS-4.8] sq-4hiqe (maintainer directive 2026-07-05) — the /try SPARQL playground is REMOVED.
// The deployed /app (the gui/app workbench overlaid at /app/ by pages.yml) is now the single
// operational workbench, so the in-tab REPL that used to live here is retired.
//
// This route stays alive as a PERMANENT CLIENT REDIRECT STUB so bookmarks, deep links, and any
// stale inbound link land on /app instead of hitting the catch-all 404. The site is
// `output: "export"` (static, no server) and cannot issue a real 301 — so this mirrors /gui:
// a tiny prerendered page that client-redirects on mount, with a no-JS fallback link.
//
// HARD (full-page) redirect: /app is served in production by a SEPARATE Next.js app overlaid at
// /app/ (sq-vnd0i). A next/router soft `replace` across two distinct Next builds would fetch the
// foreign RSC payload (/app/index.txt) and land on a raw .txt, so `hard` forces window.location.
import { RedirectStub } from "@/components/redirect-stub";

export const metadata: Metadata = {
  title: "SPARQL playground — moved to App",
  description: "The in-tab SPARQL playground has moved to /app, the sparq workbench.",
};

export default function TryRedirectPage() {
  return <RedirectStub to="/app" label="App (the sparq workbench)" hard />;
}
