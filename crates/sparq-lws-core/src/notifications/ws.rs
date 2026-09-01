// AUTHORED-BY Claude Opus 4.8
//! The WebSocketChannel2023 HTTP surface: discovery, subscribe, and the WS receive endpoint.
//!
//! Route layout (mounted by [`crate::app::build_router`]):
//! - `POST /.notifications/WebSocketChannel2023/`  — subscribe; returns a channel description with a
//!   `receiveFrom` `ws(s)://` URL that carries a minted receive token. **Auth-gated** (behind the
//!   DPoP middleware — fail-closed) AND **WAC-gated**: the authenticated WebID must hold `acl:Read`
//!   on the topic, else 403 and no channel. The token binds receive to that subscriber+topic.
//! - `GET  /.notifications/WebSocketChannel2023/receive?topic=<iri>&token=<tok>` — upgrade to a
//!   WebSocket and register the connection under `<iri>`. **Token-gated:** the `token` must be a
//!   valid, unexpired receive token whose bound topic matches `<iri>`, else the upgrade is rejected
//!   (401, no socket). **Then WAC-gated:** the WebID the token is bound to must STILL hold
//!   `acl:Read` on `<iri>` (re-checked, so a revocation takes effect before the token expires), else
//!   403 and no socket. The server then pushes AS2.0 notifications on change.
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
use crate::authz::{AccessMode, Decision, WacAuthorizer};
use crate::ldp::handler::{request_origin, LdpState};
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

/// State for the notification routes: the hub + the server's public base URL (for building the
/// absolute `receiveFrom` / subscription-service URLs in discovery + subscribe responses) + the LDP
/// state handle the WAC authorizer reads each topic's `.acl` through.
///
/// All three are derived from ONE [`LdpState`] handle so they cannot drift: the hub is the same
/// registry the LDP write path emits into, the base URL is the same one the LDP routes parse targets
/// against, and the store + ACL cache are the same ones the LDP routes authorize through. Carrying
/// the store handle here is what lets [`subscribe_handler`] and [`receive_handler`] evaluate the
/// topic's own `.acl` rather than trusting authentication alone.
pub struct NotifyState<S: Store> {
    pub hub: NotificationHub,
    pub base_url: String,
    /// The LDP state — the [`Store`] the topic's `.acl` is read through plus the shared
    /// [`AclCache`](crate::acl_cache::AclCache). Private: it is an authorization capability, not
    /// route configuration.
    ldp: Arc<LdpState<S>>,
}

// Hand-written rather than `#[derive(Clone)]`: the derive would generate an `S: Clone` bound, and a
// `Store` implementation is generally NOT `Clone` (the composite store owns its backends), so the
// derived impl would exist for almost no `S`. Only the shared handles are cloned here.
impl<S: Store> Clone for NotifyState<S> {
    fn clone(&self) -> Self {
        Self {
            hub: self.hub.clone(),
            base_url: self.base_url.clone(),
            ldp: Arc::clone(&self.ldp),
        }
    }
}

impl<S: Store> NotifyState<S> {
    /// Build the notification state from the LDP state handle, sharing its hub, base URL, store and
    /// ACL cache.
    pub fn new(ldp: Arc<LdpState<S>>) -> Self {
        Self {
            hub: ldp.notifications.clone(),
            base_url: ldp.base_url().to_string(),
            ldp,
        }
    }

