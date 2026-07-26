// [OPUS-4.8] sq-314: derivation_step module + entailmentRegime end-to-end.
//! Derivation steps: the recorded inference that justifies a DERIVED triple under
//! a non-`Simple` entailment regime (plan §S2.5 with-inference).
//!
//! # Why this module exists (the sq-314 soundness gap it closes)
//! `ProofManifest::entailment_regime` used to be FREE METADATA: the verifier never
//! checked it, so a manifest could claim `Rdfs`/`Owl` while the proof attested only
//! the asserted triples (`Simple` semantics). A relying party reading the regime
//! field would believe inference was applied when nothing established it. This
//! module + the verifier's `crate::verifier::bind_entailment` make the regime
//! ENFORCED:
//! - the relying party declares which regimes it accepts (`EntailmentPolicy`);
//! - a regime the policy does not accept REJECTS (fail-closed);
//! - a non-`Simple` regime REQUIRES `derivation_steps` that STRUCTURALLY justify
//!   every derived triple against the regime's rule set, re-checked verifier-side.
//!
//! # Scope (HONEST — what is and is NOT proved)
//! A [`DerivationStep`] records a single rule application: the rule, its antecedent
//! triples, and the derived triple. The verifier STRUCTURALLY re-checks that each
//! step is a valid instance of its rule and that the derived triple chains from
//! antecedents that are either themselves derived by an earlier step or are
//! ASSERTED (disclosed by a scan sub-proof). This makes the regime claim
//! non-vacuous and auditable.
//!
//! What is DEFERRED (documented, not silently assumed): a ZERO-KNOWLEDGE proof
//! that each antecedent triple is in the committed graph's closure — i.e. an
//! in-circuit inference proof. v1 ties antecedents to DISCLOSED scan rows (the
//! asserted base) by encoding-equality, so a derivation over disclosed triples is
//! soundly re-checkable; antecedents that are themselves only claimed (not
//! disclosed and not chained to disclosed triples) are NOT accepted. So the
//! capability is sound for the disclosed-base fragment and fail-closed otherwise.
//! The full in-circuit RDFS/OWL-RL closure proof is the inference-circuit
//! deliverable (plan §S2.5), tracked separately.

use serde::{Deserialize, Serialize};

use crate::manifest::{EntailmentRegime, FieldHex};

/// The RDFS/OWL-RL rule a [`DerivationStep`] instantiates. v1 ships the RDFS
/// rules whose antecedents are fixed-shape Datalog over term encodings (the
/// subset expressible over disclosed/committed triple encodings — the phased
/// RULE SCOPE of `research/zk-inference-and-credentials.md §3.5`); the enum is
/// the extension point for the full rule set. `owl:sameAs` is gated SEPARATELY
/// (`sq-rsd3v.7`) — encoding-equality re-checks are UNSOUND under equality
/// reasoning — and must never ride this fixed-shape path.
///
/// Each rule's antecedent/consequent SHAPE is fixed and re-checked by
/// [`DerivationStep::is_well_formed`]; the rule is identified in the manifest by
/// its kebab-case tag so the schema is stable as rules are added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntailmentRule {
    /// RDFS rule rdfs9 (subClassOf transitivity into type):
    /// `(?x rdf:type ?c1)` + `(?c1 rdfs:subClassOf ?c2)` ⊢ `(?x rdf:type ?c2)`.
    Rdfs9SubClassType,
    /// RDFS rule rdfs7 (subPropertyOf):
    /// `(?p1 rdfs:subPropertyOf ?p2)` + `(?x ?p1 ?y)` ⊢ `(?x ?p2 ?y)`.
    Rdfs7SubProperty,
    /// RDFS rule rdfs2 (property domain into type):
    /// `(?p rdfs:domain ?c)` + `(?x ?p ?y)` ⊢ `(?x rdf:type ?c)`.
    Rdfs2Domain,
    /// RDFS rule rdfs3 (property range into type):
    /// `(?p rdfs:range ?c)` + `(?x ?p ?y)` ⊢ `(?y rdf:type ?c)`.
    Rdfs3Range,
    /// RDFS rule rdfs5 (subPropertyOf transitivity):
    /// `(?p1 rdfs:subPropertyOf ?p2)` + `(?p2 rdfs:subPropertyOf ?p3)`
    /// ⊢ `(?p1 rdfs:subPropertyOf ?p3)`.
    Rdfs5SubPropertyTrans,
    /// RDFS rule rdfs11 (subClassOf transitivity):
    /// `(?c1 rdfs:subClassOf ?c2)` + `(?c2 rdfs:subClassOf ?c3)`
    /// ⊢ `(?c1 rdfs:subClassOf ?c3)`.
    Rdfs11SubClassTrans,
}

