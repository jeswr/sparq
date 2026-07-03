// [SONNET-4.6] sq-pbz04.4.2 — integration tests for the L2 syntactic OWL 2 EL/QL/RL
// profile-membership checker.
//
// 🤖 SPARQ agent. Tests use the STRUCTURAL model directly (no RDF parsing), building
// ontologies by pushing axioms to Ontology::new(). References to the W3C OWL 2 Profiles
// spec (https://www.w3.org/TR/owl2-profiles/) are cited in test names/comments.

use sparq_reason_dl::{Axiom, ClassExpression as CE, ExtractError, ObjectPropertyExpression as OPE, Ontology};
use sparq_reason_dl::profile::{profiles, profiles_from_extraction, Membership, ProfileSet};

// -------------------------------------------------------------------------------------------
// Tiny helper constructors for building structural axioms in tests.
// -------------------------------------------------------------------------------------------

/// Named class identified by a u32 dict id.
fn named(id: u32) -> CE {
	CE::Class(id)
}

/// Named object property.
fn prop(id: u32) -> OPE {
	OPE::ObjectProperty(id)
}

// Stable, human-readable class ids used across tests.
const A: u32 = 1;
const B: u32 = 2;
const C: u32 = 3;
const R: u32 = 10;

// -------------------------------------------------------------------------------------------
// Test 1: Empty ontology is In all profiles (all profiles admit the empty ontology)
// -------------------------------------------------------------------------------------------

#[test]
fn empty_ontology_in_all_profiles() {
	let onto = Ontology::new();
	let ps = profiles(&onto);
	assert_eq!(ps.el, Membership::In, "empty ontology must be In EL");
	assert_eq!(ps.ql, Membership::In, "empty ontology must be In QL");
	assert_eq!(ps.rl, Membership::In, "empty ontology must be In RL");
	assert!(ps.in_all(), "in_all() must hold for the empty ontology");
	assert!(ps.in_any(), "in_any() must hold for the empty ontology");
}

// -------------------------------------------------------------------------------------------
// Test 2: EL accept — SubClassOf(A, B) with named classes (EL §2 named class leaves)
// -------------------------------------------------------------------------------------------

#[test]
fn el_accept_subclassof_named_classes() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf { sub: named(A), sup: named(B) });
	let ps = profiles(&onto);
	assert_eq!(ps.el, Membership::In, "SubClassOf(A, B) with named classes must be In EL (§2)");
}

// -------------------------------------------------------------------------------------------
// Test 3: EL accept — SubClassOf(A ⊓ B, ∃R.A) (intersection and someValuesFrom, EL §2)
// -------------------------------------------------------------------------------------------

#[test]
fn el_accept_intersection_and_somefrom() {
	// EL-CE: ObjectIntersectionOf([A, B]) and ObjectSomeValuesFrom(R, A) both valid (§2)
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: CE::ObjectIntersectionOf(vec![named(A), named(B)]),
		sup: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(A))),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.el, Membership::In, "⊓ and ∃R.A are EL-CEs (EL §2)");
}

// -------------------------------------------------------------------------------------------
// Test 4: EL reject — SubClassOf(A, ∀R.B) (ObjectAllValuesFrom not EL, EL §2)
// -------------------------------------------------------------------------------------------

#[test]
fn el_reject_allvaluesfrom_not_el_ce() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectAllValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert!(ps.el.is_not_in(), "ObjectAllValuesFrom is not an EL-CE (EL §2)");
}

// -------------------------------------------------------------------------------------------
// Test 5: EL reject — SubClassOf(A, B ⊔ C) (ObjectUnionOf not EL, EL §2)
// -------------------------------------------------------------------------------------------

#[test]
fn el_reject_unionof_not_el_ce() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectUnionOf(vec![named(B), named(C)]),
	});
	let ps = profiles(&onto);
	assert!(ps.el.is_not_in(), "ObjectUnionOf is not an EL-CE (EL §2)");
}

// -------------------------------------------------------------------------------------------
// Test 6: EL reject — SubClassOf(A, ¬B) (ObjectComplementOf not EL, EL §2)
// -------------------------------------------------------------------------------------------

#[test]
fn el_reject_complementof_not_el_ce() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectComplementOf(Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert!(ps.el.is_not_in(), "ObjectComplementOf is not an EL-CE (EL §2)");
}

// -------------------------------------------------------------------------------------------
// Test 7: QL accept — SubClassOf(A, B) with named classes (QL §3 named class leaves)
// -------------------------------------------------------------------------------------------

#[test]
fn ql_accept_subclassof_named_classes() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf { sub: named(A), sup: named(B) });
	let ps = profiles(&onto);
	assert_eq!(ps.ql, Membership::In, "SubClassOf(A, B) with named classes must be In QL (§3)");
}

