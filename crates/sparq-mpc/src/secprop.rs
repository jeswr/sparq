//! # `secprop` — the per-protocol security-properties annotation graph (Phase 6)
//!
//! The **machine-readable form** of this crate's per-operator security posture. A
//! **static annotation graph** ([`ontologies/secprop-protocols.ttl`]) keyed on the
//! `mpc:` protocol IRIs — one per [`OperatorClass`], the axis along which the
//! crate's guarantees genuinely differ — expressed with the `secx:` extension
//! vocabulary from the ZERO-dependency [`sparq_secprop_vocab`] leaf. Each
//! `secx:hasProperty` is a reified [`PropertyAssertion`]: a `(property, level,
//! assurance, audit-status, assumptions)` claim.
//!
//! [`ontologies/secprop-protocols.ttl`]: https://github.com/jeswr/sparq/blob/main/crates/sparq-mpc/ontologies/secprop-protocols.ttl
//!
//! ## What this is FOR (design record §5c)
//!
//! Admission is **positive-evidence-only**. A protocol satisfies an
//! [`AdversaryRequirement`] only when the block is structurally usable *and* every
//! assertion in it carries a `secx:assumption` that is positive **evidence for**
//! that requirement — see [`supporting_assumptions`]. Absence of
//! [`SECX_SEMI_HONEST`] is *not* evidence of malicious security, and the code never
//! reads it as such.
//!
//! Phase 1 minted no positive `secx:Malicious` / `secx:DishonestMajority` term, so
//! [`supporting_assumptions`] is **empty for both requirements** and *no* annotation
//! — sparq's or anyone else's — can currently supply that evidence. Every protocol
//! is therefore excluded by a malicious-security or dishonest-majority preference.
//! Independently, every assertion in sparq's graph carries both
//! [`SECX_HONEST_MAJORITY`] and [`SECX_SEMI_HONEST`], each an explicit
//! [`disqualifying_assumptions`] entry for the corresponding requirement. That is
//! the honest encoding of "semi-honest only" **in the property data**, not merely in
//! prose: prose can be skimmed past, a `secx:assumption` edge cannot.
//!
//! ## What this records — and what it is NOT
//!
//! It **records** a (protocol, property) → (level, assurance, audit-status,
//! assumptions) claim and its **epistemic basis**; it is **NOT** a proof of any
//! property. The sparq MPC/ZK estate is research-grade and **externally UNAUDITED**
//! (`sq-qhy4`, P0; CR-G1 of `compliance/cryptoreview/gap-register.md`), and this
//! crate is honest-majority **semi-honest only** — not malicious, not
//! dishonest-majority.
//!
//! ## The three guards
//!
//! 1. **Anti-drift** ([`descriptor_drift_violations`]) — the load-bearing one. The
//!    annotated assumption set is recomputed from the [`SecurityDescriptor`] the
//!    code *actually* reports for that operator
//!    (`ShamirBackend::operator_descriptor`) via [`assumptions_for`], and compared
//!    for **set equality**. So the graph can neither over-claim a stronger
//!    adversary model than the implementation reports, nor under-claim one.
//! 2. **No over-claim on assurance** ([`audit_overclaim_violations`]) — no protocol
//!    may carry [`Assurance::Proven`] on a **positive** property while `sq-qhy4` is
//!    open. The privacy-claims gate, encoded in the data.
//! 3. **Completeness** ([`completeness_violations`]) — every [`OperatorClass`] in
//!    [`ANNOTATED_OPERATORS`] has an annotation block, so a new operator cannot be
//!    added without stating its posture.
//!
//! ## Coverage, and the honest gap
//!
//! The subjects are exactly the three [`OperatorClass`] variants. The
//! malicious-with-abort IT-MAC comparison path ([`crate::auth_compare`]) is
//! **deliberately not a subject**: Phase 1 minted only `secx:HonestMajority` and
//! `secx:SemiHonest` and has **no** positive `secx:Malicious` /
//! `secx:DishonestMajority` term, so annotating it could only mislabel it
//! semi-honest (a false exclusion) or omit its adversary axis (a silent
//! fail-open). Omission is the honest option — an unannotated protocol makes no
//! claim, and [`admits_protocol`] denies it by default (fail-closed); so is an
//! annotated-but-structurally-unusable one (see
//! [`ProtocolAnnotations::is_structurally_valid`]). Annotating it needs the Phase-1
//! vocabulary extended first.
//!
//! ## Opt-in by construction
//!
//! Behind the **default-OFF `secprop-annotations`** cargo feature, so it adds
//! nothing to the lean default build. The `secx:` IRIs come from the
//! ZERO-dependency [`sparq_secprop_vocab`] leaf, so the opt-in build grows by a
//! `const &str` table plus the `oxttl` Turtle parser and nothing else.
//!
//! [OPUS-5] sq-dz10l (Phase 6; epic sq-0dksu; design record
//! `research/security-properties-ontology-design.md` §5c). 🤖 SPARQ agent —
//! security-properties ontology.

