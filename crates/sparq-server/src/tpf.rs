//! [OPUS-4.8] sq-bzh1 (epic sq-3183): OPT-IN Triple Pattern Fragments (TPF) / Linked Data
//! Fragments server endpoint.
//!
//! A [Triple Pattern Fragments](https://www.hydra-cg.com/spec/latest/triple-pattern-fragments/)
//! (Hartig & Verborgh, [LDF](http://linkeddatafragments.org/)) interface turns `sparq-server`
//! into a *low-cost federation SOURCE*: instead of accepting a full SPARQL query, the endpoint
//! answers a single **triple pattern** (`?s ?p ?o` with any of the three positions bound) and
//! returns a *page* of the matching triples plus Hydra controls so a client-side TPF query
//! engine (Comunica, the LDF client) can drive a join itself. This pushes far less work onto
//! the server than a full SPARQL endpoint — the per-request cost is bounded by ONE page plus a
//! cheap cardinality estimate — which is exactly why TPF exists.
//!
//! ## What this module is
//!
//! A PURE (no async, no HTTP types) builder — like [`crate::graph`] / [`crate::descriptors`] —
//! that, given a parsed triple pattern, a pinned [`Graph`] snapshot and a page number,
//! produces the response **triples** (the page of data triples + the Hydra metadata/control
//! triples) ready for the crate's existing graph serialisers ([`crate::graph::triples_to_turtle`]
//! / [`triples_to_ntriples`](crate::graph::triples_to_ntriples)). The async handler that wires
//! it onto the axum router and content-negotiates the serialisation lives in
//! [`crate::http`] (`tpf_endpoint`). Keeping the logic here makes the pattern-match, the
//! `hydra:totalItems` estimate, and the paging boundary unit-testable without booting a server.
//!
//! ## Pattern selection + the cardinality estimate (cheap, not a full scan)
//!
//! The three positions come from the `subject` / `predicate` / `object` query parameters, each
//! an **N-Triples term** (`<iri>`, `"lit"`, `"lit"@en`, `"lit"^^<dt>`); an absent or empty
//! parameter is a *variable* (unbound). The pattern resolves through the public
//! [`Graph::pattern`] dictionary lookup, then:
//!
//!   * `hydra:totalItems` (and the void:triples count on the metadata node) is the engine's
//!     **cardinality ESTIMATE** for the pattern ([`sparq_core::store::TripleStore::estimate`]) —
//!     the same cheap index-range count the query planner uses (binary-search bounds on the raw
//!     indexes; two boundary blocks on the compressed mode), NEVER a full scan. On sparq's
//!     in-memory permutation indexes this estimate is in fact EXACT, but we advertise it as an
//!     estimate (the TPF spec's `void:triples` is explicitly an estimate) so the contract holds
//!     if a future storage mode makes it approximate. A bound term that is *absent from the
//!     dictionary* (so the pattern can match nothing) short-circuits to a count of 0 and an
//!     empty page without touching the index.
//!   * The page itself scans only the requested window: the sorted index range for the pattern,
//!     sliced to `[page_offset, page_offset + page_size)`. Resolving each row's three ids back
//!     to terms is O(page_size), independent of the pattern's total cardinality.
//!
//! ## Hydra controls
//!
//! The fragment is a small RDF document with three logical parts (all in the SAME graph, the
//! TPF convention):
//!
//!   * the **data** — the page of matching triples;
//!   * the **fragment metadata** — a node for *this page URL* carrying `void:triples`
//!     /`hydra:totalItems` (the estimated count), `hydra:itemsPerPage` (the page size), and the
//!     `hydra:next` / `hydra:previous` paging controls (present only when there is a next /
//!     previous page);
//!   * the **Hydra search control** — a `hydra:search` / `hydra:template` / `hydra:mapping`
//!     description of the `{subject}` / `{predicate}` / `{object}` URI template, so a generic
//!     TPF client can discover how to request any other pattern from this same dataset.
//!
//! ## OPT-IN — feature `tpf` AND a config flag, both OFF by default
//!
//! Compiled only behind the `tpf` cargo feature; even then the route refuses (`404`) unless
//! [`ServerConfig::tpf`](crate::ServerConfig::tpf) is set (CLI `--tpf` / env `SPARQ_TPF=1`).
//! This mirrors the `federation-descriptors` double-opt-in exactly. A build without the feature
//! pays ZERO cost — no route, no new dependency, byte-identical to before — and
//! `sparq-core`/`sparq-engine` are untouched (this reads a pinned `&Graph` snapshot through the
//! existing public [`Graph::pattern`] / `store.estimate` / `store.scan` / `dict.term` API).
//!
//! It is a **READ-only** source interface — there is no write path.