// -------------------------------------------------------------------------------------------
// Test 8: QL accept — SubClassOf(∃R.⊤, A): ∃R.⊤ is a valid QL sub-CE (QL §3)
// -------------------------------------------------------------------------------------------

#[test]
fn ql_accept_some_thing_as_sub_ce() {
	// QL §3 sub-CE includes ObjectSomeValuesFrom(OPE owl:Thing).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: CE::ObjectSomeValuesFrom(prop(R), Box::new(CE::Thing)),
		sup: named(A),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.ql, Membership::In, "∃R.⊤ ⊑ A is a valid QL axiom (QL §3 sub-CE)");
}

// -------------------------------------------------------------------------------------------
// Test 9: QL reject — SubClassOf(A, ∃R.B) where B is named: ∃ not a QL super-CE (QL §3)
// -------------------------------------------------------------------------------------------

#[test]
fn ql_reject_some_as_super_ce() {
	// ObjectSomeValuesFrom is not a valid QL super-CE (QL §3).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert!(ps.ql.is_not_in(), "∃R.B as QL super-CE is not valid (QL §3)");
}

// -------------------------------------------------------------------------------------------
// Test 10: QL reject — SubClassOf(∃R.B, A) where B ≠ owl:Thing: filler must be ⊤ (QL §3)
// -------------------------------------------------------------------------------------------

#[test]
fn ql_reject_some_non_thing_filler_as_sub_ce() {
	// QL §3 sub-CE admits ObjectSomeValuesFrom only with owl:Thing as filler.
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(B))),
		sup: named(A),
	});
	let ps = profiles(&onto);
	assert!(
		ps.ql.is_not_in(),
		"∃R.B (B ≠ owl:Thing) as QL sub-CE is not valid (QL §3)"
	);
}

// -------------------------------------------------------------------------------------------
// Test 11: QL reject — SubClassOf(A, ∀R.B): allValuesFrom not a QL super-CE (QL §3)
// -------------------------------------------------------------------------------------------

#[test]
fn ql_reject_allvaluesfrom_as_super_ce() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectAllValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert!(ps.ql.is_not_in(), "∀R.B is not a valid QL super-CE (QL §3)");
}

// -------------------------------------------------------------------------------------------
// Test 12: QL reject — SubClassOf(⊥, A): owl:Nothing not a QL sub-CE (QL §3)
// -------------------------------------------------------------------------------------------

#[test]
fn ql_reject_nothing_as_sub_ce() {
	// QL §3 sub-CE does NOT include owl:Nothing.
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: CE::Nothing,
		sup: named(A),
	});
	let ps = profiles(&onto);
	assert!(ps.ql.is_not_in(), "owl:Nothing is not a valid QL sub-CE (QL §3)");
}

// -------------------------------------------------------------------------------------------
// Test 13: RL accept — SubClassOf(A, B ⊔ C): union is a valid RL super-CE (RL §4)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_accept_union_as_super_ce() {
	// ObjectUnionOf is a valid RL super-CE (RL §4) but NOT EL or QL.
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectUnionOf(vec![named(B), named(C)]),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.rl, Membership::In, "ObjectUnionOf as RL super-CE is valid (RL §4)");
}

// -------------------------------------------------------------------------------------------
// Test 14: RL accept — SubClassOf(A, ∀R.B): allValuesFrom is a valid RL super-CE (RL §4)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_accept_allvaluesfrom_as_super_ce() {
	// ObjectAllValuesFrom is a valid RL super-CE (RL §4).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectAllValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.rl, Membership::In, "∀R.B as RL super-CE is valid (RL §4)");
}

// -------------------------------------------------------------------------------------------
// Test 15: RL accept — SubClassOf(∃R.A, B): someValuesFrom in sub position is RL (RL §4)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_accept_some_in_sub_position() {
	// RL §4 sub-CE includes ObjectSomeValuesFrom(OPE sub-CE).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(A))),
		sup: named(B),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.rl, Membership::In, "∃R.A in RL sub position is valid (RL §4)");
}

// -------------------------------------------------------------------------------------------
// Test 16: RL reject — SubClassOf(A, ∃R.B): someValuesFrom NOT a RL super-CE (RL §4)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_reject_some_in_super_position() {
	// ObjectSomeValuesFrom is not a valid RL super-CE (RL §4).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert!(ps.rl.is_not_in(), "∃R.B as RL super-CE is not valid (RL §4)");
}

