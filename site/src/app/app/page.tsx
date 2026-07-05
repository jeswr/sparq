import type { Metadata } from "next";
import Link from "next/link";
import { MonitorPlay, PlayCircle, Download } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { withBasePath } from "@/lib/base-path";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

// [OPUS-4.8] sq-vw3ax.11 / sq-vnd0i — the "App" destination: the LIVE operational GUI.
//
// WHAT ACTUALLY SERVES /app IN PRODUCTION. The hosted web GUI is a SEPARATE Next.js app
// (gui/app — a workbench, not this site) that the Pages deploy builds with `build:web` and
// OVERLAYS at /app/, deliberately replacing this source page (pages.yml, sq-vnd0i /
// maintainer's Option B). So in the deployed site, /app IS the real hosted GUI.
//
// WHY THIS SOURCE STILL EXISTS. It is (1) the link-check target for the deploy's lychee pass,
// which runs over site/out BEFORE the gui/app overlay, so /app must resolve to an out/app/
// index.html at that point; (2) the destination the site-only build + `next dev` (and the e2e
// suite) render, since the overlay only happens in the Pages pipeline; and (3) a graceful
// fallback. Because the navigation to /app is a HARD, full-page link (app-shell NavLink
// `external`) — not a next/link soft nav across the two distinct Next builds — the browser loads
// the overlaid GUI's own HTML in production and this fallback never shows there. This copy is
// therefore written honestly for the local/preview context where it DOES render.
export const metadata: Metadata = {
  title: "App — the sparq GUI",
  description:
    "The sparq operational GUI — a workbench over a persistent local store, hosted in your browser at /app. Run a quick query in the live REPL, or install the desktop GUI.",
};

export default function AppPage() {
  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <MonitorPlay className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">App</h1>
            <p className="text-sm text-muted-foreground">
              The sparq operational GUI — import, query, and operate over a
              persistent local store.
            </p>
          </div>
        </div>
        <Badge variant="success">Hosted web GUI — live at /app</Badge>
      </header>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            The operational GUI, in your browser
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CardDescription className="leading-relaxed">
            sparq ships two frontends that do different jobs: this site (which
            persuades and proves with live demos) and the GUI (a workbench you use
            to do real RDF/SPARQL work over your own data). The hosted,
            run-it-in-your-browser GUI is served here at{" "}
            <code className="font-mono text-[0.9em]">/app/</code> — the same
            Rust engine compiled to wasm, nothing sent to a server. Want a quick
            query without leaving this tab, or a native build with no wasm ceiling?
            Both paths are below.
          </CardDescription>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="rounded-xl border bg-muted/30 p-4">
              <h2 className="flex items-center gap-2 text-sm font-semibold">
                <PlayCircle className="size-4 text-primary" aria-hidden />
                In your browser — no install
              </h2>
              <p className="mt-1.5 text-sm text-muted-foreground">
                Just want to run a query? The web workbench runs real queries
                against a sample graph right now, in this tab — the same Rust
                engine compiled to wasm, nothing sent to a server.
              </p>
              {/* [OPUS-4.8] sq-4hiqe — /app IS the single in-tab workbench (the /try REPL was
                  removed). Hard anchor: /app is a separate overlaid Next app. */}
              <Button asChild size="sm" className="mt-3">
                <a href={withBasePath("/app/")}>Open the workbench</a>
              </Button>
            </div>
            <div className="rounded-xl border bg-muted/30 p-4">
              <h2 className="flex items-center gap-2 text-sm font-semibold">
                <Download className="size-4 text-primary" aria-hidden />
                The desktop GUI
              </h2>
              <p className="mt-1.5 text-sm text-muted-foreground">
                A native desktop app (macOS / Windows / Linux) with the engine
                linked directly — threads, mmap, and persistence with no wasm
                ceiling. Builds are unsigned developer builds today.
              </p>
              <Button asChild size="sm" variant="outline" className="mt-3">
                <Link href="/download">Install the desktop GUI</Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
