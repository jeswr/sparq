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

// ===========================================================================
// [OPUS-4.8] sq-qo1a9 (epic sq-pbz04) — the GRADUATED DL-Lite_R CERTAIN-ANSWER
// oracle. This is the FORMAL OWL 2 QL conformance arm that earns a pinned floor.
//
// The broad `pr:QL` `sparql11/entailment` set (run by `run_ql_experimental_arm`
// above) is NOT the formal OWL2-QL / DL-Lite_R suite: it mixes intensional /
// non-DL-Lite certain-answer cases the sound query-rewriting fragment cannot
// answer (so it stays EXPERIMENTAL / OutOfScope, with its 1 documented
// divergence). The FORMAL suite is the HAND-DERIVED DL-Lite_R oracle from
// sq-g19x0 — every case is a conjunctive query within sound rewriting, with a
// hand-checked EXACT certain-answer set. On that suite the rewrite is sound AND
// complete case by case, so it graduates.
//
// What "matches the oracle on a case" MEANS here (the load-bearing definition):
// given the case's (TBox, ABox, CQ), `rewrite_production` produces a UCQ that,
// evaluated over the UNMODIFIED ABox through the REAL engine, returns EXACTLY the
// hand-derived certain-answer set — no missing answer (completeness) and no extra
// answer (soundness). Each case's certain answers are derived by hand from the
// DL-Lite_R semantics (the same derivations the rewrite-shape oracle in
// `sparq-reason-ql/tests/oracle.rs` re-checks), so the comparison is against an
// INDEPENDENT ground truth, not the rewriter's own output.
//
// HONEST SCOPE: this is a faithful DL-Lite_R certain-answer oracle, NOT the full
// normative OWL 2 QL conformance suite (there is no runnable W3C answer-comparison
// QL suite — the W3C QL material is structural). So the graduated ratchet is a
// sparq EXTENSION row (like RIF-Core / RSP / BM25), tallied SEPARATELY and NEVER
// folded into the standards-conformance total. The pinned floor is exactly the
// count of cases on which the rewrite is sound AND complete.
// ===========================================================================

/// One hand-derived DL-Lite_R certain-answer oracle case: a TBox + ABox + a
/// conjunctive `SELECT` query, with the EXACT certain-answer set derived by hand
/// from the DL-Lite_R semantics. The runner asserts `rewrite_production`'s UCQ,
/// evaluated over the UNMODIFIED ABox, returns exactly `certain`.
#[cfg(feature = "ql-experimental")]
pub struct QlOracleCase {
    /// Short stable id (for the report + the per-case outcome).
    pub id: &'static str,
    /// The DL-Lite_R TBox, as Turtle (prefixes are prepended by the runner).
    pub tbox: &'static str,
    /// The ABox (the unmodified data the rewrite is evaluated over), as Turtle.
    pub abox: &'static str,
    /// The conjunctive `SELECT` query (a single answer variable `?x`, or two
    /// `?x ?y` — the runner reads the projected variables from the parse).
    pub query: &'static str,
    /// The EXACT certain answers, each a row of local-name bindings aligned to the
    /// query's projected variables in order (e.g. `&[&["alice"]]` for one row
    /// binding `?x` to `:alice`). The empty slice means "no certain answers".
    /// Local names resolve against the `http://ex/` prefix.
    pub certain: &'static [&'static [&'static str]],
}

/// The result of running ONE [`QlOracleCase`] through the graduated arm.
#[cfg(feature = "ql-experimental")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QlOracleOutcome {
    /// The rewrite's UCQ, evaluated over the unmodified ABox, returned EXACTLY the
    /// hand-derived certain answers — sound AND complete on this case. This is the
    /// only outcome that counts toward the pinned floor.
    SoundAndComplete,
    /// The rewrite produced answers that differ from the certain-answer oracle (a
    /// missing or an extra answer). NEVER counted into the floor — a divergence is
    /// an honest gap (or a bug), reported with the mismatch.
    Diverged(String),
    /// The rewriter ABSTAINED (the fail-closed CQ-shape / non-DL-Lite gate). A
    /// formal-suite case is constructed to be within sound rewriting, so an abstain
    /// here is reported (never counted as a pass), but means the case is outside the
    /// rewriter's current fragment — kept honest, not summed into the floor.
    Abstained(String),
    /// The harness could not set the case up (parse error). Never a pass.
    Inconclusive(String),
}

