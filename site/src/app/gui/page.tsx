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

// [OPUS-4.8] sq-vw3ax.7 — the "Try the GUI" top-bar destination.
//
// COORDINATION (the maintainer's flagged gap). The hosted web GUI "try-live" page is being
// built on a PARALLEL track (the GUI epic sq-ixc3). This is a deliberate, honest PLACEHOLDER
// so the slim top bar's "Try the GUI" slot is a real, stable route today (not a 404) and the
// GUI track has a clean handoff point — it fills in THIS page when the hosted GUI lands. Until
// then it routes a visitor to the two ways to use sparq that DO exist: the desktop GUI download
// and the in-tab live REPL. A bead (filed by this PR) tracks repointing/replacing this once the
// hosted GUI route is ready.
export const metadata: Metadata = {
  title: "Try the GUI",
  description:
    "The sparq desktop GUI — a workbench over a persistent local store. The hosted web GUI is being built; for now, download the desktop GUI or try the live SPARQL REPL in your browser.",
};

export default function GuiPage() {
  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <MonitorPlay className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">Try the GUI</h1>
            <p className="text-sm text-muted-foreground">
              A desktop workbench for sparq — import, query, and operate over a
              persistent local store.
            </p>
          </div>
        </div>
        <Badge variant="warning">Hosted web GUI — coming soon</Badge>
      </header>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            The hosted web GUI is being built
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CardDescription className="leading-relaxed">
            sparq ships two frontends that do different jobs: this site (which
            persuades and proves with live demos) and the GUI (a workbench you use
            to do real RDF/SPARQL work over your own data). A hosted, try-it-in-your
            browser version of the GUI is on the way. In the meantime, here are the
            two ways to use sparq that work today.
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
                Run real SPARQL against a sample graph right now, in this tab —
                the same Rust engine compiled to wasm, nothing sent to a server.
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