    /// Authorize `web_id` to READ `topic` under Web Access Control — the SAME
    /// [`WacAuthorizer`] (and the same shared parsed-ACL cache) the LDP read path uses, so a
    /// subscription can never observe a resource the requester could not simply `GET`.
    ///
    /// Returns `Ok(())` when permitted, or `Err(response)` — the denial to return verbatim — when not.
    ///
    /// Fail-closed in three ways:
    /// - a topic OUTSIDE this server's `base_url` is refused without consulting any ACL: this server
    ///   holds no `.acl` governing a foreign IRI, so an ACL walk would fall through to the storage
    ///   root's `acl:default` and wrongly apply the root's grants to someone else's resource;
    /// - a topic with NO governing ACL anywhere grants nothing ([`WacAuthorizer`]'s own fail-closed
    ///   resolution) ⇒ denied;
    /// - a storage error while resolving the ACL is a 500, never a silent allow.
    ///
    /// The denial is deliberately NOT conditioned on whether the topic EXISTS — no ACL read touches
    /// the topic's own bytes — so this adds no existence oracle (the same property the LDP routes
    /// preserve).
    // [OPUS-5] `Response` is a large type (128 bytes — at clippy's `large-error-threshold`), so
    // carrying it in the `Err` arm trips `clippy::result_large_err` under the workspace's
    // `-D warnings` gate. Returning the ready-made denial is the point: both call sites `return` it
    // verbatim, so the handler cannot accidentally turn a deny into an allow. Boxing would only move
    // the allocation and force a deref at each `return`. Same allow, same reason, as the
    // `Result<_, Response>` handler helpers in `sparq-server::solid_authz`.
    #[allow(clippy::result_large_err)]
    async fn authorize_topic_read(
        &self,
        topic: &str,
        web_id: &str,
        origin: Option<&str>,
    ) -> Result<(), Response> {
        if !self.topic_is_local(topic) {
            return Err((StatusCode::FORBIDDEN, "forbidden").into_response());
        }
        let wac = WacAuthorizer::with_cache(&self.ldp.store, &self.base_url, &self.ldp.acl_cache);
        match wac
            .authorize(topic, AccessMode::Read, Some(web_id), origin)
            .await
        {
            Ok(Decision::Allow(_)) => Ok(()),
            Ok(Decision::Forbidden) => Err((StatusCode::FORBIDDEN, "forbidden").into_response()),
            // Unreachable in practice: both call sites pass an authenticated WebID (subscribe is
            // behind the auth middleware; receive takes the WebID the token is bound to). Mapped to
            // the same 401 the LDP layer emits so the fallback is fail-closed rather than a panic.
            Ok(Decision::Unauthenticated) => Err((
                StatusCode::UNAUTHORIZED,
                "authentication required to subscribe",
            )
                .into_response()),
            // A transient storage failure must never read as "allowed"; it is also NOT mapped to
            // 403/404, which would let a storage blip masquerade as an access decision.
            Err(_) => Err(
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response(),
            ),
        }
    }

    /// Whether `topic` names a resource under THIS server's storage root — the only IRIs whose
    /// `.acl` this server is authoritative for. See [`authorize_topic_read`](Self::authorize_topic_read).
    fn topic_is_local(&self, topic: &str) -> bool {
        let root = format!("{}/", self.base_url.trim_end_matches('/'));
        topic.starts_with(&root)
    }

    /// The absolute subscription-service URL (the POST target).
    fn subscription_service_url(&self) -> String {
        format!("{}{SUBSCRIPTION_PATH}", self.base_url.trim_end_matches('/'))
    }

    /// The `receiveFrom` WebSocket URL for a topic, carrying the minted receive `token`. The base
    /// URL's scheme is mapped http→ws / https→wss (WebSocketChannel2023 §receiveFrom — the receive
    /// endpoint is a WebSocket URL). The token authorizes the WS upgrade for this topic (a browser
    /// `WebSocket` cannot send the DPoP `Authorization` header, so the spec carries authz in the URL).
    fn receive_from_url(&self, topic: &str, token: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
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

/// `POST /.notifications/WebSocketChannel2023/` — subscribe to a topic.
///
/// **Auth (fail-closed):** the caller MUST be authenticated (a WebID). An anonymous/public caller is
/// rejected with 401 — there are NO anonymous subscriptions. (This handler runs behind the DPoP auth
/// middleware, which injects the [`VerifiedToken`]; `is_public()` ⇒ unauthenticated.)
///
/// **Authorization (fail-closed):** the authenticated WebID must additionally hold `acl:Read` on the
/// topic under Web Access Control — evaluated by the SAME [`WacAuthorizer`] the LDP routes use, via
/// `NotifyState::authorize_topic_read`. A caller with no Read on the topic gets 403 and NO channel,
/// so a subscription can never surface changes to a resource the caller could not `GET`.
///
/// Order: authenticate → validate the request shape (400s for a wrong channel type / missing topic,
/// which are request errors independent of any ACL) → authorize the topic → mint. Authorization runs
/// as soon as there IS a topic to authorize, and strictly before anything is minted.
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

    // Per-resource WAC: this WebID must hold `acl:Read` on the topic. Runs BEFORE anything is minted,
    // so a denied subscribe leaves no token and creates no channel.
    if let Err(denied) = state
        .authorize_topic_read(topic, &web_id, request_origin(&headers))
        .await
    {
        return denied;
    }

    // Mint the receive token: unguessable, short-lived, bound to (this authenticated WebID, topic).
    // Without it the receive endpoint refuses the upgrade — so only this authenticated subscriber of
    // this topic can connect. The token (never logged) is embedded in `receiveFrom`.
    let receive_token = state.hub.mint_receive_token(&web_id, topic).await;
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
/// ## Per-resource WAC on the upgrade (re-checked, not inherited from the token)
/// The token proves only that the connecting party completed an authenticated subscribe to THIS
/// topic. Because it stays valid for its whole TTL, that alone would let a WebID keep receiving after
/// its `acl:Read` was revoked. So the handler resolves the token to the WebID it is BOUND to
/// ([`NotificationHub::resolve_receive_token`]) and re-runs the same WAC `Read` check as subscribe
/// (`NotifyState::authorize_topic_read`) — a revoked subscriber is refused the upgrade with 403 and
/// no subscriber is registered.
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
    let Some(web_id) = state.hub.resolve_receive_token(&token, &topic).await else {
        return (
            StatusCode::UNAUTHORIZED,
            "a valid receive token is required",
        )
            .into_response();
    };
    // The token validated and named its subscriber. Re-run per-resource WAC against THAT WebID, so a
    // grant revoked since the subscribe is honoured immediately instead of at token expiry.
    if let Err(denied) = state
        .authorize_topic_read(&topic, &web_id, request_origin(&headers))
        .await
    {
        return denied;
    }
    // The token validated. Now the request MUST be a genuine WS upgrade; surface the extractor's own
    // rejection (e.g. 426 Upgrade Required) if not.
    let ws = match ws {
        Ok(ws) => ws,
        Err(rej) => return rej.into_response(),
    };
    // Only AFTER the token validates do we upgrade + register a subscriber.
    let hub = state.hub.clone();
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

    type TestStore = CompositeStore<InMemorySparqClient, InMemoryBlobStore>;

    /// A state over an EMPTY store — no `.acl` anywhere, so WAC grants nothing (fail-closed). Used by
    /// the URL-shape tests (which never authorize) and by the no-ACL denial test.
    fn state() -> Arc<NotifyState<TestStore>> {
        Arc::new(NotifyState::new(Arc::new(LdpState::new(
            CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new()),
            BASE,
        ))))
    }

