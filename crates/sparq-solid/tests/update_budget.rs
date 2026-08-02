//! [SONNET-4.6] sq-yhlf0: the BUDGETED write path — `PodStore::update_as_with_budget`
//! and its ACP twin.
//!
//! The read path has always been bounded by a [`QueryBudget`]; the write path was not, so
//! a caller that must bound every evaluation it issues (mcp-solid §9.4's MUST, honoured by
//! `sparq_mcp::SolidMcpServer`'s `update` tool) had no way to do it. These tests pin the
//! three properties that makes true:
//!
//! - **Equivalence**: an unlimited budget is exactly `update_as` / `update_as_acp`.
//! - **The APPLY is bounded**: a `DELETE … WHERE` whose WHERE blows up aborts with the
//!   engine's budget error instead of running to completion.
//! - **The AUTHORIZATION CHECK is bounded too**: the `GRAPH ?var` binding SELECT that
//!   resolves an update's precise write set is an evaluation the update triggered, so it
//!   runs under the same budget — and a trip there is reported AS a budget trip, not
//!   laundered into an "update denied" the caller would misread as a permissions problem.
//!
//! An ALREADY-EXHAUSTED budget is the deterministic way to observe the bound (the same
//! device `sparq-engine`'s own budget tests use) — it removes the race an honestly
//! pathological query would otherwise need to win. How that is spelled is target-dependent
//! (see `expired` below): `wasm32-unknown-unknown` has no monotonic clock, so
//! [`QueryBudget`] carries no `deadline` field there and the portable row cap stands in.

use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_solid::{PodStore, Session};

const ALICE: &str = "https://alice.ex/card#me";
const BOB: &str = "https://bob.ex/card#me";

/// A two-document pod: alice holds Read/Write/Control on the root and by default; bob
/// holds nothing.
fn pod() -> PodStore {
    let nq = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/n2#it> <https://ex.dev/ns#title> "world" <https://pod.ex/notes/n2> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Write> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Control> <https://pod.ex/.acl> .
"#;
    let mut s = PodStore::new(Graph::load_dataset(nq, "nquads").expect("fixture parses"));
    s.materialize_wac().expect("wac materializes");
    s
}

fn sess(agent: &str) -> Session<'_> {
    Session { agent: Some(agent), client: None, issuer: None, now: None }
}

/// A budget that is already exhausted before evaluation starts: a wall-clock deadline in
/// the past, so the executor's first cooperative poll trips it.
#[cfg(not(target_arch = "wasm32"))]
fn expired() -> QueryBudget {
    QueryBudget {
        deadline: Some(std::time::Instant::now() - std::time::Duration::from_millis(1)),
        ..QueryBudget::unlimited()
    }
}

/// The `wasm32-unknown-unknown` spelling of [`expired`]. `std::time::Instant` is unusable
/// there (no monotonic clock — `Instant::now()` panics), so `QueryBudget::deadline` is
/// `#[cfg(not(target_arch = "wasm32"))]` and does not exist on this target. The portable
/// ROW dimension expresses the same "already exhausted" precondition: a cap of zero rows
/// admits no materialised result, so it trips at the same cooperative poll sites the
/// deadline does, with the same `"query budget exceeded (…)"` error class. Verified
/// equivalent for every assertion in this file by running the whole module against this
/// budget on the native target. [SONNET-4.6]
#[cfg(target_arch = "wasm32")]
fn expired() -> QueryBudget {
    QueryBudget { max_rows: Some(0), ..QueryBudget::unlimited() }
}

fn triples(store: &PodStore, name: &str) -> usize {
    let term = oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(name));
    store
        .graph
        .named
        .iter()
        .find(|(n, _)| *n == term)
        .map(|(_, g)| {
            let pat: sparq_core::store::Pattern = [None, None, None];
            g.store.scan(&pat).rows.len()
        })
        .unwrap_or(0)
}

/// A `DELETE … WHERE` whose WHERE is a cross product over every named graph, targeting a
/// STATIC graph alice may write — so authorization passes and only the budget can stop it.
const PATHOLOGICAL: &str = "\
DELETE { GRAPH <https://pod.ex/notes/n1> { ?s ?p ?o } } \
WHERE { GRAPH ?g1 { ?s ?p ?o } GRAPH ?g2 { ?a ?b ?c } GRAPH ?g3 { ?d ?e ?f } }";

/// The same shape with a VARIABLE graph target: the authorization check must resolve the
/// precise write set by evaluating this WHERE, so the budget bites there — before the
/// apply is ever reached.
const PATHOLOGICAL_VAR_TARGET: &str = "\
DELETE { GRAPH ?g1 { ?s ?p ?o } } \
WHERE { GRAPH ?g1 { ?s ?p ?o } GRAPH ?g2 { ?a ?b ?c } GRAPH ?g3 { ?d ?e ?f } }";

#[test]
fn an_unlimited_budget_is_exactly_the_unbudgeted_update() {
    let ins = "INSERT DATA { GRAPH <https://pod.ex/notes/n1> { \
        <https://pod.ex/notes/n1#it> <https://ex.dev/ns#tag> \"x\" } }";

    let mut plain = pod();
    plain.update_as(&sess(ALICE), ins).expect("unbudgeted applies");

    let mut budgeted = pod();
    budgeted
        .update_as_with_budget(&sess(ALICE), ins, &QueryBudget::unlimited())
        .expect("unlimited budget applies");

    assert_eq!(
        triples(&plain, "https://pod.ex/notes/n1"),
        triples(&budgeted, "https://pod.ex/notes/n1"),
        "an unlimited budget must not change what the update does"
    );

    // …and the deny path is likewise untouched: bob still cannot write.
    assert!(
        budgeted
            .update_as_with_budget(&sess(BOB), ins, &QueryBudget::unlimited())
            .is_err(),
        "an unlimited budget must not weaken write authorization"
    );
}

