//! Multi-valued `odrl:rightOperand` parsing — RDF lists (`odrl:rightOperand
//! ( <a> <b> )`) and multiple objects (`odrl:rightOperand <a>, <b>`) folded into
//! the `|`-separated set encoding the set-relation operators
//! (`isPartOf`/`isAnyOf`/`isNoneOf`) consume. End-to-end through the REAL
//! `parse_policy_str` + `evaluate` path (not a mock). [FABLE-5] sq-ueydm.
//!
//! Load-bearing invariants pinned here:
//! * a single-valued right operand (including a ONE-element list) keeps the
//!   TYPED value path (numeric/dateTime magnitude comparison — no behavior
//!   change to the pre-fold parse);
//! * a multi-value under a NON-set operator (`eq`, …) is ambiguous and fails
//!   CLOSED (the unsatisfiable guard), and the folded `a|b` encoding never
//!   leaks into equality comparison;
//! * a member that carries a set-encoding separator (`|`/whitespace/`,`) cannot
//!   be encoded faithfully — the whole constraint fails closed rather than
//!   splitting into unintended members (a fail-OPEN hazard);
//! * a malformed CYCLIC list terminates and still yields its complete member set.

use sparq_policy::{evaluate, parse_policy_str, Operator, Policy, Request, Value};

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";

fn action(local: &str) -> String {
    format!("{}{}", ODRL, local)
}

/// A permission gated on `purpose <op> <right>` (TTL helper — same shape as
/// `odrl_set_operators.rs`).
fn purpose_policy(op: &str, right_ttl: &str) -> Policy {
    let ttl = format!(
        r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/p> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:{} ;
                      odrl:rightOperand {} ] ] .
"#,
        op, right_ttl
    );
    parse_policy_str(&ttl, "turtle").unwrap()
}

fn read_x() -> Request {
    Request::new(action("read")).on("urn:asset/x")
}

fn purpose(iri: &str) -> Request {
    read_x().for_purpose(Value::Iri(iri.to_owned()))
}

// ===========================================================================
// Structural: the parsed constraint carries the folded set encoding.
// ===========================================================================

/// An RDF-list right operand folds to the `|`-joined set-encoding string, in
/// list order — a DIRECT parse-level assertion on the encoding (not only on
/// evaluation behavior).
#[test]
fn list_right_operand_folds_to_set_encoding() {
    let p = purpose_policy("isAnyOf", "( <urn:purpose/research> <urn:purpose/education> )");
    let c = &p.permissions[0].constraints[0];
    assert_eq!(c.operator, Operator::IsAnyOf);
    assert_eq!(
        c.right,
        Value::Str("urn:purpose/research|urn:purpose/education".to_owned()),
        "rdf:first/rdf:rest chain must fold into the |-separated set encoding"
    );
}

/// Multiple `rightOperand` objects fold into ONE constraint carrying the full
/// member set (object order in RDF is unspecified, so assert set equality).
#[test]
fn multi_object_right_operand_folds_to_set_encoding() {
    let p = purpose_policy("isAnyOf", "<urn:purpose/research>, <urn:purpose/education>");
    assert_eq!(
        p.permissions[0].constraints.len(),
        1,
        "several objects of one rightOperand are ONE constraint, not several"
    );
    let c = &p.permissions[0].constraints[0];
    let Value::Str(s) = &c.right else {
        panic!("multi-object rightOperand must fold to Value::Str, got {:?}", c.right);
    };
    let mut members: Vec<&str> = s.split('|').collect();
    members.sort_unstable();
    assert_eq!(members, ["urn:purpose/education", "urn:purpose/research"]);
}

// ===========================================================================
// Evaluation: list / multi-object sets behave as faithful set membership.
// ===========================================================================

/// `isAnyOf ( <a> <b> )`: every listed member grants, a non-member denies.
#[test]
fn is_any_of_list_member_grants_non_member_denied() {
    let p = purpose_policy("isAnyOf", "( <urn:purpose/research> <urn:purpose/education> )");
    assert!(evaluate(&p, &purpose("urn:purpose/research")).allow);
    assert!(evaluate(&p, &purpose("urn:purpose/education")).allow);
    assert!(!evaluate(&p, &purpose("urn:purpose/marketing")).allow);
}

/// `isPartOf` consumes the same folded encoding (set membership).
#[test]
fn is_part_of_list_member_grants_non_member_denied() {
    let p = purpose_policy("isPartOf", "( <urn:purpose/research> <urn:purpose/education> )");
    assert!(evaluate(&p, &purpose("urn:purpose/education")).allow);
    assert!(!evaluate(&p, &purpose("urn:purpose/marketing")).allow);
}

/// Multi-object `isAnyOf <a>, <b>`: both members grant, a non-member denies —
/// previously first-binding-wins silently dropped the second member.
#[test]
fn is_any_of_multi_object_every_member_grants() {
    let p = purpose_policy("isAnyOf", "<urn:purpose/research>, <urn:purpose/education>");
    assert!(evaluate(&p, &purpose("urn:purpose/research")).allow);
    assert!(
        evaluate(&p, &purpose("urn:purpose/education")).allow,
        "the second rightOperand object must not be dropped"
    );
    assert!(!evaluate(&p, &purpose("urn:purpose/marketing")).allow);
}

/// `isNoneOf ( <a> <b> )` on a permission: a non-member grants, each listed
/// member is a definite mismatch (deny).
#[test]
fn is_none_of_list_excludes_every_member() {
    let p = purpose_policy("isNoneOf", "( <urn:purpose/a> <urn:purpose/b> )");
    assert!(evaluate(&p, &purpose("urn:purpose/c")).allow);
    assert!(!evaluate(&p, &purpose("urn:purpose/a")).allow);
    assert!(
        !evaluate(&p, &purpose("urn:purpose/b")).allow,
        "the second list member must also be excluded"
    );
}

