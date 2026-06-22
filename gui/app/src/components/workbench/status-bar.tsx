"use client";

// [OPUS-4.8] sq-ixc3.9 — the bottom STATUS BAR (h-6, research/gui-design.md §A.2):
// MEASURED performance.now() latency of the last run · row count · target · persistence backend.
//
// HONESTY: the latency is the wall-clock of the query the user JUST ran, measured with
// performance.now() and LABELLED as a per-run latency — it is NOT a benchmark claim, and no
// canonical number is baked in (this work box / CI runner is non-canonical).

import { useEngine } from "@/lib/engine-context";
import { useWorkspace } from "@/lib/workspace-context";

/**
 * [OPUS-4.8] sq-ixc3.13 — an honest persistence-backend label from the workspace store's
 * RESOLVED backend (not just a Tauri-presence guess): on-device disk (Tauri fs capability
 * granted), this browser (localStorage), or this session only (in-memory fallback).
 */
function backendLabel(backend: "tauri" | "web" | "memory" | null): string {
  if (backend === "tauri") return "saved on device";
  if (backend === "web") return "saved in this browser";
  if (backend === "memory") return "this session only";
  return "resolving…";
}

export function StatusBar() {
  const { lastLatencyMs, lastRowCount, storeSize, nativeLoaderAvailable } = useEngine();
  const { backend } = useWorkspace();
  return (
    <footer className="flex h-6 shrink-0 items-center gap-4 border-t bg-card px-3 text-[11px] text-muted-foreground">
      <span className="tabular" title="Wall-clock latency of the last query (performance.now)">
        {lastLatencyMs === null ? "— ms" : `${lastLatencyMs.toFixed(1)} ms`}
      </span>
      <span className="tabular" title="Rows / triples returned by the last run">
        {lastRowCount === null ? "— rows" : `${lastRowCount.toLocaleString()} rows`}
      </span>
      <span className="ml-auto tabular">{storeSize.toLocaleString()} quads</span>
      <span title="Where queries run">target: local (in-tab WASM)</span>
      <span
        title={
          nativeLoaderAvailable
            ? "Imports decode through the native engine (compressed + native-only HDT)"
            : "Imports parse in the in-tab WASM engine (no compressed-file / HDT path)"
        }
      >
        loader: {nativeLoaderAvailable ? "native" : "in-tab"}
      </span>
      <span title="Where the workspace persists">backend: {backendLabel(backend)}</span>
    </footer>
  );
}
