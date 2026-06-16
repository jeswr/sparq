//! [OPUS-4.8] sq-h3uk — the ODRL→AUTH_GRAPH bridge: a definite ODRL Permit
//! materializes the equivalent WAC/ACP grant into `<urn:sparq:auth>` so the EXISTING
//! graph-level enforcement honours it; a Deny / ambiguous / unmapped eval
//! materializes NOTHING (fail-closed); the action→mode mapping is correct; and a
//! round-trip through the real enforcement path (`PodStore::accessible` /
//! `query_as`) grants exactly the intended access.
//!
//! Gated by the `odrl-bridge` feature (the whole test file no-ops without it).
#![cfg(feature = "odrl-bridge")]

use sparq_core::Graph;
use sparq_policy::{parse_policy_str, Request, Value};
use sparq_solid::{
    action_to_mode, materialize_permission, materialize_policy, materialize_prohibition, Mode,
    PodStore, Session,
};

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
const ALICE: &str = "https://alice.ex/card#me";
const BOB: &str = "https://bob.ex/card#me";
const CAROL: &str = "https://carol.ex/card#me";
const N1: &str = "https://pod.ex/notes/n1";

fn odrl(local: &str) -> String {
    format!("{ODRL}{local}")
}

/// A pod with one content graph + no static ACL (so the only grants are bridged ones).
fn pod() -> Graph {
    Graph::load_dataset(
        "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> \"hello\" <https://pod.ex/notes/n1> .",
        "nquads",
    )
    .expect("pod loads")
}

/// alice MAY read n1 (a bare matching permission, no constraints).
fn read_policy() -> sparq_policy::Policy {
    parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/read> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .expect("policy parses")
}

// ---------------------------------------------------------------------------
// 1. action → mode mapping correctness.
// ---------------------------------------------------------------------------
#[test]
fn action_mode_mapping() {
    assert_eq!(action_to_mode(&odrl("read")), Some(Mode::Read));
    assert_eq!(action_to_mode(&odrl("display")), Some(Mode::Read));
    assert_eq!(action_to_mode(&odrl("present")), Some(Mode::Read));
    assert_eq!(action_to_mode(&odrl("print")), Some(Mode::Read));
    assert_eq!(action_to_mode(&odrl("play")), Some(Mode::Read));
    assert_eq!(action_to_mode(&odrl("append")), Some(Mode::Append));
    assert_eq!(action_to_mode(&odrl("modify")), Some(Mode::Write));
    assert_eq!(action_to_mode(&odrl("delete")), Some(Mode::Write));
    assert_eq!(action_to_mode(&odrl("write")), Some(Mode::Write));
    // fail-closed: the umbrella + unknown + non-odrl actions are unmapped.
    assert_eq!(action_to_mode(&odrl("use")), None);
    assert_eq!(action_to_mode(&odrl("aggregate")), None);
    assert_eq!(action_to_mode("https://example.org/custom"), None);
}

// ---------------------------------------------------------------------------
// 2. Permit → materializes the expected grant triple in AUTH_GRAPH.
// ---------------------------------------------------------------------------
#[test]
fn permit_materializes_grant_triple() {
    let mut g = pod();
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    let out = materialize_permission(&mut g, &read_policy(), &req);

    assert!(out.granted, "definite Permit should grant: {out:?}");
    assert_eq!(out.mode, Some(Mode::Read));
    assert_eq!(
        out.grant_triple,
        Some((ALICE.to_owned(), "https://sparq.dev/ns/auth#read".to_owned(), N1.to_owned())),
    );

    // The grant is a real triple in the <urn:sparq:auth> view, readable as such.
    let q = "SELECT ?who WHERE { GRAPH <urn:sparq:auth> { \
        ?who <https://sparq.dev/ns/auth#read> <https://pod.ex/notes/n1> } }";
    let rows = sparq_engine::query(&g, q).expect("query");
    assert_eq!(rows.rows.len(), 1, "exactly alice's read grant");
}

