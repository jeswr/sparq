// [OPUS-4.8] sq-x0kp — unit tests for the framework-agnostic SPARQL-results formatting that
// backs the /try REPL results panel (packages/sparq-client/src/results.ts). These cover the
// PURE shaping the panel (and the Tauri webview) draw: table extraction from a SPARQL-1.1-JSON
// SELECT document, the ASK boolean discrimination, the CSV/TSV exports (incl. RFC-4180 quoting
// and TSV sanitisation), and the raw-JSON pretty-print. The query SEMANTICS themselves are
// proven by the Rust engine tests; here we only test the JS render of what the wasm binding
// returns. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  extractTable,
  resultVars,
  resultRows,
  isAskResult,
  askValue,
  resultsToCsv,
  resultsToTsv,
  csvCell,
  tsvCell,
  formatSparqlJson,
} from "../../packages/sparq-client/src/results.ts";

// A 2-column SELECT with: a URI + plain literal row, a URI + typed-integer row, and a row
// with an UNBOUND `name` (the OPTIONAL did not match) — exactly the SPARQL 1.1 JSON shape the
// lean wasm `Store.query` returns.
const SELECT_RESULTS = {
  head: { vars: ["person", "name"] },
  results: {
    bindings: [
      {
        person: { type: "uri", value: "http://ex/alice" },
        name: { type: "literal", value: "Alice" },
      },
      {
        person: { type: "uri", value: "http://ex/bob" },
        name: {
          type: "literal",
          value: "42",
          datatype: "http://www.w3.org/2001/XMLSchema#integer",
        },
      },
      // `name` is unbound in this solution.
      { person: { type: "uri", value: "http://ex/carol" } },
    ],
  },
};

const ASK_TRUE = { head: { vars: [] }, boolean: true };
const ASK_FALSE = { head: { vars: [] }, boolean: false };

test("resultVars / resultRows read the projection and the solutions in order", () => {
  assert.deepEqual(resultVars(SELECT_RESULTS), ["person", "name"]);
  assert.equal(resultRows(SELECT_RESULTS).length, 3);
});

test("resultVars / resultRows default to empty on a malformed document", () => {
  assert.deepEqual(resultVars({}), []);
  assert.deepEqual(resultRows({ head: { vars: ["x"] } }), []);
});

test("extractTable: columns from head vars, typed cells, unbound → empty string", () => {
  const table = extractTable(SELECT_RESULTS);
  assert.deepEqual(table.vars, ["person", "name"]);
  assert.deepEqual(table.rows, [
    ["<http://ex/alice>", '"Alice"'],
    ["<http://ex/bob>", '"42"^^xsd:integer'],
    // the unbound `name` cell is the empty string, and the row still has 2 columns
    ["<http://ex/carol>", ""],
  ]);
  // every row has exactly one cell per column
  for (const row of table.rows) assert.equal(row.length, table.vars.length);
});

test("extractTable: an empty SELECT keeps its columns and yields zero rows", () => {
  const empty = { head: { vars: ["s", "p", "o"] }, results: { bindings: [] } };
  const table = extractTable(empty);
  assert.deepEqual(table.vars, ["s", "p", "o"]);
  assert.deepEqual(table.rows, []);
});

test("isAskResult / askValue discriminate ASK from SELECT", () => {
  assert.equal(isAskResult(ASK_TRUE), true);
  assert.equal(isAskResult(ASK_FALSE), true);
  assert.equal(isAskResult(SELECT_RESULTS), false);
  assert.equal(askValue(ASK_TRUE), true);
  assert.equal(askValue(ASK_FALSE), false);
  assert.equal(askValue(SELECT_RESULTS), null);
});

test("extractTable on an ASK document yields an empty table (not a row)", () => {
  // The panel branches on isAskResult first; extractTable must not misrender a boolean.
  assert.deepEqual(extractTable(ASK_TRUE), { vars: [], rows: [] });
});

test("resultsToCsv: header + raw lexical values, no display decoration", () => {
  const csv = resultsToCsv(SELECT_RESULTS);
  assert.equal(
    csv,
    [
      "person,name",
      "http://ex/alice,Alice",
      // the typed integer exports its RAW lexical value (42), NOT "42"^^xsd:integer
      "http://ex/bob,42",
      // the unbound name is an empty trailing cell
      "http://ex/carol,",
    ].join("\r\n"),
  );
});

test("resultsToCsv: an empty SELECT exports just the header row", () => {
  const empty = { head: { vars: ["a", "b"] }, results: { bindings: [] } };
  assert.equal(resultsToCsv(empty), "a,b");
});

test("resultsToCsv: an ASK document exports the empty string (no projection)", () => {
  assert.equal(resultsToCsv(ASK_TRUE), "");
});

test("csvCell: RFC-4180 quoting of comma, quote, and newline", () => {
  assert.equal(csvCell("plain"), "plain");
  assert.equal(csvCell("a,b"), '"a,b"');
  assert.equal(csvCell('say "hi"'), '"say ""hi"""');
  assert.equal(csvCell("line1\nline2"), '"line1\nline2"');
});

test("resultsToCsv: a value containing a comma is quoted so columns stay aligned", () => {
  const tricky = {
    head: { vars: ["label"] },
    results: {
      bindings: [{ label: { type: "literal", value: "Doe, John" } }],
    },
  };
  assert.equal(resultsToCsv(tricky), 'label\r\n"Doe, John"');
});

test("resultsToTsv: tab-separated header + raw values", () => {
  const tsv = resultsToTsv(SELECT_RESULTS);
  assert.equal(
    tsv,
    [
      "person\tname",
      "http://ex/alice\tAlice",
      "http://ex/bob\t42",
      "http://ex/carol\t",
    ].join("\n"),
  );
});

test("tsvCell: embedded tab/newline are collapsed to a space (TSV has no quoting)", () => {
  assert.equal(tsvCell("a\tb"), "a b");
  assert.equal(tsvCell("line1\nline2"), "line1 line2");
  assert.equal(tsvCell("clean"), "clean");
});

test("resultsToTsv: a value with a tab is sanitised so the row stays one column-set", () => {
  const tricky = {
    head: { vars: ["x"] },
    results: { bindings: [{ x: { type: "literal", value: "a\tb" } }] },
  };
  // exactly one tab in the data row (the header has none for a single column)
  const dataRow = resultsToTsv(tricky).split("\n")[1];
  assert.equal(dataRow, "a b");
  assert.equal((dataRow.match(/\t/g) ?? []).length, 0);
});

test("formatSparqlJson: stable 2-space-indented re-serialisation of the document", () => {
  const pretty = formatSparqlJson(ASK_TRUE);
  assert.equal(pretty, JSON.stringify(ASK_TRUE, null, 2));
  // round-trips back to the same object (it is the engine's document, pretty-printed)
  assert.deepEqual(JSON.parse(pretty), ASK_TRUE);
  // SELECT documents pretty-print too
  assert.deepEqual(JSON.parse(formatSparqlJson(SELECT_RESULTS)), SELECT_RESULTS);
});
