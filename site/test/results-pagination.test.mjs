// [OPUS-4.8] sq-9w4t (#817) — unit tests for the PAGINATED result rendering + bounded
// read-ahead page cache (packages/sparq-client/src/results.ts). These cover the PURE shaping
// the GUI Query workbench's SELECT table draws: slicing one display-page out of an extracted
// `ResultTable`, page-count / clamp arithmetic, and the read-ahead cache's lazy materialisation
// + bounded-window eviction. The query SEMANTICS are proven by the Rust engine tests; here we
// only test the JS page-math over the rows already streamed into JS. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  extractTable,
  pageCount,
  clampPage,
  paginateTable,
  createPageCache,
} from "../../packages/sparq-client/src/results.ts";

/** Build a SELECT-results doc with `n` single-column rows (?i = "row-0".."row-(n-1)"). */
function makeResults(n) {
  return {
    head: { vars: ["i"] },
    results: {
      bindings: Array.from({ length: n }, (_, k) => ({
        i: { type: "literal", value: `row-${k}` },
      })),
    },
  };
}

// --- pageCount / clampPage arithmetic -------------------------------------------------------

test("pageCount: ceil division, and ≥1 even for an empty/zero result", () => {
  assert.equal(pageCount(0, 50), 1);
  assert.equal(pageCount(1, 50), 1);
  assert.equal(pageCount(50, 50), 1);
  assert.equal(pageCount(51, 50), 2);
  assert.equal(pageCount(250, 100), 3);
  // pageSize ≤ 0 collapses to a single page.
  assert.equal(pageCount(999, 0), 1);
});

test("clampPage: negatives → 0, past-the-end → last page, floors fractions", () => {
  // 250 rows @ 100/pp → pages 0,1,2.
  assert.equal(clampPage(-5, 250, 100), 0);
  assert.equal(clampPage(0, 250, 100), 0);
  assert.equal(clampPage(2, 250, 100), 2);
  assert.equal(clampPage(99, 250, 100), 2);
  assert.equal(clampPage(1.9, 250, 100), 1);
  assert.equal(clampPage(NaN, 250, 100), 0);
});

// --- paginateTable: only the visible slice is shaped ----------------------------------------

test("paginateTable: slices exactly the requested page, with correct metadata", () => {
  const table = extractTable(makeResults(250));
  const p0 = paginateTable(table, 0, 100);
  assert.equal(p0.rows.length, 100);
  assert.deepEqual(p0.rows[0], ['"row-0"']);
  assert.deepEqual(p0.rows[99], ['"row-99"']);
  assert.equal(p0.page, 0);
  assert.equal(p0.startRow, 0);
  assert.equal(p0.totalRows, 250);
  assert.equal(p0.totalPages, 3);
  assert.deepEqual(p0.vars, ["i"]);

  // The last (short) page holds the remainder only.
  const p2 = paginateTable(table, 2, 100);
  assert.equal(p2.rows.length, 50);
  assert.deepEqual(p2.rows[0], ['"row-200"']);
  assert.equal(p2.startRow, 200);
});

test("paginateTable: an out-of-range page clamps to the last (never an empty slice)", () => {
  const table = extractTable(makeResults(250));
  const p = paginateTable(table, 99, 100);
  assert.equal(p.page, 2);
  assert.equal(p.rows.length, 50);
});

test("paginateTable: pageSize ≤ 0 yields one page of everything", () => {
  const table = extractTable(makeResults(7));
  const p = paginateTable(table, 0, 0);
  assert.equal(p.totalPages, 1);
  assert.equal(p.rows.length, 7);
});

test("paginateTable: an empty result is one empty page, not a crash", () => {
  const table = extractTable(makeResults(0));
  const p = paginateTable(table, 0, 100);
  assert.equal(p.totalPages, 1);
  assert.equal(p.totalRows, 0);
  assert.equal(p.rows.length, 0);
});

// --- createPageCache: lazy materialisation + bounded read-ahead window ----------------------

test("createPageCache: constructing it shapes NOTHING until a page is visited", () => {
  const table = extractTable(makeResults(1000)); // 10 pages @ 100
  const cache = createPageCache(table, 100, 2);
  assert.equal(cache.size, 0);
  assert.equal(cache.totalPages, 10);
  assert.equal(cache.pageSize, 100);
});

test("createPageCache: get warms ~readAhead pages ahead (and behind), bounded", () => {
  const table = extractTable(makeResults(1000)); // 10 pages @ 100
  const cache = createPageCache(table, 100, 2);
  const p = cache.get(0);
  assert.equal(p.page, 0);
  assert.deepEqual(p.rows[0], ['"row-0"']);
  // Page 0 + 2 ahead warmed (no pages behind 0). The bounded window cap is 1 + 2·readAhead = 5.
  assert.equal(cache.size, 3);
  assert.ok(cache.size <= 1 + 2 * 2);
});

test("createPageCache: paging across the result never exceeds the bounded window", () => {
  const table = extractTable(makeResults(10_000)); // 100 pages @ 100
  const cache = createPageCache(table, 100, 2);
  const cap = 1 + 2 * 2;
  for (let pg = 0; pg < 100; pg += 1) {
    const got = cache.get(pg);
    assert.equal(got.page, pg);
    assert.ok(
      cache.size <= cap,
      `cache held ${cache.size} slices at page ${pg}, exceeding the ${cap} window`,
    );
  }
});

test("createPageCache: the slice is the right rows for the requested page", () => {
  const table = extractTable(makeResults(1000));
  const cache = createPageCache(table, 100, 2);
  const p5 = cache.get(5);
  assert.equal(p5.startRow, 500);
  assert.deepEqual(p5.rows[0], ['"row-500"']);
  assert.deepEqual(p5.rows[99], ['"row-599"']);
});
