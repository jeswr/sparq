// AUTHORED-BY Claude Opus 4.8
//! The WebSocketChannel2023 HTTP surface: discovery, subscribe, and the WS receive endpoint.
//!
//! Route layout (mounted by [`crate::app::build_router`]):
//! - `POST /.notifications/WebSocketChannel2023/`  — subscribe; returns a channel description with a
//!   `receiveFrom` `ws(s)://` URL that carries a minted receive token. **Auth-gated** (behind the
//!   DPoP middleware — fail-closed) **and WAC-gated** (the WebID needs `acl:Read` on the topic); the
//!   token binds receive to the authenticated subscriber+topic.
//! - `GET  /.notifications/WebSocketChannel2023/receive?topic=<iri>&token=<tok>` — upgrade to a
//!   WebSocket and register the connection under `<iri>`. **Token-gated:** the `token` must be a
//!   valid, unexpired receive token whose bound topic matches `<iri>`, else the upgrade is rejected
//!   (401, no socket); **and WAC-gated** against the token's bound WebID. The server then pushes
//!   AS2.0 notifications on change.
//! - `GET  /.well-known/solid`                     — a storage-description document advertising the
//!   subscription service (discovery; unauthenticated, like a storage description).
//!
//! ## Discovery (per the Solid Notifications Protocol)
//! A client finds the channel two ways, both implemented here:
//! 1. the `/.well-known/solid` storage description lists the `notificationChannel` subscription
//!    service + its supported `channelType`, and
//! 2. [`link_headers`] returns the `Link` rels (`describedby` + `solid:storageDescription`) the LDP
//!    GET/HEAD handler can attach so a client can `HEAD` a resource and discover the same service.
//!    (Attaching them to the LDP responses is a one-line wire in the handler; this module owns the
//!    values so the discovery contract lives in one place.)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::json;

use crate::auth::VerifiedToken;
use crate::authz::wac::{Decision, WacAuthorizer};
use crate::authz::{is_acl_resource, AccessMode};
use crate::error::ServerError;
use crate::ldp::handler::{request_origin, LdpState};
use crate::ldp::target::parse_target;
use crate::notifications::activity::{AS2_CONTEXT, NOTIFICATIONS_CONTEXT};
use crate::notifications::NotificationHub;
use crate::store::Store;

/// The WebSocketChannel2023 channel-type IRI (the spec's `type` value).
pub const WEBSOCKET_CHANNEL_2023_TYPE: &str =
    "http://www.w3.org/ns/solid/notifications#WebSocketChannel2023";
/// The path of the subscription service (the POST target).
pub const SUBSCRIPTION_PATH: &str = "/.notifications/WebSocketChannel2023/";
/// The path of the WS receive endpoint (the GET-upgrade target; topic in `?topic=`).
pub const RECEIVE_PATH: &str = "/.notifications/WebSocketChannel2023/receive";
/// The storage-description / well-known discovery document path.
pub const WELL_KNOWN_SOLID_PATH: &str = "/.well-known/solid";

// [GPT-5.6] Keep the hot-path counters private so callers can observe but cannot mutate them.
struct NotificationMetricCounters {
    dropped_subscribers_total: AtomicU64,
    lagged_events_total: AtomicU64,
}

static NOTIFICATION_METRICS: LazyLock<NotificationMetricCounters> =
    LazyLock::new(|| NotificationMetricCounters {
        dropped_subscribers_total: AtomicU64::new(0),
        lagged_events_total: AtomicU64::new(0),
    });

/// Process-wide WebSocket notification overflow counters.
///
/// Obtain the current values with [`NotificationMetrics::snapshot`]. Counts are monotonic for the
/// lifetime of the process and use relaxed atomic ordering because they are observability-only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotificationMetrics {
    /// Subscribers closed after falling behind the per-topic broadcast buffer.
    pub dropped_subscribers_total: u64,
    /// Notification events skipped by lagged subscribers before they were closed.
    pub lagged_events_total: u64,
}

impl NotificationMetrics {
    /// Returns a point-in-time snapshot of the process-wide notification overflow counters.
    #[must_use]
    pub fn snapshot() -> Self {
        Self {
            dropped_subscribers_total: NOTIFICATION_METRICS
                .dropped_subscribers_total
                .load(Ordering::Relaxed),
            lagged_events_total: NOTIFICATION_METRICS
                .lagged_events_total
                .load(Ordering::Relaxed),
        }
    }
}

