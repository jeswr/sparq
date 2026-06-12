//! RDFC10 canonicalization of a named graph's content (plan §2.2).
//!
//! The commitment target is the **RDFC10-canonicalized content graph**: a
//! credential document's triples, canonicalized per graph at ingest, giving
//! canonical bnode labels (`c14n0`, `c14n1`, …) and a total order over the
//! triples (canonical code-point-sorted N-Quads). The position of a triple in
//! that order is its **leaf index** in the per-graph commitment.
//!
//! Implementation: the maintained zkp-ld `rdf-canon` crate (W3C
//! rdf-canon-test-suite validated — see `tests/rdf_canon_suite.rs`). It
//! speaks oxrdf 0.2 while sparq speaks oxrdf 0.3, so we bridge through
//! canonical N-Triples text: serialize 0.3 terms with their canonical
//! `Display` form, parse with oxttl 0.1 into 0.2 quads, canonicalize, and
//! parse the canonical N-Quads back into 0.3 triples. N-Quads is the
//! interchange form the RDFC10 spec itself is defined over, so the text seam
//! is lossless by construction (and exercised by the whole W3C suite).
//!
//! RDFC10 has pathological-input blow-ups (plan §8): `rdf-canon`'s
//! HNDQ-call-limit guard is kept at its default, and limit hits surface as
//! [`CanonError::Canonicalization`] — ingest fails closed on poison graphs.

use oxrdf::Triple;
use oxttl::NQuadsParser;
use sparq_core::Graph;

/// Canonicalization failure (all variants fail closed at ingest).
#[derive(Debug)]
pub enum CanonError {
    /// RDF 1.2 triple terms are outside RDFC10's data model; a credential
    /// graph containing them cannot be committed.
    TripleTerm,
    /// Bridge serialization/parse failure (should not happen for RDFC10-model
    /// content; surfaced rather than swallowed).
    Bridge(String),
    /// rdf-canon rejected the graph (including the HNDQ poison-graph limit).
    Canonicalization(String),
}

impl std::fmt::Display for CanonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanonError::TripleTerm => write!(f, "RDF 1.2 triple terms cannot be canonicalized (RDFC10 data model)"),
            CanonError::Bridge(e) => write!(f, "oxrdf bridge error: {e}"),
            CanonError::Canonicalization(e) => write!(f, "RDFC10 canonicalization failed: {e}"),
        }
    }
}

impl std::error::Error for CanonError {}

/// A graph's RDFC10 canonical form: the sorted canonical N-Quads lines (the
/// leaf order of the per-graph commitment) and the same triples re-parsed
/// with canonical bnode labels.
#[derive(Debug, Clone)]
pub struct CanonicalGraph {
    /// Canonical N-Quads lines in canonical (code point) order, one triple
    /// each, no trailing newline. Index = leaf index.
    pub lines: Vec<String>,
    /// The canonical triples (bnode labels are `c14nN`), same order as
    /// [`Self::lines`].
    pub triples: Vec<Triple>,
}

/// Canonicalizes a slice of triples (one named graph's content, treated as
/// the default graph of a single-graph dataset, exactly as a credential
/// document's content graph is canonicalized per graph at ingest).
pub fn canonicalize_triples(triples: &[Triple]) -> Result<CanonicalGraph, CanonError> {
    // Serialize to N-Triples text (oxrdf Display = canonical N-Triples terms).
    let mut doc = String::new();
    for t in triples {
        if contains_triple_term(t) {
            return Err(CanonError::TripleTerm);
        }
        doc.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object));
    }

    // Parse into oxrdf 0.2 quads for rdf-canon.
    let mut quads02 = Vec::with_capacity(triples.len());
    for item in oxttl01::NQuadsParser::new().for_reader(doc.as_bytes()) {
        let q = item.map_err(|e| CanonError::Bridge(e.to_string()))?;
        quads02.push(q);
    }

    // RDFC10 (default SHA-256, default HNDQ call limit).
    let canonical = rdf_canon::canonicalize_quads(&quads02)
        .map_err(|e| CanonError::Canonicalization(e.to_string()))?;

    parse_canonical(&canonical)
}

/// Canonicalizes the content of a `sparq_core::Graph` (the representation of
/// one named graph in the store).
pub fn canonicalize_graph_content(g: &Graph) -> Result<CanonicalGraph, CanonError> {
    let triples: Vec<Triple> = graph_triples(g)?;
    canonicalize_triples(&triples)
}

