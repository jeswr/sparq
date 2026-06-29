//! SPARQL 1.1 entailment-regime evaluation (`sparql11/entailment` in the same
//! rdf-tests clone the SPARQL harness uses): each test names the regimes
//! under which its expected answers hold (`sd:entailmentRegime` — RDF / RDFS /
//! D / OWL-RDF-Based / OWL-Direct / RIF). The runner picks the strongest
//! regime sparq-reason materializes:
//!
//! - **RDFS** → `materialize_rdfs` (+ rdfD2 predicate typing),
//! - **RDF** (only) → rdfD2 predicate typing,
//! - **OWL-RDF-Based** (without RDF/RDFS tag) → `materialize_owl_rl` — the RL
//!   rules are a sound subset of the OWL 2 RDF-based semantics, so missing
//!   answers are honest fails, not crashes,
//! - **D / OWL-Direct / RIF only** → out of scope with the reason.
//!
//! The query then runs over the materialized closure through the SAME
//! evaluation+comparison path as the W3C SPARQL harness
//! (`run::run_query_test_on` with a data override). Generalized triples the
//! materializer may produce (literal subjects, from range typing) cannot be
//! answers to any SPARQL query and are filtered out of the closure dataset.

use super::report::{Outcome, TestResult};
use crate::manifest::{self, EntryKind, TestEntry};
use crate::run::{self, EntailmentAnswerFilter, Status};
use oxrdf::vocab::rdf;
use oxrdf::Term;
use rustc_hash::FxHashSet;
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use spargebra::{Query, SparqlParser};
use sparq_core::dict::Dict;
use std::path::Path;

pub fn notes() -> Vec<String> {
    vec![
        "Source: `sparql11/entailment` from the pinned rdf-tests clone. Each test runs as \
         query-over-materialized-closure through the same evaluation/comparison machinery as \
         the gating SPARQL harness; the regime materialized is the strongest the reasoner \
         supports of the test's `sd:entailmentRegime` set (RDFS ⊃ RDF; OWL-RDF-Based via the \
         OWL RL rules, a sound subset — its incompleteness shows up as listed fails). The \
         OwlRl closure adds a harness-side eq-ref layer (OWL 2 Profiles §4.3 Table 4: \
         reflexive owl:sameAs for every closure term — omitted by the production \
         materializer as store bloat). The regimes' ANSWER RESTRICTION is applied to engine \
         solutions before comparison (SPARQL 1.1 Entailment Regimes): (C1/skolemization, \
         §2/§3.1) bindings to blank nodes not in the queried graph — i.e. introduced by the \
         saturation — are never answers; and for tests whose expectations are sanctioned \
         under OWL-Direct (§7), variables in class/property-NAME positions cannot bind to \
         anonymous class expressions (a bnode is not a name). \
         D-only / OWL-Direct-only / RIF tests are out of scope with their reason."
            .to_string(),
    ]
}

pub fn run_suite(rdf_tests_root: &Path, out: &mut Vec<TestResult>) -> Result<(), String> {
    let suites_root = rdf_tests_root
        .join("sparql")
        .canonicalize()
        .map_err(|e| format!("rdf-tests sparql dir: {e}"))?;
    let manifest_path = suites_root.join("sparql11/entailment/manifest.ttl");
    let mut entries: Vec<TestEntry> = Vec::new();
    manifest::collect(&manifest_path, &suites_root, &mut entries)?;

    for entry in &entries {
        if entry.kind != EntryKind::QueryEval {
            out.push(result(entry, Outcome::OutOfScope("not a QueryEvaluationTest".into())));
            continue;
        }
        let regimes: FxHashSet<&str> = entry
            .action
            .entailment_regimes
            .iter()
            .filter_map(|r| r.rsplit('/').next())
            .collect();
        let profile = if regimes.contains("RDFS") {
            Profile::Rdfs
        } else if regimes.contains("RDF") {
            Profile::Rdf
        } else if regimes.contains("OWL-RDF-Based") {
            Profile::OwlRl
        } else if d_regime_profile(&regimes).is_some() {
            // [OPUS-4.8] sq-e5atd — a D-only (datatype / value-space) test: route
            // it through the opt-in `Profile::D` materializer when the `d-entail`
            // feature is ON. When OFF, `d_regime_profile` returns None and the test
            // stays OutOfScope (below), so the default inference binary's ratchet is
            // unchanged — the lean opt-in posture.
            d_regime_profile(&regimes).unwrap()
        } else {
            let mut rs: Vec<&str> = regimes.iter().copied().collect();
            rs.sort_unstable();
            out.push(result(
                entry,
                Outcome::OutOfScope(format!(
                    "entailment regime(s) {} not supported (no materialization mapping)",
                    rs.join("+")
                )),
            ));
            continue;
        };
        let direct_sanctioned = regimes.contains("OWL-Direct");
        out.push(result(entry, run_one(entry, profile, direct_sanctioned)));
    }
    Ok(())
}

