//! # `secprop` — first-class `secx:requires…` ODRL leftOperands (the sparq
//! security-property profile)
//!
//! Makes the custom `secx:requires…` ODRL leftOperands of the sparq
//! **security-property profile** first-class in the policy model: their IRIs as
//! Rust constants, the leftOperand → security-property-dimension mapping
//! ([`over_dimension`]), a recogniser ([`is_secprop_left_operand`]), and the
//! published profile IRI ([`PROFILE_IRI`]).
//!
//! ## Why this exists (the ODRL profile nuance)
//!
//! An [`crate::Constraint`] over a `leftOperand` may carry **any** IRI — sparq-policy
//! already parses and evaluates a `secx:requires…` constraint as an opaque
//! custom-leftOperand string (design §4.3.1). But a **conforming ODRL processor MAY
//! reject an undeclared custom leftOperand**: the W3C ODRL 2.2 contract is that a
//! custom leftOperand MUST be declared in a published [`odrl:Profile`](PROFILE_IRI)
//! (as `owl:NamedIndividual, odrl:LeftOperand, skos:Concept` with `rdfs:isDefinedBy`
//! → the profile), and a policy using them must assert `odrl:profile <profileIRI>`.
//! This module ships that profile (`ontologies/odrl-secprop-profile.ttl`) and makes
//! the leftOperands first-class — a small, standards-conformant addition, not ad-hoc.
//!
//! ## What this is — and is NOT
//!
//! This is the **bridge vocabulary**: it NAMES the requireable dimensions and the
//! leftOperand→dimension map ([`SECPROP_LEFT_OPERANDS`]). It does **not** perform the
//! admissibility reduction (that is the N3 ruleset, Phase 2, `sq-ufsi9`, over the
//! per-method annotation graph, Phase 3, `sq-bevd3`), and it asserts **NO** security
//! property of any method. Whether a sparq method actually *has* a property is
//! recorded — with its epistemic basis — by the annotation graph; **sparq's whole ZK
//! estate is research-grade and NOT externally audited** (bead `sq-qhy4`), and
//! `sparq-mpc` is semi-honest-only. This module only lets a policy *name the bar*; it
//! makes no privacy/soundness claim.
//!
//! ## The ODRL half of the ZK↔ODRL constraint-discharge envelope (sq-yh427)
//!
//! [`discharge_obligations`] projects a parsed [`crate::Policy`] to the list of
//! `secx:requires…` constraints it carries — *what a presented proof would have to
//! establish*, over which dimension, at which level, and on which deontic rule. That
//! is the **ODRL-side interface** the ZK↔ODRL envelope plugs into: a host reads it to
//! know which properties to demand of a presentation before it can decide.
//!
//! The **wire half** of that envelope — a VC 2.0 Verifiable Presentation carrying a
//! Data-Integrity proof under a `sparql-zk`-style cryptosuite — is **deliberately NOT
//! implemented here**: no such cryptosuite exists in this workspace (`sparq-vc`
//! implements `eddsa-rdfc-2022` only), its identifier and claim shape are still under
//! cross-agent design with the Solid-server sibling, and sparq's ZK estate has **no**
//! external accredited-cryptographer sign-off (`sq-qhy4`). Extraction makes **no**
//! security claim and performs **no** verification.
//!
//! Nor does this module *discharge* anything. It does not order levels (the
//! `secx:atLeast` partial order + the admissibility reduction live in
//! `sparq-trust`'s `admissibility` module, `sq-ufsi9`) and it does not change the
//! stateless [`crate::evaluate`] path: a `secx:` constraint the request supplies no
//! evidence for remains fail-closed unsatisfied, exactly as before.
//!
//! ## Desugaring (design §4.5)
//!
//! Each `secx:requiresX` leftOperand is convenience **sugar** for the single generic
//! primitive "a constraint over dimension `X`": the one `secx:overDimension` fact per
//! leftOperand is what the admissibility rule reads, so a working group standardises
//! only the generic `hasProperty`/`overDimension`/`atLeast` machinery — one rule, not
//! one per leftOperand.
//!
//! ## Opt-in by construction
//!
//! This whole module (and the `.ttl` it pins) is behind the **default-OFF
//! `secprop-leftoperands`** cargo feature, so it adds **nothing** to the lean default
//! build — it is plain `const &str` data plus tests, no new dependency, strictly
//! additive, and it does **not** change the stateless `evaluate` path.
//!
//! The canonical machine-readable form is `ontologies/odrl-secprop-profile.ttl`; the
//! drift tests below keep that profile and these constants from diverging (the
//! `crates/sparq-trust/src/secprop.rs` discipline).
//!
//! [OPUS-4.8] sq-uor3g (epic sq-0dksu, Phase 4; design record
//! `research/security-properties-ontology-design.md` §4.3.1 + §9). 🤖 SPARQ agent —
//! security-properties ontology. Flag for re-review when Fable returns.

use crate::model::{Constraint, ConstraintNode, LogicalConstraint, Operator, Policy, Rule, Value};

/// The `secx:` namespace base — the vendored ZKP-SPARQL `sec-prop:` extension
/// namespace these leftOperands and dimensions live under (a real `w3id.org`
/// permanent identifier; the same namespace `crates/sparq-trust/src/secprop.rs`
/// declares the vocabulary under). NOT a placeholder.
pub const SECX_NS: &str = "https://w3id.org/zkp-sparql/sec-prop#";

