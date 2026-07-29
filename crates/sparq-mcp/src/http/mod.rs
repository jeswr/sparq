//! The MCP **Streamable HTTP** transport — the remote/multiplexed counterpart to the
//! `stdio` transport (feature `http`, OFF by default).
//!
//! [SONNET-4.6] (sq-2c0f0) Where `stdio` serves exactly one client over one pipe pair,
//! this serves **many concurrent clients over one TCP listener**, each holding its own
//! `Mcp-Session-Id` and its own Server-Sent-Events stream, all multiplexed onto the one
//! shared [`McpServer`] (one dataset, one lock). The MCP spec calls this shape
//! *Streamable HTTP*; it is a single endpoint (`/mcp` by default) that answers:
//!
//! | request | behaviour |
//! | --- | --- |
//! | `POST` | one JSON-RPC message (object or batch) in, one JSON response out — or `202 Accepted` with no body when the message was nothing but notifications |
//! | `GET` | opens the session's SSE stream (`text/event-stream`) for **server→client** messages, with `Last-Event-ID` resumption |
//! | `DELETE` | terminates the session (`204`) |
//! | `OPTIONS` | CORS preflight (`204`). A request carrying a *disallowed* `Origin` never reaches it — the origin check runs first, for every method |
//!
//! # What this is not
//!
//! It is a **transport**, not a security boundary. `sparq-mcp` has no authentication and
//! this feature adds none: anyone who can open a TCP connection to the listener gets
//! exactly the access the [`McpServer`] was configured with, and a session id is a
//! *correlation handle*, not a credential. Two mitigations are implemented and are worth
//! naming precisely, because they are the whole of what is on offer:
//!
//! - **`Origin` validation** ([`HttpConfig::allowed_origins`]) — the DNS-rebinding
//!   defence the MCP spec requires. The default list is **empty**, which refuses every
//!   request that carries an `Origin` header at all. A browser page therefore cannot
//!   reach the server until the operator names its origin; a non-browser MCP client
//!   sends no `Origin` and is unaffected.
//! - **Bind to loopback.** [`serve_http`] takes an already-bound [`TcpListener`], so the
//!   bind address is the caller's decision — bind `127.0.0.1:0`, not `0.0.0.0`, unless
//!   the network path is itself trusted.
//!
//! Neither of those is authentication. Do not expose this listener to a network you do
//! not control.
//!
//! # Concurrency
//!
//! One thread per connection — capped by [`HttpConfig::max_connections`], because a
//! connection costs a thread from the moment it is accepted, long before it has a session
//! — and one `Mutex<McpServer>` shared by all of them, so tool calls from different
//! sessions are **serialised** against the dataset. That is the honest description: the
//! transport is concurrent, the engine access underneath it is not. A long `query` blocks
//! other sessions for its duration — which is what
//! [`crate::ServerConfig::query_timeout_secs`] bounds.

mod session;
mod wire;

use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;

use crate::server::{McpServer, SUPPORTED_PROTOCOL_VERSIONS};
use session::{Poll, SessionRegistry, StreamError};
use wire::{HttpRequest, HttpResponse, WireError};

/// How long a connection may take to deliver its request head + body before the
/// transport gives up on it, so a slow-loris peer cannot hold a thread indefinitely.
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a single SSE write may block before the stream is abandoned as dead.
const STREAM_WRITE_TIMEOUT: Duration = Duration::from_secs(30);
/// How long the accept loop will spend telling an over-cap peer `503` before dropping it.
/// Short on purpose: the refusal is written on the accept thread (spawning a thread for it
/// would reintroduce the very unbounded growth the cap exists to stop), so a peer that
/// never reads must not be able to stall accepting for long.
const REFUSAL_WRITE_TIMEOUT: Duration = Duration::from_secs(1);

/// Configuration for the Streamable HTTP transport.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// The single MCP endpoint path. Any other path is `404`. Default `/mcp`.
    pub endpoint: String,
    /// Origins allowed to talk to this server, matched **exactly** against the request's
    /// `Origin` header. **Empty by default**, which refuses every request that carries an
    /// `Origin` at all — the DNS-rebinding defence the MCP spec requires. A non-browser
    /// client sends no `Origin` and is always allowed; add an entry only to admit a
    /// specific browser origin (e.g. `http://localhost:3000`).
    pub allowed_origins: Vec<String>,
    /// Largest accepted request body. A larger `Content-Length` is refused with `413`
    /// before any body byte is read. Default 4 MiB.
    pub max_body_bytes: usize,
    /// How long an idle SSE stream waits before emitting a keepalive comment.
    /// Default 15 s.
    pub sse_keepalive: Duration,
    /// How many already-delivered events each session keeps for `Last-Event-ID`
    /// resumption. `0` disables resumption. Default 64.
    pub replay_buffer: usize,
    /// Largest number of concurrent sessions. A handshake past this answers `503` and
    /// mints nothing. `0` removes the cap. Default 256.
    ///
    /// This exists because nothing authenticates a client: without a cap, anyone who can
    /// reach the listener can hold memory by POSTing `initialize` in a loop. It is a
    /// blunt ceiling, not a fairness quota — exactly like
    /// [`crate::ServerConfig::max_rows`].
    pub max_sessions: usize,
    /// Largest number of connections being served at once. An accepted connection past
    /// this is answered `503` and closed on the accept thread without spawning a worker.
    /// `0` removes the cap. Default 512.
    ///
    /// [`Self::max_sessions`] does **not** cover this: a connection costs a thread and a
    /// read buffer from the moment it is accepted, and a peer that never sends a complete
    /// request never mints a session at all — so without this cap a slow-loris peer can
    /// hold one thread per socket for the whole request read timeout with the session ceiling
    /// untouched. This is the *pre-session* half of the same blunt hedge.
    pub max_connections: usize,
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            endpoint: "/mcp".to_string(),
            // Security default: no browser origin is trusted until one is named.
            allowed_origins: Vec::new(),
            max_body_bytes: 4 * 1024 * 1024,
            sse_keepalive: Duration::from_secs(15),
            replay_buffer: 64,
            max_sessions: 256,
            max_connections: 512,
        }
    }
}

