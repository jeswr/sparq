// [OPUS-4.8] sq-vjn4 — the in-site benchmark data layer. Consumes the committed
// snapshot (src/data/benchmarks.generated.json, refreshed by scripts/sync-benchmarks.mjs
// from the benchmark-data branch + bench/dashboard/{metric-labels,competitors}.json) and
// derives the structures the /benchmarks pages render: a per-TYPE (family) grouping for
// the left sidebar, per-suite grouping for the SPARQL page's collapsible groups, and the
// LIVE-COMPUTED competitive summary.
//
// HONESTY (load-bearing, per the bead + the empirical-honesty rule):
//   * Every metric is smaller-is-better (github-action-benchmark customSmallerIsBetter).
//   * A competitive speedup is computed ONLY from REAL competitor numbers that exist in
//     the data, MEASURED ON THE SAME BOX as the sparq number it is divided against:
//       - SHACL: competitors.values (pyshacl + jena-shacl) gathered same-machine vs the
//         same sparq SHACL metrics in the series.
//       - SP2Bench: competitors.same_box_comparisons (sparq vs Oxigraph/QLever on one
//         ephemeral box) — but Oxigraph/QLever rows are currently null, so NO speedup is
//         computed there; it shows "competitor baseline pending".
//   * Cross-machine `references` numbers (QLever on an M1, EYE DeepTaxonomy) are surfaced
//     SEPARATELY and never divided into a same-box speedup.
//   * A group with NO same-box competitor baseline shows sparq's absolute numbers + an
//     honest "competitor baseline pending" note — never a fabricated ratio.
//   * The CI series runs on a GitHub-hosted runner; the SHACL/SP2Bench gathers ran on an
//     ephemeral AWS Graviton box (NON-CANONICAL). Absolute µs across host classes are not
//     directly comparable — every speedup is labelled with its gather provenance.

import data from "./benchmarks.generated.json";

export interface Bench {
  name: string;
  value: number;
  unit: string;
}

export interface MetricLabel {
  label: string;
  suite: string;
  dataset?: string;
  query?: string;
  mode?: string;
  regime?: string;
  unit?: string;
}

export interface CompetitorEngine {
  id: string;
  label: string;
  version: string;
  env?: string;
  source?: string;
}

export interface ReferenceBaseline {
  suite: string;
  engine: string;
  version: string;
  env: string;
  scale: string;
  metric: string;
  value: number | null;
  unit: string;
  regime: string;
  source: string;
  caveat: string;
}

export interface SameBoxRow {
  query: string;
  unit: string;
  rows: number;
  values: Record<string, number | null>;
  count_match: boolean | null;
}

export interface SameBoxComparison {
  suite: string;
  scale: string;
  iters: number;
  git_commit: string;
  gathered_at_utc: string;
  source: string;
  env: {
    host_class: string;
    quiet_box: boolean;
    gathered_at_utc?: string;
    note?: string;
  };
  engines: { id: string; label: string; version: string; env?: string }[];
  rows: SameBoxRow[];
}

interface CompetitorsData {
  schema_version?: number;
  gathered_at_utc?: string;
  gathered_by?: string;
  engines: CompetitorEngine[];
  oxigraph_compare_note?: string;
  values: Record<string, Record<string, number>>;
  references: ReferenceBaseline[];
  same_box_comparisons: SameBoxComparison[];
}

interface Snapshot {
  generatedAt: string;
  source: string;
  latest: { commit: string | null; date: string | null; benches: Bench[] };
  labels: Record<string, MetricLabel>;
  competitors: CompetitorsData;
}

const SNAPSHOT = data as unknown as Snapshot;

export const LATEST = SNAPSHOT.latest;
export const SOURCE = SNAPSHOT.source;
export const GENERATED_AT = SNAPSHOT.generatedAt;
const LABELS = SNAPSHOT.labels;
export const COMPETITORS = SNAPSHOT.competitors;

// ---- metric-label lookup (mirror of dashboard.js lookup/labelFor/suiteFor) ----------

function stem(name: string): string {
  return name.replace(/_us$/, "");
}

export function lookup(name: string): MetricLabel | null {
  return LABELS[stem(name)] || LABELS[name] || null;
}

function humanize(name: string): string {
  return name
    .replace(/_us$/, " (µs)")
    .replace(/_s$/, " (s)")
    .replace(/_/g, " ")
    .trim();
}

export function labelFor(name: string): string {
  const rec = lookup(name);
  return rec && rec.label ? rec.label : humanize(name);
}

