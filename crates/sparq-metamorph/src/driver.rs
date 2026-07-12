//! **SPARQL 1.1 Protocol drivers for external engines** (`protocol-drivers` feature).
//! [FABLE-5] sq-gum8.6
//!
//! The off-CI bug-hunting campaign points these drivers at live endpoints (Jena/Fuseki,
//! Virtuoso, Blazegraph, GraphDB, QLever, Oxigraph, MillenniumDB, and sparq's own
//! server). **CI never opens a socket**: the request-construction layer
//! ([`build_request_parts`]) is a pure function unit-tested without a network, and the
//! HTTP send in [`HttpSparqlEngine`] is a thin dispatch over it.
//!
//! Implemented here:
//!
//! * [`EndpointConfig`] — declarative endpoint description (URL, query method, extra
//!   parameters, headers, timeout), with presets for the engines whose protocol
//!   behaviour is standard ([`EndpointConfig::generic`], [`EndpointConfig::fuseki`],
//!   [`EndpointConfig::oxigraph`]).
//! * A **quirk layer** for engines that deviate from the plain protocol:
//!   [`EndpointConfig::virtuoso`] encodes Virtuoso's documented preferences
//!   (form-encoded POST against `/sparql`, an explicit `format=` output parameter —
//!   Virtuoso's content negotiation historically ignores a bare `Accept` on some
//!   deployments).
//!
//! Presets for Blazegraph, GraphDB, QLever, and MillenniumDB are deliberately **not**
//! shipped yet: their URL/quirk conventions must be validated against live instances
//! during campaign bring-up (tracked as follow-up work), and an unvalidated preset
//! would be a silent misconfiguration source. [`EndpointConfig::generic`] plus manual
//! URL/parameters covers them meanwhile.

use std::time::Duration;

use sparq_difftest::{parse_results_json, QueryResults};

use crate::engine::SparqlEngine;
use crate::verdict::{EngineFailure, FailureKind};

/// How the query is transmitted (SPARQL 1.1 Protocol §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryMethod {
    /// `POST` with `Content-Type: application/sparql-query`, query as the raw body
    /// (§2.1.3 "query via POST directly").
    PostSparqlQuery,
    /// `POST` with `Content-Type: application/x-www-form-urlencoded`, query in a
    /// `query=` field (§2.1.2 — the most broadly supported method; mirrors
    /// sparq-engine's own `SERVICE` transport choice).
    PostForm,
    /// `GET` with the query URL-encoded in the query string (§2.1.1; subject to URL
    /// length limits — some engines, e.g. QLever's public endpoints, prefer it).
    Get,
}

/// Declarative description of one SPARQL-protocol endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointConfig {
    /// Engine name for verdicts / the ledger.
    pub name: String,
    /// Absolute query endpoint URL.
    pub query_url: String,
    /// Transmission method.
    pub method: QueryMethod,
    /// Extra `key=value` pairs appended to the form body / query string (engine
    /// quirks, e.g. Virtuoso's `format=`).
    pub extra_params: Vec<(String, String)>,
    /// Extra request headers beyond the standard `Accept`.
    pub headers: Vec<(String, String)>,
    /// Per-request timeout.
    pub timeout: Duration,
    /// Maximum response body size in bytes. Requests returning more data are truncated
    /// via `ureq`'s body-read limiter and treated as a transport error. Defaults to
    /// 64 MiB — sufficient for any sane SPARQL result set; raise for unusually large
    /// results in a campaign run (e.g. full-graph dumps). [SONNET-4.6]
    pub max_body_bytes: u64,
}

impl EndpointConfig {
    /// A plain SPARQL 1.1 Protocol endpoint: form-encoded POST, no quirks.
    pub fn generic(name: &str, query_url: &str) -> Self {
        EndpointConfig {
            name: name.to_string(),
            query_url: query_url.to_string(),
            method: QueryMethod::PostForm,
            extra_params: Vec::new(),
            headers: Vec::new(),
            timeout: Duration::from_secs(60),
            max_body_bytes: 64 * 1024 * 1024,
        }
    }

    /// Apache Jena Fuseki: standard protocol at `{base}/{dataset}/query`.
    pub fn fuseki(base_url: &str, dataset: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        EndpointConfig::generic("fuseki", &format!("{base}/{dataset}/query"))
    }

