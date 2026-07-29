//! Minimal HTTP/1.1 request reading + response writing for the MCP **Streamable
//! HTTP** transport, plus Server-Sent-Events framing.
//!
//! [SONNET-4.6] (sq-2c0f0) This is deliberately a *very* small subset of HTTP/1.1 —
//! exactly what the MCP Streamable HTTP transport needs and nothing else:
//!
//! - one request per connection (every non-SSE response carries `Connection: close`),
//! - `Content-Length` bodies only — a `Transfer-Encoding` request is refused with
//!   `501`, never silently mis-framed,
//! - bounded header lines, header count and body size, so a hostile peer cannot make
//!   the parser allocate without limit,
//! - no compression, no keep-alive pipelining, no HTTP/2.
//!
//! It is *not* a general-purpose HTTP server and the crate README says so. The reason
//! it exists rather than a dependency: `sparq-mcp`'s defining property is that it pulls
//! no heavy dependency, and `research/mcp-rmcp-sdk-adoption-assessment.md` measured the
//! alternative at +45 crates and an unconditional async runtime.

use std::io::{BufRead, ErrorKind, Write};

/// The largest single header line (request line included) the parser will buffer.
const MAX_LINE_BYTES: usize = 8 * 1024;
/// The largest number of header lines the parser will accept.
const MAX_HEADERS: usize = 64;