/// The HAND-DERIVED DL-Lite_R certain-answer oracle corpus (sq-qo1a9). Each case
/// is within the SOUND query-rewriting fragment of DL-Lite_R; the `certain` field
/// is derived by hand from the DL-Lite_R semantics, INDEPENDENTLY of the rewriter.
///
/// The corpus exercises the DL-Lite_R rewriting axes the formal oracle proves:
/// class subsumption (incl. a transitive chain), the existential domain/range
/// inclusions (`∃R ⊑ A`, `∃R⁻ ⊑ A`), role inclusion, `owl:inverseOf`, the
/// unqualified `∃R` super-class generator with its applicability condition (must
/// NOT fire on a bound/projected filler), a two-distinguished-variable product
/// (no over-minimisation), and the empty-TBox identity. Every certain-answer set
/// is the set of bindings that hold in EVERY model of (TBox ∪ ABox).
#[cfg(feature = "ql-experimental")]
pub const QL_DLLITE_ORACLE: &[QlOracleCase] = &[
    // C1 — class subsumption: Manager ⊑ Employee. A Manager is certainly an
    // Employee, so { ?x a :Employee } over { :a a :Manager . :b a :Employee }
    // certainly answers both :a and :b.
    QlOracleCase {
        id: "ql-class-subsumption",
        tbox: ":Manager rdfs:subClassOf :Employee .",
        abox: ":a a :Manager . :b a :Employee .",
        query: "SELECT ?x WHERE { ?x a :Employee }",
        certain: &[&["a"], &["b"]],
    },
    // C2 — transitive subclass chain: A ⊑ B ⊑ C. Any A or B is certainly a C.
    QlOracleCase {
        id: "ql-transitive-subclass",
        tbox: ":A rdfs:subClassOf :B . :B rdfs:subClassOf :C .",
        abox: ":a a :A . :b a :B . :c a :C . :d a :Other .",
        query: "SELECT ?x WHERE { ?x a :C }",
        certain: &[&["a"], &["b"], &["c"]],
    },
    // C3 — domain inclusion (∃worksFor ⊑ Employee): anyone who worksFor anything is
    // certainly an Employee. { ?x a :Employee } answers :a (asserted) and :p (has a
    // worksFor edge).
    QlOracleCase {
        id: "ql-domain-introduces-role",
        tbox: ":worksFor rdfs:domain :Employee .",
        abox: ":a a :Employee . :p :worksFor :acme .",
        query: "SELECT ?x WHERE { ?x a :Employee }",
        certain: &[&["a"], &["p"]],
    },
    // C4 — range inclusion (∃worksFor⁻ ⊑ Company): anything something worksFor is
    // certainly a Company. :acme is the object of a worksFor edge.
    QlOracleCase {
        id: "ql-range-introduces-inverse-role",
        tbox: ":worksFor rdfs:range :Company .",
        abox: ":acme a :Company . :p :worksFor :globex .",
        query: "SELECT ?y WHERE { ?y a :Company }",
        certain: &[&["acme"], &["globex"]],
    },
    // C5 — role inclusion (manages ⊑ worksFor): a manages edge is certainly a
    // worksFor edge. Two distinguished variables — the answer is the union of the
    // asserted worksFor pairs and the manages pairs.
    QlOracleCase {
        id: "ql-role-inclusion",
        tbox: ":manages rdfs:subPropertyOf :worksFor .",
        abox: ":p :worksFor :acme . :q :manages :globex .",
        query: "SELECT ?x ?y WHERE { ?x :worksFor ?y }",
        certain: &[&["p", "acme"], &["q", "globex"]],
    },
    // C6 — owl:inverseOf (employs ≡ worksFor⁻) composed with a role inclusion
    // (manages ⊑ employs). A :manages b ⇒ a employs b ⇒ b worksFor a. So
    // { ?x :worksFor ?y } certainly answers the asserted worksFor pairs PLUS the
    // inverse of every manages pair.
    QlOracleCase {
        id: "ql-inverse-of-role-chain",
        tbox: ":employs owl:inverseOf :worksFor . :manages rdfs:subPropertyOf :employs .",
        abox: ":p :worksFor :acme . :boss :manages :emp .",
        query: "SELECT ?x ?y WHERE { ?x :worksFor ?y }",
        certain: &[&["p", "acme"], &["emp", "boss"]],
    },
    // C7 — the unqualified ∃R super-class generator (Employee ⊑ ∃worksFor) FIRES on
    // an UNBOUND (non-distinguished) filler. { ?x :worksFor ?y } with ?y not
    // projected: anyone asserted Employee certainly has SOME worksFor witness, so
    // :e (an Employee with no asserted edge) IS a certain answer, alongside :p (an
    // asserted worksFor subject).
    QlOracleCase {
        id: "ql-exists-super-fires-unbound",
        tbox: ":Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ] .",
        abox: ":e a :Employee . :p :worksFor :acme .",
        query: "SELECT ?x WHERE { ?x :worksFor ?y }",
        certain: &[&["e"], &["p"]],
    },
    // C8 — the APPLICABILITY CONDITION (the #1 unsoundness trap): the SAME
    // Employee ⊑ ∃worksFor generator MUST NOT fire when the filler ?y is PROJECTED
    // (bound). With ?y distinguished, { ?x :worksFor ?y } answers ONLY the asserted
    // edges — :e (the witness-only Employee) is NOT a certain answer because its
    // worksFor witness is anonymous (not a named binding for ?y).
    QlOracleCase {
        id: "ql-exists-super-blocked-bound",
        tbox: ":Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ] .",
        abox: ":e a :Employee . :p :worksFor :acme .",
        query: "SELECT ?x ?y WHERE { ?x :worksFor ?y }",
        certain: &[&["p", "acme"]],
    },
    // C9 — two distinguished variables under A ⊑ B: { ?x a :B . ?y a :B } over an
    // ABox with an A and a B. The certain answers are the full 2×2 product over the
    // set { :a (an A, hence a B), :b (a B) } — the anti-over-minimisation case
    // (minimisation must not collapse the disjuncts and lose a pair).
    QlOracleCase {
        id: "ql-two-var-product",
        tbox: ":A rdfs:subClassOf :B .",
        abox: ":a a :A . :b a :B .",
        query: "SELECT ?x ?y WHERE { ?x a :B . ?y a :B }",
        certain: &[&["a", "a"], &["a", "b"], &["b", "a"], &["b", "b"]],
    },
    // C10 — A∧C under A⊑B⊑C minimises to { A(?x) }: the conjunctive query
    // { ?x a :A . ?x a :C } has the same certain answers as { ?x a :A } (A ⇒ C), so
    // only the asserted/derivable A individuals answer. :a is an A; :c is only a C
    // (not certainly an A), so it is NOT an answer.
    QlOracleCase {
        id: "ql-conjunction-minimises",
        tbox: ":A rdfs:subClassOf :B . :B rdfs:subClassOf :C .",
        abox: ":a a :A . :c a :C .",
        query: "SELECT ?x WHERE { ?x a :A . ?x a :C }",
        certain: &[&["a"]],
    },
    // C11 — empty TBox is the identity: no schema ⇒ the certain answers are exactly
    // the asserted matches (a regression guard that the rewrite adds nothing).
    QlOracleCase {
        id: "ql-empty-tbox-identity",
        tbox: "",
        abox: ":a a :Employee . :b a :Manager .",
        query: "SELECT ?x WHERE { ?x a :Employee }",
        certain: &[&["a"]],
    },
];

