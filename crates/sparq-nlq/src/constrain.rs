//! N2 — dictionary-grounded constraint of generated SPARQL (`sq-9yjp`,
//! `research/genai-nl-to-sparql.md` §8.1, §6 step 2). [OPUS-4.8]
//!
//! Grammar-constrained *decoding* in the strict sense (mask the model's logits at
//! every step so only grammar-valid tokens are ever emitted) requires **logit
//! access** and is therefore only feasible on a *local* backend, or on an API that
//! accepts a supplied grammar (GBNF / structured-output). The crate's only live
//! backend — `crate::live::AnthropicLlm` — is the Anthropic Messages API, which
//! exposes **neither** logit bias **nor** a grammar parameter, so the strict variant
//! is not implementable against it (`research/genai-nl-to-sparql.md` §11: "needs
//! logit access — only works on local/open backends; API backends … fall back to
//! post-hoc validation"). Honest scope.
//!
//! What *is* both achievable and the part the bead literally names ("against the
//! **live dictionary**") is the design doc's prescribed fallback (§6 step 2): once a
//! candidate query parses, walk its algebra, collect every IRI used in **predicate**
//! or **class** position, and check each against the live dictionary
//! ([`sparq_core::Graph::id_of`]). An IRI absent from the dictionary cannot match any
//! triple, so the query is *valid-but-wrong* — it would execute to a silent empty (or
//! mis-grounded) result. We instead turn it into a **targeted repair signal**: the
//! unknown IRI plus the nearest known terms from the same namespace ("did you mean
//! `dbo:director`?"), candidates that are free from the dictionary. This is the
//! SPARQL-LLM repair pattern, sharpened by the store's own vocabulary.
//!
//! The syntactic half of constraint (does it parse at all?) is already covered by the
//! `spargebra` parse in [`crate::Nlq::ask`]; this module adds the **semantic**
//! (vocabulary) half.

use oxrdf::{NamedNode, Term};
use spargebra::algebra::{Expression, GraphPattern, PropertyPathExpression};
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use spargebra::Query;
use sparq_core::dict::TermParts;
use sparq_core::Graph;

/// `rdf:type` — the predicate whose *object* is a class IRI, the one object position
/// we treat as a vocabulary term (every other object is data, not schema).
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// How many "did you mean" candidates to surface per unknown term.
const MAX_SUGGESTIONS: usize = 3;

/// Where in the query an unknown IRI appeared — drives the wording of the repair hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermRole {
    /// Used as a triple/path predicate.
    Predicate,
    /// Used as the object of an `rdf:type` triple (a class).
    Class,
}

impl TermRole {
    fn label(self) -> &'static str {
        match self {
            TermRole::Predicate => "predicate",
            TermRole::Class => "class",
        }
    }
}

/// One IRI used in predicate/class position that is **absent from the live
/// dictionary** — so no triple can match it. Carries up to a few nearest known terms
/// from the same namespace as repair candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTerm {
    /// The offending IRI, exactly as written in the query.
    pub iri: String,
    /// Predicate vs class position.
    pub role: TermRole,
    /// Nearest known terms (same namespace, smallest edit distance first); possibly
    /// empty when the namespace itself is unknown to the store.
    pub suggestions: Vec<String>,
}

