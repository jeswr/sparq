//! L2 — syntactic OWL 2 EL / QL / RL profile-membership checker.
//!
//! 🤖 SPARQ agent [SONNET-4.6]. Bead sq-pbz04.4.2. Implements a purely SYNTACTIC
//! grammar walk over a [`crate::model::Ontology`] — terminating by construction, no
//! semantic reasoning — to decide membership in the three principal tractable OWL 2
//! profiles per the W3C OWL 2 Profiles specification:
//! `<https://www.w3.org/TR/owl2-profiles/>`.
//!
//! # Design
//! The check is a single, non-recursive pass: each axiom is routed to a per-profile
//! grammar predicate that recurses into the class-expression tree. Termination is
//! guaranteed by the structural finiteness of [`crate::model::ClassExpression`]; no
//! memoization or cycle guard is needed (L1 extraction already rejects cyclic
//! class-expression encodings). The checker returns `Membership::NotIn` on the FIRST
//! violation — not a full list — so callers get a fast "is this ontology in EL/QL/RL?"
//! decision without scanning the whole axiom set.
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
/// expressions (e.g. some QL property-range patterns) are therefore trivially satisfied in
/// L1 — the extractor would have rejected the graph before this function is ever called.
#[must_use]
pub fn profiles(onto: &Ontology) -> ProfileSet {
	ProfileSet {
		el: check_el(onto),
		ql: check_ql(onto),
		rl: check_rl(onto),
	}
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
		// QL §3: ClassAssertion — class must be a sub-CE
		Axiom::ClassAssertion { class, .. } => ql_sub_ce(class, "ClassAssertion"),
		Axiom::ObjectPropertyAssertion { .. } => Ok(()),
	}
}

/// OWL 2 QL sub-class expression (sub-CE, left-side grammar, §3):
///
/// ```text
/// sub-CE := Class | owl:Thing
///         | ObjectIntersectionOf(sub-CE sub-CE ...)   [≥ 2 conjuncts]
///         | ObjectSomeValuesFrom(OPE owl:Thing)
/// ```
///
/// NOT valid: `owl:Nothing`, `ObjectUnionOf`, `ObjectComplementOf`,
/// `ObjectAllValuesFrom`, `ObjectSomeValuesFrom(_, non-Thing)`.
fn ql_sub_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
	match ce {
		ClassExpression::Class(_) | ClassExpression::Thing => Ok(()),
		// owl:Nothing is NOT a QL sub-CE (§3)
		ClassExpression::Nothing => Err(format!(
			"{}: owl:Nothing is not a valid QL sub-class expression (QL §3)",
			ctx
		)),
		ClassExpression::ObjectIntersectionOf(members) => {
			if members.len() < 2 {
				return Err(format!(
					"{}: ObjectIntersectionOf requires at least 2 members (QL §3), got {}",
					ctx,
					members.len()
				));
			}
			for m in members {
				ql_sub_ce(m, ctx)?;
			}
			Ok(())
		}
		// ∃R.⊤ is a valid QL sub-CE; ∃R.C (C ≠ ⊤) is NOT (§3)
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
	}
}

/// OWL 2 QL super-class expression (super-CE, right-side grammar, §3):
///
/// ```text
/// super-CE := Class | owl:Thing | owl:Nothing
///           | ObjectIntersectionOf(super-CE super-CE ...)   [≥ 2 conjuncts]
///           | ObjectComplementOf(sub-CE)
/// ```
///
/// NOT valid: `ObjectUnionOf`, `ObjectAllValuesFrom`, `ObjectSomeValuesFrom`.
/// (`ObjectSomeValuesFrom` as a super-CE would need an inverse property expression to
/// encode `∃R⁻.C ⊑ …`, which is out of the L1 named-properties-only fragment.)
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
		// ObjectComplementOf(sub-CE) is a valid QL super-CE (§3)
		ClassExpression::ObjectComplementOf(inner) => ql_sub_ce(inner, ctx),
		ClassExpression::ObjectUnionOf(_) => Err(format!(
			"{}: ObjectUnionOf is not a valid QL super-class expression (QL §3)",
			ctx
		)),
		ClassExpression::ObjectAllValuesFrom(_, _) => Err(format!(
			"{}: ObjectAllValuesFrom is not a valid QL super-class expression (QL §3)",
			ctx
		)),
		// ObjectSomeValuesFrom is NOT a valid QL super-CE (§3): the QL super-grammar
		// permits ∃R⁻.C only via InverseObjectProperty, which L1 does not admit.
		ClassExpression::ObjectSomeValuesFrom(_, _) => Err(format!(
			"{}: ObjectSomeValuesFrom is not a valid QL super-class expression (QL §3)",
			ctx
		)),
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
		// RL §4: ClassAssertion — class must be a sub-CE
		Axiom::ClassAssertion { class, .. } => rl_sub_ce(class, "ClassAssertion"),
		Axiom::ObjectPropertyAssertion { .. } => Ok(()),
	}
}

