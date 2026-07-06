//! [OPUS-4.8] sq-6qw3: the opt-in `Reasoner::why(...)` derivation proof tree.
//!
//! Behind the non-default `explain` feature (which turns on `sparq-reason`'s `explain`
//! feature), so the standard reason bundle carries zero proof-tree code. Exposes
//! `sparq-reason`'s existing `MaterializedGraph::why` / `MaterializedOwlGraph::why` to JS as
//! `Reasoner.why(data, format, profile, s, p, o)`, returning the proof tree as JSON
//! (`ProofTree::to_json` — the same flat, premises-before-conclusion DAG the native API and
//! the ZK derivation circuit consume).
//!
//! The queried triple is supplied as three N-Triples term strings (`s`, `p`, `o`), exactly
//! the form the materialize/entailed N-Triples output uses, so the inference page can let the
//! user click an entailed triple and ask "why?".

use sparq_core::dict::{Dict, Id, NO_ID};
use sparq_core::Graph;
use sparq_reason::{ExplainOpts, MaterializedGraph, MaterializedOwlGraph, ProofTree};
use wasm_bindgen::prelude::*;

use crate::Reasoner;

/// Resolves the three N-Triples term strings of the queried triple to ids in `dict` (the
/// closure's dictionary). Returns `None` (so `why` answers `null`) if any term is absent from
/// the dictionary — i.e. it cannot be in the closure, so there is nothing to explain.
///
/// The terms are recovered by parsing the one-line N-Triples document `s p o .` into a
/// throwaway dictionary (reusing the tested parser rather than a hand-rolled term lexer), then
/// looking each parsed term UP in the closure dictionary via [`Dict::lookup`].
fn resolve_triple(dict: &Dict, s: &str, p: &str, o: &str) -> Option<[Id; 3]> {
    let line = format!("{s} {p} {o} .");
    let (scratch, parsed) = Graph::parse_to_triples(&line, "ntriples").ok()?;
    let [si, pi, oi] = *parsed.first()?;
    // Render each id back to its term against the SCRATCH dict, then look it up in the
    // closure dict — the two dictionaries assign different ids to the same term, so we must
    // translate through the term, not the id.
    let resolve = |scratch_id: Id| -> Option<Id> {
        let term = scratch.term(scratch_id);
        let id = dict.lookup(&term);
        if id == NO_ID {
            None
        } else {
            Some(id)
        }
    };
    Some([resolve(si)?, resolve(pi)?, resolve(oi)?])
}

/// Serialises a `why()` result to JSON: the proof tree's house-style JSON, or the JSON literal
/// `null` when the triple is not in the closure (no derivation).
fn proof_to_json(proof: Option<ProofTree>) -> String {
    match proof {
        Some(tree) => tree.to_json(),
        None => "null".to_string(),
    }
}

