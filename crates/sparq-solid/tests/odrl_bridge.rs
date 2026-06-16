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

// ===========================================================================
// [OPUS-4.8] sq-dpk4 — refresh / REVOCATION of bridged ODRL grants when the
// underlying ODRL policy changes. SECURITY-SENSITIVE: a withdrawn permission, a
// lapsed time window, or a re-evaluation that now Denies must LOSE access; a static
// WAC/ACP grant must NEVER be dropped; a still-valid bridged grant must survive.
// All assertions go through the REAL enforcement path (accessible / query_as).
// ===========================================================================
use sparq_solid::BridgeKind;

/// alice MAY read n1 ONLY until 2026-01-01 (a time-windowed permission).
fn windowed_read_policy() -> sparq_policy::Policy {
    parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/win> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-01-01T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
"#,
        "turtle",
    )
    .expect("policy parses")
}

/// A policy that grants NOTHING (the permission has been WITHDRAWN entirely).
fn empty_policy() -> sparq_policy::Policy {
    parse_policy_str(
        r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> . <urn:pol/read> a odrl:Set ."#,
        "turtle",
    )
    .expect("policy parses")
}

// 20. ADVERSARIAL "stale grant loses access": a bridged grant is materialized →
//     access granted; the ODRL policy then WITHDRAWS the permission → refresh →
//     access is GONE through the real enforcement path. We adversarially check the
//     grant did not survive in ANY form (accessible, query_as, the raw auth view, the
//     provenance graph) and that re-refreshing cannot resurrect it.
#[test]
fn withdrawn_permission_loses_access_after_refresh() {
    let mut store = PodStore::new(pod());
    let req = Request::new(odrl("read")).on(N1).by(ALICE);

    // Materialize → alice has read.
    assert!(store.materialize_odrl_permission(&read_policy(), &req).granted);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(
        store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1),
        "bridged grant is live before withdrawal",
    );
    let sel = "SELECT ?t WHERE { ?s <https://ex.dev/ns#title> ?t }";
    assert_eq!(store.query_as(&alice, Mode::Read, sel).unwrap().rows.len(), 1);

    // The policy WITHDRAWS the permission → refresh against the new (empty) policy.
    let (matched, retracted) =
        store.refresh_odrl_grant(&empty_policy(), &req, BridgeKind::Permission);
    assert!(matched, "the tracked grant slot matched");
    assert_eq!(retracted, 1, "the withdrawn grant was retracted");

    // ADVERSARIAL: access is GONE through every observable surface.
    assert!(
        store.accessible(&alice, Mode::Read).is_empty(),
        "STALE GRANT MUST LOSE ACCESS: alice can no longer read n1",
    );
    assert_eq!(
        store.query_as(&alice, Mode::Read, sel).unwrap().rows.len(),
        0,
        "query_as returns nothing after revocation",
    );
    // The raw auth view holds no residual bridged grant for alice…
    let leftover = "SELECT ?p ?o WHERE { GRAPH <urn:sparq:auth> { \
        <https://alice.ex/card#me> ?p ?o } }";
    assert_eq!(sparq_engine::query(&store.graph, leftover).unwrap().rows.len(), 0,
        "no residual alice triple in the enforcement view");
    // …and the provenance graph was cleared of it.
    let prov = "SELECT ?s ?p ?o WHERE { GRAPH <urn:sparq:auth-bridged> { ?s ?p ?o } }";
    assert_eq!(sparq_engine::query(&store.graph, prov).unwrap().rows.len(), 0,
        "no residual provenance after retraction");
    // Re-refreshing cannot resurrect a dropped grant (the ledger is empty now).
    assert_eq!(store.refresh_odrl_grants(), 0, "nothing left to retract");
    assert!(store.accessible(&alice, Mode::Read).is_empty(), "stays revoked");
}