// -------------------------------------------------------------------------------------------
// Test 17: RL reject — SubClassOf(A ⊔ B, C): union NOT a RL sub-CE (RL §4)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_reject_union_in_sub_position() {
	// ObjectUnionOf is not a valid RL sub-CE (RL §4).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: CE::ObjectUnionOf(vec![named(A), named(B)]),
		sup: named(C),
	});
	let ps = profiles(&onto);
	assert!(ps.rl.is_not_in(), "ObjectUnionOf in RL sub position is not valid (RL §4)");
}

// -------------------------------------------------------------------------------------------
// Test 18: Cross-profile — SubClassOf(A, ∀R.B): NotIn EL, NotIn QL, In RL
// -------------------------------------------------------------------------------------------

#[test]
fn cross_allvalues_not_el_not_ql_in_rl() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectAllValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert!(ps.el.is_not_in(), "∀R.B is not EL (EL §2)");
	assert!(ps.ql.is_not_in(), "∀R.B is not a valid QL super-CE (QL §3)");
	assert_eq!(ps.rl, Membership::In, "∀R.B is a valid RL super-CE (RL §4)");
	assert!(!ps.in_all());
	assert!(ps.in_any());
}

// -------------------------------------------------------------------------------------------
// Test 19: Cross-profile — SubClassOf(A, ∃R.B): In EL, NotIn QL, NotIn RL
// -------------------------------------------------------------------------------------------

#[test]
fn cross_some_in_el_not_ql_not_rl() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(B))),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.el, Membership::In, "∃R.B as super is a valid EL-CE (EL §2)");
	assert!(ps.ql.is_not_in(), "∃R.B as QL super-CE is not valid (QL §3)");
	assert!(ps.rl.is_not_in(), "∃R.B as RL super-CE is not valid (RL §4)");
	assert!(!ps.in_all());
	assert!(ps.in_any());
}

// -------------------------------------------------------------------------------------------
// Test 20: Cross-profile — SubClassOf(A, B ⊔ C): NotIn EL, NotIn QL, In RL
// -------------------------------------------------------------------------------------------

#[test]
fn cross_union_not_el_not_ql_in_rl() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf {
		sub: named(A),
		sup: CE::ObjectUnionOf(vec![named(B), named(C)]),
	});
	let ps = profiles(&onto);
	assert!(ps.el.is_not_in(), "ObjectUnionOf is not EL (EL §2)");
	assert!(ps.ql.is_not_in(), "ObjectUnionOf is not a valid QL super-CE (QL §3)");
	assert_eq!(ps.rl, Membership::In, "ObjectUnionOf as RL super-CE is valid (RL §4)");
	assert!(!ps.in_all());
	assert!(ps.in_any());
}

// -------------------------------------------------------------------------------------------
// Test 21: profiles_from_extraction with Ok ontology delegates to profiles()
// -------------------------------------------------------------------------------------------

#[test]
fn profiles_from_extraction_ok_delegates() {
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::SubClassOf { sub: named(A), sup: named(B) });
	let result: Result<Ontology, ExtractError> = Ok(onto.clone());
	let ps_direct = profiles(&onto);
	let ps_via = profiles_from_extraction(&result);
	assert_eq!(ps_direct.el, ps_via.el);
	assert_eq!(ps_direct.ql, ps_via.ql);
	assert_eq!(ps_direct.rl, ps_via.rl);
}

// -------------------------------------------------------------------------------------------
// Test 22: profiles_from_extraction with Err yields Unknown for all three profiles
// -------------------------------------------------------------------------------------------

#[test]
fn profiles_from_extraction_err_all_unknown() {
	let result: Result<Ontology, ExtractError> =
		Err(ExtractError::OutOfFragment("test-fragment".into()));
	let ps = profiles_from_extraction(&result);
	assert!(ps.el.is_unknown(), "extraction failure must yield Unknown EL");
	assert!(ps.ql.is_unknown(), "extraction failure must yield Unknown QL");
	assert!(ps.rl.is_unknown(), "extraction failure must yield Unknown RL");
	// Unknown is not In, so in_all and in_any should both be false.
	assert!(!ps.in_all());
	assert!(!ps.in_any());
}

// -------------------------------------------------------------------------------------------
// Test 23: EquivalentClasses(A, B) with named classes is In RL (§4 requires sub∧super)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_accept_equivalentclasses_named() {
	// Named classes are BOTH RL sub-CE AND RL super-CE, so EquivalentClasses(A, B) is RL.
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::EquivalentClasses(named(A), named(B)));
	let ps = profiles(&onto);
	assert_eq!(ps.rl, Membership::In, "EquivalentClasses(A, B) must be In RL (RL §4)");
}

// -------------------------------------------------------------------------------------------
// Test 24: EquivalentClasses(A, ∃R.C) is NotIn RL (∃R.C is RL sub-CE but NOT super-CE)
// -------------------------------------------------------------------------------------------

