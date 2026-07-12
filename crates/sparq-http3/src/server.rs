use std::{future::Future, io, net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response},
    Router,
};
use bytes::{Buf, Bytes};
use futures_util::stream;
use h3::server::RequestStream;
use http_body_util::BodyExt as _;
use quinn::crypto::rustls::{NoInitialCipherSuite, QuicServerConfig};
use tower::ServiceExt as _;

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

/// Converts an aws-lc-rs-backed rustls server configuration into Quinn's server config.
///
/// The input must advertise the exact `h3` ALPN token. Certificate loading, client-auth
/// policy, and transport tuning remain the caller's responsibility.
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
    Ok(quinn::ServerConfig::with_crypto(Arc::new(crypto)))
}

/// Serves HTTP/3 connections from `endpoint` until `shutdown` resolves.
///
/// Every request is dispatched through a clone of `router`, and the QUIC peer address is
/// inserted as `ConnectInfo<SocketAddr>`. Connection- and request-local protocol failures
/// close only the affected connection or stream; the endpoint keeps accepting other peers.
/// On shutdown, all endpoint connections are closed and the function waits for them to drain.
pub async fn serve_h3(
    endpoint: quinn::Endpoint,
    router: Router,
    shutdown: impl Future<Output = ()>,
) -> io::Result<()> {
    tokio::pin!(shutdown);

    loop {
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
                let router = router.clone();
                tokio::spawn(async move {
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
