//! [FABLE-5] sq-lrtc3.1 — e2e integration tests for the OPT-IN ODRL lane on `POST /authz/query`
//! (the `odrl-authz` cargo feature; the whole file no-ops without it).
//!
//! Two acceptance pillars (the bead's criteria):
//!
//! 1. **Differential decision test** — for a permit / prohibit / deny-overrides / constraint /
//!    conditional(recipient) / fail-closed matrix, the HTTP-lane decision (does the SAME SPARQL
//!    query return the target graph's rows for this session?) must EQUAL
//!    `sparq_policy::evaluate` over the same `(policy, request)` — the library evaluator is the
//!    decision oracle, the HTTP lane its enforcement.
//! 2. **Two requesters, same query, provably different result sets** — one ODRL policy grants
//!    alice g1 and bob g2; the identical query returns disjoint rows per session.
//!
//! Plus the lane's fail-closed HTTP refusals (anonymous / non-read mode / targetless rule /
//! `odrl:perm` conflict strategy) and the dormant-lane invariant (no ODRL in the dataset =>
//! byte-identical `solid-authz` behaviour).
#![cfg(feature = "odrl-authz")]

use sparq_core::Graph;
use sparq_policy::{evaluate, parse_policy_str, Request, Value, ODRL_RECIPIENT};
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

const ALICE: &str = "https://alice.ex/card#me";
const BOB: &str = "https://bob.ex/card#me";
const N1: &str = "https://pod.ex/notes/n1";
const ODRL: &str = "http://www.w3.org/ns/odrl/2/";

/// A minimal boot store — `/authz/*` authorises over the request-body dataset, never this.
const BOOT: &str = "@prefix ex: <http://example.org/> . ex:alice a ex:Person .";

/// One content graph (n1) with NO static ACL — every grant must come from the ODRL bridge.
const CONTENT: &str = "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> \"hello\" <https://pod.ex/notes/n1> .\n";

/// The SAME query every session issues: the n1 titles this session is allowed to see.
const QUERY: &str = "SELECT ?o WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?o } }";

