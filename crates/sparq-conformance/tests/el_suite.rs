//! [SONNET-4.6] sq-pbz04.2.4 (epic sq-pbz04) — the OWL 2 EL entailment-regime
//! EXPRESSIVITY ratchet, wired as a crate-local `cargo test` + a pinned pass-count
//! FLOOR that may only RISE (registered in the central `scoreboard::SUITES` and
//! guarded by `tests/scoreboard_floors.rs`), mirroring the sibling `d-entail` /
//! `rif-core` / `ql-experimental` lanes in this crate.
//!
//! ## What is gated
//!
//! The W3C OWL WG test-repository export (`tests/w3c/owl2/all.rdf`, the same pinned
//! snapshot the `owl_suite` RL lane reads), filtered to the tests applicable to an
//! OWL 2 EL consequence-based classifier: `test:profile test:EL` AND `test:semantics
//! test:RDF-BASED`, `test:Approved` status, an inline RDF/XML premise, and no
//! `owl:imports` (the harness does not dereference). Each selected case contributes
//! one row per declared check:
//!
//! - `ConsistencyTest` — classify; PASS iff NO named class is unsatisfiable.
//! - `InconsistencyTest` — classify; PASS iff SOME named class is unsatisfiable
//!   (the TBox-level clash the classifier can see — see the honest-scope note below).
//! - `PositiveEntailmentTest` — classify the premise (materialize the complete
//!   `rdfs:subClassOf` subsumption lattice IN PLACE via `classify_graph`), then the
//!   materialized closure must ENTAIL the conclusion ontology under the shared
//!   bnode-homomorphism entailment check (the same `entail::entails` the RL
//!   `owl_suite` uses).
//! - `NegativeEntailmentTest` — the non-conclusion must NOT be entailed by the closure.
//!
//! The export inlines every ontology as an RDF/XML literal; ontology-header triples
//! (`owl:Ontology` typing + `owl:imports`) are stripped from both sides, as the
//! official harness compares axioms, not headers.
//!
//! ## Why a sparq-EXTENSION row, NOT a standards-conformance number (the HONEST scope)
//!
//! This is HONESTLY tallied as a sparq EXTENSION ratchet, NOT folded into the
//! conformance total — exactly like the RIF-Core / RSP / BM25 / DL-Lite_R extension
//! rows. Even though OWL 2 EL is a real W3C profile, this lane compares each test's
//! expected outcome against what `sparq-reason-el`'s consequence-based classifier
//! (CR1–CR6, plus safe nominals; RBox / concrete domains are separately gated)
//! genuinely COMPUTES over the EL fragment it implements. It is NOT a full OWL 2 EL
//! conformance claim: the concrete-domain rules CR7–CR9 (faceted datatype
//! restrictions, `DataHasValue` over literals) are deferred, and the classifier is
//! TBox-only (it detects unsatisfiable CLASSES, not every ABox-driven ontology
//! inconsistency). Cases needing those are DOCUMENTED divergences, never faked as
//! passes and never summed into the floor.
//!
//! ## Tri-state accounting + fail-closed
//!
//! Every selected check lands in exactly one bucket: PASS (the classifier's answer
//! matches the expected outcome — the ONLY bucket the floor counts), DOCUMENTED
//! DIVERGENCE (an audited permanent EL-classifier limitation — CR7–CR9 concrete
//! domains, `DataHasValue`, or an ABox-only inconsistency the TBox classifier cannot
//! see), or DEFERRED / OutOfScope (not-Approved status, `owl:imports`, or a
//! premise/conclusion only available in functional syntax). The classifier is SOUND,
//! so a positive entailment it cannot derive is a completeness gap, not a wrong
//! answer; any UNDOCUMENTED fail fails this lane hard (no silent skips), so the
//! divergence list can only grow via a conscious reviewed audit.
//!
//! ## Feature gating (both states)
//!
//! The whole lane is behind this crate's opt-in `el-suite` feature (forwards to the
//! OPT-IN `sparq-reason-el` crate AND also enables `sparq-reason-el/rbox` +
//! `sparq-reason-el/cdomain` so the CI lane exercises the full shipped feature set —
//! sq-pbz04.2.9). With the feature OFF this file compiles to a single self-SKIP
//! `#[test]` (no EL classifier links — the lean opt-in posture, so the default +
//! `--workspace` byte/bundle ratchets are byte-for-byte unchanged).
//! The `all.rdf` export is fetched by `scripts/fetch-inference-suites.sh` into the
//! gitignored `tests/w3c/owl2/`; when absent the runner SKIPS so a fresh offline
//! checkout stays green. `EL_SUITE_FLOOR` is read TEXTUALLY by
//! `tests/scoreboard_floors.rs`, so the central scoreboard's mirrored value can never
//! silently drift.

// [SONNET-4.6] When the lane feature is OFF the runner is a single self-SKIP test so
// the default + `--workspace` builds neither link the EL classifier nor go red on a
// fresh checkout. (cfg gate, not a runtime branch — zero EL code compiles in the
// default state, the lean-core invariant.)
#[cfg(not(feature = "el-suite"))]
#[test]
fn el_suite_skipped_without_feature() {
    eprintln!(
        "SKIP: the OWL 2 EL classification conformance lane is OFF — build with \
         `--features el-suite` (and run scripts/fetch-inference-suites.sh) to run it."
    );
}