    /// Oxigraph server: standard protocol at `{base}/query`.
    pub fn oxigraph(base_url: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        EndpointConfig::generic("oxigraph", &format!("{base}/query"))
    }

    /// OpenLink Virtuoso quirk layer: `{base}/sparql`, form-encoded POST, plus an
    /// explicit `format=application/sparql-results+json` output parameter (Virtuoso's
    /// documented output-selection parameter; bare-`Accept` negotiation is unreliable
    /// on some deployments).
    pub fn virtuoso(base_url: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        let mut config = EndpointConfig::generic("virtuoso", &format!("{base}/sparql"));
        config.extra_params.push((
            "format".to_string(),
            "application/sparql-results+json".to_string(),
        ));
        config
    }
}

/// The fully constructed request, before any socket is opened. Pure data — this is the
/// seam the no-network CI tests check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestParts {
    /// Final URL (including the query string for [`QueryMethod::Get`]).
    pub url: String,
    /// `"GET"` or `"POST"`.
    pub http_method: &'static str,
    /// `Content-Type` for POST bodies (`None` for GET).
    pub content_type: Option<&'static str>,
    /// Request body (`None` for GET).
    pub body: Option<String>,
    /// Extra headers from the config (the standard `Accept` is added at send time).
    pub headers: Vec<(String, String)>,
}

/// Percent-encode one value for `application/x-www-form-urlencoded` / a URL query
/// string: unreserved characters (RFC 3986 §2.3) pass through, space becomes `+`,
/// everything else becomes `%XX` per UTF-8 byte.
pub fn form_urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Append an already-encoded query-string `params` to `base_url`, using `?` when
/// `base_url` has no query string yet and `&` when it already has one.  A pre-existing
/// trailing `?` is treated the same as having a query string (we just append). [SONNET-4.6]
fn append_query_string(base_url: &str, params: &str) -> String {
    if base_url.contains('?') {
        format!("{}&{}", base_url, params)
    } else {
        format!("{}?{}", base_url, params)
    }
}

