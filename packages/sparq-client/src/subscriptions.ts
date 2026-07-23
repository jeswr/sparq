// [OPUS-4.8] sq-9ij6 — live SPARQL subscriptions client (the SSE transport of the
// server's `/subscriptions` surface).
//
// This is the framework-agnostic companion to the endpoint-mode HTTP client in
// `./endpoint.ts`. Where that one runs a one-shot query against a SPARQL 1.1 Protocol
// endpoint, this one SUBSCRIBES a SELECT to the server's SEPA-style subscriptions surface
// and streams the result DELTAS (added/removed bindings) as the dataset mutates — the one
// thing a static Pages site fundamentally cannot do on its own.
//
// It CONSUMES the existing `sparq-server` subscriptions API
// (`crates/sparq-server/src/subscriptions.rs`) — it changes nothing server-side and claims
// no security the server does not provide. It reuses the SAME connection + bearer posture
// the endpoint client (sq-2mke) established:
//   * the SAME `EndpointConfig` (endpoint URL + optional bearer token) and the SAME honest
//     `connectionSafetyWarnings` classifier — a live stream over plaintext to a non-loopback
//     host, or a token in the clear, is surfaced exactly as for a query, never bypassed;
//   * the bearer token is sent ONLY in the `Authorization: Bearer <token>` header — the same
//     channel the server's SSE GET read gate (`--auth-token-read`) validates (see
//     `crates/sparq-server/src/main.rs`: "The SSE GET takes the `Authorization: Bearer`
//     header like any GET").
//
// WHY THE SSE TRANSPORT (not WebSocket). The server exposes BOTH `/subscriptions` (a
// multiplexed WS) and `/subscriptions/sse` (a one-subscription-per-stream GET). We use SSE,
// consumed over `fetch` + a streaming `ReadableStream` reader rather than the browser's
// native `EventSource`. The reason is auth honesty: native `EventSource` CANNOT set an
// `Authorization` header, so it could only reach an UNAUTHENTICATED server — but a `fetch`
// stream sets the exact same `Authorization: Bearer` header a query uses, so the SAME read
// token works for both. The WS path's `bearer.<token>` subprotocol (`wsSubprotocols`) exists
// for browsers that cannot header a WS upgrade; the SSE GET has no such limitation, so the
// header is the honest, single auth channel here.
//
// No React, no Next.js, no DOM beyond the `fetch` / `URL` / `TextDecoder` / `AbortController`
// web-platform globals. No performance claim is made here (this repo's work box is
// non-canonical).

import type { SparqlResults, SparqlBinding } from "./index.js";
import { type EndpointConfig, parseEndpointUrl } from "./endpoint.js";

// ---------------------------------------------------------------------------
// SSE endpoint-URL derivation.
// ---------------------------------------------------------------------------

/**
 * Derive the `/subscriptions/sse` URL from the configured `/sparql` endpoint URL, carrying
 * the SELECT (and optional alias) as the query-string parameters the server's
 * `sse_endpoint` reads (`?query=<SELECT>[&alias=<x>]`).
 *
 * The server mounts the SSE route at the path `/subscriptions/sse` on the SAME origin as
 * `/sparql` (see `crates/sparq-server/src/http.rs` route table). We map the endpoint URL's
 * path conservatively: a trailing `/sparql` segment is rewritten to `/subscriptions/sse`
 * (the common `sparq-server` shape), otherwise `/subscriptions/sse` is resolved against the
 * endpoint's origin. Returns `null` when the endpoint URL is not a valid absolute http(s)
 * URL (the caller surfaces the same `invalid-url` safety error the query path does).
 */
