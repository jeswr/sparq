// [GPT-5.6] sq-oprna.2
//! HTTP/3 integration coverage for the Solid/LDP server's default-off `http3` feature.
//!
//! The test serves one production `build_router_with_overload` router over both the existing TLS
//! TCP path and the QUIC helper. A throwaway test CA anchors the checked-in localhost certificate.
//! It proves a public LDP GET has the same status and body over HTTP/1.1 and HTTP/3, then exhausts a
//! tight per-IP bucket. The following HTTP/3 request must be 429: if the QUIC peer is not injected as
//! `ConnectInfo<SocketAddr>`, the limiter fails open to the public LDP handler and returns 200 instead.

#![cfg(feature = "http3")]

mod common;

use std::{error::Error, future::poll_fn, sync::Arc, time::Duration};

use axum::body::{Body, Bytes};
use axum::http::{Method, Request, StatusCode};
use bytes::Buf as _;
use common::{jwks_provider, KeyKit, BASE_URL};
use h3::quic::OpenStreams;
use quinn::crypto::rustls::QuicClientConfig;
use rustls_pemfile::certs;
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_http3::{quic_server_config, serve_h3};
use sparq_lws_core::app::{build_router_with_overload, AppState, OverloadConfig};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::overload::AdmissionControl;
use sparq_lws_core::rate_limit::RateLimiter;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient, Store};
use sparq_lws_core::tls::{build_rustls_config, TlsMode};
use tower::ServiceExt as _;

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

const CERT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test-cert.pem");
const KEY_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test-key.pem");
const CA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test-ca.pem");
const RESOURCE_BODY: &str =
    "<https://pod.example/pub#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";

async fn production_router(rate: f64, burst: f64) -> axum::Router {
    let issuer_key = KeyKit::generate();
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
    let auth = AuthContext::new(verifier, BASE_URL);
    let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());

    store
        .write(
            "https://pod.example/pub",
            Bytes::from_static(RESOURCE_BODY.as_bytes()),
            "text/turtle",
        )
        .await
        .expect("seed public LDP resource");
    store
        .write(
            "https://pod.example/pub.acl",
            Bytes::from_static(
                br#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
@prefix foaf: <http://xmlns.com/foaf/0.1/>.
<#public> a acl:Authorization;
  acl:agentClass foaf:Agent;
  acl:accessTo <https://pod.example/pub>;
  acl:mode acl:Read."#,
            ),
            "text/turtle",
        )
        .await
        .expect("seed public resource ACL");

    let overload = OverloadConfig {
        admission: AdmissionControl::new(64),
        request_timeout: None,
        rate_limiter: Some(RateLimiter::new(
            rate, burst, /* trusted_proxy_hops */ 0, /* exempt_loopback */ false,
            /* exempt_internal */ false,
        )),
        body_limit_bytes: sparq_lws_core::body_limit::DEFAULT_MAX_BODY_BYTES,
    };
    let ldp = LdpState::new(store, BASE_URL);
    build_router_with_overload(AppState::new(auth, ldp), overload)
}

fn fixture_roots() -> TestResult<rustls::RootCertStore> {
    let pem = std::fs::read(CA_PATH)?;
    let mut roots = rustls::RootCertStore::empty();
    for cert in certs(&mut pem.as_slice()) {
        roots.add(cert?)?;
    }
    Ok(roots)
}

