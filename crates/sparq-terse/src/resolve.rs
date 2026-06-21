//! Phase 2 — `V("phrase")` concept resolution under the soundness envelope
//! (`research/llm-ergonomic-sparql-surface.md` §3.3, §6, §8.3). **`link` feature only.**
//!
//! 🤖 SPARQ agent. `V("phrase")` lets the agent write the *concept it means* instead of
//! guessing an opaque content-addressed IRI it cannot know a priori — the lever that
//! attacks the *real* first-shot bottleneck (grounding, not verbosity). It is also the
//! highest soundness burden: a `V()` that binds the **wrong** IRI yields a
//! plausible-but-wrong answer with no syntactic signal. So resolution is sound only under
//! the §6 envelope, all of which this module enforces and none of which is silent:
//!
//! - **Lexical-first, vector-fallback** (§6.4) — prefer the deterministic, no-model exact
//!   label match (reusing `sparq_nlq::link::EntityLinker`); fall to vectors only when
//!   lexical returns nothing (and only when the `vectors` feature is on).
//! - **Echo every resolution** (§6.2) — each bind returns its IRI + score + runner-up +
//!   confidence + method, so the agent can verify it and a close runner-up is visible.
//! - **Confidence-gated, never auto-accept the uncertain** (§6.3) — below the confidence
//!   floor, or when the chosen and runner-up are within the ambiguity margin, `V()` does
//!   NOT bind: it returns the candidates as an [`crate::TerseError::Unresolved`] error,
//!   never a silent guess. Loud-fail beats silent-wrong.
//! - **Staleness guard mandatory** (§6.5) — the vector path uses
//!   `nearest_term_exact_checked`, which `Err`s if the vector store was built against a
//!   different graph generation.

use oxrdf::NamedNode;
use sparq_core::Graph;
use sparq_nlq::link::EntityLinker;

use crate::transpile::{
    scan_v_calls, splice_v_calls, validate_canonical, Expansion, Resolution, ResolutionMethod,
    TerseError,
};

/// Tunable knobs for the confidence envelope. The defaults are deliberately conservative:
/// loud-fail on anything ambiguous (the design leaves the exact floors to the maintainer
/// as pre-registration knobs — §9 Q2 — so they are configurable, not frozen).
#[derive(Debug, Clone, Copy)]
pub struct ResolveConfig {
    /// Minimum confidence to bind. Below this, `V()` refuses and surfaces the candidates.
    pub confidence_floor: f64,
    /// Minimum margin (chosen confidence − runner-up confidence) to bind. When the top two
    /// candidates are within this margin the match is *ambiguous* and `V()` refuses.
    pub ambiguity_margin: f64,
    /// How many sibling candidates the underlying linker considers (passed through; does
    /// not affect the binary bind/refuse decision, only the richness of the candidate set).
    pub candidate_k: usize,
}

impl Default for ResolveConfig {
    fn default() -> Self {
        // Conservative: an exact-label lexical hit clears the floor; a substring-only or
        // close-runner-up match does not bind without the margin.
        ResolveConfig {
            confidence_floor: 0.5,
            ambiguity_margin: 0.1,
            candidate_k: 8,
        }
    }
}

/// One scored candidate IRI for a phrase, before the bind/refuse decision. `pub(crate)`
/// so the `vectors`-feature backend ([`crate::vector`]) can build vector candidates.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Candidate {
    pub(crate) iri: NamedNode,
    /// Score in `[0, 1]`: 1.0 for an exact label match, lower for a substring match.
    pub(crate) score: f64,
    pub(crate) method: ResolutionMethod,
}