/// Walks the parsed `query` and returns every IRI used as a **predicate** or as an
/// `rdf:type` **object (class)** that is **not present in `graph`'s dictionary**, each
/// paired with nearest known terms (see [`UnknownTerm`]). De-duplicated; order is
/// stable (first appearance). An IRI that *is* in the dictionary is omitted — only the
/// genuinely-ungrounded vocabulary is reported.
///
/// Scope is deliberately predicate and class position: those are the vocabulary terms
/// a schema-grounded query must get right, and the two the design doc's repair step
/// targets. Subject/data-object IRIs (specific entities) are left to entity linking
/// (`sq-uw40`), not this vocabulary constraint.
pub fn unknown_terms(graph: &Graph, query: &Query) -> Vec<UnknownTerm> {
    let mut found: Vec<(String, TermRole)> = Vec::new();
    let mut seen: std::collections::HashSet<(String, TermRole)> = std::collections::HashSet::new();
    let mut push = |iri: &str, role: TermRole| {
        let key = (iri.to_string(), role);
        if seen.insert(key.clone()) {
            found.push(key);
        }
    };
    match query {
        Query::Select { pattern, .. }
        | Query::Ask { pattern, .. }
        | Query::Describe { pattern, .. } => walk_pattern(pattern, &mut push),
        Query::Construct {
            pattern, template, ..
        } => {
            walk_pattern(pattern, &mut push);
            for tp in template {
                walk_triple(tp, &mut push);
            }
        }
    }

    found
        .into_iter()
        .filter(|(iri, _)| !iri_in_dictionary(graph, iri))
        .map(|(iri, role)| {
            let suggestions = nearest_known(graph, &iri);
            UnknownTerm {
                iri,
                role,
                suggestions,
            }
        })
        .collect()
}

/// Renders the repair message handed back to the model for a non-empty set of unknown
/// terms — the dictionary-constraint feedback. Stable, deterministic wording (so a
/// repair prompt is reproducible / replayable).
pub fn dictionary_repair_message(unknowns: &[UnknownTerm]) -> String {
    let mut msg = String::from(
        "the query parses, but these terms are not in the dataset's dictionary, so they \
         match no triples:",
    );
    for u in unknowns {
        msg.push_str(&format!("\n- {} <{}>", u.role.label(), u.iri));
        if !u.suggestions.is_empty() {
            let hints: Vec<String> = u.suggestions.iter().map(|s| format!("<{s}>")).collect();
            msg.push_str(&format!(" — did you mean {}?", hints.join(", ")));
        }
    }
    msg.push_str("\nUse only predicates and classes that appear in the schema summary above.");
    msg
}

/// Is `iri` a term in the live dictionary? Uses the public [`Graph::id_of`] membership
/// test (absent IRI → no id → cannot match).
fn iri_in_dictionary(graph: &Graph, iri: &str) -> bool {
    match NamedNode::new(iri) {
        Ok(node) => graph.id_of(&Term::NamedNode(node)).is_some(),
        // A syntactically invalid IRI can never be in the dictionary; spargebra would
        // not have produced it as a NamedNode, but guard anyway.
        Err(_) => false,
    }
}

/// Nearest known IRIs to `iri` from the same namespace, smallest edit distance first,
/// capped at [`MAX_SUGGESTIONS`]. A single dictionary scan; only IRI terms sharing the
/// `iri`'s namespace are considered (suggestions across namespaces are noise) and only
/// those within edit distance ≤ the local-name length (a larger distance shares almost
/// nothing — noise too).
fn nearest_known(graph: &Graph, iri: &str) -> Vec<String> {
    let Some(split) = iri.rfind(['#', '/']) else {
        return Vec::new();
    };
    let prefix = &iri[..=split];
    let suffix = &iri[split + 1..];
    let cap = suffix.chars().count().max(1);

    let mut scored: Vec<(usize, String)> = Vec::new();
    for (_, parts) in graph.dict.iter() {
        let TermParts::Iri {
            prefix: p,
            suffix: s,
        } = parts
        else {
            continue;
        };
        if p != prefix || s == suffix {
            continue;
        }
        let d = edit_distance(suffix, s);
        if d <= cap {
            scored.push((d, format!("{p}{s}")));
        }
    }
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    scored.truncate(MAX_SUGGESTIONS);
    scored.into_iter().map(|(_, iri)| iri).collect()
}

/// Levenshtein edit distance (two rolling rows). Local names are short, so the
/// quadratic cost is trivial; no allocation beyond the two rows.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

// ---------------------------------------------------------------------------
// Algebra walk
// ---------------------------------------------------------------------------

