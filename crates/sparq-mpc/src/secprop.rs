//! # `secprop` — the per-protocol security-properties annotation graph (Phase 6)
//!
//! The **machine-readable, honest encoding of "sparq-mpc is semi-honest only"** —
//! the §5c analogue of the sparq-zk per-method annotation graph (Phase 3,
//! `crates/sparq-zk/src/secprop.rs`). A **static annotation graph**
//! (`ontologies/secprop-mpc.ttl`) keyed on the minted `mpc:` protocol IRIs,
//! expressed with the `secx:` extension vocabulary (design record
//! `research/security-properties-ontology-design.md` §5c). Each protocol's
//! `secx:hasProperty` is a reified [`PropertyAssertion`]: a `(property, level,
//! assurance, audit-status, assumptions)` claim.
//!
//! ## The load-bearing encoding — the assumption axis
//!
//! The whole point (design §5c, §7.5) is that the honest-majority / semi-honest
//! preconditions live **in the data**, so a preference expressed against the
//! `secx:assumption` axis mechanically excludes the matching protocols rather
//! than relying on prose:
//!
//! - **Every** sparq-mpc protocol carries [`SECX_HONEST_MAJORITY`] — v1 is
//!   honest-majority by design. A preference requiring **dishonest-majority**
//!   tolerance forbids that assumption and excludes the whole estate
//!   ([`excluded_by_forbidding`]).
//! - The **semi-honest** protocols additionally carry [`SECX_SEMI_HONEST`]; the
//!   **malicious-with-abort** twins ([`crate::auth_compare`],
//!   [`crate::auth_disclose`]) do **not**. A preference requiring
//!   **malicious-security** forbids [`SECX_SEMI_HONEST`] and excludes the
//!   semi-honest set while admitting the malicious twins.
//!
//! ## What this records — and what it is NOT
//!
//! It **records** a (protocol, property) → (level, assurance, audit-status,
//! assumptions) claim and its **epistemic basis**; it is **NOT** a proof of any
//! property. sparq-mpc's cryptography is research-grade, **externally UNAUDITED**
//! (`sq-qhy4`; CR-G1/CR-G4 of `compliance/cryptoreview/gap-register.md`) and
//! **not FIPS-validated**. So every **positive** soundness claim is
//! [`Assurance::Claimed`] + `secx:ExternalSignOffPending`, never
//! [`Assurance::Proven`] — the same §5a honesty rule the sparq-zk graph follows,
//! enforced here by [`audit_overclaim_violations`].
//!
//! ## The guards (design §5c, mirroring §5a)
//!
//! 1. **No over-claim on assurance** ([`audit_overclaim_violations`]): no
//!    protocol may carry [`Assurance::Proven`] on a positive property while
//!    `sq-qhy4` is open.
//! 2. **Completeness** ([`completeness_violations`]): every declared
//!    [`MPC_PROTOCOL_IRIS`] protocol has an annotation block.
//! 3. **Honest-majority coverage** ([`honest_majority_violations`]): every
//!    protocol carries the [`SECX_HONEST_MAJORITY`] assumption — the whole v1
//!    estate is honest-majority.
//!
//! And the annotation is **pinned to the crate's own `SecurityDescriptor`** by
//! the `semi_honest_annotation_matches_backend_descriptor` test, so the
//! `secx:SemiHonest` set cannot silently drift from
//! [`crate::shamir::ShamirBackend::operator_descriptor`].
//!
//! ## Opt-in by construction
//!
//! Behind the **default-OFF `secprop-annotations`** cargo feature, so it adds
//! nothing to the lean default build. The `secx:` IRIs are declared locally and
//! pinned to the canonical published `secprop-ext.ttl` (owned by `sparq-trust`)
//! by the `local_secx_iris_are_declared_in_the_canonical_vocab` drift test — an
//! `include_str!` compile-time read, **not** a crate edge.
//!
//! [OPUS-4.8] sq-dz10l (Phase 6; epic sq-0dksu; design record
//! `research/security-properties-ontology-design.md` §5c). 🤖 SPARQ agent —
//! security-properties ontology. Flag for re-review when Fable returns.

use oxrdf::{NamedOrBlankNode, Term};
use oxttl::TurtleParser;
use std::collections::{BTreeMap, BTreeSet};

