//! L2 — syntactic OWL 2 EL / QL / RL profile-membership checker.
//!
//! 🤖 SPARQ agent [SONNET-4.6]. Bead sq-pbz04.4.2. Implements a purely SYNTACTIC
//! grammar walk over a [`crate::model::Ontology`] — terminating by construction, no
//! semantic reasoning — to decide membership in the three principal tractable OWL 2
//! profiles per the W3C OWL 2 Profiles specification:
//! `<https://www.w3.org/TR/owl2-profiles/>`.
//!
//! # Design
//! Each profile is checked by its own linear scan over the axiom list (up to three
//! independent passes — `check_el`/`check_ql`/`check_rl`): each axiom is routed to that
//! profile's grammar predicate, which recursively validates the class-expression tree.
//! Termination is
//! guaranteed by the structural finiteness of [`crate::model::ClassExpression`]; no
//! memoization or cycle guard is needed (L1 extraction already rejects cyclic
//! class-expression encodings). The checker returns `Membership::NotIn` on the FIRST
//! violation — not a full list — so callers get a fast "is this ontology in EL/QL/RL?"
//! decision without scanning the whole axiom set.
//!
//! # Honest scope (L1 ALCH fragment)
//! The L1 extractor admits only a subset of OWL 2 syntax; the profile checker inherits
//! this scope. Constructs outside L1 (cardinality restrictions, data properties, nominal
//! classes, inverse properties) cause L1 extraction to fail before this code runs, so
//! those ontologies yield `Membership::Unknown` from [`profiles_from_extraction`] rather
//! than `Membership::NotIn`. Within the admitted ALCH fragment the grammar rules are
//! implemented per the W3C OWL 2 Profiles specification (2012-12-11). This covers a
//! narrow slice of the full 414-case `ProfileIdentificationTest` conformance suite.
//!
//! # Concrete domain (opt-in `dl_datatypes`) — the checker ABSTAINS
//! [SONNET-4.6] sq-pbz04.4.19. With the `dl_datatypes` feature on, L1 admits data
//! properties, data ranges and the `∃T.dr` / `∀T.dr` quantifiers. This L2 grammar walk does
//! **not** model the profiles' datatype rules (each profile restricts BOTH the data-range
//! syntax and the admissible datatype MAP, and the three restrictions differ), so an
//! ontology carrying any concrete-domain construct yields [`Membership::Unknown`] for all
//! three profiles rather than a guessed `In`/`NotIn`. That is the crate's fail-closed
//! discipline applied one layer up: an unmodelled construct must never manufacture a
//! definitive profile verdict in EITHER direction. The L4 dispatch reads the abstention as
//! "not in any profile" and routes such ontologies straight to the ALCH(D) tableau — the only
//! branch whose argument covers them (`tableau` module docs §5b).
//!
//! # References
//! - OWL 2 EL §2: `<https://www.w3.org/TR/owl2-profiles/#OWL_2_EL>`
//! - OWL 2 QL §3: `<https://www.w3.org/TR/owl2-profiles/#OWL_2_QL>`
//! - OWL 2 RL §4: `<https://www.w3.org/TR/owl2-profiles/#OWL_2_RL>`

use crate::extract::ExtractError;
use crate::model::{Axiom, ClassExpression, Ontology};

// -------------------------------------------------------------------------------------------
// Public API
// -------------------------------------------------------------------------------------------

/// Whether a structural ontology belongs to an OWL 2 profile.
///
/// Returned by [`profiles`] and [`profiles_from_extraction`] for each checked profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Membership {
    /// The ontology is IN the profile: every axiom satisfies the profile grammar.
    In,
    /// The ontology is NOT in the profile: at least one axiom violates the grammar.
    ///
    /// The payload is a human-readable diagnostic naming the first violation. Only the
    /// first violation is reported (fail-fast, not a full list).
    NotIn(String),
    /// Profile membership cannot be determined because the RDF extraction failed.
    ///
    /// The payload is the [`ExtractError`] rendered as a string. Never produced by
    /// [`profiles`] — only by [`profiles_from_extraction`] on an `Err` result.
    Unknown(String),
}

impl Membership {
    /// `true` iff membership is confirmed (`In`).
    #[must_use]
    pub fn is_in(&self) -> bool {
        matches!(self, Membership::In)
    }

    /// `true` iff membership is definitively denied (`NotIn`).
    #[must_use]
    pub fn is_not_in(&self) -> bool {
        matches!(self, Membership::NotIn(_))
    }

    /// `true` iff membership is indeterminate due to an extraction failure (`Unknown`).
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Membership::Unknown(_))
    }
}

