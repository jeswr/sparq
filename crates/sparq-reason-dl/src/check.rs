//! L4 — fragment-dispatch Direct-Semantics checker + entailment-by-refutation.
//!
//! 🤖 SPARQ agent [FABLE-5]. Bead sq-pbz04.4.4 (epic sq-pbz04.4, design record
//! `research/owl2-direct-semantics-scoping.md` §4 — the SPEC; its soundness arguments are
//! binding on this module). Feature-gated behind `dispatch`, which pulls the existing
//! profile reasoners (`sparq-reason` RL, `sparq-reason-el` EL) as optional dependencies.
//!
//! # Dispatch (design record §4)
//!
//! [`DirectChecker::consistency`] extracts the graph through the fail-closed L1 mapping,
//! asks the L2 syntactic profile checker where the ontology lives, and dispatches IN ORDER:
//!
//! | # | Branch | Engine | `Consistent` | `Inconsistent` | Abstention |
//! |---|--------|--------|--------------|----------------|------------|
//! | 1 | RL (L2 says in-RL) | `sparq_reason::materialize(Profile::OwlRl)` + `inconsistencies()` | only past the **divergence guard** | sound via **Theorem PR1** (preconditions CHECKED, not assumed) | [`UnknownReason::RlPr1Preconditions`], [`UnknownReason::RlDivergenceGuard`] |
//! | 2 | EL (in-EL) | `sparq_reason_el::Classifier::classify` | only for a pure ⊤-free EL+⊥ TBox (argument below) | never (see the ⊤ guard) | [`UnknownReason::ElSkippedAxioms`], [`UnknownReason::ElUnappliedAxioms`], [`UnknownReason::ElTopGuard`] |
//! | 3 | QL (in-QL) | none — deferred | never | never | [`UnknownReason::QlConsistencyPending`] (always; DL-Lite_R consistency is sq-pbz04.3.4's, not duplicated here) |
//! | 4 | ALCH (everything else L1 extracted) | the L3 tableau | complete for the fragment | complete for the fragment | [`UnknownReason::ResourceBudget`] |
//! | — | extraction failed | none | never | never | [`UnknownReason::OutOfFragment`] |
//!
//! The first branch whose profile test matches OWNS the verdict — an abstaining branch does
//! NOT fall through (the record specifies dispatch-in-order; a tableau fallback for
//! guard-abstained RL/EL/QL inputs is possible future work, recorded as such, not silently
//! added). Every verdict carries the [`Branch`] that produced it (the traceability
//! invariant), and every guard fails CLOSED: uncertainty is [`UnknownReason`], never a
//! guessed `Consistent`/`Inconsistent`.
//!
//! ## RL branch soundness
//!
//! - `Inconsistent`: the implemented RL/RDF rules are sound, and for RL ontologies Theorem
//!   PR1 (W3C OWL 2 Profiles §4.3) ties rule-derived inconsistency to Direct-Semantics
//!   inconsistency. The PR1 preconditions are **checked**: (a) *no punning* — no id is used
//!   as more than one of {class, object property, individual} anywhere in the L1 model
//!   (usage-level punning is exactly what reaches the rule closure; declaration-only typing
//!   contributes no logical triples in the L1 fragment, and unknown typings fail extraction);
//!   (b) *no annotation axioms* — guaranteed structurally: the fail-closed L1 mapping has no
//!   annotation-axiom shape (built-in annotations are ignored, unknown predicates refuse the
//!   graph). A punning violation abstains BOTH verdicts (under punning the input is not an
//!   OWL 2 DL ontology, so a Direct-Semantics verdict is ill-posed).
//! - `Consistent` ("no clash") additionally needs rule-set *completeness*, which sparq's RL
//!   does not globally have — 13 documented divergences (`DOCUMENTED_DIVERGENCES` in
//!   sparq-conformance's `owl_suite.rs`). The **divergence guard** abstains when the model
//!   touches any implicated construct reachable through the L1∩RL funnel:
//!   `owl:complementOf` (DisjointClasses-001/-003, New-Feature-ObjectQCR-002),
//!   `owl:unionOf` (WebOnt-I5.5-005), and `owl:disjointWith` (DisjointClasses-001/-003) —
//!   a deliberate over-approximation (fail-closed). The other ten entries implicate
//!   constructs (property chains/transitivity, maxQC, `differentFrom`/`AllDifferent`,
//!   functional/inverse-functional/reflexive/disjoint properties, datatype ranges) that L1
//!   extraction already refuses, so they cannot reach this branch.
//!
//! ## EL branch soundness
//!
//! `Classifier::classify` is a TBox classifier: it applies `subClassOf` /
//! `equivalentClass` / `disjointWith` axioms only, counting out-of-fragment class
//! expressions in `Report::skipped_axioms`; ABox assertions, `rdfs:subPropertyOf`,
//! `rdfs:domain`, and `rdfs:range` are neither applied nor counted, and it does NOT track
//! the satisfiability of ⊤ (verified empirically: `owl:Thing rdfs:subClassOf owl:Nothing`
//! yields an empty unsatisfiable-class list). Three guards therefore protect the verdict:
//! skipped axioms abstain ([`UnknownReason::ElSkippedAxioms`], per the record); axiom kinds
//! the classifier does not apply abstain ([`UnknownReason::ElUnappliedAxioms`]); any
//! occurrence of `owl:Thing` abstains ([`UnknownReason::ElTopGuard`]). Past all three the
//! input is a pure EL+⊥ TBox not mentioning ⊤, and `Consistent` is sound by a direct model
//! construction: interpret every named class and role as empty over a one-element domain —
//! by induction every ⊤-free EL class expression (named class, ⊥, ⊓, ∃) denotes ∅, so every
//! GCI/equivalence/disjointness holds. This branch never returns `Inconsistent` (a pure
//! ⊤-free EL+⊥ TBox is always consistent; anything else is guarded away).
//!
//! # Entailment by refutation (design record §4)
//!
//! [`DirectChecker::entailment`] checks `O ⊨ α` per conclusion axiom by encoding the
//! refutation into a consistency question and deciding it with the **L3 ALCH tableau**
//! (sound and complete for the entire L1 fragment, which contains every premise the L1
//! mapping accepts and every encoding below — so routing refutations to the tableau, the
//! strongest complete branch, preserves the record's "definitive verdicts only from a
//! complete branch" rule; RL/EL refutation routes would add only budget robustness and are
//! future work):
//!
//! - `SubClassOf(C, D)` — `O ∪ {(C ⊓ ¬D)(x)}` with `x` fresh: inconsistent iff `O ⊨ C ⊑ D`.
//! - `ClassAssertion(C, a)` — `O ∪ {(¬C)(a)}`: inconsistent iff `O ⊨ C(a)`.
//! - `ObjectPropertyAssertion(R, a, b)` — `O ∪ {B(b), (∀R.¬B)(a)}` with `B` a FRESH class
//!   name. Sound AND complete (record §4; re-verified for this implementation): if every
//!   model has `(a,b) ∈ R` then `b ∈ B` and `a ∈ ∀R.¬B` clash, so the union is
//!   inconsistent; conversely a model `I` of `O` with `(aᴵ,bᴵ) ∉ Rᴵ` extends to a model of
//!   the union by interpreting `Bᴵ = {bᴵ}` (`B` occurs nowhere in `O`, and no R-successor
//!   of `aᴵ` is `bᴵ`), so non-entailment yields consistency.
//! - `EquivalentClasses(C, D)` — the two GCI refutations (`C ⊑ D` and `D ⊑ C`); exact by
//!   the Direct-Semantics desugaring the record §3 lists.
//! - `DisjointClasses(C, D)` — `O ∪ {(C ⊓ D)(x)}`, `x` fresh: the `C ⊓ D ⊑ ⊥` desugaring.
//! - `ObjectPropertyDomain(R, C)` / `ObjectPropertyRange(R, C)` — the `∃R.⊤ ⊑ C` /
//!   `⊤ ⊑ ∀R.C` desugarings (record §3), then the GCI refutation.
//! - `SubObjectPropertyOf(R, S)` — `O ∪ {R(a,b), B(b), (∀S.¬B)(a)}` with `a`, `b` FRESH
//!   individuals and `B` a FRESH class name ([FABLE-5] sq-pbz04.4.9; record §4 as amended —
//!   the role-subsumption lift of the fresh-class trick). Sound AND complete: if every
//!   model of `O` has `Rᴵ ⊆ Sᴵ`, then in any model of the union `(a,b) ∈ R ⊆ S` makes `b`
//!   an `S`-successor of `a`, so `a ∈ ∀S.¬B` forces `b ∈ ¬B`, clashing with `B(b)` — the
//!   union is inconsistent. Conversely a model `I` of `O` with some `(u,v) ∈ Rᴵ ∖ Sᴵ`
//!   extends to a model of the union via `aᴵ = u`, `bᴵ = v`, `Bᴵ = {v}` (`a`/`b`/`B` occur
//!   nowhere in `O`; `(u,v) ∉ Sᴵ` means no `S`-successor of `u` is `v`, the sole member of
//!   `B`, so `(∀S.¬B)(a)` holds) — non-entailment yields consistency. The tableau side
//!   holds because its `∀`-rule fires modulo the role hierarchy (`System::is_subrole`,
//!   reflexive-transitive), so the `R`-edge `a → b` is seen by `∀S` exactly when
//!   `O ⊢ R ⊑* S`. All three additions stay inside the L1 fragment.
//!
//! `Entailed` needs every conclusion axiom's refutation(s) unsatisfiable; a single
//! definitively-satisfiable refutation is `NotEntailed` (sound because the tableau is
//! complete — exactly the record's rule that negative-entailment verdicts come only from a
//! complete branch); anything else abstains. An empty conclusion ontology is trivially
//! `Entailed`. Fresh names are minted as `urn:` IRIs checked against every id occurring in
//! the premise AND conclusion models.
//!
//! ## Conclusion anonymous individuals ([OPUS-4.8] sq-pbz04.4.13)
//!
//! Official OWL 2 Direct-Semantics entailment reads a blank-node individual in the CONCLUSION
//! EXISTENTIALLY (`a p _:x` means "`a` has *some* `p`-successor", not "`a p c` for a specific
//! constant `c`"). L1 maps a blank node to an ordinary (skolem) constant: entailment-preserving
//! on the PREMISE side (kept), but UNSOUND on the CONCLUSION side — the skolem refutation of
//! `a p _:x` stays satisfiable even when the premise's `∃p.⊤` typing grants the existential, so
//! it would certify a WRONG `NotEntailed`. Before the refutation loop, `roll_conclusion_anonymous`
//! resolves conclusion blank nodes under the existential reading: a TREE-shaped anonymous
//! assertion set (each blank node the object of exactly one property edge, reaching only blank
//! successors, acyclic, anchored to a named root) ROLLS UP into an existential class assertion
//! `root : ∃p.(⊓ types ⊓ ⊓ ∃q.⟨child⟩)` the tableau decides SOUNDLY in both directions (so
//! somevaluesfrom2bnode / WebOnt-someValuesFrom-003 become genuine passes). Any non-rollable
//! shape — shared between two assertions, cyclic, a named/nominal successor, or an unanchored
//! free-existential root — abstains fail-closed ([`UnknownReason::ConclusionAnonymousIndividual`]),
//! NEVER a skolem-constant `NotEntailed`.
//!
//! # Honest boundary
//!
//! This module never claims completeness beyond the argued fragment: the only complete
//! branch is the ALCH tableau (within budget); RL `Consistent` is guard-narrowed, EL
//! `Consistent` is ⊤-free-TBox-narrowed, QL consistency is wholly deferred, and every
//! uncertain case is a typed [`UnknownReason`].

