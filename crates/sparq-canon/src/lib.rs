//! # sparq-canon — RDFC-1.0 (RDF Dataset Canonicalization) as a public API
//!
//! [RDFC-1.0] (the W3C *RDF Dataset Canonicalization*, the URDNA2015
//! successor) gives an RDF dataset a **deterministic, blank-node-relabelled
//! canonical form**: two datasets are RDF-isomorphic iff their canonical
//! N-Quads serializations are byte-for-byte identical. That is the basis for
//! dataset hashing, signing, diffing, deduplication, and content-addressing.
//!
//! [RDFC-1.0]: https://www.w3.org/TR/rdf-canon/
//!
//! ## Opt-in by construction
//!
//! This is a **standalone, opt-in** crate. Nothing in sparq's default build or
//! the wasm artifact depends on it — `sparq-core` stays lean. Pull it in
//! explicitly (`sparq-canon = { path = "..." }`) only when you need
//! canonicalization. `sparq-zk` depends on it for the ZK commitment pipeline.
//!
//! ## The algorithm is single-sourced
//!
//! The RDFC-1.0 algorithm itself (issuer / first-degree + n-degree hashing /
//! the HNDQ recursion) is the maintained zkp-ld [`rdf_canon`] crate, validated
//! against the [W3C rdf-canon test suite] (see `tests/rdf_canon_suite.rs`).
//! `rdf_canon` speaks oxrdf 0.2 while sparq speaks oxrdf 0.3, so this crate
//! owns the single **canonical N-Quads text bridge** that every sparq consumer
//! shares: serialize 0.3 terms with their canonical `Display` form, parse with
//! oxttl 0.1 into 0.2 quads, canonicalize, parse the canonical N-Quads back.
//! N-Quads is the interchange form RDFC-1.0 is defined over, so the seam is
//! lossless by construction.
//!
//! [W3C rdf-canon test suite]: https://github.com/w3c/rdf-canon
//!
//! ## API shape
//!
//! - Dataset level (the general case): [`canonicalize`] / [`canonicalize_quads`]
//!   take quads and return the canonical N-Quads `String`; [`issued_identifiers`]
//!   / [`issue_quads`] return the canonical blank-node issuer map (input label
//!   → `c14nN`).
//! - Single graph (a default-graph-only dataset): [`canonicalize_triples`] /
//!   [`canonicalize_graph_content`] return a [`CanonicalGraph`] (the sorted
//!   canonical N-Quads lines + the re-parsed canonical triples), which is what
//!   the ZK per-graph commitment pipeline consumes.
//!
//! ## Pathological inputs
//!
//! RDFC-1.0 has worst-case blow-ups. `rdf_canon`'s HNDQ-call-limit guard is
//! kept at its default; limit hits surface as [`CanonError::Canonicalization`]
//! so a caller can fail closed on poison graphs.
//!
//! [OPUS-4.8] sq-0qip — surfaced from `sparq-zk::canon` (Fable unavailable; flag
//! for re-review when Fable returns).
//!
//! ## Opt-in NON-STANDARD RDF-1.2 triple-term profile (`rdf12-triple-terms`)
//!
//! Triple terms (`Term::Triple`, the RDF-1.2 `<<( s p o )>>` object) are
//! **outside** the W3C RDFC-1.0 data model, so the standard paths above fail
//! closed with [`CanonError::TripleTerm`]. Enabling the **opt-in, off-by-default**
//! `rdf12-triple-terms` cargo feature adds a *separate, clearly non-standard* v2
//! profile (`canonicalize_rdf12`, `canonicalize_triples_rdf12`, …) that
//! natively re-implements the RDFC-1.0 algorithm over oxrdf-0.3 and **descends
//! the Hash-N-Degree-Quads gossip into triple-term objects**, so blank nodes
//! nested inside triple terms get relabelled. It is byte-identical to the
//! standard path on triple-term-free input. This is **not** W3C RDFC-1.0
//! (RDF-1.2 canonicalization is unsettled upstream); see the `rdf12` module
//! (only present with the feature on). With the feature OFF the crate is
//! byte-identical to before — the standard surface still returns
//! [`CanonError::TripleTerm`] on triple terms.

#![forbid(unsafe_code)]

use oxrdf::{GraphName, Quad, Triple};
use oxttl::NQuadsParser;
use sparq_core::Graph;
use std::collections::HashMap;

/// **NON-STANDARD, opt-in (`rdf12-triple-terms` feature).** Native RDF-1.2
/// triple-term canonicalization profile — see the [module docs](rdf12) and the
/// crate-level banner. Not W3C RDFC-1.0.
#[cfg(feature = "rdf12-triple-terms")]
pub mod rdf12;

