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
//! OPT-IN `sparq-reason-el` crate). With the feature OFF this file compiles to a
//! single self-SKIP `#[test]` (no EL classifier links — the lean opt-in posture, so
//! the default + `--workspace` byte/bundle ratchets are byte-for-byte unchanged).
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
    /// [SONNET-4.6] sq-pbz04.2.4
    pub const EL_SUITE_FLOOR: usize = 50;

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
             owl:propertyChainAxiom; role chains (CR10/CR11) are the OFF-by-default `rbox` feature \
             and the conclusion is an instance fact, neither in this lane's TBox subsumption output",
        ),
        (
            "New-Feature-ObjectPropertyChain-BJP-003",
            "PERMANENT — ABox property assertion `a p c` via a property chain; RBox (CR10/CR11) is \
             the OFF-by-default `rbox` feature and the conclusion is an instance fact",
        ),
        (
            "New-Feature-ReflexiveProperty-001",
            "PERMANENT — conclusion is the ABox assertion `Peter knows Peter` from a reflexive \
             object property; general role reflexivity is RBox/ABox, not a TBox class subsumption",
        ),
        (
            "New-Feature-SelfRestriction-001",
            "PERMANENT — conclusion `Peter likes Peter` from owl:hasSelf; owl:hasSelf is outside \
             the classifier's EL fragment and the conclusion is an ABox fact",
        ),
        (
            "New-Feature-SelfRestriction-002",
            "PERMANENT — conclusion types the individual Peter into an owl:hasSelf restriction; \
             owl:hasSelf is outside the classifier's EL fragment and ClassAssertion is ABox",
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
             equivalence; property (RBox) reasoning is the OFF-by-default `rbox` feature and the \
             conclusion is an instance fact",
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
             (from a self property-chain); role reasoning (CR10/CR11) is the OFF-by-default `rbox` \
             feature and no rule emits owl:TransitiveProperty",
        ),
        (
            "WebOnt-equivalentClass-003",
            "PERMANENT — the classifier materializes the two-way rdfs:subClassOf edges \
             (Car ⊑ Automobile ∧ Automobile ⊑ Car) but emits them as rdfs:subClassOf, not the \
             owl:equivalentClass axiom the conclusion asserts, so the syntactic bnode-homomorphism \
             check does not match (an output-vocabulary divergence, not an inference gap)",
        ),
        (
            "WebOnt-equivalentProperty-002",
            "PERMANENT — conclusion is the property subsumption `hasHead rdfs:subPropertyOf \
             hasLeader` (both directions); property/RBox reasoning is the OFF-by-default `rbox` \
             feature (roles are compared for equality only here)",
        ),
        (
            "WebOnt-equivalentProperty-003",
            "PERMANENT — conclusion is `hasHead owl:equivalentProperty hasLeader`; property/RBox \
             reasoning is the OFF-by-default `rbox` feature",
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
                    Outcome::Pass => pass += 1,
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
                            let closure = augment_datatypes(&classified.closure, &conclusion, &d);
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
    fn classify_under_watchdog(premise: Vec<Row>) -> Result<Classified, String> {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
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
        match rx.recv_timeout(TEST_TIMEOUT) {
            Ok(Ok(c)) => Ok(c),
            Ok(Err(_)) => Err("EL classifier panicked".into()),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err("timeout (20s) in EL classification".into())
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("EL classifier panicked".into()),
        }
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
