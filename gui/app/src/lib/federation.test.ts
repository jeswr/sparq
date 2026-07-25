// [FABLE-5] sq-ixc3.14 — unit tests for federation.ts pure helpers.
//
// Covers: queryUsesService (the SERVICE routing detector), describeServiceRefusal (the
// fail-closed refusal classifier), parseServiceResults (the native-result parse + row cap).
// The IPC round-trip itself is exercised by the Playwright mocked-IPC spec
// (e2e-playwright/specs/federation.spec.ts) and, against the REAL engine + a live loopback
// endpoint, by gui/src-tauri/src/federation.rs's native-lane tests.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  queryUsesService,
  describeServiceRefusal,
  parseServiceResults,
  SERVICE_EGRESS_REFUSED_MARKER,
} from "./federation.js";

// ---------------------------------------------------------------------------
// queryUsesService
// ---------------------------------------------------------------------------

test("queryUsesService – detects a plain SERVICE clause (any case)", () => {
  assert.equal(
    queryUsesService(
      "SELECT * WHERE { ?s ?p ?o . SERVICE <http://example.org/sparql> { ?s ?p2 ?o2 } }",
    ),
    true,
  );
  assert.equal(queryUsesService("select * where { service <http://e/> { ?s ?p ?o } }"), true);
  assert.equal(queryUsesService("SELECT * WHERE { SERVICE SILENT ?ep { ?s ?p ?o } }"), true);
});

test("queryUsesService – a plain local query does not route", () => {
  assert.equal(queryUsesService("SELECT * WHERE { ?s ?p ?o }"), false);
});

test("queryUsesService – SERVICE inside a string literal / comment / IRI is NOT a clause", () => {
  assert.equal(
    queryUsesService('SELECT * WHERE { ?s ?p "the SERVICE desk" }'),
    false,
    "double-quoted literal",
  );
  assert.equal(
    queryUsesService("SELECT * WHERE { ?s ?p '''SERVICE\nmulti''' }"),
    false,
    "long literal",
  );
  assert.equal(
    queryUsesService("# SERVICE is only mentioned here\nSELECT * WHERE { ?s ?p ?o }"),
    false,
    "comment",
  );
  assert.equal(
    queryUsesService("SELECT * WHERE { ?s <http://example.org/service> ?o }"),
    false,
    "IRI containing /service",
  );
});

test("queryUsesService – a FILTER comparison before a real SERVICE does not swallow it", () => {
  // The `<` of `?x < 5` must NOT be treated as an IRI opener that eats the SERVICE keyword.
  assert.equal(
    queryUsesService(
      "SELECT * WHERE { ?s ?p ?x . FILTER(?x < 5) SERVICE <http://e/sparql> { ?s ?q ?y } }",
    ),
    true,
  );
});

test("queryUsesService – an escaped quote inside a literal does not desync the scanner", () => {
  assert.equal(queryUsesService('SELECT * WHERE { ?s ?p "a \\" SERVICE b" }'), false);
});

// ---------------------------------------------------------------------------
// describeServiceRefusal
// ---------------------------------------------------------------------------

test("describeServiceRefusal – classifies the engine's stable egress marker", () => {
  const msg = describeServiceRefusal(
    `${SERVICE_EGRESS_REFUSED_MARKER}: host "example.org" is not allowlisted`,
  );
  assert.ok(msg, "a marker-bearing error must classify as a refusal");
  assert.match(msg, /fail-closed/i);
  assert.match(msg, /allowlist/i);
  // The engine's own message is preserved for honesty (no detail is swallowed).
  assert.ok(msg.includes(SERVICE_EGRESS_REFUSED_MARKER));
});

test("describeServiceRefusal – any other error passes through as null", () => {
  assert.equal(describeServiceRefusal("parse error: unexpected token"), null);
});

// ---------------------------------------------------------------------------
// parseServiceResults
// ---------------------------------------------------------------------------

const SELECT_JSON = JSON.stringify({
  head: { vars: ["name", "age"] },
  results: {
    bindings: [
      { name: { type: "literal", value: "Alice" }, age: { type: "literal", value: "30" } },
      { name: { type: "literal", value: "Bob" }, age: { type: "literal", value: "31" } },
    ],
  },
});

test("parseServiceResults – SELECT rows parse into the table outcome", () => {
  const out = parseServiceResults(SELECT_JSON, 100);
  assert.equal(out.kind, "select");
  if (out.kind !== "select") return;
  assert.equal(out.rowCount, 2);
  assert.equal(out.totalRows, 2);
  assert.equal(out.truncated, false);
  assert.deepEqual(out.results.head.vars, ["name", "age"]);
  assert.equal(out.results.results.bindings[0]?.name?.value, "Alice");
});

test("parseServiceResults – rows beyond the cap are counted but dropped (truncated)", () => {
  const out = parseServiceResults(SELECT_JSON, 1);
  assert.equal(out.kind, "select");
  if (out.kind !== "select") return;
  assert.equal(out.rowCount, 1);
  assert.equal(out.totalRows, 2);
  assert.equal(out.truncated, true);
});

test("parseServiceResults – an ASK document parses into the boolean outcome", () => {
  const out = parseServiceResults('{"head":{},"boolean":true}', 100);
  assert.equal(out.kind, "ask");
  if (out.kind !== "ask") return;
  assert.equal(out.value, true);
});

test("parseServiceResults – garbage is an error outcome, never a crash", () => {
  const out = parseServiceResults("not json at all", 100);
  assert.equal(out.kind, "error");
});