/// Materializes a store graph's triples as oxrdf [`Triple`]s.
pub fn graph_triples(g: &Graph) -> Result<Vec<Triple>, CanonError> {
    let mut out = Vec::with_capacity(g.len());
    for t in g.iter_ids() {
        let s = g.dict.term(t[0]);
        let p = g.dict.term(t[1]);
        let o = g.dict.term(t[2]);
        out.push(terms_to_triple(s, p, o)?);
    }
    Ok(out)
}

fn terms_to_triple(s: oxrdf::Term, p: oxrdf::Term, o: oxrdf::Term) -> Result<Triple, CanonError> {
    use oxrdf::Term;
    let subject = match s {
        Term::NamedNode(n) => oxrdf::NamedOrBlankNode::NamedNode(n),
        Term::BlankNode(b) => oxrdf::NamedOrBlankNode::BlankNode(b),
        other => return Err(CanonError::Bridge(format!("invalid subject term: {other}"))),
    };
    let predicate = match p {
        Term::NamedNode(n) => n,
        other => return Err(CanonError::Bridge(format!("invalid predicate term: {other}"))),
    };
    Ok(Triple::new(subject, predicate, o))
}

fn parse_canonical(canonical: &str) -> Result<CanonicalGraph, CanonError> {
    let mut lines: Vec<String> = Vec::new();
    let mut triples: Vec<Triple> = Vec::new();
    for line in canonical.lines() {
        if line.trim().is_empty() {
            continue;
        }
        lines.push(line.to_string());
        let mut parsed = None;
        for item in NQuadsParser::new().for_slice(line.as_bytes()) {
            let q = item.map_err(|e| CanonError::Bridge(e.to_string()))?;
            parsed = Some(Triple::new(q.subject, q.predicate, q.object));
        }
        triples.push(parsed.ok_or_else(|| {
            CanonError::Bridge(format!("canonical line did not parse as one quad: {line}"))
        })?);
    }
    Ok(CanonicalGraph { lines, triples })
}

fn contains_triple_term(t: &Triple) -> bool {
    matches!(t.object, oxrdf::Term::Triple(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{BlankNode, NamedNode, NamedOrBlankNode, Term};

    fn iri(s: &str) -> NamedNode {
        NamedNode::new(s).unwrap()
    }

    #[test]
    fn canonical_labels_and_order() {
        // Two bnodes; input labels must be replaced by c14n labels and the
        // output must be code-point sorted.
        let b1 = BlankNode::new("zzz").unwrap();
        let b2 = BlankNode::new("aaa").unwrap();
        let t1 = Triple::new(
            NamedOrBlankNode::BlankNode(b1.clone()),
            iri("http://ex/p"),
            Term::BlankNode(b2.clone()),
        );
        let t2 = Triple::new(
            NamedOrBlankNode::BlankNode(b2),
            iri("http://ex/q"),
            Term::Literal(oxrdf::Literal::new_simple_literal("x")),
        );
        let c = canonicalize_triples(&[t1, t2]).unwrap();
        assert_eq!(c.lines.len(), 2);
        let mut sorted = c.lines.clone();
        sorted.sort();
        assert_eq!(c.lines, sorted, "canonical lines must be sorted");
        assert!(c.lines.iter().all(|l| l.contains("_:c14n")));
        // Determinism under input permutation.
        let c2 = canonicalize_graph_like_permuted();
        assert_eq!(c.lines, c2.lines);
    }

    fn canonicalize_graph_like_permuted() -> CanonicalGraph {
        let b1 = BlankNode::new("other1").unwrap();
        let b2 = BlankNode::new("other2").unwrap();
        let t2 = Triple::new(
            NamedOrBlankNode::BlankNode(b2.clone()),
            iri("http://ex/q"),
            Term::Literal(oxrdf::Literal::new_simple_literal("x")),
        );
        let t1 = Triple::new(
            NamedOrBlankNode::BlankNode(b1),
            iri("http://ex/p"),
            Term::BlankNode(b2),
        );
        canonicalize_triples(&[t2, t1]).unwrap()
    }

    #[test]
    fn rejects_triple_terms() {
        let inner = Triple::new(NamedOrBlankNode::NamedNode(iri("http://ex/s")), iri("http://ex/p"), Term::Literal(oxrdf::Literal::new_simple_literal("v")));
        let t = Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/s")),
            iri("http://ex/p"),
            Term::Triple(Box::new(inner)),
        );
        assert!(matches!(canonicalize_triples(&[t]), Err(CanonError::TripleTerm)));
    }
}
