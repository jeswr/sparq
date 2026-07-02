import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax.11 — /try stays IN-BROWSER-BY-DEFAULT and makes that legible with a SLIM,
// distinct header (fixing the copy duplication with the home hero). The home page now owns the
// marketing "A full SPARQL engine. In this tab." pitch + a lightweight in-fold runner; /try is
// the WORKBENCH — a tooling-voice header short enough that the editor + Run are above the fold on
// a laptop, then the full IDE workbench. The teal-forward foundation (globals.css tokens
// --hero-grad / --teal-glow / --elevation-*, utilities .bg-atmos / .text-gradient / .display-2 /
// .kicker) is unchanged; the heavy REPL workbench is code-split (repl-lazy.tsx) so the header
// paints before the wasm engine streams in. Every result below comes from REAL query execution
// against the real sample datasets (no illustrative figures).
import { ReplLazy } from "@/components/repl-lazy";

export const metadata: Metadata = {
  title: "SPARQL Playground — the real Rust engine, live in your tab",
  description:
    "Run real SPARQL queries with the sparq engine compiled to WebAssembly — live in your browser tab by default — or connect the same editor to a running sparq-server over the SPARQL 1.1 Protocol.",
};

export default function TryPage() {
  return (
    <div className="space-y-6">
      {/* Atmospheric ground — the foundation's fixed, layered teal/cyan glow + masked grid.
          Opt-in (a page must mount it), so un-redesigned routes are unaffected. Inert to
          pointer events; sits behind all content. */}
      <div className="bg-atmos" aria-hidden="true" />
      <div className="bg-grid" aria-hidden="true" />

      {/* ── Slim workbench header (distinct from the home hero; keeps editor + Run above the fold). */}
      <header className="relative pt-1">
        <p className="kicker text-primary">Workbench</p>
        <h1 className="display-2 mt-2 font-semibold">
          Your SPARQL workbench.{" "}
          <span className="text-gradient">No install.</span>
        </h1>
        <p className="measure mt-3 text-[15px] leading-relaxed text-muted-foreground">
          Load Turtle / TriG / JSON-LD, run{" "}
          <span className="font-medium text-foreground">SELECT / CONSTRUCT / UPDATE</span> on the
          real Rust engine in your tab, and inspect the query plan with{" "}
          <span className="font-medium text-foreground">EXPLAIN</span>. Nothing leaves this tab
          unless you connect a server.
        </p>
      </header>

      {/* ── The IDE workbench (3-pane). Heavy + wasm-backed, so code-split behind a skeleton.
          Owns the editor, the Run split-button, the typed results table + plan-tree, the
          dataset / workspace / graphs rail, and the Advanced · Connect drawer. ───────────────── */}
      <ReplLazy />
    </div>
  );
}
