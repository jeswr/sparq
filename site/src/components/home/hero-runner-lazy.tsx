"use client";

// [OPUS-4.8] sq-vw3ax.11 — the CODE-SPLIT boundary for the home hero's in-browser runner.
//
// The HeroQueryRunner pulls in the SPARQL/RDF editors and the `@sparq/client` query surface, and
// it pre-warms the wasm engine. Static-importing it would put all of that on the home route's
// first-load bundle and block the hero from painting. `next/dynamic({ ssr: false })` moves it into
// its own async chunk fetched AFTER the hero shell renders; the runner's pre-warm then fires on
// THIS chunk's hydration (see hero-runner.tsx), so the ~300 kB engine wasm never competes with the
// initial paint. A server component cannot call `dynamic(ssr:false)` directly in the App Router,
// so this thin "use client" wrapper owns the split.
//
// The loading placeholder is a STATIC PREVIEW (not a skeleton): it mirrors the runner's idle state
// — the same dimmed expected-answer table under a "Preview — press Run" pill — so the swap to the
// live runner is seamless and there is no skeleton flash on the hero (the design's load-bearing
// "fixes CTA→skeleton" rule).

import dynamic from "next/dynamic";

import { cn } from "@/lib/utils";
import { HERO_RESULT_VARS, HERO_PREVIEW_ROWS } from "@/data/hero-sample";

/** A non-interactive static twin of the runner's idle state — no wasm, no client state. */
function HeroRunnerFallback() {
  return (
    <div
      aria-busy="true"
      aria-label="Loading the in-browser SPARQL runner"
      className="overflow-hidden rounded-xl border border-primary/25 bg-card shadow-elevation-2"
    >
      <div className="flex items-center gap-1 border-b bg-muted/25 px-2 py-1.5">
        <span className="rounded-md bg-background px-3 py-1 text-xs font-medium text-foreground shadow-sm">
          Query
        </span>
        <span className="rounded-md px-3 py-1 text-xs font-medium text-muted-foreground">
          Data
        </span>
        <span className="ml-auto pr-1 font-mono text-[10px] uppercase tracking-[0.12em] text-muted-foreground">
          in-browser SPARQL
        </span>
      </div>
      <div className="p-3">
        <div className="h-[13.5rem] rounded-lg border bg-muted/40" />
      </div>
      <div className="relative border-t bg-background/40">
        <table className="w-full border-collapse text-[13px] opacity-45">
          <thead>
            <tr className="border-b">
              {HERO_RESULT_VARS.map((v, i) => (
                <th
                  key={v}
                  className={cn(
                    "px-3 py-2 font-mono text-xs font-medium text-primary",
                    i === 0 ? "text-left" : "text-right",
                  )}
                >
                  ?{v}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {HERO_PREVIEW_ROWS.map((r) => (
              <tr key={r.name} className="border-b border-border/50 last:border-0">
                <td className="px-3 py-1.5">
                  <span className="sq-tok-string">&quot;{r.name}&quot;</span>
                </td>
                <td className="px-3 py-1.5 text-right tabular">
                  <span className="sq-tok-number">{r.linesOfRust}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <span className="rounded-full border border-border bg-card/90 px-3 py-1.5 text-xs font-medium text-muted-foreground shadow-sm">
            Preview — press Run to compute it live in your tab
          </span>
        </div>
      </div>
    </div>
  );
}

const HeroQueryRunner = dynamic(
  () => import("@/components/home/hero-runner").then((m) => m.HeroQueryRunner),
  { ssr: false, loading: () => <HeroRunnerFallback /> },
);

/** The code-split entry point the (server) home hero renders. */
export function HeroQueryRunnerLazy() {
  return <HeroQueryRunner />;
}
