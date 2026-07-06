//! [FABLE-5] sq-pbz04.4.5 (epic sq-pbz04.4) — the OWL 2 **Direct-Semantics arm** ratchet:
//! the `inference::dl_suite` runner over the DIRECT-sanctioned arm of the OWL WG
//! test-repository export (`tests/w3c/owl2/all.rdf`), with PINNED sparq-extension floors.
//! 🤖 SPARQ agent.
//!
//! ## What is pinned (and why EXACT equality, not a `>=` ratchet)
//!
//! Two floors, both MEASUREMENTS of what the layered `sparq-reason-dl` checker genuinely
//! computes at the pinned export snapshot — HONESTLY sparq-EXTENSION rows over the
//! **scoped fragment — NOT full OWL 2 DL**, never folded into standards-conformance
//! totals (design record `research/owl2-direct-semantics-scoping.md` §4):
//!
//! - [`gated::DL_PROFILE_FLOOR`] — profile-identification lane passes: the L2 syntactic
//!   checker vs the export's POSITIVE `test:profile` tags ONLY (the design record's
//!   positive-only fallback — see the runner module docs for the measurement that
//!   forced it: L2's `In` is fragment-grammar membership and cannot refute full-profile
//!   membership, so the explicit-negative direction is not checked).
//! - [`gated::DL_DIRECT_FLOOR`] — Direct consistency / inconsistency / positive- /
//!   negative-entailment passes through the L4 `DirectChecker` dispatch under the PINNED
//!   deterministic count budget (`dl_suite::pinned_budget`; wall-clock budgets banned).
//!
//! Both are asserted with EXACT equality (the `ql_entailment_floor` precedent, not the
//! `>=` el-suite shape) — deliberately: the bead's INVARIANT is *an abstention is NEVER
//! counted as a pass*, and a `>=` floor cannot catch the inflation direction (a mutation
//! that counts abstentions as passes only RAISES the count). With `==`, a regression AND
//! an unexplained rise both go RED; a genuine improvement re-pins the consts in a
//! reviewed PR. The abstained totals are pinned the same way, so every selected row
//! stays accounted for in exactly one bucket.
//!
//! ## Divergences: pinned per-case, audited per-mechanism, NEVER passes
//!
//! A `Fail` row means the checker produced a DEFINITIVE verdict that contradicts the
//! export's expectation. Every such row is pinned BY NAME in
//! [`gated::DOCUMENTED_DIVERGENCES`] with an audited mechanism (exact SET equality: an
//! unpinned new fail AND a stale entry that stops failing both go RED). The mechanisms
//! are NOT laundered as acceptable: M4 was a genuine fidelity gap in the merged L1
//! extractor that this conformance arm DISCOVERED; it was FIXED by sq-pbz04.4.12 (see
//! M4 below) and the 4 entries removed from the divergence pin as promised. M1 was
//! FIXED by sq-pbz04.4.11. The mechanisms:
//!
//! - **M1 — FIXED (sq-pbz04.4.11).** Named-composite inlining previously lost the
//!   name↔expression binding; `extract()` now emits `EquivalentClasses(A, expr)` for
//!   every NAMED class carrying an inline backbone definition, and the 12 corpus cases
//!   that required this binding now PASS.
//! - **M2 — anonymous individuals in a conclusion read as constants.** The official
//!   expectation reads conclusion bnodes EXISTENTIALLY; the L1 model treats them as
//!   (skolem) constants, so the refutation is satisfiable and the checker certifies a
//!   `NotEntailed` that is wrong under the existential reading.
//! - **M3 — reserved vocabulary treated as plain names.** OWL-Full-shaped axioms over
//!   reserved IRIs (`rdfs:Class ≡ owl:Class`, `owl:imports` domain/range, `rdf:type`
//!   domain) extract as ordinary names; the legacy dual-tagged expectation assumes the
//!   reserved semantics.
//! - **M4 — FIXED (sq-pbz04.4.12).** Orphan/cyclic `rdf:first`/`rdf:rest` cells (including
//!   `rdf:nil rdf:rest _:b`, cyclic lists) and unconsumed anonymous class-expression
//!   backbones now correctly refuse extraction (`MalformedList` /
//!   `MalformedClassExpression`), yielding an `OutOfFragment` abstention instead of a wrong
//!   `Consistent` verdict (I5.5-003/004) or a trivially-entailed empty non-conclusion
//!   (I5.5-006/007). The load-bearing part of the fix (the escalated-review REFUTATION of
//!   the first attempt): the orphan-reachability seed must NOT count an ignorable declaration
//!   typing (`_:list a rdf:List`, `_:x a owl:Class`) as a consumer — that bypass wrongly
//!   rescued the very cyclic/orphan cells the check exists to catch. Beyond I5.5-006/-007,
//!   closing it also correctly refuses three graphs that carry an anonymous composite
//!   appearing in NO axiom and had been passing only by that accident (I5.26-001, I5.26-006
//!   consistency; I5.5-005 positive-entailment) — all honest fail-closed abstentions, never
//!   wrong verdicts. [OPUS-4.8]
//! - **M5 — ontology-header stripping vs a header-sensitive legacy expectation.** The
//!   harness strips `owl:Ontology` typings on both sides (the RL/EL-lane convention);
//!   WebOnt-Ontology-003's OWL-1-era non-entailment hinges on the stripped header.
//! - **M6 — the premise/input literal does not parse with oxrdfxml** (a nested
//!   `rdf:RDF` property element the strict parser rejects).
//! - **M7 — L2 profile-grammar arity vs the official singleton-list normalization.**
//!   L2 requires `ObjectIntersectionOf` arity ≥ 2 (the structural-spec arity); the
//!   official tagging accepts the RDF singleton-intersection encoding (normalizing it
//!   to its sole member), so positively-tagged cases carrying `owl:intersectionOf (x)`
//!   come back `NotIn`. (WebOnt-I5.26-001 left this pin under sq-pbz04.4.12: its input's
//!   anonymous singleton intersection appears in no axiom, so the M4 fix now ABSTAINS on it
//!   rather than reaching the arity rule.)
//!
//! ## Feature gating (both states)
//!
//! Behind the opt-in `dl-direct` feature (forwards to `sparq-reason-dl/dispatch`). OFF:
//! this file compiles to a single self-SKIP `#[test]` — no Direct-Semantics code links.
//! The export is fetched by `scripts/fetch-inference-suites.sh` into the gitignored
//! `tests/w3c/owl2/`; when absent the runner SKIPS so a fresh offline checkout stays
//! green. The floor consts are read TEXTUALLY by `tests/scoreboard_floors.rs`, so the
//! central scoreboard's mirrored values can never silently drift.