export function suiteFor(name: string): string {
  const rec = lookup(name);
  if (rec && rec.suite) return rec.suite;
  return "Other";
}

export function unitFor(name: string, fallback: string): string {
  const rec = lookup(name);
  return (rec && rec.unit) || fallback;
}

// ---- top-level capability FAMILIES (mirror of dashboard.js FAMILIES, label-driven) ---
// One entry per benchmark TYPE — drives the left sidebar. `suites` are matched against
// suiteFor() (normalised lower-case); `prefixes` are a structural fallback on the raw
// metric name's leading token. Order is the display order. A family with no data is KEPT
// (shows "not yet reported") so coverage is honest, never fabricated.

export interface Family {
  key: string;
  title: string;
  blurb: string;
  suites: string[];
  prefixes: string[];
}

export const FAMILIES: Family[] = [
  {
    key: "sparql",
    title: "SPARQL query suites",
    blurb:
      "Recognised public SPARQL benchmarks — LUBM, WatDiv, SP2Bench, BSBM, DBPSB — plus operator micro-benchmarks and the synthetic qlever-style suite.",
    suites: [
      "sp2bench",
      "watdiv",
      "lubm (reasoning)",
      "bsbm",
      "dbpsb",
      "synthetic (qlever-style)",
      "operators",
    ],
    prefixes: [
      "sp2b",
      "watdiv",
      "lubm",
      "bsbm",
      "dbpsb",
      "op",
      "q01",
      "q02",
      "q03",
      "q04",
      "q05",
      "q06",
      "q07",
      "q08",
      "q09",
      "q10",
      "q11",
      "q12",
    ],
  },
  {
    key: "shacl",
    title: "SHACL validation",
    blurb:
      "SHACL Core + SHACL-SPARQL validation latency over the LUBM(1) ABox × five hand-authored shape graphs, with same-box pySHACL + Apache Jena baselines.",
    suites: ["shacl validation", "shacl"],
    prefixes: ["shacl"],
  },
  {
    key: "geo",
    title: "GeoSPARQL",
    blurb:
      "geof: spatial functions + R-tree GeoIndex over a fixed ~100k CRS84 point corpus — within / nearest-k / topology compliance.",
    suites: ["geosparql", "geo"],
    prefixes: ["geo"],
  },
  {
    key: "fts",
    title: "Full-Text search",
    blurb:
      "BM25 full-text search via magic predicates over a synthetic 100k-literal corpus, plus index footprint.",
    suites: ["full-text", "fulltext", "fts", "text"],
    prefixes: ["text", "fts"],
  },
  {
    key: "vector",
    title: "Vector / ANN",
    blurb:
      "Approximate-nearest-neighbour recall deficits (HNSW / Vamana / PQ) vs exact kNN.",
    suites: ["vector", "vector / ann", "vectors", "ann"],
    prefixes: ["vector", "vectors", "vec", "ann", "knn", "hnsw", "diskann"],
  },
  {
    key: "reasoning",
    title: "Reasoning (N3 · RDFS · OWL-RL)",
    blurb:
      "Forward-closure materialization — the Deep Taxonomy depth series — with an EYE cross-engine reference.",
    suites: [
      "deep taxonomy",
      "deeptax",
      "deep-taxonomy",
      "deeptaxonomy",
      "reasoning",
      "inference",
    ],
    prefixes: ["deeptax", "deep", "owl", "infer", "incremental", "reason"],
  },
  {
    key: "core",
    title: "Core (load · dict · memory)",
    blurb:
      "End-to-end load (parse + index build), dictionary footprint, and per-triple store memory.",
    suites: ["pipeline", "memory / size"],
    prefixes: ["load", "parse", "store", "dict", "wasm", "rdfs"],
  },
];

const CAPABILITY_KEYS = new Set(["shacl", "geo", "fts", "vector", "reasoning"]);

export function familyOf(name: string): Family {
  const suite = suiteFor(name).toLowerCase();
  const token = name.toLowerCase().split("_")[0];
  for (const f of FAMILIES) {
    if (f.suites.includes(suite)) return f;
  }
  // capability families before core/sparql so a `q`/`op` prefix never mis-buckets them
  for (const f of FAMILIES) {
    if (CAPABILITY_KEYS.has(f.key) && f.prefixes.includes(token)) return f;
  }
  for (const f of FAMILIES) {
    if (!CAPABILITY_KEYS.has(f.key) && f.prefixes.includes(token)) return f;
  }
  if (/^(q\d+|op)_/.test(name.toLowerCase())) {
    return FAMILIES.find((f) => f.key === "sparql")!;
  }
  return FAMILIES.find((f) => f.key === "core")!;
}

