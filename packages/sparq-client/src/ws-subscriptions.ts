// [FABLE-5] sq-140b — live SPARQL subscriptions client, the MULTIPLEXED WebSocket
// transport of the server's `/subscriptions` surface (many subscriptions per socket).
//
// This is the companion to the one-subscription-per-stream SSE transport in
// `./subscriptions.ts` (sq-9ij6). Where that one holds one `fetch` stream per live SELECT,
// this one holds ONE WebSocket and multiplexes any number of subscriptions over it with the
// server's `{"subscribe":{…}}` / `{"unsubscribe":{…}}` frames
// (`crates/sparq-server/src/subscriptions.rs`) — so a GUI holding several live views pays
// one connection, not one per view.
//
// It CONSUMES the existing `sparq-server` WebSocket API — it changes nothing server-side and
// claims no security the server does not provide. Auth reuses the endpoint client's posture
// (sq-2mke) verbatim:
//   * the SAME `EndpointConfig` (endpoint URL + optional bearer token) and the SAME honest
//     `connectionSafetyWarnings` classifier drive the caller's Connect UX — a live socket
//     over plaintext to a non-loopback host, or a token in the clear, is surfaced exactly as
//     for a query, never bypassed;
//   * the bearer token is offered ONLY as the `Sec-WebSocket-Protocol: bearer.<token>`
//     subprotocol `wsSubprotocols` derives — the exact channel the server's `ws_auth_gate`
//     validates for a browser WS upgrade (a browser cannot set an `Authorization` header on
//     the handshake; RFC 6455 has the server echo the accepted subprotocol back).
//
// WIRE CONTRACT (mirrors `subscriptions.rs`): the client sends JSON text frames
// `{"subscribe":{"query":"<SELECT>","alias"?:"…"}}` and `{"unsubscribe":{"id":n}}`. The
// server answers each `subscribe`, IN ORDER, with either a `{"subscribed":{"id"}}` ack (then
// the sequence-0 full-snapshot `notification`) or ONE `{"error":{…}}` refusal — that ordering
// is what lets this client attribute an id-less refusal to the oldest in-flight subscribe.
// Live frames (`notification`, a terminating `error`) carry the subscription `id` and are
// routed by it. The envelopes are the SAME JSON the SSE transport speaks, so
// `parseSubscriptionData` (and the `applyNotification` / `LiveResultSet` reducers) are reused
// verbatim — only the framing differs.
//
// No React, no Next.js, no DOM beyond the `WebSocket` / `URL` web-platform globals (the
// constructor is injectable for tests / Node). No performance claim is made here (this
// repo's work box is non-canonical).

import {
  type EndpointConfig,
  parseEndpointUrl,
  wsSubprotocols,
} from "./endpoint.js";
import {
  type SubscriptionEvent,
  type SubscriptionHandlers,
  type SubscriptionHandle,
  parseSubscriptionData,
} from "./subscriptions.js";

// ---------------------------------------------------------------------------
// WS endpoint-URL derivation.
// ---------------------------------------------------------------------------

/**
 * Derive the `/subscriptions` WebSocket URL from the configured `/sparql` endpoint URL:
 * `http:` → `ws:`, `https:` → `wss:`, and the path mapped exactly as the SSE sibling
 * (`buildSubscriptionUrl`) maps it — a trailing `/sparql` segment is rewritten to
 * `/subscriptions` (the common `sparq-server` shape), otherwise `/subscriptions` is mounted
 * at the endpoint's origin root. No query string: subscriptions travel as `subscribe`
 * frames, not URL parameters. Returns `null` when the endpoint URL is not a valid absolute
 * http(s) URL (the caller surfaces the same `invalid-url` safety error the query path does).
 */
export function buildSubscriptionSocketUrl(config: EndpointConfig): string | null {
  const parsed = parseEndpointUrl(config.url);
  if (!parsed) return null;
  const ws = new URL(parsed.toString());
  ws.protocol = parsed.protocol === "https:" ? "wss:" : "ws:";
  if (/\/sparql\/?$/.test(ws.pathname)) {
    ws.pathname = ws.pathname.replace(/\/sparql\/?$/, "/subscriptions");
  } else {
    ws.pathname = "/subscriptions";
  }
  ws.search = "";
  ws.hash = "";
  return ws.toString();
}