/// The published sparq **security-property ODRL profile** IRI. A policy that uses any
/// `secx:requires…` leftOperand MUST assert `odrl:profile <PROFILE_IRI>` so a
/// conforming ODRL processor admits the custom leftOperands (design §4.3.1). The
/// profile document is `ontologies/odrl-secprop-profile.ttl`.
pub const PROFILE_IRI: &str = "https://sparq.dev/ns/odrl-secprop-profile#";

/// The `secx:overDimension` property IRI — maps a `secx:requires…` leftOperand to the
/// `secx:` security-property dimension a constraint over it ranges over (the single
/// primitive the admissibility rule reads).
pub const OVER_DIMENSION: &str = "https://w3id.org/zkp-sparql/sec-prop#overDimension";

// ── the requireable leftOperands ─────────────────────────────────────────────

/// `secx:requiresUnlinkabilityScope` — over the `UnlinkabilityScope` dimension.
pub const REQUIRES_UNLINKABILITY_SCOPE: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresUnlinkabilityScope";
/// `secx:requiresUnlinkabilityStrength` — over the `UnlinkabilityStrength` dimension.
pub const REQUIRES_UNLINKABILITY_STRENGTH: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresUnlinkabilityStrength";
/// `secx:requiresPostQuantumForgery` — over the `PostQuantumForgery` dimension.
pub const REQUIRES_POST_QUANTUM_FORGERY: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresPostQuantumForgery";
/// `secx:requiresPostQuantumSnooping` — over the `PostQuantumSnooping` dimension.
pub const REQUIRES_POST_QUANTUM_SNOOPING: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresPostQuantumSnooping";
/// `secx:requiresZeroKnowledge` — over the `ZeroKnowledgeType` dimension.
pub const REQUIRES_ZERO_KNOWLEDGE: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresZeroKnowledge";
/// `secx:requiresSoundness` — over the `Soundness` dimension.
pub const REQUIRES_SOUNDNESS: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresSoundness";
/// `secx:requiresCompleteness` — over the `Completeness` dimension.
pub const REQUIRES_COMPLETENESS: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresCompleteness";
/// `secx:requiresHiding` — over the `Hiding` dimension.
pub const REQUIRES_HIDING: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresHiding";
/// `secx:requiresBinding` — over the `Binding` dimension.
pub const REQUIRES_BINDING: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresBinding";
/// `secx:requiresAnonymity` — over the `Anonymity` dimension.
pub const REQUIRES_ANONYMITY: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresAnonymity";
/// `secx:requiresSelectiveDisclosure` — over the `SelectiveDisclosure` dimension.
pub const REQUIRES_SELECTIVE_DISCLOSURE: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresSelectiveDisclosure";
/// `secx:requiresSingleUse` — over the `SingleUse` dimension.
pub const REQUIRES_SINGLE_USE: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresSingleUse";
/// `secx:requiresSetup` — over the `Setup` dimension.
pub const REQUIRES_SETUP: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresSetup";
/// `secx:requiresInteractivity` — over the `Interactivity` dimension.
pub const REQUIRES_INTERACTIVITY: &str =
    "https://w3id.org/zkp-sparql/sec-prop#requiresInteractivity";
/// `secx:requiresAssurance` — over the `AssuranceLevel` dimension. The conservative
/// gate: `requiresAssurance gteq secx:Proven` mechanically removes every unaudited
/// sparq method from the admissible set (`sq-qhy4` enters the data flow).
pub const REQUIRES_ASSURANCE: &str = "https://w3id.org/zkp-sparql/sec-prop#requiresAssurance";

// ── the dimension IRIs (the `secx:overDimension` targets) ────────────────────

/// `secx:UnlinkabilityScope` dimension.
pub const DIM_UNLINKABILITY_SCOPE: &str = "https://w3id.org/zkp-sparql/sec-prop#UnlinkabilityScope";
/// `secx:UnlinkabilityStrength` dimension.
pub const DIM_UNLINKABILITY_STRENGTH: &str =
    "https://w3id.org/zkp-sparql/sec-prop#UnlinkabilityStrength";
/// `secx:PostQuantumForgery` dimension (a vendored `sec-prop:` property).
pub const DIM_POST_QUANTUM_FORGERY: &str =
    "https://w3id.org/zkp-sparql/sec-prop#PostQuantumForgery";
/// `secx:PostQuantumSnooping` dimension (a vendored `sec-prop:` property).
pub const DIM_POST_QUANTUM_SNOOPING: &str =
    "https://w3id.org/zkp-sparql/sec-prop#PostQuantumSnooping";
/// `secx:ZeroKnowledgeType` dimension.
pub const DIM_ZERO_KNOWLEDGE_TYPE: &str = "https://w3id.org/zkp-sparql/sec-prop#ZeroKnowledgeType";
/// `secx:Soundness` dimension.
pub const DIM_SOUNDNESS: &str = "https://w3id.org/zkp-sparql/sec-prop#Soundness";
/// `secx:Completeness` dimension.
pub const DIM_COMPLETENESS: &str = "https://w3id.org/zkp-sparql/sec-prop#Completeness";
/// `secx:Hiding` dimension.
pub const DIM_HIDING: &str = "https://w3id.org/zkp-sparql/sec-prop#Hiding";
/// `secx:Binding` dimension.
pub const DIM_BINDING: &str = "https://w3id.org/zkp-sparql/sec-prop#Binding";
/// `secx:Anonymity` dimension.
pub const DIM_ANONYMITY: &str = "https://w3id.org/zkp-sparql/sec-prop#Anonymity";
/// `secx:SelectiveDisclosure` dimension.
pub const DIM_SELECTIVE_DISCLOSURE: &str =
    "https://w3id.org/zkp-sparql/sec-prop#SelectiveDisclosure";