#[cfg(feature = "el-suite")]
mod gated {
    use oxrdf::{NamedOrBlankNode, Term};
    use sparq_conformance::inference::entail::{self, Recognized, Row};
    use sparq_conformance::rdf::MiniGraph;
    use sparq_core::dict::{Dict, Id};
    use sparq_reason_el::classify_graph;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;

    /// The OWL 2 EL classification pass FLOOR (the RATCHET). The MEASURED count of
    /// selected EL (`test:EL` ∧ `test:RDF-BASED`, Approved, no-imports) check rows on
    /// which `sparq_reason_el`'s classifier computes the expected outcome — consistency
    /// (no unsatisfiable class), inconsistency (some unsatisfiable class), positive
    /// entailment (conclusion entailed by the materialized lattice), or negative
    /// entailment (non-conclusion not entailed). MIRRORED in the central scoreboard
    /// (`scoreboard::SUITES`) and read TEXTUALLY by the guard test
    /// `tests/scoreboard_floors.rs`. It may only RISE — never lower it (raise as the EL
    /// classifier's fragment coverage grows). This is the ACTUAL current pass count, not
    /// an aspirational target. HONESTLY a sparq EXTENSION-shaped ratchet over the EL
    /// fragment the classifier implements, NOT a full-OWL-2-EL-conformance claim.
    ///
    /// FLOOR COMPOSITION (51 total): 45 consistency / 0 inconsistency /
    /// 3 positive-entailment / 3 negative-entailment. The 3 positive-entailment passes
    /// are WebOnt-equivalentClass-002 (trivial equivalentClass → 2 subClassOf directions),
    /// WebOnt-equivalentClass-003 (mutual subClassOf → owl:equivalentClass via the
    /// output-vocabulary augment_equivalent_classes completion, sq-pbz04.2.9), and one
    /// further W3C positive-entailment case. The lane-local `el_cr4_derivation_canary`
    /// test below closes the gap for CR4 existential/conjunction derivation — it is NOT
    /// counted into this floor.
    /// [SONNET-4.6] sq-pbz04.2.4 / sq-pbz04.2.9 (RAISED 50→51: WebOnt-equivalentClass-003
    /// graduates via the equivalentClass output-vocabulary completion; rbox+cdomain now on)
    pub const EL_SUITE_FLOOR: usize = 51;

    const T: &str = "http://www.w3.org/2007/OWL/testOntology#";
    const OWL_ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
    const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
    const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

    const TEST_TIMEOUT: Duration = Duration::from_secs(20);