/// The Streamable HTTP transport: a shared [`McpServer`], the [`HttpConfig`], and the
/// live session registry.
///
/// Build one, wrap it in an [`Arc`], and hand it to [`serve_http`]. The same handle is
/// the embedder's seam for **server→client** messages: [`HttpTransport::notify`] and
/// [`HttpTransport::broadcast`] queue a raw JSON-RPC message onto a session's SSE stream.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::net::TcpListener;
/// use std::sync::{Arc, Mutex};
/// use sparq_core::Graph;
/// use sparq_mcp::{McpServer, http::{HttpConfig, HttpTransport, serve_http}};
///
/// let graph = Graph::load_str("<http://ex/a> <http://ex/p> <http://ex/b> .", "ntriples")?;
/// let server = Arc::new(Mutex::new(McpServer::new(graph)));
/// let transport = Arc::new(HttpTransport::new(server, HttpConfig::default()));
/// // Loopback, not 0.0.0.0 — there is no authentication.
/// serve_http(TcpListener::bind("127.0.0.1:0")?, transport)?;
/// # Ok(()) }
/// ```
pub struct HttpTransport {
    server: Arc<Mutex<McpServer>>,
    config: HttpConfig,
    sessions: SessionRegistry,
    connections: Arc<AtomicUsize>,
}

/// A live-connection slot, released on **every** exit path of the connection thread —
/// a normal return, an early `return` from a dead socket, or an unwinding panic — because
/// the release is a `Drop`, not a statement someone has to remember to write.
struct ConnectionPermit(Arc<AtomicUsize>);

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// What routing decided to do with one request.
enum Outcome {
    /// Write this response and close.
    Respond(HttpResponse),
    /// Write this head, then run the session's SSE stream.
    Stream {
        head: HttpResponse,
        session: String,
        replay: Vec<(u64, String)>,
    },
}

impl HttpTransport {
    /// Build a transport over a shared [`McpServer`].
    pub fn new(server: Arc<Mutex<McpServer>>, config: HttpConfig) -> Self {
        let replay = config.replay_buffer;
        HttpTransport {
            server,
            config,
            sessions: SessionRegistry::new(replay),
            connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// The configuration this transport was built with.
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// The number of live sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Whether `session` is live.
    pub fn has_session(&self, session: &str) -> bool {
        self.sessions.contains(session)
    }

    /// The number of connections currently being served — the quantity
    /// [`HttpConfig::max_connections`] caps. Counts sockets, not sessions: a peer that
    /// has not (or never will) complete a request is in here and in no session.
    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::Acquire)
    }

