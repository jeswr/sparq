//! [FABLE-5] sq-pbz04.4.5 (epic sq-pbz04.4) — the OWL 2 **Direct-Semantics arm** of the
//! OWL WG test-repository export (`tests/w3c/owl2/all.rdf`), behind the opt-in
//! `dl-direct` feature. 🤖 SPARQ agent.
//!
//! Design record `research/owl2-direct-semantics-scoping.md` §4 (double-counting +
//! profile-lane semantics) / §6 (wiring). Two lanes, both driven through the REAL
//! `sparq-reason-dl` pipeline and both HONESTLY tallied as **sparq-extension** ratchets
//! over the **scoped fragment — NOT full OWL 2 DL**, never folded into
//! standards-conformance totals:
//!
//! 1. **Profile identification** — every `ProfileIdentificationTest` in the DIRECT arm,
//!    checked with the L2 syntactic profile checker (`profiles_from_extraction` — bead
//!    sq-pbz04.4.2), **POSITIVE tags only** (the design record §4 fallback, and here is
//!    the MEASUREMENT that forced it): the record allowed a negative direction only if a
//!    validation held in the L2 work; #1469 performed no such validation, and this
//!    lane's implementation MEASURED the export's EXPLICIT
//!    `owl:NegativePropertyAssertion` profile negations against L2 — on 211 of 322
//!    checkable explicit-negative rows the checker answered `In`, because L2's `In` is
//!    AXIOM-GRAMMAR membership over the extracted ALCH shadow and cannot refute
//!    full-profile membership (the profile specs' non-grammar restrictions — anonymous
//!    individuals, datatype maps, global constraints — are not implemented at L2). So
//!    no negative expectation is checked, and NOTHING is ever inferred from a missing
//!    tag. `test:species` (DL/FULL) is the deferred species check (record §4), not
//!    checked at all.
//! 2. **Direct reasoning** — every `ConsistencyTest` / `InconsistencyTest` /
//!    `PositiveEntailmentTest` / `NegativeEntailmentTest` carrying
//!    `test:semantics test:DIRECT`, decided by the L4 fragment-dispatch
//!    `DirectChecker` (bead sq-pbz04.4.4) under a PINNED deterministic count budget
//!    (wall-clock budgets are banned — floors must be reproducible).
//!
//! # Tri-state accounting (the load-bearing invariant)
//!
//! Every selected check row lands in exactly one [`TriState`] bucket: **Pass** (the
//! checker produced the expected DEFINITIVE verdict), **Fail** (the checker produced the
//! WRONG definitive verdict — a soundness bug; the gating test asserts there are none),
//! or **OutOfFragment** (the checker ABSTAINED with a typed reason — L1 refusal, a
//! dispatch guard, a deferred branch, or budget exhaustion). **An abstention is NEVER a
//! pass**: only a definitive expected verdict increments `pass`. Selection-level
//! exclusions (functional-syntax-only inputs — no `.ofn` parser in-tree; `owl:imports` —
//! no dereferencing in the harness; `test:status test:Rejected`) are OutOfScope rows,
//! outside all three verdict buckets.
//!
//! The dual-tagged (DIRECT ∧ RDF-BASED) tests keep their RDF-Based runs in the existing
//! `owl_suite` (RL) / `el_suite` lanes — one test may legitimately appear in both
//! tallies because the two runs test DIFFERENT semantics (record §4); nothing is
//! double-counted within a single lane.

use super::entail::{self, Row};
use crate::rdf::MiniGraph;
use oxrdf::{NamedOrBlankNode, Term};
use sparq_core::dict::{Dict, Id};
use sparq_reason_dl::check::{ConsistencyVerdict, DirectChecker, EntailmentVerdict, UnknownReason};
use sparq_reason_dl::profile::{profiles_from_extraction, Membership};
use sparq_reason_dl::tableau::Budget;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

const T: &str = "http://www.w3.org/2007/OWL/testOntology#";
const OWL_ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Pinned deterministic tableau budget for every `DirectChecker` run in this lane —
/// COUNT-based only (nodes / rule applications), so the floors are reproducible on any
/// machine (wall-clock budgets are banned by design). Pinned HERE — smaller than
/// `Budget::default()` — for two reasons: a future default change cannot silently move
/// the conformance floors, and the corpus contains adversarial search spaces (deep
/// `⊔`-backtracking) where a generous cap costs minutes per case in a debug-built
/// harness; a budget exhaustion is an HONEST abstention (`OutOfFragment` in the
/// tri-state), never a verdict. Raising the budget (graduating budget-abstained cases
/// to definitive verdicts) is a deliberate re-pin of the floors in a reviewed PR.
pub const DL_TABLEAU_MAX_NODES: usize = 2_000;
/// See [`DL_TABLEAU_MAX_NODES`] — the companion rule-application count cap.
pub const DL_TABLEAU_MAX_RULE_APPLICATIONS: usize = 20_000;

