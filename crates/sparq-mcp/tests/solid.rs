//! [FABLE-5] sq-u16eq — end-to-end tests for the pod-backed MCP server (feature
//! `solid`): LDP resource/container CRUD tools over a real `PodStore` with a real
//! materialized WAC view, driven through the actual JSON-RPC dispatch core.
//!
//! Load-bearing invariants (each would flip red under the matching mutation):
//! - **Existence non-disclosure (draft §9.3)**: the error for an EXISTING document the
//!   session may not read is byte-identical to the error for a NONEXISTENT one.
//! - **Data-derived containment (draft §6.4)**: `container_list` returns exactly the
//!   stored `ldp:contains` members — including one whose IRI lives OUTSIDE the
//!   container's path — and omits a stored document that has no containment triple
//!   (an IRI-path-guessing implementation would get both wrong).
//! - **Shared dataset (draft §6.4)**: a `resource_put` is immediately visible to the
//!   SPARQL `query` tool, and vice-versa surfaces cannot disagree.
//! - **Write gating (draft §7.1)**: with `allow_update = false` the mutating tools are
//!   neither advertised nor callable.
//! - **ACL write-through (draft §7.3)**: a `resource_put`/`resource_delete` of an
//!   `.acl` re-derives authorization atomically (grant appears/disappears with no
//!   separate materialize call), and a malformed ACL body changes nothing.

#![cfg(feature = "solid")]

use serde_json::{json, Value};
use sparq_core::Graph;
use sparq_mcp::{SolidMcpServer, SolidServerConfig};
use sparq_solid::PodStore;

const ALICE: &str = "https://alice.ex/card#me";
const BOB: &str = "https://bob.ex/card#me";

/// A small WAC pod: a root container; `notes/` (one doc, one out-of-path member, one
/// UNLISTED doc); a `secret/` subtree governed by its own ACL (bob-only); and a root
/// ACL granting alice Read/Write/Control on the root and by default.
fn pod() -> PodStore {
    let nq = r#"
<https://pod.ex/> <http://www.w3.org/ns/ldp#contains> <https://pod.ex/notes/> <https://pod.ex/> .
<https://pod.ex/> <http://www.w3.org/ns/ldp#contains> <https://pod.ex/secret/> <https://pod.ex/> .
<https://pod.ex/notes/> <http://www.w3.org/ns/ldp#contains> <https://pod.ex/notes/n1> <https://pod.ex/notes/> .
<https://pod.ex/notes/> <http://www.w3.org/ns/ldp#contains> <https://elsewhere.example/shared/doc> <https://pod.ex/notes/> .
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/unlisted#it> <https://ex.dev/ns#title> "orphan" <https://pod.ex/notes/unlisted> .
<https://pod.ex/secret/s1#it> <https://ex.dev/ns#title> "classified" <https://pod.ex/secret/s1> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Write> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Control> <https://pod.ex/.acl> .
<https://pod.ex/secret/.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/secret/.acl> .
<https://pod.ex/secret/.acl#a> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/secret/> <https://pod.ex/secret/.acl> .
<https://pod.ex/secret/.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> <https://pod.ex/secret/.acl> .
<https://pod.ex/secret/.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/secret/.acl> .
"#;
    PodStore::new(Graph::load_dataset(nq, "nquads").expect("fixture parses"))
}

fn server_for(agent: &str, allow_update: bool) -> SolidMcpServer {
    let config = SolidServerConfig {
        agent: Some(agent.to_string()),
        allow_update,
        ..SolidServerConfig::default()
    };
    SolidMcpServer::with_config(pod(), config).expect("materializes")
}

fn parse(resp: &str) -> Value {
    serde_json::from_str(resp).expect("response is valid JSON")
}

fn rpc(server: &mut SolidMcpServer, raw: &str) -> Value {
    parse(&server.handle_message(raw).expect("request gets a response"))
}