use crate::backend::{AdversaryModel, OperatorClass, SecurityDescriptor};
use oxrdf::{NamedOrBlankNode, Term};
use oxttl::TurtleParser;
use std::collections::{BTreeMap, BTreeSet};

/// The canonical machine-readable per-protocol annotation graph (the source of
/// truth this module's accessors and guards parse).
const PROTOCOLS_TTL: &str = include_str!("../ontologies/secprop-protocols.ttl");

// ── the `secx:` IRIs this module references ──────────────────────────────────
// Imported from the ZERO-dependency `sparq-secprop-vocab` leaf, which owns the
// constants, the canonical `secprop-ext.ttl` and the single drift test that pins
// the two together (sq-3705). There is no local copy to drift.
use sparq_secprop_vocab::{
    SECX_ASSUMPTION, SECX_ASSURANCE, SECX_AUDIT_STATUS, SECX_CLAIMED, SECX_CONJECTURED,
    SECX_HAS_PROPERTY, SECX_LEVEL, SECX_NOT_BINDING, SECX_NOT_HIDING, SECX_NOT_ZK,
    SECX_PQ_FORGEABLE, SECX_PROPERTY, SECX_PROVEN, SECX_REPLAYABLE, SECX_SCHEME_REVEALED,
    SECX_UNSOUND,
};

/// The `sec-prop:` namespace base the `secx:` IRIs share. Re-exported from
/// [`sparq_secprop_vocab`] — this crate declares no copy of its own.
pub use sparq_secprop_vocab::SEC_PROP_NS;

/// `secx:HonestMajority` — the MPC honest-majority assumption every sparq-mpc
/// protocol rests on. Re-exported so a caller can name it in a preference.
pub use sparq_secprop_vocab::SECX_HONEST_MAJORITY;

/// `secx:SemiHonest` — the semi-honest (honest-but-curious) adversary assumption
/// every sparq-mpc protocol rests on. Re-exported so a caller can name it in a
/// preference. **This is the exclusion hook of §5c.**
pub use sparq_secprop_vocab::SECX_SEMI_HONEST;

/// `secx:ExternalSignOffPending` — the live audit status of every positive sparq
/// property while `sq-qhy4` is open.
pub use sparq_secprop_vocab::SECX_EXTERNAL_SIGN_OFF_PENDING;

// ── the `mpc:` protocol IRIs ─────────────────────────────────────────────────

/// The `mpc:` namespace base for this crate's protocol IRIs (the sparq-local
/// counterpart of `zk:` in `sparq-zk`).
pub const MPC_NS: &str = "https://sparq.dev/ns/mpc#";

/// `mpc:shamir-linear-aggregate` — [`OperatorClass::LinearAggregate`].
pub const MPC_SHAMIR_LINEAR_AGGREGATE: &str = "https://sparq.dev/ns/mpc#shamir-linear-aggregate";

/// `mpc:shamir-equality-join` — [`OperatorClass::EqualityJoin`].
pub const MPC_SHAMIR_EQUALITY_JOIN: &str = "https://sparq.dev/ns/mpc#shamir-equality-join";

/// `mpc:shamir-comparison` — [`OperatorClass::Comparison`].
pub const MPC_SHAMIR_COMPARISON: &str = "https://sparq.dev/ns/mpc#shamir-comparison";

/// Every [`OperatorClass`] that MUST carry an annotation block — the completeness
/// guard's key. Adding an `OperatorClass` variant without adding it here (and a
/// block to the Turtle) fails [`completeness_violations`].
pub const ANNOTATED_OPERATORS: &[OperatorClass] = &[
    OperatorClass::LinearAggregate,
    OperatorClass::EqualityJoin,
    OperatorClass::Comparison,
];

/// The `mpc:` protocol IRI an [`OperatorClass`] is annotated under. Total — every
/// variant has an IRI, so a new variant is a compile error here rather than a
/// silently unannotated protocol.
pub fn protocol_iri(operator: OperatorClass) -> &'static str {
    match operator {
        OperatorClass::LinearAggregate => MPC_SHAMIR_LINEAR_AGGREGATE,
        OperatorClass::EqualityJoin => MPC_SHAMIR_EQUALITY_JOIN,
        OperatorClass::Comparison => MPC_SHAMIR_COMPARISON,
    }
}

// ── typed model ──────────────────────────────────────────────────────────────

/// The epistemic-basis axis of a [`PropertyAssertion`] — `Proven ⊐ Claimed ⊐
/// Conjectured`. The honesty mechanism: a **positive** sparq property may only ever
/// be [`Self::Claimed`] or weaker while `sq-qhy4` is open; [`Self::Proven`] is
/// reserved for settled NEGATIVE facts about the construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assurance {
    /// Backed by a formal proof or an external audit (including a provable
    /// NEGATIVE result). Not applicable to a sparq positive property while
    /// `sq-qhy4` is open.
    Proven,
    /// Asserted, not independently verified. The default for a sparq property.
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

