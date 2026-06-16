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
    s.accessible(&Session { agent, client, issuer: None }, mode).iter().any(|g| g.as_str() == graph)
}

/// [OPUS-4.8] sq-3jtd.6: as [`can`] but with the issuer dimension supplied.
fn can_iss(
    s: &mut PodStore,
    agent: Option<&str>,
    client: Option<&str>,
    issuer: Option<&str>,
    mode: Mode,
    graph: &str,
) -> bool {
    s.accessible(&Session { agent, client, issuer }, mode).iter().any(|g| g.as_str() == graph)
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
    s.update_as_acp(&Session { agent: Some(ALICE), client: None, issuer: None }, &ins(priv0))
        .expect("alice (cumulative owner) writes priv0");
    assert!(!can(&mut s, Some(BOB), None, Mode::Write, priv0));
    assert!(
        s.update_as_acp(&Session { agent: Some(BOB), client: None, issuer: None }, &ins(priv0)).is_err(),
        "bob has no write on priv0"
    );

    let team2 = "https://pod.ex/team2/c3/g0/d0.ttl";
    assert!(can(&mut s, Some(BOB), None, Mode::Write, team2));
    s.update_as_acp(&Session { agent: Some(BOB), client: None, issuer: None }, &ins(team2))
        .expect("bob (anyOf) writes team2");
    assert!(!can(&mut s, Some(DAVE), None, Mode::Write, team2));
    assert!(
        s.update_as_acp(&Session { agent: Some(DAVE), client: None, issuer: None }, &ins(team2)).is_err(),
        "dave denied write on team2"
    );
}

// ── [OPUS-4.8] sq-3jtd.6: acp:issuer support — the (agent, client, issuer) principal ──
//
// The issuer dimension is the exact twin of the client dimension (design doc §3.6): an
// ACP Matcher can constrain on the OIDC issuer that vouched for the requester's WebID.
// These tests build small inline ACP pods (one .acr graph) so the issuer matrix is
// hand-verifiable, and exercise BOTH outcomes — allow when the issuer matches, deny when
// it does not (or is absent), through the unchanged `materialize_acp` + `accessible` path.

const GOOD_IDP: &str = "https://good-idp.ex";
const EVIL_IDP: &str = "https://evil-idp.ex";
const ISS_DOC: &str = "https://pod.ex/iss/d0.ttl";
const ISS_ACR: &str = "https://pod.ex/iss/.acr";

/// One inline ACP pod: a single document `iss/d0.ttl` whose containing `iss/` ACR
/// (`iss/.acr`) carries `policy_body` (raw N-Triple body lines, sharing the `#pol` policy
/// node) under `acp:memberAccessControl`, so the policy applies cumulatively to the
/// member document — the same inheritance shape as the shared fixture.
fn iss_store(policy_body: &str) -> PodStore {
    let acr = format!(
        "<{ISS_DOC}#it> <https://ex.dev/ns#title> \"iss\" <{ISS_DOC}> .\n\
         <{ISS_ACR}> <http://www.w3.org/ns/solid/acp#memberAccessControl> <{ISS_ACR}#ctl> <{ISS_ACR}> .\n\
         <{ISS_ACR}#ctl> <http://www.w3.org/ns/solid/acp#apply> <{ISS_ACR}#pol> <{ISS_ACR}> .\n\
         {policy_body}"
    );
    let g = Graph::load_dataset(&acr, "nquads").expect("inline acp pod loads");
    let mut s = PodStore::new(g);
    s.materialize_acp().expect("acp materializes");
    s
}

