// [OPUS-4.8] sq-11zy — unit tests for the pure helpers that back the live
// /surface/streaming-rsp playground: parsing the closed-window JSON array the wasm
// `Rsp.push`/`Rsp.flush` bindings return, and shaping each window's SPARQL-1.1-JSON
// table into framework-free cells. The wasm windowing SEMANTICS themselves (boundaries,
// lateness, R2S diffs) are proven by the Rust tests in crates/sparq-rsp(/ -wasm); here we
// only test the JS parse/render of what those bindings return. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  parseClosedWindows,
  windowVars,
  windowRows,
  windowCells,
  windowLabel,
  windowSummary,
} from "../src/lib/rsp-window.ts";

// The AVG(?v) tumbling-window walkthrough: a single closed window [0,60) whose result is
// the AVG of readings 10 and 20 = "15.0"^^xsd:decimal (SPARQL aggregate typing), exactly
// as the W-rsp bundle's `push` returns it for the default example.
const AVG_PUSH = JSON.stringify([
  {
    start: 0,
    end: 60,
    results: {
      head: { vars: ["avg"] },
      results: {
        bindings: [
          {
            avg: {
              type: "literal",
              value: "15.0",
              datatype: "http://www.w3.org/2001/XMLSchema#decimal",
            },
          },
        ],
      },
    },
  },
]);

test("parseClosedWindows: an empty array means no window closed", () => {
  assert.deepEqual(parseClosedWindows("[]"), []);
});

test("parseClosedWindows: parses {start,end,results} windows", () => {
  const windows = parseClosedWindows(AVG_PUSH);
  assert.equal(windows.length, 1);
  assert.equal(windows[0].start, 0);
  assert.equal(windows[0].end, 60);
  assert.equal(windows[0].results.head.vars[0], "avg");
});

test("parseClosedWindows: rejects a non-array payload", () => {
  assert.throws(() => parseClosedWindows('{"start":0}'), /array of closed windows/);
});

test("parseClosedWindows: rejects a malformed window object", () => {
  assert.throws(
    () => parseClosedWindows('[{"start":0,"end":"oops"}]'),
    /not \{start,end,results\}/,
  );
});

test("windowVars / windowRows: read the projection and solutions", () => {
  const [w] = parseClosedWindows(AVG_PUSH);
  assert.deepEqual(windowVars(w), ["avg"]);
  assert.equal(windowRows(w).length, 1);
});

test("windowCells: renders the decimal AVG with its xsd suffix", () => {
  const [w] = parseClosedWindows(AVG_PUSH);
  const { vars, rows } = windowCells(w);
  assert.deepEqual(vars, ["avg"]);
  assert.deepEqual(rows, [['"15.0"^^xsd:decimal']]);
});

test("windowCells: an unbound variable renders as the empty string", () => {
  const [w] = parseClosedWindows(
    JSON.stringify([
      {
        start: 0,
        end: 10,
        results: {
          head: { vars: ["s", "o"] },
          results: {
            bindings: [{ s: { type: "uri", value: "http://ex/a" } }],
          },
        },
      },
    ]),
  );
  const { rows } = windowCells(w);
  assert.deepEqual(rows, [["<http://ex/a>", ""]]);
});

test("windowLabel: the half-open bounds", () => {
  const [w] = parseClosedWindows(AVG_PUSH);
  assert.equal(windowLabel(w), "[0, 60)");
});

test("windowSummary: row count, and 'empty window' when the watermark jumped a gap", () => {
  const [w] = parseClosedWindows(AVG_PUSH);
  assert.equal(windowSummary(w), "[0, 60) — 1 row");

  const [empty] = parseClosedWindows(
    JSON.stringify([
      {
        start: 60,
        end: 120,
        results: { head: { vars: ["avg"] }, results: { bindings: [] } },
      },
    ]),
  );
  assert.equal(windowSummary(empty), "[60, 120) — empty window");
});
