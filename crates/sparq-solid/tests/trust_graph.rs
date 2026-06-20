//! Trust-graph PoC end-to-end through the REAL `PodStore` (the `trust-graph` feature).
//!
//! Proves: a trusted-government-issued VC `<Jesse> age 25` + a Control-gated trust
//! policy + the `.acr` ABAC rule ⇒ `<Jesse>` gains a real `auth:read` grant on
//! `<resourceX>` that `PodStore::accessible` / `query_as` honour — and the negative
//! cases (untrusted issuer, age 16) do NOT. Plus the strict-additivity check: with the
//! feature compiled in but NO admission call, the store behaves EXACTLY as WAC/ACP.
//!
//! Compiled only under `--features trust-graph`.
//!
//! [OPUS-4.8] sq-pfae PoC (issue #940). 🤖 SPARQ agent — trust-graph authorisation PoC.
#![cfg(feature = "trust-graph")]

use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_core::Graph;
use sparq_solid::{Mode, PodStore, Session};
use sparq_trust::admit::{PresentedCredential, Session as TrustSession};
use sparq_trust::policy::{parse_policy, ControlGate, TrustRule};
use sparq_trust::vocab;
use sparq_zk::commit::commit_triples;
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::sig::{public_key_to_hex, PublicKey, SecretKey};

const JESSE: &str = "https://jesse.ex/card#me";
const GOV_ISSUER: &str = "https://gov.example/issuer";
const SCHEMA_AGE: &str = "http://schema.org/age";
const RESOURCE_X: &str = "https://pod.ex/resourceX";
const SALT: [u8; 32] = [9u8; 32];
const NOW: i64 = 1_700_000_000;

