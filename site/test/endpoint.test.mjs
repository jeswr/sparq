// [OPUS-4.8] sq-2mke — unit tests for the SPARQL 1.1 Protocol endpoint client
// (`@sparq/client` `endpoint.ts`): the honest connection-safety classifier, the query-form
// classifier, the request builder (auth header + content negotiation), the WS subprotocol
// derivation, and the `runEndpointQuery` response parsing over an injected `fetch`. These
// cover the pure, framework-free logic that the Connect panel and the REPL endpoint path
// rely on. Run via `npm run test:unit`.
//
// Imported by RELATIVE path (not the `@sparq/client` alias) so the build-free ts-loader
// resolves it without a custom alias resolver. In Node there is no `window`, so the
// browser-only findings (mixed-content / cors-required) do not fire — exactly the headless
// behaviour the classifier is designed for.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  connectionSafetyWarnings,
  hasBlockingWarning,
  isLoopbackHost,
  parseEndpointUrl,
  classifyEndpointForm,
  buildSparqlRequest,
  wsSubprotocols,
  runEndpointQuery,
  EndpointError,
} from "../../packages/sparq-client/src/endpoint.ts";

// --- isLoopbackHost ---------------------------------------------------------

test("isLoopbackHost recognises localhost, 127.0.0.0/8 and ::1", () => {
  for (const h of ["localhost", "LOCALHOST", "127.0.0.1", "127.0.0.99", "::1", "[::1]"]) {
    assert.equal(isLoopbackHost(h), true, `${h} should be loopback`);
  }
  for (const h of ["example.org", "10.0.0.1", "192.168.1.5", "0.0.0.0"]) {
    assert.equal(isLoopbackHost(h), false, `${h} should NOT be loopback`);
  }
});

// --- parseEndpointUrl -------------------------------------------------------

test("parseEndpointUrl accepts http/https absolute URLs and rejects junk", () => {
  assert.ok(parseEndpointUrl("http://127.0.0.1:3030/sparql"));
  assert.ok(parseEndpointUrl("https://example.org/sparql"));
  assert.equal(parseEndpointUrl(""), null);
  assert.equal(parseEndpointUrl("   "), null);
  assert.equal(parseEndpointUrl("not a url"), null);
  // A non-http(s) scheme is refused (no ftp:/file:/ws:).
  assert.equal(parseEndpointUrl("ws://127.0.0.1/subscriptions"), null);
  assert.equal(parseEndpointUrl("file:///etc/passwd"), null);
});

// --- connectionSafetyWarnings (the honest classifier) -----------------------

function codes(warnings) {
  return warnings.map((w) => w.code);
}

test("an invalid URL yields exactly one blocking error and nothing else", () => {
  const w = connectionSafetyWarnings({ url: "nope" });
  assert.deepEqual(codes(w), ["invalid-url"]);
  assert.equal(w[0].level, "error");
  assert.equal(hasBlockingWarning(w), true);
});

test("loopback http with no token: no transport warning, only the SERVICE info note", () => {
  const w = connectionSafetyWarnings({ url: "http://127.0.0.1:3030/sparql" });
  // No token, loopback => no token-over-plaintext, no non-loopback warning.
  assert.equal(codes(w).includes("token-over-plaintext"), false);
  assert.equal(codes(w).includes("non-loopback-no-tls"), false);
  // The SERVICE-allowlist reminder is always surfaced (info).
  assert.equal(codes(w).includes("service-allowlist"), true);
  assert.equal(hasBlockingWarning(w), false);
});

test("loopback http WITH a token: token stays on-machine (info, not a warning)", () => {
  const w = connectionSafetyWarnings({
    url: "http://localhost:3030/sparql",
    token: "secret",
  });
  const tok = w.find((x) => x.code === "token-over-plaintext");
  assert.ok(tok, "expected a token-over-plaintext finding");
  assert.equal(tok.level, "info", "loopback token is an info note, not a transit warning");
});