/// OWL 2 EL / QL / RL profile membership verdicts for a single ontology.
///
/// Produced by [`profiles`] and [`profiles_from_extraction`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileSet {
    /// Membership verdict for OWL 2 EL (W3C §2).
    pub el: Membership,
    /// Membership verdict for OWL 2 QL (W3C §3).
    pub ql: Membership,
    /// Membership verdict for OWL 2 RL (W3C §4).
    pub rl: Membership,
}

impl ProfileSet {
    /// `true` iff the ontology is `In` ALL three profiles (EL, QL, and RL simultaneously).
    #[must_use]
    pub fn in_all(&self) -> bool {
        self.el.is_in() && self.ql.is_in() && self.rl.is_in()
    }

    /// `true` iff the ontology is `In` AT LEAST ONE of the three profiles.
    #[must_use]
    pub fn in_any(&self) -> bool {
        self.el.is_in() || self.ql.is_in() || self.rl.is_in()
    }
}

/// Check OWL 2 EL / QL / RL profile membership of a structural ontology.
///
/// The check is a purely syntactic, terminating grammar walk per the W3C OWL 2 Profiles
/// specification (`<https://www.w3.org/TR/owl2-profiles/>`). It returns
/// [`Membership::NotIn`] on the FIRST violation found (fail-fast, not a full list), and
/// [`Membership::In`] when every axiom satisfies the profile grammar. An empty ontology
/// is `In` all three profiles. [`Membership::Unknown`] is NEVER returned from this
/// function — only from [`profiles_from_extraction`] when extraction failed.
///
/// # L1-fragment note
/// The L1 structural model admits only NAMED object properties (no `InverseObjectProperty`
/// in `ObjectPropertyExpression`). Profile rules that would require inverse-property
/// expressions are therefore inapplicable in L1 — the extractor would have rejected the
/// graph before this function is ever called.
#[must_use]
pub fn profiles(onto: &Ontology) -> ProfileSet {
    // [SONNET-4.6] sq-pbz04.4.19 (opt-in `dl_datatypes`): a concrete-domain construct puts
    // the ontology outside what this grammar walk models, so abstain for ALL THREE profiles
    // rather than guess either direction (module docs "Concrete domain").
    #[cfg(feature = "dl_datatypes")]
    if let Some(reason) = onto.axioms().iter().find_map(concrete_domain_construct) {
        return ProfileSet {
            el: Membership::Unknown(reason.clone()),
            ql: Membership::Unknown(reason.clone()),
            rl: Membership::Unknown(reason),
        };
    }
    ProfileSet {
        el: check_el(onto),
        ql: check_ql(onto),
        rl: check_rl(onto),
    }
}

/// `Some(reason)` iff the axiom carries a concrete-domain construct — a data-property axiom
/// or a data quantifier nested anywhere in one of its class expressions
/// ([SONNET-4.6] sq-pbz04.4.19, opt-in `dl_datatypes`).
#[cfg(feature = "dl_datatypes")]
fn concrete_domain_construct(axiom: &Axiom) -> Option<String> {
    let nested = |ce: &ClassExpression| ce_has_data_quantifier(ce);
    match axiom {
        Axiom::SubDataPropertyOf { .. } => {
            Some("SubDataPropertyOf (concrete domain not modelled by L2)".to_string())
        }
        Axiom::DataPropertyDomain { .. } => {
            Some("DataPropertyDomain (concrete domain not modelled by L2)".to_string())
        }
        Axiom::DataPropertyRange { .. } => {
            Some("DataPropertyRange (concrete domain not modelled by L2)".to_string())
        }
        Axiom::SubClassOf { sub, sup } if nested(sub) || nested(sup) => {
            Some("data quantifier in SubClassOf (concrete domain not modelled by L2)".to_string())
        }
        Axiom::EquivalentClasses(left, right) | Axiom::DisjointClasses(left, right)
            if nested(left) || nested(right) =>
        {
            Some("data quantifier in a class axiom (concrete domain not modelled by L2)".to_string())
        }
        Axiom::ClassAssertion { class, .. } if nested(class) => {
            Some("data quantifier in ClassAssertion (concrete domain not modelled by L2)".to_string())
        }
        _ => None,
    }
}

/// `true` iff a data quantifier occurs anywhere inside the class expression.
#[cfg(feature = "dl_datatypes")]
fn ce_has_data_quantifier(ce: &ClassExpression) -> bool {
    match ce {
        ClassExpression::DataSomeValuesFrom(_, _) | ClassExpression::DataAllValuesFrom(_, _) => true,
        ClassExpression::ObjectIntersectionOf(members)
        | ClassExpression::ObjectUnionOf(members) => members.iter().any(ce_has_data_quantifier),
        ClassExpression::ObjectComplementOf(inner) => ce_has_data_quantifier(inner),
        ClassExpression::ObjectSomeValuesFrom(_, filler)
        | ClassExpression::ObjectAllValuesFrom(_, filler) => ce_has_data_quantifier(filler),
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => false,
    }
}

