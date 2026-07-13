use std::{
    collections::HashMap,
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{HeaderName, HeaderValue, Request, Response},
    Router,
};
use bytes::{Buf, Bytes};
use futures_util::stream;
use h3::server::RequestStream;
use http_body_util::BodyExt as _;
use quinn::crypto::rustls::{NoInitialCipherSuite, QuicServerConfig};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tower::ServiceExt as _;
use tower_http::set_header::SetResponseHeaderLayer;

/// Default ceiling for concurrently served HTTP/3 connections.
const DEFAULT_MAX_CONNECTIONS: usize = 10_000;

/// Default ceiling for concurrent HTTP/3 connections from one non-internal IP address.
const DEFAULT_MAX_CONNECTIONS_PER_IP: usize = 512;

/// Connection-level QUIC receive window used by [`quic_server_config`].
const RECEIVE_WINDOW_BYTES: u32 = 16 * 1024 * 1024;

/// Lifetime advertised for the HTTP/3 alternative service, in seconds.
const ALT_SVC_MAX_AGE_SECONDS: u32 = 86_400;

type PerIpCounts = Arc<Mutex<HashMap<IpAddr, usize>>>;

/// Concurrent-connection limits enforced by [`serve_h3_with_limits`].
///
/// Zero values are clamped to one so a direct configuration cannot accidentally disable the
/// listener. Set [`max_connections_per_ip`](Self::max_connections_per_ip) to `None` to disable only
/// the per-IP limit; the global limit is always enforced.
// [GPT-5.6] sq-h1f9g: keep the compatibility entry point safe by default while allowing tuning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct H3ConnectionLimits {
    /// Maximum number of concurrently served HTTP/3 connections.
    pub max_connections: usize,
    /// Maximum concurrent connections from one IP address, or `None` to disable this limit.
    pub max_connections_per_ip: Option<usize>,
    /// Exempt loopback, private, link-local, and IPv6 unique-local addresses from the per-IP limit.
    ///
    /// The global connection limit still applies to exempt addresses. This defaults to `true` so a
    /// reverse proxy, container bridge, or conformance runner is not treated as a single public
    /// client.
    pub exempt_internal_ips: bool,
}

impl Default for H3ConnectionLimits {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            max_connections_per_ip: Some(DEFAULT_MAX_CONNECTIONS_PER_IP),
            exempt_internal_ips: true,
        }
    }
}

/// A rustls server configuration cannot be used for HTTP/3.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Http3ConfigError {
    /// The HTTP/3 ALPN token is missing.
    #[error("rustls server configuration must include the h3 ALPN protocol")]
    MissingH3Alpn,
    /// The configured crypto provider lacks QUIC's required initial cipher suite.
    #[error("rustls server configuration is not QUIC-compatible: {0}")]
    NoInitialCipherSuite(#[from] NoInitialCipherSuite),
}

/// Builds a response-header layer advertising HTTP/3 on `port`.
///
/// The layer writes `Alt-Svc: h3=":<port>"; ma=86400` on every response and replaces any
/// pre-existing value. Apply it to the HTTP/1.1 or HTTP/2 router only after the corresponding QUIC
/// endpoint has bound successfully. Constructing this layer from configuration alone can advertise
/// a listener that does not exist.
// [GPT-5.6] sq-oprna.4: one exact advertisement format shared by both TCP servers.
pub fn alt_svc_layer(port: u16) -> SetResponseHeaderLayer<HeaderValue> {
    let value = HeaderValue::from_str(&format!("h3=\":{port}\"; ma={ALT_SVC_MAX_AGE_SECONDS}"))
        .expect("a u16 port and fixed Alt-Svc syntax always form a valid header value");
    SetResponseHeaderLayer::overriding(HeaderName::from_static("alt-svc"), value)
}

