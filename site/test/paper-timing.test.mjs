// [OPUS-5] sq-gum8.16 (paper factory F4) — unit tests for the derived `canonical-timing`
// evidence class: the drift guard, the class invariants, and — the load-bearing ones — the two
// honesty properties the class exists to guarantee:
//   * a NON-canonical (work-box) envelope cannot contribute a number, and
//   * COUNT BEFORE TIME: a comparator that returned the wrong number of rows cannot have its
//     timing published, even though the timing itself is present in the envelope.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

import {
  deriveCanonicalTiming,
  serializeCanonicalTiming,
  TIMING_OUT_PATH,
} from "../scripts/sync-canonical-timing.mjs";

const SITE = dirname(fileURLToPath(import.meta.url)).replace(/\/test$/, "");

// A minimal envelope in the derived family. `canonical` and the count columns are the knobs the
// honesty tests below turn.
function envelope({ canonical, suite = "demo", counts, tsv }) {
  return {
    canonical,
    canonical_note: "CANONICAL: dedicated quiet EC2. min-of-5 on loaded stores.",
    git_commit: "deadbeef",
    suite,
    scale: "demo corpus, 10 triples",
    iters: 5,
    tsv_format: "<query>\t<rows|ERROR>\t<best_us|engine>",
    engines: { sparq: { version: "deadbeef" }, other: { version: "1.0" } },
    statuses: { sparq: "ok", other: "ok" },
    count_crosscheck: counts,
    env: {
      host_class: "dedicated-quiet-ec2-c6i.4xlarge (CANONICAL 16 vCPU / 32 GiB x86_64)",
      cpu_model: "Demo CPU",
      nproc: 16,
      quiet_box: true,
      gathered_at_utc: "2026-01-01T00:00:00Z",
    },
    ...tsv,
  };
}

// Writes envelopes into a throwaway repo root so the derivation can be exercised on crafted
// inputs without touching the real bench/ tree.
function deriveFromFixture(envelopes) {
  const root = mkdtempSync(join(tmpdir(), "sparq-timing-fixture-"));
  const dir = join(root, "bench", "canonical-competitor-results", "fixture");
  mkdirSync(dir, { recursive: true });
  for (const [name, env] of Object.entries(envelopes)) {
    writeFileSync(join(dir, `${name}.json`), JSON.stringify(env, null, 2), "utf8");
  }
  return deriveCanonicalTiming({ repoRoot: root });
}

test("the committed paper-timing.generated.json matches a fresh derivation (drift guard)", () => {
  const expected = serializeCanonicalTiming(deriveCanonicalTiming());
  const actual = readFileSync(TIMING_OUT_PATH, "utf8");
  assert.equal(
    actual,
    expected,
    "src/data/paper-timing.generated.json is stale or hand-edited — run `npm run sync-canonical-timing`",
  );
});

