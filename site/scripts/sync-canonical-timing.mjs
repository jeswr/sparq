// [OPUS-5] sq-gum8.16 (paper factory F4) — derive the `canonical-timing` evidence class from
// the committed canonical measurement envelopes, so a paper can headline a MEASURED wall-clock
// number without a human ever typing one.
//
// WHY THIS EXISTS (research/paper-factory-2026-07.md §2 "second gap", §3.2 option (c)):
// `site/src/data/paper-evidence.json` holds only deterministic, machine-independent values
// (conformance floors, recall floors, boolean invariants). Papers therefore could not cite ANY
// measured timing — not even the provenance-complete envelopes the dedicated quiet-EC2
// protocol already commits under `bench/canonical-competitor-results/**` — leaving the
// systems-paper evaluation sections weaker than the data the project actually has.
//
// THE POLICY THIS IMPLEMENTS (papers MAY headline measured timings, under strict provenance):
//   1. Values are DERIVED here, never hand-typed. There is no author-writable timing file.
//   2. An envelope is admitted ONLY if it self-declares `"canonical": true` — so a work-box
//      timing cannot enter the class by construction (work-box numbers are NON-CANONICAL and
//      stay excluded; `benchmarks.generated.json` remains `environment: indicative` and may
//      never back a headline).
//   3. Every admitted record carries the envelope's provenance (host class, cpu, git commit,
//      gather date, dataset/scale, aggregate) and `site/papers/_lib/timing.typ` has NO raw
//      accessor — `headline_timing()` always renders the provenance alongside the number.
//   4. COUNT BEFORE TIME. The envelopes themselves state the rule ("this is why COUNT is
//      checked before trusting any timing"): a row is admitted only if that engine's own
//      result count matches `bench/<suite>/expected-rows.tsv` (`count_crosscheck.expected`),
//      or — when no expected count is committed — only if every engine agreed. So a
//      fast-but-WRONG comparator (e.g. an engine returning 0 rows on SP2Bench q08) can never
//      be published as a timing.
//
// HONEST SCOPE — read before extending:
//   - Only ONE envelope family is derived today: the same-box per-query SPARQL comparison,
//     `tsv_format == "<query>\t<rows|ERROR>\t<best_us|engine>"`. The HTTP family (6-column,
//     keep-alive vs fresh + TTFB), the materialization family (per-profile closure timings)
//     and the HDT family (per-metric) are RECOGNISED AND SKIPPED WITH A REASON rather than
//     guessed at — each needs its own key scheme. `skipped[]` in the output is the honest
//     record of what was left out.
//   - An envelope carrying a free-text `disqualified_note` is SKIPPED WHOLE. The note scopes a
//     disqualification in prose a machine cannot bound, so the fail-closed direction is to
//     refuse the envelope and surface the reason for a human.
//   - This is a MECHANICAL derivation: it guarantees value↔envelope provenance, not that the
//     comparison is fair or that the paper's framing of it is honest. Semantic framing stays
//     the Stage-5 claims↔evidence review in `skills/academic-paper/SKILL.md`.
//
// OUTPUT: `site/src/data/paper-timing.generated.json`, committed so the derivation is
// reviewable in a diff (the repo's committed-generated-artifact pattern) and re-derived on
// every `prebuild` so a newly-landed canonical envelope updates the papers automatically.
// `site/test/paper-timing.test.mjs` byte-compares the committed file against a fresh
// derivation, which is the drift guard.
//
// USAGE:
//   node scripts/sync-canonical-timing.mjs            # write the generated file
//   node scripts/sync-canonical-timing.mjs --check     # exit 1 if the committed file is stale