use crate::extract::extract;
use crate::model::{Axiom, ClassExpression, ObjectPropertyExpression, Ontology};
use crate::profile::profiles;
use crate::tableau::{self, Budget, ExhaustedBudget};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{is_inline, Dict, Id, TermParts};
use sparq_reason::{inconsistencies, materialize, Profile};
use sparq_reason_el::Classifier;

// -------------------------------------------------------------------------------------------
// Public verdict types
// -------------------------------------------------------------------------------------------

/// The dispatch branch that produced a verdict — the traceability half of the module
/// invariant: every verdict is attributable to a branch whose soundness/completeness story
/// is written in the module docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    /// The fail-closed L1 extraction itself (the graph never reached a reasoner).
    Extraction,
    /// OWL 2 RL materialization + clash scan (`sparq-reason`), PR1-checked.
    RlMaterialization,
    /// OWL 2 EL consequence-based classification (`sparq-reason-el`), triple-guarded.
    ElClassification,
    /// OWL 2 QL — consistency wholly deferred to the QL workstream (sq-pbz04.3.4).
    QlDeferred,
    /// The L3 ALCH completion-forest tableau (complete for the L1 fragment).
    AlchTableau,
}

/// Why the checker abstained. Fail-closed: no reason ever degrades into a verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnknownReason {
    /// The L1 mapping refused the graph (rendered `ExtractError`): the input uses a
    /// construct outside the ALCH structural fragment, or is malformed.
    OutOfFragment(String),
    /// The RL branch matched but a Theorem PR1 precondition failed (usage-level punning);
    /// the Direct-Semantics bridge does not apply, so BOTH verdicts are withheld.
    RlPr1Preconditions(String),
    /// The RL branch found no clash, but the model touches a construct implicated in the
    /// documented RL rule-set divergences, so "no clash" does not certify consistency.
    RlDivergenceGuard(String),
    /// The EL classifier skipped that many axioms (out-of-fragment class expressions); a
    /// skipped axiom could be the inconsistency.
    ElSkippedAxioms(usize),
    /// The model contains an axiom kind the EL classifier does not apply (ABox assertions,
    /// `rdfs:subPropertyOf`, `rdfs:domain`, `rdfs:range`) — and does not count as skipped.
    ElUnappliedAxioms(String),
    /// The model mentions `owl:Thing`: the EL classifier does not track ⊤'s
    /// satisfiability (empirically verified), so a ⊤-involving verdict is withheld.
    ElTopGuard,
    /// The ontology dispatched to the QL branch, whose Direct-Semantics consistency
    /// procedure is deferred to the QL workstream (sq-pbz04.3.4) — never duplicated here.
    QlConsistencyPending,
    /// The ALCH tableau exhausted its deterministic count budget mid-search.
    ResourceBudget(ExhaustedBudget),
    /// Entailment only: the conclusion-axiom kind has no argued refutation encoding.
    /// Every axiom kind the L1 model can currently express HAS an argued encoding
    /// (`SubObjectPropertyOf` gained one under sq-pbz04.4.9), so this reason is not
    /// produced today — it is RETAINED fail-closed for future model growth (a new axiom
    /// kind abstains here until its encoding is argued in the design record). [FABLE-5]
    UnencodedConclusion(String),
    /// Entailment only: the CONCLUSION ontology mentions a blank-node individual in a shape
    /// that does NOT roll up into a tree-shaped existential class assertion (it is shared
    /// between two assertions, cyclic, has a named or a nominal successor, or is a free
    /// existential root). Official OWL 2 Direct-Semantics entailment reads a conclusion
    /// blank node EXISTENTIALLY, so the L1 skolem-constant reading would be UNSOUND for a
    /// `NotEntailed` verdict — the checker abstains fail-closed instead (sq-pbz04.4.13).
    /// The payload names the first offending anonymous individual. [OPUS-4.8]
    ConclusionAnonymousIndividual(String),
}

