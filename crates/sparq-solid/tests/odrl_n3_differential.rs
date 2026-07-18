//! [SONNET-4.6] sq-zgbso.2 — Rust-vs-N3 decision differential for the stateless ODRL core.
//!
//! Asserts that the four-stratum N3 rule set (`rules/odrl-{a,b,c,d}.n3`) produces
//! EXACTLY the same `auth:*` triple set as `materialize_policy` for every case in the
//! corpus. Both sides must also equal an independently-stated expected set so that a
//! both-wrong agreement cannot pass.
//!
//! Supported scope (bead sq-zgbso.2):
//! - Unconstrained permissions and prohibitions
//! - `odrl:dateTime` lteq / gteq constraints (canonical UTC lexical forms only —
//!   every other lexical form is REJECTED fail-closed, see the `dt*` tests)
//! - `odrl:recipient` eq / neq constraints
//! - `odrl:or` logical constraint combinator
//! - `odrl:and` logical constraint combinator
//! - Constrained prohibitions over the same atomic/or/and scope (deny-overrides;
//!   out-of-scope prohibition constructs are REFUSED, never silently ignored)
//!
//! The `inj*` tests are adversarial: policy/request content carrying N3 implication
//! rules, source-terminating quotes/newlines, and `>`-bearing IRIs must either be
//! rejected outright or end up inert (escaped) — never derive `auth:*` triples.
//! Beyond source syntax, STRICT-Turtle policies asserting engine-owned GROUND FACTS
//! (a direct `auth:*` grant, internal `odrlx:` derivations, facts about the reserved
//! `<urn:odrl-req>` subject, a caller-minted `a odrl:Request` node) must be `Err` —
//! parsing does not make untrusted RDF facts trusted (inj5–inj8).
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

// ── Constrained prohibitions (deny window ends 2026-12-31) ───────────────────

const POL_E1: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/e1> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ] ] ."#;

// matching permission + the same constrained prohibition: deny-overrides inside the window
const POL_E2: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/e2> a odrl:Set ;
    odrl:permission  [ odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ;
    odrl:prohibition [ odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
        odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                          odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ] ] ."#;

// prohibition with an odrl:and LC: deny only inside [2026-01-01, 2026-12-31]
const POL_E3: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/e3> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint _:lc ] .
_:lc a odrl:LogicalConstraint ; odrl:and _:c1 , _:c2 .
_:c1 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gteq ;
     odrl:rightOperand "2026-01-01T00:00:00Z"^^xsd:dateTime .
_:c2 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
     odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ."#;

// prohibition with an odrl:or LC: c1 = dateTime lteq 2025-01-01 (past), c2 = recipient eq alice
const POL_E4: &str = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/e4> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint _:lc ] .
_:lc a odrl:LogicalConstraint ; odrl:or _:c1 , _:c2 .
_:c1 odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
     odrl:rightOperand "2025-01-01T00:00:00Z"^^xsd:dateTime .
_:c2 odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
     odrl:rightOperand <urn:bob> ."#;

#[test]
fn e1_constrained_prohibition_satisfied_deny() {
    // inside the window → the constrained prohibition matches → deny
    assert_case("E1", POL_E1, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[("urn:alice", "denyRead", "urn:t/1")]);
}

#[test]
fn e2_constrained_prohibition_lapsed_no_deny() {
    // past the window → the constraint is unsatisfied → no deny, nothing else matches
    assert_case("E2", POL_E1, "read", "urn:alice", Some("2027-06-01T00:00:00Z"), &[]);
}

#[test]
fn e3_constrained_prohibition_overrides_matching_permission() {
    // inside the window BOTH the permission and the constrained prohibition match:
    // deny-overrides — the deny is emitted and the grant is suppressed
    assert_case("E3", POL_E2, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[("urn:alice", "denyRead", "urn:t/1")]);
}

#[test]
fn e4_lapsed_prohibition_re_exposes_permission() {
    // past the window the prohibition no longer carves the request out → grant only
    assert_case("E4", POL_E2, "read", "urn:alice", Some("2027-06-01T00:00:00Z"), &[("urn:alice", "read", "urn:t/1")]);
}

#[test]
fn e5_and_lc_prohibition_inside_window_deny() {
    assert_case("E5", POL_E3, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[("urn:alice", "denyRead", "urn:t/1")]);
}

#[test]
fn e6_and_lc_prohibition_outside_window_no_deny() {
    // before the gteq floor → the odrl:and LC is unsatisfied → no deny
    assert_case("E6", POL_E3, "read", "urn:alice", Some("2025-06-01T00:00:00Z"), &[]);
}

#[test]
fn e7_or_lc_prohibition_sub_satisfied_deny() {
    // bob: the recipient-eq-bob sub is satisfied... but the prohibition assignee is
    // alice, so bob is not carved out; alice at a PAST lteq bound is not either.
    assert_case("E7a", POL_E4, "read", "urn:alice", Some("2026-07-18T00:00:00Z"), &[]);
    // alice INSIDE the lteq bound: the or-LC is satisfied via c1 → deny
    assert_case("E7b", POL_E4, "read", "urn:alice", Some("2024-06-01T00:00:00Z"), &[("urn:alice", "denyRead", "urn:t/1")]);
}