/// Call one tool, returning `(text, is_error)` from the MCP `CallToolResult`.
fn tool(server: &mut SolidMcpServer, name: &str, args: Value) -> (String, bool) {
    let req = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": args }
    });
    let resp = rpc(server, &req.to_string());
    let result = &resp["result"];
    assert!(
        result.is_object(),
        "tools/call must yield a tool result (got protocol error: {resp})"
    );
    (
        result["content"][0]["text"]
            .as_str()
            .expect("text content")
            .to_string(),
        result["isError"].as_bool().unwrap_or(false),
    )
}

fn tool_names(server: &mut SolidMcpServer) -> Vec<String> {
    let resp = rpc(server, r#"{"jsonrpc":"2.0","id":9,"method":"tools/list"}"#);
    resp["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|t| t["name"].as_str().expect("name").to_string())
        .collect()
}

// ───────────────────────── Class R (read core) ─────────────────────────

#[test]
fn read_only_pod_server_advertises_read_tools_only() {
    let mut s = server_for(ALICE, false);
    let names = tool_names(&mut s);
    assert_eq!(names, ["query", "resource_get", "container_list"]);
}

#[test]
fn write_enabled_pod_server_advertises_the_mutating_tools() {
    let mut s = server_for(ALICE, true);
    let names = tool_names(&mut s);
    assert_eq!(
        names,
        [
            "query",
            "resource_get",
            "container_list",
            "update",
            "resource_put",
            "resource_delete",
            "container_create"
        ]
    );
}

#[test]
fn resource_get_serves_the_document_as_ntriples() {
    let mut s = server_for(ALICE, false);
    let (text, is_err) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(!is_err, "authorized read must succeed: {text}");
    let v: Value = serde_json::from_str(&text).expect("result is JSON");
    assert_eq!(v["url"], "https://pod.ex/notes/n1");
    assert_eq!(v["content_type"], "application/n-triples");
    let content = v["content"].as_str().expect("content");
    assert!(
        content.contains("\"hello\""),
        "the document body is served: {content}"
    );
    assert!(
        !content.contains("classified"),
        "resource_get must serve ONE document, not the dataset"
    );
}

#[test]
fn resource_get_rejects_a_non_ntriples_accept_instead_of_coercing() {
    let mut s = server_for(ALICE, false);
    let (text, is_err) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1", "accept": "application/ld+json"}),
    );
    assert!(
        is_err,
        "unsupported accept must be a tool error, not silent coercion"
    );
    assert!(text.contains("unsupported accept"), "honest error: {text}");
}

#[test]
fn unauthorized_read_is_byte_identical_to_nonexistent() {
    // §9.3: alice may NOT read the (existing) secret doc; the error must be
    // byte-identical to the error for a document that does not exist, so existence is
    // never disclosed. This test fails if denial gets its own message.
    let mut s = server_for(ALICE, false);
    let (denied, e1) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/secret/s1"}),
    );
    let (absent, e2) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/secret/nope"}),
    );
    assert!(e1 && e2, "both must be tool errors");
    assert_eq!(
        denied.replace("secret/s1", "X"),
        absent.replace("secret/nope", "X"),
        "unauthorized-read and nonexistent errors must be indistinguishable"
    );
    // And the same template covers a readable-but-absent target.
    let (readable_absent, e3) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/nope"}),
    );
    assert!(e3);
    assert_eq!(
        readable_absent,
        "resource not found: <https://pod.ex/notes/nope>"
    );
    assert_eq!(denied, "resource not found: <https://pod.ex/secret/s1>");
}