/// The pinned budget both lanes hand to `DirectChecker::with_budget`.
#[must_use]
pub fn pinned_budget() -> Budget {
    Budget {
        max_nodes: DL_TABLEAU_MAX_NODES,
        max_rule_applications: DL_TABLEAU_MAX_RULE_APPLICATIONS,
    }
}

/// Outcome of one selected check row. An abstention ([`TriState::OutOfFragment`]) is
/// NEVER a pass — the module-level invariant every mapping below enforces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TriState {
    /// The checker produced the expected definitive verdict.
    Pass,
    /// The checker produced the WRONG definitive verdict (payload: what was observed).
    /// A soundness bug — the gating test asserts the lane has none.
    Fail(String),
    /// The checker abstained, fail-closed (payload: the stable reason KIND — see
    /// `reason_kind` — so histograms are deterministic across runs).
    OutOfFragment(String),
}

/// Per-lane tally. `pass + fails.len() + out_of_fragment + out_of_scope` counts every
/// selected row exactly once (asserted by the gating test).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneTally {
    /// Rows where the checker produced the expected definitive verdict.
    pub pass: usize,
    /// Wrong definitive verdicts: `(row key, observed)` — must stay empty.
    pub fails: Vec<(String, String)>,
    /// Abstention histogram keyed by the stable reason kind.
    pub out_of_fragment: BTreeMap<String, usize>,
    /// Selection-level exclusion histogram (fs-only / imports / Rejected status).
    pub out_of_scope: BTreeMap<String, usize>,
}

impl LaneTally {
    /// Record one row's tri-state outcome under `key`.
    fn record(&mut self, key: &str, outcome: TriState) {
        match outcome {
            TriState::Pass => self.pass += 1,
            TriState::Fail(observed) => self.fails.push((key.to_string(), observed)),
            TriState::OutOfFragment(kind) => {
                *self.out_of_fragment.entry(kind).or_default() += 1;
            }
        }
    }

    /// Record one row excluded at selection time (never reached the checker).
    fn record_out_of_scope(&mut self, reason: &str) {
        *self.out_of_scope.entry(reason.to_string()).or_default() += 1;
    }

    /// Total abstained rows.
    #[must_use]
    pub fn out_of_fragment_total(&self) -> usize {
        self.out_of_fragment.values().sum()
    }

    /// Total selection-excluded rows.
    #[must_use]
    pub fn out_of_scope_total(&self) -> usize {
        self.out_of_scope.values().sum()
    }

    /// Every selected row, across all four buckets.
    #[must_use]
    pub fn total(&self) -> usize {
        self.pass + self.fails.len() + self.out_of_fragment_total() + self.out_of_scope_total()
    }
}

/// The full Direct-arm report both floors are pinned against.
#[derive(Clone, Debug, Default)]
pub struct DlReport {
    /// Profile-identification lane: one row per POSITIVE `test:profile` EL/QL/RL tag on
    /// a selected `ProfileIdentificationTest` (positive-only — module docs).
    pub profile: LaneTally,
    /// Reasoning lane, per declared check kind.
    pub consistency: LaneTally,
    /// See [`DlReport::consistency`].
    pub inconsistency: LaneTally,
    /// See [`DlReport::consistency`].
    pub positive_entailment: LaneTally,
    /// See [`DlReport::consistency`].
    pub negative_entailment: LaneTally,
    /// Selected `ProfileIdentificationTest` cases (DIRECT arm).
    pub profile_cases: usize,
    /// Selected reasoning cases (any of the four check kinds, DIRECT semantics).
    pub reasoning_cases: usize,
    /// Cases dropped for `test:status test:Rejected` (the export ships none — kept
    /// defensively per the bead contract).
    pub rejected_cases: usize,
}

impl DlReport {
    /// The reasoning lane's total pass count (the `DL_DIRECT_FLOOR` measurement).
    #[must_use]
    pub fn reasoning_pass(&self) -> usize {
        self.consistency.pass
            + self.inconsistency.pass
            + self.positive_entailment.pass
            + self.negative_entailment.pass
    }

    /// All wrong-definitive-verdict rows across both lanes (must stay empty).
    #[must_use]
    pub fn all_fails(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for lane in [
            &self.profile,
            &self.consistency,
            &self.inconsistency,
            &self.positive_entailment,
            &self.negative_entailment,
        ] {
            out.extend(lane.fails.iter().cloned());
        }
        out
    }

