#![cfg(feature = "http3")]
//! [GPT-5.6] sq-oprna.3: production-binary HTTP/3 wiring and cross-protocol parity.

use std::{
    error::Error,
    future::poll_fn,
    io::Read as _,
    net::{SocketAddr, TcpListener, UdpSocket},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use bytes::{Buf as _, Bytes};
use h3::quic::OpenStreams;
use quinn::crypto::rustls::QuicClientConfig;

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

const QUERY: &str = "SELECT ?x WHERE { VALUES ?x { <http://example.com/http3> } }";
const RESULTS_JSON: &str = "application/sparql-results+json";

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn free_tcp_addr() -> TestResult<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?)
}

fn free_udp_addr() -> TestResult<SocketAddr> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    Ok(socket.local_addr()?)
}

fn h3_client(ca_path: &std::path::Path) -> TestResult<quinn::Endpoint> {
    let ca_file = std::fs::File::open(ca_path)?;
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut std::io::BufReader::new(ca_file)) {
        roots.add(cert?)?;
    }
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![b"h3".to_vec()];
    let config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(tls)?));
    let mut endpoint = quinn::Endpoint::client("127.0.0.1:0".parse()?)?;
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}

async fn h3_query<O>(
    sender: &mut h3::client::SendRequest<O, Bytes>,
    addr: SocketAddr,
) -> TestResult<(u16, Option<String>, Bytes)>
where
    O: OpenStreams<Bytes>,
{
    let request = axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri(format!("https://localhost:{}/sparql", addr.port()))
        .header(axum::http::header::CONTENT_TYPE, "application/sparql-query")
        .header(axum::http::header::ACCEPT, RESULTS_JSON)
        .body(())?;
    let mut stream = sender.send_request(request).await?;
    stream
        .send_data(Bytes::from_static(QUERY.as_bytes()))
        .await?;
    stream.finish().await?;

    let response = stream.recv_response().await?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let mut body = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await? {
        let remaining = chunk.remaining();
        body.extend_from_slice(&chunk.copy_to_bytes(remaining));
    }
    Ok((status, content_type, Bytes::from(body)))
}

#[test]
fn http3_requires_both_certificate_and_key_flags() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_sparq-server"))
        .arg("--http3")
        .output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--http3 requires both --tls-cert FILE and --tls-key FILE"),
        "unexpected stderr: {stderr}"
    );
    Ok(())
}

#[tokio::test]
async fn h3_sparql_response_matches_the_unchanged_http1_listener() -> TestResult {
    let tcp_addr = free_tcp_addr()?;
    let udp_addr = free_udp_addr()?;
    let child = Command::new(env!("CARGO_BIN_EXE_sparq-server"))
        .arg("--addr")
        .arg(tcp_addr.to_string())
        .arg("--http3")
        .arg("--http3-addr")
        .arg(udp_addr.to_string())
        .arg("--tls-cert")
        .arg(fixture("http3-cert.pem"))
        .arg("--tls-key")
        .arg(fixture("http3-key.pem"))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut server = ChildGuard(child);

    let http = reqwest::Client::new();
    let url = format!("http://{tcp_addr}/sparql");
    let mut http1 = None;
    for _ in 0..100 {
        match http
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/sparql-query")
            .header(reqwest::header::ACCEPT, RESULTS_JSON)
            .body(QUERY)
            .send()
            .await
        {
            Ok(response) => {
                http1 = Some(response);
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(25)).await,
        }
    }
    let http1 = match http1 {
        Some(response) => response,
        None => {
            let _ = server.0.kill();
            let mut stderr = String::new();
            if let Some(mut pipe) = server.0.stderr.take() {
                pipe.read_to_string(&mut stderr)?;
            }
            let _ = server.0.wait();
            return Err(format!("server did not become ready: {stderr}").into());
        }
    };
    let h1_status = http1.status().as_u16();
    let h1_content_type = http1
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let h1_body = http1.bytes().await?;

    let endpoint = h3_client(&fixture("http3-ca.pem"))?;
    let connection = endpoint.connect(udp_addr, "localhost")?.await?;
    let (mut driver, mut sender) = h3::client::new(h3_quinn::Connection::new(connection)).await?;
    let driver = tokio::spawn(async move {
        let _ = poll_fn(|context| driver.poll_close(context)).await;
    });
    let (h3_status, h3_content_type, h3_body) = h3_query(&mut sender, udp_addr).await?;

    assert_eq!(h3_status, h1_status);
    assert_eq!(h3_content_type, h1_content_type);
    assert_eq!(h3_body, h1_body);
    assert!(
        String::from_utf8_lossy(&h3_body).contains("http://example.com/http3"),
        "the parity assertion must cover a non-empty query result"
    );

    drop(sender);
    endpoint.close(quinn::VarInt::from_u32(0), b"test complete");
    tokio::time::timeout(Duration::from_secs(5), driver).await??;
    Ok(())
}