/// The canonical machine-readable per-protocol annotation graph (the source of
/// truth this module's accessors and guards parse).
const MPC_TTL: &str = include_str!("../ontologies/secprop-mpc.ttl");

// ── the `secx:` IRIs this module references (pinned to the canonical vocab) ───
// `sparq-trust::secprop` owns these constants; we declare them locally (no crate
// edge) and the `local_secx_iris_are_declared_in_the_canonical_vocab` test pins
// every one to the canonical `secprop-ext.ttl` so they cannot drift.

/// The `sec-prop:` namespace base the `secx:` IRIs share.
pub const SEC_PROP_NS: &str = "https://w3id.org/zkp-sparql/sec-prop#";

const SECX_HAS_PROPERTY: &str = "https://w3id.org/zkp-sparql/sec-prop#hasProperty";
const SECX_PROPERTY: &str = "https://w3id.org/zkp-sparql/sec-prop#property";
const SECX_LEVEL: &str = "https://w3id.org/zkp-sparql/sec-prop#level";
const SECX_ASSURANCE: &str = "https://w3id.org/zkp-sparql/sec-prop#assurance";
const SECX_AUDIT_STATUS: &str = "https://w3id.org/zkp-sparql/sec-prop#auditStatus";
const SECX_ASSUMPTION: &str = "https://w3id.org/zkp-sparql/sec-prop#assumption";

const SECX_PROVEN: &str = "https://w3id.org/zkp-sparql/sec-prop#Proven";
const SECX_CLAIMED: &str = "https://w3id.org/zkp-sparql/sec-prop#Claimed";
const SECX_CONJECTURED: &str = "https://w3id.org/zkp-sparql/sec-prop#Conjectured";

/// `secx:ExternalSignOffPending` — the live audit status of every positive
/// sparq-mpc soundness claim while `sq-qhy4` is open.
pub const SECX_EXTERNAL_SIGN_OFF_PENDING: &str =
    "https://w3id.org/zkp-sparql/sec-prop#ExternalSignOffPending";

/// `secx:HonestMajority` — the MPC assumption **every** sparq-mpc protocol rests
/// on (v1 is honest-majority). Forbidding it (a dishonest-majority preference)
/// excludes the whole estate.
pub const SECX_HONEST_MAJORITY: &str = "https://w3id.org/zkp-sparql/sec-prop#HonestMajority";

/// `secx:SemiHonest` — the MPC assumption the **semi-honest** protocols carry and
/// the malicious-with-abort twins do not. Forbidding it (a malicious-security
/// preference) excludes exactly the semi-honest set.
pub const SECX_SEMI_HONEST: &str = "https://w3id.org/zkp-sparql/sec-prop#SemiHonest";

/// The complete set of `secx:` IRIs this module declares locally, pinned to the
/// canonical published `secprop-ext.ttl` by the drift test.
pub const LOCAL_SECX_IRIS: &[&str] = &[
    SECX_HAS_PROPERTY,
    SECX_PROPERTY,
    SECX_LEVEL,
    SECX_ASSURANCE,
    SECX_AUDIT_STATUS,
    SECX_ASSUMPTION,
    SECX_PROVEN,
    SECX_CLAIMED,
    SECX_CONJECTURED,
    SECX_EXTERNAL_SIGN_OFF_PENDING,
    SECX_HONEST_MAJORITY,
    SECX_SEMI_HONEST,
];

// ── the minted `mpc:` protocol IRIs (the annotation subjects) ─────────────────

/// The secure cumulative-sum aggregate (SUM/COUNT). Semi-honest.
pub const MPC_SECURE_AGGREGATE_V1: &str = "https://sparq.dev/ns/mpc#secure-aggregate-v1";
/// The hidden-value equi-join over a private key. Semi-honest.
pub const MPC_HIDDEN_VALUE_JOIN_V1: &str = "https://sparq.dev/ns/mpc#hidden-value-join-v1";
/// The secret-shared greater-than / threshold (verdict-only). Semi-honest.
pub const MPC_SECURE_COMPARISON_V1: &str = "https://sparq.dev/ns/mpc#secure-comparison-v1";
/// The hidden-intermediate exactly-`k` property-path chain. Semi-honest.
pub const MPC_HIDDEN_PROPERTY_PATH_V1: &str = "https://sparq.dev/ns/mpc#hidden-property-path-v1";
/// The malicious-with-abort secure comparison ([`crate::auth_compare`]).
pub const MPC_AUTH_COMPARISON_V1: &str = "https://sparq.dev/ns/mpc#auth-comparison-v1";
/// The malicious-with-abort threshold disclosure ([`crate::auth_disclose`]).
pub const MPC_AUTH_DISCLOSE_V1: &str = "https://sparq.dev/ns/mpc#auth-disclose-v1";