fn walk_pattern<F: FnMut(&str, TermRole)>(p: &GraphPattern, f: &mut F) {
    match p {
        GraphPattern::Bgp { patterns } => {
            for tp in patterns {
                walk_triple(tp, f);
            }
        }
        GraphPattern::Path { path, .. } => walk_path(path, f),
        GraphPattern::Join { left, right }
        | GraphPattern::Union { left, right }
        | GraphPattern::Minus { left, right } => {
            walk_pattern(left, f);
            walk_pattern(right, f);
        }
        // `sep-0006` (lateral join) is always enabled in this workspace.
        GraphPattern::Lateral { left, right } => {
            walk_pattern(left, f);
            walk_pattern(right, f);
        }
        GraphPattern::LeftJoin {
            left,
            right,
            expression,
        } => {
            walk_pattern(left, f);
            walk_pattern(right, f);
            if let Some(e) = expression {
                walk_expr(e, f);
            }
        }
        GraphPattern::Filter { expr, inner } => {
            walk_expr(expr, f);
            walk_pattern(inner, f);
        }
        GraphPattern::Graph { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. }
        | GraphPattern::OrderBy { inner, .. }
        | GraphPattern::Project { inner, .. }
        | GraphPattern::Group { inner, .. } => walk_pattern(inner, f),
        GraphPattern::Extend {
            inner, expression, ..
        } => {
            walk_pattern(inner, f);
            walk_expr(expression, f);
        }
        // A SERVICE block targets a remote endpoint whose dictionary we do not hold;
        // its vocabulary is out of scope for the local-dictionary check.
        GraphPattern::Service { .. } => {}
        GraphPattern::Values { .. } => {}
    }
}

fn walk_triple<F: FnMut(&str, TermRole)>(tp: &TriplePattern, f: &mut F) {
    if let NamedNodePattern::NamedNode(p) = &tp.predicate {
        let p = p.as_str();
        f(p, TermRole::Predicate);
        // An `rdf:type` triple's object, when a fixed IRI, is a class.
        if p == RDF_TYPE {
            if let TermPattern::NamedNode(o) = &tp.object {
                f(o.as_str(), TermRole::Class);
            }
        }
    }
    // Quoted triple terms (RDF 1.2, `sparql-12`, always enabled here) can nest
    // predicates/classes; descend.
    if let TermPattern::Triple(inner) = &tp.subject {
        walk_triple(inner, f);
    }
    if let TermPattern::Triple(inner) = &tp.object {
        walk_triple(inner, f);
    }
}

fn walk_path<F: FnMut(&str, TermRole)>(path: &PropertyPathExpression, f: &mut F) {
    match path {
        PropertyPathExpression::NamedNode(n) => f(n.as_str(), TermRole::Predicate),
        PropertyPathExpression::Reverse(p)
        | PropertyPathExpression::ZeroOrMore(p)
        | PropertyPathExpression::OneOrMore(p)
        | PropertyPathExpression::ZeroOrOne(p) => walk_path(p, f),
        PropertyPathExpression::Sequence(a, b) | PropertyPathExpression::Alternative(a, b) => {
            walk_path(a, f);
            walk_path(b, f);
        }
        PropertyPathExpression::NegatedPropertySet(ns) => {
            for n in ns {
                f(n.as_str(), TermRole::Predicate);
            }
        }
    }
}

