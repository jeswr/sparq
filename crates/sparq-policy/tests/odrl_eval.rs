//! End-to-end ODRL parse + evaluate tests. [OPUS-4.8] sq-r06h.
//!
//! Coverage: permission grant, prohibition-overrides-permission, constraint
//! gating (inside/outside time window; purpose eq/neq; recipient isPartOf;
//! count gt/lteq), duty-not-discharged, fail-closed defaults (empty policy,
//! unknown action/target/party, malformed constraint, unknown operator).
//!
//! Policies are hand-authored in Turtle following the W3C ODRL 2.2 examples
//! (the public test-suite examples are equivalent shapes: a `odrl:Set` with a
//! blank-node permission/prohibition carrying action/target/assignee and
//! blank-node constraints).

use sparq_policy::{
    evaluate, parse_policy_str, prohibition_status, Decision, ProhibitionStatus, Request, Value,
};

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
const XSD_DT: &str = "http://www.w3.org/2001/XMLSchema#dateTime";

fn dt(s: &str) -> Value {
    Value::DateTime(s.to_owned())
}
fn left(local: &str) -> String {
    format!("{ODRL}{local}")
}

// ---------------------------------------------------------------------------
// 1. Permission grant — a bare permission with a matching action/target/party.
// ---------------------------------------------------------------------------
#[test]
fn permission_grants() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/grant> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    assert_eq!(p.permissions.len(), 1);
    let req = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    let d = evaluate(&p, &req);
    assert!(d.allow, "{d:?}");
    assert_eq!(d.matched_rules.len(), 1);
}

#[test]
fn wrong_party_denied_fail_closed() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/grant> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // Mallory is not the assignee → no permission matches → DENY.
    let req = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://mallory.ex/me");
    assert!(!evaluate(&p, &req).allow);
    // Wrong target → DENY.
    let req2 = Request::new(left("read"))
        .on("urn:asset/secret")
        .by("https://alice.ex/me");
    assert!(!evaluate(&p, &req2).allow);
    // Wrong action → DENY.
    let req3 = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    assert!(!evaluate(&p, &req3).allow);
}

#[test]
fn use_umbrella_action_subsumes() {
    // odrl:use is the umbrella action — a permission for `use` permits `read`.
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/u> a odrl:Set ; odrl:permission [
    odrl:action odrl:use ; odrl:target <urn:asset/x> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let req = Request::new(left("read")).on("urn:asset/x");
    assert!(evaluate(&p, &req).allow);
}

// ---------------------------------------------------------------------------
// 2. Prohibition overrides permission.
// ---------------------------------------------------------------------------
#[test]
fn prohibition_overrides_permission() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/po> a odrl:Set ;
  odrl:permission [ odrl:action odrl:read ; odrl:target <urn:asset/x> ] ;
  odrl:prohibition [ odrl:action odrl:read ; odrl:target <urn:asset/x> ;
                     odrl:assignee <https://mallory.ex/me> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // Alice: permission applies, no prohibition matches her → ALLOW.
    let alice = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    assert!(evaluate(&p, &alice).allow);
    // Mallory: the prohibition matches → DENY even though the permission would.
    let mallory = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://mallory.ex/me");
    let d = evaluate(&p, &mallory);
    assert!(!d.allow);
    assert_eq!(
        d.matched_rules.len(),
        1,
        "the prohibition is the justification"
    );
}

// ---------------------------------------------------------------------------
// 3. Constraint gating — dateTime window (inside ALLOW, outside DENY).
// ---------------------------------------------------------------------------
#[test]
fn datetime_window_gates() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/t> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T23:59:59Z"^^xsd:dateTime ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let base = Request::new(left("read")).on("urn:asset/x");
    // Inside the window → ALLOW.
    assert!(
        evaluate(
            &p,
            &base
                .clone()
                .with(left("dateTime"), dt("2026-06-16T09:00:00Z"))
        )
        .allow
    );
    // After the window → DENY.
    assert!(
        !evaluate(
            &p,
            &base
                .clone()
                .with(left("dateTime"), dt("2027-03-01T00:00:00Z"))
        )
        .allow
    );
    // No dateTime supplied at all → DENY (fail-closed: no evidence).
    assert!(!evaluate(&p, &base).allow);
}

