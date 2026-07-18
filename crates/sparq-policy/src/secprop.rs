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
        let mut remaining = METHODS;
        while let Some(pos) = remaining.find("secx:property secx:") {
            let after = &remaining[pos + "secx:property secx:".len()..];
            let local = after
                .split(|c: char| !c.is_alphanumeric())
                .next()
                .unwrap_or("");
            // Advance before assertions so the loop always terminates.
            remaining = &remaining[pos + 1..];
            if local.is_empty() {
                continue;
            }
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
}
