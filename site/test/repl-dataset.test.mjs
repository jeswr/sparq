// [OPUS-4.8] sq-17nw — unit tests for the REPL dataset helpers that make the live
// SPARQL REPL PRESERVE named graphs (GRAPH) on upload instead of folding everything
// into the default graph. These cover the pure, framework-free decision + serialisation
// logic; the engine-level named-graph behaviour is already proven by the Rust tests and
// js/test/store.test.mjs (loadDataset). Run via `npm run test:unit` (a node:test process
// with the in-repo TypeScript ESM loader).
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  isDatasetFormat,
  ALL_QUADS_QUERY,
  rowsToNQuads,
} from "../src/lib/repl-dataset.ts";

test("isDatasetFormat is true exactly for the quad-bearing formats", () => {
  // Quad formats carry named graphs and must be loaded with loadDataset.
  assert.equal(isDatasetFormat("nquads"), true);
  assert.equal(isDatasetFormat("trig"), true);
  // Triple-only formats have no named graphs; load() is correct (and cheaper).
  assert.equal(isDatasetFormat("turtle"), false);
  assert.equal(isDatasetFormat("ntriples"), false);
  // Unknown / empty defaults to the non-dataset path.
  assert.equal(isDatasetFormat("nt"), false);
  assert.equal(isDatasetFormat(""), false);
});

test("ALL_QUADS_QUERY spans the default graph AND every named graph", () => {
  // A UNION of the default-graph pattern and the GRAPH ?g pattern: the only way to
  // enumerate the WHOLE dataset (the default-graph-only `{ ?s ?p ?o }` misses named
  // graphs, which is exactly the folding bug this bead fixes).
  assert.match(ALL_QUADS_QUERY, /GRAPH \?g/);
  assert.match(ALL_QUADS_QUERY, /UNION/);
});

test("rowsToNQuads emits a triple line for default-graph rows (no ?g)", () => {
  const rows = [
    {
      s: { type: "uri", value: "http://ex/a" },
      p: { type: "uri", value: "http://ex/p" },
      o: { type: "literal", value: "default" },
    },
  ];
  assert.equal(
    rowsToNQuads(rows),
    '<http://ex/a> <http://ex/p> "default" .',
  );
});

test("rowsToNQuads emits a quad line (with the graph) for named-graph rows", () => {
  const rows = [
    {
      s: { type: "uri", value: "http://ex/a" },
      p: { type: "uri", value: "http://ex/p" },
      o: { type: "literal", value: "in-g1" },
      g: { type: "uri", value: "http://ex/g1" },
    },
  ];
  assert.equal(
    rowsToNQuads(rows),
    '<http://ex/a> <http://ex/p> "in-g1" <http://ex/g1> .',
  );
});

test("rowsToNQuads round-trips a mixed dataset, preserving the named graph", () => {
  const rows = [
    {
      s: { type: "uri", value: "http://ex/d" },
      p: { type: "uri", value: "http://ex/p" },
      o: { type: "literal", value: "default" },
    },
    {
      s: { type: "uri", value: "http://ex/a" },
      p: { type: "uri", value: "http://ex/p" },
      o: { type: "literal", value: "in-g1" },
      g: { type: "uri", value: "http://ex/g1" },
    },
  ];
  const nq = rowsToNQuads(rows);
  // The default-graph triple stays a triple; the named-graph triple keeps <g1>.
  assert.equal(
    nq,
    '<http://ex/d> <http://ex/p> "default" .\n' +
      '<http://ex/a> <http://ex/p> "in-g1" <http://ex/g1> .',
  );
});

test("rowsToNQuads encodes term kinds (IRI / bnode / literal w/ datatype & lang)", () => {
  const rows = [
    {
      s: { type: "bnode", value: "b0" },
      p: { type: "uri", value: "http://ex/n" },
      o: { type: "literal", value: "42", datatype: "http://www.w3.org/2001/XMLSchema#integer" },
    },
    {
      s: { type: "uri", value: "http://ex/x" },
      p: { type: "uri", value: "http://ex/label" },
      o: { type: "literal", value: "hi", "xml:lang": "en" },
    },
  ];
  assert.equal(
    rowsToNQuads(rows),
    '_:b0 <http://ex/n> "42"^^<http://www.w3.org/2001/XMLSchema#integer> .\n' +
      '<http://ex/x> <http://ex/label> "hi"@en .',
  );
});

test("rowsToNQuads escapes backslash, quote, CR and LF in literals", () => {
  // Mirrors the engine's own N-Triples escaping (js/src/sparql.ts escapeLiteral):
  // backslash, double-quote, LF and CR. A raw tab is legal inside a quoted literal
  // and is left as-is, so the re-parse is byte-faithful.
  const rows = [
    {
      s: { type: "uri", value: "http://ex/x" },
      p: { type: "uri", value: "http://ex/p" },
      o: { type: "literal", value: 'a "q"\nlf\r cr \\ bs' },
    },
  ];
  assert.equal(
    rowsToNQuads(rows),
    '<http://ex/x> <http://ex/p> "a \\"q\\"\\nlf\\r cr \\\\ bs" .',
  );
});