// 21. LAPSED TIME WINDOW: a windowed grant valid at materialization-time loses access
//     once the window lapses (re-evaluate with a NOW past the bound) → refresh → gone.
#[test]
fn lapsed_time_window_loses_access_after_refresh() {
    let mut store = PodStore::new(pod());
    let pol = windowed_read_policy();

    // In-window request → granted.
    let in_window = Request::new(odrl("read"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2025-06-01T00:00:00Z".to_owned()));
    assert!(store.materialize_odrl_permission(&pol, &in_window).granted);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1));

    // The window LAPSES: re-evaluate with a NOW past the bound (same policy, new ctx).
    let now_past = Request::new(odrl("read"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2026-06-16T00:00:00Z".to_owned()));
    let (matched, retracted) =
        store.refresh_odrl_grant(&pol, &now_past, BridgeKind::Permission);
    assert!(matched);
    assert_eq!(retracted, 1, "lapsed window retracts the grant");
    assert!(
        store.accessible(&alice, Mode::Read).is_empty(),
        "lapsed time window: access gone",
    );
}

// 22. RE-EVAL NOW DENIES (a prohibition is added): a bridged write grant is revoked
//     when the refreshed policy now carries a matching prohibition (deny-overrides).
#[test]
fn reeval_now_denies_loses_access_after_refresh() {
    let mut store = PodStore::new(pod());
    let permit = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/w> a odrl:Set ; odrl:permission [
    odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let req = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_policy(&permit, &req).granted);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Write).iter().any(|g| g.as_str() == N1));

    // The policy now ADDS a prohibition on the same action → re-eval Denies.
    let now_prohibited = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/w> a odrl:Set ;
    odrl:permission [ odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] ;
    odrl:prohibition [ odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let (matched, _) = store.refresh_odrl_grant(&now_prohibited, &req, BridgeKind::Policy);
    assert!(matched);
    // deny-overrides: even though materialize_policy emits a deny on replay, the net
    // enforcement result is DENIED (the allow is overridden upstream / by the deny).
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "re-eval now Denies: alice can no longer write n1",
    );
}

// 23. A STILL-VALID bridged grant SURVIVES refresh (no spurious retraction).
#[test]
fn valid_bridged_grant_survives_refresh() {
    let mut store = PodStore::new(pod());
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&read_policy(), &req).granted);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1));

    // Plain refresh (policy unchanged) re-evaluates and KEEPS the still-valid grant.
    assert_eq!(store.refresh_odrl_grants(), 0, "valid grant not retracted");
    assert!(
        store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1),
        "still-valid bridged grant survives refresh",
    );
    // Refresh against the SAME policy also keeps it.
    let (matched, retracted) =
        store.refresh_odrl_grant(&read_policy(), &req, BridgeKind::Permission);
    assert!(matched);
    assert_eq!(retracted, 0);
    assert!(store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1));
}

// 24. A STATIC WAC grant is NOT dropped by a bridged-grant refresh (provenance keeps
//     static and bridged apart). bob (static) keeps read; alice (bridged, then revoked)
//     loses it — in the SAME store, through the SAME enforcement path.
#[test]
fn static_grant_not_dropped_by_refresh() {
    // bob's static .acl grant + a content graph.
    let nq = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/notes/n1> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/notes/n1.acl> .
"#;
    let mut store = PodStore::new(Graph::load_dataset(nq, "nquads").unwrap());
    store.materialize_wac().expect("wac materializes");

    fn reads_n1(s: &mut PodStore, agent: &str) -> bool {
        let sess = Session { agent: Some(agent), client: None };
        s.accessible(&sess, Mode::Read).iter().any(|g| g.as_str() == N1)
    }

    // Bridge alice on top of bob's static grant.
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&read_policy(), &req).granted);
    assert!(reads_n1(&mut store, BOB), "bob static before");
    assert!(reads_n1(&mut store, ALICE), "alice bridged before");

    // Revoke alice's bridged grant. bob's STATIC grant must be untouched.
    let (matched, retracted) =
        store.refresh_odrl_grant(&empty_policy(), &req, BridgeKind::Permission);
    assert!(matched);
    assert_eq!(retracted, 1);
    assert!(reads_n1(&mut store, BOB), "STATIC GRANT PRESERVED: bob still reads n1");
    assert!(!reads_n1(&mut store, ALICE), "bridged grant revoked");
}

