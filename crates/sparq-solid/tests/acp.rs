//! ACP correctness: hand-computed expected access for the ACP fixture variant —
//! cumulative inheritance, allOf/anyOf user-app pairs, deny-overrides, noneOf.

use sparq_core::Graph;
use sparq_solid::fixture::{acp_fixture, ALICE, APP, BOB, CAROL, DAVE};
use sparq_solid::{Mode, PodStore, Session};

fn store() -> PodStore {
    let g = Graph::load_dataset(&acp_fixture(), "nquads").expect("fixture loads");
    let mut s = PodStore::new(g);
    let stats = s.materialize_acp().expect("acp materializes");
    assert_eq!(stats.strata_facts.len(), 3, "three strata");
    assert!(stats.auth_triples > 0);
    s
}

fn can(s: &mut PodStore, agent: Option<&str>, client: Option<&str>, mode: Mode, graph: &str) -> bool {
    s.accessible(&Session { agent, client }, mode).iter().any(|g| g.as_str() == graph)
}

#[test]
fn acp_expected_access_matrix() {
    let mut s = store();
    let r = Mode::Read;

    // 1. owner: root memberAccessControl reaches depth 4 (and the root itself via
    //    accessControl)
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/priv0/c4/g0/d0.ttl"));
    assert!(can(&mut s, Some(ALICE), None, Mode::Write, "https://pod.ex/priv0/c4/g0/d0.ttl"));
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/"));
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/priv0/c4/g0/d0.ttl"));
    // 2. CUMULATIVE inheritance (the key WAC difference): pub1's own ACR does NOT
    //    shadow the root owner policy — alice keeps access; and the public matcher
    //    grants everyone incl. anonymous
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/pub1/c2/g3/d1.ttl"));
    assert!(can(&mut s, None, None, r, "https://pod.ex/pub1/c2/g3/d1.ttl"));
    assert!(can(&mut s, Some(DAVE), None, r, "https://pod.ex/pub1/c2/g3/d1.ttl"));
    // 3. anyOf over enumerated agents
    assert!(can(&mut s, Some(BOB), None, Mode::Write, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(CAROL), None, r, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(!can(&mut s, Some(DAVE), None, r, "https://pod.ex/team2/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(ALICE), None, r, "https://pod.ex/team2/c3/g0/d0.ttl")); // cumulative root
    // 4. the NATIVE user/app pair: allOf { agent bob } { client app } — exactly the
    //    (user, application) pair, nothing else
    assert!(can(&mut s, Some(BOB), Some(APP), r, "https://pod.ex/friends3/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(BOB), Some("https://evil.ex"), r, "https://pod.ex/friends3/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(BOB), None, r, "https://pod.ex/friends3/c2/g0/d0.ttl"));
    assert!(!can(&mut s, Some(CAROL), Some(APP), r, "https://pod.ex/friends3/c2/g0/d0.ttl"));
    // 5. DENY-OVERRIDES: dave is denied even though the public policy allows
    assert!(!can(&mut s, Some(DAVE), None, r, "https://pod.ex/mixed4/c3/g0/d0.ttl"));
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/mixed4/c3/g0/d0.ttl"));
    assert!(can(&mut s, None, None, r, "https://pod.ex/mixed4/c3/g0/d0.ttl"));
    // 6. noneOf: public-except-carol (a conditional grant evaluated per session)
    assert!(!can(&mut s, Some(CAROL), None, r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    assert!(can(&mut s, Some(BOB), None, r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    assert!(can(&mut s, None, None, r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
    assert!(can(&mut s, Some(DAVE), None, r, "https://pod.ex/origin5/c2/g0/d0.ttl"));
}

#[test]
fn acp_rematerialization_picks_up_policy_change() {
    let mut s = store();
    assert!(!can(&mut s, Some(DAVE), None, Mode::Read, "https://pod.ex/team2/c3/g0/d0.ttl"));
    // extend team2's anyOf with dave: swap the .acr graph, re-materialize
    let name = oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked("https://pod.ex/team2/.acr"));
    let pos = s.graph.named.iter().position(|(n, _)| *n == name).expect("acr graph");
    let acr = "https://pod.ex/team2/.acr";
    let team = format!(
        "<{acr}> <http://www.w3.org/ns/solid/acp#memberAccessControl> <{acr}#ctl-team> .\n\
         <{acr}#ctl-team> <http://www.w3.org/ns/solid/acp#apply> <{acr}#pol-team> .\n\
         <{acr}#pol-team> <http://www.w3.org/ns/solid/acp#allow> <http://www.w3.org/ns/auth/acl#Read> .\n\
         <{acr}#pol-team> <http://www.w3.org/ns/solid/acp#anyOf> <{acr}#m-dave> .\n\
         <{acr}#m-dave> <http://www.w3.org/ns/solid/acp#agent> <https://dave.ex/card#me> .\n"
    );
    s.graph.named[pos].1 = Graph::load_str(&team, "ntriples").unwrap();
    s.materialize_acp().expect("re-materialize");
    assert!(can(&mut s, Some(DAVE), None, Mode::Read, "https://pod.ex/team2/c3/g0/d0.ttl"));
    // …and bob lost the write grant the old policy carried
    assert!(!can(&mut s, Some(BOB), None, Mode::Write, "https://pod.ex/team2/c3/g0/d0.ttl"));
}

/// [OPUS-4.8] sq-xor3: ACP write-path enforcement. `update_as_acp` gates a SPARQL Update
/// against `acl:Write` (cumulative ACP allow ∖ deny) before mutating. Expected from the
/// ACP fixture: ALICE has Read/Write/Control everywhere (cumulative root owner policy);
/// team2's `acp:allow Read,Write` anyOf {bob,carol} gives BOB+CAROL write; DAVE has none.
#[test]
fn acp_write_enforcement_matches_grants() {
    let mut s = store();
    let ins = |g: &str| format!("INSERT DATA {{ GRAPH <{g}> {{ <{g}#it> <https://ex.dev/ns#k> \"v\" }} }}");

    let priv0 = "https://pod.ex/priv0/c4/g0/d0.ttl";
    assert!(can(&mut s, Some(ALICE), None, Mode::Write, priv0));
    s.update_as_acp(&Session { agent: Some(ALICE), client: None }, &ins(priv0))
        .expect("alice (cumulative owner) writes priv0");
    assert!(!can(&mut s, Some(BOB), None, Mode::Write, priv0));
    assert!(
        s.update_as_acp(&Session { agent: Some(BOB), client: None }, &ins(priv0)).is_err(),
        "bob has no write on priv0"
    );

    let team2 = "https://pod.ex/team2/c3/g0/d0.ttl";
    assert!(can(&mut s, Some(BOB), None, Mode::Write, team2));
    s.update_as_acp(&Session { agent: Some(BOB), client: None }, &ins(team2))
        .expect("bob (anyOf) writes team2");
    assert!(!can(&mut s, Some(DAVE), None, Mode::Write, team2));
    assert!(
        s.update_as_acp(&Session { agent: Some(DAVE), client: None }, &ins(team2)).is_err(),
        "dave denied write on team2"
    );
}
