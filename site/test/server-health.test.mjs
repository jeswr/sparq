// [OPUS-4.8] sq-he72 — unit tests for the server health / capabilities client
// (`@sparq/client` `server-health.ts`): the endpoint-URL derivation, the Prometheus
// text-exposition parser (against the EXACT shape `crates/sparq-server/src/metrics.rs`
// emits), the VoID + Service-Description extractors (against the EXACT N-Triples
// `crates/sparq-introspect` `to_void` / `crates/sparq-server/src/descriptors.rs` `sd_ntriples`
// emit), and the `fetchServerHealth` outcome classification (404 → not-exposed, 401 →
// unauthorized, transport failure → error) over an injected `fetch`. These cover the pure,
// framework-free logic the server-health panel renders. Run via `npm run test:unit`.
//
// Imported by RELATIVE path (not the `@sparq/client` alias) so the build-free ts-loader
// resolves it without a custom alias resolver — matching `endpoint.test.mjs`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  deriveServerUrl,
  parsePrometheusMetrics,
  extractVoidSummary,
  extractServiceDescription,
  fetchServerHealth,
  shortenIri,
  formatMetricLabels,
  HEALTH_PATH,
  METRICS_PATH,
  VOID_PATH,
} from "../../packages/sparq-client/src/server-health.ts";
import { parseNTriples } from "../../packages/sparq-client/src/pretty-turtle.ts";

// --- deriveServerUrl --------------------------------------------------------

test("deriveServerUrl rebases the path onto the endpoint origin (not under /sparql)", () => {
  assert.equal(
    deriveServerUrl("http://127.0.0.1:3030/sparql", METRICS_PATH),
    "http://127.0.0.1:3030/metrics",
  );
  assert.equal(
    deriveServerUrl("http://127.0.0.1:3030/sparql", HEALTH_PATH),
    "http://127.0.0.1:3030/health",
  );
  assert.equal(
    deriveServerUrl("http://127.0.0.1:3030/sparql", VOID_PATH),
    "http://127.0.0.1:3030/.well-known/void",
  );
});

test("deriveServerUrl preserves a non-default port + drops any query string", () => {
  assert.equal(
    deriveServerUrl("https://data.example.org:8443/sparql?foo=bar", METRICS_PATH),
    "https://data.example.org:8443/metrics",
  );
});

test("deriveServerUrl returns null for an invalid endpoint URL", () => {
  assert.equal(deriveServerUrl("not a url", METRICS_PATH), null);
  assert.equal(deriveServerUrl("", METRICS_PATH), null);
  assert.equal(deriveServerUrl("ftp://host/sparql", METRICS_PATH), null);
});

// --- parsePrometheusMetrics -------------------------------------------------

// The EXACT exposition `metrics.rs::render` emits (a representative snapshot).
const METRICS_DOC = `# HELP sparq_http_requests_total Total HTTP requests by endpoint and response status.
# TYPE sparq_http_requests_total counter
sparq_http_requests_total{endpoint="/sparql",status="200"} 2
sparq_http_requests_total{endpoint="/sparql",status="400"} 1
sparq_http_requests_total{endpoint="/health",status="200"} 1
# HELP sparq_query_duration_seconds Wall time of /sparql requests (query + update operations).
# TYPE sparq_query_duration_seconds histogram
sparq_query_duration_seconds_bucket{le="0.001"} 1
sparq_query_duration_seconds_bucket{le="0.005"} 2
sparq_query_duration_seconds_bucket{le="0.1"} 3
sparq_query_duration_seconds_bucket{le="+Inf"} 3
sparq_query_duration_seconds_sum 0.062
sparq_query_duration_seconds_count 3
# HELP sparq_active_subscriptions Currently active WebSocket subscriptions.
# TYPE sparq_active_subscriptions gauge
sparq_active_subscriptions 3
# HELP sparq_graph_triples Triples in the published graph.
# TYPE sparq_graph_triples gauge
sparq_graph_triples 42
# HELP sparq_updates_total Successfully applied SPARQL updates.
# TYPE sparq_updates_total counter
sparq_updates_total 1
`;

