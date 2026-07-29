//! [SONNET-4.6] (sq-2c0f0, gh #3221) The **Streamable HTTP** transport over a real
//! socket — only compiled with the `http` feature.
//!
//! The unit tests in `src/http/` drive `HttpTransport::route` directly (protocol
//! decisions without a port). These drive the whole thing the other way round: a real
//! `TcpListener`, a real `serve_http` accept loop, a hand-written HTTP/1.1 client, and
//! the real `wire` parser in between — so the framing, the session header round-trip and
//! the SSE stream are exercised end-to-end rather than mocked.

#![cfg(feature = "http")]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use sparq_core::Graph;
use sparq_mcp::http::{serve_http, HttpConfig, HttpTransport};
use sparq_mcp::McpServer;

const DATA: &str = "<http://ex/a> <http://ex/p> <http://ex/b> .";
const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;

/// One HTTP response as the test client sees it.
struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Reply {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("a JSON body")
    }
}

/// Bind loopback, start the accept loop, and hand back the address plus the transport
/// handle (the embedder's seam for pushing server→client messages).
fn start(config: HttpConfig) -> (String, Arc<HttpTransport>) {
    let graph = Graph::load_str(DATA, "ntriples").unwrap();
    let server = Arc::new(Mutex::new(McpServer::new(graph)));
    let transport = Arc::new(HttpTransport::new(server, config));
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().unwrap().to_string();
    let serving = Arc::clone(&transport);
    // The accept loop runs for the lifetime of the test process; the test's assertions
    // are what end it.
    std::thread::spawn(move || {
        let _ = serve_http(listener, serving);
    });
    (address, transport)
}

/// Write one raw request and return the connection, ready to read the response.
fn send(address: &str, raw: &str) -> TcpStream {
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    stream.flush().unwrap();
    stream
}

/// Read a complete (non-streaming) response — the server closes after writing it.
fn read_reply(mut stream: TcpStream) -> Reply {
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("read response");
    let text = String::from_utf8(raw).expect("UTF-8 response");
    let (head, body) = text.split_once("\r\n\r\n").expect("head/body separator");
    let mut lines = head.lines();
    let status_line = lines.next().expect("status line");
    let status: u16 = status_line
        .split(' ')
        .nth(1)
        .expect("status code")
        .parse()
        .expect("numeric status");
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    Reply {
        status,
        headers,
        body: body.to_string(),
    }
}

/// POST a JSON-RPC message, optionally carrying a session id.
fn post(address: &str, session: Option<&str>, body: &str) -> Reply {
    let session_header = match session {
        Some(id) => format!("mcp-session-id: {}\r\n", id),
        None => String::new(),
    };
    let raw = format!(
        "POST /mcp HTTP/1.1\r\nhost: {}\r\ncontent-type: application/json\r\n\
         accept: application/json, text/event-stream\r\n{}content-length: {}\r\n\r\n{}",
        address,
        session_header,
        body.len(),
        body
    );
    read_reply(send(address, &raw))
}

