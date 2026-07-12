//! Bounded missing-answer explanation for a single basic graph pattern.
//!
//! [GPT-5.6] sq-lsp7k.17: [`why_not`] substitutes a caller-provided target
//! binding into every triple pattern and reports exactly the resulting triples
//! absent from the graph. More general SPARQL algebra is deliberately rejected.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use oxrdf::{NamedOrBlankNode, Term, Triple, Variable};
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use sparq_core::Graph;

/// One failing conjunct in a missing-answer explanation.
///
/// The original [`TriplePattern`] is retained for attribution to the BGP and
/// [`grounded`](Self::grounded) is the concrete triple proved absent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingPattern {
    pattern: TriplePattern,
    grounded: Triple,
}

impl MissingPattern {
    /// The original triple pattern, before target-binding substitution.
    pub fn pattern(&self) -> &TriplePattern {
        &self.pattern
    }

    /// The concrete triple absent from the graph.
    pub fn grounded(&self) -> &Triple {
        &self.grounded
    }
}

/// Why-not explanation failed closed before graph membership could be checked.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WhyNotError {
    /// The supplied algebra node is not a single basic graph pattern.
    UnsupportedAlgebra,
    /// A variable used by the BGP is absent from the target binding.
    MissingBinding(Variable),
    /// Substitution produced a term that cannot occupy an RDF triple's subject.
    InvalidSubject(Term),
    /// Substitution produced a term that is not an IRI in predicate position.
    InvalidPredicate(Term),
}

impl fmt::Display for WhyNotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedAlgebra => f.write_str(
                "why-not explanation supports one basic graph pattern only; other SPARQL algebra is unsupported",
            ),
            Self::MissingBinding(variable) => {
                write!(f, "target binding does not bind {variable}")
            }
            Self::InvalidSubject(term) => {
                write!(f, "target binding produces an invalid triple subject: {term}")
            }
            Self::InvalidPredicate(term) => {
                write!(f, "target binding produces a non-IRI triple predicate: {term}")
            }
        }
    }
}

impl Error for WhyNotError {}

/// Explains why a fully-ground target binding is absent from one BGP's answers.
///
/// Each BGP triple pattern is grounded with `target`. The returned vector
/// contains a pattern if and only if its concrete triple is absent from `graph`,
/// preserving the BGP's order. An empty vector therefore means every conjunct
/// is present and the target would be an answer to this BGP.
///
/// This bounded API deliberately accepts only [`GraphPattern::Bgp`]. `OPTIONAL`,
/// `UNION`, `FILTER`, property paths, named graphs, and all other algebra return
/// [`WhyNotError::UnsupportedAlgebra`]. A missing variable binding or an RDF-term
/// position error also fails closed instead of inventing an explanation.
pub fn why_not(
    graph: &Graph,
    algebra: &GraphPattern,
    target: &HashMap<Variable, Term>,
) -> Result<Vec<MissingPattern>, WhyNotError> {
    let GraphPattern::Bgp { patterns } = algebra else {
        return Err(WhyNotError::UnsupportedAlgebra);
    };

    let mut missing = Vec::new();
    for pattern in patterns {
        let grounded = ground_triple(pattern, target)?;
        if !contains(graph, &grounded) {
            missing.push(MissingPattern {
                pattern: pattern.clone(),
                grounded,
            });
        }
    }
    Ok(missing)
}

fn ground_triple(
    pattern: &TriplePattern,
    target: &HashMap<Variable, Term>,
) -> Result<Triple, WhyNotError> {
    let subject_term = ground_term(&pattern.subject, target)?;
    let subject = match subject_term {
        Term::NamedNode(node) => NamedOrBlankNode::NamedNode(node),
        Term::BlankNode(node) => NamedOrBlankNode::BlankNode(node),
        invalid => return Err(WhyNotError::InvalidSubject(invalid)),
    };

    let predicate = match &pattern.predicate {
        NamedNodePattern::NamedNode(node) => node.clone(),
        NamedNodePattern::Variable(variable) => match binding(target, variable)? {
            Term::NamedNode(node) => node,
            invalid => return Err(WhyNotError::InvalidPredicate(invalid)),
        },
    };

    Ok(Triple::new(
        subject,
        predicate,
        ground_term(&pattern.object, target)?,
    ))
}

fn ground_term(
    pattern: &TermPattern,
    target: &HashMap<Variable, Term>,
) -> Result<Term, WhyNotError> {
    match pattern {
        TermPattern::NamedNode(node) => Ok(Term::NamedNode(node.clone())),
        TermPattern::BlankNode(node) => Ok(Term::BlankNode(node.clone())),
        TermPattern::Literal(literal) => Ok(Term::Literal(literal.clone())),
        TermPattern::Triple(triple) => Ok(Term::Triple(Box::new(ground_triple(triple, target)?))),
        TermPattern::Variable(variable) => binding(target, variable),
    }
}

fn binding(target: &HashMap<Variable, Term>, variable: &Variable) -> Result<Term, WhyNotError> {
    target
        .get(variable)
        .cloned()
        .ok_or_else(|| WhyNotError::MissingBinding(variable.clone()))
}

