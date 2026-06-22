//! Parse-validation of the published `sec-prop:` extension vocabulary
//! `ontologies/zkp-sparql/secprop-ext.ttl` (sq-5oru9). The Turtle a working group /
//! reasoner reads MUST stay valid Turtle and MUST declare exactly the term set the
//! Rust constants name. The `secprop::tests::secprop_ext_iris_match_rust_constants`
//! unit test pins the IRIs against the constants; this test is the *syntactic* guard
//! (the `vocab_ttl.rs` discipline applied to the extension).
//!
//! Behind the default-OFF `secprop-vocab` feature — exercised by the matching
//! feature-matrix.yml leg (the silent-skip guard, #1171).
//!
//! [OPUS-4.8] sq-5oru9 (epic sq-0dksu; design PR #972). 🤖 SPARQ agent —
//! security-properties ontology.
#![cfg(feature = "secprop-vocab")]

use oxttl::TurtleParser;

const TTL: &str = include_str!("../ontologies/zkp-sparql/secprop-ext.ttl");

/// The published extension vocabulary is valid Turtle and non-empty.
#[test]
fn secprop_ext_ttl_is_valid_turtle() {
    let mut triples = 0usize;
    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        result.expect("secprop-ext.ttl must be valid Turtle");
        triples += 1;
    }
    assert!(
        triples > 0,
        "secprop-ext.ttl parsed to zero triples — the published vocabulary is empty",
    );
}

/// Every constant in `secprop::ALL_SECPROP_IRIS` appears as a SUBJECT in the parsed
/// Turtle (a coarse presence check independent of the unit-test constant pinning, so
/// a refactor of `secprop.rs` cannot silently drop a published term). The two reused
/// vendored class IRIs (`sec-prop:Unlinkability`, `sec-prop:SecurityProperty`) are
/// referenced (range / owl:imports) but not re-declared as subjects in the extension
/// file, so they are exempt — exactly the split the unit test makes.
#[test]
fn secprop_ext_ttl_declares_every_minted_term() {
    use oxrdf::NamedOrBlankNode;
    use sparq_trust::secprop::{
        ALL_SECPROP_IRIS, SEC_PROP_SECURITY_PROPERTY, SEC_PROP_UNLINKABILITY,
    };

    let reused_only = [SEC_PROP_UNLINKABILITY, SEC_PROP_SECURITY_PROPERTY];

    let mut subjects = std::collections::HashSet::new();
    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        let t = result.expect("valid Turtle");
        if let NamedOrBlankNode::NamedNode(s) = t.subject {
            subjects.insert(s.into_string());
        }
    }

    for &iri in ALL_SECPROP_IRIS {
        if reused_only.contains(&iri) {
            continue;
        }
        assert!(
            subjects.contains(iri),
            "secprop-ext.ttl does not declare `{}` as a subject",
            iri,
        );
    }
}

/// The assurance axis carries exactly the three ordered levels, the audit-status set
/// includes the live `ExternalSignOffPending` state, and the file references the open
/// `sq-qhy4` audit gate — the load-bearing honesty invariants of the vocabulary.
#[test]
fn secprop_ext_ttl_carries_the_assurance_axis_and_audit_gate() {
    use oxrdf::{NamedNode, NamedOrBlankNode, Term};
    use sparq_trust::secprop::{
        SECX_ASSURANCE_LEVEL, SECX_AUDIT_STATUS_CLASS, SECX_CLAIMED, SECX_CONJECTURED,
        SECX_EXTERNAL_SIGN_OFF_PENDING, SECX_PROVEN,
    };

    const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

    // Collect (subject, type) pairs.
    let mut typed: Vec<(String, String)> = Vec::new();
    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        let t = result.expect("valid Turtle");
        if t.predicate == NamedNode::new(RDF_TYPE).unwrap() {
            if let (NamedOrBlankNode::NamedNode(s), Term::NamedNode(o)) = (t.subject, t.object) {
                typed.push((s.into_string(), o.into_string()));
            }
        }
    }
    let is_a = |s: &str, class: &str| typed.iter().any(|(sub, ty)| sub == s && ty == class);

    for level in [SECX_PROVEN, SECX_CLAIMED, SECX_CONJECTURED] {
        assert!(
            is_a(level, SECX_ASSURANCE_LEVEL),
            "`{}` must be a secx:AssuranceLevel",
            level,
        );
    }
    assert!(
        is_a(SECX_EXTERNAL_SIGN_OFF_PENDING, SECX_AUDIT_STATUS_CLASS),
        "the live sq-qhy4 state ExternalSignOffPending must be a secx:AuditStatus",
    );

    assert!(
        TTL.contains("sq-qhy4"),
        "secprop-ext.ttl must reference the open external-audit gate (sq-qhy4)",
    );
}