/// Why an inbound HTTP request could not be read.
#[derive(Debug)]
pub(crate) enum WireError {
    /// The socket failed, or the peer disappeared mid-request. The `io::Error` is
    /// deliberately *not* retained: there is nobody left to write a response to and
    /// this crate logs nothing, so keeping it would only be a field no code reads.
    Io,
    /// The bytes are not a request this parser accepts. Carries the HTTP status the
    /// caller should answer with and a short, non-echoing reason.
    Malformed(u16, &'static str),
}

impl From<std::io::Error> for WireError {
    fn from(_: std::io::Error) -> Self {
        WireError::Io
    }
}

/// One parsed HTTP request: the method, the path (query string stripped), the headers
/// with **lowercased** names, and the body bytes.
#[derive(Debug, Clone, Default)]
pub(crate) struct HttpRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

impl HttpRequest {
    /// The first value of `name` (which must already be lowercase), or `None`.
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Read one line, dropping the trailing `CRLF`/`LF`, refusing anything longer than
/// `limit`. `Ok(None)` means a clean EOF *before any bytes of the line*.
fn read_line<R: BufRead>(reader: &mut R, limit: usize) -> Result<Option<String>, WireError> {
    let mut buf: Vec<u8> = Vec::with_capacity(128);
    loop {
        let (found, used) = {
            let available = match reader.fill_buf() {
                Ok(bytes) => bytes,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return Err(WireError::Io),
            };
            if available.is_empty() {
                if buf.is_empty() {
                    return Ok(None);
                }
                return Err(WireError::Malformed(400, "unterminated header line"));
            }
            match available.iter().position(|&b| b == b'\n') {
                Some(index) => {
                    buf.extend_from_slice(&available[..index]);
                    (true, index + 1)
                }
                None => {
                    buf.extend_from_slice(available);
                    (false, available.len())
                }
            }
        };
        reader.consume(used);
        if buf.len() > limit {
            return Err(WireError::Malformed(431, "header line too long"));
        }
        if found {
            break;
        }
    }
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    match String::from_utf8(buf) {
        Ok(line) => Ok(Some(line)),
        Err(_) => Err(WireError::Malformed(400, "non-UTF-8 header line")),
    }
}

/// Read one complete request from `reader`. `Ok(None)` is a clean EOF before the
/// request line — the peer opened a connection and closed it without speaking.
///
/// `max_body` bounds the declared `Content-Length`; a larger declaration is refused
/// with `413` *before* a single body byte is read, so an oversized POST costs no memory.
pub(crate) fn read_request<R: BufRead>(
    reader: &mut R,
    max_body: usize,
) -> Result<Option<HttpRequest>, WireError> {
    let request_line = match read_line(reader, MAX_LINE_BYTES)? {
        Some(line) => line,
        None => return Ok(None),
    };
    let mut parts = request_line.split(' ').filter(|p| !p.is_empty());
    let method = parts
        .next()
        .ok_or(WireError::Malformed(400, "empty request line"))?;
    let target = parts
        .next()
        .ok_or(WireError::Malformed(400, "request line has no target"))?;
    let version = parts
        .next()
        .ok_or(WireError::Malformed(400, "request line has no version"))?;
    if !version.starts_with("HTTP/1.") {
        return Err(WireError::Malformed(505, "only HTTP/1.x is supported"));
    }

    let mut headers: Vec<(String, String)> = Vec::new();
    loop {
        let line = read_line(reader, MAX_LINE_BYTES)?
            .ok_or(WireError::Malformed(400, "headers truncated"))?;
        if line.is_empty() {
            break;
        }
        if headers.len() >= MAX_HEADERS {
            return Err(WireError::Malformed(431, "too many header lines"));
        }
        let (name, value) = line
            .split_once(':')
            .ok_or(WireError::Malformed(400, "malformed header line"))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
    }

    let lookup = |name: &str| {
        headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };
    // Refuse rather than guess: mis-framing a chunked body is how request smuggling
    // starts, and no MCP client needs chunked upload for a JSON-RPC message.
    if lookup("transfer-encoding").is_some() {
        return Err(WireError::Malformed(
            501,
            "chunked transfer coding is not supported; send Content-Length",
        ));
    }
    let declared = match lookup("content-length") {
        Some(raw) => raw
            .parse::<usize>()
            .map_err(|_| WireError::Malformed(400, "malformed Content-Length"))?,
        None => 0,
    };
    if declared > max_body {
        return Err(WireError::Malformed(413, "request body too large"));
    }
    let mut body = vec![0u8; declared];
    if declared > 0 {
        std::io::Read::read_exact(reader, &mut body)?;
    }

    Ok(Some(HttpRequest {
        method: method.to_string(),
        // Route on the path only; a query string is not part of the MCP endpoint.
        path: target.split(['?', '#']).next().unwrap_or(target).to_string(),
        headers,
        body,
    }))
}

/// One outbound HTTP response. `body` is ignored for the SSE head (see
/// [`HttpResponse::write_head_to`]).
#[derive(Debug, Clone)]
pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

impl HttpResponse {
    /// An empty response with `status` and no headers.
    pub(crate) fn new(status: u16) -> Self {
        HttpResponse {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// A `200 OK` carrying `body` as `application/json`.
    pub(crate) fn json(body: String) -> Self {
        HttpResponse::new(200)
            .header("content-type", "application/json")
            .body(body.into_bytes())
    }

    /// A short `text/plain` diagnostic. The message is always a server-authored
    /// constant — request bytes are never echoed back into a response body.
    pub(crate) fn text(status: u16, message: &str) -> Self {
        HttpResponse::new(status)
            .header("content-type", "text/plain; charset=utf-8")
            .body(message.as_bytes().to_vec())
    }

    /// Builder: append one header.
    pub(crate) fn header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.to_string(), value.into()));
        self
    }

    /// Builder: set the body.
    pub(crate) fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Write the complete response (status line, headers, `Content-Length`, body).
    /// Always closes the connection: this transport serves one request per socket.
    pub(crate) fn write_to<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        let mut head = self.head_bytes();
        head.push_str(&format!("content-length: {}\r\n", self.body.len()));
        head.push_str("connection: close\r\n\r\n");
        out.write_all(head.as_bytes())?;
        out.write_all(&self.body)?;
        out.flush()
    }

    /// Write only the status line + headers, with no `Content-Length` and without
    /// closing — the head of an open SSE stream, whose body is written incrementally.
    pub(crate) fn write_head_to<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        let mut head = self.head_bytes();
        head.push_str("connection: keep-alive\r\n\r\n");
        out.write_all(head.as_bytes())?;
        out.flush()
    }

    fn head_bytes(&self) -> String {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status, reason(self.status));
        for (name, value) in &self.headers {
            head.push_str(&format!("{}: {}\r\n", name, value));
        }
        head
    }
}