#[cfg(feature = "rdf12-triple-terms")]
pub use rdf12::{
    canonicalize_graph_content_rdf12, canonicalize_rdf12, canonicalize_triples_rdf12,
    issue_dataset_rdf12,
};

/// The hash-function trait RDFC-1.0 is parameterized over (`digest::Digest`),
/// re-exported so callers of [`canonicalize_quads_with`] / [`issue_quads_with`]
/// can name a hasher (e.g. `sha2::Sha256`, `sha2::Sha384`) without taking a
/// direct `digest` dependency. The spec default is SHA-256
/// ([`canonicalize_quads`]).
pub use digest::Digest;

/// Canonicalization failure (RDFC-1.0 over sparq's term model).
#[derive(Debug)]
pub enum CanonError {
    /// RDF 1.2 triple terms are outside W3C RDFC-1.0's data model; the
    /// standard paths fail closed on them. Enable the opt-in `rdf12-triple-terms`
    /// feature and use the non-standard `canonicalize_rdf12` /
    /// `canonicalize_triples_rdf12` profile to canonicalize triple terms.
    TripleTerm,
    /// Bridge serialization/parse failure (should not happen for RDFC-1.0-model
    /// content; surfaced rather than swallowed).
    Bridge(String),
    /// `rdf_canon` rejected the dataset (including the HNDQ poison-graph limit).
    Canonicalization(String),
}

impl std::fmt::Display for CanonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanonError::TripleTerm => {
                write!(
                    f,
                    "RDF 1.2 triple terms are outside the W3C RDFC-1.0 data model; \
                     enable the `rdf12-triple-terms` profile to canonicalize them"
                )
            }
            CanonError::Bridge(e) => write!(f, "oxrdf bridge error: {e}"),
            CanonError::Canonicalization(e) => write!(f, "RDFC-1.0 canonicalization failed: {e}"),
        }
    }
}

impl std::error::Error for CanonError {}

/// A single graph's RDFC-1.0 canonical form: the sorted canonical N-Quads
/// lines (a stable total order over the triples) and the same triples
/// re-parsed with canonical blank-node labels (`c14nN`).
///
/// In the ZK commitment pipeline the index of a line is its **leaf index**.
#[derive(Debug, Clone)]
pub struct CanonicalGraph {
    /// Canonical N-Quads lines in canonical (code point) order, one triple
    /// each, no trailing newline. `lines[i]` is the canonical serialization of
    /// `triples[i]`.
    pub lines: Vec<String>,
    /// The canonical triples (blank-node labels are `c14nN`), same order as
    /// [`Self::lines`].
    pub triples: Vec<Triple>,
}