// ---------------------------------------------------------------------------
// 3. Round-trip through the REAL enforcement path (PodStore::accessible / query_as).
//    The bridged grant grants exactly the intended access, nothing more.
// ---------------------------------------------------------------------------
#[test]
fn round_trip_through_enforcement() {
    let mut store = PodStore::new(pod());
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    let out = store.materialize_odrl_permission(&read_policy(), &req);
    assert!(out.granted);

    let alice = Session { agent: Some(ALICE), client: None };
    // alice can READ n1 via the materialized grant…
    assert!(store.accessible(&alice, Mode::Read).iter().any(|gph| gph.as_str() == N1));
    // …but NOT write (the bridge only materialized a read grant — fail-closed)…
    assert!(store.accessible(&alice, Mode::Write).is_empty());

    // …and a DIFFERENT agent gets nothing (the grant is scoped to alice's WebID).
    let mallory = Session { agent: Some("https://mallory.ex/card#me"), client: None };
    assert!(store.accessible(&mallory, Mode::Read).is_empty());
    // anonymous likewise.
    assert!(store.accessible(&Session::default(), Mode::Read).is_empty());

    // End-to-end: alice's authorized query returns the content; others see nothing.
    let sel = "SELECT ?t WHERE { ?s <https://ex.dev/ns#title> ?t }";
    assert_eq!(store.query_as(&alice, Mode::Read, sel).unwrap().rows.len(), 1);
    assert_eq!(store.query_as(&mallory, Mode::Read, sel).unwrap().rows.len(), 0);
    assert_eq!(store.query_as(&Session::default(), Mode::Read, sel).unwrap().rows.len(), 0);
}

// ---------------------------------------------------------------------------
// 4. Deny → materializes NOTHING (fail-closed). Wrong party never matches.
// ---------------------------------------------------------------------------
#[test]
fn deny_materializes_nothing() {
    let mut store = PodStore::new(pod());
    // Mallory is not the assignee → no permission matches → DENY.
    let req = Request::new(odrl("read")).on(N1).by("https://mallory.ex/card#me");
    let out = store.materialize_odrl_permission(&read_policy(), &req);
    assert!(!out.granted, "deny must not grant: {out:?}");
    assert!(out.grant_triple.is_none());

    // Nobody gains access — the auth view holds no bridged grant.
    let mallory = Session { agent: Some("https://mallory.ex/card#me"), client: None };
    assert!(store.accessible(&mallory, Mode::Read).is_empty());
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Read).is_empty());
}

// ---------------------------------------------------------------------------
// 5. Ambiguous / unsatisfied-constraint eval → NOTHING (fail-closed).
//    A time-windowed permission with NO dateTime context fails the constraint.
// ---------------------------------------------------------------------------
#[test]
fn unsatisfied_constraint_materializes_nothing() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/win> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator odrl:lteq ;
                      odrl:rightOperand "2020-01-01T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