// 25. PROVENANCE distinguishes bridged vs static: a static grant never appears in the
//     bridged-provenance graph; a bridged grant does (and exactly the auth triple it
//     emitted).
#[test]
fn provenance_distinguishes_bridged_from_static() {
    let nq = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/notes/n1> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/notes/n1.acl> .
"#;
    let mut store = PodStore::new(Graph::load_dataset(nq, "nquads").unwrap());
    store.materialize_wac().expect("wac materializes");
    // After a pure static materialize, the provenance graph is empty.
    let prov_all = "SELECT ?s ?p ?o WHERE { GRAPH <urn:sparq:auth-bridged> { ?s ?p ?o } }";
    assert_eq!(sparq_engine::query(&store.graph, prov_all).unwrap().rows.len(), 0,
        "no provenance for static grants");

    // Bridge alice → exactly her grant triple appears in provenance, bob's does not.
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&read_policy(), &req).granted);
    let alice_prov = "SELECT ?p WHERE { GRAPH <urn:sparq:auth-bridged> { \
        <https://alice.ex/card#me> <https://sparq.dev/ns/auth#read> <https://pod.ex/notes/n1> } }";
    assert_eq!(sparq_engine::query(&store.graph, alice_prov).unwrap().rows.len(), 1,
        "alice's bridged grant is marked in provenance");
    let bob_prov = "SELECT ?p ?o WHERE { GRAPH <urn:sparq:auth-bridged> { \
        <https://bob.ex/card#me> ?p ?o } }";
    assert_eq!(sparq_engine::query(&store.graph, bob_prov).unwrap().rows.len(), 0,
        "bob's STATIC grant is NOT in provenance");
}

// 26. A STATIC RE-MATERIALIZATION re-applies still-valid bridged grants (reconcile):
//     materialize_wac rebuilds <urn:sparq:auth> wholesale, but a valid bridged grant is
//     replayed back on top — and a static grant change still takes effect.
#[test]
fn static_rematerialization_preserves_valid_bridged_grant() {
    let nq = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/notes/n1> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/notes/n1.acl> .
"#;
    let mut store = PodStore::new(Graph::load_dataset(nq, "nquads").unwrap());
    store.materialize_wac().expect("wac");
    let req = Request::new(odrl("read")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&read_policy(), &req).granted);

    fn reads_n1(s: &mut PodStore, agent: &str) -> bool {
        let sess = Session { agent: Some(agent), client: None };
        s.accessible(&sess, Mode::Read).iter().any(|g| g.as_str() == N1)
    }
    assert!(reads_n1(&mut store, ALICE), "alice bridged before re-materialize");

    // A wholesale static re-materialization would normally CLOBBER the bridged grant.
    store.materialize_wac().expect("re-materialize");
    assert!(reads_n1(&mut store, BOB), "bob static after re-materialize");
    assert!(
        reads_n1(&mut store, ALICE),
        "RECONCILE: the valid bridged grant survives a wholesale static re-materialization",
    );
}

// 27. FAIL-CLOSED on AMBIGUOUS re-eval: a windowed grant refreshed with NO dateTime
//     evidence (cannot prove the window still holds) is RETRACTED (never left stale).
#[test]
fn ambiguous_reeval_retracts_fail_closed() {
    let mut store = PodStore::new(pod());
    let pol = windowed_read_policy();
    let in_window = Request::new(odrl("read"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2025-06-01T00:00:00Z".to_owned()));
    assert!(store.materialize_odrl_permission(&pol, &in_window).granted);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Read).iter().any(|g| g.as_str() == N1));

    // Refresh with NO dateTime evidence → constraint cannot be proven → fail-closed Deny.
    let no_evidence = Request::new(odrl("read")).on(N1).by(ALICE);
    let (matched, retracted) =
        store.refresh_odrl_grant(&pol, &no_evidence, BridgeKind::Permission);
    assert!(matched);
    assert_eq!(retracted, 1, "ambiguous re-eval is retracted, not left stale");
    assert!(
        store.accessible(&alice, Mode::Read).is_empty(),
        "FAIL-CLOSED: no evidence the window holds → access retracted",
    );
}