#[test]
fn a_real_socket_session_initializes_calls_a_tool_and_terminates() {
    let (address, transport) = start(HttpConfig::default());

    // 1. Handshake. The session id comes back as a response header.
    let init = post(&address, None, INIT);
    assert_eq!(init.status, 200);
    assert_eq!(
        init.header("content-type"),
        Some("application/json"),
        "a POST is answered with JSON, not an SSE upgrade"
    );
    let session = init
        .header("mcp-session-id")
        .expect("initialize mints a session")
        .to_string();
    assert!(init.json()["result"]["serverInfo"]["name"].is_string());
    assert!(transport.has_session(&session));

    // 2. The lifecycle notification: nothing to respond with.
    let notified = post(
        &address,
        Some(&session),
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );
    assert_eq!(notified.status, 202);
    assert!(notified.body.is_empty());

    // 3. A real tool call over the real dataset.
    let called = post(
        &address,
        Some(&session),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"stats","arguments":{}}}"#,
    );
    assert_eq!(called.status, 200);
    let payload: Value = serde_json::from_str(
        called.json()["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(payload["triples"], 1, "the tool ran against the served graph");

    // 4. Explicit termination, then the id is dead.
    let deleted = read_reply(send(
        &address,
        &format!(
            "DELETE /mcp HTTP/1.1\r\nhost: {}\r\nmcp-session-id: {}\r\n\r\n",
            address, session
        ),
    ));
    assert_eq!(deleted.status, 204);
    assert!(!transport.has_session(&session));
    assert_eq!(
        post(&address, Some(&session), INIT).status,
        404,
        "a terminated session is gone, not silently re-created"
    );
}

#[test]
fn two_clients_hold_independent_sessions_on_one_listener() {
    let (address, transport) = start(HttpConfig::default());
    let first = post(&address, None, INIT)
        .header("mcp-session-id")
        .unwrap()
        .to_string();
    let second = post(&address, None, INIT)
        .header("mcp-session-id")
        .unwrap()
        .to_string();
    assert_ne!(first, second);
    assert_eq!(transport.session_count(), 2);

    // Both work against the one shared server.
    for session in [&first, &second] {
        assert_eq!(
            post(&address, Some(session), r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#).status,
            200
        );
    }
    // Terminating one leaves the other serving.
    assert!(transport.end_session(&first));
    assert_eq!(
        post(&address, Some(&first), r#"{"jsonrpc":"2.0","id":4,"method":"ping"}"#).status,
        404
    );
    assert_eq!(
        post(&address, Some(&second), r#"{"jsonrpc":"2.0","id":4,"method":"ping"}"#).status,
        200
    );
}

#[test]
fn a_get_streams_server_pushed_messages_as_sse_and_ends_with_the_session() {
    let (address, transport) = start(HttpConfig {
        // Short enough that an idle tick lands inside the test's read window.
        sse_keepalive: Duration::from_millis(50),
        ..HttpConfig::default()
    });
    let session = post(&address, None, INIT)
        .header("mcp-session-id")
        .unwrap()
        .to_string();

    // Queue one message BEFORE the stream opens: it must still be delivered.
    assert!(transport.notify(
        &session,
        r#"{"jsonrpc":"2.0","method":"notifications/message","params":{"level":"info"}}"#
    ));

    let stream = send(
        &address,
        &format!(
            "GET /mcp HTTP/1.1\r\nhost: {}\r\naccept: text/event-stream\r\n\
             mcp-session-id: {}\r\n\r\n",
            address, session
        ),
    );
    let mut reader = BufReader::new(stream);

    // The SSE head.
    let mut head = String::new();
    loop {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0, "stream closed early");
        head.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{}", head);
    assert!(head.to_ascii_lowercase().contains("content-type: text/event-stream"));
    assert!(
        !head.to_ascii_lowercase().contains("content-length"),
        "a stream has no declared length: {}",
        head
    );

    // The queued message, framed as an SSE event with a resumable id.
    let mut event = String::new();
    loop {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0, "stream closed early");
        if line == "\n" && !event.is_empty() {
            break;
        }
        event.push_str(&line);
    }
    assert_eq!(
        event,
        "id: 1\nevent: message\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/message\",\"params\":{\"level\":\"info\"}}\n",
        "the pushed message arrives verbatim inside an SSE `message` event"
    );

    // A second GET while the first stream holds the session is a conflict.
    let conflict = read_reply(send(
        &address,
        &format!(
            "GET /mcp HTTP/1.1\r\nhost: {}\r\naccept: text/event-stream\r\n\
             mcp-session-id: {}\r\n\r\n",
            address, session
        ),
    ));
    assert_eq!(conflict.status, 409);

    // Idle: a keepalive comment, not a message.
    let mut keepalive = String::new();
    reader.read_line(&mut keepalive).unwrap();
    assert_eq!(keepalive, ": keepalive\n");

    // Terminating the session ends the stream (EOF), rather than hanging the client.
    assert!(transport.end_session(&session));
    let mut rest = String::new();
    reader.read_to_string(&mut rest).expect("stream drains to EOF");
}

#[test]
fn a_reconnecting_client_resumes_from_its_last_event_id() {
    let (address, transport) = start(HttpConfig {
        sse_keepalive: Duration::from_millis(50),
        ..HttpConfig::default()
    });
    let session = post(&address, None, INIT)
        .header("mcp-session-id")
        .unwrap()
        .to_string();
    transport.notify(&session, r#"{"n":1}"#);
    transport.notify(&session, r#"{"n":2}"#);

    // First connection: read both events, then drop the socket mid-stream.
    let open = |last: Option<&str>| {
        let resume = match last {
            Some(id) => format!("last-event-id: {}\r\n", id),
            None => String::new(),
        };
        BufReader::new(send(
            &address,
            &format!(
                "GET /mcp HTTP/1.1\r\nhost: {}\r\naccept: text/event-stream\r\n\
                 mcp-session-id: {}\r\n{}\r\n",
                address, session, resume
            ),
        ))
    };
    let mut reader = open(None);
    let mut seen = String::new();
    while !seen.contains("data: {\"n\":2}") {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0, "stream closed early");
        seen.push_str(&line);
    }
    assert!(seen.contains("id: 1\n"), "{}", seen);
    assert!(seen.contains("id: 2\n"), "{}", seen);
    drop(reader);

    // The server needs a moment to notice the dropped socket and free the stream slot.
    let mut resumed = None;
    for _ in 0..100 {
        // Nudge the stream so the abandoned writer hits its error and closes.
        transport.notify(&session, r#"{"n":3}"#);
        std::thread::sleep(Duration::from_millis(50));
        let mut reader = open(Some("1"));
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line.starts_with("HTTP/1.1 200") {
            resumed = Some(reader);
            break;
        }
    }
    let mut reader = resumed.expect("the stream slot is released once the client goes");

    // Everything after event 1 is replayed; event 1 itself is not.
    let mut replayed = String::new();
    while !replayed.contains("data: {\"n\":3}") {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0, "stream closed early");
        replayed.push_str(&line);
    }
    assert!(
        !replayed.contains("data: {\"n\":1}"),
        "the client already had event 1: {}",
        replayed
    );
    assert!(replayed.contains("data: {\"n\":2}"), "{}", replayed);
}

#[test]
fn malformed_and_unauthorized_requests_are_answered_on_the_wire() {
    let (address, _transport) = start(HttpConfig::default());

    // Chunked upload: refused, not mis-framed.
    let chunked = read_reply(send(
        &address,
        &format!(
            "POST /mcp HTTP/1.1\r\nhost: {}\r\ntransfer-encoding: chunked\r\n\r\n",
            address
        ),
    ));
    assert_eq!(chunked.status, 501);

    // An oversized body is refused before it is read.
    let big = read_reply(send(
        &address,
        &format!(
            "POST /mcp HTTP/1.1\r\nhost: {}\r\ncontent-type: application/json\r\n\
             content-length: 99999999\r\n\r\n",
            address
        ),
    ));
    assert_eq!(big.status, 413);

    // A browser origin nobody allowed: refused before dispatch.
    let cross_origin = read_reply(send(
        &address,
        &format!(
            "POST /mcp HTTP/1.1\r\nhost: {}\r\ncontent-type: application/json\r\n\
             origin: http://evil.test\r\ncontent-length: {}\r\n\r\n{}",
            address,
            INIT.len(),
            INIT
        ),
    ));
    assert_eq!(cross_origin.status, 403);

    // Any path but the endpoint.
    let elsewhere = read_reply(send(
        &address,
        &format!("GET /admin HTTP/1.1\r\nhost: {}\r\n\r\n", address),
    ));
    assert_eq!(elsewhere.status, 404);
}
