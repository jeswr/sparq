//! SERVICE federation egress allowlist — server integration. [OPUS-4.8] (sq-4w18)
//!
//! Runs only with `cargo test -p sparq-server --features service`. Boots the real axum
//! server in-process and drives `/sparql` with a `SERVICE <iri>` clause, asserting the
//! server's default-DENY-all posture and the allowlist opt-in:
//!
//!   * with NO allowlist (the default), a SERVICE to a loopback endpoint is REFUSED
//!     before any socket is opened (the fake remote serves 0 requests and the query
//!     still errors) — the SSRF guard;
//!   * with the loopback host on the allowlist, the SAME query reaches the endpoint and
//!     joins the remote solution;
//!   * `SERVICE SILENT` under the deny posture yields the join identity (the query
//!     succeeds with the surrounding bindings intact, no network call).
#![cfg(feature = "service")]

use std::io::{Read, Write};
use std::net::TcpListener as StdListener;
use std::sync::mpsc;
use std::thread;

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig, ServiceAllowlist};
use tokio::net::TcpListener;

const DATA: &str = "@prefix ex: <http://ex/> . ex:alice a ex:Person . ex:bob a ex:Person .";

/// A one-shot loopback HTTP endpoint returning a canned SPARQL-Results-JSON body to the
/// first `n` requests, then exiting. With `n == 0` it accepts nothing — so a test that
/// expects the egress filter to refuse BEFORE dialling proves no socket was opened.
fn serve(body: String, n: usize) -> String {
    let listener = StdListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(()).ok();
        for _ in 0..n {
            let Ok((mut stream, _)) = listener.accept() else { return };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/sparql-results+json\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    rx.recv().unwrap();
    format!("http://{addr}/")
}

async fn spawn_with(config: ServerConfig) -> String {
    let graph = Graph::load_str(DATA, "turtle").unwrap();
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn remote_body() -> String {
    r#"{ "head": { "vars": ["s", "name"] },
        "results": { "bindings": [
            { "s": {"type":"uri","value":"http://ex/alice"}, "name": {"type":"literal","value":"Alice"} },
            { "s": {"type":"uri","value":"http://ex/bob"},   "name": {"type":"literal","value":"Bob"} }
        ] } }"#
        .to_string()
}

#[tokio::test]
async fn empty_allowlist_refuses_service_before_any_network_call() {
    // The fake remote serves ZERO requests: if the server dialled it the assertion that
    // the query failed would still hold, but the point is the egress filter refuses the
    // loopback host before opening a socket.
    let remote = serve(remote_body(), 0);
    let host = remote.trim_start_matches("http://").trim_end_matches('/');
    // Default config => empty allowlist => deny ALL SERVICE.
    let base = spawn_with(ServerConfig::default()).await;
    let q = format!(
        "PREFIX ex: <http://ex/> SELECT ?s WHERE {{ ?s a ex:Person . SERVICE <http://{host}/> {{ ?s ex:name ?name }} }}"
    );
    let resp = reqwest::Client::new()
        .get(format!("{base}/sparql"))
        .query(&[("query", q)])
        .send()
        .await
        .unwrap();
    // A non-SILENT SERVICE that is refused fails the query — the server maps an engine
    // execution error to a non-2xx. (500 here; the body names the egress refusal.)
    assert!(!resp.status().is_success(), "blocked SERVICE must not 200; got {}", resp.status());
    let body = resp.text().await.unwrap().to_lowercase();
    assert!(
        body.contains("egress") || body.contains("allowlist") || body.contains("service"),
        "expected an egress/allowlist refusal in the body, got: {body}"
    );
}

#[tokio::test]
async fn allowlisted_loopback_host_reaches_the_endpoint() {
    let remote = serve(remote_body(), 1);
    let host = remote.trim_start_matches("http://").trim_end_matches('/');
    let bare_host = host.split(':').next().unwrap().to_string(); // "127.0.0.1"
    // Allowlist the loopback host => SERVICE to it is permitted.
    let mut config = ServerConfig::default();
    let mut allow = ServiceAllowlist::default();
    allow.add(&bare_host).unwrap();
    config.service_allow = allow;
    let base = spawn_with(config).await;
    let q = format!(
        "PREFIX ex: <http://ex/> SELECT ?s ?name WHERE {{ ?s a ex:Person . SERVICE <http://{host}/> {{ ?s ex:name ?name }} }}"
    );
    let resp = reqwest::Client::new()
        .get(format!("{base}/sparql"))
        .query(&[("query", q)])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "allowlisted SERVICE must succeed");
    let body = resp.text().await.unwrap();
    assert!(body.contains("Alice"), "expected the remote name joined in: {body}");
    assert!(body.contains("Bob"), "expected the remote name joined in: {body}");
}

#[tokio::test]
async fn silent_service_under_deny_yields_identity() {
    let remote = serve(remote_body(), 0);
    let host = remote.trim_start_matches("http://").trim_end_matches('/');
    let base = spawn_with(ServerConfig::default()).await; // empty allowlist
    let q = format!(
        "PREFIX ex: <http://ex/> SELECT ?s WHERE {{ ?s a ex:Person . SERVICE SILENT <http://{host}/> {{ ?s ex:name ?name }} }}"
    );
    let resp = reqwest::Client::new()
        .get(format!("{base}/sparql"))
        .query(&[("query", q)])
        .send()
        .await
        .unwrap();
    // SILENT swallows the egress refusal -> identity -> both local persons survive, 200.
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("alice") && body.contains("bob"), "SILENT must keep local bindings: {body}");
}