#[test]
fn container_list_derives_members_from_stored_containment_only() {
    let mut s = server_for(ALICE, false);
    let (text, is_err) = tool(
        &mut s,
        "container_list",
        json!({"url": "https://pod.ex/notes/"}),
    );
    assert!(!is_err, "{text}");
    let v: Value = serde_json::from_str(&text).expect("JSON");
    let members: Vec<(&str, bool)> = v["members"]
        .as_array()
        .expect("members")
        .iter()
        .map(|m| {
            (
                m["url"].as_str().unwrap(),
                m["container"].as_bool().unwrap(),
            )
        })
        .collect();
    // The out-of-path member IS listed (containment is data, not IRI prefixes)…
    assert!(members.contains(&("https://elsewhere.example/shared/doc", false)));
    assert!(members.contains(&("https://pod.ex/notes/n1", false)));
    // …and the stored-but-unlisted document is NOT (no ldp:contains triple).
    assert!(
        !members.iter().any(|(u, _)| u.contains("unlisted")),
        "a document without a containment triple must not be listed: {members:?}"
    );
    assert_eq!(members.len(), 2);

    // The root lists its two child containers with the container flag set.
    let (text, _) = tool(&mut s, "container_list", json!({"url": "https://pod.ex/"}));
    let v: Value = serde_json::from_str(&text).expect("JSON");
    let flags: Vec<bool> = v["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["container"].as_bool().unwrap())
        .collect();
    assert_eq!(flags, [true, true]);
}

#[test]
fn container_list_non_disclosure_matches_resource_get() {
    let mut s = server_for(ALICE, false);
    let (denied, e1) = tool(
        &mut s,
        "container_list",
        json!({"url": "https://pod.ex/secret/"}),
    );
    assert!(e1);
    assert_eq!(denied, "resource not found: <https://pod.ex/secret/>");
}

#[test]
fn query_tool_is_session_scoped() {
    // The same query under two sessions: alice sees notes, not secrets; bob sees
    // secrets, not notes. Unauthorized data contributes nothing and raises no error.
    let q = json!({
        "sparql": "SELECT ?t WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?t } } ORDER BY ?t"
    });
    let mut alice = server_for(ALICE, false);
    let (text, is_err) = tool(&mut alice, "query", q.clone());
    assert!(!is_err, "{text}");
    assert!(
        text.contains("hello") && !text.contains("classified"),
        "{text}"
    );

    let mut bob = server_for(BOB, false);
    let (text, is_err) = tool(&mut bob, "query", q);
    assert!(!is_err, "{text}");
    assert!(
        text.contains("classified") && !text.contains("hello"),
        "{text}"
    );
}

// ───────────────────────── Class U (gated writes) ─────────────────────────

#[test]
fn mutating_tools_are_refused_when_updates_are_disabled() {
    let mut s = server_for(ALICE, false);
    for name in [
        "update",
        "resource_put",
        "resource_delete",
        "container_create",
    ] {
        let req = json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": name, "arguments": {"url": "https://pod.ex/x"} }
        });
        let resp = rpc(&mut s, &req.to_string());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("read-only"),
            "`{name}` must be refused at the protocol level: {resp}"
        );
    }
}

#[test]
fn resource_put_create_links_containment_and_is_visible_to_query() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "resource_put",
        json!({
            "url": "https://pod.ex/notes/n2",
            "content": "<https://pod.ex/notes/n2#it> <https://ex.dev/ns#title> \"second\" .",
            "content_type": "application/n-triples"
        }),
    );
    assert!(!is_err, "{text}");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["created"], true);
    assert_eq!(v["triples"], 1);

    // Containment: the parent listing now includes n2.
    let (text, _) = tool(
        &mut s,
        "container_list",
        json!({"url": "https://pod.ex/notes/"}),
    );
    assert!(
        text.contains("https://pod.ex/notes/n2"),
        "created doc must be contained: {text}"
    );

    // Shared dataset (§6.4): the SPARQL tool sees the new document immediately.
    let (text, is_err) = tool(
        &mut s,
        "query",
        json!({"sparql":
            "ASK { GRAPH <https://pod.ex/notes/n2> { ?s <https://ex.dev/ns#title> \"second\" } }"}),
    );
    assert!(
        !is_err && text.contains("true"),
        "query must see the put: {text}"
    );

    // And resource_get round-trips it.
    let (text, is_err) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n2"}),
    );
    assert!(!is_err && text.contains("second"), "{text}");
}