use std::str::FromStr;

use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};

use sparq_core::Graph;

/// The default page size — the maximum number of data triples one fragment carries. A bounded
/// page is the whole point of TPF (the per-request cost ceiling); 100 is the conventional LDF
/// server default (the reference Node LDF server / Comunica use it), small enough to keep a
/// page cheap and large enough that a join's per-page overhead amortises.
pub const DEFAULT_PAGE_SIZE: usize = 100;

/// `hydra:` — the Hydra Core vocabulary (the TPF control vocabulary).
const HYDRA: &str = "http://www.w3.org/ns/hydra/core#";
/// `void:` — VoID (the fragment's `void:triples` count + the dataset/subset linkage).
const VOID: &str = "http://rdfs.org/ns/void#";
/// `rdf:type`.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
/// `xsd:integer` — the datatype of the count / page-size literals.
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// A parsed triple pattern: each position is `Some(term)` when the request bound it, or `None`
/// (a variable). The predicate is a [`NamedNode`] (a predicate is always an IRI), matching the
/// shape [`Graph::pattern`] wants.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TriplePattern {
    pub subject: Option<Term>,
    pub predicate: Option<NamedNode>,
    pub object: Option<Term>,
}

/// An error parsing one of the `subject` / `predicate` / `object` parameters into a term. The
/// HTTP layer maps it to a `400`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TriplePattern {
    /// Parses a triple pattern from the three raw query-parameter values (each already
    /// form-decoded). An absent (`None`) or whitespace-only value is a *variable* (unbound);
    /// a present value is parsed as an **N-Triples term** — `<iri>`, `"literal"`,
    /// `"literal"@lang`, `"literal"^^<datatype>`. The subject and object accept any term
    /// (IRI / blank node / literal — though a literal subject can never match); the predicate
    /// MUST be an IRI (a non-IRI predicate is a parse error, since a predicate is always an
    /// IRI and the [`Graph::pattern`] API takes a [`NamedNode`]).
    pub fn parse(
        subject: Option<&str>,
        predicate: Option<&str>,
        object: Option<&str>,
    ) -> Result<Self, ParseError> {
        Ok(Self {
            subject: parse_term(subject, "subject")?,
            predicate: parse_predicate(predicate)?,
            object: parse_term(object, "object")?,
        })
    }

    /// `true` when no position is bound — the `?s ?p ?o` "everything" pattern (a full dump,
    /// still paged).
    pub fn is_all(&self) -> bool {
        self.subject.is_none() && self.predicate.is_none() && self.object.is_none()
    }
}

/// Parses an optional N-Triples term parameter; empty / whitespace-only ⇒ a variable (`None`).
fn parse_term(raw: Option<&str>, pos: &str) -> Result<Option<Term>, ParseError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(s) => Term::from_str(s)
            .map(Some)
            .map_err(|e| ParseError(format!("invalid {} term '{}': {e}", pos, s))),
    }
}

/// Parses the optional predicate parameter — which must be an IRI (a [`NamedNode`]).
fn parse_predicate(raw: Option<&str>) -> Result<Option<NamedNode>, ParseError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(s) => match Term::from_str(s) {
            Ok(Term::NamedNode(n)) => Ok(Some(n)),
            Ok(_) => Err(ParseError(format!("predicate must be an IRI, got '{s}'"))),
            Err(e) => Err(ParseError(format!("invalid predicate term '{s}': {e}"))),
        },
    }
}