impl CanonicalGraph {
    /// The canonical N-Quads document (joined lines, each terminated by `\n`).
    pub fn to_nquads(&self) -> String {
        let mut s = String::new();
        for l in &self.lines {
            s.push_str(l);
            s.push('\n');
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Dataset-level API (the general RDFC-1.0 case: a set of quads).
// ---------------------------------------------------------------------------

/// Canonicalizes an RDF dataset (a slice of [`Quad`]s) and returns its
/// RDFC-1.0 **canonical N-Quads** document (canonically sorted, one quad per
/// line, blank nodes relabelled to `c14nN`, trailing newline on each line).
///
/// Two datasets that are RDF-isomorphic produce byte-identical output; blank
/// node identifiers and quad order in the input do not affect it.
///
/// ```
/// use oxrdf::{Quad, NamedNode, NamedOrBlankNode, Term, GraphName, BlankNode};
/// let p = NamedNode::new("http://ex/p").unwrap();
/// let q = Quad::new(
///     NamedOrBlankNode::BlankNode(BlankNode::new("x").unwrap()),
///     p.clone(),
///     Term::Literal(oxrdf::Literal::new_simple_literal("v")),
///     GraphName::DefaultGraph,
/// );
/// let canon = sparq_canon::canonicalize(&[q]).unwrap();
/// assert!(canon.contains("_:c14n0"));
/// ```
pub fn canonicalize(dataset: &[Quad]) -> Result<String, CanonError> {
    canonicalize_quads(dataset)
}

/// Alias of [`canonicalize`] for callers that prefer the explicit `_quads`
/// name (mirrors [`rdf_canon::canonicalize_quads`]).
pub fn canonicalize_quads(dataset: &[Quad]) -> Result<String, CanonError> {
    let quads02 = bridge_to_02(dataset)?;
    rdf_canon::canonicalize_quads(&quads02).map_err(|e| CanonError::Canonicalization(e.to_string()))
}

/// Like [`canonicalize_quads`] but parameterized over the RDFC-1.0 hash
/// function `D` (the spec default is SHA-256; e.g. `sha2::Sha384` selects the
/// SHA-384 profile). Uses the default HNDQ call limit.
pub fn canonicalize_quads_with<D: Digest>(dataset: &[Quad]) -> Result<String, CanonError> {
    let quads02 = bridge_to_02(dataset)?;
    let opts = rdf_canon::CanonicalizationOptions::default();
    rdf_canon::canonicalize_quads_with::<D>(&quads02, &opts)
        .map_err(|e| CanonError::Canonicalization(e.to_string()))
}

/// Like [`issue_quads`] but parameterized over the RDFC-1.0 hash function `D`.
pub fn issue_quads_with<D: Digest>(
    dataset: &[Quad],
) -> Result<HashMap<String, String>, CanonError> {
    let quads02 = bridge_to_02(dataset)?;
    let opts = rdf_canon::CanonicalizationOptions::default();
    let map = rdf_canon::issue_quads_with::<D>(&quads02, &opts)
        .map_err(|e| CanonError::Canonicalization(e.to_string()))?;
    Ok(map.into_iter().collect())
}

/// Returns the RDFC-1.0 **issued-identifier map** for a dataset: input
/// blank-node label → canonical `c14nN` label. Cheap relative to a full
/// canonicalization-and-reparse when only the relabelling is needed.
pub fn issued_identifiers(dataset: &[Quad]) -> Result<HashMap<String, String>, CanonError> {
    issue_quads(dataset)
}

/// Alias of [`issued_identifiers`] (mirrors [`rdf_canon::issue_quads`]).
pub fn issue_quads(dataset: &[Quad]) -> Result<HashMap<String, String>, CanonError> {
    let quads02 = bridge_to_02(dataset)?;
    let map = rdf_canon::issue_quads(&quads02)
        .map_err(|e| CanonError::Canonicalization(e.to_string()))?;
    Ok(map.into_iter().collect())
}

// ---------------------------------------------------------------------------
// Single-graph API (a default-graph-only dataset). What the ZK per-graph
// commitment pipeline consumes; kept here so the bridge is single-sourced.
// ---------------------------------------------------------------------------

/// Canonicalizes a slice of triples (one graph's content, treated as the
/// default graph of a single-graph dataset) into a [`CanonicalGraph`].
pub fn canonicalize_triples(triples: &[Triple]) -> Result<CanonicalGraph, CanonError> {
    for t in triples {
        if contains_triple_term(t) {
            return Err(CanonError::TripleTerm);
        }
    }
    let quads02 = bridge_triples_to_02(triples)?;
    let canonical = rdf_canon::canonicalize_quads(&quads02)
        .map_err(|e| CanonError::Canonicalization(e.to_string()))?;
    parse_canonical(&canonical)
}

/// Canonicalizes the content of a [`sparq_core::Graph`] into a
/// [`CanonicalGraph`].
pub fn canonicalize_graph_content(g: &Graph) -> Result<CanonicalGraph, CanonError> {
    let triples = graph_triples(g)?;
    canonicalize_triples(&triples)
}

/// The RDFC-1.0 issued-identifier map for a single graph's content (input
/// blank-node label → canonical `c14nN` label).
pub fn issue_triples(triples: &[Triple]) -> Result<HashMap<String, String>, CanonError> {
    for t in triples {
        if contains_triple_term(t) {
            return Err(CanonError::TripleTerm);
        }
    }
    let quads02 = bridge_triples_to_02(triples)?;
    let map = rdf_canon::issue_quads(&quads02)
        .map_err(|e| CanonError::Canonicalization(e.to_string()))?;
    Ok(map.into_iter().collect())
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

// ---------------------------------------------------------------------------
// Bridge internals (the single oxrdf 0.3 -> text -> oxrdf 0.2 seam).
// ---------------------------------------------------------------------------

/// Serialize oxrdf-0.3 quads to canonical N-Quads text and parse into oxrdf-0.2
/// quads for `rdf_canon`. `GraphName::Display` renders the default graph as the
/// literal word `DEFAULT`, so the default graph is emitted explicitly as a
/// 3-term line.
fn bridge_to_02(dataset: &[Quad]) -> Result<Vec<oxrdf02::Quad>, CanonError> {
    let mut doc = String::new();
    for q in dataset {
        if matches!(q.object, oxrdf::Term::Triple(_)) {
            return Err(CanonError::TripleTerm);
        }
        match &q.graph_name {
            GraphName::DefaultGraph => {
                doc.push_str(&format!("{} {} {} .\n", q.subject, q.predicate, q.object));
            }
            g => {
                doc.push_str(&format!(
                    "{} {} {} {} .\n",
                    q.subject, q.predicate, q.object, g
                ));
            }
        }
    }
    parse_02(&doc)
}

fn bridge_triples_to_02(triples: &[Triple]) -> Result<Vec<oxrdf02::Quad>, CanonError> {
    let mut doc = String::new();
    for t in triples {
        doc.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object));
    }
    parse_02(&doc)
}

fn parse_02(doc: &str) -> Result<Vec<oxrdf02::Quad>, CanonError> {
    let mut quads02 = Vec::new();
    for item in oxttl01::NQuadsParser::new().for_reader(doc.as_bytes()) {
        quads02.push(item.map_err(|e| CanonError::Bridge(e.to_string()))?);
    }
    Ok(quads02)
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
        other => {
            return Err(CanonError::Bridge(format!(
                "invalid predicate term: {other}"
            )))
        }
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
    use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term};

    fn iri(s: &str) -> NamedNode {
        NamedNode::new(s).unwrap()
    }

    #[test]
    fn canonical_labels_and_order() {
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
            Term::Literal(Literal::new_simple_literal("x")),
        );
        let c = canonicalize_triples(&[t1, t2]).unwrap();
        assert_eq!(c.lines.len(), 2);
        let mut sorted = c.lines.clone();
        sorted.sort();
        assert_eq!(c.lines, sorted, "canonical lines must be sorted");
        assert!(c.lines.iter().all(|l| l.contains("_:c14n")));
    }