/// allOf { acp:agent BOB ; acp:issuer GOOD_IDP }: BOB is granted Read ONLY when the
/// session's issuer is GOOD_IDP. Allow AND deny are both decided by the issuer.
#[test]
fn acp_issuer_allof_allow_and_deny() {
    let acl = "http://www.w3.org/ns/auth/acl#";
    let acp = "http://www.w3.org/ns/solid/acp#";
    let body = format!(
        "<{ISS_ACR}#pol> <{acp}allow> <{acl}Read> <{ISS_ACR}> .\n\
         <{ISS_ACR}#pol> <{acp}allOf> <{ISS_ACR}#m> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}agent> <{BOB}> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}issuer> <{GOOD_IDP}> <{ISS_ACR}> .\n"
    );
    let mut s = iss_store(&body);
    let r = Mode::Read;

    // ALLOW: bob through the trusted issuer.
    assert!(can_iss(&mut s, Some(BOB), None, Some(GOOD_IDP), r, ISS_DOC), "bob @ good-idp allowed");
    // DENY decided by issuer: same agent, wrong issuer.
    assert!(!can_iss(&mut s, Some(BOB), None, Some(EVIL_IDP), r, ISS_DOC), "bob @ evil-idp denied");
    // DENY fail-closed: issuer-constrained grant never matches a session with no issuer.
    assert!(!can_iss(&mut s, Some(BOB), None, None, r, ISS_DOC), "bob w/o issuer denied");
    // DENY: right issuer, wrong agent (allOf needs both).
    assert!(!can_iss(&mut s, Some(CAROL), None, Some(GOOD_IDP), r, ISS_DOC), "carol @ good-idp denied");
}

/// anyOf { acp:issuer GOOD_IDP } (issuer-only, no agent attr): ANY agent vouched for by
/// GOOD_IDP is granted, regardless of WebID; nobody from another issuer is.
#[test]
fn acp_issuer_only_matcher_any_agent() {
    let acl = "http://www.w3.org/ns/auth/acl#";
    let acp = "http://www.w3.org/ns/solid/acp#";
    let body = format!(
        "<{ISS_ACR}#pol> <{acp}allow> <{acl}Read> <{ISS_ACR}> .\n\
         <{ISS_ACR}#pol> <{acp}anyOf> <{ISS_ACR}#m> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}issuer> <{GOOD_IDP}> <{ISS_ACR}> .\n"
    );
    let mut s = iss_store(&body);
    let r = Mode::Read;

    // ALLOW: any agent from the trusted issuer (the agent dimension is unconstrained).
    assert!(can_iss(&mut s, Some(BOB), None, Some(GOOD_IDP), r, ISS_DOC), "bob @ good-idp allowed");
    assert!(can_iss(&mut s, Some(CAROL), None, Some(GOOD_IDP), r, ISS_DOC), "carol @ good-idp allowed");
    // DENY decided by issuer: wrong issuer, or no issuer asserted at all.
    assert!(!can_iss(&mut s, Some(BOB), None, Some(EVIL_IDP), r, ISS_DOC), "bob @ evil-idp denied");
    assert!(!can_iss(&mut s, Some(CAROL), None, None, r, ISS_DOC), "carol w/o issuer denied");
    assert!(!can_iss(&mut s, None, None, None, r, ISS_DOC), "anonymous w/o issuer denied");
    // The matcher is agent-unconstrained, so it grants the agent-dimension top
    // (auth:Public): a session presenting only the trusted issuer is accepted even without
    // a WebID — faithful, since the matcher asserts nothing about the agent.
    assert!(can_iss(&mut s, None, None, Some(GOOD_IDP), r, ISS_DOC), "any context @ good-idp allowed");
}

/// allOf { acp:agent BOB ; acp:client APP ; acp:issuer GOOD_IDP }: the full three-
/// dimension principal — exactly the (user, app, issuer) triple, nothing wider.
#[test]
fn acp_issuer_client_triple() {
    let acl = "http://www.w3.org/ns/auth/acl#";
    let acp = "http://www.w3.org/ns/solid/acp#";
    let body = format!(
        "<{ISS_ACR}#pol> <{acp}allow> <{acl}Read> <{ISS_ACR}> .\n\
         <{ISS_ACR}#pol> <{acp}allOf> <{ISS_ACR}#m> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}agent> <{BOB}> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}client> <{APP}> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}issuer> <{GOOD_IDP}> <{ISS_ACR}> .\n"
    );
    let mut s = iss_store(&body);
    let r = Mode::Read;

    // ALLOW: the exact triple.
    assert!(can_iss(&mut s, Some(BOB), Some(APP), Some(GOOD_IDP), r, ISS_DOC), "bob+app@good-idp allowed");
    // DENY decided by issuer alone (agent + client correct).
    assert!(!can_iss(&mut s, Some(BOB), Some(APP), Some(EVIL_IDP), r, ISS_DOC), "wrong issuer denied");
    // DENY: missing the issuer dimension entirely.
    assert!(!can_iss(&mut s, Some(BOB), Some(APP), None, r, ISS_DOC), "no issuer denied");
    // DENY: correct agent + issuer, wrong client.
    assert!(
        !can_iss(&mut s, Some(BOB), Some("https://evil.ex"), Some(GOOD_IDP), r, ISS_DOC),
        "wrong client denied"
    );
}