/// One TPF page (fragment) for a triple pattern: the matched-triple count estimate and the
/// data triples on THIS page, plus the paging position. The HTTP layer combines this with the
/// page URL to add the Hydra control triples ([`fragment_triples`]).
#[derive(Debug, Clone)]
pub struct Fragment {
    /// The estimated total number of matches for the pattern (`hydra:totalItems` /
    /// `void:triples`) — the cheap index estimate, NOT a count of the page.
    pub total_estimate: usize,
    /// The (zero-based) page number this fragment represents.
    pub page: usize,
    /// The page size (`hydra:itemsPerPage`).
    pub page_size: usize,
    /// The data triples on this page (at most `page_size`).
    pub triples: Vec<Triple>,
}

impl Fragment {
    /// `true` when a page after this one exists (there are more matches than
    /// `(page + 1) * page_size`) — drives the `hydra:next` control.
    pub fn has_next(&self) -> bool {
        // A page is non-final iff at least one match lies beyond its window. We key this on the
        // ESTIMATE (the same number we advertise as totalItems), so `next` is consistent with
        // the count we report. The final-page-exactly boundary therefore reports no `next`.
        (self.page + 1) * self.page_size < self.total_estimate
    }

    /// `true` when there is a page before this one (`page > 0`) — drives `hydra:previous`.
    pub fn has_previous(&self) -> bool {
        self.page > 0
    }
}

/// Evaluates `pattern` against the pinned `graph` snapshot, returning the [`Fragment`] for
/// (zero-based) `page` with `page_size` data triples.
///
/// The cardinality estimate is the cheap index-range count ([`store.estimate`]); the page
/// scans only its `[page*page_size, …)` window and resolves that window's ids back to terms.
/// A bound term absent from the dictionary makes the pattern unmatchable: estimate 0, empty
/// page, no index work.
pub fn evaluate(graph: &Graph, pattern: &TriplePattern, page: usize, page_size: usize) -> Fragment {
    let page_size = page_size.max(1);
    // Resolve the bound terms to dictionary ids. `None` here means a BOUND term that is absent
    // from the dictionary — the pattern can match nothing, so short-circuit to an empty page.
    let id_pattern = match graph.pattern(
        pattern.subject.as_ref(),
        pattern.predicate.as_ref(),
        pattern.object.as_ref(),
    ) {
        Some(p) => p,
        None => {
            return Fragment {
                total_estimate: 0,
                page,
                page_size,
                triples: Vec::new(),
            }
        }
    };

    // Cheap cardinality estimate for `hydra:totalItems` — the planner's index-range count.
    let total_estimate = graph.store.estimate(&id_pattern);

    // Scan the pattern's sorted index range and slice out this page's window. We materialise the
    // page's ids first (the scan borrows the index), then resolve them to terms — O(page_size),
    // independent of the total cardinality.
    let offset = page.saturating_mul(page_size);
    let scan = graph.store.scan(&id_pattern);
    let page_ids: Vec<[sparq_core::dict::Id; 3]> = scan
        .rows
        .iter()
        .skip(offset)
        .take(page_size)
        .map(|row| scan.to_spo(row))
        .collect();

    let triples = page_ids
        .into_iter()
        .filter_map(|[s, p, o]| spo_to_triple(graph, s, p, o))
        .collect();

    Fragment {
        total_estimate,
        page,
        page_size,
        triples,
    }
}

/// Resolves a canonical `[s, p, o]` id triple to an [`oxrdf::Triple`] via the public
/// [`Dict::term`](sparq_core::dict::Dict::term). A row whose subject is a literal or whose
/// predicate is not an IRI cannot form a valid RDF triple (it never arises from a well-formed
/// store, but we are total rather than panic): such a row is dropped (`None`).
fn spo_to_triple(
    graph: &Graph,
    s: sparq_core::dict::Id,
    p: sparq_core::dict::Id,
    o: sparq_core::dict::Id,
) -> Option<Triple> {
    let subject = match graph.dict.term(s) {
        Term::NamedNode(n) => oxrdf::NamedOrBlankNode::NamedNode(n),
        Term::BlankNode(b) => oxrdf::NamedOrBlankNode::BlankNode(b),
        // A literal or a quoted-triple subject cannot be a NamedOrBlankNode — drop the row
        // (never arises from a well-formed store; we are total rather than panic).
        Term::Literal(_) | Term::Triple(_) => return None,
    };
    let predicate = match graph.dict.term(p) {
        Term::NamedNode(n) => n,
        _ => return None,
    };
    let object = graph.dict.term(o);
    Some(Triple::new(subject, predicate, object))
}