// [FABLE-5] cfg gate, not a runtime branch — zero Direct-Semantics code compiles in the
// default state (the lean opt-in posture).
#[cfg(not(feature = "dl-direct"))]
#[test]
fn dl_suite_skipped_without_feature() {
    eprintln!(
        "SKIP: the OWL 2 Direct-Semantics arm is OFF — build with `--features dl-direct` \
         (and run scripts/fetch-inference-suites.sh) to run it."
    );
}

#[cfg(feature = "dl-direct")]
mod gated {
    use sparq_conformance::inference::dl_suite::{self, TriState};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    /// PROFILE-IDENTIFICATION lane pass count (positive `test:profile` tags only) — the
    /// MEASURED value at the pinned export snapshot, EXACT-pinned (module docs).
    /// Mirrored in `scoreboard::SUITES` and read TEXTUALLY by
    /// `tests/scoreboard_floors.rs`. A sparq EXTENSION measurement over the L1/L2
    /// fragment, NOT a W3C ProfileIdentificationTest conformance claim.
    /// [FABLE-5] sq-pbz04.4.5
    pub const DL_PROFILE_FLOOR: usize = 68;

    /// DIRECT consistency/entailment lane pass count through the L4 `DirectChecker` —
    /// the MEASURED value at the pinned export snapshot, EXACT-pinned (module docs).
    /// Mirrored in `scoreboard::SUITES` and read TEXTUALLY by
    /// `tests/scoreboard_floors.rs`. A sparq EXTENSION measurement over the scoped
    /// fragment — NOT full OWL 2 DL.
    ///
    /// COMPOSITION: 105 consistency + 14 inconsistency + 68 positive-entailment +
    /// 2 negative-entailment = 189. [FABLE-5] sq-pbz04.4.5 / [OPUS-4.8] sq-pbz04.4.12
    /// Re-pinned by sq-pbz04.4.11 (M1 named-composite fix): 12 positive-entailment cases
    /// now PASS (the EquivalentClasses name-binding axioms enable their entailments);
    /// 4 consistency cases shift from Pass to abstain because their updated ontology
    /// models now include EquivalentClasses axioms that, combined with the existing TBox,
    /// cause budget exhaustion — an honest abstention, not a regression. Net: +8.
    /// Re-pinned by sq-pbz04.4.12 (M4 orphan/cyclic fix): −3 (192 → 189). Beyond the 2 M4
    /// negative-entailment rows (I5.5-006/-007) moving from Fail → abstain, closing the
    /// declaration-typing reachability bypass also correctly REFUSES three graphs that
    /// previously extracted (and happened to pass) despite carrying an anonymous composite
    /// class-expression that appears in NO axiom — a genuinely unconsumed backbone the
    /// `a owl:Class` / `a rdf:List` declaration typing was wrongly rescuing:
    ///   - `WebOnt-I5.26-001` (consistency): `[ owl:intersectionOf (:C _:B) ]` in no axiom;
    ///   - `WebOnt-I5.26-006` (consistency): a nested intersection/union backbone in no axiom;
    ///   - `WebOnt-I5.5-005` (positive-entailment): the conclusion is a bare
    ///     `[ owl:unionOf (:a) ]` that "does not appear in an axiom" (its own description).
    ///
    /// These are HONEST fail-closed abstentions (the checker refuses to give a definitive
    /// verdict over a graph it cannot map in full), not wrong verdicts — exactly the M4
    /// contract. [OPUS-4.8] sq-pbz04.4.12
    pub const DL_DIRECT_FLOOR: usize = 189;