    /// Documented divergences — audited PERMANENT limitations of the EL classifier's
    /// implemented fragment (keyed by `test:identifier`). Each fail was checked against
    /// the OWL 2 EL profile: a divergence is PERMANENT when the expected outcome needs a
    /// rule the classifier deliberately defers (CR7–CR9 concrete domains — faceted
    /// datatype restrictions / `DataHasValue` over literals) OR needs ABox-level
    /// inconsistency the TBox-only classifier cannot reach (it detects unsatisfiable
    /// CLASSES, not an ontology made inconsistent purely by instance assertions). A
    /// divergence counts as pass+divergence in the report but is NEVER summed into the
    /// ratchet floor; adding one is a conscious reviewed act (the `push` mapping reports
    /// a stale entry that actually passes, and an undocumented fail fails the lane).
    ///
    /// The 28 entries below were each audited from their raw export premise/conclusion.
    /// They fall into three PERMANENT mechanisms, none of which is a fixable bug within
    /// the classifier's documented (sound, TBox-only, CR1–CR6, `rbox`/`cdomain` off)
    /// contract:
    /// * ABox / instance reasoning — the classifier emits the `rdfs:subClassOf`
    ///   subsumption lattice and does NOT internalize `rdf:type` / property / (in)equality
    ///   assertions (materializing instance closures is the RL path's job), so a
    ///   conclusion that is an individual fact (`owl:sameAs` / `owl:differentFrom` / a
    ///   property assertion / a `rdf:type owl:Thing` ClassAssertion), or an inconsistency
    ///   forced purely by instance assertions, is out of reach.
    /// * RBox / property reasoning — role inclusions / chains / transitivity /
    ///   reflexivity / `owl:hasSelf` / `owl:hasKey` / negative-property-assertions are the
    ///   OFF-by-default `rbox` feature (or outside EL entirely); without it roles are
    ///   compared for equality only.
    /// * Output-vocabulary / outside-EL — a conclusion in the `owl:equivalentClass` /
    ///   `owl:equivalentProperty` / `owl:TransitiveProperty` axiom form (the classifier
    ///   emits `rdfs:subClassOf` edges, not those axioms) or using `owl:unionOf` (outside
    ///   EL — recorded as a skipped axiom).
    ///
    /// [SONNET-4.6] sq-pbz04.2.4
    const DOCUMENTED_DIVERGENCES: &[(&str, &str)] = &[
        // --- positive-entailment: conclusion is an ABox individual fact ---
        (
            "New-Feature-Keys-001",
            "PERMANENT — conclusion is the ABox fact `Peter owl:sameAs Peter_Griffin` derived \
             via owl:hasKey; the TBox subsumption classifier emits no owl:sameAs and owl:hasKey \
             is outside its fragment (individual equality is ABox — the RL path's job)",
        ),
        (
            "New-Feature-Keys-003",
            "PERMANENT — identical owl:hasKey-driven `Peter owl:sameAs Peter_Griffin` ABox \
             conclusion as New-Feature-Keys-001; no owl:sameAs is emitted by the TBox classifier",
        ),
        (
            "New-Feature-ObjectPropertyChain-001",
            "PERMANENT — conclusion is the ABox property assertion `Stewie hasAunt Carol` via an \
             owl:propertyChainAxiom; the TBox subsumption classifier emits rdfs:subClassOf lattice \
             edges only — instance-level property assertions are ABox and outside its output \
             vocabulary (rbox/CR10/CR11 is on but drives TBox class-subsumption derivation, not \
             ABox instance-fact emission)",
        ),
        (
            "New-Feature-ObjectPropertyChain-BJP-003",
            "PERMANENT — conclusion is the ABox property assertion `a p c` via a property chain; \
             the TBox subsumption classifier does not emit instance-level property assertions even \
             with rbox/CR10/CR11 on (rbox drives class-subsumption derivation, not ABox output)",
        ),
        (
            "New-Feature-ReflexiveProperty-001",
            "PERMANENT — conclusion is the ABox assertion `Peter knows Peter` from a reflexive \
             object property; general role reflexivity is RBox/ABox, not a TBox class subsumption",
        ),
        (
            "New-Feature-SelfRestriction-001",
            "ABox — conclusion `Peter likes Peter` is a property assertion; owl:hasSelf is now \
             in-fragment (CR-Self, sq-pbz04.2.6) and the `abox` realiser reads the self-loop off, \
             but this lane runs the TBox `classify_graph` (no `abox`), which by contract emits no \
             instance facts (graduates under the C2 ABox-mechanism re-pin — see \
             research/owl2-rl-el-wave2-disposition.md §4)",
        ),
        (
            "New-Feature-SelfRestriction-002",
            "ABox — conclusion types the individual Peter into an owl:hasSelf restriction (the \
             CONVERSE of CR-Self-1: a raw self-link ⇒ ∃r.Self); owl:hasSelf is now in-fragment \
             but this converse nominal readoff + the ABox ClassAssertion are out of this TBox \
             lane's reach (documented sq-pbz04.2.6 follow-up)",
        ),
        (
            "WebOnt-Ontology-001",
            "PERMANENT — conclusion is the ABox ClassAssertions `auto/car rdf:type owl:Thing`; \
             the TBox classifier emits no instance typings",
        ),
        (
            "WebOnt-differentFrom-001",
            "PERMANENT — conclusion is the ABox fact `b owl:differentFrom a`; individual \
             (in)equality is ABox, outside the TBox subsumption lattice",
        ),
        (
            "WebOnt-disjointWith-001",
            "PERMANENT — conclusion is the ABox fact `a owl:differentFrom b` derived from a \
             disjointWith; individual inequality is ABox",
        ),
        (
            "WebOnt-equivalentClass-001",
            "PERMANENT — conclusion is the ABox ClassAssertions `auto/car rdf:type owl:Thing`; \
             no instance typings are emitted by the TBox classifier",
        ),
        (
            "WebOnt-equivalentProperty-001",
            "PERMANENT — conclusion is the ABox property assertion `X hasHead Y` via property \
             equivalence; the conclusion is an ABox instance fact outside the TBox classifier's \
             output vocabulary (even with rbox on, the classifier emits class-subsumption edges, \
             not individual property assertions)",
        ),
        (
            "WebOnt-sameAs-001",
            "PERMANENT — conclusion transfers an annotation across an owl:sameAs individual merge \
             (`c2 first:annotate ...`); annotation + individual equality are ABox, outside the \
             TBox subsumption lattice",
        ),
        // --- positive-entailment: TBox axiom in a form the classifier does not emit / outside EL ---
        (
            "chain2trans1",
            "PERMANENT — conclusion is the TBox property axiom `p rdf:type owl:TransitiveProperty` \
             (from a self property-chain); rbox/CR10/CR11 is on but no EL rule emits the \
             owl:TransitiveProperty axiom form — the classifier emits rdfs:subClassOf and role-inclusion \
             edges, not property-type declarations",
        ),
        (
            "WebOnt-equivalentProperty-002",
            "PERMANENT — conclusion is the property subsumption `hasHead rdfs:subPropertyOf \
             hasLeader` (both directions); the EL classifier emits class-subsumption \
             (rdfs:subClassOf) edges, not property-subsumption (rdfs:subPropertyOf) edges — \
             property-hierarchy output is outside its output vocabulary (rbox/CR10/CR11 is on \
             but its role-inclusion derivations feed class-subsumption computation, not \
             rdfs:subPropertyOf emission)",
        ),
        (
            "WebOnt-equivalentProperty-003",
            "PERMANENT — conclusion is `hasHead owl:equivalentProperty hasLeader`; the EL \
             classifier does not emit owl:equivalentProperty axioms (output vocabulary is \
             rdfs:subClassOf only; rbox is on but role inclusions feed class-subsumption, \
             not property-equivalence-axiom emission)",
        ),
        (
            "WebOnt-I5.5-005",
            "PERMANENT — conclusion asserts the existence of an anonymous `[ owl:unionOf (a) ]` \
             class; owl:unionOf is outside EL (recorded as a skipped axiom) and no EL rule emits \
             union / rdf:List cells",
        ),
        // --- inconsistency: TBox-only classifier finds unsatisfiable NAMED classes, not ABox/global clashes ---
        (
            "DisjointClasses-002",
            "PERMANENT — the inconsistency is ABox-driven (`Stewie` asserted into both disjoint \
             Boy and Girl); Boy/Girl are individually satisfiable, so no NAMED class is \
             unsatisfiable — the classifier does not internalize ABox rdf:type assertions",
        ),
        (
            "New-Feature-BottomDataProperty-001",
            "PERMANENT — an individual is typed into `∃owl:bottomDataProperty.Literal`; the clash \
             is ABox (an instance of an unsatisfiable ANONYMOUS class), not a named-class \
             unsatisfiability the TBox classifier surfaces",
        ),
        (
            "New-Feature-BottomObjectProperty-001",
            "PERMANENT — as New-Feature-BottomDataProperty-001 with owl:bottomObjectProperty: an \
             ABox instance of `∃⊥.⊤`, so no NAMED class is unsatisfiable",
        ),
        (
            "New-Feature-Keys-002",
            "PERMANENT — an owl:hasKey collision forces `Peter sameAs Peter_Griffin` against a \
             differentFrom; owl:hasKey is outside the classifier's fragment and the clash is ABox \
             (individual (in)equality)",
        ),
        (
            "New-Feature-Keys-006",
            "PERMANENT — a functional data property + owl:hasKey clash over the individual Peter; \
             both owl:hasKey and functional-property ABox reasoning are outside the TBox classifier",
        ),
        (
            "New-Feature-NegativeDataPropertyAssertion-001",
            "PERMANENT — an owl:NegativePropertyAssertion contradicts an asserted data-property \
             value; NegativePropertyAssertion + ABox are outside the TBox classifier",
        ),
        (
            "New-Feature-NegativeObjectPropertyAssertion-001",
            "PERMANENT — as the data-property case with an object property; \
             NegativePropertyAssertion + ABox are outside the TBox classifier",
        ),
        (
            "WebOnt-Restriction-001",
            "PERMANENT — individuals a,b are typed into `∃op.owl:Nothing`; the clash is ABox \
             (instances of an unsatisfiable ANONYMOUS class), so no NAMED class is unsatisfiable",
        ),
        (
            "WebOnt-Restriction-002",
            "PERMANENT — as WebOnt-Restriction-001 with a shared-nodeID restriction; an ABox \
             instance clash, no named-class unsatisfiability",
        ),
        (
            "WebOnt-Thing-003",
            "PERMANENT — `owl:Thing owl:equivalentClass owl:Nothing` is a global ⊤ ⊑ ⊥ \
             inconsistency; the classifier decides NAMED-class satisfiability (⊤/⊥ excluded from \
             that surface) and by its documented contract does not decide whole-ontology \
             consistency, so it flags no unsatisfiable named class",
        ),
    ];