/// Builds the COMPLETE fragment document as a triple list: the page's data triples PLUS the
/// Hydra metadata + control triples, all in one graph (the TPF convention). Ready for the
/// crate's graph serialisers.
///
/// * `fragment_url` is the absolute URL of THIS page (used as the metadata node and the subject
///   of the paging controls); it already carries this page's `subject`/`predicate`/`object` and
///   `page` parameters.
/// * `dataset_url` names the `void:Dataset` (conventionally the TPF endpoint URL, e.g.
///   `http://host/tpf`); it is the `hydra:search` host and the `void:subset` parent.
/// * `template` is the URI template advertised by `hydra:template` (e.g.
///   `http://host/tpf{?subject,predicate,object}`).
/// * `next_url` / `prev_url` are the `hydra:next` / `hydra:previous` page URLs, present only
///   when [`Fragment::has_next`] / [`Fragment::has_previous`].
///
/// Defensive on IRI validity: the URLs are derived from the request `Host` header (attacker-
/// controlled), so any that is not a valid IRI is dropped rather than emitted as malformed RDF.
pub fn fragment_triples(
    frag: &Fragment,
    fragment_url: &str,
    dataset_url: &str,
    template: &str,
    next_url: Option<&str>,
    prev_url: Option<&str>,
) -> Vec<Triple> {
    let hydra = |local: &str| NamedNode::new(format!("{HYDRA}{local}")).ok();
    let void = |local: &str| NamedNode::new(format!("{VOID}{local}")).ok();
    let iri = |s: &str| NamedNode::new(s).ok();
    let count =
        |n: usize| Literal::new_typed_literal(n.to_string(), NamedNode::new_unchecked(XSD_INTEGER));

    let mut out = Vec::with_capacity(frag.triples.len() + 12);

    // ---- The data: the page of matching triples.
    out.extend(frag.triples.iter().cloned());

    // ---- Fragment + dataset nodes (need valid IRIs to attach metadata to).
    let (Some(frag_node), Some(dataset_node)) = (iri(fragment_url), iri(dataset_url)) else {
        // No valid fragment/dataset IRI (a hostile Host header) — return just the data; the
        // document is still well-formed RDF, just without controls.
        return out;
    };

    // A `subject predicate object` triple, emitted only when the predicate IRI resolved. Term
    // construction is explicit (`Term::from` / `Term::NamedNode` / `Term::BlankNode`) because
    // several crates in scope (oxttl / spargebra) also impl `From<…>`, making bare `.into()`
    // ambiguous.
    let mut emit = |s: NamedOrBlankNode, p: Option<NamedNode>, o: Term| {
        if let Some(p) = p {
            out.push(Triple::new(s, p, o));
        }
    };
    let frag_subj = || NamedOrBlankNode::NamedNode(frag_node.clone());
    let dataset_subj = || NamedOrBlankNode::NamedNode(dataset_node.clone());

    // Type the fragment as hydra:Collection + hydra:PartialCollectionView (the TPF view typing)
    // and link the dataset → this page as a void:subset.
    if let Some(t) = iri(RDF_TYPE) {
        if let Some(coll) = hydra("Collection") {
            emit(frag_subj(), Some(t.clone()), Term::NamedNode(coll));
        }
        if let Some(view) = hydra("PartialCollectionView") {
            emit(frag_subj(), Some(t), Term::NamedNode(view));
        }
    }
    emit(
        dataset_subj(),
        void("subset"),
        Term::NamedNode(frag_node.clone()),
    );

    // ---- Counts: hydra:totalItems + void:triples (the estimate) and hydra:itemsPerPage.
    emit(
        frag_subj(),
        hydra("totalItems"),
        Term::Literal(count(frag.total_estimate)),
    );
    emit(
        frag_subj(),
        void("triples"),
        Term::Literal(count(frag.total_estimate)),
    );
    emit(
        frag_subj(),
        hydra("itemsPerPage"),
        Term::Literal(count(frag.page_size)),
    );

    // ---- Paging controls: hydra:next / hydra:previous.
    if let (Some(next), Some(p)) = (next_url, hydra("next")) {
        if let Some(n) = iri(next) {
            emit(frag_subj(), Some(p), Term::NamedNode(n));
        }
    }
    if let (Some(prev), Some(p)) = (prev_url, hydra("previous")) {
        if let Some(pv) = iri(prev) {
            emit(frag_subj(), Some(p), Term::NamedNode(pv));
        }
    }

    // ---- The Hydra search control: how to request ANY pattern from this dataset.
    // dataset hydra:search _:search . _:search hydra:template "…{?subject,predicate,object}" ;
    //   hydra:mapping _:s, _:p, _:o . _:s hydra:variable "subject" ; hydra:property rdf:subject .
    let search = oxrdf::BlankNode::default();
    emit(
        dataset_subj(),
        hydra("search"),
        Term::BlankNode(search.clone()),
    );
    emit(
        NamedOrBlankNode::BlankNode(search.clone()),
        hydra("template"),
        Term::Literal(Literal::new_simple_literal(template)),
    );
    // The three variable mappings (subject/predicate/object), each binding the template variable
    // to the corresponding rdf: property so a client knows which position it controls.
    for (var, prop) in [
        (
            "subject",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#subject",
        ),
        (
            "predicate",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate",
        ),
        (
            "object",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#object",
        ),
    ] {
        let mapping = oxrdf::BlankNode::default();
        emit(
            NamedOrBlankNode::BlankNode(search.clone()),
            hydra("mapping"),
            Term::BlankNode(mapping.clone()),
        );
        emit(
            NamedOrBlankNode::BlankNode(mapping.clone()),
            hydra("variable"),
            Term::Literal(Literal::new_simple_literal(var)),
        );
        emit(
            NamedOrBlankNode::BlankNode(mapping),
            hydra("property"),
            // `prop` is a constant rdf: IRI; if it ever failed to parse, the mapping simply
            // omits the property link (still valid RDF) rather than panicking.
            match iri(prop) {
                Some(n) => Term::NamedNode(n),
                None => continue,
            },
        );
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r#"
        @prefix ex: <http://ex/> .
        ex:alice a ex:Person ; ex:knows ex:bob ; ex:name "Alice" .
        ex:bob   a ex:Person ; ex:name "Bob" .
        ex:carol a ex:Person ; ex:knows ex:alice .
    "#;

    fn graph() -> Graph {
        Graph::load_str(DATA, "turtle").unwrap()
    }

    // ---- Pattern parsing -------------------------------------------------------------

    #[test]
    fn parse_unbound_is_all() {
        assert!(TriplePattern::parse(None, None, None).unwrap().is_all());
        // Empty / whitespace-only are also variables.
        assert!(TriplePattern::parse(Some(""), Some("  "), None)
            .unwrap()
            .is_all());
    }

    #[test]
    fn parse_iri_and_literal_terms() {
        let p = TriplePattern::parse(
            Some("<http://ex/alice>"),
            Some("<http://ex/name>"),
            Some("\"Alice\""),
        )
        .unwrap();
        assert_eq!(
            p.subject,
            Some(Term::NamedNode(NamedNode::new("http://ex/alice").unwrap()))
        );
        assert_eq!(p.predicate, Some(NamedNode::new("http://ex/name").unwrap()));
        assert!(matches!(p.object, Some(Term::Literal(_))));
        assert!(!p.is_all());
    }

    #[test]
    fn parse_rejects_non_iri_predicate() {
        let err = TriplePattern::parse(None, Some("\"not an iri\""), None);
        assert!(err.is_err(), "a literal predicate must be rejected");
    }

    #[test]
    fn parse_rejects_malformed_term() {
        assert!(TriplePattern::parse(Some("not a term"), None, None).is_err());
    }

    // ---- Pattern match → correct triples ---------------------------------------------

    #[test]
    fn predicate_bound_matches_all_with_that_predicate() {
        // ?s ex:knows ?o  → alice knows bob, carol knows alice (2 triples).
        let p = TriplePattern::parse(None, Some("<http://ex/knows>"), None).unwrap();
        let f = evaluate(&graph(), &p, 0, DEFAULT_PAGE_SIZE);
        assert_eq!(f.total_estimate, 2);
        assert_eq!(f.triples.len(), 2);
        assert!(f
            .triples
            .iter()
            .all(|t| t.predicate.as_str() == "http://ex/knows"));
    }

    #[test]
    fn subject_bound_matches_that_subject() {
        // ex:alice ?p ?o → a Person, knows bob, name "Alice" (3 triples).
        let p = TriplePattern::parse(Some("<http://ex/alice>"), None, None).unwrap();
        let f = evaluate(&graph(), &p, 0, DEFAULT_PAGE_SIZE);
        assert_eq!(f.total_estimate, 3);
        assert_eq!(f.triples.len(), 3);
        assert!(f.triples.iter().all(|t| t.subject
            == oxrdf::NamedOrBlankNode::NamedNode(NamedNode::new("http://ex/alice").unwrap())));
    }

    #[test]
    fn fully_bound_exact_match() {
        let p = TriplePattern::parse(
            Some("<http://ex/alice>"),
            Some("<http://ex/knows>"),
            Some("<http://ex/bob>"),
        )
        .unwrap();
        let f = evaluate(&graph(), &p, 0, DEFAULT_PAGE_SIZE);
        assert_eq!(f.total_estimate, 1);
        assert_eq!(f.triples.len(), 1);
    }

    #[test]
    fn absent_term_yields_empty_page_no_match() {
        // A bound term that is NOT in the dictionary: empty page, estimate 0.
        let p = TriplePattern::parse(Some("<http://ex/nobody>"), None, None).unwrap();
        let f = evaluate(&graph(), &p, 0, DEFAULT_PAGE_SIZE);
        assert_eq!(f.total_estimate, 0);
        assert!(f.triples.is_empty());
        assert!(!f.has_next());
        assert!(!f.has_previous());
    }

    #[test]
    fn all_pattern_returns_every_triple() {
        // ?s ?p ?o over the whole graph: 3 type + 2 knows + 2 name = 7 triples.
        let p = TriplePattern::parse(None, None, None).unwrap();
        let f = evaluate(&graph(), &p, 0, DEFAULT_PAGE_SIZE);
        assert_eq!(f.total_estimate, 7);
        assert_eq!(f.triples.len(), 7);
    }

    // ---- Paging boundary -------------------------------------------------------------

    #[test]
    fn paging_splits_and_next_boundary() {
        // Page size 3 over 7 triples → pages 0,1 have next; page 2 (the 7th item) does not.
        let p = TriplePattern::parse(None, None, None).unwrap();
        let g = graph();

        let p0 = evaluate(&g, &p, 0, 3);
        assert_eq!(p0.triples.len(), 3);
        assert!(p0.has_next());
        assert!(!p0.has_previous());

        let p1 = evaluate(&g, &p, 1, 3);
        assert_eq!(p1.triples.len(), 3);
        assert!(p1.has_next());
        assert!(p1.has_previous());

        // Page 2: the final item (7 = 2*3 + 1) — has previous, NO next.
        let p2 = evaluate(&g, &p, 2, 3);
        assert_eq!(p2.triples.len(), 1);
        assert!(!p2.has_next());
        assert!(p2.has_previous());

        // A page beyond the data is empty but still consistent.
        let p3 = evaluate(&g, &p, 3, 3);
        assert!(p3.triples.is_empty());
        assert!(!p3.has_next());
        assert!(p3.has_previous());
    }

    #[test]
    fn pages_partition_the_results_without_overlap() {
        // Concatenating page 0 + page 1 + page 2 (size 3) reproduces the full result set.
        let p = TriplePattern::parse(None, None, None).unwrap();
        let g = graph();
        let mut all = Vec::new();
        for page in 0..3 {
            all.extend(evaluate(&g, &p, page, 3).triples);
        }
        let full = evaluate(&g, &p, 0, DEFAULT_PAGE_SIZE).triples;
        // Same multiset of triples (the scan order is deterministic, so they even align).
        assert_eq!(all.len(), full.len());
        let mut a: Vec<String> = all.iter().map(|t| t.to_string()).collect();
        let mut b: Vec<String> = full.iter().map(|t| t.to_string()).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
    }

    // ---- Hydra controls --------------------------------------------------------------

    #[test]
    fn fragment_carries_hydra_controls() {
        let p = TriplePattern::parse(None, None, None).unwrap();
        let f = evaluate(&graph(), &p, 0, 3);
        let next = "http://host/tpf?page=2";
        let triples = fragment_triples(
            &f,
            "http://host/tpf?page=1",
            "http://host/tpf",
            "http://host/tpf{?subject,predicate,object}",
            f.has_next().then_some(next),
            None, // page 0 → no previous
        );
        let doc = crate::graph::triples_to_ntriples(&triples);
        // totalItems + itemsPerPage estimate.
        assert!(doc.contains("hydra/core#totalItems"), "{doc}");
        assert!(doc.contains("hydra/core#itemsPerPage"), "{doc}");
        assert!(doc.contains("void#triples"), "{doc}");
        // The search template control.
        assert!(doc.contains("hydra/core#search"), "{doc}");
        assert!(doc.contains("hydra/core#template"), "{doc}");
        assert!(doc.contains("subject,predicate,object"), "{doc}");
        // next present (page 0 has a next), previous absent.
        assert!(doc.contains("hydra/core#next"), "{doc}");
        assert!(!doc.contains("hydra/core#previous"), "{doc}");
        // The data triples are in the document too (7 total > 3 page → 3 data triples present).
        assert_eq!(f.triples.len(), 3);
    }

    #[test]
    fn fragment_previous_present_on_later_page() {
        let p = TriplePattern::parse(None, None, None).unwrap();
        let f = evaluate(&graph(), &p, 1, 3);
        let triples = fragment_triples(
            &f,
            "http://host/tpf?page=1",
            "http://host/tpf",
            "http://host/tpf{?subject,predicate,object}",
            f.has_next().then_some("http://host/tpf?page=2"),
            f.has_previous().then_some("http://host/tpf?page=0"),
        );
        let doc = crate::graph::triples_to_ntriples(&triples);
        assert!(doc.contains("hydra/core#previous"), "{doc}");
        assert!(doc.contains("hydra/core#next"), "{doc}");
    }

    #[test]
    fn empty_pattern_page_still_well_formed_rdf() {
        // An empty page (no matches) must still produce a valid RDF document with the metadata
        // (totalItems 0) and the search control — a TPF client relies on that to learn the count.
        let p = TriplePattern::parse(Some("<http://ex/nobody>"), None, None).unwrap();
        let f = evaluate(&graph(), &p, 0, DEFAULT_PAGE_SIZE);
        let triples = fragment_triples(
            &f,
            "http://host/tpf?subject=%3Chttp%3A%2F%2Fex%2Fnobody%3E",
            "http://host/tpf",
            "http://host/tpf{?subject,predicate,object}",
            None,
            None,
        );
        let ttl = crate::graph::triples_to_turtle(&triples);
        // Re-parses as valid Turtle.
        let g = sparq_core::Graph::load_str(&ttl, "turtle").unwrap();
        assert!(g.len() >= 3, "metadata + search control present: {ttl}");
        let nt = crate::graph::triples_to_ntriples(&triples);
        // The count is a typed integer literal: `…#totalItems> "0"^^<xsd:integer>`.
        assert!(nt.contains("totalItems> \"0\""), "{nt}");
    }
}
