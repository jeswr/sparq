import * as React from "react";
import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax (bold /try redesign) — the playground page becomes a teal-forward
// IDE workbench on top of the merged dark-first foundation (globals.css tokens
// --hero-grad / --teal-glow / --elevation-*, utilities .bg-atmos / .text-gradient /
// .display-2 / .kicker). The hero strip states the SAME honest claims as before — the real
// Rust engine compiled to WebAssembly, running in-tab — but with flagship-product confidence
// instead of a docs intro. The heavy REPL workbench itself is code-split (repl-lazy.tsx) so the
// hero/shell paint before the wasm engine streams in. Faithful to
// sparq-design-system/proposals/web-try.html; every result/stat below the hero comes from REAL
// query execution against the real sample datasets (no illustrative figures).
import { ReplLazy } from "@/components/repl-lazy";

export const metadata: Metadata = {
  title: "SPARQL Playground — the real Rust engine, live in your tab",
  description:
    "Run real SPARQL queries with the sparq engine compiled to WebAssembly — live in your browser tab by default — or connect the same editor to a running sparq-server over the SPARQL 1.1 Protocol.",
};

// The capability pills carry the ACTUAL supported surface — the same honest claims the prose
// made, stated as a flagship product. Kept as data so the hero markup stays scannable.
const FORM_PILLS = ["SELECT", "ASK", "CONSTRUCT", "DESCRIBE", "UPDATE"];

export default function TryPage() {
  return (
    <div className="space-y-8">
      {/* Atmospheric ground — the foundation's fixed, layered teal/cyan glow + masked grid.
          Opt-in (a page must mount it), so un-redesigned routes are unaffected. Inert to
          pointer events; sits behind all content. */}
      <div className="bg-atmos" aria-hidden="true" />
      <div className="bg-grid" aria-hidden="true" />

      {/* ── Teal-forward hero strip ─────────────────────────────────────────────────── */}
      <header className="relative pt-2">
        <span className="kicker inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-3 py-1.5 normal-case tracking-[0.14em]">
          <span
            className="size-1.5 rounded-full bg-[var(--success)] shadow-[0_0_0_4px_color-mix(in_oklch,var(--success)_18%,transparent)]"
            aria-hidden="true"
          />
          WebAssembly · in-tab · nothing leaves the page
        </span>

        <h1 className="display-2 mt-5 font-semibold">
          The SPARQL playground that runs the{" "}
          <span className="text-gradient">real Rust engine</span>, live in your tab.
        </h1>

        <p className="measure mt-4 text-[15.5px] leading-relaxed text-muted-foreground">
          Edit the query, pick a dataset, hit{" "}
          <span className="font-medium text-foreground">Run</span> — every query executes
          against the in-tab store using the actual sparq engine compiled to{" "}
          <span className="font-medium text-foreground">WebAssembly</span>. The lean bundle
          ships the parser, triplestore and full SPARQL 1.1 / 1.2 engine. Switch to{" "}
          <span className="font-medium text-foreground">EXPLAIN / ANALYZE</span> to read the
          plan, or <span className="font-medium text-foreground">Connect</span> the same editor
          to a running sparq-server.
        </p>

        <div className="mt-5 flex flex-wrap items-center gap-2">
          <span className="inline-flex items-center gap-2 rounded-full border border-border bg-card/80 px-3 py-1.5 text-xs font-medium text-muted-foreground">
            {FORM_PILLS.map((form, i) => (
              <React.Fragment key={form}>
                {i > 0 && <span aria-hidden="true">·</span>}
                <code className="font-mono text-foreground">{form}</code>
              </React.Fragment>
            ))}
          </span>
          <span className="inline-flex items-center gap-2 rounded-full border border-border bg-card/80 px-3 py-1.5 text-xs font-medium text-muted-foreground">
            BGP · FILTER · OPTIONAL · UNION · MINUS · BIND · VALUES · paths · sub-SELECT
          </span>
          <span className="inline-flex items-center gap-2 rounded-full border border-border bg-card/80 px-3 py-1.5 text-xs font-medium text-muted-foreground">
            RDF&nbsp;1.2 triple terms
          </span>
        </div>
      </header>

      {/* ── The IDE workbench (3-pane). Heavy + wasm-backed, so code-split behind a skeleton.
          Owns the editor, the run/EXPLAIN/ANALYZE toolbar, the typed results table + plan-tree,
          the dataset / workspace / graphs rail, and the Connect strip. ───────────────────── */}
      <ReplLazy />
    </div>
  );
}