/// Tri-state Direct-Semantics consistency verdict (design record §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsistencyVerdict {
    /// The ontology has a model — asserted only by a branch complete for its guard.
    Consistent,
    /// The ontology has no model — asserted only where the producing branch is sound.
    Inconsistent,
    /// No verdict; the payload says exactly why (fail-closed).
    Unknown(UnknownReason),
}

impl ConsistencyVerdict {
    /// `true` iff the verdict is [`ConsistencyVerdict::Consistent`].
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        matches!(self, ConsistencyVerdict::Consistent)
    }

    /// `true` iff the verdict is [`ConsistencyVerdict::Inconsistent`].
    #[must_use]
    pub fn is_inconsistent(&self) -> bool {
        matches!(self, ConsistencyVerdict::Inconsistent)
    }

    /// `true` iff the checker abstained.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, ConsistencyVerdict::Unknown(_))
    }
}

/// Tri-state Direct-Semantics entailment verdict (refutation-based, design record §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntailmentVerdict {
    /// Every conclusion axiom is entailed (each refutation is unsatisfiable).
    Entailed,
    /// At least one conclusion axiom is definitively NOT entailed (its refutation is
    /// satisfiable, certified by the complete ALCH tableau).
    NotEntailed,
    /// No verdict; the payload says exactly why (fail-closed).
    Unknown(UnknownReason),
}

impl EntailmentVerdict {
    /// `true` iff the verdict is [`EntailmentVerdict::Entailed`].
    #[must_use]
    pub fn is_entailed(&self) -> bool {
        matches!(self, EntailmentVerdict::Entailed)
    }

    /// `true` iff the verdict is [`EntailmentVerdict::NotEntailed`].
    #[must_use]
    pub fn is_not_entailed(&self) -> bool {
        matches!(self, EntailmentVerdict::NotEntailed)
    }

    /// `true` iff the checker abstained.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, EntailmentVerdict::Unknown(_))
    }
}

/// A consistency verdict plus the branch that produced it (traceability invariant).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsistencyOutcome {
    /// The tri-state verdict.
    pub verdict: ConsistencyVerdict,
    /// The dispatch branch that owned the decision.
    pub branch: Branch,
}

/// An entailment verdict plus the branch that produced it (traceability invariant).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntailmentOutcome {
    /// The tri-state verdict.
    pub verdict: EntailmentVerdict,
    /// The branch that owned the decision — an ATTRIBUTION, not a claim that a tableau
    /// ran: [`Branch::AlchTableau`] covers both executed refutations and pre-tableau
    /// abstentions (e.g. `UnencodedConclusion`, where the verdict is `Unknown` and no
    /// refutation was attempted); [`Branch::Extraction`] when an input graph was refused.
    pub branch: Branch,
}

// -------------------------------------------------------------------------------------------
// The checker
// -------------------------------------------------------------------------------------------

/// The L4 fragment-dispatch Direct-Semantics checker (module docs; design record §4).
///
/// Stateless apart from the deterministic tableau [`Budget`] used by the ALCH branch and
/// every refutation check. Construct with [`DirectChecker::new`] (default budget) or
/// [`DirectChecker::with_budget`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectChecker {
    budget: Budget,
}

impl Default for DirectChecker {
    fn default() -> DirectChecker {
        DirectChecker::new()
    }
}

impl DirectChecker {
    /// A checker with the default deterministic tableau budget.
    #[must_use]
    pub fn new() -> DirectChecker {
        DirectChecker {
            budget: Budget::default(),
        }
    }

    /// A checker with a caller-pinned tableau budget (conformance lanes pin their own for
    /// reproducibility).
    #[must_use]
    pub fn with_budget(budget: Budget) -> DirectChecker {
        DirectChecker { budget }
    }