fn encode_params(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", form_urlencode(k), form_urlencode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Build the request for `sparql` against `config` — pure, no network.
pub fn build_request_parts(config: &EndpointConfig, sparql: &str) -> RequestParts {
    match config.method {
        QueryMethod::PostSparqlQuery => {
            // Extra params ride on the URL for a raw-body POST.
            let url = if config.extra_params.is_empty() {
                config.query_url.clone()
            } else {
                append_query_string(&config.query_url, &encode_params(&config.extra_params))
            };
            RequestParts {
                url,
                http_method: "POST",
                content_type: Some("application/sparql-query"),
                body: Some(sparql.to_string()),
                headers: config.headers.clone(),
            }
        }
        QueryMethod::PostForm => {
            let mut pairs = vec![("query".to_string(), sparql.to_string())];
            pairs.extend(config.extra_params.iter().cloned());
            RequestParts {
                url: config.query_url.clone(),
                http_method: "POST",
                content_type: Some("application/x-www-form-urlencoded"),
                body: Some(encode_params(&pairs)),
                headers: config.headers.clone(),
            }
        }
        QueryMethod::Get => {
            let mut pairs = vec![("query".to_string(), sparql.to_string())];
            pairs.extend(config.extra_params.iter().cloned());
            RequestParts {
                url: append_query_string(&config.query_url, &encode_params(&pairs)),
                http_method: "GET",
                content_type: None,
                body: None,
                headers: config.headers.clone(),
            }
        }
    }
}

/// A live SPARQL-protocol engine. Campaign use only — CI self-tests never construct a
/// request over the network (the pure layer above carries the tested logic).
pub struct HttpSparqlEngine {
    config: EndpointConfig,
    agent: ureq::Agent,
}

impl HttpSparqlEngine {
    /// Build a driver over `config` with its timeout applied to the whole round trip.
    pub fn new(config: EndpointConfig) -> Self {
        let agent_config = ureq::Agent::config_builder()
            .timeout_global(Some(config.timeout))
            .user_agent(concat!("sparq-metamorph/", env!("CARGO_PKG_VERSION")))
            .build();
        HttpSparqlEngine {
            config,
            agent: ureq::Agent::new_with_config(agent_config),
        }
    }

    fn failure(&self, query: &str, kind: FailureKind, message: String) -> EngineFailure {
        EngineFailure {
            engine: self.config.name.clone(),
            query: query.to_string(),
            kind,
            message,
        }
    }
}

const ACCEPT: &str = "application/sparql-results+json";

impl SparqlEngine for HttpSparqlEngine {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn select(&self, sparql: &str) -> Result<QueryResults, EngineFailure> {
        let parts = build_request_parts(&self.config, sparql);
        let response = if parts.http_method == "GET" {
            let mut request = self.agent.get(&parts.url).header("Accept", ACCEPT);
            for (key, value) in &parts.headers {
                request = request.header(key, value);
            }
            request.call()
        } else {
            let mut request = self.agent.post(&parts.url).header("Accept", ACCEPT);
            if let Some(content_type) = parts.content_type {
                request = request.header("Content-Type", content_type);
            }
            for (key, value) in &parts.headers {
                request = request.header(key, value);
            }
            request.send(parts.body.as_deref().unwrap_or(""))
        };
        let body = match response {
            Ok(mut r) => r
                .body_mut()
                .with_config()
                .limit(self.config.max_body_bytes)
                .read_to_string()
                .map_err(|e| {
                    self.failure(
                        sparql,
                        FailureKind::Transport,
                        format!("reading response: {e}"),
                    )
                })?,
            Err(ureq::Error::StatusCode(code)) => {
                return Err(self.failure(
                    sparql,
                    FailureKind::HttpStatus(code),
                    format!("endpoint returned HTTP {code}"),
                ))
            }
            Err(e) => {
                return Err(self.failure(sparql, FailureKind::Transport, e.to_string()));
            }
        };
        parse_results_json(&body)
            .map_err(|e| self.failure(sparql, FailureKind::InvalidResults, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_urlencode_covers_unreserved_space_and_utf8() {
        assert_eq!(form_urlencode("abc-XYZ_0.9~"), "abc-XYZ_0.9~");
        assert_eq!(form_urlencode("a b"), "a+b");
        assert_eq!(
            form_urlencode("?v < 5 && ?w = \"é\""),
            "%3Fv+%3C+5+%26%26+%3Fw+%3D+%22%C3%A9%22"
        );
    }

    #[test]
    fn build_request_parts_post_form_encodes_query_and_extras() {
        let mut config = EndpointConfig::generic("g", "http://localhost:1234/sparql");
        config.extra_params.push((
            "format".to_string(),
            "application/sparql-results+json".to_string(),
        ));
        let parts = build_request_parts(&config, "SELECT * WHERE { ?s ?p ?o }");
        assert_eq!(parts.http_method, "POST");
        assert_eq!(parts.url, "http://localhost:1234/sparql");
        assert_eq!(
            parts.content_type,
            Some("application/x-www-form-urlencoded")
        );
        let body = parts.body.unwrap();
        assert!(body.starts_with("query=SELECT"));
        assert!(body.contains("&format=application%2Fsparql-results%2Bjson"));
    }

    #[test]
    fn build_request_parts_raw_post_keeps_the_query_verbatim() {
        let mut config = EndpointConfig::generic("g", "http://localhost:1234/sparql");
        config.method = QueryMethod::PostSparqlQuery;
        let parts = build_request_parts(&config, "ASK { ?s ?p ?o }");
        assert_eq!(parts.content_type, Some("application/sparql-query"));
        assert_eq!(parts.body.as_deref(), Some("ASK { ?s ?p ?o }"));
        assert_eq!(parts.url, "http://localhost:1234/sparql");
    }

    #[test]
    fn build_request_parts_get_puts_the_query_in_the_url() {
        let mut config = EndpointConfig::generic("g", "http://localhost:1234/sparql");
        config.method = QueryMethod::Get;
        let parts = build_request_parts(&config, "ASK { }");
        assert_eq!(parts.http_method, "GET");
        assert!(parts
            .url
            .starts_with("http://localhost:1234/sparql?query=ASK"));
        assert_eq!(parts.body, None);
        assert_eq!(parts.content_type, None);
    }

    /// A `query_url` that already carries a query string must get `&` not a second `?`.
    /// [SONNET-4.6]
    #[test]
    fn build_request_parts_no_double_question_mark_on_pre_existing_query_string() {
        // PostSparqlQuery arm: extra_params appended to a URL that already has '?'
        let mut config =
            EndpointConfig::generic("g", "http://localhost:1234/sparql?default-graph-uri=urn:x");
        config.method = QueryMethod::PostSparqlQuery;
        config
            .extra_params
            .push(("format".to_string(), "json".to_string()));
        let parts = build_request_parts(&config, "ASK { }");
        let url = &parts.url;
        assert!(!url.contains("??"), "double '?' in URL: {url}");
        assert!(
            url.contains("default-graph-uri=urn%3Ax&format=json")
                || url.contains("default-graph-uri=urn:x&format=json"),
            "expected '&' separator, got: {url}"
        );

        // Get arm: query string appended to a URL that already has '?'
        let mut config2 =
            EndpointConfig::generic("g", "http://localhost:1234/sparql?service=default");
        config2.method = QueryMethod::Get;
        let parts2 = build_request_parts(&config2, "ASK { }");
        let url2 = &parts2.url;
        assert!(!url2.contains("??"), "double '?' in URL: {url2}");
        assert!(
            url2.contains("service=default&query="),
            "expected '&' separator, got: {url2}"
        );
    }

    #[test]
    fn presets_encode_the_documented_conventions() {
        let fuseki = EndpointConfig::fuseki("http://localhost:3030/", "ds");
        assert_eq!(fuseki.query_url, "http://localhost:3030/ds/query");
        assert_eq!(fuseki.method, QueryMethod::PostForm);

        let oxigraph = EndpointConfig::oxigraph("http://localhost:7878");
        assert_eq!(oxigraph.query_url, "http://localhost:7878/query");

        let virtuoso = EndpointConfig::virtuoso("http://localhost:8890");
        assert_eq!(virtuoso.query_url, "http://localhost:8890/sparql");
        assert!(virtuoso.extra_params.contains(&(
            "format".to_string(),
            "application/sparql-results+json".to_string()
        )));
    }

    /// Serve exactly one canned HTTP response on an ephemeral **loopback** port (no
    /// external network) and return the endpoint URL. Reads the request headers plus a
    /// `Content-Length` body before answering, so the client's write never races the
    /// response.
    fn serve_once(response: &'static str) -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            // Read until the header terminator, then drain the declared body length.
            let header_end = loop {
                let n = stream.read(&mut chunk).unwrap();
                buf.extend_from_slice(&chunk[..n]);
                if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break pos + 4;
                }
                if n == 0 {
                    return;
                }
            };
            let headers = String::from_utf8_lossy(&buf[..header_end]).to_ascii_lowercase();
            let content_length: usize = headers
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);
            while buf.len() < header_end + content_length {
                let n = stream.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            stream.write_all(response.as_bytes()).unwrap();
        });
        format!("http://127.0.0.1:{port}/sparql")
    }

    #[test]
    fn http_engine_parses_a_valid_json_response() {
        let body = r#"{"head":{"vars":["s"]},"results":{"bindings":[{"s":{"type":"uri","value":"http://example.org/x"}}]}}"#;
        let response: &'static str = Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/sparql-results+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .into_boxed_str(),
        );
        let engine =
            HttpSparqlEngine::new(EndpointConfig::generic("loopback", &serve_once(response)));
        assert_eq!(engine.name(), "loopback");
        match engine.select("SELECT * WHERE { ?s ?p ?o }").unwrap() {
            QueryResults::Solutions { solutions, .. } => assert_eq!(solutions.len(), 1),
            QueryResults::Boolean(_) => panic!("SELECT returned a boolean"),
        }
    }

    #[test]
    fn http_engine_maps_a_500_to_http_status_failure() {
        let response =
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let engine =
            HttpSparqlEngine::new(EndpointConfig::generic("loopback", &serve_once(response)));
        let err = engine.select("ASK { }").unwrap_err();
        assert_eq!(
            err.kind,
            FailureKind::HttpStatus(500),
            "fail-closed: {err:?}"
        );
    }

    #[test]
    fn http_engine_maps_garbage_body_to_invalid_results() {
        let response =
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nnot json!";
        let engine =
            HttpSparqlEngine::new(EndpointConfig::generic("loopback", &serve_once(response)));
        let err = engine.select("ASK { }").unwrap_err();
        assert_eq!(
            err.kind,
            FailureKind::InvalidResults,
            "fail-closed: {err:?}"
        );
    }
}
