//! Parse-validation of the published machine-readable certification-scope vocabulary
//! `ontologies/trust/trust-framework.ttl` (sq-6syab.2; epic sq-6syab / issue #1592;
//! design record `research/trust-expression-spec.md` §3.4 / D4). The
//! `framework_vocab::tests` unit tests pin the IRIs against the Rust constants (the
//! *string* guard); this test is the *syntactic + structural* guard — the Turtle a
//! working group reads MUST stay valid Turtle, declare the `trustx:` terms, `owl:imports`
//! the `trust:` vocabulary it extends, and `rdfs:seeAlso` the vendored `sec-req:`
//! eIDAS/UK-DVS individuals rather than duplicating them (the design's D5).
//!
//! Behind the default-OFF `framework-vocab` feature, exactly like the module it guards.
//!
//! [FABLE-5] sq-6syab.2 (issue #1592). 🤖 SPARQ agent — trust-expression
//! certification-scope layer.
#![cfg(feature = "framework-vocab")]

use oxrdf::{NamedOrBlankNode, Term};
use oxttl::TurtleParser;

const TTL: &str = include_str!("../ontologies/trust/trust-framework.ttl");

const TRUSTX_NS: &str = "https://sparq.dev/ns/trust#";
const SEC_REQ_NS: &str = "https://w3id.org/zkp-sparql/sec-req#";
const RDFS_SEE_ALSO: &str = "http://www.w3.org/2000/01/rdf-schema#seeAlso";
const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// The published vocabulary is valid Turtle (parses without error) and is non-empty.
#[test]
fn trust_framework_ttl_is_valid_turtle() {
    let mut triples = 0usize;
    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        result.expect("trust-framework.ttl must be valid Turtle");
        triples += 1;
    }
    assert!(
        triples > 0,
        "trust-framework.ttl parsed to zero triples — the published vocabulary is empty",
    );
}

/// Every `trustx:` term the layer mints is declared in the Turtle as a subject — a
/// coarse presence check independent of the unit-test constant pinning, so a refactor of
/// `framework_vocab.rs` cannot silently drop a published term from the gate.
#[test]
fn trust_framework_ttl_declares_all_trustx_terms() {
    let expected = [
        // trust requirements
        "TrustRequirements",
        "question",
        "trustsIssuer",
        "trustsFramework",
        "requiresScopeConformance",
        "requiresValidStatusAt",
        "methodPolicy",
        // certification-scope layer
        "Framework",
        "Certification",
        "certifies",
        "underFramework",
        "scope",
        "validFrom",
        "validUntil",
        "AnyServiceScope",
        // status attestation
        "StatusAttestation",
        "coveredBy",
        // framework individuals
        "eIDAS2",
        "DIATF",
    ];

    let mut subjects = std::collections::HashSet::new();
    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        let t = result.expect("valid Turtle");
        if let NamedOrBlankNode::NamedNode(s) = t.subject {
            subjects.insert(s.into_string());
        }
    }
    for local in expected {
        let iri = format!("{TRUSTX_NS}{local}");
        assert!(
            subjects.contains(&iri),
            "trust-framework.ttl does not declare `trustx:{local}` ({iri}) as a subject",
        );
    }
}

/// The framework individuals `rdfs:seeAlso` the vendored `sec-req:` eIDAS/UK-DVS
/// individuals (no duplication — design D5), and the ontology node `owl:imports` the
/// `trust:` vocabulary it extends. Both are verified against the PARSED graph, not a
/// substring — the real invariant the design pins.
#[test]
fn extends_trust_and_references_vendored_sec_req_in_the_parsed_graph() {
    let mut see_also_objects: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut import_objects: std::collections::HashSet<String> = std::collections::HashSet::new();
    // eIDAS2 → sec-req:Eidas20, DIATF → sec-req:UkDvs, both by subject+object.
    let mut eidas_sees_vendored = false;
    let mut diatf_sees_vendored = false;

    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        let t = result.expect("valid Turtle");
        let pred = t.predicate.as_str().to_owned();
        if pred == RDFS_SEE_ALSO {
            if let Term::NamedNode(o) = &t.object {
                see_also_objects.insert(o.as_str().to_owned());
                if let NamedOrBlankNode::NamedNode(s) = &t.subject {
                    if s.as_str() == format!("{TRUSTX_NS}eIDAS2")
                        && o.as_str() == format!("{SEC_REQ_NS}Eidas20")
                    {
                        eidas_sees_vendored = true;
                    }
                    if s.as_str() == format!("{TRUSTX_NS}DIATF")
                        && o.as_str() == format!("{SEC_REQ_NS}UkDvs")
                    {
                        diatf_sees_vendored = true;
                    }
                }
            }
        }
        if pred == OWL_IMPORTS {
            if let Term::NamedNode(o) = &t.object {
                import_objects.insert(o.as_str().to_owned());
            }
        }
    }

    assert!(
        eidas_sees_vendored,
        "trustx:eIDAS2 must rdfs:seeAlso the vendored sec-req:Eidas20 (no duplication — D5)",
    );
    assert!(
        diatf_sees_vendored,
        "trustx:DIATF must rdfs:seeAlso the vendored sec-req:UkDvs (no duplication — D5)",
    );
    assert!(
        import_objects.contains("https://sparq.dev/ns/trust"),
        "the framework ontology node must owl:imports the trust: vocabulary it extends",
    );
}

/// No vendored `sec-req:` individual is REDECLARED (given an `rdf:type`) in this
/// extension file — they are referenced only. Verified on the parsed graph: no triple
/// has a `sec-req:` subject with predicate `rdf:type` (the D5 no-duplication invariant).
#[test]
fn does_not_redeclare_vendored_sec_req_individuals() {
    for result in TurtleParser::new().for_reader(TTL.as_bytes()) {
        let t = result.expect("valid Turtle");
        if let NamedOrBlankNode::NamedNode(s) = &t.subject {
            if s.as_str().starts_with(SEC_REQ_NS) {
                assert_ne!(
                    t.predicate.as_str(),
                    RDF_TYPE,
                    "trust-framework.ttl redeclares (types) the vendored `{}` — the D5 \
                     no-duplication rule forbids editing/duplicating vendored sec-req:",
                    s.as_str(),
                );
            }
        }
    }
}