/// [OPUS-4.8] sq-e5atd — run ONLY the genuinely D-only (`sd:entailmentRegime
/// ent:D` WITHOUT any stronger RDFS/RDF/OWL-RDF-Based regime) tests of the
/// `sparql11/entailment` suite, through the real `Profile::D` materializer. This
/// is the DEDICATED D-regime lane the `tests/d_entail_suite.rs` ratchet asserts a
/// floor over; it is deliberately a SUBSET of `run_suite` (the stronger-regime
/// tests are exercised under their own regime there). Opt-in, `d-entail`.
#[cfg(feature = "d-entail")]
pub fn run_d_regime_suite(rdf_tests_root: &Path, out: &mut Vec<TestResult>) -> Result<(), String> {
    let suites_root = rdf_tests_root
        .join("sparql")
        .canonicalize()
        .map_err(|e| format!("rdf-tests sparql dir: {e}"))?;
    let manifest_path = suites_root.join("sparql11/entailment/manifest.ttl");
    let mut entries: Vec<TestEntry> = Vec::new();
    manifest::collect(&manifest_path, &suites_root, &mut entries)?;
    for entry in &entries {
        if entry.kind != EntryKind::QueryEval {
            continue;
        }
        let regimes: FxHashSet<&str> = entry
            .action
            .entailment_regimes
            .iter()
            .filter_map(|r| r.rsplit('/').next())
            .collect();
        // A test is D-ONLY when its only materializable regime is D — none of the
        // stronger regimes (which `run_suite` picks first) apply.
        let stronger = regimes.contains("RDFS")
            || regimes.contains("RDF")
            || regimes.contains("OWL-RDF-Based");
        if stronger || !regimes.contains("D") {
            continue;
        }
        let direct_sanctioned = regimes.contains("OWL-Direct");
        out.push(result(entry, run_one(entry, Profile::D, direct_sanctioned)));
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Profile {
    Rdf,
    Rdfs,
    OwlRl,
    /// [OPUS-4.8] sq-e5atd — D-entailment (datatype / value-space). Opt-in,
    /// `d-entail`. Only reached for a test whose regime set is D WITHOUT any of
    /// RDFS/RDF/OWL-RDF-Based (those stronger regimes subsume D and are picked
    /// first), so this arm graduates the genuinely D-only tests.
    #[cfg(feature = "d-entail")]
    D,
}

/// The `Profile::D` for a regime set IFF the `d-entail` feature is ON and the set
/// contains the D regime; `None` otherwise (so the caller keeps the test
/// OutOfScope when the feature is off — the default binary is unchanged).
#[cfg(feature = "d-entail")]
fn d_regime_profile(regimes: &FxHashSet<&str>) -> Option<Profile> {
    regimes.contains("D").then_some(Profile::D)
}

/// Feature-OFF stub: D is never a supported profile, so a D-only test stays
/// OutOfScope. (Kept as a function so the call site is identical in both states.)
#[cfg(not(feature = "d-entail"))]
fn d_regime_profile(_regimes: &FxHashSet<&str>) -> Option<Profile> {
    None
}

fn result(entry: &TestEntry, outcome: Outcome) -> TestResult {
    TestResult {
        suite: "sparql11/entailment".into(),
        name: entry.name.clone(),
        outcome,
    }
}

fn run_one(entry: &TestEntry, profile: Profile, direct_sanctioned: bool) -> Outcome {
    // [OPUS-4.8] sq-oy1f — the `sparql11/entailment` manifest at the pinned
    // rdf-tests revision contains NO `qt:graphData` entries (every entailment test
    // uses a single default-graph `qt:data` dataset), so this guard is currently
    // unreachable: there are zero named-graph entailment cases to graduate. It is
    // RETAINED as a fail-closed safeguard — should a future suite bump add a
    // named-graph entailment test, it would be reported OutOfScope (honest) rather
    // than silently materialized through the default-graph-only path below, which
    // would give a WRONG closure (the named-graph dataset would need per-graph
    // entailment semantics in sparq-reason, not just harness wiring). Wiring that
    // properly is tracked as a deferred bead; do NOT drop the guard to "make it run"
    // until the reasoner models named-graph materialization.
    if !entry.action.graph_data.is_empty() {
        return Outcome::OutOfScope("named-graph entailment dataset not wired".into());
    }
    // Load and materialize the default-graph data. The blank-node labels of
    // the ORIGINAL data are recorded before materialization: condition (C1)'s
    // Skolemization function is defined exactly for the blank nodes of the
    // queried graph, so they delimit the bnode bindings that can be answers.
    let mut dict = Dict::new();
    let mut ids: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
    let mut data_bnodes: FxHashSet<String> = FxHashSet::default();
    for d in &entry.action.data {
        match crate::rdf::parse_file(d) {
            Ok(triples) => {
                for t in triples {
                    let row = [
                        Term::from(t.subject),
                        Term::NamedNode(t.predicate),
                        t.object,
                    ];
                    for term in &row {
                        if let Term::BlankNode(b) = term {
                            data_bnodes.insert(b.as_str().to_string());
                        }
                    }
                    ids.push([dict.intern(&row[0]), dict.intern(&row[1]), dict.intern(&row[2])]);
                }
            }
            Err(e) => return Outcome::Fail(format!("data parse error: {e}")),
        }
    }
    match profile {
        Profile::Rdfs => {
            add_declared_reflexives(&mut dict, &mut ids, false);
            sparq_reason::materialize(sparq_reason::Profile::Rdfs, &mut dict, &mut ids);
            add_rdfd2(&mut dict, &mut ids);
        }
        Profile::Rdf => add_rdfd2(&mut dict, &mut ids),
        Profile::OwlRl => {
            add_declared_reflexives(&mut dict, &mut ids, true);
            sparq_reason::materialize(sparq_reason::Profile::OwlRl, &mut dict, &mut ids);
            add_eq_ref(&mut dict, &mut ids);
        }
        // [OPUS-4.8] sq-e5atd — D-entailment: the rdfD1 datatype-typing closure
        // under the STANDARD recognized datatype map (this suite's D tests do not
        // restrict D via a per-test set). The emitted typing triples are
        // GENERALIZED (literal subject `"l"^^d rdf:type d`); the N-Triples
        // serialization below DROPS literal-subject rows (they can never be a
        // SPARQL answer — the regime answer restriction, also why `d-ent-01`
        // correctly returns NO rows), so D-entailment adds no bindable triple but
        // is computed through the real `Profile::D` materializer.
        #[cfg(feature = "d-entail")]
        Profile::D => {
            sparq_reason::materialize(sparq_reason::Profile::D, &mut dict, &mut ids);
        }
    }

    // Serialize the closure as N-Triples (default graph). Generalized triples
    // (literal subjects from rdfs3 range typing) cannot appear in any SPARQL
    // answer and the loader would reject them.
    let mut nquads = String::new();
    let mut seen: FxHashSet<[sparq_core::dict::Id; 3]> = FxHashSet::default();
    for t in &ids {
        if !seen.insert(*t) {
            continue;
        }
        let (s, p, o) = (dict.term(t[0]), dict.term(t[1]), dict.term(t[2]));
        if matches!(s, Term::Literal(_)) {
            continue;
        }
        nquads.push_str(&format!("{s} {p} {o} .\n"));
    }

    // The regimes' answer restriction on the closure's solutions:
    //  - every regime (C1, Entailment Regimes §2/§3.1): bindings to blank
    //    nodes NOT in the queried graph (saturation-introduced) are never
    //    answers — sk is undefined for them, so sk(P(BGP)) is not ground;
    //  - tests whose expected answers are also sanctioned under OWL-Direct
    //    (§7): a variable in a class/property-NAME position cannot bind to an
    //    anonymous class expression (a blank node is not a name).
    let filter = EntailmentAnswerFilter {
        data_bnodes,
        name_position_vars: if direct_sanctioned {
            name_position_vars(entry)
        } else {
            FxHashSet::default()
        },
    };

    match run::run_query_test_filtered(entry, Some(nquads), Some(&filter)) {
        Status::Pass => Outcome::Pass,
        Status::Fail(e) => Outcome::Fail(e),
        Status::Skip(e) => Outcome::OutOfScope(e),
    }
}

/// Variables of the query's BGPs that stand in class-name or property-name
/// positions (Entailment Regimes §7: under the OWL 2 Direct Semantics regime
/// variables stand "in place of class names, object property names, datatype
/// property names, individual names, or literals"; the extended grammar of
/// §7.1.2 substitutes them for the *name* productions, so an anonymous class
/// expression — a blank node — is not a legal binding for them). Detection is
/// positional over the schema vocabulary; property paths don't occur in the
/// entailment suite's queries.
fn name_position_vars(entry: &TestEntry) -> FxHashSet<String> {
    let mut vars = FxHashSet::default();
    let Some(query_path) = &entry.action.query else {
        return vars;
    };
    let Ok(query_text) = std::fs::read_to_string(query_path) else {
        return vars;
    };
    let base = crate::rdf::file_iri(query_path);
    let Ok(parser) = SparqlParser::new().with_base_iri(&base) else {
        return vars;
    };
    let pattern = match parser.parse_query(&query_text) {
        Ok(Query::Select { pattern, .. }) | Ok(Query::Ask { pattern, .. }) => pattern,
        _ => return vars,
    };
    collect_name_position_vars(&pattern, &mut vars);
    vars
}

const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const OWL: &str = "http://www.w3.org/2002/07/owl#";

fn collect_name_position_vars(p: &GraphPattern, vars: &mut FxHashSet<String>) {
    match p {
        GraphPattern::Bgp { patterns } => {
            for tp in patterns {
                triple_name_positions(tp, vars);
            }
        }
        GraphPattern::Path { .. } | GraphPattern::Values { .. } => {}
        GraphPattern::Join { left, right }
        | GraphPattern::LeftJoin { left, right, .. }
        | GraphPattern::Lateral { left, right }
        | GraphPattern::Union { left, right }
        | GraphPattern::Minus { left, right } => {
            collect_name_position_vars(left, vars);
            collect_name_position_vars(right, vars);
        }
        GraphPattern::Filter { inner, .. }
        | GraphPattern::Graph { inner, .. }
        | GraphPattern::Extend { inner, .. }
        | GraphPattern::OrderBy { inner, .. }
        | GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. }
        | GraphPattern::Group { inner, .. }
        | GraphPattern::Service { inner, .. } => collect_name_position_vars(inner, vars),
    }
}

fn triple_name_positions(tp: &TriplePattern, vars: &mut FxHashSet<String>) {
    // A variable in predicate position is a property-name position.
    let NamedNodePattern::NamedNode(pred) = &tp.predicate else {
        if let NamedNodePattern::Variable(v) = &tp.predicate {
            vars.insert(v.as_str().to_string());
        }
        return;
    };
    let pred = pred.as_str();
    let mut grab = |t: &TermPattern| {
        if let TermPattern::Variable(v) = t {
            vars.insert(v.as_str().to_string());
        }
    };
    // Object is a class-name position.
    if pred == rdf::TYPE.as_str()
        || pred == format!("{RDFS}domain")
        || pred == format!("{RDFS}range")
        || pred == format!("{OWL}someValuesFrom")
        || pred == format!("{OWL}allValuesFrom")
        || pred == format!("{OWL}onClass")
    {
        grab(&tp.object);
    }
    // Both ends are class-name positions.
    if pred == format!("{RDFS}subClassOf")
        || pred == format!("{OWL}equivalentClass")
        || pred == format!("{OWL}disjointWith")
        || pred == format!("{OWL}complementOf")
    {
        grab(&tp.subject);
        grab(&tp.object);
    }
    // Subject is a property-name position.
    if pred == format!("{RDFS}domain") || pred == format!("{RDFS}range") {
        grab(&tp.subject);
    }
    // Both ends are property-name positions.
    if pred == format!("{RDFS}subPropertyOf")
        || pred == format!("{OWL}equivalentProperty")
        || pred == format!("{OWL}propertyDisjointWith")
        || pred == format!("{OWL}inverseOf")
    {
        grab(&tp.subject);
        grab(&tp.object);
    }
    // Object is a property-name position.
    if pred == format!("{OWL}onProperty") {
        grab(&tp.object);
    }
}

/// eq-ref (OWL 2 Profiles §4.3, Table 4): `T(?s ?p ?o) → T(?s owl:sameAs ?s),
/// T(?p owl:sameAs ?p), T(?o owl:sameAs ?o)`. The production materializer
/// deliberately omits the full reflexive owl:sameAs layer (store bloat, like
/// the declared reflexives above), but the regime's answers need it — e.g.
/// sparqldl-10 walks `?a owl:sameAs ?b` over individuals with NO asserted
/// sameAs. Literal subjects would be generalized triples and are skipped.
fn add_eq_ref(dict: &mut Dict, ids: &mut Vec<[sparq_core::dict::Id; 3]>) {
    let same_as = dict.intern_iri("http://www.w3.org/2002/07/owl#sameAs");
    let mut terms: FxHashSet<sparq_core::dict::Id> = FxHashSet::default();
    for &[s, p, o] in ids.iter() {
        terms.insert(s);
        terms.insert(p);
        terms.insert(o);
    }
    for t in terms {
        if !matches!(dict.term(t), Term::Literal(_)) {
            ids.push([t, same_as, t]);
        }
    }
}

/// The finite-vocabulary reflexive layer the entailment regimes expect of
/// DECLARED terms (the production materializer omits all reflexives as store
/// bloat): every declared class is its own subclass (rdfs10/scm-cls), every
/// declared property its own subproperty (rdfs6/scm-op), and — under the OWL
/// regimes — every declared named individual an `owl:Thing`.
fn add_declared_reflexives(
    dict: &mut Dict,
    ids: &mut Vec<[sparq_core::dict::Id; 3]>,
    owl: bool,
) {
    let ty = dict.intern_iri(rdf::TYPE.as_str());
    let sc = dict.intern_iri(oxrdf::vocab::rdfs::SUB_CLASS_OF.as_str());
    let sp = dict.intern_iri(oxrdf::vocab::rdfs::SUB_PROPERTY_OF.as_str());
    let class_tys = [
        dict.intern_iri("http://www.w3.org/2002/07/owl#Class"),
        dict.intern_iri("http://www.w3.org/2000/01/rdf-schema#Class"),
    ];
    let prop_tys = [
        dict.intern_iri("http://www.w3.org/2002/07/owl#ObjectProperty"),
        dict.intern_iri("http://www.w3.org/2002/07/owl#DatatypeProperty"),
        dict.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property"),
    ];
    let named_ind = dict.intern_iri("http://www.w3.org/2002/07/owl#NamedIndividual");
    let thing = dict.intern_iri("http://www.w3.org/2002/07/owl#Thing");
    let nothing = dict.intern_iri("http://www.w3.org/2002/07/owl#Nothing");
    let mut add: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
    for &[s, p, o] in ids.iter() {
        if p == sc || p == sp {
            // rdfs6/10 via the axiomatic domain/range of subClassOf and
            // subPropertyOf: both endpoints are classes/properties.
            add.push([s, p, s]);
            add.push([o, p, o]);
        }
        if p != ty {
            continue;
        }
        if class_tys.contains(&o) {
            add.push([s, sc, s]);
            if owl {
                // scm-cls: every class is ⊑ owl:Thing and ⊒ owl:Nothing.
                add.push([s, sc, thing]);
                add.push([nothing, sc, s]);
            }
        } else if prop_tys.contains(&o) {
            add.push([s, sp, s]);
        } else if owl && o == named_ind {
            add.push([s, ty, thing]);
        }
    }
    ids.extend(add);
}

/// rdfD2: every asserted predicate is an `rdf:Property` — part of the RDF (and
/// RDFS) regime vocabulary the production materializer omits as store bloat.
fn add_rdfd2(dict: &mut Dict, ids: &mut Vec<[sparq_core::dict::Id; 3]>) {
    let ty = dict.intern_iri(rdf::TYPE.as_str());
    let property = dict.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property");
    let preds: FxHashSet<_> = ids.iter().map(|t| t[1]).collect();
    for p in preds {
        ids.push([p, ty, property]);
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-kuvu3 (epic sq-pbz04) — the EXPERIMENTAL OWL 2 QL (DL-Lite_R)
// query-rewriting arm. Opt-in, `ql-experimental`.
// ---------------------------------------------------------------------------

/// The OWL profile IRI a test must list under `sd:EntailmentProfile` for the QL
/// arm to attempt it.
#[cfg(feature = "ql-experimental")]
const PR_QL: &str = "http://www.w3.org/ns/owl-profile/QL";

/// [OPUS-4.8] sq-kuvu3 — what the EXPERIMENTAL QL rewriting arm genuinely computed
/// for one `pr:QL`-tagged entailment test. EVERY variant is reported in the
/// `OutOfScope`/experimental bucket (never a graduated Pass/Fail that counts to a
/// conformance rate or a ratchet floor), so a future QL regression can never
/// silently claim OWL 2 QL conformance — yet the report still records, honestly,
/// exactly what `sparq-reason-ql` produced.
#[cfg(feature = "ql-experimental")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QlArmOutcome {
    /// The rewriter ABSTAINED — the fail-closed CQ-shape gate (or a TBox/atom
    /// mapping it cannot soundly handle) rejected the query as out of QL rewriting
    /// scope. NEVER a guessed answer. Carries the rewriter's honest reason.
    Abstained(String),
    /// The query rewrote to a UCQ and its evaluation over the UNMODIFIED data was
    /// RESULT-EQUIVALENT to the suite oracle (`disjuncts` = the minimised UCQ size).
    /// Reported as experimental evidence, NOT a graduated conformance pass.
    ComputedEquivalent { disjuncts: usize },
    /// The query rewrote but its evaluation DIVERGED from the oracle (the honest
    /// experimental gap — these are exactly the cases the deferred conformance-floor
    /// graduation bead would have to close). Carries the observed mismatch.
    ComputedDivergent { disjuncts: usize, detail: String },
    /// The harness could not set the test up (e.g. a data/result parse error or a
    /// missing result file) — distinct from an abstain, and never a pass.
    Inconclusive(String),
}

#[cfg(feature = "ql-experimental")]
impl QlArmOutcome {
    /// The single-line `OutOfScope` reason this outcome is reported under. The
    /// stable `QL experimental:` prefix lets the report group every QL row in one
    /// experimental histogram bucket (and lets a reader see it is NOT a conformance
    /// claim).
    pub fn reason(&self) -> String {
        match self {
            QlArmOutcome::Abstained(why) => {
                format!("QL experimental (abstain, fail-closed): {}", why)
            }
            QlArmOutcome::ComputedEquivalent { disjuncts } => format!(
                "QL experimental (computed result-equivalent to oracle, UCQ {} disjunct(s); \
                 NOT a graduated conformance pass)",
                disjuncts
            ),
            QlArmOutcome::ComputedDivergent { disjuncts, detail } => format!(
                "QL experimental (computed DIVERGENT from oracle, UCQ {} disjunct(s)): {}",
                disjuncts, detail
            ),
            QlArmOutcome::Inconclusive(why) => format!("QL experimental (inconclusive): {}", why),
        }
    }
}

/// [OPUS-4.8] sq-kuvu3 — run the EXPERIMENTAL OWL 2 QL query-rewriting arm over the
/// `pr:QL`-tagged tests of the `sparql11/entailment` suite and append one
/// `OutOfScope`/experimental `TestResult` per case.
///
/// HONESTY CONTRACT (the load-bearing invariant this arm preserves):
/// * NO FLOOR GRADUATION — every result lands in the `Outcome::OutOfScope`
///   (experimental) bucket; the arm registers NO `scoreboard::SUITES` row and NO
///   ratchet `const`, so it can never assert OWL 2 QL conformance.
/// * FAIL-CLOSED PRESERVED — a query the rewriter cannot soundly rewrite (the
///   fail-closed `CqError::OutOfScope` gate) is reported `Abstained`, never a
///   guessed pass.
/// * NO FAKED PASSES — the arm reports exactly what `sparq-reason-ql` computes:
///   `ComputedEquivalent` only when the rewritten UCQ's evaluation over the
///   UNMODIFIED data genuinely matches the oracle; an abstain / divergence / setup
///   failure is reported as such, never inflated into a pass.
///
/// The suite key is `sparql11/entailment (QL experimental)` so these rows are
/// visibly separate from the gating regime rows.
#[cfg(feature = "ql-experimental")]
pub fn run_ql_experimental_arm(
    rdf_tests_root: &Path,
    out: &mut Vec<TestResult>,
) -> Result<(), String> {
    let suites_root = rdf_tests_root
        .join("sparql")
        .canonicalize()
        .map_err(|e| format!("rdf-tests sparql dir: {e}"))?;
    let manifest_path = suites_root.join("sparql11/entailment/manifest.ttl");
    let mut entries: Vec<TestEntry> = Vec::new();
    manifest::collect(&manifest_path, &suites_root, &mut entries)?;

    for entry in &entries {
        if entry.kind != EntryKind::QueryEval {
            continue;
        }
        // Select only the cases whose expected answers are sanctioned under OWL 2
        // QL (`sd:EntailmentProfile` lists `pr:QL`). Everything else is left to the
        // gating regime arms in `run_suite`.
        if !entry.action.entailment_profiles.iter().any(|p| p == PR_QL) {
            continue;
        }
        let outcome = ql_arm_one(entry);
        out.push(TestResult {
            suite: "sparql11/entailment (QL experimental)".into(),
            name: entry.name.clone(),
            // EVERY QL row is OutOfScope/experimental — never a graduated pass.
            outcome: Outcome::OutOfScope(outcome.reason()),
        });
    }
    Ok(())
}

/// Notes rendered under the QL experimental section (when the arm is wired into a
/// report). Honest scoping: this is a query-rewriting EXPERIMENT, not a
/// conformance-graduated regime.
#[cfg(feature = "ql-experimental")]
pub fn ql_experimental_notes() -> Vec<String> {
    vec![
        "EXPERIMENTAL OWL 2 QL (DL-Lite_R) query-rewriting arm (sq-kuvu3, opt-in \
         `ql-experimental`). Each `sd:EntailmentProfile pr:QL` case is rewritten by \
         `sparq_reason_ql::rewrite_production` (PerfectRef ∪ bounded tree-witness ∪ \
         UCQ-containment minimisation) into a union of conjunctive queries that is \
         evaluated over the UNMODIFIED data (the DL-Lite query-rewriting semantics — no \
         materialised closure) and compared to the suite oracle. The TBox is read from \
         the test's own data graph. EVERY row is reported in the experimental \
         OUT-OF-SCOPE bucket: this arm is NOT a graduated conformance floor and asserts \
         NO OWL 2 QL conformance — it records, honestly, exactly what the rewriter \
         computes. The rewriter is sound-but-FAIL-CLOSED: a query outside the \
         conjunctive-query fragment (OPTIONAL / FILTER / MINUS / UNION / a property path \
         / aggregation / a variable predicate) is reported as an ABSTAIN, never a \
         guessed pass. Computed result-equivalence is experimental evidence only; the \
         graduation of QL to a pinned conformance floor is a separate, deferred bead \
         (it must sequence through the contended conformance scoreboard)."
            .to_string(),
    ]
}

/// Run the EXPERIMENTAL QL arm for ONE `pr:QL` test and report what the rewriter
/// genuinely computed. Pure (no global state); the watchdog mirrors the other
/// runners so a rewriter hang/panic becomes a recorded `Inconclusive`, not a dead
/// harness.
#[cfg(feature = "ql-experimental")]
fn ql_arm_one(entry: &TestEntry) -> QlArmOutcome {
    // The QL entailment cases all use a single default-graph `qt:data` dataset
    // (the same precondition `run_one` relies on); a named-graph dataset would
    // need per-graph rewriting semantics, so report it inconclusive rather than
    // silently rewrite over the wrong TBox.
    if !entry.action.graph_data.is_empty() {
        return QlArmOutcome::Inconclusive("named-graph QL dataset not wired".into());
    }
    let Some(query_path) = &entry.action.query else {
        return QlArmOutcome::Inconclusive("manifest entry has no qt:query".into());
    };
    let Some(result_path) = &entry.result_file else {
        return QlArmOutcome::Inconclusive("manifest entry has no mf:result".into());
    };

    // Load the test's data triples — these carry BOTH the ABox and the (DL-Lite_R)
    // TBox the rewriter extracts from. Blank-node labels are irrelevant to the
    // schema axioms, so a plain triple parse suffices.
    let mut tbox: Vec<oxrdf::Triple> = Vec::new();
    for d in &entry.action.data {
        match crate::rdf::parse_file(d) {
            Ok(triples) => tbox.extend(triples),
            // Positional `format!` args (not inline `{e}`) per the CodeQL
            // `rust/unused-variable` false-positive guard in the shared agent contract.
            Err(e) => return QlArmOutcome::Inconclusive(format!("data parse error: {}", e)),
        }
    }

    let query_text = match std::fs::read_to_string(query_path) {
        Ok(t) => t,
        Err(e) => return QlArmOutcome::Inconclusive(format!("read query: {}", e)),
    };
    let base = crate::rdf::file_iri(query_path);
    let parser = match SparqlParser::new().with_base_iri(&base) {
        Ok(p) => p,
        Err(e) => return QlArmOutcome::Inconclusive(format!("bad base IRI: {}", e)),
    };
    let query = match parser.parse_query(&query_text) {
        Ok(q) => q,
        Err(e) => return QlArmOutcome::Inconclusive(format!("query parse error: {}", e)),
    };

    // Rewrite under a watchdog: a rewriter hang/panic is a recorded Inconclusive.
    let q_for_thread = query.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(move || {
            sparq_reason_ql::rewrite_production(&q_for_thread, &tbox)
        });
        let _ = tx.send(result);
    });
    let rewritten = match rx.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(Ok(Ok(r))) => r,
        // FAIL-CLOSED: the rewriter rejected a non-CQ / unsupported query — abstain,
        // never a guessed answer.
        Ok(Ok(Err(e))) => return QlArmOutcome::Abstained(e.to_string()),
        Ok(Err(_)) => return QlArmOutcome::Inconclusive("rewriter panicked".into()),
        Err(_) => return QlArmOutcome::Inconclusive("rewriter timeout (20s)".into()),
    };

    // Evaluate the rewritten UCQ over the UNMODIFIED data and compare to the
    // oracle. The rewritten query serialises to standard SPARQL and runs through
    // the SAME engine the gating harness uses — but we report ONLY experimentally.
    let disjuncts = rewritten.report.disjuncts;
    match ql_eval_matches_oracle(entry, &rewritten.query, result_path) {
        Ok(true) => QlArmOutcome::ComputedEquivalent { disjuncts },
        Ok(false) => QlArmOutcome::ComputedDivergent {
            disjuncts,
            detail: "rewritten-UCQ answers differ from the oracle".into(),
        },
        Err(e) => QlArmOutcome::Inconclusive(e),
    }
}