"#,
        "turtle",
    )
    .unwrap();

    let mut store = PodStore::new(pod());
    // Out-of-window request (after the bound) → constraint unsatisfied → DENY → nothing.
    let req = Request::new(odrl("read"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2026-06-16T00:00:00Z".to_owned()));
    let out = store.materialize_odrl_permission(&pol, &req);
    assert!(!out.granted, "out-of-window must not grant: {out:?}");
    assert!(store.accessible(&Session { agent: Some(ALICE), client: None }, Mode::Read).is_empty());

    // And the SAME policy with NO dateTime evidence also fails closed.
    let mut store2 = PodStore::new(pod());
    let req2 = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(!store2.materialize_odrl_permission(&pol, &req2).granted);
}

// ---------------------------------------------------------------------------
// 6. Unmapped action (the umbrella) → Permit but NO grant (fail-closed).
//    `odrl:use` PERMITS a `use` request in the evaluator, yet `use` has no faithful
//    single WAC mode, so the bridge must materialize nothing.
// ---------------------------------------------------------------------------
#[test]
fn unmapped_action_materializes_nothing() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/use> a odrl:Set ; odrl:permission [
    odrl:action odrl:use ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();

    let mut g = pod();
    // The request action is `odrl:use` itself — a definite Permit, but unmapped.
    let req = Request::new(odrl("use")).on(N1).by(ALICE);
    let out = materialize_permission(&mut g, &pol, &req);
    assert!(!out.granted, "umbrella action has no mode mapping: {out:?}");
    assert!(out.grant_triple.is_none());
    assert!(!out.reasons.is_empty());

    // But a CONCRETE `read` request against the same `use` permission DOES grant
    // (the evaluator's umbrella permits it; the bridge maps the concrete request).
    let mut g2 = pod();
    let req2 = Request::new(odrl("read")).on(N1).by(ALICE);
    let out2 = materialize_permission(&mut g2, &pol, &req2);
    assert!(out2.granted, "use permission + concrete read request grants: {out2:?}");
    assert_eq!(out2.mode, Some(Mode::Read));
}

// ---------------------------------------------------------------------------
// 7. Partyless / targetless Permit → NOTHING (a partyless grant would widen access).
// ---------------------------------------------------------------------------
#[test]
fn partyless_or_targetless_materializes_nothing() {
    // An unrefined permission (no assignee, no target) matches an anonymous request
    // — but a grant with no concrete principal would widen access to everyone.
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/any> a odrl:Set ; odrl:permission [ odrl:action odrl:read ] .
"#,
        "turtle",
    )
    .unwrap();

    // No party.
    let mut g = pod();
    let no_party = Request::new(odrl("read")).on(N1);
    assert!(!materialize_permission(&mut g, &pol, &no_party).granted);

    // Party but no target.
    let mut g2 = pod();
    let no_target = Request::new(odrl("read")).by(ALICE);
    assert!(!materialize_permission(&mut g2, &pol, &no_target).granted);
}

// ---------------------------------------------------------------------------
// 8. The bridge APPENDS to an existing WAC view without clobbering static grants.
// ---------------------------------------------------------------------------
#[test]
fn bridge_preserves_existing_wac_grants() {
    // A pod whose static .acl grants BOB read on n1.
    let nq = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/notes/n1> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/notes/n1.acl> .
"#;
    let mut store = PodStore::new(Graph::load_dataset(nq, "nquads").unwrap());
    store.materialize_wac().expect("wac materializes");

    // Does `agent` have read on n1 through the store's current enforcement view?
    fn reads_n1(s: &mut PodStore, agent: &str) -> bool {
        let sess = Session { agent: Some(agent), client: None };
        s.accessible(&sess, Mode::Read).iter().any(|g| g.as_str() == N1)
    }

    assert!(reads_n1(&mut store, "https://bob.ex/card#me"), "bob's static grant");

    // Now bridge an ODRL read grant for alice on the same graph.
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&read_policy(), &req).granted);

    // BOTH grants hold: bob (static WAC) AND alice (bridged ODRL).
    assert!(reads_n1(&mut store, "https://bob.ex/card#me"), "bob preserved");
    assert!(reads_n1(&mut store, ALICE), "alice bridged");
}

// ===========================================================================
// [OPUS-4.8] sq-w693 — Prohibition → explicit auth:deny<Mode> (deny-overrides).
// ===========================================================================

/// alice is PROHIBITED from writing n1 (a bare matching prohibition, no constraints).
fn write_prohibition() -> sparq_policy::Policy {
    parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/prohib> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:modify ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .expect("policy parses")
}

// ---------------------------------------------------------------------------
// 9. A matched Prohibition materializes the expected auth:deny<Mode> triple.
// ---------------------------------------------------------------------------
#[test]
fn prohibition_materializes_deny_triple() {
    let mut g = pod();
    let req = Request::new(odrl("modify")).on(N1).by(ALICE);
    let out = materialize_prohibition(&mut g, &write_prohibition(), &req);

    assert!(out.prohibited, "matched Prohibition should deny: {out:?}");
    assert!(!out.granted, "a prohibition is not a grant");
    assert_eq!(out.mode, Some(Mode::Write));
    assert_eq!(
        out.deny_triple,
        Some((ALICE.to_owned(), "https://sparq.dev/ns/auth#denyWrite".to_owned(), N1.to_owned())),
    );

    // The deny is a real triple in the <urn:sparq:auth> view, readable as such.
    let q = "SELECT ?who WHERE { GRAPH <urn:sparq:auth> { \
        ?who <https://sparq.dev/ns/auth#denyWrite> <https://pod.ex/notes/n1> } }";
    let rows = sparq_engine::query(&g, q).expect("query");
    assert_eq!(rows.rows.len(), 1, "exactly alice's denyWrite");
}