// ---------------------------------------------------------------------------
// The injectable WebSocket surface (structural, so tests fake it without a DOM lib).
// ---------------------------------------------------------------------------

/** The minimal structural slice of the platform `WebSocket` this transport drives. */
export interface WebSocketLike {
  send(data: string): void;
  close(code?: number, reason?: string): void;
  onopen: ((ev?: unknown) => void) | null;
  onmessage: ((ev: { data: unknown }) => void) | null;
  onerror: ((ev?: unknown) => void) | null;
  onclose: ((ev?: { code?: number; reason?: string; wasClean?: boolean }) => void) | null;
}

/** A `WebSocket`-shaped constructor (the platform global, or a test fake). */
export type WebSocketCtor = new (url: string, protocols?: string[]) => WebSocketLike;

// ---------------------------------------------------------------------------
// The multiplexed socket.
// ---------------------------------------------------------------------------

/** Socket-LEVEL callbacks (per-subscription events flow to each subscription's handlers). */
export interface SubscriptionSocketHandlers {
  /** The WebSocket handshake completed (queued subscribes are flushed right after). */
  onOpen?: () => void;
  /**
   * The socket closed. `error` is set with an honest, classified message when it closed
   * abnormally (handshake refused, transport failure); `undefined` on a clean client-side
   * `close()`. Every still-live subscription's `onClose` fires with the same error first.
   */
  onClose?: (error?: string) => void;
  /**
   * A server frame that could not be attributed to any live subscription (e.g. a
   * `notification` for an id this client never issued, or an `unknown` envelope).
   * Diagnostic only — omit to drop such frames.
   */
  onUnrouted?: (event: SubscriptionEvent) => void;
}

/** A handle to the multiplexed socket: add subscriptions, or close the whole connection. */
export interface SubscriptionSocket {
  /**
   * Subscribe one SELECT over this socket. Safe to call before the handshake completes
   * (the frame is queued and flushed on open). Returns a per-subscription
   * {@link SubscriptionHandle} whose `close()` unsubscribes JUST this subscription
   * (sending the `unsubscribe` frame once its server id is known) — the socket itself
   * stays up for the others.
   */
  subscribe(
    query: string,
    handlers: SubscriptionHandlers,
    options?: { alias?: string },
  ): SubscriptionHandle;
  /** Close the socket, ending every subscription on it (each gets a clean `onClose()`). */
  close(): void;
}

/** One subscription's client-side state, keyed through its lifecycle stages. */
interface SubEntry {
  query: string;
  alias?: string;
  handlers: SubscriptionHandlers;
  /** The server-assigned id, once the `subscribed` ack arrives. */
  id?: number;
  /** The caller closed the handle (suppress further dispatch to its handlers). */
  cancelled?: boolean;
}

/** A dead handle for the fail-closed paths (invalid URL / no WebSocket / closed socket). */
const DEAD_HANDLE: SubscriptionHandle = { close: () => {} };

/**
 * [FABLE-5] sq-140b — open the multiplexed subscription WebSocket and return a
 * {@link SubscriptionSocket} to subscribe/unsubscribe any number of SELECTs over it.
 *
 * The bearer token (if any) is offered ONLY as the `bearer.<token>` subprotocol
 * (`wsSubprotocols`) the server's `ws_auth_gate` validates — never in the URL, never
 * logged. Lifecycle is reported honestly, per subscription AND per socket:
 *
 * - per subscription: `onOpen` when its `subscribe` frame is actually sent; then `onEvent`
 *   with the `subscribed` ack, the sequence-0 snapshot `notification`, and each subsequent
 *   diff. A server refusal or terminating error arrives as `onEvent({kind:"error"})`
 *   followed by `onClose()` — the same order the SSE transport reports, so the same
 *   handler code drives both transports.
 * - per socket: `onOpen` on handshake completion; `onClose(error?)` when the socket ends.
 *   A browser deliberately hides WHY a WS handshake failed (no status is exposed on
 *   `onerror`/`onclose`), so an abnormal close carries an honest LIST of the possible
 *   causes (401 token refusal / unreachable server / mixed content) rather than a guessed
 *   single one.
 *
 * Closing a per-subscription handle sends `{"unsubscribe":{"id"}}` once the id is known
 * (releasing the server-side slot) and swallows the matching `unsubscribed` ack; closing
 * the socket ends everything. The caller runs the SAME `connectionSafetyWarnings`
 * classifier before connecting — this transport adds no bypass of it.
 */