/// The `Err` a profile grammar predicate returns for a concrete-domain construct.
///
/// [`profiles`] abstains on such an ontology BEFORE any grammar predicate runs, so this is
/// never the verdict a caller sees; the arms exist so each grammar walk stays TOTAL over
/// [`Axiom`] / [`ClassExpression`] (and stays fail-closed if a future caller reaches a
/// predicate directly). [SONNET-4.6] sq-pbz04.4.19
#[cfg(feature = "dl_datatypes")]
fn concrete_domain_not_modelled<T>(what: &str, ctx: &str) -> Result<T, String> {
    Err(format!(
        "{} in {}: the concrete domain is not modelled by the L2 profile grammar",
        what, ctx
    ))
}

/// Check OWL 2 profile membership from an extraction result.
///
/// An `Err(e)` extraction failure yields `Membership::Unknown(format!("{}", e))` for all
/// three profiles — membership cannot be decided when the graph could not be fully mapped.
/// An `Ok(onto)` delegates to [`profiles`].
///
/// This is the ergonomic entry point when you have a `Result<Ontology, ExtractError>`
/// directly from [`crate::extract()`] and want to fold the extraction outcome into the
/// profile verdict without a manual `match`.
#[must_use]
pub fn profiles_from_extraction(result: &Result<Ontology, ExtractError>) -> ProfileSet {
    match result {
        Ok(onto) => profiles(onto),
        Err(e) => {
            let msg = format!("{}", e);
            ProfileSet {
                el: Membership::Unknown(msg.clone()),
                ql: Membership::Unknown(msg.clone()),
                rl: Membership::Unknown(msg),
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// OWL 2 EL grammar (W3C OWL 2 EL §2)
// -------------------------------------------------------------------------------------------

/// Check every axiom in `onto` against the OWL 2 EL axiom grammar (§2).
/// Returns `Membership::In` iff every axiom passes; `Membership::NotIn(reason)` on the
/// first violation.
fn check_el(onto: &Ontology) -> Membership {
    for axiom in onto.axioms() {
        if let Err(reason) = el_axiom(axiom) {
            return Membership::NotIn(reason);
        }
    }
    Membership::In
}

/// OWL 2 EL axiom-level grammar (§2). Returns `Err(reason)` on first violation.
fn el_axiom(axiom: &Axiom) -> Result<(), String> {
    match axiom {
        Axiom::SubClassOf { sub, sup } => {
            el_ce(sub, "SubClassOf sub")?;
            el_ce(sup, "SubClassOf sup")
        }
        Axiom::EquivalentClasses(left, right) => {
            el_ce(left, "EquivalentClasses left")?;
            el_ce(right, "EquivalentClasses right")
        }
        Axiom::DisjointClasses(left, right) => {
            el_ce(left, "DisjointClasses left")?;
            el_ce(right, "DisjointClasses right")
        }
        // SubObjectPropertyOf over named props is always EL (§2 RBox)
        Axiom::SubObjectPropertyOf { .. } => Ok(()),
        Axiom::ObjectPropertyDomain { domain, .. } => el_ce(domain, "ObjectPropertyDomain"),
        Axiom::ObjectPropertyRange { range, .. } => el_ce(range, "ObjectPropertyRange"),
        Axiom::ClassAssertion { class, .. } => el_ce(class, "ClassAssertion"),
        // Ground role assertions are always EL (§2 ABox)
        Axiom::ObjectPropertyAssertion { .. } => Ok(()),
        // TransitiveObjectProperty is in the EL grammar (§2 supports transitive object
        // properties, a special case of property chains). [GPT-5.6] sq-zfwzq
        #[cfg(feature = "dl_transitive")]
        Axiom::TransitiveObjectProperty { .. } => Ok(()),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        Axiom::SubDataPropertyOf { .. } => {
            concrete_domain_not_modelled("SubDataPropertyOf", "the OWL 2 EL axiom grammar")
        }
        #[cfg(feature = "dl_datatypes")]
        Axiom::DataPropertyDomain { .. } => {
            concrete_domain_not_modelled("DataPropertyDomain", "the OWL 2 EL axiom grammar")
        }
        #[cfg(feature = "dl_datatypes")]
        Axiom::DataPropertyRange { .. } => {
            concrete_domain_not_modelled("DataPropertyRange", "the OWL 2 EL axiom grammar")
        }
    }
}

/// OWL 2 EL class expression grammar (EL-CE, §2):
///
/// ```text
/// EL-CE := Class | owl:Thing | owl:Nothing
///        | ObjectIntersectionOf(EL-CE EL-CE ...)   [≥ 2 conjuncts]
///        | ObjectSomeValuesFrom(OPE EL-CE)
/// ```
///
/// NOT valid: `ObjectUnionOf`, `ObjectComplementOf`, `ObjectAllValuesFrom`.
fn el_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
    match ce {
        // Leaves are always EL-CE (§2)
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => Ok(()),
        ClassExpression::ObjectIntersectionOf(members) => {
            if members.len() < 2 {
                return Err(format!(
                    "{}: ObjectIntersectionOf requires at least 2 members (EL §2), got {}",
                    ctx,
                    members.len()
                ));
            }
            for m in members {
                el_ce(m, ctx)?;
            }
            Ok(())
        }
        // ∃R.C is EL-CE iff C is EL-CE (§2)
        ClassExpression::ObjectSomeValuesFrom(_, filler) => el_ce(filler, ctx),
        // The three forbidden constructors (§2)
        ClassExpression::ObjectUnionOf(_) => Err(format!(
            "{}: ObjectUnionOf is not permitted in OWL 2 EL (EL §2)",
            ctx
        )),
        ClassExpression::ObjectComplementOf(_) => Err(format!(
            "{}: ObjectComplementOf is not permitted in OWL 2 EL (EL §2)",
            ctx
        )),
        ClassExpression::ObjectAllValuesFrom(_, _) => Err(format!(
            "{}: ObjectAllValuesFrom is not permitted in OWL 2 EL (EL §2)",
            ctx
        )),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataSomeValuesFrom", ctx)
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataAllValuesFrom", ctx)
        }
    }
}

// -------------------------------------------------------------------------------------------
// OWL 2 QL grammar (W3C OWL 2 QL §3)
// -------------------------------------------------------------------------------------------

/// Check every axiom in `onto` against the OWL 2 QL axiom grammar (§3).
fn check_ql(onto: &Ontology) -> Membership {
    for axiom in onto.axioms() {
        if let Err(reason) = ql_axiom(axiom) {
            return Membership::NotIn(reason);
        }
    }
    Membership::In
}

/// OWL 2 QL axiom-level grammar (§3). Returns `Err(reason)` on first violation.
fn ql_axiom(axiom: &Axiom) -> Result<(), String> {
    match axiom {
        Axiom::SubClassOf { sub, sup } => {
            ql_sub_ce(sub, "SubClassOf sub")?;
            ql_super_ce(sup, "SubClassOf sup")
        }
        // QL §3: EquivalentClasses — both operands must be sub-CEs
        Axiom::EquivalentClasses(left, right) => {
            ql_sub_ce(left, "EquivalentClasses left")?;
            ql_sub_ce(right, "EquivalentClasses right")
        }
        // QL §3: DisjointClasses — both operands must be sub-CEs
        Axiom::DisjointClasses(left, right) => {
            ql_sub_ce(left, "DisjointClasses left")?;
            ql_sub_ce(right, "DisjointClasses right")
        }
        Axiom::SubObjectPropertyOf { .. } => Ok(()),
        // QL §3: domain and range are super-CEs
        Axiom::ObjectPropertyDomain { domain, .. } => ql_super_ce(domain, "ObjectPropertyDomain"),
        Axiom::ObjectPropertyRange { range, .. } => ql_super_ce(range, "ObjectPropertyRange"),
        // QL §3.2.5: ClassAssertion admits only an atomic Class (not any sub-CE or super-CE).
        // The W3C QL grammar restricts class assertions to atomic classes — complex CEs
        // (including the otherwise-valid sub-CE ∃R.⊤) are not permitted here.
        Axiom::ClassAssertion { class, .. } => match class {
            ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => Ok(()),
            _ => Err(
                "ClassAssertion: QL §3 restricts class assertions to atomic classes; \
                 got a complex class expression"
                    .into(),
            ),
        },
        Axiom::ObjectPropertyAssertion { .. } => Ok(()),
        // TransitiveObjectProperty is NOT in the QL grammar (§3 explicitly excludes
        // transitive object properties). [GPT-5.6] sq-zfwzq
        #[cfg(feature = "dl_transitive")]
        Axiom::TransitiveObjectProperty { .. } => Err(
            "TransitiveObjectProperty is not permitted in OWL 2 QL (QL §3)".into(),
        ),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        Axiom::SubDataPropertyOf { .. } => {
            concrete_domain_not_modelled("SubDataPropertyOf", "the OWL 2 QL axiom grammar")
        }
        #[cfg(feature = "dl_datatypes")]
        Axiom::DataPropertyDomain { .. } => {
            concrete_domain_not_modelled("DataPropertyDomain", "the OWL 2 QL axiom grammar")
        }
        #[cfg(feature = "dl_datatypes")]
        Axiom::DataPropertyRange { .. } => {
            concrete_domain_not_modelled("DataPropertyRange", "the OWL 2 QL axiom grammar")
        }
    }
}

/// OWL 2 QL sub-class expression (sub-CE, left-side grammar, §3):
///
/// ```text
/// sub-CE := Class (including owl:Thing and owl:Nothing)
///         | ObjectSomeValuesFrom(OPE owl:Thing)   [filler must be exactly owl:Thing]
/// ```
///
/// NOT valid: `ObjectIntersectionOf` (intersection is only in the super-CE grammar),
/// `ObjectUnionOf`, `ObjectComplementOf`, `ObjectAllValuesFrom`,
/// `ObjectSomeValuesFrom(_, non-Thing)`.
fn ql_sub_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
    match ce {
        // Named class, owl:Thing, and owl:Nothing are all valid QL sub-CEs (§3 "Class")
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => Ok(()),
        // ∃R.⊤ is a valid QL sub-CE; ∃R.C (C ≠ ⊤) is NOT (§3 subObjectSomeValuesFrom)
        ClassExpression::ObjectSomeValuesFrom(_, filler) => {
            if **filler == ClassExpression::Thing {
                Ok(())
            } else {
                Err(format!(
                    "{}: ObjectSomeValuesFrom as QL sub-CE requires filler owl:Thing (QL §3)",
                    ctx
                ))
            }
        }
        // ObjectIntersectionOf is NOT in the QL sub-CE grammar (§3); it only appears in the
        // super-CE production (superObjectIntersectionOf).
        ClassExpression::ObjectIntersectionOf(_) => Err(format!(
            "{}: ObjectIntersectionOf is not a valid QL sub-class expression (QL §3)",
            ctx
        )),
        ClassExpression::ObjectUnionOf(_) => Err(format!(
            "{}: ObjectUnionOf is not a valid QL sub-class expression (QL §3)",
            ctx
        )),
        ClassExpression::ObjectComplementOf(_) => Err(format!(
            "{}: ObjectComplementOf is not a valid QL sub-class expression (QL §3)",
            ctx
        )),
        ClassExpression::ObjectAllValuesFrom(_, _) => Err(format!(
            "{}: ObjectAllValuesFrom is not a valid QL sub-class expression (QL §3)",
            ctx
        )),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataSomeValuesFrom", ctx)
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataAllValuesFrom", ctx)
        }
    }
}

