// [OPUS-4.8] sq-9ij6 — unit tests for the live SPARQL subscriptions client
// (`@sparq/client` `subscriptions.ts`): the SSE-URL derivation (reusing the endpoint
// config + bearer posture), the SEPA notification parser, the SSE frame splitter / data
// extractor, the row-keyed live-diff reducer (matching the server's set-semantics diff),
// and the `openSubscription` stream lifecycle (open / notification / error / clean close)
// driven over an injected `fetch`. These cover the pure, framework-free logic the
// subscriptions VIEW relies on. Run via `npm run test:unit`.
//
// [FABLE-5] sq-140b adds the MULTIPLEXED WebSocket transport suite (`ws-subscriptions.ts`):
// the ws(s) URL derivation, the `bearer.<token>` subprotocol auth channel, and the
// many-subscriptions-per-socket routing (ordered acks, id-routed notifications, refusal
// attribution, per-handle unsubscribe, socket close) driven over an injected WebSocket.
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
// [FABLE-5] sq-140b — the multiplexed WebSocket transport of the same surface: one socket,
// many subscriptions, `subscribe`/`unsubscribe` frames, `bearer.<token>` subprotocol auth.
import {
  buildSubscriptionSocketUrl,
  openSubscriptionSocket,
} from "../../packages/sparq-client/src/ws-subscriptions.ts";

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