#[wasm_bindgen]
impl Reasoner {
    /// [OPUS-4.8] sq-6qw3: returns ONE derivation of the triple `(s, p, o)` from the asserted
    /// base of `data` under `profile`, as a proof-tree JSON document — or the JSON literal
    /// `null` if the triple is not entailed (not in the closure). `s`/`p`/`o` are N-Triples
    /// term strings (e.g. `"<http://ex/Socrates>"`, `"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"`,
    /// `"<http://ex/Mortal>"`), matching the form [`Reasoner::materialize`](crate::Reasoner::materialize)
    /// emits, so a UI can let the user click an entailed triple and ask "why?".
    ///
    /// The JSON is `sparq-reason`'s [`ProofTree::to_json`] shape:
    /// `{"root":R,"nodes":[{"id":i,"conclusion":[s,p,o],"rule":"…","premises":[…]},…]}` — a
    /// flat, premises-before-conclusion DAG (leaves are `"asserted"` base facts; internal
    /// nodes name the rule that fired). `JSON.parse` it on the JS side and render the tree.
    /// Only available when this bundle is built with the `explain` feature.
    ///
    /// Errors if the document fails to parse or the profile name is unknown.
    pub fn why(
        data: &str,
        format: &str,
        profile: &str,
        s: &str,
        p: &str,
        o: &str,
    ) -> Result<String, JsError> {
        let profile = crate::parse_profile(profile)?;
        let (mut dict, base) =
            Graph::parse_to_triples(data, format).map_err(|e| JsError::new(&e))?;
        // Resolve the queried triple's ids against the closure dictionary. An unknown term
        // means the triple cannot be in the closure -> `null` (no derivation).
        let Some(triple) = resolve_triple(&dict, s, p, o) else {
            return Ok("null".to_string());
        };
        let opts = ExplainOpts::default();
        let proof = match profile {
            sparq_reason::Profile::Rdfs => {
                let g = MaterializedGraph::new(&mut dict, &base);
                g.why_with(&dict, triple, opts)
            }
            sparq_reason::Profile::OwlRl => {
                let g = MaterializedOwlGraph::new(&mut dict, &base);
                g.why_with(&dict, triple, opts)
            }
            // [OPUS-4.8] sq-e5atd: D-entailment is a value-space materialiser (the rdfD1
            // datatype-typing closure with typed literal equality), not a rule-based
            // forward-chainer that records a premises-before-conclusion derivation DAG, so
            // there is no `why` proof tree for it. `Profile::D` only exists when the bundle is
            // built with the `d-entail` feature; gate the arm on the same cfg so the match is
            // exhaustive in BOTH feature states.
            #[cfg(feature = "d-entail")]
            sparq_reason::Profile::D => {
                return Err(JsError::new(
                    "why() is not available for the \"d\" (D-entailment) profile: D-entailment \
                     is a value-space datatype materialiser, not a rule-based derivation, so it \
                     has no proof tree. Use materialize/entailed for the D closure, or why() \
                     with the \"rdfs\"/\"owl-rl\" profile.",
                ));
            }
        };
        Ok(proof_to_json(proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_reason::Profile;

    const SOCRATES_TTL: &str = r#"@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix ex: <http://ex/> .
        ex:Human rdfs:subClassOf ex:Mortal .
        ex:Socrates a ex:Human ."#;

    // As with the lean bundle, the `JsError`-returning wasm wrapper cannot run natively, so
    // we assert against the resolver + the `MaterializedGraph::why` it delegates to; the
    // headless `wasm-pack test --node` suite drives the real #[wasm_bindgen] `why`.

    /// `why` produces a proof tree for an entailed triple, whose root conclusion IS the
    /// queried triple and whose leaves are asserted facts.
    #[test]
    fn why_explains_entailed_triple() {
        let (mut dict, base) = Graph::parse_to_triples(SOCRATES_TTL, "turtle").unwrap();
        let triple = resolve_triple(
            &dict,
            "<http://ex/Socrates>",
            "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>",
            "<http://ex/Mortal>",
        )
        .expect("entailed triple's terms must all be in the closure dict");
        let g = MaterializedGraph::new(&mut dict, &base);
        let proof = g
            .why_with(&dict, triple, ExplainOpts::default())
            .expect("the entailed Mortal typing must have a derivation");
        let json = proof.to_json();
        // The root conclusion is the queried triple.
        assert!(json.contains("\"root\":"), "{json}");
        assert!(
            json.contains("http://ex/Mortal"),
            "proof must conclude the Mortal typing: {json}"
        );
        // At least one leaf is an asserted fact.
        assert!(
            json.contains("\"asserted\""),
            "proof must bottom out in asserted facts: {json}"
        );
    }

    /// A triple that is NOT in the closure resolves to `null` (no derivation), and a triple
    /// whose terms are entirely absent from the data resolves to `None` before any prover runs.
    #[test]
    fn why_null_for_absent() {
        let (dict, _base) = Graph::parse_to_triples(SOCRATES_TTL, "turtle").unwrap();
        // A term not present anywhere in the data -> resolve_triple returns None.
        let absent = resolve_triple(
            &dict,
            "<http://ex/NotInData>",
            "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>",
            "<http://ex/Mortal>",
        );
        assert!(absent.is_none(), "an absent subject term cannot resolve");
        assert_eq!(proof_to_json(None), "null");
    }

    /// The OWL-RL prover path also answers `why` (it includes the RDFS rules), so the
    /// dispatch on `Profile::OwlRl` is exercised, not just the RDFS branch.
    #[test]
    fn why_owl_rl_path() {
        let (mut dict, base) = Graph::parse_to_triples(SOCRATES_TTL, "turtle").unwrap();
        let triple = resolve_triple(
            &dict,
            "<http://ex/Socrates>",
            "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>",
            "<http://ex/Mortal>",
        )
        .unwrap();
        let g = MaterializedOwlGraph::new(&mut dict, &base);
        let proof = g.why_with(&dict, triple, ExplainOpts::default());
        assert!(
            proof.is_some(),
            "OWL-RL (which includes RDFS) must also derive the Mortal typing"
        );
        // Sanity: the profile enum round-trips so the dispatch can't silently mis-route.
        assert_eq!(Profile::parse("owl-rl"), Some(Profile::OwlRl));
    }
}
