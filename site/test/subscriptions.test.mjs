// [OPUS-4.8] sq-9ij6 — unit tests for the live SPARQL subscriptions client
// (`@sparq/client` `subscriptions.ts`): the SSE-URL derivation (reusing the endpoint
// config + bearer posture), the SEPA notification parser, the SSE frame splitter / data
// extractor, the row-keyed live-diff reducer (matching the server's set-semantics diff),
// and the `openSubscription` stream lifecycle (open / notification / error / clean close)
// driven over an injected `fetch`. These cover the pure, framework-free logic the
// subscriptions VIEW relies on. Run via `npm run test:unit`.
//
// Imported by RELATIVE path (not the `@sparq/client` alias) so the build-free ts-loader
// resolves it without a custom alias resolver — exactly like endpoint.test.mjs.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  buildSubscriptionUrl,
  parseSubscriptionData,
  splitSseFrames,
  frameData,
  rowKey,
  emptyLiveResultSet,
  applyNotification,
  liveResults,
  openSubscription,
} from "../../packages/sparq-client/src/subscriptions.ts";

// --- buildSubscriptionUrl ---------------------------------------------------

test("buildSubscriptionUrl rewrites a trailing /sparql to /subscriptions/sse with the query param", () => {
  const url = buildSubscriptionUrl(
    { url: "http://127.0.0.1:3030/sparql" },
    "SELECT ?s WHERE { ?s ?p ?o }",
  );
  const u = new URL(url);
  assert.equal(u.origin, "http://127.0.0.1:3030");
  assert.equal(u.pathname, "/subscriptions/sse");
  assert.equal(u.searchParams.get("query"), "SELECT ?s WHERE { ?s ?p ?o }");
  assert.equal(u.searchParams.get("alias"), null);
});

test("buildSubscriptionUrl carries a non-empty alias and trims it", () => {
  const url = buildSubscriptionUrl(
    { url: "https://data.example.org/sparql" },
    "SELECT ?s WHERE { ?s ?p ?o }",
    "  watch  ",
  );
  const u = new URL(url);
  assert.equal(u.searchParams.get("alias"), "watch");
});

test("buildSubscriptionUrl mounts at the origin root when the path is not /sparql", () => {
  const url = buildSubscriptionUrl({ url: "http://localhost:3030/" }, "SELECT * WHERE {?s ?p ?o}");
  const u = new URL(url);
  assert.equal(u.pathname, "/subscriptions/sse");
});

test("buildSubscriptionUrl returns null for an invalid endpoint URL", () => {
  assert.equal(buildSubscriptionUrl({ url: "not a url" }, "SELECT * WHERE {?s ?p ?o}"), null);
  assert.equal(buildSubscriptionUrl({ url: "" }, "SELECT * WHERE {?s ?p ?o}"), null);
});

// --- parseSubscriptionData (the SEPA envelope parser) -----------------------

test("parseSubscriptionData tags the subscribed ack", () => {
  const ev = parseSubscriptionData(`{"subscribed":{"alias":"ages","id":1}}`);
  assert.equal(ev.kind, "subscribed");
  assert.equal(ev.ack.id, 1);
  assert.equal(ev.ack.alias, "ages");
});

test("parseSubscriptionData tags a notification with its diff results + sequence", () => {
  // The byte-for-byte sequence-1 frame from the captured server transcript.
  const data = `{"notification":{"addedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[{"age":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"63"},"s":{"type":"uri","value":"http://ex/frank"}}]}},"alias":"ages","id":1,"removedResults":{"head":{"vars":["s","age"]},"results":{"bindings":[]}},"sequence":1}}`;
  const ev = parseSubscriptionData(data);
  assert.equal(ev.kind, "notification");
  assert.equal(ev.notification.id, 1);
  assert.equal(ev.notification.sequence, 1);
  assert.equal(ev.notification.alias, "ages");
  assert.equal(ev.notification.addedResults.results.bindings.length, 1);
  assert.equal(ev.notification.removedResults.results.bindings.length, 0);
});