    /// Claim a connection slot, or `None` when the cap is already reached.
    ///
    /// The claim is a compare-and-swap rather than a `fetch_add` + check, so a burst of
    /// simultaneous accepts cannot momentarily overshoot the cap.
    fn acquire_connection(&self) -> Option<ConnectionPermit> {
        let cap = self.config.max_connections;
        self.connections
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |live| {
                (cap == 0 || live < cap).then_some(live + 1)
            })
            .ok()
            .map(|_| ConnectionPermit(Arc::clone(&self.connections)))
    }

    /// Queue one raw JSON-RPC message for delivery on `session`'s SSE stream. Returns
    /// `false` if there is no such session. The message is queued whether or not a
    /// stream is currently open, so a client that reconnects still receives it.
    ///
    /// `message` is written to the wire verbatim — the caller owns its JSON-RPC
    /// well-formedness.
    pub fn notify(&self, session: &str, message: &str) -> bool {
        self.sessions.enqueue(session, message)
    }

    /// Queue `message` for every live session; returns how many received it.
    pub fn broadcast(&self, message: &str) -> usize {
        self.sessions.broadcast(message)
    }

    /// Terminate `session` server-side (as a client's `DELETE` would). Its open stream,
    /// if any, ends. Returns `false` if it was not live.
    pub fn end_session(&self, session: &str) -> bool {
        self.sessions.remove(session)
    }

    /// Route one parsed request. Split out from the socket handling so the whole
    /// protocol decision surface — origin policy, session lifecycle, status codes — is
    /// unit-testable without opening a port.
    fn route(&self, request: &HttpRequest) -> Outcome {
        // 1. Origin first: a disallowed origin never reaches routing, let alone dispatch.
        let origin = request.header("origin");
        let origin_allowed =
            |value: &str| self.config.allowed_origins.iter().any(|a| a == value);
        if origin.is_some_and(|value| !origin_allowed(value)) {
            return Outcome::Respond(HttpResponse::text(
                403,
                "origin not allowed; add it to HttpConfig::allowed_origins\n",
            ));
        }
        if request.path != self.config.endpoint {
            return Outcome::Respond(HttpResponse::text(404, "no MCP endpoint here\n"));
        }

        let mut outcome = match request.method.as_str() {
            "POST" => self.route_post(request),
            "GET" => self.route_get(request),
            "DELETE" => self.route_delete(request),
            // A preflight for an origin that already passed the check above.
            "OPTIONS" => Outcome::Respond(
                HttpResponse::new(204)
                    .header("access-control-allow-methods", "GET, POST, DELETE, OPTIONS")
                    .header(
                        "access-control-allow-headers",
                        "content-type, mcp-session-id, mcp-protocol-version, last-event-id",
                    )
                    .header("access-control-expose-headers", "mcp-session-id")
                    .header("access-control-max-age", "600"),
            ),
            _ => Outcome::Respond(
                HttpResponse::text(405, "use POST, GET, DELETE or OPTIONS\n")
                    .header("allow", "GET, POST, DELETE, OPTIONS"),
            ),
        };

        // 2. Echo the (already validated) origin so a browser client can read the reply.
        if let Some(origin) = origin {
            let head = match &mut outcome {
                Outcome::Respond(response) => response,
                Outcome::Stream { head, .. } => head,
            };
            head.headers
                .push(("access-control-allow-origin".to_string(), origin.to_string()));
            head.headers.push((
                "access-control-expose-headers".to_string(),
                "mcp-session-id".to_string(),
            ));
        }
        outcome
    }

    /// `POST /mcp` — the request/response half of the transport.
    fn route_post(&self, request: &HttpRequest) -> Outcome {
        // We answer with JSON, never by upgrading the POST to an SSE stream, so the
        // client must be willing to read `application/json`.
        if request
            .header("accept")
            .is_some_and(|accept| !wire::accepts(accept, "application/json"))
        {
            return respond(406, "this endpoint answers a POST with application/json\n");
        }
        let content_type = request.header("content-type").unwrap_or("");
        if !wire::content_type_is(content_type, "application/json") {
            return respond(415, "send Content-Type: application/json\n");
        }
        let Ok(body) = std::str::from_utf8(&request.body) else {
            return respond(400, "request body is not UTF-8\n");
        };
        if body.trim().is_empty() {
            return respond(400, "empty request body\n");
        }
        if request
            .header("mcp-protocol-version")
            .is_some_and(|version| !SUPPORTED_PROTOCOL_VERSIONS.contains(&version))
        {
            return respond(400, "unsupported MCP-Protocol-Version\n");
        }

        // Session lifecycle. The handshake is the one thing a client may send without a
        // session id, so a sessionless body is classified in full *before* dispatch: a
        // batch is admitted only if EVERY element belongs to the handshake. Testing
        // "contains an initialize" instead would let `[initialize, tools/call]` run the
        // tool call pre-session, since `handle_message` dispatches the whole batch.
        let pre_session = match request.header("mcp-session-id") {
            Some(id) if self.sessions.contains(id) => None,
            Some(_) => return respond(404, "unknown or terminated session\n"),
            None => match classify_pre_session(body) {
                PreSession::Refused => {
                    return respond(
                        400,
                        "Mcp-Session-Id header required for anything but an initialize handshake\n",
                    )
                }
                admitted => Some(admitted),
            },
        };

        let reply = {
            let Ok(mut server) = self.server.lock() else {
                return respond(500, "server state is poisoned\n");
            };
            server.handle_message(body)
        };

        // Nothing but notifications: MCP prescribes 202 with no body.
        let Some(reply) = reply else {
            return Outcome::Respond(HttpResponse::new(202));
        };

        let mut response = HttpResponse::json(reply);
        // A successful handshake mints one. `Handshake` is only reached for a sessionless
        // body carrying exactly one `initialize` *request*, and the classification admits
        // nothing else that can produce a response — so a reply here is that handshake's.
        if matches!(pre_session, Some(PreSession::Handshake)) {
            match new_session_id() {
                // Fail closed: a session id the OS could not make random is not a
                // session id, and inventing a predictable one would be worse than 500.
                None => return respond(500, "no OS entropy available for a session id\n"),
                Some(id) => {
                    if !self.sessions.create(&id, self.config.max_sessions) {
                        return respond(503, "session limit reached\n");
                    }
                    response = response.header("mcp-session-id", id);
                }
            }
        }
        Outcome::Respond(response)
    }

    /// `GET /mcp` — open the session's server→client SSE stream.
    fn route_get(&self, request: &HttpRequest) -> Outcome {
        let accept = request.header("accept").unwrap_or("text/event-stream");
        if !wire::accepts(accept, "text/event-stream") {
            return respond(406, "a GET on this endpoint is an SSE stream\n");
        }
        let Some(session) = request.header("mcp-session-id") else {
            return respond(400, "Mcp-Session-Id header required\n");
        };
        let last_event_id = request
            .header("last-event-id")
            .and_then(|raw| raw.trim().parse::<u64>().ok());
        match self.sessions.open_stream(session, last_event_id) {
            Ok(replay) => Outcome::Stream {
                head: HttpResponse::new(200)
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-store")
                    .header("x-accel-buffering", "no"),
                session: session.to_string(),
                replay,
            },
            Err(StreamError::NoSession) => respond(404, "unknown or terminated session\n"),
            Err(StreamError::AlreadyOpen) => {
                respond(409, "this session already has an open SSE stream\n")
            }
        }
    }

    /// `DELETE /mcp` — explicit session termination.
    fn route_delete(&self, request: &HttpRequest) -> Outcome {
        let Some(session) = request.header("mcp-session-id") else {
            return respond(400, "Mcp-Session-Id header required\n");
        };
        if self.sessions.remove(session) {
            Outcome::Respond(HttpResponse::new(204))
        } else {
            respond(404, "unknown or terminated session\n")
        }
    }

    /// Serve one already-accepted connection: read one request, route it, and either
    /// write the response or run the SSE stream until the client or the session goes.
    fn serve_connection(&self, stream: TcpStream) {
        let _ = stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT));
        let _ = stream.set_write_timeout(Some(STREAM_WRITE_TIMEOUT));
        let Ok(read_half) = stream.try_clone() else {
            return;
        };
        let mut reader = BufReader::new(read_half);
        let mut writer = stream;

        let request = match wire::read_request(&mut reader, self.config.max_body_bytes) {
            Ok(Some(request)) => request,
            // A clean EOF or a dead socket: nothing to say back.
            Ok(None) | Err(WireError::Io) => return,
            Err(WireError::Malformed(status, why)) => {
                let _ = HttpResponse::text(status, &format!("{}\n", why)).write_to(&mut writer);
                return;
            }
        };

        match self.route(&request) {
            Outcome::Respond(response) => {
                let _ = response.write_to(&mut writer);
            }
            Outcome::Stream {
                head,
                session,
                replay,
            } => {
                self.run_stream(head, &session, replay, &mut writer);
                self.sessions.close_stream(&session);
            }
        }
    }

    /// The SSE write loop: replay first, then drain queued messages, emitting a comment
    /// keepalive whenever nothing arrives. Ends on the first write error (the client is
    /// gone) or when the session is terminated.
    fn run_stream<W: Write>(
        &self,
        head: HttpResponse,
        session: &str,
        replay: Vec<(u64, String)>,
        writer: &mut W,
    ) {
        if head.write_head_to(writer).is_err() {
            return;
        }
        for (id, payload) in replay {
            if write_all(writer, &wire::sse_event(id, &payload)).is_err() {
                return;
            }
        }
        loop {
            let written = match self.sessions.poll(session, self.config.sse_keepalive) {
                Poll::Events(events) => events.iter().try_for_each(|(id, payload)| {
                    write_all(writer, &wire::sse_event(*id, payload))
                }),
                Poll::Idle => write_all(writer, wire::SSE_KEEPALIVE),
                Poll::Gone => return,
            };
            if written.is_err() {
                return;
            }
        }
    }
}