/// The `secx:level` IRIs that are settled **NEGATIVE** facts about a construction.
/// Only these may carry [`Assurance::Proven`] while `sq-qhy4` is open.
const NEGATIVE_LEVELS: &[&str] = &[
    SECX_NOT_HIDING,
    SECX_NOT_BINDING,
    SECX_NOT_ZK,
    SECX_UNSOUND,
    SECX_REPLAYABLE,
    SECX_PQ_FORGEABLE,
    SECX_SCHEME_REVEALED,
];

/// One reified `secx:hasProperty` claim: a `(property, level, assurance,
/// audit-status, assumptions)` tuple.
///
/// Note `assumptions` is a **set**, not the single `Option` the `sparq-zk`
/// counterpart carries: an MPC posture is irreducibly two-axis (adversary model ×
/// corruption threshold), so a protocol rests on `secx:HonestMajority` **and**
/// `secx:SemiHonest` simultaneously. Collapsing that to one value would silently
/// drop half the exclusion criterion of §5c.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyAssertion {
    /// The `secx:property` dimension the assertion is about.
    pub property: String,
    /// The `secx:level` held within that dimension, if stated.
    pub level: Option<String>,
    /// The epistemic basis of the claim.
    pub assurance: Assurance,
    /// The `secx:auditStatus`, if stated.
    pub audit_status: Option<String>,
    /// Every `secx:assumption` this claim rests on. **Empty means the claim states
    /// no assumption** — which the guards treat as a drift, not as "assumption-free",
    /// and which makes the enclosing block structurally invalid for admission (see
    /// [`ProtocolAnnotations::is_structurally_valid`]).
    pub assumptions: BTreeSet<String>,
}

impl PropertyAssertion {
    /// Whether this is a **positive** claim (so it may not be [`Assurance::Proven`]
    /// while `sq-qhy4` is open). A claim is negative only if its level is one of
    /// the settled-negative levels.
    pub fn is_positive(&self) -> bool {
        match self.level.as_deref() {
            None => true,
            Some(l) => !NEGATIVE_LEVELS.contains(&l),
        }
    }

    /// Whether this claim rests on `assumption`.
    pub fn rests_on(&self, assumption: &str) -> bool {
        self.assumptions.iter().any(|a| a == assumption)
    }
}

/// Every annotation on one `mpc:` protocol IRI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolAnnotations {
    /// The `mpc:` protocol IRI these assertions are about.
    pub protocol: String,
    /// The reified claims, in a stable `(property, level)` order.
    pub assertions: Vec<PropertyAssertion>,
    /// How many `secx:hasProperty` nodes on this protocol were **discarded** as
    /// malformed by the parser (no `secx:property`, or a missing/unknown
    /// `secx:assurance`). Non-zero means the block is not a faithful reading of the
    /// graph, so [`Self::is_structurally_valid`] — and hence every admission —
    /// fails: a dropped assertion must never be mistaken for one that was never
    /// there.
    pub malformed_assertions: usize,
}

impl ProtocolAnnotations {
    /// The union of every assumption any of this protocol's claims rests on.
    pub fn assumptions(&self) -> BTreeSet<&str> {
        self.assertions
            .iter()
            .flat_map(|a| a.assumptions.iter().map(String::as_str))
            .collect()
    }

    /// Whether **any** claim on this protocol rests on `assumption`.
    pub fn rests_on(&self, assumption: &str) -> bool {
        self.assertions.iter().any(|a| a.rests_on(assumption))
    }

    /// Whether this block is usable as the basis of an admission decision at all:
    /// no assertion was discarded as malformed, there is at least one assertion, and
    /// **every** assertion states at least one `secx:assumption`.
    ///
    /// An empty block, a block whose assertions state no assumptions, and a block
    /// the parser had to drop nodes from all describe *nothing checkable* about the
    /// threat model. Treating any of them as "no limitation found" is the fail-open
    /// this predicate exists to prevent.
    pub fn is_structurally_valid(&self) -> bool {
        self.malformed_assertions == 0
            && !self.assertions.is_empty()
            && self.assertions.iter().all(|a| !a.assumptions.is_empty())
    }

    /// Whether this protocol satisfies `requirement`.
    ///
    /// This is the §5c exclusion, computed from the graph — **not** from prose — and
    /// it is **positive-evidence-only**. It holds when all three are true:
    ///
    /// 1. [`Self::is_structurally_valid`] — otherwise there is no checkable claim.
    /// 2. Every assertion rests on at least one [`supporting_assumptions`] IRI for
    ///    `requirement`.
    /// 3. No assertion rests on a [`disqualifying_assumptions`] IRI for it.
    ///
    /// Because the Phase-1 vocabulary minted no positive `secx:Malicious` /
    /// `secx:DishonestMajority` term, (2) is currently unsatisfiable and this returns
    /// `false` for **every** protocol. That is deliberate: the alternative — reading
    /// the *absence* of `secx:SemiHonest` as evidence of malicious security — would
    /// admit an empty, assumption-less or partially-dropped block under the strongest
    /// threat models. (3) is not yet load-bearing on its own; it becomes so the moment
    /// (2) can be satisfied, and keeps a self-contradictory annotation denied then.
    ///
    /// Prefer [`admits_protocol`] at a lookup boundary: it additionally denies an
    /// **unannotated** protocol (fail-closed), which this method cannot see.
    pub fn admits(&self, requirement: AdversaryRequirement) -> bool {
        if !self.is_structurally_valid() {
            return false;
        }
        let support = supporting_assumptions(requirement);
        let disqualifiers = disqualifying_assumptions(requirement);
        self.assertions.iter().all(|a| {
            support.iter().any(|s| a.rests_on(s)) && disqualifiers.iter().all(|d| !a.rests_on(d))
        })
    }
}