// ---------------------------------------------------------------------------
// 10. DENY-OVERRIDES through the REAL enforcement path: a principal with BOTH an
//     allow grant AND a deny for the same mode is DENIED (deny beats allow).
// ---------------------------------------------------------------------------
#[test]
fn deny_overrides_allow_through_enforcement() {
    let mut store = PodStore::new(pod());

    // First grant alice WRITE on n1 (an ODRL Permit → auth:write).
    let permit = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/w> a odrl:Set ; odrl:permission [
    odrl:action odrl:modify ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let wreq = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&permit, &wreq).granted);

    let alice = Session { agent: Some(ALICE), client: None };
    // Sanity: the allow grant is live through the real enforcement path.
    assert!(
        store.accessible(&alice, Mode::Write).iter().any(|g| g.as_str() == N1),
        "alice can write n1 BEFORE the deny is materialized",
    );

    // Now materialize the matching Prohibition (auth:denyWrite for the same mode).
    let dreq = Request::new(odrl("modify")).on(N1).by(ALICE);
    let out = store.materialize_odrl_prohibition(&write_prohibition(), &dreq);
    assert!(out.prohibited, "deny materialized: {out:?}");

    // DENY-OVERRIDES: alice is now DENIED write through the real enforcement path,
    // even though the allow grant is still present in the auth view.
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "deny beats allow: alice can no longer write n1",
    );
    // And the write-path update enforcement honours it too (fail-closed).
    let ins = "INSERT DATA { GRAPH <https://pod.ex/notes/n1> { \
        <https://pod.ex/notes/n1#it> <https://ex.dev/ns#tag> \"x\" } }";
    assert!(store.update_as(&alice, ins).is_err(), "denied write update fails closed");
}

// ---------------------------------------------------------------------------
// 11. A single policy with BOTH a permission and a prohibition on the same
//     principal/target/mode → materialize both → DENIED (deny wins).
// ---------------------------------------------------------------------------
#[test]
fn permit_plus_prohibition_same_subject_is_denied() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/both> a odrl:Set ;
    odrl:permission [
        odrl:action odrl:modify ;
        odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] ;
    odrl:prohibition [
        odrl:action odrl:modify ;
        odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();

    let mut store = PodStore::new(pod());
    let req = Request::new(odrl("modify")).on(N1).by(ALICE);
    let out = store.materialize_odrl_policy(&pol, &req);

    // The prohibition side materializes a deny. The permit side does NOT: the ODRL
    // evaluator ALREADY applies deny-overrides (a matching prohibition overrides any
    // permission), so `evaluate(...).allow == false` and no allow grant is emitted —
    // deny-overrides holds even more strongly (the allow is never written at all).
    assert!(!out.granted, "the permit is overridden by the prohibition (evaluator): {out:?}");
    assert!(out.prohibited, "the prohibition side materialized: {out:?}");
    assert_eq!(out.mode, Some(Mode::Write), "deny mode is operative under deny-overrides");
    assert!(out.deny_triple.is_some());

    // Net effect through the real enforcement: alice is DENIED.
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "deny-overrides: a permission + prohibition on the same subject denies",
    );
}