/// The reason phrase for the statuses this transport emits. Unknown statuses get the
/// generic phrase rather than a wrong one.
pub(crate) fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        409 => "Conflict",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        505 => "HTTP Version Not Supported",
        _ => "Status",
    }
}

/// Frame one JSON-RPC message as an SSE `message` event with a resumption `id`.
///
/// A payload containing newlines is split across several `data:` lines, which is what
/// the SSE grammar requires — the client rejoins them with `\n`. `serde_json` emits
/// compact one-line JSON, so in practice this is a single `data:` line; the split is
/// here so an embedder queueing a pretty-printed message still produces valid SSE.
pub(crate) fn sse_event(id: u64, payload: &str) -> String {
    let mut out = String::with_capacity(payload.len() + 32);
    out.push_str(&format!("id: {}\n", id));
    out.push_str("event: message\n");
    for line in payload.split('\n') {
        out.push_str("data: ");
        out.push_str(line.trim_end_matches('\r'));
        out.push('\n');
    }
    out.push('\n');
    out
}

/// An SSE comment line — a keepalive that carries no event, so a client sitting behind
/// an idle-timeout proxy keeps its stream without observing a spurious message.
pub(crate) const SSE_KEEPALIVE: &str = ": keepalive\n\n";

/// Whether an `Accept` header value admits `mime`. Handles the exact token, `*/*` and
/// `type/*`, and ignores parameters (`;q=…`). Absent-header handling is the caller's.
pub(crate) fn accepts(header: &str, mime: &str) -> bool {
    let wanted_type = mime.split('/').next().unwrap_or(mime);
    header.split(',').any(|part| {
        let token = part.split(';').next().unwrap_or("").trim();
        token.eq_ignore_ascii_case(mime)
            || token == "*/*"
            || token
                .strip_suffix("/*")
                .is_some_and(|t| t.eq_ignore_ascii_case(wanted_type))
    })
}

