"use client";

// [OPUS-4.8] sq-ixc3.9 — the bottom STATUS BAR (h-6, research/gui-design.md §A.2):
// MEASURED performance.now() latency of the last run · row count · target · persistence backend.
//
// HONESTY: the latency is the wall-clock of the query the user JUST ran, measured with
// performance.now() and LABELLED as a per-run latency — it is NOT a benchmark claim, and no
// canonical number is baked in (this work box / CI runner is non-canonical).

import { useEngine } from "@/lib/engine-context";

/** Detect the desktop (Tauri) host so the bar reports the real persistence backend honestly. */
function isTauri(): boolean {
  return (
    typeof window !== "undefined" &&
    // Tauri 2 injects a global the webview can detect.
    ("__TAURI_INTERNALS__" in window || "__TAURI__" in window)
  );
}

export function StatusBar() {
  const { lastLatencyMs, lastRowCount, storeSize } = useEngine();
  const desktop = isTauri();
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
      <span title="Where the workspace persists">
        backend: {desktop ? "desktop (Tauri)" : "browser (in-memory)"}
      </span>
    </footer>
  );
}