/// Evaluate a REWRITTEN UCQ over the test's UNMODIFIED default-graph data and
/// report whether the answers are result-equivalent to the suite oracle. SELECT
/// and ASK only (the entailment suite is all SELECT/ASK). Runs the engine on a
/// watchdog thread, like the sibling runners.
#[cfg(feature = "ql-experimental")]
fn ql_eval_matches_oracle(
    entry: &TestEntry,
    rewritten: &spargebra::Query,
    result_path: &Path,
) -> Result<bool, String> {
    use crate::compare::Row;
    use crate::results::Expected;
    use std::collections::BTreeSet;

    let expected = crate::results::parse_expected(result_path)?;

    // Build the default-graph dataset from the UNMODIFIED data files.
    let mut nquads = String::new();
    for d in &entry.action.data {
        // Positional `format!` args per the CodeQL `rust/unused-variable` guard.
        for t in crate::rdf::parse_file(d).map_err(|e| format!("data parse: {}", e))? {
            nquads.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object));
        }
    }
    // The rewritten query serialises to standard SPARQL (a UNION-folded UCQ under
    // the original projection); run it through the engine unchanged.
    let query_str = rewritten.to_string();

    let is_ask = matches!(rewritten, spargebra::Query::Ask { .. });
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let graph = sparq_core::Graph::load_dataset(&nquads, "nquads")?;
            if is_ask {
                let b = sparq_engine::ask(&graph, &query_str)?;
                Ok::<_, String>((true, b, Vec::new(), Vec::new()))
            } else {
                let res = sparq_engine::query(&graph, &query_str)?;
                let vars: Vec<String> = res.vars.iter().map(|v| v.as_str().to_string()).collect();
                Ok((false, false, vars, res.rows))
            }
        })();
        let _ = tx.send(result);
    });
    let (was_ask, ask_bool, actual_vars, actual_rows) =
        match rx.recv_timeout(std::time::Duration::from_secs(20)) {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => return Err(format!("engine error: {}", e)),
            Err(_) => return Err("engine timeout/panic (20s)".into()),
        };

    if was_ask {
        return match expected {
            Expected::Boolean(b) => Ok(ask_bool == b),
            Expected::Bindings { .. } => Err("expected bindings, rewritten query is ASK".into()),
        };
    }
    let (exp_vars, exp_rows) = match expected {
        Expected::Bindings { vars, rows, .. } => (vars, rows),
        Expected::Boolean(_) => return Err("expected boolean, rewritten query is SELECT".into()),
    };

    // Align both sides on a shared variable order, then compare as multisets (the
    // QL entailment queries are unordered).
    let mut all_vars: BTreeSet<String> = actual_vars.iter().cloned().collect();
    all_vars.extend(exp_vars.iter().cloned());
    for row in &exp_rows {
        all_vars.extend(row.iter().map(|(v, _)| v.clone()));
    }
    let order: Vec<String> = all_vars.into_iter().collect();
    let exp: Vec<Row> = exp_rows
        .iter()
        .map(|r| {
            order
                .iter()
                .map(|v| r.iter().find(|(name, _)| name == v).map(|(_, t)| t.clone()))
                .collect()
        })
        .collect();
    let act: Vec<Row> = actual_rows
        .iter()
        .map(|r| {
            order
                .iter()
                .map(|v| {
                    actual_vars
                        .iter()
                        .position(|av| av == v)
                        .and_then(|i| r.get(i).cloned().flatten())
                })
                .collect()
        })
        .collect();
    crate::compare::rows_equal(&exp, &act, false)
}