    /// Blank-node relabelling invariance: the same graph under different input
    /// bnode labels and a permuted triple order canonicalizes identically.
    #[test]
    fn relabelling_and_order_invariance() {
        let make = |l1: &str, l2: &str, swap: bool| -> CanonicalGraph {
            let b1 = BlankNode::new(l1).unwrap();
            let b2 = BlankNode::new(l2).unwrap();
            let t1 = Triple::new(
                NamedOrBlankNode::BlankNode(b1),
                iri("http://ex/p"),
                Term::BlankNode(b2.clone()),
            );
            let t2 = Triple::new(
                NamedOrBlankNode::BlankNode(b2),
                iri("http://ex/q"),
                Term::Literal(Literal::new_simple_literal("x")),
            );
            let triples = if swap { vec![t2, t1] } else { vec![t1, t2] };
            canonicalize_triples(&triples).unwrap()
        };
        let a = make("zzz", "aaa", false);
        let b = make("other1", "other2", true);
        assert_eq!(
            a.lines, b.lines,
            "isomorphic graphs must canonicalize identically"
        );
    }

    /// Dataset-level API: a named-graph quad round-trips through the bridge and
    /// the canonical form carries the graph name.
    #[test]
    fn dataset_named_graph() {
        let g = GraphName::NamedNode(iri("http://ex/g"));
        let q = Quad::new(
            NamedOrBlankNode::BlankNode(BlankNode::new("x").unwrap()),
            iri("http://ex/p"),
            Term::Literal(Literal::new_simple_literal("v")),
            g,
        );
        let canon = canonicalize(std::slice::from_ref(&q)).unwrap();
        assert!(
            canon.contains("<http://ex/g>"),
            "graph name preserved: {canon}"
        );
        assert!(canon.contains("_:c14n0"));
        // Issuer map relabels the one bnode.
        let map = issued_identifiers(&[q]).unwrap();
        assert_eq!(map.get("x").map(String::as_str), Some("c14n0"));
    }

    #[test]
    fn rejects_triple_terms() {
        let inner = Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/s")),
            iri("http://ex/p"),
            Term::Literal(Literal::new_simple_literal("v")),
        );
        let t = Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/s")),
            iri("http://ex/p"),
            Term::Triple(Box::new(inner)),
        );
        assert!(matches!(
            canonicalize_triples(&[t]),
            Err(CanonError::TripleTerm)
        ));
    }

    #[test]
    fn to_nquads_round_trips_lines() {
        let t = Triple::new(
            NamedOrBlankNode::NamedNode(iri("http://ex/s")),
            iri("http://ex/p"),
            Term::Literal(Literal::new_simple_literal("v")),
        );
        let c = canonicalize_triples(&[t]).unwrap();
        assert_eq!(c.to_nquads(), format!("{}\n", c.lines[0]));
    }
}
