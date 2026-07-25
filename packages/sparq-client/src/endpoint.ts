// [OPUS-4.8] sq-2mke — SPARQL 1.1 Protocol HTTP client (endpoint mode).
//
// This is the framework-agnostic client that lets the SAME editor run against any
// running SPARQL 1.1 Protocol endpoint (a `sparq-server`, or any conformant endpoint)
// instead of the in-tab WASM store. It is the companion to the WASM `Store` surface in
// `./index.ts`: where that one executes locally with no network, this one speaks the
// wire protocol over `fetch`.
//
// It is built to CONSUME the existing `sparq-server` HTTP API
// (`crates/sparq-server/src/http.rs`) — it changes nothing server-side and claims no
// security the server does not provide. It honours the server's security posture
// (`crates/sparq-server/README.md`) as connection-safety UX: it surfaces honest warnings
// (a bearer token over plaintext to a non-loopback host is exposed in transit; an empty
// SERVICE allowlist on the server means federation is refused) but NEVER bypasses any
// gate. The bearer token is sent ONLY in the `Authorization` header (and, for the WS
// handshake that browsers cannot set headers on, the `Sec-WebSocket-Protocol:
// bearer.<token>` subprotocol the server's `ws_auth_gate` validates) — never echoed,
// never logged.
//
// No React, no Next.js, no DOM beyond the `fetch` / `URL` / `WebSocket` web platform
// globals. No performance claim is made here (this repo's work box is non-canonical).

import type { SparqlResults } from "./index.js";

// ---------------------------------------------------------------------------
// Content types — exactly what the server's `negotiate.rs` understands.
// ---------------------------------------------------------------------------

/** SPARQL 1.1 Protocol media types (mirrors `crates/sparq-server/src/negotiate.rs`). */
export const SPARQL_RESULTS_JSON = "application/sparql-results+json";
/** RDF graph media type the server returns for CONSTRUCT / DESCRIBE / GSP reads. */
export const SPARQL_GRAPH_NTRIPLES = "application/n-triples";
/** Direct-POST query Content-Type (SPARQL 1.1 Protocol §2.1.3). */
const SPARQL_QUERY_CT = "application/sparql-query";
/** Direct-POST update Content-Type (SPARQL 1.1 Protocol §2.2.2). */
const SPARQL_UPDATE_CT = "application/sparql-update";

// ---------------------------------------------------------------------------
// Connection config + the honest safety classifier.
// ---------------------------------------------------------------------------

/** A configured connection to a SPARQL 1.1 Protocol endpoint. */
export interface EndpointConfig {
  /** The endpoint URL, e.g. `http://127.0.0.1:3030/sparql`. */
  url: string;
  /**
   * Optional Bearer token. The server's write gate (`--auth-token`) and read gate
   * (`--auth-token-read`) are one shared secret with no per-user identity; this is sent
   * verbatim as `Authorization: Bearer <token>`. Empty / whitespace-only is treated as
   * "no token".
   */
  token?: string;
}

/** Severity of a {@link SafetyWarning}: a hard block vs an advisory the user can proceed past. */
export type SafetyLevel = "error" | "warning" | "info";

/** A stable identifier for each safety finding (so the UI / tests key on it, not prose). */
export type SafetyCode =
  | "invalid-url"
  | "token-over-plaintext"
  | "mixed-content"
  | "non-loopback-no-tls"
  | "service-allowlist"
  | "cors-required";

/** One honest connection-safety finding surfaced in the Connect panel UI. */
export interface SafetyWarning {
  level: SafetyLevel;
  code: SafetyCode;
  message: string;
}

/** Is this hostname a loopback address (the server's safe default bind)? */
export function isLoopbackHost(hostname: string): boolean {
  const h = hostname.toLowerCase();
  if (h === "localhost" || h === "127.0.0.1" || h === "::1" || h === "[::1]") {
    return true;
  }
  // 127.0.0.0/8 is entirely loopback.
  if (/^127\.\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(h)) return true;
  return false;
}

/** Parse the endpoint URL, or `null` if it is not a valid absolute http(s) URL. */
export function parseEndpointUrl(url: string): URL | null {
  const trimmed = url.trim();
  if (trimmed === "") return null;
  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return null;
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
  return parsed;
}

/**
 * The current page origin's protocol, if the client runs in a browser (`"https:"` /
 * `"http:"`), else `undefined` (Node / SSR type-check). Used only to detect the
 * mixed-content block a browser enforces on an HTTPS page fetching an HTTP endpoint.
 */
function pageProtocol(): string | undefined {
  return typeof window !== "undefined" ? window.location?.protocol : undefined;
}