/// The `secx:assumption` IRIs that are positive **evidence for** `requirement` — an
/// assertion carrying one of them supports the claim that the protocol meets it.
///
/// **Both are currently empty.** Phase 1 minted `secx:HonestMajority` and
/// `secx:SemiHonest` only, and no positive `secx:Malicious` / `secx:DishonestMajority`
/// counterpart, so the stronger threat models are honestly *unrepresentable* rather
/// than inferred from an absence. Extending the Phase-1 vocabulary is what fills these
/// in; until then [`ProtocolAnnotations::admits`] denies both requirements outright.
pub fn supporting_assumptions(requirement: AdversaryRequirement) -> &'static [&'static str] {
    match requirement {
        AdversaryRequirement::MaliciousSecurity => &[],
        AdversaryRequirement::DishonestMajority => &[],
    }
}

/// The `secx:assumption` IRIs that **disqualify** an assertion from `requirement` —
/// resting on one is positive evidence *against* it.
///
/// Unlike [`supporting_assumptions`] these are representable today, and every
/// sparq-mpc assertion carries both, which is why the crate's own posture is excluded
/// on evidence rather than on silence.
pub fn disqualifying_assumptions(requirement: AdversaryRequirement) -> &'static [&'static str] {
    match requirement {
        AdversaryRequirement::MaliciousSecurity => &[SECX_SEMI_HONEST],
        AdversaryRequirement::DishonestMajority => &[SECX_HONEST_MAJORITY],
    }
}

/// A threat-model requirement a requester's privacy preference can demand, and
/// which the annotation graph is checked against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdversaryRequirement {
    /// The protocol must remain secure against an **actively deviating** party.
    /// Excludes anything resting on `secx:SemiHonest`.
    MaliciousSecurity,
    /// The protocol must remain secure without an honest majority (up to `n − 1`
    /// corruptions). Excludes anything resting on `secx:HonestMajority`.
    DishonestMajority,
}

/// Whether the protocol named by `protocol` satisfies `requirement`, **fail-closed**.
///
/// An **unannotated** protocol makes no claim, so it satisfies no requirement and
/// this returns `false`. That is the default-deny posture the trust-graph admission
/// gate needs: absence of an annotation must never read as absence of a limitation.
///
/// The same holds one level down — [`ProtocolAnnotations::admits`] requires positive
/// evidence over a [structurally valid](ProtocolAnnotations::is_structurally_valid)
/// block, so an annotation that is *present but says nothing checkable* is denied too.
pub fn admits_protocol(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
    protocol: &str,
    requirement: AdversaryRequirement,
) -> bool {
    annotations
        .get(protocol)
        .is_some_and(|a| a.admits(requirement))
}

// ── the code-derived assumption set (the anti-drift source of truth) ─────────

/// The `secx:` assumption IRIs a [`SecurityDescriptor`] **actually** implies —
/// computed from the crate's own three-axis typed model, not from the Turtle.
///
/// This is what makes the annotation graph honest rather than aspirational: the
/// guard [`descriptor_drift_violations`] compares the graph against this, so the
/// Turtle cannot drift from what the implementation reports.
///
/// - AXIS-3 honest-/super-honest-majority ⇒ `secx:HonestMajority`.
/// - AXIS-1 [`AdversaryModel::SemiHonest`] ⇒ `secx:SemiHonest`.
///
/// [`AdversaryModel::Covert`] and [`AdversaryModel::Malicious`] contribute **no**
/// IRI: Phase 1 minted no term for either, so they are honestly unrepresentable
/// rather than approximated by the nearest available label.
pub fn assumptions_for(descriptor: &SecurityDescriptor) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    if descriptor.threshold.is_honest_majority() {
        out.insert(SECX_HONEST_MAJORITY);
    }
    if descriptor.adversary == AdversaryModel::SemiHonest {
        out.insert(SECX_SEMI_HONEST);
    }
    out
}

// ── violations ───────────────────────────────────────────────────────────────

/// A guard violation — a parse-stable, human-readable record, never a panic.
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
    /// An [`OperatorClass`] in [`ANNOTATED_OPERATORS`] has no annotation block.
    MissingAnnotation,
    /// The annotated assumption set disagrees with the one [`assumptions_for`]
    /// derives from the [`SecurityDescriptor`] the code reports.
    AssumptionDrift,
}

// ── parsing ──────────────────────────────────────────────────────────────────