export function openSubscriptionSocket(
  config: EndpointConfig,
  handlers: SubscriptionSocketHandlers = {},
  options: { webSocketImpl?: WebSocketCtor } = {},
): SubscriptionSocket {
  const url = buildSubscriptionSocketUrl(config);

  /** A socket that failed before connecting: every subscribe fails closed with `message`. */
  const deadSocket = (message: string): SubscriptionSocket => {
    handlers.onClose?.(message);
    return {
      subscribe: (_query, subHandlers) => {
        subHandlers.onClose?.(message);
        return DEAD_HANDLE;
      },
      close: () => {},
    };
  };

  if (url === null) {
    // Invalid endpoint URL — fail closed, consistent with the query path's `invalid-url`.
    return deadSocket(
      "Enter a valid absolute endpoint URL (http:// or https://) in the Connect panel before subscribing.",
    );
  }

  const Ctor =
    options.webSocketImpl ??
    (globalThis as unknown as { WebSocket?: WebSocketCtor }).WebSocket;
  if (!Ctor) {
    return deadSocket("WebSocket is not available in this environment.");
  }

  let ws: WebSocketLike;
  try {
    ws = new Ctor(url, wsSubprotocols(config.token));
  } catch (e) {
    // A browser throws synchronously on e.g. a mixed-content ws:// from an https:// page.
    const base = e instanceof Error ? e.message : String(e);
    return deadSocket(
      `Could not open the subscription socket (${base}). An https:// page cannot open a plaintext ws:// socket (mixed content) — use a wss:// (https) endpoint.`,
    );
  }

  let open = false;
  let closedByClient = false;
  let sawTransportError = false;
  let finished = false;
  /** Subscribes requested before the handshake completed (flushed on open, in order). */
  const queued: SubEntry[] = [];
  /** Subscribe frames sent, ack not yet received — FIFO, matching the server's ordering. */
  const pending: SubEntry[] = [];
  /** Acked subscriptions, keyed by their server id. */
  const active = new Map<number, SubEntry>();
  /** Ids we sent `unsubscribe` for: their `unsubscribed` ack (or failure) is swallowed. */
  const closing = new Set<number>();

  const sendFrame = (frame: unknown): void => {
    ws.send(JSON.stringify(frame));
  };

  const sendSubscribe = (entry: SubEntry): void => {
    const body: Record<string, unknown> = { query: entry.query };
    if (entry.alias !== undefined) body.alias = entry.alias;
    pending.push(entry);
    sendFrame({ subscribe: body });
    entry.handlers.onOpen?.();
  };

  /** Route one parsed server frame to its subscription (see the wire contract above). */
  const route = (event: SubscriptionEvent): void => {
    switch (event.kind) {
      case "subscribed": {
        // Acks answer subscribes in order: the oldest in-flight subscribe owns this ack.
        const entry = pending.shift();
        if (!entry) {
          handlers.onUnrouted?.(event);
          return;
        }
        entry.id = event.ack.id;
        if (entry.cancelled) {
          // Closed while in flight: release the server slot now that the id is known.
          closing.add(entry.id);
          sendFrame({ unsubscribe: { id: entry.id } });
          return;
        }
        active.set(entry.id, entry);
        entry.handlers.onEvent(event);
        return;
      }
      case "notification": {
        const entry = active.get(event.notification.id);
        if (entry) {
          entry.handlers.onEvent(event);
        } else if (!closing.has(event.notification.id)) {
          handlers.onUnrouted?.(event);
        }
        return;
      }
      case "unsubscribed": {
        // The ack for an `unsubscribe` we sent; anything else is unrouted.
        if (!closing.delete(event.id)) handlers.onUnrouted?.(event);
        return;
      }
      case "error": {
        const id = event.error.id;
        if (id !== undefined) {
          if (closing.delete(id)) return; // failure answering an unsubscribe we already reported closed
          const entry = active.get(id);
          if (entry) {
            // A terminating server-side error on a live subscription: event then close,
            // the same order the SSE transport reports. The socket stays up.
            active.delete(id);
            entry.handlers.onEvent(event);
            entry.handlers.onClose?.();
            return;
          }
        }
        // An id-less error while a subscribe is in flight is that subscribe's refusal
        // (the server answers each subscribe, in order, before reading the next frame).
        const refused = pending.shift();
        if (refused) {
          if (!refused.cancelled) {
            refused.handlers.onEvent(event);
            refused.handlers.onClose?.();
          }
          return;
        }
        handlers.onUnrouted?.(event);
        return;
      }
      default:
        handlers.onUnrouted?.(event);
    }
  };

  ws.onopen = () => {
    open = true;
    handlers.onOpen?.();
    for (const entry of queued.splice(0)) {
      if (!entry.cancelled) sendSubscribe(entry);
    }
  };

  ws.onmessage = (ev) => {
    // Binary frames are not part of the protocol (the server refuses them); text only.
    if (typeof ev.data !== "string") return;
    route(parseSubscriptionData(ev.data));
  };

  ws.onerror = () => {
    // A browser exposes NO detail here (deliberately — the handshake status is hidden);
    // the flag only informs the honest close message below.
    sawTransportError = true;
  };

  ws.onclose = (ev) => {
    if (finished) return;
    finished = true;
    const error = closedByClient
      ? undefined
      : abnormalCloseMessage(open, sawTransportError, ev);
    for (const entry of [...queued, ...pending, ...active.values()]) {
      if (!entry.cancelled) {
        // Mark cancelled so a later handle.close() cannot double-fire this onClose.
        entry.cancelled = true;
        entry.handlers.onClose?.(error);
      }
    }
    queued.length = 0;
    pending.length = 0;
    active.clear();
    closing.clear();
    handlers.onClose?.(error);
  };

  return {
    subscribe(query, subHandlers, subOptions = {}) {
      if (finished || closedByClient) {
        subHandlers.onClose?.("The subscription socket is closed.");
        return DEAD_HANDLE;
      }
      const a = (subOptions.alias ?? "").trim();
      const entry: SubEntry = {
        query,
        handlers: subHandlers,
        alias: a === "" ? undefined : a,
      };
      if (open) {
        sendSubscribe(entry);
      } else {
        queued.push(entry);
      }
      return {
        close: () => {
          if (entry.cancelled) return;
          entry.cancelled = true;
          if (entry.id !== undefined && active.delete(entry.id)) {
            closing.add(entry.id);
            if (!finished) sendFrame({ unsubscribe: { id: entry.id } });
          } else {
            // Still queued (drop it) or in flight (the ack-handler unsubscribes on arrival).
            const qi = queued.indexOf(entry);
            if (qi >= 0) queued.splice(qi, 1);
          }
          entry.handlers.onClose?.();
        },
      };
    },
    close() {
      if (closedByClient) return;
      closedByClient = true;
      try {
        ws.close();
      } catch {
        /* already closed */
      }
    },
  };
}

/**
 * The honest message for an abnormal socket close. A browser hides the WHY of a failed WS
 * handshake (no HTTP status reaches `onerror`/`onclose`), so a handshake-stage failure
 * lists the real possibilities instead of fabricating one; a post-open drop reports the
 * close code/reason when the server supplied them.
 */
function abnormalCloseMessage(
  wasOpen: boolean,
  sawTransportError: boolean,
  ev?: { code?: number; reason?: string },
): string {
  const detail =
    ev && typeof ev.code === "number" && ev.code !== 1005
      ? ` (close code ${ev.code}${ev.reason ? `: ${ev.reason}` : ""})`
      : "";
  if (!wasOpen) {
    return (
      `The subscription socket handshake failed${detail}. The browser hides the exact cause; the usual ones are: ` +
      `the server refused the upgrade with 401 (--auth-token-read is set — enter a valid Bearer token; it is offered as the bearer.<token> subprotocol), ` +
      `the server is not running or not reachable at this address, ` +
      `or an https:// page opening a plaintext ws:// socket (mixed content — use a wss:// endpoint).`
    );
  }
  return sawTransportError
    ? `The subscription socket was interrupted${detail}.`
    : `The subscription socket closed unexpectedly${detail}.`;
}