/**
 * [OPUS-4.8] sq-2mke — the honest connection-safety classifier. Pure (no network), so it
 * is unit-testable and drives the Connect-panel warnings. It NEVER blocks a query by
 * itself (that is the UI's choice); it returns findings that faithfully describe the
 * server's posture (`crates/sparq-server/README.md`) and the browser's transport rules:
 *
 * - `invalid-url` (error): the URL is not a valid absolute http(s) URL.
 * - `mixed-content` (error): an HTTPS page cannot `fetch` a plaintext-`http:` endpoint —
 *   the browser blocks it before any request. A hard block, not advice.
 * - `token-over-plaintext` (warning): a Bearer token sent over `http:` (not `https:`) to a
 *   NON-loopback host travels in cleartext and can be intercepted. (Loopback `http:` stays
 *   on the machine, so a token there is only an info note, below.)
 * - `non-loopback-no-tls` (warning): talking plaintext `http:` to a non-loopback host at
 *   all — even without a token the query + results are in cleartext, and the server refuses
 *   a non-loopback bind unless `--allow-remote` (so reaching it means it was deliberately
 *   exposed).
 * - `service-allowlist` (info): a reminder that the server's SERVICE federation is OFF by
 *   default and, even with `--features service`, default-DENY — a `SERVICE` clause is
 *   refused before any socket unless the operator set an egress allowlist. We cannot see
 *   the server's config from the client, so this is surfaced as an honest "may be refused".
 * - `cors-required` (info): the server emits NO CORS headers by default, so a cross-origin
 *   browser fetch is blocked until the operator opts an origin in (`--cors-allow-origin`).
 *   Surfaced when the endpoint origin differs from the page origin.
 */
export function connectionSafetyWarnings(config: EndpointConfig): SafetyWarning[] {
  const warnings: SafetyWarning[] = [];
  const parsed = parseEndpointUrl(config.url);
  if (!parsed) {
    warnings.push({
      level: "error",
      code: "invalid-url",
      message:
        "Enter a valid absolute endpoint URL (http:// or https://), e.g. http://127.0.0.1:3030/sparql.",
    });
    return warnings;
  }

  const hasToken = (config.token ?? "").trim() !== "";
  const isTls = parsed.protocol === "https:";
  const loopback = isLoopbackHost(parsed.hostname);
  const page = pageProtocol();

  // A browser blocks an HTTPS page from fetching a plaintext http: endpoint outright.
  if (page === "https:" && parsed.protocol === "http:") {
    warnings.push({
      level: "error",
      code: "mixed-content",
      message:
        "This page is served over HTTPS, so the browser will block a request to a plaintext http:// endpoint (mixed content). Serve the endpoint over HTTPS, or open this editor over http (e.g. a local dev server).",
    });
  }

  if (!isTls && !loopback) {
    if (hasToken) {
      warnings.push({
        level: "warning",
        code: "token-over-plaintext",
        message:
          "You are sending a Bearer token over plaintext http:// to a non-loopback host — it travels in cleartext and can be intercepted. Use https:// for any remote, authenticated endpoint. The server's Bearer gate is a single shared secret with no per-user identity (sparq-server README, Security posture).",
      });
    }
    warnings.push({
      level: "warning",
      code: "non-loopback-no-tls",
      message:
        "Plaintext http:// to a non-loopback host: the query and results travel in cleartext. The server refuses a non-loopback bind unless it was started with --allow-remote, so this endpoint was deliberately exposed.",
    });
  } else if (!isTls && loopback && hasToken) {
    // Loopback http: keeps the token on the machine — not a transit risk, just a note.
    warnings.push({
      level: "info",
      code: "token-over-plaintext",
      message:
        "Bearer token sent over loopback http:// — it stays on this machine (no transit exposure). The token is sent only in the Authorization header, never logged by this client.",
    });
  }

  // Cross-origin browser fetch needs server CORS, which is OFF by default.
  if (page && typeof window !== "undefined") {
    const pageOrigin = window.location?.origin;
    if (pageOrigin && pageOrigin !== parsed.origin) {
      warnings.push({
        level: "info",
        code: "cors-required",
        message:
          "The endpoint is a different origin from this page. sparq-server emits NO CORS headers by default, so the browser will block this cross-origin request until the operator opts this origin in (--cors-allow-origin). A same-origin or CORS-enabled endpoint is required.",
      });
    }
  }

  warnings.push({
    level: "info",
    code: "service-allowlist",
    message:
      "If your query uses SERVICE (federation), note the server's SERVICE egress is OFF by default and, even with the service feature, default-DENY — a SERVICE clause is refused before any network call unless the operator set an egress allowlist (sparq-server README).",
  });

  return warnings;
}