test("parsePrometheusMetrics parses families, types, help and labelled samples", () => {
  const { families } = parsePrometheusMetrics(METRICS_DOC);
  const byName = new Map(families.map((f) => [f.name, f]));

  // The counter family with three labelled samples.
  const reqs = byName.get("sparq_http_requests_total");
  assert.ok(reqs, "request counter family present");
  assert.equal(reqs.type, "counter");
  assert.match(reqs.help, /Total HTTP requests/);
  assert.equal(reqs.samples.length, 3);
  const sparql200 = reqs.samples.find(
    (s) =>
      s.labels.some((l) => l.name === "endpoint" && l.value === "/sparql") &&
      s.labels.some((l) => l.name === "status" && l.value === "200"),
  );
  assert.ok(sparql200, "the /sparql 200 sample is parsed");
  assert.equal(sparql200.value, 2);

  // The two gauges.
  assert.equal(byName.get("sparq_active_subscriptions").type, "gauge");
  assert.equal(byName.get("sparq_active_subscriptions").samples[0].value, 3);
  assert.equal(byName.get("sparq_graph_triples").samples[0].value, 42);
  assert.equal(byName.get("sparq_updates_total").samples[0].value, 1);
});

test("parsePrometheusMetrics groups histogram _bucket/_sum/_count under the base family", () => {
  const { families } = parsePrometheusMetrics(METRICS_DOC);
  const hist = families.find((f) => f.name === "sparq_query_duration_seconds");
  assert.ok(hist, "the histogram family exists under its base name");
  assert.equal(hist.type, "histogram");
  // 4 buckets + _sum + _count = 6 samples, all under the one family (no stray families).
  assert.equal(hist.samples.length, 6);
  const inf = hist.samples.find((s) =>
    s.labels.some((l) => l.name === "le" && l.value === "+Inf"),
  );
  assert.ok(inf, "+Inf bucket present");
  assert.equal(inf.value, 3);
  const count = hist.samples.find((s) => s.name === "sparq_query_duration_seconds_count");
  assert.equal(count.value, 3);
  // No spurious top-level families for the suffixed sample names.
  assert.equal(
    families.filter((f) => f.name.endsWith("_bucket")).length,
    0,
    "no family was created for a _bucket sample name",
  );
});

test("parsePrometheusMetrics handles +Inf / -Inf / NaN values and empty input", () => {
  const { families } = parsePrometheusMetrics(
    "# TYPE g gauge\ng_inf +Inf\ng_ninf -Inf\ng_nan NaN\n",
  );
  const byName = new Map(families.map((f) => [f.name, f]));
  assert.equal(byName.get("g_inf").samples[0].value, Number.POSITIVE_INFINITY);
  assert.equal(byName.get("g_ninf").samples[0].value, Number.NEGATIVE_INFINITY);
  assert.ok(Number.isNaN(byName.get("g_nan").samples[0].value));
  assert.deepEqual(parsePrometheusMetrics("").families, []);
  assert.deepEqual(parsePrometheusMetrics("   \n\n").families, []);
});

test("parsePrometheusMetrics is total — junk lines are ignored, not thrown on", () => {
  assert.doesNotThrow(() =>
    parsePrometheusMetrics("this is not metrics\n{malformed\nfoo{bar} \n"),
  );
});

// --- extractVoidSummary -----------------------------------------------------

// The EXACT N-Triples `Introspection::to_void` emits (dataset summary + a class + a
// property partition). The dataset IRI matches the server's `{base}/.well-known/void#dataset`.
const VOID_DOC = `<http://127.0.0.1:3030/.well-known/void#dataset> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://rdfs.org/ns/void#Dataset> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#triples> "42"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#entities> "7"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#distinctSubjects> "9"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#classes> "2"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#properties> "5"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#classPartition> _:c0 .
_:c0 <http://rdfs.org/ns/void#class> <http://ex/Person> .
_:c0 <http://rdfs.org/ns/void#entities> "3"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://127.0.0.1:3030/.well-known/void#dataset> <http://rdfs.org/ns/void#propertyPartition> _:p1 .
_:p1 <http://rdfs.org/ns/void#property> <http://ex/age> .
_:p1 <http://rdfs.org/ns/void#triples> "3"^^<http://www.w3.org/2001/XMLSchema#integer> .
_:p1 <http://rdfs.org/ns/void#distinctSubjects> "3"^^<http://www.w3.org/2001/XMLSchema#integer> .
`;

test("extractVoidSummary reads the dataset-level VoID counts", () => {
  const { statements } = parseNTriples(VOID_DOC);
  const summary = extractVoidSummary(statements);
  assert.equal(summary.datasetIri, "http://127.0.0.1:3030/.well-known/void#dataset");
  assert.equal(summary.triples, 42);
  assert.equal(summary.entities, 7);
  assert.equal(summary.distinctSubjects, 9);
  assert.equal(summary.classes, 2);
  assert.equal(summary.properties, 5);
});