// ---------------------------------------------------------------------------
// 12. No matching prohibition (wrong party / different mode / unmapped / partyless /
//     targetless) → materialize NOTHING (fail-closed; access never silently widened).
// ---------------------------------------------------------------------------
#[test]
fn unmatched_prohibition_materializes_nothing() {
    // (a) Wrong party — the prohibition names alice; mallory isn't carved out.
    let mut g = pod();
    let wrong_party = Request::new(odrl("modify")).on(N1).by("https://mallory.ex/card#me");
    let out = materialize_prohibition(&mut g, &write_prohibition(), &wrong_party);
    assert!(!out.prohibited, "wrong party is not carved out: {out:?}");
    assert!(out.deny_triple.is_none());

    // (b) Different action/mode — the prohibition forbids modify (Write); a read
    //     request matches no prohibition, so no deny is materialized.
    let mut g2 = pod();
    let read_req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(!materialize_prohibition(&mut g2, &write_prohibition(), &read_req).prohibited);

    // (c) Unmapped action — a prohibition on the `use` umbrella matches a `use`
    //     request, but `use` has no faithful single mode → materialize nothing, and
    //     SAY SO (a dropped deny would widen access).
    let use_prohib = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/u> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:use ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let mut g3 = pod();
    let use_req = Request::new(odrl("use")).on(N1).by(ALICE);
    let out3 = materialize_prohibition(&mut g3, &use_prohib, &use_req);
    assert!(!out3.prohibited, "unmapped umbrella deny not materialized: {out3:?}");
    assert!(!out3.reasons.is_empty(), "the unmappable carve-out is reported, not silent");

    // (d) Partyless prohibition (no assignee) matched by a partyless request → nothing
    //     (a deny with no concrete principal is meaningless here).
    let any_prohib = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/anyp> a odrl:Set ; odrl:prohibition [ odrl:action odrl:modify ] .
"#,
        "turtle",
    )
    .unwrap();
    let mut g4 = pod();
    let no_party = Request::new(odrl("modify")).on(N1);
    assert!(!materialize_prohibition(&mut g4, &any_prohib, &no_party).prohibited);

    // (e) Targetless request → nothing.
    let mut g5 = pod();
    let no_target = Request::new(odrl("modify")).by(ALICE);
    assert!(!materialize_prohibition(&mut g5, &any_prohib, &no_target).prohibited);
}

// ---------------------------------------------------------------------------
// 13. Regression: the Permit-only path still works unchanged via materialize_policy
//     (a policy with no prohibition grants exactly as before, no deny emitted).
// ---------------------------------------------------------------------------
#[test]
fn permit_only_regression_via_policy() {
    let mut g = pod();
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    let out = materialize_policy(&mut g, &read_policy(), &req);

    assert!(out.granted, "permit-only still grants: {out:?}");
    assert!(!out.prohibited, "no prohibition → no deny");
    assert_eq!(out.mode, Some(Mode::Read));
    assert!(out.deny_triple.is_none());
    assert!(out.grant_triple.is_some());

    // End-to-end through the enforcement path: alice reads, deny absent.
    let mut store = PodStore::new(pod());
    assert!(store.materialize_odrl_policy(&read_policy(), &req).granted);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1));
}

// ===========================================================================
// [OPUS-4.8] sq-hiz4 — conditional grants: a FAITHFULLY-mappable constraint
// (recipient/assignee → agent matcher) persists as an `auth:ConditionalGrant`
// and is RE-CHECKED per session; an UNmappable constraint stays one-shot.
// ===========================================================================
use sparq_solid::materialize_permission_conditional;

fn reads(store: &mut PodStore, agent: &str) -> bool {
    let s = Session { agent: Some(agent), client: None };
    store.accessible(&s, Mode::Read).iter().any(|g| g.as_str() == N1)
}

/// Count `auth:ConditionalGrant` heads naming `agent` (or any, if `agent` is None) in
/// the materialized auth view of a freshly-bridged `graph`.
fn cond_grants_for(graph: &Graph, agent: Option<&str>) -> usize {
    let q = match agent {
        Some(a) => format!(
            "SELECT ?g WHERE {{ GRAPH <urn:sparq:auth> {{ \
             ?g <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
                <https://sparq.dev/ns/auth#ConditionalGrant> ; \
                <https://sparq.dev/ns/auth#agent> <{a}> }} }}"
        ),
        None => "SELECT ?g WHERE { GRAPH <urn:sparq:auth> { \
             ?g <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
                <https://sparq.dev/ns/auth#ConditionalGrant> } }"
            .to_owned(),
    };
    sparq_engine::query(graph, &q).expect("query").rows.len()
}