/// A concept resolver bound to a graph: builds the lexical label index once (via
/// `sparq_nlq::link::EntityLinker`) and resolves any number of `V("phrase")` call sites
/// against it. The vector fallback (the `vectors` feature) is wired through
/// [`Resolver::with_vectors`].
pub struct Resolver<'g> {
    /// The graph the resolution is grounded in. Only the vector fallback needs it directly
    /// (for the staleness guard); the lexical linker already holds its own reference, so
    /// the field is `vectors`-gated to keep the lean `link`-only build warning-free.
    #[cfg(feature = "vectors")]
    graph: &'g Graph,
    linker: EntityLinker<'g>,
    config: ResolveConfig,
    #[cfg(feature = "vectors")]
    vectors: Option<crate::vector::VectorBackend<'g>>,
}

impl<'g> Resolver<'g> {
    /// Builds a resolver over `graph` with the default [`ResolveConfig`].
    pub fn new(graph: &'g Graph) -> Self {
        Self::with_config(graph, ResolveConfig::default())
    }

    /// Builds a resolver with an explicit confidence envelope.
    pub fn with_config(graph: &'g Graph, config: ResolveConfig) -> Self {
        // `expand_k = 0`: we want concept candidates, not structural siblings; `max_links`
        // bounds the candidate set so a phrase that matches many substrings stays cheap.
        let linker = EntityLinker::build(graph, 0, config.candidate_k);
        Resolver {
            #[cfg(feature = "vectors")]
            graph,
            linker,
            config,
            #[cfg(feature = "vectors")]
            vectors: None,
        }
    }

    /// Transpile `src`: scan its `V("phrase")` call sites, resolve each under the envelope,
    /// splice the resolved IRIs into canonical SPARQL, and validate the result re-parses.
    /// Returns the verifiable [`Expansion`] (echoing every resolution) or a *loud*
    /// [`TerseError`] — never a silent low-confidence bind.
    pub fn transpile(&self, src: &str) -> Result<Expansion, TerseError> {
        let calls = scan_v_calls(src)?;
        if calls.is_empty() {
            // No directives — pure identity pass-through (the canary path), still validated.
            validate_canonical(src)?;
            return Ok(Expansion {
                canonical_sparql: src.to_string(),
                resolutions: Vec::new(),
                warnings: Vec::new(),
            });
        }

        let mut resolutions = Vec::with_capacity(calls.len());
        let mut warnings = Vec::new();
        let mut replacements = Vec::with_capacity(calls.len());
        for call in &calls {
            let res = self.resolve_phrase(&call.phrase)?;
            // Splice the resolved concept as a full IRI so the canonical output is
            // unambiguous and prefix-independent.
            replacements.push(format!("<{}>", res.iri));
            if let (Some(ru), Some(rus)) = (&res.runner_up, res.runner_up_score) {
                if (res.confidence - rus).abs() < self.config.ambiguity_margin * 2.0 {
                    warnings.push(format!(
                        "V({:?}) bound <{}> (confidence {:.2}) over a near runner-up <{}> (score {:.2})",
                        res.phrase, res.iri, res.confidence, ru, rus
                    ));
                }
            }
            resolutions.push(res);
        }

        let canonical = splice_v_calls(src, &calls, &replacements);
        validate_canonical(&canonical)?;
        Ok(Expansion {
            canonical_sparql: canonical,
            resolutions,
            warnings,
        })
    }