/// OWL 2 QL super-class expression (super-CE, right-side grammar, §3):
///
/// ```text
/// super-CE := Class (including owl:Thing and owl:Nothing)
///           | ObjectIntersectionOf(super-CE super-CE ...)   [≥ 2 conjuncts]
///           | ObjectComplementOf(sub-CE)
///           | ObjectSomeValuesFrom(OPE Class)   [§3.2.3 superObjectSomeValuesFrom;
///                                                Class := IRI includes owl:Thing/owl:Nothing]
/// ```
///
/// NOT valid: `ObjectUnionOf`, `ObjectAllValuesFrom`.
///
/// Note on the QL ∃ asymmetry (§3.2.3):
/// - `subObjectSomeValuesFrom`   := `ObjectSomeValuesFrom(OPE owl:Thing)` — filler EXACTLY ⊤.
/// - `superObjectSomeValuesFrom` := `ObjectSomeValuesFrom(OPE Class)` — filler any `Class := IRI`,
///   which includes owl:Thing AND owl:Nothing (OWL 2 Syntax §5.1).  The two productions are NOT
///   a disjoint pair; the super-∃ is a strict superset.
fn ql_super_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
    match ce {
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => Ok(()),
        ClassExpression::ObjectIntersectionOf(members) => {
            if members.len() < 2 {
                return Err(format!(
                    "{}: ObjectIntersectionOf requires at least 2 members (QL §3), got {}",
                    ctx,
                    members.len()
                ));
            }
            for m in members {
                ql_super_ce(m, ctx)?;
            }
            Ok(())
        }
        // ObjectComplementOf(sub-CE) is a valid QL super-CE (§3 superObjectComplementOf)
        ClassExpression::ObjectComplementOf(inner) => ql_sub_ce(inner, ctx),
        // ObjectSomeValuesFrom(OPE, Class) is a valid QL super-CE (§3.2.3 superObjectSomeValuesFrom).
        // "Class" in the OWL 2 grammar (§5.1) is any IRI — it includes owl:Thing and owl:Nothing.
        // Only a complex CE filler (intersection, union, complement, etc.) is invalid here.
        // [SONNET-4.6] sq-pbz04.4.2: corrected from "named-only" to "any Class (IRI)".
        ClassExpression::ObjectSomeValuesFrom(_, filler) => match filler.as_ref() {
            ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => Ok(()),
            _ => Err(format!(
                "{}: ObjectSomeValuesFrom as QL super-CE requires a Class (IRI) filler \
                 (QL §3.2.3 superObjectSomeValuesFrom; Class includes owl:Thing and owl:Nothing; \
                 got a complex class expression)",
                ctx
            )),
        },
        ClassExpression::ObjectUnionOf(_) => Err(format!(
            "{}: ObjectUnionOf is not a valid QL super-class expression (QL §3)",
            ctx
        )),
        ClassExpression::ObjectAllValuesFrom(_, _) => Err(format!(
            "{}: ObjectAllValuesFrom is not a valid QL super-class expression (QL §3)",
            ctx
        )),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataSomeValuesFrom", ctx)
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataAllValuesFrom", ctx)
        }
    }
}

