"use client";

// [OPUS-4.8] sq-ixc3.9 — the bottom STATUS BAR (research/gui-design.md §A.2):
// MEASURED performance.now() latency of the last run · row count · target · persistence backend.
//
// [OPUS-4.8] sq-vw3ax (#820 "GUI: unintrusive stats") — the bold redesign adds the issue's two
// asked-for stats, unintrusively (no modal): a slim INGEST meter (the real file label + a live
// MEASURED elapsed while an import is in flight) and a tiny DISK gauge (the REAL on-device store
// footprint — the byte length of the persisted whole-dataset N-Quads snapshot).
//
// HONESTY. Every figure here is real:
//   * latency  — wall-clock of the query the user JUST ran (performance.now), labelled per-run,
//                NOT a benchmark claim (this work box / CI runner is non-canonical).
//   * disk     — the actual size of what the workspace persists (snapshot bytes). There is no
//                fixed capacity, so the gauge shows the live footprint; the conic ring's fill is a
//                log-scaled visual cue only, and the tooltip states the exact bytes. A precise
//                filesystem probe of the app-data dir is a follow-up bead (new Tauri command).
//   * ingest   — the loaders are synchronous (no byte-level progress callback), so this is an
//                honest indeterminate "ingesting <file>…" with a real elapsed — never a fake %/ETA.

import * as React from "react";

import { useEngine, type IngestState } from "@/lib/engine-context";
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

/** Format a real byte count as a compact human size (KB/MB/GB). Pure presentation of a real value. */
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = bytes / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  return `${v < 10 ? v.toFixed(1) : Math.round(v)} ${units[i]}`;
}

/**
 * A log-scaled 0–100 fill for the disk gauge's conic ring. There is no fixed capacity, so this is
 * a purely VISUAL cue (empty store → a sliver, ~1 GB → nearly full); the tooltip + label carry the
 * exact real bytes. Kept monotonic so a bigger store always reads as a fuller ring.
 */
function diskRingPct(bytes: number): number {
  if (bytes <= 0) return 0;
  // 1 KB → ~0%, 1 GB → ~100%, log-spaced across the six orders of magnitude in between.
  const pct = (Math.log10(bytes) - 3) * (100 / 6);
  return Math.max(2, Math.min(100, pct));
}

/**
 * The unintrusive #820 ingest meter: an indeterminate sweep + the source label + a LIVE measured
 * elapsed (re-rendered ~4×/s from `performance.now()` against the ingest start). No fabricated %.
 */
function IngestMeter({ ingest }: { ingest: IngestState }) {
  const [elapsedMs, setElapsedMs] = React.useState(0);
  React.useEffect(() => {
    const tick = () => setElapsedMs(performance.now() - ingest.startedAt);
    tick();
    const id = window.setInterval(tick, 250);
    return () => window.clearInterval(id);
  }, [ingest.startedAt]);
  return (
    <span
      className="flex items-center gap-2 text-muted-foreground"
      title={`Ingesting ${ingest.label} — ${(elapsedMs / 1000).toFixed(1)} s elapsed (measured)`}
      data-status-ingest
    >
      <span className="max-w-[10rem] truncate">↧ {ingest.label}</span>
      <span className="sq-ingest-bar" aria-hidden>
        <span className="sq-ingest-fill" />
      </span>
      <span className="tabular">{(elapsedMs / 1000).toFixed(1)} s</span>
    </span>
  );
}

export function StatusBar() {
  const { lastLatencyMs, lastRowCount, storeSize, storeBytes, ingest, nativeLoaderAvailable } =
    useEngine();
  const { backend } = useWorkspace();
  return (
    <footer className="sq-statusbar flex h-7 shrink-0 items-center gap-4 border-t px-3 font-mono text-[11px] text-muted-foreground">
      <span
        className="tabular text-primary"
        title="Wall-clock latency of the last query (performance.now) — measured, not a benchmark"
      >
        {lastLatencyMs === null ? "— ms" : `${lastLatencyMs.toFixed(1)} ms`}
      </span>
      <span className="tabular" title="Rows / triples returned by the last run">
        {lastRowCount === null ? "— rows" : `${lastRowCount.toLocaleString()} rows`}
      </span>
      <span title="Where queries run">target: local · in-tab WASM</span>
      <span
        title={
          nativeLoaderAvailable
            ? "Imports decode through the native engine (compressed + native-only HDT)"
            : "Imports parse in the in-tab WASM engine (no compressed-file / HDT path)"
        }
      >
        loader: <span className="text-[var(--success)]">{nativeLoaderAvailable ? "native" : "in-tab"}</span>
      </span>
      <span title="Where the workspace persists">backend: {backendLabel(backend)}</span>

      {/* push the #820 stats to the right edge. */}
      <span className="ml-auto" />

      <span className="tabular" title="Quads in the live store (default + every named graph)">
        {storeSize.toLocaleString()} quads
      </span>

      {/* [OPUS-4.8] sq-vw3ax (#820) — the unintrusive ingest meter (only while an import runs). */}
      {ingest && <IngestMeter ingest={ingest} />}

      {/* [OPUS-4.8] sq-vw3ax (#820) — the disk gauge: the REAL persisted store footprint. */}
      <span
        className="flex items-center gap-1.5"
        title={`Workspace store on disk: ${storeBytes.toLocaleString()} bytes (the persisted whole-dataset snapshot)`}
        data-status-disk
      >
        <span
          className="sq-disk-ring"
          style={{ ["--disk-pct" as string]: diskRingPct(storeBytes) }}
          aria-hidden
        />
        disk {formatBytes(storeBytes)}
      </span>
    </footer>
  );
}