// ===========================================================================
// [OPUS-4.8] sq-2pcf — DENY RETRACTION on prohibition withdrawal. The symmetric
// dual of sq-dpk4's grant retraction, but with the OPPOSITE fail-closed bias: a
// materialized `auth:deny*` is retracted (access restored) ONLY when the underlying
// ODRL Prohibition is DEFINITELY withdrawn / lapsed — on an *ambiguous* re-eval the
// deny is KEPT (never restore access on missing evidence). These tests drive the
// real enforcement path (`accessible` / `query_as` / `update_as`).
// ===========================================================================

/// alice is prohibited from writing n1 ONLY while a window holds (dateTime < bound).
/// Used to exercise definite-lapse vs ambiguous (no-evidence) deny retraction.
fn windowed_write_prohibition() -> sparq_policy::Policy {
    parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/winprohib> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:modify ;
    odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
                      odrl:rightOperand "2026-01-01T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
"#,
        "turtle",
    )
    .expect("policy parses")
}

/// A policy that prohibits NOTHING (the prohibition has been WITHDRAWN entirely).
fn empty_prohibition_policy() -> sparq_policy::Policy {
    parse_policy_str(
        r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> . <urn:pol/prohib> a odrl:Set ."#,
        "turtle",
    )
    .expect("policy parses")
}

// 28. WITHDRAWN PROHIBITION → DENY RETRACTED → access RESTORED through the real
//     enforcement path — but ONLY because a still-valid allow grant re-exposes the
//     slot. A standalone deny that is withdrawn restores nothing (test 29).
#[test]
fn withdrawn_prohibition_restores_access_after_refresh() {
    let mut store = PodStore::new(pod());
    let wreq = Request::new(odrl("modify")).on(N1).by(ALICE);

    // alice has a (still-valid, unconstrained) WRITE permit AND a matching prohibition
    // → deny-overrides denies write now.
    let permit = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/w> a odrl:Set ; odrl:permission [
    odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    assert!(store.materialize_odrl_permission(&permit, &wreq).granted);
    assert!(store.materialize_odrl_prohibition(&write_prohibition(), &wreq).prohibited);

    let alice = Session { agent: Some(ALICE), client: None };
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "deny-overrides: alice is denied write while the prohibition holds",
    );

    // The Prohibition is WITHDRAWN entirely → refresh the deny entry against the empty
    // policy. The deny is DEFINITELY gone (no prohibition structurally names the request).
    let (matched, retracted) =
        store.refresh_odrl_grant(&empty_prohibition_policy(), &wreq, BridgeKind::Prohibition);
    assert!(matched, "the tracked deny slot matched");
    assert_eq!(retracted, 1, "the withdrawn prohibition's deny was retracted");

    // ACCESS RESTORED: deny gone + the allow grant survives → alice can write again.
    assert!(
        store.accessible(&alice, Mode::Write).iter().any(|g| g.as_str() == N1),
        "deny retracted + allow grant intact → write access restored",
    );
    // And the write-path update enforcement now permits it.
    let ins = "INSERT DATA { GRAPH <https://pod.ex/notes/n1> { \
        <https://pod.ex/notes/n1#it> <https://ex.dev/ns#tag> \"y\" } }";
    assert!(store.update_as(&alice, ins).is_ok(), "restored write update succeeds");
    // No residual deny triple in the auth view.
    let leftover = "SELECT ?o WHERE { GRAPH <urn:sparq:auth> { \
        <https://alice.ex/card#me> <https://sparq.dev/ns/auth#denyWrite> ?o } }";
    assert_eq!(sparq_engine::query(&store.graph, leftover).unwrap().rows.len(), 0,
        "no residual denyWrite after retraction");
}

// 29. WITHDRAWN STANDALONE DENY restores NOTHING: a deny with no underlying allow grant,
//     when retracted, does not widen access (the lack of a grant still denies — the deny
//     retraction is fail-closed by construction).
#[test]
fn withdrawn_standalone_deny_grants_no_access() {
    let mut store = PodStore::new(pod());
    let wreq = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_prohibition(&write_prohibition(), &wreq).prohibited);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Write).is_empty(), "no grant → denied");

    // Withdraw the prohibition; the deny is retracted but there was never an allow.
    let (matched, retracted) =
        store.refresh_odrl_grant(&empty_prohibition_policy(), &wreq, BridgeKind::Prohibition);
    assert!(matched);
    assert_eq!(retracted, 1, "the standalone deny is retracted");
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "retracting a standalone deny restores no access (no grant to re-expose)",
    );
}

