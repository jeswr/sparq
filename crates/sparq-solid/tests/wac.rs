//! WAC correctness: hand-computed expected access for the fixture pod
//! (crates/sparq-solid/src/fixture.rs — see the subtree semantics table there).

use sparq_core::Graph;
use sparq_solid::{wac_fixture, Mode, PodStore, Session};

fn store() -> PodStore {
    let g = Graph::load_dataset(&wac_fixture(), "nquads").expect("fixture loads");
    let mut s = PodStore::new(g);
    let stats = s.materialize_wac().expect("wac materializes");
    assert!(stats.auth_triples > 0, "auth view non-empty");
    s
}

fn can(s: &mut PodStore, agent: Option<&str>, client: Option<&str>, mode: Mode, graph: &str) -> bool {
    // WAC has no issuer dimension; the field stays None for these cases. [OPUS-4.8] sq-3jtd.6
    s.accessible(&Session { agent, client, issuer: None }, mode).iter().any(|g| g.as_str() == graph)
}

use sparq_solid::fixture::{ALICE, APP, BOB, CAROL, DAVE};

#[test]
fn wac_expected_access_matrix() {
    let mut s = store();
    let r = Mode::Read;

    // 1. owner via root acl:default, inherited across DEPTH 4
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/priv0/c4/g0/d0.ttl"));
    // 2. …and nobody else
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/priv0/c4/g0/d0.ttl"));
    // 3. public subtree: anonymous + any agent (inherited + restated-ACL paths)
    assert!(can(&mut s, None, None, r, "https://pod.ex/pub1/c2/g3/d1.ttl"));
    assert!(can(&mut s, None, None, r, "https://pod.ex/pub1/c0/g0/d0.ttl"));
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/pub1/c2/g3/d1.ttl"));
    // 4. agentGroup: carol + bob read AND write via vcard group; dave does not
    assert!(can(&mut s, Some(CAROL), None, r, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(CAROL), None, Mode::Write, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(!can(&mut s, Some(DAVE), None, r, "https://pod.ex/team2/c3/g0/d0.ttl"));
    // 5. NEAREST-ACL SEMANTICS: team2's own ACL does NOT re-grant alice — the root
    //    owner grant is shadowed (this is the "no closer ACL" negation working)
    assert!(!can(&mut s, Some(ALICE), None, r, "https://pod.ex/team2/c3/g0/d0.ttl"));
    // 6. acl:default vs acl:accessTo: bob reads friends3 MEMBERS but not the container
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/friends3/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/friends3/"));
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/friends3/"));
    assert!(!can(&mut s, Some(CAROL), None, r, "https://pod.ex/friends3/c2/g0/d0.ttl"));
    // 7. acl:AuthenticatedAgent: any logged-in agent, but not anonymous
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/mixed4/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(DAVE), None, r, "https://pod.ex/mixed4/c3/g0/d0.ttl"));
    assert!(!can(&mut s, None, None, r, "https://pod.ex/mixed4/c3/g0/d0.ttl"));
    // 8. deep container override (mixed4/c0): carol only — alice and authenticated lose
    assert!(can(&mut s, Some(CAROL), None, r, "https://pod.ex/mixed4/c0/g1/d2.ttl"));
    assert!(!can(&mut s, Some(ALICE), None, r, "https://pod.ex/mixed4/c0/g1/d2.ttl"));
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/mixed4/c0/g1/d2.ttl"));
    // 8b. a deeper OWNER-ONLY ACL (mixed4/c2) narrows: authenticated agents lose it
    assert!(!can(&mut s, Some(CAROL), None, r, "https://pod.ex/mixed4/c2/g0/d0.ttl"));
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/mixed4/c2/g0/d0.ttl"));
    // 9. resource-specific ACL overrides the container default for THAT document only
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/mixed4/c1/g0/d0.ttl"));
    assert!(!can(&mut s, Some(ALICE), None, r, "https://pod.ex/mixed4/c1/g0/d0.ttl"));
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/mixed4/c1/g0/d1.ttl"));
    // 10. acl:origin = the (user, app) pair: bob only through the app client
    assert!(can(&mut s, Some(BOB), Some(APP), r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(BOB), Some("https://evil.ex"), r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(CAROL), Some(APP), r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    // 11. acl:Control gates the ACL document itself
    assert!(can(&mut s, Some(ALICE), None, Mode::Control, "https://pod.ex/priv0/c4/g0/d0.ttl"));
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/.acl"));
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/.acl"));
    // 12. fail-closed: anonymous sees only the public subtree
    let anon = s.accessible(&Session::default(), r);
    assert!(anon.iter().all(|g| g.as_str().starts_with("https://pod.ex/pub1/")));
}

#[test]
fn wac_rematerialization_revokes() {
    // an ACL change = swap the .acl graph + re-materialize; the session cache resets
    let mut s = store();
    assert!(can(&mut s, Some(CAROL), None, Mode::Read, "https://pod.ex/team2/c3/g0/d0.ttl"));
    // remove carol from the team group document
    let name = oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked("https://pod.ex/groups/team"));
    let pos = s.graph.named.iter().position(|(n, _)| *n == name).expect("group graph");
    let without_carol = "<https://pod.ex/groups/team#g> \
         <http://www.w3.org/2006/vcard/ns#hasMember> <https://bob.ex/card#me> .";
    s.graph.named[pos].1 = Graph::load_str(without_carol, "ntriples").unwrap();
    s.materialize_wac().expect("re-materialize");
    assert!(!can(&mut s, Some(CAROL), None, Mode::Read, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(BOB), None, Mode::Read, "https://pod.ex/team2/c3/g0/d0.ttl"));
}

/// [OPUS-4.8] sq-xor3: the WRITE path mirrors the read matrix. A SPARQL Update is
/// permitted iff the actor holds write permission on EVERY graph it could touch, checked
/// (fail-closed) before any mutation. Expected outcomes hand-derived from the fixture's
/// write grants: ALICE owns Read/Write/Control via the root `acl:default` (inherited but
/// SHADOWED out of team2/), the team group (BOB, CAROL) has Read+Write on team2/, and
/// Control→`auth:write` on the `.acl` graph. See tests/update.rs for the per-operation
/// matrix; this asserts the matrix lines up with the read-path grants.
#[test]
fn wac_write_enforcement_matches_grants() {
    let mut s = store();
    let ins = |g: &str| format!("INSERT DATA {{ GRAPH <{g}> {{ <{g}#it> <https://ex.dev/ns#k> \"v\" }} }}");

    // owner writes inherited to depth 4 (read-matrix #1 has the Read twin)
    let priv0 = "https://pod.ex/priv0/c4/g0/d0.ttl";
    assert!(can(&mut s, Some(ALICE), None, Mode::Write, priv0));
    s.update_as(&Session { agent: Some(ALICE), client: None, issuer: None }, &ins(priv0)).expect("alice writes");
    // …and nobody else (read-matrix #2 twin)
    assert!(!can(&mut s, Some(BOB), None, Mode::Write, priv0));
    assert!(s.update_as(&Session { agent: Some(BOB), client: None, issuer: None }, &ins(priv0)).is_err());

    // group read+write; nearest-ACL shadows alice (read-matrix #4/#5 twins)
    let team2 = "https://pod.ex/team2/c3/g0/d0.ttl";
    assert!(can(&mut s, Some(CAROL), None, Mode::Write, team2));
    s.update_as(&Session { agent: Some(CAROL), client: None, issuer: None }, &ins(team2)).expect("carol writes");
    assert!(!can(&mut s, Some(ALICE), None, Mode::Write, team2));
    assert!(s.update_as(&Session { agent: Some(ALICE), client: None, issuer: None }, &ins(team2)).is_err());

    // Control gates the .acl document: writing it needs Write, granted only to alice
    let acl = "https://pod.ex/.acl";
    assert!(can(&mut s, Some(ALICE), None, Mode::Write, acl)); // via Control->write
    assert!(!can(&mut s, Some(BOB), None, Mode::Write, acl));
    assert!(s.update_as(&Session { agent: Some(BOB), client: None, issuer: None }, &ins(acl)).is_err());
}