/// [OPUS-5] #4877: state for the notification routes — a handle on the SAME [`LdpState`] the LDP
/// routes carry.
///
/// It is the single source of every input this surface needs — the notification hub (so a subscriber
/// registered via `…/receive` is in the registry the LDP emit path fans to), the server's public base
/// URL (for the absolute `receiveFrom` / subscription-service URLs), and the store + ACL cache the
/// [`WacAuthorizer`] reads `.acl` resources through. Sharing ONE state (rather than copying the hub +
/// base URL out of it) is what makes "the notification surface authorizes with the same engine,
/// against the same ACLs, under the same base URL as LDP" true by construction.
pub struct NotifyState<S: Store> {
    ldp: Arc<LdpState<S>>,
}

// Hand-written (not derived) so the impl does NOT require `S: Clone` — cloning is an `Arc` bump.
impl<S: Store> Clone for NotifyState<S> {
    fn clone(&self) -> Self {
        Self {
            ldp: self.ldp.clone(),
        }
    }
}

impl<S: Store> NotifyState<S> {
    pub fn new(ldp: Arc<LdpState<S>>) -> Self {
        Self { ldp }
    }

    /// The shared notification registry (the hub the LDP emit path fans to).
    pub fn hub(&self) -> &NotificationHub {
        &self.ldp.notifications
    }

    /// The server's public base URL.
    fn base_url(&self) -> &str {
        self.ldp.base_url()
    }

    /// The absolute subscription-service URL (the POST target).
    fn subscription_service_url(&self) -> String {
        format!("{}{SUBSCRIPTION_PATH}", self.base_url().trim_end_matches('/'))
    }

    /// The `receiveFrom` WebSocket URL for a topic, carrying the minted receive `token`. The base
    /// URL's scheme is mapped http→ws / https→wss (WebSocketChannel2023 §receiveFrom — the receive
    /// endpoint is a WebSocket URL). The token authorizes the WS upgrade for this topic (a browser
    /// `WebSocket` cannot send the DPoP `Authorization` header, so the spec carries authz in the URL).
    fn receive_from_url(&self, topic: &str, token: &str) -> String {
        let base = self.base_url().trim_end_matches('/');
        let ws_base = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{rest}")
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{rest}")
        } else {
            base.to_string()
        };
        // URL-encode the topic + token into the query string (minimal: encode the few reserved chars
        // that matter for a query value; the topic is a server-issued absolute IRI and the token is a
        // server-issued base64url string — neither is user free-text).
        format!(
            "{ws_base}{RECEIVE_PATH}?topic={}&token={}",
            encode_query_value(topic),
            encode_query_value(token),
        )
    }
}

/// The JSON-LD subscription request body a client POSTs. Per WebSocketChannel2023 the client sends a
/// `type` (the channel-type IRI) and a `topic` (the resource/container to watch). We accept the flat
/// shape from the skill; extra JSON-LD framing fields are ignored.
#[derive(Debug, Deserialize)]
pub struct SubscriptionRequest {
    /// The channel type IRI; must be the WebSocketChannel2023 type. (Optional in the parse — a
    /// missing/other type is rejected in the handler with a clear 400, not a silent accept.)
    #[serde(rename = "type")]
    pub channel_type: Option<String>,
    /// The resource OR container IRI to watch.
    pub topic: Option<String>,
}

/// Resolve a client-supplied `topic` IRI to the LDP target it names ON THIS SERVER.
///
/// A topic is authorizable here only if it is a resource of this pod: it must sit under the server's
/// base URL, and its path must survive the SAME [`parse_target`] validation every LDP request pays
/// (path traversal, empty interior segments, and the reserved `/.identity/**` namespace are all
/// refused there). A foreign IRI has no ACL hierarchy on this server, so there is nothing to
/// authorize — it is a request error, not a denial, and the base URL it is compared against is
/// public, so the 400 discloses nothing.
///
/// The parsed target must round-trip to the topic string EXACTLY. `parse_target` normalises (it
/// strips a query/fragment), so without this the IRI we AUTHORIZE could differ from the IRI the hub
/// keys the subscription on — the classic authorize-one-thing/act-on-another gap. Requiring equality
/// makes the authorized IRI and the registry key the same string by construction.
fn topic_resource_iri(base_url: &str, topic: &str) -> Result<String, ServerError> {
    let base = base_url.trim_end_matches('/');
    let path = topic
        .strip_prefix(base)
        .filter(|p| p.starts_with('/'))
        .ok_or_else(|| ServerError::BadRequest("topic is not a resource on this server".into()))?;
    let target = parse_target(base, path)?;
    if target.iri != topic {
        return Err(ServerError::BadRequest(
            "topic must be a canonical resource IRI".into(),
        ));
    }
    Ok(target.iri)
}

