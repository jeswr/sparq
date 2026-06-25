"use client";

// [OPUS-4.8] sq-vw3ax (#820 bold redesign) — the NATIVE TITLE BAR.
//
// The single strongest "this is a desktop app, not the website in a frame" signal from the
// approved proposal (/home/ubuntu/sparq-design-system/proposals/desktop-gui.html §1): a thin,
// branded title strip with a centred window title + an engine/runtime badge.
//
// NATIVE FEEL, HONESTLY. The strip is a Tauri DRAG REGION (`data-tauri-drag-region`) — inside the
// desktop shell, dragging it moves the OS window, exactly like a real titlebar; on the hosted web
// target the attribute is inert (no Tauri runtime) and it is just a header strip. We do NOT claim
// custom traffic-light WINDOW CONTROLS: the Tauri window keeps its OS decorations (tauri.conf.json
// has no `decorations: false`), so the three dots here are a decorative brand flourish, not fake
// close/minimise buttons that would do nothing. The badge reports the REAL runtime: inside Tauri
// the workbench has the direct native-engine ingest link, on the web it is the in-tab WASM engine
// — read from the engine context's resolved `nativeLoaderAvailable`, never hard-coded.

import { useEngine } from "@/lib/engine-context";

export function TitleBar() {
  const { nativeLoaderAvailable } = useEngine();
  // The badge is an honest CAPABILITY statement (which engine path is wired), not a perf claim.
  const runtimeLabel = nativeLoaderAvailable ? "native engine" : "in-tab WASM";
  return (
    <div
      data-tauri-drag-region
      className="sq-titlebar grid h-8 shrink-0 select-none grid-cols-[1fr_auto_1fr] items-center border-b px-3.5"
    >
      {/* A decorative brand flourish (NOT functional window controls — OS decorations remain). */}
      <div className="flex items-center gap-2" aria-hidden>
        <span className="size-3 rounded-full bg-[oklch(0.68_0.18_25)] shadow-[0_0_0_0.5px_oklch(0_0_0/0.3)_inset]" />
        <span className="size-3 rounded-full bg-[oklch(0.82_0.14_85)] shadow-[0_0_0_0.5px_oklch(0_0_0/0.3)_inset]" />
        <span className="size-3 rounded-full bg-[oklch(0.74_0.15_150)] shadow-[0_0_0_0.5px_oklch(0_0_0/0.3)_inset]" />
      </div>
      <div className="text-center text-[12.5px] tracking-tight text-muted-foreground">
        <span className="font-semibold text-foreground">sparq</span> — RDF + SPARQL workbench
      </div>
      <div className="flex items-center justify-end">
        <span
          className="rounded-full border px-2.5 py-0.5 font-mono text-[10.5px] tracking-wide text-muted-foreground"
          title={
            nativeLoaderAvailable
              ? "Desktop shell: ingest runs through the direct native engine link (threads, no ~2 GiB wasm-tab ceiling, native-only HDT)"
              : "Hosted web target: the in-tab WASM engine (no native-only ingest path)"
          }
        >
          {runtimeLabel}
        </span>
      </div>
    </div>
  );
}
