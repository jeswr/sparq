// [OPUS-4.8] sq-vjn4 — sync the committed benchmark RESULTS into the site as a typed
// JSON the static export imports at build time (NO git/network needed at build).
//
// SOURCES OF TRUTH (all already in the repo or on a repo branch):
//   1. data.js          — github-action-benchmark series, written by .github/workflows/
//                         bench.yml onto the `benchmark-data` branch at dev/bench/data.js.
//                         Shape: window.BENCHMARK_DATA = { entries: { "sparq engine": [
//                           { commit, date, benches: [{name,value,unit}] }, ... ] } }.
//                         The LAST entry is the latest commit. EVERY metric is
//                         smaller-is-better (tool=customSmallerIsBetter).
//   2. metric-labels.json — bench/dashboard/metric-labels.json: metric STEM (name minus
//                         _us) -> { label, suite, dataset, query, mode?, regime?, unit }.
//   3. competitors.json — bench/dashboard/competitors.json: the human-reviewable
//                         versioned competitor data (engines / values / references /
//                         same_box_comparisons). The dashboard embeds a byte-for-meaning
//                         mirror; THIS file is the canonical source we read.
//
// OUTPUT: site/src/data/benchmarks.generated.json — a single typed JSON:
//   { generatedAt, source, latest: { commit, date, benches },
//     history: [ { commit, date, benches }, ... ], labels, competitors }
// committed so the site builds offline; re-run this script to refresh from the branch.
//
// [OPUS-4.8] sq-hsyg — `history` carries the trailing HISTORY_WINDOW (20) commits of the
// SAME series the standalone bench/dashboard renders (window.BENCHMARK_DATA), so the site
// can draw per-metric TREND charts (metric over commits) + SCALING charts (metric vs a
// size/depth token in the metric name) WITHOUT a chart fetch at runtime. `latest` is kept
// (it is history[history.length-1]) so every existing page is unaffected. We never
// fabricate points: if the series is shorter than the window, history is just what exists,
// and when the benchmark-data ref is unavailable we keep whatever history was committed.
//
// HOW THE BRANCH IS READ: data.js is NOT on `main`; it lives on `benchmark-data`. We
// `git show origin/benchmark-data:dev/bench/data.js`. If that ref is absent (a shallow
// clone / fork without the branch), we KEEP the already-committed generated JSON and
// only refresh labels+competitors from the working tree — so the script never fails the
// build and never fabricates data.
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..", "..");
const out = join(here, "..", "src", "data", "benchmarks.generated.json");

function readBranchDataJs() {
  for (const ref of [
    "origin/benchmark-data:dev/bench/data.js",
    "benchmark-data:dev/bench/data.js",
  ]) {
    try {
      return execSync(`git show ${ref}`, {
        cwd: repoRoot,
        maxBuffer: 64 * 1024 * 1024,
        stdio: ["ignore", "pipe", "ignore"],
      }).toString();
    } catch {
      // try next ref
    }
  }
  return null;
}

// [OPUS-4.8] sq-hsyg — same trailing window the standalone dashboard trends over.
const HISTORY_WINDOW = 20;

// Normalise one github-action-benchmark series entry to the site's slim shape
// ({ commit, date, benches:[{name,value,unit}] }). `commit` is the short-or-full id; `date`
// is the epoch-ms the entry carries (the site formats it). Defensive against a missing
// commit/benches so a malformed entry never throws the build.
function normEntry(entry) {
  return {
    commit: (entry.commit && entry.commit.id) || null,
    date: entry.date || null,
    benches: (entry.benches || []).map((b) => ({
      name: b.name,
      value: b.value,
      unit: b.unit || "",
    })),
  };
}

function parseDataJs(text) {
  const json = text
    .replace(/^\s*window\.BENCHMARK_DATA\s*=\s*/, "")
    .replace(/;\s*$/, "");
  const d = JSON.parse(json);
  const series = d.entries["sparq engine"];
  if (!Array.isArray(series) || series.length === 0) {
    throw new Error("benchmark series 'sparq engine' is empty");
  }
  // Trailing HISTORY_WINDOW commits, chronological (oldest → newest). `latest` is the last
  // element so the existing summary/competitive views read it exactly as before.
  const history = series.slice(-HISTORY_WINDOW).map(normEntry);
  return { latest: history[history.length - 1], history };
}

const labels = JSON.parse(
  readFileSync(join(repoRoot, "bench", "dashboard", "metric-labels.json"), "utf8"),
).labels;

const competitorsRaw = JSON.parse(
  readFileSync(join(repoRoot, "bench", "dashboard", "competitors.json"), "utf8"),
);
// Drop the file-only `_comment*` keys — they are reviewer prose, not render data.
const competitors = {};
for (const k of Object.keys(competitorsRaw)) {
  if (k.startsWith("_comment")) continue;
  competitors[k] = competitorsRaw[k];
}

// [OPUS-4.8] sq-1sa9r — `--competitors-only` (env SYNC_BENCHMARKS_COMPETITORS_ONLY=1)
// refreshes ONLY labels + competitors from the working tree and KEEPS the committed
// latest/history. Use it for a competitor-focused ingest (e.g. adding a canonical
// same_box_comparisons row) so the PR does not also drag in an unrelated refresh of the
// sparq CI series from the benchmark-data branch. Never fabricates: it simply preserves the
// already-committed latest/history exactly as the branch-absent fallback does.
const competitorsOnly =
  process.argv.includes("--competitors-only") ||
  process.env.SYNC_BENCHMARKS_COMPETITORS_ONLY === "1";

const dataJs = competitorsOnly ? null : readBranchDataJs();
let latest;
let history;
let source;
if (dataJs) {
  ({ latest, history } = parseDataJs(dataJs));
  source = "benchmark-data branch (dev/bench/data.js)";
} else if (existsSync(out)) {
  const prev = JSON.parse(readFileSync(out, "utf8"));
  latest = prev.latest;
  // Keep whatever history was committed; fall back to a single-point history derived from
  // `latest` so the trend charts still render (one marker) on a fork without the branch.
  history =
    Array.isArray(prev.history) && prev.history.length ? prev.history : [latest];
  // [FABLE-5] sq-7d3dj.34: strip any prior "(kept — …)" suffixes before appending, so
  // repeated --competitors-only runs stay idempotent instead of accumulating the marker.
  const baseSource = prev.source.replace(/( \(kept — [^)]*\))+$/, "");
  source =
    baseSource +
    (competitorsOnly
      ? " (kept — --competitors-only: refreshed labels+competitors only)"
      : " (kept — benchmark-data ref unavailable this run)");
  console.warn(
    competitorsOnly
      ? "[sync-benchmarks] --competitors-only: keeping committed latest+history, refreshing labels+competitors from the working tree."
      : "[sync-benchmarks] origin/benchmark-data not found — keeping committed latest+history, refreshing labels+competitors only.",
  );
} else {
  console.error(
    "[sync-benchmarks] no benchmark-data ref AND no existing generated JSON — cannot proceed.",
  );
  process.exit(1);
}

const payload = {
  generatedAt: new Date().toISOString(),
  source,
  latest,
  history,
  labels,
  competitors,
};

writeFileSync(out, JSON.stringify(payload, null, 2) + "\n");
console.log(
  `[sync-benchmarks] wrote ${latest.benches.length} benches (commit ${
    latest.commit ? latest.commit.slice(0, 8) : "?"
  }) + ${history.length}-commit history + ${
    Object.keys(labels).length
  } labels → src/data/benchmarks.generated.json`,
);