/// A permission whose RECIPIENT is constrained to carol (not whoever materialized it).
fn recipient_policy() -> sparq_policy::Policy {
    parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/recip> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ;
                      odrl:operator odrl:eq ;
                      odrl:rightOperand <https://carol.ex/card#me> ] ] .
"#,
        "turtle",
    )
    .expect("policy parses")
}

// 14. A recipient constraint persists as a ConditionalGrant re-checked per session:
//     carol (the recipient) is granted; everyone else is denied — through accessible/
//     query_as — even though ALICE materialized it.
#[test]
fn recipient_constraint_persists_as_rechecked_condition() {
    // Inspect the materialized triples via the free-function form (we own the Graph).
    // Note: the materializing request party is ALICE, but the constraint names CAROL.
    let mut g = pod();
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    let out = materialize_permission_conditional(&mut g, &recipient_policy(), &req);
    assert!(out.granted, "faithful recipient maps to a condition: {out:?}");
    // A real ConditionalGrant head naming carol now lives in the auth view.
    assert_eq!(cond_grants_for(&g, Some(CAROL)), 1, "carol condition present");
    assert_eq!(cond_grants_for(&g, Some(ALICE)), 0, "no condition for the materializer");

    // RE-CHECK through the real enforcement path: rebuild a store over the SAME graph.
    let mut store = PodStore::new(pod());
    assert!(store.materialize_odrl_permission_conditional(&recipient_policy(), &req).granted);
    assert!(reads(&mut store, CAROL), "recipient carol granted");
    assert!(!reads(&mut store, ALICE), "materializing party is NOT auto-granted");
    assert!(!reads(&mut store, BOB), "unrelated agent denied");
    assert!(!reads(&mut store, "https://x.ex/#m"), "stranger denied");
    assert!(store.accessible(&Session::default(), Mode::Read).is_empty(), "anonymous denied");

    // End-to-end through query_as: only carol sees the content.
    let sel = "SELECT ?t WHERE { ?s <https://ex.dev/ns#title> ?t }";
    let carol = Session { agent: Some(CAROL), client: None };
    let alice = Session { agent: Some(ALICE), client: None };
    assert_eq!(store.query_as(&carol, Mode::Read, sel).unwrap().rows.len(), 1);
    assert_eq!(store.query_as(&alice, Mode::Read, sel).unwrap().rows.len(), 0);
}

// 15. A recipient `isPartOf` SET → one re-checked condition per member (OR).
#[test]
fn recipient_set_persists_as_multiple_conditions() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/set> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ;
                      odrl:operator odrl:isPartOf ;
                      odrl:rightOperand "https://bob.ex/card#me|https://carol.ex/card#me" ] ] .
"#,
        "turtle",
    )
    .unwrap();
    let mut store = PodStore::new(pod());
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission_conditional(&pol, &req).granted);

    assert!(reads(&mut store, BOB), "bob in set");
    assert!(reads(&mut store, CAROL), "carol in set");
    assert!(!reads(&mut store, ALICE), "alice not in set");
    assert!(!reads(&mut store, "https://dave.ex/#m"), "dave not in set");
}

// 16. An UNmappable constraint (`odrl:purpose`) STAYS one-shot: the persisted view
//     holds NO ConditionalGrant; access is the frozen check against the supplied
//     request context (granted only if purpose matched at materialization), scoped
//     to the materializing party.
#[test]
fn purpose_constraint_stays_one_shot() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/purp> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ;
                      odrl:operator odrl:eq ;
                      odrl:rightOperand "research" ] ] .