// -------------------------------------------------------------------------------------------
// OWL 2 RL grammar (W3C OWL 2 RL §4)
// -------------------------------------------------------------------------------------------

/// Check every axiom in `onto` against the OWL 2 RL axiom grammar (§4).
fn check_rl(onto: &Ontology) -> Membership {
    for axiom in onto.axioms() {
        if let Err(reason) = rl_axiom(axiom) {
            return Membership::NotIn(reason);
        }
    }
    Membership::In
}

/// OWL 2 RL axiom-level grammar (§4). Returns `Err(reason)` on first violation.
fn rl_axiom(axiom: &Axiom) -> Result<(), String> {
    match axiom {
        Axiom::SubClassOf { sub, sup } => {
            rl_sub_ce(sub, "SubClassOf sub")?;
            rl_super_ce(sup, "SubClassOf sup")
        }
        // RL §4: EquivalentClasses — each CE must be BOTH a sub-CE AND a super-CE
        Axiom::EquivalentClasses(left, right) => {
            rl_sub_ce(left, "EquivalentClasses left")?;
            rl_super_ce(left, "EquivalentClasses left")?;
            rl_sub_ce(right, "EquivalentClasses right")?;
            rl_super_ce(right, "EquivalentClasses right")
        }
        // RL §4: DisjointClasses — both operands must be sub-CEs
        Axiom::DisjointClasses(left, right) => {
            rl_sub_ce(left, "DisjointClasses left")?;
            rl_sub_ce(right, "DisjointClasses right")
        }
        Axiom::SubObjectPropertyOf { .. } => Ok(()),
        // RL §4: domain and range are super-CEs
        Axiom::ObjectPropertyDomain { domain, .. } => rl_super_ce(domain, "ObjectPropertyDomain"),
        Axiom::ObjectPropertyRange { range, .. } => rl_super_ce(range, "ObjectPropertyRange"),
        // RL §4.2.5: ClassAssertion uses a superClassExpression (not sub-CE)
        Axiom::ClassAssertion { class, .. } => rl_super_ce(class, "ClassAssertion"),
        Axiom::ObjectPropertyAssertion { .. } => Ok(()),
        // TransitiveObjectProperty is in the RL grammar (§4 supports transitive object
        // properties). [GPT-5.6] sq-zfwzq
        #[cfg(feature = "dl_transitive")]
        Axiom::TransitiveObjectProperty { .. } => Ok(()),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        Axiom::SubDataPropertyOf { .. } => {
            concrete_domain_not_modelled("SubDataPropertyOf", "the OWL 2 RL axiom grammar")
        }
        #[cfg(feature = "dl_datatypes")]
        Axiom::DataPropertyDomain { .. } => {
            concrete_domain_not_modelled("DataPropertyDomain", "the OWL 2 RL axiom grammar")
        }
        #[cfg(feature = "dl_datatypes")]
        Axiom::DataPropertyRange { .. } => {
            concrete_domain_not_modelled("DataPropertyRange", "the OWL 2 RL axiom grammar")
        }
    }
}