/// Every sparq-mpc protocol IRI that MUST carry an annotation block (the
/// completeness set). Kept in one place so the graph and the guards agree.
pub const MPC_PROTOCOL_IRIS: &[&str] = &[
    MPC_SECURE_AGGREGATE_V1,
    MPC_HIDDEN_VALUE_JOIN_V1,
    MPC_SECURE_COMPARISON_V1,
    MPC_HIDDEN_PROPERTY_PATH_V1,
    MPC_AUTH_COMPARISON_V1,
    MPC_AUTH_DISCLOSE_V1,
];

/// The **semi-honest** protocols — those that carry `secx:SemiHonest`. Their
/// adversary model is pinned to the crate's own `SecurityDescriptor` by
/// `semi_honest_annotation_matches_backend_descriptor`.
pub const SEMI_HONEST_PROTOCOL_IRIS: &[&str] = &[
    MPC_SECURE_AGGREGATE_V1,
    MPC_HIDDEN_VALUE_JOIN_V1,
    MPC_SECURE_COMPARISON_V1,
    MPC_HIDDEN_PROPERTY_PATH_V1,
];

/// The **malicious-with-abort** protocols — honest-majority but NOT semi-honest.
pub const MALICIOUS_PROTOCOL_IRIS: &[&str] = &[MPC_AUTH_COMPARISON_V1, MPC_AUTH_DISCLOSE_V1];

// ── typed model ──────────────────────────────────────────────────────────────

/// The epistemic-basis axis of a [`PropertyAssertion`] — `Proven ⊐ Claimed ⊐
/// Conjectured`. A positive sparq-mpc property may only ever be
/// [`Self::Claimed`] (or weaker) while `sq-qhy4` is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Assurance {
    /// Backed by a formal proof or an external audit. Not applicable to a
    /// positive sparq-mpc property while `sq-qhy4` is open.
    Proven,
    /// Asserted, not independently verified. The default for a sparq-mpc claim.
    Claimed,
    /// Believed plausible but not established to the `Claimed` bar.
    Conjectured,
}

impl Assurance {
    fn from_iri(iri: &str) -> Option<Self> {
        match iri {
            SECX_PROVEN => Some(Assurance::Proven),
            SECX_CLAIMED => Some(Assurance::Claimed),
            SECX_CONJECTURED => Some(Assurance::Conjectured),
            _ => None,
        }
    }
}

/// A reified `(property, level, assurance, audit-status, assumptions)` claim
/// about one protocol IRI — a parsed `secx:PropertyAssertion`. Unlike the
/// single-assumption sparq-zk assertion, an MPC claim can rest on **several**
/// assumptions (e.g. `HonestMajority` AND `SemiHonest`), so [`assumptions`] is a
/// set.
///
/// [`assumptions`]: PropertyAssertion::assumptions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyAssertion {
    /// The dimension the assertion is about (a `secx:` property IRI, e.g.
    /// `secx:Soundness`).
    pub property: String,
    /// The level held within the dimension (a `secx:` level IRI, e.g.
    /// `secx:Sound`), if set.
    pub level: Option<String>,
    /// The epistemic basis of the claim.
    pub assurance: Assurance,
    /// The independent-review state IRI (e.g. `secx:ExternalSignOffPending`).
    pub audit_status: Option<String>,
    /// The assumptions the claim rests on (`secx:` assumption IRIs), sorted.
    pub assumptions: BTreeSet<String>,
}

/// All [`PropertyAssertion`]s attached to one protocol IRI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolAnnotations {
    /// The annotated protocol IRI (an `mpc:` IRI).
    pub protocol: String,
    /// Its property assertions, in a stable (property, level) order.
    pub assertions: Vec<PropertyAssertion>,
}