fn write_all<W: Write>(writer: &mut W, text: &str) -> std::io::Result<()> {
    writer.write_all(text.as_bytes())?;
    writer.flush()
}

fn respond(status: u16, message: &str) -> Outcome {
    Outcome::Respond(HttpResponse::text(status, message))
}

/// What a body that arrived **without** a session id is permitted to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreSession {
    /// Exactly one `initialize` request, alone or alongside handshake notifications:
    /// dispatch it, and mint a session for its response.
    Handshake,
    /// Handshake notifications only. Dispatchable — JSON-RPC forbids a response to a
    /// notification, so there is nothing to attach a session to and nothing is minted.
    NotificationsOnly,
    /// Anything else. It needs a session id, and is refused before dispatch.
    Refused,
}

/// Classify a raw JSON-RPC message for the pre-session boundary.
///
/// Deliberately narrow, because `McpServer::handle_message` dispatches a batch as a
/// whole: one non-handshake element anywhere in the batch refuses the *entire* body,
/// rather than letting it ride along with an `initialize`. Two `initialize` requests are
/// refused too — one response header cannot answer both handshakes.
fn classify_pre_session(body: &str) -> PreSession {
    /// `Some(true)` for the `initialize` request, `Some(false)` for a handshake
    /// notification, `None` for anything that needs a session.
    fn handshake_element(value: &Value) -> Option<bool> {
        let method = value.get("method").and_then(Value::as_str)?;
        // JSON-RPC: an absent (or null) id is a notification, which gets no response.
        let is_request = value.get("id").is_some_and(|id| !id.is_null());
        match (method, is_request) {
            ("initialize", true) => Some(true),
            ("initialize", false) | ("notifications/initialized", false) => Some(false),
            _ => None,
        }
    }
    let elements = match serde_json::from_str::<Value>(body) {
        // An empty batch is not a handshake; neither is a bare scalar or bad JSON.
        Ok(Value::Array(elements)) if !elements.is_empty() => elements,
        Ok(value @ Value::Object(_)) => vec![value],
        _ => return PreSession::Refused,
    };
    let mut requests = 0usize;
    for element in &elements {
        match handshake_element(element) {
            Some(true) => requests += 1,
            Some(false) => {}
            None => return PreSession::Refused,
        }
    }
    match requests {
        0 => PreSession::NotificationsOnly,
        1 => PreSession::Handshake,
        _ => PreSession::Refused,
    }
}

/// A fresh session id: 128 bits from the OS CSPRNG, lowercase hex.
///
/// `None` when the OS refused entropy — the caller answers `500` rather than falling
/// back to a guessable id. Hex keeps the id inside the visible-ASCII range the MCP
/// Streamable HTTP transport requires of `Mcp-Session-Id`.
fn new_session_id() -> Option<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).ok()?;
    let mut id = String::with_capacity(32);
    for byte in bytes {
        id.push_str(&format!("{:02x}", byte));
    }
    Some(id)
}

