// [OPUS-5] sq-f4pmk (#2933) — `streamQueryRows`'s DEMAND-DRIVEN pull bound (`options.maxRows`).
//
// WHY THIS SUITE EXISTS. The GUI workbench keeps at most `rowCap` SELECT rows in JS and every
// view + export renders only those kept rows, yet the pull used to DRAIN the whole cursor —
// building a SPARQL-JSON string in wasm and `JSON.parse`-ing it in JS for every row past the
// cap, only to drop it. `maxRows` stops the pull there instead. Because the cursor is
// forward-only over an already-materialised result, the obligation is a RESULT-EQUIVALENCE
// one: a bounded pull must deliver exactly the PREFIX of the batches an unbounded pull
// delivers, and must not change the cursor introspection the caller derives `totalRows` /
// `truncated` from. Both are asserted below against the unbounded path itself (a differential),
// not against hand-written expectations.
//
// CI coverage: this suite runs in the GATING gui.yml `shared TS client typecheck` job
// (`npm test` in packages/sparq-client) — see test/decompress.test.mjs for why js.yml's verdict
// says nothing about this directory.
import assert from "node:assert/strict";
import { test } from "node:test";

import { streamQueryRows } from "../src/index.ts";

/**
 * A faithful stand-in for the wasm `SolutionCursor` (crates/sparq-wasm/src/lib.rs). The
 * advance/exhaustion arithmetic is transcribed from `SolutionCursor::next` so the quirk the
 * production code depends on is reproduced exactly: a ZERO-row result yields one EMPTY batch
 * and is then exhausted, while a non-empty result stops as soon as `pos == total`.
 *
 * `stats` records how much work the pull actually asked for — the structural measurement this
 * change is about (batches pulled, rows serialised), plus whether the cursor was freed.
 */
function fakeStore(totalRows, vars = ["s", "p"]) {
  const stats = { pulls: 0, rowsSerialised: 0, freed: 0, batchSize: 0 };
  const row = (i) => Object.fromEntries(vars.map((v) => [v, { type: "literal", value: `${v}${i}` }]));
  const store = {
    queryCursor(_sparql, batchSize) {
      stats.batchSize = batchSize;
      let pos = 0;
      return {
        vars: () => [...vars],
        rowCount: () => totalRows,
        batchSize: () => batchSize,
        next() {
          if (pos > totalRows || (pos === totalRows && totalRows !== 0)) return undefined;
          const end = Math.min(pos + batchSize, totalRows);
          const bindings = [];
          for (let i = pos; i < end; i += 1) bindings.push(row(i));
          pos = totalRows === 0 ? 1 : end;
          stats.pulls += 1;
          stats.rowsSerialised += bindings.length;
          // The real cursor hands back a standalone SPARQL-JSON *string*; keep that so the
          // JSON.parse cost the bound is meant to avoid is genuinely on this path.
          return JSON.stringify({ head: { vars }, results: { bindings } });
        },
        free() {
          stats.freed += 1;
        },
      };
    },
  };
  return { store, stats };
}

/** Run a pull and collect everything observable about it. */
function pull(totalRows, batchSize, options) {
  const { store, stats } = fakeStore(totalRows);
  const batches = [];
  const meta = streamQueryRows(store, "SELECT * WHERE { ?s ?p ?o }", batchSize, (b) => {
    batches.push({ index: b.index, cumulative: b.cumulative, rows: b.rows });
  }, options);
  return { meta, batches, stats, rows: batches.flatMap((b) => b.rows) };
}

// ---------------------------------------------------------------------------
// The unbounded path is unchanged (every existing caller passes no options).
// ---------------------------------------------------------------------------

test("no options drains the cursor, exactly as before", () => {
  const { meta, batches, stats, rows } = pull(2_500, 1_000);
  assert.equal(stats.pulls, 3);
  assert.equal(stats.rowsSerialised, 2_500);
  assert.equal(rows.length, 2_500);
  assert.deepEqual(
    batches.map((b) => b.rows.length),
    [1_000, 1_000, 500],
  );
  assert.deepEqual(
    batches.map((b) => b.cumulative),
    [1_000, 2_000, 2_500],
  );
  assert.equal(meta.rowCount, 2_500);
  assert.equal(meta.drained, true);
  assert.equal(stats.freed, 1);
});

test("an empty result still yields exactly one empty batch, bounded or not", () => {
  for (const options of [undefined, { maxRows: 1 }, { maxRows: 5_000 }]) {
    const { meta, batches, stats } = pull(0, 1_000, options);
    assert.equal(batches.length, 1, `one batch for ${JSON.stringify(options)}`);
    assert.equal(batches[0].rows.length, 0);
    assert.equal(meta.rowCount, 0);
    // Nothing was yielded, so no bound can have been reached: the cursor is DRAINED, and a
    // caller may still trust the `rowCount || counted-total` fallback.
    assert.equal(meta.drained, true);
    assert.equal(stats.freed, 1);
  }
});

