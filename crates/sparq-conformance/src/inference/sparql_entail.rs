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
use crate::run::{self, Status};
use oxrdf::vocab::rdf;
use oxrdf::Term;
use rustc_hash::FxHashSet;
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
        out.push(result(entry, run_one(entry, profile)));
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

fn run_one(entry: &TestEntry, profile: Profile) -> Outcome {
    if !entry.action.graph_data.is_empty() {
        return Outcome::OutOfScope("named-graph entailment dataset not wired".into());
    }
    // Load and materialize the default-graph data.
    let mut dict = Dict::new();
    let mut ids: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
    for d in &entry.action.data {
        match crate::rdf::parse_file(d) {
            Ok(triples) => {
                for t in triples {
                    let row = [
                        Term::from(t.subject),
                        Term::NamedNode(t.predicate),
                        t.object,
                    ];
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

    match run::run_query_test_on(entry, Some(nquads)) {
        Status::Pass => Outcome::Pass,
        Status::Fail(e) => Outcome::Fail(annotate(&entry.name, e)),
        Status::Skip(e) => Outcome::OutOfScope(e),
    }
}

/// Root-cause annotations for known structural fails (kept as FAILS — these
/// are real engine/harness gaps, not suite errors — but the report should say
/// WHY, for the follow-up fix threads).
fn annotate(name: &str, reason: String) -> String {
    let cause = match name {
        n if n.contains("sparqldl-10") => {
            Some("needs non-distinguished (blank-node) query-variable semantics")
        }
        n if n.contains("sparqldl-11") || n.contains("sparqldl-12") => Some(
            "the regime's answer restriction (skolemization condition) excludes blank-node \
             bindings; the harness does not yet filter them from engine results",
        ),
        _ => None,
    };
    match cause {
        Some(c) => format!("{reason} [root cause: {c}]"),
        None => reason,
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