    /// Render the human-readable accounting (counts only — no timings; the two
    /// grep-able ratchet lines are printed by the gating test alongside its floors).
    #[must_use]
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut md = String::new();
        let _ = writeln!(
            md,
            "OWL 2 Direct-Semantics arm (scoped fragment — NOT full OWL 2 DL; sq-pbz04.4.5)"
        );
        let _ = writeln!(
            md,
            "  profile-identification (positive tags only): {} cases, {} assertion rows — pass {}, fail {}, abstained {}, out-of-scope {}",
            self.profile_cases,
            self.profile.total(),
            self.profile.pass,
            self.profile.fails.len(),
            self.profile.out_of_fragment_total(),
            self.profile.out_of_scope_total(),
        );
        for (kind, lane) in [
            ("consistency", &self.consistency),
            ("inconsistency", &self.inconsistency),
            ("positive-entailment", &self.positive_entailment),
            ("negative-entailment", &self.negative_entailment),
        ] {
            let _ = writeln!(
                md,
                "  {}: {} rows — pass {}, fail {}, abstained {}, out-of-scope {}",
                kind,
                lane.total(),
                lane.pass,
                lane.fails.len(),
                lane.out_of_fragment_total(),
                lane.out_of_scope_total(),
            );
        }
        let mut abstained: BTreeMap<String, usize> = BTreeMap::new();
        let mut oos: BTreeMap<String, usize> = BTreeMap::new();
        for lane in [
            &self.profile,
            &self.consistency,
            &self.inconsistency,
            &self.positive_entailment,
            &self.negative_entailment,
        ] {
            for (k, n) in &lane.out_of_fragment {
                *abstained.entry(k.clone()).or_default() += n;
            }
            for (k, n) in &lane.out_of_scope {
                *oos.entry(k.clone()).or_default() += n;
            }
        }
        if !abstained.is_empty() {
            let _ = writeln!(md, "  abstention kinds (fail-closed, NEVER counted as pass):");
            for (k, n) in &abstained {
                let _ = writeln!(md, "    - {} × {}", n, k);
            }
        }
        if !oos.is_empty() {
            let _ = writeln!(md, "  out-of-scope (selection exclusions):");
            for (k, n) in &oos {
                let _ = writeln!(md, "    - {} × {}", n, k);
            }
        }
        md
    }
}

/// Map a consistency outcome onto the expected-consistent tri-state.
#[must_use]
pub fn consistency_tri(verdict: &ConsistencyVerdict) -> TriState {
    match verdict {
        ConsistencyVerdict::Consistent => TriState::Pass,
        ConsistencyVerdict::Inconsistent => {
            TriState::Fail("wrongly judged inconsistent".to_string())
        }
        ConsistencyVerdict::Unknown(reason) => TriState::OutOfFragment(reason_kind(reason)),
    }
}

/// Map a consistency outcome onto the expected-INconsistent tri-state.
#[must_use]
pub fn inconsistency_tri(verdict: &ConsistencyVerdict) -> TriState {
    match verdict {
        ConsistencyVerdict::Inconsistent => TriState::Pass,
        ConsistencyVerdict::Consistent => {
            TriState::Fail("wrongly judged consistent".to_string())
        }
        ConsistencyVerdict::Unknown(reason) => TriState::OutOfFragment(reason_kind(reason)),
    }
}

/// Map an entailment outcome onto the expected-Entailed tri-state.
#[must_use]
pub fn positive_entailment_tri(verdict: &EntailmentVerdict) -> TriState {
    match verdict {
        EntailmentVerdict::Entailed => TriState::Pass,
        EntailmentVerdict::NotEntailed => {
            TriState::Fail("conclusion definitively not entailed".to_string())
        }
        EntailmentVerdict::Unknown(reason) => TriState::OutOfFragment(reason_kind(reason)),
    }
}

/// Map an entailment outcome onto the expected-NOT-Entailed tri-state.
#[must_use]
pub fn negative_entailment_tri(verdict: &EntailmentVerdict) -> TriState {
    match verdict {
        EntailmentVerdict::NotEntailed => TriState::Pass,
        EntailmentVerdict::Entailed => {
            TriState::Fail("non-conclusion wrongly entailed".to_string())
        }
        EntailmentVerdict::Unknown(reason) => TriState::OutOfFragment(reason_kind(reason)),
    }
}

/// Map an L2 membership verdict onto the expected tri-state (`expect_in` = positive tag
/// vs explicit negative assertion). `Membership::Unknown` is an L1 extraction refusal —
/// an abstention, never a pass in EITHER direction.
///
/// The corpus lane drives ONLY `expect_in = true` (module docs: the explicit-negative
/// direction was measured and NOT adopted — L2's `In` is fragment-grammar membership and
/// cannot refute full-profile membership); the `false` arm is the pure documented
/// mapping, kept for the day L2 grows the full-profile restrictions.
#[must_use]
pub fn membership_tri(membership: &Membership, expect_in: bool) -> TriState {
    match (membership, expect_in) {
        (Membership::In, true) | (Membership::NotIn(_), false) => TriState::Pass,
        (Membership::In, false) => {
            TriState::Fail("checker says In despite an explicit negative assertion".to_string())
        }
        (Membership::NotIn(reason), true) => {
            TriState::Fail(format!("checker says NotIn: {}", reason))
        }
        (Membership::Unknown(_), _) => {
            TriState::OutOfFragment("out-of-fragment (L1 extraction refused)".to_string())
        }
    }
}

