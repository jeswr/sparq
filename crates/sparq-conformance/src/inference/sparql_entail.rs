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
         OWL RL rules, a sound subset — its incompleteness shows up as listed fails). \
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

#[derive(Clone, Copy, PartialEq)]
enum Profile {
    Rdf,
    Rdfs,
    OwlRl,
}

fn result(entry: &TestEntry, outcome: Outcome) -> TestResult {
    TestResult {
        suite: "sparql11/entailment".into(),
        name: entry.name.clone(),
        outcome,
    }
}

fn run_one(entry: &TestEntry, profile: Profile, direct_sanctioned: bool) -> Outcome {
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