/// `secx:SingleUse` dimension.
pub const DIM_SINGLE_USE: &str = "https://w3id.org/zkp-sparql/sec-prop#SingleUse";
/// `secx:Setup` dimension.
pub const DIM_SETUP: &str = "https://w3id.org/zkp-sparql/sec-prop#Setup";
/// `secx:Interactivity` dimension.
pub const DIM_INTERACTIVITY: &str = "https://w3id.org/zkp-sparql/sec-prop#Interactivity";
/// `secx:AssuranceLevel` dimension (the epistemic-basis axis — design §4.2.2).
pub const DIM_ASSURANCE_LEVEL: &str = "https://w3id.org/zkp-sparql/sec-prop#AssuranceLevel";

/// The complete `(leftOperand, dimension)` map of the sparq security-property
/// profile — one entry per requireable dimension, in declaration order. The **single
/// source of truth** the recogniser/lookup and the TTL drift tests iterate.
///
/// Each pair is the `secx:requiresX secx:overDimension secx:Dim` fact the profile
/// declares: a constraint over the leftOperand `.0` ranges over the security-property
/// dimension `.1`.
pub const SECPROP_LEFT_OPERANDS: &[(&str, &str)] = &[
    (REQUIRES_UNLINKABILITY_SCOPE, DIM_UNLINKABILITY_SCOPE),
    (REQUIRES_UNLINKABILITY_STRENGTH, DIM_UNLINKABILITY_STRENGTH),
    (REQUIRES_POST_QUANTUM_FORGERY, DIM_POST_QUANTUM_FORGERY),
    (REQUIRES_POST_QUANTUM_SNOOPING, DIM_POST_QUANTUM_SNOOPING),
    (REQUIRES_ZERO_KNOWLEDGE, DIM_ZERO_KNOWLEDGE_TYPE),
    (REQUIRES_SOUNDNESS, DIM_SOUNDNESS),
    (REQUIRES_COMPLETENESS, DIM_COMPLETENESS),
    (REQUIRES_HIDING, DIM_HIDING),
    (REQUIRES_BINDING, DIM_BINDING),
    (REQUIRES_ANONYMITY, DIM_ANONYMITY),
    (REQUIRES_SELECTIVE_DISCLOSURE, DIM_SELECTIVE_DISCLOSURE),
    (REQUIRES_SINGLE_USE, DIM_SINGLE_USE),
    (REQUIRES_SETUP, DIM_SETUP),
    (REQUIRES_INTERACTIVITY, DIM_INTERACTIVITY),
    (REQUIRES_ASSURANCE, DIM_ASSURANCE_LEVEL),
];

/// True iff `left_operand` is a `secx:requires…` leftOperand this profile declares
/// (i.e. a [`crate::Constraint::left`] the security-property bridge recognises).
///
/// A policy carrying a recognised leftOperand SHOULD assert `odrl:profile
/// <PROFILE_IRI>`; the recogniser lets a host check that without re-parsing the
/// profile TTL.
pub fn is_secprop_left_operand(left_operand: &str) -> bool {
    SECPROP_LEFT_OPERANDS
        .iter()
        .any(|(lo, _)| *lo == left_operand)
}

/// The `secx:` security-property **dimension** IRI a `secx:requires…` leftOperand
/// ranges over (its `secx:overDimension`), or `None` if `left_operand` is not a
/// leftOperand of this profile.
///
/// This is the one fact the admissibility reduction (Phase 2) reads: a constraint
/// over `left_operand` is discharged by a method whose asserted level for
/// `over_dimension(left_operand)` meets the constraint's right-operand level.
pub fn over_dimension(left_operand: &str) -> Option<&'static str> {
    SECPROP_LEFT_OPERANDS
        .iter()
        .find(|(lo, _)| *lo == left_operand)
        .map(|(_, dim)| *dim)
}

/// The deontic force of the rule a [`DischargeObligation`] was extracted from — which
/// way discharging it moves the decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Deontic {
    /// The obligation sits on an `odrl:permission`: the permission grants only while
    /// **every** one of its constraints holds, so this is a property a presentation
    /// must establish for access to be granted.
    Permission,
    /// The obligation sits on an `odrl:prohibition`: a prohibition **fires** (and
    /// deny-overrides the decision — see [`crate::evaluate`]) when its constraints
    /// hold. Establishing this property therefore *withholds* access; a host must not
    /// treat it as something to satisfy.
    Prohibition,
}

/// One security-property **proof-discharge obligation**: a single `secx:requires…`
/// constraint found on a policy rule, resolved to the dimension it ranges over.
///
/// This is a faithful re-projection of already-parsed [`crate::Constraint`] data — it
/// asserts nothing about any proof, method, or cryptosuite (see the module docs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeObligation {
    /// The [`crate::Rule::id`] of the rule carrying the constraint (for justification).
    pub rule: String,
    /// Whether that rule is a permission or a prohibition — see [`Deontic`].
    pub deontic: Deontic,
    /// The `secx:requires…` [`crate::Constraint::left`] IRI.
    pub left_operand: String,
    /// The `secx:` dimension the leftOperand ranges over — its
    /// [`over_dimension`] (`secx:overDimension`).
    pub dimension: &'static str,
    /// The constraint's [`crate::Operator`] (`odrl:gteq` for the usual "at least this
    /// level" preference).
    pub operator: Operator,
    /// The constraint's right operand — the required level.
    pub required: Value,
}