#[test]
fn purpose_eq_and_neq() {
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/p> a odrl:Set ; odrl:permission [
    odrl:action odrl:use ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ;
                      odrl:operator odrl:eq ;
                      odrl:rightOperand <urn:purpose/research> ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let base = Request::new(left("use")).on("urn:asset/x");
    // Matching purpose → ALLOW.
    let ok = base
        .clone()
        .with(left("purpose"), Value::Iri("urn:purpose/research".into()));
    assert!(evaluate(&p, &ok).allow);
    // Different purpose → DENY.
    let bad = base.with(left("purpose"), Value::Iri("urn:purpose/marketing".into()));
    assert!(!evaluate(&p, &bad).allow);
}

#[test]
fn recipient_is_part_of_set() {
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/r> a odrl:Set ; odrl:permission [
    odrl:action odrl:distribute ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ;
                      odrl:operator odrl:isPartOf ;
                      odrl:rightOperand "nodeB|nodeC" ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let base = Request::new(left("distribute")).on("urn:asset/x");
    assert!(
        evaluate(
            &p,
            &base
                .clone()
                .with(left("recipient"), Value::Str("nodeB".into()))
        )
        .allow
    );
    assert!(
        evaluate(
            &p,
            &base
                .clone()
                .with(left("recipient"), Value::Str("nodeC".into()))
        )
        .allow
    );
    // Not in the permitted recipient set → DENY.
    assert!(
        !evaluate(
            &p,
            &base.with(left("recipient"), Value::Str("nodeD".into()))
        )
        .allow
    );
}

#[test]
fn count_numeric_operators() {
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/c> a odrl:Set ; odrl:permission [
    odrl:action odrl:use ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:count ;
                      odrl:operator odrl:lteq ;
                      odrl:rightOperand "5"^^xsd:integer ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let base = Request::new(left("use")).on("urn:asset/x");
    assert!(evaluate(&p, &base.clone().with(left("count"), Value::Num(3.0))).allow);
    assert!(evaluate(&p, &base.clone().with(left("count"), Value::Num(5.0))).allow);
    assert!(!evaluate(&p, &base.with(left("count"), Value::Num(6.0))).allow);
}

// ---------------------------------------------------------------------------
// 4. Duty not discharged → DENY; discharged → ALLOW.
// ---------------------------------------------------------------------------
#[test]
fn duty_must_be_discharged() {
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/d> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:duty [ odrl:action odrl:anonymize ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    assert_eq!(p.permissions[0].duties.len(), 1);
    let base = Request::new(left("read")).on("urn:asset/x");
    // Duty not discharged → DENY.
    let d = evaluate(&p, &base);
    assert!(!d.allow);
    assert!(
        d.unmet_constraints.iter().any(|m| m.contains("duty")),
        "{d:?}"
    );
    // Discharged → ALLOW.
    let ok = base.discharge(left("anonymize"));
    assert!(evaluate(&p, &ok).allow);
}

// ---------------------------------------------------------------------------
// 5. Fail-closed defaults.
// ---------------------------------------------------------------------------
#[test]
fn empty_policy_denies_everything() {
    let p = parse_policy_str("@prefix odrl: <http://www.w3.org/ns/odrl/2/> .", "turtle").unwrap();
    let d: Decision = evaluate(&p, &Request::new(left("read")).on("urn:asset/x"));
    assert!(!d.allow);
    assert!(d.matched_rules.is_empty());
}

#[test]
fn unknown_operator_fails_closed() {
    // odrl:hasPart is not a comparison operator we support → the constraint
    // becomes an unsatisfiable guard, so the permission can never match.
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/x> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ;
                      odrl:operator odrl:hasPart ;
                      odrl:rightOperand <urn:purpose/research> ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let req = Request::new(left("read"))
        .on("urn:asset/x")
        .with(left("purpose"), Value::Iri("urn:purpose/research".into()));
    assert!(
        !evaluate(&p, &req).allow,
        "unknown operator must fail closed"
    );
}

#[test]
fn malformed_constraint_fails_closed() {
    // A constraint missing its operator/rightOperand is an unsatisfiable guard.
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/m> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let req = Request::new(left("read")).on("urn:asset/x");
    assert!(!evaluate(&p, &req).allow);
}

#[test]
fn untyped_policy_with_rules_is_parsed() {
    // No `a odrl:Set` — the parser still finds the rule via odrl:permission.
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/untyped> odrl:permission [ odrl:action odrl:read ; odrl:target <urn:asset/x> ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    assert_eq!(p.iri.as_deref(), Some("urn:pol/untyped"));
    assert!(evaluate(&p, &Request::new(left("read")).on("urn:asset/x")).allow);
}

#[test]
fn datetime_xsd_typed_constant_roundtrips() {
    // Make sure the XSD dateTime datatype is recognized (parses to Value::DateTime,
    // compared by instant not string-coincidence).
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/t2> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator odrl:gteq ;
                      odrl:rightOperand "2026-01-01T00:00:00Z"^^<{XSD_DT}> ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let base = Request::new(left("read")).on("urn:asset/x");
    // At/after the lower bound → ALLOW.
    assert!(
        evaluate(
            &p,
            &base
                .clone()
                .with(left("dateTime"), dt("2026-06-16T00:00:00Z"))
        )
        .allow
    );
    // Before the lower bound → DENY.
    assert!(!evaluate(&p, &base.with(left("dateTime"), dt("2025-12-31T00:00:00Z"))).allow);
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-2pcf — prohibition_status: the three-valued deny-retraction dual
// of matched_prohibition (Applies / Ambiguous / Withdrawn). The bridge uses this to
// retract a materialized deny ONLY on a DEFINITE withdrawal, never on missing evidence.
// ---------------------------------------------------------------------------
fn windowed_prohibition() -> sparq_policy::Policy {
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/p> a odrl:Set ; odrl:prohibition [
    odrl:action odrl:write ; odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
        odrl:rightOperand "2026-01-01T00:00:00Z"^^<{XSD_DT}> ] ] .
"#
    );
    parse_policy_str(&ttl, "turtle").unwrap()
}

#[test]
fn prohibition_status_applies_when_constraint_holds() {
    let p = windowed_prohibition();
    // Evidence the window holds (now < bound) → still carves out → KEEP the deny.
    let req = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://alice.ex/me")
        .with(left("dateTime"), dt("2025-06-01T00:00:00Z"));
    assert_eq!(prohibition_status(&p, &req), ProhibitionStatus::Applies);
}

#[test]
fn prohibition_status_withdrawn_when_constraint_definitely_false() {
    let p = windowed_prohibition();
    // Evidence the window LAPSED (now >= bound, operator lt) → definitely no → RETRACT.
    let req = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://alice.ex/me")
        .with(left("dateTime"), dt("2026-06-01T00:00:00Z"));
    assert_eq!(prohibition_status(&p, &req), ProhibitionStatus::Withdrawn);
}

#[test]
fn prohibition_status_ambiguous_when_no_evidence() {
    let p = windowed_prohibition();
    // NO dateTime evidence → cannot prove the window lapsed → AMBIGUOUS → KEEP the deny.
    let req = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    assert_eq!(prohibition_status(&p, &req), ProhibitionStatus::Ambiguous);
}

#[test]
fn prohibition_status_withdrawn_on_structural_mismatch() {
    let p = windowed_prohibition();
    // A different party is not carved out at all — structurally Withdrawn, even with
    // NO constraint evidence (a structural attribute is a DEFINITE non-match).
    let other = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://bob.ex/me");
    assert_eq!(prohibition_status(&p, &other), ProhibitionStatus::Withdrawn);
    // An empty policy (prohibition removed) is also Withdrawn.
    let empty = parse_policy_str(
        &format!(r#"@prefix odrl: <{ODRL}> . <urn:pol/p> a odrl:Set ."#),
        "turtle",
    )
    .unwrap();
    let req = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    assert_eq!(
        prohibition_status(&empty, &req),
        ProhibitionStatus::Withdrawn
    );
}

#[test]
fn prohibition_status_ambiguous_only_if_no_other_match() {
    // Two prohibitions: one unconstrained (always matches) + one windowed (ambiguous w/o
    // evidence). The unconstrained one still matches → Applies wins over Ambiguous.
    let ttl = format!(
        r#"
@prefix odrl: <{ODRL}> .
<urn:pol/p> a odrl:Set ;
    odrl:prohibition [ odrl:action odrl:write ; odrl:target <urn:asset/x> ;
        odrl:assignee <https://alice.ex/me> ] ;
    odrl:prohibition [ odrl:action odrl:write ; odrl:target <urn:asset/x> ;
        odrl:assignee <https://alice.ex/me> ;
        odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
            odrl:rightOperand "2026-01-01T00:00:00Z"^^<{XSD_DT}> ] ] .
"#
    );
    let p = parse_policy_str(&ttl, "turtle").unwrap();
    let req = Request::new(left("write"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    assert_eq!(prohibition_status(&p, &req), ProhibitionStatus::Applies);
}