/// Parse the static annotation graph into one [`ProtocolAnnotations`] per protocol
/// IRI, keyed by IRI in a stable order. Pure over the bundled Turtle — no I/O.
///
/// Implementation note: the Turtle uses `secx:hasProperty [ … ]` blank-node
/// objects, so we first index every triple by blank-node subject, then resolve each
/// `(protocol, hasProperty, _:b)` edge to the pairs on `_:b`.
pub fn parse_annotations() -> BTreeMap<String, ProtocolAnnotations> {
    parse_annotations_str(PROTOCOLS_TTL)
}

/// [`parse_annotations`] over arbitrary Turtle — the testable seam, so the
/// malformed-assertion accounting can be exercised on inputs the bundled graph
/// deliberately never contains.
fn parse_annotations_str(ttl: &str) -> BTreeMap<String, ProtocolAnnotations> {
    // blank-node id -> its (predicate IRI, object) pairs.
    let mut bnode_pairs: BTreeMap<String, Vec<(String, Term)>> = BTreeMap::new();
    // protocol IRI -> the blank-node ids of its assertion nodes.
    let mut protocol_assertion_bnodes: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for result in TurtleParser::new().for_reader(ttl.as_bytes()) {
        let t = result.expect("secprop annotation graph must be valid Turtle");
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
        // A `secx:hasProperty` node we cannot read is COUNTED, never silently
        // dropped: an unreadable assumption list is exactly the case that must not
        // look like "this protocol states no limitation".
        let mut malformed_assertions = 0usize;
        for b in bnodes {
            match bnode_pairs
                .get(&b)
                .map(Vec::as_slice)
                .and_then(assertion_from_pairs)
            {
                Some(a) => assertions.push(a),
                None => malformed_assertions += 1,
            }
        }
        // Stable order: by (property, level) so output is deterministic.
        assertions.sort_by(|x, y| {
            (x.property.as_str(), x.level.as_deref()).cmp(&(y.property.as_str(), y.level.as_deref()))
        });
        out.insert(
            protocol.clone(),
            ProtocolAnnotations {
                protocol,
                assertions,
                malformed_assertions,
            },
        );
    }
    out
}

/// Build a [`PropertyAssertion`] from the predicate-object pairs of one blank node.
/// Returns `None` if there is no `secx:property` (not an assertion node) or the
/// assurance is missing/unknown — fail-closed: an annotation must state its basis.
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
    // ALL `secx:assumption` objects, not just the first — an MPC posture is
    // two-axis, so taking only one would silently drop half the exclusion criterion.
    let assumptions: BTreeSet<String> = pairs
        .iter()
        .filter_map(|(pred, obj)| match obj {
            Term::NamedNode(n) if pred == SECX_ASSUMPTION => Some(n.as_str().to_owned()),
            _ => None,
        })
        .collect();

    let property = iri_obj(SECX_PROPERTY)?;
    let assurance = Assurance::from_iri(&iri_obj(SECX_ASSURANCE)?)?;

    Some(PropertyAssertion {
        property,
        level: iri_obj(SECX_LEVEL),
        assurance,
        audit_status: iri_obj(SECX_AUDIT_STATUS),
        assumptions,
    })
}

// ── guards ───────────────────────────────────────────────────────────────────

/// Whether the external audit `sq-qhy4` is still open. The annotation graph and the
/// guards are written for the open state; this is the single switch a future
/// assurance-promotion flips. It is **hard-coded `true`** — only an external
/// accredited-cryptographer sign-off (out of agent scope) may flip it.
pub const fn audit_qhy4_open() -> bool {
    true
}

/// **Guard 1 — anti-drift (the load-bearing one).** For each `(operator,
/// descriptor)` the caller reports, check that the annotated assumption set for
/// that operator's protocol IRI is **set-equal** to [`assumptions_for`] of the
/// descriptor the code actually produces. An empty result = no drift.
///
/// Both directions are violations: a graph claiming a **weaker** assumption set
/// than the code reports over-claims security (it would admit the protocol under a
/// preference it should fail), and one claiming a **stronger** set under-claims
/// (it would exclude a protocol needlessly). A missing block is reported too, so
/// drift cannot be hidden by deleting an annotation.
pub fn descriptor_drift_violations(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
    reported: &[(OperatorClass, SecurityDescriptor)],
) -> Vec<Violation> {
    let mut out = Vec::new();
    for (operator, descriptor) in reported {
        let iri = protocol_iri(*operator);
        let Some(ann) = annotations.get(iri) else {
            out.push(Violation {
                protocol: iri.to_owned(),
                kind: ViolationKind::MissingAnnotation,
                detail: format!("{:?} has no annotation block to check for drift", operator),
            });
            continue;
        };
        let derived: BTreeSet<&str> = assumptions_for(descriptor).into_iter().collect();
        let annotated = ann.assumptions();
        if annotated != derived {
            out.push(Violation {
                protocol: iri.to_owned(),
                kind: ViolationKind::AssumptionDrift,
                detail: format!(
                    "annotated assumptions {:?} disagree with those derived from the reported SecurityDescriptor {:?}",
                    annotated, derived,
                ),
            });
        }
        // Every individual assertion must carry the full set too — a protocol-level
        // union assembled from assertions that each state only half the posture
        // would let a per-assertion check (the shape a preference actually
        // evaluates) see an incomplete assumption list.
        for a in &ann.assertions {
            let per_assertion: BTreeSet<&str> = a.assumptions.iter().map(String::as_str).collect();
            if per_assertion != derived {
                out.push(Violation {
                    protocol: iri.to_owned(),
                    kind: ViolationKind::AssumptionDrift,
                    detail: format!(
                        "assertion on {} carries assumptions {:?}, not the derived {:?}",
                        a.property, per_assertion, derived,
                    ),
                });
            }
        }
    }
    out
}