export function buildSubscriptionUrl(
  config: EndpointConfig,
  query: string,
  alias?: string,
): string | null {
  const parsed = parseEndpointUrl(config.url);
  if (!parsed) return null;
  const sse = new URL(parsed.toString());
  // Rewrite a trailing `/sparql` to `/subscriptions/sse`; else mount it at the origin root.
  if (/\/sparql\/?$/.test(sse.pathname)) {
    sse.pathname = sse.pathname.replace(/\/sparql\/?$/, "/subscriptions/sse");
  } else {
    sse.pathname = "/subscriptions/sse";
  }
  sse.search = "";
  sse.searchParams.set("query", query);
  const a = (alias ?? "").trim();
  if (a !== "") sse.searchParams.set("alias", a);
  return sse.toString();
}

// ---------------------------------------------------------------------------
// The SEPA notification wire shapes (mirrors subscriptions.rs message builders).
// ---------------------------------------------------------------------------

/** The `{"subscribed":{"id",…}}` ack the server sends right after a successful subscribe. */
export interface SubscribedAck {
  id: number;
  alias?: string;
}

/**
 * One `{"notification":{…}}` frame: the SEPA-shaped diff. `addedResults` / `removedResults`
 * are both full SPARQL-JSON results objects (`head.vars` + `results.bindings`). `sequence`
 * is the per-subscription notification counter — 0 is the initial full snapshot (everything
 * in `addedResults`, `removedResults` empty), then 1, 2, … for each subsequent diff.
 */
export interface SubscriptionNotification {
  id: number;
  alias?: string;
  sequence: number;
  addedResults: SparqlResults;
  removedResults: SparqlResults;
}

/** A `{"error":{"message",…}}` frame — a terminating server error on the subscription. */
export interface SubscriptionError {
  message: string;
  id?: number;
  alias?: string;
}

/** A parsed subscription event, tagged by which envelope the server sent. */
export type SubscriptionEvent =
  | { kind: "subscribed"; ack: SubscribedAck }
  | { kind: "notification"; notification: SubscriptionNotification }
  /**
   * [FABLE-5] sq-140b — the `{"unsubscribed":{"id"}}` ack the WebSocket transport's
   * `unsubscribe` frame earns. The SSE transport never emits it (an SSE client unsubscribes
   * by closing the stream), so on that transport this kind simply never occurs.
   */
  | { kind: "unsubscribed"; id: number }
  | { kind: "error"; error: SubscriptionError }
  | { kind: "unknown"; raw: unknown };

/**
 * Parse one SSE `data:` JSON payload (the server emits one JSON object per `event:`/`data:`
 * frame) into a tagged {@link SubscriptionEvent}. The SSE `event:` NAME is advisory — the
 * envelope key (`subscribed` / `notification` / `unsubscribed` / `error`) is the source of
 * truth, exactly as the WebSocket transport speaks the same JSON (the WS transport in
 * `ws-subscriptions.ts` reuses this parser verbatim; `unsubscribed` is WS-only). A malformed /
 * unrecognised payload becomes an `unknown` event (the caller decides whether to surface it),
 * never a throw.
 */
export function parseSubscriptionData(data: string): SubscriptionEvent {
  let parsed: unknown;
  try {
    parsed = JSON.parse(data);
  } catch {
    return { kind: "unknown", raw: data };
  }
  if (typeof parsed !== "object" || parsed === null) {
    return { kind: "unknown", raw: parsed };
  }
  const obj = parsed as Record<string, unknown>;
  if (obj.subscribed && typeof obj.subscribed === "object") {
    const s = obj.subscribed as Record<string, unknown>;
    return {
      kind: "subscribed",
      ack: { id: Number(s.id), alias: typeof s.alias === "string" ? s.alias : undefined },
    };
  }
  if (obj.notification && typeof obj.notification === "object") {
    const n = obj.notification as Record<string, unknown>;
    return {
      kind: "notification",
      notification: {
        id: Number(n.id),
        sequence: Number(n.sequence),
        alias: typeof n.alias === "string" ? n.alias : undefined,
        addedResults: n.addedResults as SparqlResults,
        removedResults: n.removedResults as SparqlResults,
      },
    };
  }
  if (obj.unsubscribed && typeof obj.unsubscribed === "object") {
    const u = obj.unsubscribed as Record<string, unknown>;
    return { kind: "unsubscribed", id: Number(u.id) };
  }
  if (obj.error && typeof obj.error === "object") {
    const e = obj.error as Record<string, unknown>;
    return {
      kind: "error",
      error: {
        message: typeof e.message === "string" ? e.message : "subscription error",
        id: typeof e.id === "number" ? e.id : undefined,
        alias: typeof e.alias === "string" ? e.alias : undefined,
      },
    };
  }
  return { kind: "unknown", raw: parsed };
}

