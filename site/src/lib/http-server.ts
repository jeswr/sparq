// [OPUS-4.8] sq-rnwc — pure, framework-free data + helpers for the
// /surface/http-server walkthrough (tier-e). The hosted sparq-server is NOT in
// the wasm bundle and the static GitHub-Pages site has no backend to talk to, so
// this surface is the honest "captured curl / WS-frame walkthrough" fallback the
// feature-showcase design names for tier-e (research/feature-showcase-site-design.md
// §0, surface (e)).
//
// Everything in this module is captured I/O: each curl recipe was run against a
// real `sparq-server --format turtle` (the DEFAULT build — no opt-in cargo features)
// over the tiny `ex:` seed below, and the responses pasted here verbatim. The
// live-subscription transcript is a byte-for-byte recording of an SSE
// `/subscriptions/sse` stream firing: a sequence-0 full snapshot, then a sequence-1
// incremental `addedResults` diff when a SPARQL UPDATE committed — exactly the
// "push an Update -> watch the subscription fire" demo. Keeping it here (no React,
// no network) lets the page replay it deterministically and lets `node --test`
// assert the framing — including the SPARQL-JSON term serialization — without a
// server.
//
// HONESTY NOTE [OPUS-4.8] sq-rnwc: the RESULT payloads (SELECT/ASK/CSV JSON+CSV,
// the CONSTRUCT Turtle, every SSE `data:` frame) are deterministic engine output —
// run the same default binary over the same seed and you get the same bytes. A few
// fields are inherently RUN-DEPENDENT and so are NOT byte-identical across runs: the
// `date:` response header, and the `/metrics` request-counters / histogram timings.
// Those are shown as one representative capture and labelled as such; we do NOT claim
// them byte-reproducible. The `Sparq-Generation` response header only exists under the
// opt-in `time-travel` cargo feature, so the default-build UPDATE head below carries
// no such line. The Turtle writer registers a fixed common-prefix set (rdf/rdfs/xsd/
// owl/foaf/dc/dcterms/skos/schema) but NOT `ex:`, so `ex:` IRIs render in full — the
// CONSTRUCT output reflects exactly that.
//
// Grounded in skills/http-server/SKILL.md (the canonical endpoint contract) and the
// server's own serialisers (crates/sparq-server/src/graph.rs::triples_to_turtle,
// crates/sparq-server/src/subscriptions.rs::term_json).

/** The seed dataset the captured responses were recorded against (Turtle). */
export const SEED_TURTLE = `@prefix ex: <http://ex/> .
ex:alice ex:age 30 ; ex:knows ex:bob .
ex:bob   ex:age 25 .
ex:carol ex:age 41 ; ex:knows ex:alice .`;

/** Default loopback endpoint the recipes target. */
export const ENDPOINT = "http://127.0.0.1:3030";

/** One captured request/response recipe in the REST walkthrough. */
export interface Recipe {
  /** Stable id (also the anchor / test key). */
  id: string;
  /** Short human title. */
  title: string;
  /** One-line description of what it exercises. */
  blurb: string;
  /** The exact curl invocation, multi-line as a user would type it. */
  curl: string;
  /** The verbatim captured response body (or response head for the 204). */
  response: string;
  /** Response language hint for display. */
  lang: "json" | "turtle" | "csv" | "text" | "http";
}