async fn spawn() -> String {
    let graph = Graph::load_str(BOOT, "turtle").unwrap();
    let config = ServerConfig { solid_authz: true, ..ServerConfig::default() };
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// POST /authz/query as `agent` (anonymous when `None`), optionally with a session `now`.
async fn query_as(
    base: &str,
    dataset: &str,
    agent: Option<&str>,
    now: Option<&str>,
    mode: Option<&str>,
) -> reqwest::Response {
    let mut session = serde_json::Map::new();
    if let Some(a) = agent {
        session.insert("agent".into(), a.into());
    }
    if let Some(n) = now {
        session.insert("now".into(), n.into());
    }
    let mut body = serde_json::json!({
        "dataset": dataset,
        "session": session,
        "query": QUERY,
        "view": "wac",
    });
    if let Some(m) = mode {
        body["mode"] = m.into();
    }
    client()
        .post(format!("{base}/authz/query"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// The number of result rows in a SPARQL 1.1 JSON results body.
async fn row_count(resp: reqwest::Response) -> usize {
    assert_eq!(resp.status(), 200, "query lane must succeed for this row");
    let v: serde_json::Value = resp.json().await.unwrap();
    v["results"]["bindings"].as_array().map(|a| a.len()).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// 1. The differential decision matrix: HTTP-lane visibility == sparq_policy::evaluate.
// ---------------------------------------------------------------------------

/// One matrix row: an ODRL policy (default-graph N-Quads), the requesting agent, and the
/// request `now` (when the row exercises a dateTime constraint).
struct MatrixRow {
    name: &'static str,
    policy_nq: String,
    agent: &'static str,
    now: Option<&'static str>,
}

fn permit(assignee: &str, extra: &str) -> String {
    format!(
        "<urn:pol/p> <{ODRL}permission> _:perm .\n\
         _:perm <{ODRL}action> <{ODRL}read> .\n\
         _:perm <{ODRL}target> <{N1}> .\n\
         _:perm <{ODRL}assignee> <{assignee}> .\n{extra}"
    )
}

fn prohibit(assignee: &str) -> String {
    format!(
        "<urn:pol/p> <{ODRL}prohibition> _:proh .\n\
         _:proh <{ODRL}action> <{ODRL}read> .\n\
         _:proh <{ODRL}target> <{N1}> .\n\
         _:proh <{ODRL}assignee> <{assignee}> .\n"
    )
}

/// A `_:perm odrl:constraint` block (leftOperand / operator / rightOperand).
fn constraint(left: &str, op: &str, right_term: &str) -> String {
    format!(
        "_:perm <{ODRL}constraint> _:c .\n\
         _:c <{ODRL}leftOperand> <{ODRL}{left}> .\n\
         _:c <{ODRL}operator> <{ODRL}{op}> .\n\
         _:c <{ODRL}rightOperand> {right_term} .\n"
    )
}

const DT: &str = "^^<http://www.w3.org/2001/XMLSchema#dateTime>";

fn matrix() -> Vec<MatrixRow> {
    vec![
        MatrixRow {
            name: "bare permit, the assignee",
            policy_nq: permit(ALICE, ""),
            agent: ALICE,
            now: None,
        },
        MatrixRow {
            name: "bare permit, a NON-assignee (fail-closed)",
            policy_nq: permit(ALICE, ""),
            agent: BOB,
            now: None,
        },
        MatrixRow {
            name: "permit + matching prohibition (deny-overrides)",
            policy_nq: format!("{}{}", permit(ALICE, ""), prohibit(ALICE)),
            agent: ALICE,
            now: None,
        },
        MatrixRow {
            name: "dateTime constraint satisfied (strict lt, now inside)",
            policy_nq: permit(
                ALICE,
                &constraint("dateTime", "lt", &format!("\"2030-01-01T00:00:00Z\"{DT}")),
            ),
            agent: ALICE,
            now: Some("2026-01-01T00:00:00Z"),
        },
        MatrixRow {
            name: "dateTime constraint violated (strict lt, now past the bound)",
            policy_nq: permit(
                ALICE,
                &constraint("dateTime", "lt", &format!("\"2030-01-01T00:00:00Z\"{DT}")),
            ),
            agent: ALICE,
            now: Some("2031-01-01T00:00:00Z"),
        },
        MatrixRow {
            name: "conditional: recipient eq the requester (delivery-recipient context)",
            policy_nq: permit(ALICE, &constraint("recipient", "eq", &format!("<{ALICE}>"))),
            agent: ALICE,
            now: None,
        },
        MatrixRow {
            name: "conditional: recipient eq someone ELSE (fail-closed)",
            policy_nq: permit(ALICE, &constraint("recipient", "eq", &format!("<{BOB}>"))),
            agent: ALICE,
            now: None,
        },
        MatrixRow {
            name: "purpose constraint with NO purpose evidence (fail-closed)",
            policy_nq: permit(ALICE, &constraint("purpose", "eq", "<https://ex.dev/marketing>")),
            agent: ALICE,
            now: None,
        },
        MatrixRow {
            name: "count constraint on the stateless lane (fail-closed, sq-snopa.8)",
            policy_nq: permit(
                ALICE,
                &constraint(
                    "count",
                    "lteq",
                    "\"5\"^^<http://www.w3.org/2001/XMLSchema#integer>",
                ),
            ),
            agent: ALICE,
            now: None,
        },
    ]
}

/// The library-side oracle: `sparq_policy::evaluate` over the SAME `(policy, request)` the
/// HTTP lane materialises — party = the session agent, action = `odrl:read`, target = n1,
/// recipient = the requesting agent (the delivery recipient of the results), `at` = session now.
fn oracle_allows(row: &MatrixRow) -> bool {
    let policy = parse_policy_str(&row.policy_nq, "ntriples").expect("row policy parses");
    let mut request = Request::new(format!("{ODRL}read"))
        .on(N1)
        .by(row.agent)
        .with(ODRL_RECIPIENT, Value::Iri(row.agent.to_owned()));
    if let Some(now) = row.now {
        request = request.at(now);
    }
    evaluate(&policy, &request).allow
}

#[tokio::test]
async fn http_lane_decision_equals_library_evaluate_over_the_matrix() {
    let base = spawn().await;
    for row in matrix() {
        let dataset = format!("{CONTENT}{}", row.policy_nq);
        let resp = query_as(&base, &dataset, Some(row.agent), row.now, None).await;
        let visible = row_count(resp).await > 0;
        let expected = oracle_allows(&row);
        assert_eq!(
            visible, expected,
            "row '{}': HTTP-lane visibility ({visible}) diverged from \
             sparq_policy::evaluate ({expected})",
            row.name
        );
    }
    // The matrix is non-degenerate: it exercises BOTH decisions (guards vacuity —
    // a lane that always denies or always allows cannot pass this).
    let decisions: Vec<bool> = matrix().iter().map(oracle_allows).collect();
    assert!(decisions.iter().any(|d| *d) && decisions.iter().any(|d| !*d));
}

// ---------------------------------------------------------------------------
// 2. Two requesters, SAME query => provably different result sets.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn two_requesters_same_query_get_different_result_sets() {
    let base = spawn().await;
    // Two content graphs; ONE policy: alice may read g1, bob may read g2.
    let dataset = format!(
        "<https://pod.ex/a#it> <https://ex.dev/ns#title> \"alpha\" <https://pod.ex/a> .\n\
         <https://pod.ex/b#it> <https://ex.dev/ns#title> \"beta\" <https://pod.ex/b> .\n\
         <urn:pol/p> <{ODRL}permission> _:p1 .\n\
         _:p1 <{ODRL}action> <{ODRL}read> .\n\
         _:p1 <{ODRL}target> <https://pod.ex/a> .\n\
         _:p1 <{ODRL}assignee> <{ALICE}> .\n\
         <urn:pol/p> <{ODRL}permission> _:p2 .\n\
         _:p2 <{ODRL}action> <{ODRL}read> .\n\
         _:p2 <{ODRL}target> <https://pod.ex/b> .\n\
         _:p2 <{ODRL}assignee> <{BOB}> .\n"
    );
    let titles = |v: serde_json::Value| -> Vec<String> {
        v["results"]["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["o"]["value"].as_str().unwrap().to_owned())
            .collect()
    };
    let alice_resp = query_as(&base, &dataset, Some(ALICE), None, None).await;
    assert_eq!(alice_resp.status(), 200);
    let alice_rows = titles(alice_resp.json().await.unwrap());
    let bob_resp = query_as(&base, &dataset, Some(BOB), None, None).await;
    assert_eq!(bob_resp.status(), 200);
    let bob_rows = titles(bob_resp.json().await.unwrap());
    // The SAME query, two sessions: disjoint, non-empty, policy-scoped results.
    assert_eq!(alice_rows, vec!["alpha".to_owned()], "alice sees exactly her granted graph");
    assert_eq!(bob_rows, vec!["beta".to_owned()], "bob sees exactly his granted graph");
}

// ---------------------------------------------------------------------------
// 3. Fail-closed HTTP refusals — NEVER a silent allow.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn lane_refusals_are_4xx_fail_closed() {
    let base = spawn().await;
    let permit_ds = format!("{CONTENT}{}", permit(ALICE, ""));

    // Anonymous session + ODRL in the dataset => 403 (a prohibition could not bind).
    let resp = query_as(&base, &permit_ds, None, None, None).await;
    assert_eq!(resp.status(), 403, "anonymous + ODRL must refuse");

    // A non-read mode => 400 (the lane covers the query/read action only).
    let resp = query_as(&base, &permit_ds, Some(ALICE), None, Some("write")).await;
    assert_eq!(resp.status(), 400, "non-read mode + ODRL must refuse");

    // A targetless prohibition => 400 (under-materialising it would widen access).
    let targetless = format!(
        "{CONTENT}<urn:pol/p> <{ODRL}prohibition> _:proh .\n\
         _:proh <{ODRL}action> <{ODRL}read> .\n\
         _:proh <{ODRL}assignee> <{ALICE}> .\n"
    );
    let resp = query_as(&base, &targetless, Some(ALICE), None, None).await;
    assert_eq!(resp.status(), 400, "targetless rule must refuse");

    // An unimplementable odrl:conflict strategy (odrl:perm) => 400.
    let perm_strategy = format!(
        "{CONTENT}{}<urn:pol/p> <{ODRL}conflict> <{ODRL}perm> .\n",
        permit(ALICE, "")
    );
    let resp = query_as(&base, &perm_strategy, Some(ALICE), None, None).await;
    assert_eq!(resp.status(), 400, "odrl:perm conflict strategy must refuse");
}

// ---------------------------------------------------------------------------
// 4. Dormant lane: no ODRL in the dataset => plain solid-authz behaviour.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn lane_is_dormant_without_odrl_in_the_dataset() {
    let base = spawn().await;
    // The module's static-WAC shape: alice holds an inherited Read grant on n1.
    let wac = "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> \"hello\" <https://pod.ex/notes/n1> .\n\
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .\n\
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .\n\
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .\n\
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .\n";
    // Anonymous is NOT refused (no ODRL => the lane never fires) — it just sees nothing.
    let resp = query_as(&base, wac, None, None, None).await;
    assert_eq!(row_count(resp).await, 0, "anonymous WAC-only session sees nothing");
    // And the static WAC grant still works untouched.
    let resp = query_as(&base, wac, Some(ALICE), None, None).await;
    assert_eq!(row_count(resp).await, 1, "static WAC grant admits alice as before");
}