/// Every security-property proof-discharge obligation a policy carries, in rule order
/// (permissions before prohibitions), nested compound constraints included.
///
/// A host uses this to answer *"before I can decide this request, what would a
/// presented proof have to establish?"* — the ODRL half of the ZK↔ODRL
/// constraint-discharge envelope (`sq-yh427`). An empty result means the policy asks
/// for no security property, so no presentation is needed.
///
/// **Scope, stated honestly.** Extraction is pure projection: it verifies nothing,
/// orders no levels, and changes no evaluation semantics. It also **excludes duty
/// constraints** — the single-node base case checks a [`crate::Duty`] by *action*
/// discharge and never evaluates the duty's own constraints (see [`crate::Duty`]), so
/// reporting them as obligations the evaluator will check would misdescribe it.
///
/// ```
/// # // cargo: sparq-policy with --features secprop-leftoperands
/// use sparq_policy::secprop::{discharge_obligations, Deontic, REQUIRES_ASSURANCE};
/// use sparq_policy::{Action, Constraint, Operator, Policy, Rule, Value, ODRL_NS};
///
/// let policy = Policy {
///     iri: None,
///     permissions: vec![Rule {
///         id: "urn:rule:1".into(),
///         action: Action(format!("{}read", ODRL_NS)),
///         target: None,
///         assignee: None,
///         assigner: None,
///         constraints: vec![Constraint {
///             left: REQUIRES_ASSURANCE.into(),
///             operator: Operator::Gteq,
///             right: Value::Iri("https://w3id.org/zkp-sparql/sec-prop#Proven".into()),
///         }],
///         logical_constraints: vec![],
///         duties: vec![],
///     }],
///     prohibitions: vec![],
///     conflict: None,
/// };
///
/// let obligations = discharge_obligations(&policy);
/// assert_eq!(obligations.len(), 1);
/// assert_eq!(obligations[0].deontic, Deontic::Permission);
/// assert_eq!(
///     obligations[0].dimension,
///     "https://w3id.org/zkp-sparql/sec-prop#AssuranceLevel"
/// );
/// ```
pub fn discharge_obligations(policy: &Policy) -> Vec<DischargeObligation> {
    let mut out = Vec::new();
    for rule in &policy.permissions {
        collect_rule(rule, Deontic::Permission, &mut out);
    }
    for rule in &policy.prohibitions {
        collect_rule(rule, Deontic::Prohibition, &mut out);
    }
    out
}

/// Collect a rule's atomic constraints, then its compound ones.
fn collect_rule(rule: &Rule, deontic: Deontic, out: &mut Vec<DischargeObligation>) {
    for c in &rule.constraints {
        push_if_secprop(&rule.id, deontic, c, out);
    }
    for lc in &rule.logical_constraints {
        collect_logical(&rule.id, deontic, lc, out);
    }
}

/// Walk an `odrl:LogicalConstraint` tree, collecting its atomic leaves. A compound
/// constraint's combinator (`and`/`or`/`xone`) does not change *what a proof would
/// have to establish* — only how the leaves fold into the rule's verdict — so every
/// leaf is reported and the combinator is left to [`crate::evaluate`].
fn collect_logical(
    rule_id: &str,
    deontic: Deontic,
    lc: &LogicalConstraint,
    out: &mut Vec<DischargeObligation>,
) {
    for operand in &lc.operands {
        match operand {
            ConstraintNode::Atomic(c) => push_if_secprop(rule_id, deontic, c, out),
            ConstraintNode::Compound(inner) => collect_logical(rule_id, deontic, inner, out),
        }
    }
}

