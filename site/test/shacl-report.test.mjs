// [OPUS-4.8] sq-egy6 — unit tests for the pure SHACL-report rendering helpers that
// back the live /surface/shacl playground. These cover the framework-free CURIE
// shortening, component/severity local names, conformance summary, and the W3C
// Turtle serialisation. The wasm `Store.validate` binding itself is proven by the
// Rust tests in crates/sparq-wasm/src/shacl.rs; here we only test the JS rendering
// of the JSON report it returns. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  shortenIri,
  componentName,
  severityName,
  reportSummary,
  reportToTurtle,
} from "../src/lib/shacl-report.ts";

// The `ex:age "thirty"` datatype violation, as the wasm validate binding returns it.
const DATATYPE_VIOLATION = {
  conforms: false,
  results: [
    {
      focusNode: "<http://example.org/alice>",
      path: "<http://example.org/age>",
      value: '"thirty"',
      sourceShape: "_:b0",
      sourceConstraintComponent:
        "http://www.w3.org/ns/shacl#DatatypeConstraintComponent",
      severity: "http://www.w3.org/ns/shacl#Violation",
      message: "ex:age must be exactly one xsd:integer",
    },
  ],
};

const CONFORMS = { conforms: true, results: [] };

test("shortenIri compacts the well-known namespaces and leaves the rest alone", () => {
  assert.equal(shortenIri("<http://example.org/alice>"), "ex:alice");
  assert.equal(shortenIri("<http://example.org/age>"), "ex:age");
  assert.equal(
    shortenIri("<http://www.w3.org/2001/XMLSchema#integer>"),
    "xsd:integer",
  );
  assert.equal(
    shortenIri("<http://www.w3.org/ns/shacl#Violation>"),
    "sh:Violation",
  );
  // Literal term strings and blank nodes pass through untouched.
  assert.equal(shortenIri('"thirty"'), '"thirty"');
  assert.equal(shortenIri("_:b0"), "_:b0");
  // An IRI outside the known prefixes is returned unchanged.
  assert.equal(shortenIri("<http://other.test/x>"), "<http://other.test/x>");
});

test("componentName / severityName take the local part after the hash", () => {
  assert.equal(
    componentName("http://www.w3.org/ns/shacl#DatatypeConstraintComponent"),
    "DatatypeConstraintComponent",
  );
  assert.equal(severityName("http://www.w3.org/ns/shacl#Violation"), "Violation");
});

test("reportSummary reflects conformance and violation count", () => {
  assert.equal(reportSummary(CONFORMS), "Conforms — no violations.");
  assert.equal(
    reportSummary(DATATYPE_VIOLATION),
    "Does not conform — 1 violation.",
  );
  assert.equal(
    reportSummary({ conforms: false, results: [{}, {}] }),
    "Does not conform — 2 violations.",
  );
});

test("reportToTurtle emits a conforming sh:ValidationReport with no results", () => {
  const ttl = reportToTurtle(CONFORMS);
  assert.match(ttl, /a sh:ValidationReport/);
  assert.match(ttl, /sh:conforms true \./);
  assert.ok(!ttl.includes("sh:result"), "no sh:result for a conforming report");
});

test("reportToTurtle emits the per-violation W3C vocabulary", () => {
  const ttl = reportToTurtle(DATATYPE_VIOLATION);
  assert.match(ttl, /sh:conforms false ;/);
  assert.match(ttl, /sh:result \[/);
  assert.match(ttl, /a sh:ValidationResult ;/);
  assert.match(ttl, /sh:focusNode <http:\/\/example\.org\/alice> ;/);
  assert.match(ttl, /sh:resultPath <http:\/\/example\.org\/age> ;/);
  assert.match(ttl, /sh:value "thirty" ;/);
  assert.match(
    ttl,
    /sh:sourceConstraintComponent <http:\/\/www\.w3\.org\/ns\/shacl#DatatypeConstraintComponent> ;/,
  );
  assert.match(
    ttl,
    /sh:resultMessage "ex:age must be exactly one xsd:integer"/,
  );
  // The blank-node and the report both terminate correctly.
  assert.match(ttl, /\] \./);
});