/// Converts an aws-lc-rs-backed rustls server configuration into Quinn's server config.
///
/// The input must advertise the exact `h3` ALPN token. Certificate loading, client-auth
/// policy, and additional transport tuning remain the caller's responsibility. The returned config
/// owns an explicit bounded connection-level receive window instead of Quinn's effectively unbounded
/// default.
pub fn quic_server_config(
    rustls_config: rustls::ServerConfig,
) -> Result<quinn::ServerConfig, Http3ConfigError> {
    if !rustls_config
        .alpn_protocols
        .iter()
        .any(|protocol| protocol.as_slice() == b"h3")
    {
        return Err(Http3ConfigError::MissingH3Alpn);
    }

    let crypto = QuicServerConfig::try_from(rustls_config)?;
    let mut transport = quinn::TransportConfig::default();
    transport.receive_window(quinn::VarInt::from_u32(RECEIVE_WINDOW_BYTES));
    let mut config = quinn::ServerConfig::with_crypto(Arc::new(crypto));
    config.transport_config(Arc::new(transport));
    Ok(config)
}

/// Serves HTTP/3 connections from `endpoint` until `shutdown` resolves.
///
/// Every request is dispatched through a clone of `router`, and the QUIC peer address is
/// inserted as `ConnectInfo<SocketAddr>`. Connection- and request-local protocol failures
/// close only the affected connection or stream; the endpoint keeps accepting other peers.
/// The default global and per-IP connection limits bound accepted connection work; internal IPs are
/// exempt from the per-IP limit but remain subject to the global limit. On shutdown, all endpoint
/// connections are closed and the function waits for them to drain.
pub async fn serve_h3(
    endpoint: quinn::Endpoint,
    router: Router,
    shutdown: impl Future<Output = ()>,
) -> io::Result<()> {
    serve_h3_with_limits(endpoint, router, H3ConnectionLimits::default(), shutdown).await
}

/// Serves HTTP/3 connections with caller-selected concurrent-connection limits.
///
/// A global semaphore permit is reserved before polling `endpoint.accept()`, so connections beyond
/// the global cap remain in Quinn's bounded incoming queue rather than creating tasks. After accept,
/// a non-exempt peer at its per-IP cap is refused before a task is spawned. Both slots are held for
/// the full handshake and connection lifetime and released on every exit path.
pub async fn serve_h3_with_limits(
    endpoint: quinn::Endpoint,
    router: Router,
    limits: H3ConnectionLimits,
    shutdown: impl Future<Output = ()>,
) -> io::Result<()> {
    tokio::pin!(shutdown);
    let limiter = ConnectionLimiter::new(limits);

    loop {
        let permit = tokio::select! {
            biased;
            () = &mut shutdown => {
                endpoint.close(quinn::VarInt::from_u32(0), b"server shutdown");
                endpoint.wait_idle().await;
                return Ok(());
            }
            permit = limiter.acquire() => permit,
        };

        tokio::select! {
            biased;
            () = &mut shutdown => {
                endpoint.close(quinn::VarInt::from_u32(0), b"server shutdown");
                endpoint.wait_idle().await;
                return Ok(());
            }
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else {
                    return Ok(());
                };
                let Some(ip_guard) = limiter.try_acquire_ip(incoming.remote_address().ip()) else {
                    incoming.refuse();
                    continue;
                };
                let router = router.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    let _ip_guard = ip_guard;
                    let Ok(connection) = incoming.await else {
                        return;
                    };
                    let peer_addr = connection.remote_address();
                    let _ = serve_connection(connection, router, peer_addr).await;
                });
            }
        }
    }
}

#[derive(Clone)]
struct ConnectionLimiter {
    semaphore: Arc<Semaphore>,
    max_per_ip: Option<usize>,
    exempt_internal_ips: bool,
    per_ip: PerIpCounts,
}