// Captured from a running `sparq-server --format turtle data.ttl` (default build) over
// SEED_TURTLE. The UPDATE then CONSTRUCT/metrics recipes reflect the same server after the
// seed plus the walkthrough's two demonstrated writes (dave, then frank), so the live
// triple count (7) and update counter (2) are internally consistent with the transcript.
export const RECIPES: Recipe[] = [
  {
    id: "select",
    title: "SELECT — content-negotiated SPARQL-JSON",
    blurb:
      "A plain SPARQL 1.1 Protocol query over GET; the default result media is SPARQL-results JSON.",
    curl: `curl -G ${ENDPOINT}/sparql \\
  --data-urlencode 'query=SELECT ?s ?age WHERE { ?s <http://ex/age> ?age } ORDER BY ?age'`,
    response: `{"head":{"vars":["s","age"]},"results":{"bindings":[
  {"s":{"type":"uri","value":"http://ex/bob"},
   "age":{"type":"literal","value":"25","datatype":"http://www.w3.org/2001/XMLSchema#integer"}},
  {"s":{"type":"uri","value":"http://ex/alice"},
   "age":{"type":"literal","value":"30","datatype":"http://www.w3.org/2001/XMLSchema#integer"}},
  {"s":{"type":"uri","value":"http://ex/carol"},
   "age":{"type":"literal","value":"41","datatype":"http://www.w3.org/2001/XMLSchema#integer"}}
]}}`,
    lang: "json",
  },
  {
    id: "ask",
    title: "ASK — POST the query as the body",
    blurb:
      "POST direct: the request body IS the query (Content-Type: application/sparql-query).",
    curl: `curl ${ENDPOINT}/sparql \\
  -H 'Content-Type: application/sparql-query' \\
  --data 'ASK { ?s <http://ex/knows> ?o }'`,
    response: `{"head":{},"boolean":true}`,
    lang: "json",
  },
  {
    id: "csv",
    title: "SELECT — Accept: text/csv",
    blurb:
      "Set Accept to choose the result media (q-value aware): JSON / XML / CSV / TSV for SELECT.",
    curl: `curl ${ENDPOINT}/sparql -H 'Accept: text/csv' \\
  --data-urlencode 'query=SELECT ?s ?age WHERE { ?s <http://ex/age> ?age } ORDER BY ?age'`,
    response: `s,age
http://ex/bob,25
http://ex/alice,30
http://ex/carol,41`,
    lang: "csv",
  },
  {
    id: "construct",
    title: "CONSTRUCT — Accept: text/turtle",
    blurb:
      "CONSTRUCT / DESCRIBE negotiate an RDF syntax (N-Triples default; prefix-compacting Turtle or RDF/XML). The Turtle writer registers a fixed common-prefix set, but not ex:, so ex: IRIs stay in full.",
    curl: `curl -G ${ENDPOINT}/sparql -H 'Accept: text/turtle' \\
  --data-urlencode 'query=CONSTRUCT { ?s ?p ?o } WHERE { ?s <http://ex/knows> ?o . ?s ?p ?o }'`,
    response: `@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix skos: <http://www.w3.org/2004/02/skos/core#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix dc: <http://purl.org/dc/elements/1.1/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix schema: <https://schema.org/> .
<http://ex/alice> <http://ex/knows> <http://ex/bob> .
<http://ex/carol> <http://ex/knows> <http://ex/alice> .`,
    lang: "turtle",
  },
  {
    id: "update",
    title: "UPDATE — atomic, 204 No Content",
    blurb:
      "A SPARQL Update commits atomically (failure → 400, no partial effect) and returns 204; every response carries the hardening header set. (The Sparq-Generation header is emitted only under the opt-in time-travel feature, so the default build below has none.)",
    curl: `curl -i ${ENDPOINT}/sparql \\
  -H 'Content-Type: application/sparql-update' \\
  --data 'INSERT DATA { <http://ex/dave> <http://ex/age> 52 }'`,
    response: `HTTP/1.1 204 No Content
x-content-type-options: nosniff
content-security-policy: default-src 'none'; frame-ancestors 'none'
x-frame-options: DENY
referrer-policy: no-referrer`,
    lang: "http",
  },
  {
    id: "gsp-put",
    title: "Graph Store write — PUT replaces a named graph",
    blurb:
      "The Graph Store HTTP Protocol: PUT replaces (201 created / 204 replaced), POST merges, DELETE drops. Body is RDF, format by Content-Type.",
    curl: `curl -X PUT '${ENDPOINT}/sparql/graph?graph=http://ex/g' \\
  -H 'Content-Type: text/turtle' \\
  --data '<http://ex/s> <http://ex/p> <http://ex/o> .'`,
    response: `HTTP/1.1 201 Created`,
    lang: "http",
  },
  {
    id: "explain",
    title: "EXPLAIN — plan only, nothing executed",
    blurb:
      "?explain=true returns the chosen join plan with index-range cardinality estimates; ?explain=analyze runs + traces.",
    curl: `curl -G '${ENDPOINT}/sparql?explain=true' \\
  --data-urlencode 'query=SELECT * WHERE { ?a <http://ex/knows> ?b . ?b <http://ex/age> ?age }'`,
    response: `EXPLAIN (SELECT) — planning-only dry run; nothing is executed.
Cardinalities are index-range estimates; join strategies marked (predicted) depend on actual row counts at run time.
Plan:
  Project ?a, ?age, ?b
    BGP [binary join plan: greedy GOO ordering] (2 patterns)
      1. scan ?a <http://ex/knows> ?b (est 2 rows, sorted by ?b) [seed: smallest estimate]
      2. merge join on ?b with scan ?b <http://ex/age> ?age (est 3 rows, sorted by ?b) → est 2 rows`,
    lang: "text",
  },
  {
    id: "metrics",
    title: "Prometheus /metrics",
    blurb:
      "Prometheus text exposition: per-endpoint request counter, a query-duration histogram, live triple count, applied-update counter and active-subscription gauge. The request counts and the histogram timings are run-dependent — this is one representative capture (after the two walkthrough writes: triples 7, updates 2).",
    curl: `curl ${ENDPOINT}/metrics`,
    response: `# HELP sparq_http_requests_total Total HTTP requests by endpoint and response status.
# TYPE sparq_http_requests_total counter
sparq_http_requests_total{endpoint="/metrics",status="200"} 2
sparq_http_requests_total{endpoint="/sparql",status="200"} 1
sparq_http_requests_total{endpoint="/sparql",status="204"} 2
# HELP sparq_query_duration_seconds Wall time of /sparql requests (query + update operations).
# TYPE sparq_query_duration_seconds histogram
sparq_query_duration_seconds_bucket{le="0.001"} 0
sparq_query_duration_seconds_bucket{le="0.005"} 3
sparq_query_duration_seconds_bucket{le="0.01"} 3
sparq_query_duration_seconds_bucket{le="0.05"} 3
sparq_query_duration_seconds_bucket{le="0.1"} 3
sparq_query_duration_seconds_bucket{le="0.5"} 3
sparq_query_duration_seconds_bucket{le="1"} 3
sparq_query_duration_seconds_bucket{le="5"} 3
sparq_query_duration_seconds_bucket{le="10"} 3
sparq_query_duration_seconds_bucket{le="+Inf"} 3
sparq_query_duration_seconds_sum 0.008690741
sparq_query_duration_seconds_count 3
# HELP sparq_active_subscriptions Currently active WebSocket subscriptions.
# TYPE sparq_active_subscriptions gauge
sparq_active_subscriptions 0
# HELP sparq_graph_triples Triples in the published graph.
# TYPE sparq_graph_triples gauge
sparq_graph_triples 7
# HELP sparq_updates_total Successfully applied SPARQL updates.
# TYPE sparq_updates_total counter
sparq_updates_total 2`,
    lang: "text",
  },
];