    /// Decide Direct-Semantics consistency of the ontology encoded in `triples` by
    /// fragment dispatch (module docs table).
    ///
    /// `dict` is mutable because the RL branch materializes (interning vocabulary /
    /// derived terms); existing ids are never changed. The dispatch order is RL → EL →
    /// QL → ALCH; extraction failure short-circuits to
    /// [`UnknownReason::OutOfFragment`]. Every guard fails closed.
    #[must_use]
    pub fn consistency(&self, dict: &mut Dict, triples: &[[Id; 3]]) -> ConsistencyOutcome {
        let onto = match extract(dict, triples) {
            Ok(onto) => onto,
            Err(e) => {
                return ConsistencyOutcome {
                    verdict: ConsistencyVerdict::Unknown(UnknownReason::OutOfFragment(format!(
                        "{}",
                        e
                    ))),
                    branch: Branch::Extraction,
                }
            }
        };
        let ps = profiles(&onto);
        if ps.rl.is_in() {
            return rl_branch(dict, triples, &onto);
        }
        if ps.el.is_in() {
            return el_branch(dict, triples, &onto);
        }
        if ps.ql.is_in() {
            return ConsistencyOutcome {
                verdict: ConsistencyVerdict::Unknown(UnknownReason::QlConsistencyPending),
                branch: Branch::QlDeferred,
            };
        }
        // Everything the L1 mapping accepts is inside the ALCH fragment by construction.
        ConsistencyOutcome {
            verdict: tableau_consistency(&onto, self.budget),
            branch: Branch::AlchTableau,
        }
    }

    /// Decide whether the premise ontology entails the conclusion ontology (both given as
    /// triples over the SAME `dict`), by per-axiom refutation on the ALCH tableau
    /// (module docs).
    ///
    /// `dict` is mutable only to mint the fresh `urn:` individual/class names the
    /// encodings need; existing ids are never changed. `Entailed` requires every
    /// conclusion axiom entailed; the first definitively-not-entailed axiom returns
    /// `NotEntailed`; otherwise any abstention returns `Unknown` (fail-closed). An empty
    /// conclusion is trivially `Entailed`.
    #[must_use]
    pub fn entailment(
        &self,
        dict: &mut Dict,
        premise: &[[Id; 3]],
        conclusion: &[[Id; 3]],
    ) -> EntailmentOutcome {
        let unknown_extraction = |e: String| EntailmentOutcome {
            verdict: EntailmentVerdict::Unknown(UnknownReason::OutOfFragment(e)),
            branch: Branch::Extraction,
        };
        let prem = match extract(dict, premise) {
            Ok(onto) => onto,
            Err(e) => return unknown_extraction(format!("premise: {}", e)),
        };
        let concl = match extract(dict, conclusion) {
            Ok(onto) => onto,
            Err(e) => return unknown_extraction(format!("conclusion: {}", e)),
        };
        // [OPUS-4.8] sq-pbz04.4.13 — conclusion anonymous individuals are EXISTENTIAL under
        // the official OWL 2 Direct-Semantics entailment reading. L1 maps a conclusion blank
        // node to an ordinary (skolem) constant; that reading is entailment-preserving on the
        // PREMISE side but UNSOUND on the conclusion side (it would certify a `NotEntailed`
        // that is wrong existentially — the pinned M2 divergences somevaluesfrom2bnode /
        // WebOnt-someValuesFrom-003). `roll_conclusion_anonymous` rolls a tree-shaped bnode
        // assertion set into an existential class assertion the tableau decides SOUNDLY (both
        // directions); a non-rollable shape (shared / cyclic / nominal / free-root bnode)
        // abstains fail-closed, never a skolem `NotEntailed`.
        let concl_axioms = match roll_conclusion_anonymous(dict, &concl) {
            Ok(axioms) => axioms,
            Err(reason) => {
                return EntailmentOutcome {
                    verdict: EntailmentVerdict::Unknown(reason),
                    branch: Branch::AlchTableau,
                }
            }
        };
        let mut fresh = FreshNames::new(dict, [&prem, &concl]);
        let mut first_unknown: Option<UnknownReason> = None;
        for axiom in &concl_axioms {
            match self.axiom_entailed(&prem, axiom, &mut fresh) {
                AxiomVerdict::Entailed => {}
                AxiomVerdict::NotEntailed => {
                    // Sound early exit: the conjunction of conclusion axioms fails as soon
                    // as one conjunct definitively fails, regardless of abstentions.
                    return EntailmentOutcome {
                        verdict: EntailmentVerdict::NotEntailed,
                        branch: Branch::AlchTableau,
                    };
                }
                AxiomVerdict::Unknown(reason) => {
                    first_unknown.get_or_insert(reason);
                }
            }
        }
        let verdict = match first_unknown {
            Some(reason) => EntailmentVerdict::Unknown(reason),
            None => EntailmentVerdict::Entailed,
        };
        EntailmentOutcome {
            verdict,
            branch: Branch::AlchTableau,
        }
    }

    /// One conclusion axiom: build its refutation check(s), decide each on the tableau,
    /// and aggregate (all-unsat = entailed; any-sat = not entailed; else unknown).
    fn axiom_entailed(
        &self,
        premise: &Ontology,
        conclusion: &Axiom,
        fresh: &mut FreshNames,
    ) -> AxiomVerdict {
        let Some(checks) = refutation_checks(conclusion, fresh) else {
            return AxiomVerdict::Unknown(UnknownReason::UnencodedConclusion(format!(
                "no argued refutation encoding for the conclusion-axiom kind of {:?}",
                conclusion
            )));
        };
        let mut first_unknown: Option<UnknownReason> = None;
        for additions in checks {
            let mut augmented = premise.clone();
            augmented.axioms.extend(additions);
            match tableau::consistency(&augmented, self.budget) {
                tableau::Verdict::Unsatisfiable => {} // this component is entailed
                tableau::Verdict::Satisfiable => return AxiomVerdict::NotEntailed,
                tableau::Verdict::Unknown(tableau::UnknownReason::ResourceBudget(b)) => {
                    first_unknown.get_or_insert(UnknownReason::ResourceBudget(b));
                }
                tableau::Verdict::Unknown(tableau::UnknownReason::OutOfFragment(m)) => {
                    // Unreachable in practice (both inputs already extracted, and every
                    // encoding stays inside ALCH) — kept fail-closed regardless.
                    first_unknown.get_or_insert(UnknownReason::OutOfFragment(m));
                }
            }
        }
        match first_unknown {
            Some(reason) => AxiomVerdict::Unknown(reason),
            None => AxiomVerdict::Entailed,
        }
    }
}

/// Per-conclusion-axiom tri-state (internal aggregation of one axiom's refutation checks).
enum AxiomVerdict {
    Entailed,
    NotEntailed,
    Unknown(UnknownReason),
}

// -------------------------------------------------------------------------------------------
// RL branch
// -------------------------------------------------------------------------------------------