// ---------------------------------------------------------------------------
// The bound: where it stops, and that stopping changes nothing else.
// ---------------------------------------------------------------------------

test("maxRows stops at the batch that reaches the bound, not before it", () => {
  // 5_000-row cap over 1_000-row batches on a 1_000_000-row result: 5 pulls, not 1_000.
  const { meta, batches, stats, rows } = pull(1_000_000, 1_000, { maxRows: 5_000 });
  assert.equal(stats.pulls, 5);
  assert.equal(stats.rowsSerialised, 5_000);
  assert.equal(rows.length, 5_000);
  assert.equal(batches.at(-1).cumulative, 5_000);
  assert.equal(meta.drained, false);
  assert.equal(stats.freed, 1);
});

test("a bound that lands mid-batch still delivers that whole batch", () => {
  // Batches are never split: a 1_500 bound over 1_000-row batches pulls two batches (2_000
  // rows) and leaves the trimming to the consumer.
  const { batches, stats, rows } = pull(10_000, 1_000, { maxRows: 1_500 });
  assert.equal(stats.pulls, 2);
  assert.equal(rows.length, 2_000);
  assert.deepEqual(
    batches.map((b) => b.rows.length),
    [1_000, 1_000],
  );
});

test("a bound at or above the result size drains, and never over-pulls", () => {
  for (const maxRows of [2_500, 2_501, 10_000]) {
    const { meta, stats, rows } = pull(2_500, 1_000, { maxRows });
    assert.equal(rows.length, 2_500, `all rows for maxRows=${maxRows}`);
    assert.equal(stats.pulls, 3);
    // `drained` reports the ROWS, not whether the extra exhausting `next()` was made: a bound
    // reached exactly on the last batch has still delivered the complete result, so a caller
    // using the flag for continuation is told there is nothing left to fetch.
    assert.equal(meta.drained, true, `drained for maxRows=${maxRows}`);
  }
});

test("a bound landing inside the FINAL PARTIAL batch reports drained", () => {
  // 2_500 rows over 1_000-row batches: the bound is crossed by the 500-row tail batch, which
  // completes the result even though the pull stopped at the bound, not at exhaustion.
  const { meta, stats, rows } = pull(2_500, 1_000, { maxRows: 2_400 });
  assert.equal(stats.pulls, 3);
  assert.equal(rows.length, 2_500);
  assert.equal(meta.drained, true);
  // ...whereas a bound that stops before the tail genuinely leaves rows behind.
  const short = pull(2_500, 1_000, { maxRows: 1_200 });
  assert.equal(short.stats.pulls, 2);
  assert.equal(short.rows.length, 2_000);
  assert.equal(short.meta.drained, false);
});

test("a non-positive or non-finite maxRows means no bound", () => {
  for (const maxRows of [undefined, 0, -1, Number.NaN, Number.POSITIVE_INFINITY]) {
    const { meta, stats, rows } = pull(2_500, 1_000, { maxRows });
    assert.equal(rows.length, 2_500, `drained for maxRows=${String(maxRows)}`);
    assert.equal(stats.pulls, 3);
    assert.equal(meta.drained, true);
  }
});

test("cursor introspection is identical bounded vs drained", () => {
  // This is what keeps the consumer's `totalRows` / `truncated` honest: `rowCount` is read
  // from the cursor BEFORE the first pull, so an early stop cannot understate the total.
  const bounded = pull(50_000, 1_000, { maxRows: 5_000 });
  const drained = pull(50_000, 1_000);
  assert.deepEqual(bounded.meta.vars, drained.meta.vars);
  assert.equal(bounded.meta.rowCount, drained.meta.rowCount);
  assert.equal(bounded.meta.rowCount, 50_000);
  assert.equal(bounded.stats.pulls, 5);
  assert.equal(drained.stats.pulls, 50);
  assert.equal(bounded.meta.batchSize, drained.meta.batchSize);
});

test("the cursor is freed when the bound stops the pull and when onBatch throws", () => {
  const { store, stats } = fakeStore(10_000);
  streamQueryRows(store, "SELECT *", 1_000, () => {}, { maxRows: 100 });
  assert.equal(stats.freed, 1);

  const boom = fakeStore(10_000);
  assert.throws(
    () =>
      streamQueryRows(
        boom.store,
        "SELECT *",
        1_000,
        () => {
          throw new Error("consumer exploded");
        },
        { maxRows: 5_000 },
      ),
    /consumer exploded/,
  );
  assert.equal(boom.stats.freed, 1);
});