    /// Abstained (fail-closed OutOfFragment / guard / deferred / budget) row totals,
    /// EXACT-pinned so the tri-state accounting is closed: profile lane, then the four
    /// reasoning lanes summed. [FABLE-5] sq-pbz04.4.5
    /// Re-pinned by sq-pbz04.4.12 (M4 orphan/cyclic fix): +3 (114 → 117). Closing the
    /// declaration-typing reachability bypass makes the profile lane's extraction refuse the
    /// three `WebOnt-I5.26-001` profile rows (EL/QL/RL): the input carries an anonymous
    /// `owl:intersectionOf` that appears in no axiom, so extraction now fails → the profile
    /// set is `Unknown` → abstain (previously it extracted and answered a wrong `NotIn`, the
    /// M7 singleton-intersection divergence — now superseded by the honest abstention).
    /// [OPUS-4.8] sq-pbz04.4.12
    pub const DL_PROFILE_ABSTAINED: usize = 117;
    /// See [`DL_PROFILE_ABSTAINED`].
    /// Re-pinned by sq-pbz04.4.11: +4 from the 4 consistency cases that shifted from
    /// Pass to OutOfFragment (budget exhaustion on the now-more-complete model).
    /// Re-pinned by sq-pbz04.4.12 (M4 orphan/cyclic fix): MEASURED 463 against the pinned
    /// export. Closing the declaration-typing reachability bypass moves the 2 M4
    /// negative-entailment rows (I5.5-006/-007) from Fail (wrong definitive verdict) to
    /// OutOfFragment (fail-closed refusal), and additionally REFUSES 3 previously-passing
    /// rows that carried an unconsumed anonymous composite the declaration typing had been
    /// rescuing (I5.26-001, I5.26-006 consistency; I5.5-005 positive-entailment) — all
    /// honest abstentions, never wrong verdicts. [OPUS-4.8] sq-pbz04.4.12
    pub const DL_DIRECT_ABSTAINED: usize = 463;