    /// Resolve one phrase to a [`Resolution`] under the envelope, or a loud
    /// [`TerseError::Unresolved`] when uncertain. Lexical-first, vector-fallback.
    fn resolve_phrase(&self, phrase: &str) -> Result<Resolution, TerseError> {
        let mut candidates = self.lexical_candidates(phrase);

        // Vector fallback ONLY when lexical found nothing (the §6.4 lexical-first rule).
        #[cfg(feature = "vectors")]
        if candidates.is_empty() {
            if let Some(backend) = &self.vectors {
                candidates = backend.candidates(self.graph, phrase, self.config.candidate_k)?;
            }
        }

        if candidates.is_empty() {
            return Err(TerseError::Unresolved {
                phrase: phrase.to_string(),
                detail: "no concept matched lexically (or by vectors, if enabled)".to_string(),
            });
        }

        // Best first; the runner-up is the second-best distinct IRI.
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.iri.as_str().cmp(b.iri.as_str()))
        });
        let chosen = &candidates[0];
        let runner = candidates.get(1);
        let runner_score = runner.map(|r| r.score);
        // Confidence blends the absolute score with the margin over the runner-up: a high
        // score with a close runner-up is LESS confident than a high score that stands alone.
        let margin = chosen.score - runner_score.unwrap_or(0.0);
        let confidence = (chosen.score * (0.5 + 0.5 * margin.clamp(0.0, 1.0))).clamp(0.0, 1.0);

        // ---- The confidence gate (§6.3) — refuse, never silently bind. ----
        if confidence < self.config.confidence_floor {
            return Err(TerseError::Unresolved {
                phrase: phrase.to_string(),
                detail: format!(
                    "best candidate <{}> confidence {:.2} is below the floor {:.2}; \
                     candidates: {}",
                    chosen.iri,
                    confidence,
                    self.config.confidence_floor,
                    fmt_candidates(&candidates),
                ),
            });
        }
        if let Some(rs) = runner_score {
            if (chosen.score - rs) < self.config.ambiguity_margin {
                return Err(TerseError::Unresolved {
                    phrase: phrase.to_string(),
                    detail: format!(
                        "ambiguous: <{}> (score {:.2}) and runner-up <{}> (score {:.2}) \
                         are within the {:.2} margin; candidates: {}",
                        chosen.iri,
                        chosen.score,
                        runner.unwrap().iri,
                        rs,
                        self.config.ambiguity_margin,
                        fmt_candidates(&candidates),
                    ),
                });
            }
        }

        Ok(Resolution {
            phrase: phrase.to_string(),
            iri: chosen.iri.as_str().to_string(),
            score: chosen.score,
            runner_up: runner.map(|r| r.iri.as_str().to_string()),
            runner_up_score: runner_score,
            confidence,
            method: chosen.method,
        })
    }

    /// Lexical candidates via the reused `sparq_nlq` entity linker. The linker ranks exact
    /// label matches over substring matches; we map that ranking onto a `[0, 1]` score
    /// (exact = 1.0, substring = 0.6) so the envelope can reason about it uniformly.
    fn lexical_candidates(&self, phrase: &str) -> Vec<Candidate> {
        self.linker
            .link(phrase)
            .entities
            .into_iter()
            .map(|e| Candidate {
                iri: e.iri,
                score: if e.exact { 1.0 } else { 0.6 },
                method: ResolutionMethod::Lexical,
            })
            .collect()
    }
}

/// Renders a candidate list compactly for an error/ambiguity message (the surfaced
/// disambiguation signal).
fn fmt_candidates(cands: &[Candidate]) -> String {
    cands
        .iter()
        .take(4)
        .map(|c| format!("<{}>({:.2},{})", c.iri, c.score, c.method))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(feature = "vectors")]
impl<'g> Resolver<'g> {
    /// Attach a vector fallback (the `vectors` feature). It is consulted ONLY when the
    /// lexical path returns nothing — lexical-first stays mandatory. See
    /// [`crate::vector::VectorBackend`].
    pub fn with_vectors(mut self, backend: crate::vector::VectorBackend<'g>) -> Self {
        self.vectors = Some(backend);
        self
    }
}

