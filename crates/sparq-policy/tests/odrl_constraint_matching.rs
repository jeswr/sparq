//! ODRL constraint-matching conformance batch — end-to-end parse + evaluate tests
//! for the three related matching-semantics features, each through the REAL
//! `parse_policy_str` + `evaluate` path (not a mock). [OPUS-4.8]
//!
//! - **`odrl:use` umbrella vs the ODRL action hierarchy (sq-euhr3).** `use` subsumes
//!   its sub-actions (`read`/`write`/…) but NOT the ownership-`transfer` subtree
//!   (`sell`/`give`) — the canonical ODRL 2.2 reading the SolidLab suite expects.
//! - **`odrl:PartyCollection`/`odrl:AssetCollection` membership (sq-k7itg).** A rule
//!   targeting a collection matches a request whose party/asset is `odrl:partOf` it.
//! - **`odrl:LogicalConstraint` compound constraints (sq-a0zef).** `odrl:and`/`or`/
//!   `xone` over a (possibly nested) operand set, fail-closed on unprovable operands.

use sparq_policy::{evaluate, parse_policy_str, Request, Value};

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";

fn left(local: &str) -> String {
    format!("{ODRL}{local}")
}
fn dt(s: &str) -> Value {
    Value::DateTime(s.to_owned())
}

// ===========================================================================
// sq-euhr3 — odrl:use umbrella reconciled with the ODRL 2.2 action hierarchy.
// ===========================================================================

/// A `use` permission permits its included sub-actions (read/write) but NOT the
/// disjoint `transfer` subtree (sell/give) — matching SolidLab cases 007/009 (Active)
/// vs 010/017 (Inactive).
#[test]
fn use_subsumes_sub_actions_but_not_transfer_subtree() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/u> a odrl:Set ; odrl:permission [
    odrl:action odrl:use ; odrl:target <urn:asset/x> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // read, write, display → permitted (included in `use`).
    for act in ["read", "write", "display", "modify", "print"] {
        let req = Request::new(left(act)).on("urn:asset/x");
        assert!(evaluate(&p, &req).allow, "use should permit {act}");
    }
    // sell, give, transfer → DENIED (the ownership-transfer subtree is outside `use`).
    for act in ["sell", "give", "transfer"] {
        let req = Request::new(left(act)).on("urn:asset/x");
        assert!(
            !evaluate(&p, &req).allow,
            "use must NOT permit the transfer-subtree action {act}"
        );
    }
}

/// A concrete (non-`use`) permission still matches only its own action exactly.
#[test]
fn concrete_action_is_exact() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/r> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    assert!(evaluate(&p, &Request::new(left("read")).on("urn:asset/x")).allow);
    // a `read` permission does not cover `write`/`sell`.
    assert!(!evaluate(&p, &Request::new(left("write")).on("urn:asset/x")).allow);
    assert!(!evaluate(&p, &Request::new(left("sell")).on("urn:asset/x")).allow);
}

/// A `use` PROHIBITION carves out its sub-actions but not the transfer subtree, so a
/// `sell` request is not blocked by a `use` prohibition (consistency with the grant
/// path through the deny-overrides carve-out).
#[test]
fn use_prohibition_carves_sub_actions_not_transfer() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/p> a odrl:Set ;
    odrl:permission [ odrl:action odrl:use ; odrl:target <urn:asset/x> ] ;
    odrl:prohibition [ odrl:action odrl:use ; odrl:target <urn:asset/x> ;
        odrl:assignee <https://bob.ex/me> ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // bob asks to read → the use prohibition carves him out → DENY.
    let read = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://bob.ex/me");
    assert!(!evaluate(&p, &read).allow);
    // bob asks to sell → outside `use`; the prohibition does NOT carve it out, and the
    // `use` permission also does not grant it → DENY (no permission), but NOT via the
    // prohibition (the request is not in the use subtree at all).
    let sell = Request::new(left("sell"))
        .on("urn:asset/x")
        .by("https://bob.ex/me");
    assert!(!evaluate(&p, &sell).allow);
}

// ===========================================================================
// sq-k7itg — Party/Asset collection membership matching.
// ===========================================================================