    /// Audited, PINNED divergence rows (module docs — mechanisms M2–M7): every row
    /// where a checker verdict contradicts the export expectation, keyed by the
    /// runner's row key. EXACT set equality is asserted: an UNPINNED new fail and a
    /// STALE entry that stops failing both turn the lane RED (the el-suite /
    /// ql_entailment_floor discipline). M4 was FIXED by sq-pbz04.4.12 and removed from
    /// this list — those 4 cases now ABSTAIN (OutOfFragment refusal) rather than producing
    /// wrong definitive verdicts. M1 (named-composite name-binding) was FIXED by
    /// sq-pbz04.4.11. [FABLE-5] sq-pbz04.4.5 / [OPUS-4.8] sq-pbz04.4.12
    const DOCUMENTED_DIVERGENCES: &[(&str, &str)] = &[
        // --- M2: anonymous individuals in the conclusion read as constants (L4 gap) ---
        (
            "owl2-dl/positive-entailment: somevaluesfrom2bnode",
            "M2 — conclusion `a p _:x` is the existential the premise's ∃p.⊤ typing \
             grants; as a constant the refutation stays satisfiable",
        ),
        (
            "owl2-dl/positive-entailment: WebOnt-someValuesFrom-003",
            "M2 — conclusion is an anonymous parent-chain (person ≡ ∃parent.person); \
             bnode individuals as constants block the existential reading",
        ),
        // --- M3: reserved vocabulary treated as plain names ---
        (
            "owl2-dl/positive-entailment: WebOnt-Class-001",
            "M3 — conclusion `rdfs:Class owl:equivalentClass owl:Class` is an \
             OWL-Full-shaped reserved-vocabulary axiom",
        ),
        (
            "owl2-dl/positive-entailment: WebOnt-imports-010",
            "M3 — conclusion gives owl:imports a domain/range (reserved vocabulary)",
        ),
        (
            "owl2-dl/positive-entailment: WebOnt-I5.3-014",
            "M3 — premise gives rdf:type an rdfs:domain (reserved vocabulary); the \
             expected subclass conclusion needs that OWL-Full reading",
        ),
        // M4 (orphan/cyclic list and unconsumed backbone) FIXED by sq-pbz04.4.12 —
        // WebOnt-I5.5-003/-004/-006/-007 now correctly ABSTAIN (OutOfFragment refusal)
        // instead of producing wrong Consistent / Entailed verdicts. [OPUS-4.8]
        //
        // --- M5: header-stripping vs a header-sensitive legacy expectation ---
        (
            "owl2-dl/negative-entailment: WebOnt-Ontology-003",
            "M5 — with the owl:Ontology headers stripped (the harness convention shared \
             with the RL/EL lanes), Car ≡ Automobile makes the swapped typings genuinely \
             entailed; the OWL-1-era non-entailment hinges on the header triple",
        ),
        // --- M6: the inline ontology literal does not parse with oxrdfxml ---
        (
            "owl2-dl/consistency: FS2RDF-literals-ar",
            "M6 — the premise literal embeds a nested rdf:RDF property element oxrdfxml \
             rejects (strict RDF/XML)",
        ),
    ];

    /// PINNED profile-lane divergence rows (positive tags where L2 answers `NotIn`) —
    /// same exact-set discipline as [`DOCUMENTED_DIVERGENCES`]. All 30 are ONE audited
    /// mechanism (M7 in the module docs): the OWL-1-era singleton
    /// `owl:intersectionOf ( :A )` encoding — `_:B owl:intersectionOf` with a
    /// one-member list — which the export tags in-EL/QL/RL but L2's structural-spec
    /// arity rule (≥ 2 members) rejects. [FABLE-5] sq-pbz04.4.5
    const PROFILE_DIVERGENCES: &[(&str, &str)] = &[
        // WebOnt-I5.26-001 (EL/QL/RL) REMOVED by sq-pbz04.4.12: its input carries an anonymous
        // `owl:intersectionOf` that appears in no axiom, so the M4 orphan fix now makes
        // extraction refuse it → the profile set is `Unknown` → the row ABSTAINS rather than
        // producing the wrong `NotIn` (the M7 singleton divergence no longer applies). [OPUS-4.8]
        ("owl2-dl/profile-EL: WebOnt-I5.26-002", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-I5.26-002", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-I5.26-002", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-I5.26-005", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-I5.26-005", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-I5.26-005", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-003", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-003", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-003", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-004", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-004", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-004", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-005", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-005", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-005", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-006", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-006", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-006", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-007", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-007", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-007", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-008", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-008", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-008", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-EL: WebOnt-disjointWith-009", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-QL: WebOnt-disjointWith-009", "M7 — singleton owl:intersectionOf operand list"),
        ("owl2-dl/profile-RL: WebOnt-disjointWith-009", "M7 — singleton owl:intersectionOf operand list"),
    ];

    /// Locate the OWL WG export the same way the inference binary + el-suite lane do.
    fn owl_export() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/owl2/all.rdf")
    }