fn walk_expr<F: FnMut(&str, TermRole)>(e: &Expression, f: &mut F) {
    match e {
        Expression::Exists(p) => walk_pattern(p, f),
        Expression::Or(a, b)
        | Expression::And(a, b)
        | Expression::Equal(a, b)
        | Expression::SameTerm(a, b)
        | Expression::Greater(a, b)
        | Expression::GreaterOrEqual(a, b)
        | Expression::Less(a, b)
        | Expression::LessOrEqual(a, b)
        | Expression::Add(a, b)
        | Expression::Subtract(a, b)
        | Expression::Multiply(a, b)
        | Expression::Divide(a, b) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Expression::In(a, list) => {
            walk_expr(a, f);
            for e in list {
                walk_expr(e, f);
            }
        }
        Expression::UnaryPlus(a) | Expression::UnaryMinus(a) | Expression::Not(a) => {
            walk_expr(a, f)
        }
        Expression::FunctionCall(_, args) => {
            for a in args {
                walk_expr(a, f);
            }
        }
        Expression::If(a, b, c) => {
            walk_expr(a, f);
            walk_expr(b, f);
            walk_expr(c, f);
        }
        Expression::Coalesce(list) => {
            for e in list {
                walk_expr(e, f);
            }
        }
        // A literal/variable/IRI/bound leaf carries no nested schema vocabulary we
        // check (a bare IRI in expression position is a value, not a predicate/class).
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> Graph {
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            ex:alice a ex:Person ; rdfs:label "Alice" ; ex:knows ex:bob .
            ex:bob a ex:Person ; rdfs:label "Bob" .
        "#;
        Graph::load_str(ttl, "turtle").expect("graph parses")
    }

    fn parse(q: &str) -> Query {
        spargebra::SparqlParser::new()
            .parse_query(q)
            .expect("query parses")
    }

    #[test]
    fn known_predicate_and_class_yield_no_unknowns() {
        let g = graph();
        let q = parse(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX ex: <http://example.org/>\n\
             SELECT ?s WHERE { ?s rdf:type ex:Person ; ex:knows ?o }",
        );
        assert!(unknown_terms(&g, &q).is_empty());
    }

    #[test]
    fn unknown_predicate_is_flagged_with_same_namespace_suggestion() {
        let g = graph();
        // `ex:know` (typo for `ex:knows`) is absent.
        let q = parse(
            "PREFIX ex: <http://example.org/>\n\
             SELECT ?s WHERE { ?s ex:know ?o }",
        );
        let unknowns = unknown_terms(&g, &q);
        assert_eq!(unknowns.len(), 1);
        assert_eq!(unknowns[0].iri, "http://example.org/know");
        assert_eq!(unknowns[0].role, TermRole::Predicate);
        assert!(
            unknowns[0]
                .suggestions
                .contains(&"http://example.org/knows".to_string()),
            "expected ex:knows suggested, got {:?}",
            unknowns[0].suggestions
        );
    }

    #[test]
    fn unknown_class_in_rdf_type_object_is_flagged_as_class() {
        let g = graph();
        let q = parse(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX ex: <http://example.org/>\n\
             SELECT ?s WHERE { ?s rdf:type ex:Persons }",
        );
        let unknowns = unknown_terms(&g, &q);
        assert_eq!(unknowns.len(), 1);
        assert_eq!(unknowns[0].iri, "http://example.org/Persons");
        assert_eq!(unknowns[0].role, TermRole::Class);
        assert!(unknowns[0]
            .suggestions
            .contains(&"http://example.org/Person".to_string()));
    }

    #[test]
    fn property_path_predicate_is_checked() {
        let g = graph();
        let q = parse(
            "PREFIX ex: <http://example.org/>\n\
             SELECT ?s WHERE { ?s ex:knows+ ?o . ?s ex:nope* ?z }",
        );
        let unknowns = unknown_terms(&g, &q);
        assert_eq!(unknowns.len(), 1);
        assert_eq!(unknowns[0].iri, "http://example.org/nope");
    }

    #[test]
    fn unknown_namespace_gives_no_suggestions_but_still_flags() {
        let g = graph();
        let q = parse(
            "PREFIX foo: <http://other.example/>\n\
             SELECT ?s WHERE { ?s foo:bar ?o }",
        );
        let unknowns = unknown_terms(&g, &q);
        assert_eq!(unknowns.len(), 1);
        assert!(unknowns[0].suggestions.is_empty());
    }

    #[test]
    fn repair_message_lists_terms_and_hints() {
        let unknowns = vec![UnknownTerm {
            iri: "http://example.org/know".into(),
            role: TermRole::Predicate,
            suggestions: vec!["http://example.org/knows".into()],
        }];
        let msg = dictionary_repair_message(&unknowns);
        assert!(msg.contains("predicate <http://example.org/know>"));
        assert!(msg.contains("did you mean <http://example.org/knows>?"));
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("knows", "know"), 1);
        assert_eq!(edit_distance("Person", "Persons"), 1);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