impl ProtocolAnnotations {
    /// The union of every assumption IRI this protocol's assertions rest on.
    pub fn assumptions(&self) -> BTreeSet<&str> {
        self.assertions
            .iter()
            .flat_map(|a| a.assumptions.iter().map(String::as_str))
            .collect()
    }

    /// Whether this protocol rests on `assumption` (any assertion carries it).
    pub fn requires_assumption(&self, assumption: &str) -> bool {
        self.assertions
            .iter()
            .any(|a| a.assumptions.iter().any(|x| x == assumption))
    }

    /// Whether a preference that **forbids** any of `forbidden` assumptions
    /// EXCLUDES this protocol — the honest, mechanical encoding of §5c: a
    /// protocol is excluded iff it rests on a forbidden assumption. Forbidding
    /// [`SECX_SEMI_HONEST`] (a malicious-security preference) excludes the
    /// semi-honest set; forbidding [`SECX_HONEST_MAJORITY`] (a
    /// dishonest-majority preference) excludes the whole estate.
    pub fn excluded_by_forbidding(&self, forbidden: &[&str]) -> bool {
        forbidden.iter().any(|f| self.requires_assumption(f))
    }
}

/// An over-claim / coverage guard violation (a parse-stable, human-readable
/// record, never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The protocol IRI the violation is on.
    pub protocol: String,
    /// The machine-readable reason kind.
    pub kind: ViolationKind,
    /// A one-line human-readable explanation.
    pub detail: String,
}

/// The kind of a [`Violation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationKind {
    /// A positive property carries `secx:Proven` while `sq-qhy4` is open.
    ProvenPositiveWhileAuditOpen,
    /// A declared protocol has no annotation block.
    MissingAnnotation,
    /// A protocol does not carry the `secx:HonestMajority` assumption.
    MissingHonestMajority,
}

// ── parsing ──────────────────────────────────────────────────────────────────

/// Parse the static annotation graph into one [`ProtocolAnnotations`] per
/// protocol IRI, keyed by IRI in a stable order. Pure over the bundled Turtle —
/// no I/O, never panics on the well-formed bundled file (a compiled-in constant
/// a test asserts parses).
pub fn parse_annotations() -> BTreeMap<String, ProtocolAnnotations> {
    // blank-node id -> its (predicate IRI, object) pairs.
    let mut bnode_pairs: BTreeMap<String, Vec<(String, Term)>> = BTreeMap::new();
    // protocol IRI -> the blank-node ids of its assertion nodes.
    let mut protocol_assertion_bnodes: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for result in TurtleParser::new().for_reader(MPC_TTL.as_bytes()) {
        let t = result.expect("bundled secprop-mpc.ttl must be valid Turtle");
        let pred = t.predicate.into_string();

        match &t.subject {
            NamedOrBlankNode::NamedNode(s) => {
                if pred == SECX_HAS_PROPERTY {
                    if let Term::BlankNode(b) = &t.object {
                        protocol_assertion_bnodes
                            .entry(s.as_str().to_owned())
                            .or_default()
                            .push(b.as_str().to_owned());
                    }
                }
            }
            NamedOrBlankNode::BlankNode(b) => {
                bnode_pairs
                    .entry(b.as_str().to_owned())
                    .or_default()
                    .push((pred, t.object.clone()));
            }
        }
    }

    let mut out: BTreeMap<String, ProtocolAnnotations> = BTreeMap::new();
    for (protocol, bnodes) in protocol_assertion_bnodes {
        let mut assertions = Vec::new();
        for b in bnodes {
            if let Some(pairs) = bnode_pairs.get(&b) {
                if let Some(a) = assertion_from_pairs(pairs) {
                    assertions.push(a);
                }
            }
        }
        // Stable order: by (property, level) so output is deterministic.
        assertions.sort_by(|x, y| {
            (x.property.as_str(), x.level.as_deref())
                .cmp(&(y.property.as_str(), y.level.as_deref()))
        });
        out.insert(
            protocol.clone(),
            ProtocolAnnotations {
                protocol,
                assertions,
            },
        );
    }
    out
}