#[test]
fn resource_put_replace_swaps_the_named_graph_atomically() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "resource_put",
        json!({
            "url": "https://pod.ex/notes/n1",
            "content": "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> \"rewritten\" .",
            "content_type": "application/n-triples"
        }),
    );
    assert!(!is_err, "{text}");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        v["created"], false,
        "replacing an existing doc is not a create"
    );
    let (text, _) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(text.contains("rewritten"), "{text}");
    assert!(
        !text.contains("hello"),
        "PUT is a full replacement, not a merge: {text}"
    );
}

#[test]
fn resource_put_malformed_body_mutates_nothing() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "resource_put",
        json!({
            "url": "https://pod.ex/notes/n1",
            "content": "this is not turtle @@@",
            "content_type": "text/turtle"
        }),
    );
    assert!(is_err, "malformed content must be rejected: {text}");
    let (text, _) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(
        text.contains("hello"),
        "parse-first: the prior content survives: {text}"
    );
}

#[test]
fn resource_put_write_gates_follow_non_disclosure() {
    let mut s = server_for(ALICE, true);
    // No read on the secret subtree → the not-found error, never a write-denied one.
    let (text, is_err) = tool(
        &mut s,
        "resource_put",
        json!({
            "url": "https://pod.ex/secret/s1",
            "content": "<https://pod.ex/secret/s1#it> <https://ex.dev/ns#title> \"pwned\" .",
            "content_type": "application/n-triples"
        }),
    );
    assert!(is_err);
    assert_eq!(text, "resource not found: <https://pod.ex/secret/s1>");

    // Bob CAN read secret/ but has no Write → the distinguishable denied error.
    let mut bob = server_for(BOB, true);
    let (text, is_err) = tool(
        &mut bob,
        "resource_put",
        json!({
            "url": "https://pod.ex/secret/s1",
            "content": "<https://pod.ex/secret/s1#it> <https://ex.dev/ns#title> \"mine\" .",
            "content_type": "application/n-triples"
        }),
    );
    assert!(is_err);
    assert_eq!(text, "write access denied: <https://pod.ex/secret/s1>");
}

#[test]
fn resource_delete_removes_doc_and_containment() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "resource_delete",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(!is_err, "{text}");
    let (text, is_err) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(is_err);
    assert_eq!(text, "resource not found: <https://pod.ex/notes/n1>");
    let (text, _) = tool(
        &mut s,
        "container_list",
        json!({"url": "https://pod.ex/notes/"}),
    );
    assert!(
        !text.contains("notes/n1"),
        "containment link must be gone: {text}"
    );
}

#[test]
fn resource_delete_rejects_a_non_empty_container() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "resource_delete",
        json!({"url": "https://pod.ex/notes/"}),
    );
    assert!(is_err);
    assert!(text.contains("not empty"), "{text}");
    // Still listable afterwards — nothing was deleted.
    let (text, is_err) = tool(
        &mut s,
        "container_list",
        json!({"url": "https://pod.ex/notes/"}),
    );
    assert!(!is_err && text.contains("notes/n1"), "{text}");
}

#[test]
fn container_create_creates_a_typed_linked_empty_container() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "container_create",
        json!({"url": "https://pod.ex/projects/"}),
    );
    assert!(!is_err, "{text}");
    // Linked into the root listing, flagged as a container.
    let (text, _) = tool(&mut s, "container_list", json!({"url": "https://pod.ex/"}));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert!(v["members"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["url"] == "https://pod.ex/projects/" && m["container"] == true));
    // Typed, empty, and listable.
    let (text, is_err) = tool(
        &mut s,
        "container_list",
        json!({"url": "https://pod.ex/projects/"}),
    );
    assert!(!is_err, "{text}");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["members"].as_array().unwrap().len(), 0);
    let (text, _) = tool(
        &mut s,
        "resource_get",
        json!({"url": "https://pod.ex/projects/"}),
    );
    assert!(text.contains("BasicContainer"), "{text}");
    // Slash discipline + duplicate rejection.
    let (text, is_err) = tool(
        &mut s,
        "container_create",
        json!({"url": "https://pod.ex/x"}),
    );
    assert!(is_err && text.contains("slash-terminated"), "{text}");
    let (text, is_err) = tool(
        &mut s,
        "container_create",
        json!({"url": "https://pod.ex/projects/"}),
    );
    assert!(is_err && text.contains("already exists"), "{text}");
}

