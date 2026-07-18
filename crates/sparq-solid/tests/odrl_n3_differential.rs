//! [SONNET-4.6] sq-zgbso.2 — Rust-vs-N3 decision differential for the stateless ODRL core.
//!
//! Asserts that the three-stratum N3 rule set (`rules/odrl-{a,b,c}.n3`) produces
//! EXACTLY the same `auth:*` triple set as `materialize_policy` for every case in the
//! corpus. Both sides must also equal an independently-stated expected set so that a
//! both-wrong agreement cannot pass.
//!
//! Supported scope (bead sq-zgbso.2):
//! - Unconstrained permissions and prohibitions
//! - `odrl:dateTime` lteq / gteq constraints
//! - `odrl:recipient` eq / neq constraints
//! - `odrl:or` logical constraint combinator
//! - `odrl:and` logical constraint combinator
//!
//! Gated on the default-OFF `odrl-bridge` feature (requires sparq-policy).
#![cfg(feature = "odrl-bridge")]

use sparq_core::Graph;
use sparq_policy::{parse_policy_str, Request};
use sparq_solid::odrl_bridge::{materialize_odrl_n3, materialize_policy};
use sparq_solid::AUTH_NS;

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
const TARGET: &str = "urn:t/1";
type AuthSet = Vec<(String, String, String)>;

fn rust_set(policy_ttl: &str, action_local: &str, party: &str, at: Option<&str>) -> AuthSet {
    let policy = parse_policy_str(policy_ttl, "turtle").expect("policy parses");
    let mut req = Request::new(format!("{}{}", ODRL, action_local)).on(TARGET).by(party);
    if let Some(t) = at {
        req = req.at(t);
    }
    let mut graph = Graph::new();
    let outcome = materialize_policy(&mut graph, &policy, &req);
    let mut set = AuthSet::new();
    if let Some(t) = outcome.grant_triple {
        set.push(t);
    }
    if let Some(t) = outcome.deny_triple {
        set.push(t);
    }
    set.sort();
    set
}

fn n3_set(policy_ttl: &str, action_local: &str, party: &str, at: Option<&str>) -> AuthSet {
    let mut req = Request::new(format!("{}{}", ODRL, action_local)).on(TARGET).by(party);
    if let Some(t) = at {
        req = req.at(t);
    }
    let mut graph = Graph::new();
    let outcome = materialize_odrl_n3(&mut graph, policy_ttl, &req).expect("N3 reasoning succeeds");
    let mut set = AuthSet::new();
    if let Some(t) = outcome.grant_triple {
        set.push(t);
    }
    if let Some(t) = outcome.deny_triple {
        set.push(t);
    }
    set.sort();
    set
}

fn expect(pairs: &[(&str, &str, &str)]) -> AuthSet {
    let mut s: AuthSet = pairs
        .iter()
        .map(|(a, m, t)| ((*a).to_owned(), format!("{}{}", AUTH_NS, m), (*t).to_owned()))
        .collect();
    s.sort();
    s
}

fn assert_case(label: &str, policy: &str, action: &str, party: &str, at: Option<&str>, want: &[(&str, &str, &str)]) {
    let rust = rust_set(policy, action, party, at);
    let n3 = n3_set(policy, action, party, at);
    let expected = expect(want);
    assert_eq!(rust, expected, "{}: Rust path != expected", label);
    assert_eq!(n3, expected, "{}: N3 path != expected", label);
    assert_eq!(rust, n3, "{}: Rust and N3 diverge", label);
}

// ── Policies ─────────────────────────────────────────────────────────────────

const POL_B1: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/b1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#;

const POL_A1: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/a1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ] ] ."#;

const POL_A3: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/a3> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gteq ;
                      odrl:rightOperand "2026-01-01T00:00:00Z"^^xsd:dateTime ] ] ."#;

const POL_A4: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/a4> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
                      odrl:rightOperand <urn:alice> ] ] ."#;

const POL_A6: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/a6> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ; odrl:operator odrl:neq ;
                      odrl:rightOperand <urn:bob> ] ] ."#;

const POL_C1: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/c1> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#;

const POL_C2: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/c2> a odrl:Set ;
    odrl:permission  [ odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ;
    odrl:prohibition [ odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#;

// odrl:or LC: c1 = dateTime lteq 2026-12-31, c2 = recipient eq alice
const POL_D1: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/d1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint _:lc ] .
_:lc a odrl:LogicalConstraint ; odrl:or _:c1 , _:c2 .
_:c1 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
     odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime .
_:c2 odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
     odrl:rightOperand <urn:alice> ."#;

// odrl:or LC: c1 = dateTime lteq 2025-01-01 (past), c2 = recipient eq bob (wrong party)
const POL_D3: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/d3> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint _:lc ] .
_:lc a odrl:LogicalConstraint ; odrl:or _:c1 , _:c2 .
_:c1 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
     odrl:rightOperand "2025-01-01T00:00:00Z"^^xsd:dateTime .