/// The RL branch (module docs §"RL branch soundness").
fn rl_branch(dict: &mut Dict, triples: &[[Id; 3]], onto: &Ontology) -> ConsistencyOutcome {
    let out = |verdict| ConsistencyOutcome {
        verdict,
        branch: Branch::RlMaterialization,
    };
    if let Some(msg) = pr1_punning_violation(onto) {
        return out(ConsistencyVerdict::Unknown(
            UnknownReason::RlPr1Preconditions(msg),
        ));
    }
    let mut work = triples.to_vec();
    materialize(Profile::OwlRl, dict, &mut work);
    let clashes = inconsistencies(dict, &work);
    if !clashes.is_empty() {
        return out(ConsistencyVerdict::Inconsistent);
    }
    if let Some(construct) = rl_divergence_guard(onto) {
        return out(ConsistencyVerdict::Unknown(
            UnknownReason::RlDivergenceGuard(construct),
        ));
    }
    out(ConsistencyVerdict::Consistent)
}

/// Usage-level punning scan for the PR1 precondition: no id may be used as more than one
/// of {class, object property, individual} across the model. Returns a diagnostic naming
/// the first overlap.
fn pr1_punning_violation(onto: &Ontology) -> Option<String> {
    let mut classes: FxHashSet<Id> = FxHashSet::default();
    let mut properties: FxHashSet<Id> = FxHashSet::default();
    let mut individuals: FxHashSet<Id> = FxHashSet::default();
    let mut ces: Vec<&ClassExpression> = Vec::new();
    for axiom in onto.axioms() {
        ces.extend(axiom_ces(axiom));
        match axiom {
            Axiom::SubObjectPropertyOf { sub, sup } => {
                properties.extend([sub.named(), sup.named()]);
            }
            Axiom::ObjectPropertyDomain { property, .. }
            | Axiom::ObjectPropertyRange { property, .. } => {
                properties.insert(property.named());
            }
            Axiom::ClassAssertion { individual, .. } => {
                individuals.insert(*individual);
            }
            Axiom::ObjectPropertyAssertion {
                property,
                source,
                target,
            } => {
                properties.insert(property.named());
                individuals.extend([*source, *target]);
            }
            Axiom::SubClassOf { .. }
            | Axiom::EquivalentClasses(_, _)
            | Axiom::DisjointClasses(_, _) => {}
        }
    }
    while let Some(ce) = ces.pop() {
        match ce {
            ClassExpression::Class(id) => {
                classes.insert(*id);
            }
            ClassExpression::Thing | ClassExpression::Nothing => {}
            ClassExpression::ObjectIntersectionOf(members)
            | ClassExpression::ObjectUnionOf(members) => ces.extend(members),
            ClassExpression::ObjectComplementOf(inner) => ces.push(inner),
            ClassExpression::ObjectSomeValuesFrom(p, filler)
            | ClassExpression::ObjectAllValuesFrom(p, filler) => {
                properties.insert(p.named());
                ces.push(filler);
            }
        }
    }
    let overlap = |a: &FxHashSet<Id>, b: &FxHashSet<Id>| a.intersection(b).next().copied();
    if let Some(id) = overlap(&classes, &individuals) {
        return Some(format!("id {} is used as both class and individual", id));
    }
    if let Some(id) = overlap(&classes, &properties) {
        return Some(format!("id {} is used as both class and property", id));
    }
    if let Some(id) = overlap(&properties, &individuals) {
        return Some(format!("id {} is used as both property and individual", id));
    }
    None
}

/// The RL divergence guard (module docs §"RL branch soundness"): the first implicated
/// construct the model touches, or `None` when "no clash" may soundly read as consistent.
fn rl_divergence_guard(onto: &Ontology) -> Option<String> {
    for axiom in onto.axioms() {
        if matches!(axiom, Axiom::DisjointClasses(_, _)) {
            return Some(
                "owl:disjointWith (implicated by DOCUMENTED_DIVERGENCES \
                 DisjointClasses-001/-003)"
                    .to_string(),
            );
        }
        if let Some(construct) = axiom_ces(axiom).into_iter().find_map(implicated_construct) {
            return Some(construct);
        }
    }
    None
}

/// The class expressions carried by an axiom (none for `SubObjectPropertyOf` /
/// `ObjectPropertyAssertion`).
fn axiom_ces(axiom: &Axiom) -> Vec<&ClassExpression> {
    match axiom {
        Axiom::SubClassOf { sub, sup } => vec![sub, sup],
        Axiom::EquivalentClasses(l, r) | Axiom::DisjointClasses(l, r) => vec![l, r],
        Axiom::ObjectPropertyDomain { domain, .. } => vec![domain],
        Axiom::ObjectPropertyRange { range, .. } => vec![range],
        Axiom::ClassAssertion { class, .. } => vec![class],
        Axiom::SubObjectPropertyOf { .. } | Axiom::ObjectPropertyAssertion { .. } => Vec::new(),
    }
}

/// First divergence-implicated construct inside a class expression, if any.
fn implicated_construct(ce: &ClassExpression) -> Option<String> {
    match ce {
        ClassExpression::Class(_) | ClassExpression::Thing | ClassExpression::Nothing => None,
        ClassExpression::ObjectComplementOf(_) => Some(
            "owl:complementOf (implicated by DOCUMENTED_DIVERGENCES DisjointClasses-001/-003, \
             New-Feature-ObjectQCR-002)"
                .to_string(),
        ),
        ClassExpression::ObjectUnionOf(_) => {
            Some("owl:unionOf (implicated by DOCUMENTED_DIVERGENCES WebOnt-I5.5-005)".to_string())
        }
        ClassExpression::ObjectIntersectionOf(members) => {
            members.iter().find_map(implicated_construct)
        }
        ClassExpression::ObjectSomeValuesFrom(_, filler)
        | ClassExpression::ObjectAllValuesFrom(_, filler) => implicated_construct(filler),
    }
}

// -------------------------------------------------------------------------------------------
// EL branch
// -------------------------------------------------------------------------------------------

/// The EL branch (module docs §"EL branch soundness").
fn el_branch(dict: &Dict, triples: &[[Id; 3]], onto: &Ontology) -> ConsistencyOutcome {
    let out = |verdict| ConsistencyOutcome {
        verdict,
        branch: Branch::ElClassification,
    };
    let report = Classifier::classify(dict, triples).report();
    if report.skipped_axioms > 0 {
        return out(ConsistencyVerdict::Unknown(UnknownReason::ElSkippedAxioms(
            report.skipped_axioms,
        )));
    }
    if let Some(kind) = el_unapplied_kind(onto) {
        return out(ConsistencyVerdict::Unknown(
            UnknownReason::ElUnappliedAxioms(kind),
        ));
    }
    if onto
        .axioms()
        .iter()
        .any(|axiom| axiom_ces(axiom).into_iter().any(ce_mentions_thing))
    {
        return out(ConsistencyVerdict::Unknown(UnknownReason::ElTopGuard));
    }
    // Pure ⊤-free EL+⊥ TBox: consistent by the empty-interpretation model construction
    // (module docs). The classifier run above contributed the skipped-axioms honesty gate.
    out(ConsistencyVerdict::Consistent)
}