/// [OPUS-5] #4877: per-resource Web Access Control for a subscription — may `web_id` READ `topic`?
///
/// This is the SAME [`WacAuthorizer`], over the SAME store, base URL and ACL cache, that the LDP
/// routes authorize with — so a subscription can never see a change signal for a resource the WebID
/// could not have read with a GET. The required mode mirrors the LDP mapping exactly: `acl:Read`
/// normally, but [`AccessMode::Control`] for an `.acl` target (managing/observing access rules is
/// always the Control privilege), which is what [`crate::ldp::handler`] applies too.
///
/// Denials are uniform and fail-closed: [`ServerError::Forbidden`] (403) for an authenticated-caller
/// denial, the [`parse_target`] 400 for a topic that is not a resource here, and a propagated store
/// error for a backend fault (never silently "no ACL" — that would fail OPEN). `web_id` is always
/// `Some` at both call sites, so [`Decision::Unauthenticated`] is unreachable; it is folded into the
/// same 403 rather than left as an unhandled arm.
async fn authorize_topic_read<S: Store>(
    ldp: &LdpState<S>,
    topic: &str,
    web_id: &str,
    origin: Option<&str>,
) -> Result<(), ServerError> {
    let target = topic_resource_iri(ldp.base_url(), topic)?;
    let required = if is_acl_resource(&target) {
        AccessMode::Control
    } else {
        AccessMode::Read
    };
    let wac = WacAuthorizer::with_cache(&ldp.store, ldp.base_url(), &ldp.acl_cache);
    match wac.authorize(&target, required, Some(web_id), origin).await? {
        Decision::Allow(_) => Ok(()),
        Decision::Unauthenticated | Decision::Forbidden => Err(ServerError::Forbidden),
    }
}

/// `POST /.notifications/WebSocketChannel2023/` — subscribe to a topic.
///
/// **Auth (fail-closed):** the caller MUST be authenticated (a WebID). An anonymous/public caller is
/// rejected with 401 — there are NO anonymous subscriptions. (This handler runs behind the DPoP auth
/// middleware, which injects the [`VerifiedToken`]; `is_public()` ⇒ unauthenticated.)
///
/// **Authorization (fail-closed):** the authenticated WebID must additionally hold `acl:Read` on the
/// topic, evaluated by the same WAC engine ([`crate::authz::WacAuthorizer`]) as the LDP routes. A
/// WebID with no read access to the topic gets a 403 and NO token is minted — so it can neither
/// subscribe nor reach the receive endpoint. This is the check the pre-WAC surface documented as
/// missing: an authenticated caller could formerly subscribe to ANY topic IRI.
///
/// On success the handler MINTS an unguessable, short-lived **receive token** bound to
/// `(authenticated WebID, topic, expiry)` and embeds it in the `receiveFrom` URL — this is what gates
/// the otherwise-headerless WS receive endpoint (see [`receive_handler`]).
pub async fn subscribe_handler<S: Store>(
    State(state): State<Arc<NotifyState<S>>>,
    Extension(token): Extension<VerifiedToken>,
    headers: HeaderMap,
    Json(req): Json<SubscriptionRequest>,
) -> Response {
    // Fail-closed: no anonymous subscriptions. After this check `web_id` is `Some`.
    let web_id = match &token.web_id {
        Some(w) => w.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "authentication required to subscribe",
            )
                .into_response();
        }
    };

    // Validate the channel type if the client sent one (reject a wrong type rather than silently
    // treating it as WebSocketChannel2023).
    if let Some(ty) = req.channel_type.as_deref() {
        if ty != WEBSOCKET_CHANNEL_2023_TYPE {
            return (
                StatusCode::BAD_REQUEST,
                "unsupported channel type (only WebSocketChannel2023)",
            )
                .into_response();
        }
    }

    let topic = match req.topic.as_deref() {
        Some(t) if !t.is_empty() => t,
        _ => return (StatusCode::BAD_REQUEST, "missing topic").into_response(),
    };

    // Per-resource WAC: this WebID must be able to READ the topic. Runs BEFORE the token is minted,
    // so an unauthorized subscribe leaves no credential behind. The request `Origin` is threaded in
    // exactly as the LDP routes thread it, so an `acl:origin`-restricted grant behaves identically
    // here (a rule with no `acl:origin` is unaffected).
    let origin = request_origin(&headers);
    if let Err(err) = authorize_topic_read(&state.ldp, topic, &web_id, origin).await {
        return err.into_response();
    }

    // Mint the receive token: unguessable, short-lived, bound to (this authenticated WebID, topic).
    // Without it the receive endpoint refuses the upgrade — so only this authenticated subscriber of
    // this topic can connect. The token (never logged) is embedded in `receiveFrom`.
    let receive_token = state.hub().mint_receive_token(&web_id, topic).await;
    let receive_from = state.receive_from_url(topic, &receive_token);

    // The channel description: per WebSocketChannel2023, `receiveFrom` is the ws(s):// URL the client
    // opens. We do NOT pre-register the topic here — registration happens when the WebSocket connects
    // (so a subscribe POST that is never followed by a connect leaks nothing).
    let body = json!({
        "@context": [NOTIFICATIONS_CONTEXT, AS2_CONTEXT],
        "id": receive_from,
        "type": WEBSOCKET_CHANNEL_2023_TYPE,
        "topic": topic,
        "receiveFrom": receive_from,
    });
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/ld+json")],
        body.to_string(),
    )
        .into_response()
}