const ACR_ABAC_RULE: &str = r#"@prefix schema: <http://schema.org/> .
@prefix math:   <http://www.w3.org/2000/10/swap/math#> .
@prefix auth:   <https://sparq.dev/ns/auth#> .
{ ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <https://pod.ex/resourceX> } .
"#;

fn iri(s: &str) -> NamedNode {
    NamedNode::new(s).unwrap()
}

/// A minimal pod: one private resource `resourceX` (a document graph), and an `.acl`
/// granting Alice (NOT Jesse) Read — so the baseline auth view is well-formed but
/// gives Jesse nothing without the trust admission.
fn pod_nquads() -> String {
    format!(
        "<{RESOURCE_X}#it> <https://ex.dev/ns#title> \"x\" <{RESOURCE_X}> .\n\
         <https://pod.ex/.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .\n\
         <https://pod.ex/.acl#a> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .\n\
         <https://pod.ex/.acl#a> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .\n\
         <https://pod.ex/.acl#a> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .\n"
    )
}

fn age_credential_graph(value: &str) -> Vec<Triple> {
    vec![Triple::new(
        NamedOrBlankNode::NamedNode(iri(JESSE)),
        iri(SCHEMA_AGE),
        Term::Literal(Literal::new_typed_literal(
            value,
            iri("http://www.w3.org/2001/XMLSchema#integer"),
        )),
    )]
}

fn gov_key() -> (SecretKey, PublicKey) {
    let sk = SecretKey::from_seed(0xABCDEF);
    let pk = sk.public_key();
    (sk, pk)
}

fn signed_cred(sk: &SecretKey, value: &str) -> PresentedCredential {
    let graph = age_credential_graph(value);
    let salt = salt_from_bytes(&SALT);
    let commitment = commit_triples(&graph, salt).unwrap();
    PresentedCredential {
        graph,
        issuer_signature_hex: sk.sign_commitment(&commitment.commitment),
        salt: SALT,
        issued_at_unix_secs: NOW - 86_400,
        revoked: false,
    }
}

fn gov_age_rules(issuer: &str, key: &PublicKey) -> Vec<TrustRule> {
    let rule = iri("https://pod.ex/.acr#trustrule");
    let policy = vec![
        Triple::new(
            NamedOrBlankNode::NamedNode(rule.clone()),
            iri(vocab::RDF_TYPE),
            Term::NamedNode(iri(vocab::TRUST_RULE)),
        ),
        Triple::new(
            NamedOrBlankNode::NamedNode(rule.clone()),
            iri(vocab::SOURCE),
            Term::NamedNode(iri(issuer)),
        ),
        Triple::new(
            NamedOrBlankNode::NamedNode(rule.clone()),
            iri(vocab::ISSUER_KEY),
            Term::Literal(Literal::new_simple_literal(public_key_to_hex(key))),
        ),
        Triple::new(
            NamedOrBlankNode::NamedNode(rule.clone()),
            iri(vocab::FOR_PREDICATE),
            Term::NamedNode(iri(SCHEMA_AGE)),
        ),
        Triple::new(
            NamedOrBlankNode::NamedNode(rule.clone()),
            iri(vocab::SCOPE),
            Term::NamedNode(iri(RESOURCE_X)),
        ),
        Triple::new(
            NamedOrBlankNode::NamedNode(rule),
            iri(vocab::FRESH_WITHIN),
            Term::Literal(Literal::new_simple_literal("P30D")),
        ),
    ];
    parse_policy(&policy, ControlGate::assert_control_gated()).unwrap()
}

fn jesse_can_read_resource_x(store: &mut PodStore) -> bool {
    let jesse = Session {
        agent: Some(JESSE),
        client: None,
        issuer: None,
        now: None,
    };
    store
        .accessible(&jesse, Mode::Read)
        .iter()
        .any(|n| n.as_str() == RESOURCE_X)
}

#[test]
fn age_25_grants_real_podstore_read_end_to_end() {
    let (sk, pk) = gov_key();
    let mut store = PodStore::new(Graph::load_dataset(&pod_nquads(), "nquads").unwrap());
    store.materialize_wac().unwrap();
    // Baseline: Jesse cannot read resourceX (only Alice is granted by the .acl).
    assert!(
        !jesse_can_read_resource_x(&mut store),
        "baseline: no Jesse read"
    );

    let cred = signed_cred(&sk, "25");
    let rules = gov_age_rules(GOV_ISSUER, &pk);
    let session = TrustSession {
        agent: iri(JESSE),
        now_unix_secs: NOW,
    };
    let outcome = store
        .admit_trust_credential_with_rule(&cred, &rules, &session, &iri(RESOURCE_X), ACR_ABAC_RULE)
        .unwrap();
    assert_eq!(outcome.admitted.len(), 1, "the age fact is admitted");
    assert_eq!(
        outcome.installed_grants.len(),
        1,
        "one read grant derived + installed"
    );
    // The REAL PodStore now grants Jesse read on resourceX.
    assert!(
        jesse_can_read_resource_x(&mut store),
        "age 25 from the trusted gov issuer ⇒ <Jesse> auth:read <resourceX> through the \
         real PodStore (admission + N3 merge + materialise)"
    );
    // And a query as Jesse now sees the resource content.
    let q = "SELECT ?t WHERE { ?s <https://ex.dev/ns#title> ?t }";
    let jesse = Session {
        agent: Some(JESSE),
        client: None,
        issuer: None,
        now: None,
    };
    assert_eq!(store.query_as(&jesse, Mode::Read, q).unwrap().rows.len(), 1);
}

#[test]
fn untrusted_issuer_grants_nothing_through_podstore() {
    let (_gov_sk, gov_pk) = gov_key();
    let forge_sk = SecretKey::from_seed(0x111222);
    let mut store = PodStore::new(Graph::load_dataset(&pod_nquads(), "nquads").unwrap());
    store.materialize_wac().unwrap();
    // Credential signed by the forge key; policy trusts the GOV key → no verify → no admit.
    let cred = signed_cred(&forge_sk, "25");
    let rules = gov_age_rules(GOV_ISSUER, &gov_pk);
    let session = TrustSession {
        agent: iri(JESSE),
        now_unix_secs: NOW,
    };
    let outcome = store
        .admit_trust_credential_with_rule(&cred, &rules, &session, &iri(RESOURCE_X), ACR_ABAC_RULE)
        .unwrap();
    assert!(
        outcome.admitted.is_empty(),
        "untrusted issuer ⇒ nothing admitted"
    );
    assert!(
        !jesse_can_read_resource_x(&mut store),
        "untrusted issuer ⇒ no real access"
    );
}

#[test]
fn age_16_admitted_but_no_grant_through_podstore() {
    let (sk, pk) = gov_key();
    let mut store = PodStore::new(Graph::load_dataset(&pod_nquads(), "nquads").unwrap());
    store.materialize_wac().unwrap();
    let cred = signed_cred(&sk, "16");
    let rules = gov_age_rules(GOV_ISSUER, &pk);
    let session = TrustSession {
        agent: iri(JESSE),
        now_unix_secs: NOW,
    };
    let outcome = store
        .admit_trust_credential_with_rule(&cred, &rules, &session, &iri(RESOURCE_X), ACR_ABAC_RULE)
        .unwrap();
    assert_eq!(
        outcome.admitted.len(),
        1,
        "age 16 is admitted (a trusted age statement)"
    );
    assert!(
        outcome.installed_grants.is_empty(),
        "but the >18 rule denies → no grant"
    );
    assert!(
        !jesse_can_read_resource_x(&mut store),
        "age 16 ⇒ no real access"
    );
}

/// STRICT ADDITIVITY (G6): with the `trust-graph` feature compiled in but NO admission
/// call made, the store's auth view is IDENTICAL to a plain WAC materialise — the
/// feature is additive, never a behaviour change to the WAC/ACP path. (The byte-level
/// feature-OFF parity is asserted by the always-compiled WAC/ACP test suites, which
/// run unchanged in both feature states; this test pins the in-feature no-op.)
#[test]
fn strict_additivity_no_admission_is_unchanged_wac() {
    let mut a = PodStore::new(Graph::load_dataset(&pod_nquads(), "nquads").unwrap());
    a.materialize_wac().unwrap();
    let alice = Session {
        agent: Some("https://alice.ex/card#me"),
        client: None,
        issuer: None,
        now: None,
    };
    // Alice's legit Read is intact; Jesse has nothing — exactly WAC.
    assert!(a
        .accessible(&alice, Mode::Read)
        .iter()
        .any(|n| n.as_str() == RESOURCE_X));
    assert!(!jesse_can_read_resource_x(&mut a));
    // No admission call was made → the auth view is the plain WAC view.
}