// [OPUS-4.8] sq-kuvu3 — DIRECT unit tests for the new public QL-arm surface (the
// coverage-ratchet rule: one direct test per new public fn). These are hermetic
// (no fetched fixtures) and assert the HONESTY INVARIANTS the arm must preserve.
#[cfg(all(test, feature = "ql-experimental"))]
mod ql_tests {
    use super::*;

    #[test]
    fn outcome_reason_strings_never_claim_conformance() {
        // Every QL outcome's reason must carry the experimental marker AND must
        // never read as a graduated conformance pass.
        let abstain = QlArmOutcome::Abstained("BIND (Extend) is not conjunctive".into());
        assert!(abstain.reason().contains("QL experimental"));
        assert!(abstain.reason().contains("fail-closed"));

        let equiv = QlArmOutcome::ComputedEquivalent { disjuncts: 2 };
        let r = equiv.reason();
        assert!(r.contains("QL experimental"));
        // The load-bearing disclaimer: equivalence is evidence, NOT a graduated pass.
        assert!(r.contains("NOT a graduated conformance pass"));
        assert!(r.contains("2 disjunct"));

        let div = QlArmOutcome::ComputedDivergent {
            disjuncts: 1,
            detail: "rewritten-UCQ answers differ from the oracle".into(),
        };
        assert!(div.reason().contains("DIVERGENT"));
        assert!(div.reason().contains("QL experimental"));

        let inc = QlArmOutcome::Inconclusive("named-graph QL dataset not wired".into());
        assert!(inc.reason().contains("inconclusive"));
    }