/// The first axiom kind the EL classifier neither applies nor counts, if any.
fn el_unapplied_kind(onto: &Ontology) -> Option<String> {
    onto.axioms().iter().find_map(|axiom| match axiom {
        Axiom::SubClassOf { .. }
        | Axiom::EquivalentClasses(_, _)
        | Axiom::DisjointClasses(_, _) => None,
        Axiom::SubObjectPropertyOf { .. } => Some("SubObjectPropertyOf".to_string()),
        Axiom::ObjectPropertyDomain { .. } => Some("ObjectPropertyDomain".to_string()),
        Axiom::ObjectPropertyRange { .. } => Some("ObjectPropertyRange".to_string()),
        Axiom::ClassAssertion { .. } => Some("ClassAssertion".to_string()),
        Axiom::ObjectPropertyAssertion { .. } => Some("ObjectPropertyAssertion".to_string()),
    })
}

/// Whether a class expression mentions `owl:Thing` anywhere.
fn ce_mentions_thing(ce: &ClassExpression) -> bool {
    match ce {
        ClassExpression::Thing => true,
        ClassExpression::Class(_) | ClassExpression::Nothing => false,
        ClassExpression::ObjectIntersectionOf(members)
        | ClassExpression::ObjectUnionOf(members) => members.iter().any(ce_mentions_thing),
        ClassExpression::ObjectComplementOf(inner) => ce_mentions_thing(inner),
        ClassExpression::ObjectSomeValuesFrom(_, filler)
        | ClassExpression::ObjectAllValuesFrom(_, filler) => ce_mentions_thing(filler),
    }
}

// -------------------------------------------------------------------------------------------
// ALCH branch
// -------------------------------------------------------------------------------------------

/// Map a tableau verdict into the dispatch verdict (the ALCH branch is complete for the
/// whole L1 fragment, so both definitive verdicts pass through).
fn tableau_consistency(onto: &Ontology, budget: Budget) -> ConsistencyVerdict {
    match tableau::consistency(onto, budget) {
        tableau::Verdict::Satisfiable => ConsistencyVerdict::Consistent,
        tableau::Verdict::Unsatisfiable => ConsistencyVerdict::Inconsistent,
        tableau::Verdict::Unknown(tableau::UnknownReason::ResourceBudget(b)) => {
            ConsistencyVerdict::Unknown(UnknownReason::ResourceBudget(b))
        }
        tableau::Verdict::Unknown(tableau::UnknownReason::OutOfFragment(m)) => {
            // Unreachable in practice: the ontology was already extracted. Fail-closed.
            ConsistencyVerdict::Unknown(UnknownReason::OutOfFragment(m))
        }
    }
}

// -------------------------------------------------------------------------------------------
// Refutation encodings
// -------------------------------------------------------------------------------------------

/// Fresh-name minting for the refutation encodings: `urn:` IRIs interned into the dict and
/// checked against every id occurring in the premise/conclusion models, so the freshness
/// premise of the module-doc arguments holds by construction.
struct FreshNames<'d> {
    dict: &'d mut Dict,
    used: FxHashSet<Id>,
    counter: usize,
}