/** One frame in the live-subscription transcript (an SSE event or a client action). */
export interface SubFrame {
  /** Who emits this frame. */
  side: "client" | "server" | "note";
  /** Display label, e.g. "GET /subscriptions/sse", "event: notification". */
  label: string;
  /** The verbatim frame payload — for a server frame the RAW SSE wire lines
   * (`event:` / `data:` JSON / `id:`), for a client/note frame the curl command. */
  body: string;
  /** The per-subscription SSE sequence id this frame carries, if any. */
  sequence?: number;
}

// Byte-for-byte capture of an SSE subscription firing: open the stream, get the `subscribed`
// ack + a sequence-0 full snapshot, then a POST UPDATE commits and the SAME stream pushes a
// sequence-1 incremental `addedResults` diff. This is the load-bearing "push an Update ->
// watch the subscription fire" demo, recorded from `GET /subscriptions/sse` on the default
// build. The frame bodies below are the RAW wire lines, exactly as the server emits them:
//   * each notification is `event:` then a single-line `data:` JSON then `id:` (the SSE id
//     line follows the data line — see subscriptions.rs::notification_event),
//   * every non-string literal carries its `datatype` (term_json always emits it for a
//     non-xsd:string literal — so the ages are typed xsd:integer, not bare strings),
//   * serde serialises object keys in sorted order (no preserve_order), so the notification
//     object is {addedResults, alias, id, removedResults, sequence} and each binding is
//     {age, s} with the literal as {datatype, type, value}.
// Do not "tidy" these into pretty-printed JSON: that would no longer be the wire bytes, and
// the unit test asserts the literal serialization to keep this from drifting back to a mock.
export const SUBSCRIPTION_TRANSCRIPT: SubFrame[] = [
  {
    side: "client",
    label: "GET /subscriptions/sse",
    body: `curl -N -G '${ENDPOINT}/subscriptions/sse' \\
  --data-urlencode 'query=SELECT ?s ?age WHERE { ?s <http://ex/age> ?age } ORDER BY ?age' \\
  --data-urlencode 'alias=ages'`,
  },
  {
    side: "server",
    label: "event: subscribed",
    body: `event: subscribed
data: {"subscribed":{"alias":"ages","id":1}}`,
  },
  {
    side: "server",
    label: "event: notification  (sequence 0 — full snapshot)",
    sequence: 0,
    body: `event: notification
data: {"notification":{"addedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[{"age":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"25"},"s":{"type":"uri","value":"http://ex/bob"}},{"age":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"30"},"s":{"type":"uri","value":"http://ex/alice"}},{"age":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"41"},"s":{"type":"uri","value":"http://ex/carol"}},{"age":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"52"},"s":{"type":"uri","value":"http://ex/dave"}}]}},"alias":"ages","id":1,"removedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[]}},"sequence":0}}
id: 0`,
  },
  {
    side: "note",
    label: "… in another tab: a SPARQL UPDATE commits …",
    body: `curl ${ENDPOINT}/sparql -H 'Content-Type: application/sparql-update' \\
  --data 'INSERT DATA { <http://ex/frank> <http://ex/age> 63 }'`,
  },
  {
    side: "server",
    label: "event: notification  (sequence 1 — incremental diff)",
    sequence: 1,
    body: `event: notification
data: {"notification":{"addedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[{"age":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"63"},"s":{"type":"uri","value":"http://ex/frank"}}]}},"alias":"ages","id":1,"removedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[]}},"sequence":1}}
id: 1`,
  },
];

/**
 * Replay the transcript up to and including `step` frames. Returns the visible
 * prefix. `step` is clamped to `[0, length]`; the page steps it on a timer / click.
 */
export function transcriptUpTo(step: number): SubFrame[] {
  const n = Math.max(0, Math.min(step, SUBSCRIPTION_TRANSCRIPT.length));
  return SUBSCRIPTION_TRANSCRIPT.slice(0, n);
}

/**
 * The strictly-increasing SSE sequence numbers carried by the server `notification`
 * frames, in order. Used to assert the snapshot-then-diff shape (0, 1, …).
 */
export function transcriptSequences(): number[] {
  return SUBSCRIPTION_TRANSCRIPT.filter(
    (f) => f.side === "server" && typeof f.sequence === "number",
  ).map((f) => f.sequence as number);
}

/** Look up a recipe by id (undefined if unknown). */
export function recipeById(id: string): Recipe | undefined {
  return RECIPES.find((r) => r.id === id);
}