// ---- per-family + per-suite views -----------------------------------------------------

export interface MetricRow {
  name: string;
  label: string;
  value: number;
  unit: string;
  suite: string;
}

function benchesForFamily(key: string): Bench[] {
  return LATEST.benches.filter((b) => familyOf(b.name).key === key);
}

export function familyMetricCount(key: string): number {
  return benchesForFamily(key).length;
}

export interface SuiteGroup {
  suite: string;
  rows: MetricRow[];
  summary: CompetitiveSummary;
}

// Rows for a family, grouped by their fine-grained suite, sorted by label. Each suite
// group carries its live-computed competitive summary.
export function suiteGroupsForFamily(key: string): SuiteGroup[] {
  const bySuite: Record<string, MetricRow[]> = {};
  for (const b of benchesForFamily(key)) {
    const suite = suiteFor(b.name);
    (bySuite[suite] = bySuite[suite] || []).push({
      name: b.name,
      label: labelFor(b.name),
      value: b.value,
      unit: b.unit,
      suite,
    });
  }
  return Object.keys(bySuite)
    .sort()
    .map((suite) => {
      const rows = bySuite[suite].sort((a, b) =>
        a.label < b.label ? -1 : a.label > b.label ? 1 : 0,
      );
      return { suite, rows, summary: competitiveSummary(suite, rows) };
    });
}

// Full assembly for a family's suite groups: rows + live summary + any same-box table +
// any cross-machine references — the shape the collapsible SuiteGroup component renders.
export interface FullSuiteGroup {
  suite: string;
  rows: MetricRow[];
  summary: CompetitiveSummary;
  sameBox?: SameBoxComparison;
  references: ReferenceBaseline[];
}

export function fullSuiteGroupsForFamily(key: string): FullSuiteGroup[] {
  return suiteGroupsForFamily(key).map((g) => ({
    suite: g.suite,
    rows: g.rows,
    summary: g.summary,
    sameBox: sameBoxFor(g.suite),
    references: referencesForSuite(g.suite),
  }));
}

// ---- LIVE competitive summary (the load-bearing honesty computation) ------------------
// For a given suite, compute the speedup of sparq vs the NEXT-BEST competitor across the
// suite's benchmarks, using ONLY same-box competitor numbers. Returns a discriminated
// union so the UI can render an honest state for every case.

export type CompetitiveSummary =
  | {
      kind: "speedup";
      // per-benchmark ratios competitor/sparq (>1 means sparq faster)
      min: number;
      max: number;
      median: number;
      n: number; // number of benchmarks contributing a real ratio
      competitor: string; // the next-best (fastest) competitor used per benchmark
      engines: string[]; // competitor labels present in the comparison
      provenance: string; // host/scale/quiet-box provenance string (NON-CANONICAL)
      nonCanonical: boolean;
    }
  | {
      kind: "pending";
      // a same-box comparison EXISTS for this suite but competitor cells are null
      reason: string;
      provenance?: string;
    }
  | {
      kind: "sparq-only";
      // no same-box competitor data at all for this suite
      reason: string;
    };

// median of a sorted-or-unsorted numeric array
function median(xs: number[]): number {
  const s = [...xs].sort((a, b) => a - b);
  const mid = Math.floor(s.length / 2);
  return s.length % 2 ? s[mid] : (s[mid - 1] + s[mid]) / 2;
}

const round1 = (x: number) => Math.round(x * 10) / 10;

// Find the same_box_comparisons entry whose suite matches (case-insensitive).
function sameBoxFor(suite: string): SameBoxComparison | undefined {
  const norm = suite.toLowerCase();
  return (COMPETITORS.same_box_comparisons || []).find(
    (c) =>
      c.suite.toLowerCase() === norm ||
      norm.includes(c.suite.toLowerCase()) ||
      c.suite.toLowerCase().includes(norm.split(" ")[0]),
  );
}