    #[test]
    fn dl_direct_arm_ratchet() {
        let export = owl_export();
        if !export.exists() {
            eprintln!(
                "SKIP: OWL WG export not present at {} — run scripts/fetch-inference-suites.sh",
                export.display()
            );
            return;
        }
        let text = std::fs::read_to_string(&export)
            .unwrap_or_else(|e| panic!("read {}: {e}", export.display()));
        let report = dl_suite::run_direct_arm(&text).expect("all.rdf parses");

        // [FABLE-5] The grep-able ratchet lines — pass count in field $7. Positional
        // `println!` args (CodeQL rust/unused-variable guard).
        println!("{}", report.render());
        println!(
            "OWL 2 DL profile-identification ratchet pass {} of {} (floor {})",
            report.profile.pass,
            report.profile.total(),
            DL_PROFILE_FLOOR
        );
        let reasoning_total = report.consistency.total()
            + report.inconsistency.total()
            + report.positive_entailment.total()
            + report.negative_entailment.total();
        println!(
            "OWL 2 DL direct ratchet pass {} of {} (floor {})",
            report.reasoning_pass(),
            reasoning_total,
            DL_DIRECT_FLOOR
        );

        // Divergence discipline (module docs): every Fail row must be a PINNED, audited
        // entry, and every pinned entry must still fail — exact set equality, both
        // lanes. An unpinned fail is an unaudited regression; a stale pin means a fix
        // landed and the floors must be deliberately re-pinned.
        let reasoning_fails: BTreeSet<String> = [
            &report.consistency,
            &report.inconsistency,
            &report.positive_entailment,
            &report.negative_entailment,
        ]
        .iter()
        .flat_map(|lane| lane.fails.iter().map(|(k, _)| k.clone()))
        .collect();
        let pinned_reasoning: BTreeSet<String> = DOCUMENTED_DIVERGENCES
            .iter()
            .map(|(k, _)| (*k).to_string())
            .collect();
        assert_eq!(
            reasoning_fails, pinned_reasoning,
            "reasoning-lane divergence set moved — unpinned new fails: {:?}; stale pins \
             (now passing/abstaining — re-audit + re-pin): {:?}",
            reasoning_fails.difference(&pinned_reasoning).collect::<Vec<_>>(),
            pinned_reasoning.difference(&reasoning_fails).collect::<Vec<_>>()
        );
        let profile_fails: BTreeSet<String> =
            report.profile.fails.iter().map(|(k, _)| k.clone()).collect();
        let pinned_profile: BTreeSet<String> =
            PROFILE_DIVERGENCES.iter().map(|(k, _)| (*k).to_string()).collect();
        assert_eq!(
            profile_fails, pinned_profile,
            "profile-lane divergence set moved — unpinned new fails: {:?}; stale pins: {:?}",
            profile_fails.difference(&pinned_profile).collect::<Vec<_>>(),
            pinned_profile.difference(&profile_fails).collect::<Vec<_>>()
        );

        // The EXACT pins (module docs: `==`, not `>=`, so abstention-inflation and
        // regression BOTH go red; re-pin deliberately when the fragment grows).
        assert_eq!(
            report.profile.pass, DL_PROFILE_FLOOR,
            "profile-identification pass count moved (got {}, pinned {}) — re-pin \
             DL_PROFILE_FLOOR deliberately with evidence",
            report.profile.pass, DL_PROFILE_FLOOR
        );
        assert_eq!(
            report.reasoning_pass(),
            DL_DIRECT_FLOOR,
            "Direct-lane pass count moved (got {}, pinned {}) — re-pin DL_DIRECT_FLOOR \
             deliberately with evidence",
            report.reasoning_pass(),
            DL_DIRECT_FLOOR
        );
        assert_eq!(
            report.profile.out_of_fragment_total(),
            DL_PROFILE_ABSTAINED,
            "profile-lane abstention total moved — the tri-state accounting is pinned"
        );
        let reasoning_abstained = report.consistency.out_of_fragment_total()
            + report.inconsistency.out_of_fragment_total()
            + report.positive_entailment.out_of_fragment_total()
            + report.negative_entailment.out_of_fragment_total();
        assert_eq!(
            reasoning_abstained, DL_DIRECT_ABSTAINED,
            "Direct-lane abstention total moved — the tri-state accounting is pinned"
        );

        // Closed accounting: every selected row is in exactly one bucket.
        assert_eq!(
            report.profile.total(),
            report.profile.pass
                + report.profile.fails.len()
                + report.profile.out_of_fragment_total()
                + report.profile.out_of_scope_total()
        );
    }