// 30. A STILL-APPLICABLE prohibition SURVIVES refresh — the deny is KEPT, access stays
//     denied (no spurious restoration).
#[test]
fn applicable_prohibition_survives_refresh() {
    let mut store = PodStore::new(pod());
    let wreq = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_prohibition(&write_prohibition(), &wreq).prohibited);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Write).is_empty());

    // Plain refresh (policy unchanged) → the prohibition still matches → deny KEPT.
    assert_eq!(store.refresh_odrl_grants(), 0, "applicable deny not retracted");
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "still-applicable prohibition: deny survives refresh, access stays denied",
    );
    // Refresh against the SAME prohibition policy also keeps it.
    let (matched, retracted) =
        store.refresh_odrl_grant(&write_prohibition(), &wreq, BridgeKind::Prohibition);
    assert!(matched);
    assert_eq!(retracted, 0, "deny kept on an unchanged, still-matching prohibition");
    assert!(store.accessible(&alice, Mode::Write).is_empty(), "stays denied");
}

// 31. CORE sq-2pcf: AMBIGUOUS re-eval of a windowed prohibition KEEPS the deny
//     (fail-closed) — access is NOT restored when we cannot prove the carve-out is gone.
//     This is the asymmetry vs grant retraction (a windowed GRANT is dropped on ambiguity).
#[test]
fn ambiguous_prohibition_reeval_keeps_deny_fail_closed() {
    let mut store = PodStore::new(pod());
    // A windowed prohibition that holds at materialization time (now < bound).
    let in_window = Request::new(odrl("modify"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2025-06-01T00:00:00Z".to_owned()));
    assert!(store
        .materialize_odrl_prohibition(&windowed_write_prohibition(), &in_window)
        .prohibited);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Write).is_empty(), "denied while window holds");

    // Refresh with NO dateTime evidence → we CANNOT prove the window lapsed → AMBIGUOUS.
    // The deny must be KEPT (fail-closed: do NOT restore access on missing evidence).
    let no_evidence = Request::new(odrl("modify")).on(N1).by(ALICE);
    let (matched, retracted) = store.refresh_odrl_grant(
        &windowed_write_prohibition(),
        &no_evidence,
        BridgeKind::Prohibition,
    );
    assert!(matched);
    assert_eq!(retracted, 0, "AMBIGUOUS deny is KEPT, not retracted");
    assert!(
        store.accessible(&alice, Mode::Write).is_empty(),
        "FAIL-CLOSED: no evidence the prohibition lapsed → deny kept → access stays denied",
    );
    // The denyWrite triple is still present in the auth view (re-emitted on refresh).
    let still = "SELECT ?o WHERE { GRAPH <urn:sparq:auth> { \
        <https://alice.ex/card#me> <https://sparq.dev/ns/auth#denyWrite> ?o } }";
    assert_eq!(sparq_engine::query(&store.graph, still).unwrap().rows.len(), 1,
        "ambiguous deny re-emitted (kept) in the enforcement view");
}