// ---------------------------------------------------------------------------
// The live result set — applying added/removed diffs against a row-keyed map.
// ---------------------------------------------------------------------------

/**
 * The canonical identity key of a solution row, matching the server's diff key: the binding
 * object serialised with its keys in sorted order. The server keys on
 * `serde_json::Value::to_string()` of the binding object (variables in `serde_json` map
 * order, which is sorted), so we reproduce that ordering here so a row added in one
 * notification and removed in a later one collapses to the SAME key.
 */
export function rowKey(binding: SparqlBinding): string {
  const keys = Object.keys(binding).sort();
  const ordered: Record<string, unknown> = {};
  for (const k of keys) ordered[k] = binding[k];
  return JSON.stringify(ordered);
}

/**
 * The live, accumulated result set of a subscription: the projected variables plus the
 * current rows keyed by their canonical {@link rowKey}. The view renders `vars` + the row
 * VALUES; the map de-duplicates exactly as the server's set-semantics diff does.
 */
export interface LiveResultSet {
  vars: string[];
  rows: Map<string, SparqlBinding>;
}

/** An empty live result set (before the sequence-0 snapshot arrives). */
export function emptyLiveResultSet(): LiveResultSet {
  return { vars: [], rows: new Map() };
}

/**
 * [OPUS-4.8] sq-9ij6 — apply one {@link SubscriptionNotification} diff to a {@link
 * LiveResultSet}, returning the NEW set (pure — the input is not mutated, so a React state
 * setter can compare identities). Removed rows are deleted first, then added rows inserted,
 * keyed by the canonical {@link rowKey} (so a re-added row replaces, and a removed-then-added
 * row nets correctly). The variable list is refreshed from the notification's `addedResults`
 * head when it carries one (the head is identical across a subscription's lifetime, but the
 * sequence-0 snapshot is where it first lands).
 *
 * Returns the counts of rows actually added / removed so the diff LOG can report the real
 * deltas (an added key already present, or a removed key absent, does not double-count).
 */
export function applyNotification(
  set: LiveResultSet,
  notification: SubscriptionNotification,
): { next: LiveResultSet; added: number; removed: number } {
  const rows = new Map(set.rows);
  let removed = 0;
  for (const binding of notification.removedResults?.results?.bindings ?? []) {
    if (rows.delete(rowKey(binding))) removed += 1;
  }
  let added = 0;
  for (const binding of notification.addedResults?.results?.bindings ?? []) {
    const key = rowKey(binding);
    if (!rows.has(key)) added += 1;
    rows.set(key, binding);
  }
  const headVars =
    notification.addedResults?.head?.vars ??
    notification.removedResults?.head?.vars ??
    set.vars;
  return { next: { vars: headVars.length > 0 ? headVars : set.vars, rows }, added, removed };
}

/** The live rows as a `SparqlResults` document, so the SAME results helpers render them. */
export function liveResults(set: LiveResultSet): SparqlResults {
  return {
    head: { vars: set.vars },
    results: { bindings: [...set.rows.values()] },
  };
}

// ---------------------------------------------------------------------------
// The fetch-based SSE reader (one subscription per stream).
// ---------------------------------------------------------------------------