/// Serve MCP over Streamable HTTP on an already-bound listener, one thread per
/// connection, until `listener.accept()` fails.
///
/// At most [`HttpConfig::max_connections`] connections are served at once; an accepted
/// connection past that is answered `503` and closed **without** a thread being spawned
/// for it, so a peer that opens sockets and then withholds request bytes cannot grow the
/// thread pool without bound.
///
/// The caller owns the bind address, and should bind **loopback**: this transport has no
/// authentication (see the module docs). Individual connection failures are handled
/// per-connection and never stop the accept loop.
pub fn serve_http(listener: TcpListener, transport: Arc<HttpTransport>) -> std::io::Result<()> {
    loop {
        let (stream, _peer) = listener.accept()?;
        let Some(permit) = transport.acquire_connection() else {
            refuse_connection(stream);
            continue;
        };
        let transport = Arc::clone(&transport);
        std::thread::spawn(move || {
            // Held for the whole connection; its `Drop` frees the slot however we leave.
            let _permit = permit;
            transport.serve_connection(stream);
        });
    }
}

/// Tell an over-cap peer `503` and close, on the accept thread and under a short write
/// timeout — a refusal must never cost the resource it is refusing.
fn refuse_connection(mut stream: TcpStream) {
    let _ = stream.set_write_timeout(Some(REFUSAL_WRITE_TIMEOUT));
    let _ = HttpResponse::text(503, "connection limit reached\n").write_to(&mut stream);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;

    fn build(config: HttpConfig) -> HttpTransport {
        let graph =
            Graph::load_str("<http://ex/a> <http://ex/p> <http://ex/b> .", "ntriples").unwrap();
        HttpTransport::new(Arc::new(Mutex::new(McpServer::new(graph))), config)
    }

    fn request(method: &str, headers: &[(&str, &str)], body: &str) -> HttpRequest {
        HttpRequest {
            method: method.to_string(),
            path: "/mcp".to_string(),
            headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: body.as_bytes().to_vec(),
        }
    }

    fn post(headers: &[(&str, &str)], body: &str) -> HttpRequest {
        let mut all = vec![("content-type", "application/json")];
        all.extend_from_slice(headers);
        request("POST", &all, body)
    }

    /// The response of an outcome that must not be a stream.
    fn responded(outcome: Outcome) -> HttpResponse {
        match outcome {
            Outcome::Respond(response) => response,
            Outcome::Stream { .. } => panic!("expected a plain response, got an SSE stream"),
        }
    }

    fn header_of(response: &HttpResponse, name: &str) -> Option<String> {
        response
            .headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }

    const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;

    /// Drive a full handshake and return the minted session id.
    fn initialize(transport: &HttpTransport) -> String {
        let response = responded(transport.route(&post(&[], INIT)));
        assert_eq!(response.status, 200);
        header_of(&response, "mcp-session-id").expect("initialize mints a session")
    }

    #[test]
    fn initialize_mints_a_session_and_later_requests_must_carry_it() {
        let transport = build(HttpConfig::default());
        let session = initialize(&transport);
        assert_eq!(session.len(), 32, "128 bits of hex: {}", session);
        assert!(session.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(transport.has_session(&session));
        assert_eq!(transport.session_count(), 1);

        // With the session: served.
        let ping = r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#;
        let ok = responded(transport.route(&post(&[("mcp-session-id", &session)], ping)));
        assert_eq!(ok.status, 200);

        // Without it: refused before dispatch.
        assert_eq!(responded(transport.route(&post(&[], ping))).status, 400);
        // With a bogus one: 404, the status MCP reserves for an expired session.
        let bogus = responded(transport.route(&post(&[("mcp-session-id", "nope")], ping)));
        assert_eq!(bogus.status, 404);
    }

    #[test]
    fn two_sessions_are_multiplexed_over_the_one_server() {
        let transport = build(HttpConfig::default());
        let first = initialize(&transport);
        let second = initialize(&transport);
        assert_ne!(first, second, "each initialize mints a distinct id");
        assert_eq!(transport.session_count(), 2);

        // Both can call tools against the same dataset.
        let call = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"stats","arguments":{}}}"#;
        for session in [&first, &second] {
            let response = responded(transport.route(&post(&[("mcp-session-id", session)], call)));
            assert_eq!(response.status, 200);
            let reply: Value = serde_json::from_slice(&response.body).unwrap();
            let payload: Value =
                serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap())
                    .unwrap();
            assert_eq!(payload["triples"], 1);
        }

        // Terminating one leaves the other alone.
        let deleted = responded(transport.route(&request(
            "DELETE",
            &[("mcp-session-id", &first)],
            "",
        )));
        assert_eq!(deleted.status, 204);
        assert!(!transport.has_session(&first));
        assert!(transport.has_session(&second));
        assert_eq!(transport.session_count(), 1);
    }

    #[test]
    fn a_notification_only_post_is_answered_202_with_no_body() {
        let transport = build(HttpConfig::default());
        let session = initialize(&transport);
        let notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let response = responded(transport.route(&post(
            &[("mcp-session-id", &session)],
            notification,
        )));
        assert_eq!(response.status, 202);
        assert!(response.body.is_empty());
    }

    #[test]
    fn an_initialize_notification_gets_202_and_mints_nothing() {
        let transport = build(HttpConfig::default());
        // No `id` ⇒ a notification: JSON-RPC forbids a response, so there is no
        // handshake result to attach a session to.
        let response = responded(transport.route(&post(
            &[],
            r#"{"jsonrpc":"2.0","method":"initialize","params":{}}"#,
        )));
        assert_eq!(response.status, 202);
        assert!(header_of(&response, "mcp-session-id").is_none());
        assert_eq!(transport.session_count(), 0);
    }

    #[test]
    fn the_session_cap_refuses_a_handshake_past_the_limit() {
        let transport = build(HttpConfig {
            max_sessions: 2,
            ..HttpConfig::default()
        });
        let first = initialize(&transport);
        let _second = initialize(&transport);
        let over = responded(transport.route(&post(&[], INIT)));
        assert_eq!(over.status, 503);
        assert!(header_of(&over, "mcp-session-id").is_none());
        assert_eq!(transport.session_count(), 2, "nothing was minted");

        // Freeing a slot lets the next client in.
        assert!(transport.end_session(&first));
        assert_eq!(
            responded(transport.route(&post(&[], INIT))).status,
            200,
            "a terminated session frees its slot"
        );
    }

    #[test]
    fn an_initialize_batch_still_mints_one_session() {
        let transport = build(HttpConfig::default());
        let batch = format!(
            r#"[{},{{"jsonrpc":"2.0","method":"notifications/initialized"}}]"#,
            INIT
        );
        let response = responded(transport.route(&post(&[], &batch)));
        assert_eq!(response.status, 200);
        assert!(header_of(&response, "mcp-session-id").is_some());
        assert_eq!(transport.session_count(), 1);
    }

    #[test]
    fn a_sessionless_batch_cannot_smuggle_a_call_alongside_the_handshake() {
        // The dataset is writable here *only* so a smuggled mutation would leave a mark:
        // if the batch were dispatched, the graph would grow and the assertion below
        // would catch it. Nothing about the boundary depends on `allow_update`.
        let graph =
            Graph::load_str("<http://ex/a> <http://ex/p> <http://ex/b> .", "ntriples").unwrap();
        let server = Arc::new(Mutex::new(McpServer::with_config(
            graph,
            crate::ServerConfig {
                allow_update: true,
                ..crate::ServerConfig::default()
            },
        )));
        let transport = HttpTransport::new(server, HttpConfig::default());

        let smuggled = format!(
            r#"[{},{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"update","arguments":{{"sparql":"INSERT DATA {{ <http://ex/x> <http://ex/y> <http://ex/z> }}"}}}}}}]"#,
            INIT
        );
        let refused = responded(transport.route(&post(&[], &smuggled)));
        assert_eq!(refused.status, 400, "a mixed batch is refused before dispatch");
        assert_eq!(transport.session_count(), 0, "and mints nothing");

        // Prove the `update` never ran: a legitimate session still sees one triple.
        let session = initialize(&transport);
        let stats = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"stats","arguments":{}}}"#;
        let response = responded(transport.route(&post(&[("mcp-session-id", &session)], stats)));
        let reply: Value = serde_json::from_slice(&response.body).unwrap();
        let payload: Value =
            serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(payload["triples"], 1, "the smuggled INSERT DATA never ran");
    }

    #[test]
    fn a_sessionless_batch_mints_only_for_a_real_initialize_request() {
        let transport = build(HttpConfig::default());
        // An `initialize` *notification* plus a `ping` request: the ping would produce a
        // response, but there is no handshake to hang a session on, and the ping needs a
        // session of its own — so the whole body is refused.
        let batch = r#"[{"jsonrpc":"2.0","method":"initialize","params":{}},{"jsonrpc":"2.0","id":2,"method":"ping"}]"#;
        let refused = responded(transport.route(&post(&[], batch)));
        assert_eq!(refused.status, 400);
        assert!(header_of(&refused, "mcp-session-id").is_none());
        assert_eq!(transport.session_count(), 0);

        // Two handshakes in one batch: one response header cannot answer both.
        let doubled = format!(r#"[{},{}]"#, INIT, INIT);
        assert_eq!(responded(transport.route(&post(&[], &doubled))).status, 400);
        assert_eq!(transport.session_count(), 0);
    }

    #[test]
    fn an_unknown_origin_is_refused_and_a_configured_one_is_echoed() {
        let transport = build(HttpConfig::default());
        // Default policy: any Origin at all is refused (DNS-rebinding defence).
        let refused = responded(transport.route(&post(&[("origin", "http://evil.test")], INIT)));
        assert_eq!(refused.status, 403);
        assert_eq!(transport.session_count(), 0, "refused before dispatch");

        let allowed = build(HttpConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            ..HttpConfig::default()
        });
        let still_refused =
            responded(allowed.route(&post(&[("origin", "http://evil.test")], INIT)));
        assert_eq!(still_refused.status, 403);
        let ok = responded(allowed.route(&post(&[("origin", "http://localhost:3000")], INIT)));
        assert_eq!(ok.status, 200);
        assert_eq!(
            header_of(&ok, "access-control-allow-origin").as_deref(),
            Some("http://localhost:3000")
        );
    }

    #[test]
    fn a_preflight_is_answered_only_for_an_allowed_origin() {
        let transport = build(HttpConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            ..HttpConfig::default()
        });
        let ok = responded(transport.route(&request(
            "OPTIONS",
            &[("origin", "http://localhost:3000")],
            "",
        )));
        assert_eq!(ok.status, 204);
        assert!(header_of(&ok, "access-control-allow-methods").is_some());
        let refused =
            responded(transport.route(&request("OPTIONS", &[("origin", "http://evil.test")], "")));
        assert_eq!(refused.status, 403);
    }

    #[test]
    fn content_negotiation_and_framing_faults_map_to_their_statuses() {
        let transport = build(HttpConfig::default());
        // Wrong Content-Type.
        assert_eq!(
            responded(transport.route(&request(
                "POST",
                &[("content-type", "text/plain")],
                INIT
            )))
            .status,
            415
        );
        // Accept that excludes JSON.
        assert_eq!(
            responded(transport.route(&post(&[("accept", "text/html")], INIT))).status,
            406
        );
        // Empty body.
        assert_eq!(responded(transport.route(&post(&[], "   "))).status, 400);
        // A protocol revision this server does not speak.
        assert_eq!(
            responded(transport.route(&post(&[("mcp-protocol-version", "1999-01-01")], INIT)))
                .status,
            400
        );
        // One it does.
        assert_eq!(
            responded(transport.route(&post(
                &[("mcp-protocol-version", SUPPORTED_PROTOCOL_VERSIONS[0])],
                INIT
            )))
            .status,
            200
        );
    }

    #[test]
    fn non_utf8_bodies_are_rejected_rather_than_lossily_decoded() {
        let transport = build(HttpConfig::default());
        let mut request = post(&[], "");
        request.body = vec![0xff, 0xfe];
        assert_eq!(responded(transport.route(&request)).status, 400);
    }

    #[test]
    fn a_foreign_path_and_an_unsupported_method_are_refused() {
        let transport = build(HttpConfig::default());
        let mut elsewhere = post(&[], INIT);
        elsewhere.path = "/somewhere".to_string();
        assert_eq!(responded(transport.route(&elsewhere)).status, 404);

        let session = initialize(&transport);
        let response =
            responded(transport.route(&request("PUT", &[("mcp-session-id", &session)], "")));
        assert_eq!(response.status, 405);
        assert_eq!(
            header_of(&response, "allow").as_deref(),
            Some("GET, POST, DELETE, OPTIONS")
        );
    }

    #[test]
    fn a_custom_endpoint_path_is_honoured() {
        let transport = build(HttpConfig {
            endpoint: "/agent/mcp".to_string(),
            ..HttpConfig::default()
        });
        let mut on_default = post(&[], INIT);
        on_default.path = "/mcp".to_string();
        assert_eq!(responded(transport.route(&on_default)).status, 404);

        let mut on_custom = post(&[], INIT);
        on_custom.path = "/agent/mcp".to_string();
        assert_eq!(responded(transport.route(&on_custom)).status, 200);
    }

    #[test]
    fn a_get_opens_one_stream_per_session_and_needs_a_live_one() {
        let transport = build(HttpConfig::default());
        let session = initialize(&transport);

        // No session id at all.
        assert_eq!(
            responded(transport.route(&request("GET", &[], ""))).status,
            400
        );
        // A session that never existed.
        assert_eq!(
            responded(transport.route(&request("GET", &[("mcp-session-id", "nope")], ""))).status,
            404
        );
        // A client that will not read SSE.
        assert_eq!(
            responded(transport.route(&request(
                "GET",
                &[("mcp-session-id", &session), ("accept", "application/json")],
                ""
            )))
            .status,
            406
        );

        // The real thing.
        let opened = transport.route(&request("GET", &[("mcp-session-id", &session)], ""));
        match &opened {
            Outcome::Stream { head, .. } => {
                assert_eq!(head.status, 200);
                assert_eq!(
                    header_of(head, "content-type").as_deref(),
                    Some("text/event-stream")
                );
            }
            Outcome::Respond(response) => panic!("expected a stream, got {}", response.status),
        }
        // A second concurrent stream for the same session is a conflict.
        assert_eq!(
            responded(transport.route(&request("GET", &[("mcp-session-id", &session)], ""))).status,
            409
        );
    }

    #[test]
    fn notify_and_broadcast_queue_only_for_live_sessions() {
        let transport = build(HttpConfig::default());
        let first = initialize(&transport);
        let second = initialize(&transport);
        assert!(transport.notify(&first, r#"{"jsonrpc":"2.0","method":"x"}"#));
        assert!(!transport.notify("nope", "{}"));
        assert_eq!(transport.broadcast("{}"), 2);
        assert!(transport.end_session(&second));
        assert!(!transport.end_session(&second));
        assert_eq!(transport.broadcast("{}"), 1);
    }

    #[test]
    fn a_stream_replays_queued_messages_then_keepalives_then_ends_with_the_session() {
        let transport = Arc::new(build(HttpConfig {
            // Force a keepalive almost immediately so the loop is exercised in-test.
            sse_keepalive: Duration::from_millis(20),
            ..HttpConfig::default()
        }));
        let session = initialize(&transport);
        transport.notify(&session, r#"{"jsonrpc":"2.0","method":"notifications/message"}"#);

        let Outcome::Stream {
            head,
            session: streaming,
            replay,
        } = transport.route(&request("GET", &[("mcp-session-id", &session)], ""))
        else {
            panic!("expected a stream");
        };

        // End the session from another thread so the write loop terminates.
        let closer = Arc::clone(&transport);
        let closing = session.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(120));
            closer.end_session(&closing);
        });

        let mut out: Vec<u8> = Vec::new();
        transport.run_stream(head, &streaming, replay, &mut out);
        handle.join().unwrap();

        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"), "{}", text);
        assert!(text.contains("content-type: text/event-stream\r\n"));
        assert!(
            text.contains("id: 1\nevent: message\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/message\"}\n\n"),
            "the queued message is framed as an SSE event: {}",
            text
        );
        assert!(text.contains(": keepalive\n\n"), "idle ticks emit a comment: {}", text);
    }

    #[test]
    fn a_reconnecting_stream_resumes_after_its_last_event_id() {
        let transport = Arc::new(build(HttpConfig {
            sse_keepalive: Duration::from_millis(20),
            ..HttpConfig::default()
        }));
        let session = initialize(&transport);
        transport.notify(&session, r#"{"id":"one"}"#);
        transport.notify(&session, r#"{"id":"two"}"#);

        // First stream: drains both, then the client "disconnects".
        let Outcome::Stream {
            head,
            session: streaming,
            replay,
        } = transport.route(&request("GET", &[("mcp-session-id", &session)], ""))
        else {
            panic!("expected a stream");
        };
        let closer = Arc::clone(&transport);
        let closing = session.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(120));
            closer.sessions.remove(&closing);
        });
        let mut first: Vec<u8> = Vec::new();
        transport.run_stream(head, &streaming, replay, &mut first);
        handle.join().unwrap();
        assert!(String::from_utf8(first).unwrap().contains("data: {\"id\":\"two\"}"));

        // Re-create a session and check the replay filter directly: everything after
        // event 1 comes back, event 1 itself does not.
        let resumed = initialize(&transport);
        transport.notify(&resumed, r#"{"id":"one"}"#);
        transport.notify(&resumed, r#"{"id":"two"}"#);
        assert!(matches!(
            transport.sessions.poll(&resumed, Duration::from_millis(20)),
            Poll::Events(events) if events.len() == 2
        ));
        transport.sessions.close_stream(&resumed);
        let Outcome::Stream { replay, .. } = transport.route(&request(
            "GET",
            &[("mcp-session-id", &resumed), ("last-event-id", "1")],
            "",
        )) else {
            panic!("expected a stream");
        };
        assert_eq!(replay, vec![(2, r#"{"id":"two"}"#.to_string())]);
    }

    #[test]
    fn pre_session_classification_admits_only_the_handshake() {
        use PreSession::*;
        let init_note = r#"{"jsonrpc":"2.0","method":"initialize","params":{}}"#;
        let initialized = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

        assert_eq!(classify_pre_session(INIT), Handshake);
        assert_eq!(classify_pre_session(&format!("[{}]", INIT)), Handshake);
        assert_eq!(
            classify_pre_session(&format!("[{},{}]", INIT, initialized)),
            Handshake
        );
        // No response to attach a session to.
        assert_eq!(classify_pre_session(init_note), NotificationsOnly);
        assert_eq!(classify_pre_session(initialized), NotificationsOnly);

        // Everything else needs a session id.
        let ping = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        assert_eq!(classify_pre_session(ping), Refused);
        assert_eq!(classify_pre_session(&format!("[{},{}]", INIT, ping)), Refused);
        assert_eq!(
            classify_pre_session(&format!("[{},{}]", init_note, ping)),
            Refused
        );
        assert_eq!(classify_pre_session(&format!("[{},{}]", INIT, INIT)), Refused);
        // An `initialize` with a null id is a notification, not a mintable handshake.
        assert_eq!(
            classify_pre_session(r#"{"jsonrpc":"2.0","id":null,"method":"initialize"}"#),
            NotificationsOnly
        );
        assert_eq!(classify_pre_session("not json"), Refused);
        assert_eq!(classify_pre_session("[]"), Refused);
        assert_eq!(classify_pre_session("5"), Refused);
        assert_eq!(classify_pre_session(r#"{"jsonrpc":"2.0"}"#), Refused);
    }

    #[test]
    fn the_connection_cap_is_claimed_and_released_by_the_permit() {
        let transport = build(HttpConfig {
            max_connections: 2,
            ..HttpConfig::default()
        });
        assert_eq!(transport.connection_count(), 0);
        let first = transport.acquire_connection().expect("a free slot");
        let second = transport.acquire_connection().expect("the last free slot");
        assert_eq!(transport.connection_count(), 2);
        assert!(
            transport.acquire_connection().is_none(),
            "the cap refuses rather than overshooting"
        );
        drop(second);
        assert_eq!(transport.connection_count(), 1);
        assert!(transport.acquire_connection().is_some(), "a slot came back");
        drop(first);

        // `0` is the documented escape hatch.
        let uncapped = build(HttpConfig {
            max_connections: 0,
            ..HttpConfig::default()
        });
        let permits: Vec<_> = (0..64).map(|_| uncapped.acquire_connection()).collect();
        assert!(permits.iter().all(Option::is_some));
        assert_eq!(uncapped.connection_count(), 64);
    }

    #[test]
    fn session_ids_are_distinct_128_bit_hex() {
        let a = new_session_id().expect("OS entropy");
        let b = new_session_id().expect("OS entropy");
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn the_default_config_is_the_locked_down_one() {
        let config = HttpConfig::default();
        assert_eq!(config.endpoint, "/mcp");
        assert!(
            config.allowed_origins.is_empty(),
            "no browser origin is trusted by default"
        );
        assert_eq!(config.max_body_bytes, 4 * 1024 * 1024);
        assert_eq!(config.replay_buffer, 64);
        assert_eq!(config.max_sessions, 256);
        assert_eq!(config.max_connections, 512);
        let transport = build(config);
        assert_eq!(transport.config().endpoint, "/mcp");
    }
}