fn contains(graph: &Graph, triple: &Triple) -> bool {
    let subject = match &triple.subject {
        NamedOrBlankNode::NamedNode(node) => Term::NamedNode(node.clone()),
        NamedOrBlankNode::BlankNode(node) => Term::BlankNode(node.clone()),
    };
    let Some([Some(subject), Some(predicate), Some(object)]) = graph.pattern(
        Some(&subject),
        Some(&triple.predicate),
        Some(&triple.object),
    ) else {
        return false;
    };
    graph.store.contains([subject, predicate, object])
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{Literal, NamedNode};

    fn iri(local: &str) -> NamedNode {
        NamedNode::new_unchecked(format!("http://example.com/{local}"))
    }

    fn variable(name: &str) -> Variable {
        Variable::new_unchecked(name)
    }

    fn variable_term(name: &str) -> TermPattern {
        TermPattern::Variable(variable(name))
    }

    fn iri_term(local: &str) -> TermPattern {
        TermPattern::NamedNode(iri(local))
    }

    fn pattern(subject: TermPattern, predicate: &str, object: TermPattern) -> TriplePattern {
        TriplePattern {
            subject,
            predicate: NamedNodePattern::NamedNode(iri(predicate)),
            object,
        }
    }

    #[test]
    fn mutation_witness_reports_exact_failing_conjunct_set() {
        let graph =
            Graph::load_str("@prefix : <http://example.com/> . :a :p :b .", "turtle").unwrap();
        let present = pattern(variable_term("x"), "p", variable_term("y"));
        let absent = pattern(variable_term("x"), "q", variable_term("z"));
        let algebra = GraphPattern::Bgp {
            patterns: vec![present.clone(), absent.clone()],
        };
        let target = HashMap::from([
            (variable("x"), Term::NamedNode(iri("a"))),
            (variable("y"), Term::NamedNode(iri("b"))),
            (variable("z"), Term::NamedNode(iri("c"))),
        ]);

        let explanation = why_not(&graph, &algebra, &target).unwrap();

        assert_eq!(explanation.len(), 1);
        assert_eq!(explanation[0].pattern(), &absent);
        assert_ne!(explanation[0].pattern(), &present);
        assert_eq!(
            explanation[0].grounded(),
            &Triple::new(iri("a"), iri("q"), iri("c"))
        );
    }

    #[test]
    fn all_present_returns_empty() {
        let graph = Graph::load_str(
            "@prefix : <http://example.com/> . :a :p :b; :q :c .",
            "turtle",
        )
        .unwrap();
        let algebra = GraphPattern::Bgp {
            patterns: vec![
                pattern(variable_term("x"), "p", variable_term("y")),
                pattern(variable_term("x"), "q", variable_term("z")),
            ],
        };
        let target = HashMap::from([
            (variable("x"), Term::NamedNode(iri("a"))),
            (variable("y"), Term::NamedNode(iri("b"))),
            (variable("z"), Term::NamedNode(iri("c"))),
        ]);

        assert!(why_not(&graph, &algebra, &target).unwrap().is_empty());
    }

    #[test]
    fn reports_every_absent_pattern_in_bgp_order() {
        let graph = Graph::default();
        let first = pattern(iri_term("a"), "p", iri_term("b"));
        let second = pattern(iri_term("a"), "q", iri_term("c"));
        let algebra = GraphPattern::Bgp {
            patterns: vec![first.clone(), second.clone()],
        };

        let explanation = why_not(&graph, &algebra, &HashMap::new()).unwrap();

        assert_eq!(
            explanation
                .iter()
                .map(MissingPattern::pattern)
                .collect::<Vec<_>>(),
            vec![&first, &second]
        );
    }

    #[test]
    fn unsupported_algebra_fails_closed() {
        let algebra = GraphPattern::Union {
            left: Box::new(GraphPattern::Bgp { patterns: vec![] }),
            right: Box::new(GraphPattern::Bgp { patterns: vec![] }),
        };

        assert_eq!(
            why_not(&Graph::default(), &algebra, &HashMap::new()),
            Err(WhyNotError::UnsupportedAlgebra)
        );
    }

    #[test]
    fn incomplete_or_invalid_target_fails_closed() {
        let subject_variable = variable("s");
        let predicate_variable = variable("p");
        let algebra = GraphPattern::Bgp {
            patterns: vec![TriplePattern {
                subject: TermPattern::Variable(subject_variable.clone()),
                predicate: NamedNodePattern::Variable(predicate_variable.clone()),
                object: iri_term("o"),
            }],
        };

        assert_eq!(
            why_not(&Graph::default(), &algebra, &HashMap::new()),
            Err(WhyNotError::MissingBinding(subject_variable.clone()))
        );

        let invalid_subject = HashMap::from([
            (
                subject_variable.clone(),
                Term::Literal(Literal::new_simple_literal("not-a-subject")),
            ),
            (predicate_variable.clone(), Term::NamedNode(iri("p"))),
        ]);
        assert!(matches!(
            why_not(&Graph::default(), &algebra, &invalid_subject),
            Err(WhyNotError::InvalidSubject(Term::Literal(_)))
        ));

        let target = HashMap::from([
            (subject_variable, Term::NamedNode(iri("s"))),
            (
                predicate_variable,
                Term::Literal(Literal::new_simple_literal("not-an-iri")),
            ),
        ]);
        assert!(matches!(
            why_not(&Graph::default(), &algebra, &target),
            Err(WhyNotError::InvalidPredicate(Term::Literal(_)))
        ));
    }
}