fn h3_client_endpoint() -> TestResult<quinn::Endpoint> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(fixture_roots()?)
        .with_no_client_auth();
    tls.alpn_protocols = vec![b"h3".to_vec()];

    let client_config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(tls)?));
    let mut endpoint = quinn::Endpoint::client("127.0.0.1:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

async fn h3_get<O>(
    sender: &mut h3::client::SendRequest<O, Bytes>,
    uri: &str,
) -> TestResult<(StatusCode, Bytes)>
where
    O: OpenStreams<Bytes>,
{
    let request = Request::builder().method(Method::GET).uri(uri).body(())?;
    let mut stream = sender.send_request(request).await?;
    stream.finish().await?;

    let response = stream.recv_response().await?;
    let mut body = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await? {
        let remaining = chunk.remaining();
        body.extend_from_slice(&chunk.copy_to_bytes(remaining));
    }
    Ok((response.status(), Bytes::from(body)))
}

#[tokio::test]
async fn h3_ldp_get_matches_h1_and_carries_peer_connect_info() -> TestResult {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Two requests fit: one h1 baseline and one h3 parity request. The next h3 request must be 429.
    let app = production_router(0.0001, 2.0).await;
    let mode = TlsMode::Tls {
        cert_path: CERT_PATH.into(),
        key_path: KEY_PATH.into(),
    };
    let tcp_tls = build_rustls_config(&mode, false)
        .await?
        .expect("TLS mode yields a config");
    assert_eq!(
        tcp_tls.get_inner().alpn_protocols,
        vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        "the TCP listener's ALPN contract must remain unchanged"
    );

    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let server_addr = tcp_listener.local_addr()?;
    let std_listener = tcp_listener.into_std()?;
    std_listener.set_nonblocking(true)?;

    // Clone before adding h3: QUIC and TCP reuse cert/provider/policy, but never share ALPN mutation.
    let mut quic_tls = (*tcp_tls.get_inner()).clone();
    quic_tls.alpn_protocols = vec![b"h3".to_vec()];
    let h3_endpoint = quinn::Endpoint::server(quic_server_config(quic_tls)?, server_addr)?;
    let h3_addr = h3_endpoint.local_addr()?;

    let tcp_handle = axum_server::Handle::new();
    let tcp_server_handle = tcp_handle.clone();
    let tcp_app = app
        .clone()
        .layer(sparq_http3::alt_svc_layer(h3_addr.port()));
    let tcp_server = tokio::spawn(async move {
        axum_server::from_tcp_rustls(std_listener, tcp_tls)
            .expect("construct TLS server")
            .handle(tcp_server_handle)
            .serve(tcp_app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
    });

    let (h3_shutdown_tx, h3_shutdown_rx) = tokio::sync::oneshot::channel();
    let h3_server = tokio::spawn(serve_h3(h3_endpoint, app, async move {
        let _ = h3_shutdown_rx.await;
    }));

    let ca_pem = std::fs::read(CA_PATH)?;
    let ca = reqwest::Certificate::from_pem(&ca_pem)?;
    let h1_client = reqwest::Client::builder()
        .add_root_certificate(ca)
        .resolve("localhost", server_addr)
        .use_rustls_tls()
        .http1_only()
        .build()?;
    let resource_uri = format!("https://localhost:{}/pub", server_addr.port());
    let h1_response = h1_client.get(&resource_uri).send().await?;
    assert_eq!(h1_response.version(), reqwest::Version::HTTP_11);
    let expected_alt_svc = format!("h3=\":{}\"; ma=86400", h3_addr.port());
    assert_eq!(
        h1_response
            .headers()
            .get("alt-svc")
            .and_then(|value| value.to_str().ok()),
        Some(expected_alt_svc.as_str()),
        "the h1 GET response must advertise the live h3 listener port"
    );
    let h1_status = h1_response.status();
    let h1_body = h1_response.bytes().await?;
    assert_eq!(h1_status, StatusCode::OK);
    assert_eq!(h1_body, RESOURCE_BODY);

    let client_endpoint = h3_client_endpoint()?;
    let connection = client_endpoint.connect(server_addr, "localhost")?.await?;
    let (mut driver, mut sender) = h3::client::new(h3_quinn::Connection::new(connection)).await?;
    let driver = tokio::spawn(async move {
        let _ = poll_fn(|context| driver.poll_close(context)).await;
    });

    let (h3_status, h3_body) = h3_get(&mut sender, &resource_uri).await?;
    assert_eq!(h3_status, h1_status, "h3 and h1 LDP status must match");
    assert_eq!(h3_body, h1_body, "h3 and h1 LDP bodies must match");

    let (limited_status, _) = h3_get(&mut sender, &resource_uri).await?;
    assert_eq!(
        limited_status,
        StatusCode::TOO_MANY_REQUESTS,
        "the third same-IP request must be rate-limited; a missing QUIC ConnectInfo would fail open \
         to the public handler and return 200"
    );

    drop(sender);
    client_endpoint.close(quinn::VarInt::from_u32(0), b"test complete");
    tokio::time::timeout(Duration::from_secs(5), driver).await??;
    h3_shutdown_tx
        .send(())
        .map_err(|()| "h3 server stopped before shutdown")?;
    tokio::time::timeout(Duration::from_secs(5), h3_server).await???;

    drop(h1_client);
    tcp_handle.graceful_shutdown(Some(Duration::from_secs(1)));
    tokio::time::timeout(Duration::from_secs(5), tcp_server).await???;
    Ok(())
}

#[tokio::test]
async fn unconfigured_http3_router_does_not_advertise_alt_svc() -> TestResult {
    let app = production_router(100.0, 100.0).await;
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("{BASE_URL}/pub"))
                .body(Body::empty())?,
        )
        .await?;

    assert!(
        response.headers().get("alt-svc").is_none(),
        "the original TCP router must not advertise HTTP/3 before a QUIC endpoint binds"
    );
    Ok(())
}