export function competitiveSummary(
  suite: string,
  rows: MetricRow[],
): CompetitiveSummary {
  // (1) SHACL-style: competitors.values keyed by metric stem, gathered same-machine
  //     against the SAME sparq metrics in the series. Compute per-metric ratios.
  const valueRatios: number[] = [];
  const valueEngines = new Set<string>();
  let usedNextBest = "";
  for (const r of rows) {
    const cell =
      COMPETITORS.values[r.name] || COMPETITORS.values[stem(r.name)] || null;
    if (!cell) continue;
    // next-best competitor = the FASTEST (smallest) competitor number for this metric
    let best = Infinity;
    let bestEngine = "";
    for (const [eng, v] of Object.entries(cell)) {
      valueEngines.add(eng);
      if (typeof v === "number" && v > 0 && v < best) {
        best = v;
        bestEngine = eng;
      }
    }
    if (best !== Infinity && r.value > 0) {
      valueRatios.push(best / r.value);
      if (!usedNextBest) usedNextBest = bestEngine;
    }
  }
  if (valueRatios.length > 0) {
    return {
      kind: "speedup",
      min: round1(Math.min(...valueRatios)),
      max: round1(Math.max(...valueRatios)),
      median: round1(median(valueRatios)),
      n: valueRatios.length,
      competitor: "next-best competitor",
      engines: engineLabels([...valueEngines]),
      provenance:
        "same-box vs " +
        engineLabels([...valueEngines]).join(" / ") +
        " on an ephemeral AWS Graviton box (NON-CANONICAL; for cross-engine ordering)",
      nonCanonical: true,
    };
  }

  // (2) same_box_comparisons (SP2Bench): per-query rows with a sparq value + competitor
  //     values measured on ONE box. Compute ratios only from rows where BOTH sparq and at
  //     least one competitor have a real number.
  const sb = sameBoxFor(suite);
  if (sb) {
    const ratios: number[] = [];
    const engines = new Set<string>();
    for (const row of sb.rows) {
      const sparq = row.values["sparq"];
      if (typeof sparq !== "number" || sparq <= 0) continue;
      let best = Infinity;
      for (const [eng, v] of Object.entries(row.values)) {
        if (eng === "sparq") continue;
        if (typeof v === "number" && v > 0) {
          engines.add(eng);
          if (v < best) best = v;
        }
      }
      if (best !== Infinity) ratios.push(best / sparq);
    }
    if (ratios.length > 0) {
      return {
        kind: "speedup",
        min: round1(Math.min(...ratios)),
        max: round1(Math.max(...ratios)),
        median: round1(median(ratios)),
        n: ratios.length,
        competitor: "next-best competitor",
        engines: engineLabels([...engines]),
        provenance:
          (sb.env.host_class || "ephemeral box") +
          " (NON-CANONICAL; for cross-engine ordering)",
        nonCanonical: true,
      };
    }
    // a same-box gather EXISTS but every competitor cell is null → honest pending
    return {
      kind: "pending",
      reason:
        "A same-box " +
        sb.suite +
        " comparison was run, but competitor (Oxigraph / QLever) timings were not captured this run, so no speedup can be computed yet.",
      provenance: sb.env.host_class,
    };
  }

  // (3) nothing comparable on the same box.
  return {
    kind: "sparq-only",
    reason:
      "No same-box competitor baseline has been gathered for this suite yet — sparq's absolute numbers are shown below.",
  };
}

function engineLabels(ids: string[]): string[] {
  return ids.map((id) => {
    const e = COMPETITORS.engines.find((x) => x.id === id);
    return e ? e.label : id;
  });
}

// External-reference (cross-machine) baselines for a suite — shown SEPARATELY, never
// divided into a same-box speedup. The featured-suite key the dashboard uses.
export function referencesForSuite(suite: string): ReferenceBaseline[] {
  const norm = suite.toLowerCase();
  return (COMPETITORS.references || []).filter(
    (r) =>
      r.suite.toLowerCase() === norm ||
      norm.includes(r.suite.toLowerCase()) ||
      r.suite.toLowerCase().includes(norm.split(" ")[0]),
  );
}

export function sameBoxComparison(suite: string): SameBoxComparison | undefined {
  return sameBoxFor(suite);
}

// Number-formatter matching dashboard.js fmtNum (no fabrication, just display).
export function fmtNum(v: number | null | undefined): string {
  if (v == null) return "—";
  if (v === 0) return "0";
  const abs = Math.abs(v);
  if (abs >= 1000) return v.toLocaleString("en-US", { maximumFractionDigits: 0 });
  if (abs >= 1) return (Math.round(v * 100) / 100).toLocaleString("en-US");
  return (Math.round(v * 10000) / 10000).toString();
}

// A one-line headline for a suite's collapsed group header.
export function summaryHeadline(suite: string, s: CompetitiveSummary): string {
  if (s.kind === "speedup") {
    const range =
      s.min === s.max
        ? `${s.min}×`
        : `${s.min}×–${s.max}× (median ${s.median}×)`;
    return `${range} faster than ${s.competitor} across ${suite}`;
  }
  if (s.kind === "pending") return "competitor baseline pending";
  return "sparq absolute numbers (no competitor baseline yet)";
}