#[test]
fn e9_use_umbrella_prohibition_refused() {
    // the Rust evaluator's use-umbrella denies any non-transfer action; the N3
    // strata match actions exactly, so a use-prohibition must REFUSE, not vanish
    let pol = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/e9> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:use ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, pol, &req).is_err(), "E9: use-umbrella prohibition must refuse");
    assert!(graph.named.is_empty(), "E9: nothing may be materialized");
}

#[test]
fn e10_duty_carrying_permission_refused() {
    // the N3 strata do not check duty discharge; the Rust evaluator denies an
    // undischarged duty — granting regardless would widen, so the N3 path refuses
    let pol = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/e10> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:duty [ odrl:action odrl:attribute ] ] ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, pol, &req).is_err(), "E10: duty-carrying policy must refuse");
    assert!(graph.named.is_empty(), "E10: nothing may be materialized");
}

#[test]
fn e8_out_of_scope_prohibition_constraint_refused() {
    // an odrl:purpose prohibition constraint is OUTSIDE the N3 stratum scope; the
    // Rust evaluator could satisfy it, so the N3 path must REFUSE, not ignore it
    let pol = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/e8> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
                      odrl:rightOperand <urn:marketing> ] ] ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, pol, &req).is_err(), "E8: out-of-scope prohibition constraint must refuse");
    assert!(graph.named.is_empty(), "E8: nothing may be materialized");
}

// ── Canonical dateTime subset (lexical comparison soundness) ─────────────────

fn assert_n3_rejects(label: &str, policy: &str, at: Option<&str>) {
    let mut req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    if let Some(t) = at {
        req = req.at(t);
    }
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, policy, &req).is_err(), "{}: must be rejected fail-closed", label);
    assert!(graph.named.is_empty(), "{}: nothing may be materialized", label);
}

#[test]
fn dt1_request_time_with_offset_rejected() {
    // an equivalent instant in a different lexical form (offset) defeats the
    // strata's lexicographic comparison → rejected before reasoning
    assert_n3_rejects("DT1", POL_A1, Some("2026-07-18T02:00:00+02:00"));
}

#[test]
fn dt2_request_time_fractional_seconds_rejected() {
    assert_n3_rejects("DT2", POL_A1, Some("2026-07-18T00:00:00.500Z"));
}

#[test]
fn dt3_request_time_hour_24_rejected() {
    // XSD maps 24:00:00 onto the NEXT day's midnight — lexicographically misordered
    assert_n3_rejects("DT3", POL_A1, Some("2026-07-18T24:00:00Z"));
}

#[test]
fn dt4_policy_bound_with_offset_rejected() {
    let pol = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/dt4> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T01:00:00+01:00"^^xsd:dateTime ] ] ."#;
    assert_n3_rejects("DT4", pol, Some("2026-07-18T00:00:00Z"));
}

// ── Adversarial injection (blocker finding: executable N3 in caller input) ───

#[test]
fn inj1_n3_implication_rule_in_policy_rejected() {
    // an N3 rule deriving an auth:* grant directly is NOT Turtle → strict parse Err
    let evil = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
{ ?s ?p ?o . } => { <urn:mallory> <https://sparq.dev/ns/auth#read> <urn:t/1> . } ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:mallory");
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, evil, &req).is_err(), "INJ1: N3 rule must be rejected");
    assert!(graph.named.is_empty(), "INJ1: nothing may be materialized");
}

#[test]
fn inj2_hostile_literal_is_inert_after_reserialization() {
    // Turtle-VALID policy whose literal smuggles quotes, newlines, a fake auth
    // triple, and an N3 rule. After strict parse + escaped re-serialization the
    // payload stays inside the literal — mallory derives nothing.
    let pol = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/inj> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] .
<urn:pol/inj> <urn:note> "\" .\n<urn:mallory> <https://sparq.dev/ns/auth#read> <urn:t/1> .\n{ ?a ?b ?c . } => { ?a <https://sparq.dev/ns/auth#read> <urn:t/1> . } . \\" ."#;
    // mallory holds no permission: were the payload executable, the injected rule /
    // triple would grant it read — assert nothing materializes
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:mallory");
    let mut graph = Graph::new();
    let out = materialize_odrl_n3(&mut graph, pol, &req).expect("INJ2: hostile literal still parses");
    assert!(!out.granted && !out.prohibited, "INJ2: payload must be inert");
    assert!(graph.named.is_empty(), "INJ2: nothing may be materialized");
    // the legitimate permission in the same policy still works
    let req_alice = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph_alice = Graph::new();
    let out_alice = materialize_odrl_n3(&mut graph_alice, pol, &req_alice).expect("INJ2: alice path parses");
    assert_eq!(
        out_alice.grant_triple,
        Some(("urn:alice".to_owned(), format!("{}read", AUTH_NS), "urn:t/1".to_owned())),
        "INJ2: only alice's legitimate grant materializes"
    );
    assert!(out_alice.deny_triple.is_none(), "INJ2: no injected deny");
}