    /// A state whose storage root carries an `.acl` granting `owner` Read (+ `acl:default`, so every
    /// descendant inherits it). Any OTHER WebID holds nothing — the denial case.
    async fn state_with_root_read(owner: &str) -> Arc<NotifyState<TestStore>> {
        let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
        let acl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
<#owner> a acl:Authorization;
         acl:agent <{}>;
         acl:accessTo <{BASE}/>;
         acl:default <{BASE}/>;
         acl:mode acl:Read."#,
            owner
        );
        store
            .write(&format!("{BASE}/.acl"), Bytes::from(acl), "text/turtle")
            .await
            .expect("seed root acl");
        Arc::new(NotifyState::new(Arc::new(LdpState::new(store, BASE))))
    }

    /// Build a subscribe request for `topic` as `web_id`.
    async fn subscribe_as(
        state: Arc<NotifyState<TestStore>>,
        web_id: &str,
        topic: &str,
    ) -> Response {
        let token = VerifiedToken {
            web_id: Some(web_id.to_string()),
            ..VerifiedToken::default()
        };
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

    /// A `WebSocketUpgrade` extraction outcome for a request that is NOT a WS upgrade — i.e. an
    /// `Err(rejection)`. The authorization gates in `receive_handler` all run BEFORE this value is
    /// inspected, so a request that reaches the extractor surfaces its (non-401/403) rejection —
    /// which is exactly how the tests below tell "authorized" apart from "denied".
    async fn non_upgrade_ws() -> Result<WebSocketUpgrade, WebSocketUpgradeRejection> {
        use axum::extract::FromRequestParts;
        let req = axum::http::Request::builder()
            .uri("/")
            .body(axum::body::Body::empty())
            .expect("build request");
        let (mut parts, _) = req.into_parts();
        WebSocketUpgrade::from_request_parts(&mut parts, &()).await
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
        let resp = subscribe_handler(
            State(state()),
            Extension(VerifiedToken::public()),
            HeaderMap::new(),
            Json(SubscriptionRequest {
                channel_type: Some(WEBSOCKET_CHANNEL_2023_TYPE.to_string()),
                topic: Some("https://pod.example/a".to_string()),
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn subscribe_handler_accepts_authenticated_and_returns_receive_from() {
        // Alice holds `acl:Read` on the topic (inherited from the root `acl:default`), so the WAC
        // gate permits the subscribe.
        let resp = subscribe_as(
            state_with_root_read(ALICE).await,
            ALICE,
            "https://pod.example/a",
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
        let token = VerifiedToken {
            web_id: Some("https://alice.example/profile#me".to_string()),
            ..VerifiedToken::default()
        };
        let resp = subscribe_handler(
            State(state()),
            Extension(token),
            HeaderMap::new(),
            Json(SubscriptionRequest {
                channel_type: Some("http://example/OtherChannel".to_string()),
                topic: Some("https://pod.example/a".to_string()),
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn subscribe_handler_rejects_missing_topic() {
        let token = VerifiedToken {
            web_id: Some("https://alice.example/profile#me".to_string()),
            ..VerifiedToken::default()
        };
        let resp = subscribe_handler(
            State(state()),
            Extension(token),
            HeaderMap::new(),
            Json(SubscriptionRequest {
                channel_type: None,
                topic: None,
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // --- per-resource WAC on subscribe (issue #4877) --------------------------------------------

    /// THE acceptance case: an AUTHENTICATED WebID with no `acl:Read` on the topic is refused. Before
    /// per-resource WAC was wired, this returned 200 + a `receiveFrom` channel.
    #[tokio::test]
    async fn subscribe_denied_for_topic_the_webid_cannot_read() {
        // The root ACL grants Read to Alice ONLY; Bob is authenticated but holds nothing.
        let resp = subscribe_as(
            state_with_root_read(ALICE).await,
            BOB,
            "https://pod.example/alice/private",
        )
        .await;
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "an authenticated WebID without acl:Read on the topic must not get a channel"
        );
        // ...and it hands back no channel to connect with.
        let body = body_to_string(resp).await;
        assert!(
            !body.contains("receiveFrom"),
            "a denied subscribe must not return a receiveFrom URL: {body}"
        );
    }

    /// Fail-closed: no `.acl` governs the topic anywhere, so nobody — including an authenticated
    /// caller — is granted Read.
    #[tokio::test]
    async fn subscribe_denied_when_no_acl_governs_the_topic() {
        let resp = subscribe_as(state(), ALICE, "https://pod.example/a").await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    /// A topic outside this server's storage root is refused outright — otherwise the ancestor walk
    /// would fall through to the ROOT `.acl` and apply this pod's grants to a foreign resource.
    #[tokio::test]
    async fn subscribe_denied_for_topic_outside_the_storage_root() {
        let resp = subscribe_as(
            state_with_root_read(ALICE).await,
            ALICE,
            "https://evil.example/someone-elses-resource",
        )
        .await;
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "a foreign topic IRI must not inherit this pod's root acl:default"
        );
    }

    // --- per-resource WAC on receive ------------------------------------------------------------

    /// A receive token stays valid for its whole TTL, so the upgrade must RE-CHECK WAC rather than
    /// trust token possession. Here the token is genuine and topic-matching, but no ACL grants its
    /// WebID Read — the upgrade is refused with 403 (an access decision), not 401 (a token problem).
    #[tokio::test]
    async fn receive_denied_when_token_holder_lacks_read() {
        let s = state();
        let topic = "https://pod.example/a";
        let tok = s.hub.mint_receive_token(ALICE, topic).await;
        let resp = receive_handler(
            State(s),
            Query(ReceiveQuery {
                topic: Some(topic.to_string()),
                token: Some(tok),
            }),
            HeaderMap::new(),
            non_upgrade_ws().await,
        )
        .await;
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "a valid token must not authorize receive once the WebID lacks acl:Read"
        );
    }

    /// The converse: a token holder who DOES hold Read passes both gates and reaches the WebSocket
    /// extractor — whose rejection (this is not a real upgrade request) is neither 401 nor 403.
    #[tokio::test]
    async fn receive_authorized_token_holder_passes_the_wac_gate() {
        let s = state_with_root_read(ALICE).await;
        let topic = "https://pod.example/a";
        let tok = s.hub.mint_receive_token(ALICE, topic).await;
        let resp = receive_handler(
            State(s),
            Query(ReceiveQuery {
                topic: Some(topic.to_string()),
                token: Some(tok),
            }),
            HeaderMap::new(),
            non_upgrade_ws().await,
        )
        .await;
        assert!(
            resp.status() != StatusCode::UNAUTHORIZED && resp.status() != StatusCode::FORBIDDEN,
            "an authorized subscriber must get past the auth gates (got {})",
            resp.status()
        );
    }

    /// The pre-existing token gate still runs FIRST: no token ⇒ 401 before any ACL is consulted.
    #[tokio::test]
    async fn receive_still_rejects_a_missing_token() {
        let resp = receive_handler(
            State(state_with_root_read(ALICE).await),
            Query(ReceiveQuery {
                topic: Some("https://pod.example/a".to_string()),
                token: None,
            }),
            HeaderMap::new(),
            non_upgrade_ws().await,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
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