/// noneOf { acp:issuer EVIL_IDP } on an otherwise-public policy: a CONDITIONAL grant whose
/// exception is evaluated per session on the ISSUER dimension. Everyone reads EXCEPT a
/// session asserting the excluded issuer — the deny is decided by the issuer.
#[test]
fn acp_issuer_noneof_conditional_grant() {
    let acl = "http://www.w3.org/ns/auth/acl#";
    let acp = "http://www.w3.org/ns/solid/acp#";
    let public_agent = format!("{acp}PublicAgent");
    let body = format!(
        "<{ISS_ACR}#pol> <{acp}allow> <{acl}Read> <{ISS_ACR}> .\n\
         <{ISS_ACR}#pol> <{acp}anyOf> <{ISS_ACR}#mpub> <{ISS_ACR}> .\n\
         <{ISS_ACR}#mpub> <{acp}agent> <{public_agent}> <{ISS_ACR}> .\n\
         <{ISS_ACR}#pol> <{acp}noneOf> <{ISS_ACR}#mno> <{ISS_ACR}> .\n\
         <{ISS_ACR}#mno> <{acp}issuer> <{EVIL_IDP}> <{ISS_ACR}> .\n"
    );
    let mut s = iss_store(&body);
    let r = Mode::Read;

    // ALLOW: public — anonymous, and any issuer that is NOT the excluded one.
    assert!(can_iss(&mut s, None, None, None, r, ISS_DOC), "anonymous reads");
    assert!(can_iss(&mut s, Some(BOB), None, Some(GOOD_IDP), r, ISS_DOC), "bob @ good-idp reads");
    assert!(can_iss(&mut s, Some(CAROL), None, None, r, ISS_DOC), "carol w/o issuer reads");
    // DENY decided by the issuer exception: a session asserting the excluded issuer is
    // carved out (the noneOf exception matcher accepts it, suppressing the grant).
    assert!(!can_iss(&mut s, Some(BOB), None, Some(EVIL_IDP), r, ISS_DOC), "bob @ evil-idp carved out");
}

/// Fail-closed: a malicious `acp:issuer` value inside the reserved `urn:sparq:` space (or
/// carrying the pair delimiter) is rejected at materialization, exactly like a malicious
/// agent/client value — it cannot smuggle a forged triple principal into the auth view.
#[test]
fn acp_issuer_reserved_encoding_rejected() {
    let acl = "http://www.w3.org/ns/auth/acl#";
    let acp = "http://www.w3.org/ns/solid/acp#";
    let evil = "urn:sparq:triple?agent=x&client=y&issuer=z";
    let body = format!(
        "<{ISS_ACR}#pol> <{acp}allow> <{acl}Read> <{ISS_ACR}> .\n\
         <{ISS_ACR}#pol> <{acp}allOf> <{ISS_ACR}#m> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}agent> <{BOB}> <{ISS_ACR}> .\n\
         <{ISS_ACR}#m> <{acp}issuer> <{evil}> <{ISS_ACR}> .\n"
    );
    let acr = format!(
        "<{ISS_DOC}#it> <https://ex.dev/ns#title> \"iss\" <{ISS_DOC}> .\n\
         <{ISS_ACR}> <{acp}accessControl> <{ISS_ACR}#ctl> <{ISS_ACR}> .\n\
         <{ISS_ACR}#ctl> <{acp}apply> <{ISS_ACR}#pol> <{ISS_ACR}> .\n\
         {body}"
    );
    let g = Graph::load_dataset(&acr, "nquads").expect("loads");
    let mut s = PodStore::new(g);
    assert!(s.materialize_acp().is_err(), "reserved-space issuer IRI rejected at materialization");
}