test("parseSubscriptionData tags a terminating error", () => {
  const ev = parseSubscriptionData(`{"error":{"message":"re-evaluation failed","id":5}}`);
  assert.equal(ev.kind, "error");
  assert.equal(ev.error.message, "re-evaluation failed");
  assert.equal(ev.error.id, 5);
});

test("parseSubscriptionData yields 'unknown' for junk, never throws", () => {
  assert.equal(parseSubscriptionData("not json").kind, "unknown");
  assert.equal(parseSubscriptionData(`{"surprise":1}`).kind, "unknown");
  assert.equal(parseSubscriptionData("42").kind, "unknown");
});

// --- splitSseFrames + frameData (the wire framing) --------------------------

test("splitSseFrames splits on blank lines and keeps a trailing partial frame", () => {
  const buf = "event: subscribed\ndata: {\"subscribed\":{\"id\":1}}\n\nevent: notification\ndata: {\"x\":1}";
  const { frames, rest } = splitSseFrames(buf);
  assert.equal(frames.length, 1);
  assert.match(frames[0], /subscribed/);
  // The second (incomplete) frame is carried forward, not parsed.
  assert.match(rest, /notification/);
});

test("splitSseFrames tolerates CRLF line endings", () => {
  const buf = "event: subscribed\r\ndata: {\"subscribed\":{\"id\":1}}\r\n\r\n";
  const { frames, rest } = splitSseFrames(buf);
  assert.equal(frames.length, 1);
  assert.equal(rest, "");
});

test("frameData extracts the data payload and ignores keep-alive comment frames", () => {
  // The raw notification frame shape: event:, data: JSON, id: (the id line is not data).
  const frame = `event: notification\ndata: {"notification":{"sequence":0}}\nid: 0`;
  assert.equal(frameData(frame), `{"notification":{"sequence":0}}`);
  // A keep-alive ping (a bare comment line) carries no data.
  assert.equal(frameData(": ping"), null);
  // A dataless frame yields null.
  assert.equal(frameData("event: foo"), null);
});

// --- rowKey + applyNotification (the live set-semantics diff) ----------------

test("rowKey is order-independent over the binding's variables", () => {
  const a = { s: { type: "uri", value: "http://x" }, age: { type: "literal", value: "1" } };
  const b = { age: { type: "literal", value: "1" }, s: { type: "uri", value: "http://x" } };
  assert.equal(rowKey(a), rowKey(b), "the same row keys identically regardless of var order");
});

function notification(seq, added, removed = []) {
  return {
    id: 1,
    sequence: seq,
    addedResults: { head: { vars: ["s"] }, results: { bindings: added } },
    removedResults: { head: { vars: ["s"] }, results: { bindings: removed } },
  };
}

test("applyNotification builds the snapshot then nets added/removed deltas", () => {
  let set = emptyLiveResultSet();

  // Sequence 0: the full snapshot (two rows added, none removed).
  const snap = applyNotification(
    set,
    notification(0, [
      { s: { type: "uri", value: "http://a" } },
      { s: { type: "uri", value: "http://b" } },
    ]),
  );
  assert.equal(snap.added, 2);
  assert.equal(snap.removed, 0);
  set = snap.next;
  assert.deepEqual(set.vars, ["s"]);
  assert.equal(set.rows.size, 2);

  // Sequence 1: add one, remove one — the live set tracks the net.
  const diff = applyNotification(
    set,
    notification(
      1,
      [{ s: { type: "uri", value: "http://c" } }],
      [{ s: { type: "uri", value: "http://a" } }],
    ),
  );
  assert.equal(diff.added, 1);
  assert.equal(diff.removed, 1);
  set = diff.next;
  assert.equal(set.rows.size, 2);

  const values = liveResults(set).results.bindings.map((r) => r.s.value).sort();
  assert.deepEqual(values, ["http://b", "http://c"]);
});

test("applyNotification does not double-count an already-present add or an absent remove", () => {
  let set = emptyLiveResultSet();
  set = applyNotification(set, notification(0, [{ s: { type: "uri", value: "http://a" } }])).next;

  // Re-adding the same row + removing a row that was never present nets to nothing.
  const r = applyNotification(
    set,
    notification(
      1,
      [{ s: { type: "uri", value: "http://a" } }],
      [{ s: { type: "uri", value: "http://zzz" } }],
    ),
  );
  assert.equal(r.added, 0, "an already-present row is not counted as added");
  assert.equal(r.removed, 0, "an absent row is not counted as removed");
  assert.equal(r.next.rows.size, 1);
});