test("every derived record satisfies the canonical-timing class invariants", () => {
  const d = deriveCanonicalTiming();
  assert.ok(Object.keys(d.records).length > 0, "the committed envelopes must yield some records");
  for (const [key, r] of Object.entries(d.records)) {
    assert.equal(r.environment, "canonical-timing", `${key}: wrong environment`);
    assert.equal(r.unit, "us", `${key}: every derived timing is in microseconds`);
    assert.ok(Number.isFinite(r.value) && r.value > 0, `${key}: value must be a positive number`);
    assert.equal(key, `timing.${r.suite}.${r.engine}.${r.query}`, `${key}: key/fields disagree`);
    assert.match(r.aggregate, /^min-of-\d+$/, `${key}: aggregate must be read off the envelope`);
    // The forced provenance line cannot render without a resolvable, fully-described envelope.
    const e = d.envelopes[r.envelope];
    assert.ok(e, `${key}: cites unknown envelope ${r.envelope}`);
    for (const field of ["path", "scale", "hostClass", "gitCommit", "gatheredAtUtc", "aggregate"]) {
      assert.ok(e[field], `envelope ${r.envelope}: missing provenance field ${field}`);
    }
    assert.match(e.path, /^bench\/canonical-competitor-results\//, "envelopes come only from the canonical tree");
  }
});

test("a non-canonical (work-box) envelope cannot contribute a timing", () => {
  const counts = { q01: { sparq: "1", other: "1", all_agree: true, expected: "1" } };
  const tsv = { sparq_tsv: "q01\t1\t42.0\n", other_tsv: "q01\t1\t99.0\n" };
  const d = deriveFromFixture({
    good: envelope({ canonical: true, suite: "yes", counts, tsv }),
    workbox: envelope({ canonical: false, suite: "no", counts, tsv }),
  });
  assert.ok(d.records["timing.yes.sparq.q01"], "the canonical envelope must be derived");
  assert.equal(
    Object.keys(d.records).filter((k) => k.startsWith("timing.no.")).length,
    0,
    "an envelope that does not self-declare `canonical: true` must contribute nothing",
  );
  assert.ok(
    d.skipped.some((s) => /workbox\.json$/.test(s.path) && /canonical/.test(s.reason)),
    "the refusal must be recorded in `skipped` with a reason",
  );
});

test("COUNT BEFORE TIME: a wrong-count engine's timing is refused, a correct one's is kept", () => {
  // `other` is FASTER (1.0 µs) but returned 0 rows where 358 were expected — publishing that
  // timing would be the archetypal dishonest comparison.
  const d = deriveFromFixture({
    e: envelope({
      canonical: true,
      counts: {
        q08: { sparq: "358", other: "0", all_agree: false, expected: "358", matches_expected: false },
      },
      tsv: { sparq_tsv: "q08\t358\t153318.0\n", other_tsv: "q08\t0\t1.0\n" },
    }),
  });
  assert.ok(d.records["timing.demo.sparq.q08"], "the engine with the correct count is published");
  assert.equal(
    d.records["timing.demo.other.q08"],
    undefined,
    "the engine with the wrong row count must NOT have its timing published",
  );
  const env = d.envelopes[Object.keys(d.envelopes)[0]];
  assert.ok(
    env.refusedRows.some((r) => r.startsWith("other/q08:")),
    "the per-row refusal must be recorded on the envelope, not silently dropped",
  );
});

test("this repo's real envelopes exercise the count rule (the guard is not vacuous)", () => {
  const d = deriveCanonicalTiming();
  // SP2Bench q08/q12b: one comparator returns 0 rows against a committed expected count, so its
  // timing must be absent while the engines that agreed with expected-rows are present.
  assert.ok(d.records["timing.sp2b.sparq.q08"], "sparq q08 (correct count) must be present");
  assert.equal(d.records["timing.sp2b.qlever.q08"], undefined, "a 0-row q08 result must be refused");
  assert.equal(d.records["timing.sp2b.qlever.q12b"], undefined, "a 0-row q12b result must be refused");
  const refused = Object.values(d.envelopes).flatMap((e) => e.refusedRows ?? []);
  assert.ok(
    refused.some((r) => /^qlever\/q08: count 0 != expected 358$/.test(r)),
    "the refusal reason must be recorded verbatim",
  );
});

test("an ERROR / timeout row is never turned into a number", () => {
  const d = deriveFromFixture({
    e: envelope({
      canonical: true,
      counts: { q01: { sparq: "1", other: "1", all_agree: true, expected: "1" } },
      tsv: { sparq_tsv: "q01\t1\t42.0\n", other_tsv: "q01\tERROR\tother\n" },
    }),
  });
  assert.ok(d.records["timing.demo.sparq.q01"]);
  assert.equal(d.records["timing.demo.other.q01"], undefined, "an ERROR row must not be derived");
});

test("an envelope carrying a free-text disqualification note is refused whole", () => {
  const base = envelope({
    canonical: true,
    counts: { q01: { sparq: "1", other: "1", all_agree: true, expected: "1" } },
    tsv: { sparq_tsv: "q01\t1\t42.0\n" },
  });
  const d = deriveFromFixture({ e: { ...base, disqualified_note: "engine X is not a valid comparator here" } });
  assert.equal(Object.keys(d.records).length, 0, "a disqualification note the machine cannot scope fails closed");
  assert.ok(d.skipped.some((s) => /disqualification/.test(s.reason)));
});

test("an envelope whose stated aggregate disagrees with `iters` is refused", () => {
  const base = envelope({
    canonical: true,
    counts: { q01: { sparq: "1", all_agree: true, expected: "1" } },
    tsv: { sparq_tsv: "q01\t1\t42.0\n" },
  });
  const d = deriveFromFixture({ e: { ...base, iters: 3 } }); // note still says min-of-5
  assert.equal(Object.keys(d.records).length, 0);
  assert.ok(d.skipped.some((s) => /aggregate\/iters disagreement/.test(s.reason)));
});

test("latest-wins is per record, so a narrow re-run supersedes only the queries it covers", () => {
  const counts = {
    q01: { sparq: "1", all_agree: true, expected: "1" },
    q02: { sparq: "2", all_agree: true, expected: "2" },
  };
  const older = envelope({ canonical: true, counts, tsv: { sparq_tsv: "q01\t1\t10.0\nq02\t2\t20.0\n" } });
  const newer = envelope({ canonical: true, counts, tsv: { sparq_tsv: "q01\t1\t5.0\n" } });
  newer.env = { ...newer.env, gathered_at_utc: "2026-02-02T00:00:00Z" };
  const d = deriveFromFixture({ a_older: older, b_newer: newer });
  assert.equal(d.records["timing.demo.sparq.q01"].value, 5.0, "the later envelope wins its own query");
  assert.equal(d.records["timing.demo.sparq.q02"].value, 20.0, "the earlier envelope still supplies q02");
  assert.equal(Object.keys(d.envelopes).length, 2, "both envelopes are cited, each by the records it won");
});

test("two envelopes claiming one suite with different datasets is a hard error", () => {
  const counts = { q01: { sparq: "1", all_agree: true, expected: "1" } };
  const tsv = { sparq_tsv: "q01\t1\t10.0\n" };
  const a = envelope({ canonical: true, counts, tsv });
  const b = envelope({ canonical: true, counts, tsv });
  b.scale = "a completely different corpus, 10000000 triples";
  assert.throws(
    () => deriveFromFixture({ a, b }),
    /two different datasets/,
    "latest-wins across differing corpora would conflate them, so it must fail closed",
  );
});

test("timing.typ exposes NO raw-value accessor (provenance is unavoidable)", () => {
  const src = readFileSync(join(SITE, "papers", "_lib", "timing.typ"), "utf8");
  // The only key-taking accessors are the internal lookup and the two provenance-forced ones.
  const accessors = [...src.matchAll(/^#let\s+([a-z_]+)\(key\)\s*=/gm)].map((m) => m[1]);
  assert.deepEqual(
    accessors.sort(),
    ["_trec", "headline_timing", "timing_provenance"],
    "a new key-taking accessor in timing.typ must not be able to return a bare value — a " +
      "measured number may only render with its provenance",
  );
  // `headline_timing` must actually emit provenance, not just be named as though it does.
  const body = src.slice(src.indexOf("#let headline_timing(key) ="));
  assert.match(body, /aggregate/, "headline_timing must render the aggregate");
  assert.match(body, /hostClass/, "headline_timing must render the host class");
  assert.match(body, /gitCommit/, "headline_timing must render the git commit");
});