/// OWL 2 RL sub-class expression (sub-CE, left-side grammar, §4):
///
/// ```text
/// sub-CE := Class other than owl:Thing (owl:Nothing is allowed)
///         | ObjectIntersectionOf(sub-CE sub-CE ...)   [≥ 2 conjuncts]
///         | ObjectUnionOf(sub-CE sub-CE ...)           [≥ 2 disjuncts; union is on the SUB side]
///         | ObjectSomeValuesFrom(OPE (sub-CE | owl:Thing))
/// ```
///
/// NOT valid: `owl:Thing` as a leaf, `ObjectComplementOf`, `ObjectAllValuesFrom`.
fn rl_sub_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
    match ce {
        // Named user-defined class is a valid RL sub-CE (§4 "Class other than owl:Thing")
        ClassExpression::Class(_) => Ok(()),
        // owl:Nothing is "Class other than owl:Thing" — valid (§4)
        ClassExpression::Nothing => Ok(()),
        // owl:Thing is explicitly excluded from RL sub-CE (§4 "Class other than owl:Thing")
        ClassExpression::Thing => Err(format!(
            "{}: owl:Thing is not a valid RL sub-class expression \
             (RL §4 requires 'Class other than owl:Thing')",
            ctx
        )),
        ClassExpression::ObjectIntersectionOf(members) => {
            if members.len() < 2 {
                return Err(format!(
                    "{}: ObjectIntersectionOf requires at least 2 members (RL §4), got {}",
                    ctx,
                    members.len()
                ));
            }
            for m in members {
                rl_sub_ce(m, ctx)?;
            }
            Ok(())
        }
        // ObjectUnionOf is a valid RL sub-CE (§4 subObjectUnionOf) — NOT valid in super position
        ClassExpression::ObjectUnionOf(members) => {
            if members.len() < 2 {
                return Err(format!(
                    "{}: ObjectUnionOf requires at least 2 members (RL §4), got {}",
                    ctx,
                    members.len()
                ));
            }
            for m in members {
                rl_sub_ce(m, ctx)?;
            }
            Ok(())
        }
        // ∃R.(sub-CE | owl:Thing) is valid in RL sub position (§4 subObjectSomeValuesFrom)
        ClassExpression::ObjectSomeValuesFrom(_, filler) => {
            if **filler == ClassExpression::Thing {
                Ok(()) // owl:Thing filler is explicitly allowed by §4 alongside sub-CE
            } else {
                rl_sub_ce(filler, ctx)
            }
        }
        ClassExpression::ObjectComplementOf(_) => Err(format!(
            "{}: ObjectComplementOf is not a valid RL sub-class expression (RL §4)",
            ctx
        )),
        ClassExpression::ObjectAllValuesFrom(_, _) => Err(format!(
            "{}: ObjectAllValuesFrom is not a valid RL sub-class expression (RL §4)",
            ctx
        )),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataSomeValuesFrom", ctx)
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataAllValuesFrom", ctx)
        }
    }
}