/// OWL 2 RL sub-class expression (sub-CE, left-side grammar, §4):
///
/// ```text
/// sub-CE := Class | owl:Thing
///         | ObjectIntersectionOf(sub-CE sub-CE ...)   [≥ 2 conjuncts]
///         | ObjectSomeValuesFrom(OPE sub-CE)
/// ```
///
/// NOT valid: `owl:Nothing`, `ObjectUnionOf`, `ObjectComplementOf`,
/// `ObjectAllValuesFrom`.
fn rl_sub_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
	match ce {
		ClassExpression::Class(_) | ClassExpression::Thing => Ok(()),
		// owl:Nothing is NOT a valid RL sub-CE (§4)
		ClassExpression::Nothing => Err(format!(
			"{}: owl:Nothing is not a valid RL sub-class expression (RL §4)",
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
		// ∃R.C where C is sub-CE is valid in RL sub position (§4)
		ClassExpression::ObjectSomeValuesFrom(_, filler) => rl_sub_ce(filler, ctx),
		ClassExpression::ObjectUnionOf(_) => Err(format!(
			"{}: ObjectUnionOf is not a valid RL sub-class expression (RL §4)",
			ctx
		)),
		ClassExpression::ObjectComplementOf(_) => Err(format!(
			"{}: ObjectComplementOf is not a valid RL sub-class expression (RL §4)",
			ctx
		)),
		ClassExpression::ObjectAllValuesFrom(_, _) => Err(format!(
			"{}: ObjectAllValuesFrom is not a valid RL sub-class expression (RL §4)",
			ctx
		)),
	}
}

/// OWL 2 RL super-class expression (super-CE, right-side grammar, §4):
///
/// ```text
/// super-CE := Class | owl:Thing | owl:Nothing
///           | ObjectIntersectionOf(super-CE super-CE ...)   [≥ 2 conjuncts]
///           | ObjectUnionOf(super-CE super-CE ...)          [≥ 2 disjuncts]
///           | ObjectComplementOf(sub-CE)
///           | ObjectAllValuesFrom(OPE super-CE)
/// ```
///
/// NOT valid: `ObjectSomeValuesFrom`.
fn rl_super_ce(ce: &ClassExpression, ctx: &str) -> Result<(), String> {
	match ce {
		ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => Ok(()),
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
		// ObjectUnionOf is a valid RL super-CE (§4) — NOT valid in EL or QL super position
		ClassExpression::ObjectUnionOf(members) => {
			if members.len() < 2 {
				return Err(format!(
					"{}: ObjectUnionOf requires at least 2 members (RL §4), got {}",
					ctx,
					members.len()
				));
			}
			for m in members {
				rl_super_ce(m, ctx)?;
			}
			Ok(())
		}
		// ObjectComplementOf(sub-CE) is a valid RL super-CE (§4)
		ClassExpression::ObjectComplementOf(inner) => rl_sub_ce(inner, ctx),
		// ObjectAllValuesFrom(OPE, super-CE) is a valid RL super-CE (§4)
		ClassExpression::ObjectAllValuesFrom(_, filler) => rl_super_ce(filler, ctx),
		// ObjectSomeValuesFrom is NOT a valid RL super-CE (§4)
		ClassExpression::ObjectSomeValuesFrom(_, _) => Err(format!(
			"{}: ObjectSomeValuesFrom is not a valid RL super-class expression (RL §4)",
			ctx
		)),
	}
}