/// Query params for the WS receive endpoint.
#[derive(Debug, Deserialize)]
pub struct ReceiveQuery {
    pub topic: Option<String>,
    /// The receive token minted by the authenticated subscribe (see [`subscribe_handler`]). Required.
    pub token: Option<String>,
}

/// `GET /.notifications/WebSocketChannel2023/receive?topic=<iri>&token=<tok>` — upgrade to a
/// WebSocket and stream notifications for `<iri>`.
///
/// ## Auth on the WS upgrade — token-gated (the spec reality, implemented)
/// A browser `WebSocket` cannot carry the DPoP-bound `Authorization` header, so per the spec the
/// `receiveFrom` URL carries its own short-lived authorization. We REQUIRE a valid **receive token**
/// here: it must exist, be unexpired, and its bound topic must equal the requested `topic`. The token
/// is minted ONLY by the authenticated subscribe (bound to that WebID + topic), so a connection
/// without a token — or with an invalid / expired / wrong-topic token — is rejected (401, NO socket,
/// NO subscriber registered). This closes the previously-open receive bypass (anyone who guessed a
/// resource IRI could receive its change notifications without subscribing).
///
/// ## Authorization on the WS upgrade — per-resource WAC (implemented)
/// The token proves only WHICH authenticated subscriber is connecting. The DEEPER question — is that
/// WebID allowed to READ this resource? — is answered here by the SAME [`WacAuthorizer`] the LDP
/// routes and [`subscribe_handler`] use, against the WebID the token is BOUND to. Re-checking at
/// connect (rather than trusting the subscribe-time decision baked into the token) means access
/// revoked inside the token's lifetime is honoured at the next connect instead of at expiry. A
/// denial is a 403 with NO socket and NO subscriber registered.
///
/// `ws` is taken as a `Result` (not a bare `WebSocketUpgrade`) ON PURPOSE: the token-gate must run
/// FIRST and UNCONDITIONALLY. If `WebSocketUpgrade` were a plain extractor, its rejection would
/// short-circuit BEFORE the token check — so a request with bad/missing upgrade headers would 426
/// without ever validating authorization, and (more importantly) the security gate would be coupled
/// to the WS extractor's success. By deferring the `Result`, we reject an absent/invalid/expired/
/// wrong-topic token with 401 regardless of the upgrade headers, and only surface the WS rejection
/// after the token has validated.
pub async fn receive_handler<S: Store>(
    State(state): State<Arc<NotifyState<S>>>,
    Query(q): Query<ReceiveQuery>,
    headers: HeaderMap,
    ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Response {
    let topic = match q.topic {
        Some(t) if !t.is_empty() => t,
        _ => return (StatusCode::BAD_REQUEST, "missing topic").into_response(),
    };
    // Token-gate (runs FIRST, unconditionally): require a valid, unexpired, topic-matching receive
    // token. Reject (401, no socket) otherwise. We deliberately do NOT echo the token or distinguish
    // absent/invalid/expired in the response body — a uniform 401 avoids leaking which condition
    // failed.
    let token = match q.token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                "a valid receive token is required",
            )
                .into_response()
        }
    };
    let web_id = match state.hub().receive_token_web_id(&token, &topic).await {
        Some(w) => w,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "a valid receive token is required",
            )
                .into_response()
        }
    };
    // Per-resource WAC against the WebID the token is BOUND to — re-evaluated now, not inherited from
    // the subscribe-time decision, so revoked access is honoured at connect. Still BEFORE the upgrade
    // extractor, so a denial is a plain 403 with no socket and no registered subscriber.
    let origin = request_origin(&headers);
    if let Err(err) = authorize_topic_read(&state.ldp, &topic, &web_id, origin).await {
        return err.into_response();
    }
    // The token validated and the WebID is authorized. Now the request MUST be a genuine WS upgrade;
    // surface the extractor's own rejection (e.g. 426 Upgrade Required) if not.
    let ws = match ws {
        Ok(ws) => ws,
        Err(rej) => return rej.into_response(),
    };
    // Only AFTER authorization do we upgrade + register a subscriber.
    let hub = state.hub().clone();
    ws.on_upgrade(move |socket| stream_notifications(socket, hub, topic))
}