#[test]
fn rl_reject_equivalentclasses_some_not_super_ce() {
	// RL §4: each CE in EquivalentClasses must be BOTH sub-CE and super-CE.
	// ∃R.C is a valid RL sub-CE but NOT a valid RL super-CE → the equivalence fails.
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::EquivalentClasses(
		named(A),
		CE::ObjectSomeValuesFrom(prop(R), Box::new(named(C))),
	));
	let ps = profiles(&onto);
	assert!(
		ps.rl.is_not_in(),
		"EquivalentClasses(A, ∃R.C) must be NotIn RL because ∃R.C is not RL super-CE (RL §4)"
	);
}

// -------------------------------------------------------------------------------------------
// Test 25: Membership helpers (is_in / is_not_in / is_unknown) and ProfileSet (in_all / in_any)
// -------------------------------------------------------------------------------------------

#[test]
fn membership_helpers_and_profileset_helpers() {
	let m_in = Membership::In;
	let m_not = Membership::NotIn("violation".into());
	let m_unk = Membership::Unknown("err".into());

	assert!(m_in.is_in());
	assert!(!m_in.is_not_in());
	assert!(!m_in.is_unknown());

	assert!(!m_not.is_in());
	assert!(m_not.is_not_in());
	assert!(!m_not.is_unknown());

	assert!(!m_unk.is_in());
	assert!(!m_unk.is_not_in());
	assert!(m_unk.is_unknown());

	// in_all: all In
	let all_in = ProfileSet {
		el: Membership::In,
		ql: Membership::In,
		rl: Membership::In,
	};
	assert!(all_in.in_all());
	assert!(all_in.in_any());

	// in_all: none In
	let none_in = ProfileSet {
		el: Membership::NotIn("x".into()),
		ql: Membership::NotIn("x".into()),
		rl: Membership::NotIn("x".into()),
	};
	assert!(!none_in.in_all());
	assert!(!none_in.in_any());

	// in_any: one In
	let one_in = ProfileSet {
		el: Membership::In,
		ql: Membership::NotIn("x".into()),
		rl: Membership::NotIn("x".into()),
	};
	assert!(!one_in.in_all());
	assert!(one_in.in_any());
}

// -------------------------------------------------------------------------------------------
// Test 26: DisjointClasses(A, B) is In all profiles (named classes valid everywhere)
// -------------------------------------------------------------------------------------------

#[test]
fn disjoint_classes_named_in_all_profiles() {
	// DisjointClasses requires both operands to be EL-CE (EL), QL sub-CE (QL), RL sub-CE (RL).
	// Named classes satisfy all of these.
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::DisjointClasses(named(A), named(B)));
	let ps = profiles(&onto);
	assert_eq!(ps.el, Membership::In, "DisjointClasses(A, B) must be In EL (§2)");
	assert_eq!(ps.ql, Membership::In, "DisjointClasses(A, B) must be In QL (§3)");
	assert_eq!(ps.rl, Membership::In, "DisjointClasses(A, B) must be In RL (§4)");
	assert!(ps.in_all());
}

// -------------------------------------------------------------------------------------------
// Test 27: ObjectPropertyDomain / ObjectPropertyRange axioms
// -------------------------------------------------------------------------------------------

#[test]
fn property_domain_range_named_in_all_profiles() {
	// Domain/range with a named class: valid EL-CE (EL §2), QL super-CE (QL §3), RL super-CE (RL §4).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::ObjectPropertyDomain {
		property: prop(R),
		domain: named(A),
	});
	onto.axioms.push(Axiom::ObjectPropertyRange {
		property: prop(R),
		range: named(A),
	});
	let ps = profiles(&onto);
	assert_eq!(ps.el, Membership::In, "PropertyDomain/Range(R, A) must be In EL (§2)");
	assert_eq!(ps.ql, Membership::In, "PropertyDomain/Range(R, A) must be In QL (§3)");
	assert_eq!(ps.rl, Membership::In, "PropertyDomain/Range(R, A) must be In RL (§4)");
	assert!(ps.in_all());
}

#[test]
fn property_domain_some_not_ql_super_ce() {
	// ObjectPropertyDomain with ∃R.A as domain: not a valid QL super-CE (QL §3).
	let mut onto = Ontology::new();
	onto.axioms.push(Axiom::ObjectPropertyDomain {
		property: prop(R),
		domain: CE::ObjectSomeValuesFrom(prop(R), Box::new(named(A))),
	});
	let ps = profiles(&onto);
	assert!(ps.ql.is_not_in(), "∃R.A as QL property domain super-CE is not valid (QL §3)");
}