// --- openSubscription (the stream lifecycle over an injected fetch) ---------

/** Build a Response whose body streams the given SSE text (one chunk). */
function sseResponse(text, status = 200) {
  const body = new ReadableStream({
    start(ctrl) {
      ctrl.enqueue(new TextEncoder().encode(text));
      ctrl.close();
    },
  });
  return new Response(status === 200 ? body : null, {
    status,
    headers: { "Content-Type": "text/event-stream" },
  });
}

test("openSubscription opens, parses each frame, and closes cleanly when the stream ends", async () => {
  const transcript =
    `event: subscribed\ndata: {"subscribed":{"id":1}}\n\n` +
    `event: notification\ndata: {"notification":{"id":1,"sequence":0,"addedResults":{"head":{"vars":["s"]},"results":{"bindings":[{"s":{"type":"uri","value":"http://a"}}]}},"removedResults":{"head":{"vars":["s"]},"results":{"bindings":[]}}}}\nid: 0\n\n`;
  const fetchImpl = async () => sseResponse(transcript);

  const events = [];
  let opened = false;
  let closedError;
  let closed = false;
  await new Promise((resolve) => {
    openSubscription(
      { url: "http://127.0.0.1:3030/sparql" },
      "SELECT ?s WHERE { ?s ?p ?o }",
      {
        onOpen: () => {
          opened = true;
        },
        onEvent: (e) => events.push(e),
        onClose: (err) => {
          closed = true;
          closedError = err;
          resolve();
        },
      },
      { fetchImpl },
    );
  });

  assert.equal(opened, true, "onOpen fired once the 200 body flowed");
  assert.equal(closed, true);
  assert.equal(closedError, undefined, "a server stream that ends is a clean close (no error)");
  assert.equal(events.length, 2);
  assert.equal(events[0].kind, "subscribed");
  assert.equal(events[1].kind, "notification");
  assert.equal(events[1].notification.sequence, 0);
});

test("openSubscription surfaces a 401 refusal as an honest onClose error, before any event", async () => {
  const fetchImpl = async () =>
    new Response(JSON.stringify({ error: "authentication required" }), {
      status: 401,
      headers: { "Content-Type": "application/json" },
    });

  const events = [];
  let closedError;
  await new Promise((resolve) => {
    openSubscription(
      { url: "http://127.0.0.1:3030/sparql" },
      "SELECT ?s WHERE { ?s ?p ?o }",
      {
        onOpen: () => events.push("open"),
        onEvent: (e) => events.push(e),
        onClose: (err) => {
          closedError = err;
          resolve();
        },
      },
      { fetchImpl },
    );
  });

  assert.equal(events.length, 0, "no onOpen / onEvent on a refusal");
  assert.match(closedError, /Authentication required/);
});

test("openSubscription turns a transport failure into a CORS/reachability hint", async () => {
  const fetchImpl = async () => {
    throw new TypeError("Failed to fetch");
  };
  let closedError;
  await new Promise((resolve) => {
    openSubscription(
      { url: "http://data.example.org/sparql" },
      "SELECT ?s WHERE { ?s ?p ?o }",
      {
        onEvent: () => {},
        onClose: (err) => {
          closedError = err;
          resolve();
        },
      },
      { fetchImpl },
    );
  });
  assert.match(closedError, /CORS|reachable|subscription stream/);
});

test("openSubscription fails closed on an invalid endpoint URL", async () => {
  let closedError;
  await new Promise((resolve) => {
    const handle = openSubscription(
      { url: "nope" },
      "SELECT ?s WHERE { ?s ?p ?o }",
      {
        onEvent: () => {},
        onClose: (err) => {
          closedError = err;
          resolve();
        },
      },
      { fetchImpl: async () => sseResponse("") },
    );
    // close() is a no-op on the dead handle — must not throw.
    handle.close();
  });
  assert.match(closedError, /valid absolute endpoint URL/);
});