"#,
        "turtle",
    )
    .unwrap();

    // (a) purpose SATISFIED at materialization → one-shot grant (frozen) for alice.
    let req = Request::new(odrl("read"))
        .on(N1)
        .by(ALICE)
        .with(odrl("purpose"), Value::Str("research".to_owned()));
    let mut g = pod();
    let out = materialize_permission_conditional(&mut g, &pol, &req);
    assert!(out.granted, "purpose satisfied → one-shot grant: {out:?}");
    // NO ConditionalGrant was persisted (purpose has no faithful ACP analogue).
    assert_eq!(cond_grants_for(&g, None), 0, "purpose must NOT become a re-checked condition");

    // The frozen grant is scoped to the materializing party (alice).
    let mut store = PodStore::new(pod());
    assert!(store.materialize_odrl_permission_conditional(&pol, &req).granted);
    assert!(reads(&mut store, ALICE), "alice one-shot grant");
    assert!(!reads(&mut store, CAROL), "no widening");

    // (b) purpose NOT satisfied (missing context) → fail-closed, nothing granted.
    let mut store2 = PodStore::new(pod());
    let bad = Request::new(odrl("read")).on(N1).by(ALICE); // no purpose context
    assert!(!store2.materialize_odrl_permission_conditional(&pol, &bad).granted);
    assert!(!reads(&mut store2, ALICE), "unsatisfied purpose grants nothing");
}

// 17. MIXED constraints fail SAFE: recipient (mappable) + dateTime (unmappable) →
//     the WHOLE rule stays one-shot so the time bound is still enforced (frozen).
//     A persisted recipient-only condition would have LOST the time bound (over-grant).
#[test]
fn mixed_mappable_and_unmappable_stays_one_shot() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/mix> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ;
                      odrl:operator odrl:eq ;
                      odrl:rightOperand <https://carol.ex/card#me> ] ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator odrl:lteq ;
                      odrl:rightOperand "2020-01-01T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
"#,
        "turtle",
    )
    .unwrap();

    // Out-of-window request → DENY → nothing (the time bound is NOT dropped).
    let req = Request::new(odrl("read"))
        .on(N1)
        .by(CAROL)
        .with(odrl("dateTime"), Value::DateTime("2026-06-16T00:00:00Z".to_owned()));
    let mut g = pod();
    let out = materialize_permission_conditional(&mut g, &pol, &req);
    assert!(!out.granted, "out-of-window with mixed constraints must NOT grant: {out:?}");
    // No ConditionalGrant leaked carol an unconditional re-checked allow.
    assert_eq!(cond_grants_for(&g, None), 0, "no condition emitted when a bound is unmappable");
    let mut store = PodStore::new(pod());
    assert!(!store.materialize_odrl_permission_conditional(&pol, &req).granted);
    assert!(!reads(&mut store, CAROL), "no over-grant from dropping the time bound");
}

// 18. Compose-with-deny: a matching prohibition overrides the conditional path
//     (deny-overrides) — nothing is materialized.
#[test]
fn prohibition_overrides_conditional_path() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/po> a odrl:Set ;
  odrl:permission [ odrl:action odrl:read ; odrl:target <https://pod.ex/notes/n1> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
                      odrl:rightOperand <https://carol.ex/card#me> ] ] ;
  odrl:prohibition [ odrl:action odrl:read ; odrl:target <https://pod.ex/notes/n1> ] .
"#,
        "turtle",
    )
    .unwrap();
    let mut store = PodStore::new(pod());
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    let out = store.materialize_odrl_permission_conditional(&pol, &req);
    assert!(!out.granted, "prohibition overrides: {out:?}");
    assert!(!reads(&mut store, CAROL), "deny-overrides: carol gets nothing");
}

// 19. A bare permission with NO constraints, via the conditional entry point, grants
//     PUBLIC (any session) — only valid because action/target/duties already held.
//     (Documents the no-constraint → auth:Public head behaviour.)
#[test]
fn no_constraint_conditional_grants_public() {
    let pol = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/pub> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <https://pod.ex/notes/n1> ] .
"#,
        "turtle",
    )
    .unwrap();
    // Free-function form: a PUBLIC ConditionalGrant head is materialized.
    let mut g = pod();
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    let out = materialize_permission_conditional(&mut g, &pol, &req);
    assert!(out.granted, "bare permission grants: {out:?}");
    assert_eq!(cond_grants_for(&g, Some("https://sparq.dev/ns/auth#Public")), 1);

    // Through the real enforcement path: public read = any agent AND anonymous.
    let mut store = PodStore::new(pod());
    assert!(store.materialize_odrl_permission_conditional(&pol, &req).granted);
    assert!(reads(&mut store, BOB));
    assert!(!store.accessible(&Session::default(), Mode::Read).is_empty(), "anon public read");
}