    /// Locate the OWL WG export the same way the inference binary + d-entail lane do:
    /// the workspace-root `tests/w3c/owl2/all.rdf` (fetched by
    /// scripts/fetch-inference-suites.sh). `CARGO_MANIFEST_DIR` is crates/sparq-conformance.
    fn owl_export() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/owl2/all.rdf")
    }

    /// One outcome bucket for a single check row.
    enum Outcome {
        Pass,
        /// An audited PERMANENT EL-classifier limitation (rationale, observed).
        Divergence(&'static str, String),
        /// An undocumented failure — the observed reason (fails the lane).
        Fail(String),
        /// Deliberately not run, with the honest reason (excluded from the floor).
        Deferred(String),
    }

    struct Case {
        ident: String,
        status: Option<String>,
        checks: Vec<&'static str>,
        premise: Option<String>,
        conclusion: Option<String>,
        nonconclusion: Option<String>,
        imports: bool,
    }

    /// The classification result of one premise ontology, produced under a watchdog.
    struct Classified {
        /// The materialized closure (original premise + derived `rdfs:subClassOf`
        /// edges), as term rows for the bnode-homomorphism entailment check.
        closure: Vec<Row>,
        /// Count of named classes found unsatisfiable (`⊑ owl:Nothing`).
        unsatisfiable: usize,
    }

    #[test]
    fn el_classification_ratchet() {
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
        // oxrdfxml rejects single-quoted DOCTYPE entity values; normalize just the
        // DOCTYPE block (same fix the RL `owl_suite` applies).
        let fixed = fix_doctype_quotes(&text);
        let parser = oxrdfxml::RdfXmlParser::new()
            .with_base_iri("http://owl.semanticweb.org/exports/all.rdf")
            .expect("base IRI");
        let mut triples = Vec::new();
        for t in parser.for_slice(fixed.as_bytes()) {
            triples.push(t.unwrap_or_else(|e| panic!("all.rdf: {e}")));
        }
        let g = MiniGraph { triples };

        // Accounting buckets.
        let mut pass = 0usize;
        // [SONNET-4.6] Sub-floor: how many of the `pass` rows are positive-entailment
        // checks. A neutered classifier (empty saturation) cannot derive the W3C
        // equivalentClass conclusions, so this drops to 0 and the sub-floor fires
        // BEFORE `fails.is_empty()` — guaranteeing the derivation gap is caught even
        // when all checked rows become undocumented fails at once.
        let mut positive_entailment_passes = 0usize;
        let mut divergences: Vec<(String, String, String)> = Vec::new(); // (test, rationale, observed)
        let mut fails: Vec<(String, String)> = Vec::new(); // (test/kind, reason)
        let mut deferred: Vec<String> = Vec::new(); // reason strings (histogrammed)
        let mut selected_cases = 0usize;

        for case_node in g.subjects_with_type(&format!("{}TestCase", T)) {
            let types = g.types_of(&case_node);
            let has = |t: &str| types.iter().any(|x| x == &format!("{}{}", T, t));
            let mut checks: Vec<&'static str> = Vec::new();
            if has("ConsistencyTest") {
                checks.push("consistency");
            }
            if has("InconsistencyTest") {
                checks.push("inconsistency");
            }
            if has("PositiveEntailmentTest") {
                checks.push("positive-entailment");
            }
            if has("NegativeEntailmentTest") {
                checks.push("negative-entailment");
            }
            if checks.is_empty() {
                continue; // ProfileIdentificationTest-only — not a reasoning check.
            }
            let iri_objs = |p: &str| -> Vec<String> {
                g.objects(&case_node, &format!("{}{}", T, p))
                    .into_iter()
                    .filter_map(|t| match t {
                        Term::NamedNode(n) => Some(n.as_str().to_string()),
                        _ => None,
                    })
                    .collect()
            };
            // The EL applicability rule: EL profile AND RDF-based semantics.
            if !iri_objs("profile").iter().any(|p| p == &format!("{}EL", T)) {
                continue;
            }
            if !iri_objs("semantics")
                .iter()
                .any(|s| s == &format!("{}RDF-BASED", T))
            {
                continue;
            }

            let lit = |p: &str| -> Option<String> {
                g.str_object(&case_node, &format!("{}{}", T, p))
            };
            let case = Case {
                ident: lit("identifier").unwrap_or_else(|| match &case_node {
                    NamedOrBlankNode::NamedNode(n) => n
                        .as_str()
                        .rsplit('/')
                        .next()
                        .unwrap_or(n.as_str())
                        .to_string(),
                    NamedOrBlankNode::BlankNode(b) => format!("_:{}", b.as_str()),
                }),
                status: g
                    .object(&case_node, &format!("{}status", T))
                    .and_then(|t| match t {
                        Term::NamedNode(n) => n.as_str().strip_prefix(T).map(|s| s.to_string()),
                        _ => None,
                    }),
                checks,
                premise: lit("rdfXmlPremiseOntology").or_else(|| lit("rdfXmlInputOntology")),
                conclusion: lit("rdfXmlConclusionOntology"),
                nonconclusion: lit("rdfXmlNonConclusionOntology"),
                imports: g
                    .object(&case_node, &format!("{}importedOntology", T))
                    .is_some()
                    || g.object(&case_node, &format!("{}importedOntologyIRI", T))
                        .is_some(),
            };
            selected_cases += 1;
            for (kind, outcome) in run_case(case) {
                match outcome {
                    Outcome::Pass => {
                        pass += 1;
                        if kind.contains("positive-entailment") {
                            positive_entailment_passes += 1;
                        }
                    }
                    Outcome::Divergence(rationale, observed) => {
                        divergences.push((kind, rationale.to_string(), observed))
                    }
                    Outcome::Fail(reason) => fails.push((kind, reason)),
                    Outcome::Deferred(reason) => deferred.push(reason),
                }
            }
        }

        let total = pass + divergences.len() + fails.len() + deferred.len();

        // [SONNET-4.6] The grep-able ratchet line — pass count in field $6 (`OWL`, `2`,
        // `EL`, `ratchet`, `pass`, N, `of`, M, `(floor`, F). Positional `println!` args
        // (not inline `{pass}`) per the CodeQL `rust/unused-variable` false-positive guard.
        println!("\nOWL 2 EL classification conformance (pinned OWL WG export, sq-pbz04.2.4)");
        println!(
            "OWL 2 EL ratchet pass {} of {} (floor {})",
            pass, total, EL_SUITE_FLOOR
        );
        println!(
            "  selected EL∧RDF-BASED cases {}; pass {}, documented-divergence {}, deferred/OoS {}, undocumented-fail {}",
            selected_cases,
            pass,
            divergences.len(),
            deferred.len(),
            fails.len()
        );
        if !divergences.is_empty() {
            println!("EL documented divergences (NOT summed into the floor):");
            for (test, rationale, observed) in &divergences {
                println!("  - {} — {} [observed: {}]", test, rationale, observed);
            }
        }
        if !deferred.is_empty() {
            let mut hist: std::collections::BTreeMap<&str, usize> =
                std::collections::BTreeMap::new();
            for r in &deferred {
                *hist.entry(r.as_str()).or_default() += 1;
            }
            println!("EL deferred / out-of-scope (excluded from the floor):");
            for (reason, n) in &hist {
                println!("  - {} × {}", n, reason);
            }
        }
        if !fails.is_empty() {
            println!("EL UNDOCUMENTED FAILS (fail the lane — audit into a divergence or fix):");
            for (test, reason) in &fails {
                println!("  - {} — {}", test, reason);
            }
        }

        // [SONNET-4.6] Positive-entailment sub-floor: at least 3 of the counted passes
        // must be positive-entailment checks. A neutered classifier (empty saturation)
        // loses all W3C positive-entailment passes, dropping this to 0 and turning
        // THIS assert RED before `fails.is_empty()` gets a chance to fire — so the
        // derivation gap is caught even if all new undocumented fails are added to the
        // divergence list first. See FLOOR COMPOSITION note on EL_SUITE_FLOOR.
        // [SONNET-4.6] sq-pbz04.2.9: raised from 2 to 3 — WebOnt-equivalentClass-003
        // now graduates via the augment_equivalent_classes output-vocabulary completion.
        assert!(
            positive_entailment_passes >= 3,
            "OWL 2 EL positive-entailment sub-floor: expected >= 3 positive-entailment \
             passes, got {} — floor composition (45 consistency / 0 inconsistency / \
             3 positive-entailment / 3 negative-entailment) has shifted; a neutered \
             classifier scores 0 here",
            positive_entailment_passes
        );
        // No silent fails: every failure must be an audited documented divergence.
        assert!(
            fails.is_empty(),
            "OWL 2 EL lane has {} UNDOCUMENTED failure(s) — audit each into \
             DOCUMENTED_DIVERGENCES with a rationale, or fix the classifier: {:?}",
            fails.len(),
            fails
        );
        // The ratchet.
        assert!(
            pass >= EL_SUITE_FLOOR,
            "OWL 2 EL pass count {} regressed below the ratchet floor {}",
            pass,
            EL_SUITE_FLOOR
        );
    }

    /// Runs one selected case, returning `(kind, outcome)` per declared check.
    fn run_case(case: Case) -> Vec<(String, Outcome)> {
        let ident = case.ident.clone();
        // Map an undocumented Fail on a documented-divergence test to a Divergence; a
        // divergence-listed test that PASSES stays a Pass (surfacing a stale entry).
        let finish = |kind: &str, outcome: Outcome| -> (String, Outcome) {
            let key = format!("owl2-el/{}: {}", kind, ident);
            let outcome = match outcome {
                Outcome::Fail(observed) => match DOCUMENTED_DIVERGENCES
                    .iter()
                    .find(|(name, _)| *name == ident)
                {
                    Some((_, rationale)) => Outcome::Divergence(rationale, observed),
                    None => Outcome::Fail(observed),
                },
                other => other,
            };
            (key, outcome)
        };

        if case.status.as_deref() != Some("Approved") {
            let why = format!(
                "status {} (only Approved cases are conformance tests)",
                case.status.as_deref().unwrap_or("absent")
            );
            return case
                .checks
                .iter()
                .map(|k| finish(k, Outcome::Deferred(why.clone())))
                .collect();
        }
        if case.imports {
            return case
                .checks
                .iter()
                .map(|k| {
                    finish(
                        k,
                        Outcome::Deferred(
                            "uses owl:imports (no dereferencing in the harness)".into(),
                        ),
                    )
                })
                .collect();
        }
        let Some(premise_xml) = case.premise.clone() else {
            return case
                .checks
                .iter()
                .map(|k| {
                    finish(
                        k,
                        Outcome::Deferred("premise only available in functional syntax".into()),
                    )
                })
                .collect();
        };
        let base = format!("http://owl.semanticweb.org/id/{}", case.ident);
        let premise = match parse_ontology(&premise_xml, &base) {
            Ok(rows) => rows,
            Err(e) => {
                return case
                    .checks
                    .iter()
                    .map(|k| finish(k, Outcome::Fail(format!("premise RDF/XML parse error: {}", e))))
                    .collect();
            }
        };

        // Classify once per case under a watchdog (a hang/panic in the classifier is a
        // recorded FAIL, not a dead harness) — the REAL `classify_graph` path.
        let classified = match classify_under_watchdog(premise) {
            Ok(c) => c,
            Err(reason) => {
                return case
                    .checks
                    .iter()
                    .map(|k| finish(k, Outcome::Fail(reason.clone())))
                    .collect();
            }
        };

        let d = Recognized::standard();
        let mut out = Vec::new();
        for kind in case.checks.clone() {
            let outcome = match kind {
                "consistency" => {
                    if classified.unsatisfiable == 0 {
                        Outcome::Pass
                    } else {
                        Outcome::Fail(format!(
                            "wrongly judged {} class(es) unsatisfiable",
                            classified.unsatisfiable
                        ))
                    }
                }
                "inconsistency" => {
                    if classified.unsatisfiable > 0 {
                        Outcome::Pass
                    } else {
                        Outcome::Fail(
                            "inconsistency not detected (no unsatisfiable named class — \
                             ABox-only inconsistency is outside the TBox classifier)"
                                .into(),
                        )
                    }
                }
                "positive-entailment" => match &case.conclusion {
                    None => Outcome::Deferred(
                        "conclusion only available in functional syntax".into(),
                    ),
                    Some(xml) => match parse_ontology(xml, &base) {
                        Err(e) => Outcome::Fail(format!("conclusion RDF/XML parse error: {}", e)),
                        Ok(conclusion) => {
                            // [SONNET-4.6] sq-pbz04.2.9 — chain both augmentations:
                            // (1) datatype axiomatic set (existing), then (2) the
                            // output-vocabulary equivalentClass completion (new).
                            let closure = augment_datatypes(&classified.closure, &conclusion, &d);
                            let closure = augment_equivalent_classes(&closure);
                            if entail::entails(&closure, &conclusion, &d) {
                                Outcome::Pass
                            } else {
                                Outcome::Fail(
                                    "conclusion not entailed by the EL subsumption lattice".into(),
                                )
                            }
                        }
                    },
                },
                "negative-entailment" => match &case.nonconclusion {
                    None => Outcome::Deferred(
                        "non-conclusion only available in functional syntax".into(),
                    ),
                    Some(xml) => match parse_ontology(xml, &base) {
                        Err(e) => {
                            Outcome::Fail(format!("non-conclusion RDF/XML parse error: {}", e))
                        }
                        Ok(nonconclusion) => {
                            if entail::entails(&classified.closure, &nonconclusion, &d) {
                                Outcome::Fail("non-conclusion wrongly entailed".into())
                            } else {
                                Outcome::Pass
                            }
                        }
                    },
                },
                _ => unreachable!(),
            };
            out.push(finish(kind, outcome));
        }
        out
    }

    /// Runs `classify_graph` on a watchdog thread, catching panics + capping runtime.
    ///
    /// [SONNET-4.6] Thread-lifecycle note: the JoinHandle is stored and then
    /// EXPLICITLY dropped after `recv_timeout` returns. On the normal path the worker
    /// has already sent its result and will exit momentarily; dropping the handle
    /// detaches it (matching the pre-existing implicit-drop behaviour). On the 20 s
    /// TIMEOUT path the worker may still be running — Rust's JoinHandle drop detaches
    /// the thread and lets it run to completion or panic on its own; we have no
    /// cancellation channel so this is the least-harmful option for a test-harness
    /// thread. The explicit drop here (rather than the previous implicit statement-end
    /// drop) makes the intentional detach visible to reviewers.
    fn classify_under_watchdog(premise: Vec<Row>) -> Result<Classified, String> {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            let result = std::panic::catch_unwind(move || {
                let mut dict = Dict::new();
                let mut ids: Vec<[Id; 3]> = premise
                    .iter()
                    .map(|[s, p, o]| [dict.intern(s), dict.intern(p), dict.intern(o)])
                    .collect();
                let report = classify_graph(&mut dict, &mut ids);
                let closure: Vec<Row> = ids
                    .into_iter()
                    .map(|[s, p, o]| [dict.term(s), dict.term(p), dict.term(o)])
                    .collect();
                Classified {
                    closure,
                    unsatisfiable: report.unsatisfiable_classes,
                }
            });
            let _ = tx.send(result);
        });
        let out = match rx.recv_timeout(TEST_TIMEOUT) {
            Ok(Ok(c)) => Ok(c),
            Ok(Err(_)) => Err("EL classifier panicked".into()),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err("timeout (20s) in EL classification".into())
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("EL classifier panicked".into()),
        };
        // Intentional detach: see the function-level comment above.
        drop(handle);
        out
    }

    /// Finite-restriction datatype axioms: every recognized datatype is an
    /// `rdfs:Datatype` in any OWL 2 datatype map; add the typing for the datatypes the
    /// CONCLUSION mentions (the conclusion-vocabulary device rdf-mt uses for its infinite
    /// axiomatic sets), matching the RL `owl_suite` positive-entailment path. Returns a
    /// fresh closure so the negative-entailment check keeps the un-augmented one.
    fn augment_datatypes(closure: &[Row], conclusion: &[Row], d: &Recognized) -> Vec<Row> {
        let mut closure = closure.to_vec();
        for row in conclusion {
            for t in row {
                if let Term::NamedNode(n) = t {
                    if d.contains(n.as_str()) {
                        closure.push([
                            t.clone(),
                            Term::NamedNode(oxrdf::NamedNode::new_unchecked(RDF_TYPE)),
                            Term::NamedNode(oxrdf::NamedNode::new_unchecked(
                                oxrdf::vocab::rdfs::DATATYPE.as_str(),
                            )),
                        ]);
                    }
                }
            }
        }
        closure
    }

    /// Output-vocabulary completion: T ⊨ 'A owl:equivalentClass B' iff BOTH
    /// `A rdfs:subClassOf B` AND `B rdfs:subClassOf A` are in the materialized closure
    /// (mutual subsumption is the semantic identity for owl:equivalentClass in OWL 2
    /// RDF-based semantics — Table 8 of OWL 2 Mapping to RDF Graphs). The classifier
    /// emits `rdfs:subClassOf` edges but not the axiom-form `owl:equivalentClass` triple;
    /// adding it here is an EXACT output-vocabulary completion, NOT a test weakening.
    ///
    /// This mirrors the `augment_datatypes` precedent and is applied ONLY to the closure
    /// used for the positive-entailment bnode-homomorphism check (the negative-entailment
    /// path keeps the un-augmented closure, per the fail-closed invariant).
    ///
    /// [SONNET-4.6] sq-pbz04.2.9
    fn augment_equivalent_classes(closure: &[Row]) -> Vec<Row> {
        const RDFS_SUBCLASSOF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
        const OWL_EQUIVALENT_CLASS: &str = "http://www.w3.org/2002/07/owl#equivalentClass";

        // Collect all (subject, object) pairs of rdfs:subClassOf where both subject and
        // object are named nodes (not blank nodes).  A blank-node restriction keeps the
        // augmentation conservative: anonymous restriction nodes are structural, not
        // named classes, and the W3C equivalentClass tests only assert pairs of IRIs.
        use std::collections::HashSet;
        let mut sub_of: HashSet<(String, String)> = HashSet::new();
        for [s, p, o] in closure {
            if let (Term::NamedNode(sn), Term::NamedNode(pn), Term::NamedNode(on)) = (s, p, o) {
                if pn.as_str() == RDFS_SUBCLASSOF {
                    sub_of.insert((sn.as_str().to_string(), on.as_str().to_string()));
                }
            }
        }

        let mut extra: Vec<Row> = Vec::new();
        let eq_prop = Term::NamedNode(oxrdf::NamedNode::new_unchecked(OWL_EQUIVALENT_CLASS));
        for (a, b) in &sub_of {
            if sub_of.contains(&(b.clone(), a.clone())) {
                // Both A ⊑ B and B ⊑ A hold — emit A owl:equivalentClass B (one direction;
                // the symmetric B owl:equivalentClass A will be emitted when (b,a) is the
                // outer loop iteration).
                let an = Term::NamedNode(oxrdf::NamedNode::new_unchecked(a.clone()));
                let bn = Term::NamedNode(oxrdf::NamedNode::new_unchecked(b.clone()));
                let row: Row = [an, eq_prop.clone(), bn];
                if !closure.contains(&row) {
                    extra.push(row);
                }
            }
        }

        if extra.is_empty() {
            closure.to_vec()
        } else {
            let mut augmented = closure.to_vec();
            augmented.extend(extra);
            augmented
        }
    }

    /// Parses an inline RDF/XML ontology literal, dropping the ontology header
    /// (`?x rdf:type owl:Ontology` typings and `owl:imports` edges) — the harness
    /// compares axiom triples, mirroring the official OWLWG harness (and the RL lane).
    fn parse_ontology(xml: &str, base: &str) -> Result<Vec<Row>, String> {
        let parser = oxrdfxml::RdfXmlParser::new()
            .with_base_iri(base)
            .map_err(|e| e.to_string())?;
        let mut rows = Vec::new();
        for t in parser.for_slice(xml.as_bytes()) {
            let t = t.map_err(|e| e.to_string())?;
            rows.push(entail::triple_row(&t));
        }
        rows.retain(|[_, p, o]| {
            let is_type = matches!(p, Term::NamedNode(n) if n.as_str() == RDF_TYPE);
            let is_imports = matches!(p, Term::NamedNode(n) if n.as_str() == OWL_IMPORTS);
            let is_ontology = matches!(o, Term::NamedNode(n) if n.as_str() == OWL_ONTOLOGY);
            !(is_imports || (is_type && is_ontology))
        });
        Ok(rows)
    }

    /// [SONNET-4.6] EL derivation canary — lane-local guard, NOT a W3C test case and
    /// NOT counted into the W3C pass total or `EL_SUITE_FLOOR`.
    ///
    /// Verifies the CR4 existential-traversal rule through the SAME
    /// `classify_under_watchdog` → `classify_graph` path the W3C suite uses:
    ///
    ///   TBox: A ⊑ ∃r.B,  B ⊑ C,  ∃r.C ⊑ D  ⊨  A ⊑ D
    ///
    /// This chain requires CR3 (emit the r-successor link (A, B)) then CR4 (pull D into
    /// S(A) via the (A,B)∈R(r) link and B⊑C and ∃r.C⊑D) — the exact derivation OWL 2
    /// RL cannot perform. A neutered classifier (empty saturation) leaves A⊑D absent
    /// from the closure and turns this test RED, proving the ratchet exercises real
    /// existential reasoning.
    #[test]
    fn el_cr4_derivation_canary() {
        // Inline RDF/XML — does NOT require the OWL WG export file; always runs when
        // the `el-suite` feature is ON.
        let premise_xml = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
  <owl:Ontology rdf:about="http://ex/canary-cr4"/>
  <owl:Class rdf:about="http://ex/A">
    <rdfs:subClassOf>
      <owl:Restriction>
        <owl:onProperty rdf:resource="http://ex/r"/>
        <owl:someValuesFrom rdf:resource="http://ex/B"/>
      </owl:Restriction>
    </rdfs:subClassOf>
  </owl:Class>
  <owl:Class rdf:about="http://ex/B">
    <rdfs:subClassOf rdf:resource="http://ex/C"/>
  </owl:Class>
  <owl:Class rdf:about="http://ex/D"/>
  <owl:Restriction>
    <owl:onProperty rdf:resource="http://ex/r"/>
    <owl:someValuesFrom rdf:resource="http://ex/C"/>
    <rdfs:subClassOf rdf:resource="http://ex/D"/>
  </owl:Restriction>
</rdf:RDF>"#;
        let base = "http://ex/canary-cr4";
        let premise =
            parse_ontology(premise_xml, base).expect("canary premise RDF/XML must parse cleanly");
        let classified =
            classify_under_watchdog(premise).expect("canary: classify_graph must not panic/timeout");
        // A ⊑ D must appear in the derived closure as a plain rdfs:subClassOf triple.
        // [SONNET-4.6] CodeQL positional format! args used throughout (rust/unused-variable guard).
        let a = Term::NamedNode(oxrdf::NamedNode::new_unchecked(
            "http://ex/A".to_string(),
        ));
        let sub_class_of = Term::NamedNode(oxrdf::NamedNode::new_unchecked(
            "http://www.w3.org/2000/01/rdf-schema#subClassOf".to_string(),
        ));
        let d = Term::NamedNode(oxrdf::NamedNode::new_unchecked(
            "http://ex/D".to_string(),
        ));
        let expected: Row = [a, sub_class_of, d];
        assert!(
            classified.closure.contains(&expected),
            "EL CR4 derivation canary FAILED: A ⊑ D not found in the classify_graph \
             closure for TBox {{A ⊑ ∃r.B, B ⊑ C, ∃r.C ⊑ D}}. \
             A neutered (empty-saturation) classifier produces this failure — \
             restore CR1–CR6 saturation in sparq-reason-el::classify_graph."
        );
    }

    /// The export's internal DTD uses single-quoted ENTITY values, which oxrdfxml
    /// rejects; rewrite just the DOCTYPE block to double quotes (same fix the RL lane
    /// applies).
    fn fix_doctype_quotes(text: &str) -> String {
        let Some(start) = text.find("<!DOCTYPE") else {
            return text.to_string();
        };
        let Some(end) = text[start..].find("]>").map(|e| start + e + 2) else {
            return text.to_string();
        };
        let mut fixed = String::with_capacity(text.len());
        fixed.push_str(&text[..start]);
        fixed.push_str(&text[start..end].replace('\'', "\""));
        fixed.push_str(&text[end..]);
        fixed
    }
}