// 32. DEFINITELY-LAPSED window → deny RETRACTED: when the refresh request supplies
//     evidence the window has PROVABLY lapsed (now >= bound), the carve-out is known
//     gone → deny retracted. Paired with an allow grant → access restored.
#[test]
fn definitely_lapsed_prohibition_retracts_deny() {
    let mut store = PodStore::new(pod());

    // alice has a still-valid WRITE permit + a windowed prohibition (holds now).
    let permit = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/w> a odrl:Set ; odrl:permission [
    odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
    odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let permit_req = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_permission(&permit, &permit_req).granted);
    let in_window = Request::new(odrl("modify"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2025-06-01T00:00:00Z".to_owned()));
    assert!(store
        .materialize_odrl_prohibition(&windowed_write_prohibition(), &in_window)
        .prohibited);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Write).is_empty(), "denied while window holds");

    // Refresh with evidence the window has PROVABLY lapsed (now >= 2026-01-01 bound,
    // operator is `lt`, so now is NOT < bound → constraint definitely false → Withdrawn).
    let now_lapsed = Request::new(odrl("modify"))
        .on(N1)
        .by(ALICE)
        .with(odrl("dateTime"), Value::DateTime("2026-06-16T00:00:00Z".to_owned()));
    let (matched, retracted) = store.refresh_odrl_grant(
        &windowed_write_prohibition(),
        &now_lapsed,
        BridgeKind::Prohibition,
    );
    assert!(matched);
    assert_eq!(retracted, 1, "provably-lapsed prohibition's deny is retracted");
    assert!(
        store.accessible(&alice, Mode::Write).iter().any(|g| g.as_str() == N1),
        "provably-lapsed deny retracted + allow intact → write access restored",
    );
}

// 33. A STATIC WAC grant is NEVER retracted by a bridged-deny refresh — only the bridged
//     deny is. bob's static read grant survives the full ledger refresh.
#[test]
fn static_grant_never_dropped_by_deny_refresh() {
    let nq = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#accessTo> <https://pod.ex/notes/n1> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://bob.ex/card#me> <https://pod.ex/notes/n1.acl> .
<https://pod.ex/notes/n1.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/notes/n1.acl> .
"#;
    let mut store = PodStore::new(Graph::load_dataset(nq, "nquads").unwrap());
    store.materialize_wac().expect("wac materializes");

    fn reads_n1(s: &mut PodStore, agent: &str) -> bool {
        let sess = Session { agent: Some(agent), client: None };
        s.accessible(&sess, Mode::Read).iter().any(|g| g.as_str() == N1)
    }
    assert!(reads_n1(&mut store, BOB), "bob static read before");

    // Bridge alice's WRITE prohibition (a bridged deny) on top of the static baseline.
    let wreq = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_prohibition(&write_prohibition(), &wreq).prohibited);

    // Withdraw alice's prohibition → refresh. The bridged deny is retracted; bob's STATIC
    // grant (in the captured baseline, never in the ledger) is untouched.
    let (matched, retracted) =
        store.refresh_odrl_grant(&empty_prohibition_policy(), &wreq, BridgeKind::Prohibition);
    assert!(matched);
    assert_eq!(retracted, 1, "only the bridged deny is retracted");
    assert!(
        reads_n1(&mut store, BOB),
        "STATIC GRANT PRESERVED across a bridged-deny refresh",
    );
}

// 34. DENY-OVERRIDES composition stays correct ACROSS a deny refresh: a permit + a
//     prohibition bridged via a single Policy entry; the refresh re-applies the allow and
//     re-evaluates the deny with the fail-closed deny rule. A withdrawn prohibition (in
//     the refreshed Policy) drops the deny and re-exposes the allow.
#[test]
fn policy_refresh_deny_overrides_composition() {
    let mut store = PodStore::new(pod());
    let both = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/both> a odrl:Set ;
    odrl:permission [ odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] ;
    odrl:prohibition [ odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let req = Request::new(odrl("modify")).on(N1).by(ALICE);
    assert!(store.materialize_odrl_policy(&both, &req).prohibited);
    let alice = Session { agent: Some(ALICE), client: None };
    assert!(store.accessible(&alice, Mode::Write).is_empty(), "deny-overrides denies");

    // Refresh against a Policy that keeps the permission but DROPS the prohibition.
    let permit_only = parse_policy_str(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/both> a odrl:Set ;
    odrl:permission [ odrl:action odrl:modify ; odrl:target <https://pod.ex/notes/n1> ;
        odrl:assignee <https://alice.ex/card#me> ] .
"#,
        "turtle",
    )
    .unwrap();
    let (matched, _retracted) = store.refresh_odrl_grant(&permit_only, &req, BridgeKind::Policy);
    assert!(matched);
    assert!(
        store.accessible(&alice, Mode::Write).iter().any(|g| g.as_str() == N1),
        "deny dropped + allow re-applied → write restored under deny-overrides",
    );
}