/// Record `c` as an obligation iff its leftOperand is one this profile declares.
fn push_if_secprop(
    rule_id: &str,
    deontic: Deontic,
    c: &Constraint,
    out: &mut Vec<DischargeObligation>,
) {
    let Some(dimension) = over_dimension(&c.left) else {
        return; // a core ODRL (or unrelated custom) leftOperand — not a proof obligation
    };
    out.push(DischargeObligation {
        rule: rule_id.to_owned(),
        deontic,
        left_operand: c.left.clone(),
        dimension,
        operator: c.operator,
        required: c.right.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The machine-readable `odrl-secprop-profile.ttl` is the canonical form; keep it
    /// pinned to these constants so the two cannot drift (the
    /// `crates/sparq-trust/src/secprop.rs` discipline).
    const TTL: &str = include_str!("../ontologies/odrl-secprop-profile.ttl");

    /// Every leftOperand and every dimension IRI is in the `secx:` namespace.
    #[test]
    fn all_iris_share_the_secx_namespace() {
        for (lo, dim) in SECPROP_LEFT_OPERANDS {
            assert!(
                lo.starts_with(SECX_NS),
                "leftOperand `{}` is not in the secx: namespace",
                lo
            );
            assert!(
                dim.starts_with(SECX_NS),
                "dimension `{}` is not in the secx: namespace",
                dim
            );
        }
        assert!(OVER_DIMENSION.starts_with(SECX_NS));
    }

    /// No duplicate leftOperand, and no two leftOperands map to the same dimension
    /// (one leftOperand per requireable dimension — a copy-paste guard).
    #[test]
    fn no_duplicate_left_operand_or_dimension() {
        let mut seen_lo = std::collections::HashSet::new();
        let mut seen_dim = std::collections::HashSet::new();
        for (lo, dim) in SECPROP_LEFT_OPERANDS {
            assert!(seen_lo.insert(*lo), "duplicate leftOperand: {}", lo);
            assert!(
                seen_dim.insert(*dim),
                "two leftOperands map to dimension {}",
                dim
            );
        }
    }

    /// The recogniser + lookup agree with the registry (and reject an unrelated IRI).
    #[test]
    fn recogniser_and_lookup_agree_with_registry() {
        for (lo, dim) in SECPROP_LEFT_OPERANDS {
            assert!(is_secprop_left_operand(lo), "recogniser missed {}", lo);
            assert_eq!(over_dimension(lo), Some(*dim), "wrong dimension for {}", lo);
        }
        // A core ODRL leftOperand and a nonsense IRI are NOT secprop leftOperands.
        assert!(!is_secprop_left_operand(crate::ODRL_PURPOSE));
        assert!(!is_secprop_left_operand(
            "https://example.org/notAProfileTerm"
        ));
        assert_eq!(over_dimension(crate::ODRL_PURPOSE), None);
    }

    // ── discharge-obligation extraction (sq-yh427, the ODRL half) ────────────

    /// A permission rule carrying `constraints`, for the extraction tests.
    fn rule_with(id: &str, constraints: Vec<Constraint>) -> Rule {
        Rule {
            id: id.into(),
            action: crate::Action(format!("{}read", crate::ODRL_NS)),
            target: None,
            assignee: None,
            assigner: None,
            constraints,
            logical_constraints: vec![],
            duties: vec![],
        }
    }

    fn secprop_constraint(left: &str, level: &str) -> Constraint {
        Constraint {
            left: left.into(),
            operator: Operator::Gteq,
            right: Value::Iri(format!("{}{}", SECX_NS, level)),
        }
    }

    /// A `secx:requires…` constraint is extracted with its dimension, operator and
    /// required level; a core ODRL constraint alongside it is NOT (it is not a
    /// property a proof discharges).
    #[test]
    fn extracts_secprop_constraints_and_ignores_core_left_operands() {
        let policy = Policy {
            iri: None,
            permissions: vec![rule_with(
                "urn:rule:1",
                vec![
                    secprop_constraint(REQUIRES_ASSURANCE, "Proven"),
                    Constraint {
                        left: crate::ODRL_PURPOSE.into(),
                        operator: Operator::Eq,
                        right: Value::Iri("https://w3id.org/dpv#ResearchAndDevelopment".into()),
                    },
                ],
            )],
            prohibitions: vec![],
            conflict: None,
        };

        let obligations = discharge_obligations(&policy);
        assert_eq!(obligations.len(), 1, "only the secx: constraint is an obligation");
        let o = &obligations[0];
        assert_eq!(o.rule, "urn:rule:1");
        assert_eq!(o.deontic, Deontic::Permission);
        assert_eq!(o.left_operand, REQUIRES_ASSURANCE);
        assert_eq!(o.dimension, DIM_ASSURANCE_LEVEL);
        assert_eq!(o.operator, Operator::Gteq);
        assert_eq!(
            o.required,
            Value::Iri(format!("{}Proven", SECX_NS)),
            "the required level is carried verbatim",
        );
    }

    /// A policy with no `secx:` constraint yields no obligations — the host needs no
    /// presentation at all (and an empty policy likewise).
    #[test]
    fn no_secprop_constraints_yields_no_obligations() {
        assert!(discharge_obligations(&Policy::default()).is_empty());
        let policy = Policy {
            iri: None,
            permissions: vec![rule_with("urn:rule:1", vec![])],
            prohibitions: vec![],
            conflict: None,
        };
        assert!(discharge_obligations(&policy).is_empty());
    }

    /// LOAD-BEARING: an obligation on a PROHIBITION is reported as such. Establishing
    /// it makes the prohibition fire (deny-overrides), so a host must never read it as
    /// "a property to satisfy".
    #[test]
    fn prohibition_obligations_carry_the_prohibition_deontic() {
        let policy = Policy {
            iri: None,
            permissions: vec![rule_with(
                "urn:perm:1",
                vec![secprop_constraint(REQUIRES_ZERO_KNOWLEDGE, "PerfectZK")],
            )],
            prohibitions: vec![rule_with(
                "urn:proh:1",
                vec![secprop_constraint(REQUIRES_SINGLE_USE, "Nullifier")],
            )],
            conflict: None,
        };

        let obligations = discharge_obligations(&policy);
        assert_eq!(obligations.len(), 2);
        // Permissions are reported before prohibitions.
        assert_eq!(obligations[0].deontic, Deontic::Permission);
        assert_eq!(obligations[0].rule, "urn:perm:1");
        assert_eq!(obligations[1].deontic, Deontic::Prohibition);
        assert_eq!(obligations[1].rule, "urn:proh:1");
        assert_eq!(obligations[1].dimension, DIM_SINGLE_USE);
    }

    /// Constraints nested inside `odrl:LogicalConstraint` compounds (including a
    /// compound nested in a compound) are collected too — a preference written as an
    /// `or` of `and`s must not silently drop its obligations.
    #[test]
    fn collects_obligations_nested_in_logical_constraints() {
        let inner = LogicalConstraint {
            id: "urn:lc:inner".into(),
            operator: crate::LogicalOperator::And,
            operands: vec![
                ConstraintNode::Atomic(secprop_constraint(REQUIRES_HIDING, "PerfectHiding")),
                ConstraintNode::Atomic(Constraint {
                    left: crate::ODRL_RECIPIENT.into(),
                    operator: Operator::Eq,
                    right: Value::Iri("https://bob.example/profile#me".into()),
                }),
            ],
        };
        let outer = LogicalConstraint {
            id: "urn:lc:outer".into(),
            operator: crate::LogicalOperator::Or,
            operands: vec![
                ConstraintNode::Compound(inner),
                ConstraintNode::Atomic(secprop_constraint(REQUIRES_SOUNDNESS, "Statistical")),
            ],
        };
        let mut rule = rule_with("urn:rule:1", vec![]);
        rule.logical_constraints = vec![outer];

        let policy = Policy {
            iri: None,
            permissions: vec![rule],
            prohibitions: vec![],
            conflict: None,
        };

        let dims: Vec<&str> = discharge_obligations(&policy)
            .iter()
            .map(|o| o.dimension)
            .collect();
        assert_eq!(
            dims,
            vec![DIM_HIDING, DIM_SOUNDNESS],
            "both nested secx: leaves are reported, the core recipient leaf is not",
        );
    }

    /// Duty constraints are deliberately NOT reported: the base-case evaluator checks a
    /// duty by ACTION discharge and never evaluates the duty's own constraints, so
    /// reporting them would misdescribe what the evaluator will check.
    #[test]
    fn duty_constraints_are_not_reported_as_obligations() {
        let mut rule = rule_with("urn:rule:1", vec![]);
        rule.duties = vec![crate::Duty {
            id: "urn:duty:1".into(),
            action: crate::Action(format!("{}anonymize", crate::ODRL_NS)),
            constraints: vec![secprop_constraint(REQUIRES_ANONYMITY, "Anonymous")],
        }];
        let policy = Policy {
            iri: None,
            permissions: vec![rule],
            prohibitions: vec![],
            conflict: None,
        };
        assert!(
            discharge_obligations(&policy).is_empty(),
            "a duty constraint is not an obligation the evaluator ever checks",
        );
    }

    /// Every Rust leftOperand is declared in the profile TTL as a `secx:LocalName`
    /// subject with its `secx:overDimension` target — the published profile and these
    /// constants cannot diverge.
    #[test]
    fn ttl_declares_every_left_operand_with_its_dimension() {
        for (lo, dim) in SECPROP_LEFT_OPERANDS {
            let lo_local = lo
                .strip_prefix(SECX_NS)
                .expect("leftOperand is in the secx: namespace");
            let dim_local = dim
                .strip_prefix(SECX_NS)
                .expect("dimension is in the secx: namespace");
            // `secx:requiresFoo a owl:NamedIndividual, odrl:LeftOperand, skos:Concept`
            let decl = format!(
                "secx:{} a owl:NamedIndividual, odrl:LeftOperand, skos:Concept",
                lo_local
            );
            assert!(
                TTL.contains(&decl),
                "odrl-secprop-profile.ttl is missing the leftOperand declaration `{}` \
                 (the profile drifted from the Rust constants, sq-uor3g)",
                decl
            );
            // `secx:overDimension secx:Dim`
            let over = format!("secx:overDimension secx:{}", dim_local);
            assert!(
                TTL.contains(&over),
                "odrl-secprop-profile.ttl is missing the overDimension fact `{}` for \
                 leftOperand secx:{} (sq-uor3g)",
                over,
                lo_local
            );
        }
    }

    /// The inverse: every `secx:requires…` leftOperand DECLARED in the TTL is backed
    /// by a Rust constant — no published leftOperand without a constant.
    #[test]
    fn every_ttl_left_operand_has_a_rust_constant() {
        let declared: Vec<&str> = TTL
            .lines()
            .filter_map(|line| {
                let t = line.trim_start();
                let rest = t.strip_prefix("secx:requires")?;
                let name = rest.split_whitespace().next()?;
                // A leftOperand declaration line: `secx:requiresFoo a owl:NamedIndividual …`
                if rest[name.len()..].trim_start().starts_with("a ") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            !declared.is_empty(),
            "no secx:requires… leftOperand declarations found in the profile TTL — \
             parser/format drift",
        );
        let constant_locals: std::collections::HashSet<&str> = SECPROP_LEFT_OPERANDS
            .iter()
            .filter_map(|(lo, _)| lo.strip_prefix(&format!("{}requires", SECX_NS)))
            .collect();
        for name in declared {
            assert!(
                constant_locals.contains(name),
                "odrl-secprop-profile.ttl declares `secx:requires{}` but no Rust constant \
                 names it — add the constant or remove the term (sq-uor3g)",
                name
            );
        }
    }

    /// The profile resource + the `odrl:profile` assertion contract are documented in
    /// the TTL — the load-bearing ODRL nuance (a custom leftOperand MUST be declared
    /// in a published profile, design §4.3.1).
    #[test]
    fn profile_resource_and_contract_are_documented() {
        assert!(
            TTL.contains("a odrl:Profile"),
            "the profile TTL must declare the odrl:Profile resource",
        );
        assert!(
            TTL.contains(PROFILE_IRI),
            "the profile TTL must use the published PROFILE_IRI",
        );
        assert!(
            TTL.contains("odrl:profile"),
            "the profile TTL must document the `odrl:profile <…>` assertion contract",
        );
    }

    /// The HONESTY invariant: the profile asserts NO security property and references
    /// the open external-audit gate (sq-qhy4) — this bridge names the bar, it does not
    /// claim any method meets it.
    #[test]
    fn honesty_caveat_is_present() {
        assert!(
            TTL.contains("sq-qhy4"),
            "the profile TTL must reference the open external-audit gate (sq-qhy4)",
        );
        assert!(
            TTL.contains("NO security claim") || TTL.contains("NO security property"),
            "the profile TTL must state it asserts no security property of any method",
        );
    }

    // ── cross-crate dimension-IRI drift guards (sq-mgxz8) ────────────────────
    //
    // Three crates independently declare `secx:` dimension IRIs as `const &str`
    // data with no crate-dependency edge between them:
    //   (1) THIS file — the `DIM_*` constants in `SECPROP_LEFT_OPERANDS`
    //   (2) `sparq-trust/src/secprop.rs` — the full `SECX_*` vocabulary
    //   (3) `sparq-zk/ontologies/secprop-methods.ttl` — the per-method annotation
    //       graph (`secx:property secx:Foo` triples)
    //
    // Each has its own per-crate TTL↔Rust drift test, but a typo in ONE crate's
    // dimension IRI would NOT be caught against the others. These two tests fill
    // that gap without adding a crate-dependency edge: they use `include_str!`
    // (a compile-time file read) to compare across crate boundaries.
    //
    // `sec-prop:` VENDORED DIMS: `PostQuantumForgery`, `PostQuantumSnooping`, and
    // `SignatureTypeLeakage` are original vendored class IRIs from the ZKP-SPARQL
    // paper (`vocab/sec-prop.yaml.ld`). They appear under the same `secx:` namespace
    // and are USED as dimension IRIs throughout the estate, but `secprop-ext.ttl`
    // does NOT re-declare them as subjects (the non-forking design §4.1 keeps only
    // the LEVELS there). They are exempted from the subject-declaration check and
    // receive a namespace check only.

    /// Local names of `sec-prop:` property IRIs that are VENDORED from the original
    /// ZKP-SPARQL vocabulary and therefore NOT re-declared as new `secx:X a …`
    /// subjects in `secprop-ext.ttl`. Their levels are added in the extension file
    /// but their own class IRIs are kept in the original vocabulary.
    const VENDORED_SEC_PROP_DIMS: &[&str] = &[
        "PostQuantumForgery",
        "PostQuantumSnooping",
        "SignatureTypeLeakage",
    ];

    /// Cross-crate dimension-IRI drift guard (sq-mgxz8): every `DIM_*` in
    /// `SECPROP_LEFT_OPERANDS` must be declared as a `secx:LocalName` subject in
    /// the canonical `secprop-ext.ttl` owned by `sparq-trust`, UNLESS its local
    /// name is in `VENDORED_SEC_PROP_DIMS` (vendored class IRIs that are exempt
    /// from the subject-declaration check — see comment block above). Uses
    /// `include_str!` — a compile-time read, NOT a crate-dependency edge.
    #[test]
    fn policy_dim_iris_are_in_trust_vocab_or_vendored() {
        const TRUST_VOCAB: &str =
            include_str!("../../sparq-trust/ontologies/zkp-sparql/secprop-ext.ttl");

        for (_, dim) in SECPROP_LEFT_OPERANDS {
            let dim_local = dim
                .strip_prefix(SECX_NS)
                .expect("dimension is in the secx: namespace");

            if VENDORED_SEC_PROP_DIMS.contains(&dim_local) {
                // Vendored class IRI: only the namespace can be checked here
                // (the class declaration is in sec-prop.yaml.ld, not secprop-ext.ttl).
                assert!(
                    dim.starts_with(SECX_NS),
                    "vendored dimension `secx:{}` must be in the secx: namespace",
                    dim_local,
                );
                continue;
            }

            // Extension term: must be declared as `secx:LocalName a …` in the
            // trust-crate vocabulary so a rename in the ext TTL is caught here.
            let decl = format!("secx:{} ", dim_local);
            assert!(
                TRUST_VOCAB.contains(&decl),
                "dimension `secx:{}` in SECPROP_LEFT_OPERANDS is not declared as a \
                 subject in sparq-trust/ontologies/zkp-sparql/secprop-ext.ttl — \
                 IRI drift between the policy constants and the trust vocabulary \
                 (sq-mgxz8); check for a rename or a missing declaration",
                dim_local,
            );
        }
    }

    /// Minimal Turtle-aware tokenizer for the drift guards below: drops `#`
    /// comments (outside `<…>` IRIs and `"…"` literals), splits on whitespace,
    /// and emits `[ ] ( ) ; ,` as their own tokens — so a predicate and its
    /// object are ADJACENT tokens regardless of line breaks, indentation, or
    /// interleaved comments. NOT a Turtle parser (no long strings, no escapes
    /// beyond `\"`); just enough to make the scan formatting-independent.
    fn turtle_tokens(src: &str) -> Vec<&str> {
        let bytes = src.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                c if c.is_ascii_whitespace() => i += 1,
                b'#' => {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
                b'<' => {
                    let start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'>' {
                        i += 1;
                    }
                    i = (i + 1).min(bytes.len());
                    tokens.push(&src[start..i]);
                }
                b'"' => {
                    let start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        i += if bytes[i] == b'\\' { 2 } else { 1 };
                    }
                    i = (i + 1).min(bytes.len());
                    tokens.push(&src[start..i]);
                }
                b'[' | b']' | b'(' | b')' | b';' | b',' => {
                    tokens.push(&src[i..i + 1]);
                    i += 1;
                }
                _ => {
                    let start = i;
                    while i < bytes.len()
                        && !bytes[i].is_ascii_whitespace()
                        && !matches!(bytes[i], b'#' | b'[' | b']' | b'(' | b')' | b';' | b',')
                    {
                        i += 1;
                    }
                    tokens.push(&src[start..i]);
                }
            }
        }
        tokens
    }

    /// The object token of every `secx:property` assertion in `src`, in order.
    /// Token-based (see [`turtle_tokens`]), so `secx:property\n  # note\n  secx:X`
    /// is found exactly like the single-line form, and a commented-out
    /// annotation is NOT.
    fn secx_property_objects(src: &str) -> Vec<&str> {
        let tokens = turtle_tokens(src);
        tokens
            .windows(2)
            .filter(|pair| pair[0] == "secx:property")
            .map(|pair| pair[1])
            .collect()
    }

    /// Cross-crate dimension-IRI drift guard (sq-mgxz8): every `secx:property
    /// secx:*` dimension IRI in `sparq-zk/ontologies/secprop-methods.ttl` must be
    /// one of: a policy `DIM_*` local name (in `SECPROP_LEFT_OPERANDS`), a known
    /// vendored dimension (`VENDORED_SEC_PROP_DIMS`), or a `secx:LocalName` subject
    /// declared in `secprop-ext.ttl`. A typo or rename in the annotation graph's
    /// dimension IRI fails at least one condition. Uses `include_str!` — no crate
    /// edge.
    #[test]
    fn methods_ttl_dims_are_policy_or_trust_vocab_or_vendored() {
        const TRUST_VOCAB: &str =
            include_str!("../../sparq-trust/ontologies/zkp-sparql/secprop-ext.ttl");
        const METHODS: &str =
            include_str!("../../sparq-zk/ontologies/secprop-methods.ttl");

        let policy_dim_locals: std::collections::HashSet<&str> = SECPROP_LEFT_OPERANDS
            .iter()
            .filter_map(|(_, dim)| dim.strip_prefix(SECX_NS))
            .collect();

        let mut found = 0usize;
        for object in secx_property_objects(METHODS) {
            // A non-`secx:` object (e.g. the full-IRI form) must fail LOUDLY,
            // not fall out of the scan: silent skips are exactly the drift this
            // guard exists to catch.
            let local = object.strip_prefix("secx:").unwrap_or_else(|| {
                panic!(
                    "secprop-methods.ttl has a `secx:property` object `{}` that is \
                     not in `secx:LocalName` prefix form — this drift guard only \
                     understands prefixed dimension IRIs (sq-mgxz8); either keep \
                     the annotation graph in prefix form or extend the guard",
                    object,
                )
            });
            found += 1;

            if policy_dim_locals.contains(local) || VENDORED_SEC_PROP_DIMS.contains(&local) {
                continue;
            }

            // Not a policy dimension and not a known vendored one: must be declared
            // in the extension vocab to be a verifiable term, not an undetected typo.
            let decl = format!("secx:{} ", local);
            assert!(
                TRUST_VOCAB.contains(&decl),
                "secprop-methods.ttl uses `secx:property secx:{}` but this IRI is \
                 neither a policy dimension (SECPROP_LEFT_OPERANDS), a known vendored \
                 sec-prop: dimension (VENDORED_SEC_PROP_DIMS), nor declared as a subject \
                 in sparq-trust's secprop-ext.ttl — possible IRI drift or undeclared \
                 term (sq-mgxz8); if this is intentional, add it to VENDORED_SEC_PROP_DIMS",
                local,
            );
        }

        assert!(
            found > 0,
            "no `secx:property secx:*` dimension values found in secprop-methods.ttl — \
             the methods TTL format may have changed, breaking this drift guard (sq-mgxz8)",
        );
    }

    /// Regression (PR #3440 review): the drift-guard scan must be Turtle-
    /// formatting-independent. A reformatted annotation with the object on its
    /// own line behind a comment — plus a typo'd dimension name — MUST still be
    /// surfaced to the validation loop; the old exact-text `secx:property secx:`
    /// scan silently skipped it.
    #[test]
    fn secx_property_scan_survives_multiline_and_comment_formatting() {
        let ttl = "[ secx:property # the dimension\n      secx:TypoDim ;\n\
                   secx:level secx:Sound ] .\n\
                   [ secx:property secx:Soundness ; secx:level secx:Sound ] .";
        assert_eq!(
            secx_property_objects(ttl),
            vec!["secx:TypoDim", "secx:Soundness"],
            "a multiline/commented `secx:property` annotation must be found by \
             the scan exactly like the single-line form (sq-mgxz8)",
        );
    }

    /// Regression (PR #3440 review): a commented-out annotation is NOT a
    /// dimension assertion and must not be counted by the scan.
    #[test]
    fn secx_property_scan_ignores_comments_and_iris() {
        let ttl = "# [ secx:property secx:CommentedOut ] .\n\
                   <http://example.org/x#secx:property> a secx:Thing .\n\
                   [ secx:property secx:Real ] .";
        assert_eq!(
            secx_property_objects(ttl),
            vec!["secx:Real"],
            "commented-out annotations and IRI-internal text must not count as \
             `secx:property` assertions (sq-mgxz8)",
        );
    }
}