/// `isNoneOf` list on a PROHIBITION: the prohibition fires (deny) exactly when
/// the stated purpose is outside the set — the folded set drives the negative
/// operator faithfully in the deny direction too.
#[test]
fn prohibition_is_none_of_list_fires_outside_the_set() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/p> a odrl:Set ;
  odrl:permission  [ odrl:action odrl:read ; odrl:target <urn:asset/x> ] ;
  odrl:prohibition [ odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:isNoneOf ;
                      odrl:rightOperand ( <urn:purpose/a> <urn:purpose/b> ) ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    // Purpose outside the set → prohibition satisfied → DENY overrides the permission.
    assert!(!evaluate(&p, &purpose("urn:purpose/c")).allow);
    // Purpose in the set → prohibition constraint unsatisfied → the permission grants.
    assert!(evaluate(&p, &purpose("urn:purpose/b")).allow);
}

/// A list right operand inside a compound `odrl:LogicalConstraint` operand is
/// folded through the same path (the logical-constraint atom table). The
/// combinator's operands stay several objects (`odrl:or _:c1` — the suite's
/// form); the LIST is the operand's `rightOperand`.
#[test]
fn logical_constraint_operand_folds_list_right_operand() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/p> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:or _:c1 ] ] .
_:c1 odrl:leftOperand odrl:purpose ; odrl:operator odrl:isAnyOf ;
     odrl:rightOperand ( <urn:purpose/research> <urn:purpose/education> ) .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    assert_eq!(
        p.permissions[0].logical_constraints.len(),
        1,
        "the compound constraint must parse (this test must not pass vacuously)"
    );
    assert!(evaluate(&p, &purpose("urn:purpose/education")).allow);
    assert!(!evaluate(&p, &purpose("urn:purpose/marketing")).allow);
}

// ===========================================================================
// Typed single-value path is unchanged.
// ===========================================================================

/// A ONE-element list stays TYPED: `dateTime lteq ( "…"^^xsd:dateTime )`
/// compares by instant, proving the single-member fold takes the typed
/// `value_of` path (not a lexical set string).
#[test]
fn single_element_list_keeps_typed_comparison() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/p> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand ( "2026-12-31T00:00:00Z"^^xsd:dateTime ) ] ] .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    let c = &p.permissions[0].constraints[0];
    assert!(
        matches!(c.right, Value::DateTime(_)),
        "one-element list must keep the typed dateTime value, got {:?}",
        c.right
    );
    assert!(evaluate(&p, &read_x().at("2026-06-01T00:00:00Z")).allow);
    assert!(!evaluate(&p, &read_x().at("2027-06-01T00:00:00Z")).allow);
}

// ===========================================================================
// Fail-closed degradations.
// ===========================================================================

/// A multi-value under a NON-set operator (`eq`) is ambiguous → the constraint
/// is unsatisfiable (fail-closed), and the folded `a|b` encoding must NOT leak
/// into equality: even a request value that IS literally the joined string
/// never grants.
#[test]
fn eq_with_multiple_values_fails_closed() {
    let p = purpose_policy("eq", "<urn:purpose/a>, <urn:purpose/b>");
    assert!(!evaluate(&p, &purpose("urn:purpose/a")).allow);
    assert!(!evaluate(&p, &purpose("urn:purpose/b")).allow);
    let joined = read_x().for_purpose(Value::Str("urn:purpose/a|urn:purpose/b".to_owned()));
    assert!(
        !evaluate(&p, &joined).allow,
        "the fold encoding must not become spuriously equal-comparable"
    );
}

/// A set member carrying a separator character (here: a space) cannot be
/// encoded faithfully — the WHOLE constraint fails closed: neither the clean
/// member, nor the dirty member, nor its would-be split fragments grant.
#[test]
fn separator_carrying_member_fails_the_whole_constraint_closed() {
    let p = purpose_policy("isAnyOf", r#"( "two words" <urn:purpose/b> )"#);
    assert!(
        !evaluate(&p, &purpose("urn:purpose/b")).allow,
        "an unencodable sibling member must fail the whole constraint closed"
    );
    for v in ["two words", "two", "words"] {
        let req = read_x().for_purpose(Value::Str(v.to_owned()));
        assert!(!evaluate(&p, &req).allow, "value {:?} must not grant", v);
    }
}

/// A malformed CYCLIC list terminates the parse and still yields its complete
/// member set (every cons cell is visited exactly once).
#[test]
fn cyclic_list_terminates_and_yields_complete_member_set() {
    let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
<urn:pol/p> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:isAnyOf ;
                      odrl:rightOperand _:l1 ] ] .
_:l1 rdf:first <urn:purpose/a> ; rdf:rest _:l2 .
_:l2 rdf:first <urn:purpose/b> ; rdf:rest _:l1 .
"#;
    let p = parse_policy_str(ttl, "turtle").unwrap();
    assert!(evaluate(&p, &purpose("urn:purpose/a")).allow);
    assert!(evaluate(&p, &purpose("urn:purpose/b")).allow);
    assert!(!evaluate(&p, &purpose("urn:purpose/c")).allow);
}

/// An EMPTY list (`rdf:nil` directly as the object) has no members: ordinary
/// request values never match (fail-closed, same as the pre-fold parse).
#[test]
fn empty_list_right_operand_never_matches_ordinary_values() {
    let p = purpose_policy("isAnyOf", "( )");
    assert!(!evaluate(&p, &purpose("urn:purpose/a")).allow);
}
