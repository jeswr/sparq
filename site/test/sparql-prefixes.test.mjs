// [OPUS-4.8] sq-n5aw — unit tests for the framework-agnostic common-prefix helpers
// (packages/sparq-client/src/sparql-prefixes.ts), the dependency-free "prefix awareness" core
// of the query-editor uplift. Covers: reading declared/used prefixes, computing the well-known
// prefixes a query uses but omits, and the pure prepend that adds only the missing ones. Run
// via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  COMMON_PREFIXES,
  declaredPrefixes,
  usedPrefixes,
  missingCommonPrefixes,
  renderPrefixLines,
  withPrefixes,
} from "../../packages/sparq-client/src/sparql-prefixes.ts";

test("the registry has the core RDF families with absolute IRIs", () => {
  const byPrefix = Object.fromEntries(COMMON_PREFIXES.map((b) => [b.prefix, b.iri]));
  for (const p of ["rdf", "rdfs", "owl", "xsd", "foaf", "dc", "ex"]) {
    assert.ok(byPrefix[p], `expected a common prefix '${p}'`);
    assert.match(byPrefix[p], /^https?:\/\//, `'${p}' IRI should be absolute`);
  }
});

test("declaredPrefixes reads PREFIX declarations (case-insensitive, empty prefix)", () => {
  const q = `PREFIX foaf: <http://xmlns.com/foaf/0.1/>\nprefix ex: <http://example.org/>\nPREFIX : <http://base/>\nSELECT * {}`;
  const declared = declaredPrefixes(q);
  assert.ok(declared.has("foaf"));
  assert.ok(declared.has("ex"));
  assert.ok(declared.has("")); // the empty prefix `:`
  assert.equal(declared.has("rdf"), false);
});

test("usedPrefixes finds prefixed names but not IRIs or variables", () => {
  const q = "SELECT ?s { ?s foaf:name ?n ; rdf:type ex:Thing . <http://x/> a owl:Class }";
  const used = usedPrefixes(q);
  assert.ok(used.has("foaf"));
  assert.ok(used.has("rdf"));
  assert.ok(used.has("ex"));
  assert.ok(used.has("owl"));
  // The bare IRI <http://x/> must not register as a prefix, and ?s/?n are variables.
  assert.equal(used.has("http"), false);
});

test("missingCommonPrefixes = used well-known prefixes that are not declared", () => {
  const q = "SELECT ?n WHERE { ?s foaf:name ?n ; rdf:type ?t }";
  const missing = missingCommonPrefixes(q).map((b) => b.prefix);
  assert.deepEqual(new Set(missing), new Set(["rdf", "foaf"]));
  // Once declared, foaf drops out of the missing set.
  const q2 = "PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n" + q;
  assert.deepEqual(
    missingCommonPrefixes(q2).map((b) => b.prefix),
    ["rdf"],
  );
  // A used prefix NOT in the registry is not suggested (no IRI to offer).
  const q3 = "SELECT * { ?s wibble:thing ?o }";
  assert.deepEqual(missingCommonPrefixes(q3), []);
});

test("renderPrefixLines emits one PREFIX line per binding", () => {
  const lines = renderPrefixLines([
    { prefix: "foaf", iri: "http://xmlns.com/foaf/0.1/" },
    { prefix: "ex", iri: "http://example.org/" },
  ]);
  assert.equal(
    lines,
    "PREFIX foaf: <http://xmlns.com/foaf/0.1/>\nPREFIX ex: <http://example.org/>",
  );
});

test("withPrefixes prepends only the missing prefixes and is a no-op when none are new", () => {
  const q = "SELECT ?n WHERE { ?s foaf:name ?n }";
  const out = withPrefixes(q, missingCommonPrefixes(q));
  assert.match(out, /^PREFIX foaf: <http:\/\/xmlns\.com\/foaf\/0\.1\/>\n\nSELECT/);
  // Idempotent: a second pass adds nothing.
  assert.equal(withPrefixes(out, missingCommonPrefixes(out)), out);
  // Already-declared prefixes are skipped.
  assert.equal(
    withPrefixes(out, [{ prefix: "foaf", iri: "http://xmlns.com/foaf/0.1/" }]),
    out,
  );
});
