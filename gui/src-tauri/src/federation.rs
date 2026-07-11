// [FABLE-5] sq-ixc3.14 — native federated SERVICE query execution for the Query tool.
//
// The GUI's query path runs in the in-tab WASM engine, which fundamentally cannot evaluate a
// SPARQL 1.1 `SERVICE` clause: the browser has no blocking cross-origin HTTP (CORS), so the
// engine's federation client (`sparq-engine-service`, the `service` feature) is native-only by
// design. This module is the DELIBERATE, reviewed native query surface (see the lib.rs note on
// the removed pre-wired commands): exactly ONE command, `query_service`, scoped to the one job
// the webview cannot do itself — evaluate a SERVICE-bearing SELECT/ASK over a snapshot of the
// live workspace store, joining local rows with rows fetched from remote SPARQL endpoints.
//
// OPT-IN: the whole capability sits behind this crate's non-default `federation` cargo feature
// (which forwards to `sparq-engine/service`). A lean build compiles a stub that fails LOUDLY
// with a rebuild hint — never a silent no-op — so the default build's dependency graph gains no
// HTTP/TLS stack (the same discipline as the `hdt` feature).
//
// SECURITY — FAIL-CLOSED EGRESS (threat-model B4, the SERVICE SSRF primitive): every call
// installs the engine's STRICT allowlist-only egress policy (`with_service_egress_policy(true,
// …)` — the SAME policy the network-exposed sparq-server wires to `--service-allow`). Only the
// endpoints on the per-workspace allowlist the user explicitly configured in the GUI are
// reachable; EVERYTHING else — including public hosts — is refused PRE-HTTP with a typed error
// carrying `SERVICE_EGRESS_REFUSED_MARKER` (a clean refusal, never a hang, no socket is ever
// dialed). An empty allowlist therefore refuses every SERVICE endpoint: the default is closed.
// The allowlist is the user's explicit per-workspace configuration (the GUI equivalent of the
// server operator's `--service-allow`); the webview passes it per-call because the setting
// lives with the workspace record the frontend owns.
//
// HONESTY: no performance number is asserted. Remote round-trips are bounded by the transport's
// own finite default timeout (sparq-engine-service `HttpTransport`), so a slow endpoint errors
// rather than hanging the command indefinitely.

/// Evaluate a (typically SERVICE-bearing) SELECT/ASK query over a snapshot of the live
/// workspace store, under the STRICT fail-closed egress allowlist. `dataset` is the whole
/// workspace dataset as N-Quads (the same wire format the native loader returns); `allow` is
/// the per-workspace egress allowlist (`host` / `host:port` / `*.suffix` entries — the same
/// grammar as sparq-server's `--service-allow`). Returns the SPARQL 1.1 JSON results document
/// (the same shape the in-tab WASM engine produces), so the existing multi-view result panel
/// renders it unchanged.
#[tauri::command]
pub fn query_service(dataset: String, query: String, allow: Vec<String>) -> Result<String, String> {
    run_service_query(&dataset, &query, allow)
}

/// The command body, factored out so it can be unit-tested against a real loopback SPARQL
/// endpoint without a Tauri runtime (the same pattern as `engine::load_approved_path`).
#[cfg(feature = "federation")]
fn run_service_query(dataset: &str, query: &str, allow: Vec<String>) -> Result<String, String> {
    // An empty snapshot is a legal store (a fresh workspace); parse otherwise. N-Quads is the
    // GUI's whole-dataset wire format, so named graphs survive into the native evaluation.
    let graph = if dataset.trim().is_empty() {
        sparq_core::Graph::new()
    } else {
        sparq_core::Graph::load_dataset(dataset, "nquads")?
    };
    // STRICT allowlist-only egress for the whole evaluation: any SERVICE endpoint not on the
    // user's per-workspace allowlist is refused pre-HTTP (fail-closed; empty list = refuse all).
    sparq_engine::with_service_egress_policy(true, allow, || {
        sparq_engine::query_json(&graph, query)
    })
}