/** The callbacks a {@link openSubscription} stream drives. */
export interface SubscriptionHandlers {
  /** A parsed event arrived (subscribed ack / notification diff / terminating error). */
  onEvent: (event: SubscriptionEvent) => void;
  /** The stream opened (the HTTP response is 200 and the body is flowing). */
  onOpen?: () => void;
  /**
   * The stream closed. `error` is set when it closed because of a transport failure or a
   * non-2xx response (with an honest, classified message); `undefined` on a clean end (the
   * caller aborted, or the server ended the stream after a terminating `error` event).
   */
  onClose?: (error?: string) => void;
}

/** A handle to an open subscription stream — call {@link close} to unsubscribe. */
export interface SubscriptionHandle {
  /** Abort the stream (drops the server-side subscription, releasing its slot). */
  close: () => void;
}

/**
 * Split a chunk of decoded SSE text into complete frames (separated by a blank line) plus the
 * trailing partial frame to carry into the next read. Each complete frame is the raw block of
 * `event:` / `data:` / `id:` lines. Exported for unit testing the framing.
 */
export function splitSseFrames(buffer: string): { frames: string[]; rest: string } {
  // SSE frames are separated by a blank line; tolerate both \n\n and \r\n\r\n.
  const normalised = buffer.replace(/\r\n/g, "\n");
  const parts = normalised.split("\n\n");
  const rest = parts.pop() ?? "";
  return { frames: parts.filter((p) => p.trim() !== ""), rest };
}

/**
 * Extract the `data:` payload from one SSE frame (joining multiple `data:` lines with a
 * newline, per the SSE spec). A keep-alive comment frame (lines starting with `:`) or a frame
 * with no `data:` line yields `null` — the caller skips it. Exported for unit testing.
 */
export function frameData(frame: string): string | null {
  const dataLines: string[] = [];
  for (const line of frame.split("\n")) {
    if (line.startsWith(":")) continue; // SSE comment / keep-alive ping
    if (line.startsWith("data:")) {
      // Per the SSE spec a single leading space after the colon is stripped.
      dataLines.push(line.slice(5).replace(/^ /, ""));
    }
  }
  return dataLines.length > 0 ? dataLines.join("\n") : null;
}

/** Add the `Authorization: Bearer <token>` header iff a non-empty token is configured. */
function authHeaders(token?: string): Record<string, string> {
  const headers: Record<string, string> = { Accept: "text/event-stream" };
  const t = (token ?? "").trim();
  if (t !== "") headers["Authorization"] = `Bearer ${t}`;
  return headers;
}

/**
 * [OPUS-4.8] sq-9ij6 — open a live subscription to the endpoint's `/subscriptions/sse` GET,
 * over `fetch` + a streaming `ReadableStream` reader, and drive `handlers` as events arrive.
 *
 * The bearer token (if any) is sent ONLY in the `Authorization: Bearer` header — the same
 * channel the server's SSE read gate validates, and the same posture the endpoint query
 * client uses. `fetch` (not native `EventSource`) is used precisely BECAUSE it can set that
 * header (native `EventSource` cannot, so it could never reach an authenticated server).
 *
 * Lifecycle is reported honestly: `onOpen` once the 200 stream is flowing; `onEvent` per
 * parsed frame; `onClose(error)` with an HONEST classified message on a non-2xx (401 → auth
 * required; 413 → result too large; 503 → capacity/timeout, retry; transport failure → the
 * CORS/reachability hint), or `onClose()` on a clean end (server stream ended, or the caller
 * aborted). A registration refusal is a non-2xx HTTP status BEFORE the stream opens (the SSE
 * endpoint sets a real status then, mirroring `/sparql`), so it surfaces as an `onClose`
 * error, never a silent dead stream.
 *
 * Returns a {@link SubscriptionHandle}; `close()` aborts the fetch, which drops the
 * server-side subscription (its `ConnSlots` `Drop` releases the global slot — no leak).
 */