/// A permission whose `assignee` is an `odrl:PartyCollection` grants a request whose
/// party is `odrl:partOf` that collection (membership evidence supplied per the sotw).
#[test]
fn party_collection_membership_grants_member() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix ex: <http://example.org/> .
<urn:pol/pc> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target ex:x ; odrl:assignee ex:partyCollection ] .
ex:partyCollection a odrl:PartyCollection ; odrl:source ex:partyIdentifier .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // alice IS a member → grant.
    let member = Request::new(left("read"))
        .on("http://example.org/x")
        .by("http://example.org/alice")
        .with_party_membership(
            "http://example.org/alice",
            "http://example.org/partyCollection",
        );
    assert!(
        evaluate(&p, &member).allow,
        "a collection member must be granted"
    );

    // carol is NOT a member (no membership edge) → DENY (fail-closed: not widened).
    let nonmember = Request::new(left("read"))
        .on("http://example.org/x")
        .by("http://example.org/carol");
    assert!(
        !evaluate(&p, &nonmember).allow,
        "a non-member must be denied (membership is never inferred)"
    );
}

/// A permission whose `target` is an `odrl:AssetCollection` grants a request whose asset
/// is `odrl:partOf` that collection.
#[test]
fn asset_collection_membership_grants_member_asset() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix ex: <http://example.org/> .
<urn:pol/ac> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target ex:assetCollection ; odrl:assignee ex:alice ] .
ex:assetCollection a odrl:AssetCollection ; odrl:source ex:assetIdentifier .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let in_collection = Request::new(left("read"))
        .on("http://example.org/x")
        .by("http://example.org/alice")
        .with_asset_membership("http://example.org/x", "http://example.org/assetCollection");
    assert!(evaluate(&p, &in_collection).allow);

    // an asset NOT in the collection → DENY.
    let outside = Request::new(left("read"))
        .on("http://example.org/y")
        .by("http://example.org/alice");
    assert!(!evaluate(&p, &outside).allow);
}

/// Both collections at once (the suite's 055 shape): party-in-collection AND
/// asset-in-collection.
#[test]
fn both_collections_compose() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix ex: <http://example.org/> .
<urn:pol/both> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target ex:assetCollection ; odrl:assignee ex:partyCollection ] .
ex:partyCollection a odrl:PartyCollection ; odrl:source ex:partyIdentifier .
ex:assetCollection a odrl:AssetCollection ; odrl:source ex:assetIdentifier .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let req = Request::new(left("read"))
        .on("http://example.org/x")
        .by("http://example.org/alice")
        .with_party_memberships([(
            "http://example.org/alice",
            "http://example.org/partyCollection",
        )])
        .with_asset_memberships([("http://example.org/x", "http://example.org/assetCollection")]);
    assert!(evaluate(&p, &req).allow);

    // missing the party membership → DENY (one half short).
    let req_no_party = Request::new(left("read"))
        .on("http://example.org/x")
        .by("http://example.org/alice")
        .with_asset_membership("http://example.org/x", "http://example.org/assetCollection");
    assert!(!evaluate(&p, &req_no_party).allow);
}

/// Membership for the WRONG collection does not match (the edge must name the rule's
/// collection, not just any collection).
#[test]
fn membership_for_wrong_collection_does_not_match() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix ex: <http://example.org/> .
<urn:pol/pc> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target ex:x ; odrl:assignee ex:groupA ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let req = Request::new(left("read"))
        .on("http://example.org/x")
        .by("http://example.org/alice")
        // alice is in groupB, but the rule names groupA.
        .with_party_membership("http://example.org/alice", "http://example.org/groupB");
    assert!(!evaluate(&p, &req).allow);
}

// ===========================================================================
// sq-a0zef — LogicalConstraint (and / or / xone) compound constraints.
// ===========================================================================

/// `odrl:and` of two dateTime bounds (a closed time window) — satisfied inside the
/// window, definitely-unsatisfied outside, fail-closed with no time evidence.
#[test]
fn logical_and_two_sided_window() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/and> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ a odrl:LogicalConstraint ; odrl:and
        [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gt ;
          odrl:rightOperand "2024-01-01T00:00:00Z"^^xsd:dateTime ] ,
        [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
          odrl:rightOperand "2024-12-31T23:59:59Z"^^xsd:dateTime ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    assert_eq!(p.permissions[0].logical_constraints.len(), 1);
    let base = Request::new(left("read")).on("urn:asset/x");
    // inside the window → both operands hold → AND satisfied → grant.
    assert!(evaluate(&p, &base.clone().at("2024-06-15T12:00:00Z")).allow);
    // before the window → the `gt` operand definitely fails → AND fails → deny.
    assert!(!evaluate(&p, &base.clone().at("2023-12-31T00:00:00Z")).allow);
    // after the window → the `lt` operand definitely fails → deny.
    assert!(!evaluate(&p, &base.clone().at("2025-06-01T00:00:00Z")).allow);
    // NO time evidence → unprovable → fail-closed (no silent pass on an AND).
    assert!(!evaluate(&p, &base).allow);
}