/// **Guard 2 — no over-claim on assurance.** Every annotation where a **positive**
/// property carries [`Assurance::Proven`] while `sq-qhy4` is open. An empty result
/// = no over-claim.
pub fn audit_overclaim_violations(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
) -> Vec<Violation> {
    if !audit_qhy4_open() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (protocol, ann) in annotations {
        for a in &ann.assertions {
            if a.assurance == Assurance::Proven && a.is_positive() {
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

/// **Guard 3 — completeness.** Every [`OperatorClass`] in `required` that has no
/// annotation block. An empty result = complete coverage. Callers normally pass
/// [`ANNOTATED_OPERATORS`].
pub fn completeness_violations(
    annotations: &BTreeMap<String, ProtocolAnnotations>,
    required: &[OperatorClass],
) -> Vec<Violation> {
    required
        .iter()
        .map(|op| (op, protocol_iri(*op)))
        .filter(|(_, iri)| !annotations.contains_key(*iri))
        .map(|(op, iri)| Violation {
            protocol: iri.to_owned(),
            kind: ViolationKind::MissingAnnotation,
            detail: format!("operator {:?} has no annotation block", op),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{CorruptionThreshold, MpcBackend, OutputGuarantee, PublicVerifiability};
    use crate::shamir::ShamirBackend;

    fn annotations() -> BTreeMap<String, ProtocolAnnotations> {
        parse_annotations()
    }

    #[test]
    fn graph_parses_with_a_block_per_operator() {
        let ann = annotations();
        assert_eq!(ann.len(), ANNOTATED_OPERATORS.len());
        for op in ANNOTATED_OPERATORS {
            let block = ann
                .get(protocol_iri(*op))
                .unwrap_or_else(|| panic!("no annotation block for {:?}", op));
            assert!(
                !block.assertions.is_empty(),
                "{:?} block has no assertions",
                op
            );
        }
        assert!(completeness_violations(&ann, ANNOTATED_OPERATORS).is_empty());
    }

    /// The §5c headline: EVERY protocol rests on BOTH assumptions, so a
    /// malicious-security or dishonest-majority preference excludes ALL of them.
    #[test]
    fn every_protocol_rests_on_honest_majority_and_semi_honest() {
        let ann = annotations();
        for op in ANNOTATED_OPERATORS {
            let block = &ann[protocol_iri(*op)];
            for a in &block.assertions {
                assert!(
                    a.rests_on(SECX_HONEST_MAJORITY),
                    "{:?} / {} does not rest on secx:HonestMajority",
                    op,
                    a.property
                );
                assert!(
                    a.rests_on(SECX_SEMI_HONEST),
                    "{:?} / {} does not rest on secx:SemiHonest",
                    op,
                    a.property
                );
            }
        }
    }

    #[test]
    fn malicious_and_dishonest_majority_preferences_exclude_every_protocol() {
        let ann = annotations();
        for op in ANNOTATED_OPERATORS {
            let iri = protocol_iri(*op);
            assert!(
                !admits_protocol(&ann, iri, AdversaryRequirement::MaliciousSecurity),
                "{:?} must be excluded by a malicious-security preference",
                op
            );
            assert!(
                !admits_protocol(&ann, iri, AdversaryRequirement::DishonestMajority),
                "{:?} must be excluded by a dishonest-majority preference",
                op
            );
        }
    }

    /// Fail-closed: a protocol with NO annotation satisfies NO requirement. Absence
    /// of an annotation must never read as absence of a limitation — this is the
    /// guard that keeps the deliberately-unannotated `auth_compare` path denied.
    #[test]
    fn unannotated_protocol_is_denied() {
        let ann = annotations();
        let unknown = "https://sparq.dev/ns/mpc#not-annotated";
        assert!(!admits_protocol(
            &ann,
            unknown,
            AdversaryRequirement::MaliciousSecurity
        ));
        assert!(!admits_protocol(
            &ann,
            unknown,
            AdversaryRequirement::DishonestMajority
        ));
    }

    /// Build an otherwise-valid single-assertion block resting on `assumptions`.
    fn block_resting_on(assumptions: &[&str]) -> ProtocolAnnotations {
        ProtocolAnnotations {
            protocol: "https://sparq.dev/ns/mpc#hypothetical".to_owned(),
            assertions: vec![PropertyAssertion {
                property: sparq_secprop_vocab::SECX_SOUNDNESS.to_owned(),
                level: Some(sparq_secprop_vocab::SECX_SOUND.to_owned()),
                assurance: Assurance::Claimed,
                audit_status: Some(SECX_EXTERNAL_SIGN_OFF_PENDING.to_owned()),
                assumptions: assumptions.iter().map(|a| (*a).to_owned()).collect(),
            }],
            malformed_assertions: 0,
        }
    }

    /// Admission is POSITIVE-EVIDENCE-only: dropping `secx:SemiHonest` from a block
    /// does NOT buy it malicious security. This is the fail-open the absence-based
    /// rule had — a protocol that simply never mentions the semi-honest assumption
    /// used to be admitted under the strongest threat model on no evidence at all.
    #[test]
    fn absence_of_semi_honest_is_not_evidence_of_malicious_security() {
        let honest_majority_only = block_resting_on(&[SECX_HONEST_MAJORITY]);
        assert!(
            !honest_majority_only.admits(AdversaryRequirement::MaliciousSecurity),
            "no secx:SemiHonest edge is not the same as evidence of malicious security"
        );
        assert!(
            !honest_majority_only.admits(AdversaryRequirement::DishonestMajority),
            "an honest-majority protocol must fail a dishonest-majority preference"
        );
    }

    /// The structural-validity vectors the absence-based rule admitted for BOTH
    /// stronger threat models: an empty block, a block whose assertions state no
    /// assumptions, and a block the parser had to discard nodes from.
    #[test]
    fn structurally_unusable_blocks_are_denied_every_requirement() {
        let empty = ProtocolAnnotations {
            protocol: "https://sparq.dev/ns/mpc#empty".to_owned(),
            assertions: Vec::new(),
            malformed_assertions: 0,
        };
        let no_assumptions = block_resting_on(&[]);
        let mut partly_dropped = block_resting_on(&[SECX_HONEST_MAJORITY]);
        partly_dropped.malformed_assertions = 1;

        for block in [&empty, &no_assumptions, &partly_dropped] {
            assert!(
                !block.is_structurally_valid(),
                "{} must not be structurally valid",
                block.protocol
            );
            for requirement in [
                AdversaryRequirement::MaliciousSecurity,
                AdversaryRequirement::DishonestMajority,
            ] {
                assert!(
                    !block.admits(requirement),
                    "{} must be denied {:?}",
                    block.protocol,
                    requirement
                );
            }
        }
        // ...and the sound shape IS structurally valid, so the predicate is not a
        // hard-coded `false`.
        assert!(block_resting_on(&[SECX_HONEST_MAJORITY]).is_structurally_valid());
    }

    /// An assertion the parser cannot read (missing/unknown `secx:assurance`) is
    /// COUNTED, not silently discarded — otherwise a graph could shed exactly the
    /// assumptions that disqualify it and read as unconstrained.
    #[test]
    fn unreadable_assertions_are_counted_and_deny_admission() {
        let ttl = format!(
            r#"
            @prefix secx: <{}> .
            <{}> secx:hasProperty [
                secx:property secx:Soundness ;
                secx:assumption secx:HonestMajority
            ] .
            "#,
            SEC_PROP_NS, MPC_SHAMIR_COMPARISON,
        );
        let parsed = parse_annotations_str(&ttl);
        let block = &parsed[MPC_SHAMIR_COMPARISON];
        assert_eq!(
            block.malformed_assertions, 1,
            "an assertion with no secx:assurance must be counted as malformed"
        );
        assert!(block.assertions.is_empty());
        assert!(!admits_protocol(
            &parsed,
            MPC_SHAMIR_COMPARISON,
            AdversaryRequirement::MaliciousSecurity
        ));
        // The bundled graph, by contrast, parses cleanly.
        assert!(annotations()
            .values()
            .all(|b| b.malformed_assertions == 0 && b.is_structurally_valid()));
    }

    /// The two evidence tables. `supporting_assumptions` being empty is the whole
    /// reason every requirement is currently denied, so it is asserted explicitly
    /// rather than left implicit in the admission results.
    #[test]
    fn evidence_tables_record_the_phase_1_vocabulary_gap() {
        for requirement in [
            AdversaryRequirement::MaliciousSecurity,
            AdversaryRequirement::DishonestMajority,
        ] {
            assert!(
                supporting_assumptions(requirement).is_empty(),
                "Phase 1 minted no positive term for {:?}; update this test and the \
                 module docs when it does",
                requirement
            );
        }
        assert_eq!(
            disqualifying_assumptions(AdversaryRequirement::MaliciousSecurity),
            [SECX_SEMI_HONEST]
        );
        assert_eq!(
            disqualifying_assumptions(AdversaryRequirement::DishonestMajority),
            [SECX_HONEST_MAJORITY]
        );
    }

    /// The anti-drift guard against the descriptors the REAL backend reports, over
    /// every `n` the public constructor accepts in a representative range (it always
    /// picks `t = ⌊(n−1)/2⌋`, so every reachable configuration is honest-majority).
    #[test]
    fn annotations_match_the_reported_descriptors() {
        let ann = annotations();
        for n in 2..=9 {
            let backend = ShamirBackend::new(n).expect("valid n");
            let reported: Vec<_> = ANNOTATED_OPERATORS
                .iter()
                .map(|op| (*op, backend.operator_security(*op)))
                .collect();
            let violations = descriptor_drift_violations(&ann, &reported);
            assert!(
                violations.is_empty(),
                "assumption drift at n = {}: {:?}",
                n,
                violations
            );
        }
    }

    /// The drift guard actually fires — a descriptor whose adversary model is
    /// `Malicious` derives a different assumption set than the graph states, and
    /// must be reported. Without this the guard could be vacuously green.
    #[test]
    fn drift_guard_fires_on_a_mismatched_descriptor() {
        let ann = annotations();
        let malicious = SecurityDescriptor {
            adversary: AdversaryModel::Malicious,
            output_guarantee: OutputGuarantee::Abort(crate::backend::AbortKind::Unanimous),
            threshold: CorruptionThreshold::HonestMajority { t: 1 },
            public_verifiability: PublicVerifiability(false),
        };
        let violations =
            descriptor_drift_violations(&ann, &[(OperatorClass::Comparison, malicious)]);
        assert!(
            violations
                .iter()
                .any(|v| v.kind == ViolationKind::AssumptionDrift),
            "a Malicious descriptor must drift from the SemiHonest annotation, got {:?}",
            violations
        );
    }

    #[test]
    fn assumptions_for_maps_both_axes() {
        let semi_honest_majority = SecurityDescriptor::semi_honest_only(3, 1);
        let derived = assumptions_for(&semi_honest_majority);
        assert!(derived.contains(SECX_HONEST_MAJORITY));
        assert!(derived.contains(SECX_SEMI_HONEST));

        // A dishonest-majority descriptor drops the honest-majority assumption; a
        // malicious one drops the semi-honest assumption. Neither is representable
        // POSITIVELY (Phase 1 minted no term), so the set simply shrinks.
        let dishonest = SecurityDescriptor {
            adversary: AdversaryModel::Malicious,
            output_guarantee: OutputGuarantee::Abort(crate::backend::AbortKind::Unanimous),
            threshold: CorruptionThreshold::DishonestMajority { t: 2 },
            public_verifiability: PublicVerifiability(false),
        };
        assert!(assumptions_for(&dishonest).is_empty());
    }

    /// The privacy-claims gate encoded in the data: no positive property may be
    /// `Proven` while sq-qhy4 is open.
    #[test]
    fn no_positive_property_is_proven_while_the_audit_is_open() {
        let ann = annotations();
        assert!(audit_qhy4_open());
        let violations = audit_overclaim_violations(&ann);
        assert!(violations.is_empty(), "over-claim: {:?}", violations);
        // Non-vacuous: every assertion in the graph IS positive and IS Claimed, so
        // the guard has real subjects rather than an empty input.
        for block in ann.values() {
            for a in &block.assertions {
                assert!(a.is_positive());
                assert_eq!(a.assurance, Assurance::Claimed);
                assert_eq!(
                    a.audit_status.as_deref(),
                    Some(SECX_EXTERNAL_SIGN_OFF_PENDING)
                );
            }
        }
    }

    /// The over-claim guard fires on a `Proven` positive property.
    #[test]
    fn overclaim_guard_fires_on_a_proven_positive() {
        let mut ann = BTreeMap::new();
        ann.insert(
            MPC_SHAMIR_COMPARISON.to_owned(),
            ProtocolAnnotations {
                protocol: MPC_SHAMIR_COMPARISON.to_owned(),
                assertions: vec![PropertyAssertion {
                    property: sparq_secprop_vocab::SECX_SOUNDNESS.to_owned(),
                    level: Some(sparq_secprop_vocab::SECX_SOUND.to_owned()),
                    assurance: Assurance::Proven,
                    audit_status: None,
                    assumptions: BTreeSet::new(),
                }],
                malformed_assertions: 0,
            },
        );
        let violations = audit_overclaim_violations(&ann);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].kind,
            ViolationKind::ProvenPositiveWhileAuditOpen
        );
    }

    /// A settled NEGATIVE level may be `Proven` — the guard must not fire there.
    #[test]
    fn negative_levels_may_be_proven() {
        let negative = PropertyAssertion {
            property: sparq_secprop_vocab::SECX_SOUNDNESS.to_owned(),
            level: Some(SECX_UNSOUND.to_owned()),
            assurance: Assurance::Proven,
            audit_status: None,
            assumptions: BTreeSet::new(),
        };
        assert!(!negative.is_positive());
    }

    #[test]
    fn completeness_guard_fires_on_a_missing_block() {
        let violations = completeness_violations(&BTreeMap::new(), ANNOTATED_OPERATORS);
        assert_eq!(violations.len(), ANNOTATED_OPERATORS.len());
        assert!(violations
            .iter()
            .all(|v| v.kind == ViolationKind::MissingAnnotation));
    }
}