/** Does any warning block the request outright (a hard error, not an advisory)? */
export function hasBlockingWarning(warnings: SafetyWarning[]): boolean {
  return warnings.some((w) => w.level === "error");
}

// ---------------------------------------------------------------------------
// The query form classifier (shared shape with the WASM REPL).
// ---------------------------------------------------------------------------

/** The SPARQL form the endpoint client dispatches on. */
export type EndpointForm =
  | "select"
  | "ask"
  | "graph" // CONSTRUCT / DESCRIBE
  | "update";

/** The result of running a query against the endpoint, tagged by form. */
export type EndpointResult =
  | { kind: "select"; results: SparqlResults; status: number }
  | { kind: "boolean"; value: boolean; status: number }
  | { kind: "graph"; ntriples: string; status: number }
  | { kind: "update"; status: number };

/**
 * Classify a SPARQL string into the protocol operation the endpoint client uses. This
 * folds to the four wire shapes: a query is a SELECT/ASK (SPARQL-results body), a
 * CONSTRUCT/DESCRIBE (RDF graph body), or an update (no body, a 204).
 *
 * To find the operative keyword past a PREFIX/BASE prologue, the noise that would
 * otherwise yield spurious word tokens is stripped FIRST — line comments (`#…`), IRIs
 * (`<…>`), and string literals (`"…"` / `'…'`) — so a `PREFIX ex: <http://example.org/>`
 * declaration contributes only the `PREFIX` and `ex` tokens, not `http`/`example`/`org`.
 * A whole `PREFIX <label>` declaration is then skipped. The default is `select` (the
 * protocol's query operation); the server's parser remains the source of truth on
 * validity, so an unrecognised lead keyword is treated as a query, never silently an
 * update (which would mis-route a read to the write path).
 */
export function classifyEndpointForm(sparql: string): EndpointForm {
  const stripped = sparql
    .replace(/#[^\n]*/g, " ") // line comments
    .replace(/<[^>]*>/g, " ") // IRIs
    .replace(/"(?:\\.|[^"\\])*"/g, " ") // double-quoted literals
    .replace(/'(?:\\.|[^'\\])*'/g, " "); // single-quoted literals
  const tokens = stripped.match(/[A-Za-z]+/g) ?? [];
  for (let i = 0; i < tokens.length; i++) {
    const kw = tokens[i].toUpperCase();
    if (kw === "PREFIX") {
      i += 1; // skip the prefix label token (the IRI was stripped above)
      continue;
    }
    if (kw === "BASE") continue;
    switch (kw) {
      case "SELECT":
        return "select";
      case "ASK":
        return "ask";
      case "CONSTRUCT":
      case "DESCRIBE":
        return "graph";
      case "INSERT":
      case "DELETE":
      case "LOAD":
      case "CLEAR":
      case "CREATE":
      case "DROP":
      case "COPY":
      case "MOVE":
      case "ADD":
      case "WITH":
        return "update";
      default:
        return "select";
    }
  }
  return "select";
}

// ---------------------------------------------------------------------------
// Request building (pure — so it is unit-testable without a network).
// ---------------------------------------------------------------------------

/** A prepared `fetch` request: the resolved URL + the RequestInit. */
export interface PreparedRequest {
  url: string;
  init: RequestInit;
}

/** Add the `Authorization: Bearer <token>` header iff a non-empty token is configured. */
function withAuth(headers: Record<string, string>, token?: string): Record<string, string> {
  const t = (token ?? "").trim();
  if (t !== "") headers["Authorization"] = `Bearer ${t}`;
  return headers;
}

/**
 * Build the `fetch` request for a SPARQL operation against the endpoint. Reads
 * (SELECT/ASK/CONSTRUCT/DESCRIBE) use a direct `application/sparql-query` POST — the most
 * robust form (no URL-length limit, no query-string PII), accepted by the server's
 * `sparql_endpoint` POST path and the wider SPARQL 1.1 Protocol. Updates use a direct
 * `application/sparql-update` POST (the server's write path, gated by `--auth-token`).
 * The `Accept` header asks for the precise media type per form so content negotiation is
 * deterministic.
 */
export function buildSparqlRequest(
  config: EndpointConfig,
  sparql: string,
  form: EndpointForm,
): PreparedRequest {
  if (form === "update") {
    return {
      url: config.url,
      init: {
        method: "POST",
        headers: withAuth({ "Content-Type": SPARQL_UPDATE_CT }, config.token),
        body: sparql,
      },
    };
  }
  const accept = form === "graph" ? SPARQL_GRAPH_NTRIPLES : SPARQL_RESULTS_JSON;
  return {
    url: config.url,
    init: {
      method: "POST",
      headers: withAuth(
        { "Content-Type": SPARQL_QUERY_CT, Accept: accept },
        config.token,
      ),
      body: sparql,
    },
  };
}