/// Build a [`PropertyAssertion`] from the predicate-object pairs of one blank
/// node. Returns `None` if there is no `secx:property` (not an assertion node) or
/// the assurance is missing/unknown (fail-closed: an annotation must state its
/// basis). Collects **every** `secx:assumption` (an MPC claim rests on more than
/// one).
fn assertion_from_pairs(pairs: &[(String, Term)]) -> Option<PropertyAssertion> {
    let iri_obj = |p: &str| -> Option<String> {
        pairs.iter().find_map(|(pred, obj)| {
            if pred == p {
                if let Term::NamedNode(n) = obj {
                    return Some(n.as_str().to_owned());
                }
            }
            None
        })
    };
    let iri_objs = |p: &str| -> BTreeSet<String> {
        pairs
            .iter()
            .filter_map(|(pred, obj)| {
                if pred == p {
                    if let Term::NamedNode(n) = obj {
                        return Some(n.as_str().to_owned());
                    }
                }
                None
            })
            .collect()
    };

    let property = iri_obj(SECX_PROPERTY)?;
    let assurance = Assurance::from_iri(&iri_obj(SECX_ASSURANCE)?)?;

    Some(PropertyAssertion {
        property,
        level: iri_obj(SECX_LEVEL),
        assurance,
        audit_status: iri_obj(SECX_AUDIT_STATUS),
        assumptions: iri_objs(SECX_ASSUMPTION),
    })
}

// ── guards (design §5c, mirroring §5a) ───────────────────────────────────────

/// Whether the external audit `sq-qhy4` is still open. The annotation graph and
/// the guards are written for the open state; this is the single switch a future
/// assurance-promotion bead flips. **Hard-coded `true`** — only an external
/// accredited-cryptographer sign-off (out of agent scope) may flip it.
pub const fn audit_qhy4_open() -> bool {
    true
}

/// **Guard 1.** Every protocol where a property carries [`Assurance::Proven`]
/// while `sq-qhy4` is open. An empty result = no over-claim. Every sparq-mpc
/// assertion is a **positive** soundness claim (unlike the sparq-zk graph, this
/// graph has no settled-negative levels), so `Proven` is never legitimate here
/// while the external audit is open.
pub fn audit_overclaim_violations(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
) -> Vec<Violation> {
    if !audit_qhy4_open() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (protocol, ann) in annotations {
        for a in &ann.assertions {
            if a.assurance == Assurance::Proven {
                out.push(Violation {
                    protocol: protocol.clone(),
                    kind: ViolationKind::ProvenPositiveWhileAuditOpen,
                    detail: format!(
                        "positive property {} (level {}) is Proven while sq-qhy4 is open",
                        a.property,
                        a.level.as_deref().unwrap_or("<none>"),
                    ),
                });
            }
        }
    }
    out
}

/// **Guard 2.** Every declared [`MPC_PROTOCOL_IRIS`] protocol that has no
/// annotation block. An empty result = complete coverage.
pub fn completeness_violations(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
    required_protocol_iris: &[&str],
) -> Vec<Violation> {
    required_protocol_iris
        .iter()
        .filter(|iri| !annotations.contains_key(**iri))
        .map(|iri| Violation {
            protocol: (*iri).to_owned(),
            kind: ViolationKind::MissingAnnotation,
            detail: format!("declared protocol {} has no annotation block", iri),
        })
        .collect()
}

/// **Guard 3.** Every protocol that does NOT rest on the [`SECX_HONEST_MAJORITY`]
/// assumption — the whole v1 estate is honest-majority, so an empty result is the
/// invariant. Catches an annotation that silently drops the honest-majority
/// precondition (which would let a dishonest-majority preference wrongly admit
/// it).
pub fn honest_majority_violations(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
) -> Vec<Violation> {
    annotations
        .values()
        .filter(|ann| !ann.requires_assumption(SECX_HONEST_MAJORITY))
        .map(|ann| Violation {
            protocol: ann.protocol.clone(),
            kind: ViolationKind::MissingHonestMajority,
            detail: format!(
                "{} does not carry the secx:HonestMajority assumption (every sparq-mpc \
                 protocol is honest-majority)",
                ann.protocol,
            ),
        })
        .collect()
}