test("non-loopback http WITH a token: token-over-plaintext + non-loopback are WARNINGS", () => {
  const w = connectionSafetyWarnings({
    url: "http://data.example.org/sparql",
    token: "secret",
  });
  const tok = w.find((x) => x.code === "token-over-plaintext");
  const non = w.find((x) => x.code === "non-loopback-no-tls");
  assert.ok(tok && tok.level === "warning", "token over plaintext to a remote host is a warning");
  assert.ok(non && non.level === "warning", "plaintext to a non-loopback host is a warning");
  // Honest, not a hard block — the user can proceed (the browser/server decides).
  assert.equal(hasBlockingWarning(w), false);
});

test("non-loopback http with NO token still warns about cleartext transport", () => {
  const w = connectionSafetyWarnings({ url: "http://data.example.org/sparql" });
  assert.equal(codes(w).includes("non-loopback-no-tls"), true);
  // No token => no token-specific finding.
  assert.equal(codes(w).includes("token-over-plaintext"), false);
});

test("https to a remote host with a token: no transport warnings (TLS protects it)", () => {
  const w = connectionSafetyWarnings({
    url: "https://data.example.org/sparql",
    token: "secret",
  });
  assert.equal(codes(w).includes("token-over-plaintext"), false);
  assert.equal(codes(w).includes("non-loopback-no-tls"), false);
  assert.equal(hasBlockingWarning(w), false);
  // The SERVICE reminder is still surfaced.
  assert.equal(codes(w).includes("service-allowlist"), true);
});

// --- classifyEndpointForm ---------------------------------------------------

test("classifyEndpointForm folds to the four wire shapes, skipping the prologue", () => {
  assert.equal(classifyEndpointForm("SELECT * WHERE { ?s ?p ?o }"), "select");
  assert.equal(classifyEndpointForm("  ASK { ?s ?p ?o }"), "ask");
  assert.equal(classifyEndpointForm("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }"), "graph");
  assert.equal(classifyEndpointForm("DESCRIBE <http://example.org/x>"), "graph");
  assert.equal(classifyEndpointForm("INSERT DATA { <a> <b> <c> }"), "update");
  assert.equal(classifyEndpointForm("DELETE WHERE { ?s ?p ?o }"), "update");
  assert.equal(classifyEndpointForm("DROP GRAPH <http://example.org/g>"), "update");
  // A PREFIX/BASE prologue is skipped so the operative keyword is found.
  assert.equal(
    classifyEndpointForm("PREFIX ex: <http://example.org/>\nSELECT * WHERE { ?s ?p ?o }"),
    "select",
  );
  assert.equal(
    classifyEndpointForm("# a comment\nPREFIX ex: <http://example.org/>\nINSERT DATA { ex:a ex:b ex:c }"),
    "update",
  );
});

// --- buildSparqlRequest (auth + content negotiation) ------------------------

test("buildSparqlRequest uses direct sparql-query POST with the right Accept per form", () => {
  const cfg = { url: "http://127.0.0.1:3030/sparql" };
  const sel = buildSparqlRequest(cfg, "SELECT * WHERE {?s ?p ?o}", "select");
  assert.equal(sel.init.method, "POST");
  assert.equal(sel.init.headers["Content-Type"], "application/sparql-query");
  assert.equal(sel.init.headers["Accept"], "application/sparql-results+json");
  assert.equal(sel.init.body, "SELECT * WHERE {?s ?p ?o}");

  const graph = buildSparqlRequest(cfg, "CONSTRUCT {?s ?p ?o} WHERE {?s ?p ?o}", "graph");
  assert.equal(graph.init.headers["Accept"], "application/n-triples");

  const upd = buildSparqlRequest(cfg, "INSERT DATA { <a> <b> <c> }", "update");
  assert.equal(upd.init.headers["Content-Type"], "application/sparql-update");
  // An update body asks for no SPARQL-results Accept (the server acks a 204).
  assert.equal(upd.init.headers["Accept"], undefined);
});

