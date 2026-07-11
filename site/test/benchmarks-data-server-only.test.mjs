// [FABLE-5] sq-qgkwy.1 — unit tests for the DATA-ISOLATION invariant of the bundle guard
// (scripts/check-bundle.mjs `findSnapshotLeaks`): the ~1.3 MB benchmark snapshot
// (src/data/benchmarks.generated.json) is SERVER-ONLY — it reaches the browser only as
// prerendered HTML/RSC, never embedded in a client JS chunk. Before this bead, client-bundled
// modules (metric-table & co., pulled in by the "use client" suite-group) imported the VALUE
// `fmtNum` from src/data/benchmarks.ts, folding the whole snapshot into /benchmarks/[type]'s
// first-load page chunk (~762 KB raw) on top of the already-prerendered HTML.
//
// A source-level "use client" scan is NOT sufficient — the leak came from modules with no
// directive at all, client-bundled transitively — so the guard fingerprints the BUILT chunks:
// `latest.commit` (a full commit sha) is by construction embedded in any chunk that inlines
// the JSON. These tests feed synthetic chunks; `npm run check:bundle` / postbuild runs the
// same function against the real .next artifacts.
import { test } from "node:test";
import assert from "node:assert/strict";

import { findSnapshotLeaks } from "../scripts/check-bundle.mjs";

const SHA = "0123456789abcdef0123456789abcdef01234567";

test("clean chunks pass", () => {
  const chunks = [
    { file: "static/chunks/3131-aa.js", text: "self.webpackChunk=[]" },
    { file: "static/chunks/app/benchmarks/[type]/page-bb.js", text: "render(tables)" },
  ];
  assert.deepEqual(findSnapshotLeaks(SHA, chunks), []);
});

test("a chunk embedding the snapshot is flagged (the real pre-fix regression shape)", () => {
  // A minified chunk inlining the JSON carries the latest.commit sha as a string literal.
  const chunks = [
    { file: "static/chunks/3131-aa.js", text: "self.webpackChunk=[]" },
    {
      file: "static/chunks/app/benchmarks/[type]/page-bb.js",
      text: `JSON.parse('{"latest":{"commit":"${SHA}","benches":[]}}')`,
    },
  ];
  assert.deepEqual(findSnapshotLeaks(SHA, chunks), [
    "static/chunks/app/benchmarks/[type]/page-bb.js",
  ]);
});

test("every offending chunk is reported, not just the first", () => {
  const chunks = [
    { file: "a.js", text: `x="${SHA}"` },
    { file: "b.js", text: "clean" },
    { file: "c.js", text: `y="${SHA}"` },
  ];
  assert.deepEqual(findSnapshotLeaks(SHA, chunks), ["a.js", "c.js"]);
});

test("an unusable fingerprint is a loud error, never a silent pass", () => {
  // If benchmarks.generated.json ever loses latest.commit, the guard must fail closed —
  // a missing/short fingerprint would otherwise make the scan vacuously green.
  for (const bad of [undefined, null, "", "abc1234"]) {
    assert.throws(() => findSnapshotLeaks(bad, [{ file: "a.js", text: "x" }]), /fingerprint/);
  }
});