#[test]
fn update_tool_enforces_session_write_authorization() {
    let mut bob = server_for(BOB, true);
    // Bob has Read-only on secret/ — a write there must be DENIED atomically.
    let (text, is_err) = tool(
        &mut bob,
        "update",
        json!({"sparql":
            "INSERT DATA { GRAPH <https://pod.ex/secret/s1> { <urn:x> <urn:y> \"z\" } }"}),
    );
    assert!(
        is_err,
        "unwritable target must reject the whole update: {text}"
    );
    // Alice CAN write under notes/.
    let mut alice = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut alice,
        "update",
        json!({"sparql":
            "INSERT DATA { GRAPH <https://pod.ex/notes/n1> { <urn:x> <urn:y> \"z\" } }"}),
    );
    assert!(!is_err, "{text}");
}

// ───────────────────────── ACL write-through (§7.3) ─────────────────────────

#[test]
fn acl_put_and_delete_rederive_authorization_atomically() {
    let mut alice = server_for(ALICE, true);

    // Before: bob cannot see notes/n1 (non-disclosure not-found).
    let mut bob = server_for(BOB, false);
    let (text, is_err) = tool(
        &mut bob,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(is_err && text == "resource not found: <https://pod.ex/notes/n1>");

    // Alice (Control via the root ACL) PUTs notes/.acl granting alice full + bob Read.
    // Absolute IRIs: the storage layer parses the body with no base IRI.
    let acl_body = r#"
@prefix acl: <http://www.w3.org/ns/auth/acl#> .
<https://pod.ex/notes/.acl#owner> a acl:Authorization ;
  acl:accessTo <https://pod.ex/notes/> ;
  acl:default <https://pod.ex/notes/> ;
  acl:agent <https://alice.ex/card#me> ;
  acl:mode acl:Read, acl:Write, acl:Control .
<https://pod.ex/notes/.acl#reader> a acl:Authorization ;
  acl:default <https://pod.ex/notes/> ;
  acl:agent <https://bob.ex/card#me> ;
  acl:mode acl:Read .
"#;
    let (text, is_err) = tool(
        &mut alice,
        "resource_put",
        json!({
            "url": "https://pod.ex/notes/.acl",
            "content": acl_body,
            "content_type": "text/turtle"
        }),
    );
    assert!(!is_err, "{text}");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["access_control"], true);
    assert_eq!(v["created"], true);

    // The grant took effect on ALICE'S OWN STORE immediately (no separate
    // materialize call): bob's session against that store now reads n1. We check on
    // alice's server by asking its store directly.
    let bob_session = sparq_solid::Session {
        agent: Some(BOB),
        client: None,
        issuer: None,
        now: None,
    };
    let d = alice.store().decide(
        &bob_session,
        "https://pod.ex/notes/n1",
        sparq_solid::Mode::Read,
    );
    assert!(
        d.allow,
        "the ACL write-through must re-derive authorization atomically"
    );

    // A malformed ACL body is rejected parse-first and changes nothing.
    let (text, is_err) = tool(
        &mut alice,
        "resource_put",
        json!({
            "url": "https://pod.ex/notes/.acl",
            "content": "@@@ not turtle",
            "content_type": "text/turtle"
        }),
    );
    assert!(is_err, "{text}");
    let d = alice.store().decide(
        &bob_session,
        "https://pod.ex/notes/n1",
        sparq_solid::Mode::Read,
    );
    assert!(
        d.allow,
        "a failed ACL write must leave the prior policy in force"
    );

    // DELETE the ACL: bob's grant disappears with it (delete narrows, atomically).
    let (text, is_err) = tool(
        &mut alice,
        "resource_delete",
        json!({"url": "https://pod.ex/notes/.acl"}),
    );
    assert!(!is_err, "{text}");
    let d = alice.store().decide(
        &bob_session,
        "https://pod.ex/notes/n1",
        sparq_solid::Mode::Read,
    );
    assert!(
        !d.allow,
        "deleting the ACL must revoke its grants immediately"
    );
}

