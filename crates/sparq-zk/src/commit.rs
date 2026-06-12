//! Per-named-graph commitments (plan §2.2, flat-hash primary shape).
//!
//! Pipeline: RDFC10-canonicalize the graph's content ([`crate::canon`]) →
//! encode each canonical triple to a leaf ([`crate::encode`]) in canonical
//! N-Quads order (index = leaf index) → `C(G)` = one Poseidon2 sponge over
//! the leaf sequence — the same single `Poseidon2::hash(leaves, n)` call a
//! Noir circuit makes to recompute the commitment in-circuit (the
//! length-bearing IV gives domain separation per leaf count; runtime
//! `message_size` < the bucket's `n_max` handles padding slots).
//!
//! The per-graph Merkle fallback for large graphs (plan §2.2 shape 2) is a
//! later deliverable; this module is the credential-scale primary path.

use crate::canon::{self, CanonError, CanonicalGraph};
use crate::encode;
use crate::field::Fr;
use crate::poseidon2;
use oxrdf::Triple;
use std::collections::HashMap;

/// Commitment failure.
#[derive(Debug)]
pub enum CommitError {
    Canon(CanonError),
    /// A canonical triple contained a term outside the committed data model.
    UncommittableTerm(String),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitError::Canon(e) => write!(f, "{e}"),
            CommitError::UncommittableTerm(t) => write!(f, "uncommittable term: {t}"),
        }
    }
}

impl std::error::Error for CommitError {}

impl From<CanonError> for CommitError {
    fn from(e: CanonError) -> Self {
        CommitError::Canon(e)
    }
}

/// A committed graph: canonical form, ordered leaves, and the commitment.
#[derive(Debug, Clone)]
pub struct GraphCommitment {
    /// The RDFC10 canonical form (leaf order = `canonical.lines` order).
    pub canonical: CanonicalGraph,
    /// Per-triple leaf hashes, in canonical order.
    pub leaves: Vec<Fr>,
    /// `C(G)`: Poseidon2 sponge over `leaves`.
    pub commitment: Fr,
    /// The per-graph salt the leaves were encoded under (`zk:rdfc10Salt`).
    pub salt: Fr,
}

impl GraphCommitment {
    /// Leaf index of a canonical-form triple (bnode labels must be the
    /// RDFC10 canonical labels). `None` if the triple is not in the graph.
    pub fn leaf_index(&self, canonical_triple: &Triple) -> Option<usize> {
        self.canonical.triples.iter().position(|t| t == canonical_triple)
    }

    /// An index from canonical triple to leaf position, for bulk resolution
    /// (the zk-trace seam resolves many lookups per query).
    pub fn leaf_index_map(&self) -> HashMap<&Triple, usize> {
        self.canonical
            .triples
            .iter()
            .enumerate()
            .map(|(i, t)| (t, i))
            .collect()
    }
}

/// Canonicalizes and commits one named graph's content under `salt`.
pub fn commit_triples(triples: &[Triple], salt: Fr) -> Result<GraphCommitment, CommitError> {
    let canonical = canon::canonicalize_triples(triples)?;
    commit_canonical(canonical, salt)
}

/// Commits the content of a `sparq_core::Graph` under `salt`.
pub fn commit_graph_content(g: &sparq_core::Graph, salt: Fr) -> Result<GraphCommitment, CommitError> {
    let canonical = canon::canonicalize_graph_content(g)?;
    commit_canonical(canonical, salt)
}

fn commit_canonical(canonical: CanonicalGraph, salt: Fr) -> Result<GraphCommitment, CommitError> {
    let mut leaves = Vec::with_capacity(canonical.triples.len());
    for t in &canonical.triples {
        let leaf = encode::encode_triple(t, &salt)
            .ok_or_else(|| CommitError::UncommittableTerm(t.to_string()))?;
        leaves.push(leaf);
    }
    let commitment = poseidon2::hash(&leaves);
    Ok(GraphCommitment { canonical, leaves, commitment, salt })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::salt_from_bytes;
    use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term};

    fn bnode_graph(label: &str, value: &str) -> Vec<Triple> {
        let b = BlankNode::new(label).unwrap();
        vec![
            Triple::new(
                NamedOrBlankNode::BlankNode(b.clone()),
                NamedNode::new("http://ex/p").unwrap(),
                Term::Literal(Literal::new_simple_literal(value)),
            ),
            Triple::new(
                NamedOrBlankNode::BlankNode(b),
                NamedNode::new("http://ex/q").unwrap(),
                Term::NamedNode(NamedNode::new("http://ex/o").unwrap()),
            ),
        ]
    }

    #[test]
    fn deterministic_and_input_order_independent() {
        let salt = salt_from_bytes(&[7u8; 32]);
        let g = bnode_graph("x", "v");
        let mut rev = g.clone();
        rev.reverse();
        let c1 = commit_triples(&g, salt).unwrap();
        let c2 = commit_triples(&rev, salt).unwrap();
        assert_eq!(c1.commitment, c2.commitment);
        assert_eq!(c1.leaves, c2.leaves);
    }

    #[test]
    fn salt_separation_same_content_different_graphs() {
        // The Q6 property: identical canonical content (same canonical bnode
        // labels) committed in two different graphs (different salts) must
        // produce different LEAVES, not just different commitments.
        let g = bnode_graph("x", "v");
        let c1 = commit_triples(&g, salt_from_bytes(&[1u8; 32])).unwrap();
        let c2 = commit_triples(&g, salt_from_bytes(&[2u8; 32])).unwrap();
        assert_eq!(c1.canonical.lines, c2.canonical.lines, "canonical form is salt-free");
        for (l1, l2) in c1.leaves.iter().zip(&c2.leaves) {
            assert_ne!(l1, l2, "every bnode-bearing leaf must be salt-separated");
        }
        assert_ne!(c1.commitment, c2.commitment);
    }

    #[test]
    fn leaf_index_resolves_canonical_triples() {
        let salt = salt_from_bytes(&[7u8; 32]);
        let c = commit_triples(&bnode_graph("x", "v"), salt).unwrap();
        for (i, t) in c.canonical.triples.iter().enumerate() {
            assert_eq!(c.leaf_index(t), Some(i));
        }
        let absent = Triple::new(
            NamedOrBlankNode::NamedNode(NamedNode::new("http://ex/zzz").unwrap()),
            NamedNode::new("http://ex/p").unwrap(),
            Term::Literal(Literal::new_simple_literal("v")),
        );
        assert_eq!(c.leaf_index(&absent), None);
    }
}
