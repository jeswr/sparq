//! **Cross-engine differential oracle** — the same query and data on two or more
//! engines, results compared with sparq-difftest's engine-independent comparators.
//! [FABLE-5] sq-gum8.6
//!
//! Unlike TLP/NoREC (which need only one engine's *internal* consistency), the
//! differential oracle needs an agreement standard across engines. That standard is
//! deliberately **not** any engine's own value code: results are compared through
//! [`sparq_difftest`]'s independent normalisation (see that crate's load-bearing
//! independence constraint), so a value bug shared by harness and engine cannot cancel.
//!
//! # Comparison semantics
//!
//! * `SELECT` results: **multiset equality** over value-canonical solution keys
//!   ([`sparq_difftest::multiset_equal`]) — order-insensitive, duplicate-preserving.
//!   Queries with `ORDER BY` are out of scope for this entry point (the generator emits
//!   unordered queries; `sparq_difftest::order_by_equal` exists for a future ordered
//!   campaign mode).
//! * `ASK` results: boolean equality.
//! * Shape mismatch (one engine answers a `SELECT`, another a boolean): classified as
//!   [`FailureKind::InvalidResults`] — a protocol/driver problem to triage, **not** a
//!   wrong-result claim (fail-closed).
//!
//! # Scope limits (documented, generator-enforced)
//!
//! * **Blank nodes**: sparq-difftest compares blank nodes by engine-local label, which
//!   is only meaningful within one engine (cross-engine label agreement is not required
//!   by SPARQL). The query/data generator therefore emits no blank nodes; cross-engine
//!   blank-node isomorphism is a separate difftest DAG node (bead `sq-qcnn.7`).
//! * **Implementation-defined behaviour**: engines may legitimately differ where the
//!   spec leaves latitude (extended-datatype comparisons, some canonical lexical
//!   choices — the value-canonical keying absorbs the known lexical variance). A
//!   reported divergence is a *candidate* bug; campaign triage attributes it to an
//!   engine (or to spec latitude) before it enters the found-bug ledger with an
//!   upstream issue link.

use sparq_difftest::{multiset_equal, QueryResults};

use crate::engine::SparqlEngine;
use crate::verdict::{EngineFailure, FailureKind, OracleKind, Verdict, Violation};

/// Run `query` on every engine and require agreement (see the module docs for the
/// comparison semantics). At least two engines are required — fewer is a
/// [`FailureKind::Harness`] failure, never a vacuous pass.
///
/// Fail-closed: if any engine fails to evaluate the query, the verdict is
/// [`Verdict::EngineFailure`] for the first failing engine.
pub fn check_differential(engines: &[&dyn SparqlEngine], query: &str) -> Verdict {
    if engines.len() < 2 {
        return Verdict::EngineFailure(EngineFailure {
            engine: String::new(),
            query: query.to_string(),
            kind: FailureKind::Harness,
            message: format!(
                "differential check needs at least 2 engines, got {}",
                engines.len()
            ),
        });
    }

    let mut results: Vec<(&str, QueryResults)> = Vec::with_capacity(engines.len());
    for engine in engines {
        match engine.select(query) {
            Ok(r) => results.push((engine.name(), r)),
            Err(failure) => return Verdict::EngineFailure(failure),
        }
    }

    let (reference_name, reference) = &results[0];
    for (name, candidate) in &results[1..] {
        let (agree, detail) = match (reference, candidate) {
            (
                QueryResults::Solutions { solutions: a, .. },
                QueryResults::Solutions { solutions: b, .. },
            ) => (
                multiset_equal(a, b),
                format!("{reference_name}={} rows, {name}={} rows", a.len(), b.len()),
            ),
            (QueryResults::Boolean(a), QueryResults::Boolean(b)) => {
                (a == b, format!("{reference_name}={a}, {name}={b}"))
            }
            _ => {
                return Verdict::EngineFailure(EngineFailure {
                    engine: (*name).to_string(),
                    query: query.to_string(),
                    kind: FailureKind::InvalidResults,
                    message: format!(
                        "result-shape mismatch vs {reference_name} (SELECT vs ASK shape)"
                    ),
                })
            }
        };
        if !agree {
            return Verdict::Violation(Violation {
                oracle: OracleKind::Differential,
                engines: vec![(*reference_name).to_string(), (*name).to_string()],
                queries: vec![query.to_string()],
                detail: format!("results disagree: {detail}"),
            });
        }
    }
    Verdict::Pass {
        detail: format!("{} engines agree", results.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Canned(QueryResults);
    impl SparqlEngine for Canned {
        fn name(&self) -> &str {
            "canned"
        }
        fn select(&self, _sparql: &str) -> Result<QueryResults, EngineFailure> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn fewer_than_two_engines_is_a_harness_failure_not_a_pass() {
        let one = Canned(QueryResults::Boolean(true));
        let verdict = check_differential(&[&one], "ASK { ?s ?p ?o }");
        match verdict {
            Verdict::EngineFailure(f) => assert_eq!(f.kind, FailureKind::Harness),
            other => panic!("expected a harness failure, got {other:?}"),
        }
    }

    #[test]
    fn boolean_agreement_passes_and_disagreement_is_a_violation() {
        let yes = Canned(QueryResults::Boolean(true));
        let also_yes = Canned(QueryResults::Boolean(true));
        let no = Canned(QueryResults::Boolean(false));
        assert!(check_differential(&[&yes, &also_yes], "ASK {}").is_pass());
        assert!(check_differential(&[&yes, &no], "ASK {}").is_violation());
    }

    #[test]
    fn shape_mismatch_is_an_engine_failure_not_a_wrong_result() {
        let boolean = Canned(QueryResults::Boolean(true));
        let solutions = Canned(QueryResults::Solutions {
            vars: vec![],
            solutions: vec![],
        });
        let verdict = check_differential(&[&boolean, &solutions], "ASK {}");
        match verdict {
            Verdict::EngineFailure(f) => assert_eq!(f.kind, FailureKind::InvalidResults),
            other => panic!("expected an engine failure, got {other:?}"),
        }
    }
}