test("buildSparqlRequest adds the bearer header iff a non-empty token is set", () => {
  const sel = buildSparqlRequest(
    { url: "http://127.0.0.1:3030/sparql", token: "  tok  " },
    "SELECT * WHERE {?s ?p ?o}",
    "select",
  );
  // The token is trimmed and sent only in the Authorization header.
  assert.equal(sel.init.headers["Authorization"], "Bearer tok");

  const noTok = buildSparqlRequest(
    { url: "http://127.0.0.1:3030/sparql", token: "   " },
    "SELECT * WHERE {?s ?p ?o}",
    "select",
  );
  assert.equal(noTok.init.headers["Authorization"], undefined);
});

// --- wsSubprotocols ---------------------------------------------------------

test("wsSubprotocols derives the bearer.<token> subprotocol (browser WS auth)", () => {
  assert.deepEqual(wsSubprotocols("tok"), ["bearer.tok"]);
  assert.deepEqual(wsSubprotocols(""), []);
  assert.deepEqual(wsSubprotocols("  "), []);
  assert.deepEqual(wsSubprotocols(undefined), []);
});

// --- runEndpointQuery (over an injected fetch) ------------------------------

function jsonResponse(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/sparql-results+json" },
  });
}

test("runEndpointQuery parses a SELECT results document", async () => {
  const fetchImpl = async () =>
    jsonResponse({
      head: { vars: ["s"] },
      results: { bindings: [{ s: { type: "uri", value: "http://x" } }] },
    });
  const r = await runEndpointQuery(
    { url: "http://127.0.0.1:3030/sparql" },
    "SELECT * WHERE {?s ?p ?o}",
    fetchImpl,
  );
  assert.equal(r.kind, "select");
  assert.equal(r.results.results.bindings.length, 1);
});

test("runEndpointQuery surfaces an ASK boolean", async () => {
  const fetchImpl = async () => jsonResponse({ head: {}, boolean: true });
  const r = await runEndpointQuery(
    { url: "http://127.0.0.1:3030/sparql" },
    "ASK { ?s ?p ?o }",
    fetchImpl,
  );
  assert.equal(r.kind, "boolean");
  assert.equal(r.value, true);
});

test("runEndpointQuery returns the N-Triples body for CONSTRUCT", async () => {
  const fetchImpl = async () =>
    new Response("<http://a> <http://b> <http://c> .\n", {
      status: 200,
      headers: { "Content-Type": "application/n-triples" },
    });
  const r = await runEndpointQuery(
    { url: "http://127.0.0.1:3030/sparql" },
    "CONSTRUCT {?s ?p ?o} WHERE {?s ?p ?o}",
    fetchImpl,
  );
  assert.equal(r.kind, "graph");
  assert.match(r.ntriples, /<http:\/\/a>/);
});

test("runEndpointQuery returns an update ack on a 204", async () => {
  const fetchImpl = async () => new Response(null, { status: 204 });
  const r = await runEndpointQuery(
    { url: "http://127.0.0.1:3030/sparql" },
    "INSERT DATA { <a> <b> <c> }",
    fetchImpl,
  );
  assert.equal(r.kind, "update");
  assert.equal(r.status, 204);
});

test("runEndpointQuery throws a classified EndpointError on a 401", async () => {
  const fetchImpl = async () =>
    new Response(JSON.stringify({ error: "authentication required" }), {
      status: 401,
      headers: { "Content-Type": "application/json" },
    });
  await assert.rejects(
    () => runEndpointQuery({ url: "http://127.0.0.1:3030/sparql" }, "ASK {}", fetchImpl),
    (e) => {
      assert.ok(e instanceof EndpointError);
      assert.equal(e.status, 401);
      assert.match(e.message, /Authentication required/);
      return true;
    },
  );
});

test("runEndpointQuery turns a transport failure into an honest CORS/reachability hint", async () => {
  const fetchImpl = async () => {
    throw new TypeError("Failed to fetch");
  };
  await assert.rejects(
    () => runEndpointQuery({ url: "http://data.example.org/sparql" }, "ASK {}", fetchImpl),
    (e) => {
      assert.ok(e instanceof EndpointError);
      assert.equal(e.status, null);
      assert.match(e.message, /CORS|reach the endpoint/);
      return true;
    },
  );
});