import { readFileSync, writeFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { dirname, join, relative, basename, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const SITE = resolve(__dirname, "..");
const REPO_ROOT = resolve(SITE, "..");
const ENVELOPE_DIR = join(REPO_ROOT, "bench", "canonical-competitor-results");
const OUT_PATH = join(SITE, "src", "data", "paper-timing.generated.json");

export const SCHEMA_VERSION = 1;

// The ONE envelope family derived today (see "HONEST SCOPE" above). Compared after
// normalisation because the committed envelopes are inconsistently escaped: some spell the
// separator as a real tab, others as a literal backslash-t inside the JSON string.
const SUPPORTED_TSV_FORMAT = "<query>\t<rows|ERROR>\t<best_us|engine>";

function normalizeTsvFormat(fmt) {
  return typeof fmt === "string" ? fmt.replace(/\\t/g, "\t").trim() : "";
}

// ---- envelope discovery -------------------------------------------------------------------
// Sorted so the derivation is deterministic regardless of directory-read order.
function findEnvelopes(dir) {
  if (!existsSync(dir)) return [];
  const out = [];
  for (const entry of readdirSync(dir).sort()) {
    const p = join(dir, entry);
    if (statSync(p).isDirectory()) out.push(...findEnvelopes(p));
    else if (entry.endsWith(".json")) out.push(p);
  }
  return out;
}

// ---- admission of a whole envelope --------------------------------------------------------
// Returns { ok: true, meta } or { ok: false, reason } — never throws on a malformed envelope,
// because an unrelated new file under bench/ must not break the site build; it is skipped
// with a reason instead.
function admitEnvelope(env, relPath) {
  if (env?.canonical !== true) {
    return { ok: false, reason: "does not self-declare `\"canonical\": true`" };
  }
  if (env.disqualified_note !== undefined || env.disqualified !== undefined) {
    return {
      ok: false,
      reason:
        "carries a free-text disqualification note (a machine cannot bound its scope; " +
        "refusing the whole envelope is the fail-closed direction)",
    };
  }
  const fmt = normalizeTsvFormat(env.tsv_format);
  if (fmt !== SUPPORTED_TSV_FORMAT) {
    return {
      ok: false,
      reason: `tsv_format ${JSON.stringify(env.tsv_format ?? null)} is not the derived ` +
        `same-box per-query family ${JSON.stringify(SUPPORTED_TSV_FORMAT)}`,
    };
  }
  if (typeof env.suite !== "string" || !env.suite) return { ok: false, reason: "missing `suite`" };
  if (typeof env.scale !== "string" || !env.scale) return { ok: false, reason: "missing `scale`" };
  if (typeof env.git_commit !== "string" || !env.git_commit) {
    return { ok: false, reason: "missing `git_commit` (provenance is mandatory)" };
  }
  const e = env.env;
  if (!e || typeof e !== "object" || typeof e.host_class !== "string" || !e.gathered_at_utc) {
    return {
      ok: false,
      reason: "missing `env.host_class` / `env.gathered_at_utc` (the forced provenance line " +
        "cannot be rendered without them)",
    };
  }
  // The aggregate is READ OFF the envelope's own methodology note rather than assumed, and
  // cross-checked against `iters` — so a note/iters disagreement fails closed instead of
  // silently mislabelling how the number was aggregated.
  const m = /min-of-(\d+)/.exec(String(env.canonical_note ?? ""));
  if (!m) {
    return {
      ok: false,
      reason: "`canonical_note` does not state a `min-of-N` aggregate (the aggregate is read " +
        "off the envelope, never assumed)",
    };
  }
  if (Number(m[1]) !== Number(env.iters)) {
    return {
      ok: false,
      reason: `aggregate/iters disagreement: canonical_note says min-of-${m[1]} but iters=${env.iters}`,
    };
  }
  return {
    ok: true,
    meta: {
      path: relPath,
      suite: env.suite,
      scale: env.scale,
      iters: env.iters,
      aggregate: `min-of-${m[1]}`,
      gitCommit: env.git_commit,
      gatheredAtUtc: e.gathered_at_utc,
      hostClass: e.host_class,
      cpuModel: e.cpu_model ?? null,
      nproc: e.nproc ?? null,
      quietBox: e.quiet_box ?? null,
      canonicalNote: env.canonical_note ?? null,
    },
  };
}

// ---- COUNT BEFORE TIME --------------------------------------------------------------------
// A per-(engine, query) admission check: publish a timing only for a row whose result count is
// verified correct. Returns null when admitted, else the reason it was refused.
function refuseOnCount(crosscheck, engine, query, count) {
  const cc = crosscheck?.[query];
  if (!cc || typeof cc !== "object") {
    return "no count_crosscheck entry (an unverified count may not back a published timing)";
  }
  if (cc.expected !== undefined && cc.expected !== null) {
    if (String(cc.expected) !== count) {
      return `count ${count} != expected ${cc.expected}`;
    }
    if (String(cc[engine] ?? "") !== count) {
      return `count_crosscheck disagrees with the TSV row (${JSON.stringify(cc[engine] ?? null)} vs ${count})`;
    }
    return null;
  }
  if (cc.all_agree !== true) {
    return "no expected-rows count is committed and the engines did not all agree";
  }
  return null;
}

// ---- the derivation -----------------------------------------------------------------------
export function deriveCanonicalTiming({ repoRoot = REPO_ROOT } = {}) {
  const envelopes = {};
  const records = {};
  const skipped = [];
  // suite -> { scale, path }: two envelopes keyed into the same suite namespace with DIFFERENT
  // datasets would let `latest wins` silently conflate two corpora, so that is a hard error.
  const suiteScale = new Map();
  // key -> the winning envelope's sort tuple, for the latest-wins resolution below.
  const winner = new Map();

  for (const abs of findEnvelopes(join(repoRoot, "bench", "canonical-competitor-results"))) {
    const relPath = relative(repoRoot, abs);
    let env;
    try {
      env = JSON.parse(readFileSync(abs, "utf8"));
    } catch (e) {
      skipped.push({ path: relPath, reason: `unparseable JSON: ${e.message}` });
      continue;
    }
    const verdict = admitEnvelope(env, relPath);
    if (!verdict.ok) {
      skipped.push({ path: relPath, reason: verdict.reason });
      continue;
    }
    const meta = verdict.meta;

    const prior = suiteScale.get(meta.suite);
    if (prior && prior.scale !== meta.scale) {
      throw new Error(
        `sync-canonical-timing: suite '${meta.suite}' appears with two different datasets — ` +
          `'${prior.scale}' (${prior.path}) vs '${meta.scale}' (${meta.path}). ` +
          "Latest-wins would conflate them; give one of the runs a distinct `suite`.",
      );
    }
    suiteScale.set(meta.suite, { scale: meta.scale, path: meta.path });

    const envelopeId = basename(relPath).replace(/\.json$/, "");
    if (envelopes[envelopeId] && envelopes[envelopeId].path !== relPath) {
      throw new Error(
        `sync-canonical-timing: envelope id collision '${envelopeId}' ` +
          `(${envelopes[envelopeId].path} vs ${relPath}) — rename one file.`,
      );
    }

    const engines = Object.keys(env)
      .filter((k) => k.endsWith("_tsv"))
      .map((k) => k.slice(0, -4))
      .sort();
    let admittedFromThisEnvelope = 0;
    const refusedRows = [];

    for (const engine of engines) {
      if (env.statuses?.[engine] !== "ok") {
        refusedRows.push(`${engine}: whole engine (status=${JSON.stringify(env.statuses?.[engine] ?? null)})`);
        continue;
      }
      for (const raw of String(env[`${engine}_tsv`]).split("\n")) {
        const line = raw.trim();
        if (!line) continue;
        const [query, count, best] = line.split("\t");
        if (!query || count === undefined || best === undefined) continue;
        if (!/^\d+$/.test(count)) continue; // ERROR / timeout row — never fabricated, never used
        const us = Number(best);
        if (!Number.isFinite(us) || us <= 0) continue; // the engine-name column of an ERROR row
        const refusal = refuseOnCount(env.count_crosscheck, engine, query, count);
        if (refusal) {
          refusedRows.push(`${engine}/${query}: ${refusal}`);
          continue;
        }
        const key = `timing.${meta.suite}.${engine}.${query}`;
        // Latest-wins per RECORD (not per envelope), so a narrow re-run of one query supersedes
        // just that query and the broader earlier run still supplies the rest. The provenance
        // rendered next to the number always names the envelope that actually won.
        const rank = [meta.gatheredAtUtc, relPath];
        const held = winner.get(key);
        if (held && (held[0] > rank[0] || (held[0] === rank[0] && held[1] > rank[1]))) continue;
        winner.set(key, rank);
        records[key] = {
          value: us,
          unit: "us",
          environment: "canonical-timing",
          kind: "measured-wall-clock",
          engine,
          suite: meta.suite,
          query,
          rows: Number(count),
          aggregate: meta.aggregate,
          envelope: envelopeId,
          source: `${relPath}#/${engine}_tsv (${query})`,
        };
        admittedFromThisEnvelope += 1;
      }
    }

    if (admittedFromThisEnvelope === 0) {
      skipped.push({
        path: relPath,
        reason: `admitted as canonical but no row passed the count check (${refusedRows.join("; ") || "no parseable rows"})`,
      });
      continue;
    }
    envelopes[envelopeId] = meta;
    if (refusedRows.length) meta.refusedRows = refusedRows.sort();
  }

  // Drop envelopes every one of whose records lost the latest-wins race, so `envelopes` is
  // exactly the set the records actually cite.
  const cited = new Set(Object.values(records).map((r) => r.envelope));
  for (const id of Object.keys(envelopes)) {
    if (!cited.has(id)) {
      skipped.push({
        path: envelopes[id].path,
        reason: "every row it contributed was superseded by a later canonical envelope",
      });
      delete envelopes[id];
    }
  }

  const sortKeys = (o) =>
    Object.fromEntries(Object.keys(o).sort().map((k) => [k, o[k]]));

  return {
    _comment:
      "GENERATED by site/scripts/sync-canonical-timing.mjs from the committed canonical " +
      "measurement envelopes under bench/canonical-competitor-results/**. DO NOT HAND-EDIT: " +
      "regenerate with `npm run sync-canonical-timing`. Every record is environment=" +
      "'canonical-timing' — a MEASURED wall-clock value from an envelope that self-declares " +
      "`\"canonical\": true` (the dedicated quiet-EC2 protocol), whose result count is verified " +
      "against the suite's committed expected-rows. Work-box timings are NON-CANONICAL and " +
      "cannot enter this file. Papers may render these ONLY through headline_timing() in " +
      "site/papers/_lib/timing.typ, which always prints the provenance alongside the number. " +
      "`skipped` is the honest record of which envelopes were left out and why.",
    schemaVersion: SCHEMA_VERSION,
    generatedBy: "site/scripts/sync-canonical-timing.mjs",
    envelopes: sortKeys(envelopes),
    records: sortKeys(records),
    skipped: skipped.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0)),
  };
}