/// OWL 2 RL super-class expression (super-CE, right-side grammar, §4):
///
/// ```text
/// super-CE := Class other than owl:Thing (owl:Nothing is allowed)
///           | ObjectIntersectionOf(super-CE super-CE ...)   [≥ 2 conjuncts]
///           | ObjectComplementOf(sub-CE)
///           | ObjectAllValuesFrom(OPE super-CE)
/// ```
///
/// NOT valid: `owl:Thing` as a leaf, `ObjectUnionOf` (only on sub side), `ObjectSomeValuesFrom`.
fn rl_super_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
    match ce {
        // Named user-defined class is a valid RL super-CE (§4 "Class other than owl:Thing")
        ClassExpression::Class(_) => Ok(()),
        // owl:Nothing is "Class other than owl:Thing" — valid (§4)
        ClassExpression::Nothing => Ok(()),
        // owl:Thing is explicitly excluded from RL super-CE (§4 "Class other than owl:Thing")
        ClassExpression::Thing => Err(format!(
            "{}: owl:Thing is not a valid RL super-class expression \
             (RL §4 requires 'Class other than owl:Thing')",
            ctx
        )),
        ClassExpression::ObjectIntersectionOf(members) => {
            if members.len() < 2 {
                return Err(format!(
                    "{}: ObjectIntersectionOf requires at least 2 members (RL §4), got {}",
                    ctx,
                    members.len()
                ));
            }
            for m in members {
                rl_super_ce(m, ctx)?;
            }
            Ok(())
        }
        // ObjectUnionOf is NOT a valid RL super-CE (§4); union only appears in the sub-CE grammar
        ClassExpression::ObjectUnionOf(_) => Err(format!(
            "{}: ObjectUnionOf is not a valid RL super-class expression (RL §4); \
             union is only permitted in sub position (subObjectUnionOf)",
            ctx
        )),
        // ObjectComplementOf(sub-CE) is a valid RL super-CE (§4 superObjectComplementOf)
        ClassExpression::ObjectComplementOf(inner) => rl_sub_ce(inner, ctx),
        // ObjectAllValuesFrom(OPE, super-CE) is a valid RL super-CE (§4 superObjectAllValuesFrom)
        ClassExpression::ObjectAllValuesFrom(_, filler) => rl_super_ce(filler, ctx),
        // ObjectSomeValuesFrom is NOT a valid RL super-CE (§4)
        ClassExpression::ObjectSomeValuesFrom(_, _) => Err(format!(
            "{}: ObjectSomeValuesFrom is not a valid RL super-class expression (RL §4)",
            ctx
        )),
        // [SONNET-4.6] sq-pbz04.4.19: `profiles` ABSTAINS on a concrete-domain construct
        // before this predicate runs (module docs); the arm keeps the walk total.
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataSomeValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataSomeValuesFrom", ctx)
        }
        #[cfg(feature = "dl_datatypes")]
        ClassExpression::DataAllValuesFrom(_, _) => {
            concrete_domain_not_modelled("DataAllValuesFrom", ctx)
        }
    }
}