/**
 * Build the WebSocket subprotocols for a `/subscriptions` handshake: the
 * `bearer.<token>` subprotocol the server's `ws_auth_gate` accepts (browsers cannot set
 * an `Authorization` header on a WS upgrade). Returns `[]` when no token is configured.
 * The consumer is the multiplexed WebSocket subscriptions transport (sq-140b,
 * `ws-subscriptions.ts` `openSubscriptionSocket`); the helper lives here so the auth
 * posture stays derived in ONE place, next to the classifier it belongs with.
 */
export function wsSubprotocols(token?: string): string[] {
  const t = (token ?? "").trim();
  return t === "" ? [] : [`bearer.${t}`];
}

// ---------------------------------------------------------------------------
// Execution (the single network path).
// ---------------------------------------------------------------------------

/** A structured failure from an endpoint request (HTTP error, or a transport failure). */
export class EndpointError extends Error {
  readonly status: number | null;
  constructor(message: string, status: number | null) {
    super(message);
    this.name = "EndpointError";
    this.status = status;
  }
}

/**
 * Turn a non-2xx response into an {@link EndpointError} with an honest, classified message
 * that mirrors the server's status contract (`crates/sparq-server` `status_contract`):
 * 401 → auth required; 4xx → fix the request; 429/503 → transient, retry; 5xx → server
 * defect. The server's error body is a generic `{"error":"…"}` class (it never leaks
 * internals), so we surface the class verbatim when present.
 */
async function toEndpointError(resp: Response): Promise<EndpointError> {
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
      ? "Authentication required — the server gates this operation behind a Bearer token. Enter a valid token in the Connect panel."
      : resp.status === 429 || resp.status === 503
        ? `Server temporarily unavailable (${resp.status}) — transient, safe to retry.`
        : resp.status >= 500
          ? `Server error (${resp.status}).`
          : `Request rejected (${resp.status}).`;
  const message = detail ? `${prefix} ${detail}` : prefix;
  return new EndpointError(message, resp.status);
}

/**
 * Run a SPARQL operation against the configured endpoint over `fetch`. Classifies the
 * form, builds the request, sends it, and parses the response per form:
 * - SELECT → `{ kind: "select", results }` (parses the SPARQL-results JSON; an ASK that
 *   was classified as SELECT still surfaces as a boolean when the body carries one).
 * - ASK → `{ kind: "boolean", value }`.
 * - CONSTRUCT/DESCRIBE → `{ kind: "graph", ntriples }` (the N-Triples body).
 * - Update → `{ kind: "update" }` on the server's `204`.
 *
 * Throws an {@link EndpointError} on any non-2xx (with the server's error class) or a
 * transport failure (CORS block, DNS, refused connection) with an honest hint. The bearer
 * token is sent only in the `Authorization` header; it is never logged here.
 */
export async function runEndpointQuery(
  config: EndpointConfig,
  sparql: string,
  fetchImpl: typeof fetch = fetch,
): Promise<EndpointResult> {
  const form = classifyEndpointForm(sparql);
  const { url, init } = buildSparqlRequest(config, sparql, form);

  let resp: Response;
  try {
    resp = await fetchImpl(url, init);
  } catch (e) {
    // A browser surfaces a CORS block / network failure as an opaque TypeError; give an
    // honest hint rather than a bare "Failed to fetch".
    const base = e instanceof Error ? e.message : String(e);
    throw new EndpointError(
      `Could not reach the endpoint (${base}). This is usually a CORS block (the server emits no CORS headers by default — opt this origin in with --cors-allow-origin), a refused connection (is the server running and bound to a reachable address?), or a mixed-content block (HTTPS page → http endpoint).`,
      null,
    );
  }

  if (!resp.ok) {
    throw await toEndpointError(resp);
  }

  if (form === "update") {
    return { kind: "update", status: resp.status };
  }
  if (form === "graph") {
    const ntriples = (await resp.text()).trim();
    return { kind: "graph", ntriples, status: resp.status };
  }
  // SELECT / ASK — a SPARQL-results JSON document.
  const text = await resp.text();
  const parsed = JSON.parse(text) as SparqlResults;
  if (typeof parsed.boolean === "boolean") {
    return { kind: "boolean", value: parsed.boolean, status: resp.status };
  }
  return { kind: "select", results: parsed, status: resp.status };
}