/// The set of protocol IRIs a preference that **forbids** `forbidden`
/// assumptions excludes — the mechanical realisation of §5c. A convenience over
/// [`ProtocolAnnotations::excluded_by_forbidding`] across the whole graph.
pub fn excluded_by_forbidding<'a>(
    annotations: &'a BTreeMap<String, ProtocolAnnotations>,
    forbidden: &[&str],
) -> BTreeSet<&'a str> {
    annotations
        .values()
        .filter(|ann| ann.excluded_by_forbidding(forbidden))
        .map(|ann| ann.protocol.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{AdversaryModel, OperatorClass};
    use crate::shamir::ShamirBackend;

    const VOCAB_TTL: &str =
        include_str!("../../sparq-trust/ontologies/zkp-sparql/secprop-ext.ttl");

    /// The bundled annotation graph is valid Turtle and non-empty.
    #[test]
    fn mpc_ttl_is_valid_turtle() {
        let mut triples = 0usize;
        for result in TurtleParser::new().for_reader(MPC_TTL.as_bytes()) {
            result.expect("secprop-mpc.ttl must be valid Turtle");
            triples += 1;
        }
        assert!(triples > 0, "secprop-mpc.ttl parsed to zero triples");
    }

    /// Every `secx:` IRI this module declares locally is in the `sec-prop:`
    /// namespace AND is declared in the canonical published vocab — the drift
    /// guard (an `include_str!` compile-time read, not a crate edge).
    #[test]
    fn local_secx_iris_are_declared_in_the_canonical_vocab() {
        for &iri in LOCAL_SECX_IRIS {
            assert!(
                iri.starts_with(SEC_PROP_NS),
                "local constant {} is not in the sec-prop: namespace",
                iri,
            );
            let local = iri.strip_prefix(SEC_PROP_NS).unwrap();
            let decl = format!("secx:{} ", local);
            assert!(
                VOCAB_TTL.contains(&decl),
                "local secx: constant secx:{} is not declared in the canonical \
                 secprop-ext.ttl — the annotation graph drifted from the vocabulary",
                local,
            );
        }
    }

    /// The graph parses to every declared protocol, each with at least one
    /// assertion.
    #[test]
    fn parses_each_protocol_with_assertions() {
        let ann = parse_annotations();
        for iri in MPC_PROTOCOL_IRIS {
            let p = ann
                .get(*iri)
                .unwrap_or_else(|| panic!("no annotation block for {}", iri));
            assert!(!p.assertions.is_empty(), "{} has an empty assertion set", iri);
        }
    }

    /// **Guard 2 holds**: every declared protocol is annotated.
    #[test]
    fn completeness_guard_is_satisfied() {
        let ann = parse_annotations();
        let v = completeness_violations(&ann, MPC_PROTOCOL_IRIS);
        assert!(v.is_empty(), "completeness violations: {:?}", v);
    }

    /// **Guard 1 holds**: no positive property is `Proven` while sq-qhy4 is open
    /// (the whole estate is unaudited, so every claim is `Claimed`).
    #[test]
    fn no_proven_positive_property_while_audit_open() {
        assert!(audit_qhy4_open(), "the guard is written for the open state");
        let ann = parse_annotations();
        let v = audit_overclaim_violations(&ann);
        assert!(v.is_empty(), "over-claim violations: {:?}", v);
    }

    /// Every positive assertion is `Claimed` + `ExternalSignOffPending` — the
    /// honesty pairing (an unaudited claim must say so).
    #[test]
    fn claimed_positives_are_external_sign_off_pending() {
        let ann = parse_annotations();
        for p in ann.values() {
            for a in &p.assertions {
                assert_eq!(a.assurance, Assurance::Claimed, "{} is not Claimed", p.protocol);
                assert_eq!(
                    a.audit_status.as_deref(),
                    Some(SECX_EXTERNAL_SIGN_OFF_PENDING),
                    "{} is Claimed but not ExternalSignOffPending",
                    p.protocol,
                );
            }
        }
    }

    /// **Guard 3 holds**: every protocol carries the `secx:HonestMajority`
    /// assumption — the whole v1 estate is honest-majority.
    #[test]
    fn every_protocol_is_honest_majority() {
        let ann = parse_annotations();
        let v = honest_majority_violations(&ann);
        assert!(v.is_empty(), "honest-majority violations: {:?}", v);
    }

    /// The semi-honest protocols carry `secx:SemiHonest`; the malicious-with-abort
    /// twins deliberately do NOT — the honest binary the assumption axis encodes.
    #[test]
    fn semi_honest_set_carries_semi_honest_and_malicious_set_does_not() {
        let ann = parse_annotations();
        for iri in SEMI_HONEST_PROTOCOL_IRIS {
            assert!(
                ann[*iri].requires_assumption(SECX_SEMI_HONEST),
                "{} must carry secx:SemiHonest",
                iri,
            );
        }
        for iri in MALICIOUS_PROTOCOL_IRIS {
            assert!(
                !ann[*iri].requires_assumption(SECX_SEMI_HONEST),
                "{} is malicious-with-abort and must NOT carry secx:SemiHonest",
                iri,
            );
        }
    }

    /// **The §5c mechanical-exclusion property.** A preference requiring
    /// MALICIOUS-SECURITY (forbid `secx:SemiHonest`) excludes exactly the
    /// semi-honest set and ADMITS the malicious twins; a preference requiring
    /// DISHONEST-MAJORITY tolerance (forbid `secx:HonestMajority`) excludes the
    /// WHOLE estate.
    #[test]
    fn preference_mechanically_excludes_by_assumption() {
        let ann = parse_annotations();

        // "requires malicious-security" ⇒ forbid SemiHonest.
        let excluded_by_malicious_req = excluded_by_forbidding(&ann, &[SECX_SEMI_HONEST]);
        let expected_semi: BTreeSet<&str> = SEMI_HONEST_PROTOCOL_IRIS.iter().copied().collect();
        assert_eq!(
            excluded_by_malicious_req, expected_semi,
            "a malicious-security preference must exclude exactly the semi-honest set",
        );
        for iri in MALICIOUS_PROTOCOL_IRIS {
            assert!(
                !excluded_by_malicious_req.contains(*iri),
                "the malicious twin {} must survive a malicious-security preference",
                iri,
            );
        }

        // "requires dishonest-majority tolerance" ⇒ forbid HonestMajority.
        let excluded_by_dishonest_req = excluded_by_forbidding(&ann, &[SECX_HONEST_MAJORITY]);
        let all: BTreeSet<&str> = MPC_PROTOCOL_IRIS.iter().copied().collect();
        assert_eq!(
            excluded_by_dishonest_req, all,
            "a dishonest-majority preference must exclude the whole sparq-mpc estate",
        );
    }

    /// **The honesty keystone — pin the `secx:SemiHonest` annotation to the
    /// crate's OWN `SecurityDescriptor`.** For each semi-honest protocol that
    /// maps to an [`OperatorClass`], the Shamir backend's per-operator descriptor
    /// must actually report `AdversaryModel::SemiHonest`. This makes the graph a
    /// read of the code, not an independent claim that could silently over-claim
    /// malicious security. `n = 3` (`t = 1`, the honest-majority `n = 2t+1`) is
    /// the worst case: the degree-`2t` equality open has zero RS redundancy there.
    #[test]
    fn semi_honest_annotation_matches_backend_descriptor() {
        let ann = parse_annotations();
        let backend = ShamirBackend::new(3).expect("n=3 honest-majority backend");
        let mapped = [
            (MPC_SECURE_AGGREGATE_V1, OperatorClass::LinearAggregate),
            (MPC_HIDDEN_VALUE_JOIN_V1, OperatorClass::EqualityJoin),
            (MPC_SECURE_COMPARISON_V1, OperatorClass::Comparison),
        ];
        for (iri, op) in mapped {
            let adversary = backend.operator_descriptor(op).adversary;
            assert_eq!(
                adversary,
                AdversaryModel::SemiHonest,
                "{:?} descriptor is not SemiHonest — the {} annotation would over-claim",
                op,
                iri,
            );
            assert!(
                ann[iri].requires_assumption(SECX_SEMI_HONEST),
                "{} maps to a SemiHonest operator but does not carry secx:SemiHonest",
                iri,
            );
        }
    }

    /// A claim can rest on more than one assumption — the parser collects every
    /// `secx:assumption` (unlike the single-assumption sparq-zk parser).
    #[test]
    fn semi_honest_claims_carry_both_assumptions() {
        let ann = parse_annotations();
        let a = &ann[MPC_SECURE_AGGREGATE_V1];
        let assumptions = a.assumptions();
        assert!(assumptions.contains(SECX_HONEST_MAJORITY));
        assert!(assumptions.contains(SECX_SEMI_HONEST));
        assert_eq!(assumptions.len(), 2, "expected exactly the two MPC assumptions");
    }
}
