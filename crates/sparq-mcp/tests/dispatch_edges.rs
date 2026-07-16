//! [GPT-5.6] (sq-bif.35) JSON-RPC dispatch edge contracts for the always-compiled
//! MCP server core.

use serde_json::Value;
use sparq_core::Graph;
use sparq_mcp::McpServer;

const TTL: &str = r#"@prefix ex: <http://ex/> .
ex:alice ex:name "Alice" .
"#;

fn graph() -> Graph {
    Graph::load_str(TTL, "turtle").expect("load turtle")
}

fn parse(response: &str) -> Value {
    serde_json::from_str(response).expect("response is valid JSON")
}

fn call(server: &mut McpServer, raw: &str) -> Value {
    parse(&server.handle_message(raw).expect("request gets a response"))
}

#[test]
fn unknown_tool_name_returns_exact_method_not_found_error() {
    let mut server = McpServer::new(graph());
    let response = call(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nonesuch","arguments":{}}}"#,
    );

    assert_eq!(response["error"]["code"], -32601);
    assert_eq!(response["error"]["message"], "unknown tool: nonesuch");
}

#[test]
fn ping_returns_empty_object_and_echoes_request_id() {
    let mut server = McpServer::new(graph());
    let response = call(&mut server, r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#);

    assert_eq!(response["id"], 7);
    assert_eq!(response["result"], serde_json::json!({}));
}

#[test]
fn unknown_method_notification_gets_no_response() {
    let mut server = McpServer::new(graph());
    let response = server.handle_message(r#"{"jsonrpc":"2.0","method":"nonesuch"}"#);

    assert!(
        response.is_none(),
        "an erroneous notification must remain silent"
    );
}

#[test]
fn malformed_json_error_has_null_id() {
    let mut server = McpServer::new(graph());
    let response = parse(
        &server
            .handle_message("{ not json")
            .expect("parse error gets a response"),
    );

    assert!(
        response["id"].is_null(),
        "an uncorrelatable parse error must use a null id"
    );
}

#[test]
fn initialized_request_returns_null_result() {
    let mut server = McpServer::new(graph());
    let response = call(
        &mut server,
        r#"{"jsonrpc":"2.0","id":9,"method":"initialized"}"#,
    );

    assert!(response["result"].is_null());
}