impl EntailmentRule {
    /// The minimum entailment regime under which this rule is admissible. All v1
    /// rules are RDFS rules, so they require at least `Rdfs` (and are also valid
    /// under `Owl`, which subsumes RDFS). A `Simple` manifest may carry NO
    /// derivation steps (no inference).
    pub fn min_regime(self) -> EntailmentRegime {
        match self {
            EntailmentRule::Rdfs9SubClassType
            | EntailmentRule::Rdfs7SubProperty
            | EntailmentRule::Rdfs2Domain
            | EntailmentRule::Rdfs3Range
            | EntailmentRule::Rdfs5SubPropertyTrans
            | EntailmentRule::Rdfs11SubClassTrans => EntailmentRegime::Rdfs,
        }
    }
}

/// Whether `regime` permits `rule` (the regime is at least the rule's minimum).
/// `Owl` subsumes `Rdfs`; `Simple` permits NO rule.
pub fn regime_admits(regime: EntailmentRegime, rule: EntailmentRule) -> bool {
    match (regime, rule.min_regime()) {
        (EntailmentRegime::Simple, _) => false,
        (EntailmentRegime::Rdfs, EntailmentRegime::Rdfs) => true,
        (EntailmentRegime::Owl, EntailmentRegime::Rdfs | EntailmentRegime::Owl) => true,
        // Owl rule under Rdfs regime, etc.
        _ => false,
    }
}

/// A single recorded inference step: a rule applied to `antecedents` yielding the
/// `derived` triple. Triples are carried as their per-slot TERM ENCODINGS (the
/// same `FieldHex` the scan sub-proofs disclose), so the verifier can tie an
/// antecedent to a disclosed scan row by encoding-equality.
///
/// The verifier re-checks (see `crate::verifier::bind_entailment`):
/// 1. the step is well-formed for its rule (arity + the rule's variable-sharing
///    shape — [`Self::is_well_formed`]);
/// 2. its regime admits the rule ([`regime_admits`]);
/// 3. every antecedent is GROUNDED: it equals either an earlier step's `derived`
///    triple or a triple disclosed by a scan sub-proof (the asserted base).
///
/// A step that fails any of these REJECTS the manifest (fail-closed) — a derived
/// triple cannot rest on an ungrounded antecedent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationStep {
    /// The rule this step instantiates.
    pub rule: EntailmentRule,
    /// The antecedent triples (term encodings), in the rule's declared order.
    pub antecedents: Vec<[FieldHex; 3]>,
    /// The derived triple (term encodings).
    pub derived: [FieldHex; 3],
}