_:c2 odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
     odrl:rightOperand <urn:bob> ."#;

// odrl:and LC: c1 = dateTime gteq 2026-01-01, c2 = dateTime lteq 2026-12-31
const POL_D4: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/d4> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint _:lc ] .
_:lc a odrl:LogicalConstraint ; odrl:and _:c1 , _:c2 .
_:c1 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gteq ;
     odrl:rightOperand "2026-01-01T00:00:00Z"^^xsd:dateTime .
_:c2 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
     odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ."#;

// odrl:and LC: c1 = dateTime gteq 2026-01-01 (satisfied), c2 = recipient eq bob (wrong party)
const POL_D5: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/d5> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint _:lc ] .
_:lc a odrl:LogicalConstraint ; odrl:and _:c1 , _:c2 .
_:c1 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gteq ;
     odrl:rightOperand "2026-01-01T00:00:00Z"^^xsd:dateTime .
_:c2 odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
     odrl:rightOperand <urn:bob> ."#;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn b1_unconstrained_permission_grant() {
    assert_case("B1", POL_B1, "read", "urn:alice", None, &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn b2_unconstrained_permission_wrong_assignee_no_grant() {
    assert_case("B2", POL_B1, "read", "urn:bob", None, &[]);
}

#[test]
fn a1_datetime_lteq_within_window_grant() {
    assert_case("A1", POL_A1, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn a2_datetime_lteq_past_deadline_no_grant() {
    assert_case("A2", POL_A1, "read", "urn:alice", Some("2027-01-01T00:00:00Z"), &[]);
}

#[test]
fn a3_datetime_gteq_above_floor_grant() {
    assert_case("A3", POL_A3, "read", "urn:alice", Some("2026-06-01T00:00:00Z"), &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn a4_recipient_eq_match_grant() {
    assert_case("A4", POL_A4, "read", "urn:alice", None, &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn a5_recipient_eq_mismatch_no_grant() {
    assert_case("A5", POL_A4, "read", "urn:bob", None, &[]);
}

#[test]
fn a6_recipient_neq_not_excluded_grant() {
    // urn:alice is NOT the excluded party (urn:bob is) → grant
    assert_case("A6", POL_A6, "read", "urn:alice", None, &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn a7_recipient_neq_is_excluded_no_grant() {
    // urn:bob IS the excluded party → no grant (permission assignee also doesn't match)
    assert_case("A7", POL_A6, "read", "urn:bob", None, &[]);
}

#[test]
fn c1_unconstrained_prohibition_deny() {
    assert_case("C1", POL_C1, "read", "urn:alice", None, &[("urn:alice", "denyRead", "urn:t/1")]);
}

#[test]
fn c2_deny_overrides_permission() {
    // prohibition + permission both match: deny wins (deny-overrides)
    assert_case("C2", POL_C2, "read", "urn:alice", None, &[("urn:alice", "denyRead", "urn:t/1")]);
}

#[test]
fn d1_or_lc_first_sub_satisfied_grant() {
    // c1 (lteq 2026-12-31) satisfied at 2026-07-18 → or satisfied → grant
    assert_case("D1", POL_D1, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn d2_or_lc_second_sub_satisfied_grant() {
    // c1 not satisfied (past deadline at 2027-01-01), c2 (recipient eq alice) satisfied → or satisfied → grant
    assert_case("D2", POL_D1, "read", "urn:alice", Some("2027-01-01T00:00:00Z"), &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn d3_or_lc_none_satisfied_no_grant() {
    // c1 (lteq 2025-01-01) past, c2 (recipient eq bob) wrong party → or not satisfied → no grant
    assert_case("D3", POL_D3, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[]);
}

#[test]
fn d4_and_lc_both_satisfied_grant() {
    // c1 (gteq 2026-01-01) satisfied, c2 (lteq 2026-12-31) satisfied at 2026-07-18 → and satisfied → grant
    assert_case("D4", POL_D4, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn d5_and_lc_one_unsatisfied_no_grant() {
    // c1 (gteq 2026-01-01) satisfied, c2 (recipient eq bob) not satisfied for alice → and not satisfied → no grant
    assert_case("D5", POL_D5, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[]);
}
