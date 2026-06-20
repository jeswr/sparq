//! # `wire.rs` — admitted facts → N3 merge → derived auth grants
//!
//! Takes the [`AdmittedFact`]s the gate produced and the controller-authored ABAC
//! rule (carried in the `.acr` Control-gated channel), feeds the admitted age fact
//! into the **shipped** `sparq-reason` N3 reasoner ahead of the materialiser, and
//! reads off the derived `auth:*` grants — the §3.1 worked example, concretely:
//!
//! ```n3
//! # admitted fact (issuer-tagged, trust:admitted):
//! <Jesse> schema:age 25 .
//! # the .acr ABAC rule (controller-authored, trusted root):
//! { ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <resourceX> } .
//! # ⇒ derived:  <Jesse> auth:read <resourceX> .
//! ```
//!
//! The derived grants are returned as plain `auth:*` triples. The
//! `sparq-solid`-side `trust-graph` feature installs them into `<urn:sparq:auth>`
//! **on top of** the unchanged WAC/ACP view (the ODRL-bridge precedent), so
//! everything downstream — the session-scoped `∪ allow ∖ ∪ deny` enforcement, the
//! query rewrite — is the existing shipped code, untouched.
//!
//! This is the "the existing N3 reasoner merges the admitted age fact with the
//! `.acl`/`.acr` rule" step (§6.0 pipeline step 4): the same full `reason_n3` the
//! materialiser uses, which supports `math:greaterThan`.
//!
//! [OPUS-4.8] sq-pfae PoC (issue #940). 🤖 SPARQ agent — trust-graph authorisation PoC.

use crate::admit::AdmittedFact;
use oxrdf::{NamedNode, Term, Triple};
use sparq_core::dict::Dict;
use sparq_reason::reason_n3;
use std::fmt::Write as _;

/// The `auth:` view namespace (`auth:read|write|append|control` — the SAME predicates
/// the shipped WAC/ACP rules emit and `sparq_solid::AuthIndex` reads).
pub const AUTH_NS: &str = "https://sparq.dev/ns/auth#";

/// Run the N3 merge: the admitted facts + the controller-authored ABAC `rule_n3`
/// (from the `.acr`) → the derived `auth:*` grant triples.
///
/// `rule_n3` is the trusted ABAC rule(s) carried in the Control-gated `.acr` channel,
/// e.g.:
///
/// ```n3
/// @prefix schema: <http://schema.org/> .
/// @prefix math:   <http://www.w3.org/2000/10/swap/math#> .
/// @prefix auth:   <https://sparq.dev/ns/auth#> .
/// { ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <https://pod.ex/resourceX> } .
/// ```
///
/// Returns only the `auth:*`-predicated closure triples (the derived grants), as
/// oxrdf [`Triple`]s ready to install into `<urn:sparq:auth>`. NEVER panics on a
/// reasoner error — a malformed rule yields an `Err` the caller fails closed on.
///
/// Honest boundary: the merge admits ONLY directly-attested admitted facts of a
/// trusted type — there is **no entailment laundering** of derived facts through the
/// trust boundary (§3.3 G3). The ABAC rule itself comes from the trusted `.acr`
/// channel, so it is not an admission surface.
pub fn derive_grants(admitted: &[AdmittedFact], rule_n3: &str) -> Result<Vec<Triple>, String> {
    if admitted.is_empty() {
        return Ok(Vec::new()); // default-deny: nothing admitted ⇒ nothing derived
    }
    // Render the admitted facts as N3 ground facts, prepend them to the rule, and run
    // the full `reason_n3` (the same evaluator the materialiser uses — supports
    // math:greaterThan). The facts are the issuer-attested claims, NOT pod content:
    // they reached here only by passing the admission gate.
    let mut src = String::new();
    for f in admitted {
        let _ = writeln!(
            src,
            "{} {} {} .",
            f.triple.subject, f.triple.predicate, f.triple.object
        );
    }
    src.push('\n');
    src.push_str(rule_n3);

    let mut dict = Dict::new();
    let closure = reason_n3(&mut dict, &src)?;

    // Keep only the derived `auth:*` grant triples (the derivation-stratum output).
    let mut grants: Vec<Triple> = Vec::new();
    for t in &closure {
        let p = dict.term(t[1]);
        let Term::NamedNode(pred) = &p else { continue };
        if !pred.as_str().starts_with(AUTH_NS) {
            continue;
        }
        let s = dict.term(t[0]);
        let o = dict.term(t[2]);
        if let Some(triple) = to_triple(s, pred.clone(), o) {
            if !grants.contains(&triple) {
                grants.push(triple);
            }
        }
    }
    Ok(grants)
}

fn to_triple(s: Term, p: NamedNode, o: Term) -> Option<Triple> {
    let subject = match s {
        Term::NamedNode(n) => oxrdf::NamedOrBlankNode::NamedNode(n),
        Term::BlankNode(b) => oxrdf::NamedOrBlankNode::BlankNode(b),
        _ => return None,
    };
    Some(Triple::new(subject, p, o))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{Literal, NamedNode};

    fn iri(s: &str) -> NamedNode {
        NamedNode::new(s).unwrap()
    }

    fn admitted_age(value: &str) -> Vec<AdmittedFact> {
        vec![AdmittedFact {
            triple: Triple::new(
                oxrdf::NamedOrBlankNode::NamedNode(iri("https://jesse.ex/card#me")),
                iri("http://schema.org/age"),
                Term::Literal(Literal::new_typed_literal(
                    value,
                    iri("http://www.w3.org/2001/XMLSchema#integer"),
                )),
            ),
            issuer: iri("https://gov.example/issuer"),
        }]
    }

    const RULE: &str = r#"@prefix schema: <http://schema.org/> .
@prefix math:   <http://www.w3.org/2000/10/swap/math#> .
@prefix auth:   <https://sparq.dev/ns/auth#> .
{ ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <https://pod.ex/resourceX> } .
"#;

    #[test]
    fn age_over_18_derives_the_read_grant() {
        let grants = derive_grants(&admitted_age("25"), RULE).unwrap();
        assert_eq!(
            grants.len(),
            1,
            "age 25 > 18 derives exactly one read grant"
        );
        let g = &grants[0];
        assert_eq!(g.predicate.as_str(), "https://sparq.dev/ns/auth#read");
        assert!(
            matches!(&g.subject, oxrdf::NamedOrBlankNode::NamedNode(n) if n.as_str() == "https://jesse.ex/card#me")
        );
    }

    #[test]
    fn age_16_admitted_but_rule_denies() {
        // age 16 is ADMITTED (it's a trusted age statement) but the > 18 rule does not
        // fire — so NO grant is derived. The negative case proves the merge is the
        // decision point, not admission.
        let grants = derive_grants(&admitted_age("16"), RULE).unwrap();
        assert!(
            grants.is_empty(),
            "age 16 admitted but rule denies: no grant"
        );
    }

    #[test]
    fn nothing_admitted_derives_nothing() {
        let grants = derive_grants(&[], RULE).unwrap();
        assert!(
            grants.is_empty(),
            "default-deny: empty admission ⇒ empty derivation"
        );
    }
}