/// Lean-build stub: federation is compiled out, so a SERVICE run fails LOUDLY with an
/// actionable rebuild hint instead of silently mis-reporting (the `hdt` feature's discipline).
#[cfg(not(feature = "federation"))]
fn run_service_query(_dataset: &str, _query: &str, _allow: Vec<String>) -> Result<String, String> {
    Err(
        "Federated SERVICE queries are not compiled into this build — rebuild the desktop app \
         with `cargo build --features federation` to run SERVICE queries."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The live workspace store's snapshot: one local triple binding `?s` → `?name`.
    const LOCAL_NQUADS: &str =
        "<http://example.org/alice> <http://example.org/name> \"Alice\" .";

    /// A SELECT that JOINS the local store with a remote SERVICE endpoint on `?s`.
    #[cfg(feature = "federation")]
    fn service_query(endpoint: &str) -> String {
        format!(
            "SELECT ?name ?age WHERE {{ ?s <http://example.org/name> ?name . \
             SERVICE <{endpoint}> {{ ?s <http://example.org/age> ?age }} }}"
        )
    }

    // ── the native-lane acceptance e2e (feature ON): a LIVE loopback SPARQL endpoint ──

    /// Bind a loopback server that accepts one HTTP request (draining headers + body) and
    /// answers with `body` as a SPARQL-Results-JSON 200. Returns the ephemeral port. The same
    /// single-shot pattern as sparq-engine-service's `http_transport_tests::serve_once`.
    #[cfg(feature = "federation")]
    fn serve_srj_once(body: String) -> u16 {
        use std::io::{BufRead, BufReader, Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port = listener.local_addr().expect("local addr").port();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream);
            // Drain the request headers, remembering Content-Length so the (POSTed) SPARQL
            // request body can be drained too before we respond.
            let mut content_length = 0usize;
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                    break;
                }
                if let Some(v) = line
                    .to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(str::trim)
                    .and_then(|v| v.parse::<usize>().ok())
                {
                    content_length = v;
                }
            }
            if content_length > 0 {
                let mut body_buf = vec![0u8; content_length];
                let _ = reader.read_exact(&mut body_buf);
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/sparql-results+json\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = reader.get_mut().write_all(response.as_bytes());
        });
        port
    }

    /// ACCEPTANCE (sq-ixc3.14): a query with `SERVICE <local-test-sparq-endpoint>` returns rows
    /// JOINED across the live workspace snapshot and the remote endpoint — through the REAL
    /// engine federation client (HTTP transport + SRJ parse + join), with the endpoint
    /// explicitly allowlisted.
    #[test]
    #[cfg(feature = "federation")]
    fn service_query_joins_local_store_with_a_live_loopback_endpoint() {
        // The remote half: `?s` → `?age` for the SAME subject the local store binds.
        let srj = r#"{"head":{"vars":["s","age"]},"results":{"bindings":[
            {"s":{"type":"uri","value":"http://example.org/alice"},
             "age":{"type":"literal","value":"30",
                    "datatype":"http://www.w3.org/2001/XMLSchema#integer"}}
        ]}}"#
            .to_string();
        let port = serve_srj_once(srj);
        let endpoint = format!("http://127.0.0.1:{port}/sparql");

        let json = run_service_query(
            LOCAL_NQUADS,
            &service_query(&endpoint),
            vec![format!("127.0.0.1:{port}")],
        )
        .expect("the allowlisted SERVICE join must succeed");

        // The JOINED row carries the LOCAL binding ("Alice") AND the REMOTE binding ("30") —
        // a mutation dropping either side of the join flips this red.
        assert!(json.contains("Alice"), "local binding must survive the join: {json}");
        assert!(json.contains("\"30\""), "remote binding must be joined in: {json}");
    }

    /// ACCEPTANCE (sq-ixc3.14): an endpoint NOT on the allowlist is refused CLEANLY — a typed
    /// pre-HTTP refusal carrying the stable marker, not a hang (no socket is ever dialed: the
    /// port below has no listener, so anything but a pre-HTTP refusal would surface as a
    /// connect error or a timeout instead of the marker).
    #[test]
    #[cfg(feature = "federation")]
    fn service_endpoint_off_the_allowlist_is_refused_fail_closed() {
        let query = service_query("http://127.0.0.1:9/sparql");

        // Empty allowlist (the fresh-workspace default): EVERY endpoint refused.
        let err = run_service_query(LOCAL_NQUADS, &query, Vec::new())
            .expect_err("an empty allowlist must refuse every SERVICE endpoint");
        assert!(
            err.contains(sparq_engine::SERVICE_EGRESS_REFUSED_MARKER),
            "refusal must carry the stable egress marker, got: {err}"
        );

        // A non-empty allowlist that does NOT cover the endpoint refuses it too (strict
        // allowlist-only, not deny-private): being on the list is the ONLY way through.
        let err = run_service_query(
            LOCAL_NQUADS,
            &query,
            vec!["query.example.org".to_string()],
        )
        .expect_err("a non-matching allowlist must refuse the endpoint");
        assert!(
            err.contains(sparq_engine::SERVICE_EGRESS_REFUSED_MARKER),
            "refusal must carry the stable egress marker, got: {err}"
        );
    }

    /// A refused SERVICE evaluation must not leak partial results: the refusal is an `Err`,
    /// never an empty-but-`Ok` result set (the frontend distinguishes "no rows" from "refused").
    #[test]
    #[cfg(feature = "federation")]
    fn refusal_is_an_error_not_an_empty_result() {
        let out = run_service_query(LOCAL_NQUADS, &service_query("http://127.0.0.1:9/x"), vec![]);
        assert!(out.is_err(), "refusal must be Err, got: {out:?}");
    }

    /// A plain (non-SERVICE) query still runs natively under the strict policy — the policy
    /// gates egress, not local evaluation.
    #[test]
    #[cfg(feature = "federation")]
    fn non_service_query_runs_under_the_strict_policy() {
        let json = run_service_query(
            LOCAL_NQUADS,
            "SELECT ?name WHERE { ?s <http://example.org/name> ?name }",
            Vec::new(),
        )
        .expect("a local-only query needs no egress");
        assert!(json.contains("Alice"), "local rows must come back: {json}");
    }

    /// Lean build (feature OFF): the stub fails LOUDLY with the actionable rebuild hint —
    /// never a silent no-op or a mis-parse.
    #[test]
    #[cfg(not(feature = "federation"))]
    fn lean_build_fails_loudly_with_a_rebuild_hint() {
        let err = run_service_query(LOCAL_NQUADS, "SELECT * WHERE { ?s ?p ?o }", Vec::new())
            .expect_err("the lean-build stub must be an Err");
        assert!(err.contains("--features federation"), "got: {err}");
    }
}
