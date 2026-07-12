#![cfg(feature = "server")]

use std::{error::Error, future::poll_fn, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    body::Bytes,
    extract::ConnectInfo,
    http::{Method, Request, StatusCode},
    routing::{get, post},
    Router,
};
use bytes::Buf as _;
use h3::quic::OpenStreams;
use quinn::crypto::rustls::QuicClientConfig;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use sparq_http3::{quic_server_config, serve_h3, Http3ConfigError};

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

fn certificate() -> TestResult<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec!["localhost".to_owned()])?;
    Ok((
        cert.der().clone(),
        PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into(),
    ))
}

fn rustls_server_config(
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
) -> TestResult<rustls::ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    Ok(rustls::ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?)
}

fn client_endpoint(cert: CertificateDer<'static>) -> TestResult<quinn::Endpoint> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert)?;
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![b"h3".to_vec()];

    let client_config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(tls)?));
    let mut endpoint = quinn::Endpoint::client("127.0.0.1:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

async fn request<O>(
    sender: &mut h3::client::SendRequest<O, Bytes>,
    method: Method,
    uri: String,
    body: Option<Bytes>,
) -> TestResult<(StatusCode, Bytes)>
where
    O: OpenStreams<Bytes>,
{
    let request = Request::builder().method(method).uri(uri).body(())?;
    let mut stream = sender.send_request(request).await?;
    if let Some(body) = body {
        stream.send_data(body).await?;
    }
    stream.finish().await?;

    let response = stream.recv_response().await?;
    let mut body = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await? {
        let remaining = chunk.remaining();
        body.extend_from_slice(&chunk.copy_to_bytes(remaining));
    }
    Ok((response.status(), Bytes::from(body)))
}

#[test]
fn config_rejects_a_server_without_h3_alpn() -> TestResult {
    let (cert, key) = certificate()?;
    let config = rustls_server_config(cert, key)?;

    let error = quic_server_config(config).expect_err("missing h3 ALPN must fail closed");
    assert!(matches!(error, Http3ConfigError::MissingH3Alpn));
    Ok(())
}

#[tokio::test]
async fn h3_dispatches_body_status_and_peer_connect_info() -> TestResult {
    async fn echo(body: Bytes) -> (StatusCode, Bytes) {
        (StatusCode::CREATED, body)
    }

    async fn peer(ConnectInfo(peer): ConnectInfo<SocketAddr>) -> String {
        peer.to_string()
    }

    let (cert, key) = certificate()?;
    let mut tls = rustls_server_config(cert.clone(), key)?;
    tls.alpn_protocols = vec![b"h3".to_vec()];
    let endpoint = quinn::Endpoint::server(quic_server_config(tls)?, "127.0.0.1:0".parse()?)?;
    let server_addr = endpoint.local_addr()?;
    let router = Router::new()
        .route("/echo", post(echo))
        .route("/peer", get(peer));

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(serve_h3(endpoint, router, async move {
        let _ = shutdown_rx.await;
    }));

    let client_endpoint = client_endpoint(cert)?;
    let client_addr = client_endpoint.local_addr()?;
    let connection = client_endpoint.connect(server_addr, "localhost")?.await?;
    let (mut driver, mut sender) = h3::client::new(h3_quinn::Connection::new(connection)).await?;
    let driver = tokio::spawn(async move {
        let _ = poll_fn(|context| driver.poll_close(context)).await;
    });

    let (status, body) = request(
        &mut sender,
        Method::POST,
        format!("https://localhost:{}/echo", server_addr.port()),
        Some(Bytes::from_static(b"request body crosses h3 and axum")),
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body, "request body crosses h3 and axum");

    let (status, body) = request(
        &mut sender,
        Method::GET,
        format!("https://localhost:{}/peer", server_addr.port()),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, client_addr.to_string());

    drop(sender);
    client_endpoint.close(quinn::VarInt::from_u32(0), b"test complete");
    tokio::time::timeout(Duration::from_secs(5), driver).await??;
    shutdown_tx
        .send(())
        .map_err(|()| "server stopped before shutdown")?;
    tokio::time::timeout(Duration::from_secs(5), server).await???;
    Ok(())
}