test("parseSubscriptionData tags the WS-only unsubscribed ack (sq-140b)", () => {
  const ev = parseSubscriptionData(`{"unsubscribed":{"id":3}}`);
  assert.equal(ev.kind, "unsubscribed");
  assert.equal(ev.id, 3);
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

// --- [FABLE-5] sq-140b: the multiplexed WebSocket transport ------------------

// --- buildSubscriptionSocketUrl ---

test("buildSubscriptionSocketUrl rewrites a trailing /sparql to ws://…/subscriptions", () => {
  assert.equal(
    buildSubscriptionSocketUrl({ url: "http://127.0.0.1:3030/sparql" }),
    "ws://127.0.0.1:3030/subscriptions",
  );
});

test("buildSubscriptionSocketUrl maps https to wss and strips any query string", () => {
  assert.equal(
    buildSubscriptionSocketUrl({ url: "https://data.example.org/sparql?x=1" }),
    "wss://data.example.org/subscriptions",
  );
});

test("buildSubscriptionSocketUrl mounts at the origin root when the path is not /sparql", () => {
  assert.equal(
    buildSubscriptionSocketUrl({ url: "http://localhost:3030/" }),
    "ws://localhost:3030/subscriptions",
  );
});

test("buildSubscriptionSocketUrl returns null for an invalid endpoint URL", () => {
  assert.equal(buildSubscriptionSocketUrl({ url: "nope" }), null);
  assert.equal(buildSubscriptionSocketUrl({ url: "" }), null);
});

// --- openSubscriptionSocket (the multiplexed lifecycle over an injected WebSocket) ---

/** A scriptable stand-in for the platform WebSocket (the transport's WebSocketLike slice). */
class FakeWebSocket {
  static last = null;
  constructor(url, protocols) {
    this.url = url;
    this.protocols = protocols;
    this.sent = []; // parsed JSON frames the client sent
    this.closeCalls = 0;
    this.onopen = null;
    this.onmessage = null;
    this.onerror = null;
    this.onclose = null;
    FakeWebSocket.last = this;
  }
  send(data) {
    this.sent.push(JSON.parse(data));
  }
  close() {
    this.closeCalls += 1;
    // Mirror the platform: the close event fires after a client-side close().
    this.onclose?.({ code: 1000, wasClean: true });
  }
  // Test drivers (server side of the wire):
  open() {
    this.onopen?.();
  }
  message(obj) {
    this.onmessage?.({ data: JSON.stringify(obj) });
  }
  fail(code = 1006) {
    this.onerror?.();
    this.onclose?.({ code, wasClean: false });
  }
}

/** A recording per-subscription handler set. */
function recordingHandlers() {
  const rec = { events: [], opened: 0, closed: 0, closeError: undefined };
  rec.handlers = {
    onOpen: () => (rec.opened += 1),
    onEvent: (e) => rec.events.push(e),
    onClose: (err) => {
      rec.closed += 1;
      rec.closeError = err;
    },
  };
  return rec;
}

const WS_CONFIG = { url: "http://127.0.0.1:3030/sparql", token: "s3cret" };
const SELECT = "SELECT ?s WHERE { ?s ?p ?o }";

function wsNotification(id, seq, values) {
  return {
    notification: {
      id,
      sequence: seq,
      addedResults: {
        head: { vars: ["s"] },
        results: { bindings: values.map((v) => ({ s: { type: "uri", value: v } })) },
      },
      removedResults: { head: { vars: ["s"] }, results: { bindings: [] } },
    },
  };
}

test("openSubscriptionSocket offers the bearer subprotocol and multiplexes two subscriptions by id", () => {
  const socket = openSubscriptionSocket(
    WS_CONFIG,
    {},
    { webSocketImpl: FakeWebSocket },
  );
  const ws = FakeWebSocket.last;
  assert.equal(ws.url, "ws://127.0.0.1:3030/subscriptions");
  assert.deepEqual(ws.protocols, ["bearer.s3cret"], "the token travels ONLY as bearer.<token>");

  // Both subscribes are requested before the handshake completes: queued, then flushed.
  const a = recordingHandlers();
  const b = recordingHandlers();
  socket.subscribe(SELECT, a.handlers, { alias: "  ages  " });
  socket.subscribe(SELECT, b.handlers);
  assert.equal(ws.sent.length, 0, "nothing is sent before the socket opens");

  ws.open();
  assert.equal(ws.sent.length, 2);
  assert.deepEqual(ws.sent[0], { subscribe: { query: SELECT, alias: "ages" } });
  assert.deepEqual(ws.sent[1], { subscribe: { query: SELECT } });
  assert.equal(a.opened, 1);
  assert.equal(b.opened, 1);

  // Acks answer in order: first ack → a, second → b.
  ws.message({ subscribed: { id: 1, alias: "ages" } });
  ws.message({ subscribed: { id: 2 } });
  assert.equal(a.events[0].kind, "subscribed");
  assert.equal(a.events[0].ack.id, 1);
  assert.equal(b.events[0].ack.id, 2);

  // Live frames route by id, on ONE socket.
  ws.message(wsNotification(2, 0, ["http://b"]));
  ws.message(wsNotification(1, 0, ["http://a"]));
  assert.equal(a.events[1].notification.id, 1);
  assert.equal(b.events[1].notification.id, 2);
  assert.equal(a.events.length, 2);
  assert.equal(b.events.length, 2);
});

test("openSubscriptionSocket does not offer a subprotocol when no token is configured", () => {
  openSubscriptionSocket(
    { url: "http://127.0.0.1:3030/sparql" },
    {},
    { webSocketImpl: FakeWebSocket },
  );
  assert.deepEqual(FakeWebSocket.last.protocols, []);
});

test("an id-less refusal error is attributed to the oldest in-flight subscribe only", () => {
  const socket = openSubscriptionSocket(WS_CONFIG, {}, { webSocketImpl: FakeWebSocket });
  const ws = FakeWebSocket.last;
  ws.open();
  const bad = recordingHandlers();
  const good = recordingHandlers();
  socket.subscribe("ASK { ?s ?p ?o }", bad.handlers);
  socket.subscribe(SELECT, good.handlers);

  // The server answers each subscribe in order: refusal for the ASK, ack for the SELECT.
  ws.message({ error: { message: "only SELECT queries can be subscribed" } });
  ws.message({ subscribed: { id: 1 } });

  assert.equal(bad.events.length, 1);
  assert.equal(bad.events[0].kind, "error");
  assert.equal(bad.closed, 1, "the refused subscription closes");
  assert.equal(bad.closeError, undefined, "…cleanly (the error came via onEvent, as on SSE)");
  assert.equal(good.events[0].kind, "subscribed", "the later subscribe is unaffected");
  assert.equal(good.closed, 0);
});

test("an error with an unknown/stale id is unrouted and never consumes a pending subscribe", () => {
  const unrouted = [];
  const socket = openSubscriptionSocket(
    WS_CONFIG,
    { onUnrouted: (e) => unrouted.push(e) },
    { webSocketImpl: FakeWebSocket },
  );
  const ws = FakeWebSocket.last;
  ws.open();
  const a = recordingHandlers();
  socket.subscribe(SELECT, a.handlers); // in flight — its ack has not arrived yet

  // A stale id (e.g. an unsubscribe answered after the server already terminated that
  // subscription) matches neither `closing` nor `active`: it must NOT be treated as the
  // pending subscribe's refusal.
  ws.message({ error: { message: "unknown subscription id", id: 99 } });
  assert.equal(unrouted.length, 1, "the stale-id error is reported as unrouted");
  assert.equal(unrouted[0].kind, "error");
  assert.equal(a.events.length, 0, "the pending subscribe saw nothing");
  assert.equal(a.closed, 0, "…and was not closed");

  // The subsequent ack still belongs to that pending subscribe.
  ws.message({ subscribed: { id: 1 } });
  assert.equal(a.events[0].kind, "subscribed");
  assert.equal(a.events[0].ack.id, 1);
});

test("a terminating error with an id closes just that subscription; the socket stays up", () => {
  const socketHandlers = { closed: 0 };
  const socket = openSubscriptionSocket(
    WS_CONFIG,
    { onClose: () => (socketHandlers.closed += 1) },
    { webSocketImpl: FakeWebSocket },
  );
  const ws = FakeWebSocket.last;
  ws.open();
  const a = recordingHandlers();
  const b = recordingHandlers();
  socket.subscribe(SELECT, a.handlers);
  socket.subscribe(SELECT, b.handlers);
  ws.message({ subscribed: { id: 1 } });
  ws.message({ subscribed: { id: 2 } });

  ws.message({ error: { message: "re-evaluation failed, subscription terminated: timeout", id: 1 } });
  assert.equal(a.events.at(-1).kind, "error");
  assert.equal(a.closed, 1);
  assert.equal(b.closed, 0, "the sibling subscription lives on");
  assert.equal(socketHandlers.closed, 0, "the socket itself stays open");

  // The dead id no longer routes.
  ws.message(wsNotification(1, 1, ["http://zombie"]));
  assert.equal(a.events.at(-1).kind, "error", "no further events after the terminating error");
});

test("closing one handle sends unsubscribe, swallows the ack, and leaves the socket open", () => {
  const unrouted = [];
  const socket = openSubscriptionSocket(
    WS_CONFIG,
    { onUnrouted: (e) => unrouted.push(e) },
    { webSocketImpl: FakeWebSocket },
  );
  const ws = FakeWebSocket.last;
  ws.open();
  const a = recordingHandlers();
  const b = recordingHandlers();
  const handleA = socket.subscribe(SELECT, a.handlers);
  socket.subscribe(SELECT, b.handlers);
  ws.message({ subscribed: { id: 1 } });
  ws.message({ subscribed: { id: 2 } });

  handleA.close();
  assert.deepEqual(ws.sent.at(-1), { unsubscribe: { id: 1 } });
  assert.equal(a.closed, 1);
  ws.message({ unsubscribed: { id: 1 } });
  assert.equal(unrouted.length, 0, "the expected unsubscribed ack is swallowed, not noise");
  assert.equal(ws.closeCalls, 0, "the socket stays up for the sibling");

  // Closing the same handle again is a no-op (no double onClose, no second frame).
  handleA.close();
  assert.equal(a.closed, 1);
  assert.deepEqual(ws.sent.at(-1), { unsubscribe: { id: 1 } });
});

test("socket.close() ends every subscription cleanly", () => {
  let socketErr = "unset";
  const socket = openSubscriptionSocket(
    WS_CONFIG,
    { onClose: (err) => (socketErr = err) },
    { webSocketImpl: FakeWebSocket },
  );
  const ws = FakeWebSocket.last;
  ws.open();
  const a = recordingHandlers();
  socket.subscribe(SELECT, a.handlers);
  ws.message({ subscribed: { id: 1 } });

  socket.close();
  assert.equal(ws.closeCalls, 1);
  assert.equal(a.closed, 1);
  assert.equal(a.closeError, undefined, "a client-initiated close is clean");
  assert.equal(socketErr, undefined);

  // A subscribe after close fails closed.
  const late = recordingHandlers();
  socket.subscribe(SELECT, late.handlers);
  assert.equal(late.closed, 1);
  assert.match(late.closeError, /closed/);
});

test("a handshake failure surfaces the honest possible-causes message to every waiting subscription", () => {
  let socketErr;
  const socket = openSubscriptionSocket(
    WS_CONFIG,
    { onClose: (err) => (socketErr = err) },
    { webSocketImpl: FakeWebSocket },
  );
  const ws = FakeWebSocket.last;
  const a = recordingHandlers();
  socket.subscribe(SELECT, a.handlers); // queued — the handshake never completes
  ws.fail(1006);

  assert.match(socketErr, /handshake failed/);
  assert.match(socketErr, /401/, "names the token refusal possibility");
  assert.match(socketErr, /mixed content/, "names the ws:// -from-https possibility");
  assert.equal(a.closed, 1);
  assert.equal(a.closeError, socketErr);
  assert.equal(a.opened, 0, "the queued subscribe never claimed to open");
});

test("openSubscriptionSocket fails closed on an invalid endpoint URL", () => {
  let socketErr;
  const socket = openSubscriptionSocket(
    { url: "nope" },
    { onClose: (err) => (socketErr = err) },
    { webSocketImpl: FakeWebSocket },
  );
  assert.match(socketErr, /valid absolute endpoint URL/);
  const a = recordingHandlers();
  const handle = socket.subscribe(SELECT, a.handlers);
  assert.equal(a.closed, 1);
  assert.match(a.closeError, /valid absolute endpoint URL/);
  handle.close(); // dead handle — must not throw
  socket.close();
});