/// Whether a `Content-Type` header value *is* `mime`, ignoring parameters and case.
pub(crate) fn content_type_is(header: &str, mime: &str) -> bool {
    header
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .eq_ignore_ascii_case(mime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn parse(raw: &str) -> Result<Option<HttpRequest>, WireError> {
        let mut reader = BufReader::new(raw.as_bytes());
        read_request(&mut reader, 1024)
    }

    #[test]
    fn parses_a_post_with_headers_and_body() {
        let request = parse(
            "POST /mcp?x=1 HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\
             Content-Length: 2\r\n\r\n{}",
        )
        .unwrap()
        .unwrap();
        assert_eq!(request.method, "POST");
        // The query string is stripped for routing.
        assert_eq!(request.path, "/mcp");
        // Header names are lowercased for lookup.
        assert_eq!(request.header("content-type"), Some("application/json"));
        assert_eq!(request.header("host"), Some("localhost"));
        assert_eq!(request.body, b"{}");
    }

    #[test]
    fn a_bodyless_get_parses_with_an_empty_body() {
        let request = parse("GET /mcp HTTP/1.1\r\nAccept: text/event-stream\r\n\r\n")
            .unwrap()
            .unwrap();
        assert_eq!(request.method, "GET");
        assert!(request.body.is_empty());
        assert_eq!(request.header("accept"), Some("text/event-stream"));
    }

    #[test]
    fn a_connection_closed_before_the_request_line_is_a_clean_eof() {
        assert!(parse("").unwrap().is_none());
    }

    #[test]
    fn chunked_bodies_are_refused_rather_than_misframed() {
        match parse("POST /mcp HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n") {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 501),
            other => panic!("expected a 501, got {:?}", other),
        }
    }

    #[test]
    fn an_oversized_declared_body_is_refused_before_it_is_read() {
        match parse("POST /mcp HTTP/1.1\r\nContent-Length: 99999\r\n\r\n") {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 413),
            other => panic!("expected a 413, got {:?}", other),
        }
    }

    #[test]
    fn a_malformed_content_length_is_a_400() {
        match parse("POST /mcp HTTP/1.1\r\nContent-Length: twelve\r\n\r\n") {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 400),
            other => panic!("expected a 400, got {:?}", other),
        }
    }

    #[test]
    fn an_over_long_header_line_is_refused() {
        let huge = "x".repeat(MAX_LINE_BYTES + 1);
        match parse(&format!("GET /mcp HTTP/1.1\r\nX-Big: {}\r\n\r\n", huge)) {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 431),
            other => panic!("expected a 431, got {:?}", other),
        }
    }

    #[test]
    fn too_many_header_lines_are_refused() {
        let mut raw = String::from("GET /mcp HTTP/1.1\r\n");
        for index in 0..(MAX_HEADERS + 1) {
            raw.push_str(&format!("x-h{}: v\r\n", index));
        }
        raw.push_str("\r\n");
        match parse(&raw) {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 431),
            other => panic!("expected a 431, got {:?}", other),
        }
    }

    #[test]
    fn http_09_style_and_http_2_request_lines_are_refused() {
        match parse("GET /mcp HTTP/2.0\r\n\r\n") {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 505),
            other => panic!("expected a 505, got {:?}", other),
        }
        match parse("GET\r\n\r\n") {
            Err(WireError::Malformed(status, _)) => assert_eq!(status, 400),
            other => panic!("expected a 400, got {:?}", other),
        }
    }

    #[test]
    fn a_response_writes_a_status_line_headers_and_length() {
        let mut out: Vec<u8> = Vec::new();
        HttpResponse::json("{\"a\":1}".to_string())
            .header("mcp-session-id", "abc")
            .write_to(&mut out)
            .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"), "{}", text);
        assert!(text.contains("content-type: application/json\r\n"));
        assert!(text.contains("mcp-session-id: abc\r\n"));
        assert!(text.contains("content-length: 7\r\n"));
        assert!(text.contains("connection: close\r\n"));
        assert!(text.ends_with("\r\n\r\n{\"a\":1}"));
    }

    #[test]
    fn an_sse_head_carries_no_content_length_and_does_not_close() {
        let mut out: Vec<u8> = Vec::new();
        HttpResponse::new(200)
            .header("content-type", "text/event-stream")
            .write_head_to(&mut out)
            .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("content-type: text/event-stream\r\n"));
        assert!(!text.contains("content-length"));
        assert!(text.contains("connection: keep-alive\r\n"));
        assert!(text.ends_with("\r\n\r\n"));
    }

    #[test]
    fn sse_events_carry_an_id_and_one_data_line_per_payload_line() {
        assert_eq!(sse_event(7, "{\"a\":1}"), "id: 7\nevent: message\ndata: {\"a\":1}\n\n");
        assert_eq!(sse_event(1, "a\nb"), "id: 1\nevent: message\ndata: a\ndata: b\n\n");
        // A CR left by a pretty-printer must not leak into the data line.
        assert_eq!(sse_event(2, "a\r\nb"), "id: 2\nevent: message\ndata: a\ndata: b\n\n");
    }

    #[test]
    fn accept_matching_handles_wildcards_parameters_and_lists() {
        assert!(accepts("application/json", "application/json"));
        assert!(accepts("application/json, text/event-stream", "text/event-stream"));
        assert!(accepts("*/*", "application/json"));
        assert!(accepts("application/*", "application/json"));
        assert!(accepts("APPLICATION/JSON;q=0.9", "application/json"));
        assert!(!accepts("text/html", "application/json"));
        assert!(!accepts("text/*", "application/json"));
    }

    #[test]
    fn content_type_matching_ignores_parameters_and_case() {
        assert!(content_type_is("application/json", "application/json"));
        assert!(content_type_is("Application/JSON; charset=utf-8", "application/json"));
        assert!(!content_type_is("text/plain", "application/json"));
    }

    #[test]
    fn reason_phrases_cover_the_statuses_this_transport_emits() {
        assert_eq!(reason(202), "Accepted");
        assert_eq!(reason(409), "Conflict");
        assert_eq!(reason(599), "Status");
    }
}