test("extractVoidSummary yields all-null when no void:Dataset subject is present", () => {
  const { statements } = parseNTriples(
    "<http://ex/s> <http://ex/p> <http://ex/o> .\n",
  );
  const summary = extractVoidSummary(statements);
  assert.equal(summary.datasetIri, null);
  assert.equal(summary.triples, null);
  assert.equal(summary.classes, null);
});

// --- extractServiceDescription ---------------------------------------------

// The EXACT N-Triples `descriptors.rs::sd_ntriples` emits (anonymous update allowed,
// federated query on, one extension function, one named graph with a triple count).
const SD_DOC = `<http://127.0.0.1:3030/sparql> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Service> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#endpoint> <http://127.0.0.1:3030/sparql> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#supportedLanguage> <http://www.w3.org/ns/sparql-service-description#SPARQL11Query> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#supportedLanguage> <http://www.w3.org/ns/sparql-service-description#SPARQL11Update> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#resultFormat> <http://www.w3.org/ns/formats/SPARQL_Results_JSON> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#inputFormat> <http://www.w3.org/ns/formats/Turtle> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#feature> <http://www.w3.org/ns/sparql-service-description#BasicFederatedQuery> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#extensionFunction> <http://www.opengis.net/def/function/geosparql/distance> .
<http://127.0.0.1:3030/sparql> <http://www.w3.org/ns/sparql-service-description#defaultDataset> _:dataset .
_:dataset <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Dataset> .
_:dataset <http://www.w3.org/ns/sparql-service-description#defaultGraph> _:defaultGraph .
_:defaultGraph <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Graph> .
_:defaultGraph <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://rdfs.org/ns/void#Dataset> .
<http://127.0.0.1:3030/sparql> <http://purl.org/dc/terms/source> <http://127.0.0.1:3030/.well-known/void#dataset> .
_:dataset <http://www.w3.org/ns/sparql-service-description#namedGraph> _:ng0 .
_:ng0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#NamedGraph> .
_:ng0 <http://www.w3.org/ns/sparql-service-description#name> <http://ex/graph1> .
_:ng0 <http://www.w3.org/ns/sparql-service-description#graph> _:ngG0 .
_:ngG0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sparql-service-description#Graph> .
_:ngG0 <http://rdfs.org/ns/void#triples> "11"^^<http://www.w3.org/2001/XMLSchema#integer> .
`;

test("extractServiceDescription reads the service capabilities + named graphs", () => {
  const { statements } = parseNTriples(SD_DOC);
  const sd = extractServiceDescription(statements);
  assert.equal(sd.serviceIri, "http://127.0.0.1:3030/sparql");
  assert.equal(sd.endpoint, "http://127.0.0.1:3030/sparql");
  assert.deepEqual(sd.supportedLanguages, [
    "http://www.w3.org/ns/sparql-service-description#SPARQL11Query",
    "http://www.w3.org/ns/sparql-service-description#SPARQL11Update",
  ]);
  assert.deepEqual(sd.features, [
    "http://www.w3.org/ns/sparql-service-description#BasicFederatedQuery",
  ]);
  assert.deepEqual(sd.resultFormats, [
    "http://www.w3.org/ns/formats/SPARQL_Results_JSON",
  ]);
  assert.deepEqual(sd.inputFormats, ["http://www.w3.org/ns/formats/Turtle"]);
  assert.deepEqual(sd.extensionFunctions, [
    "http://www.opengis.net/def/function/geosparql/distance",
  ]);
  // The named graph followed through the defaultDataset → namedGraph → name/graph chain.
  assert.equal(sd.namedGraphs.length, 1);
  assert.equal(sd.namedGraphs[0].name, "http://ex/graph1");
  assert.equal(sd.namedGraphs[0].triples, 11);
});

test("extractServiceDescription yields empty when no sd:Service subject is present", () => {
  const { statements } = parseNTriples(
    "<http://ex/s> <http://ex/p> <http://ex/o> .\n",
  );
  const sd = extractServiceDescription(statements);
  assert.equal(sd.serviceIri, null);
  assert.equal(sd.endpoint, null);
  assert.deepEqual(sd.supportedLanguages, []);
  assert.deepEqual(sd.namedGraphs, []);
});

// --- shortenIri / formatMetricLabels ---------------------------------------

test("shortenIri abbreviates the well-known sd: / void: namespaces", () => {
  assert.equal(
    shortenIri("http://www.w3.org/ns/sparql-service-description#SPARQL11Query"),
    "sd:SPARQL11Query",
  );
  assert.equal(shortenIri("http://rdfs.org/ns/void#triples"), "void:triples");
  assert.equal(shortenIri("http://ex/other"), "http://ex/other");
});

