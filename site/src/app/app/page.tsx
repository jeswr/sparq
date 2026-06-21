import type { Metadata } from "next";
import Link from "next/link";
import { MonitorPlay, Download, PlayCircle } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

// [OPUS-4.8] sq-vw3ax.7 / sq-rclb8 — the "App" top-bar destination: the live operational GUI.
//
// OPTION-B RECONCILIATION (the maintainer's decision after #1004 opened). The website's /try
// page STAYS the lightweight in-browser SPARQL REPL playground. The live operational GUI is a
// SEPARATE destination here at /app (the hosted web app will live at /sparq/app/). This is a
// deliberate, honest PLACEHOLDER so the slim top bar's "App" slot is a real, stable route today
// (not a 404) and the GUI track (epic sq-ixc3) has a clean handoff point — it fills in THIS page
// when the hosted web GUI lands. Until then it routes a visitor to the way to operate sparq that
// DOES exist today: the desktop GUI download (the in-tab REPL is a separate, linked destination
// for quick experiments, not the operational app). A bead (sq-rclb8) tracks repointing this /app
// placeholder at the hosted web GUI once that route is ready.
export const metadata: Metadata = {
  title: "App — the sparq GUI",
  description:
    "The sparq operational GUI — a workbench over a persistent local store. The hosted web app is being built and will live here at /sparq/app/; for now, download the desktop GUI.",
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
        <Badge variant="warning">Hosted web app — coming soon</Badge>
      </header>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            The hosted web app is being built
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CardDescription className="leading-relaxed">
            sparq ships two frontends that do different jobs: this site (which
            persuades and proves with live demos) and the GUI (a workbench you use
            to do real RDF/SPARQL work over your own data). A hosted, run-it-in-your
            browser version of the GUI is on the way and will live here at{" "}
            <code className="font-mono text-[0.9em]">/sparq/app/</code>. In the
            meantime, the desktop GUI is the way to operate sparq today — and the
            live REPL is here for a quick query without installing anything.
          </CardDescription>
          <div className="grid gap-4 sm:grid-cols-2">
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
              <Button asChild size="sm" className="mt-3">
                <Link href="/download">Download the desktop GUI</Link>
              </Button>
            </div>
            <div className="rounded-xl border bg-muted/30 p-4">
              <h2 className="flex items-center gap-2 text-sm font-semibold">
                <PlayCircle className="size-4 text-primary" aria-hidden />
                The live REPL — no install
              </h2>
              <p className="mt-1.5 text-sm text-muted-foreground">
                Just want to run a query? The live SPARQL REPL runs real queries
                against a sample graph right now, in this tab — the same Rust
                engine compiled to wasm, nothing sent to a server.
              </p>
              <Button asChild size="sm" variant="outline" className="mt-3">
                <Link href="/try">Open the live REPL</Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
