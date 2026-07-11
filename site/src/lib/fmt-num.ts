// [FABLE-5] sq-qgkwy.1 — the benchmark number-formatter, in its OWN dependency-free module.
//
// It used to live in src/data/benchmarks.ts, but that module imports the full
// benchmarks.generated.json snapshot (~1.3 MB raw). Client components ("use client":
// metric-table, same-box-table, http-panel-table, references-note, scaling-charts) imported
// the VALUE `fmtNum` from there, which pulled the entire snapshot into the browser bundle —
// /benchmarks/[type]'s first-load page chunk was ~762 KB raw (~72 kB gz) of mostly JSON the
// server component had ALREADY rendered into the static HTML (double-shipped). Client code
// must import `fmtNum` from HERE; server-side code may keep using the re-export in
// src/data/benchmarks.ts. Guarded by test/benchmarks-data-server-only.test.mjs.
//
// Matches dashboard.js fmtNum (no fabrication, just display).
export function fmtNum(v: number | null | undefined): string {
  if (v == null) return "—";
  if (v === 0) return "0";
  const abs = Math.abs(v);
  if (abs >= 1000) return v.toLocaleString("en-US", { maximumFractionDigits: 0 });
  if (abs >= 1) return (Math.round(v * 100) / 100).toLocaleString("en-US");
  return (Math.round(v * 10000) / 10000).toString();
}
