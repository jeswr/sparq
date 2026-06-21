// [OPUS-4.8] sq-3p0z (#822) — unit tests for the pure helpers behind the Verifiable-
// Credentials import-and-query surface. These cover the framework-free decision logic
// (format-from-filename + JWT-envelope recognition) and the integrity of the seeded
// fixtures; the engine-level JSON-LD/Turtle parsing + SPARQL is already proven by the
// Rust tests and js/test/store.test.mjs. Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  SAMPLE_CREDENTIALS,
  SAMPLE_QUERIES,
  VC_FORMAT_OPTIONS,
  vcFormatFromName,
  looksLikeJwtVc,
} from "../src/data/verifiable-credentials.ts";

test("vcFormatFromName maps filenames to the engine format, defaulting to JSON-LD", () => {
  // JSON-LD is the canonical VC serialisation, so it is the default for unknown / .json.
  assert.equal(vcFormatFromName("credential.jsonld"), "jsonld");
  assert.equal(vcFormatFromName("credential.json"), "jsonld");
  assert.equal(vcFormatFromName("credential.json-ld"), "jsonld");
  assert.equal(vcFormatFromName("licence.ttl"), "turtle");
  assert.equal(vcFormatFromName("data.nq"), "nquads");
  assert.equal(vcFormatFromName("data.trig"), "trig");
  assert.equal(vcFormatFromName("data.nt"), "ntriples");
  // A query string / fragment must not defeat the extension sniff.
  assert.equal(vcFormatFromName("https://ex.org/c.ttl?v=2#frag"), "turtle");
  // Unknown extension → the JSON-LD default (a VC is most often JSON-LD).
  assert.equal(vcFormatFromName("credential"), "jsonld");
  assert.equal(vcFormatFromName("credential.bin"), "jsonld");
});

test("looksLikeJwtVc recognises JWT / SD-JWT-VC envelopes but not RDF/JSON-LD", () => {
  // A bare three-segment JWT.
  assert.equal(looksLikeJwtVc("eyJhbGciOiJFUzI1NiJ9.eyJzdWIiOiIxMjMifQ.c2lnbmF0dXJl"), true);
  // An SD-JWT-VC: a JWT followed by `~`-separated disclosures (+ optional trailing `~`).
  assert.equal(
    looksLikeJwtVc("eyJhbGciOiJFUzI1NiJ9.eyJ2YyI6e319.sig~WyJzYWx0IiwibmFtZSIsIkFkYSJd~"),
    true,
  );
  // JSON-LD and Turtle are NOT JWT-shaped.
  assert.equal(looksLikeJwtVc('{ "@context": "x", "type": "VerifiableCredential" }'), false);
  assert.equal(looksLikeJwtVc("@prefix ex: <http://ex/> . ex:a ex:b ex:c ."), false);
  assert.equal(looksLikeJwtVc(""), false);
  // A two-segment token (not a full JWT) is not recognised as one.
  assert.equal(looksLikeJwtVc("eyJhbGciOiJFUzI1NiJ9.eyJzdWIiOiIxMjMifQ"), false);
});

test("the seeded fixtures are internally consistent", () => {
  // Two seeded sample credentials, each in a format the engine can load.
  assert.equal(SAMPLE_CREDENTIALS.length, 2);
  const formats = new Set(VC_FORMAT_OPTIONS.map((f) => f.value));
  for (const c of SAMPLE_CREDENTIALS) {
    assert.ok(c.source.trim().length > 0, `${c.id} has a non-empty source`);
    assert.ok(formats.has(c.format), `${c.id} format ${c.format} is a known engine format`);
  }
  // The JSON-LD sample must inline its @context (the lean in-tab bundle fetches no remote
  // context) — i.e. it must NOT reference the W3C VC context only by URL.
  const jsonld = SAMPLE_CREDENTIALS.find((c) => c.format === "jsonld");
  assert.ok(jsonld, "a JSON-LD sample exists");
  assert.ok(
    !/"@context"\s*:\s*\[?\s*"https?:\/\//.test(jsonld.source),
    "the JSON-LD sample does not start its @context with a remote URL",
  );

  // Every sample query carries non-empty SPARQL.
  assert.ok(SAMPLE_QUERIES.length >= 1);
  for (const q of SAMPLE_QUERIES) {
    assert.ok(q.sparql.trim().length > 0, `${q.id} has non-empty SPARQL`);
  }
});