export function openSubscription(
  config: EndpointConfig,
  query: string,
  handlers: SubscriptionHandlers,
  options: { alias?: string; fetchImpl?: typeof fetch } = {},
): SubscriptionHandle {
  const fetchImpl = options.fetchImpl ?? fetch;
  const controller = new AbortController();
  const url = buildSubscriptionUrl(config, query, options.alias);

  if (url === null) {
    // Invalid endpoint URL — fail closed, consistent with the query path's `invalid-url`.
    handlers.onClose?.(
      "Enter a valid absolute endpoint URL (http:// or https://) in the Connect panel before subscribing.",
    );
    return { close: () => {} };
  }

  void (async () => {
    let resp: Response;
    try {
      resp = await fetchImpl(url, {
        method: "GET",
        headers: authHeaders(config.token),
        signal: controller.signal,
      });
    } catch (e) {
      if (controller.signal.aborted) {
        handlers.onClose?.();
        return;
      }
      const base = e instanceof Error ? e.message : String(e);
      handlers.onClose?.(
        `Could not open the subscription stream (${base}). This is usually a CORS block (the server emits no CORS headers by default — opt this origin in with --cors-allow-origin), a refused connection (is the server running and reachable?), or a mixed-content block (HTTPS page → http endpoint).`,
      );
      return;
    }

    if (!resp.ok) {
      handlers.onClose?.(await subscriptionErrorMessage(resp));
      return;
    }
    if (!resp.body) {
      handlers.onClose?.("The subscription stream opened but carried no body.");
      return;
    }

    handlers.onOpen?.();

    const reader = resp.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const { frames, rest } = splitSseFrames(buffer);
        buffer = rest;
        for (const frame of frames) {
          const data = frameData(frame);
          if (data === null) continue; // keep-alive ping / dataless frame
          handlers.onEvent(parseSubscriptionData(data));
        }
      }
      handlers.onClose?.();
    } catch (e) {
      if (controller.signal.aborted) {
        handlers.onClose?.();
        return;
      }
      const base = e instanceof Error ? e.message : String(e);
      handlers.onClose?.(`The subscription stream was interrupted (${base}).`);
    }
  })();

  return {
    close: () => controller.abort(),
  };
}

/**
 * Turn a non-2xx subscription-registration response into an honest, classified message that
 * mirrors the server's SSE refusal contract (`crates/sparq-server/src/subscriptions.rs`
 * `sse_endpoint`): 401 → auth required; 400 → fix the SELECT (only SELECT is subscribable);
 * 413 → result too large for the server's row cap; 429/503 → transient capacity/timeout, safe
 * to retry; 5xx → server defect. The server's JSON error body (`{"error":"…"}`) is surfaced
 * verbatim when present.
 */
async function subscriptionErrorMessage(resp: Response): Promise<string> {
  let detail = "";
  try {
    const text = await resp.text();
    if (text) {
      try {
        const parsed = JSON.parse(text) as { error?: string };
        detail = parsed?.error ?? text;
      } catch {
        detail = text;
      }
    }
  } catch {
    /* body unreadable — fall through to the status-only message */
  }
  const prefix =
    resp.status === 401
      ? "Authentication required — the server gates this read surface behind a Bearer token (--auth-token-read). Enter a valid token in the Connect panel."
      : resp.status === 400
        ? "The subscription was refused (400) — only a SELECT query can be subscribed; check the query form."
        : resp.status === 413
          ? "The subscription was refused (413) — the initial result exceeds the server's max-results row cap."
          : resp.status === 429 || resp.status === 503
            ? `Server temporarily unavailable (${resp.status}) — the subscription slot is exhausted or the query timed out; transient, safe to retry.`
            : resp.status >= 500
              ? `Server error (${resp.status}).`
              : `Subscription refused (${resp.status}).`;
  return detail ? `${prefix} ${detail}` : prefix;
}
