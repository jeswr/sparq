// [OPUS-4.8] sq-xoxu — unit tests for the pure helpers that back the live /surface/full-text
// playground: shaping the SPARQL-1.1-JSON result set the wasm `TextSearch.query` binding
// returns into framework-free cells, and reading a numeric (text:score) cell. The BM25
// ranking + `text:` rewrite SEMANTICS themselves are proven by the Rust tests in
// crates/sparq-text(/ -wasm); here we only test the JS render of what that binding returns.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  formatTerm,
  resultVars,
  resultRows,
  resultCells,
  numericValue,
} from "../src/lib/text-results.ts";

// The quick-brown-fox walkthrough: `text:matches "quick fox" ; text:score ?score` over the
// corpus binds only <http://ex/a> (the literal with BOTH tokens), with its BM25 score as an
// xsd:double — exactly the SPARQL 1.1 JSON shape the W-text bundle's `query` returns.
const FOX_RESULTS = {
  head: { vars: ["s", "lit", "score"] },
  results: {
    bindings: [
      {
        s: { type: "uri", value: "http://ex/a" },
        lit: {
          type: "literal",
          value: "The quick brown fox jumps over the lazy dog",
        },
        score: {
          type: "literal",
          value: "0.6931472",
          datatype: "http://www.w3.org/2001/XMLSchema#double",
        },
      },
    ],
  },
};

test("formatTerm: uri, plain literal, and a typed score literal", () => {
  assert.equal(formatTerm({ type: "uri", value: "http://ex/a" }), "<http://ex/a>");
  assert.equal(formatTerm({ type: "literal", value: "fox" }), '"fox"');
  assert.equal(
    formatTerm({
      type: "literal",
      value: "0.69",
      datatype: "http://www.w3.org/2001/XMLSchema#double",
    }),
    '"0.69"^^xsd:double',
  );
});

test("formatTerm: a plain xsd:string literal drops the redundant datatype suffix", () => {
  assert.equal(
    formatTerm({
      type: "literal",
      value: "fox",
      datatype: "http://www.w3.org/2001/XMLSchema#string",
    }),
    '"fox"',
  );
});

test("formatTerm: an undefined (unbound) term renders as the empty string", () => {
  assert.equal(formatTerm(undefined), "");
});

test("resultVars / resultRows: read the projection and the solutions", () => {
  assert.deepEqual(resultVars(FOX_RESULTS), ["s", "lit", "score"]);
  assert.equal(resultRows(FOX_RESULTS).length, 1);
});

test("resultVars: an empty result set has no projection and no rows", () => {
  const empty = { head: { vars: [] }, results: { bindings: [] } };
  assert.deepEqual(resultVars(empty), []);
  assert.deepEqual(resultRows(empty), []);
});

test("resultCells: renders the ranked fox hit with its xsd:double score suffix", () => {
  const { vars, rows } = resultCells(FOX_RESULTS);
  assert.deepEqual(vars, ["s", "lit", "score"]);
  assert.deepEqual(rows, [
    [
      "<http://ex/a>",
      '"The quick brown fox jumps over the lazy dog"',
      '"0.6931472"^^xsd:double',
    ],
  ]);
});

test("resultCells: an unbound variable in a row renders as the empty string", () => {
  const partial = {
    head: { vars: ["s", "score"] },
    results: { bindings: [{ s: { type: "uri", value: "http://ex/a" } }] },
  };
  const { rows } = resultCells(partial);
  assert.deepEqual(rows, [["<http://ex/a>", ""]]);
});

test("numericValue: reads a finite score, null for unbound / non-numeric / uri", () => {
  const [row] = resultRows(FOX_RESULTS);
  assert.ok(Math.abs(numericValue(row, "score") - 0.6931472) < 1e-6);
  assert.equal(numericValue(row, "missing"), null);
  assert.equal(numericValue(row, "s"), null); // a uri is not a numeric literal
  assert.equal(
    numericValue({ x: { type: "literal", value: "not a number" } }, "x"),
    null,
  );
  assert.equal(numericValue(undefined, "score"), null);
});