// -------------------------------------------------------------------------------------------
// Direct unit tests (the integration suite lives in tests/profile.rs)
// -------------------------------------------------------------------------------------------

/// [SONNET-4.6] sq-pbz04.4.19 — the concrete-domain ABSTENTION path (opt-in `dl_datatypes`).
#[cfg(all(test, feature = "dl_datatypes"))]
mod concrete_domain_tests {
    use super::*;
    use crate::cdomain::Datatype;
    use crate::model::{DataPropertyExpression, DataRange};

    fn dp() -> DataPropertyExpression {
        DataPropertyExpression::DataProperty(11)
    }
    fn dr() -> DataRange {
        DataRange::Datatype(Datatype::XsdInteger)
    }
    fn onto(axioms: Vec<Axiom>) -> Ontology {
        Ontology { axioms }
    }

    /// Every concrete-domain AXIOM kind makes all three verdicts `Unknown` — never a guessed
    /// `In` (which would fabricate a profile claim) and never a guessed `NotIn` (which would
    /// fabricate a refutation).
    #[test]
    fn concrete_domain_axioms_abstain_in_all_three_profiles() {
        let cases = vec![
            Axiom::SubDataPropertyOf {
                sub: dp(),
                sup: DataPropertyExpression::DataProperty(12),
            },
            Axiom::DataPropertyDomain {
                property: dp(),
                domain: ClassExpression::Class(1),
            },
            Axiom::DataPropertyRange {
                property: dp(),
                range: dr(),
            },
        ];
        for axiom in cases {
            let ps = profiles(&onto(vec![axiom.clone()]));
            assert!(ps.el.is_unknown(), "EL must abstain on {:?}", axiom);
            assert!(ps.ql.is_unknown(), "QL must abstain on {:?}", axiom);
            assert!(ps.rl.is_unknown(), "RL must abstain on {:?}", axiom);
            assert!(!ps.in_any() && !ps.in_all());
        }
    }

    /// A data quantifier NESTED inside an ordinary class axiom abstains too — the scan has
    /// to look through the class-expression tree, not just at the axiom kind.
    #[test]
    fn nested_data_quantifiers_abstain() {
        let deep = ClassExpression::ObjectIntersectionOf(vec![
            ClassExpression::Class(2),
            ClassExpression::some(
                3,
                ClassExpression::ObjectComplementOf(Box::new(ClassExpression::DataAllValuesFrom(
                    dp(),
                    dr(),
                ))),
            ),
        ]);
        for axiom in [
            Axiom::SubClassOf {
                sub: ClassExpression::Class(1),
                sup: deep.clone(),
            },
            Axiom::EquivalentClasses(ClassExpression::Class(1), deep.clone()),
            Axiom::DisjointClasses(ClassExpression::Class(1), deep.clone()),
            Axiom::ClassAssertion {
                class: deep.clone(),
                individual: 9,
            },
        ] {
            assert!(
                profiles(&onto(vec![axiom.clone()])).el.is_unknown(),
                "a nested data quantifier must abstain: {:?}",
                axiom
            );
        }
        // A datatype-free ontology is unaffected — the pre-pass must not over-trigger.
        let plain = onto(vec![Axiom::SubClassOf {
            sub: ClassExpression::Class(1),
            sup: ClassExpression::Class(2),
        }]);
        assert!(profiles(&plain).in_all());
    }

    /// The grammar predicates stay TOTAL: reached directly (as a future caller might), each
    /// refuses the concrete-domain constructs rather than panicking or silently accepting.
    #[test]
    fn grammar_predicates_remain_total_over_the_concrete_domain() {
        let axioms = [
            Axiom::SubDataPropertyOf {
                sub: dp(),
                sup: dp(),
            },
            Axiom::DataPropertyDomain {
                property: dp(),
                domain: ClassExpression::Class(1),
            },
            Axiom::DataPropertyRange {
                property: dp(),
                range: dr(),
            },
        ];
        for axiom in &axioms {
            assert!(el_axiom(axiom).is_err());
            assert!(ql_axiom(axiom).is_err());
            assert!(rl_axiom(axiom).is_err());
        }
        for ce in [
            ClassExpression::DataSomeValuesFrom(dp(), dr()),
            ClassExpression::DataAllValuesFrom(dp(), dr()),
        ] {
            assert!(el_ce(&ce, "ctx").is_err());
            assert!(ql_sub_ce(&ce, "ctx").is_err());
            assert!(ql_super_ce(&ce, "ctx").is_err());
            assert!(rl_sub_ce(&ce, "ctx").is_err());
            assert!(rl_super_ce(&ce, "ctx").is_err());
        }
    }
}