/// `odrl:or` — satisfied iff at least one operand holds; definitely-unsatisfied iff every
/// operand definitely fails.
#[test]
fn logical_or_at_least_one() {
    // Permit if the purpose is research OR teaching.
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/or> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ a odrl:LogicalConstraint ; odrl:or
        [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
          odrl:rightOperand <urn:purpose/research> ] ,
        [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
          odrl:rightOperand <urn:purpose/teaching> ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let base = Request::new(left("read")).on("urn:asset/x");
    // research → one operand holds → grant.
    let research = base
        .clone()
        .for_purpose(Value::Iri("urn:purpose/research".into()));
    assert!(evaluate(&p, &research).allow);
    // teaching → the other operand holds → grant.
    let teaching = base
        .clone()
        .for_purpose(Value::Iri("urn:purpose/teaching".into()));
    assert!(evaluate(&p, &teaching).allow);
    // marketing → both operands definitely fail → deny.
    let marketing = base
        .clone()
        .for_purpose(Value::Iri("urn:purpose/marketing".into()));
    assert!(!evaluate(&p, &marketing).allow);
    // no purpose → both unprovable → fail-closed (no grant).
    assert!(!evaluate(&p, &base).allow);
}

/// `odrl:xone` — satisfied iff EXACTLY ONE operand holds; an unprovable operand keeps the
/// exact-one count from being provable (fail-closed).
#[test]
fn logical_xone_exactly_one() {
    // Two disjoint purpose options; exactly one must hold.
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/xone> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ a odrl:LogicalConstraint ; odrl:xone
        [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
          odrl:rightOperand <urn:purpose/a> ] ,
        [ odrl:leftOperand odrl:recipient ; odrl:operator odrl:eq ;
          odrl:rightOperand <https://bob.ex/me> ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // purpose=a, recipient!=bob → exactly one holds → grant.
    let one = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://carol.ex/me")
        .for_purpose(Value::Iri("urn:purpose/a".into()));
    assert!(evaluate(&p, &one).allow);
    // purpose=a AND recipient=bob → TWO hold → xone fails → deny.
    let both = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://bob.ex/me")
        .for_purpose(Value::Iri("urn:purpose/a".into()));
    assert!(!evaluate(&p, &both).allow);
    // purpose=other, recipient=bob → exactly one (recipient) holds → grant.
    let other_one = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://bob.ex/me")
        .for_purpose(Value::Iri("urn:purpose/other".into()));
    assert!(evaluate(&p, &other_one).allow);
    // purpose missing (unprovable) + recipient=bob (holds): the count is not provably
    // exactly-one (the purpose operand could also hold) → fail-closed deny.
    let unprovable = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://bob.ex/me");
    assert!(!evaluate(&p, &unprovable).allow);
}

/// A NESTED LogicalConstraint — an `odrl:or` of `odrl:and`s (the suite's 062 shape:
/// a disjunction of time windows). Satisfied iff the request instant falls in any window.
#[test]
fn nested_or_of_ands_time_windows() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/nested> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ a odrl:LogicalConstraint ; odrl:or
        [ a odrl:LogicalConstraint ; odrl:and
            [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gt ;
              odrl:rightOperand "2024-01-01T09:00:00Z"^^xsd:dateTime ] ,
            [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
              odrl:rightOperand "2024-01-01T17:00:00Z"^^xsd:dateTime ] ] ,
        [ a odrl:LogicalConstraint ; odrl:and
            [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gt ;
              odrl:rightOperand "2024-02-12T09:00:00Z"^^xsd:dateTime ] ,
            [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
              odrl:rightOperand "2024-02-12T17:00:00Z"^^xsd:dateTime ] ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let base = Request::new(left("read")).on("urn:asset/x");
    // 2024-02-12T11:20 → falls in the SECOND window → grant.
    assert!(evaluate(&p, &base.clone().at("2024-02-12T11:20:10Z")).allow);
    // 2024-01-01T12:00 → falls in the FIRST window → grant.
    assert!(evaluate(&p, &base.clone().at("2024-01-01T12:00:00Z")).allow);
    // 2024-03-01T12:00 → in NEITHER window → both AND-windows definitely fail → deny.
    assert!(!evaluate(&p, &base.clone().at("2024-03-01T12:00:00Z")).allow);
    // no time at all → both windows unprovable → fail-closed.
    assert!(!evaluate(&p, &base).allow);
}

/// A LogicalConstraint AND a sibling atomic constraint on one rule are themselves ANDed:
/// the rule grants only when both the compound and the atomic constraint hold.
#[test]
fn logical_and_atomic_constraints_are_anded() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/mix> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
        odrl:rightOperand <urn:purpose/research> ] ;
    odrl:constraint [ a odrl:LogicalConstraint ; odrl:or
        [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
          odrl:rightOperand "2024-12-31T23:59:59Z"^^xsd:dateTime ] ,
        [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gt ;
          odrl:rightOperand "2030-01-01T00:00:00Z"^^xsd:dateTime ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    assert_eq!(
        p.permissions[0].constraints.len(),
        1,
        "one atomic constraint"
    );
    assert_eq!(
        p.permissions[0].logical_constraints.len(),
        1,
        "one compound"
    );
    let base = Request::new(left("read")).on("urn:asset/x");
    // purpose=research AND time<2024-end (the OR's first branch) → grant.
    let ok = base
        .clone()
        .for_purpose(Value::Iri("urn:purpose/research".into()))
        .with(left("dateTime"), dt("2024-06-01T00:00:00Z"));
    assert!(evaluate(&p, &ok).allow);
    // right time but WRONG purpose → the atomic constraint fails → deny.
    let wrong_purpose = base
        .clone()
        .for_purpose(Value::Iri("urn:purpose/marketing".into()))
        .with(left("dateTime"), dt("2024-06-01T00:00:00Z"));
    assert!(!evaluate(&p, &wrong_purpose).allow);
    // right purpose but time in NEITHER OR branch → the compound fails → deny.
    let wrong_time = base
        .clone()
        .for_purpose(Value::Iri("urn:purpose/research".into()))
        .with(left("dateTime"), dt("2027-06-01T00:00:00Z"));
    assert!(!evaluate(&p, &wrong_time).allow);
}

/// A LogicalConstraint that gates a PROHIBITION carves out the request only when the
/// compound holds (deny-overrides), and the carve-out lifts when the compound fails.
#[test]
fn logical_constraint_on_prohibition() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/p> a odrl:Set ;
    odrl:permission [ odrl:action odrl:read ; odrl:target <urn:asset/x> ;
        odrl:assignee <https://alice.ex/me> ] ;
    odrl:prohibition [ odrl:action odrl:read ; odrl:target <urn:asset/x> ;
        odrl:assignee <https://alice.ex/me> ;
        odrl:constraint [ a odrl:LogicalConstraint ; odrl:and
            [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:gt ;
              odrl:rightOperand "2024-01-01T00:00:00Z"^^xsd:dateTime ] ,
            [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
              odrl:rightOperand "2024-12-31T23:59:59Z"^^xsd:dateTime ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let base = Request::new(left("read"))
        .on("urn:asset/x")
        .by("https://alice.ex/me");
    // inside the prohibited window → the compound holds → carve-out fires → DENY.
    assert!(!evaluate(&p, &base.clone().at("2024-06-01T00:00:00Z")).allow);
    // outside the window → the compound fails → carve-out gone → the permission grants.
    assert!(evaluate(&p, &base.clone().at("2025-06-01T00:00:00Z")).allow);
}

/// An empty `odrl:or` (a LogicalConstraint with no operand) can never be satisfied —
/// fail-closed. (A malformed compound must not silently pass.)
#[test]
fn empty_or_is_fail_closed() {
    // An `or` whose only operand is a malformed (unknown-operator) atomic constraint →
    // that operand is the unsatisfiable guard → the `or` definitely fails → deny.
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/bad> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ a odrl:LogicalConstraint ; odrl:or
        [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:bogusOp ;
          odrl:rightOperand <urn:purpose/x> ] ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let req = Request::new(left("read"))
        .on("urn:asset/x")
        .for_purpose(Value::Iri("urn:purpose/x".into()));
    assert!(
        !evaluate(&p, &req).allow,
        "a malformed compound operand must fail closed"
    );
}