/// The per-connection task: register a subscriber, forward every notification to the socket, and
/// clean up (drop the receiver ⇒ the hub prunes the topic on its next emit) when the socket closes.
///
/// Concurrency: a `tokio::select!` over (a) the next broadcast notification and (b) the next inbound
/// socket message. Inbound frames from the client are drained (a WebSocketChannel2023 receive socket
/// is server→client only; we read solely to observe a Close / a transport error so we can tear down
/// promptly and not leak the subscription).
async fn stream_notifications(mut socket: WebSocket, hub: NotificationHub, topic: String) {
    let mut rx = hub.subscribe(&topic).await;

    loop {
        tokio::select! {
            // (a) A notification for this topic — forward it as a text frame.
            received = rx.recv() => {
                match received {
                    Ok(body) => {
                        if socket.send(Message::text(body.to_string())).await.is_err() {
                            break; // the client went away mid-send
                        }
                    }
                    // The buffer overran for this slow client: a frame was dropped. Tell the client to
                    // reconcile by closing — it should re-subscribe + re-read (missed-update safety).
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        NOTIFICATION_METRICS
                            .dropped_subscribers_total
                            .fetch_add(1, Ordering::Relaxed);
                        NOTIFICATION_METRICS
                            .lagged_events_total
                            .fetch_add(skipped, Ordering::Relaxed);
                        let _ = socket
                            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: 1011, // "internal error" / server overload — client reconnects
                                reason: "notification backlog overflow; reconnect and reconcile".into(),
                            })))
                            .await;
                        break;
                    }
                    // The sender was dropped (the topic channel went away) — nothing more will arrive.
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            // (b) An inbound socket message — only meaningful as a Close / error signal.
            inbound = socket.recv() => {
                match inbound {
                    Some(Ok(Message::Close(_))) | None => break, // clean close or stream end
                    Some(Ok(_)) => { /* ignore any client frame; receive socket is server→client */ }
                    Some(Err(_)) => break, // transport error — tear down
                }
            }
        }
    }
    // `rx` drops here ⇒ the broadcast receiver count for `topic` decrements; the hub prunes a
    // now-0-receiver topic on its next emit. No explicit deregister call is needed — the registry is
    // self-cleaning, which is leak-free even if this task is cancelled.
}

/// `GET /.well-known/solid` — the storage-description / discovery document.
///
/// Advertises the notification subscription service + the supported channel type so a client can find
/// where to subscribe WITHOUT hardcoding the path. Unauthenticated (discovery is public, like a
/// storage description).
pub async fn storage_description_handler<S: Store>(
    State(state): State<Arc<NotifyState<S>>>,
) -> Response {
    let body = json!({
        "@context": [NOTIFICATIONS_CONTEXT, AS2_CONTEXT],
        "notificationChannel": [
            {
                "id": state.subscription_service_url(),
                "channelType": WEBSOCKET_CHANNEL_2023_TYPE,
                // The subscription service: POST a channel request here to obtain a `receiveFrom` URL.
                "subscriptionService": state.subscription_service_url(),
            }
        ],
    });
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/ld+json")],
        body.to_string(),
    )
        .into_response()
}