// ---------------------------------------------------------------------------
// RESULT EQUIVALENCE — randomised differentials against the unbounded path.
// ---------------------------------------------------------------------------

/** A seeded LCG so a failure is reproducible (no Math.random in a differential). */
function rng(seed) {
  let s = seed >>> 0;
  return (n) => {
    s = (Math.imul(s, 1_664_525) + 1_013_904_223) >>> 0;
    return s % n;
  };
}

test("randomised: a bounded pull delivers a strict PREFIX of the drained pull", () => {
  const next = rng(2_933);
  for (let i = 0; i < 200; i += 1) {
    const totalRows = next(3_000);
    const batchSize = 1 + next(64);
    const maxRows = next(3_200); // includes 0 (= no bound) and values past the result size
    const bounded = pull(totalRows, batchSize, { maxRows });
    const drained = pull(totalRows, batchSize);
    const label = `total=${totalRows} batch=${batchSize} max=${maxRows}`;

    // 1. Batch-for-batch prefix: same count, same index, same cumulative, same rows.
    assert.ok(bounded.batches.length <= drained.batches.length, `no over-pull (${label})`);
    assert.deepEqual(
      bounded.batches,
      drained.batches.slice(0, bounded.batches.length),
      `prefix (${label})`,
    );

    // 2. Every row up to the bound is present and byte-identical to the drained answer.
    const wanted = maxRows > 0 ? Math.min(maxRows, totalRows) : totalRows;
    assert.ok(bounded.rows.length >= wanted, `covers the bound (${label})`);
    assert.deepEqual(
      bounded.rows.slice(0, wanted),
      drained.rows.slice(0, wanted),
      `rows (${label})`,
    );

    // 3. The introspection the consumer derives its totals from is untouched.
    assert.equal(bounded.meta.rowCount, drained.meta.rowCount, `rowCount (${label})`);
    assert.deepEqual(bounded.meta.vars, drained.meta.vars, `vars (${label})`);

    // 3b. `drained` is a statement about the ROWS: true exactly when the bounded pull
    //     delivered the whole result, whether it stopped at the bound or at exhaustion. So
    //     `drained === false` always means "rows remain", which is what a paginating caller
    //     reads it as; a bounded pull that finished the result agrees with the drained one.
    assert.equal(
      bounded.meta.drained,
      bounded.rows.length >= totalRows,
      `drained (${label})`,
    );
    if (bounded.meta.drained) {
      assert.deepEqual(bounded.rows, drained.rows, `complete result (${label})`);
    }

    // 4. No wasted serialisation: work is bounded by the rows actually asked for, rounded up
    //    to a whole batch. (The point of the change — asserted, not claimed.) A bound never
    //    costs MORE than a drain, and costs strictly less whenever the rounded-up bound does
    //    not already span the whole result.
    assert.ok(
      bounded.stats.rowsSerialised <= drained.stats.rowsSerialised,
      `never over-serialises (${label})`,
    );
    if (maxRows > 0 && maxRows < totalRows) {
      const pulls = Math.ceil(maxRows / batchSize);
      assert.equal(bounded.stats.pulls, pulls, `pull count (${label})`);
      if (pulls * batchSize < totalRows) {
        assert.ok(
          bounded.stats.rowsSerialised < drained.stats.rowsSerialised,
          `saves serialisation (${label})`,
        );
      }
    }
  }
});

test("randomised: the GUI row-cap consumer keeps the same rows either way", () => {
  // The exact composition gui/app/src/lib/engine-context.tsx performs: append until `rowCap`,
  // then derive `totalRows` from the cursor. Bounded and drained must agree on ALL THREE
  // outcome fields — kept rows, totalRows, truncated — for every shape.
  const consume = (totalRows, batchSize, rowCap, bound) => {
    const { store } = fakeStore(totalRows);
    const kept = [];
    let counted = 0;
    const meta = streamQueryRows(
      store,
      "SELECT *",
      batchSize,
      (batch) => {
        counted += batch.rows.length;
        for (const r of batch.rows) if (kept.length < rowCap) kept.push(r);
      },
      bound ? { maxRows: rowCap } : {},
    );
    const total = meta.rowCount || counted;
    return { kept, totalRows: total, truncated: total > kept.length };
  };

  const next = rng(24_601);
  for (let i = 0; i < 200; i += 1) {
    const totalRows = next(3_000);
    const batchSize = 1 + next(64);
    const rowCap = 1 + next(1_500);
    const label = `total=${totalRows} batch=${batchSize} cap=${rowCap}`;
    assert.deepEqual(
      consume(totalRows, batchSize, rowCap, true),
      consume(totalRows, batchSize, rowCap, false),
      `outcome (${label})`,
    );
  }
});