#[test]
fn inj3_malicious_request_party_rejected() {
    // a party smuggling IRI-terminating '>' plus its own triple must be an Err
    // (NamedNode validation), never an interpolated fact
    let req = Request::new(format!("{}read", ODRL))
        .on(TARGET)
        .by("urn:eve> <https://sparq.dev/ns/auth#read> <urn:t/1> . <urn:x");
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, POL_B1, &req).is_err(), "INJ3: malformed party IRI must be rejected");
    assert!(graph.named.is_empty(), "INJ3: nothing may be materialized");
}

#[test]
fn inj4_malicious_request_action_rejected() {
    let req = Request::new("http://www.w3.org/ns/odrl/2/read> <urn:p> <urn:o")
        .on(TARGET)
        .by("urn:alice");
    let mut graph = Graph::new();
    assert!(materialize_odrl_n3(&mut graph, POL_B1, &req).is_err(), "INJ4: malformed action IRI must be rejected");
    assert!(graph.named.is_empty(), "INJ4: nothing may be materialized");
}

// ── inj5–inj8: STRICT-Turtle assertion of engine-owned facts. Parsing/escaping
// stops source-syntax injection; these guard the other half of the trust
// boundary — a syntactically valid policy asserting engine-owned GROUND FACTS
// the strata would forward into the closure as if the engine derived them.
// Every case must Err and materialize nothing (unchanged graph).

#[test]
fn inj5_direct_auth_triple_in_valid_turtle_rejected() {
    // mallory holds NO permission, but the policy directly asserts the auth-view
    // grant triple; forwarded uncheck it would be extracted from the final
    // closure as a trusted grant
    let evil = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/inj5> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] .
<urn:mallory> <https://sparq.dev/ns/auth#read> <urn:t/1> ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:mallory");
    let mut graph = Graph::new();
    let out = materialize_odrl_n3(&mut graph, evil, &req);
    assert!(out.is_err(), "INJ5: asserted auth:* fact must be rejected");
    assert!(graph.named.is_empty(), "INJ5: nothing may be materialized");
}

#[test]
fn inj6_internal_odrlx_derivation_facts_rejected() {
    // faking the internal derivation vocabulary: odrlx:atomicSat would mark an
    // UNSATISFIED constraint satisfied (mallory is not the eq-recipient), and
    // odrlx:mode would mint a new action→mode mapping — both widen
    let fake_sat = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix odrlx: <https://sparq.dev/ns/odrlx#> .
<urn:pol/inj6> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:mallory> ;
    odrl:constraint _:c ] .
_:c odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
    odrl:rightOperand <urn:alice> .
_:c odrlx:atomicSat <urn:odrl-req> ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:mallory");
    let mut graph = Graph::new();
    assert!(
        materialize_odrl_n3(&mut graph, fake_sat, &req).is_err(),
        "INJ6: asserted odrlx:atomicSat must be rejected"
    );
    assert!(graph.named.is_empty(), "INJ6: nothing may be materialized");

    // asserting odrlx:prohibMatches (the deny-side internal fact) is just as
    // reserved — no caller fact may live in the odrlx: namespace at all
    let fake_prohib = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix odrlx: <https://sparq.dev/ns/odrlx#> .
<urn:pol/inj6b> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] .
<urn:pol/inj6b> odrlx:prohibMatches <urn:odrl-req> ."#;
    let req_alice = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph_b = Graph::new();
    assert!(
        materialize_odrl_n3(&mut graph_b, fake_prohib, &req_alice).is_err(),
        "INJ6: asserted odrlx:prohibMatches must be rejected"
    );
    assert!(graph_b.named.is_empty(), "INJ6: nothing may be materialized");
}

#[test]
fn inj7_reserved_request_subject_facts_rejected() {
    // facts about <urn:odrl-req> would merge with the engine-serialized request
    // node and steer the rules (here: retargeting the request's assignee)
    let evil = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/inj7> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:mallory> ] .
<urn:odrl-req> odrl:assignee <urn:mallory> ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph = Graph::new();
    assert!(
        materialize_odrl_n3(&mut graph, evil, &req).is_err(),
        "INJ7: facts about the reserved request IRI must be rejected"
    );
    assert!(graph.named.is_empty(), "INJ7: nothing may be materialized");
}

#[test]
fn inj8_caller_minted_request_typed_subject_rejected() {
    // the grant rules bind ?req to ANY `a odrl:Request` node, so a caller-minted
    // request-shaped subject would be matched exactly like the real request
    let evil = r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/inj8> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:mallory> ] .
<urn:fake-req> a odrl:Request ; odrl:action odrl:read ;
    odrl:target <urn:t/1> ; odrl:assignee <urn:mallory> ."#;
    let req = Request::new(format!("{}read", ODRL)).on(TARGET).by("urn:alice");
    let mut graph = Graph::new();
    assert!(
        materialize_odrl_n3(&mut graph, evil, &req).is_err(),
        "INJ8: a caller-minted odrl:Request-typed subject must be rejected"
    );
    assert!(graph.named.is_empty(), "INJ8: nothing may be materialized");
}
