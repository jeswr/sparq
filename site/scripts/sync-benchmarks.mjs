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
//   { generatedAt, source, latest: { commit, date, benches }, labels, competitors }
// committed so the site builds offline; re-run this script to refresh from the branch.
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

function parseDataJs(text) {
  const json = text
    .replace(/^\s*window\.BENCHMARK_DATA\s*=\s*/, "")
    .replace(/;\s*$/, "");
  const d = JSON.parse(json);
  const series = d.entries["sparq engine"];
  if (!Array.isArray(series) || series.length === 0) {
    throw new Error("benchmark series 'sparq engine' is empty");
  }
  const latest = series[series.length - 1];
  return {
    commit: (latest.commit && latest.commit.id) || null,
    date: latest.date || null,
    benches: latest.benches.map((b) => ({
      name: b.name,
      value: b.value,
      unit: b.unit || "",
    })),
  };
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

const dataJs = readBranchDataJs();
let latest;
let source;
if (dataJs) {
  latest = parseDataJs(dataJs);
  source = "benchmark-data branch (dev/bench/data.js)";
} else if (existsSync(out)) {
  const prev = JSON.parse(readFileSync(out, "utf8"));
  latest = prev.latest;
  source = prev.source + " (kept — benchmark-data ref unavailable this run)";
  console.warn(
    "[sync-benchmarks] origin/benchmark-data not found — keeping committed latest, refreshing labels+competitors only.",
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
  labels,
  competitors,
};

writeFileSync(out, JSON.stringify(payload, null, 2) + "\n");
console.log(
  `[sync-benchmarks] wrote ${latest.benches.length} benches (commit ${
    latest.commit ? latest.commit.slice(0, 8) : "?"
  }) + ${Object.keys(labels).length} labels → src/data/benchmarks.generated.json`,
);