/// Stable, deterministic abstention-kind label for a dispatch [`UnknownReason`] — the
/// variant, not the payload (payload strings can embed hash-ordered diagnostics).
fn reason_kind(reason: &UnknownReason) -> String {
    let kind = match reason {
        UnknownReason::OutOfFragment(_) => "out-of-fragment (L1 extraction refused)",
        UnknownReason::RlPr1Preconditions(_) => "RL PR1 precondition (usage-level punning)",
        UnknownReason::RlDivergenceGuard(_) => "RL divergence guard",
        UnknownReason::ElSkippedAxioms(_) => "EL skipped axioms",
        UnknownReason::ElUnappliedAxioms(_) => "EL unapplied axiom kinds",
        UnknownReason::ElTopGuard => "EL top guard",
        UnknownReason::QlConsistencyPending => "QL consistency pending (sq-pbz04.3.4)",
        UnknownReason::ResourceBudget(_) => "deterministic count budget exhausted",
        UnknownReason::UnencodedConclusion(_) => "unencoded conclusion-axiom kind",
        UnknownReason::ConclusionAnonymousIndividual(_) => {
            "conclusion anonymous individual (non-rollable existential shape)"
        }
        // `UnknownReason` is #[non_exhaustive]; a future variant is still an abstention.
        _ => "other typed abstention",
    };
    kind.to_string()
}

/// One selected test case, as read from the export.
struct Case {
    ident: String,
    checks: Vec<&'static str>,
    profile_test: bool,
    positive_profiles: Vec<String>,
    premise: Option<String>,
    input: Option<String>,
    conclusion: Option<String>,
    nonconclusion: Option<String>,
    imports: bool,
}