/// [OPUS-4.8] sq-qo1a9 — run the GRADUATED DL-Lite_R certain-answer oracle: for
/// every [`QlOracleCase`], rewrite the CQ with `rewrite_production`, evaluate the
/// UCQ over the UNMODIFIED ABox through the REAL engine, and compare to the
/// hand-derived certain answers EXACTLY. Returns the per-case outcomes in corpus
/// order. Pure (no fetched fixtures — the corpus is in-source), so it runs on any
/// checkout. The caller (the ratchet test) counts `SoundAndComplete` outcomes as
/// the pinned floor and FAILS on any divergence.
#[cfg(feature = "ql-experimental")]
pub fn run_ql_dllite_oracle() -> Vec<(&'static str, QlOracleOutcome)> {
    QL_DLLITE_ORACLE
        .iter()
        .map(|case| (case.id, ql_oracle_one(case)))
        .collect()
}

/// The `http://ex/` prefix the oracle corpus's local names resolve against.
#[cfg(feature = "ql-experimental")]
const QL_EX: &str = "http://ex/";

/// Run ONE [`QlOracleCase`]: parse the TBox + ABox, rewrite the CQ, evaluate the
/// UCQ over the unmodified ABox, compare to the hand-derived certain answers.
#[cfg(feature = "ql-experimental")]
fn ql_oracle_one(case: &QlOracleCase) -> QlOracleOutcome {
    use crate::compare::Row;
    use oxrdf::{NamedNode, Term};

    // The Turtle prefix preamble shared by the TBox + ABox (and the query base).
    const TTL_PRE: &str = "\
@prefix : <http://ex/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
";
    const Q_PRE: &str = "\
PREFIX : <http://ex/>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
";

    // Parse the TBox triples (the schema the rewriter extracts DL-Lite_R axioms
    // from). Positional `format!` args per the CodeQL `rust/unused-variable` guard.
    let tbox_src = format!("{}{}", TTL_PRE, case.tbox);
    let mut tbox: Vec<oxrdf::Triple> = Vec::new();
    for t in oxttl::TurtleParser::new().for_slice(tbox_src.as_bytes()) {
        match t {
            Ok(tr) => tbox.push(tr),
            Err(e) => return QlOracleOutcome::Inconclusive(format!("TBox parse: {}", e)),
        }
    }

    // Parse + serialise the ABox as N-Triples for the engine (the UNMODIFIED data).
    let abox_src = format!("{}{}", TTL_PRE, case.abox);
    let mut nt = String::new();
    for t in oxttl::TurtleParser::new().for_slice(abox_src.as_bytes()) {
        match t {
            Ok(tr) => nt.push_str(&format!("{} {} {} .\n", tr.subject, tr.predicate, tr.object)),
            Err(e) => return QlOracleOutcome::Inconclusive(format!("ABox parse: {}", e)),
        }
    }

    // Parse the conjunctive query and rewrite it (fail-closed: an abstain is
    // reported, never a guessed pass).
    let q_src = format!("{}{}", Q_PRE, case.query);
    let query = match SparqlParser::new().parse_query(&q_src) {
        Ok(q) => q,
        Err(e) => return QlOracleOutcome::Inconclusive(format!("query parse: {}", e)),
    };
    let rewritten = match sparq_reason_ql::rewrite_production(&query, &tbox) {
        Ok(r) => r,
        Err(e) => return QlOracleOutcome::Abstained(e.to_string()),
    };

    // The projected variables, in order — defines the column alignment for both the
    // expected certain answers and the engine's actual rows.
    let proj: Vec<String> = match &rewritten.query {
        spargebra::Query::Select { pattern, .. } => projected_vars(pattern),
        _ => return QlOracleOutcome::Inconclusive("oracle case query is not SELECT".into()),
    };

    // Evaluate the rewritten UCQ over the unmodified ABox through the REAL engine.
    let query_str = rewritten.query.to_string();
    let graph = match sparq_core::Graph::load_dataset(&nt, "nquads") {
        Ok(g) => g,
        Err(e) => return QlOracleOutcome::Inconclusive(format!("load ABox: {}", e)),
    };
    let res = match sparq_engine::query(&graph, &query_str) {
        Ok(r) => r,
        Err(e) => return QlOracleOutcome::Inconclusive(format!("engine: {}", e)),
    };
    let actual_vars: Vec<String> = res.vars.iter().map(|v| v.as_str().to_string()).collect();

    // Build the EXPECTED rows from the hand-derived certain answers (local names →
    // `http://ex/<name>` IRIs), aligned to the projected variable order.
    let exp: Vec<Row> = case
        .certain
        .iter()
        .map(|binds| {
            binds
                .iter()
                .map(|name| {
                    Some(Term::NamedNode(NamedNode::new_unchecked(format!("{}{}", QL_EX, name))))
                })
                .collect::<Row>()
        })
        .collect();

    // Align the engine's actual rows to the SAME projected-variable order.
    let act: Vec<Row> = res
        .rows
        .iter()
        .map(|r| {
            proj.iter()
                .map(|v| {
                    actual_vars
                        .iter()
                        .position(|av| av == v)
                        .and_then(|i| r.get(i).cloned().flatten())
                })
                .collect::<Row>()
        })
        .collect();

    // EXACT multiset comparison: sound AND complete iff the actual rows equal the
    // hand-derived certain answers (no missing, no extra).
    match crate::compare::rows_equal(&exp, &act, false) {
        Ok(true) => QlOracleOutcome::SoundAndComplete,
        Ok(false) => QlOracleOutcome::Diverged(format!(
            "expected {} certain row(s), engine returned {} row(s) over the rewritten UCQ",
            exp.len(),
            act.len()
        )),
        Err(e) => QlOracleOutcome::Inconclusive(format!("compare: {}", e)),
    }
}