/// The discovery `Link` header VALUES the LDP GET/HEAD handler can attach to a resource response so a
/// client can `HEAD` the resource and find the storage description (which lists the subscription
/// service). Returns `(rel, target)` pairs; the caller formats `<target>; rel="rel"`.
///
/// This is the single home for the discovery contract — both the well-known document and the LDP
/// `Link` headers point at the same storage description, so the two never drift.
pub fn link_headers(base_url: &str) -> Vec<(&'static str, String)> {
    let base = base_url.trim_end_matches('/');
    let storage_desc = format!("{base}{WELL_KNOWN_SOLID_PATH}");
    vec![
        // The resource is described by the storage description (which lists notification channels).
        ("describedby", storage_desc.clone()),
        // The Solid storage-description rel (the protocol's discovery anchor).
        (
            "http://www.w3.org/ns/solid/terms#storageDescription",
            storage_desc,
        ),
    ]
}

/// Minimal percent-encoding for a URL query VALUE. Encodes the characters that would otherwise break
/// the query (`&`, `=`, `#`, `?`, space, `%`) and the IRI scheme separators are left as-is since the
/// topic is a server-issued absolute IRI. (Deliberately not a general URL-encoder — see the note in
/// [`NotifyState::receive_from_url`].)
fn encode_query_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            // unreserved per RFC 3986 + the IRI chars common in an http(s) IRI we keep readable.
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b':' | b'/' => {
                out.push(b as char)
            }
            other => {
                out.push('%');
                out.push(hex_digit(other >> 4));
                out.push(hex_digit(other & 0x0f));
            }
        }
    }
    out
}

