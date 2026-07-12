// [FABLE-5] sq-hmd7l.40 — OPT-IN well-known-suite corpus mode for the
// cross-library comparison (compare.mjs --corpus <name>). Node-side only
// (the browser page receives the resolved corpus as JSON over the harness
// server); the environment-agnostic runner is `runCorpusWorkload` in
// compare-workload.mjs.
//
// NOT a second harness and NOT a second expected-rows source: each corpus is
// the NATIVE suite's FIXED per-commit tier, produced by the suite's own
// cached deterministic generator (bench/<suite>/gen.sh — the exact script
// scripts/ci-bench.sh runs), and every expected row count is read verbatim
// from the suite's expected-rows.tsv — the same file the native ci-bench
// hard equality check gates on. No other tier is offered, because no other
// tier has a native expected-rows source.

import path from "node:path";
import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..", "..", "..");

/**
 * The corpora on offer — exactly the tiers pinned by the native suites'
 * expected-rows.tsv files (see each file's header for the pin rationale).
 * `gen` args match the scripts/ci-bench.sh invocations (SP2B_TRIPLES=250000,
 * WATDIV_SF=1 defaults).
 */
export const CORPORA = {
  sp2b: {
    gen: ["bench/sp2b/gen.sh", "250000"],
    format: "turtle",
    queriesDir: "bench/sp2b/queries",
    expectedTsv: "bench/sp2b/expected-rows.tsv",
    tier: "250k triples — the native per-commit tier (bench/sp2b/expected-rows.tsv)",
  },
  watdiv: {
    gen: ["bench/watdiv/gen.sh", "1"],
    format: "ntriples",
    queriesDir: "bench/watdiv/queries",
    expectedTsv: "bench/watdiv/expected-rows.tsv",
    tier: "SF=1 (~106k distinct triples) — the native per-commit tier (bench/watdiv/expected-rows.tsv)",
  },
};

export const CORPUS_NAMES = Object.keys(CORPORA);

/**
 * Ensures the corpus file exists (build-once-and-cache via the suite's own
 * gen.sh; steady-state runs touch no network) and returns its absolute path.
 * Throws with the generator's stderr tail if generation fails — corpus mode
 * is explicitly requested, so an unavailable generator is a hard error,
 * never a silently different corpus.
 */
export function ensureCorpusFile(name) {
  const spec = CORPORA[name];
  if (!spec) throw new Error(`unknown corpus '${name}' (expected ${CORPUS_NAMES.join("/")})`);
  const [script, ...args] = spec.gen;
  const r = spawnSync("bash", [path.join(REPO, script), ...args], {
    encoding: "utf8",
    timeout: 600_000,
  });
  const out = (r.stdout ?? "").trim().split("\n").filter(Boolean).at(-1);
  if (r.status !== 0 || !out) {
    const tail = (r.stderr ?? "").trim().split("\n").slice(-5).join("\n");
    throw new Error(`corpus '${name}' generation failed (${script} exit ${r.status}):\n${tail}`);
  }
  return out;
}

/** True iff the query text is an ASK form (after PREFIX/BASE/comment lines). */
export function isAskQuery(sparql) {
  const body = sparql
    .split("\n")
    .filter((l) => !/^\s*(?:PREFIX|BASE|#)/i.test(l))
    .join("\n")
    .trim();
  return /^ASK\b/i.test(body);
}

/**
 * Parses the native suite's expected-rows.tsv (`<query>\t<rows>` lines,
 * `#` comments) — the SAME file scripts/ci-bench.sh diffs against.
 */
export function parseExpectedRows(tsvText, tsvPath = "expected-rows.tsv") {
  const expected = new Map();
  for (const line of tsvText.split("\n")) {
    const t = line.trim();
    if (t === "" || t.startsWith("#")) continue;
    const [q, rows, ...rest] = t.split("\t");
    if (!q || rows === undefined || rest.length > 0 || !/^\d+$/.test(rows)) {
      throw new Error(`${tsvPath}: malformed line '${t}' (expected <query>\\t<rows>)`);
    }
    expected.set(q, Number(rows));
  }
  if (expected.size === 0) throw new Error(`${tsvPath}: no expected rows parsed`);
  return expected;
}

/**
 * Resolves a corpus name (+ the generated corpus file) into the descriptor
 * `runCorpusWorkload` consumes: `{ name, format, tier, text, queries }` with
 * `queries: [{ name, sparql, expected, ask }]` — one entry per line of the
 * native expected-rows.tsv, in tsv order. Every tsv query MUST have a .rq
 * file (a missing one is a hard error, not a silent subset).
 */
export async function loadCorpusSpec(name, corpusFile) {
  const spec = CORPORA[name];
  if (!spec) throw new Error(`unknown corpus '${name}' (expected ${CORPUS_NAMES.join("/")})`);
  const text = await readFile(corpusFile, "utf8");
  const tsvPath = path.join(REPO, spec.expectedTsv);
  const expected = parseExpectedRows(await readFile(tsvPath, "utf8"), spec.expectedTsv);
  const queries = [];
  for (const [qname, rows] of expected) {
    const qfile = path.join(REPO, spec.queriesDir, `${qname}.rq`);
    let sparql;
    try {
      sparql = await readFile(qfile, "utf8");
    } catch {
      throw new Error(`corpus '${name}': ${spec.expectedTsv} lists '${qname}' but ${spec.queriesDir}/${qname}.rq is missing`);
    }
    queries.push({ name: qname, sparql, expected: rows, ask: isAskQuery(sparql) });
  }
  return { name, format: spec.format, tier: spec.tier, text, queries };
}