impl<'d> FreshNames<'d> {
    fn new(dict: &'d mut Dict, ontologies: [&Ontology; 2]) -> FreshNames<'d> {
        let mut used = FxHashSet::default();
        for onto in ontologies {
            collect_ids(onto, &mut used);
        }
        FreshNames {
            dict,
            used,
            counter: 0,
        }
    }

    /// Mint an id occurring in neither input model.
    fn mint(&mut self) -> Id {
        loop {
            let iri = format!("urn:sparq-reason-dl:check:fresh:{}", self.counter);
            self.counter += 1;
            let id = self.dict.intern_iri(&iri);
            if self.used.insert(id) {
                return id;
            }
        }
    }
}

/// Every id (class, property, individual) occurring in the model.
fn collect_ids(onto: &Ontology, into: &mut FxHashSet<Id>) {
    let mut ces: Vec<&ClassExpression> = Vec::new();
    for axiom in onto.axioms() {
        ces.extend(axiom_ces(axiom));
        match axiom {
            Axiom::SubObjectPropertyOf { sub, sup } => {
                into.extend([sub.named(), sup.named()]);
            }
            Axiom::ObjectPropertyDomain { property, .. }
            | Axiom::ObjectPropertyRange { property, .. } => {
                into.insert(property.named());
            }
            Axiom::ClassAssertion { individual, .. } => {
                into.insert(*individual);
            }
            Axiom::ObjectPropertyAssertion {
                property,
                source,
                target,
            } => {
                into.extend([property.named(), *source, *target]);
            }
            Axiom::SubClassOf { .. }
            | Axiom::EquivalentClasses(_, _)
            | Axiom::DisjointClasses(_, _) => {}
        }
    }
    while let Some(ce) = ces.pop() {
        match ce {
            ClassExpression::Class(id) => {
                into.insert(*id);
            }
            ClassExpression::Thing | ClassExpression::Nothing => {}
            ClassExpression::ObjectIntersectionOf(members)
            | ClassExpression::ObjectUnionOf(members) => ces.extend(members),
            ClassExpression::ObjectComplementOf(inner) => ces.push(inner),
            ClassExpression::ObjectSomeValuesFrom(p, filler)
            | ClassExpression::ObjectAllValuesFrom(p, filler) => {
                into.insert(p.named());
                ces.push(filler);
            }
        }
    }
}

/// The refutation check(s) for one conclusion axiom (module docs): each inner `Vec<Axiom>`
/// is one set of additions whose UNSATISFIABILITY (together with the premise) certifies
/// one component of the axiom; the axiom is entailed iff EVERY component's union is
/// unsatisfiable. `None` = no argued encoding for this axiom kind.
fn refutation_checks(conclusion: &Axiom, fresh: &mut FreshNames) -> Option<Vec<Vec<Axiom>>> {
    let gci = |sub: &ClassExpression, sup: &ClassExpression, fresh: &mut FreshNames| {
        vec![Axiom::ClassAssertion {
            class: ClassExpression::ObjectIntersectionOf(vec![
                sub.clone(),
                ClassExpression::ObjectComplementOf(Box::new(sup.clone())),
            ]),
            individual: fresh.mint(),
        }]
    };
    match conclusion {
        Axiom::SubClassOf { sub, sup } => Some(vec![gci(sub, sup, fresh)]),
        Axiom::EquivalentClasses(l, r) => Some(vec![gci(l, r, fresh), gci(r, l, fresh)]),
        // DisjointClasses(C, D) ≡ C ⊓ D ⊑ ⊥ (record §3): refute with a fresh instance of
        // the intersection.
        Axiom::DisjointClasses(l, r) => Some(vec![vec![Axiom::ClassAssertion {
            class: ClassExpression::ObjectIntersectionOf(vec![l.clone(), r.clone()]),
            individual: fresh.mint(),
        }]]),
        // Domain/Range desugar to GCIs (record §3): ∃R.⊤ ⊑ C and ⊤ ⊑ ∀R.C.
        Axiom::ObjectPropertyDomain { property, domain } => {
            let some_top = ClassExpression::some(property.named(), ClassExpression::Thing);
            Some(vec![gci(&some_top, domain, fresh)])
        }
        Axiom::ObjectPropertyRange { property, range } => {
            let all_range = ClassExpression::only(property.named(), range.clone());
            Some(vec![gci(&ClassExpression::Thing, &all_range, fresh)])
        }
        Axiom::ClassAssertion { class, individual } => Some(vec![vec![Axiom::ClassAssertion {
            class: ClassExpression::ObjectComplementOf(Box::new(class.clone())),
            individual: *individual,
        }]]),
        // The fresh-class trick (module docs; sound AND complete).
        Axiom::ObjectPropertyAssertion {
            property,
            source,
            target,
        } => {
            let b = fresh.mint();
            Some(vec![vec![
                Axiom::ClassAssertion {
                    class: ClassExpression::Class(b),
                    individual: *target,
                },
                Axiom::ClassAssertion {
                    class: ClassExpression::only(
                        property.named(),
                        ClassExpression::ObjectComplementOf(Box::new(ClassExpression::Class(b))),
                    ),
                    individual: *source,
                },
            ]])
        }
        // The fresh-individual-pair role-subsumption trick (module docs; record §4 as
        // amended by sq-pbz04.4.9 — sound AND complete). [FABLE-5]
        Axiom::SubObjectPropertyOf { sub, sup } => {
            let a = fresh.mint();
            let b = fresh.mint();
            let witness = fresh.mint();
            Some(vec![vec![
                Axiom::ObjectPropertyAssertion {
                    property: sub.clone(),
                    source: a,
                    target: b,
                },
                Axiom::ClassAssertion {
                    class: ClassExpression::Class(witness),
                    individual: b,
                },
                Axiom::ClassAssertion {
                    class: ClassExpression::only(
                        sup.named(),
                        ClassExpression::ObjectComplementOf(Box::new(ClassExpression::Class(
                            witness,
                        ))),
                    ),
                    individual: a,
                },
            ]])
        }
    }
}

// -------------------------------------------------------------------------------------------
// Conclusion anonymous-individual rolling ([OPUS-4.8] sq-pbz04.4.13)
// -------------------------------------------------------------------------------------------
//
// Official OWL 2 Direct-Semantics entailment reads a blank-node individual in the CONCLUSION
// ontology EXISTENTIALLY (an "existential variable" — the somevaluesfrom2bnode test's own
// description). L1 maps every blank node to an ordinary dict id, i.e. a (skolem) CONSTANT. On
// the PREMISE side skolemisation is entailment-preserving and stays; on the CONCLUSION side it
// is NOT — a `NotEntailed` certificate from the skolem reading is UNSOUND, because the premise
// may entail the conclusion existentially (a p _:x) without entailing it for any specific
// constant. So a conclusion blank-node assertion set must be resolved BEFORE the refutation
// loop: rolled up into an existential class assertion when it is tree-shaped, or refused
// fail-closed otherwise (never a skolem `NotEntailed`).
//
// ROLLING-UP is the standard tree-shaped-query technique: an anonymous individual reached by a
// single property edge `s p _:x`, whose asserted types are `C₁ … Cₙ` and whose own outgoing
// edges are `_:x qⱼ _:yⱼ`, denotes exactly `s : ∃p.(C₁ ⊓ … ⊓ Cₙ ⊓ ∃q₁.⟨roll _:y₁⟩ ⊓ …)`. This
// is SOUND and COMPLETE for the tree shape (no inverse roles, no shared variables, no nominals
// — all guaranteed by the rollability checks + the ALCH fragment), so a `NotEntailed` from a
// ROLLED existential class assertion is decided by the complete tableau and is genuinely
// sound, and the graduated cases (somevaluesfrom2bnode / WebOnt-someValuesFrom-003) become
// real passes rather than abstentions.

/// Whether `id` is an anonymous (blank-node) individual in `dict`.
fn is_anonymous_individual(dict: &Dict, id: Id) -> bool {
    !is_inline(id) && matches!(dict.term_parts(id), TermParts::Blank(_))
}

/// Render an id for a fail-closed diagnostic (positional-arg friendly — CodeQL note).
fn term_of(dict: &Dict, id: Id) -> String {
    if is_inline(id) {
        return format!("(inline literal id {})", id);
    }
    match dict.term_parts(id) {
        TermParts::Iri { prefix, suffix } => format!("<{}{}>", prefix, suffix),
        TermParts::Lit { value, .. } => format!("\"{}\"", value),
        TermParts::Blank(label) => format!("_:{}", label),
        TermParts::Triple(_) => "<<triple term>>".to_string(),
    }
}

/// Resolve the CONCLUSION ontology's anonymous (blank-node) individuals under the official
/// EXISTENTIAL reading (module note above; sq-pbz04.4.13).
///
/// Returns the blank-node-FREE conclusion axiom list the caller decides by refutation:
/// - if the conclusion mentions no blank-node individual, its axioms unchanged (clone);
/// - otherwise, the blank-node ABox assertions ROLLED UP into existential class assertions on
///   their named roots, with every blank-node-free conclusion axiom kept verbatim.
///
/// # Errors
/// [`UnknownReason::ConclusionAnonymousIndividual`] when a blank-node individual is NOT in a
/// tree-shaped roll-up: it must be the object of EXACTLY ONE property assertion (its unique
/// parent edge), reach only blank-node successors (a named/literal successor would need a
/// nominal, out of the ALCH fragment), and be acyclic / anchored to a named root. Fail-closed
/// — the caller abstains, never a skolem-constant `NotEntailed`.
fn roll_conclusion_anonymous(dict: &Dict, concl: &Ontology) -> Result<Vec<Axiom>, UnknownReason> {
    // Fast path: no anonymous individual anywhere → decide the conclusion verbatim, so the
    // blank-node-free path (every existing entailment case) is byte-identical to before.
    let mentions_bnode = concl.axioms().iter().any(|axiom| match axiom {
        Axiom::ClassAssertion { individual, .. } => is_anonymous_individual(dict, *individual),
        Axiom::ObjectPropertyAssertion { source, target, .. } => {
            is_anonymous_individual(dict, *source) || is_anonymous_individual(dict, *target)
        }
        _ => false,
    });
    if !mentions_bnode {
        return Ok(concl.axioms().to_vec());
    }

    // Build the roll-up structure over the ABox. `in_degree` counts parent property edges per
    // blank individual; `types` collects each blank individual's asserted class types;
    // `outgoing` its child property edges; `top_edges` the named→blank edges that ANCHOR a
    // tree; `clean` keeps every blank-node-free axiom verbatim.
    let mut in_degree: FxHashMap<Id, usize> = FxHashMap::default();
    let mut types: FxHashMap<Id, Vec<ClassExpression>> = FxHashMap::default();
    let mut outgoing: FxHashMap<Id, Vec<(Id, Id)>> = FxHashMap::default();
    let mut top_edges: Vec<(Id, Id, Id)> = Vec::new();
    let mut clean: Vec<Axiom> = Vec::new();

    for axiom in concl.axioms() {
        match axiom {
            Axiom::ClassAssertion { class, individual } => {
                if is_anonymous_individual(dict, *individual) {
                    in_degree.entry(*individual).or_insert(0);
                    types.entry(*individual).or_default().push(class.clone());
                } else {
                    clean.push(axiom.clone());
                }
            }
            Axiom::ObjectPropertyAssertion {
                property,
                source,
                target,
            } => {
                let s_anon = is_anonymous_individual(dict, *source);
                let t_anon = is_anonymous_individual(dict, *target);
                match (s_anon, t_anon) {
                    (false, false) => clean.push(axiom.clone()),
                    (false, true) => {
                        // named --p--> blank : a top edge anchoring a tree.
                        *in_degree.entry(*target).or_insert(0) += 1;
                        top_edges.push((property.named(), *source, *target));
                    }
                    (true, true) => {
                        // blank --p--> blank : an internal tree edge.
                        in_degree.entry(*source).or_insert(0);
                        *in_degree.entry(*target).or_insert(0) += 1;
                        outgoing
                            .entry(*source)
                            .or_default()
                            .push((property.named(), *target));
                    }
                    (true, false) => {
                        // blank --p--> named : the anonymous individual has a NAMED successor,
                        // rollable only through a nominal {named} — out of ALCH. Recorded so
                        // `roll_bnode` refuses at the exact offending node.
                        in_degree.entry(*source).or_insert(0);
                        outgoing
                            .entry(*source)
                            .or_default()
                            .push((property.named(), *target));
                    }
                }
            }
            other => clean.push(other.clone()),
        }
    }

    // Every blank-node individual must have EXACTLY ONE parent edge: in-degree 0 is an
    // unanchored free existential root, in-degree ≥ 2 is a node SHARED between two assertions.
    // Both are outside the tree-shaped roll-up (and exactly the negative-probe shapes).
    for (&b, &deg) in &in_degree {
        if deg != 1 {
            return Err(UnknownReason::ConclusionAnonymousIndividual(format!(
                "anonymous individual {} has {} parent property-assertion(s); a tree-shaped \
                 roll-up needs exactly one (shared / unanchored anonymous individual)",
                term_of(dict, b),
                deg
            )));
        }
    }

    // Fold each named-anchored subtree into an existential class assertion on its named root.
    // `consumed` records every blank node folded into a tree; a blank node reached from no
    // named root (a pure cycle / an anchorless component) stays unconsumed and is refused.
    let mut rolled: Vec<Axiom> = clean;
    let mut consumed: FxHashSet<Id> = FxHashSet::default();
    for &(property, source, target) in &top_edges {
        let mut visiting: FxHashSet<Id> = FxHashSet::default();
        let filler = roll_bnode(
            dict,
            target,
            &types,
            &outgoing,
            &mut visiting,
            &mut consumed,
        )?;
        rolled.push(Axiom::ClassAssertion {
            class: ClassExpression::ObjectSomeValuesFrom(
                ObjectPropertyExpression::ObjectProperty(property),
                Box::new(filler),
            ),
            individual: source,
        });
    }

    // Any blank-node individual not folded into a named-rooted tree is a cyclic / anchorless
    // shape (each has in-degree 1 from the check above, so a component with no named anchor is
    // necessarily cyclic) — refuse fail-closed.
    for &b in in_degree.keys() {
        if !consumed.contains(&b) {
            return Err(UnknownReason::ConclusionAnonymousIndividual(format!(
                "anonymous individual {} is not reachable from a named root (cyclic or \
                 anchorless anonymous shape)",
                term_of(dict, b)
            )));
        }
    }
    Ok(rolled)
}

/// Roll one blank-node subtree (rooted at `b`) into the class expression whose existential the
/// parent edge asserts: the conjunction of `b`'s asserted class types and `∃p.⟨child subtree⟩`
/// per outgoing edge. An empty conjunction is `owl:Thing`. Cycle- and nominal-guarded.
fn roll_bnode(
    dict: &Dict,
    b: Id,
    types: &FxHashMap<Id, Vec<ClassExpression>>,
    outgoing: &FxHashMap<Id, Vec<(Id, Id)>>,
    visiting: &mut FxHashSet<Id>,
    consumed: &mut FxHashSet<Id>,
) -> Result<ClassExpression, UnknownReason> {
    if !visiting.insert(b) {
        return Err(UnknownReason::ConclusionAnonymousIndividual(format!(
            "anonymous individual {} lies on a cycle (no tree-shaped roll-up)",
            term_of(dict, b)
        )));
    }
    // A node reachable through two named-anchored paths would already have in-degree ≥ 2 and be
    // refused before we get here; `consumed` insertion is the fold record.
    consumed.insert(b);
    let mut members: Vec<ClassExpression> = types.get(&b).cloned().unwrap_or_default();
    if let Some(edges) = outgoing.get(&b) {
        for &(property, target) in edges {
            if !is_anonymous_individual(dict, target) {
                // A named/literal successor of an anonymous individual needs a nominal {t}.
                return Err(UnknownReason::ConclusionAnonymousIndividual(format!(
                    "anonymous individual {} has a non-anonymous successor {} (a nominal \
                     roll-up is out of the ALCH fragment)",
                    term_of(dict, b),
                    term_of(dict, target)
                )));
            }
            let filler = roll_bnode(dict, target, types, outgoing, visiting, consumed)?;
            members.push(ClassExpression::ObjectSomeValuesFrom(
                ObjectPropertyExpression::ObjectProperty(property),
                Box::new(filler),
            ));
        }
    }
    visiting.remove(&b);
    Ok(match members.len() {
        0 => ClassExpression::Thing,
        1 => members.pop().expect("len checked == 1"),
        _ => ClassExpression::ObjectIntersectionOf(members),
    })
}