/// Run the whole Direct arm over the export text (`tests/w3c/owl2/all.rdf`).
///
/// # Errors
/// Returns `Err` only on an export-level failure (the RDF/XML export itself failed to
/// parse) — per-case problems are accounted in the report, never dropped.
pub fn run_direct_arm(export_text: &str) -> Result<DlReport, String> {
    let fixed = fix_doctype_quotes(export_text);
    let parser = oxrdfxml::RdfXmlParser::new()
        .with_base_iri("http://owl.semanticweb.org/exports/all.rdf")
        .map_err(|e| e.to_string())?;
    let mut triples = Vec::new();
    for t in parser.for_slice(fixed.as_bytes()) {
        triples.push(t.map_err(|e| format!("all.rdf: {}", e))?);
    }
    let g = MiniGraph { triples };

    let mut report = DlReport::default();
    for case_node in g.subjects_with_type(&format!("{}TestCase", T)) {
        let iri_objs = |p: &str| -> Vec<String> {
            g.objects(&case_node, &format!("{}{}", T, p))
                .into_iter()
                .filter_map(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .collect()
        };
        // The DIRECT arm: only tests sanctioned under the Direct semantics. The
        // RDF-Based runs of dual-tagged tests stay in the owl_suite / el_suite lanes.
        if !iri_objs("semantics").iter().any(|s| s == &format!("{}DIRECT", T)) {
            continue;
        }
        let status = g
            .object(&case_node, &format!("{}status", T))
            .and_then(|t| match t {
                Term::NamedNode(n) => n.as_str().strip_prefix(T).map(|s| s.to_string()),
                _ => None,
            });
        // Bead contract: `test:status test:Rejected` is EXCLUDED from the arm entirely
        // (the pinned export ships none — the export query already filters them).
        // Proposed / Extracredit / untagged cases RUN, honestly: this is an extension
        // ratchet over what the checker computes, not an Approved-only conformance claim.
        if status.as_deref() == Some("Rejected") {
            report.rejected_cases += 1;
            continue;
        }

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
        let profile_test = has("ProfileIdentificationTest");
        if checks.is_empty() && !profile_test {
            continue;
        }

        let lit = |p: &str| -> Option<String> { g.str_object(&case_node, &format!("{}{}", T, p)) };
        let case_iri = match &case_node {
            NamedOrBlankNode::NamedNode(n) => n.as_str().to_string(),
            NamedOrBlankNode::BlankNode(b) => format!("_:{}", b.as_str()),
        };
        let known = |p: &String| ["EL", "QL", "RL"].iter().any(|k| p == &format!("{}{}", T, k));
        let profile_name = |p: &str| p.strip_prefix(T).unwrap_or(p).to_string();
        let case = Case {
            ident: lit("identifier").unwrap_or_else(|| {
                case_iri.rsplit('/').next().unwrap_or(&case_iri).to_string()
            }),
            checks,
            profile_test,
            positive_profiles: iri_objs("profile")
                .iter()
                .filter(|p| known(p))
                .map(|p| profile_name(p))
                .collect(),
            premise: lit("rdfXmlPremiseOntology"),
            input: lit("rdfXmlInputOntology"),
            conclusion: lit("rdfXmlConclusionOntology"),
            nonconclusion: lit("rdfXmlNonConclusionOntology"),
            imports: g.object(&case_node, &format!("{}importedOntology", T)).is_some()
                || g.object(&case_node, &format!("{}importedOntologyIRI", T)).is_some(),
        };
        run_case(&case, &mut report);
    }
    Ok(report)
}

/// Run both lanes' checks for one selected case.
fn run_case(case: &Case, report: &mut DlReport) {
    if case.profile_test {
        run_profile_lane(case, report);
    }
    if !case.checks.is_empty() {
        run_reasoning_lane(case, report);
    }
}

/// The profile-identification lane — POSITIVE `test:profile` tags only (module docs).
fn run_profile_lane(case: &Case, report: &mut DlReport) {
    report.profile_cases += 1;
    let assertions = &case.positive_profiles;
    if assertions.is_empty() {
        // No positive EL/QL/RL tag: either a species-only case (test:species DL/FULL —
        // the design record's deferred species check) or an explicit-negatives-only
        // case (the direction this lane measured and does not check — module docs).
        report
            .profile
            .record_out_of_scope("no positive EL/QL/RL tag (species check deferred; explicit-negative direction not checked)");
        return;
    }
    if case.imports {
        for _ in assertions {
            report
                .profile
                .record_out_of_scope("uses owl:imports (no dereferencing in the harness)");
        }
        return;
    }
    let Some(input_xml) = case.input.clone().or_else(|| case.premise.clone()) else {
        for _ in assertions {
            report
                .profile
                .record_out_of_scope("input only available in functional syntax");
        }
        return;
    };
    let base = format!("http://owl.semanticweb.org/id/{}", case.ident);
    let profile_set = catch_unwind(AssertUnwindSafe(
        || -> Result<sparq_reason_dl::profile::ProfileSet, String> {
            let rows = parse_ontology(&input_xml, &base)?;
            let (dict, ids) = intern_rows(&rows);
            Ok(profiles_from_extraction(&sparq_reason_dl::extract(&dict, &ids)))
        },
    ));
    let profile_set = match profile_set {
        Ok(Ok(ps)) => ps,
        Ok(Err(e)) => {
            for name in assertions {
                let key = format!("owl2-dl/profile-{}: {}", name, case.ident);
                report
                    .profile
                    .record(&key, TriState::Fail(format!("input RDF/XML parse error: {}", e)));
            }
            return;
        }
        Err(_) => {
            for name in assertions {
                let key = format!("owl2-dl/profile-{}: {}", name, case.ident);
                report
                    .profile
                    .record(&key, TriState::Fail("profile pipeline panicked".to_string()));
            }
            return;
        }
    };
    for name in assertions {
        let membership = match name.as_str() {
            "EL" => &profile_set.el,
            "QL" => &profile_set.ql,
            _ => &profile_set.rl,
        };
        let key = format!("owl2-dl/profile-{}: {}", name, case.ident);
        report.profile.record(&key, membership_tri(membership, true));
    }
}

/// The Direct consistency / entailment lane, through the L4 `DirectChecker`.
fn run_reasoning_lane(case: &Case, report: &mut DlReport) {
    report.reasoning_cases += 1;
    // Record OutOfScope on every declared check of this case.
    let all_out_of_scope = |report: &mut DlReport, reason: &str| {
        for kind in &case.checks {
            record_kind_oos(report, kind, reason);
        }
    };
    if case.imports {
        all_out_of_scope(report, "uses owl:imports (no dereferencing in the harness)");
        return;
    }
    let Some(premise_xml) = case.premise.clone().or_else(|| case.input.clone()) else {
        all_out_of_scope(report, "premise only available in functional syntax");
        return;
    };
    let base = format!("http://owl.semanticweb.org/id/{}", case.ident);
    let premise = match parse_ontology(&premise_xml, &base) {
        Ok(rows) => rows,
        Err(e) => {
            let observed = format!("premise RDF/XML parse error: {}", e);
            for kind in &case.checks {
                record_kind(report, kind, &case.ident, TriState::Fail(observed.clone()));
            }
            return;
        }
    };
    let checker = DirectChecker::with_budget(pinned_budget());
    for kind in &case.checks {
        let outcome = match *kind {
            "consistency" | "inconsistency" => {
                let verdict = catch_unwind(AssertUnwindSafe(|| {
                    let (mut dict, ids) = intern_rows(&premise);
                    checker.consistency(&mut dict, &ids).verdict
                }));
                match verdict {
                    Ok(v) if *kind == "consistency" => consistency_tri(&v),
                    Ok(v) => inconsistency_tri(&v),
                    Err(_) => TriState::Fail("checker panicked".to_string()),
                }
            }
            "positive-entailment" | "negative-entailment" => {
                let (doc, expect_positive) = if *kind == "positive-entailment" {
                    (&case.conclusion, true)
                } else {
                    (&case.nonconclusion, false)
                };
                let Some(xml) = doc else {
                    let side = if expect_positive { "conclusion" } else { "non-conclusion" };
                    record_kind_oos(
                        report,
                        kind,
                        &format!("{} only available in functional syntax", side),
                    );
                    continue;
                };
                match parse_ontology(xml, &base) {
                    Err(e) => TriState::Fail(format!("conclusion RDF/XML parse error: {}", e)),
                    Ok(conclusion_rows) => {
                        let verdict = catch_unwind(AssertUnwindSafe(|| {
                            let (mut dict, prem_ids) = intern_rows(&premise);
                            let concl_ids: Vec<[Id; 3]> = conclusion_rows
                                .iter()
                                .map(|[s, p, o]| {
                                    [dict.intern(s), dict.intern(p), dict.intern(o)]
                                })
                                .collect();
                            checker.entailment(&mut dict, &prem_ids, &concl_ids).verdict
                        }));
                        match verdict {
                            Ok(v) if expect_positive => positive_entailment_tri(&v),
                            Ok(v) => negative_entailment_tri(&v),
                            Err(_) => TriState::Fail("checker panicked".to_string()),
                        }
                    }
                }
            }
            _ => unreachable!(),
        };
        record_kind(report, kind, &case.ident, outcome);
    }
}

/// Record one row outcome into the lane for `kind`.
fn record_kind(report: &mut DlReport, kind: &str, ident: &str, outcome: TriState) {
    let key = format!("owl2-dl/{}: {}", kind, ident);
    match kind {
        "consistency" => report.consistency.record(&key, outcome),
        "inconsistency" => report.inconsistency.record(&key, outcome),
        "positive-entailment" => report.positive_entailment.record(&key, outcome),
        _ => report.negative_entailment.record(&key, outcome),
    }
}

/// Record one selection-excluded row into the lane for `kind`.
fn record_kind_oos(report: &mut DlReport, kind: &str, reason: &str) {
    match kind {
        "consistency" => report.consistency.record_out_of_scope(reason),
        "inconsistency" => report.inconsistency.record_out_of_scope(reason),
        "positive-entailment" => report.positive_entailment.record_out_of_scope(reason),
        _ => report.negative_entailment.record_out_of_scope(reason),
    }
}

/// Intern term rows into a fresh dict (one dict per checker run — the RL branch
/// materializes into it, so runs must not share state).
fn intern_rows(rows: &[Row]) -> (Dict, Vec<[Id; 3]>) {
    let mut dict = Dict::new();
    let ids = rows
        .iter()
        .map(|[s, p, o]| [dict.intern(s), dict.intern(p), dict.intern(o)])
        .collect();
    (dict, ids)
}

/// Parses an inline RDF/XML ontology literal, dropping the ontology header
/// (`?x rdf:type owl:Ontology` typings and `owl:imports` edges) — the harness compares
/// axioms, not headers, mirroring the official OWLWG harness (and the RL / EL lanes).
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

/// The export's internal DTD uses single-quoted ENTITY values, which oxrdfxml rejects;
/// rewrite just the DOCTYPE block to double quotes (same fix the RL / EL lanes apply).
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- the fail-closed tri-state mappings (the load-bearing invariant) -------------

    #[test]
    fn abstention_is_never_a_pass() {
        let unknown = ConsistencyVerdict::Unknown(UnknownReason::QlConsistencyPending);
        assert!(matches!(consistency_tri(&unknown), TriState::OutOfFragment(_)));
        assert!(matches!(inconsistency_tri(&unknown), TriState::OutOfFragment(_)));
        let eu = EntailmentVerdict::Unknown(UnknownReason::UnencodedConclusion("x".into()));
        assert!(matches!(positive_entailment_tri(&eu), TriState::OutOfFragment(_)));
        assert!(matches!(negative_entailment_tri(&eu), TriState::OutOfFragment(_)));
        let mu = Membership::Unknown("refused".into());
        assert!(matches!(membership_tri(&mu, true), TriState::OutOfFragment(_)));
        assert!(matches!(membership_tri(&mu, false), TriState::OutOfFragment(_)));
    }

    #[test]
    fn definitive_verdicts_map_both_ways() {
        assert_eq!(consistency_tri(&ConsistencyVerdict::Consistent), TriState::Pass);
        assert!(matches!(
            consistency_tri(&ConsistencyVerdict::Inconsistent),
            TriState::Fail(_)
        ));
        assert_eq!(inconsistency_tri(&ConsistencyVerdict::Inconsistent), TriState::Pass);
        assert!(matches!(
            inconsistency_tri(&ConsistencyVerdict::Consistent),
            TriState::Fail(_)
        ));
        assert_eq!(positive_entailment_tri(&EntailmentVerdict::Entailed), TriState::Pass);
        assert!(matches!(
            positive_entailment_tri(&EntailmentVerdict::NotEntailed),
            TriState::Fail(_)
        ));
        assert_eq!(
            negative_entailment_tri(&EntailmentVerdict::NotEntailed),
            TriState::Pass
        );
        assert!(matches!(
            negative_entailment_tri(&EntailmentVerdict::Entailed),
            TriState::Fail(_)
        ));
        assert_eq!(membership_tri(&Membership::In, true), TriState::Pass);
        assert!(matches!(membership_tri(&Membership::In, false), TriState::Fail(_)));
        assert_eq!(membership_tri(&Membership::NotIn("r".into()), false), TriState::Pass);
        assert!(matches!(
            membership_tri(&Membership::NotIn("r".into()), true),
            TriState::Fail(_)
        ));
    }

    #[test]
    fn lane_tally_accounting_is_total() {
        let mut lane = LaneTally::default();
        lane.record("a", TriState::Pass);
        lane.record("b", TriState::Fail("wrong".into()));
        lane.record("c", TriState::OutOfFragment("kind".into()));
        lane.record_out_of_scope("fs-only");
        assert_eq!(lane.pass, 1);
        assert_eq!(lane.fails.len(), 1);
        assert_eq!(lane.out_of_fragment_total(), 1);
        assert_eq!(lane.out_of_scope_total(), 1);
        assert_eq!(lane.total(), 4);
    }

    #[test]
    fn pinned_budget_is_the_named_counts() {
        let b = pinned_budget();
        assert_eq!(b.max_nodes, DL_TABLEAU_MAX_NODES);
        assert_eq!(b.max_rule_applications, DL_TABLEAU_MAX_RULE_APPLICATIONS);
    }

    // ---- end-to-end over a tiny synthetic export --------------------------------------

    /// A minimal all.rdf-shaped export: one in-fragment consistency case (Pass), one
    /// out-of-fragment consistency case (abstains — the fail-closed path), one
    /// inconsistency case (Pass via the ALCH tableau), and one profile case with a
    /// positive RL tag + an explicit negative EL assertion.
    const MINI_EXPORT: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:test="http://www.w3.org/2007/OWL/testOntology#">
  <test:TestCase rdf:about="http://ex/case-consistent">
    <rdf:type rdf:resource="http://www.w3.org/2007/OWL/testOntology#ConsistencyTest"/>
    <test:identifier rdf:datatype="http://www.w3.org/2001/XMLSchema#string">case-consistent</test:identifier>
    <test:status rdf:resource="http://www.w3.org/2007/OWL/testOntology#Approved"/>
    <test:semantics rdf:resource="http://www.w3.org/2007/OWL/testOntology#DIRECT"/>
    <test:rdfXmlPremiseOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;owl:Class rdf:about="A"&gt;&lt;rdfs:subClassOf&gt;&lt;owl:Class rdf:about="B"/&gt;&lt;/rdfs:subClassOf&gt;&lt;/owl:Class&gt;&lt;/rdf:RDF&gt;</test:rdfXmlPremiseOntology>
  </test:TestCase>
  <test:TestCase rdf:about="http://ex/case-out-of-fragment">
    <rdf:type rdf:resource="http://www.w3.org/2007/OWL/testOntology#ConsistencyTest"/>
    <test:identifier rdf:datatype="http://www.w3.org/2001/XMLSchema#string">case-out-of-fragment</test:identifier>
    <test:status rdf:resource="http://www.w3.org/2007/OWL/testOntology#Approved"/>
    <test:semantics rdf:resource="http://www.w3.org/2007/OWL/testOntology#DIRECT"/>
    <test:rdfXmlPremiseOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;rdf:Description rdf:about="a"&gt;&lt;owl:sameAs rdf:resource="b"/&gt;&lt;/rdf:Description&gt;&lt;/rdf:RDF&gt;</test:rdfXmlPremiseOntology>
  </test:TestCase>
  <test:TestCase rdf:about="http://ex/case-inconsistent">
    <rdf:type rdf:resource="http://www.w3.org/2007/OWL/testOntology#InconsistencyTest"/>
    <test:identifier rdf:datatype="http://www.w3.org/2001/XMLSchema#string">case-inconsistent</test:identifier>
    <test:status rdf:resource="http://www.w3.org/2007/OWL/testOntology#Approved"/>
    <test:semantics rdf:resource="http://www.w3.org/2007/OWL/testOntology#DIRECT"/>
    <test:rdfXmlPremiseOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;owl:Class rdf:about="A"&gt;&lt;owl:equivalentClass&gt;&lt;owl:Class&gt;&lt;owl:complementOf rdf:resource="A"/&gt;&lt;/owl:Class&gt;&lt;/owl:equivalentClass&gt;&lt;/owl:Class&gt;&lt;/rdf:RDF&gt;</test:rdfXmlPremiseOntology>
  </test:TestCase>
  <test:TestCase rdf:about="http://ex/case-profile">
    <rdf:type rdf:resource="http://www.w3.org/2007/OWL/testOntology#ProfileIdentificationTest"/>
    <test:identifier rdf:datatype="http://www.w3.org/2001/XMLSchema#string">case-profile</test:identifier>
    <test:status rdf:resource="http://www.w3.org/2007/OWL/testOntology#Approved"/>
    <test:semantics rdf:resource="http://www.w3.org/2007/OWL/testOntology#DIRECT"/>
    <test:profile rdf:resource="http://www.w3.org/2007/OWL/testOntology#RL"/>
    <test:rdfXmlInputOntology rdf:datatype="http://www.w3.org/2001/XMLSchema#string">&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:owl="http://www.w3.org/2002/07/owl#" xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="http://example.org/"&gt;&lt;owl:Ontology/&gt;&lt;owl:Class rdf:about="A"&gt;&lt;rdfs:subClassOf&gt;&lt;owl:Restriction&gt;&lt;owl:onProperty&gt;&lt;owl:ObjectProperty rdf:about="r"/&gt;&lt;/owl:onProperty&gt;&lt;owl:allValuesFrom rdf:resource="B"/&gt;&lt;/owl:Restriction&gt;&lt;/rdfs:subClassOf&gt;&lt;/owl:Class&gt;&lt;/rdf:RDF&gt;</test:rdfXmlInputOntology>
  </test:TestCase>
  <owl:NegativePropertyAssertion>
    <owl:sourceIndividual rdf:resource="http://ex/case-profile"/>
    <owl:assertionProperty rdf:resource="http://www.w3.org/2007/OWL/testOntology#profile"/>
    <owl:targetIndividual rdf:resource="http://www.w3.org/2007/OWL/testOntology#EL"/>
  </owl:NegativePropertyAssertion>
</rdf:RDF>"#;

    #[test]
    fn mini_export_end_to_end() {
        let report = run_direct_arm(MINI_EXPORT).expect("mini export parses");
        // Reasoning lane: the in-fragment consistent case passes; the owl:sameAs case
        // abstains (L1 refuses — fail-closed, NOT a pass); the ¬A ≡ A case is
        // inconsistent via the real ALCH tableau.
        assert_eq!(report.reasoning_cases, 3);
        assert_eq!(report.consistency.pass, 1);
        assert_eq!(report.consistency.out_of_fragment_total(), 1);
        assert_eq!(report.inconsistency.pass, 1);
        assert!(report.all_fails().is_empty(), "{:?}", report.all_fails());
        assert_eq!(report.reasoning_pass(), 2);
        // Profile lane (positive tags only): the ∀-restriction TBox is RL, so the
        // positive RL tag passes; the mini export's explicit owl:NegativePropertyAssertion
        // (NOT in EL) is deliberately NOT checked (module docs) — exactly one row.
        assert_eq!(report.profile_cases, 1);
        assert_eq!(report.profile.total(), 1);
        assert_eq!(report.profile.pass, 1);
        // The render is counts-only and names the honest scope label.
        let md = report.render();
        assert!(md.contains("NOT full OWL 2 DL"));
        assert!(md.contains("NEVER counted as pass"));
    }

    #[test]
    fn rejected_status_is_excluded() {
        let rejected = MINI_EXPORT.replace(
            "http://www.w3.org/2007/OWL/testOntology#Approved",
            "http://www.w3.org/2007/OWL/testOntology#Rejected",
        );
        let report = run_direct_arm(&rejected).expect("mini export parses");
        assert_eq!(report.rejected_cases, 4);
        assert_eq!(report.reasoning_cases, 0);
        assert_eq!(report.profile_cases, 0);
        assert_eq!(report.reasoning_pass() + report.profile.pass, 0);
    }

    #[test]
    fn fs_only_and_imports_are_out_of_scope_not_passes() {
        // Strip the premise literal from the consistent case → fs-only OutOfScope.
        let fs_only = MINI_EXPORT.replacen("test:rdfXmlPremiseOntology", "test:fsPremiseOntology", 2);
        let report = run_direct_arm(&fs_only).expect("mini export parses");
        assert_eq!(report.consistency.out_of_scope_total(), 1);
        assert_eq!(report.consistency.pass, 0);
    }
}