test("formatMetricLabels renders a compact label block (empty for no labels)", () => {
  assert.equal(formatMetricLabels([]), "");
  assert.equal(
    formatMetricLabels([
      { name: "endpoint", value: "/sparql" },
      { name: "status", value: "200" },
    ]),
    '{endpoint="/sparql",status="200"}',
  );
});

// --- fetchServerHealth (outcome classification over an injected fetch) ------

/** A fetch stub keyed by the requested URL's path; missing path → a transport rejection. */
function stubFetch(responsesByPath) {
  return async (url) => {
    const path = new URL(url).pathname;
    const r = responsesByPath[path];
    if (!r) throw new TypeError("Failed to fetch");
    return {
      ok: r.status >= 200 && r.status < 300,
      status: r.status,
      text: async () => r.body ?? "",
    };
  };
}

test("fetchServerHealth classifies ok / not-exposed / unauthorized / error per endpoint", async () => {
  const fetchImpl = stubFetch({
    "/health": { status: 200, body: "ok" },
    "/metrics": { status: 200, body: METRICS_DOC },
    // The opt-in descriptor endpoints are OFF → the server answers 404.
    "/.well-known/void": { status: 404, body: "not found" },
    // The Service Description is served on /sparql (no query); here it is gated by a read token.
    "/sparql": { status: 401, body: '{"error":"unauthorized"}' },
  });
  const health = await fetchServerHealth(
    { url: "http://127.0.0.1:3030/sparql", token: "" },
    fetchImpl,
  );
  assert.equal(health.health.status, "ok");
  assert.equal(health.health.data.body, "ok");
  assert.equal(health.metrics.status, "ok");
  assert.equal(health.metrics.data.families.length > 0, true);
  assert.equal(health.voidDescriptor.status, "not-exposed");
  assert.equal(health.serviceDescription.status, "unauthorized");
});

test("fetchServerHealth treats the SD's 400 'missing query' fall-through as not-exposed", async () => {
  // With the federation-descriptors feature OFF, GET /sparql (no query) returns the historical
  // 400, NOT a 404 — the SD read must still report that honestly as "not exposed".
  const fetchImpl = stubFetch({
    "/health": { status: 200, body: "ok" },
    "/metrics": { status: 200, body: METRICS_DOC },
    "/.well-known/void": { status: 404, body: "not found" },
    "/sparql": { status: 400, body: '{"error":"missing \'query\' parameter"}' },
  });
  const health = await fetchServerHealth(
    { url: "http://127.0.0.1:3030/sparql" },
    fetchImpl,
  );
  assert.equal(health.serviceDescription.status, "not-exposed");
  // The VoID 404 stays not-exposed; metrics (no 400 mapping) would still be an error on a 400,
  // but here it is ok.
  assert.equal(health.voidDescriptor.status, "not-exposed");
  assert.equal(health.metrics.status, "ok");
});

test("fetchServerHealth folds a transport failure into an error outcome (never throws)", async () => {
  // Every path rejects (server unreachable / CORS block).
  const fetchImpl = stubFetch({});
  const health = await fetchServerHealth(
    { url: "http://127.0.0.1:3030/sparql" },
    fetchImpl,
  );
  assert.equal(health.health.status, "error");
  assert.match(health.health.message, /Could not reach the server/);
  assert.equal(health.metrics.status, "error");
});

test("fetchServerHealth throws only for an invalid endpoint URL", async () => {
  await assert.rejects(
    () => fetchServerHealth({ url: "not a url" }, stubFetch({})),
    /valid absolute endpoint URL/,
  );
});

test("fetchServerHealth extracts the VoID + SD summaries when both are exposed", async () => {
  const fetchImpl = stubFetch({
    "/health": { status: 200, body: "ok" },
    "/metrics": { status: 200, body: METRICS_DOC },
    "/.well-known/void": { status: 200, body: VOID_DOC },
    "/sparql": { status: 200, body: SD_DOC },
  });
  const health = await fetchServerHealth(
    { url: "http://127.0.0.1:3030/sparql" },
    fetchImpl,
  );
  assert.equal(health.voidDescriptor.status, "ok");
  assert.equal(health.voidDescriptor.data.triples, 42);
  assert.equal(health.serviceDescription.status, "ok");
  assert.equal(health.serviceDescription.data.namedGraphs[0].name, "http://ex/graph1");
});