/// The projected variable names (in order) of a `SELECT` pattern — the answer
/// columns the oracle aligns expected vs actual rows on. Peels the
/// Project/Distinct/Reduced wrappers the rewrite re-wraps the UCQ in.
#[cfg(feature = "ql-experimental")]
fn projected_vars(p: &GraphPattern) -> Vec<String> {
    match p {
        GraphPattern::Project { variables, .. } => {
            variables.iter().map(|v| v.as_str().to_string()).collect()
        }
        GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. } => projected_vars(inner),
        _ => Vec::new(),
    }
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

    // [OPUS-4.8] sq-qo1a9 — DIRECT unit tests for the GRADUATED DL-Lite_R
    // certain-answer oracle surface (the coverage-ratchet rule: one direct test
    // per new public fn). Hermetic — the corpus is in-source, no fetched fixtures.
    #[test]
    fn dllite_oracle_is_sound_and_complete_on_every_formal_case() {
        // The load-bearing graduation claim: on the FORMAL DL-Lite_R suite the
        // rewrite is sound AND complete case by case — every case's rewritten UCQ,
        // evaluated over the unmodified ABox, returns EXACTLY the hand-derived
        // certain answers. ANY divergence/abstain/inconclusive must fail here.
        let results = run_ql_dllite_oracle();
        assert_eq!(
            results.len(),
            QL_DLLITE_ORACLE.len(),
            "the runner must report one outcome per oracle case"
        );
        for (id, outcome) in &results {
            assert_eq!(
                *outcome,
                QlOracleOutcome::SoundAndComplete,
                "DL-Lite_R oracle case {} is not sound-and-complete: {:?}",
                id,
                outcome
            );
        }
    }

    #[test]
    fn dllite_oracle_corpus_is_non_trivial_and_well_formed() {
        // The corpus must be a meaningful conformance claim: enough cases, every id
        // unique, and at least one case with NO certain answers (the applicability
        // condition C8 — the existential generator must not fire on a bound filler).
        assert!(
            QL_DLLITE_ORACLE.len() >= 10,
            "too few formal cases to be a meaningful conformance claim"
        );
        let mut ids: Vec<&str> = QL_DLLITE_ORACLE.iter().map(|c| c.id).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "oracle case ids must be unique");
        // A genuine certain-answer oracle must include a case whose answer set is a
        // STRICT subset of the naive asserted matches (the soundness direction).
        assert!(
            QL_DLLITE_ORACLE.iter().any(|c| c.id == "ql-exists-super-blocked-bound"),
            "the applicability-condition (bound-filler) soundness case must be present"
        );
    }

    #[test]
    fn dllite_oracle_outcome_variants_are_distinct() {
        // Direct surface coverage of the public QlOracleOutcome enum: the floor
        // only ever counts SoundAndComplete; the other three are honest non-passes.
        assert_ne!(
            QlOracleOutcome::SoundAndComplete,
            QlOracleOutcome::Diverged("x".into())
        );
        assert_ne!(
            QlOracleOutcome::Abstained("y".into()),
            QlOracleOutcome::Inconclusive("z".into())
        );
        // The corpus's projected-var helper handles the re-wrapped Project/Distinct.
        let q = SparqlParser::new()
            .parse_query("SELECT DISTINCT ?x WHERE { ?x <http://ex/p> ?y }")
            .unwrap();
        let spargebra::Query::Select { pattern, .. } = &q else {
            panic!("select");
        };
        assert_eq!(projected_vars(pattern), vec!["x".to_string()]);
    }
}