fn hex_digit(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'A' + (n - 10)) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient};
    use axum::body::Bytes;

    const BASE: &str = "https://pod.example";
    const ALICE: &str = "https://alice.example/profile#me";
    const BOB: &str = "https://bob.example/profile#me";
    const TOPIC: &str = "https://pod.example/a";

    type TestStore = CompositeStore<InMemorySparqClient, InMemoryBlobStore>;

    /// A state whose store has NO ACLs at all. WAC is fail-closed, so every subscribe against it is
    /// denied — which is what the non-authorization tests below (401 / 400 shapes) assert reach their
    /// own rejection FIRST.
    fn state() -> Arc<NotifyState<TestStore>> {
        state_with(CompositeStore::new(
            InMemorySparqClient::new(),
            InMemoryBlobStore::new(),
        ))
    }

    fn state_with(store: TestStore) -> Arc<NotifyState<TestStore>> {
        Arc::new(NotifyState::new(Arc::new(LdpState::new(store, BASE))))
    }

    /// A state whose root `.acl` grants `agent` Read (inheritable) over the whole pod.
    async fn state_granting_read_to(agent: &str) -> Arc<NotifyState<TestStore>> {
        let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
        let acl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#r> a acl:Authorization;
                 acl:agent <{agent}>;
                 acl:accessTo <{BASE}/>;
                 acl:default <{BASE}/>;
                 acl:mode acl:Read."#
        );
        store
            .write(&format!("{BASE}/.acl"), Bytes::from(acl), "text/turtle")
            .await
            .expect("seed root acl");
        state_with(store)
    }

    fn web_id_token(web_id: &str) -> VerifiedToken {
        VerifiedToken {
            web_id: Some(web_id.to_string()),
            ..VerifiedToken::default()
        }
    }

    /// A genuine `WebSocketUpgrade` rejection — the extractor's OWN `Err`, produced by running it
    /// over a plain GET that carries no upgrade headers. The rejection variants are
    /// `#[non_exhaustive]`, so a test cannot construct one directly; this is how the receive tests
    /// exercise the "authorization runs regardless of the upgrade headers" contract.
    async fn upgrade_rejection() -> WebSocketUpgradeRejection {
        use axum::extract::FromRequestParts;
        let (mut parts, _) = axum::http::Request::builder()
            .uri("/")
            .body(())
            .expect("request builds")
            .into_parts();
        WebSocketUpgrade::from_request_parts(&mut parts, &())
            .await
            .expect_err("a plain GET is not a WebSocket upgrade")
    }

    async fn subscribe(
        state: Arc<NotifyState<TestStore>>,
        token: VerifiedToken,
        topic: &str,
    ) -> Response {
        subscribe_handler(
            State(state),
            Extension(token),
            HeaderMap::new(),
            Json(SubscriptionRequest {
                channel_type: Some(WEBSOCKET_CHANNEL_2023_TYPE.to_string()),
                topic: Some(topic.to_string()),
            }),
        )
        .await
    }

    #[test]
    fn receive_from_maps_https_to_wss() {
        let s = state();
        let url = s.receive_from_url("https://pod.example/a", "tok123");
        assert!(
            url.starts_with("wss://pod.example/.notifications/WebSocketChannel2023/receive?topic="),
            "{url}"
        );
        // The topic IRI round-trips (its reserved query chars are encoded).
        assert!(
            url.contains("https%3A%2F%2Fpod.example%2Fa") || url.contains("https://pod.example/a"),
            "{url}"
        );
        // The receive token is carried in the URL.
        assert!(url.contains("&token=tok123"), "{url}");
    }

    #[test]
    fn receive_from_maps_http_to_ws() {
        let s = Arc::new(NotifyState::new(Arc::new(LdpState::new(
            CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new()),
            "http://localhost:3000",
        ))));
        let url = s.receive_from_url("http://localhost:3000/a", "tok123");
        assert!(url.starts_with("ws://localhost:3000/"), "{url}");
    }

    #[test]
    fn subscription_service_url_is_absolute() {
        assert_eq!(
            state().subscription_service_url(),
            "https://pod.example/.notifications/WebSocketChannel2023/"
        );
    }

    #[test]
    fn link_headers_point_at_well_known() {
        let links = link_headers("https://pod.example");
        assert!(links
            .iter()
            .any(|(rel, t)| *rel == "describedby" && t == "https://pod.example/.well-known/solid"));
        assert!(links
            .iter()
            .any(|(rel, _)| rel.contains("storageDescription")));
    }

    #[test]
    fn encode_query_value_escapes_reserved() {
        // `&` and `=` and space and `#` must be encoded so they cannot break out of the query value.
        let e = encode_query_value("a&b=c d#e");
        assert!(!e.contains('&'));
        assert!(!e.contains(' '));
        assert!(!e.contains('#'));
        assert!(e.contains("%26") && e.contains("%3D") && e.contains("%20") && e.contains("%23"));
    }

    #[tokio::test]
    async fn subscribe_handler_rejects_anonymous() {
        let resp = subscribe(state(), VerifiedToken::public(), TOPIC).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn subscribe_handler_accepts_authenticated_and_returns_receive_from() {
        let resp = subscribe(
            state_granting_read_to(ALICE).await,
            web_id_token(ALICE),
            TOPIC,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_to_string(resp).await;
        assert!(body.contains("\"receiveFrom\""), "{body}");
        assert!(
            body.contains("wss://pod.example/.notifications/WebSocketChannel2023/receive"),
            "{body}"
        );
        assert!(body.contains(WEBSOCKET_CHANNEL_2023_TYPE), "{body}");
    }

    #[tokio::test]
    async fn subscribe_handler_rejects_wrong_channel_type() {
        // Authorized on the topic, so the 400 is the channel-type rejection and not a WAC denial.
        let resp = subscribe_handler(
            State(state_granting_read_to(ALICE).await),
            Extension(web_id_token(ALICE)),
            HeaderMap::new(),
            Json(SubscriptionRequest {
                channel_type: Some("http://example/OtherChannel".to_string()),
                topic: Some(TOPIC.to_string()),
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn subscribe_handler_rejects_missing_topic() {
        let resp = subscribe_handler(
            State(state()),
            Extension(web_id_token(ALICE)),
            HeaderMap::new(),
            Json(SubscriptionRequest {
                channel_type: None,
                topic: None,
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // --- per-resource WAC on subscribe (#4877) ------------------------------------------------

    /// The issue's acceptance criterion: a subscribe for a topic the WebID has NO `acl:Read` on is
    /// DENIED. Alice holds Read on the whole pod; Bob holds nothing, so his subscribe is a 403 —
    /// before WAC was wired here he would have received a 200 + a `receiveFrom` token.
    #[tokio::test]
    async fn subscribe_denied_for_topic_the_web_id_cannot_read() {
        let s = state_granting_read_to(ALICE).await;
        let resp = subscribe(s.clone(), web_id_token(BOB), TOPIC).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        // Fail-closed all the way through: a denied subscribe mints NO receive token, so Bob cannot
        // reach the receive endpoint either.
        let body = body_to_string(resp).await;
        assert!(!body.contains("receiveFrom"), "{body}");
    }

    /// Fail-closed by default: with no ACL anywhere in the hierarchy, even an authenticated caller is
    /// denied (no ACL ⇒ no grants — never "unprotected ⇒ open").
    #[tokio::test]
    async fn subscribe_denied_when_no_acl_governs_the_topic() {
        let resp = subscribe(state(), web_id_token(ALICE), TOPIC).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    /// A topic outside this server's base URL has no ACL hierarchy here, so there is nothing to
    /// authorize — refused rather than subscribed-to. (Before WAC, any IRI string was accepted.)
    #[tokio::test]
    async fn subscribe_rejects_topic_off_this_server() {
        let resp = subscribe(
            state_granting_read_to(ALICE).await,
            web_id_token(ALICE),
            "https://elsewhere.example/secret",
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// The topic must be the CANONICAL resource IRI: a traversal path is refused by the same
    /// `parse_target` the LDP routes use, so it can never be authorized against one IRI and
    /// registered under another.
    #[tokio::test]
    async fn subscribe_rejects_traversal_topic() {
        let resp = subscribe(
            state_granting_read_to(ALICE).await,
            web_id_token(ALICE),
            "https://pod.example/alice/../bob/private",
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Watching an `.acl` needs `acl:Control`, mirroring the LDP read mapping exactly — plain Read on
    /// the pod is not enough.
    #[tokio::test]
    async fn subscribe_to_an_acl_topic_requires_control_not_read() {
        let resp = subscribe(
            state_granting_read_to(ALICE).await,
            web_id_token(ALICE),
            "https://pod.example/a.acl",
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // --- per-resource WAC on receive (#4877) --------------------------------------------------

    /// A token minted for a WebID that (now) cannot read the topic does NOT authorize the upgrade:
    /// the receive endpoint re-runs WAC against the token's bound WebID, so the check cannot be
    /// bypassed by holding a token whose grant has been revoked.
    #[tokio::test]
    async fn receive_denied_when_bound_web_id_cannot_read_topic() {
        let s = state_granting_read_to(ALICE).await;
        // Mint directly for Bob — the shape of a token issued before Bob's access was revoked.
        let token = s.hub().mint_receive_token(BOB, TOPIC).await;
        let resp = receive_handler(
            State(s.clone()),
            Query(ReceiveQuery {
                topic: Some(TOPIC.to_string()),
                token: Some(token),
            }),
            HeaderMap::new(),
            Err(upgrade_rejection().await),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        // No socket, and no subscriber was registered for the topic.
        assert_eq!(s.hub().subscriber_count(TOPIC).await, 0);
    }

    /// The complement: an AUTHORIZED bound WebID gets past the token + WAC gates and reaches the WS
    /// upgrade extractor — whose own rejection (this is not a real upgrade request) is what surfaces.
    /// That 426-family status, rather than a 403, is the witness that authorization passed.
    #[tokio::test]
    async fn receive_authorized_bound_web_id_reaches_the_upgrade() {
        let s = state_granting_read_to(ALICE).await;
        let token = s.hub().mint_receive_token(ALICE, TOPIC).await;
        let resp = receive_handler(
            State(s.clone()),
            Query(ReceiveQuery {
                topic: Some(TOPIC.to_string()),
                token: Some(token),
            }),
            HeaderMap::new(),
            Err(upgrade_rejection().await),
        )
        .await;
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(s.hub().subscriber_count(TOPIC).await, 0);
    }

    #[tokio::test]
    async fn storage_description_advertises_subscription_service() {
        let resp = storage_description_handler(State(state())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_to_string(resp).await;
        assert!(body.contains("notificationChannel"), "{body}");
        assert!(body.contains(WEBSOCKET_CHANNEL_2023_TYPE), "{body}");
        assert!(
            body.contains("https://pod.example/.notifications/WebSocketChannel2023/"),
            "{body}"
        );
    }

    /// Drain a Response body to a String (test helper).
    async fn body_to_string(resp: Response) -> String {
        use http_body_util::BodyExt;
        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("body collects")
            .to_bytes();
        String::from_utf8(bytes.to_vec()).expect("utf8 body")
    }
}