// Single serialisation point so the writer and the drift-guard test compare identical bytes.
export function serializeCanonicalTiming(derived) {
  return JSON.stringify(derived, null, 2) + "\n";
}

export const TIMING_OUT_PATH = OUT_PATH;

function main(argv) {
  const check = argv.includes("--check");
  const derived = deriveCanonicalTiming();
  const text = serializeCanonicalTiming(derived);
  const nRecords = Object.keys(derived.records).length;
  const nEnvelopes = Object.keys(derived.envelopes).length;

  if (check) {
    const current = existsSync(OUT_PATH) ? readFileSync(OUT_PATH, "utf8") : null;
    if (current !== text) {
      console.error(
        "\n[canonical-timing] STALE: site/src/data/paper-timing.generated.json does not match a " +
          "fresh derivation from bench/canonical-competitor-results/**.\n" +
          "  Run `npm run sync-canonical-timing` and commit the result. Never hand-edit it.\n",
      );
      process.exit(1);
    }
    console.log(
      `[canonical-timing] up to date: ${nRecords} record(s) from ${nEnvelopes} canonical envelope(s).`,
    );
    return;
  }

  writeFileSync(OUT_PATH, text, "utf8");
  console.log(
    `[canonical-timing] wrote ${relative(SITE, OUT_PATH)}: ${nRecords} record(s) from ` +
      `${nEnvelopes} canonical envelope(s); ${derived.skipped.length} envelope(s) skipped.`,
  );
  for (const s of derived.skipped) console.log(`  skipped ${s.path}: ${s.reason}`);
}

// Only run the CLI when invoked directly (the module is imported by the build step + tests).
if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  main(process.argv.slice(2));
}