impl DerivationStep {
    /// Whether this step is a WELL-FORMED instance of its rule: the antecedent
    /// arity and the rule's variable-sharing/consequent shape hold over the term
    /// encodings (an encoding equality is a term equality, so the shape check is
    /// sound over disclosed encodings). Returns `false` on any mismatch.
    ///
    /// Rule shapes (positions are [s, p, o]):
    /// - `Rdfs9SubClassType`: antecedents `[(x, type, c1), (c1, subClassOf, c2)]`,
    ///   derived `(x, type, c2)`. Checks: 2 antecedents; a0.p == rdf:type (the
    ///   `rdf_type` arg); a1.p == rdfs:subClassOf (`rdfs_subclassof`); a0.o == a1.s
    ///   (the shared class c1); derived == (a0.s, rdf:type, a1.o).
    /// - `Rdfs7SubProperty`: antecedents `[(p1, subPropertyOf, p2), (x, p1, y)]`,
    ///   derived `(x, p2, y)`. Checks: 2 antecedents; a0.p == rdfs:subPropertyOf;
    ///   a0.s == a1.p (the shared property p1); derived == (a1.s, a0.o, a1.o).
    /// - `Rdfs2Domain`: antecedents `[(p, domain, c), (x, p, y)]`, derived
    ///   `(x, type, c)`. Checks: 2 antecedents; a0.p == rdfs:domain; a0.s == a1.p
    ///   (the shared property p); derived == (a1.s, rdf:type, a0.o).
    /// - `Rdfs3Range`: antecedents `[(p, range, c), (x, p, y)]`, derived
    ///   `(y, type, c)`. Checks: 2 antecedents; a0.p == rdfs:range; a0.s == a1.p
    ///   (the shared property p); derived == (a1.o, rdf:type, a0.o).
    /// - `Rdfs5SubPropertyTrans`: antecedents
    ///   `[(p1, subPropertyOf, p2), (p2, subPropertyOf, p3)]`, derived
    ///   `(p1, subPropertyOf, p3)`. Checks: 2 antecedents; a0.p == a1.p ==
    ///   rdfs:subPropertyOf; a0.o == a1.s (the shared property p2); derived ==
    ///   (a0.s, subPropertyOf, a1.o).
    /// - `Rdfs11SubClassTrans`: antecedents
    ///   `[(c1, subClassOf, c2), (c2, subClassOf, c3)]`, derived
    ///   `(c1, subClassOf, c3)`. Checks: 2 antecedents; a0.p == a1.p ==
    ///   rdfs:subClassOf; a0.o == a1.s (the shared class c2); derived ==
    ///   (a0.s, subClassOf, a1.o).
    ///
    /// The schema-term encodings (`rdf:type`, `rdfs:subClassOf`,
    /// `rdfs:subPropertyOf`, `rdfs:domain`, `rdfs:range`) are supplied by the
    /// caller (the verifier computes them once via `encode_term`) so this module
    /// stays free of the encoding layer.
    pub fn is_well_formed(
        &self,
        rdf_type: &FieldHex,
        rdfs_subclassof: &FieldHex,
        rdfs_subpropertyof: &FieldHex,
        rdfs_domain: &FieldHex,
        rdfs_range: &FieldHex,
    ) -> bool {
        match self.rule {
            EntailmentRule::Rdfs9SubClassType => {
                if self.antecedents.len() != 2 {
                    return false;
                }
                let a0 = &self.antecedents[0]; // (x, type, c1)
                let a1 = &self.antecedents[1]; // (c1, subClassOf, c2)
                a0[1] == *rdf_type
                    && a1[1] == *rdfs_subclassof
                    && a0[2] == a1[0] // shared c1
                    && self.derived[0] == a0[0] // x
                    && self.derived[1] == *rdf_type
                    && self.derived[2] == a1[2] // c2
            }
            EntailmentRule::Rdfs7SubProperty => {
                if self.antecedents.len() != 2 {
                    return false;
                }
                let a0 = &self.antecedents[0]; // (p1, subPropertyOf, p2)
                let a1 = &self.antecedents[1]; // (x, p1, y)
                a0[1] == *rdfs_subpropertyof
                    && a0[0] == a1[1] // shared p1
                    && self.derived[0] == a1[0] // x
                    && self.derived[1] == a0[2] // p2
                    && self.derived[2] == a1[2] // y
            }
            EntailmentRule::Rdfs2Domain => {
                if self.antecedents.len() != 2 {
                    return false;
                }
                let a0 = &self.antecedents[0]; // (p, domain, c)
                let a1 = &self.antecedents[1]; // (x, p, y)
                a0[1] == *rdfs_domain
                    && a0[0] == a1[1] // shared p
                    && self.derived[0] == a1[0] // x
                    && self.derived[1] == *rdf_type
                    && self.derived[2] == a0[2] // c
            }
            EntailmentRule::Rdfs3Range => {
                if self.antecedents.len() != 2 {
                    return false;
                }
                let a0 = &self.antecedents[0]; // (p, range, c)
                let a1 = &self.antecedents[1]; // (x, p, y)
                a0[1] == *rdfs_range
                    && a0[0] == a1[1] // shared p
                    && self.derived[0] == a1[2] // y
                    && self.derived[1] == *rdf_type
                    && self.derived[2] == a0[2] // c
            }
            EntailmentRule::Rdfs5SubPropertyTrans => {
                if self.antecedents.len() != 2 {
                    return false;
                }
                let a0 = &self.antecedents[0]; // (p1, subPropertyOf, p2)
                let a1 = &self.antecedents[1]; // (p2, subPropertyOf, p3)
                a0[1] == *rdfs_subpropertyof
                    && a1[1] == *rdfs_subpropertyof
                    && a0[2] == a1[0] // shared p2
                    && self.derived[0] == a0[0] // p1
                    && self.derived[1] == *rdfs_subpropertyof
                    && self.derived[2] == a1[2] // p3
            }
            EntailmentRule::Rdfs11SubClassTrans => {
                if self.antecedents.len() != 2 {
                    return false;
                }
                let a0 = &self.antecedents[0]; // (c1, subClassOf, c2)
                let a1 = &self.antecedents[1]; // (c2, subClassOf, c3)
                a0[1] == *rdfs_subclassof
                    && a1[1] == *rdfs_subclassof
                    && a0[2] == a1[0] // shared c2
                    && self.derived[0] == a0[0] // c1
                    && self.derived[1] == *rdfs_subclassof
                    && self.derived[2] == a1[2] // c3
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fh(s: &str) -> FieldHex {
        FieldHex(s.to_string())
    }

    // Schema-vocabulary encodings the verifier supplies (arbitrary distinct
    // constants; the shape check only cares about equality structure).
    const T: &str = "0x01"; // rdf:type
    const SC: &str = "0x02"; // rdfs:subClassOf
    const SP: &str = "0x03"; // rdfs:subPropertyOf
    const DOM: &str = "0x04"; // rdfs:domain
    const RNG: &str = "0x05"; // rdfs:range

    /// Every schema encoding, in the `is_well_formed` argument order.
    fn schema() -> (FieldHex, FieldHex, FieldHex, FieldHex, FieldHex) {
        (fh(T), fh(SC), fh(SP), fh(DOM), fh(RNG))
    }

    /// `step.is_well_formed(..)` with the fixed schema constants spread in.
    fn wf(step: &DerivationStep) -> bool {
        let (t, sc, sp, dom, rng) = schema();
        step.is_well_formed(&t, &sc, &sp, &dom, &rng)
    }

    #[test]
    fn rdfs9_well_formed_accepts_valid_shape() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs9SubClassType,
            // (x type c1), (c1 subClassOf c2) ⊢ (x type c2)
            antecedents: vec![
                [fh("0xaa"), fh(T), fh("0xc1")],
                [fh("0xc1"), fh(SC), fh("0xc2")],
            ],
            derived: [fh("0xaa"), fh(T), fh("0xc2")],
        };
        assert!(wf(&step));
    }

    #[test]
    fn rdfs9_rejects_broken_class_chain() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs9SubClassType,
            antecedents: vec![
                [fh("0xaa"), fh(T), fh("0xc1")],
                // c1 != cX (broken shared class)
                [fh("0xcX"), fh(SC), fh("0xc2")],
            ],
            derived: [fh("0xaa"), fh(T), fh("0xc2")],
        };
        assert!(!wf(&step));
    }

    #[test]
    fn rdfs7_well_formed_accepts_valid_shape() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs7SubProperty,
            // (p1 subPropertyOf p2), (x p1 y) ⊢ (x p2 y)
            antecedents: vec![
                [fh("0xp1"), fh(SP), fh("0xp2")],
                [fh("0xaa"), fh("0xp1"), fh("0xbb")],
            ],
            derived: [fh("0xaa"), fh("0xp2"), fh("0xbb")],
        };
        assert!(wf(&step));
    }

    #[test]
    fn rdfs2_domain_accepts_valid_shape() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs2Domain,
            // (p domain c), (x p y) ⊢ (x type c)
            antecedents: vec![
                [fh("0xp1"), fh(DOM), fh("0xcc")],
                [fh("0xaa"), fh("0xp1"), fh("0xbb")],
            ],
            derived: [fh("0xaa"), fh(T), fh("0xcc")],
        };
        assert!(wf(&step));
    }

    #[test]
    fn rdfs2_domain_rejects_wrong_derived_subject() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs2Domain,
            antecedents: vec![
                [fh("0xp1"), fh(DOM), fh("0xcc")],
                [fh("0xaa"), fh("0xp1"), fh("0xbb")],
            ],
            // domain types the SUBJECT (x); using the object (y) is unsound.
            derived: [fh("0xbb"), fh(T), fh("0xcc")],
        };
        assert!(!wf(&step));
    }

    #[test]
    fn rdfs3_range_accepts_valid_shape() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs3Range,
            // (p range c), (x p y) ⊢ (y type c)
            antecedents: vec![
                [fh("0xp1"), fh(RNG), fh("0xcc")],
                [fh("0xaa"), fh("0xp1"), fh("0xbb")],
            ],
            derived: [fh("0xbb"), fh(T), fh("0xcc")],
        };
        assert!(wf(&step));
    }

    #[test]
    fn rdfs3_range_rejects_wrong_derived_subject() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs3Range,
            antecedents: vec![
                [fh("0xp1"), fh(RNG), fh("0xcc")],
                [fh("0xaa"), fh("0xp1"), fh("0xbb")],
            ],
            // range types the OBJECT (y); using the subject (x) is unsound.
            derived: [fh("0xaa"), fh(T), fh("0xcc")],
        };
        assert!(!wf(&step));
    }

    #[test]
    fn rdfs2_and_rdfs3_do_not_cross_accept() {
        // A rdfs2 (domain) antecedent must NOT satisfy the rdfs3 (range) shape:
        // the predicate slot differs (domain vs range), so the tag is load-bearing.
        let domain_step = DerivationStep {
            rule: EntailmentRule::Rdfs3Range, // claim range...
            antecedents: vec![
                [fh("0xp1"), fh(DOM), fh("0xcc")], // ...over a domain edge
                [fh("0xaa"), fh("0xp1"), fh("0xbb")],
            ],
            derived: [fh("0xbb"), fh(T), fh("0xcc")],
        };
        assert!(!wf(&domain_step));
    }

    #[test]
    fn rdfs5_subproperty_transitivity_accepts_valid_shape() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs5SubPropertyTrans,
            // (p1 subPropertyOf p2), (p2 subPropertyOf p3) ⊢ (p1 subPropertyOf p3)
            antecedents: vec![
                [fh("0xp1"), fh(SP), fh("0xp2")],
                [fh("0xp2"), fh(SP), fh("0xp3")],
            ],
            derived: [fh("0xp1"), fh(SP), fh("0xp3")],
        };
        assert!(wf(&step));
    }

    #[test]
    fn rdfs5_rejects_broken_property_chain() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs5SubPropertyTrans,
            antecedents: vec![
                [fh("0xp1"), fh(SP), fh("0xp2")],
                // p2 != pX (broken shared middle property)
                [fh("0xpX"), fh(SP), fh("0xp3")],
            ],
            derived: [fh("0xp1"), fh(SP), fh("0xp3")],
        };
        assert!(!wf(&step));
    }

    #[test]
    fn rdfs11_subclass_transitivity_accepts_valid_shape() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs11SubClassTrans,
            // (c1 subClassOf c2), (c2 subClassOf c3) ⊢ (c1 subClassOf c3)
            antecedents: vec![
                [fh("0xc1"), fh(SC), fh("0xc2")],
                [fh("0xc2"), fh(SC), fh("0xc3")],
            ],
            derived: [fh("0xc1"), fh(SC), fh("0xc3")],
        };
        assert!(wf(&step));
    }

    #[test]
    fn rdfs11_rejects_broken_class_chain() {
        let step = DerivationStep {
            rule: EntailmentRule::Rdfs11SubClassTrans,
            antecedents: vec![
                [fh("0xc1"), fh(SC), fh("0xc2")],
                // c2 != cX (broken shared middle class)
                [fh("0xcX"), fh(SC), fh("0xc3")],
            ],
            derived: [fh("0xc1"), fh(SC), fh("0xc3")],
        };
        assert!(!wf(&step));
    }

    #[test]
    fn regime_admits_only_matching_rules() {
        assert!(!regime_admits(
            EntailmentRegime::Simple,
            EntailmentRule::Rdfs9SubClassType
        ));
        assert!(regime_admits(
            EntailmentRegime::Rdfs,
            EntailmentRule::Rdfs7SubProperty
        ));
        assert!(regime_admits(
            EntailmentRegime::Owl,
            EntailmentRule::Rdfs9SubClassType
        ));
        // The four new rules are RDFS rules and admitted under Rdfs and Owl.
        for rule in [
            EntailmentRule::Rdfs2Domain,
            EntailmentRule::Rdfs3Range,
            EntailmentRule::Rdfs5SubPropertyTrans,
            EntailmentRule::Rdfs11SubClassTrans,
        ] {
            assert!(!regime_admits(EntailmentRegime::Simple, rule));
            assert!(regime_admits(EntailmentRegime::Rdfs, rule));
            assert!(regime_admits(EntailmentRegime::Owl, rule));
        }
    }
}
