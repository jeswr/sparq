// [OPUS-4.8] sq-rnwc — pure, framework-free data + helpers for the
// /surface/http-server walkthrough (tier-e). The hosted sparq-server is NOT in
// the wasm bundle and the static GitHub-Pages site has no backend to talk to, so
// this surface is the honest "captured curl / WS-frame walkthrough" fallback the
// feature-showcase design names for tier-e (research/feature-showcase-site-design.md
// §0, surface (e)).
//
// EVERYTHING in this module is REAL captured I/O. The curl recipes were run, and
// their responses recorded verbatim, against a `sparq-server --format turtle` over
// the tiny `ex:` dataset below. The live-subscription transcript is a verbatim
// recording of an SSE `/subscriptions/sse` stream firing: a sequence-0 full
// snapshot, then a sequence-1 incremental `addedResults` diff when a SPARQL UPDATE
// committed — exactly the "push an Update -> watch the subscription fire" demo.
// Keeping it here (no React, no network) lets the page replay it deterministically
// and lets `node --test` assert the framing without a server.
//
// Grounded in skills/http-server/SKILL.md (the canonical endpoint contract).

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

// Captured verbatim from a running `sparq-server --format turtle data.ttl` over SEED_TURTLE.
// The UPDATE/CONSTRUCT/metrics recipes reflect the same server after the seed plus the
// walkthrough's own writes, so the numbers are internally consistent with the transcript.
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
      "CONSTRUCT / DESCRIBE negotiate an RDF syntax (N-Triples default; prefix-compacting Turtle or RDF/XML).",
    curl: `curl -G ${ENDPOINT}/sparql -H 'Accept: text/turtle' \\
  --data-urlencode 'query=CONSTRUCT { ?s ?p ?o } WHERE { ?s <http://ex/knows> ?o . ?s ?p ?o }'`,
    response: `@prefix ex: <http://ex/> .

ex:alice ex:knows ex:bob ;
    ex:age 30 .

ex:carol ex:knows ex:alice ;
    ex:age 41 .`,
    lang: "turtle",
  },
  {
    id: "update",
    title: "UPDATE — atomic, 204 No Content",
    blurb:
      "A SPARQL Update commits atomically (failure → 400, no partial effect) and returns 204; every response carries the hardening header set.",
    curl: `curl -i ${ENDPOINT}/sparql \\
  -H 'Content-Type: application/sparql-update' \\
  --data 'INSERT DATA { <http://ex/dave> <http://ex/age> 52 }'`,
    response: `HTTP/1.1 204 No Content
sparq-generation: 1
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
Plan:
  Project ?a, ?age, ?b
    BGP [binary join plan: greedy GOO ordering] (2 patterns)
      1. scan ?a <http://ex/knows> ?b (est 2 rows, sorted by ?b) [seed: smallest estimate]
      2. merge join on ?b with scan ?b <http://ex/age> ?age (est 6 rows, sorted by ?b) → est 4 rows`,
    lang: "text",
  },
  {
    id: "metrics",
    title: "Prometheus /metrics",
    blurb:
      "Hand-rolled Prometheus text exposition: live triple count, applied-update counter, active-subscription gauge (gated by --auth-token-read).",
    curl: `curl ${ENDPOINT}/metrics`,
    response: `# TYPE sparq_active_subscriptions gauge
sparq_active_subscriptions 0
# TYPE sparq_graph_triples gauge
sparq_graph_triples 8
# TYPE sparq_updates_total counter
sparq_updates_total 4`,
    lang: "text",
  },
];

/** One frame in the live-subscription transcript (an SSE event or a client action). */
export interface SubFrame {
  /** Who emits this frame. */
  side: "client" | "server" | "note";
  /** Display label, e.g. "GET /subscriptions/sse", "event: notification". */
  label: string;
  /** The verbatim frame payload (SSE `data:` JSON, or a note). */
  body: string;
  /** The per-subscription SSE sequence id this frame carries, if any. */
  sequence?: number;
}

// Verbatim capture of an SSE subscription firing: open the stream, get the `subscribed`
// ack + a sequence-0 full snapshot, then a POST UPDATE commits and the SAME stream pushes a
// sequence-1 incremental `addedResults` diff. This is the load-bearing "push an Update ->
// watch the subscription fire" demo, recorded from `GET /subscriptions/sse`.
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
    body: `data: {"subscribed":{"alias":"ages","id":1}}`,
  },
  {
    side: "server",
    label: "event: notification  (sequence 0 — full snapshot)",
    sequence: 0,
    body: `id: 0
data: {"notification":{"id":1,"alias":"ages","sequence":0,
  "addedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[
    {"s":{"type":"uri","value":"http://ex/bob"},  "age":{"value":"25","type":"literal"}},
    {"s":{"type":"uri","value":"http://ex/alice"},"age":{"value":"30","type":"literal"}},
    {"s":{"type":"uri","value":"http://ex/carol"},"age":{"value":"41","type":"literal"}},
    {"s":{"type":"uri","value":"http://ex/dave"}, "age":{"value":"52","type":"literal"}}]}},
  "removedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[]}}}}`,
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
    body: `id: 1
data: {"notification":{"id":1,"alias":"ages","sequence":1,
  "addedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[
    {"s":{"type":"uri","value":"http://ex/frank"},"age":{"value":"63","type":"literal"}}]}},
  "removedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[]}}}}`,
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