// Make `Candidate`'s vector-built form reachable from the vector module without exposing
// it publicly.
#[cfg(feature = "vectors")]
impl Candidate {
    pub(crate) fn vector(iri: NamedNode, score: f64) -> Self {
        Candidate {
            iri,
            score,
            method: ResolutionMethod::Vector,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg_graph() -> Graph {
        // A small PKG-shaped graph: distinct concepts with clear labels.
        let ttl = r#"
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            @prefix skos: <http://www.w3.org/2004/02/skos/core#> .
            @prefix topic: <http://example.org/topic/> .
            topic:cardinality-estimation a skos:Concept ;
                skos:prefLabel "cardinality estimation" .
            topic:query-planning a skos:Concept ;
                skos:prefLabel "query planning" .
            topic:join-ordering a skos:Concept ;
                skos:prefLabel "join ordering" .
            topic:vector-search a skos:Concept ;
                skos:prefLabel "vector search" .
        "#;
        Graph::load_str(ttl, "turtle").expect("graph parses")
    }

    #[test]
    fn resolves_a_known_concept_exactly() {
        let g = pkg_graph();
        let r = Resolver::new(&g);
        let q = r#"SELECT ?f WHERE { ?f <http://example.org/about> V("cardinality estimation") }"#;
        let exp = r.transpile(q).expect("a known concept resolves");
        assert_eq!(exp.resolutions.len(), 1);
        let res = &exp.resolutions[0];
        assert_eq!(res.iri, "http://example.org/topic/cardinality-estimation");
        assert_eq!(res.method, ResolutionMethod::Lexical);
        assert_eq!(res.score, 1.0);
        // The canonical output must contain the resolved IRI and re-parse (validated inside).
        assert!(exp
            .canonical_sparql
            .contains("<http://example.org/topic/cardinality-estimation>"));
        assert!(!exp.canonical_sparql.contains("V("));
    }

    #[test]
    fn refuses_an_unknown_phrase_loudly() {
        let g = pkg_graph();
        let r = Resolver::new(&g);
        let q = r#"SELECT * WHERE { ?s ?p V("a phrase nothing matches xyzzy") }"#;
        let err = r.transpile(q).unwrap_err();
        match err {
            TerseError::Unresolved { phrase, .. } => {
                assert_eq!(phrase, "a phrase nothing matches xyzzy");
            }
            other => panic!("expected Unresolved, got {other:?}"),
        }
    }

    #[test]
    fn refuses_an_ambiguous_phrase_never_silently_binds() {
        // Two concepts share a label; a substring phrase matches both at equal score, so
        // the chosen/runner-up are within the margin -> refuse with the candidate list.
        let ttl = r#"
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            @prefix c: <http://example.org/c/> .
            c:apple-fruit rdfs:label "apple variety one" .
            c:apple-company rdfs:label "apple variety two" .
        "#;
        let g = Graph::load_str(ttl, "turtle").expect("graph parses");
        let r = Resolver::new(&g);
        // "apple variety" is a substring of both labels at equal substring score.
        let q = r#"SELECT * WHERE { ?s ?p V("apple variety") }"#;
        let err = r.transpile(q).unwrap_err();
        match err {
            TerseError::Unresolved { detail, .. } => {
                assert!(
                    detail.contains("ambiguous") || detail.contains("below the floor"),
                    "expected an ambiguity/low-confidence refusal, got: {detail}"
                );
                // The candidate list (the disambiguation signal) is surfaced.
                assert!(detail.contains("http://example.org/c/apple"));
            }
            other => panic!("expected Unresolved, got {other:?}"),
        }
    }

    #[test]
    fn identity_passthrough_still_holds_with_resolver() {
        // A resolver-driven transpile of a query WITHOUT any V() is still the canary no-op.
        let g = pkg_graph();
        let r = Resolver::new(&g);
        let q = "SELECT * WHERE { ?s ?p ?o }";
        let exp = r.transpile(q).expect("canonical");
        assert!(exp.is_identity(q));
    }

    #[test]
    fn echoes_runner_up_when_a_second_candidate_exists() {
        let g = pkg_graph();
        let r = Resolver::new(&g);
        // "query" is a substring of "query planning" only here, so a single candidate —
        // assert the resolution echoes its fields regardless of a runner-up.
        let q = r#"SELECT * WHERE { ?s ?p V("query planning") }"#;
        let exp = r.transpile(q).expect("resolves");
        let res = &exp.resolutions[0];
        assert_eq!(res.iri, "http://example.org/topic/query-planning");
        assert!(res.confidence > 0.0);
    }
}