    #[test]
    fn ql_experimental_notes_disclaim_a_conformance_floor() {
        let notes = ql_experimental_notes();
        assert_eq!(notes.len(), 1);
        let n = &notes[0];
        // The honest scoping note: experimental, fail-closed, NOT a conformance floor.
        assert!(n.contains("EXPERIMENTAL"));
        assert!(n.contains("NOT a graduated conformance floor"));
        assert!(n.contains("FAIL-CLOSED"));
        assert!(n.contains("ABSTAIN"));
    }

    #[test]
    fn arm_reports_only_out_of_scope_rows() {
        // The arm appends ONLY OutOfScope (experimental) rows — never a Pass / Fail /
        // Divergence that would count toward a conformance rate or ratchet floor.
        // (Drive it over the fixtures when present; otherwise this asserts the
        // contract vacuously on an empty run — the floor invariant holds either way.)
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/w3c/rdf-tests");
        if !root.join("sparql/sparql11/entailment/manifest.ttl").exists() {
            eprintln!("SKIP: rdf-tests entailment fixtures absent");
            return;
        }
        let mut out: Vec<TestResult> = Vec::new();
        run_ql_experimental_arm(&root, &mut out).expect("QL arm runs");
        assert!(!out.is_empty(), "the suite has pr:QL-tagged tests");
        for r in &out {
            assert!(
                matches!(r.outcome, Outcome::OutOfScope(_)),
                "QL arm emitted a non-OutOfScope row ({:?}) — that would risk a faked \
                 conformance pass / floor graduation",
                r.outcome
            );
            // Every reason carries the stable experimental marker.
            if let Outcome::OutOfScope(reason) = &r.outcome {
                assert!(reason.starts_with("QL experimental"), "reason: {reason}");
            }
        }
    }
}
