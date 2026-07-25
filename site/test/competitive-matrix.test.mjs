// [SONNET-4.6] sq-vw3ax.16 — data-level guard for the /compare honesty invariant.
//
// The page (app/compare/page.tsx) and the data header both make a build-in-public
// promise about beads. The ACCURATE, narrowed claim is:
//   * a PARTIAL/GAP row that names TRACKED future work carries its governing bead id, and
//   * a row without a bead must OPENLY flag that it is untracked ("deferred" / "not beaded")
//     rather than silently omit the link while implying the work is tracked.
// This test pins exactly that so the copy and the data cannot drift apart (e.g. a future
// row naming future work with neither a bead nor an honest "not beaded" acknowledgement).
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import { MATRIX } from "../src/data/competitive-matrix.ts";

const ALL_ROWS = MATRIX.flatMap((section) =>
  section.rows.map((row) => ({ section: section.id, row })),
);

// A well-formed bead id: `sq-` + a base slug + optional dotted sub-task suffixes.
const BEAD_ID = /^sq-[a-z0-9]+(?:\.\d+)*$/;

// Honest self-declarations that a row's future work is NOT tracked by a bead. Kept in
// sync with the copy in data/competitive-matrix.ts and app/compare/page.tsx.
const UNTRACKED_MARKERS = ["not beaded", "deferred", "parked", "scoped out"];

function flagsUntracked(note) {
  const lower = note.toLowerCase();
  return UNTRACKED_MARKERS.some((m) => lower.includes(m));
}

test("every bead id in the matrix is well-formed", () => {
  for (const { section, row } of ALL_ROWS) {
    for (const bead of row.beads ?? []) {
      assert.match(
        bead,
        BEAD_ID,
        `row "${row.feature}" (${section}) has a malformed bead id: ${bead}`,
      );
    }
  }
});

test("beads, where present, are non-empty and unique within a row", () => {
  for (const { section, row } of ALL_ROWS) {
    if (row.beads === undefined) continue;
    assert.ok(
      row.beads.length > 0,
      `row "${row.feature}" (${section}) has an empty beads array — omit the field instead`,
    );
    assert.equal(
      new Set(row.beads).size,
      row.beads.length,
      `row "${row.feature}" (${section}) lists a duplicate bead id`,
    );
  }
});

test("every PARTIAL/GAP row is either bead-tracked or openly flags itself untracked", () => {
  const offenders = [];
  for (const { section, row } of ALL_ROWS) {
    if (row.tier !== "PARTIAL" && row.tier !== "GAP") continue;
    const tracked = (row.beads?.length ?? 0) > 0;
    if (!tracked && !flagsUntracked(row.note)) {
      offenders.push(`"${row.feature}" (${section})`);
    }
  }
  assert.deepEqual(
    offenders,
    [],
    "these PARTIAL/GAP rows name future work with neither a governing bead nor an " +
      'honest "deferred"/"not beaded" acknowledgement, contradicting the /compare copy:\n  ' +
      offenders.join("\n  "),
  );
});