    /// Lane-local derivation canary — NOT a W3C case, NOT counted in any floor. Proves
    /// the arm drives REAL Direct-Semantics reasoning end-to-end (mini manifest → RDF/XML
    /// premise/conclusion → L1 extraction → L4 dispatch → ALCH refutation): a neutered
    /// checker (one that abstains everywhere, or a runner that miscounts abstentions as
    /// passes) turns the exact expected counts RED.
    #[test]
    fn dl_entailment_derivation_canary() {
        let mini = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:test="http://www.w3.org/2007/OWL/testOntology#">
  <test:TestCase rdf:about="http://ex/canary-entails">
    <rdf:type rdf:resource="http://www.w3.org/2007/OWL/testOntology#PositiveEntailmentTest"/>
    <rdf:type rdf:resource="http://www.w3.org/2007/OWL/testOntology#NegativeEntailmentTest"/>
    <test:identifier rdf:datatype="http://www.w3.org/2001/XMLSchema#string">canary-entails</test:identifier>
    <test:status rdf:resource="http://www.w3.org/2007/OWL/testOntology#Approved"/>
    <test:semantics rdf:resource="http://www.w3.org/2007/OWL/testOntology#DIRECT"/>
    <test:rdfXmlPremiseOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;owl:Class rdf:about="A"&gt;&lt;rdfs:subClassOf&gt;&lt;owl:Class rdf:about="B"/&gt;&lt;/rdfs:subClassOf&gt;&lt;/owl:Class&gt;&lt;owl:Class rdf:about="B"&gt;&lt;rdfs:subClassOf&gt;&lt;owl:Class rdf:about="C"/&gt;&lt;/rdfs:subClassOf&gt;&lt;/owl:Class&gt;&lt;/rdf:RDF&gt;</test:rdfXmlPremiseOntology>
    <test:rdfXmlConclusionOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;owl:Class rdf:about="A"&gt;&lt;rdfs:subClassOf&gt;&lt;owl:Class rdf:about="C"/&gt;&lt;/rdfs:subClassOf&gt;&lt;/owl:Class&gt;&lt;/rdf:RDF&gt;</test:rdfXmlConclusionOntology>
    <test:rdfXmlNonConclusionOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;owl:Class rdf:about="C"&gt;&lt;rdfs:subClassOf&gt;&lt;owl:Class rdf:about="A"/&gt;&lt;/rdfs:subClassOf&gt;&lt;/owl:Class&gt;&lt;/rdf:RDF&gt;</test:rdfXmlNonConclusionOntology>
  </test:TestCase>
</rdf:RDF>"#;
        let report = dl_suite::run_direct_arm(mini).expect("canary export parses");
        // A ⊑ B ⊑ C: the conclusion A ⊑ C is ENTAILED (refutation A⊓¬C unsatisfiable on
        // the real ALCH tableau) and the non-conclusion C ⊑ A is definitively NOT
        // entailed — both DEFINITIVE verdicts, so both rows are passes and nothing
        // abstained. Exact counts: an abstain-everywhere checker OR an
        // abstention-counted-as-pass mutation breaks these.
        assert_eq!(report.positive_entailment.pass, 1);
        assert_eq!(report.negative_entailment.pass, 1);
        assert_eq!(
            report.positive_entailment.out_of_fragment_total()
                + report.negative_entailment.out_of_fragment_total(),
            0
        );
        assert!(report.all_fails().is_empty(), "{:?}", report.all_fails());
        // And the tri-state mapping itself: an abstention maps to OutOfFragment.
        assert!(matches!(
            dl_suite::positive_entailment_tri(
                &sparq_reason_dl::check::EntailmentVerdict::Unknown(
                    sparq_reason_dl::check::UnknownReason::QlConsistencyPending
                )
            ),
            TriState::OutOfFragment(_)
        ));
    }
}