#[test]
fn acl_write_requires_control_and_non_discloses_without_it() {
    // Bob has Read on secret/ but NOT Control → writing secret/.acl must yield the
    // not-found error (never a distinguishable denied), and the policy must survive.
    let mut bob = server_for(BOB, true);
    let (text, is_err) = tool(
        &mut bob,
        "resource_put",
        json!({
            "url": "https://pod.ex/secret/.acl",
            "content": "<https://pod.ex/secret/.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> .",
            "content_type": "application/n-triples"
        }),
    );
    assert!(is_err);
    assert_eq!(text, "resource not found: <https://pod.ex/secret/.acl>");
    let (text, is_err) = tool(
        &mut bob,
        "resource_delete",
        json!({"url": "https://pod.ex/secret/.acl"}),
    );
    assert!(is_err);
    assert_eq!(text, "resource not found: <https://pod.ex/secret/.acl>");
    // Bob still reads s1 — the policy was not touched.
    let (text, is_err) = tool(
        &mut bob,
        "resource_get",
        json!({"url": "https://pod.ex/secret/s1"}),
    );
    assert!(!is_err && text.contains("classified"), "{text}");
}

// ───────────────────────── misc hardening ─────────────────────────

#[test]
fn reserved_graph_space_is_never_a_write_target() {
    let mut s = server_for(ALICE, true);
    let (text, is_err) = tool(
        &mut s,
        "resource_put",
        json!({
            "url": "urn:sparq:auth",
            "content": "<urn:a> <urn:b> <urn:c> .",
            "content_type": "application/n-triples"
        }),
    );
    assert!(is_err && text.contains("reserved"), "{text}");
    let (text, is_err) = tool(&mut s, "resource_delete", json!({"url": "urn:sparq:auth"}));
    assert!(is_err && text.contains("reserved"), "{text}");
}

#[test]
fn anonymous_session_fails_closed_everywhere() {
    let mut anon =
        SolidMcpServer::with_config(pod(), SolidServerConfig::default()).expect("materializes");
    assert!(!anon.allow_update(), "default config must be read-only");
    let (text, is_err) = tool(
        &mut anon,
        "resource_get",
        json!({"url": "https://pod.ex/notes/n1"}),
    );
    assert!(is_err && text == "resource not found: <https://pod.ex/notes/n1>");
    let (text, is_err) = tool(
        &mut anon,
        "container_list",
        json!({"url": "https://pod.ex/"}),
    );
    assert!(is_err && text == "resource not found: <https://pod.ex/>");
    let (text, is_err) = tool(
        &mut anon,
        "query",
        json!({"sparql": "SELECT ?s WHERE { GRAPH ?g { ?s ?p ?o } }"}),
    );
    assert!(!is_err, "query never errors on authorization: {text}");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["results"]["bindings"].as_array().unwrap().len(), 0);
}

#[test]
fn initialize_reports_the_pod_server_name() {
    let mut s = server_for(ALICE, false);
    let resp = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );
    assert_eq!(resp["result"]["serverInfo"]["name"], "sparq-mcp-solid");
    assert!(resp["result"]["protocolVersion"].is_string());
}