impl ConnectionLimiter {
    fn new(limits: H3ConnectionLimits) -> Self {
        let max_connections = limits.max_connections.clamp(1, Semaphore::MAX_PERMITS);
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            max_per_ip: limits.max_connections_per_ip.map(|limit| limit.max(1)),
            exempt_internal_ips: limits.exempt_internal_ips,
            per_ip: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn acquire(&self) -> OwnedSemaphorePermit {
        self.semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("the private HTTP/3 connection semaphore is never closed")
    }

    fn try_acquire_ip(&self, ip: IpAddr) -> Option<IpConnectionGuard> {
        let Some(max_per_ip) = self.max_per_ip else {
            return Some(IpConnectionGuard::disabled());
        };
        if self.exempt_internal_ips && is_internal_ip(ip) {
            return Some(IpConnectionGuard::disabled());
        }

        let mut counts = match self.per_ip.lock() {
            Ok(counts) => counts,
            Err(poisoned) => poisoned.into_inner(),
        };
        let count = counts.get(&ip).copied().unwrap_or(0);
        if count >= max_per_ip {
            return None;
        }
        counts.insert(ip, count + 1);
        Some(IpConnectionGuard::tracked(self.per_ip.clone(), ip))
    }
}

struct IpConnectionGuard {
    tracked: Option<(PerIpCounts, IpAddr)>,
}

impl IpConnectionGuard {
    fn disabled() -> Self {
        Self { tracked: None }
    }

    fn tracked(counts: PerIpCounts, ip: IpAddr) -> Self {
        Self {
            tracked: Some((counts, ip)),
        }
    }
}

impl Drop for IpConnectionGuard {
    fn drop(&mut self) {
        let Some((counts, ip)) = &self.tracked else {
            return;
        };
        let mut counts = match counts.lock() {
            Ok(counts) => counts,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(count) = counts.get_mut(ip) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.remove(ip);
            }
        }
    }
}

fn is_internal_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
        IpAddr::V6(ip) => {
            if ip.is_loopback() {
                return true;
            }
            if let Some(ipv4) = ip.to_ipv4() {
                return ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local();
            }
            let prefix = ip.segments()[0];
            (prefix & 0xfe00) == 0xfc00 || (prefix & 0xffc0) == 0xfe80
        }
    }
}

async fn serve_connection(
    connection: quinn::Connection,
    router: Router,
    peer_addr: SocketAddr,
) -> Result<(), h3::error::ConnectionError> {
    let mut connection = h3::server::Connection::new(h3_quinn::Connection::new(connection)).await?;

    while let Some(resolver) = connection.accept().await? {
        let router = router.clone();
        tokio::spawn(async move {
            let Ok((request, stream)) = resolver.resolve_request().await else {
                return;
            };
            let _ = dispatch_request(request, stream, router, peer_addr).await;
        });
    }

    Ok(())
}

async fn dispatch_request<S>(
    request: Request<()>,
    stream: RequestStream<S, Bytes>,
    router: Router,
    peer_addr: SocketAddr,
) -> Result<(), h3::error::StreamError>
where
    S: h3::quic::BidiStream<Bytes> + Send + 'static,
    S::SendStream: Send,
    S::RecvStream: Send,
{
    let (mut send, recv) = stream.split();
    let body_stream = stream::try_unfold(recv, |mut recv| async move {
        match recv.recv_data().await? {
            Some(mut data) => {
                let remaining = data.remaining();
                Ok::<_, h3::error::StreamError>(Some((data.copy_to_bytes(remaining), recv)))
            }
            None => Ok::<_, h3::error::StreamError>(None),
        }
    });

    let (mut parts, ()) = request.into_parts();
    parts.extensions.insert(ConnectInfo(peer_addr));
    let request = Request::from_parts(parts, Body::from_stream(body_stream));
    let response = router
        .oneshot(request)
        .await
        .expect("axum Router is infallible");

    send_response(&mut send, response).await
}

async fn send_response<S>(
    stream: &mut RequestStream<S, Bytes>,
    response: Response<Body>,
) -> Result<(), h3::error::StreamError>
where
    S: h3::quic::SendStream<Bytes>,
{
    let (parts, mut body) = response.into_parts();
    stream
        .send_response(Response::from_parts(parts, ()))
        .await?;

    while let Some(frame) = body.frame().await {
        let Ok(frame) = frame else {
            stream.stop_stream(h3::error::Code::H3_INTERNAL_ERROR);
            return Ok(());
        };
        match frame.into_data() {
            Ok(data) => stream.send_data(data).await?,
            Err(frame) => {
                if let Ok(trailers) = frame.into_trailers() {
                    stream.send_trailers(trailers).await?;
                    return Ok(());
                }
            }
        }
    }

    stream.finish().await
}