#[test]
fn an_exhausted_budget_aborts_the_apply_of_a_pathological_where() {
    let mut s = pod();
    let before = triples(&s, "https://pod.ex/notes/n1");
    let err = s
        .update_as_with_budget(&sess(ALICE), PATHOLOGICAL, &expired())
        .expect_err("an exhausted budget must abort the apply");
    assert!(err.contains("query budget exceeded"), "must name the budget: {err}");
    assert!(
        !err.contains("update denied"),
        "the update WAS authorized — the failure is the bound, not the ACL: {err}"
    );
    assert_eq!(
        triples(&s, "https://pod.ex/notes/n1"),
        before,
        "an aborted WHERE deletes nothing"
    );
}

#[test]
fn the_same_pathological_update_applies_under_an_unlimited_budget() {
    // The positive control for the test above: without the budget this update is perfectly
    // legal and DOES delete, so the abort above is attributable to the budget alone.
    let mut s = pod();
    assert!(triples(&s, "https://pod.ex/notes/n1") > 0, "there is something to delete");
    s.update_as_with_budget(&sess(ALICE), PATHOLOGICAL, &QueryBudget::unlimited())
        .expect("unbounded, the update is authorized and well-formed");
    assert_eq!(
        triples(&s, "https://pod.ex/notes/n1"),
        0,
        "the apply really ran to completion"
    );
}

#[test]
fn an_exhausted_budget_aborts_the_variable_graph_authorization_check() {
    // The `GRAPH ?var` write-set resolution evaluates the WHERE *during authorization*.
    // Under an expired budget it must surface the BUDGET error, not fall back to the
    // conservative all-graphs wildcard (which would deny and misattribute the cause).
    let mut s = pod();
    let before = triples(&s, "https://pod.ex/notes/n1");
    let err = s
        .update_as_with_budget(&sess(ALICE), PATHOLOGICAL_VAR_TARGET, &expired())
        .expect_err("an exhausted budget must abort the write-set resolution");
    assert!(err.contains("query budget exceeded"), "must name the budget: {err}");
    assert!(
        !err.contains("update denied"),
        "a bounded evaluation must not be laundered into a permission denial: {err}"
    );
    assert_eq!(triples(&s, "https://pod.ex/notes/n1"), before, "nothing was applied");
}

#[test]
fn an_exhausted_budget_still_denies_an_unauthorized_write() {
    // Fail-closed ordering: bob's DROP names a static graph, so authorization rejects it
    // without evaluating anything — the budget must not turn a deny into a budget error.
    let mut s = pod();
    let err = s
        .update_as_with_budget(&sess(BOB), "DROP GRAPH <https://pod.ex/notes/n1>", &expired())
        .expect_err("bob holds no write grant");
    assert!(err.contains("update denied"), "the deny path is unchanged: {err}");
    assert!(triples(&s, "https://pod.ex/notes/n1") > 0, "the store is untouched");
}

#[test]
fn a_whereless_insert_data_ignores_an_exhausted_budget() {
    // The documented boundary of the bound: INSERT/DELETE DATA and CLEAR/DROP consult no
    // budget (they are bounded by their operand size), so an exhausted budget must not
    // block them. Without this, "bound the update path" could silently become "block it".
    let mut s = pod();
    let before = triples(&s, "https://pod.ex/notes/n1");
    s.update_as_with_budget(
        &sess(ALICE),
        "INSERT DATA { GRAPH <https://pod.ex/notes/n1> { \
             <https://pod.ex/notes/n1#it> <https://ex.dev/ns#tag> \"x\" } }",
        &expired(),
    )
    .expect("INSERT DATA carries no WHERE, so it consults no budget");
    assert_eq!(triples(&s, "https://pod.ex/notes/n1"), before + 1, "it really applied");
}

#[test]
fn the_acp_twin_budgets_the_same_way() {
    // `update_as_acp_with_budget` differs from its WAC sibling only in which view a
    // control-doc write re-materializes, so the budget behaviour must be identical: the
    // pathological WHERE trips, and the whereless INSERT DATA does not.
    let mut s = pod();
    // (The ACP twin over a WAC-materialized view: the auth view is the same shape either
    // way — only the re-materialization branch differs, and neither update below touches
    // a control document.)
    let err = s
        .update_as_acp_with_budget(&sess(ALICE), PATHOLOGICAL, &expired())
        .expect_err("the ACP twin is bounded too");
    assert!(err.contains("query budget exceeded"), "must name the budget: {err}");

    let before = triples(&s, "https://pod.ex/notes/n2");
    s.update_as_acp_with_budget(
        &sess(ALICE),
        "INSERT DATA { GRAPH <https://pod.ex/notes/n2> { \
             <https://pod.ex/notes/n2#it> <https://ex.dev/ns#tag> \"y\" } }",
        &expired(),
    )
    .expect("whereless INSERT DATA consults no budget on the ACP twin either");
    assert_eq!(triples(&s, "https://pod.ex/notes/n2"), before + 1);
}
